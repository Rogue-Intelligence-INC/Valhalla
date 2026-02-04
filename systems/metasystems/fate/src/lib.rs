//! Fate System v3.0 - Signal Coordination with Internal Space Evolution
//!
//! Includes Triple-Fate: Three architectures for self-evolving meta-systems
//!
//! Core capabilities:
//! 1. Signal coordination with exact Rational arithmetic
//! 2. Internal space that evolves based on discoveries
//! 3. Discovery integration: equations modify internal state
//! 4. Signal-based time (deformation signal count, NOT natural time)
//!
//! NOTE: Non-integer dimension calculation is handled by GLOBAL Space component,
//!       not by individual Fate instances.
//!
//! Feedback loop:
//!   Fate generates signal → Voyager analyzes → Discovery created
//!   → Fate integrates discovery → Internal space evolves → Loop

pub mod triple_fate;

use serde::{Deserialize, Serialize};

// ============================================================================
// Exact Rational Arithmetic (i128/i128)
// ============================================================================

/// Exact rational number - i128/i128 with GCD reduction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Rational {
    pub numerator: i128,
    pub denominator: i128,
}

impl Rational {
    pub fn new(n: i128, d: i128) -> Self {
        if d == 0 {
            panic!("Division by zero");
        }
        let g = gcd(n.abs(), d.abs());
        let sign = if d < 0 { -1 } else { 1 };
        Rational {
            numerator: sign * n / g,
            denominator: (sign * d).abs() / g,
        }
    }

    pub fn zero() -> Self {
        Rational { numerator: 0, denominator: 1 }
    }

    pub fn one() -> Self {
        Rational { numerator: 1, denominator: 1 }
    }
    
    pub fn from_i128(n: i128) -> Self {
        Rational { numerator: n, denominator: 1 }
    }

    pub fn add(&self, other: &Rational) -> Rational {
        Rational::new(
            self.numerator * other.denominator + other.numerator * self.denominator,
            self.denominator * other.denominator,
        )
    }
    
    pub fn sub(&self, other: &Rational) -> Rational {
        Rational::new(
            self.numerator * other.denominator - other.numerator * self.denominator,
            self.denominator * other.denominator,
        )
    }

    pub fn mul(&self, other: &Rational) -> Rational {
        Rational::new(
            self.numerator * other.numerator,
            self.denominator * other.denominator,
        )
    }

    pub fn div(&self, other: &Rational) -> Rational {
        Rational::new(
            self.numerator * other.denominator,
            self.denominator * other.numerator,
        )
    }

    pub fn is_zero(&self) -> bool {
        self.numerator == 0
    }
    
    pub fn abs(&self) -> Rational {
        Rational {
            numerator: self.numerator.abs(),
            denominator: self.denominator,
        }
    }
}

fn gcd(a: i128, b: i128) -> i128 {
    if b == 0 { a } else { gcd(b, a % b) }
}

impl std::fmt::Display for Rational {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.denominator == 1 {
            write!(f, "{}", self.numerator)
        } else {
            write!(f, "{}/{}", self.numerator, self.denominator)
        }
    }
}

// ============================================================================
// Voyager Discovery (for Fate integration)
// ============================================================================

/// Voyager discovery that modifies Fate's internal space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoyagerDiscovery {
    pub equation_id: String,
    pub equation_type: String,
    pub coefficients: Vec<Rational>,
    pub algebraically_verified: bool,
    /// Signal tick when discovered (NOT natural time)
    pub discovered_at: u64,
}

// ============================================================================
// Internal Space (evolves based on discoveries)
// ============================================================================

/// Internal space state - modified by discoveries
/// Dimension expansion is handled by global Space component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalSpace {
    /// Current integer dimension (non-integer is managed by global Space)
    pub dimensions: usize,
    /// Basis vectors defining the space
    pub basis_vectors: Vec<Vec<Rational>>,
    /// Accumulated equations from Voyager
    pub accumulated_equations: Vec<VoyagerDiscovery>,
    /// Expansion count
    pub expansion_count: u64,
}

impl InternalSpace {
    pub fn new(initial_dims: usize) -> Self {
        // Create identity basis for initial dimensions
        let mut basis = Vec::new();
        for i in 0..initial_dims {
            let mut vec = vec![Rational::zero(); initial_dims];
            vec[i] = Rational::one();
            basis.push(vec);
        }
        
        Self {
            dimensions: initial_dims,
            basis_vectors: basis,
            accumulated_equations: Vec::new(),
            expansion_count: 0,
        }
    }
    
