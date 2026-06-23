//! Task-type decomposition — LM/transformer front → Valhalla shards → recompose.
//!
//! v4 pipeline entry: different **DecomposeMode** per question type instead of
//! ad-hoc MCQ-only `mcq_fate` + whole-question grinder elsewhere.

use serde::{Deserialize, Serialize};

use crate::dl_front::{build_body_ports, hash_embed, BodyFrontPorts, DlFrontConfig, DL_EMBED_DIM};
use crate::mcq_fate::{self, McqParts};
use crate::signal_ingress::{BodyTarget, TriadFeedReport, TriadSession, ValhallaModelPatch};

/// High-level task kind (mirrors fair `score_type` + future auto-detect).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskKind {
    Open,
    Mcq,
    Numeric,
    /// Auto from score_type string; unknown → Open.
    Auto,
}

impl TaskKind {
    pub fn parse(score_type: &str) -> Self {
        match score_type {
            "mcq" => Self::Mcq,
            "numeric" => Self::Numeric,
            "open" => Self::Open,
            _ => Self::Open,
        }
    }
}

/// How the transformer-facing front splits a prompt before Valhalla ingress.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecomposeMode {
    /// Single shard — current default for open/numeric Tier B.
    Whole,
    /// MCQ: stem + each option as separate compare shards (tile-heavy).
    McqStemOptions,
    /// MCQ variant: pairwise option-vs-stem entailment shards (stem-heavy).
    McqPairwiseCompare,
    /// Numeric: expression units + answer slot (tile + cortex path).
    NumericSteps,
    /// Open with corpus: query + retrieval-aligned keyword shards.
    OpenRetrieval,
}

impl DecomposeMode {
    pub fn default_for(kind: TaskKind) -> Self {
        match kind {
            TaskKind::Mcq => Self::McqStemOptions,
            TaskKind::Numeric => Self::NumericSteps,
            TaskKind::Open | TaskKind::Auto => Self::Whole,
        }
    }
}

/// Which Valhalla body subsystem ingests a shard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShardBody {
    Triad,
    Tile,
    Stem,
    Hub,
}

impl ShardBody {
    pub fn to_body_target(self) -> BodyTarget {
        match self {
            Self::Triad => BodyTarget::Triad,
            Self::Tile => BodyTarget::Tile,
            Self::Stem => BodyTarget::StemCell,
            Self::Hub => BodyTarget::Hub,
        }
    }
}

/// One ingress unit after transformer/grinder front.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecomposeShard {
    pub label: String,
    pub text: String,
    pub body: ShardBody,
    pub phase: ShardPhase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShardPhase {
    Decompose,
    Aggregate,
    Recompose,
}

/// Full decomposition plan for one question.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecomposePlan {
    pub task: TaskKind,
    pub mode: DecomposeMode,
    pub shards: Vec<DecomposeShard>,
    pub whole_question: String,
}

/// Output of Valhalla pass — ready for transformer recompose / weight patch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecomposePatch {
    pub model_patch: ValhallaModelPatch,
    /// Token ids (from LM tokenizer) to apply embedding delta — anchor + shard heads.
    pub patch_token_ids: Vec<u32>,
    /// Chat/user prompt for optional second LM pass.
    pub lm_user_text: String,
    /// Per-shard closure scores (aligned with decompose shards).
    pub shard_scores: Vec<f64>,
    /// MCQ only: affinity per option letter A–D from structure (for lm_head bias later).
    #[serde(default)]
    pub mcq_option_affinity: Vec<(char, f64)>,
}

pub fn plan_decomposition(
    question: &str,
    score_type: &str,
    mode_override: Option<DecomposeMode>,
) -> DecomposePlan {
    let task = TaskKind::parse(score_type);
    let mode = mode_override.unwrap_or_else(|| DecomposeMode::default_for(task));
    let shards = match mode {
        DecomposeMode::Whole => vec![DecomposeShard {
            label: "WHOLE".into(),
            text: question.to_string(),
            body: ShardBody::Triad,
            phase: ShardPhase::Decompose,
        }],
        DecomposeMode::McqStemOptions => plan_mcq_stem_options(question),
        DecomposeMode::McqPairwiseCompare => plan_mcq_pairwise(question),
        DecomposeMode::NumericSteps => plan_numeric_steps(question),
        DecomposeMode::OpenRetrieval => plan_open_retrieval(question),
    };
    DecomposePlan {
        task,
        mode,
        shards,
        whole_question: question.to_string(),
    }
}

