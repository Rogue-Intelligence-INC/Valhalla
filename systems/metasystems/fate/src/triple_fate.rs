//! Triple-Fate: Three Architectures for Self-Evolving Meta-Systems
//!
//! This module implements three approaches to create self-evolving systems:
//! 1. Fate-in-Fate (Nested/Hierarchical) - Inner Fate observes Outer Fate
//! 2. Fate Loop (Self-Referential/Cyclic) - Output feeds back as input
//! 3. Combined (Nested + Cyclic) - Both approaches together [WINNER]
//!
//! Key Insight: Nesting + Looping = True Self-Evolution
//! - Nesting provides hierarchical control and observation
//! - Looping provides self-reference and emergence
//! - Together they enable creation of genuinely NEW meta-systems
//!
//! This is the f64 version for performance-critical applications.

// Note: These types are available from crate but not used in the standalone implementation
// use crate::{Rational, FateSystem, ResourceLimits, InternalSpace, VoyagerDiscovery};

// ============================================================================
// Core Types
// ============================================================================

/// Signal that Triple-Fate processes (f64 for performance)
#[derive(Debug, Clone)]
pub struct TripleSignal {
    pub values: Vec<f64>,
    pub tick: u64,
    pub source: SignalSource,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SignalSource {
    External,
    SelfOutput,
    InnerFate,
    OuterFate,
}

/// Fate's internal state
#[derive(Debug, Clone)]
pub struct TripleFateState {
    pub signal_count: u64,
    pub discovery_count: u64,
    pub efficiency: f64,
    pub dimension_changes: u64,
    pub last_discovery_tick: u64,
    pub pattern_diversity: u64,
}

impl TripleFateState {
    fn new() -> Self {
        TripleFateState {
            signal_count: 0,
            discovery_count: 0,
            efficiency: 0.0,
            dimension_changes: 0,
            last_discovery_tick: 0,
            pattern_diversity: 0,
        }
    }

    fn update_efficiency(&mut self) {
        if self.signal_count > 0 {
            self.efficiency = self.discovery_count as f64 / self.signal_count as f64;
        }
    }
}

/// Rule that Fate follows
#[derive(Debug, Clone)]
pub struct TripleRule {
    pub id: String,
    pub priority: f64,
    pub condition: String,
    pub action: RuleAction,
}

#[derive(Debug, Clone)]
pub enum RuleAction {
    CreateContainer { dimension_delta: f64 },
    AdjustDimension { factor: f64 },
    SpawnNewRule { rule: Box<TripleRule> },
}

/// Discovery made by any Fate level
#[derive(Debug, Clone)]
pub struct TripleDiscovery {
    pub id: String,
    pub level: usize,
    pub discovery_type: DiscoveryType,
    pub description: String,
    pub created_at_tick: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiscoveryType {
    MathPattern,
    SystemPattern,
    NewRule,
    NewFate,
}

// ============================================================================
// Simple Space (f64)
// ============================================================================

#[derive(Debug, Clone)]
pub struct SimpleSpaceF64 {
    pub signal_tick: u64,
    pub dimension: f64,
    pub containers: u64,
}

impl SimpleSpaceF64 {
    pub fn new() -> Self {
        SimpleSpaceF64 {
            signal_tick: 0,
            dimension: 4.0,
            containers: 0,
        }
    }
    
    pub fn on_discovery(&mut self, _pattern: &str) {
        self.signal_tick += 1;
        self.containers += 1;
        self.dimension = (self.dimension + 0.25).min(11.0);
    }
}

impl Default for SimpleSpaceF64 {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// VERSION 1: Fate-in-Fate (Nested/Hierarchical)
// ============================================================================

pub struct FateInFate {
    pub outer: BaseFate,
    pub inner: MetaFate,
    pub meta_discoveries: Vec<TripleDiscovery>,
    pub global_tick: u64,
}

pub struct BaseFate {
    pub state: TripleFateState,
    pub rules: Vec<TripleRule>,
    pub space: SimpleSpaceF64,
    pub discoveries: Vec<TripleDiscovery>,
    pub signal_buffer: Vec<TripleSignal>,
}

pub struct MetaFate {
    pub state: TripleFateState,
    pub observation_interval: u64,
    pub rules_created: Vec<TripleRule>,
    pub interventions: u64,
}

impl FateInFate {
    pub fn new() -> Self {
        FateInFate {
            outer: BaseFate::new(),
            inner: MetaFate::new(),
            meta_discoveries: Vec::new(),
            global_tick: 0,
        }
    }

