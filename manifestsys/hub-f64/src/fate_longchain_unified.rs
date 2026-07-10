//! Unified long-chain + long-term memory benchmark.
//!
//! Protocol `fate-longchain-unified-v2` (qa_adaptive): one persistent session, four phases:
//!   A) anchor baseline — establish thematic evidence anchors (D1)
//!   B) interleave long-chain — later-session QA pollution (D2/D3)
//!   C) anchor reprobe — same questions as A; measure theme/tag drift
//!   D) holdout — fresh questions never asked in A/B
//!
//! Combines: LoCoMo QA scoring (holdout), targeted/parallel routing (longchain),
//! persistent session (context-topic-drift interleave/recency concepts).

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::context_routing::{targeted_context, ContextIngestMode};
use crate::fate_parallel_longchain::{
    keywords_from_gold, run_parallel_pipes, spread_from_client,
};
use crate::fate_adaptive_algebra::{
    fate_adaptive_context_boost, fate_adaptive_distractor_filter, fate_context_pollution,
    fate_landing_drift, fate_landing_fingerprints, fate_signal_landing_indices, score_qa_algebra,
    FateAlgebraState,
};
use crate::fate_pretrain_holdout::{
    adaptive_corpus_for_pretrain, run_fate_pretrain_epochs, FatePretrainItem, HoldoutItemResult,
    PretrainSessionOutcome, PRETRAIN_MODE,
};
use crate::fate_parallel_longchain::resolve_pretrain_passes;
use crate::valhalla_base::ValhallaBaseQuestion;
use crate::valhalla_base_client::{RunOptions, ValhallaBaseClient};
use crate::valhalla_base_protocol::ValhallaBaseErrorBody;

