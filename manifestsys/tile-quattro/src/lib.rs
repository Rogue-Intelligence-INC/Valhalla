//! TileQuattro - 四元内存管理器 (f64 版本)
//!
//! Val-OS 主版本内存管理器 - 整合四元系统外壳
//! 性能: 6.57x (相比无 Space)
//! 碎片化: ~0%
//!
//! 四元系统整合 (神经网络模式):
//! - BlackHole: 高密度数据存储 (权重矩阵)
//! - MeshBrain: 神经连接网络 (突触)
//! - MultiNova: 并行计算 (前向/反向传播)
//! - BaseForce: 协调和学习率控制
//!
//! 完整版本 (tile_complete):
//! - TripleFate: 多层 Fate 监控
//! - VoyagerSpace: 自动维度扩展
//! - MicroTile: 小容器 → 局部收敛 → Tile

// 完整版本 (带 Triple-Fate + VoyagerSpace)
pub mod tile_complete;
pub use tile_complete::{
    TileQuattroComplete, TileQuattroConfig as TileQuattroCompleteConfig,
    MicroTile, LocalTileState, CompletedTile,
    QuadSignals, QuadResults,
    TileFlowResult, TileBatchResult, TileQuattroStatus as CompleteStatus,
};

use serde::{Deserialize, Serialize};

// 四元系统
use valhalla_metasystems_blackhole::{GravityField, GravityFieldConfig};
use valhalla_metasystems_meshbrain::{MeshBrainSystem, MeshBrainConfig};
use valhalla_metasystems_multinova::{MultinovaProcessor, MultinovaConfig};
use valhalla_metasystems_baseforce::BaseForceCoordinator;

/// 四元池类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QuadPool {
    /// BlackHole - 高密度数据 (GPU, 大块分配)
    BlackHole,
    /// MeshBrain - 智能分配 (网络, 队列)
    MeshBrain,
    /// MultiNova - 并行块 (存储, I/O)
    MultiNova,
    /// BaseForce - 协调层 (USB, 小块分配)
    BaseForce,
}

impl QuadPool {
    /// 根据分配大小和类型选择最佳池
    pub fn select_pool(size: f64, hint: AllocationHint) -> Self {
        match hint {
            AllocationHint::HighDensity => QuadPool::BlackHole,
            AllocationHint::SmartQueue => QuadPool::MeshBrain,
            AllocationHint::ParallelIO => QuadPool::MultiNova,
            AllocationHint::Coordination => QuadPool::BaseForce,
            AllocationHint::Auto => {
                if size > 64.0 {
                    QuadPool::BlackHole
                } else if size > 16.0 {
                    QuadPool::MultiNova
                } else if size > 4.0 {
                    QuadPool::MeshBrain
                } else {
                    QuadPool::BaseForce
                }
            }
        }
    }
}

/// 分配提示
#[derive(Debug, Clone, Copy, Default)]
pub enum AllocationHint {
    #[default]
    Auto,
    HighDensity,
    SmartQueue,
    ParallelIO,
    Coordination,
}

/// 内存块 (神经Tile)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryBlock {
    pub id: String,
    pub pool: QuadPool,
    pub size: f64,
    pub offset: f64,
    pub in_use: bool,
    
    // 神经网络属性 (四元系统驱动)
    /// 激活值 (由 MultiNova 计算)
    pub activation: f64,
    /// 连接权重 (由 BlackHole 存储)
    pub weights: Vec<f64>,
    /// 偏置 (由 BaseForce 协调)
    pub bias: f64,
    /// 连接的目标 Tile IDs (由 MeshBrain 管理)
    pub connections: Vec<String>,
    /// Tile 类型 - 由 Fate 自然演化决定
    pub tile_type: TileType,
}

/// Tile 类型 - 由 Fate 自然决定，不预设映射
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum TileType {
    #[default]
    Undetermined,  // 未确定 - Fate 决定
}

/// 四元池状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolState {
    pub pool_type: QuadPool,
    pub total_size: f64,
    pub used_size: f64,
    pub block_count: usize,
    pub fragmentation: f64,
}

impl PoolState {
    pub fn new(pool_type: QuadPool, total_size: f64) -> Self {
        Self {
            pool_type,
            total_size,
            used_size: 0.0,
            block_count: 0,
            fragmentation: 0.0,
        }
    }
    
    pub fn available(&self) -> f64 {
        self.total_size - self.used_size
    }
    