    pub fn tick(&mut self, external_signal: Option<Vec<f64>>) -> Vec<TripleDiscovery> {
        self.global_tick += 1;
        let mut new_discoveries = Vec::new();

        if let Some(values) = external_signal {
            let signal = TripleSignal {
                values,
                tick: self.global_tick,
                source: SignalSource::External,
            };
            if let Some(discovery) = self.outer.process_signal(signal) {
                new_discoveries.push(discovery);
            }
        }

        if self.global_tick % self.inner.observation_interval == 0 {
            let outer_state = self.outer.state.clone();
            
            if let Some(meta_discovery) = self.inner.observe_and_analyze(&outer_state, self.global_tick) {
                if let DiscoveryType::NewRule = meta_discovery.discovery_type {
                    if let Some(new_rule) = self.inner.rules_created.last() {
                        self.outer.rules.push(new_rule.clone());
                        self.inner.interventions += 1;
                    }
                }
                new_discoveries.push(meta_discovery.clone());
                self.meta_discoveries.push(meta_discovery);
            }
        }

        new_discoveries
    }

    pub fn stats(&self) -> String {
        format!(
            "FateInFate (f64):\n\
             - Tick: {}\n\
             - Outer discoveries: {}\n\
             - Meta discoveries: {}\n\
             - Interventions: {}",
            self.global_tick,
            self.outer.discoveries.len(),
            self.meta_discoveries.len(),
            self.inner.interventions
        )
    }
}

impl Default for FateInFate {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseFate {
    pub fn new() -> Self {
        BaseFate {
            state: TripleFateState::new(),
            rules: vec![
                TripleRule {
                    id: "default".into(),
                    priority: 1.0,
                    condition: "any".into(),
                    action: RuleAction::CreateContainer { dimension_delta: 0.25 },
                },
            ],
            space: SimpleSpaceF64::new(),
            discoveries: Vec::new(),
            signal_buffer: Vec::new(),
        }
    }

    pub fn process_signal(&mut self, signal: TripleSignal) -> Option<TripleDiscovery> {
        self.state.signal_count += 1;
        self.signal_buffer.push(signal.clone());

        if let Some(pattern) = detect_pattern_f64(&signal.values) {
            self.state.discovery_count += 1;
            self.state.last_discovery_tick = signal.tick;
            self.state.pattern_diversity += 1;
            self.state.update_efficiency();

            self.space.on_discovery(&pattern);
            self.state.dimension_changes += 1;

            let discovery = TripleDiscovery {
                id: format!("base_{}_{}", pattern, signal.tick),
                level: 0,
                discovery_type: DiscoveryType::MathPattern,
                description: format!("Found: {}", pattern),
                created_at_tick: signal.tick,
            };
            self.discoveries.push(discovery.clone());
            return Some(discovery);
        }

        None
    }
}

impl MetaFate {
    pub fn new() -> Self {
        MetaFate {
            state: TripleFateState::new(),
            observation_interval: 10,
            rules_created: Vec::new(),
            interventions: 0,
        }
    }

