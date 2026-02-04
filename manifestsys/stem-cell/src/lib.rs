//! Stem-Cell - Intelligent Cell Network (f64 Performance Version)
//!
//! Self-adaptive cell network evolution with Quad-System integration
//! Uses f64 for high-performance computation
//!
//! 四元系统整合:
//! - BlackHole: 细胞记忆存储 (memory_store)
//! - MeshBrain: 细胞间信号连接 (signal_mesh)
//! - MultiNova: 并行细胞活动 (parallel_activity)
//! - BaseForce: 四元协调 (coordination)
//!
//! 完整版本 (stemcell_complete):
//! - TripleFate: 多层 Fate 监控
//! - VoyagerSpace: 自动维度扩展
//! - MicroContainer: 小容器 → 局部收敛 → 细胞

// 完整版本 (带 Triple-Fate + VoyagerSpace)
pub mod stemcell_complete;
pub use stemcell_complete::{
    StemCellComplete, StemCellConfig as StemCellCompleteConfig,
    MicroContainer, LocalConvergenceState,
    DataFlowResult, BatchFlowResult,
    EvolutionResult as CompleteEvolutionResult,
    StemCellStatus as CompleteStatus,
    Cell as CompleteCell,  // stemcell_complete 的 Cell
};

use serde::{Deserialize, Serialize};
use log::info;

// 四元系统
use valhalla_metasystems_blackhole::{GravityField, GravityFieldConfig};
use valhalla_metasystems_meshbrain::{MeshBrainSystem, MeshBrainConfig, MeshBrainDeformationSignal};
use valhalla_metasystems_multinova::{MultinovaProcessor, MultinovaConfig};
use valhalla_metasystems_baseforce::BaseForceCoordinator;

/// Cell network configuration (f64)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StemCellConfig {
    pub initial_cell_count: usize,
    pub max_cell_count: usize,
    pub division_threshold: f64,
    pub death_threshold: f64,
    pub mutation_rate: f64,
}

impl Default for StemCellConfig {
    fn default() -> Self {
        Self {
            initial_cell_count: 100,
            max_cell_count: 10000,
            division_threshold: 0.7,
            death_threshold: 0.2,
            mutation_rate: 0.01,
        }
    }
}

/// Cell state (f64) - 整合四元系统
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    pub cell_id: String,
    pub energy: f64,
    pub age: u64,
    pub generation: u32,
    pub specialization: CellSpecialization,
    
    // 四元系统状态
    /// BlackHole 引力场强度 (记忆密度)
    pub memory_density: f64,
    /// MeshBrain 连接强度 (信号网络)
    pub signal_strength: f64,
    /// MultiNova 活动级别 (并行处理)
    pub activity_level: f64,
    /// BaseForce 协调值 (系统协调)
    pub coordination: f64,
    
    // 组织形态
    /// 位置坐标 (组织中的位置)
    pub position: [f64; 3],
    /// 邻居细胞 ID
    pub neighbors: Vec<String>,
    /// 层级 (组织层)
    pub layer: TissueLayer,
}

/// Cell specialization types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CellSpecialization {
    Stem,
    Sensory,
    Processing,
    Memory,
    Coordinator,
}

/// 组织层 - 由 Fate 代数自然演化决定，不预设类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum TissueLayer {
    #[default]
    Undetermined,  // 未确定 - 等待 Fate 代数触发
}

/// Stem cell network (f64) - 整合四元系统外壳
#[derive(Debug)]
pub struct StemCell {
    pub id: String,
    pub config: StemCellConfig,
    pub cells: Vec<Cell>,
    pub generation_count: u32,
    pub total_divisions: u64,
    pub total_deaths: u64,
    rng_state: u64,
    
    // 四元系统实例 (外壳容器内部)
    /// BlackHole - 细胞记忆/数据压缩
    pub blackhole: GravityField,
    /// MeshBrain - 细胞间信号网络
    pub meshbrain: MeshBrainSystem,
    /// MultiNova - 并行细胞活动处理
    pub multinova: MultinovaProcessor,
    /// BaseForce - 四元系统协调
    pub baseforce: BaseForceCoordinator,
    
