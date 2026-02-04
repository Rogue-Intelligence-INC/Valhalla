//! Stem-Cell-Optimized - Intelligent Cell Network (Algebraic Version)
//!
//! Self-adaptive cell network evolution based on Fate signals
//! Uses exact Rational arithmetic - NO f64

use serde::{Deserialize, Serialize};
use log::info;
use fate::{
    FateSystem, DeformationSignal, SystemType, ResourceLimits,
    SystemPreferences, Rational, VoyagerDiscovery,
};

/// Cell network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StemCellConfig {
    pub initial_cell_count: usize,
    pub max_cell_count: usize,
    /// Division threshold as Rational
    pub division_threshold: Rational,
    /// Death threshold as Rational
    pub death_threshold: Rational,
    /// Mutation rate as Rational
    pub mutation_rate: Rational,
}

impl Default for StemCellConfig {
    fn default() -> Self {
        Self {
            initial_cell_count: 100,
            max_cell_count: 10000,
            division_threshold: Rational::new(7, 10),  // 0.7
            death_threshold: Rational::new(2, 10),     // 0.2
            mutation_rate: Rational::new(1, 100),      // 0.01
        }
    }
}

/// Cell state (all Rational)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    pub cell_id: String,
    pub energy: Rational,
    pub age: u64,
    pub generation: u32,
    pub specialization: CellSpecialization,
}

/// Cell specialization types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CellSpecialization {
    Stem,           // Stem cell
    Sensory,        // Sensory cell
    Processing,     // Processing cell
    Memory,         // Memory cell
    Coordinator,    // Coordinator cell
}

/// Stem cell network
#[derive(Debug)]
pub struct StemCell {
    pub id: String,
    pub config: StemCellConfig,
    pub cells: Vec<Cell>,
    pub fate_system: FateSystem,
    pub generation_count: u32,
    pub total_divisions: u64,
    pub total_deaths: u64,
    /// RNG state (deterministic from seed)
    rng_state: u64,
}

impl StemCell {
    pub fn new() -> Self {
        Self::with_config(StemCellConfig::default())
    }
    
