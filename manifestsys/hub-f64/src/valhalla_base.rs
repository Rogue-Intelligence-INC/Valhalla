//! Valhalla-as-base inference — context-flexible structure substrate + optional LM decode.
//!
//! v1 extensions: hybrid routing, persistent session, multimodal context ingress.

use serde::{Deserialize, Serialize};

use crate::dl_front::{encode_modality, BodyFrontPorts, DlFrontConfig, ModalityInput};
use crate::fair_score::{extract_mcq_letter, format_prompt_text, score_answer, FairPrompt, ScoreResult};
use crate::free_multimodal_fate::{fuse_universal_core_on_snapshot, UniversalCoreMetrics};
use crate::grinder::{default_port_cap, grind_text_hash};
use crate::native_qa::{
    fog_tau_algebraic, run_native_qa, run_native_qa_batch, FogDecision, FogEvidence,
    NativeQaBatchRequest, NativeQaQuestionSpec, NativeQaRequest,
};
use crate::patch_lm::{PatchLmConfig, DEFAULT_MAX_NEW, DEFAULT_STRENGTH};
use crate::signal_ingress::{BodyTarget, IngressBudget, TriadIncubationSnapshot, TriadSession};
use crate::{FateWeightMode, IngressRoutingMode};

pub const VALHALLA_BASE_PROTOCOL: &str = "valhalla-base-v1";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ValhallaBaseDecode {
    Native,
    #[default]
    LmPatch,
    /// open → native retrieval; mcq/numeric → LM logprob/generate.
    Hybrid,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ValhallaMcqDecode {
    #[default]
    LmPatch,
    /// MCQ via native pipeline + mcq_fate (no LM logprob).
    StructureFate,
    /// Run native mcq_option + lm_patch logprob; deploy pick by letter agreement + margin.
    MaxPick,
}

/// How to decode open questions when session has prior turns.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ValhallaFollowUpDecode {
    /// Same routing as first question (native under Hybrid).
    #[default]
    Inherit,
    /// Follow-ups use LM patch on incubated snapshot (structure→LM).
    LmPatchOnFollowUp,
    /// Stay native but rank corpus with prior-question-aware retrieval.
    NativeFollowUpAware,
}

/// Fate quality assignment during ingress (high constraint → relaxed).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ValhallaFateWeightMode {
    #[default]
    Fixed,
    Evaluated,
}

