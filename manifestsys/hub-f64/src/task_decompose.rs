//! Task-type decomposition — LM/transformer front → Valhalla shards → recompose.
//!
//! v4 pipeline entry: different **DecomposeMode** per question type instead of
//! ad-hoc MCQ-only `mcq_fate` + whole-question grinder elsewhere.

use serde::{Deserialize, Serialize};

use crate::dl_front::{build_body_ports, hash_embed, BodyFrontPorts, DlFrontConfig, ModalityInput, DL_EMBED_DIM};
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
    /// Text (LM/grinder) + TTS mel bands decomposed in parallel → Hub Fate → Triad recompose.
    MultimodalCompose,
    /// MCQ stem/options (text) + TTS mel bands → Hub Fate → logprob recompose.
    McqMultimodalCompose,
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

/// Ingress modality for a decompose shard (non-traditional: not concat at embed layer).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ShardModality {
    #[default]
    Text,
    /// TTS / mel band — ingested via `feed_modality`, routed to Tile/Stem by body.
    AudioMel,
}

/// One ingress unit after transformer/grinder front.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecomposeShard {
    pub label: String,
    pub text: String,
    pub body: ShardBody,
    pub phase: ShardPhase,
    #[serde(default)]
    pub modality: ShardModality,
    /// Populated for `AudioMel` shards — mel sub-band, not full sidecar dump.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio_mel: Option<Vec<f64>>,
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
    /// Open/numeric: mean decompose shard closure (for patch strength bias).
    #[serde(default)]
    pub open_structure_score: f64,
    /// Multimodal compose: text vs audio shard closure balance in [0,1] (0.5 = balanced).
    #[serde(default)]
    pub cross_modal_score: f64,
    /// Per-shard (label, score) for audit — includes TEXT_* and AUDIO_* shards.
    #[serde(default)]
    pub modality_shard_scores: Vec<(String, f64)>,
    #[serde(default)]
    pub compose_protocol: String,
}

pub fn plan_decomposition(
    question: &str,
    score_type: &str,
    mode_override: Option<DecomposeMode>,
    audio_mel: Option<&[f64]>,
) -> DecomposePlan {
    let task = TaskKind::parse(score_type);
    let mode = mode_override.unwrap_or_else(|| DecomposeMode::default_for(task));
    let shards = match mode {
        DecomposeMode::Whole => vec![DecomposeShard {
            label: "WHOLE".into(),
            text: question.to_string(),
            body: ShardBody::Triad,
            phase: ShardPhase::Decompose,
            modality: ShardModality::Text,
            audio_mel: None,
        }],
        DecomposeMode::McqStemOptions => plan_mcq_stem_options(question),
        DecomposeMode::McqPairwiseCompare => plan_mcq_pairwise(question),
        DecomposeMode::NumericSteps => plan_numeric_steps(question),
        DecomposeMode::OpenRetrieval => plan_open_retrieval(question),
        DecomposeMode::MultimodalCompose => {
            plan_multimodal_compose(question, audio_mel.unwrap_or(&[]))
        }
        DecomposeMode::McqMultimodalCompose => {
            plan_mcq_multimodal_compose(question, audio_mel.unwrap_or(&[]))
        }
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
        modality: ShardModality::Text,
        audio_mel: None,
    }];
    for (letter, line) in &parts.options {
        out.push(DecomposeShard {
            label: format!("MCQ_OPTION_{letter}"),
            text: format!("MCQ_OPTION {line}"),
            body: ShardBody::Tile,
            phase: ShardPhase::Decompose,
            modality: ShardModality::Text,
            audio_mel: None,
        });
    }
    if out.len() == 1 {
        out.push(DecomposeShard {
            label: "MCQ_WHOLE".into(),
            text: format!("MCQ_WHOLE {question}"),
            body: ShardBody::Triad,
            phase: ShardPhase::Decompose,
            modality: ShardModality::Text,
            audio_mel: None,
        });
    }
    out.push(DecomposeShard {
        label: "MCQ_AGGREGATE".into(),
        text: "MCQ_AGGREGATE full question context".into(),
        body: ShardBody::Triad,
        phase: ShardPhase::Aggregate,
        modality: ShardModality::Text,
        audio_mel: None,
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
            modality: ShardModality::Text,
            audio_mel: None,
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
        modality: ShardModality::Text,
        audio_mel: None,
    });
    out
}