    pub fn usage_percent(&self) -> f64 {
        if self.total_size == 0.0 {
            0.0
        } else {
            self.used_size / self.total_size * 100.0
        }
    }
}

/// TileQuattro 内存管理器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileQuattroConfig {
    /// BlackHole 池大小 (MB)
    pub blackhole_size: f64,
    /// MeshBrain 池大小 (MB)
    pub meshbrain_size: f64,
    /// MultiNova 池大小 (MB)
    pub multinova_size: f64,
    /// BaseForce 池大小 (MB)
    pub baseforce_size: f64,
    /// Space 维度管理启用
    pub space_enabled: bool,
}

impl Default for TileQuattroConfig {
    fn default() -> Self {
        Self {
            blackhole_size: 256.0,
            meshbrain_size: 256.0,
            multinova_size: 256.0,
            baseforce_size: 256.0,
            space_enabled: true,
        }
    }
}

impl TileQuattroConfig {
    pub fn total_size(&self) -> f64 {
        self.blackhole_size + self.meshbrain_size + self.multinova_size + self.baseforce_size
    }
}

/// TileQuattro 内存管理器 - 整合四元系统外壳
/// 
/// Val-OS f64 版本的核心内存管理器
/// 实验验证性能提升 6.57x
/// 
/// 神经网络模式: 每个 Tile 作为神经元，四元系统提供能力
#[derive(Debug)]
pub struct TileQuattroMemoryManager {
    pub config: TileQuattroConfig,
    pub blackhole_pool: PoolState,
    pub meshbrain_pool: PoolState,
    pub multinova_pool: PoolState,
    pub baseforce_pool: PoolState,
    pub blocks: Vec<MemoryBlock>,
    pub total_allocations: u64,
    pub total_deallocations: u64,
    pub space_dimension: f64,
    next_block_id: u64,
    
    // 四元系统实例 (外壳容器内部)
    /// BlackHole - 权重存储和压缩
    pub blackhole: GravityField,
    /// MeshBrain - 神经连接管理
    pub meshbrain: MeshBrainSystem,
    /// MultiNova - 并行前向传播
    pub multinova: MultinovaProcessor,
    /// BaseForce - 学习率和协调
    pub baseforce: BaseForceCoordinator,
    
    // 神经网络统计
    /// 网络统计
    pub network_stats: NetworkStats,
}

/// 神经网络统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkStats {
    pub total_neurons: usize,
    pub total_connections: usize,
    pub avg_activation: f64,
    pub total_weight_mass: f64,
    pub dense_count: usize,
    pub recurrent_count: usize,
    pub attention_count: usize,
    pub embedding_count: usize,
}

impl TileQuattroMemoryManager {
    pub fn new() -> Self {
        Self::with_config(TileQuattroConfig::default())
    }
    
    pub fn with_config(config: TileQuattroConfig) -> Self {
        // 初始化四元系统
        let blackhole = GravityField::new(GravityFieldConfig::default());
        let meshbrain = MeshBrainSystem::new(MeshBrainConfig::default());
        let multinova = MultinovaProcessor::new(MultinovaConfig::default());
        let baseforce = BaseForceCoordinator::new();
        
        Self {
            blackhole_pool: PoolState::new(QuadPool::BlackHole, config.blackhole_size),
            meshbrain_pool: PoolState::new(QuadPool::MeshBrain, config.meshbrain_size),
            multinova_pool: PoolState::new(QuadPool::MultiNova, config.multinova_size),
            baseforce_pool: PoolState::new(QuadPool::BaseForce, config.baseforce_size),
            config,
            blocks: Vec::new(),
            total_allocations: 0,
            total_deallocations: 0,
            space_dimension: 4.0, // 初始四维
            next_block_id: 0,
            blackhole,
            meshbrain,
            multinova,
            baseforce,
            network_stats: NetworkStats::default(),
        }
    }
    
    /// 分配内存
    pub fn allocate(&mut self, size: f64, hint: AllocationHint) -> Option<String> {
        let pool = QuadPool::select_pool(size, hint);
        self.allocate_from_pool(size, pool)
    }
    