    pub fn observe_and_analyze(&mut self, outer_state: &TripleFateState, tick: u64) -> Option<TripleDiscovery> {
        self.state.signal_count += 1;

        if outer_state.efficiency < 0.1 && outer_state.signal_count > 20 {
            let new_rule = TripleRule {
                id: format!("meta_explore_{}", tick),
                priority: 2.0,
                condition: "any".into(),
                action: RuleAction::AdjustDimension { factor: 1.5 },
            };
            self.rules_created.push(new_rule);
            self.state.discovery_count += 1;

            return Some(TripleDiscovery {
                id: format!("meta_rule_{}", tick),
                level: 1,
                discovery_type: DiscoveryType::NewRule,
                description: "Created exploration rule".into(),
                created_at_tick: tick,
            });
        }

        if tick > outer_state.last_discovery_tick + 50 {
            let new_rule = TripleRule {
                id: format!("meta_diversify_{}", tick),
                priority: 3.0,
                condition: "stagnation".into(),
                action: RuleAction::CreateContainer { dimension_delta: -0.5 },
            };
            self.rules_created.push(new_rule);
            self.state.discovery_count += 1;

            return Some(TripleDiscovery {
                id: format!("meta_diversify_{}", tick),
                level: 1,
                discovery_type: DiscoveryType::NewRule,
                description: "Created diversification rule".into(),
                created_at_tick: tick,
            });
        }

        if outer_state.efficiency > 0.3 && outer_state.pattern_diversity > 5 {
            self.state.discovery_count += 1;
            return Some(TripleDiscovery {
                id: format!("spawn_ready_{}", tick),
                level: 1,
                discovery_type: DiscoveryType::SystemPattern,
                description: "Ready to spawn new Fate".into(),
                created_at_tick: tick,
            });
        }

        None
    }
}

// ============================================================================
// VERSION 2: Fate Loop (Self-Referential/Cyclic)
// ============================================================================

pub struct FateLoop {
    pub state: TripleFateState,
    pub space: SimpleSpaceF64,
    pub rules: Vec<TripleRule>,
    pub discoveries: Vec<TripleDiscovery>,
    pub feedback_buffer: Vec<TripleSignal>,
    pub feedback_delay: u64,
    pub feedback_strength: f64,
    pub generation: u64,
    pub emergent_rules: Vec<TripleRule>,
}

impl FateLoop {
    pub fn new() -> Self {
        FateLoop {
            state: TripleFateState::new(),
            space: SimpleSpaceF64::new(),
            rules: vec![
                TripleRule {
                    id: "default".into(),
                    priority: 1.0,
                    condition: "any".into(),
                    action: RuleAction::CreateContainer { dimension_delta: 0.25 },
                },
            ],
            discoveries: Vec::new(),
            feedback_buffer: Vec::new(),
            feedback_delay: 5,
            feedback_strength: 0.5,
            generation: 0,
            emergent_rules: Vec::new(),
        }
    }

    pub fn tick(&mut self, external_signal: Option<Vec<f64>>) -> Vec<TripleDiscovery> {
        self.state.signal_count += 1;
        let current_tick = self.state.signal_count;
        let mut new_discoveries = Vec::new();

        if let Some(values) = external_signal {
            let signal = TripleSignal {
                values,
                tick: current_tick,
                source: SignalSource::External,
            };
            if let Some(discovery) = self.process_signal(&signal) {
                new_discoveries.push(discovery);
            }
        }

        let ready_feedback: Vec<TripleSignal> = self.feedback_buffer
            .iter()
            .filter(|s| current_tick - s.tick >= self.feedback_delay)
            .cloned()
            .collect();

        for feedback in ready_feedback {
            self.feedback_buffer.retain(|s| s.tick != feedback.tick);
            
            let mut processed = feedback.clone();
            processed.source = SignalSource::SelfOutput;
            
            if let Some(discovery) = self.process_signal(&processed) {
                let meta_discovery = TripleDiscovery {
                    id: format!("self_ref_{}_{}", discovery.id, current_tick),
                    level: 1,
                    discovery_type: DiscoveryType::SystemPattern,
                    description: format!("Self-ref: {}", discovery.description),
                    created_at_tick: current_tick,
                };
                new_discoveries.push(meta_discovery.clone());
                self.discoveries.push(meta_discovery);
                self.maybe_create_emergent_rule(current_tick);
            }
        }

        if self.should_evolve() {
            self.evolve();
            new_discoveries.push(TripleDiscovery {
                id: format!("evolution_{}", self.generation),
                level: 2,
                discovery_type: DiscoveryType::NewFate,
                description: format!("Evolved to gen {}", self.generation),
                created_at_tick: current_tick,
            });
        }

        if self.feedback_buffer.len() > 100 {
            self.feedback_buffer = self.feedback_buffer.split_off(self.feedback_buffer.len() - 50);
        }

        new_discoveries
    }