fn plan_numeric_steps(question: &str) -> Vec<DecomposeShard> {
    let mut out = vec![DecomposeShard {
        label: "NUM_WHOLE".into(),
        text: format!("NUM_WHOLE {question}"),
        body: ShardBody::Tile,
        phase: ShardPhase::Decompose,
        modality: ShardModality::Text,
        audio_mel: None,
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
            modality: ShardModality::Text,
            audio_mel: None,
        });
    }
    out.push(DecomposeShard {
        label: "NUM_ANSWER_SLOT".into(),
        text: "NUM_ANSWER final numeric answer slot".into(),
        body: ShardBody::Hub,
        phase: ShardPhase::Recompose,
        modality: ShardModality::Text,
        audio_mel: None,
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
            modality: ShardModality::Text,
            audio_mel: None,
        },
        DecomposeShard {
            label: "OPEN_RECOMPOSE".into(),
            text: "OPEN_RECOMPOSE generation slot".into(),
            body: ShardBody::Triad,
            phase: ShardPhase::Recompose,
            modality: ShardModality::Text,
            audio_mel: None,
        },
    ]
}

/// Split mel vector into bands — each band is a separate Tile shard (non-traditional decompose).
fn split_mel_bands(mel: &[f64], bands: usize) -> Vec<Vec<f64>> {
    if mel.is_empty() || bands == 0 {
        return Vec::new();
    }
    let n = mel.len();
    let chunk = (n / bands).max(1);
    (0..bands)
        .map(|b| {
            let start = b * chunk;
            let end = if b == bands - 1 { n } else { (start + chunk).min(n) };
            mel[start..end].to_vec()
        })
        .filter(|v| !v.is_empty())
        .collect()
}

/// Split LM question text into spans — each span is a separate Stem/Tile shard.
fn split_lm_text_spans(question: &str, max_spans: usize) -> Vec<String> {
    let max_spans = max_spans.max(1);
    let mut parts: Vec<String> = question
        .split(['.', '?', '!', '\n'])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    if parts.is_empty() {
        parts.push(question.trim().to_string());
    }
    if parts.len() > max_spans {
        let tail = parts[max_spans - 1..].join(". ");
        parts.truncate(max_spans - 1);
        parts.push(tail);
    }
    parts
}

/// Parallel text (Stem) + TTS mel bands (Tile) → Hub Fate aggregate → Triad LM recompose slot.
fn plan_multimodal_compose(question: &str, audio_mel: &[f64]) -> Vec<DecomposeShard> {
    let mut out = Vec::new();
    for (i, span) in split_lm_text_spans(question, 4).into_iter().enumerate() {
        let body = if i == 0 {
            ShardBody::Stem
        } else {
            ShardBody::Tile
        };
        out.push(DecomposeShard {
            label: format!("TEXT_LM_SPAN_{i}"),
            text: format!("TEXT_LM_SPAN_{i} {span}"),
            body,
            phase: ShardPhase::Decompose,
            modality: ShardModality::Text,
            audio_mel: None,
        });
    }
    for (i, band) in split_mel_bands(audio_mel, 4).into_iter().enumerate() {
        out.push(DecomposeShard {
            label: format!("AUDIO_TTS_BAND_{i}"),
            text: format!("AUDIO_TTS_BAND_{i} mel prosody shard"),
            body: ShardBody::Tile,
            phase: ShardPhase::Decompose,
            modality: ShardModality::AudioMel,
            audio_mel: Some(band),
        });
    }
    out.push(DecomposeShard {
        label: "CROSS_MODAL_FATE".into(),
        text: "CROSS_MODAL Fate converge text+tts".into(),
        body: ShardBody::Hub,
        phase: ShardPhase::Aggregate,
        modality: ShardModality::Text,
        audio_mel: None,
    });
    out.push(DecomposeShard {
        label: "LM_RECOMPOSE".into(),
        text: "LM_RECOMPOSE Valhalla structure → second LM pass".into(),
        body: ShardBody::Triad,
        phase: ShardPhase::Recompose,
        modality: ShardModality::Text,
        audio_mel: None,
    });
    out
}

