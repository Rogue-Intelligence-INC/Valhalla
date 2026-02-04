//! Valhalla Space - Global Adaptive Dimension Management
//!
//! Architecture: Space (global) ← Voyager ← Fate
//!
//! Space is the GLOBAL component that handles:
//! - Non-integer (fractional) dimension calculation
//! - Adaptive dimension expansion
//! - Container creation based on dimension analysis
//!
//! Time system: uses signal_tick (deformation signal count), NOT natural time

use serde::{Deserialize, Serialize};
use log::info;
use thiserror::Error;
use std::collections::HashMap;

pub use voyager::{DimensionManager, DimensionConfig, Rational};
pub use valhalla_mathematics::PassiveMathematicalLibrary;

/// Space error types
#[derive(Error, Debug)]
pub enum SpaceError {
    #[error("Container creation failed: {0}")]
    ContainerCreationError(String),
    #[error("Dimension error: {0}")]
    DimensionError(String),
}

// ============================================================================
// Fractional Dimension Calculator (GLOBAL component)
// ============================================================================

/// Dimension type
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DimensionType {
    Integer,
    Fractional,
    Complex,
    Auto,
}

/// Context for dimension calculation
#[derive(Debug, Clone)]
pub enum DimensionContext {
    ContainerCreation,
    SystemEvolution,
    SignalAnalysis,
}

/// Dimension calculation request
#[derive(Debug, Clone)]
pub struct DimensionRequest {
    pub data: Vec<Rational>,
    pub dimension_type: DimensionType,
    pub context: DimensionContext,
}

/// Dimension calculation result
#[derive(Debug, Clone)]
pub struct DimensionResult {
    pub recommended_dimension: Rational,
    pub dimension_type: DimensionType,
    pub mathematical_basis: String,
    pub hausdorff_estimate: Rational,
    pub box_estimate: Rational,
    pub info_estimate: Rational,
}

/// Fractional Dimension Calculator - GLOBAL adaptive component
/// Calculates Hausdorff, box, and information dimensions
#[derive(Debug, Clone)]
pub struct FractionalDimensionCalculator {
    /// Calculation count
    pub calculation_count: u64,
    /// Current global dimension (can be non-integer as Rational)
    pub current_dimension: Rational,
    /// Max dimension limit
    pub max_dimension: Rational,
}

impl FractionalDimensionCalculator {
    pub fn new() -> Self {
        Self { 
            calculation_count: 0,
            current_dimension: Rational::new(4, 1),  // Start at 4
            max_dimension: Rational::new(11, 1),     // Max 11
        }
    }
    
    /// Calculate optimal dimension (including non-integers)
    pub fn calculate_optimal_dimension(&mut self, request: &DimensionRequest) -> DimensionResult {
        self.calculation_count += 1;
        
        match request.dimension_type {
            DimensionType::Integer => self.calculate_integer_dimension(request),
            DimensionType::Fractional => self.calculate_fractional_dimension(request),
            DimensionType::Complex => self.calculate_fractional_dimension(request),
            DimensionType::Auto => self.auto_determine_dimension(request),
        }
    }
    
    /// Calculate integer dimension
    fn calculate_integer_dimension(&self, request: &DimensionRequest) -> DimensionResult {
        let dim = Rational::new(request.data.len().max(1) as i128, 1);
        DimensionResult {
            recommended_dimension: dim,
            dimension_type: DimensionType::Integer,
            mathematical_basis: format!("Integer dimension from data length: {}", request.data.len()),
            hausdorff_estimate: dim,
            box_estimate: dim,
            info_estimate: dim,
        }
    }
    