/// Hub ingress routing while Fate routing is locked.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ValhallaFateIngressRouting {
    #[default]
    /// Within each corpus-line loop, rotate signals across all 4 quads (production default).
    QuadCycle,
    RoundRobin,
    /// Route corpus lines by semantic keywords → asymmetric quad load.
    Semantic,
    /// 50% quad-cycle + 50% semantic primary quad per corpus line.
    HybridQuadSemantic,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ValhallaContextItem {
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub audio_features: Option<Vec<f64>>,
    #[serde(default)]
    pub image_features: Option<Vec<f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValhallaBaseSessionState {
    pub corpus: Vec<String>,
    pub fingerprint: String,
    pub lines_ingested: usize,
    pub snapshot: TriadIncubationSnapshot,
    /// Prior user questions in this session (for follow-up routing / retrieval).
    #[serde(default)]
    pub question_history: Vec<String>,
    /// Fate tick after context ingest — proactive gate counts signals since this baseline.
    #[serde(default)]
    pub fate_baseline_tick: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValhallaBaseQuestion {
    pub id: String,
    pub text: String,
    #[serde(default = "default_open")]
    pub score_type: String,
    #[serde(default)]
    pub reference_answer: String,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub audio_features: Option<Vec<f64>>,
    #[serde(default)]
    pub image_features: Option<Vec<f64>>,
}

fn default_open() -> String {
    "open".into()
}

impl From<ValhallaBaseQuestion> for FairPrompt {
    fn from(q: ValhallaBaseQuestion) -> Self {
        FairPrompt {
            id: q.id,
            domain: String::new(),
            text: q.text,
            reference_answer: q.reference_answer,
            keywords: q.keywords,
            score_type: q.score_type,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValhallaBaseConfig {
    #[serde(default = "default_lm_model")]
    pub lm_model: String,
    #[serde(default = "default_triad")]
    pub patch_body: String,
    #[serde(default = "default_strength")]
    pub strength: f64,
    #[serde(default = "default_max_new")]
    pub max_new: usize,
    #[serde(default = "default_cycles")]
    pub incubation_cycles: usize,
    #[serde(default)]
    pub use_cpu: bool,
    #[serde(default)]
    pub decode: ValhallaBaseDecode,
    /// Fuse task text/TTS/image into context snapshot before decode (structure→LM).
    #[serde(default)]
    pub core_fusion: bool,
    /// When decode=Hybrid, how to answer MCQ (default: LM logprob).
    #[serde(default)]
    pub mcq_decode: ValhallaMcqDecode,
    /// Follow-up turn decode policy for open questions.
    #[serde(default)]
    pub follow_up_decode: ValhallaFollowUpDecode,
    /// Fate ingress weight mode (fixed = high constraint).
    #[serde(default)]
    pub fate_weight_mode: ValhallaFateWeightMode,
    /// When true, Fate preference routing is enabled; false = round-robin explore.
    #[serde(default)]
    pub fate_routing_free: bool,
    /// Ingress routing while `fate_routing_free` is false.
    #[serde(default)]
    pub fate_ingress_routing: ValhallaFateIngressRouting,
    /// When true, push QA correct/incorrect back into Fate after each answer.
    #[serde(default = "default_true")]
    pub fate_qa_feedback: bool,
    /// Enable proactive idle path (self-probe + FOG-gated nudge after Fate signal warmup).
    #[serde(default = "default_true")]
    pub proactive_enabled: bool,
    /// Minimum Fate signal triggers since context baseline before proactive idle may fire.
    #[serde(default = "default_proactive_min_fate_signals")]
    pub proactive_min_fate_signals: usize,
    /// Deprecated alias — mapped to fate signals if set and min_fate_signals unset in env.
    #[serde(default = "default_proactive_min_turns_legacy")]
    pub proactive_min_turns: usize,
    /// Minimum hub_prefs spread — default τ(0)=12/138 from E7 algebra.
    #[serde(default = "default_proactive_min_spread")]
    pub proactive_min_spread: f64,
    /// Multi-hop structure composition for open QA (hop split + yes/no / entity synthesis).
    #[serde(default)]
    pub multihop_v2: bool,
}

fn default_proactive_min_fate_signals() -> usize {
    std::env::var("VALHALLA_PROACTIVE_MIN_FATE_SIGNALS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(64)
}

fn default_proactive_min_turns_legacy() -> usize {
    0
}

fn default_proactive_min_spread() -> f64 {
    // τ(0) × δ/(dim+r) — E7 coupling scale (≈0.003), not raw τ(0)
    fog_tau_algebraic(0) * crate::native_qa::fog_delta_required_algebraic(1.0)
}

fn default_true() -> bool {
    true
}

fn default_triad() -> String {
    "triad".into()
}
fn default_lm_model() -> String {
    "Qwen/Qwen2.5-0.5B-Instruct".into()
}
fn default_strength() -> f64 {
    DEFAULT_STRENGTH
}
fn default_max_new() -> usize {
    DEFAULT_MAX_NEW
}
fn default_cycles() -> usize {
    2
}

fn question_history_cap() -> usize {
    std::env::var("VALHALLA_QUESTION_HISTORY_MAX")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(12)
}

fn cap_question_history(history: Vec<String>) -> Vec<String> {
    let cap = question_history_cap();
    if history.len() <= cap {
        return history;
    }
    history[history.len() - cap..].to_vec()
}

impl Default for ValhallaBaseConfig {
    fn default() -> Self {
        Self {
            lm_model: default_lm_model(),
            patch_body: default_triad(),
            strength: default_strength(),
            max_new: default_max_new(),
            incubation_cycles: default_cycles(),
            use_cpu: false,
            decode: ValhallaBaseDecode::LmPatch,
            core_fusion: false,
            mcq_decode: ValhallaMcqDecode::LmPatch,
            follow_up_decode: ValhallaFollowUpDecode::Inherit,
            fate_weight_mode: ValhallaFateWeightMode::Fixed,
            fate_routing_free: false,
            fate_ingress_routing: ValhallaFateIngressRouting::QuadCycle,
            fate_qa_feedback: true,
            proactive_enabled: true,
            proactive_min_fate_signals: default_proactive_min_fate_signals(),
            proactive_min_turns: 0,
            proactive_min_spread: default_proactive_min_spread(),
            multihop_v2: crate::multihop_compose::multihop_v2_enabled(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ValhallaBaseRequest {
    /// Legacy text-only context lines.
    #[serde(default)]
    pub context: Vec<String>,
    /// Multimodal context bundle (text / audio_features / image_features per item).
    #[serde(default)]
    pub context_items: Vec<ValhallaContextItem>,
    /// Incremental append — merged with session or full context (no full re-send).
    #[serde(default)]
    pub append_lines: Vec<String>,
    #[serde(default)]
    pub append_items: Vec<ValhallaContextItem>,
    #[serde(default)]
    pub session: Option<ValhallaBaseSessionState>,
    pub questions: Vec<ValhallaBaseQuestion>,
    /// When true with empty `questions`, run proactive idle (self-probe + FOG nudge).
    #[serde(default)]
    pub proactive_idle: bool,
    #[serde(default)]
    pub config: ValhallaBaseConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProactiveOutcome {
    pub triggered: bool,
    pub trigger_reason: String,
    pub probe_id: String,
    pub probe_question: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nudge: Option<String>,
    pub fog_decision: String,
    pub emitted: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fog_evidence: Option<FogEvidence>,
    #[serde(default)]
    pub fate_signals_since_baseline: u64,
    #[serde(default)]
    pub fate_baseline_tick: u64,
    #[serde(default)]
    pub proactive_min_fate_signals: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValhallaBaseRow {
    pub id: String,
    pub score_type: String,
    pub answer: String,
    pub decode_path: String,
    pub correct: bool,
    pub score: ScoreResult,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub core_fusion: Option<CoreFusionRow>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fog_evidence: Option<FogEvidence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fog_decision: Option<FogDecision>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mcq_native_aggregate: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mcq_patch_confidence: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retrieval_corpus_idx: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreFusionRow {
    pub universal_score: f64,
    pub universal_formed: bool,
    pub shared_patch: f64,
    pub decode_snap: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FateTelemetry {
    pub hub_prefs: Vec<f64>,
    pub hub_prefs_spread: f64,
    pub hub_routed: [u64; 4],
    pub fate_weight_mode: String,
    pub fate_routing_free: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fate_ingress_routing: Option<String>,
    pub incubation_cycles: usize,
    /// E7 algebra identifiers for Fate/FOG gates
    #[serde(default = "default_algebra_formula")]
    pub algebra_formula: String,
    #[serde(default)]
    pub corpus_line_affinity_nonzero: usize,
    #[serde(default)]
    pub corpus_line_affinity_max: f64,
}

fn default_algebra_formula() -> String {
    "e7_tau_v1".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValhallaBaseResponse {
    pub protocol: String,
    pub context_lines: usize,
    pub context_fingerprint: String,
    pub decode: ValhallaBaseDecode,
    pub session: ValhallaBaseSessionState,
    pub features: Vec<String>,
    pub rows: Vec<ValhallaBaseRow>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fate_telemetry: Option<FateTelemetry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proactive: Option<ProactiveOutcome>,
}

pub fn context_fingerprint(context: &[String]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    context.len().hash(&mut h);
    for line in context {
        line.trim().hash(&mut h);
    }
    format!("{:016x}", h.finish())
}

pub fn item_to_corpus_line(item: &ValhallaContextItem) -> Option<String> {
    if let Some(t) = item.text.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
        return Some(t.to_string());
    }
    if item.audio_features.as_ref().is_some_and(|v| !v.is_empty()) {
        return Some("[audio_context]".into());
    }
    if item.image_features.as_ref().is_some_and(|v| !v.is_empty()) {
        return Some("[image_context]".into());
    }
    None
}

fn corpus_from_items(items: &[ValhallaContextItem]) -> Vec<String> {
    items.iter().filter_map(item_to_corpus_line).collect()
}

fn merge_context(req: &ValhallaBaseRequest) -> (Vec<String>, Vec<ValhallaContextItem>, bool) {
    let mut lines: Vec<String> = req
        .context
        .iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let mut items = req.context_items.clone();

    if let Some(sess) = &req.session {
        if req.context.is_empty() && req.context_items.is_empty() {
            lines = sess.corpus.clone();
        }
    }

    for l in &req.append_lines {
        let t = l.trim();
        if !t.is_empty() {
            lines.push(t.to_string());
            items.push(ValhallaContextItem {
                text: Some(t.to_string()),
                ..Default::default()
            });
        }
    }
    items.extend(req.append_items.clone());

    for item in &req.context_items {
        if let Some(l) = item_to_corpus_line(item) {
            if !lines.contains(&l) {
                lines.push(l);
            }
        }
    }

    let incremental = req.session.is_some()
        && (!req.append_lines.is_empty() || !req.append_items.is_empty())
        && req.context.is_empty()
        && req.context_items.is_empty();

    (lines, items, incremental)
}

fn feed_context_item(session: &mut TriadSession, item: &ValhallaContextItem, front: &DlFrontConfig) {
    if let Some(text) = item.text.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
        session.feed_modality(
            ModalityInput::Text {
                content: text.to_string(),
            },
            front,
        );
    }
    if let Some(audio) = item.audio_features.as_ref().filter(|v| !v.is_empty()) {
        session.feed_modality(
            ModalityInput::AudioFeatures {
                values: audio.clone(),
            },
            front,
        );
    }
    if let Some(img) = item.image_features.as_ref().filter(|v| !v.is_empty()) {
        session.feed_modality(
            ModalityInput::ImageFeatures {
                values: img.clone(),
            },
            front,
        );
    }
}

fn apply_fate_config(session: &mut TriadSession, cfg: &ValhallaBaseConfig) {
    let mode = match cfg.fate_weight_mode {
        ValhallaFateWeightMode::Fixed => FateWeightMode::Fixed,
        ValhallaFateWeightMode::Evaluated => FateWeightMode::Evaluated,
    };
    let ingress = match cfg.fate_ingress_routing {
        ValhallaFateIngressRouting::RoundRobin => IngressRoutingMode::RoundRobin,
        ValhallaFateIngressRouting::Semantic => IngressRoutingMode::Semantic,
        ValhallaFateIngressRouting::QuadCycle => IngressRoutingMode::QuadCycle,
        ValhallaFateIngressRouting::HybridQuadSemantic => IngressRoutingMode::HybridQuadSemantic,
    };
    session.hub.set_fate_policy(mode, cfg.fate_routing_free, ingress);
}

fn hub_prefs_spread(prefs: &[f64]) -> f64 {
    if prefs.is_empty() {
        return 0.0;
    }
    let max = prefs.iter().copied().fold(0.0f64, f64::max);
    let min = prefs.iter().copied().fold(f64::INFINITY, f64::min);
    if min.is_finite() {
        max - min
    } else {
        0.0
    }
}

fn fate_telemetry_from_snap(snap: &TriadIncubationSnapshot, cfg: &ValhallaBaseConfig) -> FateTelemetry {
    let aff = &snap.corpus_line_affinity;
    FateTelemetry {
        hub_prefs: snap.hub_prefs.clone(),
        hub_prefs_spread: hub_prefs_spread(&snap.hub_prefs),
        hub_routed: snap.hub_routed,
        fate_weight_mode: match cfg.fate_weight_mode {
            ValhallaFateWeightMode::Fixed => "fixed".into(),
            ValhallaFateWeightMode::Evaluated => "evaluated".into(),
        },
        fate_routing_free: cfg.fate_routing_free,
        fate_ingress_routing: Some(match cfg.fate_ingress_routing {
            ValhallaFateIngressRouting::RoundRobin => "round_robin".into(),
            ValhallaFateIngressRouting::Semantic => "semantic".into(),
            ValhallaFateIngressRouting::QuadCycle => "quad_cycle".into(),
            ValhallaFateIngressRouting::HybridQuadSemantic => "hybrid_quad_semantic".into(),
        }),
        incubation_cycles: cfg.incubation_cycles.max(1),
        algebra_formula: default_algebra_formula(),
        corpus_line_affinity_nonzero: aff.iter().filter(|&&v| v > 0.0).count(),
        corpus_line_affinity_max: aff.iter().copied().fold(0.0, f64::max),
    }
}

fn build_snapshot_from_items(
    body: &str,
    lines: &[String],
    items: &[ValhallaContextItem],
    cycles: usize,
    cfg: &ValhallaBaseConfig,
) -> TriadIncubationSnapshot {
    let has_mm = items.iter().any(|i| {
        i.audio_features.as_ref().is_some_and(|v| !v.is_empty())
            || i.image_features.as_ref().is_some_and(|v| !v.is_empty())
    });
    if !has_mm {
        let target = BodyTarget::parse(body);
        let budget = IngressBudget::experiment();
        let mut session = TriadSession::with_body(budget, target);
        apply_fate_config(&mut session, cfg);
        let cap = default_port_cap();
        for line in lines {
            let ports = grind_text_hash(line, cap);
            session.emit_body_ports_with_line(&ports, Some(line));
        }
        session.run_optimize_cycles(cycles.max(1));
        return session.incubation_snapshot();
    }
    let target = BodyTarget::parse(body);
    let budget = IngressBudget::experiment();
    let mut session = TriadSession::with_body(budget, target);
    apply_fate_config(&mut session, cfg);
    let front = DlFrontConfig::default();
    let cap = default_port_cap();
    for line in lines {
        let ports = grind_text_hash(line, cap);
        session.emit_body_ports_with_line(&ports, Some(line));
    }
    for item in items {
        feed_context_item(&mut session, item, &front);
    }
    session.run_optimize_cycles(cycles.max(1));
    session.incubation_snapshot()
}

fn extend_snapshot(
    body: &str,
    prev: TriadIncubationSnapshot,
    new_lines: &[String],
    new_items: &[ValhallaContextItem],
    cycles: usize,
    cfg: &ValhallaBaseConfig,
) -> TriadIncubationSnapshot {
    let target = BodyTarget::parse(body);
    let budget = IngressBudget::experiment();
    let mut session = TriadSession::from_incubation_snapshot(budget, target, prev);
    apply_fate_config(&mut session, cfg);
    let front = DlFrontConfig::default();
    let cap = default_port_cap();
    for line in new_lines {
        let ports = grind_text_hash(line, cap);
        session.emit_body_ports_with_line(&ports, Some(line));
    }
    for item in new_items {
        feed_context_item(&mut session, item, &front);
    }
    session.run_optimize_cycles(cycles.max(1));
    session.incubation_snapshot()
}

fn resolve_snapshot(
    req: &ValhallaBaseRequest,
    lines: &[String],
    items: &[ValhallaContextItem],
    incremental: bool,
) -> TriadIncubationSnapshot {
    let cycles = req.config.incubation_cycles.max(1);
    let body = &req.config.patch_body;
    if incremental {
        if let Some(sess) = &req.session {
            return extend_snapshot(
                body,
                sess.snapshot.clone(),
                &req.append_lines,
                &req.append_items,
                cycles,
                &req.config,
            );
        }
    }
    if let Some(sess) = &req.session {
        if req.context.is_empty()
            && req.context_items.is_empty()
            && req.append_lines.is_empty()
            && req.append_items.is_empty()
        {
            return sess.snapshot.clone();
        }
    }
    build_snapshot_from_items(body, lines, items, cycles, &req.config)
}

fn effective_decode(cfg: &ValhallaBaseConfig, score_type: &str) -> ValhallaBaseDecode {
    match cfg.decode {
        ValhallaBaseDecode::Hybrid => match score_type {
            "mcq" | "numeric" if cfg.mcq_decode == ValhallaMcqDecode::StructureFate => {
                ValhallaBaseDecode::Native
            }
            "mcq" | "numeric" => ValhallaBaseDecode::LmPatch,
            _ => ValhallaBaseDecode::Native,
        },
        other => other,
    }
}

fn effective_decode_for_question(
    cfg: &ValhallaBaseConfig,
    score_type: &str,
    is_follow_up: bool,
) -> ValhallaBaseDecode {
    let base = effective_decode(cfg, score_type);
    if score_type != "open" || !is_follow_up {
        return base;
    }
    match cfg.follow_up_decode {
        ValhallaFollowUpDecode::Inherit => base,
        ValhallaFollowUpDecode::LmPatchOnFollowUp => ValhallaBaseDecode::LmPatch,
        ValhallaFollowUpDecode::NativeFollowUpAware => ValhallaBaseDecode::Native,
    }
}

fn task_ports(q: &ValhallaBaseQuestion, p: &FairPrompt, front: &DlFrontConfig) -> (BodyFrontPorts, BodyFrontPorts, BodyFrontPorts) {
    let llm = encode_modality(
        &ModalityInput::Text {
            content: format_prompt_text(p),
        },
        front,
    );
    let tts = q
        .audio_features
        .as_ref()
        .filter(|v| !v.is_empty())
        .map(|v| {
            encode_modality(
                &ModalityInput::AudioFeatures {
                    values: v.clone(),
                },
                front,
            )
        })
        .unwrap_or_else(|| llm.clone());
    let img = q
        .image_features
        .as_ref()
        .filter(|v| !v.is_empty())
        .map(|v| {
            encode_modality(
                &ModalityInput::ImageFeatures {
                    values: v.clone(),
                },
                front,
            )
        })
        .unwrap_or_else(|| llm.clone());
    (llm, tts, img)
}

fn maybe_fuse_core(
    cfg: &ValhallaBaseConfig,
    context_snap: TriadIncubationSnapshot,
    q: &ValhallaBaseQuestion,
    p: &FairPrompt,
) -> (TriadIncubationSnapshot, Option<CoreFusionRow>) {
    if !cfg.core_fusion {
        return (context_snap, None);
    }
    let has_mm = q.audio_features.as_ref().is_some_and(|v| !v.is_empty())
        || q.image_features.as_ref().is_some_and(|v| !v.is_empty());
    if !has_mm {
        return (context_snap, None);
    }
    let front = DlFrontConfig::default();
    let body = BodyTarget::parse(&cfg.patch_body);
    let (llm, tts, img) = task_ports(q, p, &front);
    let (fused, metrics) = fuse_universal_core_on_snapshot(context_snap, body, &llm, &tts, &img);
    let row = core_fusion_row(&metrics, "context+task_non_isolated");
    (fused, Some(row))
}

fn core_fusion_row(metrics: &UniversalCoreMetrics, snap_label: &str) -> CoreFusionRow {
    CoreFusionRow {
        universal_score: metrics.universal_core_score,
        universal_formed: metrics.universal_core_formed,
        shared_patch: metrics.shared_patch_pairwise_avg,
        decode_snap: snap_label.to_string(),
    }
}

fn apply_qa_feedback_to_snapshot(
    snap: TriadIncubationSnapshot,
    cfg: &ValhallaBaseConfig,
    question: &str,
    on_topic: bool,
    question_ports: &BodyFrontPorts,
    retrieved_corpus_idx: Option<usize>,
    question_sig: Option<Vec<f64>>,
    line_sig: Option<Vec<f64>>,
    line_text: Option<&str>,
) -> TriadIncubationSnapshot {
    let budget = IngressBudget::experiment();
    let target = BodyTarget::parse(&cfg.patch_body);
    let mut session = TriadSession::from_incubation_snapshot(budget, target, snap);
    apply_fate_config(&mut session, cfg);
    session.emit_body_ports(question_ports);
    if cfg.fate_qa_feedback {
        session.apply_qa_feedback(question, on_topic);
        if let (Some(idx), Some(qs), Some(ls), Some(lt)) =
            (retrieved_corpus_idx, question_sig, line_sig, line_text)
        {
            session.apply_corpus_retrieval_feedback(idx, &qs, &ls, lt, on_topic);
        }
    }
    session.incubation_snapshot()
}

fn effective_proactive_min_fate_signals(cfg: &ValhallaBaseConfig) -> usize {
    if cfg.proactive_min_turns > 0 {
        cfg.proactive_min_turns.saturating_mul(32)
    } else {
        cfg.proactive_min_fate_signals
    }
}

fn fate_signals_since_baseline(snap: &TriadIncubationSnapshot, baseline: u64) -> u64 {
    snap.fate_signal_tick.saturating_sub(baseline)
}

fn grind_ports_for_line(line: &str) -> BodyFrontPorts {
    let cap = default_port_cap();
    grind_text_hash(line, cap)
}

fn run_native_one(
    corpus: &[String],
    p: &FairPrompt,
    q: &ValhallaBaseQuestion,
    cfg: &ValhallaBaseConfig,
    context_snap: TriadIncubationSnapshot,
    prior_questions: &[String],
) -> Result<ValhallaBaseRow, String> {
    let (fused_snap, core_row) = maybe_fuse_core(cfg, context_snap, q, p);
    let corpus_ports: Vec<BodyFrontPorts> =
        corpus.iter().map(|l| grind_ports_for_line(l)).collect();
    let q_ports = grind_ports_for_line(&format_prompt_text(p));
    let batch = NativeQaBatchRequest {
        body: cfg.patch_body.clone(),
        corpus: corpus.to_vec(),
        corpus_ports,
        questions: vec![NativeQaQuestionSpec {
            id: Some(p.id.clone()),
            question: format_prompt_text(p),
            score_type: p.score_type.clone(),
            question_ports: Some(q_ports),
            audio_features: q.audio_features.clone(),
            prior_questions: if prior_questions.is_empty() {
                None
            } else {
                Some(prior_questions.to_vec())
            },
        }],
        incubation_cycles: cfg.incubation_cycles.max(1),
        qa_cycles: 1,
        ingress_mode: Some("grinder".into()),
        mcq_fate: p.score_type == "mcq" && cfg.mcq_decode == ValhallaMcqDecode::StructureFate,
        pipeline_v4: true,
        multihop_v2: cfg.multihop_v2,
        incubation_snapshot: Some(fused_snap),
    };
    let resp = run_native_qa_batch(&batch);
    let row = resp.results.into_iter().next().ok_or("native batch empty")?;
    let sc = score_answer(&row.answer, p);
    Ok(ValhallaBaseRow {
        id: p.id.clone(),
        score_type: p.score_type.clone(),
        answer: row.answer,
        decode_path: format!("valhalla_base/native/{}", row.decode_path),
        correct: sc.correct,
        score: sc,
        core_fusion: core_row,
        fog_evidence: row.fog_evidence,
        fog_decision: row.fog_decision,
        mcq_native_aggregate: if p.score_type == "mcq" {
            Some(row.mcq_aggregate_score)
        } else {
            None
        },
        mcq_patch_confidence: None,
        retrieval_corpus_idx: row.retrieval_corpus_idx,
    })
}

fn run_lm_one(
    _corpus: &[String],
    p: &FairPrompt,
    q: &ValhallaBaseQuestion,
    cfg: &ValhallaBaseConfig,
    context_snap: TriadIncubationSnapshot,
) -> Result<ValhallaBaseRow, String> {
    let (fused_snap, core_row) = maybe_fuse_core(cfg, context_snap, q, p);
    let patch_cfg = PatchLmConfig {
        model_dir: crate::patch_lm::resolve_model_dir(&cfg.lm_model),
        patch_body: cfg.patch_body.clone(),
        strength: cfg.strength,
        max_new: cfg.max_new,
        export_cycles: cfg.incubation_cycles.max(1),
        use_cpu: cfg.use_cpu,
        pipeline_v4: true,
        mcq_decode: crate::patch_lm::McqDecodeMode::LogprobBatch,
        runtime_patch: Some(true),
    };
    let mut audio_by_id = std::collections::HashMap::new();
    if let Some(v) = &q.audio_features {
        if !v.is_empty() {
            audio_by_id.insert(p.id.clone(), v.clone());
        }
    }
    let patch_rows = crate::patch_lm::run_patch_track_v4_with_snap(
        &fused_snap,
        std::slice::from_ref(p),
        &patch_cfg,
        if audio_by_id.is_empty() {
            None
        } else {
            Some(&audio_by_id)
        },
    )?;
    let pr = &patch_rows[0];
    let answer = pr
        .assistant
        .clone()
        .or_else(|| pr.native_answer.clone())
        .unwrap_or_default();
    let sc = score_answer(&answer, p);
    let decode_path = pr
        .decode_path
        .clone()
        .unwrap_or_else(|| "lm_patch".into());
    Ok(ValhallaBaseRow {
        id: p.id.clone(),
        score_type: p.score_type.clone(),
        answer,
        decode_path: format!("valhalla_base/lm_patch/{decode_path}"),
        correct: sc.correct,
        score: sc,
        core_fusion: core_row,
        fog_evidence: None,
        fog_decision: None,
        mcq_native_aggregate: None,
        mcq_patch_confidence: if p.score_type == "mcq" {
            pr.mcq_patch_confidence
        } else {
            None
        },
        retrieval_corpus_idx: None,
    })
}

/// Deploy MCQ pick: native structure vs lm_patch logprob (no oracle labels).
fn pick_mcq_max_deploy(native: ValhallaBaseRow, patch: ValhallaBaseRow) -> ValhallaBaseRow {
    let n_l = extract_mcq_letter(&native.answer);
    let p_l = extract_mcq_letter(&patch.answer);
    let native_agg = native.mcq_native_aggregate.unwrap_or(0.0);
    let patch_conf = patch.mcq_patch_confidence.unwrap_or(0.0);
    let native_margin = native
        .fog_evidence
        .as_ref()
        .map(|e| e.margin)
        .unwrap_or(0.0);

    let use_native = match (n_l, p_l) {
        (Some(_), None) => true,
        (None, Some(_)) | (None, None) => false,
        (Some(a), Some(b)) if a == b => false,
        (Some(_), Some(_)) => native_agg >= 0.48 && patch_conf <= 0.45 && native_margin >= 0.08,
    };

    if use_native {
        ValhallaBaseRow {
            decode_path: format!(
                "valhalla_base/mcq_max_pick/native/{}",
                native
                    .decode_path
                    .strip_prefix("valhalla_base/native/")
                    .unwrap_or(&native.decode_path)
            ),
            mcq_native_aggregate: native.mcq_native_aggregate,
            mcq_patch_confidence: patch.mcq_patch_confidence,
            ..native
        }
    } else {
        ValhallaBaseRow {
            decode_path: format!(
                "valhalla_base/mcq_max_pick/lm_patch/{}",
                patch
                    .decode_path
                    .strip_prefix("valhalla_base/lm_patch/")
                    .unwrap_or(&patch.decode_path)
            ),
            mcq_native_aggregate: native.mcq_native_aggregate,
            mcq_patch_confidence: patch.mcq_patch_confidence,
            ..patch
        }
    }
}

fn run_mcq_max_pick(
    corpus: &[String],
    p: &FairPrompt,
    q: &ValhallaBaseQuestion,
    cfg: &ValhallaBaseConfig,
    context_snap: TriadIncubationSnapshot,
    prior: &[String],
) -> Result<ValhallaBaseRow, String> {
    let native = run_native_one(corpus, p, q, cfg, context_snap.clone(), prior)?;
    let patch = run_lm_one(corpus, p, q, cfg, context_snap)?;
    Ok(pick_mcq_max_deploy(native, patch))
}

fn run_legacy_native_track(
    context: &[String],
    questions: &[FairPrompt],
    cfg: &ValhallaBaseConfig,
) -> Result<Vec<ValhallaBaseRow>, String> {
    let corpus = context.to_vec();
    let mut rows = Vec::with_capacity(questions.len());
    for p in questions {
        let req = NativeQaRequest {
            body: cfg.patch_body.clone(),
            cycles: cfg.incubation_cycles.max(1),
            corpus: corpus.clone(),
            question: format_prompt_text(p),
            score_type: p.score_type.clone(),
            ingress_mode: None,
            corpus_ports: None,
            question_ports: None,
            corpus_cycles: Some(cfg.incubation_cycles.max(1)),
            qa_cycles: Some(1),
            ingest_all_corpus: Some(true),
        };
        let resp = run_native_qa(&req);
        let sc = score_answer(&resp.answer, p);
        rows.push(ValhallaBaseRow {
            id: p.id.clone(),
            score_type: p.score_type.clone(),
            answer: resp.answer,
            decode_path: format!("valhalla_base/native/{}", resp.decode_path),
            correct: sc.correct,
            score: sc,
            core_fusion: None,
            fog_evidence: None,
            fog_decision: None,
            mcq_native_aggregate: None,
            mcq_patch_confidence: None,
            retrieval_corpus_idx: None,
        });
    }
    Ok(rows)
}

fn is_cjk(c: char) -> bool {
    ('\u{4e00}'..='\u{9fff}').contains(&c)
        || ('\u{3040}'..='\u{309f}').contains(&c)
        || ('\u{30a0}'..='\u{30ff}').contains(&c)
        || ('\u{ac00}'..='\u{d7a3}').contains(&c)
}

const PROACTIVE_STOP: &[&str] = &[
    "the", "what", "does", "context", "about", "which", "that", "this", "from", "with",
    "have", "been", "were", "when", "where", "would", "could", "should", "does", "say",
    "anything", "mention", "stated", "based", "conversation", "related", "our",
];

fn text_tokens(text: &str) -> std::collections::HashSet<String> {
    use std::collections::HashSet;
    let mut out = HashSet::new();
    for w in text.split_whitespace() {
        let w = w.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase();
        if w.len() > 2 && !PROACTIVE_STOP.contains(&w.as_str()) {
            out.insert(w);
        }
    }
    let mut digits = String::new();
    for c in text.chars() {
        if c.is_ascii_digit() {
            digits.push(c);
        } else if !digits.is_empty() {
            out.insert(digits.clone());
            digits.clear();
        }
    }
    if !digits.is_empty() {
        out.insert(digits);
    }
    let cjk: Vec<char> = text.chars().filter(|c| is_cjk(*c)).collect();
    // CJK bigrams only (avoid single-char false positives on mismatch corpora)
    for w in cjk.windows(2) {
        out.insert(w.iter().collect());
    }
    // Single CJK only when rare/specific (e.g. 秦)
    if cjk.len() == 1 {
        out.insert(cjk[0].to_string());
    }
    out
}

/// Acronym in prior (e.g. GDP) ↔ consecutive word initials in corpus line.
fn proactive_acronym_matches(line: &str, prior: &[String]) -> usize {
    let words: Vec<String> = line
        .split_whitespace()
        .map(|w| {
            w.trim_matches(|c: char| !c.is_alphabetic())
                .to_lowercase()
        })
        .filter(|w| !w.is_empty())
        .collect();
    let mut hits = 0usize;
    for p in prior {
        for raw in p.split_whitespace() {
            let w = raw.trim_matches(|c: char| !c.is_alphabetic());
            if w.len() < 2
                || w.len() > 6
                || !w.chars().all(|c| c.is_ascii_uppercase())
            {
                continue;
            }
            let initials: Vec<char> = w.chars().map(|c| c.to_ascii_lowercase()).collect();
            if words.len() < initials.len() {
                continue;
            }
            let matched = (0..=words.len() - initials.len()).any(|start| {
                initials
                    .iter()
                    .enumerate()
                    .all(|(i, c)| words[start + i].starts_with(*c))
            });
            if matched {
                hits += 1;
            }
        }
    }
    hits
}

fn proactive_overlap_count(line: &str, prior: &[String]) -> usize {
    let line_set = text_tokens(line);
    let mut best = 0usize;
    for p in prior {
        let prior_set = text_tokens(p);
        let mut overlap = prior_set.intersection(&line_set).count();
        overlap += prior_set
            .iter()
            .filter(|t| t.chars().any(is_cjk) && line.contains(t.as_str()))
            .count();
        best = best.max(overlap);
    }
    best + proactive_acronym_matches(line, prior)
}

fn dominant_script(prior: &[String]) -> &'static str {
    let mut cjk = 0usize;
    let mut latin = 0usize;
    for p in prior {
        for c in p.chars() {
            if is_cjk(c) {
                cjk += 1;
            } else if c.is_ascii_alphabetic() {
                latin += 1;
            }
        }
    }
    if cjk > latin { "cjk" } else { "latin" }
}

fn topic_snippet(line: &str) -> String {
    let cjk: String = line.chars().filter(|c| is_cjk(*c)).take(12).collect();
    if cjk.chars().count() >= 2 {
        return cjk;
    }
    line.split_whitespace().take(8).collect::<Vec<_>>().join(" ")
}

fn format_probe_question(topic: &str, prior: &[String]) -> String {
    if dominant_script(prior) == "cjk" {
        format!("根据上下文，关于「{topic}」说了什么？")
    } else {
        format!("Based on the context, what is stated about: {topic}?")
    }
}

fn build_proactive_probe(corpus: &[String], prior: &[String]) -> Option<(String, String, String)> {
    if corpus.is_empty() || prior.is_empty() {
        return None;
    }
    // Topic-continuous: pick corpus line with highest overlap with prior user turns.
    let (idx, overlap) = corpus
        .iter()
        .enumerate()
        .map(|(i, line)| (i, proactive_overlap_count(line, prior)))
        .max_by_key(|(_, count)| *count)?;
    if overlap == 0 {
        return None;
    }
    // Require meaningful overlap — single weak token insufficient on distractor corpora.
    if overlap < 2 {
        let picked = &corpus[idx];
        let line_set = text_tokens(picked);
        let strong = prior.iter().any(|p| {
            text_tokens(p).intersection(&line_set).any(|t| {
                t.len() >= 4
                    || t.chars().any(is_cjk)
                    || (t.chars().all(|c| c.is_ascii_digit()) && !t.is_empty())
            })
        });
        if !strong && proactive_acronym_matches(picked, prior) == 0 {
            return None;
        }
    }
    let line = &corpus[idx];
    let topic = topic_snippet(line);
    let id = format!("proactive_{}", prior.len());
    let text = format_probe_question(&topic, prior);
    Some((id, text, line.clone()))
}

fn proactive_trigger_gate(
    snap: &TriadIncubationSnapshot,
    fate_baseline: u64,
    cfg: &ValhallaBaseConfig,
) -> (bool, String, u64, usize) {
    let since = fate_signals_since_baseline(snap, fate_baseline);
    let min = effective_proactive_min_fate_signals(cfg);
    if !cfg.proactive_enabled {
        return (false, "proactive_disabled".into(), since, min);
    }
    if since < min as u64 {
        return (
            false,
            format!("fate_signals={since}<min_fate_signals={min}"),
            since,
            min,
        );
    }
    let spread = hub_prefs_spread(&snap.hub_prefs);
    if spread < cfg.proactive_min_spread {
        return (
            false,
            format!(
                "spread={spread:.4}<{min:.4}",
                spread = spread,
                min = cfg.proactive_min_spread
            ),
            since,
            min,
        );
    }
    (true, "idle_ready".into(), since, min)
}

fn proactive_outcome_blocked(
    reason: String,
    since: u64,
    min: usize,
    fate_baseline: u64,
) -> ProactiveOutcome {
    ProactiveOutcome {
        triggered: false,
        trigger_reason: reason,
        probe_id: String::new(),
        probe_question: String::new(),
        nudge: None,
        fog_decision: "defer".into(),
        emitted: false,
        fog_evidence: None,
        fate_signals_since_baseline: since,
        fate_baseline_tick: fate_baseline,
        proactive_min_fate_signals: min,
    }
}

fn run_proactive_idle(
    _req: &ValhallaBaseRequest,
    lines: &[String],
    context_snap: TriadIncubationSnapshot,
    prior: &[String],
    fate_baseline: u64,
    cfg: &ValhallaBaseConfig,
) -> Result<(ProactiveOutcome, Vec<ValhallaBaseRow>), String> {
    let (ready, reason, since, min) =
        proactive_trigger_gate(&context_snap, fate_baseline, cfg);
    if !ready {
        return Ok((proactive_outcome_blocked(reason, since, min, fate_baseline), vec![]));
    }

    let Some((probe_id, probe_text, source_line)) = build_proactive_probe(lines, prior) else {
        return Ok((
            proactive_outcome_blocked("no_topic_overlap".into(), since, min, fate_baseline),
            vec![],
        ));
    };

    let q = ValhallaBaseQuestion {
        id: probe_id.clone(),
        text: probe_text.clone(),
        score_type: "open".into(),
        reference_answer: source_line,
        keywords: vec![],
        audio_features: None,
        image_features: None,
    };
    let p: FairPrompt = q.clone().into();
    let row = run_native_one(lines, &p, &q, cfg, context_snap, prior)?;
    let fog_decision = row
        .fog_decision
        .as_ref()
        .map(|d| d.decision.clone())
        .unwrap_or_else(|| "emit".into());
    let emitted = fog_decision == "emit";
    let nudge = if emitted {
        Some(format!(
            "Related to our conversation: {}",
            row.answer.trim()
        ))
    } else {
        None
    };

    Ok((
        ProactiveOutcome {
            triggered: true,
            trigger_reason: reason,
            probe_id,
            probe_question: probe_text,
            nudge,
            fog_decision,
            emitted,
            fog_evidence: row.fog_evidence.clone(),
            fate_signals_since_baseline: since,
            fate_baseline_tick: fate_baseline,
            proactive_min_fate_signals: min,
        },
        vec![row],
    ))
}

/// Run Valhalla-as-base with hybrid / session / multimodal context.
pub fn run_valhalla_base(req: &ValhallaBaseRequest) -> Result<ValhallaBaseResponse, String> {
    let (lines, items, incremental) = merge_context(req);
    if lines.is_empty() && items.is_empty() {
        return Err("valhalla_base: context must contain at least one item or line".into());
    }
    if req.questions.is_empty() && !req.proactive_idle {
        return Err("valhalla_base: questions must be non-empty (or set proactive_idle)".into());
    }

    let mut context_snap = resolve_snapshot(req, &lines, &items, incremental);
    let cfg = &req.config;

    let continuing_session_only = req.session.is_some()
        && req.context.is_empty()
        && req.context_items.is_empty()
        && req.append_lines.is_empty()
        && req.append_items.is_empty();
    let mut fate_baseline = req
        .session
        .as_ref()
        .map(|s| s.fate_baseline_tick)
        .unwrap_or(0);
    if !continuing_session_only || fate_baseline == 0 {
        fate_baseline = context_snap.fate_signal_tick;
    }
    let mut features = vec!["context_flex".into()];
    if cfg.decode == ValhallaBaseDecode::Hybrid {
        features.push("hybrid_routing".into());
    }
    if cfg.mcq_decode == ValhallaMcqDecode::StructureFate {
        features.push("mcq_structure_fate".into());
    }
    if cfg.mcq_decode == ValhallaMcqDecode::MaxPick {
        features.push("mcq_max_pick".into());
    }
    if cfg.core_fusion {
        features.push("core_fusion".into());
    }
    if incremental {
        features.push("session_append".into());
    }
    match cfg.follow_up_decode {
        ValhallaFollowUpDecode::LmPatchOnFollowUp => features.push("follow_up_lm_patch".into()),
        ValhallaFollowUpDecode::NativeFollowUpAware => {
            features.push("follow_up_aware_retrieval".into())
        }
        ValhallaFollowUpDecode::Inherit => {}
    }
    match cfg.fate_weight_mode {
        ValhallaFateWeightMode::Evaluated => features.push("fate_weight_evaluated".into()),
        ValhallaFateWeightMode::Fixed => features.push("fate_weight_fixed".into()),
    }
    if cfg.fate_routing_free {
        features.push("fate_routing_free".into());
    }
    if cfg.fate_ingress_routing == ValhallaFateIngressRouting::Semantic {
        features.push("fate_ingress_semantic".into());
    }
    if cfg.fate_ingress_routing == ValhallaFateIngressRouting::QuadCycle {
        features.push("fate_ingress_quad_cycle".into());
    }
    if cfg.fate_ingress_routing == ValhallaFateIngressRouting::HybridQuadSemantic {
        features.push("fate_ingress_hybrid_quad_semantic".into());
    }
    if cfg.fate_qa_feedback {
        features.push("fate_qa_feedback".into());
    }
    if cfg.proactive_enabled {
        features.push("proactive_idle".into());
    }
    if req.proactive_idle {
        features.push("proactive_idle_request".into());
    }
    if items.iter().any(|i| {
        i.audio_features.as_ref().is_some_and(|v| !v.is_empty())
            || i.image_features.as_ref().is_some_and(|v| !v.is_empty())
    }) {
        features.push("multimodal_context".into());
    }

    let mut prior: Vec<String> = cap_question_history(
        req.session
            .as_ref()
            .map(|s| s.question_history.clone())
            .unwrap_or_default(),
    );

    let mut proactive_outcome: Option<ProactiveOutcome> = None;
    let mut rows = Vec::new();

    if req.proactive_idle && req.questions.is_empty() {
        let (outcome, probe_rows) = run_proactive_idle(
            req,
            &lines,
            context_snap.clone(),
            &prior,
            fate_baseline,
            cfg,
        )?;
        proactive_outcome = Some(outcome);
        rows = probe_rows;
    } else {
        rows = Vec::with_capacity(req.questions.len());
        for q in &req.questions {
            let p: FairPrompt = q.clone().into();
            let is_follow_up = !prior.is_empty();
            let row = if p.score_type == "mcq" && cfg.mcq_decode == ValhallaMcqDecode::MaxPick {
                run_mcq_max_pick(&lines, &p, q, cfg, context_snap.clone(), &prior)?
            } else {
                let mode = effective_decode_for_question(cfg, &p.score_type, is_follow_up);
                match mode {
                    ValhallaBaseDecode::Native => {
                        run_native_one(&lines, &p, q, cfg, context_snap.clone(), &prior)?
                    }
                    ValhallaBaseDecode::LmPatch => {
                        run_lm_one(&lines, &p, q, cfg, context_snap.clone())?
                    }
                    ValhallaBaseDecode::Hybrid => unreachable!(),
                }
            };
            prior.push(q.text.clone());
            rows.push(row.clone());
            let q_ports = grind_ports_for_line(&format_prompt_text(&p));
            context_snap = apply_qa_feedback_to_snapshot(
                context_snap,
                cfg,
                &q.text,
                row.correct,
                &q_ports,
                row.retrieval_corpus_idx,
                Some(q_ports.embedding.clone()),
                row.retrieval_corpus_idx.and_then(|i| {
                    lines.get(i).map(|l| grind_ports_for_line(l).embedding)
                }),
                row.retrieval_corpus_idx
                    .and_then(|i| lines.get(i).map(|l| l.as_str())),
            );
        }
    }

    let snap = context_snap;
    let fate_tel = fate_telemetry_from_snap(&snap, cfg);
    let session = ValhallaBaseSessionState {
        corpus: lines.clone(),
        fingerprint: context_fingerprint(&lines),
        lines_ingested: lines.len(),
        snapshot: snap,
        question_history: cap_question_history(prior),
        fate_baseline_tick: fate_baseline,
    };

    Ok(ValhallaBaseResponse {
        protocol: VALHALLA_BASE_PROTOCOL.to_string(),
        context_lines: lines.len(),
        context_fingerprint: session.fingerprint.clone(),
        decode: cfg.decode,
        session,
        features,
        rows,
        fate_telemetry: Some(fate_tel),
        proactive: proactive_outcome,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_changes_with_context() {
        let a = context_fingerprint(&["alpha".into()]);
        let b = context_fingerprint(&["beta".into()]);
        assert_ne!(a, b);
    }

    #[test]
    fn hybrid_routes_mcq_to_lm_by_default() {
        let cfg = ValhallaBaseConfig {
            decode: ValhallaBaseDecode::Hybrid,
            mcq_decode: ValhallaMcqDecode::LmPatch,
            ..Default::default()
        };
        assert_eq!(
            effective_decode(&cfg, "mcq"),
            ValhallaBaseDecode::LmPatch
        );
        assert_eq!(
            effective_decode(&cfg, "open"),
            ValhallaBaseDecode::Native
        );
    }

    #[test]
    fn proactive_acronym_gdp_matches_gross_domestic_product() {
        let prior = vec![
            "What does GDP stand for?".into(),
            "Does the context say anything about gross?".into(),
        ];
        let line = "Gross Domestic Product sums consumption investment government and net exports.";
        assert!(proactive_acronym_matches(line, &prior) >= 1);
        assert!(proactive_overlap_count(line, &prior) >= 1);
        assert!(build_proactive_probe(&[line.into()], &prior).is_some());
    }

    #[test]
    fn proactive_single_digit_overlap() {
        let prior = vec![
            "How many electrons in first shell?".into(),
            "Does the context say anything about 2?".into(),
        ];
        let line = "The first electron shell holds at most 2 electrons.";
        assert!(proactive_overlap_count(line, &prior) >= 1);
    }
}