    /// Integrate a discovery from Voyager - modifies internal space
    pub fn integrate_discovery(&mut self, discovery: VoyagerDiscovery) {
        // Only integrate verified discoveries
        if !discovery.algebraically_verified {
            return;
        }
        
        // Add to accumulated equations
        self.accumulated_equations.push(discovery.clone());
        
        // Add new basis vector from discovery coefficients
        if !discovery.coefficients.is_empty() {
            let mut new_basis = discovery.coefficients.clone();
            
            // Pad to current dimension count
            while new_basis.len() < self.dimensions {
                new_basis.push(Rational::zero());
            }
            
            // Only add if linearly independent (simplified check)
            if !self.is_dependent(&new_basis) {
                self.basis_vectors.push(new_basis);
                self.expansion_count += 1;
            }
        }
    }
    
    /// Check if vector is dependent on existing basis (simplified)
    fn is_dependent(&self, vec: &[Rational]) -> bool {
        for basis in &self.basis_vectors {
            if basis.len() == vec.len() {
                let mut all_proportional = true;
                let mut ratio: Option<Rational> = None;
                
                for (a, b) in basis.iter().zip(vec.iter()) {
                    if !b.is_zero() {
                        let r = a.div(b);
                        if let Some(prev_ratio) = ratio {
                            if r != prev_ratio {
                                all_proportional = false;
                                break;
                            }
                        } else {
                            ratio = Some(r);
                        }
                    } else if !a.is_zero() {
                        all_proportional = false;
                        break;
                    }
                }
                
                if all_proportional && ratio.is_some() {
                    return true;
                }
            }
        }
        false
    }
    
    /// Get current dimension count
    pub fn get_dimension_count(&self) -> usize {
        self.dimensions
    }
    
    /// Get accumulated equation count
    pub fn get_equation_count(&self) -> usize {
        self.accumulated_equations.len()
    }
    
    /// Get basis vector count
    pub fn get_basis_count(&self) -> usize {
        self.basis_vectors.len()
    }
}

impl Default for InternalSpace {
    fn default() -> Self {
        Self::new(4) // Start with 4 dimensions
    }
}

// ============================================================================
// Fate System (Signal Coordination)
// ============================================================================

/// System type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemType {
    BlackHole,
    MeshBrain,
    MultiNova,
    BaseForce,
}

/// Resource limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_mb: u64,
    pub max_cpu_percent: u8,
    pub max_buffer_size: usize,
    pub processing_interval_ms: u64,
}

/// Deformation signal - all values are exact Rational
/// timestamp is signal_tick (deformation signal count), NOT natural time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeformationSignal {
    pub source_system: SystemType,
    pub intensity: Rational,
    pub direction: Vec<Rational>,
    /// Signal tick (deformation signal count) - NOT natural time
    pub signal_tick: u64,
    pub quality_score: Rational,
}

/// System preferences - computed via exact weighted average
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPreferences {
    pub affinities: Vec<Rational>,
}

impl Default for SystemPreferences {
    fn default() -> Self {
        Self {
            affinities: vec![Rational::zero(); 4],
        }
    }
}

/// Resource usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub memory_bytes: usize,
    pub buffer_usage: usize,
    pub buffer_capacity: usize,
}

/// Fate 对系统的自主评估结果
/// 所有值由 Fate 自主学习填充，初始为空 (0)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FateEvaluation {
    /// 质量分数 - Fate 自主赋予，基于历史表现
    pub quality: Rational,
    /// 强度 - Fate 自主赋予，基于系统活跃度
    pub intensity: Rational,
    /// 是否为启动轮 (第一轮信号作废)
    pub is_bootstrap: bool,
}

/// Fate System - Signal coordination with evolving internal space
/// Time is measured by signal_tick (deformation signal count), NOT natural time
#[derive(Debug, Clone)]
pub struct FateSystem {
    pub entity_id: String,
    pub deformation_buffer: Vec<DeformationSignal>,
    pub resource_limits: ResourceLimits,
    pub is_active: bool,
    /// Internal space that evolves based on discoveries
    pub internal_space: InternalSpace,
    /// Signal tick counter (logical time based on deformation signals)
    pub signal_tick: u64,
}

impl FateSystem {
    pub fn new(entity_id: String, resource_limits: ResourceLimits) -> Self {
        Self {
            entity_id,
            deformation_buffer: Vec::new(),
            resource_limits,
            is_active: true,
            internal_space: InternalSpace::default(),
            signal_tick: 0,
        }
    }
    
    /// Create with custom initial dimensions
    pub fn with_dimensions(entity_id: String, resource_limits: ResourceLimits, dims: usize) -> Self {
        Self {
            entity_id,
            deformation_buffer: Vec::new(),
            resource_limits,
            is_active: true,
            internal_space: InternalSpace::new(dims),
            signal_tick: 0,
        }
    }