    /// Calculate fractional dimension using Hausdorff, box, and information estimates
    fn calculate_fractional_dimension(&self, request: &DimensionRequest) -> DimensionResult {
        if request.data.is_empty() {
            return DimensionResult {
                recommended_dimension: Rational::one(),
                dimension_type: DimensionType::Fractional,
                mathematical_basis: "Empty data, default dimension 1".into(),
                hausdorff_estimate: Rational::one(),
                box_estimate: Rational::one(),
                info_estimate: Rational::one(),
            };
        }
        
        // Calculate Hausdorff dimension estimate
        let hausdorff = self.estimate_hausdorff(&request.data);
        
        // Calculate box dimension estimate
        let box_dim = self.estimate_box_dimension(&request.data);
        
        // Calculate information dimension
        let info_dim = self.estimate_information_dimension(&request.data);
        
        // Combine: weighted average (hausdorff + 2*box + info) / 4
        let sum = hausdorff.add(&box_dim).add(&box_dim).add(&info_dim);
        let fractional_dim = sum.div(&Rational::new(4, 1));
        
        DimensionResult {
            recommended_dimension: fractional_dim,
            dimension_type: DimensionType::Fractional,
            mathematical_basis: format!(
                "Fractional analysis: Hausdorff={}, Box={}, Info={}",
                hausdorff, box_dim, info_dim
            ),
            hausdorff_estimate: hausdorff,
            box_estimate: box_dim,
            info_estimate: info_dim,
        }
    }
    
    /// Auto-determine best dimension type
    fn auto_determine_dimension(&self, request: &DimensionRequest) -> DimensionResult {
        self.calculate_fractional_dimension(request)
    }
    
    /// Estimate Hausdorff dimension from signal variance
    fn estimate_hausdorff(&self, data: &[Rational]) -> Rational {
        if data.len() < 2 {
            return Rational::one();
        }
        
        let n = data.len() as i128;
        let sum: Rational = data.iter().fold(Rational::zero(), |acc, x| acc.add(x));
        let mean = sum.div(&Rational::new(n, 1));
        
        let mut variance_sum = Rational::zero();
        for x in data {
            let diff = x.sub(&mean);
            variance_sum = variance_sum.add(&diff.mul(&diff));
        }
        let variance = variance_sum.div(&Rational::new(n, 1));
        
        // Base + variance modifier, clamped to [1, max]
        let base = Rational::one();
        let modifier = variance.abs();
        let raw = base.add(&modifier);
        
        self.clamp_dimension(raw)
    }
    
    /// Estimate box dimension from data range
    fn estimate_box_dimension(&self, data: &[Rational]) -> Rational {
        if data.len() < 2 {
            return Rational::one();
        }
        
        // Find range using Rational comparison
        let mut min = data[0];
        let mut max = data[0];
        for x in data.iter().skip(1) {
            if x.numerator * min.denominator < min.numerator * x.denominator {
                min = *x;
            }
            if x.numerator * max.denominator > max.numerator * x.denominator {
                max = *x;
            }
        }
        
        let range = max.sub(&min).abs();
        let n = Rational::new(data.len() as i128, 1);
        
        let base = Rational::one();
        if range.is_zero() {
            return base;
        }
        
        let normalized = n.div(&range.add(&Rational::one()));
        let raw = base.add(&normalized.div(&Rational::new(10, 1)));
        
        self.clamp_dimension(raw)
    }
    
    /// Estimate information dimension (entropy proxy)
    fn estimate_information_dimension(&self, data: &[Rational]) -> Rational {
        if data.is_empty() {
            return Rational::one();
        }
        
        // Count distinct values
        let mut distinct = 1i128;
        for i in 1..data.len() {
            if data[i] != data[i-1] {
                distinct += 1;
            }
        }
        
        let n = data.len() as i128;
        let ratio = Rational::new(distinct, n.max(1));
        
        Rational::one().add(&ratio)
    }
    
    /// Clamp dimension to valid range [1, max]
    fn clamp_dimension(&self, dim: Rational) -> Rational {
        let one = Rational::one();
        
        // Compare dim < 1
        if dim.numerator * one.denominator < one.numerator * dim.denominator {
            return one;
        }
        
        // Compare dim > max
        if dim.numerator * self.max_dimension.denominator > 
           self.max_dimension.numerator * dim.denominator {
            return self.max_dimension;
        }
        
        dim
    }
    
    /// Adaptive dimension expansion - updates global dimension if needed (only increase)
    pub fn adaptive_expand(&mut self, data: &[Rational]) -> Rational {
        let request = DimensionRequest {
            data: data.to_vec(),
            dimension_type: DimensionType::Auto,
            context: DimensionContext::SystemEvolution,
        };
        
        let result = self.calculate_optimal_dimension(&request);
        
        // Update if recommended is higher (conservative: only increase)
        if result.recommended_dimension.numerator * self.current_dimension.denominator >
           self.current_dimension.numerator * result.recommended_dimension.denominator {
            self.current_dimension = result.recommended_dimension;
        }
        
        self.current_dimension
    }
    