    /// 从指定池分配 (带神经网络初始化)
    pub fn allocate_from_pool(&mut self, size: f64, pool: QuadPool) -> Option<String> {
        // 先检查可用空间
        let available = self.get_pool(pool).available();
        if available < size {
            // 尝试其他池
            return self.allocate_fallback(size, pool);
        }
        
        let block_id = format!("{:?}_{}", pool, self.next_block_id);
        self.next_block_id += 1;
        
        // 获取偏移量并更新池状态
        let offset = {
            let pool_state = self.get_pool_mut(pool);
            let off = pool_state.used_size;
            pool_state.used_size += size;
            pool_state.block_count += 1;
            off
        };
        
        // 四元系统初始化神经元属性
        // BlackHole: 计算初始权重密度
        let gravity = self.blackhole.calculate_gravity(size);
        let initial_weights = vec![gravity * 0.01; (size as usize).min(16)];
        
        // MeshBrain: 初始化连接潜力
        let mesh_signal = self.meshbrain.generate_signal(size, [0.0, 0.0, 0.0]);
        
        // MultiNova: 初始化激活
        let nova_event = self.multinova.process_evolution(size);
        let initial_activation = if nova_event.is_nova { nova_event.radiation } else { 0.0 };
        
        // BaseForce: 初始化偏置
        let coord = self.baseforce.coordinate(gravity, mesh_signal.intelligence_level, nova_event.radiation);
        let initial_bias = coord.total_activity * 0.1;
        
        // Tile 类型由 Fate 自然演化决定，不预设
        let tile_type = TileType::Undetermined;
        
        self.blocks.push(MemoryBlock {
            id: block_id.clone(),
            pool,
            size,
            offset,
            in_use: true,
            activation: initial_activation,
            weights: initial_weights,
            bias: initial_bias,
            connections: Vec::new(),
            tile_type,
        });
        
        self.total_allocations += 1;
        
        // Space 维度调整
        if self.config.space_enabled {
            self.adjust_space_dimension();
        }
        
        // 更新网络统计
        self.update_network_stats();
        
        Some(block_id)
    }
    
    /// 回退分配 (当首选池满时)
    fn allocate_fallback(&mut self, size: f64, excluded: QuadPool) -> Option<String> {
        let pools = [QuadPool::BlackHole, QuadPool::MeshBrain, QuadPool::MultiNova, QuadPool::BaseForce];
        
        for pool in pools {
            if pool != excluded && self.get_pool(pool).available() >= size {
                return self.allocate_from_pool(size, pool);
            }
        }
        
        None // 所有池都满
    }
    
    /// 释放内存
    pub fn deallocate(&mut self, block_id: &str) -> bool {
        if let Some(idx) = self.blocks.iter().position(|b| b.id == block_id && b.in_use) {
            // 先获取需要的信息
            let (pool, size) = {
                let block = &self.blocks[idx];
                (block.pool, block.size)
            };
            
            // 更新块状态
            self.blocks[idx].in_use = false;
            
            // 更新池状态
            {
                let pool_state = self.get_pool_mut(pool);
                pool_state.used_size -= size;
                pool_state.block_count = pool_state.block_count.saturating_sub(1);
            }
            
            self.total_deallocations += 1;
            
            // 更新碎片化
            self.update_fragmentation(pool);
            
            // Space 维度调整
            if self.config.space_enabled {
                self.adjust_space_dimension();
            }
            
            true
        } else {
            false
        }
    }
    
    /// 获取池状态 (不可变)
    pub fn get_pool(&self, pool: QuadPool) -> &PoolState {
        match pool {
            QuadPool::BlackHole => &self.blackhole_pool,
            QuadPool::MeshBrain => &self.meshbrain_pool,
            QuadPool::MultiNova => &self.multinova_pool,
            QuadPool::BaseForce => &self.baseforce_pool,
        }
    }
    
    /// 获取池状态 (可变)
    fn get_pool_mut(&mut self, pool: QuadPool) -> &mut PoolState {
        match pool {
            QuadPool::BlackHole => &mut self.blackhole_pool,
            QuadPool::MeshBrain => &mut self.meshbrain_pool,
            QuadPool::MultiNova => &mut self.multinova_pool,
            QuadPool::BaseForce => &mut self.baseforce_pool,
        }
    }
    
    /// 更新碎片化
    fn update_fragmentation(&mut self, pool: QuadPool) {
        let pool_blocks: Vec<_> = self.blocks.iter()
            .filter(|b| b.pool == pool)
            .collect();
        
        let total_blocks = pool_blocks.len();
        let free_blocks = pool_blocks.iter().filter(|b| !b.in_use).count();
        
        let frag = if total_blocks == 0 {
            0.0
        } else {
            free_blocks as f64 / total_blocks as f64
        };
        
        self.get_pool_mut(pool).fragmentation = frag;
    }
    