    fn process_signal(&mut self, signal: &TripleSignal) -> Option<TripleDiscovery> {
        if let Some(pattern) = detect_pattern_f64(&signal.values) {
            self.state.discovery_count += 1;
            self.state.last_discovery_tick = signal.tick;
            self.state.pattern_diversity += 1;
            self.state.update_efficiency();

            self.space.on_discovery(&pattern);
            self.state.dimension_changes += 1;

            let feedback_values: Vec<f64> = signal.values.iter()
                .map(|v| v * self.feedback_strength)
                .collect();
            
            self.feedback_buffer.push(TripleSignal {
                values: feedback_values,
                tick: signal.tick,
                source: SignalSource::SelfOutput,
            });

            let discovery = TripleDiscovery {
                id: format!("loop_{}_{}", pattern, signal.tick),
                level: 0,
                discovery_type: DiscoveryType::MathPattern,
                description: format!("Found: {} ({:?})", pattern, signal.source),
                created_at_tick: signal.tick,
            };
            self.discoveries.push(discovery.clone());
            return Some(discovery);
        }

        let feedback_values: Vec<f64> = signal.values.iter()
            .map(|v| v * self.feedback_strength)
            .collect();
        
        self.feedback_buffer.push(TripleSignal {
            values: feedback_values,
            tick: signal.tick,
            source: SignalSource::SelfOutput,
        });

        None
    }

    fn maybe_create_emergent_rule(&mut self, tick: u64) {
        let self_ref_count = self.discoveries.iter()
            .filter(|d| d.discovery_type == DiscoveryType::SystemPattern)
            .count();

        if self_ref_count > 0 && self_ref_count % 50 == 0 {
            let new_rule = TripleRule {
                id: format!("emergent_{}", tick),
                priority: self_ref_count as f64,
                condition: "self_pattern".into(),
                action: RuleAction::AdjustDimension { 
                    factor: 1.0 + self.feedback_strength,
                },
            };
            self.emergent_rules.push(new_rule.clone());
            self.rules.push(new_rule);
        }
    }

    fn should_evolve(&self) -> bool {
        let self_discoveries = self.discoveries.iter()
            .filter(|d| d.level >= 1)
            .count();
        
        self_discoveries > 0 && self_discoveries % 100 == 0
    }

    fn evolve(&mut self) {
        self.generation += 1;
        self.feedback_delay = (self.feedback_delay + 1) % 10 + 1;
        self.feedback_strength *= 0.9;
        if self.feedback_strength < 0.1 {
            self.feedback_strength = 0.1;
        }
    }

    pub fn stats(&self) -> String {
        let self_ref = self.discoveries.iter().filter(|d| d.level >= 1).count();
        format!(
            "FateLoop (f64):\n\
             - Signals: {}\n\
             - Generation: {}\n\
             - Discoveries: {} (self-ref: {})\n\
             - Emergent rules: {}\n\
             - Efficiency: {:.2}%",
            self.state.signal_count,
            self.generation,
            self.discoveries.len(),
            self_ref,
            self.emergent_rules.len(),
            self.state.efficiency * 100.0
        )
    }
}

impl Default for FateLoop {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// VERSION 3: Combined (THE WINNER)
// ============================================================================

pub struct TripleFateCombined {
    pub base: FateLoop,
    pub meta: FateLoop,
    pub ultimate_discoveries: Vec<TripleDiscovery>,
    pub global_tick: u64,
    pub cross_feedback: bool,
}

impl TripleFateCombined {
    pub fn new() -> Self {
        let mut base = FateLoop::new();
        base.feedback_strength = 0.33;
        
        let mut meta = FateLoop::new();
        meta.feedback_strength = 0.25;
        meta.feedback_delay = 3;
        
        TripleFateCombined {
            base,
            meta,
            ultimate_discoveries: Vec::new(),
            global_tick: 0,
            cross_feedback: true,
        }
    }