/// MCQ: stem (Stem) + options (Tile) + TTS bands (Tile) → Hub cross-modal → Triad recompose.
fn plan_mcq_multimodal_compose(question: &str, audio_mel: &[f64]) -> Vec<DecomposeShard> {
    let parts = mcq_fate::parse_mcq_parts(question);
    let mut out = vec![DecomposeShard {
        label: "MCQ_STEM".into(),
        text: format!("MCQ_STEM {}", parts.stem),
        body: ShardBody::Stem,
        phase: ShardPhase::Decompose,
        modality: ShardModality::Text,
        audio_mel: None,
    }];
    for (letter, line) in &parts.options {
        out.push(DecomposeShard {
            label: format!("MCQ_OPTION_{letter}"),
            text: format!("MCQ_OPTION {line}"),
            body: ShardBody::Tile,
            phase: ShardPhase::Decompose,
            modality: ShardModality::Text,
            audio_mel: None,
        });
    }
    if parts.options.is_empty() {
        out.push(DecomposeShard {
            label: "MCQ_WHOLE".into(),
            text: format!("MCQ_WHOLE {question}"),
            body: ShardBody::Stem,
            phase: ShardPhase::Decompose,
            modality: ShardModality::Text,
            audio_mel: None,
        });
    }
    for (i, band) in split_mel_bands(audio_mel, 4).into_iter().enumerate() {
        out.push(DecomposeShard {
            label: format!("AUDIO_TTS_BAND_{i}"),
            text: format!("AUDIO_TTS_BAND_{i} mel prosody shard"),
            body: ShardBody::Tile,
            phase: ShardPhase::Decompose,
            modality: ShardModality::AudioMel,
            audio_mel: Some(band),
        });
    }
    out.push(DecomposeShard {
        label: "CROSS_MODAL_FATE".into(),
        text: "CROSS_MODAL MCQ text+tts Fate".into(),
        body: ShardBody::Hub,
        phase: ShardPhase::Aggregate,
        modality: ShardModality::Text,
        audio_mel: None,
    });
    out.push(DecomposeShard {
        label: "LM_RECOMPOSE".into(),
        text: "LM_RECOMPOSE MCQ affinity from Valhalla shards".into(),
        body: ShardBody::Triad,
        phase: ShardPhase::Recompose,
        modality: ShardModality::Text,
        audio_mel: None,
    });
    out
}

const HASH_SEED_TEXT: u64 = 0x5445_5854_5F46_4E54;

pub fn shard_to_ports(shard: &DecomposeShard, front: &DlFrontConfig) -> BodyFrontPorts {
    let emb = hash_embed(&shard.text, DL_EMBED_DIM, HASH_SEED_TEXT);
    build_body_ports(&emb, front, "hash_grinder", &shard.label)
}

/// Feed decompose shards — text via grinder ports, audio mel via `feed_modality` per shard body.
pub fn feed_decompose_plan(
    session: &mut TriadSession,
    plan: &DecomposePlan,
    front: &DlFrontConfig,
    cycles_per_shard: usize,
) -> Vec<f64> {
    let compose = matches!(
        plan.mode,
        DecomposeMode::MultimodalCompose | DecomposeMode::McqMultimodalCompose
    );
    let mut scores = Vec::with_capacity(plan.shards.len());
    for shard in &plan.shards {
        let structural = if compose && shard.phase == ShardPhase::Recompose {
            feed_compose_shard(session, shard, front, cycles_per_shard);
            session.body_closure_metric(shard.body.to_body_target())
        } else if shard.phase == ShardPhase::Recompose {
            scores.push(1.0);
            continue;
        } else {
            feed_one_shard(session, shard, front, cycles_per_shard, compose)
        };
        scores.push(structural);
    }
    scores
}

fn feed_one_shard(
    session: &mut TriadSession,
    shard: &DecomposeShard,
    front: &DlFrontConfig,
    cycles_per_shard: usize,
    compose: bool,
) -> f64 {
    if compose {
        return feed_compose_shard(session, shard, front, cycles_per_shard);
    }
    match shard.modality {
        ShardModality::AudioMel => {
            if let Some(ref mel) = shard.audio_mel {
                session.feed_modality(
                    ModalityInput::AudioFeatures {
                        values: mel.clone(),
                    },
                    front,
                );
            }
        }
        ShardModality::Text => {
            let ports = shard_to_ports(shard, front);
            session.emit_body_ports(&ports);
        }
    }
    session.run_optimize_cycles(cycles_per_shard.max(1));
    if session.hub.is_data_flowing() {
        session.hub.mark_epoch_complete();
    }
    let mp = &session.tile.completed_tiles;
    if mp.is_empty() {
        0.0
    } else {
        (mp.iter().map(|t| t.energy.abs()).sum::<f64>() / mp.len() as f64 / 10.0).clamp(0.0, 1.0)
    }
}

