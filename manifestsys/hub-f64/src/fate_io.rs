//! Unified Fate I/O — generic ingress / egress for ValhallaBase.
//!
//! **Address model**
//! - Internal navigation: [`FateRelAddr`](crate::fate_adaptive_algebra::FateRelAddr) (4-quad + τ)
//! - Absolute IDs only at these adapters (`D*`, `corpus:I`, `fate:T@I`)
//!
//! **Unified solution stack** = RelAddr land + Fate adapt + multihop v2 compose
//! (`open_fate` + `multihop_v2` = former fatewithv2 peak arm).

use crate::fate_adaptive_algebra::{
    fate_land_tick, fate_retrieve_context_lines, fate_time_cite_bracket,
    resolve_evidence_line_indices, strip_all_citations, FateAlgebraState,
};
use crate::valhalla_base::{
    ValhallaBaseConfig, ValhallaBaseQuestion, ValhallaFateIngressRouting,
};

/// Bundle returned by the generic Fate ingress adapter.
#[derive(Debug, Clone, Default)]
pub struct FateIngressBundle {
    /// Lines ranked by RelAddr land (no absolute navigator).
    pub lines: Vec<String>,
    /// Evidence tags resolved to corpus indices (ingress-only; scoring / eval).
    pub evidence_indices: Vec<usize>,
}

/// Generic ingress: RelAddr retrieve (+ optional evidence-tag resolve).
///
/// Usable by longchain unified QA, Multihop, LaRA, and any ValhallaBase caller.
pub fn fate_ingress(
    q: &ValhallaBaseQuestion,
    corpus: &[String],
    state: &FateAlgebraState,
    affinities: Option<&[f64]>,
    evidence: Option<&[String]>,
    full_session: Option<&[String]>,
) -> FateIngressBundle {
    let lines = fate_retrieve_context_lines(q, corpus, state, affinities);
    let evidence_indices = evidence
        .filter(|e| !e.is_empty())
        .map(|e| resolve_evidence_line_indices(corpus, e, full_session))
        .unwrap_or_default();
    FateIngressBundle {
        lines,
        evidence_indices,
    }
}

/// Generic egress: materialize land as `[corpus:I] [fate:T@I] body`.
///
/// Short answers (Yes / entity) keep the body intact for contains-scoring;
/// cites are aliases only.
pub fn fate_egress(
    body: &str,
    memory_text: Option<&str>,
    corpus_idx: Option<usize>,
    session_tick: u64,
    baseline_tick: u64,
) -> String {
    let mut text = strip_all_citations(body).trim().to_string();
    if text.is_empty() {
        if let Some(mem) = memory_text {
            text = strip_all_citations(mem).trim().to_string();
        }
    }
    if let Some(idx) = corpus_idx {
        let tick = fate_land_tick(baseline_tick, idx, session_tick);
        let fate_cite = fate_time_cite_bracket(idx, tick);
        format!("[corpus:{idx}] {fate_cite} {text}")
    } else {
        text
    }
}

/// Strip egress cites for compare / short-answer scoring (generic exit helper).
pub fn fate_egress_body(answer: &str) -> String {
    strip_all_citations(answer).trim().to_string()
}

/// Compare model answer to gold after egress cite strip (shared by bench + client).
pub fn fate_score_short_answer(answer: &str, gold: &str) -> bool {
    let ref_s = gold.trim();
    if ref_s.is_empty() {
        return false;
    }
    let low = fate_egress_body(answer).to_lowercase();
    let ref_low = ref_s.to_lowercase();
    if low.contains(&ref_low) {
        return true;
    }
    let ref_tokens: Vec<String> = ref_s
        .split(|c: char| c.is_whitespace() || c == '-' || c == '/')
        .filter_map(|w| {
            let t: String = w
                .chars()
                .filter(|c| c.is_ascii_alphanumeric())
                .collect::<String>()
                .to_lowercase();
            if t.len() >= 2 {
                Some(t)
            } else {
                None
            }
        })
        .collect();
    if ref_tokens.is_empty() {
        return false;
    }
    let significant: Vec<&String> = ref_tokens.iter().filter(|t| t.len() >= 4).collect();
    if significant.len() >= 2 {
        let hit = significant.iter().filter(|t| low.contains(t.as_str())).count();
        let need = significant.len().div_ceil(2).max(2).min(significant.len());
        return hit >= need;
    }
    let hit = ref_tokens.iter().filter(|t| low.contains(t.as_str())).count();
    hit >= (ref_tokens.len() + 1) / 2
}