    // 组织统计
    /// 组织层统计
    pub tissue_stats: TissueStats,
}

/// 组织统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// 组织统计 - 不预设类别，只记录四元系统状态
pub struct TissueStats {
    pub cell_count: usize,
    pub total_connections: usize,
    pub avg_signal_strength: f64,
    // 四元系统状态统计
    pub avg_memory_density: f64,
    pub avg_activity_level: f64,
    pub avg_coordination: f64,
}

impl StemCell {
    pub fn new() -> Self {
        Self::with_config(StemCellConfig::default())
    }
    
    pub fn with_config(config: StemCellConfig) -> Self {
        info!("🧬 Initializing stem cell network (f64) with Quad-System shell");
        
        // 初始化四元系统
        let blackhole = GravityField::new(GravityFieldConfig::default());
        let meshbrain = MeshBrainSystem::new(MeshBrainConfig::default());
        let multinova = MultinovaProcessor::new(MultinovaConfig::default());
        let baseforce = BaseForceCoordinator::new();
        
        let cells: Vec<Cell> = (0..config.initial_cell_count)
            .map(|i| {
                // 计算初始位置 (球形分布)
                let angle = (i as f64) * 2.0 * std::f64::consts::PI / config.initial_cell_count as f64;
                let radius = 1.0 + (i as f64 * 0.01);
                
                Cell {
                    cell_id: format!("cell_{}", i),
                    energy: 0.5,
                    age: 0,
                    generation: 0,
                    specialization: CellSpecialization::Stem,
                    // 四元系统初始状态
                    memory_density: 0.0,
                    signal_strength: 0.0,
                    activity_level: 0.5,
                    coordination: 0.5,
                    // 组织位置
                    position: [angle.cos() * radius, angle.sin() * radius, 0.0],
                    neighbors: Vec::new(),
                    layer: TissueLayer::Undetermined,
                }
            })
            .collect();
        
        Self {
            id: format!("stemcell_{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() % 1_000_000),
            config,
            cells,
            generation_count: 0,
            total_divisions: 0,
            total_deaths: 0,
            rng_state: 12345,
            blackhole,
            meshbrain,
            multinova,
            baseforce,
            tissue_stats: TissueStats::default(),
        }
    }
    
    /// Deterministic RNG
    fn next_random(&mut self) -> f64 {
        self.rng_state = self.rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.rng_state % 1000) as f64 / 1000.0
    }
    
