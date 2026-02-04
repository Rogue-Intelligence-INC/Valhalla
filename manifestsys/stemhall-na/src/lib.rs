//! StemHall-NA - Intelligent Load Allocation (Algebraic Version)
//!
//! Smart workload distribution based on Fate signals
//! Uses exact Rational arithmetic - NO f64

use serde::{Deserialize, Serialize};
use fate::{
    FateSystem, DeformationSignal, SystemType, ResourceLimits,
    Rational, VoyagerDiscovery,
};

/// StemHall-NA configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StemHallConfig {
    pub node_count: usize,
    /// Load balance threshold as Rational
    pub load_balance_threshold: Rational,
    pub max_queue_size: usize,
    pub health_check_interval_ms: u64,
}

impl Default for StemHallConfig {
    fn default() -> Self {
        Self {
            node_count: 4,
            load_balance_threshold: Rational::new(7, 10),  // 0.7
            max_queue_size: 1000,
            health_check_interval_ms: 100,
        }
    }
}

/// Work node status (all Rational)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatus {
    pub node_id: String,
    pub current_load: Rational,
    pub queue_size: usize,
    pub is_healthy: bool,
}

/// Task allocation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationResult {
    pub task_id: String,
    pub assigned_node: String,
    pub priority: Rational,
    pub estimated_time_ms: u64,
}

/// StemHall-NA load allocation system
#[derive(Debug)]
pub struct StemHallNA {
    pub config: StemHallConfig,
    pub nodes: Vec<NodeStatus>,
    pub fate_system: FateSystem,
    pub total_allocations: u64,
}

impl StemHallNA {
    pub fn new(config: StemHallConfig) -> Self {
        let resource_limits = ResourceLimits {
            max_memory_mb: 8192,
            max_cpu_percent: 40,
            max_buffer_size: 500,
            processing_interval_ms: 20,
        };
        
        let nodes: Vec<NodeStatus> = (0..config.node_count)
            .map(|i| NodeStatus {
                node_id: format!("node_{}", i),
                current_load: Rational::zero(),
                queue_size: 0,
                is_healthy: true,
            })
            .collect();
        
        Self {
            config,
            nodes,
            fate_system: FateSystem::new("stemhall_na".to_string(), resource_limits),
            total_allocations: 0,
        }
    }
    
    /// Allocate task to optimal node (all Rational comparisons)
    pub fn allocate_task(&mut self, task_id: String, load_weight: Rational) -> AllocationResult {
        self.total_allocations += 1;
        
        // Find lowest-load healthy node below threshold
        let threshold = &self.config.load_balance_threshold;
        
        let best_node = self.nodes
            .iter_mut()
            .filter(|n| {
                n.is_healthy && 
                n.current_load.numerator * threshold.denominator 
                    < threshold.numerator * n.current_load.denominator
            })
            .min_by(|a, b| {
                // Compare loads: a < b iff a.num * b.den < b.num * a.den
                let cmp = a.current_load.numerator * b.current_load.denominator;
                let cmp2 = b.current_load.numerator * a.current_load.denominator;
                cmp.cmp(&cmp2)
            })
            .map(|n| {
                n.current_load = n.current_load.add(&load_weight);
                n.queue_size += 1;
                n.node_id.clone()
            })
            .unwrap_or_else(|| {
                // All nodes busy, pick lowest load
                let node = self.nodes
                    .iter_mut()
                    .min_by(|a, b| {
                        let cmp = a.current_load.numerator * b.current_load.denominator;
                        let cmp2 = b.current_load.numerator * a.current_load.denominator;
                        cmp.cmp(&cmp2)
                    })
                    .unwrap();
                node.current_load = node.current_load.add(&load_weight);
                node.queue_size += 1;
                node.node_id.clone()
            });
        
        // Send Fate signal
        let signal = DeformationSignal {
            source_system: SystemType::BaseForce,
            intensity: load_weight,
            direction: vec![Rational::zero(), Rational::zero(), Rational::zero(), Rational::one()],
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            quality_score: if load_weight.numerator * 2 < load_weight.denominator {
                Rational::new(9, 10)  // 0.9 for light loads
            } else {
                Rational::new(7, 10)  // 0.7 for heavy loads
            },
        };
        self.fate_system.receive_signal(signal);
        
        // Priority = 1 - load_weight (clamped)
        let priority = if load_weight.numerator > load_weight.denominator {
            Rational::zero()
        } else {
            Rational::one().sub(&load_weight)
        };
        
        // Estimated time (approximate)
        let est_time = (load_weight.numerator * 100 / load_weight.denominator.max(1)) as u64;
        
        AllocationResult {
            task_id,
            assigned_node: best_node,
            priority,
            estimated_time_ms: est_time,
        }
    }
    