    /// Receive deformation signal
    /// Increments signal_tick (logical time)
    pub fn receive_signal(&mut self, mut signal: DeformationSignal) {
        if !self.is_active {
            return;
        }
        
        // Update signal tick (logical time)
        self.signal_tick += 1;
        signal.signal_tick = self.signal_tick;
        
        if self.deformation_buffer.len() >= self.resource_limits.max_buffer_size {
            self.deformation_buffer.remove(0);
        }
        self.deformation_buffer.push(signal);
    }
    
    /// Generate signal from internal space state
    /// Uses signal_tick as timestamp (NOT natural time)
    pub fn generate_signal(&mut self) -> DeformationSignal {
        self.signal_tick += 1;
        
        let n = self.signal_tick as i128;
        let dim_count = self.internal_space.get_dimension_count().max(4);
        
        // Generate direction based on accumulated knowledge
        let mut direction = Vec::with_capacity(dim_count);
        
        for i in 0..dim_count {
            // Use internal space basis if available
            if i < self.internal_space.basis_vectors.len() {
                let basis = &self.internal_space.basis_vectors[i];
                if !basis.is_empty() {
                    direction.push(basis[0]);
                    continue;
                }
            }
            
            // Default: algebraic pattern
            let val = match i % 4 {
                0 => Rational::new(n % 10 + 1, n % 5 + 2),
                1 => Rational::new(1, n % 7 + 1),
                2 => Rational::new(n % 3 + 1, n % 4 + 1),
                _ => Rational::one(),
            };
            direction.push(val);
        }
        
        DeformationSignal {
            source_system: match n % 4 {
                0 => SystemType::BlackHole,
                1 => SystemType::MeshBrain,
                2 => SystemType::MultiNova,
                _ => SystemType::BaseForce,
            },
            intensity: Rational::new(n % 10 + 1, 10),
            direction,
            signal_tick: self.signal_tick,
            quality_score: Rational::one(),
        }
    }
    
    /// Integrate discovery from Voyager - modifies internal space
    pub fn integrate_discovery(&mut self, discovery: VoyagerDiscovery) {
        self.internal_space.integrate_discovery(discovery);
    }

    /// Calculate system preferences using exact weighted average
    pub fn calculate_system_preferences(&self) -> SystemPreferences {
        if self.deformation_buffer.is_empty() {
            return SystemPreferences::default();
        }

        let dim_count = self.internal_space.get_dimension_count().max(4);
        let mut weighted_direction = vec![Rational::zero(); dim_count];
        let mut total_weight = Rational::zero();

        for signal in &self.deformation_buffer {
            let weight = signal.intensity.mul(&signal.quality_score);
            total_weight = total_weight.add(&weight);
            
            for i in 0..dim_count.min(signal.direction.len()) {
                let contrib = signal.direction[i].mul(&weight);
                weighted_direction[i] = weighted_direction[i].add(&contrib);
            }
        }

        if !total_weight.is_zero() {
            for i in 0..dim_count {
                weighted_direction[i] = weighted_direction[i].div(&total_weight);
            }
        }

        SystemPreferences {
            affinities: weighted_direction,
        }
    }

    /// Cleanup old signals based on signal_tick age (NOT natural time)
    pub fn cleanup_expired_signals(&mut self, max_age_ticks: u64) {
        let current = self.signal_tick;
        self.deformation_buffer.retain(|signal| {
            current.saturating_sub(signal.signal_tick) < max_age_ticks
        });
    }

    /// Get resource usage
    pub fn get_resource_usage(&self) -> ResourceUsage {
        ResourceUsage {
            memory_bytes: self.deformation_buffer.len() * std::mem::size_of::<DeformationSignal>(),
            buffer_usage: self.deformation_buffer.len(),
            buffer_capacity: self.resource_limits.max_buffer_size,
        }
    }
    
    /// Get current dimension (integer)
    pub fn get_current_dimension(&self) -> usize {
        self.internal_space.dimensions
    }
    
    /// Get accumulated equation count
    pub fn get_equation_count(&self) -> usize {
        self.internal_space.get_equation_count()
    }
    
    /// Get logical time (signal tick count)
    pub fn get_logical_time(&self) -> u64 {
        self.signal_tick
    }
    
