//! v4 product pipeline: LM front → decompose → Valhalla → recompose → LLM patch.
//!
//! Replaces the rough split where MCQ only hits `mcq_fate`, open hits patch LM,
//! and numeric uses confidence routing without a unified round-trip.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::dl_front::DlFrontConfig;
use crate::fair_score::extract_mcq_letter;
use crate::mcq_fate::{self, McqFateOutcome};
use crate::signal_ingress::{
    BodyTarget, IngressBudget, TriadFeedReport, TriadIncubationSnapshot, TriadSession,
};
use crate::patch_export::PATCH_ANCHOR;
use crate::task_decompose::{
    plan_decomposition, recompose_to_patch, DecomposeMode, DecomposePlan, RecomposePatch, TaskKind,
};

pub const PIPELINE_VERSION: &str = "pipeline-v4-decompose";

/// Minimum top-2 affinity gap before pipeline letter overrides memory decode.
pub const MCQ_AFFINITY_MIN_SPREAD: f64 = 0.012;

/// Minimum top-2 Fate option-score gap for patch/deploy fate-backed overrides.
pub const MCQ_FATE_MIN_SPREAD: f64 = 0.06;

pub fn mcq_fate_min_spread() -> f64 {
    std::env::var("VALHALLA_MCQ_FATE_MIN_SPREAD")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(MCQ_FATE_MIN_SPREAD)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub incubation_body: BodyTarget,
    pub qa_body: BodyTarget,
    pub decompose_mode_override: Option<DecomposeMode>,
    pub cycles_per_shard: usize,
    pub use_mcq_fate_aggregate: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            incubation_body: BodyTarget::Triad,
            qa_body: BodyTarget::Tile,
            decompose_mode_override: None,
            cycles_per_shard: 1,
            use_mcq_fate_aggregate: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineOutcome {
    pub pipeline_version: String,
    pub plan: DecomposePlan,
    pub report: TriadFeedReport,
    pub recompose: RecomposePatch,
    pub mcq_fate: Option<McqFateOutcome>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compose_fate: Option<crate::compose_fate::ComposeFateOutcome>,
    pub decode_hint: String,
}

pub fn pipeline_v4_enabled() -> bool {
    std::env::var("VALHALLA_PIPELINE_V4")
        .map(|v| {
            let v = v.trim().to_ascii_lowercase();
            v != "0" && v != "false" && v != "off"
        })
        .unwrap_or(true)
}

fn shard_scores_from_mcq_fate(plan: &DecomposePlan, outcome: &McqFateOutcome) -> Vec<f64> {
    let opt_map: HashMap<char, f64> = outcome.option_scores.iter().copied().collect();
    plan.shards
        .iter()
        .map(|s| {
            if s.label == "MCQ_STEM" {
                outcome.decompose_score
            } else if let Some(rest) = s.label.strip_prefix("MCQ_OPTION_") {
                let letter = rest.chars().next().unwrap_or('A');
                opt_map.get(&letter).copied().unwrap_or(0.0)
            } else if s.label == "MCQ_AGGREGATE" {
                outcome.aggregate_score
            } else {
                0.0
            }
        })
        .collect()
}

/// Run full v4 path on one question after corpus incubation snapshot exists.
pub fn run_question_pipeline(
    snap: &TriadIncubationSnapshot,
    question: &str,
    score_type: &str,
    whole_ports: &crate::dl_front::BodyFrontPorts,
    cfg: &PipelineConfig,
    front: &DlFrontConfig,
    mode_label: &str,
    audio_features: Option<&[f64]>,
) -> PipelineOutcome {
    let use_mm = crate::multimodal::fusion_enabled()
        && audio_features.map(|a| !a.is_empty()).unwrap_or(false);
    let mode_override = if use_mm {
        match score_type {
            "open" => Some(DecomposeMode::MultimodalCompose),
            "mcq" => Some(DecomposeMode::McqMultimodalCompose),
            _ => cfg.decompose_mode_override,
        }
    } else {
        cfg.decompose_mode_override
    };
    let compose = matches!(
        mode_override,
        Some(DecomposeMode::MultimodalCompose | DecomposeMode::McqMultimodalCompose)
    );
    let cycles_per_shard = if compose {
        cfg.cycles_per_shard.max(2)
    } else {
        cfg.cycles_per_shard
    };
    let plan = plan_decomposition(question, score_type, mode_override, audio_features);
    let budget = IngressBudget::experiment();
    let mut session = TriadSession::from_incubation_snapshot(budget, cfg.qa_body, snap.clone());

    let compose_fate_mode = compose
        && crate::compose_fate::compose_fate_cycles_enabled()
        && matches!(
            plan.mode,
            DecomposeMode::MultimodalCompose | DecomposeMode::McqMultimodalCompose
        );

    let (report, mcq_fate, compose_fate, shard_scores) = if compose_fate_mode {
        let outcome = crate::compose_fate::run_compose_fate_pipeline(
            &mut session,
            &plan,
            whole_ports,
            mode_label,
            front,
            cycles_per_shard,
        );
        let scores = outcome.shard_scores.clone();
        (outcome.report.clone(), None, Some(outcome), scores)
    } else if plan.task == TaskKind::Mcq
        && cfg.use_mcq_fate_aggregate
        && plan.mode != DecomposeMode::McqMultimodalCompose
    {
        let outcome = mcq_fate::run_mcq_fate_pipeline(
            &mut session,
            question,
            whole_ports,
            mode_label,
            front,
        );
        let scores = shard_scores_from_mcq_fate(&plan, &outcome);
        (outcome.report.clone(), Some(outcome), None, scores)
    } else {
        session.emit_body_ports(whole_ports);
        let scores = crate::task_decompose::feed_decompose_plan(
            &mut session,
            &plan,
            front,
            cycles_per_shard,
        );
        let report = session.build_feed_report(mode_label, 1);
        (report, None, None, scores)
    };

    let anchor_ids: Vec<u32> = Vec::new();
    let fate_option_scores = mcq_fate.as_ref().map(|o| o.option_scores.as_slice());
    let mut recompose = recompose_to_patch(
        &plan,
        &report,
        &shard_scores,
        &anchor_ids,
        fate_option_scores,
    );
    if let Some(ref cf) = compose_fate {
        recompose.cross_modal_score = cf.cross_modal_score;
    }

    let decode_hint = match plan.task {
        TaskKind::Mcq => {
            if let Some(ref cf) = compose_fate {
                format!(
                    "compose_fate_mcq_x{:.2}/a{}",
                    cf.cross_modal_score, cf.adaptation_rounds
                )
            } else if plan.mode == DecomposeMode::McqMultimodalCompose {
                format!(
                    "structure_recompose_mcq_x{:.2}",
                    recompose.cross_modal_score
                )
            } else if let Some(letter) = pick_mcq_affinity_letter(&recompose.mcq_option_affinity) {
                format!("pipeline_mcq_affinity_{letter}")
            } else {
                "pipeline_mcq".into()
            }
        }
        TaskKind::Numeric => "pipeline_numeric".into(),
        TaskKind::Open | TaskKind::Auto => {
            if let Some(ref cf) = compose_fate {
                format!(
                    "compose_fate_open_x{:.2}/a{}",
                    cf.cross_modal_score, cf.adaptation_rounds
                )
            } else if matches!(
                plan.mode,
                DecomposeMode::MultimodalCompose | DecomposeMode::McqMultimodalCompose
            ) {
                format!(
                    "structure_recompose_open_x{:.2}",
                    recompose.cross_modal_score
                )
            } else {
                "pipeline_open".into()
            }
        }
    };

    PipelineOutcome {
        pipeline_version: PIPELINE_VERSION.to_string(),
        plan,
        report,
        recompose,
        mcq_fate,
        compose_fate,
        decode_hint,
    }
}

pub fn pick_mcq_affinity_letter(affinities: &[(char, f64)]) -> Option<char> {
    affinities
        .iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(c, _)| *c)
}