fn feed_compose_shard(
    session: &mut TriadSession,
    shard: &DecomposeShard,
    front: &DlFrontConfig,
    cycles_per_shard: usize,
) -> f64 {
    let target = shard.body.to_body_target();
    let before = session.body_closure_metric(target);
    match shard.modality {
        ShardModality::AudioMel => {
            if let Some(ref mel) = shard.audio_mel {
                session.feed_modality_target(
                    ModalityInput::AudioFeatures {
                        values: mel.clone(),
                    },
                    front,
                    target,
                );
            }
        }
        ShardModality::Text => {
            let ports = shard_to_ports(shard, front);
            session.emit_ports_target(&ports, target);
        }
    }
    session.run_optimize_cycles_target(cycles_per_shard, target);
    if matches!(target, BodyTarget::Hub | BodyTarget::Triad) && session.hub.is_data_flowing() {
        session.hub.mark_epoch_complete();
    }
    let after = session.body_closure_metric(target);
    compose_shard_closure_score(before, after, shard)
}

fn compose_shard_closure_score(before: f64, after: f64, shard: &DecomposeShard) -> f64 {
    let delta = (after - before).max(0.0);
    let mut base = if delta > 1e-9 {
        delta / (delta + 0.35)
    } else {
        after / (after + 0.5)
    };
    if let Some(rest) = shard.label.strip_prefix("AUDIO_TTS_BAND_") {
        if let Ok(idx) = rest.parse::<f64>() {
            base += idx * 0.035 + 0.015;
        }
    }
    base.clamp(0.02, 0.98)
}

fn cross_modal_score_from_shards(plan: &DecomposePlan, shard_scores: &[f64]) -> f64 {
    let mut text_scores = Vec::new();
    let mut audio_scores = Vec::new();
    for (shard, sc) in plan.shards.iter().zip(shard_scores.iter()) {
        if shard.phase != ShardPhase::Decompose {
            continue;
        }
        match shard.modality {
            ShardModality::Text => text_scores.push(*sc),
            ShardModality::AudioMel => audio_scores.push(*sc),
        }
    }
    if text_scores.is_empty() && audio_scores.is_empty() {
        return 0.5;
    }
    let text_mean = if text_scores.is_empty() {
        0.0
    } else {
        text_scores.iter().sum::<f64>() / text_scores.len() as f64
    };
    let audio_mean = if audio_scores.is_empty() {
        0.0
    } else {
        audio_scores.iter().sum::<f64>() / audio_scores.len() as f64
    };
    if text_mean + audio_mean <= 1e-9 {
        return 0.5;
    }
    let balance = audio_mean / (text_mean + audio_mean);
    let audio_spread = if audio_scores.len() > 1 {
        let var = audio_scores
            .iter()
            .map(|s| (s - audio_mean).powi(2))
            .sum::<f64>()
            / audio_scores.len() as f64;
        var.sqrt().min(0.35)
    } else {
        0.0
    };
    (balance * 0.72 + audio_spread * 0.28).clamp(0.05, 0.95)
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
    let open_structure_score = if matches!(plan.task, TaskKind::Open | TaskKind::Auto) {
        let decompose: Vec<f64> = plan
            .shards
            .iter()
            .zip(shard_scores.iter())
            .filter(|(s, _)| s.phase == ShardPhase::Decompose)
            .map(|(_, sc)| *sc)
            .collect();
        if decompose.is_empty() {
            0.0
        } else {
            decompose.iter().sum::<f64>() / decompose.len() as f64
        }
    } else {
        0.0
    };
    let cross_modal_score = if matches!(
        plan.mode,
        DecomposeMode::MultimodalCompose | DecomposeMode::McqMultimodalCompose
    ) {
        cross_modal_score_from_shards(plan, shard_scores)
    } else {
        0.5
    };
    let modality_shard_scores: Vec<(String, f64)> = plan
        .shards
        .iter()
        .zip(shard_scores.iter())
        .map(|(s, sc)| (s.label.clone(), *sc))
        .collect();
    let compose_protocol = match plan.mode {
        DecomposeMode::MultimodalCompose => {
            crate::multimodal_compose::COMPOSE_PROTOCOL.to_string()
        }
        DecomposeMode::McqMultimodalCompose => {
            crate::multimodal_compose::COMPOSE_MCQ_PROTOCOL.to_string()
        }
        _ => String::new(),
    };
    RecomposePatch {
        model_patch: report.model_patch.clone(),
        patch_token_ids: anchor_token_ids.to_vec(),
        lm_user_text: plan.whole_question.clone(),
        shard_scores: shard_scores.to_vec(),
        mcq_option_affinity,
        open_structure_score,
        cross_modal_score,
        modality_shard_scores,
        compose_protocol,
    }
}