    pub fn tick(&mut self, external_signal: Option<Vec<f64>>) -> Vec<TripleDiscovery> {
        self.global_tick += 1;
        let mut all_discoveries = Vec::new();

        let base_discoveries = self.base.tick(external_signal.clone());
        all_discoveries.extend(base_discoveries);

        let base_state_signal = self.state_to_signal(&self.base.state);
        let meta_discoveries = self.meta.tick(Some(base_state_signal));
        
        for d in meta_discoveries {
            let elevated = TripleDiscovery {
                id: format!("meta_{}", d.id),
                level: d.level + 1,
                discovery_type: d.discovery_type.clone(),
                description: format!("[META] {}", d.description),
                created_at_tick: self.global_tick,
            };
            all_discoveries.push(elevated);
        }

        if self.cross_feedback && !self.meta.emergent_rules.is_empty() {
            for rule in &self.meta.emergent_rules {
                if !self.base.rules.iter().any(|r| r.id == rule.id) {
                    self.base.rules.push(rule.clone());
                }
            }
        }

        if self.should_create_new_fate() {
            let ultimate = TripleDiscovery {
                id: format!("new_fate_{}", self.global_tick),
                level: 3,
                discovery_type: DiscoveryType::NewFate,
                description: "ULTIMATE: New meta-system created!".into(),
                created_at_tick: self.global_tick,
            };
            all_discoveries.push(ultimate.clone());
            self.ultimate_discoveries.push(ultimate);
        }

        all_discoveries
    }

    fn state_to_signal(&self, state: &TripleFateState) -> Vec<f64> {
        vec![
            state.signal_count as f64,
            state.discovery_count as f64,
            state.efficiency,
            state.dimension_changes as f64,
            state.pattern_diversity as f64,
        ]
    }

    fn should_create_new_fate(&self) -> bool {
        let total_meta = self.base.discoveries.iter().filter(|d| d.level >= 1).count()
            + self.meta.discoveries.iter().filter(|d| d.level >= 1).count();
        
        (self.base.generation >= 1 && self.meta.generation >= 1 && self.ultimate_discoveries.is_empty())
        || (total_meta > 1000 && self.ultimate_discoveries.is_empty())
    }