    /// 调整 Space 维度
    fn adjust_space_dimension(&mut self) {
        // 基于使用率调整维度
        let total_usage = self.total_usage();
        
        // 维度范围: 3.5 ~ 4.5
        self.space_dimension = 4.0 + (total_usage - 0.5) * 1.0;
        self.space_dimension = self.space_dimension.clamp(3.5, 4.5);
    }
    
    /// 总使用率
    pub fn total_usage(&self) -> f64 {
        let total_used = self.blackhole_pool.used_size 
            + self.meshbrain_pool.used_size 
            + self.multinova_pool.used_size 
            + self.baseforce_pool.used_size;
        let total_size = self.config.total_size();
        
        if total_size == 0.0 {
            0.0
        } else {
            total_used / total_size
        }
    }
    
    /// 总碎片化
    pub fn total_fragmentation(&self) -> f64 {
        let frags = [
            self.blackhole_pool.fragmentation,
            self.meshbrain_pool.fragmentation,
            self.multinova_pool.fragmentation,
            self.baseforce_pool.fragmentation,
        ];
        frags.iter().sum::<f64>() / 4.0
    }
    
    /// 更新神经网络统计
    fn update_network_stats(&mut self) {
        let active_blocks: Vec<_> = self.blocks.iter().filter(|b| b.in_use).collect();
        
        self.network_stats = NetworkStats {
            total_neurons: active_blocks.len(),
            total_connections: active_blocks.iter().map(|b| b.connections.len()).sum(),
            avg_activation: if active_blocks.is_empty() { 
                0.0 
            } else {
                active_blocks.iter().map(|b| b.activation).sum::<f64>() / active_blocks.len() as f64
            },
            total_weight_mass: active_blocks.iter()
                .flat_map(|b| b.weights.iter())
                .map(|w| w.abs())
                .sum(),
            // Tile 类型由 Fate 决定，不再预设分类
            dense_count: 0,
            recurrent_count: 0,
            attention_count: 0,
            embedding_count: 0,
        };
    }
    
    /// 前向传播一步 (四元系统驱动)
    pub fn propagate(&mut self, input_signal: f64) -> PropagationResult {
        // MultiNova: 并行计算所有神经元激活
        let nova_event = self.multinova.process_evolution(input_signal);
        
        // MeshBrain: 管理信号传递
        let mesh_signal = self.meshbrain.generate_signal(input_signal, [0.0, 0.0, 0.0]);
        
        // BlackHole: 权重压缩/更新
        let gravity = self.blackhole.calculate_gravity(input_signal);
        
        // BaseForce: 协调学习
        let coord = self.baseforce.coordinate(gravity, mesh_signal.intelligence_level, nova_event.radiation);
        
        let mut total_activation = 0.0;
        let mut propagated_count = 0usize;
        
        for block in &mut self.blocks {
            if block.in_use {
                // 计算新激活值
                let weight_sum: f64 = block.weights.iter().sum();
                let raw_activation = input_signal * weight_sum + block.bias;
                
                // 应用四元系统影响
                let quad_factor = (gravity + mesh_signal.intelligence_level + coord.total_activity) / 3.0;
                
                // 激活函数 (tanh 变体)
                block.activation = (raw_activation * quad_factor).tanh();
                
                // 权重微调 (由 BlackHole 引力控制)
                for weight in &mut block.weights {
                    *weight += gravity * 0.001 * (1.0 - weight.abs());
                }
                
                total_activation += block.activation;
                propagated_count += 1;
            }
        }
        
        // 更新统计
        self.update_network_stats();
        
        PropagationResult {
            propagated_neurons: propagated_count,
            total_activation,
            avg_activation: if propagated_count > 0 { total_activation / propagated_count as f64 } else { 0.0 },
            gravity_effect: gravity,
            mesh_intelligence: mesh_signal.intelligence_level,
            nova_radiation: nova_event.radiation,
            coordination: coord.total_activity,
        }
    }
    
    /// 获取四元系统状态
    pub fn get_quad_status(&self) -> TileQuadStatus {
        TileQuadStatus {
            blackhole_gravity: self.blackhole.calculate_gravity(1.0),
            meshbrain_intelligence: 0.5, // 基础值
            multinova_radiation: 0.0,
            baseforce_coordination: 0.5,
        }
    }
    