    pub fn with_config(config: StemCellConfig) -> Self {
        info!("🧬 Initializing stem cell network");
        
        let resource_limits = ResourceLimits {
            max_memory_mb: 4096,
            max_cpu_percent: 30,
            max_buffer_size: 500,
            processing_interval_ms: 20,
        };
        
        let cells: Vec<Cell> = (0..config.initial_cell_count)
            .map(|i| Cell {
                cell_id: format!("cell_{}", i),
                energy: Rational::new(1, 2),  // 0.5
                age: 0,
                generation: 0,
                specialization: CellSpecialization::Stem,
            })
            .collect();
        
        Self {
            id: format!("stemcell_{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() % 1_000_000),
            config,
            cells,
            fate_system: FateSystem::new("stem_cell".to_string(), resource_limits),
            generation_count: 0,
            total_divisions: 0,
            total_deaths: 0,
            rng_state: 12345,
        }
    }
    
    /// Deterministic RNG (no floating point)
    fn next_random(&mut self) -> Rational {
        // LCG: state = (a * state + c) mod m
        self.rng_state = self.rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        // Return value in [0, 1)
        Rational::new((self.rng_state % 1000) as i128, 1000)
    }
    
    /// Evolve one generation (all Rational)
    pub fn evolve(&mut self, environment_signal: Rational) -> EvolutionResult {
        self.generation_count += 1;
        
        let mut divisions = 0u64;
        let mut deaths = 0u64;
        let mut new_cells = Vec::new();
        
        let death_threshold = self.config.death_threshold;
        let division_threshold = self.config.division_threshold;
        let max_cell_count = self.config.max_cell_count;
        let mutation_rate = self.config.mutation_rate;
        let current_count = self.cells.len();
        
        // Process existing cells
        self.cells.retain_mut(|cell| {
            cell.age += 1;
            // energy += env * 0.1
            cell.energy = cell.energy.add(&environment_signal.mul(&Rational::new(1, 10)));
            // energy *= 0.99 (natural decay)
            cell.energy = cell.energy.mul(&Rational::new(99, 100));
            
            // Cell death
            if cell.energy.numerator * death_threshold.denominator 
               < death_threshold.numerator * cell.energy.denominator {
                deaths += 1;
                return false;
            }
            
            // Cell division
            if cell.energy.numerator * division_threshold.denominator 
               > division_threshold.numerator * cell.energy.denominator 
               && current_count + new_cells.len() < max_cell_count {
                // energy /= 2
                cell.energy = cell.energy.div(&Rational::new(2, 1));
                
                new_cells.push(Cell {
                    cell_id: format!("cell_{}_{}", self.generation_count, new_cells.len()),
                    energy: cell.energy,
                    age: 0,
                    generation: cell.generation + 1,
                    specialization: self.specialize_cell(&cell.specialization, mutation_rate),
                });
                divisions += 1;
            }
            
            true
        });
        
        self.cells.extend(new_cells);
        self.total_divisions += divisions;
        self.total_deaths += deaths;
        
        // Create Fate signal (all Rational)
        let intensity = Rational::new(self.cells.len() as i128, self.config.max_cell_count as i128);
        let quality = if deaths < divisions {
            Rational::new(8, 10)  // 0.8
        } else {
            Rational::new(4, 10)  // 0.4
        };
        
        let signal = DeformationSignal {
            source_system: SystemType::MeshBrain,
            intensity,
            direction: vec![Rational::zero(), Rational::one(), Rational::zero(), Rational::zero()],
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            quality_score: quality,
        };
        self.fate_system.receive_signal(signal);
        
        // Calculate average energy
        let total_energy: Rational = self.cells.iter()
            .fold(Rational::zero(), |acc, c| acc.add(&c.energy));
        let average_energy = if self.cells.is_empty() {
            Rational::zero()
        } else {
            total_energy.div(&Rational::new(self.cells.len() as i128, 1))
        };
        
        EvolutionResult {
            generation: self.generation_count,
            cell_count: self.cells.len(),
            divisions,
            deaths,
            average_energy,
        }
    }
    
    /// Specialize cell based on mutation rate (deterministic)
    fn specialize_cell(&mut self, parent_spec: &CellSpecialization, mutation_rate: Rational) -> CellSpecialization {
        let rand_val = self.next_random();
        
        // Check if mutation occurs
        if rand_val.numerator * mutation_rate.denominator 
           < mutation_rate.numerator * rand_val.denominator {
            // Mutate to random type
            match (self.rng_state % 5) as usize {
                0 => CellSpecialization::Stem,
                1 => CellSpecialization::Sensory,
                2 => CellSpecialization::Processing,
                3 => CellSpecialization::Memory,
                _ => CellSpecialization::Coordinator,
            }
        } else {
            parent_spec.clone()
        }
    }
    
    /// Integrate discovery from Voyager
    pub fn integrate_discovery(&mut self, discovery: VoyagerDiscovery) {
        self.fate_system.integrate_discovery(discovery);
    }
    
    /// Get Fate preferences
    pub fn get_fate_preferences(&self) -> SystemPreferences {
        self.fate_system.calculate_system_preferences()
    }
    
    /// Get current dimension
    pub fn get_fate_dimension(&self) -> Rational {
        self.fate_system.get_current_dimension()
    }
    
    /// Get network status
    pub fn get_status(&self) -> StemCellStatus {
        StemCellStatus {
            network_id: self.id.clone(),
            cell_count: self.cells.len(),
            generation: self.generation_count,
            total_divisions: self.total_divisions,
            total_deaths: self.total_deaths,
            fate_dimension: self.fate_system.get_current_dimension(),
            fate_equations: self.fate_system.get_equation_count(),
        }
    }
}

impl Default for StemCell {
    fn default() -> Self {
        Self::new()
    }
}

/// Evolution result (all Rational)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionResult {
    pub generation: u32,
    pub cell_count: usize,
    pub divisions: u64,
    pub deaths: u64,
    pub average_energy: Rational,
}

/// Network status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StemCellStatus {
    pub network_id: String,
    pub cell_count: usize,
    pub generation: u32,
    pub total_divisions: u64,
    pub total_deaths: u64,
    pub fate_dimension: Rational,
    pub fate_equations: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_stem_cell_evolution() {
        let mut network = StemCell::new();
        
        // Evolve for 10 generations
        for _ in 0..10 {
            let result = network.evolve(Rational::new(1, 2));
            assert!(result.cell_count > 0);
        }
        
        let status = network.get_status();
        assert_eq!(status.generation, 10);
    }
    
    #[test]
    fn test_discovery_integration() {
        let mut network = StemCell::new();
        
        network.integrate_discovery(VoyagerDiscovery {
            equation_id: "test_eq".into(),
            equation_type: "linear".into(),
            coefficients: vec![Rational::one(), Rational::new(1, 2)],
            algebraically_verified: true,
            discovered_at: 0,
        });
        
        assert_eq!(network.get_status().fate_equations, 1);
    }
}