/// Apply the unified solution flags onto a ValhallaBase config.
///
/// Enables RelAddr-backed open Fate adapt + multihop v2 compose (fatewithv2).
pub fn apply_unified_solution(cfg: &mut ValhallaBaseConfig) {
    cfg.decode = crate::valhalla_base::ValhallaBaseDecode::Hybrid;
    cfg.follow_up_decode = crate::valhalla_base::ValhallaFollowUpDecode::NativeFollowUpAware;
    cfg.fate_ingress_routing = ValhallaFateIngressRouting::QuadCycle;
    cfg.fate_qa_feedback = true;
    cfg.fate_orchestrate = true;
    cfg.open_fate = true;
    cfg.multihop_fate = true;
    cfg.multihop_v2 = true;
    cfg.fate_warmup_enabled = true;
}

/// Production config with the unified solution stack enabled.
pub fn unified_solution_config() -> ValhallaBaseConfig {
    let mut cfg = crate::valhalla_base_protocol::production_config();
    apply_unified_solution(&mut cfg);
    cfg
}

/// Runtime env defaults so raw native / bench paths share the unified stack.
pub fn ensure_unified_runtime() {
    crate::valhalla_base_protocol::ensure_production_runtime();
    crate::elastic_resources::ElasticResourcePlan::ensure_cuda_runtime_env();
    set_env_default("VALHALLA_OPEN_FATE", "1");
    set_env_default("VALHALLA_MULTIHOP_FATE", "1");
    set_env_default("VALHALLA_MULTIHOP_V2", "1");
    set_env_default("VALHALLA_CORPUS_LINE_AFFINITY_WEIGHT", "algebra");
}

fn set_env_default(key: &str, value: &str) {
    if std::env::var(key).is_err() {
        std::env::set_var(key, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fate_adaptive_algebra::FateAlgebraState;
    use crate::valhalla_base::ValhallaBaseQuestion;

    #[test]
    fn egress_wraps_cite_and_body_strips() {
        let out = fate_egress("Yes", None, Some(3), 10, 0);
        assert!(out.contains("[corpus:3]"));
        assert!(out.contains("[fate:"));
        assert!(out.contains("Yes"));
        assert_eq!(fate_egress_body(&out), "Yes");
    }

    #[test]
    fn ingress_returns_ranked_lines() {
        let q = ValhallaBaseQuestion {
            id: "t".into(),
            text: "Where did Caroline go camping last weekend?".into(),
            score_type: "open".into(),
            reference_answer: "camping".into(),
            keywords: vec!["caroline".into(), "camping".into()],
            audio_features: None,
            image_features: None,
        };
        let corpus = vec![
            "Alice likes tea.".into(),
            "Caroline went camping last weekend near the lake.".into(),
            "Bob bought apples.".into(),
        ];
        let state = FateAlgebraState::ambient();
        let bundle = fate_ingress(&q, &corpus, &state, None, None, None);
        assert!(!bundle.lines.is_empty());
        assert!(bundle
            .lines
            .iter()
            .any(|l| l.to_ascii_lowercase().contains("camping")));
    }

    #[test]
    fn unified_solution_sets_fatewithv2_flags() {
        let mut cfg = ValhallaBaseConfig::default();
        apply_unified_solution(&mut cfg);
        assert!(cfg.open_fate && cfg.multihop_fate && cfg.multihop_v2);
        assert_eq!(cfg.fate_ingress_routing, ValhallaFateIngressRouting::QuadCycle);
    }
}
