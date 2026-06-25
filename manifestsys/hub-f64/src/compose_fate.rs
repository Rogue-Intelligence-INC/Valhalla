//! Multimodal compose — Fate multi-cycle adaptation after LLM+TTS shard decompose.
//!
//! Experiment mode (`VALHALLA_COMPOSE_FATE_CYCLES=1`):
//!   decompose shards (Stem/Tile/Hub) → Fate aggregate/adapt rounds → Triad recompose → LM patch.
//!
//! Env (reuse MCQ Fate budgets unless overridden):
//!   `VALHALLA_COMPOSE_FATE_MIN_CYCLES` (default 5)
//!   `VALHALLA_COMPOSE_FATE_MAX_CYCLES` (default 48)
//!   `VALHALLA_COMPOSE_FATE_HARD_MAX`   (default 96)

use serde::{Deserialize, Serialize};

use crate::dl_front::{BodyFrontPorts, DlFrontConfig};
use crate::mcq_fate::McqFateCycleSnapshot;
use crate::signal_ingress::{BodyTarget, TriadFeedReport, TriadSession};
use crate::task_decompose::{feed_compose_shard_pub, DecomposePlan, DecomposeShard, ShardPhase};

const COMPOSE_FATE_RULES: &[&str] = &[
    "task_tag:compose_multimodal",
    "compose_decompose: LLM text spans → Stem/Tile; TTS mel → Tile bands",
    "compose_adapt: Fate multi-cycle until hub/tile/stem closure or marginal stable",
    "compose_recompose: Triad structure → LM patch (no prompt shard lines)",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposeFateOutcome {
    pub report: TriadFeedReport,
    pub trace: Vec<McqFateCycleSnapshot>,
    pub shard_scores: Vec<f64>,
    pub cycles_executed: usize,
    pub adaptation_rounds: usize,
    pub stopped_reason: String,
    pub decompose_score: f64,
    pub aggregate_score: f64,
    pub cross_modal_score: f64,
}

pub fn compose_fate_cycles_enabled() -> bool {
    std::env::var("VALHALLA_COMPOSE_FATE_CYCLES")
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

pub fn compose_fate_max_cycles() -> usize {
    env_usize("VALHALLA_COMPOSE_FATE_MAX_CYCLES", 48).clamp(4, 512)
}

pub fn compose_fate_hard_max() -> usize {
    let hard = env_usize("VALHALLA_COMPOSE_FATE_HARD_MAX", 96);
    hard.max(compose_fate_max_cycles()).clamp(8, 512)
}

pub fn compose_fate_min_cycles() -> usize {
    env_usize("VALHALLA_COMPOSE_FATE_MIN_CYCLES", 5).clamp(2, 64)
}

fn install_compose_fate_rules(session: &mut TriadSession) {
    for rule in COMPOSE_FATE_RULES {
        session.tile.triple_fate.store_string(rule);
        session.stem.triple_fate.store_string(rule);
        session.hub.triple_fate.store_string(rule);
    }
}

fn active_rules() -> Vec<String> {
    COMPOSE_FATE_RULES.iter().map(|s| s.to_string()).collect()
}

fn mark_epoch_if_hub(session: &mut TriadSession) {
    if session.hub.is_data_flowing() {
        session.hub.mark_epoch_complete();
    }
}

fn pulse_fate_feedback(session: &mut TriadSession, phase: &str, score: f64) {
    let intensity = score.clamp(0.05, 1.0);
    let _ = session
        .hub
        .triple_fate
        .store_string(&format!("compose_{phase}_tick score={intensity:.4}"));
    session.tile.triple_fate.receive_passive(intensity * 0.5);
    session.stem.triple_fate.receive_passive(intensity * 0.5);
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
    if exploring && cycle < compose_fate_max_cycles() {
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
        return Some("compose_recompose_ready");
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

fn cross_modal_shard_balance(plan: &DecomposePlan, shard_scores: &[f64]) -> f64 {
    crate::task_decompose::cross_modal_score_for_plan(plan, shard_scores)
}

fn feed_shard(
    session: &mut TriadSession,
    shard: &DecomposeShard,
    front: &DlFrontConfig,
    cycles: usize,
) -> f64 {
    feed_compose_shard_pub(session, shard, front, cycles)
}

/// Decompose all plan shards → Fate adaptation cycles → final Triad recompose feed.
pub fn run_compose_fate_pipeline(
    session: &mut TriadSession,
    plan: &DecomposePlan,
    whole_ports: &BodyFrontPorts,
    mode_label: &str,
    front: &DlFrontConfig,
    cycles_per_shard: usize,
) -> ComposeFateOutcome {
    install_compose_fate_rules(session);
    let body = session.body;
    let soft_max = compose_fate_max_cycles();
    let hard_max = compose_fate_hard_max();
    let min_cycles = compose_fate_min_cycles();

    let mut trace = Vec::new();
    let mut tracker = FateTracker::new();
    let mut cycle = 0usize;
    let mut decompose_score = 0.0f64;
    let mut aggregate_score = 0.0f64;
    let mut stopped_reason = "hard_max".to_string();
    let mut adaptation_rounds = 0usize;

    let mut shard_scores = vec![0.0; plan.shards.len()];

    // --- Phase 1: decompose LLM + TTS shards into Stem / Tile / Hub ---
    let mut decompose_shards = 0usize;
    for (i, shard) in plan.shards.iter().enumerate() {
        if shard.phase == ShardPhase::Recompose {
            continue;
        }
        decompose_shards += 1;
        let sc = feed_shard(session, shard, front, cycles_per_shard);
        shard_scores[i] = sc;
        cycle += 1;
        decompose_score = ((decompose_shards as f64 / plan
            .shards
            .iter()
            .filter(|s| s.phase != ShardPhase::Recompose)
            .count()
            .max(1) as f64)
            * 0.65
            + sc * 0.35)
            .min(1.0);
        tracker.record(aggregate_score);
        pulse_fate_feedback(session, "decompose", decompose_score);
        trace.push(snapshot(
            session,
            cycle,
            "compose_decompose",
            decompose_score,
            aggregate_score,
            &tracker,
        ));
    }

    // --- Phase 2: Fate multi-cycle adaptation (cross-modal closure) ---
    while cycle < hard_max {
        session.emit_body_ports(whole_ports);
        if matches!(body, BodyTarget::Hub | BodyTarget::Triad) {
            session.emit_ports_target(whole_ports, BodyTarget::Hub);
        }
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
            "compose_fate_adapt",
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
            stopped_reason = "compose_soft_max".into();
            break;
        }
    }

    // --- Phase 3: Triad recompose slot ---
    for (i, shard) in plan.shards.iter().enumerate() {
        if shard.phase != ShardPhase::Recompose {
            continue;
        }
        let sc = feed_shard(session, shard, front, cycles_per_shard.max(2));
        shard_scores[i] = sc;
        cycle += 1;
        trace.push(snapshot(
            session,
            cycle,
            "compose_recompose",
            decompose_score,
            aggregate_score,
            &tracker,
        ));
    }

    let cross_modal = cross_modal_shard_balance(plan, &shard_scores);
    let mut report = session.build_feed_report(mode_label, 0);
    report.optimize_cycles = cycle;
    report.mcq_fate_trace = trace.clone();

    ComposeFateOutcome {
        report,
        trace,
        shard_scores,
        cycles_executed: cycle,
        adaptation_rounds,
        stopped_reason,
        decompose_score,
        aggregate_score,
        cross_modal_score: cross_modal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signal_ingress::IngressBudget;
    use crate::task_decompose::{DecomposeMode, plan_decomposition};

    #[test]
    fn compose_fate_runs_adaptation_rounds() {
        std::env::set_var("VALHALLA_COMPOSE_FATE_MIN_CYCLES", "2");
        let mel: Vec<f64> = (0..40).map(|i| (i as f64 * 0.05).sin().abs()).collect();
        let plan = plan_decomposition(
            "What is DNA? It carries genetic information.",
            "open",
            Some(DecomposeMode::MultimodalCompose),
            Some(&mel),
        );
        let front = DlFrontConfig::default();
        let budget = IngressBudget::experiment();
        let mut session = TriadSession::with_body(budget, BodyTarget::Triad);
        let ports = crate::task_decompose::shard_to_ports(&plan.shards[0], &front);
        let outcome = run_compose_fate_pipeline(
            &mut session,
            &plan,
            &ports,
            "test_compose_fate",
            &front,
            1,
        );
        assert!(outcome.adaptation_rounds >= 2);
        assert!(!outcome.trace.is_empty());
        assert!(outcome.cross_modal_score > 0.0);
    }
}