/// MCQ: pick letter from recompose affinities (deploy path for Tier B until LM round-trip).
pub fn decode_mcq_from_recompose(recompose: &RecomposePatch) -> Option<char> {
    pick_mcq_affinity_letter(&recompose.mcq_option_affinity)
}

pub fn pick_mcq_fate_letter(option_scores: &[(char, f64)]) -> Option<char> {
    option_scores
        .iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(c, _)| *c)
}

pub fn mcq_fate_spread(option_scores: &[(char, f64)]) -> f64 {
    mcq_score_spread(option_scores)
}

pub fn mcq_affinity_spread(affinities: &[(char, f64)]) -> f64 {
    mcq_score_spread(affinities)
}

fn mcq_score_spread(scores: &[(char, f64)]) -> f64 {
    if scores.len() < 2 {
        return 0.0;
    }
    let mut vals: Vec<f64> = scores.iter().map(|(_, s)| *s).collect();
    vals.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    vals[0] - vals[1]
}

/// Corpus keyword decode paths — deploy may prefer patch LM when they disagree with patch.
pub fn is_mcq_memory_decoder_path(path: &str) -> bool {
    matches!(path, "mcq_option" | "mcq_entailment" | "mcq_fallback" | "mcq_pairwise")
}

/// Valhalla latent merge paths (prefix only; use with letter validation for deploy).
pub fn is_mcq_latent_memory_path(path: &str) -> bool {
    path.starts_with("pipeline_v4_mcq_") || path.starts_with("pipeline_v4_latent_")
}

