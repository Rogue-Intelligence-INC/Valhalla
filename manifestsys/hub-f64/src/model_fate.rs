//! Model-level decompose → Fate multi-cycle adapt → recompose patch.
//!
//! Qwen (or any LM) weights are shredded into Hub/Tile/Stem ports (Python
//! `shred_qwen_weights.py`), ingested here, adapted by Fate, then exported as
//! `ValhallaModelPatch` for weight recompose — **not** text/TTS shard decompose.

use serde::{Deserialize, Serialize};

use crate::dl_front::BodyFrontPorts;
use crate::mcq_fate::McqFateCycleSnapshot;
use crate::signal_ingress::{BodyTarget, TriadFeedReport, TriadSession};

pub const MODEL_DECOMPOSE_PROTOCOL: &str = "valhalla-model-decompose-v1";

const MODEL_FATE_RULES: &[&str] = &[
    "task_tag:model_decompose",
    "model_decompose: LM layer quads → Hub/Tile/Stem weight signatures",
    "model_adapt: Fate multi-cycle until structure closure or marginal stable",
    "model_recompose: adapted Triad patch → LM weight surgery",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelShardPhase {
    Decompose,
    Aggregate,
    Recompose,
}

impl ModelShardPhase {
    fn from_str(s: &str) -> Self {
        match s {
            "aggregate" => Self::Aggregate,
            "recompose" => Self::Recompose,
            _ => Self::Decompose,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDecomposeShard {
    pub label: String,
    pub body: String,
    pub phase: String,
    pub ports: BodyFrontPorts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDecomposePlan {
    pub protocol: String,
    pub base_model: String,
    pub layer_count: usize,
    pub whole_ports: BodyFrontPorts,
    pub shards: Vec<ModelDecomposeShard>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelFateOutcome {
    pub report: TriadFeedReport,
    pub trace: Vec<McqFateCycleSnapshot>,
    pub shard_scores: Vec<f64>,
    pub cycles_executed: usize,
    pub adaptation_rounds: usize,
    pub stopped_reason: String,
    pub decompose_score: f64,
    pub aggregate_score: f64,
    pub model_closure_score: f64,
}

pub fn model_fate_cycles_enabled() -> bool {
    std::env::var("VALHALLA_MODEL_FATE_CYCLES")
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

pub fn model_fate_max_cycles() -> usize {
    env_usize("VALHALLA_MODEL_FATE_MAX_CYCLES", 48).clamp(4, 512)
}

pub fn model_fate_hard_max() -> usize {
    let hard = env_usize("VALHALLA_MODEL_FATE_HARD_MAX", 96);
    hard.max(model_fate_max_cycles()).clamp(8, 512)
}

pub fn model_fate_min_cycles() -> usize {
    env_usize("VALHALLA_MODEL_FATE_MIN_CYCLES", 5).clamp(2, 64)
}

fn parse_body(s: &str) -> BodyTarget {
    match s.to_ascii_lowercase().as_str() {
        "hub" => BodyTarget::Hub,
        "tile" => BodyTarget::Tile,
        "stem" | "stemcell" => BodyTarget::StemCell,
        _ => BodyTarget::Triad,
    }
}

fn install_model_fate_rules(session: &mut TriadSession) {
    for rule in MODEL_FATE_RULES {
        session.hub.triple_fate.store_string(rule);
        session.tile.follow_global_fate(&session.hub.triple_fate);
        session.stem.follow_global_fate(&session.hub.triple_fate);
    }
}

fn active_rules() -> Vec<String> {
    MODEL_FATE_RULES.iter().map(|s| s.to_string()).collect()
}

fn mark_epoch_if_hub(session: &mut TriadSession) {
    if session.hub.is_data_flowing() {
        session.hub.mark_epoch_complete();
    }
}

fn pulse_fate_feedback(session: &mut TriadSession, phase: &str, score: f64) {
    let intensity = score.clamp(0.05, 1.0);
    let _ = session.hub.triple_fate.store_string(&format!("model_{phase}_tick score={intensity:.4}"));
    session.observe_fate(intensity);
}

struct FateTracker {
    aggregate_history: Vec<f64>,
    stable_streak: usize,
}

impl FateTracker {
    fn new() -> Self {
        Self {
            aggregate_history: Vec::new(),
            stable_streak: 0,
        }
    }

    fn record(&mut self, aggregate: f64) {
        self.aggregate_history.push(aggregate);
        if self.aggregate_history.len() >= 3 {
            let n = self.aggregate_history.len();
            let d0 = (self.aggregate_history[n - 1] - self.aggregate_history[n - 2]).abs();
            let d1 = (self.aggregate_history[n - 2] - self.aggregate_history[n - 3]).abs();
            if d0 < 0.012 && d1 < 0.012 {
                self.stable_streak += 1;
            } else if d0 > 0.025 {
                self.stable_streak = 0;
            }
        }
    }

    fn marginal_delta(&self) -> f64 {
        let n = self.aggregate_history.len();
        if n < 2 {
            return 1.0;
        }
        (self.aggregate_history[n - 1] - self.aggregate_history[n - 2]).abs()
    }

    fn still_exploring(&self) -> bool {
        self.marginal_delta() > 0.03 && self.stable_streak == 0
    }
}

fn compute_aggregate_score(session: &TriadSession) -> f64 {
    let prefs = session.hub.get_fate_preferences();
    let pref_sum: f64 = prefs.affinities.iter().copied().sum();
    let pref_spread = if pref_sum > 1e-9 {
        prefs.affinities.iter().copied().fold(0.0f64, f64::max) / pref_sum
    } else {
        0.25
    };
    let acts = &session.tile.completed_tiles;
    let activation_conc = if acts.is_empty() {
        0.0
    } else {
        let energies: Vec<f64> = acts.iter().map(|t| t.energy.abs()).collect();
        let sum: f64 = energies.iter().sum();
        let max_e = energies.iter().copied().fold(0.0f64, f64::max);
        if sum > 1e-9 {
            max_e / sum
        } else {
            0.0
        }
    };
    let stem_cohesion = if session.stem.cells.is_empty() {
        0.0
    } else {
        let avg_conn: f64 = session
            .stem
            .cells
            .iter()
            .map(|c| c.connections.len() as f64)
            .sum::<f64>()
            / session.stem.cells.len() as f64;
        (avg_conn / 8.0).min(1.0)
    };
    (pref_spread * 0.4 + activation_conc * 0.35 + stem_cohesion * 0.25).clamp(0.0, 1.0)
}

fn fate_should_stop(
    session: &TriadSession,
    tracker: &FateTracker,
    cycle: usize,
    min_cycles: usize,
    aggregate_score: f64,
    decompose_score: f64,
) -> Option<&'static str> {
    if cycle < min_cycles {
        return None;
    }
    let exploring = tracker.still_exploring();
    if exploring && cycle < model_fate_max_cycles() {
        return None;
    }
    let pattern = session.hub.get_fate_convergence_pattern();
    let any_converged = session.hub.has_fate_converged()
        || session.tile.has_fate_converged()
        || session.stem.has_fate_converged();

    if matches!(pattern, Some("fixed_point")) && !exploring {
        return Some("fate_fixed_point");
    }
    if matches!(pattern, Some("converging")) && any_converged && !exploring {
        return Some("fate_converged");
    }
    if tracker.stable_streak >= 3 && !exploring {
        return Some("fate_marginal_stable");
    }
    if aggregate_score >= 0.85 && decompose_score >= 0.75 && !exploring {
        return Some("model_recompose_ready");
    }
    None
}

fn snapshot(
    session: &TriadSession,
    cycle: usize,
    phase: &str,
    decompose_score: f64,
    aggregate_score: f64,
    tracker: &FateTracker,
) -> McqFateCycleSnapshot {
    McqFateCycleSnapshot {
        cycle,
        phase: phase.to_string(),
        decompose_score,
        aggregate_score,
        tile_cells: session.tile.completed_tiles.len(),
        stem_cells: session.stem.cells.len(),
        hub_converged: session.hub.has_fate_converged(),
        tile_converged: session.tile.has_fate_converged(),
        stem_converged: session.stem.has_fate_converged(),
        fate_rules_active: active_rules(),
        fate_marginal_delta: tracker.marginal_delta(),
        stable_streak: tracker.stable_streak,
        fate_closure_score: aggregate_score,
        hub_pattern: session
            .hub
            .get_fate_convergence_pattern()
            .unwrap_or("unknown")
            .to_string(),
    }
}

fn feed_model_shard(session: &mut TriadSession, shard: &ModelDecomposeShard, cycles: usize) -> f64 {
    let body = parse_body(&shard.body);
    let tile_before = session.tile.completed_tiles.len();
    let stem_before = session.stem.cells.len();

    session.emit_ports_target(&shard.ports, body);
    session.run_optimize_cycles_target(cycles.max(1), body);

    let tile_delta = session.tile.completed_tiles.len().saturating_sub(tile_before);
    let stem_delta = session.stem.cells.len().saturating_sub(stem_before);
    let structural = ((tile_delta + stem_delta) as f64 * 0.08).min(0.5);
    (0.35 + structural + session.stats.signals_emitted as f64 * 1e-4).clamp(0.0, 1.0)
}

fn model_closure_score(shard_scores: &[f64]) -> f64 {
    if shard_scores.is_empty() {
        return 0.0;
    }
    let mean = shard_scores.iter().sum::<f64>() / shard_scores.len() as f64;
    let spread = shard_scores
        .iter()
        .map(|s| (s - mean).abs())
        .sum::<f64>()
        / shard_scores.len() as f64;
    (mean * 0.7 + (1.0 - spread) * 0.3).clamp(0.0, 1.0)
}

/// Ingest model weight shards → Fate adapt rounds → Triad recompose export.
pub fn run_model_fate_pipeline(
    session: &mut TriadSession,
    plan: &ModelDecomposePlan,
    mode_label: &str,
    cycles_per_shard: usize,
) -> ModelFateOutcome {
    install_model_fate_rules(session);
    let soft_max = model_fate_max_cycles();
    let hard_max = model_fate_hard_max();
    let min_cycles = model_fate_min_cycles();

    let mut trace = Vec::new();
    let mut tracker = FateTracker::new();
    let mut cycle = 0usize;
    let mut decompose_score = 0.0f64;
    let mut aggregate_score = 0.0f64;
    let mut stopped_reason = "hard_max".to_string();
    let mut adaptation_rounds = 0usize;
    let mut shard_scores = vec![0.0; plan.shards.len()];

    let decompose_count = plan
        .shards
        .iter()
        .filter(|s| ModelShardPhase::from_str(&s.phase) != ModelShardPhase::Recompose)
        .count()
        .max(1);
    let mut fed = 0usize;

    for (i, shard) in plan.shards.iter().enumerate() {
        let phase = ModelShardPhase::from_str(&shard.phase);
        if phase == ModelShardPhase::Recompose {
            continue;
        }
        fed += 1;
        let sc = feed_model_shard(session, shard, cycles_per_shard);
        shard_scores[i] = sc;
        cycle += 1;
        decompose_score =
            ((fed as f64 / decompose_count as f64) * 0.65 + sc * 0.35).min(1.0);
        aggregate_score = compute_aggregate_score(session);
        tracker.record(aggregate_score);
        pulse_fate_feedback(session, "decompose", decompose_score);
        trace.push(snapshot(
            session,
            cycle,
            "model_decompose",
            decompose_score,
            aggregate_score,
            &tracker,
        ));
    }

    let whole = &plan.whole_ports;
    while cycle < hard_max {
        session.emit_body_ports(whole);
        session.emit_ports_target(whole, BodyTarget::Hub);
        session.run_optimize_cycles_target(1, BodyTarget::Triad);
        mark_epoch_if_hub(session);
        cycle += 1;
        adaptation_rounds += 1;

        aggregate_score = compute_aggregate_score(session);
        tracker.record(aggregate_score);
        pulse_fate_feedback(session, "adapt", aggregate_score);
        trace.push(snapshot(
            session,
            cycle,
            "model_fate_adapt",
            decompose_score,
            aggregate_score,
            &tracker,
        ));

        if let Some(reason) = fate_should_stop(
            session,
            &tracker,
            adaptation_rounds,
            min_cycles,
            aggregate_score,
            decompose_score,
        ) {
            stopped_reason = reason.to_string();
            break;
        }
        if adaptation_rounds >= soft_max {
            stopped_reason = "model_soft_max".into();
            break;
        }
    }

    for (i, shard) in plan.shards.iter().enumerate() {
        if ModelShardPhase::from_str(&shard.phase) != ModelShardPhase::Recompose {
            continue;
        }
        let sc = feed_model_shard(session, shard, cycles_per_shard.max(2));
        shard_scores[i] = sc;
        cycle += 1;
        trace.push(snapshot(
            session,
            cycle,
            "model_recompose",
            decompose_score,
            aggregate_score,
            &tracker,
        ));
    }

    let closure = model_closure_score(&shard_scores);
    let mut report = session.build_feed_report(mode_label, 0);
    report.optimize_cycles = cycle;
    report.mcq_fate_trace = trace.clone();

    ModelFateOutcome {
        report,
        trace,
        shard_scores,
        cycles_executed: cycle,
        adaptation_rounds,
        stopped_reason,
        decompose_score,
        aggregate_score,
        model_closure_score: closure,
    }
}

pub fn load_model_decompose_plan(json: &str) -> Result<ModelDecomposePlan, serde_json::Error> {
    serde_json::from_str(json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dl_front::BodyFrontPorts;
    use crate::signal_ingress::{IngressBudget, TriadSession};

    fn dummy_ports(seed: f64) -> BodyFrontPorts {
        let v: Vec<f64> = (0..32).map(|i| ((i as f64 * 0.1 + seed).sin()).abs()).collect();
        BodyFrontPorts {
            hub: v.clone(),
            tile: v.clone(),
            stem: v,
            embedding: vec![seed; 16],
            embed_dim: 16,
            encoder: "test/model".into(),
            modality_label: "model_weight".into(),
        }
    }

    #[test]
    fn model_fate_runs_adaptation_rounds() {
        std::env::set_var("VALHALLA_MODEL_FATE_MIN_CYCLES", "2");
        let plan = ModelDecomposePlan {
            protocol: MODEL_DECOMPOSE_PROTOCOL.into(),
            base_model: "test".into(),
            layer_count: 24,
            whole_ports: dummy_ports(0.5),
            shards: vec![
                ModelDecomposeShard {
                    label: "MODEL_QUAD_0".into(),
                    body: "hub".into(),
                    phase: "decompose".into(),
                    ports: dummy_ports(0.1),
                },
                ModelDecomposeShard {
                    label: "MODEL_RECOMPOSE".into(),
                    body: "triad".into(),
                    phase: "recompose".into(),
                    ports: dummy_ports(0.9),
                },
            ],
        };
        let budget = IngressBudget::experiment();
        let mut session = TriadSession::with_body(budget, BodyTarget::Triad);
        let outcome = run_model_fate_pipeline(&mut session, &plan, "test_model_fate", 1);
        assert!(outcome.adaptation_rounds >= 2);
        assert!(!outcome.trace.is_empty());
        assert!(outcome.model_closure_score > 0.0);
    }
}