fn plan_mcq_stem_options(question: &str) -> Vec<DecomposeShard> {
    let parts = mcq_fate::parse_mcq_parts(question);
    let mut out = vec![DecomposeShard {
        label: "MCQ_STEM".into(),
        text: format!("MCQ_STEM {}", parts.stem),
        body: ShardBody::Tile,
        phase: ShardPhase::Decompose,
    }];
    for (letter, line) in &parts.options {
        out.push(DecomposeShard {
            label: format!("MCQ_OPTION_{letter}"),
            text: format!("MCQ_OPTION {line}"),
            body: ShardBody::Tile,
            phase: ShardPhase::Decompose,
        });
    }
    if out.len() == 1 {
        out.push(DecomposeShard {
            label: "MCQ_WHOLE".into(),
            text: format!("MCQ_WHOLE {question}"),
            body: ShardBody::Triad,
            phase: ShardPhase::Decompose,
        });
    }
    out.push(DecomposeShard {
        label: "MCQ_AGGREGATE".into(),
        text: "MCQ_AGGREGATE full question context".into(),
        body: ShardBody::Triad,
        phase: ShardPhase::Aggregate,
    });
    out
}

fn plan_mcq_pairwise(question: &str) -> Vec<DecomposeShard> {
    let parts = mcq_fate::parse_mcq_parts(question);
    let mut out = Vec::new();
    for (letter, line) in &parts.options {
        out.push(DecomposeShard {
            label: format!("MCQ_PAIR_{letter}"),
            text: format!("MCQ_PAIR stem={} option={letter}:{line}", parts.stem),
            body: ShardBody::Stem,
            phase: ShardPhase::Decompose,
        });
    }
    if out.is_empty() {
        return plan_mcq_stem_options(question);
    }
    out.push(DecomposeShard {
        label: "MCQ_AGGREGATE".into(),
        text: "MCQ_AGGREGATE pairwise entailment merge".into(),
        body: ShardBody::Triad,
        phase: ShardPhase::Aggregate,
    });
    out
}

fn plan_numeric_steps(question: &str) -> Vec<DecomposeShard> {
    let mut out = vec![DecomposeShard {
        label: "NUM_WHOLE".into(),
        text: format!("NUM_WHOLE {question}"),
        body: ShardBody::Tile,
        phase: ShardPhase::Decompose,
    }];
    for (i, line) in question.lines().filter(|l| !l.trim().is_empty()).enumerate() {
        if i == 0 {
            continue;
        }
        out.push(DecomposeShard {
            label: format!("NUM_LINE_{i}"),
            text: format!("NUM_STEP {line}"),
            body: ShardBody::Tile,
            phase: ShardPhase::Decompose,
        });
    }
    out.push(DecomposeShard {
        label: "NUM_ANSWER_SLOT".into(),
        text: "NUM_ANSWER final numeric answer slot".into(),
        body: ShardBody::Hub,
        phase: ShardPhase::Recompose,
    });
    out
}

fn plan_open_retrieval(question: &str) -> Vec<DecomposeShard> {
    vec![
        DecomposeShard {
            label: "OPEN_QUERY".into(),
            text: format!("OPEN_QUERY {question}"),
            body: ShardBody::Stem,
            phase: ShardPhase::Decompose,
        },
        DecomposeShard {
            label: "OPEN_RECOMPOSE".into(),
            text: "OPEN_RECOMPOSE generation slot".into(),
            body: ShardBody::Triad,
            phase: ShardPhase::Recompose,
        },
    ]
}

const HASH_SEED_TEXT: u64 = 0x5445_5854_5F46_4E54;

pub fn shard_to_ports(shard: &DecomposeShard, front: &DlFrontConfig) -> BodyFrontPorts {
    let emb = hash_embed(&shard.text, DL_EMBED_DIM, HASH_SEED_TEXT);
    build_body_ports(&emb, front, "hash_grinder", &shard.label)
}