/// Letter suffix on `pipeline_v4_mcq_X` paths.
pub fn latent_mcq_path_letter(path: &str) -> Option<char> {
    path.strip_prefix("pipeline_v4_mcq_")
        .and_then(|rest| rest.chars().next())
        .filter(|c| matches!(c, 'A' | 'B' | 'C' | 'D' | 'E'))
}

/// Deploy/memory protect only when latent path letter matches the memory answer.
pub fn is_mcq_latent_memory_protected(path: &str, mem_letter: Option<char>) -> bool {
    if path.starts_with("pipeline_v4_latent_") {
        return true;
    }
    match (latent_mcq_path_letter(path), mem_letter) {
        (Some(path_l), Some(ans_l)) => path_l == ans_l,
        _ => false,
    }
}

pub fn mcq_patch_on_disagree_enabled() -> bool {
    std::env::var("VALHALLA_MCQ_PATCH_ON_DISAGREE")
        .map(|v| {
            let v = v.trim().to_ascii_lowercase();
            v != "0" && v != "false" && v != "off"
        })
        .unwrap_or(true)
}

/// v4 MCQ merge: memory decode vs pipeline affinity when spread is meaningful.
pub fn merge_mcq_pipeline_decode(
    memory_ans: &str,
    memory_path: &str,
    affinities: &[(char, f64)],
) -> (String, String) {
    let mem_letter = extract_mcq_letter(memory_ans);
    let aff_letter = pick_mcq_affinity_letter(affinities);
    let spread = mcq_affinity_spread(affinities);

    match (aff_letter, mem_letter) {
        (Some(aff), Some(mem)) if aff == mem && spread >= MCQ_AFFINITY_MIN_SPREAD => {
            (format!("ANSWER: {aff}"), format!("pipeline_v4_mcq_{aff}"))
        }
        (Some(aff), Some(_mem)) if spread >= MCQ_AFFINITY_MIN_SPREAD => {
            (format!("ANSWER: {aff}"), format!("pipeline_v4_affinity_{aff}"))
        }
        (Some(aff), None) if spread >= MCQ_AFFINITY_MIN_SPREAD => {
            (format!("ANSWER: {aff}"), format!("pipeline_v4_affinity_{aff}"))
        }
        _ => (memory_ans.to_string(), memory_path.to_string()),
    }
}

pub fn patch_anchor_text() -> &'static str {
    PATCH_ANCHOR
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latent_memory_path_detection() {
        assert!(is_mcq_latent_memory_path("pipeline_v4_mcq_C"));
        assert!(!is_mcq_latent_memory_path("pipeline_v4_affinity_B"));
        assert!(!is_mcq_latent_memory_path("mcq_option"));
    }

    #[test]
    fn latent_protect_requires_matching_letter() {
        assert!(is_mcq_latent_memory_protected("pipeline_v4_mcq_C", Some('C')));
        assert!(!is_mcq_latent_memory_protected("pipeline_v4_mcq_D", Some('B')));
        assert!(is_mcq_latent_memory_protected("pipeline_v4_latent_x", Some('A')));
    }

    #[test]
    fn merge_mcq_weak_agreement_keeps_memory_path() {
        let aff = vec![('A', 0.001), ('B', 0.002)];
        let (ans, path) = merge_mcq_pipeline_decode("ANSWER: A", "mcq_option", &aff);
        assert_eq!(ans, "ANSWER: A");
        assert_eq!(path, "mcq_option");
    }
}
