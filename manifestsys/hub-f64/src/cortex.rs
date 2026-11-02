//! Cortex = exhaustion + exclusion (structure borrowed from s-wind; no physics written)
//!
//! **Concepts**:
//! - **Cortex**: the "exhaustive exploration + exclusion" loop implemented in this module, i.e. the thinking layer. Input comes from the body; output is structural discoveries and an exclusion set.
//! - **Body**: Valhalla (Hub, StemCell, Tile, Chamber) - produces and consumes data, observed and constrained by the cortex.
//!
//! **Flow (aligned with s-wind)**:
//! 1. Take f64 signals from the body
//! 2. Space transforms: Difference, Ratio, CumulativeSum, etc.
//! 3. Simple structure recognition on the transformed signal (constant, linear, recurrence, etc.)
//! 4. Exclusion: discard hypotheses with large error or instability
//! 5. Repeat until closure (no new discoveries or max rounds reached)

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Body sources (the Valhalla subsystems)
// ---------------------------------------------------------------------------

/// Body-unit type: which Valhalla subsystem a signal comes from
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BodyKind {
    Hub,
    StemCell,
    Tile,
    Chamber,
}

impl BodyKind {
    pub fn as_storage_key(&self) -> &'static str {
        match self {
            BodyKind::Hub => "hub",
            BodyKind::StemCell => "stemcell",
            BodyKind::Tile => "tile",
            BodyKind::Chamber => "chamber",
        }
    }
}

/// One signal channel from the body (for cortex consumption)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodySignal {
    pub source: BodyKind,
    pub channel_id: String,
    pub sequence: Vec<f64>,
}

// ---------------------------------------------------------------------------
// Space transforms (f64, corresponding to s-wind TransformType)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransformType {
    /// Delta: first difference
    Difference,
    /// Delta^2: second difference
    SecondDifference,
    /// Ratio of consecutive terms
    Ratio,
    /// Cumulative sum
    CumulativeSum,
    /// Absolute value
    AbsoluteValue,
    /// Moving average, window length
    MovingAverage(usize),
}

impl TransformType {
    /// Transform an f64 sequence, returning a new sequence (possibly shorter)
    pub fn apply(&self, signal: &[f64]) -> Vec<f64> {
        match self {
            TransformType::Difference => {
                if signal.len() < 2 {
                    return vec![];
                }
                signal.windows(2).map(|w| w[1] - w[0]).collect()
            }
            TransformType::SecondDifference => {
                let first: Vec<f64> = signal.windows(2).map(|w| w[1] - w[0]).collect();
                if first.len() < 2 {
                    return vec![];
                }
                first.windows(2).map(|w| w[1] - w[0]).collect()
            }
            TransformType::Ratio => {
                if signal.len() < 2 {
                    return vec![];
                }
                signal
                    .windows(2)
                    .filter_map(|w| {
                        if w[0].abs() < 1e-20 {
                            None
                        } else {
                            Some(w[1] / w[0])
                        }
                    })
                    .collect()
            }
            TransformType::CumulativeSum => {
                let mut out = Vec::with_capacity(signal.len());
                let mut s = 0.0;
                for &x in signal {
                    s += x;
                    out.push(s);
                }
                out
            }
            TransformType::AbsoluteValue => signal.iter().map(|x| x.abs()).collect(),
            TransformType::MovingAverage(w) => {
                let w = (*w).max(1);
                if signal.len() < w {
                    return signal.to_vec();
                }
                signal
                    .windows(w)
                    .map(|wnd| wnd.iter().sum::<f64>() / w as f64)
                    .collect()
            }
        }
    }
}

/// All enumerable transforms (for exhaustion)
pub fn all_transforms_for_exhaustion() -> Vec<TransformType> {
    vec![
        TransformType::Difference,
        TransformType::SecondDifference,
        TransformType::Ratio,
        TransformType::CumulativeSum,
        TransformType::AbsoluteValue,
        TransformType::MovingAverage(2),
        TransformType::MovingAverage(3),
    ]
}

// ---------------------------------------------------------------------------
// Simple structure recognition (for modelling + exclusion)
// ---------------------------------------------------------------------------

/// Recognised simple structure types (structure only, no physics names)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StructureKind {
    /// Approximately constant
    Constant(f64),
    /// Approximately linear y ~ a*x + b
    Linear { a: f64, b: f64 },
    /// Unrecognised or excluded
    Unknown,
}

/// Simple structure recognition on an f64 span; returns the structure type and mean squared error (for exclusion)
pub fn fit_simple_structure(signal: &[f64]) -> (StructureKind, f64) {
    if signal.is_empty() {
        return (StructureKind::Unknown, f64::MAX);
    }
    let n = signal.len() as f64;
    let mean: f64 = signal.iter().sum::<f64>() / n;
    let var: f64 = signal.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
    let std = var.sqrt();
    if std < 1e-12 || var < 1e-20 {
        return (StructureKind::Constant(mean), 0.0);
    }
    let xs: Vec<f64> = (0..signal.len()).map(|i| i as f64).collect();
    let (a, b) = linear_fit(&xs, signal);
    let mse: f64 = xs
        .iter()
        .zip(signal.iter())
        .map(|(x, y)| (a * x + b - y).powi(2))
        .sum::<f64>()
        / n;
    (StructureKind::Linear { a, b }, mse)
}

