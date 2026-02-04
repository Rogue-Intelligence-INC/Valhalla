//! StemHall - Intelligent Load Allocation (f64 Performance Version)
//!
//! Smart workload distribution
//! Uses f64 for high-performance computation

use serde::{Deserialize, Serialize};

/// StemHall configuration (f64)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StemHallConfig {
    pub node_count: usize,
    pub load_balance_threshold: f64,
    pub max_queue_size: usize,
    pub health_check_interval_ms: u64,
}

impl Default for StemHallConfig {
    fn default() -> Self {
        Self {
            node_count: 4,
            load_balance_threshold: 0.7,
            max_queue_size: 1000,
            health_check_interval_ms: 100,
        }
    }
}

/// Work node status (f64)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatus {
    pub node_id: String,
    pub current_load: f64,
    pub queue_size: usize,
    pub is_healthy: bool,
}

/// Task allocation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationResult {
    pub task_id: String,
    pub assigned_node: String,
    pub priority: f64,
    pub estimated_time_ms: u64,
}

/// StemHall load allocation system (f64)
#[derive(Debug)]
pub struct StemHall {
    pub config: StemHallConfig,
    pub nodes: Vec<NodeStatus>,
    pub total_allocations: u64,
}

impl StemHall {
    pub fn new(config: StemHallConfig) -> Self {
        let nodes: Vec<NodeStatus> = (0..config.node_count)
            .map(|i| NodeStatus {
                node_id: format!("node_{}", i),
                current_load: 0.0,
                queue_size: 0,
                is_healthy: true,
            })
            .collect();
        
        Self {
            config,
            nodes,
            total_allocations: 0,
        }
    }
    
    /// Allocate task to optimal node (f64)
    pub fn allocate_task(&mut self, task_id: String, load_weight: f64) -> AllocationResult {
        self.total_allocations += 1;
        
        let threshold = self.config.load_balance_threshold;
        
        // Find lowest-load healthy node below threshold
        let best_node = self.nodes
            .iter_mut()
            .filter(|n| n.is_healthy && n.current_load < threshold)
            .min_by(|a, b| a.current_load.partial_cmp(&b.current_load).unwrap())
            .map(|n| {
                n.current_load += load_weight;
                n.queue_size += 1;
                n.node_id.clone()
            })
            .unwrap_or_else(|| {
                // All nodes busy, pick lowest load
                let node = self.nodes
                    .iter_mut()
                    .min_by(|a, b| a.current_load.partial_cmp(&b.current_load).unwrap())
                    .unwrap();
                node.current_load += load_weight;
                node.queue_size += 1;
                node.node_id.clone()
            });
        
        let priority = (1.0 - load_weight).max(0.0);
        let est_time = (load_weight * 100.0) as u64;
        
        AllocationResult {
            task_id,
            assigned_node: best_node,
            priority,
            estimated_time_ms: est_time,
        }
    }
    
    /// Complete task and release resources
    pub fn complete_task(&mut self, node_id: &str, load_weight: f64) {
        if let Some(node) = self.nodes.iter_mut().find(|n| n.node_id == node_id) {
            node.current_load = (node.current_load - load_weight).max(0.0);
            node.queue_size = node.queue_size.saturating_sub(1);
        }
    }
    
    /// Health check
    pub fn health_check(&mut self) {
        let max_load = self.config.load_balance_threshold * 1.5;
        
        for node in &mut self.nodes {
            node.is_healthy = node.current_load < max_load 
                && node.queue_size < self.config.max_queue_size;
        }
    }
    
    /// Get system load (f64 average)
    pub fn get_system_load(&self) -> f64 {
        if self.nodes.is_empty() {
            return 0.0;
        }
        self.nodes.iter().map(|n| n.current_load).sum::<f64>() / self.nodes.len() as f64
    }
}

impl Default for StemHall {
    fn default() -> Self {
        Self::new(StemHallConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_allocation() {
        let mut system = StemHall::default();
        
        let result = system.allocate_task("task_1".into(), 0.25);
        assert!(!result.assigned_node.is_empty());
        assert_eq!(system.total_allocations, 1);
    }
    
    #[test]
    fn test_complete_task() {
        let mut system = StemHall::default();
        
        let load = 0.5;
        let result = system.allocate_task("task_1".into(), load);
        
        system.complete_task(&result.assigned_node, load);
        
        let node = system.nodes.iter()
            .find(|n| n.node_id == result.assigned_node)
            .unwrap();
        assert!(node.current_load < 0.01); // Nearly zero
    }
    
    #[test]
    fn test_system_load() {
        let mut system = StemHall::default();
        
        assert!(system.get_system_load() < 0.01);
        
        system.allocate_task("t1".into(), 0.25);
        system.allocate_task("t2".into(), 0.25);
        
        let load = system.get_system_load();
        assert!(load > 0.0);
    }
}