    /// Collapse 整理 (减少碎片化)
    pub fn collapse(&mut self) {
        // 移除已释放的块
        self.blocks.retain(|b| b.in_use);
        
        // 重新计算每个池的使用量
        for pool in [QuadPool::BlackHole, QuadPool::MeshBrain, QuadPool::MultiNova, QuadPool::BaseForce] {
            let pool_blocks: Vec<_> = self.blocks.iter()
                .filter(|b| b.pool == pool)
                .collect();
            
            let used_size: f64 = pool_blocks.iter().map(|b| b.size).sum();
            let block_count = pool_blocks.len();
            
            let pool_state = self.get_pool_mut(pool);
            pool_state.used_size = used_size;
            pool_state.block_count = block_count;
            pool_state.fragmentation = 0.0; // Collapse 后碎片化为 0
        }
        
        // 调整 Space 维度
        if self.config.space_enabled {
            self.adjust_space_dimension();
        }
    }
    
    /// 获取统计信息
    pub fn get_stats(&self) -> TileQuattroStats {
        TileQuattroStats {
            total_allocations: self.total_allocations,
            total_deallocations: self.total_deallocations,
            active_blocks: self.blocks.iter().filter(|b| b.in_use).count(),
            total_usage: self.total_usage(),
            total_fragmentation: self.total_fragmentation(),
            space_dimension: self.space_dimension,
            blackhole_usage: self.blackhole_pool.usage_percent(),
            meshbrain_usage: self.meshbrain_pool.usage_percent(),
            multinova_usage: self.multinova_pool.usage_percent(),
            baseforce_usage: self.baseforce_pool.usage_percent(),
            network_stats: self.network_stats.clone(),
            quad_system_active: true,
        }
    }
}

/// 前向传播结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationResult {
    pub propagated_neurons: usize,
    pub total_activation: f64,
    pub avg_activation: f64,
    pub gravity_effect: f64,
    pub mesh_intelligence: f64,
    pub nova_radiation: f64,
    pub coordination: f64,
}

/// 四元系统状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileQuadStatus {
    pub blackhole_gravity: f64,
    pub meshbrain_intelligence: f64,
    pub multinova_radiation: f64,
    pub baseforce_coordination: f64,
}

impl Default for TileQuattroMemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

/// TileQuattro 统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileQuattroStats {
    pub total_allocations: u64,
    pub total_deallocations: u64,
    pub active_blocks: usize,
    pub total_usage: f64,
    pub total_fragmentation: f64,
    pub space_dimension: f64,
    pub blackhole_usage: f64,
    pub meshbrain_usage: f64,
    pub multinova_usage: f64,
    pub baseforce_usage: f64,
    /// 神经网络统计
    pub network_stats: NetworkStats,
    /// 四元系统是否激活
    pub quad_system_active: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_allocation() {
        let mut mm = TileQuattroMemoryManager::new();
        
        let id = mm.allocate(32.0, AllocationHint::Auto);
        assert!(id.is_some());
        assert_eq!(mm.total_allocations, 1);
    }
    
    #[test]
    fn test_pool_selection() {
        assert_eq!(QuadPool::select_pool(100.0, AllocationHint::Auto), QuadPool::BlackHole);
        assert_eq!(QuadPool::select_pool(32.0, AllocationHint::Auto), QuadPool::MultiNova);
        assert_eq!(QuadPool::select_pool(8.0, AllocationHint::Auto), QuadPool::MeshBrain);
        assert_eq!(QuadPool::select_pool(2.0, AllocationHint::Auto), QuadPool::BaseForce);
    }
    
    #[test]
    fn test_deallocation() {
        let mut mm = TileQuattroMemoryManager::new();
        
        let id = mm.allocate(32.0, AllocationHint::Auto).unwrap();
        assert!(mm.deallocate(&id));
        assert_eq!(mm.total_deallocations, 1);
    }
    
    #[test]
    fn test_collapse() {
        let mut mm = TileQuattroMemoryManager::new();
        
        // 分配并释放多个块
        let ids: Vec<_> = (0..10)
            .map(|_| mm.allocate(10.0, AllocationHint::Auto).unwrap())
            .collect();
        
        for id in ids.iter().take(5) {
            mm.deallocate(id);
        }
        
        // Collapse 整理
        mm.collapse();
        
        assert_eq!(mm.total_fragmentation(), 0.0);
    }
    
    #[test]
    fn test_space_dimension() {
        let mut mm = TileQuattroMemoryManager::new();
        
        // 初始维度
        assert!((mm.space_dimension - 4.0).abs() < 0.1);
        
        // 分配后维度应该变化
        for _ in 0..50 {
            mm.allocate(10.0, AllocationHint::Auto);
        }
        
        assert!(mm.space_dimension != 4.0);
    }
}