    /// Complete task and release resources
    pub fn complete_task(&mut self, node_id: &str, load_weight: Rational) {
        if let Some(node) = self.nodes.iter_mut().find(|n| n.node_id == node_id) {
            node.current_load = node.current_load.sub(&load_weight);
            // Clamp to zero
            if node.current_load.numerator < 0 {
                node.current_load = Rational::zero();
            }
            node.queue_size = node.queue_size.saturating_sub(1);
        }
    }
    
    /// Health check
    pub fn health_check(&mut self) {
        let threshold = &self.config.load_balance_threshold;
        // threshold * 1.5 = threshold * 3/2
        let max_load_num = threshold.numerator * 3;
        let max_load_den = threshold.denominator * 2;
        
        for node in &mut self.nodes {
            node.is_healthy = 
                node.current_load.numerator * max_load_den < max_load_num * node.current_load.denominator
                && node.queue_size < self.config.max_queue_size;
        }
    }
    
    /// Get system load (Rational average)
    pub fn get_system_load(&self) -> Rational {
        if self.nodes.is_empty() {
            return Rational::zero();
        }
        let total: Rational = self.nodes.iter()
            .fold(Rational::zero(), |acc, n| acc.add(&n.current_load));
        total.div(&Rational::new(self.nodes.len() as i128, 1))
    }
    
    /// Integrate discovery
    pub fn integrate_discovery(&mut self, discovery: VoyagerDiscovery) {
        self.fate_system.integrate_discovery(discovery);
    }
    
    /// Get Fate preferences
    pub fn get_fate_preferences(&self) -> fate::SystemPreferences {
        self.fate_system.calculate_system_preferences()
    }
    
    /// Get Fate dimension
    pub fn get_fate_dimension(&self) -> Rational {
        self.fate_system.get_current_dimension()
    }
}

impl Default for StemHallNA {
    fn default() -> Self {
        Self::new(StemHallConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_allocation() {
        let mut system = StemHallNA::default();
        
        let result = system.allocate_task("task_1".into(), Rational::new(1, 4));
        assert!(!result.assigned_node.is_empty());
        assert_eq!(system.total_allocations, 1);
    }
    
    #[test]
    fn test_complete_task() {
        let mut system = StemHallNA::default();
        
        let load = Rational::new(1, 2);
        let result = system.allocate_task("task_1".into(), load);
        
        system.complete_task(&result.assigned_node, load);
        
        // Node should be back to zero load
        let node = system.nodes.iter()
            .find(|n| n.node_id == result.assigned_node)
            .unwrap();
        assert_eq!(node.current_load, Rational::zero());
    }
    
    #[test]
    fn test_system_load() {
        let mut system = StemHallNA::default();
        
        // Initial load should be zero
        assert_eq!(system.get_system_load(), Rational::zero());
        
        // Add some load
        system.allocate_task("t1".into(), Rational::new(1, 4));
        system.allocate_task("t2".into(), Rational::new(1, 4));
        
        let load = system.get_system_load();
        assert!(load.numerator > 0);
    }
}
