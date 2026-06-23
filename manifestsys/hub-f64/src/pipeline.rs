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
) -> PipelineOutcome {
    let plan = plan_decomposition(question, score_type, cfg.decompose_mode_override);
    let budget = IngressBudget::experiment();
    let mut session = TriadSession::from_incubation_snapshot(budget, cfg.qa_body, snap.clone());

    let (report, mcq_fate, shard_scores) = if plan.task == TaskKind::Mcq && cfg.use_mcq_fate_aggregate {
        let outcome = mcq_fate::run_mcq_fate_pipeline(
            &mut session,
            question,
            whole_ports,
            mode_label,
            front,
        );
        let scores = shard_scores_from_mcq_fate(&plan, &outcome);
        (outcome.report.clone(), Some(outcome), scores)
    } else {
        let scores = crate::task_decompose::feed_decompose_plan(
            &mut session,
            &plan,
            front,
            cfg.cycles_per_shard,
        );
        let report = session.build_feed_report(mode_label, 1);
        (report, None, scores)
    };

    let anchor_ids: Vec<u32> = Vec::new();
    let fate_option_scores = mcq_fate.as_ref().map(|o| o.option_scores.as_slice());
    let recompose = recompose_to_patch(
        &plan,
        &report,
        &shard_scores,
        &anchor_ids,
        fate_option_scores,
    );

    let decode_hint = match plan.task {
        TaskKind::Mcq => {
            if let Some(letter) = pick_mcq_affinity_letter(&recompose.mcq_option_affinity) {
                format!("pipeline_mcq_affinity_{letter}")
            } else {
                "pipeline_mcq".into()
            }
        }
        TaskKind::Numeric => "pipeline_numeric".into(),
        TaskKind::Open | TaskKind::Auto => "pipeline_open".into(),
    };

    PipelineOutcome {
        pipeline_version: PIPELINE_VERSION.to_string(),
        plan,
        report,
        recompose,
        mcq_fate,
        decode_hint,
    }
}

pub fn mcq_affinity_spread(affinities: &[(char, f64)]) -> f64 {
    if affinities.len() < 2 {
        return 0.0;
    }
    let mut vals: Vec<f64> = affinities.iter().map(|(_, s)| *s).collect();
    vals.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    vals[0] - vals[1]
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
        (Some(aff), Some(mem)) if aff == mem => {
            (format!("ANSWER: {aff}"), format!("pipeline_v4_mcq_{aff}"))
        }
        (Some(aff), Some(_mem)) if spread >= MCQ_AFFINITY_MIN_SPREAD => {
            (format!("ANSWER: {aff}"), format!("pipeline_v4_affinity_{aff}"))
        }
        (Some(aff), None) => {
            (format!("ANSWER: {aff}"), format!("pipeline_v4_affinity_{aff}"))
        }
        _ => (memory_ans.to_string(), memory_path.to_string()),
    }
}

pub fn patch_anchor_text() -> &'static str {
    PATCH_ANCHOR
}