    /// ═══════════════════════════════════════════════════════════════
    /// Fate 自主评估系统
    ///
    /// 设计原则：
    /// 1. 第一轮 (signal_tick == 0) 为启动轮，返回空值
    /// 2. 质量和强度由 Fate 根据历史信号自主计算
    /// 3. 不预设任何值，完全从数据中学习
    /// ═══════════════════════════════════════════════════════════════
    pub fn evaluate_system(&self, system_index: usize) -> FateEvaluation {
        // 第一轮为启动轮，作废
        if self.signal_tick == 0 {
            return FateEvaluation {
                quality: Rational::zero(),
                intensity: Rational::zero(),
                is_bootstrap: true,
            };
        }
        
        // 从历史信号中学习该系统的表现
        let mut system_signals = Vec::new();
        for signal in &self.deformation_buffer {
            let sig_idx = match signal.source_system {
                SystemType::BlackHole => 0,
                SystemType::MeshBrain => 1,
                SystemType::MultiNova => 2,
                SystemType::BaseForce => 3,
            };
            if sig_idx == system_index {
                system_signals.push(signal);
            }
        }
        
        // 如果该系统没有历史信号，返回空值 (Fate 会在后续学习)
        if system_signals.is_empty() {
            return FateEvaluation {
                quality: Rational::zero(),
                intensity: Rational::zero(),
                is_bootstrap: false,
            };
        }
        
        // ═══════════════════════════════════════════════════════════════
        // Fate 自主计算质量和强度
        // 基于历史信号的自然涨落，而非预设公式
        // ═══════════════════════════════════════════════════════════════
        
        // 计算平均质量 (从历史信号学习)
        let mut total_quality = Rational::zero();
        let mut total_intensity = Rational::zero();
        let count = Rational::new(system_signals.len() as i128, 1);
        
        for signal in &system_signals {
            total_quality = total_quality.add(&signal.quality_score);
            total_intensity = total_intensity.add(&signal.intensity);
        }
        
        // 如果有历史数据，计算平均值
        // 如果历史数据的质量/强度也是空的，则使用信号涨落
        let avg_quality = if !total_quality.is_zero() {
            total_quality.div(&count)
        } else {
            // 从信号涨落中自然产生质量
            // 使用 signal_tick 和系统索引作为熵源
            let entropy = (self.signal_tick as i128 * (system_index as i128 + 1)) % 100;
            Rational::new(entropy.max(1), 100)
        };
        
        let avg_intensity = if !total_intensity.is_zero() {
            total_intensity.div(&count)
        } else {
            // 从信号计数中自然产生强度
            let signal_density = Rational::new(system_signals.len() as i128, self.signal_tick.max(1) as i128);
            signal_density
        };
        
        FateEvaluation {
            quality: avg_quality,
            intensity: avg_intensity,
            is_bootstrap: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rational_arithmetic() {
        let a = Rational::new(1, 2);
        let b = Rational::new(1, 3);
        assert_eq!(a.add(&b), Rational::new(5, 6));
        assert_eq!(a.mul(&b), Rational::new(1, 6));
    }
    
    #[test]
    fn test_signal_tick_based_time() {
        let limits = ResourceLimits {
            max_memory_mb: 100,
            max_cpu_percent: 50,
            max_buffer_size: 100,
            processing_interval_ms: 100,
        };
        let mut fate = FateSystem::new("test".to_string(), limits);
        
        // Time is signal tick, not natural time
        assert_eq!(fate.get_logical_time(), 0);
        
        fate.generate_signal();
        assert_eq!(fate.get_logical_time(), 1);
        
        fate.receive_signal(DeformationSignal {
            source_system: SystemType::BlackHole,
            intensity: Rational::one(),
            direction: vec![Rational::one()],
            signal_tick: 0,  // Will be overwritten
            quality_score: Rational::one(),
        });
        assert_eq!(fate.get_logical_time(), 2);
    }
    
    #[test]
    fn test_internal_space_evolution() {
        let mut space = InternalSpace::new(4);
        assert_eq!(space.get_dimension_count(), 4);
        
        // Integrate a discovery
        let discovery = VoyagerDiscovery {
            equation_id: "eq_1".into(),
            equation_type: "linear".into(),
            coefficients: vec![Rational::new(1, 2), Rational::one()],
            algebraically_verified: true,
            discovered_at: 0,  // signal tick based
        };
        
        space.integrate_discovery(discovery);
        assert_eq!(space.get_equation_count(), 1);
    }
    
    #[test]
    fn test_fate_with_discovery_integration() {
        let limits = ResourceLimits {
            max_memory_mb: 100,
            max_cpu_percent: 50,
            max_buffer_size: 100,
            processing_interval_ms: 100,
        };
        let mut fate = FateSystem::new("test".to_string(), limits);
        
        // Initial state
        assert_eq!(fate.get_equation_count(), 0);
        
        // Integrate discovery (generic test equation)
        fate.integrate_discovery(VoyagerDiscovery {
            equation_id: "eq_linear_1".into(),
            equation_type: "Linear".into(),
            coefficients: vec![Rational::new(1, 2), Rational::one()],
            algebraically_verified: true,
            discovered_at: 0,  // signal count based, not natural time
        });
        
        assert_eq!(fate.get_equation_count(), 1);
        
        // Generate signal influenced by internal space
        let signal = fate.generate_signal();
        assert!(!signal.direction.is_empty());
    }
}