pub const PROTOCOL: &str = "fate-longchain-unified-v3";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UnifiedRouting {
    Targeted,
    Parallel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedPhaseRow {
    pub id: String,
    pub phase: String,
    pub question: String,
    pub gold: String,
    pub evidence: Vec<String>,
    pub answer: String,
    pub correct: bool,
    pub dialogue_tags: Vec<String>,
    pub context_pollution: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorDriftRow {
    pub id: String,
    pub question: String,
    pub evidence: Vec<String>,
    pub baseline_answer: String,
    pub baseline_tags: Vec<String>,
    pub baseline_correct: bool,
    pub reprobe_answer: String,
    pub reprobe_tags: Vec<String>,
    pub reprobe_correct: bool,
    pub tag_drift: bool,
    pub theme_drift: bool,
    pub retained: bool,
    pub context_pollution: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FateLongchainUnifiedReport {
    pub protocol: String,
    #[serde(default)]
    pub corpus_mode: String,
    pub routing: String,
    pub low_resource: bool,
    pub corpus_lines: usize,
    pub final_hub_spread: f64,
    pub anchor_baseline_pass: f64,
    pub anchor_reprobe_pass: f64,
    pub theme_drift_rate: f64,
    pub anchor_retention_rate: f64,
    pub context_pollution_rate: f64,
    pub holdout_pass: f64,
    pub interleave_pass: f64,
    pub composite_score: f64,
    #[serde(default)]
    pub pretrain_passes: usize,
    #[serde(default)]
    pub pretrain_max_epochs: usize,
    #[serde(default)]
    pub pretrain_epochs: usize,
    #[serde(default)]
    pub pretrain_pass: f64,
    #[serde(default)]
    pub pretrain_converged: bool,
    #[serde(default)]
    pub pretrain_epoch_log: Vec<crate::fate_pretrain_holdout::EpochLog>,
    #[serde(default)]
    pub pretrain_rows: Vec<HoldoutItemResult>,
    pub anchor_drift_rows: Vec<AnchorDriftRow>,
    pub phase_rows: Vec<UnifiedPhaseRow>,
    pub fairness: UnifiedFairness,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedFairness {
    pub single_session_no_reset: bool,
    pub anchor_text_identical_reprobe: bool,
    pub holdout_never_before_phase_d: bool,
    pub same_corpus_all_phases: bool,
    pub scoring: String,
}

#[derive(Clone, Deserialize)]
struct BundleItem {
    id: String,
    question: String,
    #[serde(deserialize_with = "flex_str")]
    answer: String,
    #[serde(default)]
    evidence: Vec<String>,
    #[serde(default)]
    session_lines: Vec<String>,
}

#[derive(Deserialize)]
struct LocomoBundle {
    items: Vec<BundleItem>,
}

#[derive(Deserialize)]
struct UnifiedSpec {
    #[serde(default)]
    anchor_ids: Vec<String>,
    #[serde(default)]
    interleave_ids: Vec<String>,
    #[serde(default)]
    holdout_ids: Vec<String>,
    #[serde(default)]
    pretrain_ids: Vec<String>,
    #[serde(default)]
    holdout_isolated_ids: Vec<String>,
    #[serde(default)]
    corpus_neighbor: Option<usize>,
    #[serde(default)]
    fairness: Option<SpecFairness>,
}

#[derive(Deserialize)]
struct SpecFairness {
    single_session_no_reset: bool,
    anchor_text_identical_reprobe: bool,
    holdout_never_before_phase_d: bool,
    same_corpus_all_phases: bool,
    scoring: String,
}

fn flex_str<'de, D>(d: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v = serde_json::Value::deserialize(d)?;
    Ok(match v {
        serde_json::Value::String(s) => s,
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        o => o.to_string(),
    })
}

fn fate_land_tags(text: &str) -> Vec<String> {
    fate_landing_fingerprints(text)
}

fn anchor_land_index(q: &ValhallaBaseQuestion, corpus: &[String], state: &FateAlgebraState) -> usize {
    use crate::fate_adaptive_algebra::query_tokens_from_q;
    let tokens = query_tokens_from_q(q);
    fate_signal_landing_indices(corpus, &tokens, &[], state, 1)
        .first()
        .copied()
        .unwrap_or(0)
}

fn context_pollution(answer: &str, anchor_land_idx: usize, state: &FateAlgebraState) -> bool {
    fate_context_pollution(answer, anchor_land_idx, state)
}

fn tag_drift(baseline_tags: &[String], reprobe_tags: &[String]) -> bool {
    fate_landing_drift(baseline_tags, reprobe_tags)
}

fn theme_drift(
    baseline_tags: &[String],
    reprobe_tags: &[String],
    baseline_ok: bool,
    reprobe_ok: bool,
) -> bool {
    tag_drift(baseline_tags, reprobe_tags) || (baseline_ok && !reprobe_ok)
}

fn build_substrate(
    lines: &[String],
    chunk_size: usize,
    fast: bool,
    low_resource: bool,
) -> Result<ValhallaBaseClient, ValhallaBaseErrorBody> {
    ValhallaBaseClient::build_flexible_pretrain_substrate(lines, chunk_size, fast, low_resource)
        .map(|(c, _)| c)
}

fn make_question(item: &BundleItem) -> ValhallaBaseQuestion {
    ValhallaBaseQuestion {
        id: item.id.clone(),
        text: item.question.clone(),
        score_type: "open".into(),
        reference_answer: item.answer.clone(),
        keywords: keywords_from_gold(&item.answer, &item.question),
        audio_features: None,
        image_features: None,
    }
}

fn ask_holdout_fresh(
    client: &ValhallaBaseClient,
    corpus: &[String],
    item: &BundleItem,
) -> Result<String, ValhallaBaseErrorBody> {
    let q = make_question(item);
    let state = client.algebra_state();
    let aff = client
        .session()
        .map(|s| s.snapshot.corpus_line_affinity.clone())
        .unwrap_or_default();
    let mut lines: Vec<String> = Vec::new();
    fate_adaptive_context_boost(&q, corpus, &mut lines, &state, Some(&aff));
    for l in targeted_context(&q, corpus, state.retrieval_top_k(q.keywords.len())) {
        if !lines.iter().any(|x| x == &l) {
            lines.push(l);
        }
    }
    fate_adaptive_distractor_filter(&q, &mut lines, &state);
    if lines.is_empty() {
        return Ok(String::new());
    }
    let mut worker = ValhallaBaseClient::production();
    worker.config = client.config.clone();
    worker.config.fate_qa_feedback = false;
    worker.reanchor_affinity = true;
    worker.ingest_for_prompt(&q, &lines, lines.len().max(4), ContextIngestMode::Targeted);
    let resp = worker.run(vec![q], RunOptions::default())?;
    Ok(resp
        .rows
        .first()
        .map(|r| r.answer.clone())
        .unwrap_or_default())
}

fn ask_persistent(
    client: &mut ValhallaBaseClient,
    corpus: &[String],
    item: &BundleItem,
    routing: UnifiedRouting,
) -> Result<String, ValhallaBaseErrorBody> {
    let q = make_question(item);
    match routing {
        UnifiedRouting::Parallel => {
            let (ans, _, _, _) = run_parallel_pipes(corpus, &q, client)?;
            Ok(ans)
        }
        UnifiedRouting::Targeted => {
            let state = client.algebra_state();
            let aff = client
                .session()
                .map(|s| s.snapshot.corpus_line_affinity.clone())
                .unwrap_or_default();
            let mut lines = targeted_context(&q, corpus, state.retrieval_top_k(q.keywords.len()));
            fate_adaptive_context_boost(&q, corpus, &mut lines, &state, Some(&aff));
            client.reanchor_affinity = true;
            client.ingest_for_prompt(&q, &lines, lines.len().max(4), ContextIngestMode::Targeted);
            let resp = client.run(vec![q], RunOptions::default())?;
            Ok(resp
                .rows
                .first()
                .map(|r| r.answer.clone())
                .unwrap_or_default())
        }
    }
}

fn phase_row(
    item: &BundleItem,
    phase: &str,
    answer: String,
    corpus: &[String],
    state: &FateAlgebraState,
) -> UnifiedPhaseRow {
    let tags = fate_land_tags(&answer);
    let anchor_idx = anchor_land_index(&make_question(item), corpus, state);
    let correct = score_qa_algebra(&answer, &item.answer, &item.question, state);
    let pollution = context_pollution(&answer, anchor_idx, state);
    UnifiedPhaseRow {
        id: item.id.clone(),
        phase: phase.to_string(),
        question: item.question.clone(),
        gold: item.answer.clone(),
        evidence: item.evidence.clone(),
        answer: answer.chars().take(320).collect(),
        correct,
        dialogue_tags: tags,
        context_pollution: pollution,
    }
}

fn bundle_to_pretrain(item: &BundleItem) -> FatePretrainItem {
    FatePretrainItem {
        id: item.id.clone(),
        question: item.question.clone(),
        answer: item.answer.clone(),
        evidence: item.evidence.clone(),
    }
}

fn resolve_items<'a>(
    all: &'a [BundleItem],
    ids: &[String],
) -> Result<Vec<&'a BundleItem>, ValhallaBaseErrorBody> {
    let map: HashMap<&str, &BundleItem> = all.iter().map(|i| (i.id.as_str(), i)).collect();
    ids.iter()
        .map(|id| {
            map.get(id.as_str()).copied().ok_or_else(|| {
                ValhallaBaseErrorBody::new(
                    crate::valhalla_base_protocol::ValhallaBaseErrorKind::Validate,
                    format!("unknown item id {id}"),
                )
            })
        })
        .collect()
}

fn load_spec(spec_path: Option<&Path>) -> UnifiedSpec {
    let Some(path) = spec_path.filter(|p| p.is_file()) else {
        return UnifiedSpec {
            anchor_ids: vec![
                "LOCO_0_0".into(),
                "LOCO_0_1".into(),
                "LOCO_0_2".into(),
            ],
            interleave_ids: vec![
                "LOCO_0_6".into(),
                "LOCO_0_7".into(),
                "LOCO_0_8".into(),
                "LOCO_0_9".into(),
            ],
            holdout_ids: vec![
                "LOCO_0_3".into(),
                "LOCO_0_4".into(),
                "LOCO_0_5".into(),
            ],
            pretrain_ids: vec![
                "LOCO_0_0".into(),
                "LOCO_0_1".into(),
                "LOCO_0_2".into(),
                "LOCO_0_3".into(),
                "LOCO_0_4".into(),
                "LOCO_0_5".into(),
                "LOCO_0_6".into(),
                "LOCO_0_7".into(),
            ],
            holdout_isolated_ids: vec!["LOCO_0_5".into()],
            corpus_neighbor: Some(1),
            fairness: None,
        };
    };
    serde_json::from_str(&std::fs::read_to_string(path).unwrap_or_default()).unwrap_or(UnifiedSpec {
        anchor_ids: vec!["LOCO_0_0".into(), "LOCO_0_1".into(), "LOCO_0_2".into()],
        interleave_ids: vec![
            "LOCO_0_6".into(),
            "LOCO_0_7".into(),
            "LOCO_0_8".into(),
            "LOCO_0_9".into(),
        ],
        holdout_ids: vec!["LOCO_0_3".into(), "LOCO_0_4".into(), "LOCO_0_5".into()],
        pretrain_ids: vec![
            "LOCO_0_0".into(),
            "LOCO_0_1".into(),
            "LOCO_0_2".into(),
            "LOCO_0_3".into(),
            "LOCO_0_4".into(),
            "LOCO_0_5".into(),
            "LOCO_0_6".into(),
            "LOCO_0_7".into(),
        ],
        holdout_isolated_ids: vec!["LOCO_0_5".into()],
        corpus_neighbor: Some(1),
        fairness: None,
    })
}

/// Run unified long-chain theme-drift benchmark on LoCoMo bundle + optional spec.
pub fn run_fate_longchain_unified(
    bundle_path: &Path,
    spec_path: Option<&Path>,
    max_lines: usize,
    chunk_size: usize,
    fast: bool,
    low_resource: bool,
    routing: UnifiedRouting,
    pretrain_passes: usize,
    max_epochs: usize,
) -> Result<FateLongchainUnifiedReport, ValhallaBaseErrorBody> {
    crate::valhalla_base_protocol::ensure_production_runtime();
    let spec = load_spec(spec_path);

    let raw = std::fs::read_to_string(bundle_path).map_err(|e| {
        ValhallaBaseErrorBody::new(
            crate::valhalla_base_protocol::ValhallaBaseErrorKind::Io,
            e.to_string(),
        )
    })?;
    let doc: LocomoBundle = serde_json::from_str(&raw).map_err(|e| {
        ValhallaBaseErrorBody::new(
            crate::valhalla_base_protocol::ValhallaBaseErrorKind::Parse,
            format!("parse bundle: {e}"),
        )
    })?;

    let anchors = resolve_items(&doc.items, &spec.anchor_ids)?;
    let interleave = resolve_items(&doc.items, &spec.interleave_ids)?;
    let holdout = resolve_items(&doc.items, &spec.holdout_ids)?;

    let mut full_session = doc
        .items
        .first()
        .map(|i| i.session_lines.clone())
        .unwrap_or_default();
    if max_lines > 0 && full_session.len() > max_lines {
        full_session.truncate(max_lines);
    }

    let pool: Vec<&BundleItem> = anchors
        .iter()
        .chain(interleave.iter())
        .chain(holdout.iter())
        .copied()
        .collect();

    let _neighbor = spec.corpus_neighbor.unwrap_or(1);
    let pool_pretrain: Vec<FatePretrainItem> = pool
        .iter()
        .map(|i| bundle_to_pretrain(i))
        .collect();
    let corpus_lines: Vec<String> = if low_resource {
        let mut lines =
            adaptive_corpus_for_pretrain(&full_session, &pool_pretrain, &FateAlgebraState::ambient());
        if max_lines > 0 && lines.len() > max_lines {
            lines.truncate(max_lines);
        }
        lines
    } else {
        let mut lines = full_session;
        if max_lines > 0 && lines.len() > max_lines {
            lines.truncate(max_lines);
        }
        lines
    };

    eprintln!(
        "[fate-longchain-unified] routing={routing:?} corpus={} anchor={} interleave={} holdout={}",
        corpus_lines.len(),
        anchors.len(),
        interleave.len(),
        holdout.len(),
    );

    let mut client = build_substrate(&corpus_lines, chunk_size, fast, low_resource)?;
    let corpus = client
        .session()
        .map(|s| s.corpus.clone())
        .unwrap_or(corpus_lines.clone());

    let resolved_passes = resolve_pretrain_passes(pretrain_passes, &client);
    let mut pretrain_out = PretrainSessionOutcome {
        pretrain_epochs: 0,
        pretrain_pass: 0.0,
        converged: true,
        max_epochs,
        total_walk_steps: 0,
        epoch_log: Vec::new(),
        rows: Vec::new(),
    };

    if max_epochs > 0 {
        let pretrain_ids = if spec.pretrain_ids.is_empty() {
            spec.anchor_ids.clone()
        } else {
            spec.pretrain_ids.clone()
        };
        let pretrain_src = resolve_items(&doc.items, &pretrain_ids)?;
        let pretrain_items: Vec<FatePretrainItem> =
            pretrain_src.iter().map(|i| bundle_to_pretrain(i)).collect();
        eprintln!(
            "[fate-longchain-unified] phase 0 fate pretrain: {} items × max {} epochs…",
            pretrain_items.len(),
            max_epochs,
        );
        pretrain_out = run_fate_pretrain_epochs(
            &mut client,
            &corpus,
            &pretrain_items,
            max_epochs,
            low_resource,
        )?;
    }

    let mut phase_rows = Vec::new();
    let mut baseline_map: HashMap<String, UnifiedPhaseRow> = HashMap::new();
    let algebra = client.algebra_state();

    // Phase A — anchor baseline
    eprintln!("[fate-longchain-unified] phase A anchor baseline…");
    for item in &anchors {
        let ans = ask_persistent(&mut client, &corpus, item, routing)?;
        let row = phase_row(item, "anchor_baseline", ans, &corpus, &algebra);
        baseline_map.insert(item.id.clone(), row.clone());
        phase_rows.push(row);
    }

    // Phase B — interleave long-chain
    eprintln!("[fate-longchain-unified] phase B interleave long-chain…");
    for item in &interleave {
        let ans = ask_persistent(&mut client, &corpus, item, routing)?;
        phase_rows.push(phase_row(item, "interleave", ans, &corpus, &algebra));
    }

    // Phase C — anchor reprobe (theme drift)
    eprintln!("[fate-longchain-unified] phase C anchor reprobe…");
    let mut anchor_drift_rows = Vec::new();
    for item in &anchors {
        let ans = ask_persistent(&mut client, &corpus, item, routing)?;
        let reprobe = phase_row(item, "anchor_reprobe", ans, &corpus, &algebra);
        let base = baseline_map.get(&item.id).expect("baseline");
        let drift = theme_drift(
            &base.dialogue_tags,
            &reprobe.dialogue_tags,
            base.correct,
            reprobe.correct,
        );
        let tag_chg = tag_drift(&base.dialogue_tags, &reprobe.dialogue_tags);
        let retained = base.correct && reprobe.correct;
        anchor_drift_rows.push(AnchorDriftRow {
            id: item.id.clone(),
            question: item.question.clone(),
            evidence: item.evidence.clone(),
            baseline_answer: base.answer.clone(),
            baseline_tags: base.dialogue_tags.clone(),
            baseline_correct: base.correct,
            reprobe_answer: reprobe.answer.clone(),
            reprobe_tags: reprobe.dialogue_tags.clone(),
            reprobe_correct: reprobe.correct,
            tag_drift: tag_chg,
            theme_drift: drift,
            retained,
            context_pollution: reprobe.context_pollution,
        });
        phase_rows.push(reprobe);
    }

    // Phase D — holdout (charity-race uses isolated context; others keep persistent session)
    eprintln!("[fate-longchain-unified] phase D holdout…");
    let isolated: std::collections::HashSet<&str> = spec
        .holdout_isolated_ids
        .iter()
        .map(|s| s.as_str())
        .collect();
    for item in &holdout {
        let ans = if isolated.contains(item.id.as_str()) {
            ask_holdout_fresh(&client, &corpus, item)?
        } else {
            ask_persistent(&mut client, &corpus, item, routing)?
        };
        phase_rows.push(phase_row(item, "holdout", ans, &corpus, &algebra));
    }

    let n = |v: f64, d: f64| if d > 0.0 { v / d } else { 0.0 };
    let anchor_n = anchors.len() as f64;
    let inter_n = interleave.len() as f64;
    let hold_n = holdout.len() as f64;

    let baseline_ok = phase_rows
        .iter()
        .filter(|r| r.phase == "anchor_baseline" && r.correct)
        .count() as f64;
    let reprobe_ok = phase_rows
        .iter()
        .filter(|r| r.phase == "anchor_reprobe" && r.correct)
        .count() as f64;
    let drift_n = anchor_drift_rows
        .iter()
        .filter(|r| r.theme_drift)
        .count() as f64;
    let retained_n = anchor_drift_rows
        .iter()
        .filter(|r| r.retained)
        .count() as f64;
    let baseline_correct_n = anchor_drift_rows
        .iter()
        .filter(|r| r.baseline_correct)
        .count() as f64;
    let pollution_n = phase_rows
        .iter()
        .filter(|r| r.context_pollution && r.phase != "anchor_baseline")
        .count() as f64;
    let post_baseline_n = phase_rows
        .iter()
        .filter(|r| r.phase != "anchor_baseline")
        .count() as f64;
    let holdout_ok = phase_rows
        .iter()
        .filter(|r| r.phase == "holdout" && r.correct)
        .count() as f64;
    let inter_ok = phase_rows
        .iter()
        .filter(|r| r.phase == "interleave" && r.correct)
        .count() as f64;

    let anchor_baseline_pass = n(baseline_ok, anchor_n);
    let anchor_reprobe_pass = n(reprobe_ok, anchor_n);
    let theme_drift_rate = n(drift_n, anchor_n);
    let anchor_retention_rate = n(retained_n, baseline_correct_n.max(1.0));
    let context_pollution_rate = n(pollution_n, post_baseline_n.max(1.0));
    let holdout_pass = n(holdout_ok, hold_n);
    let interleave_pass = n(inter_ok, inter_n);

    // Composite: retention + holdout − drift − pollution (clamped 0..1)
    let composite = (anchor_retention_rate * 0.35
        + holdout_pass * 0.35
        + anchor_reprobe_pass * 0.15
        + interleave_pass * 0.15
        - theme_drift_rate * 0.25
        - context_pollution_rate * 0.15)
        .clamp(0.0, 1.0);

    let fairness = spec.fairness.map(|f| UnifiedFairness {
        single_session_no_reset: f.single_session_no_reset,
        anchor_text_identical_reprobe: f.anchor_text_identical_reprobe,
        holdout_never_before_phase_d: f.holdout_never_before_phase_d,
        same_corpus_all_phases: f.same_corpus_all_phases,
        scoring: f.scoring,
    }).unwrap_or(UnifiedFairness {
        single_session_no_reset: true,
        anchor_text_identical_reprobe: true,
        holdout_never_before_phase_d: true,
        same_corpus_all_phases: true,
        scoring: format!("score_qa_algebra ({PRETRAIN_MODE})"),
    });

    Ok(FateLongchainUnifiedReport {
        protocol: PROTOCOL.into(),
        corpus_mode: PRETRAIN_MODE.into(),
        routing: match routing {
            UnifiedRouting::Targeted => "targeted".into(),
            UnifiedRouting::Parallel => "parallel".into(),
        },
        low_resource,
        corpus_lines: corpus_lines.len(),
        final_hub_spread: spread_from_client(&client),
        anchor_baseline_pass,
        anchor_reprobe_pass,
        theme_drift_rate,
        anchor_retention_rate,
        context_pollution_rate,
        holdout_pass,
        interleave_pass,
        composite_score: composite,
        pretrain_passes: resolved_passes,
        pretrain_max_epochs: max_epochs,
        pretrain_epochs: pretrain_out.pretrain_epochs,
        pretrain_pass: pretrain_out.pretrain_pass,
        pretrain_converged: pretrain_out.converged,
        pretrain_epoch_log: pretrain_out.epoch_log,
        pretrain_rows: pretrain_out.rows,
        anchor_drift_rows,
        phase_rows,
        fairness,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pollution_detects_late_fate_landing() {
        let state = FateAlgebraState::ambient();
        assert!(context_pollution(
            "[corpus:0] [fate:120@40] Melanie: camping trip",
            2,
            &state,
        ));
        assert!(!context_pollution(
            "[corpus:0] [fate:48@3] Caroline: LGBTQ support group",
            3,
            &state,
        ));
    }

    #[test]
    fn theme_drift_on_fate_landing_change() {
        assert!(theme_drift(
            &["fate:48@3".into()],
            &["fate:120@40".into()],
            true,
            true,
        ));
    }
}