fn mcq_option_affinity_from_report(
    plan: &DecomposePlan,
    _report: &TriadFeedReport,
    shard_scores: &[f64],
    fate_option_scores: Option<&[(char, f64)]>,
) -> Vec<(char, f64)> {
    use std::collections::HashMap;

    let parts = mcq_fate::parse_mcq_parts(&plan.whole_question);
    let fate_map: HashMap<char, f64> = fate_option_scores
        .map(|xs| xs.iter().copied().collect())
        .unwrap_or_default();

    let mut raw = Vec::new();
    for (letter, _) in &parts.options {
        let shard_idx = plan
            .shards
            .iter()
            .position(|s| s.label == format!("MCQ_OPTION_{letter}"))
            .unwrap_or(0);
        let shard_score = shard_scores.get(shard_idx).copied().unwrap_or(0.0);
        let opt_energy = fate_map.get(letter).copied().unwrap_or(0.0);
        let score = if fate_map.len() >= 2 {
            opt_energy * 0.68 + shard_score * 0.32
        } else {
            shard_score * 0.55 + opt_energy * 0.45
        };
        raw.push((*letter, score));
    }
    if raw.is_empty() {
        return raw;
    }
    let mean = raw.iter().map(|(_, s)| *s).sum::<f64>() / raw.len() as f64;
    let mut aff: Vec<(char, f64)> = raw.into_iter().map(|(l, s)| (l, s - mean)).collect();

    if crate::mcq_pairwise::mcq_pairwise_enabled() {
        let parts = mcq_fate::parse_mcq_parts(&plan.whole_question);
        let priors = aff.clone();
        let pw = crate::mcq_pairwise::pairwise_option_scores(&parts.stem, &parts.options, &priors);
        aff = aff
            .into_iter()
            .zip(pw.iter())
            .map(|((l, a), (_, p))| (l, a * 0.45 + p * 0.55))
            .collect();
        let m2 = aff.iter().map(|(_, s)| *s).sum::<f64>() / aff.len() as f64;
        aff = aff.into_iter().map(|(l, s)| (l, s - m2)).collect();
    }
    aff
}

/// LM user text: v2 structure-recompose uses **clean question** only (patch carries shards).
pub fn build_compose_lm_user_text(plan: &DecomposePlan, recompose: &RecomposePatch) -> String {
    if recompose.compose_protocol.is_empty() {
        return plan.whole_question.clone();
    }
    if crate::multimodal_compose::is_structure_recompose_protocol(&recompose.compose_protocol) {
        return plan.whole_question.clone();
    }
    let mut lines = vec![plan.whole_question.clone()];
    lines.push(format!(
        "[Valhalla compose cross_modal={:.3} structure={:.3}]",
        recompose.cross_modal_score, recompose.open_structure_score
    ));
    for (label, score) in &recompose.modality_shard_scores {
        if label.starts_with("TEXT_")
            || label.starts_with("AUDIO_")
            || label.starts_with("MCQ_")
        {
            lines.push(format!("[shard {label} closure={score:.3}]"));
        }
    }
    lines.join("\n")
}

/// Patch strength from Triad recompose closure (Stem/Tile/Hub shards → model_patch).
pub fn compose_patch_strength(base: f64, recompose: &RecomposePatch, score_type: &str) -> f64 {
    if recompose.compose_protocol.is_empty() {
        return base;
    }
    let structure = recompose.open_structure_score;
    let cm = recompose.cross_modal_score;
    match score_type {
        "open" => {
            base * (1.0 + structure * crate::multimodal::open_patch_bias_scale() + (cm - 0.5).abs() * 0.12)
        }
        "mcq" => base * (1.0 + structure * 0.28 + cm * 0.18),
        _ => base,
    }
}