    /// Evolve one generation (f64) - 利用四元系统
    pub fn evolve(&mut self, environment_signal: f64) -> EvolutionResult {
        self.generation_count += 1;
        
        let mut divisions = 0u64;
        let mut deaths = 0u64;
        
        let death_threshold = self.config.death_threshold;
        let division_threshold = self.config.division_threshold;
        let max_cell_count = self.config.max_cell_count;
        let mutation_rate = self.config.mutation_rate;
        let current_count = self.cells.len();
        let gen_count = self.generation_count;
        
        // 阶段0: 四元系统处理环境信号
        // BlackHole: 计算环境引力影响
        let gravity_effect = self.blackhole.calculate_gravity(environment_signal);
        
        // MeshBrain: 生成细胞间信号
        let mesh_signal = self.meshbrain.generate_signal(environment_signal, [0.0, 0.0, 0.0]);
        
        // MultiNova: 处理并行活动
        let nova_event = self.multinova.process_evolution(environment_signal);
        
        // BaseForce: 协调四元系统
        let coordination = self.baseforce.coordinate(gravity_effect, mesh_signal.intelligence_level, nova_event.radiation);
        
        // 阶段1: 收集需要分裂的细胞信息
        let mut division_info: Vec<(f64, u32, CellSpecialization, [f64; 3], TissueLayer)> = Vec::new();
        
        // 保存 rng_state 用于闭包
        let rng_state = self.rng_state;
        
        self.cells.retain_mut(|cell| {
            cell.age += 1;
            
            // 四元系统影响细胞状态
            cell.memory_density += gravity_effect * 0.01;
            cell.signal_strength = mesh_signal.intelligence_level * 0.5 + cell.signal_strength * 0.5;
            cell.activity_level = if nova_event.is_nova { 
                (cell.activity_level + nova_event.radiation * 0.1).min(1.0) 
            } else { 
                cell.activity_level * 0.99 
            };
            cell.coordination = coordination.total_activity;
            
            // 能量受四元系统影响
            let quad_boost = (cell.memory_density + cell.signal_strength + cell.activity_level + cell.coordination) / 4.0;
            cell.energy += environment_signal * 0.1 * (1.0 + quad_boost);
            cell.energy *= 0.99; // Natural decay
            
            // 组织层分化 (基于四元系统状态) - 使用静态函数避免借用冲突
            cell.layer = Self::determine_tissue_layer_static(cell);
            
            // Cell death
            if cell.energy < death_threshold {
                deaths += 1;
                return false;
            }
            
            // Cell division (收集信息，稍后处理)
            if cell.energy > division_threshold && current_count + division_info.len() < max_cell_count {
                cell.energy /= 2.0;
                
                // 新细胞位置偏移
                let offset_x = (rng_state as f64 % 100.0) / 100.0 * 0.5 - 0.25;
                let offset_y = ((rng_state / 100) as f64 % 100.0) / 100.0 * 0.5 - 0.25;
                let new_pos = [
                    cell.position[0] + offset_x,
                    cell.position[1] + offset_y,
                    cell.position[2],
                ];
                
                division_info.push((
                    cell.energy, 
                    cell.generation, 
                    cell.specialization.clone(),
                    new_pos,
                    cell.layer.clone(),
                ));
                divisions += 1;
            }
            
            true
        });
        
        // 阶段2: 创建新细胞 (带四元系统状态)
        for (i, (energy, generation, parent_spec, position, parent_layer)) in division_info.into_iter().enumerate() {
            let specialization = self.specialize_cell(&parent_spec, mutation_rate);
            let new_cell = Cell {
                cell_id: format!("cell_{}_{}", gen_count, i),
                energy,
                age: 0,
                generation: generation + 1,
                specialization,
                // 继承部分四元系统状态
                memory_density: gravity_effect * 0.5,
                signal_strength: mesh_signal.intelligence_level * 0.3,
                activity_level: 0.5,
                coordination: coordination.total_activity * 0.5,
                position,
                neighbors: Vec::new(),
                layer: parent_layer,
            };
            self.cells.push(new_cell);
        }
        
        self.total_divisions += divisions;
        self.total_deaths += deaths;
        
        // 阶段3: 更新细胞间连接 (MeshBrain 驱动)
        self.update_cell_connections();
        
        // 阶段4: 更新组织统计
        self.update_tissue_stats();
        
        // Calculate average energy
        let average_energy = if self.cells.is_empty() {
            0.0
        } else {
            self.cells.iter().map(|c| c.energy).sum::<f64>() / self.cells.len() as f64
        };
        
        EvolutionResult {
            generation: self.generation_count,
            cell_count: self.cells.len(),
            divisions,
            deaths,
            average_energy,
        }
    }
    
    /// 组织层 - 不预设阈值，由 Fate 代数自然演化决定
    /// 群聚结构和组织行为应该由代数触发，而非固定阈值
    fn determine_tissue_layer_static(_cell: &Cell) -> TissueLayer {
        // 移除硬编码阈值，保持未确定状态
        // 真正的组织分化应由 Fate 代数边际效应触发
        TissueLayer::Undetermined
    }
    
    #[allow(dead_code)]
    fn determine_tissue_layer(&self, cell: &Cell) -> TissueLayer {
        Self::determine_tissue_layer_static(cell)
    }
    
