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
use crate::signal_ingress::{BodyTarget, IngressBudget, TriadFeedReport, TriadSession};

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
        session.tile.triple_fate.store_string(rule);
        session.stem.triple_fate.store_string(rule);
        session.hub.triple_fate.store_string(rule);
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
}