    /// Fate-driven dimension adjustment (can increase OR decrease)
    /// Called by Fate based on internal state analysis
    pub fn fate_driven_adjust(&mut self, data: &[Rational]) -> DimensionAdjustment {
        let request = DimensionRequest {
            data: data.to_vec(),
            dimension_type: DimensionType::Auto,
            context: DimensionContext::SystemEvolution,
        };
        
        let result = self.calculate_optimal_dimension(&request);
        let old_dimension = self.current_dimension;
        
        // Fate can adjust in BOTH directions
        self.current_dimension = result.recommended_dimension;
        
        let change = if result.recommended_dimension.numerator * old_dimension.denominator >
                       old_dimension.numerator * result.recommended_dimension.denominator {
            DimensionChange::Increased
        } else if result.recommended_dimension.numerator * old_dimension.denominator <
                  old_dimension.numerator * result.recommended_dimension.denominator {
            DimensionChange::Decreased
        } else {
            DimensionChange::Unchanged
        };
        
        DimensionAdjustment {
            old_dimension,
            new_dimension: self.current_dimension,
            change,
            mathematical_basis: result.mathematical_basis,
        }
    }
    
    /// Force set dimension (used by Fate for explicit control)
    pub fn set_dimension(&mut self, dimension: Rational) {
        self.current_dimension = self.clamp_dimension(dimension);
    }
    
    /// Get current global dimension
    pub fn get_current_dimension(&self) -> Rational {
        self.current_dimension
    }
}

/// Dimension change direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DimensionChange {
    Increased,
    Decreased,
    Unchanged,
}

/// Dimension adjustment result
#[derive(Debug, Clone)]
pub struct DimensionAdjustment {
    pub old_dimension: Rational,
    pub new_dimension: Rational,
    pub change: DimensionChange,
    pub mathematical_basis: String,
}

impl Default for FractionalDimensionCalculator {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Space Container
// ============================================================================

/// Space container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceContainer {
    pub container_id: String,
    pub dimensions: usize,
    /// Fractional dimension (Rational for precision)
    pub fractional_dimension: Option<Rational>,
    pub container_type: ContainerType,
    pub status: ContainerStatus,
    /// Creation tick (signal_tick based, NOT natural time)
    pub created_at_tick: u64,
}

/// Container types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContainerType {
    Basic,
    QuadSystem { weights: QuadSystemWeights },
    Adaptive,
    /// Fractional dimension container
    Fractional { dimension: Rational },
}

/// Quad system weights (Rational for exact)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuadSystemWeights {
    pub blackhole_weight: Rational,
    pub meshbrain_weight: Rational,
    pub multinova_weight: Rational,
    pub baseforce_weight: Rational,
}

/// Container status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContainerStatus {
    Initializing,
    Running,
    Paused,
    Error(String),
}

// ============================================================================
// Voyager Space Builder (Global Space Manager)
// ============================================================================

/// Voyager Space Builder - Global space management
#[derive(Debug)]
pub struct VoyagerSpaceBuilder {
    builder_id: String,
    dimension_manager: DimensionManager,
    /// GLOBAL fractional dimension calculator
    fractional_calculator: FractionalDimensionCalculator,
    mathematics_library: PassiveMathematicalLibrary,
    containers: HashMap<String, SpaceContainer>,
    fate_system: fate::FateSystem,
    /// Current signal tick (logical time)
    signal_tick: u64,
}