    pub fn stats(&self) -> String {
        format!(
            "TripleFateCombined (f64):\n\
             - Tick: {}\n\
             \n[BASE]\n{}\n\
             \n[META]\n{}\n\
             \n[ULTIMATE]\n\
             - Ultimate discoveries: {}\n\
             - Cross-feedback: {}",
            self.global_tick,
            self.base.stats(),
            self.meta.stats(),
            self.ultimate_discoveries.len(),
            self.cross_feedback
        )
    }
}

impl Default for TripleFateCombined {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Pattern Detection (f64)
// ============================================================================

fn detect_pattern_f64(values: &[f64]) -> Option<String> {
    if values.len() < 2 {
        return None;
    }

    if values.len() >= 3 {
        let d1 = values[1] - values[0];
        let d2 = values[2] - values[1];
        if (d1 - d2).abs() < 0.001 && d1.abs() > 0.001 {
            return Some(format!("linear_d{:.2}", d1));
        }
    }

    if values.len() >= 3 {
        let diff1 = (values[1] - values[0]).abs();
        let diff2 = (values[2] - values[1]).abs();
        if diff2 < diff1 && diff1 > 0.001 {
            return Some("converging".into());
        }
    }

    if values.len() >= 3 {
        let diff1 = (values[1] - values[0]).abs();
        let diff2 = (values[2] - values[1]).abs();
        if diff2 > diff1 * 2.0 {
            return Some("diverging".into());
        }
    }

    if values.len() >= 2 && (values[0] - values[1]).abs() < 0.001 {
        return Some("fixed_point".into());
    }

    let sum: f64 = values.iter().sum();
    if sum.abs() < 0.001 && !values.iter().all(|v| v.abs() < 0.001) {
        return Some("balanced".into());
    }

    None
}

// ============================================================================
// Experiment Runner
// ============================================================================

#[derive(Debug)]
pub struct ExperimentResult {
    pub name: String,
    pub ticks: u64,
    pub total_discoveries: usize,
    pub math_discoveries: usize,
    pub system_discoveries: usize,
    pub new_rules: usize,
    pub new_fates: usize,
    pub final_efficiency: f64,
    pub generations: u64,
}

pub fn run_comparison_experiment(signal_count: usize) -> Vec<ExperimentResult> {
    let mut results = Vec::new();

    let test_signals: Vec<Vec<f64>> = (0..signal_count)
        .map(|i| {
            vec![
                (i % 50) as f64,
                (i % 50) as f64 + 1.0,
                (i % 50) as f64 + 2.0,
            ]
        })
        .collect();

    // Test Version 1
    let mut fate_in_fate = FateInFate::new();
    let mut fif_discoveries = Vec::new();
    for signal in &test_signals {
        fif_discoveries.extend(fate_in_fate.tick(Some(signal.clone())));
    }
    for _ in 0..50 {
        fif_discoveries.extend(fate_in_fate.tick(None));
    }

    results.push(ExperimentResult {
        name: "Fate-in-Fate (f64)".into(),
        ticks: fate_in_fate.global_tick,
        total_discoveries: fif_discoveries.len(),
        math_discoveries: fif_discoveries.iter().filter(|d| d.discovery_type == DiscoveryType::MathPattern).count(),
        system_discoveries: fif_discoveries.iter().filter(|d| d.discovery_type == DiscoveryType::SystemPattern).count(),
        new_rules: fif_discoveries.iter().filter(|d| d.discovery_type == DiscoveryType::NewRule).count(),
        new_fates: fif_discoveries.iter().filter(|d| d.discovery_type == DiscoveryType::NewFate).count(),
        final_efficiency: fate_in_fate.outer.state.efficiency,
        generations: 0,
    });

    // Test Version 2
    let mut fate_loop = FateLoop::new();
    let mut fl_discoveries = Vec::new();
    for signal in &test_signals {
        fl_discoveries.extend(fate_loop.tick(Some(signal.clone())));
    }
    for _ in 0..100 {
        fl_discoveries.extend(fate_loop.tick(None));
    }

    results.push(ExperimentResult {
        name: "Fate Loop (f64)".into(),
        ticks: fate_loop.state.signal_count,
        total_discoveries: fl_discoveries.len(),
        math_discoveries: fl_discoveries.iter().filter(|d| d.discovery_type == DiscoveryType::MathPattern).count(),
        system_discoveries: fl_discoveries.iter().filter(|d| d.discovery_type == DiscoveryType::SystemPattern).count(),
        new_rules: fl_discoveries.iter().filter(|d| d.discovery_type == DiscoveryType::NewRule).count(),
        new_fates: fl_discoveries.iter().filter(|d| d.discovery_type == DiscoveryType::NewFate).count(),
        final_efficiency: fate_loop.state.efficiency,
        generations: fate_loop.generation,
    });

    // Test Version 3
    let mut combined = TripleFateCombined::new();
    let mut tc_discoveries = Vec::new();
    for signal in &test_signals {
        tc_discoveries.extend(combined.tick(Some(signal.clone())));
    }
    for _ in 0..150 {
        tc_discoveries.extend(combined.tick(None));
    }

    results.push(ExperimentResult {
        name: "Combined (f64)".into(),
        ticks: combined.global_tick,
        total_discoveries: tc_discoveries.len(),
        math_discoveries: tc_discoveries.iter().filter(|d| d.discovery_type == DiscoveryType::MathPattern).count(),
        system_discoveries: tc_discoveries.iter().filter(|d| d.discovery_type == DiscoveryType::SystemPattern).count(),
        new_rules: tc_discoveries.iter().filter(|d| d.discovery_type == DiscoveryType::NewRule).count(),
        new_fates: tc_discoveries.iter().filter(|d| d.discovery_type == DiscoveryType::NewFate).count(),
        final_efficiency: combined.base.state.efficiency,
        generations: combined.base.generation + combined.meta.generation,
    });

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combined_f64() {
        let mut system = TripleFateCombined::new();
        for i in 0..200 {
            let signal = vec![i as f64, i as f64 + 1.0, i as f64 + 2.0];
            system.tick(Some(signal));
        }
        for _ in 0..100 {
            system.tick(None);
        }
        println!("{}", system.stats());
        assert!(system.base.discoveries.len() > 0);
    }
}