/// Feed decompose shards into an existing session (after corpus incubation).
pub fn feed_decompose_plan(
    session: &mut TriadSession,
    plan: &DecomposePlan,
    front: &DlFrontConfig,
    cycles_per_shard: usize,
) -> Vec<f64> {
    let mut scores = Vec::with_capacity(plan.shards.len());
    for shard in &plan.shards {
        if shard.phase == ShardPhase::Recompose {
            scores.push(1.0);
            continue;
        }
        let ports = shard_to_ports(shard, front);
        session.emit_body_ports(&ports);
        session.run_optimize_cycles(cycles_per_shard.max(1));
        if session.hub.is_data_flowing() {
            session.hub.mark_epoch_complete();
        }
        let mp = &session.tile.completed_tiles;
        let structural = if mp.is_empty() {
            0.0
        } else {
            mp.iter().map(|t| t.energy.abs()).sum::<f64>() / mp.len() as f64
        };
        scores.push((structural / 10.0).clamp(0.0, 1.0));
    }
    scores
}

/// Build LM-facing patch from Valhalla report + decomposition plan.
pub fn recompose_to_patch(
    plan: &DecomposePlan,
    report: &TriadFeedReport,
    shard_scores: &[f64],
    anchor_token_ids: &[u32],
    fate_option_scores: Option<&[(char, f64)]>,
) -> RecomposePatch {
    let mcq_option_affinity = if plan.task == TaskKind::Mcq {
        mcq_option_affinity_from_report(plan, report, shard_scores, fate_option_scores)
    } else {
        Vec::new()
    };
    RecomposePatch {
        model_patch: report.model_patch.clone(),
        patch_token_ids: anchor_token_ids.to_vec(),
        lm_user_text: plan.whole_question.clone(),
        shard_scores: shard_scores.to_vec(),
        mcq_option_affinity,
    }
}

fn mcq_option_affinity_from_report(
    plan: &DecomposePlan,
    report: &TriadFeedReport,
    shard_scores: &[f64],
    fate_option_scores: Option<&[(char, f64)]>,
) -> Vec<(char, f64)> {
    use std::collections::HashMap;

    let parts = mcq_fate::parse_mcq_parts(&plan.whole_question);
    let fate_map: HashMap<char, f64> = fate_option_scores
        .map(|xs| xs.iter().copied().collect())
        .unwrap_or_default();
    let fate = &report.model_patch.fate_affinities;
    let tile_act = &report.model_patch.tile_activations;

    let mut raw = Vec::new();
    for (i, (letter, _)) in parts.options.iter().enumerate() {
        let shard_idx = plan
            .shards
            .iter()
            .position(|s| s.label == format!("MCQ_OPTION_{letter}"))
            .unwrap_or(i + 1);
        let shard_score = shard_scores.get(shard_idx).copied().unwrap_or(0.0);
        let tile_boost = tile_act.get(i).copied().unwrap_or(0.0);
        let fate_boost = fate.get(i % fate.len().max(1)).copied().unwrap_or(0.25);
        let opt_energy = fate_map.get(letter).copied().unwrap_or(0.0);
        let s = shard_score * 0.3 + tile_boost * 0.35 + fate_boost * 0.2 + opt_energy * 0.15;
        raw.push((*letter, s));
    }
    if raw.is_empty() {
        return raw;
    }
    let mean = raw.iter().map(|(_, s)| *s).sum::<f64>() / raw.len() as f64;
    raw.into_iter().map(|(l, s)| (l, s - mean)).collect()
}

/// Parse MCQ parts re-export for pipeline consumers.
pub fn parse_mcq_parts(question: &str) -> McqParts {
    mcq_fate::parse_mcq_parts(question)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcq_plan_has_stem_and_options() {
        let q = "What is 2+2?\nA) 3\nB) 4\nC) 5\nD) 6";
        let plan = plan_decomposition(q, "mcq", None);
        assert!(plan.shards.iter().any(|s| s.label == "MCQ_STEM"));
        assert!(plan.shards.iter().any(|s| s.label == "MCQ_OPTION_B"));
        assert!(plan.shards.iter().any(|s| s.label == "MCQ_AGGREGATE"));
    }

    #[test]
    fn numeric_plan_has_answer_slot() {
        let plan = plan_decomposition("Compute 12 * 7", "numeric", None);
        assert!(plan
            .shards
            .iter()
            .any(|s| s.label == "NUM_ANSWER_SLOT"));
    }
}