impl VoyagerSpaceBuilder {
    pub fn new() -> Result<Self, SpaceError> {
        info!("🚀 Initializing Voyager Space Builder (global dimension management)");

        let resource_limits = fate::ResourceLimits {
            max_memory_mb: 32768,
            max_cpu_percent: 80,
            max_buffer_size: 1000,
            processing_interval_ms: 100,
        };

        Ok(Self {
            builder_id: format!("space_{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() % 1_000_000),
            dimension_manager: DimensionManager::new(DimensionConfig::default()),
            fractional_calculator: FractionalDimensionCalculator::new(),
            mathematics_library: PassiveMathematicalLibrary::new(),
            containers: HashMap::new(),
            fate_system: fate::FateSystem::new("space_builder".to_string(), resource_limits),
            signal_tick: 0,
        })
    }
    
    /// Create container (uses signal_tick for time)
    pub fn create_container(&mut self, container_type: ContainerType) -> Result<String, SpaceError> {
        self.signal_tick += 1;
        
        let container_id = format!("container_{}_{}", self.signal_tick, self.containers.len());
        let container = SpaceContainer {
            container_id: container_id.clone(),
            dimensions: self.dimension_manager.get_current_dimensions(),
            fractional_dimension: Some(self.fractional_calculator.get_current_dimension()),
            container_type,
            status: ContainerStatus::Running,
            created_at_tick: self.signal_tick,
        };
        self.containers.insert(container_id.clone(), container);
        info!("📦 Created container: {}", container_id);
        Ok(container_id)
    }
    
    /// Create fractional dimension container
    pub fn create_fractional_container(&mut self, data: &[Rational]) -> Result<String, SpaceError> {
        self.signal_tick += 1;
        
        // Use global calculator to determine optimal dimension
        let dimension = self.fractional_calculator.adaptive_expand(data);
        
        let container_id = format!("frac_container_{}_{}", self.signal_tick, self.containers.len());
        let container = SpaceContainer {
            container_id: container_id.clone(),
            dimensions: (dimension.numerator / dimension.denominator) as usize,
            fractional_dimension: Some(dimension),
            container_type: ContainerType::Fractional { dimension },
            status: ContainerStatus::Running,
            created_at_tick: self.signal_tick,
        };
        self.containers.insert(container_id.clone(), container);
        info!("📦 Created fractional container: {} (dim={})", container_id, dimension);
        Ok(container_id)
    }
    
    /// Adaptive dimension expansion (global, only increase)
    pub fn adaptive_dimension_expansion(&mut self, data: &[Rational]) -> Rational {
        self.fractional_calculator.adaptive_expand(data)
    }
    
    /// Fate-driven dimension adjustment (can increase OR decrease)
    pub fn fate_driven_adjustment(&mut self, data: &[Rational]) -> DimensionAdjustment {
        self.fractional_calculator.fate_driven_adjust(data)
    }
    
    /// On discovery: create container for unknown phenomenon
    /// Called by Fate when unknown signal pattern is detected
    pub fn on_discovery(&mut self, phenomenon_data: &[Rational], phenomenon_id: &str) -> Result<DiscoveryResult, SpaceError> {
        self.signal_tick += 1;
        
        // Fate-driven adjustment (can increase OR decrease)
        let adjustment = self.fractional_calculator.fate_driven_adjust(phenomenon_data);
        
        // Create new container for this discovery
        let container_id = format!("discovery_{}_{}", phenomenon_id, self.signal_tick);
        let container = SpaceContainer {
            container_id: container_id.clone(),
            dimensions: (adjustment.new_dimension.numerator 
                        / adjustment.new_dimension.denominator) as usize,
            fractional_dimension: Some(adjustment.new_dimension),
            container_type: ContainerType::Fractional { dimension: adjustment.new_dimension },
            status: ContainerStatus::Running,
            created_at_tick: self.signal_tick,
        };
        self.containers.insert(container_id.clone(), container);
        info!("🔬 Discovery container created: {} (dim={})", container_id, adjustment.new_dimension);
        
        Ok(DiscoveryResult {
            container_id,
            dimension_adjustment: adjustment,
            signal_tick: self.signal_tick,
        })
    }
    
    /// Get current global fractional dimension
    pub fn get_global_dimension(&self) -> Rational {
        self.fractional_calculator.get_current_dimension()
    }
    
    /// Set global dimension explicitly (Fate control)
    pub fn set_global_dimension(&mut self, dimension: Rational) {
        self.fractional_calculator.set_dimension(dimension);
    }
    
    /// Get Fate system
    pub fn fate_system(&self) -> &fate::FateSystem {
        &self.fate_system
    }
    
    /// Get mutable Fate system
    pub fn fate_system_mut(&mut self) -> &mut fate::FateSystem {
        &mut self.fate_system
    }
    
    /// Get mathematics library
    pub fn mathematics_library(&self) -> &PassiveMathematicalLibrary {
        &self.mathematics_library
    }
    
    /// Get dimension manager
    pub fn dimension_manager(&self) -> &DimensionManager {
        &self.dimension_manager
    }
    
    /// Get mutable dimension manager
    pub fn dimension_manager_mut(&mut self) -> &mut DimensionManager {
        &mut self.dimension_manager
    }
    
    /// Get fractional calculator
    pub fn fractional_calculator(&self) -> &FractionalDimensionCalculator {
        &self.fractional_calculator
    }
    
    /// Get logical time (signal tick)
    pub fn get_logical_time(&self) -> u64 {
        self.signal_tick
    }
}

/// Mathematical configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MathematicalConfig {
    pub enable_mathematics_library: bool,
    pub enable_unknown_pattern_detection: bool,
    pub mathematical_coverage: MathematicalCoverage,
}