    /// 更新细胞间连接 (基于 MeshBrain)
    fn update_cell_connections(&mut self) {
        let cell_positions: Vec<(String, [f64; 3])> = self.cells.iter()
            .map(|c| (c.cell_id.clone(), c.position))
            .collect();
        
        for cell in &mut self.cells {
            cell.neighbors.clear();
            let threshold = 1.5;  // 连接距离阈值
            
            for (other_id, other_pos) in &cell_positions {
                if &cell.cell_id != other_id {
                    let dist = ((cell.position[0] - other_pos[0]).powi(2) 
                              + (cell.position[1] - other_pos[1]).powi(2)
                              + (cell.position[2] - other_pos[2]).powi(2)).sqrt();
                    
                    if dist < threshold {
                        cell.neighbors.push(other_id.clone());
                    }
                }
            }
        }
    }
    
    /// 更新组织统计
    fn update_tissue_stats(&mut self) {
        let cell_count = self.cells.len();
        if cell_count == 0 {
            self.tissue_stats = TissueStats::default();
            return;
        }
        
        let mut total_connections = 0usize;
        let mut total_signal = 0.0f64;
        let mut total_memory = 0.0f64;
        let mut total_activity = 0.0f64;
        let mut total_coord = 0.0f64;
        
        for cell in &self.cells {
            total_connections += cell.neighbors.len();
            total_signal += cell.signal_strength;
            total_memory += cell.memory_density;
            total_activity += cell.activity_level;
            total_coord += cell.coordination;
        }
        
        let n = cell_count as f64;
        self.tissue_stats = TissueStats {
            cell_count,
            total_connections,
            avg_signal_strength: total_signal / n,
            avg_memory_density: total_memory / n,
            avg_activity_level: total_activity / n,
            avg_coordination: total_coord / n,
        };
    }
    
    /// Specialize cell based on mutation rate
    fn specialize_cell(&mut self, parent_spec: &CellSpecialization, mutation_rate: f64) -> CellSpecialization {
        let rand_val = self.next_random();
        
        if rand_val < mutation_rate {
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
    
    /// Get network status
    pub fn get_status(&self) -> StemCellStatus {
        StemCellStatus {
            network_id: self.id.clone(),
            cell_count: self.cells.len(),
            generation: self.generation_count,
            total_divisions: self.total_divisions,
            total_deaths: self.total_deaths,
            tissue_stats: self.tissue_stats.clone(),
            quad_system_active: true,
        }
    }
    
    /// 获取四元系统状态
    pub fn get_quad_status(&self) -> QuadSystemStatus {
        QuadSystemStatus {
            blackhole_density: self.cells.iter().map(|c| c.memory_density).sum::<f64>() / self.cells.len().max(1) as f64,
            meshbrain_connectivity: self.tissue_stats.total_connections as f64 / self.cells.len().max(1) as f64,
            multinova_activity: self.cells.iter().map(|c| c.activity_level).sum::<f64>() / self.cells.len().max(1) as f64,
            baseforce_coordination: self.cells.iter().map(|c| c.coordination).sum::<f64>() / self.cells.len().max(1) as f64,
        }
    }
}

impl Default for StemCell {
    fn default() -> Self {
        Self::new()
    }
}

/// Evolution result (f64)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionResult {
    pub generation: u32,
    pub cell_count: usize,
    pub divisions: u64,
    pub deaths: u64,
    pub average_energy: f64,
}

/// Network status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StemCellStatus {
    pub network_id: String,
    pub cell_count: usize,
    pub generation: u32,
    pub total_divisions: u64,
    pub total_deaths: u64,
    /// 组织统计
    pub tissue_stats: TissueStats,
    /// 四元系统是否激活
    pub quad_system_active: bool,
}

/// 四元系统状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuadSystemStatus {
    /// BlackHole 平均密度
    pub blackhole_density: f64,
    /// MeshBrain 平均连接度
    pub meshbrain_connectivity: f64,
    /// MultiNova 平均活动级别
    pub multinova_activity: f64,
    /// BaseForce 平均协调值
    pub baseforce_coordination: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_stem_cell_evolution() {
        let mut network = StemCell::new();
        
        for _ in 0..10 {
            let result = network.evolve(0.5);
            assert!(result.cell_count > 0);
        }
        
        let status = network.get_status();
        assert_eq!(status.generation, 10);
    }
}
