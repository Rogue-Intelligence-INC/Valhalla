//! Free multimodal fusion — LLM text + TTS audio, **no preset body routing**.
//!
//! Experiment protocol `valhalla-free-multimodal-fusion-v1`:
//!   both modalities pulse Hub / Tile / Stem freely → Fate multi-cycle adapt → measure
//!   whether cross-modal structure emerges (natural multimodal potential).
//!
//! Env:
//!   `VALHALLA_FREE_MULTIMODAL=1` — enable in downstream pipelines
//!   `VALHALLA_FREE_MULTIMODAL_MIN_CYCLES` (default 5)
//!   `VALHALLA_FREE_MULTIMODAL_MAX_CYCLES` (default 64)

use serde::{Deserialize, Serialize};

use crate::dl_front::{build_body_ports, encode_modality, BodyFrontPorts, DlFrontConfig, ModalityInput};
use crate::mcq_fate::McqFateCycleSnapshot;
use crate::signal_ingress::{BodyTarget, IngressBudget, TriadFeedReport, TriadIncubationSnapshot, TriadSession};

pub const FREE_MULTIMODAL_PROTOCOL: &str = "valhalla-free-multimodal-fusion-v1";
pub const FREE_MULTIMODAL_FUSION_V2: &str = "valhalla-free-multimodal-fusion-v2";
pub const FREE_MULTIMODAL_FUSION_V3: &str = "valhalla-free-multimodal-fusion-v3";