impl Default for MathematicalConfig {
    fn default() -> Self {
        Self {
            enable_mathematics_library: true,
            enable_unknown_pattern_detection: true,
            mathematical_coverage: MathematicalCoverage::Complete,
        }
    }
}

/// Mathematical coverage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MathematicalCoverage {
    Basic,
    Advanced,
    Complete,
}

/// Fate configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FateConfig {
    pub enable_fate_decisions: bool,
    pub auto_preference_generation: bool,
    /// Interval in signal ticks (NOT ms)
    pub decision_interval_ticks: u64,
}

impl Default for FateConfig {
    fn default() -> Self {
        Self {
            enable_fate_decisions: true,
            auto_preference_generation: true,
            decision_interval_ticks: 100,
        }
    }
}

/// Discovery result
#[derive(Debug, Clone)]
pub struct DiscoveryResult {
    pub container_id: String,
    pub dimension_adjustment: DimensionAdjustment,
    pub signal_tick: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fractional_dimension_calculator() {
        let mut calc = FractionalDimensionCalculator::new();
        
        let request = DimensionRequest {
            data: vec![
                Rational::new(1, 2),
                Rational::new(1, 1),
                Rational::new(3, 2),
            ],
            dimension_type: DimensionType::Auto,
            context: DimensionContext::SignalAnalysis,
        };
        
        let result = calc.calculate_optimal_dimension(&request);
        // Dimension should be in valid range
        assert!(result.recommended_dimension.numerator >= result.recommended_dimension.denominator);
    }
    
    #[test]
    fn test_adaptive_expand() {
        let mut calc = FractionalDimensionCalculator::new();
        
        let initial = calc.get_current_dimension();
        
        // Expand with data
        let data = vec![
            Rational::new(1, 1),
            Rational::new(10, 1),
            Rational::new(100, 1),
        ];
        
        let new_dim = calc.adaptive_expand(&data);
        
        // Should have updated
        assert!(new_dim.numerator * initial.denominator >= initial.numerator * new_dim.denominator);
    }
    
    #[test]
    fn test_fate_driven_adjust() {
        let mut calc = FractionalDimensionCalculator::new();
        
        // High variance data → adjust
        let high_var_data = vec![
            Rational::new(1, 1),
            Rational::new(100, 1),
            Rational::new(-50, 1),
        ];
        let adj1 = calc.fate_driven_adjust(&high_var_data);
        println!("After high-var: {:?}", adj1.change);
        
        // Low variance data → should decrease
        let low_var_data = vec![Rational::one(), Rational::one()];
        let adj2 = calc.fate_driven_adjust(&low_var_data);
        println!("After low-var: {:?}", adj2.change);
        
        // Fate-driven can decrease
        assert_eq!(adj2.change, DimensionChange::Decreased);
    }
    
    #[test]
    fn test_signal_tick_based_time() {
        let mut builder = VoyagerSpaceBuilder::new().unwrap();
        
        assert_eq!(builder.get_logical_time(), 0);
        
        let _ = builder.create_container(ContainerType::Basic);
        assert_eq!(builder.get_logical_time(), 1);
        
        let _ = builder.create_fractional_container(&[Rational::one()]);
        assert_eq!(builder.get_logical_time(), 2);
    }
    
    #[test]
    fn test_on_discovery() {
        let mut builder = VoyagerSpaceBuilder::new().unwrap();
        
        let data = vec![Rational::new(1, 1), Rational::new(2, 1)];
        let result = builder.on_discovery(&data, "test_pattern").unwrap();
        
        assert!(result.container_id.contains("discovery_test_pattern"));
        assert_eq!(result.signal_tick, 1);
    }
}