fn linear_fit(x: &[f64], y: &[f64]) -> (f64, f64) {
    let n = x.len() as f64;
    if n < 2.0 {
        return (0.0, y.first().copied().unwrap_or(0.0));
    }
    let sx: f64 = x.iter().sum();
    let sy: f64 = y.iter().sum();
    let sxx: f64 = x.iter().map(|v| v * v).sum();
    let sxy: f64 = x.iter().zip(y.iter()).map(|(a, b)| a * b).sum();
    let den = n * sxx - sx * sx;
    if den.abs() < 1e-20 {
        return (0.0, sy / n);
    }
    let a = (n * sxy - sx * sy) / den;
    let b = (sy - a * sx) / n;
    (a, b)
}

// ---------------------------------------------------------------------------
// Exclusion and closure
// ---------------------------------------------------------------------------

/// One exploration result: a transform + a structure; excessive error counts as excluded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryCandidate {
    pub transform: TransformType,
    pub structure: StructureKind,
    pub mse: f64,
    pub excluded: bool,
}

/// Exclusion condition: error above threshold, or Unknown structure with large error
pub fn should_exclude(mse: f64, structure: &StructureKind, error_threshold: f64) -> bool {
    if mse > error_threshold {
        return true;
    }
    matches!(structure, StructureKind::Unknown) && mse > 0.0
}

// ---------------------------------------------------------------------------
// Cortex: exhaustion + exclusion engine
// ---------------------------------------------------------------------------

/// Cortex state: current round, discovered structures, exclusion set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CortexState {
    pub round: u32,
    pub discovered: Vec<DiscoveryCandidate>,
    pub excluded_count: usize,
}

/// Cortex configuration (exhaustion and exclusion parameters)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CortexConfig {
    pub max_rounds: u32,
    pub error_threshold: f64,
    pub min_signal_len: usize,
}

impl Default for CortexConfig {
    fn default() -> Self {
        Self {
            max_rounds: 20,
            error_threshold: 1e-6,
            min_signal_len: 3,
        }
    }
}

/// Cortex: exhaustive exploration + exclusion over body signals, until closure
pub struct Cortex {
    pub config: CortexConfig,
    pub state: CortexState,
    /// Previously seen (transform, structure) signatures, used to judge closure (no new discoveries)
    seen_signatures: HashSet<String>,
}

impl Cortex {
    pub fn new(config: CortexConfig) -> Self {
        Self {
            config,
            state: CortexState {
                round: 0,
                discovered: Vec::new(),
                excluded_count: 0,
            },
            seen_signatures: HashSet::new(),
        }
    }

    /// One round on the current signal: enumerate transforms -> fit structure -> exclude
    pub fn step(&mut self, signal: &[f64]) -> bool {
        if signal.len() < self.config.min_signal_len {
            return true;
        }
        self.state.round += 1;
        let mut new_this_round = 0;
        for transform in all_transforms_for_exhaustion() {
            let transformed = transform.apply(signal);
            if transformed.len() < self.config.min_signal_len {
                continue;
            }
            let (structure, mse) = fit_simple_structure(&transformed);
            let excluded =
                should_exclude(mse, &structure, self.config.error_threshold);
            let sig = format!("{:?}_{:?}", transform, structure);
            if !self.seen_signatures.contains(&sig) {
                self.seen_signatures.insert(sig);
                new_this_round += 1;
            }
            self.state.discovered.push(DiscoveryCandidate {
                transform,
                structure: structure.clone(),
                mse,
                excluded,
            });
            if excluded {
                self.state.excluded_count += 1;
            }
        }
        new_this_round == 0
    }

    /// Exhaustion loop: from body signals, repeat transform->fit->exclude until closure or max_rounds
    pub fn run_until_closure(&mut self, body_signal: &BodySignal) {
        let mut current: Vec<f64> = body_signal.sequence.clone();
        while self.state.round < self.config.max_rounds {
            if current.len() < self.config.min_signal_len {
                break;
            }
            let start_len = self.state.discovered.len();
            let no_new = self.step(&current);
            let this_round = &self.state.discovered[start_len..];
            let best = this_round
                .iter()
                .filter(|c| !c.excluded)
                .min_by(|a, b| a.mse.partial_cmp(&b.mse).unwrap_or(std::cmp::Ordering::Equal));
            if let Some(b) = best {
                current = b.transform.apply(&current);
            } else {
                break;
            }
            if no_new {
                break;
            }
        }
    }

    pub fn state(&self) -> &CortexState {
        &self.state
    }
}

// ---------------------------------------------------------------------------
// Legacy API compatibility: CortexKind / CortexSummary treated as "body unit" markers
// ---------------------------------------------------------------------------

/// Body-unit type (synonymous with BodyKind; the external naming for "the Valhalla subsystems")
pub type CortexKind = BodyKind;

/// A body unit's duty summary at a moment (whether it carries storage, whether the cortex controls it)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CortexSummary {
    pub kind: BodyKind,
    pub is_storage_point: bool,
    pub is_controller: bool,
}

impl CortexSummary {
    pub fn hub(is_storage_point: bool, is_controller: bool) -> Self {
        Self {
            kind: BodyKind::Hub,
            is_storage_point,
            is_controller,
        }
    }
    pub fn stemcell(is_storage_point: bool, is_controller: bool) -> Self {
        Self {
            kind: BodyKind::StemCell,
            is_storage_point,
            is_controller,
        }
    }
    pub fn tile(is_storage_point: bool, is_controller: bool) -> Self {
        Self {
            kind: BodyKind::Tile,
            is_storage_point,
            is_controller,
        }
    }
    pub fn chamber(is_storage_point: bool, is_controller: bool) -> Self {
        Self {
            kind: BodyKind::Chamber,
            is_storage_point,
            is_controller,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CortexRole {
    Cortex,
}