const FREE_FUSION_RULES: &[&str] = &[
    "task_tag:free_multimodal_fusion",
    "no_preset_routing: text audio image may enter hub tile stem freely",
    "fate_adapt: multi-round until cross-modal closure or marginal stable",
    "discovery_only: no fixed multimodal phenomenon list",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeMultimodalSample {
    pub id: String,
    pub text: String,
    pub audio_features: Vec<f64>,
    #[serde(default)]
    pub image_features: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriModalSample {
    pub id: String,
    pub text: String,
    pub audio_features: Vec<f64>,
    pub image_features: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergenceMetrics {
    pub text_only_tiles: usize,
    pub audio_only_tiles: usize,
    pub fused_tiles: usize,
    pub text_only_stem: usize,
    pub audio_only_stem: usize,
    pub fused_stem: usize,
    pub tile_superadditive: bool,
    pub stem_superadditive: bool,
    pub patch_text_audio_cosine: f64,
    pub fusion_vs_text_cosine: f64,
    pub fusion_vs_audio_cosine: f64,
    pub cross_modal_binding: f64,
    pub hub_modality_separation: f64,
    pub natural_multimodal_score: f64,
    pub emergence_likely: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeMultimodalOutcome {
    pub report: TriadFeedReport,
    pub trace: Vec<McqFateCycleSnapshot>,
    pub adaptation_rounds: usize,
    pub stopped_reason: String,
    pub aggregate_score: f64,
    pub cross_modal_closure: f64,
    pub emergence: EmergenceMetrics,
}

pub fn free_multimodal_enabled() -> bool {
    std::env::var("VALHALLA_FREE_MULTIMODAL")
        .map(|v| {
            let v = v.trim().to_ascii_lowercase();
            v == "1" || v == "true" || v == "on"
        })
        .unwrap_or(false)
}

fn env_usize(key: &str, default: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

pub fn free_multimodal_min_cycles() -> usize {
    env_usize("VALHALLA_FREE_MULTIMODAL_MIN_CYCLES", 5).clamp(2, 64)
}

pub fn free_multimodal_max_cycles() -> usize {
    env_usize("VALHALLA_FREE_MULTIMODAL_MAX_CYCLES", 64).clamp(4, 512)
}

pub fn free_multimodal_hard_max() -> usize {
    let hard = env_usize("VALHALLA_FREE_MULTIMODAL_HARD_MAX", 128);
    hard.max(free_multimodal_max_cycles()).clamp(8, 512)
}

pub fn free_multimodal_deep_enabled() -> bool {
    std::env::var("VALHALLA_FREE_MULTIMODAL_DEEP")
        .map(|v| {
            let v = v.trim().to_ascii_lowercase();
            v == "1" || v == "true" || v == "on"
        })
        .unwrap_or(false)
}

pub fn free_multimodal_optimize_cycles_per_pulse() -> usize {
    if free_multimodal_deep_enabled() {
        env_usize("VALHALLA_FREE_MULTIMODAL_PULSE_CYCLES", 3).clamp(2, 8)
    } else {
        1
    }
}

pub fn free_multimodal_rounds_per_sample() -> usize {
    env_usize("VALHALLA_FREE_MULTIMODAL_ROUNDS_PER_SAMPLE", 4).clamp(1, 32)
}

pub fn free_multimodal_closure_rounds() -> usize {
    env_usize("VALHALLA_FREE_MULTIMODAL_CLOSURE_ROUNDS", 8).clamp(2, 64)
}

fn free_multimodal_mix_alpha() -> f64 {
    std::env::var("VALHALLA_FREE_MULTIMODAL_MIX_ALPHA")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.5_f64)
        .clamp(0.15, 0.85)
}

pub fn free_multimodal_tri_mix_weights() -> [f64; 3] {
    let llm = std::env::var("VALHALLA_FREE_MULTIMODAL_MIX_LLM")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.40);
    let tts = std::env::var("VALHALLA_FREE_MULTIMODAL_MIX_TTS")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.30);
    let img = std::env::var("VALHALLA_FREE_MULTIMODAL_MIX_IMG")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.30);
    let sum = (llm + tts + img).max(1e-9);
    [llm / sum, tts / sum, img / sum]
}

pub fn mix_ports_weighted(ports: &[&BodyFrontPorts], weights: &[f64]) -> BodyFrontPorts {
    let dim = ports
        .iter()
        .map(|p| p.embed_dim)
        .max()
        .unwrap_or(1)
        .max(1);
    let mut emb = vec![0.0; dim];
    for (p, w) in ports.iter().zip(weights.iter()) {
        for i in 0..dim {
            let v = p.embedding.get(i % p.embedding.len()).copied().unwrap_or(0.0);
            emb[i] += v * w;
        }
    }
    normalize_vec(&mut emb);
    let labels: Vec<&str> = ports.iter().map(|p| p.modality_label.as_str()).collect();
    let mut mixed = build_body_ports(
        &emb,
        &DlFrontConfig::default(),
        "free_fusion/tri_mix",
        &labels.join("+"),
    );
    mixed.modality_label = "free_trimodal".into();
    mixed
}

fn normalize_vec(v: &mut [f64]) {
    let n: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-9);
    for x in v.iter_mut() {
        *x /= n;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentFusionOutcome {
    pub report: TriadFeedReport,
    pub protocol: String,
    pub samples_fed: usize,
    pub rounds_per_sample: usize,
    pub closure_rounds: usize,
    pub pulse_cycles: usize,
    pub deep_fusion: bool,
    pub mix_alpha: f64,
    pub aggregate_score: f64,
    #[serde(default)]
    pub tri_modal: bool,
    #[serde(default)]
    pub mix_weights: Vec<f64>,
}

fn install_free_rules(session: &mut TriadSession) {
    for rule in FREE_FUSION_RULES {
        session.hub.triple_fate.store_string(rule);
        session.tile.follow_global_fate(&session.hub.triple_fate);
        session.stem.follow_global_fate(&session.hub.triple_fate);
    }
}

fn cosine(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len().min(b.len());
    if n == 0 {
        return 0.0;
    }
    let mut dot = 0.0;
    let mut na = 0.0;
    let mut nb = 0.0;
    for i in 0..n {
        dot += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }
    let d = na.sqrt() * nb.sqrt();
    if d < 1e-12 {
        0.0
    } else {
        dot / d
    }
}

fn mix_ports(a: &BodyFrontPorts, b: &BodyFrontPorts) -> BodyFrontPorts {
    let alpha = free_multimodal_mix_alpha();
    let beta = 1.0 - alpha;
    let dim = a.embed_dim.max(b.embed_dim).max(1);
    let mut emb = vec![0.0; dim];
    for i in 0..dim {
        let av = a.embedding.get(i % a.embedding.len()).copied().unwrap_or(0.0);
        let bv = b.embedding.get(i % b.embedding.len()).copied().unwrap_or(0.0);
        emb[i] = av * alpha + bv * beta;
    }
    let n: f64 = emb.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-9);
    for x in emb.iter_mut() {
        *x /= n;
    }
    let mut mixed = build_body_ports(&emb, &DlFrontConfig::default(), "free_fusion/mix", "text+audio");
    mixed.encoder = format!("free_fusion/{}+{}", a.encoder, b.encoder);
    mixed.modality_label = "free_multimodal".into();
    mixed
}

/// Pulse ports into **all** bodies — no Stem=text / Tile=audio preset.
pub fn free_pulse(session: &mut TriadSession, ports: &BodyFrontPorts) {
    let pulse_cycles = free_multimodal_optimize_cycles_per_pulse();
    for target in [
        BodyTarget::Hub,
        BodyTarget::Tile,
        BodyTarget::StemCell,
    ] {
        session.emit_ports_target(ports, target);
    }
    session.run_optimize_cycles_target(pulse_cycles, BodyTarget::Triad);
    if session.hub.is_data_flowing() {
        session.hub.mark_epoch_complete();
    }
}

/// One corpus line: text+audio free fusion without resetting Triad (persistent incubation).
fn fuse_sample_persistent(
    session: &mut TriadSession,
    llm_ports: &BodyFrontPorts,
    tts_ports: &BodyFrontPorts,
    rounds: usize,
) {
    let mixed = mix_ports(llm_ports, tts_ports);
    for r in 0..rounds {
        free_pulse(session, llm_ports);
        free_pulse(session, tts_ports);
        free_pulse(session, &mixed);
        session.emit_body_ports(&mixed);
        let _ = session.hub.triple_fate.store_string(&format!(
            "persistent_fusion_round={r}"
        ));
    }
}

/// Feed all LLM+TTS pairs into one Triad session → export fused structure for bake (v2).
pub fn run_persistent_corpus_fusion(
    pairs: &[(BodyFrontPorts, BodyFrontPorts)],
    mode_label: &str,
) -> PersistentFusionOutcome {
    let budget = IngressBudget::incubation();
    let mut session = TriadSession::with_body(budget, BodyTarget::Triad);
    install_free_rules(&mut session);

    let rounds = free_multimodal_rounds_per_sample();
    let closure = free_multimodal_closure_rounds();
    let pulse_cycles = free_multimodal_optimize_cycles_per_pulse();
    let alpha = free_multimodal_mix_alpha();

    for (llm, tts) in pairs {
        fuse_sample_persistent(&mut session, llm, tts, rounds);
    }

    let mut last_mixed = pairs.last().map(|(l, t)| mix_ports(l, t));
    for _ in 0..closure {
        if let Some(ref mixed) = last_mixed {
            free_pulse(&mut session, mixed);
            session.emit_body_ports(mixed);
        }
        session.run_optimize_cycles_target(pulse_cycles, BodyTarget::Triad);
        mark_epoch_if_hub(&mut session);
    }

    let aggregate = compute_aggregate(&session);
    let mut report = session.build_feed_report(mode_label, closure);
    report.optimize_cycles = pairs.len() * rounds * 3 + closure;

    PersistentFusionOutcome {
        report,
        protocol: FREE_MULTIMODAL_FUSION_V2.into(),
        samples_fed: pairs.len(),
        rounds_per_sample: rounds,
        closure_rounds: closure,
        pulse_cycles,
        deep_fusion: free_multimodal_deep_enabled(),
        mix_alpha: alpha,
        aggregate_score: aggregate,
        tri_modal: false,
        mix_weights: vec![alpha, 1.0 - alpha],
    }
}

/// One corpus line: LLM + TTS + image-gen free fusion (persistent incubation, v3).
fn fuse_sample_persistent_tri(
    session: &mut TriadSession,
    llm_ports: &BodyFrontPorts,
    tts_ports: &BodyFrontPorts,
    img_ports: &BodyFrontPorts,
    rounds: usize,
    weights: &[f64; 3],
) {
    let mixed = mix_ports_weighted(&[llm_ports, tts_ports, img_ports], weights);
    for r in 0..rounds {
        free_pulse(session, llm_ports);
        free_pulse(session, tts_ports);
        free_pulse(session, img_ports);
        free_pulse(session, &mixed);
        session.emit_body_ports(&mixed);
        let _ = session.hub.triple_fate.store_string(&format!(
            "persistent_trimodal_round={r}"
        ));
    }
}

/// Feed all LLM+TTS+image-gen triples into one Triad session → export fused structure (v3).
pub fn run_persistent_corpus_fusion_tri(
    triples: &[(BodyFrontPorts, BodyFrontPorts, BodyFrontPorts)],
    mode_label: &str,
) -> PersistentFusionOutcome {
    let budget = IngressBudget::incubation();
    let mut session = TriadSession::with_body(budget, BodyTarget::Triad);
    install_free_rules(&mut session);

    let rounds = free_multimodal_rounds_per_sample();
    let closure = free_multimodal_closure_rounds();
    let pulse_cycles = free_multimodal_optimize_cycles_per_pulse();
    let weights = free_multimodal_tri_mix_weights();

    for (llm, tts, img) in triples {
        fuse_sample_persistent_tri(&mut session, llm, tts, img, rounds, &weights);
    }

    let mut last_mixed = triples
        .last()
        .map(|(l, t, i)| mix_ports_weighted(&[l, t, i], &weights));
    for _ in 0..closure {
        if let Some(ref mixed) = last_mixed {
            free_pulse(&mut session, mixed);
            session.emit_body_ports(mixed);
        }
        session.run_optimize_cycles_target(pulse_cycles, BodyTarget::Triad);
        mark_epoch_if_hub(&mut session);
    }

    let aggregate = compute_aggregate(&session);
    let mut report = session.build_feed_report(mode_label, closure);
    report.optimize_cycles = triples.len() * rounds * 4 + closure;

    PersistentFusionOutcome {
        report,
        protocol: FREE_MULTIMODAL_FUSION_V3.into(),
        samples_fed: triples.len(),
        rounds_per_sample: rounds,
        closure_rounds: closure,
        pulse_cycles,
        deep_fusion: free_multimodal_deep_enabled(),
        mix_alpha: weights[0],
        aggregate_score: aggregate,
        tri_modal: true,
        mix_weights: weights.to_vec(),
    }
}

fn mark_epoch_if_hub(session: &mut TriadSession) {
    if session.hub.is_data_flowing() {
        session.hub.mark_epoch_complete();
    }
}

fn hub_pref_vec(session: &TriadSession) -> Vec<f64> {
    session.hub.get_fate_preferences().affinities
}

fn compute_aggregate(session: &TriadSession) -> f64 {
    let prefs = hub_pref_vec(session);
    let sum: f64 = prefs.iter().sum();
    let spread = if sum > 1e-9 {
        prefs.iter().copied().fold(0.0f64, f64::max) / sum
    } else {
        0.25
    };
    let tile_n = session.tile.completed_tiles.len();
    let stem_n = session.stem.cells.len();
    let structure = ((tile_n as f64 * 0.04) + (stem_n as f64 * 0.02)).min(0.5);
    (spread * 0.45 + structure * 0.55).clamp(0.0, 1.0)
}

struct FateTracker {
    history: Vec<f64>,
    stable_streak: usize,
}

impl FateTracker {
    fn new() -> Self {
        Self {
            history: Vec::new(),
            stable_streak: 0,
        }
    }

    fn record(&mut self, v: f64) {
        self.history.push(v);
        if self.history.len() >= 3 {
            let n = self.history.len();
            let d0 = (self.history[n - 1] - self.history[n - 2]).abs();
            let d1 = (self.history[n - 2] - self.history[n - 3]).abs();
            if d0 < 0.012 && d1 < 0.012 {
                self.stable_streak += 1;
            } else if d0 > 0.025 {
                self.stable_streak = 0;
            }
        }
    }

    fn marginal_delta(&self) -> f64 {
        let n = self.history.len();
        if n < 2 {
            return 1.0;
        }
        (self.history[n - 1] - self.history[n - 2]).abs()
    }
}

fn snapshot(
    session: &TriadSession,
    cycle: usize,
    phase: &str,
    aggregate: f64,
    tracker: &FateTracker,
) -> McqFateCycleSnapshot {
    McqFateCycleSnapshot {
        cycle,
        phase: phase.to_string(),
        decompose_score: aggregate,
        aggregate_score: aggregate,
        tile_cells: session.tile.completed_tiles.len(),
        stem_cells: session.stem.cells.len(),
        hub_converged: session.hub.has_fate_converged(),
        tile_converged: session.tile.has_fate_converged(),
        stem_converged: session.stem.has_fate_converged(),
        fate_rules_active: FREE_FUSION_RULES.iter().map(|s| s.to_string()).collect(),
        fate_marginal_delta: tracker.marginal_delta(),
        stable_streak: tracker.stable_streak,
        fate_closure_score: aggregate,
        hub_pattern: session
            .hub
            .get_fate_convergence_pattern()
            .unwrap_or("unknown")
            .to_string(),
    }
}

fn finalize_baseline(
    budget: &crate::signal_ingress::IngressBudget,
    front: &DlFrontConfig,
    feeds: &[ModalityInput],
    cycles: usize,
) -> TriadFeedReport {
    let mut session = TriadSession::with_body(budget.clone(), BodyTarget::Triad);
    for f in feeds {
        session.feed_modality(f.clone(), front);
    }
    session.finalize_with_cycles("baseline", cycles)
}

fn compute_emergence(
    text_r: &TriadFeedReport,
    audio_r: &TriadFeedReport,
    fused_r: &TriadFeedReport,
    hub_sep: f64,
    binding: f64,
) -> EmergenceMetrics {
    let text_p = &text_r.model_patch.patch_vector;
    let audio_p = &audio_r.model_patch.patch_vector;
    let fused_p = &fused_r.model_patch.patch_vector;

    let tile_super = fused_r.tile_completed > text_r.tile_completed.max(audio_r.tile_completed);
    let stem_super = fused_r.stem_cells > text_r.stem_cells.max(audio_r.stem_cells);

    let patch_ta = cosine(text_p, audio_p);
    let fusion_text = cosine(fused_p, text_p);
    let fusion_audio = cosine(fused_p, audio_p);

    let super_score = if tile_super { 0.25 } else { 0.0 } + if stem_super { 0.25 } else { 0.0 };
    let bind_score = binding.clamp(0.0, 1.0) * 0.25;
    let sep_score = hub_sep.clamp(0.0, 1.0) * 0.15;
    let complement = (1.0 - patch_ta.abs()).clamp(0.0, 1.0) * 0.10;
    let natural = (super_score + bind_score + sep_score + complement).clamp(0.0, 1.0);

    let emergence_likely = tile_super
        && stem_super
        && binding > 0.005
        && hub_sep > 0.02
        && natural >= 0.35;

    EmergenceMetrics {
        text_only_tiles: text_r.tile_completed,
        audio_only_tiles: audio_r.tile_completed,
        fused_tiles: fused_r.tile_completed,
        text_only_stem: text_r.stem_cells,
        audio_only_stem: audio_r.stem_cells,
        fused_stem: fused_r.stem_cells,
        tile_superadditive: tile_super,
        stem_superadditive: stem_super,
        patch_text_audio_cosine: patch_ta,
        fusion_vs_text_cosine: fusion_text,
        fusion_vs_audio_cosine: fusion_audio,
        cross_modal_binding: binding,
        hub_modality_separation: hub_sep,
        natural_multimodal_score: natural,
        emergence_likely,
    }
}

/// LLM text + TTS audio → free body pulses → Fate multi-round adapt → emergence metrics.
pub fn run_free_multimodal_fate(
    session: &mut TriadSession,
    llm_ports: &BodyFrontPorts,
    tts_ports: &BodyFrontPorts,
    mode_label: &str,
    mismatch_fusion_patch: Option<&[f64]>,
) -> FreeMultimodalOutcome {
    install_free_rules(session);

    let budget = session.budget.clone();
    let front = DlFrontConfig::default();
    let min_c = free_multimodal_min_cycles();
    let soft_max = free_multimodal_max_cycles();
    let hard_max = free_multimodal_hard_max();

    // Baselines for emergence comparison (separate sessions — discovery metrics only).
    let text_r = finalize_baseline(
        &budget,
        &front,
        &[ModalityInput::Embedding {
            values: llm_ports.embedding.clone(),
        }],
        2,
    );
    let audio_r = finalize_baseline(
        &budget,
        &front,
        &[ModalityInput::AudioFeatures {
            values: tts_ports.embedding.clone(),
        }],
        2,
    );

    let mixed = mix_ports(llm_ports, tts_ports);
    let mut trace = Vec::new();
    let mut tracker = FateTracker::new();
    let mut adaptation_rounds = 0usize;
    let mut stopped_reason = "hard_max".to_string();
    let mut hub_sep_accum = 0.0f64;
    let mut hub_sep_n = 0usize;
    let mut cycle = 0usize;

    let mut prev_hub = hub_pref_vec(session);

    while adaptation_rounds < hard_max {
        free_pulse(session, llm_ports);
        cycle += 1;
        let hub_after_text = hub_pref_vec(session);
        hub_sep_accum += cosine(&hub_after_text, &prev_hub).abs();
        hub_sep_n += 1;
        prev_hub = hub_after_text.clone();

        free_pulse(session, tts_ports);
        cycle += 1;
        let hub_after_audio = hub_pref_vec(session);
        hub_sep_accum += (1.0 - cosine(&hub_after_audio, &hub_after_text)).max(0.0);
        hub_sep_n += 1;
        prev_hub = hub_after_audio;

        free_pulse(session, &mixed);
        session.emit_body_ports(&mixed);
        cycle += 1;
        adaptation_rounds += 1;

        let aggregate = compute_aggregate(session);
        tracker.record(aggregate);
        trace.push(snapshot(session, cycle, "free_multimodal_adapt", aggregate, &tracker));

        let _ = session
            .hub
            .triple_fate
            .store_string(&format!("free_fusion_round={adaptation_rounds} score={aggregate:.4}"));

        if adaptation_rounds >= min_c {
            let exploring = tracker.marginal_delta() > 0.03 && tracker.stable_streak == 0;
            if !exploring && tracker.stable_streak >= 3 {
                stopped_reason = "fate_marginal_stable".into();
                break;
            }
            if session.hub.has_fate_converged()
                && (session.tile.has_fate_converged() || session.stem.has_fate_converged())
                && !exploring
            {
                stopped_reason = "fate_converged".into();
                break;
            }
            if aggregate >= 0.82 && !exploring {
                stopped_reason = "cross_modal_closure".into();
                break;
            }
        }
        if adaptation_rounds >= soft_max {
            stopped_reason = "soft_max".into();
            break;
        }
    }

    let mut report = session.build_feed_report(mode_label, 0);
    report.optimize_cycles = cycle;
    report.mcq_fate_trace = trace.clone();

    let fused_patch = report.model_patch.patch_vector.clone();
    let binding = if let Some(mismatch) = mismatch_fusion_patch {
        cosine(&fused_patch, mismatch).max(0.0)
    } else {
        0.05
    };

    let hub_sep = if hub_sep_n > 0 {
        hub_sep_accum / hub_sep_n as f64
    } else {
        0.0
    };

    let cross_modal_closure = compute_aggregate(session);
    let emergence = compute_emergence(&text_r, &audio_r, &report, hub_sep, binding);

    FreeMultimodalOutcome {
        report,
        trace,
        adaptation_rounds,
        stopped_reason,
        aggregate_score: cross_modal_closure,
        cross_modal_closure,
        emergence,
    }
}

/// LLM + TTS + image-gen → extend dual free fusion with image pulses each adapt round.
pub fn run_free_trimodal_fate(
    session: &mut TriadSession,
    llm_ports: &BodyFrontPorts,
    tts_ports: &BodyFrontPorts,
    img_ports: &BodyFrontPorts,
    mode_label: &str,
) -> FreeMultimodalOutcome {
    let weights = free_multimodal_tri_mix_weights();
    let mixed = mix_ports_weighted(&[llm_ports, tts_ports, img_ports], &weights);
    let mut outcome = run_free_multimodal_fate(session, llm_ports, tts_ports, mode_label, None);
    free_pulse(session, img_ports);
    free_pulse(session, &mixed);
    session.emit_body_ports(&mixed);
    let binding_boost = cosine(
        &outcome.report.model_patch.patch_vector,
        &img_ports.embedding,
    )
    .abs()
    .min(0.2);
    outcome.emergence.cross_modal_binding =
        (outcome.emergence.cross_modal_binding + binding_boost).min(1.0);
    outcome.emergence.natural_multimodal_score =
        (outcome.emergence.natural_multimodal_score + binding_boost * 0.5).min(1.0);
    outcome
}

// ── Universal core: same task via text / TTS / image ─────────────────────

pub const UNIVERSAL_CORE_PROTOCOL: &str = "valhalla-universal-core-v1";
pub const UNIVERSAL_CORE_NON_ISOLATED: &str = "valhalla-universal-core-v1-non-isolated";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum UniversalCoreLayout {
    Isolated,
    NonIsolated,
    #[default]
    Compare,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModalityIsolationOutcome {
    pub mode: String,
    pub patch_vector: Vec<f64>,
    pub hub_prefs: Vec<f64>,
    pub tile_count: usize,
    pub stem_count: usize,
    pub signals_emitted: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalCoreMetrics {
    /// Same task, three isolated runs — avg pairwise patch cosine (shared core?)
    pub shared_patch_pairwise_avg: f64,
    pub patch_text_audio: f64,
    pub patch_text_image: f64,
    pub patch_audio_image: f64,
    pub shared_hub_pairwise_avg: f64,
    pub shared_embedding_pairwise_avg: f64,
    pub shares_common_core: bool,
    /// Centroid of isolated patches = pre-fusion universal core candidate
    pub modality_centroid: Vec<f64>,
    /// After tri-modal fusion
    pub fusion_vs_centroid_cosine: f64,
    pub fusion_vs_text: f64,
    pub fusion_vs_audio: f64,
    pub fusion_vs_image: f64,
    /// fusion alignment − pre-fusion shared (positive ⇒ fusion tightened core)
    pub core_tightening: f64,
    pub universal_core_score: f64,
    pub universal_core_formed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SameTaskUniversalCoreOutcome {
    pub task_id: String,
    pub text_isolation: ModalityIsolationOutcome,
    pub audio_isolation: ModalityIsolationOutcome,
    pub image_isolation: ModalityIsolationOutcome,
    pub metrics: UniversalCoreMetrics,
    pub fusion_adaptation_rounds: usize,
    pub fusion_aggregate_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SameTaskUniversalCoreNonIsolatedOutcome {
    pub task_id: String,
    pub layout: String,
    /// In-session snapshot after each modality pulse (same session, cumulative)
    pub text_track: ModalityIsolationOutcome,
    pub audio_track: ModalityIsolationOutcome,
    pub image_track: ModalityIsolationOutcome,
    pub metrics: UniversalCoreMetrics,
    pub fusion_adaptation_rounds: usize,
    pub fusion_aggregate_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalCoreCompareOutcome {
    pub task_id: String,
    pub isolated: SameTaskUniversalCoreOutcome,
    pub non_isolated: SameTaskUniversalCoreNonIsolatedOutcome,
    pub shared_patch_delta: f64,
    pub fusion_centroid_delta: f64,
    pub universal_score_delta: f64,
    pub non_isolated_better: bool,
}

pub fn universal_core_isolation_rounds() -> usize {
    env_usize("VALHALLA_UNIVERSAL_CORE_ISOLATION_ROUNDS", 6).clamp(2, 32)
}

fn report_to_isolation(r: &TriadFeedReport, mode: &str) -> ModalityIsolationOutcome {
    ModalityIsolationOutcome {
        mode: mode.to_string(),
        patch_vector: r.model_patch.patch_vector.clone(),
        hub_prefs: r.hub_prefs.clone(),
        tile_count: r.tile_completed,
        stem_count: r.stem_cells,
        signals_emitted: r.stats.signals_emitted,
    }
}

/// One modality completes the same task in isolation (free pulse, no preset routing).
fn run_modality_isolated_task(ports: &BodyFrontPorts, mode: &str) -> TriadFeedReport {
    let budget = IngressBudget::incubation();
    let mut session = TriadSession::with_body(budget, BodyTarget::Triad);
    install_free_rules(&mut session);
    let rounds = universal_core_isolation_rounds();
    for r in 0..rounds {
        free_pulse(&mut session, ports);
        let _ = session
            .hub
            .triple_fate
            .store_string(&format!("{mode}_isolation_round={r}"));
    }
    session.build_feed_report(mode, rounds)
}

fn patch_centroid(patches: &[Vec<f64>]) -> Vec<f64> {
    if patches.is_empty() {
        return Vec::new();
    }
    let dim = patches.iter().map(|p| p.len()).max().unwrap_or(0);
    let mut c = vec![0.0f64; dim];
    for p in patches {
        for (i, slot) in c.iter_mut().enumerate().take(dim) {
            *slot += p.get(i).copied().unwrap_or(0.0);
        }
    }
    let n = patches.len() as f64;
    for x in c.iter_mut() {
        *x /= n;
    }
    normalize_vec(&mut c);
    c
}

fn pairwise_avg(values: &[Vec<f64>]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let mut sum = 0.0f64;
    let mut n = 0usize;
    for i in 0..values.len() {
        for j in (i + 1)..values.len() {
            sum += cosine(&values[i], &values[j]);
            n += 1;
        }
    }
    if n == 0 {
        0.0
    } else {
        sum / n as f64
    }
}

fn tri_cos(a: &[f64], b: &[f64], c: &[f64]) -> (f64, f64, f64) {
    (cosine(a, b), cosine(a, c), cosine(b, c))
}

fn snapshot_patch(session: &mut TriadSession, mode: &str) -> ModalityIsolationOutcome {
    let r = session.build_feed_report(mode, 0);
    report_to_isolation(&r, mode)
}

fn compute_universal_core_metrics(
    patches: &[Vec<f64>],
    hub_vecs: &[Vec<f64>],
    emb_vecs: &[Vec<f64>],
    fused: &[f64],
) -> UniversalCoreMetrics {
    let (patch_ta, patch_ti, patch_ai) = tri_cos(&patches[0], &patches[1], &patches[2]);
    let shared_patch = (patch_ta + patch_ti + patch_ai) / 3.0;
    let shared_hub = pairwise_avg(hub_vecs);
    let shared_emb = pairwise_avg(emb_vecs);
    let centroid = patch_centroid(patches);
    let fusion_vs_centroid = cosine(fused, &centroid);
    let fusion_vs_text = cosine(fused, &patches[0]);
    let fusion_vs_audio = cosine(fused, &patches[1]);
    let fusion_vs_image = cosine(fused, &patches[2]);
    let core_tightening = fusion_vs_centroid - shared_patch;
    let shares_common_core =
        shared_patch >= 0.40 || (shared_patch >= 0.20 && shared_hub >= 0.94);
    let universal_core_score = (shared_patch * 0.40
        + fusion_vs_centroid * 0.35
        + shared_hub * 0.15
        + shared_emb * 0.10)
        .clamp(0.0, 1.0);
    let universal_core_formed = universal_core_score >= 0.55
        && fusion_vs_centroid >= shared_patch * 0.92
        && core_tightening > -0.08;
    UniversalCoreMetrics {
        shared_patch_pairwise_avg: shared_patch,
        patch_text_audio: patch_ta,
        patch_text_image: patch_ti,
        patch_audio_image: patch_ai,
        shared_hub_pairwise_avg: shared_hub,
        shared_embedding_pairwise_avg: shared_emb,
        shares_common_core,
        modality_centroid: centroid,
        fusion_vs_centroid_cosine: fusion_vs_centroid,
        fusion_vs_text,
        fusion_vs_audio,
        fusion_vs_image,
        core_tightening,
        universal_core_score,
        universal_core_formed,
    }
}

/// Same task → text-only / TTS-only / image-only isolation → fusion → universal core metrics.
pub fn run_same_task_universal_core(
    task_id: &str,
    llm_ports: &BodyFrontPorts,
    tts_ports: &BodyFrontPorts,
    img_ports: &BodyFrontPorts,
) -> SameTaskUniversalCoreOutcome {
    let text_r = run_modality_isolated_task(llm_ports, "task_via_text");
    let audio_r = run_modality_isolated_task(tts_ports, "task_via_tts");
    let image_r = run_modality_isolated_task(img_ports, "task_via_image");

    let text_iso = report_to_isolation(&text_r, "task_via_text");
    let audio_iso = report_to_isolation(&audio_r, "task_via_tts");
    let image_iso = report_to_isolation(&image_r, "task_via_image");

    let patches = vec![
        text_iso.patch_vector.clone(),
        audio_iso.patch_vector.clone(),
        image_iso.patch_vector.clone(),
    ];
    let hub_vecs = vec![
        text_iso.hub_prefs.clone(),
        audio_iso.hub_prefs.clone(),
        image_iso.hub_prefs.clone(),
    ];
    let emb_vecs = vec![
        llm_ports.embedding.clone(),
        tts_ports.embedding.clone(),
        img_ports.embedding.clone(),
    ];

    let budget = IngressBudget::incubation();
    let mut fusion_session = TriadSession::with_body(budget, BodyTarget::Triad);
    let fusion = run_free_trimodal_fate(
        &mut fusion_session,
        llm_ports,
        tts_ports,
        img_ports,
        "universal_core_fusion",
    );
    let fused = fusion.report.model_patch.patch_vector.clone();
    let metrics = compute_universal_core_metrics(&patches, &hub_vecs, &emb_vecs, &fused);

    SameTaskUniversalCoreOutcome {
        task_id: task_id.to_string(),
        text_isolation: text_iso,
        audio_isolation: audio_iso,
        image_isolation: image_iso,
        metrics,
        fusion_adaptation_rounds: fusion.adaptation_rounds,
        fusion_aggregate_score: fusion.aggregate_score,
    }
}

/// Fuse task modalities into an existing incubation snapshot (context + core in one session).
pub fn fuse_universal_core_on_snapshot(
    snap: TriadIncubationSnapshot,
    body: BodyTarget,
    llm_ports: &BodyFrontPorts,
    tts_ports: &BodyFrontPorts,
    img_ports: &BodyFrontPorts,
) -> (TriadIncubationSnapshot, UniversalCoreMetrics) {
    let budget = IngressBudget::incubation();
    let mut session = TriadSession::from_incubation_snapshot(budget, body, snap);
    install_free_rules(&mut session);

    let rounds = universal_core_isolation_rounds();
    let weights = free_multimodal_tri_mix_weights();
    let mixed = mix_ports_weighted(&[llm_ports, tts_ports, img_ports], &weights);

    let mut text_track = snapshot_patch(&mut session, "ctx_core_init");
    let mut audio_track = text_track.clone();
    let mut image_track = text_track.clone();

    let adapt_rounds = free_multimodal_min_cycles().max(rounds);
    for r in 0..adapt_rounds {
        free_pulse(&mut session, llm_ports);
        text_track = snapshot_patch(&mut session, "ctx_core_after_text");
        free_pulse(&mut session, tts_ports);
        audio_track = snapshot_patch(&mut session, "ctx_core_after_tts");
        free_pulse(&mut session, img_ports);
        image_track = snapshot_patch(&mut session, "ctx_core_after_image");
        free_pulse(&mut session, &mixed);
        session.emit_body_ports(&mixed);
        let _ = session
            .hub
            .triple_fate
            .store_string(&format!("ctx_core_round={r}"));
    }

    let final_report = session.build_feed_report("ctx_core_final", adapt_rounds);
    let fused = final_report.model_patch.patch_vector.clone();
    let patches = vec![
        text_track.patch_vector.clone(),
        audio_track.patch_vector.clone(),
        image_track.patch_vector.clone(),
    ];
    let hub_vecs = vec![
        text_track.hub_prefs.clone(),
        audio_track.hub_prefs.clone(),
        image_track.hub_prefs.clone(),
    ];
    let emb_vecs = vec![
        llm_ports.embedding.clone(),
        tts_ports.embedding.clone(),
        img_ports.embedding.clone(),
    ];
    let metrics = compute_universal_core_metrics(&patches, &hub_vecs, &emb_vecs, &fused);
    (session.incubation_snapshot(), metrics)
}

/// Same task — text/TTS/image **in one session** (no isolation), interleaved pulses.
pub fn run_same_task_universal_core_non_isolated(
    task_id: &str,
    llm_ports: &BodyFrontPorts,
    tts_ports: &BodyFrontPorts,
    img_ports: &BodyFrontPorts,
) -> SameTaskUniversalCoreNonIsolatedOutcome {
    let budget = IngressBudget::incubation();
    let mut session = TriadSession::with_body(budget, BodyTarget::Triad);
    install_free_rules(&mut session);

    let rounds = universal_core_isolation_rounds();
    let weights = free_multimodal_tri_mix_weights();
    let mixed = mix_ports_weighted(
        &[llm_ports, tts_ports, img_ports],
        &weights,
    );

    let mut text_track = snapshot_patch(&mut session, "non_iso_init");
    let mut audio_track = text_track.clone();
    let mut image_track = text_track.clone();

    let adapt_rounds = free_multimodal_min_cycles().max(rounds);
    for r in 0..adapt_rounds {
        free_pulse(&mut session, llm_ports);
        text_track = snapshot_patch(&mut session, "non_iso_after_text");
        free_pulse(&mut session, tts_ports);
        audio_track = snapshot_patch(&mut session, "non_iso_after_tts");
        free_pulse(&mut session, img_ports);
        image_track = snapshot_patch(&mut session, "non_iso_after_image");
        free_pulse(&mut session, &mixed);
        session.emit_body_ports(&mixed);
        let _ = session
            .hub
            .triple_fate
            .store_string(&format!("non_isolated_round={r}"));
    }

    let final_report = session.build_feed_report("non_isolated_final", adapt_rounds);
    let fused = final_report.model_patch.patch_vector.clone();
    let aggregate = compute_aggregate(&session);

    text_track.mode = "non_iso_track_text".into();
    audio_track.mode = "non_iso_track_tts".into();
    image_track.mode = "non_iso_track_image".into();

    let patches = vec![
        text_track.patch_vector.clone(),
        audio_track.patch_vector.clone(),
        image_track.patch_vector.clone(),
    ];
    let hub_vecs = vec![
        text_track.hub_prefs.clone(),
        audio_track.hub_prefs.clone(),
        image_track.hub_prefs.clone(),
    ];
    let emb_vecs = vec![
        llm_ports.embedding.clone(),
        tts_ports.embedding.clone(),
        img_ports.embedding.clone(),
    ];
    let metrics = compute_universal_core_metrics(&patches, &hub_vecs, &emb_vecs, &fused);

    SameTaskUniversalCoreNonIsolatedOutcome {
        task_id: task_id.to_string(),
        layout: UNIVERSAL_CORE_NON_ISOLATED.to_string(),
        text_track,
        audio_track,
        image_track,
        metrics,
        fusion_adaptation_rounds: adapt_rounds,
        fusion_aggregate_score: aggregate,
    }
}

pub fn run_same_task_universal_core_compare(
    task_id: &str,
    llm_ports: &BodyFrontPorts,
    tts_ports: &BodyFrontPorts,
    img_ports: &BodyFrontPorts,
) -> UniversalCoreCompareOutcome {
    let isolated = run_same_task_universal_core(task_id, llm_ports, tts_ports, img_ports);
    let non_isolated =
        run_same_task_universal_core_non_isolated(task_id, llm_ports, tts_ports, img_ports);
    let shared_patch_delta =
        non_isolated.metrics.shared_patch_pairwise_avg - isolated.metrics.shared_patch_pairwise_avg;
    let fusion_centroid_delta = non_isolated.metrics.fusion_vs_centroid_cosine
        - isolated.metrics.fusion_vs_centroid_cosine;
    let universal_score_delta =
        non_isolated.metrics.universal_core_score - isolated.metrics.universal_core_score;
    UniversalCoreCompareOutcome {
        task_id: task_id.to_string(),
        isolated,
        non_isolated,
        shared_patch_delta,
        fusion_centroid_delta,
        universal_score_delta,
        non_isolated_better: universal_score_delta > 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signal_ingress::IngressBudget;

    fn dummy_ports(seed: f64) -> BodyFrontPorts {
        let emb: Vec<f64> = (0..32).map(|i| ((i as f64 * 0.07 + seed).sin()).abs()).collect();
        build_body_ports(&emb, &DlFrontConfig::default(), "test/mod", "test")
    }

    #[test]
    fn free_multimodal_runs_adaptation() {
        std::env::set_var("VALHALLA_FREE_MULTIMODAL_MIN_CYCLES", "2");
        let budget = IngressBudget::experiment();
        let mut session = TriadSession::with_body(budget, BodyTarget::Triad);
        let llm = dummy_ports(0.2);
        let tts = dummy_ports(0.8);
        let out = run_free_multimodal_fate(&mut session, &llm, &tts, "test_free", None);
        assert!(out.adaptation_rounds >= 2);
        assert!(!out.trace.is_empty());
    }

    #[test]
    fn persistent_corpus_fusion_accumulates_structure() {
        std::env::set_var("VALHALLA_FREE_MULTIMODAL_DEEP", "1");
        std::env::set_var("VALHALLA_FREE_MULTIMODAL_ROUNDS_PER_SAMPLE", "2");
        std::env::set_var("VALHALLA_FREE_MULTIMODAL_CLOSURE_ROUNDS", "3");
        let pairs: Vec<_> = (0..4)
            .map(|i| {
                let s = i as f64 * 0.11;
                (dummy_ports(s), dummy_ports(s + 0.5))
            })
            .collect();
        let out = run_persistent_corpus_fusion(&pairs, "test_persistent");
        assert_eq!(out.samples_fed, 4);
        assert!(out.report.tile_completed > 0);
        assert!(out.report.stem_cells > 0);
        assert!(out.aggregate_score > 0.0);
    }

    #[test]
    fn persistent_trimodal_fusion_accumulates_structure() {
        std::env::set_var("VALHALLA_FREE_MULTIMODAL_DEEP", "1");
        std::env::set_var("VALHALLA_FREE_MULTIMODAL_ROUNDS_PER_SAMPLE", "2");
        std::env::set_var("VALHALLA_FREE_MULTIMODAL_CLOSURE_ROUNDS", "3");
        let triples: Vec<_> = (0..3)
            .map(|i| {
                let s = i as f64 * 0.13;
                (
                    dummy_ports(s),
                    dummy_ports(s + 0.4),
                    dummy_ports(s + 0.8),
                )
            })
            .collect();
        let out = run_persistent_corpus_fusion_tri(&triples, "test_trimodal");
        assert_eq!(out.samples_fed, 3);
        assert!(out.tri_modal);
        assert_eq!(out.mix_weights.len(), 3);
        assert!(out.report.tile_completed > 0);
    }

    #[test]
    fn same_task_universal_core_runs() {
        std::env::set_var("VALHALLA_UNIVERSAL_CORE_ISOLATION_ROUNDS", "3");
        std::env::set_var("VALHALLA_FREE_MULTIMODAL_MIN_CYCLES", "2");
        let llm = dummy_ports(0.1);
        let tts = dummy_ports(0.5);
        let img = dummy_ports(0.9);
        let out = run_same_task_universal_core("t1", &llm, &tts, &img);
        assert!(!out.text_isolation.patch_vector.is_empty());
        assert!(out.metrics.universal_core_score >= 0.0);
    }
}