pub fn feed_compose_shard_pub(
    session: &mut TriadSession,
    shard: &DecomposeShard,
    front: &DlFrontConfig,
    cycles_per_shard: usize,
) -> f64 {
    feed_compose_shard(session, shard, front, cycles_per_shard)
}

pub fn cross_modal_score_for_plan(plan: &DecomposePlan, shard_scores: &[f64]) -> f64 {
    cross_modal_score_from_shards(plan, shard_scores)
}

pub fn compose_decode_tag(score_type: &str, cross_modal: f64) -> String {
    format!("structure_recompose_{score_type}/x{cross_modal:.3}")
}

pub fn compose_fate_decode_tag(score_type: &str, cross_modal: f64, adapt_rounds: usize) -> String {
    format!("compose_fate_{score_type}/x{cross_modal:.3}/a{adapt_rounds}")
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
        let plan = plan_decomposition(q, "mcq", None, None);
        assert!(plan.shards.iter().any(|s| s.label == "MCQ_STEM"));
        assert!(plan.shards.iter().any(|s| s.label == "MCQ_OPTION_B"));
        assert!(plan.shards.iter().any(|s| s.label == "MCQ_AGGREGATE"));
    }

    #[test]
    fn mcq_affinity_uses_letter_keyed_fate_scores() {
        use crate::signal_ingress::{IngressStats, TriadFeedReport, ValhallaModelPatch};

        let q = "Pick one\nA) 1\nB) 2\nC) 3\nD) 4";
        let plan = plan_decomposition(q, "mcq", None, None);
        let shard_scores: Vec<f64> = plan.shards.iter().map(|_| 0.5).collect();
        let fate = vec![('A', 1.0), ('B', 2.0), ('C', 5.0), ('D', 1.5)];
        let report = TriadFeedReport {
            mode: "test".into(),
            stats: IngressStats::default(),
            hub_signals: 0,
            hub_routed: [0; 4],
            hub_prefs: vec![],
            tile_completed: 0,
            tile_ops: 0,
            tile_dim: 0.0,
            stem_cells: 0,
            stem_connections: 0,
            stem_dim: 0.0,
            model_patch: ValhallaModelPatch {
                body_source: "tile".into(),
                fate_affinities: vec![],
                hub_routed: [0; 4],
                quad_layer_scales: [1.0; 4],
                tile_signatures: vec![],
                stem_signatures: vec![],
                patch_vector: vec![],
                signals_emitted: 0,
                tile_count: 0,
                stem_cell_count: 0,
                tile_activations: vec![],
                tile_connection_count: 0,
                tile_avg_activation: 0.0,
                stem_organ_clusters: vec![],
                stem_cluster_signatures: vec![],
                stem_cluster_energies: vec![],
                ingress_encoder: String::new(),
                tile_neurons: vec![],
                tile_synapses: vec![],
            },
            optimize_cycles: 0,
            cycle_trace: vec![],
            mcq_fate_trace: vec![],
        };
        let aff = mcq_option_affinity_from_report(&plan, &report, &shard_scores, Some(&fate));
        let top = aff
            .iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(c, _)| *c);
        assert_eq!(top, Some('C'));
    }

    #[test]
    fn numeric_plan_has_answer_slot() {
        let plan = plan_decomposition("Compute 12 * 7", "numeric", None, None);
        assert!(plan
            .shards
            .iter()
            .any(|s| s.label == "NUM_ANSWER_SLOT"));
    }

    #[test]
    fn multimodal_compose_splits_tts_bands() {
        let mel: Vec<f64> = (0..80).map(|i| i as f64 * 0.01).collect();
        let plan = plan_decomposition(
            "What is DNA?",
            "open",
            Some(DecomposeMode::MultimodalCompose),
            Some(&mel),
        );
        assert_eq!(plan.mode, DecomposeMode::MultimodalCompose);
        assert!(plan.shards.iter().any(|s| s.label.starts_with("TEXT_LM_SPAN_")));
        assert!(plan
            .shards
            .iter()
            .any(|s| s.label.starts_with("AUDIO_TTS_BAND_")));
        assert!(plan.shards.iter().any(|s| s.label == "CROSS_MODAL_FATE"));
        assert!(plan.shards.iter().any(|s| s.label == "LM_RECOMPOSE"));
    }

    #[test]
    fn structure_recompose_uses_clean_lm_prompt() {
        let plan = plan_decomposition("Q?", "open", None, None);
        let recompose = RecomposePatch {
            model_patch: ValhallaModelPatch {
                body_source: "triad".into(),
                fate_affinities: vec![],
                hub_routed: [0; 4],
                quad_layer_scales: [1.0; 4],
                tile_signatures: vec![],
                stem_signatures: vec![],
                patch_vector: vec![],
                signals_emitted: 0,
                tile_count: 0,
                stem_cell_count: 0,
                tile_activations: vec![],
                tile_connection_count: 0,
                tile_avg_activation: 0.0,
                stem_organ_clusters: vec![],
                stem_cluster_signatures: vec![],
                stem_cluster_energies: vec![],
                ingress_encoder: String::new(),
                tile_neurons: vec![],
                tile_synapses: vec![],
            },
            patch_token_ids: vec![],
            lm_user_text: "Q?".into(),
            shard_scores: vec![],
            mcq_option_affinity: vec![],
            open_structure_score: 0.5,
            cross_modal_score: 0.5,
            modality_shard_scores: vec![],
            compose_protocol: crate::multimodal_compose::COMPOSE_PROTOCOL.to_string(),
        };
        assert_eq!(build_compose_lm_user_text(&plan, &recompose), "Q?");
    }

    #[test]
    fn open_compose_splits_lm_text_spans() {
        let mel: Vec<f64> = vec![0.1; 40];
        let plan = plan_decomposition(
            "First sentence. Second sentence.",
            "open",
            Some(DecomposeMode::MultimodalCompose),
            Some(&mel),
        );
        assert!(plan
            .shards
            .iter()
            .any(|s| s.label == "TEXT_LM_SPAN_0"));
        assert!(plan
            .shards
            .iter()
            .any(|s| s.label == "TEXT_LM_SPAN_1"));
    }

    #[test]
    fn mcq_compose_has_stem_options_and_audio_bands() {
        let q = "Pick\nA) one\nB) two\nC) three\nD) four";
        let mel: Vec<f64> = (0..40).map(|i| i as f64 * 0.02).collect();
        let plan = plan_decomposition(
            q,
            "mcq",
            Some(DecomposeMode::McqMultimodalCompose),
            Some(&mel),
        );
        assert_eq!(plan.mode, DecomposeMode::McqMultimodalCompose);
        assert!(plan.shards.iter().any(|s| s.label == "MCQ_STEM"));
        assert!(plan.shards.iter().any(|s| s.label == "MCQ_OPTION_B"));
        assert!(plan
            .shards
            .iter()
            .any(|s| s.label.starts_with("AUDIO_TTS_BAND_")));
        assert!(plan.shards.iter().any(|s| s.label == "CROSS_MODAL_FATE"));
    }

    #[test]
    fn compose_cross_modal_scores_differ_by_mel_energy() {
        use crate::dl_front::DlFrontConfig;
        use crate::signal_ingress::{IngressBudget, TriadSession};

        let mel_rich: Vec<f64> = (0..80)
            .map(|i| (i as f64 * 0.11).sin().abs() + 0.4)
            .collect();
        let mel_flat: Vec<f64> = vec![0.01; 80];
        let front = DlFrontConfig::default();
        let budget = IngressBudget::experiment();

        let score_for = |mel: &[f64]| {
            let plan = plan_decomposition(
                "Explain photosynthesis briefly.",
                "open",
                Some(DecomposeMode::MultimodalCompose),
                Some(mel),
            );
            let mut session = TriadSession::with_body(budget.clone(), BodyTarget::Tile);
            let shard_scores = feed_decompose_plan(&mut session, &plan, &front, 2);
            cross_modal_score_from_shards(&plan, &shard_scores)
        };

        let rich = score_for(&mel_rich);
        let flat = score_for(&mel_flat);
        assert!(
            (rich - flat).abs() > 0.02,
            "cross_modal should differ: rich={rich:.3} flat={flat:.3}"
        );
        assert!((0.05..=0.95).contains(&rich));
        assert!((0.05..=0.95).contains(&flat));
    }
}
