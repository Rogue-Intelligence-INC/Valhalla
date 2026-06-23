//! MCQ task-tag pipeline: Fate-controlled decompose → aggregate cycles before decode.
//!
//! Enabled via `VALHALLA_MCQ_FATE_CYCLES=1`. Round count is **dynamic** — Fate decides
//! stop via hub/tile/stem convergence votes + marginal stability (not a fixed loop count).
//!
//! Env:
//!   `VALHALLA_MCQ_FATE_MAX_CYCLES` — soft target / decompose polish budget (default 64)
//!   `VALHALLA_MCQ_FATE_HARD_MAX`   — safety ceiling (default 128)
//!   `VALHALLA_MCQ_FATE_MIN_CYCLES` — minimum aggregate rounds before stop (default 5)

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::dl_front::{build_body_ports, hash_embed, BodyFrontPorts, DlFrontConfig, DL_EMBED_DIM};
use crate::signal_ingress::{BodyTarget, TriadFeedReport, TriadSession};

const HASH_SEED_TEXT: u64 = 0x544558545F464E54;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McqParts {
    pub tag: String,
    pub stem: String,
    pub options: Vec<(char, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McqFateCycleSnapshot {
    pub cycle: usize,
    pub phase: String,
    pub decompose_score: f64,
    pub aggregate_score: f64,
    pub tile_cells: usize,
    pub stem_cells: usize,
    pub hub_converged: bool,
    pub tile_converged: bool,
    pub stem_converged: bool,
    pub fate_rules_active: Vec<String>,
    #[serde(default)]
    pub fate_marginal_delta: f64,
    #[serde(default)]
    pub stable_streak: usize,
    /// Continuous closure signal in [0,1] — observational, not a vote count.
    #[serde(default)]
    pub fate_closure_score: f64,
    #[serde(default)]
    pub hub_pattern: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McqFateOutcome {
    pub report: TriadFeedReport,
    pub trace: Vec<McqFateCycleSnapshot>,
    pub cycles_executed: usize,
    pub stopped_reason: String,
    pub decompose_score: f64,
    pub aggregate_score: f64,
    pub parts_fed: usize,
    /// Per-option tile energy after MCQ_OPTION shard ingress (for affinity spread).
    #[serde(default)]
    pub option_scores: Vec<(char, f64)>,
}

pub fn mcq_fate_cycles_enabled() -> bool {
    std::env::var("VALHALLA_MCQ_FATE_CYCLES")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn env_usize(key: &str, default: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// Soft budget — decompose polish & typical Fate horizon (not a hard stop).
pub fn mcq_fate_max_cycles() -> usize {
    env_usize("VALHALLA_MCQ_FATE_MAX_CYCLES", 64).clamp(4, 512)
}

/// Safety ceiling only; Fate should stop earlier via convergence.
pub fn mcq_fate_hard_max() -> usize {
    let hard = env_usize("VALHALLA_MCQ_FATE_HARD_MAX", 128);
    hard.max(mcq_fate_max_cycles()).clamp(8, 512)
}

pub fn mcq_fate_min_cycles() -> usize {
    env_usize("VALHALLA_MCQ_FATE_MIN_CYCLES", 5).clamp(2, 64)
}

pub fn parse_mcq_options(question: &str) -> Vec<(char, String)> {
    let mut opts = Vec::new();
    for line in question.lines() {
        let t = line.trim();
        if t.len() >= 3 {
            let c = t.chars().next().unwrap();
            if matches!(c, 'A' | 'B' | 'C' | 'D') {
                if t.chars().nth(1).map(|x| x == ')' || x == '.').unwrap_or(false) {
                    opts.push((c, t.to_string()));
                }
            }
        }
    }
    opts
}

pub fn parse_mcq_parts(question: &str) -> McqParts {
    let options = parse_mcq_options(question);
    let stem = if let Some((_, first_line)) = options.first() {
        question
            .find(first_line)
            .map(|pos| question[..pos].trim().to_string())
            .unwrap_or_else(|| question.trim().to_string())
    } else {
        question.trim().to_string()
    };
    McqParts {
        tag: "mcq".into(),
        stem,
        options,
    }
}

fn text_to_ports(text: &str, cfg: &DlFrontConfig) -> BodyFrontPorts {
    let emb = hash_embed(text, DL_EMBED_DIM, HASH_SEED_TEXT);
    build_body_ports(&emb, cfg, "hash_grinder", "mcq_shard")
}

const MCQ_FATE_RULES: &[&str] = &[
    "task_tag:mcq",
    "mcq_decompose: ingress stem then each option as separate shard before compare",
    "mcq_aggregate: merge structure until hub/tile/stem fate converged or marginal stable",
    "mcq_dynamic_rounds: fate decides cycle count from closure signal; no vote quota",
];

fn install_mcq_fate_rules(session: &mut TriadSession) {
    for rule in MCQ_FATE_RULES {
        session.tile.triple_fate.store_string(rule);
        session.stem.triple_fate.store_string(rule);
        session.hub.triple_fate.store_string(rule);
    }
}

fn active_fate_rules(_session: &TriadSession) -> Vec<String> {
    MCQ_FATE_RULES.iter().map(|s| s.to_string()).collect()
}

fn compute_aggregate_score(session: &TriadSession) -> f64 {
    let prefs = session.hub.get_fate_preferences();
    let pref_sum: f64 = prefs.affinities.iter().copied().sum();
    let pref_spread = if pref_sum > 1e-9 {
        let max_p = prefs.affinities.iter().copied().fold(0.0f64, f64::max);
        max_p / pref_sum
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

struct FateCycleTracker {
    aggregate_history: Vec<f64>,
    decompose_history: Vec<f64>,
    stable_streak: usize,
}

impl FateCycleTracker {
    fn new() -> Self {
        Self {
            aggregate_history: Vec::new(),
            decompose_history: Vec::new(),
            stable_streak: 0,
        }
    }

    fn record(&mut self, decompose: f64, aggregate: f64) {
        self.decompose_history.push(decompose);
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

fn fate_closure_score(session: &TriadSession, body: BodyTarget) -> f64 {
    let prefs = session.hub.get_fate_preferences();
    let mut weighted = 0.0f64;
    let mut total_w = 0.0f64;

    let mut add = |active: bool, converged: bool, pattern_boost: f64, w: f64| {
        if !active {
            return;
        }
        total_w += w;
        if converged {
            weighted += w;
        } else if pattern_boost > 0.0 {
            weighted += w * pattern_boost;
        }
    };

    let hub_pattern = session.hub.get_fate_convergence_pattern();
    let hub_pat = match hub_pattern {
        Some("fixed_point") => 0.95,
        Some("converging") => 0.65,
        _ => 0.0,
    };

    let hub_w = prefs.affinities.first().copied().unwrap_or(0.25).max(0.05);
    add(
        matches!(body, BodyTarget::Hub | BodyTarget::Triad),
        session.hub.has_fate_converged(),
        hub_pat,
        hub_w,
    );

    let tile_w = prefs.affinities.get(2).copied().unwrap_or(0.25).max(0.05);
    add(
        matches!(body, BodyTarget::Tile | BodyTarget::Triad),
        session.tile.has_fate_converged(),
        hub_pat * 0.5,
        tile_w,
    );

    let stem_w = prefs.affinities.get(1).copied().unwrap_or(0.25).max(0.05);
    add(
        matches!(body, BodyTarget::StemCell | BodyTarget::Triad),
        session.stem.has_fate_converged(),
        hub_pat * 0.5,
        stem_w,
    );

    if total_w <= 1e-9 {
        return 0.0;
    }
    (weighted / total_w).clamp(0.0, 1.0)
}

fn aggregate_stop_floor(tracker: &FateCycleTracker) -> f64 {
    if tracker.stable_streak >= 3 {
        0.12
    } else if tracker.stable_streak >= 1 {
        0.22
    } else {
        0.32
    }
}

fn marginal_streak_target(decompose_score: f64) -> usize {
    if decompose_score >= 0.92 {
        4
    } else if decompose_score >= 0.75 {
        5
    } else {
        6
    }
}

fn hub_pattern_label(session: &TriadSession) -> String {
    session
        .hub
        .get_fate_convergence_pattern()
        .unwrap_or("unknown")
        .to_string()
}

/// Fate decides whether to stop aggregate phase (dynamic rounds, no vote quota).
fn fate_should_stop(
    session: &TriadSession,
    body: BodyTarget,
    tracker: &FateCycleTracker,
    cycle: usize,
    min_cycles: usize,
    aggregate_score: f64,
    decompose_score: f64,
) -> Option<&'static str> {
    let pattern = session.hub.get_fate_convergence_pattern();
    let exploring = tracker.still_exploring();
    let closure = fate_closure_score(session, body);

    if cycle < min_cycles {
        return None;
    }

    // Fate extends while marginal still improving.
    if exploring && cycle < mcq_fate_max_cycles() {
        return None;
    }

    let any_converged = session.hub.has_fate_converged()
        || (matches!(body, BodyTarget::Tile | BodyTarget::Triad) && session.tile.has_fate_converged())
        || (matches!(body, BodyTarget::StemCell | BodyTarget::Triad)
            && session.stem.has_fate_converged());

    // Hub pattern — algebraic, no vote count.
    if matches!(pattern, Some("fixed_point")) && !exploring {
        return Some("fate_fixed_point");
    }

    if matches!(pattern, Some("converging")) && any_converged && !exploring {
        return Some("fate_converged");
    }

    // Continuous closure + aggregate floor (from marginal history, not body count).
    let floor = aggregate_stop_floor(tracker);
    if closure > 0.0 && aggregate_score >= floor && !exploring {
        return Some("fate_converged");
    }

    // Marginal stability — streak scales with decompose completeness.
    let streak_need = marginal_streak_target(decompose_score);
    if tracker.stable_streak >= streak_need && !exploring {
        return Some("fate_marginal_stable");
    }

    if aggregate_score >= 0.88 && !exploring {
        return Some("aggregate_ready");
    }

    None
}

fn snapshot(
    session: &TriadSession,
    body: BodyTarget,
    cycle: usize,
    phase: &str,
    decompose_score: f64,
    aggregate_score: f64,
    tracker: &FateCycleTracker,
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
        fate_rules_active: active_fate_rules(session),
        fate_marginal_delta: tracker.marginal_delta(),
        stable_streak: tracker.stable_streak,
        fate_closure_score: fate_closure_score(session, body),
        hub_pattern: hub_pattern_label(session),
    }
}

fn mark_epoch_if_hub(session: &mut TriadSession) {
    if matches!(session.body, BodyTarget::Hub | BodyTarget::Triad) {
        session.hub.mark_epoch_complete();
    }
}

fn pulse_fate_feedback(session: &mut TriadSession, phase: &str, score: f64) {
    let intensity = score.clamp(0.05, 1.0);
    let _ = session.hub.triple_fate.store_string(&format!(
        "mcq_{phase}_tick score={intensity:.4}"
    ));
    session.tile.triple_fate.receive_passive(intensity * 0.5);
    session.stem.triple_fate.receive_passive(intensity * 0.5);
}

/// MCQ tagged ingress: decompose all shards → Fate-dynamic aggregate until stop.
pub fn run_mcq_fate_pipeline(
    session: &mut TriadSession,
    question: &str,
    whole_ports: &BodyFrontPorts,
    mode_label: &str,
    front_cfg: &DlFrontConfig,
) -> McqFateOutcome {
    install_mcq_fate_rules(session);

    let body = session.body;
    let parts = parse_mcq_parts(question);
    let mut shards: Vec<String> = vec![format!("MCQ_STEM {}", parts.stem)];
    for (_, opt_line) in &parts.options {
        shards.push(format!("MCQ_OPTION {}", opt_line));
    }
    if shards.len() == 1 {
        shards.push(format!("MCQ_WHOLE {}", question));
    }

    let soft_max = mcq_fate_max_cycles();
    let hard_max = mcq_fate_hard_max();
    let min_cycles = mcq_fate_min_cycles();

    let mut trace = Vec::new();
    let mut tracker = FateCycleTracker::new();
    let mut cycle = 0usize;
    let mut decompose_score = 0.0f64;
    let mut aggregate_score = 0.0f64;
    let mut prev_tiles = session.tile.completed_tiles.len();
    let mut prev_stem = session.stem.cells.len();
    let mut parts_fed = 0usize;
    let mut stopped_reason = "hard_max".to_string();
    let mut option_scores: HashMap<char, f64> = HashMap::new();

    // --- Decompose: feed every shard; Fate may add polish rounds ---
    for (pi, shard) in shards.iter().enumerate() {
        let ports = text_to_ports(shard, front_cfg);
        session.emit_body_ports(&ports);
        session.run_optimize_cycles(1);
        mark_epoch_if_hub(session);
        cycle += 1;
        parts_fed = pi + 1;

        let tile_now = session.tile.completed_tiles.len();
        let stem_now = session.stem.cells.len();
        let structural_delta = ((tile_now as i64 - prev_tiles as i64).max(0)
            + (stem_now as i64 - prev_stem as i64).max(0)) as f64;
        prev_tiles = tile_now;
        prev_stem = stem_now;

        decompose_score = ((parts_fed as f64 / shards.len() as f64) * 0.6
            + (structural_delta * 0.06).min(0.4))
            .clamp(0.0, 1.0);

        if let Some(rest) = shard.strip_prefix("MCQ_OPTION ") {
            if let Some(letter) = rest.chars().next() {
                if matches!(letter, 'A' | 'B' | 'C' | 'D' | 'E') {
                    let energy = session
                        .tile
                        .completed_tiles
                        .last()
                        .map(|t| t.energy.abs())
                        .unwrap_or(0.0);
                    option_scores
                        .entry(letter)
                        .and_modify(|e| *e = e.max(energy))
                        .or_insert(energy);
                }
            }
        }

        tracker.record(decompose_score, aggregate_score);
        pulse_fate_feedback(session, "decompose", decompose_score);
        trace.push(snapshot(
            session,
            body,
            cycle,
            "decompose",
            decompose_score,
            aggregate_score,
            &tracker,
        ));
    }

    // Decompose polish: Fate extends while structure still shifting.
    let decompose_polish_max = soft_max / 4;
    let mut polish = 0usize;
    while polish < decompose_polish_max && cycle < hard_max && decompose_score < 0.92 {
        let tile_now = session.tile.completed_tiles.len();
        let stem_now = session.stem.cells.len();
        if tile_now == prev_tiles && stem_now == prev_stem && polish >= 2 {
            break;
        }
        prev_tiles = tile_now;
        prev_stem = stem_now;

        let ports = text_to_ports(&shards[parts_fed.saturating_sub(1)], front_cfg);
        session.emit_body_ports(&ports);
        session.run_optimize_cycles(1);
        mark_epoch_if_hub(session);
        cycle += 1;
        polish += 1;
        decompose_score = (decompose_score + 0.04).min(1.0);
        tracker.record(decompose_score, aggregate_score);
        pulse_fate_feedback(session, "decompose_polish", decompose_score);
        trace.push(snapshot(
            session,
            body,
            cycle,
            "decompose_polish",
            decompose_score,
            aggregate_score,
            &tracker,
        ));
    }

    // --- Aggregate: dynamic rounds until Fate stops or hard_max ---
    while cycle < hard_max {
        session.emit_body_ports(whole_ports);
        session.run_optimize_cycles(1);
        mark_epoch_if_hub(session);
        cycle += 1;

        aggregate_score = compute_aggregate_score(session);
        tracker.record(decompose_score, aggregate_score);
        pulse_fate_feedback(session, "aggregate", aggregate_score);

        trace.push(snapshot(
            session,
            body,
            cycle,
            "aggregate",
            decompose_score,
            aggregate_score,
            &tracker,
        ));

        if let Some(reason) = fate_should_stop(
            session,
            body,
            &tracker,
            cycle,
            min_cycles,
            aggregate_score,
            decompose_score,
        ) {
            stopped_reason = reason.to_string();
            break;
        }
    }

    if cycle >= hard_max && stopped_reason == "hard_max" {
        stopped_reason = "hard_max".into();
    }

    let mut report = session.build_feed_report(mode_label, 0);
    report.optimize_cycles = cycle;
    report.mcq_fate_trace = trace.clone();

    let option_scores = finalize_option_scores(&parts, option_scores);

    McqFateOutcome {
        report,
        trace,
        cycles_executed: cycle,
        stopped_reason,
        decompose_score,
        aggregate_score,
        parts_fed,
        option_scores,
    }
}

/// Return letter-aligned option scores from decompose collection only.
fn finalize_option_scores(
    parts: &McqParts,
    collected: HashMap<char, f64>,
) -> Vec<(char, f64)> {
    let mut out: Vec<(char, f64)> = parts
        .options
        .iter()
        .map(|(letter, _)| (*letter, collected.get(letter).copied().unwrap_or(0.0)))
        .collect();
    if out.len() >= 2 {
        let mean = out.iter().map(|(_, s)| *s).sum::<f64>() / out.len() as f64;
        out = out.into_iter().map(|(l, s)| (l, s - mean)).collect();
    }
    out
}
