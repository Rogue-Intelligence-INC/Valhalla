//! Tile-Quattro Complete (f64 Version) - 四元系统并行架构
//!
//! 完整版本，从 valhalla-rational 移植
//! 自带: TripleFate (多层监控) + VoyagerSpace (自动维度)
//!
//! 架构:
//! - 四元系统并行处理
//! - 数据流 → Tile → Fate 监控
//! - VoyagerSpace 自动维度扩展
//! - **Fate 存储**: 数据由 Fate 处理并存储

use serde::{Deserialize, Serialize};
use std::path::Path;

// 系统存储 (旧版兼容)
pub use valhalla_storage::SystemStorage;

// 使用 fate-f64 的完整 Triple-Fate 架构
use fate_f64::{
    TripleFateCombined, TripleDiscovery, VoyagerSpaceF64, FateSlice,
};

// Space GPU 高维计算
use valhalla_space_f64_gpu::{SpaceGPU, FractalDimensionCalculatorGPU, HyperVector};

// ============================================================================
// 系统切片 - 用于冻结/解冻
// ============================================================================

/// Tile 系统切片
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileSlice {
    pub id: String,
    pub global_tick: u64,
    pub total_operations: u64,
    pub completed_tile_count: usize,
    /// 内嵌的 Fate 切片
    pub fate_slice: FateSlice,
}

// ============================================================================
// 配置
// ============================================================================

/// 四元瓦片配置 (f64)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileQuattroConfig {
    pub tile_count: usize,
    pub parallel_threshold: f64,
    pub load_balance_enabled: bool,
    /// 小容器容量
    pub container_capacity: usize,
}

impl Default for TileQuattroConfig {
    fn default() -> Self {
        Self {
            tile_count: 4,
            parallel_threshold: 0.5,
            load_balance_enabled: true,
            container_capacity: 100,
        }
    }
}

// ============================================================================
// 小容器 (MicroTile) - 数据流入单元
// ============================================================================

/// 小容器 - 数据流入并局部收敛后成为有效 Tile（无冗余缓冲）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroTile {
    pub id: String,
    pub local_state: LocalTileState,
    pub created_tick: u64,
}

/// 局部 Tile 状态（固定数组，O(1)操作）
/// 收敛由上层 Fate 的 ClusterEquation 控制
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalTileState {
    pub signal_count: u64,
    pub sum: f64,
    pub sum_sq: f64,
    /// 环形缓冲（供 Fate 分析）
    pub ring: [f64; 10],
    pub ring_idx: usize,
    pub ring_len: usize,
    pub converged: bool,
}

impl LocalTileState {
    pub fn new() -> Self {
        Self {
            signal_count: 0,
            sum: 0.0,
            sum_sq: 0.0,
            ring: [0.0; 10],
            ring_idx: 0,
            ring_len: 0,
            converged: false,
        }
    }

    /// 接收数据（只收集，不判断收敛）
    /// 收敛由上层 Fate 控制
    #[inline]
    pub fn receive(&mut self, value: f64) {
        self.signal_count += 1;
        self.sum += value;
        self.sum_sq += value * value;
        
        // 环形写入（供 Fate 分析）
        self.ring[self.ring_idx] = value;
        self.ring_idx = (self.ring_idx + 1) % 10;
        if self.ring_len < 10 { self.ring_len += 1; }
    }
    
    /// 获取最近的差异序列（供 Fate 分析）
    pub fn get_diffs(&self) -> [f64; 3] {
        if self.ring_len < 4 {
            return [0.0, 0.0, 0.0];
        }
        let get = |off: usize| self.ring[(self.ring_idx + 10 - 1 - off) % 10];
        let d1 = (get(0) - get(1)).abs();
        let d2 = (get(1) - get(2)).abs();
        let d3 = (get(2) - get(3)).abs();
        [d1, d2, d3]
    }
    
    /// 由 Fate 设置收敛状态
    pub fn set_converged(&mut self, converged: bool) {
        self.converged = converged;
    }
    
    pub fn last_values(&self) -> Vec<f64> {
        (0..self.ring_len).map(|i| self.ring[(self.ring_idx + 10 - self.ring_len + i) % 10]).collect()
    }
    
    pub fn mean(&self) -> f64 {
        if self.signal_count == 0 { 0.0 } else { self.sum / self.signal_count as f64 }
    }
    
    pub fn variance(&self) -> f64 {
        if self.signal_count == 0 { 0.0 }
        else {
            let mean = self.mean();
            self.sum_sq / self.signal_count as f64 - mean * mean
        }
    }
}

impl Default for LocalTileState {
    fn default() -> Self {
        Self::new()
    }
}

impl MicroTile {
    pub fn new(id: String, _capacity: usize, tick: u64) -> Self {
        Self {
            id,
            local_state: LocalTileState::new(),
            created_tick: tick,
        }
    }
    
    /// 接收数据（只收集，不判断收敛）
    #[inline]
    pub fn receive_data(&mut self, data: f64) {
        self.local_state.receive(data);
    }
    
    /// 获取差异序列（供 Fate 分析）
    pub fn get_diffs(&self) -> [f64; 3] {
        self.local_state.get_diffs()
    }
    
    pub fn is_converged(&self) -> bool {
        self.local_state.converged
    }
    
    /// 由 Fate 设置收敛
    pub fn set_converged(&mut self, converged: bool) {
        self.local_state.set_converged(converged);
    }
}

// ============================================================================
// 四元信号和结果
// ============================================================================

/// 四元信号输入 (f64)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuadSignals {
    pub blackhole_distance: f64,
    pub meshbrain_intensity: f64,
    pub multinova_energy: f64,
    pub baseforce_coordination: f64,
}

/// 四元处理结果 (f64)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuadResults {
    pub blackhole_gravity: f64,
    pub meshbrain_intelligence: f64,
    pub multinova_radiation: f64,
    pub baseforce_activity: f64,
    pub combined_output: f64,
}

// ============================================================================
// TileQuattro Complete - 完整的四元瓦片系统
// ============================================================================

/// 完整的四元瓦片系统 (f64)
/// 自带: TripleFate (多层监控) + VoyagerSpace (自动维度) + 系统存储
#[derive(Debug)]
pub struct TileQuattroComplete {
    pub id: String,
    pub config: TileQuattroConfig,
    /// 多层 Fate 系统
    pub triple_fate: TripleFateCombined,
    /// 自动维度空间
    pub space: VoyagerSpaceF64,
    /// 活跃的小容器
    pub active_tiles: Vec<MicroTile>,
    /// 已完成的 Tile 数据
    pub completed_tiles: Vec<CompletedTile>,
    pub total_operations: u64,
    pub global_tick: u64,
    /// 系统存储 - 完整的文件操作权限
    pub storage: Option<SystemStorage>,
    /// Space GPU 高维计算
    pub space_gpu: SpaceGPU,
}

/// 已完成的 Tile（神经元节点）
/// 核心宗旨：自发形成类似神经网络的连接
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedTile {
    pub id: String,
    pub mean: f64,
    pub variance: f64,
    pub signal_count: u64,
    pub created_tick: u64,
    pub completed_tick: u64,
    /// 数据特征（Tile 的"身份"）- 继承自 MicroTile
    pub data_signature: Vec<f64>,
    /// 能量（用于合并/分裂判断）
    pub energy: f64,
    
    // ═══ 神经网络属性 ═══
    /// 激活值
    pub activation: f64,
    /// 连接权重（自发形成）
    pub weights: Vec<f64>,
    /// 偏置
    pub bias: f64,
    /// 连接到其他 Tile 的 ID（自发形成的连接）
    pub connections: Vec<String>,
}

/// Tile 周期处理结果
#[derive(Debug, Clone)]
pub struct TileCycleResult {
    pub generation: u64,
    pub tile_count: usize,
    pub merges: usize,
    pub splits: usize,
    pub avg_activation: f64,
    pub total_connections: usize,
    pub new_connections: usize,
}

/// 前向传播结果
#[derive(Debug, Clone)]
pub struct PropagationResult {
    pub propagated_tiles: usize,
    pub total_activation: f64,
    pub avg_activation: f64,
    pub new_connections: usize,
    pub total_connections: usize,
}

impl TileQuattroComplete {
    pub fn new() -> Self {
        Self::with_config(TileQuattroConfig::default())
    }
    
    pub fn with_config(config: TileQuattroConfig) -> Self {
        Self {
            id: format!("tile_complete_{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() % 1_000_000),
            config,
            triple_fate: TripleFateCombined::new(),
            space: VoyagerSpaceF64::new(),
            active_tiles: Vec::new(),
            completed_tiles: Vec::new(),
            total_operations: 0,
            global_tick: 0,
            storage: None,
            space_gpu: SpaceGPU::new(),
        }
    }
    
    /// 带存储路径初始化
    pub fn with_storage(base_path: &Path) -> std::io::Result<Self> {
        let mut instance = Self::new();
        instance.init_storage(base_path)?;
        Ok(instance)
    }
    
    /// 初始化系统存储
    pub fn init_storage(&mut self, base_path: &Path) -> std::io::Result<()> {
        self.storage = Some(SystemStorage::new(base_path, "tile")?);
        Ok(())
    }
    
    /// 获取存储
    pub fn storage(&self) -> Option<&SystemStorage> {
        self.storage.as_ref()
    }
    
    /// 存储内容
    pub fn store(&self, name: &str, content: &str) -> std::io::Result<()> {
        self.storage.as_ref()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Storage not initialized"))?
            .write(name, content)
    }
    
    /// 读取内容
    pub fn load(&self, name: &str) -> std::io::Result<String> {
        self.storage.as_ref()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Storage not initialized"))?
            .read(name)
    }
    
    /// 删除内容
    pub fn remove(&self, name: &str) -> std::io::Result<()> {
        self.storage.as_ref()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Storage not initialized"))?
            .delete(name)
    }
    
    /// 列出所有
    pub fn list_stored(&self) -> std::io::Result<Vec<String>> {
        self.storage.as_ref()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Storage not initialized"))?
            .list_all()
    }
    
    /// 接收并存储内容
    pub fn receive_content(&mut self, content: &str) -> std::io::Result<String> {
        let id = format!("data_{}", self.global_tick);
        self.store(&id, content)?;
        let signal = content.len() as f64;
        self.receive_data(signal);
        Ok(id)
    }
    
    // ═══════════════════════════════════════════════════════════════════════════
    // Fate 存储层 - 委托给内部 triple_fate
    // ═══════════════════════════════════════════════════════════════════════════
    
    /// 通过 Fate 存储数据
    pub fn fate_store(&mut self, data: &[u8]) -> String {
        self.triple_fate.store_data(data)
    }
    
    /// 通过 Fate 存储字符串
    pub fn fate_store_string(&mut self, content: &str) -> String {
        self.triple_fate.store_string(content)
    }
    
    /// 系统自主创建数据
    pub fn fate_create(&mut self, data: Vec<u8>) -> String {
        self.triple_fate.create_data(data)
    }
    
    /// 修改 Fate 存储的数据
    pub fn fate_modify(&mut self, id: &str, new_data: Vec<u8>) -> bool {
        self.triple_fate.modify_data(id, new_data)
    }
    
    /// 读取 Fate 存储的数据
    pub fn fate_load(&self, id: &str) -> Option<&fate_f64::FateDataEntry> {
        self.triple_fate.load_data(id)
    }
    
    /// 删除 Fate 存储的数据
    pub fn fate_remove(&mut self, id: &str) -> Option<fate_f64::FateDataEntry> {
        self.triple_fate.remove_data(id)
    }
    
    /// 列出 Fate 存储的所有 ID
    pub fn fate_list(&self) -> Vec<&String> {
        self.triple_fate.list_stored()
    }
    
    /// Fate 存储数量
    pub fn fate_count(&self) -> usize {
        self.triple_fate.stored_count()
    }
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 系统切片 - 冻结/解冻 (bincode 二进制)
    // ═══════════════════════════════════════════════════════════════════════════
    
    /// 保存系统切片 (.tile 二进制)
    pub fn save_slice(&self, path: &Path) -> std::io::Result<()> {
        let slice = TileSlice {
            id: self.id.clone(),
            global_tick: self.global_tick,
            total_operations: self.total_operations,
            completed_tile_count: self.completed_tiles.len(),
            fate_slice: self.triple_fate.freeze(),
        };
        let bytes = bincode::serialize(&slice)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, bytes)
    }
    
    /// 从切片恢复
    pub fn load_slice(path: &Path) -> std::io::Result<Self> {
        let bytes = std::fs::read(path)?;
        let slice: TileSlice = bincode::deserialize(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        
        let mut instance = Self::new();
        instance.id = slice.id;
        instance.global_tick = slice.global_tick;
        instance.total_operations = slice.total_operations;
        instance.triple_fate = fate_f64::TripleFateCombined::thaw(slice.fate_slice);
        Ok(instance)
    }
    
    // ═══════════════════════════════════════════════════════════════════════════
    // Space GPU 高维计算
    // ═══════════════════════════════════════════════════════════════════════════
    
    /// 计算维度 (GPU 加速)
    pub fn space_calculate_dimension(&mut self, data: &[f64]) -> valhalla_space_f64_gpu::DimensionResult {
        self.space_gpu.process_and_expand(data)
    }
    
    /// 批量计算维度 (GPU 加速)
    pub fn space_calculate_batch(&mut self, data_batch: &[Vec<f64>]) -> Vec<valhalla_space_f64_gpu::DimensionResult> {
        self.space_gpu.process_batch(data_batch)
    }
    
    /// 发现未知模式
    pub fn space_on_discovery(&mut self, data: &[f64], pattern: &str) -> String {
        self.space_gpu.on_discovery(data, pattern)
    }
    
    /// 获取当前维度
    pub fn space_dimension(&self) -> f64 {
        self.space_gpu.get_dimension()
    }
    
    /// 获取 Space 状态
    pub fn space_status(&self) -> valhalla_space_f64_gpu::SpaceStatus {
        self.space_gpu.get_status()
    }
    
    /// 接收数据流
    /// 收敛由上层 Fate 的 ClusterEquation（二元一次方程）控制
    pub fn receive_data(&mut self, data: f64) -> TileFlowResult {
        self.global_tick += 1;
        let mut new_tiles = 0usize;
        let mut discoveries = Vec::new();
        
        // 找到或创建活跃的 MicroTile
        let tile_idx = if let Some(idx) = self.active_tiles.iter()
            .position(|t| !t.is_converged()) 
        {
            idx
        } else {
            let new_tile = MicroTile::new(
                format!("tile_{}", self.global_tick),
                self.config.container_capacity,
                self.global_tick,
            );
            self.active_tiles.push(new_tile);
            self.active_tiles.len() - 1
        };
        
        // 数据流入（只收集）
        self.active_tiles[tile_idx].receive_data(data);
        
        // ═══ Fate 被动捕捉收敛信号 ═══
        // Fate 用二元一次方程检测边际递减，返回是否收敛
        let converged = self.triple_fate.receive_passive(data);
        
        // 如果 Fate 捕捉到收敛信号
        if converged {
            let tile = self.active_tiles.remove(tile_idx);
            let mean = tile.local_state.mean();
            let variance = tile.local_state.variance();
            let signal_count = tile.local_state.signal_count;
            
            // 被动 Fate：自动接收数据（忽略返回值）
            let _ = self.triple_fate.receive_passive(mean);
            
            // ═══ 纯被动：创建 Tile，不主动建立连接 ═══
            // 连接由后续 process_tiles_cycle 中 Fate 被动决定
            let completed = CompletedTile {
                id: tile.id.clone(),
                mean,
                variance,
                signal_count,
                created_tick: tile.created_tick,
                completed_tick: self.global_tick,
                data_signature: vec![mean, variance],
                energy: mean.abs().min(1.0).max(0.1),
                // 神经网络属性（初始为空，由 Fate 被动填充）
                activation: 0.0,
                weights: Vec::new(),
                bias: mean * 0.01,
                connections: Vec::new(),  // 空连接，由 Fate 被动建立
            };
            
            self.completed_tiles.push(completed);
            new_tiles = 1;
            
            self.total_operations += 1;
        }
        
        TileFlowResult {
            tick: self.global_tick,
            data_received: data,
            tile_converged: converged,
            new_tiles,
            discoveries,
            active_tiles: self.active_tiles.len(),
            total_completed: self.completed_tiles.len(),
        }
    }
    
    /// 批量接收数据
    pub fn receive_batch(&mut self, data: &[f64]) -> TileBatchResult {
        let mut total_new_tiles = 0usize;
        let mut total_discoveries = Vec::new();
        let mut tiles_converged = 0usize;
        
        for &d in data {
            let result = self.receive_data(d);
            total_new_tiles += result.new_tiles;
            total_discoveries.extend(result.discoveries);
            if result.tile_converged {
                tiles_converged += 1;
            }
        }
        
        TileBatchResult {
            data_count: data.len(),
            new_tiles: total_new_tiles,
            tiles_converged,
            discoveries: total_discoveries,
            total_completed: self.completed_tiles.len(),
            fate_stats: self.triple_fate.stats(),
        }
    }
    
    /// 并行处理四元信号
    pub fn process_quad(&mut self, signals: QuadSignals) -> QuadResults {
        self.total_operations += 1;
        
        // 简化的四元处理
        let blackhole_gravity = 1.0 / (signals.blackhole_distance.abs() + 0.1);
        let meshbrain_intelligence = signals.meshbrain_intensity.tanh();
        let multinova_radiation = if signals.multinova_energy > 0.8 { 
            signals.multinova_energy * 1.5 
        } else { 
            signals.multinova_energy 
        };
        let baseforce_activity = (signals.baseforce_coordination + 1.0) / 2.0;
        
        let combined_output = (blackhole_gravity + meshbrain_intelligence 
                             + multinova_radiation + baseforce_activity) / 4.0;
        
        // 发送信号给 Fate
        let signal_values = vec![
            blackhole_gravity,
            meshbrain_intelligence,
            multinova_radiation,
            baseforce_activity,
        ];
        self.triple_fate.tick(Some(signal_values));
        
        QuadResults {
            blackhole_gravity,
            meshbrain_intelligence,
            multinova_radiation,
            baseforce_activity,
            combined_output,
        }
    }
    
    /// 获取当前维度
    pub fn get_dimension(&self) -> f64 {
        self.space.current_dimension
    }
    
    /// 获取系统状态
    pub fn get_status(&self) -> TileQuattroStatus {
        TileQuattroStatus {
            tile_id: self.id.clone(),
            total_operations: self.total_operations,
            completed_tiles: self.completed_tiles.len(),
            active_tiles: self.active_tiles.len(),
            dimension: self.space.current_dimension,
            fate_tick: self.triple_fate.global_tick,
            base_generation: self.triple_fate.base.generation,
            meta_generation: self.triple_fate.meta.generation,
        }
    }
    
    /// 纯 Tile 周期处理（第2周期+使用）
    /// 使用边际递减合并，与 StemCell 同样逻辑
    pub fn process_tiles_cycle(&mut self) -> TileCycleResult {
        self.total_operations += 1;
        let initial_count = self.completed_tiles.len();
        let mut splits = 0usize;
        let mut merges = 0usize;
        
        // 1. Fate 感知整体状态
        let total_energy: f64 = self.completed_tiles.iter().map(|t| t.energy).sum();
        let avg_energy = if self.completed_tiles.is_empty() { 0.0 } 
                         else { total_energy / self.completed_tiles.len() as f64 };
        let _ = self.triple_fate.receive_passive(avg_energy);
        
        // 2. 合并：只在 Tile 数量较少时执行（避免 O(n log n) 排序）
        if initial_count < 100_000 {
            self.execute_cluster_merge();
            merges = initial_count.saturating_sub(self.completed_tiles.len());
        }
        
        // 3. Tile 演化：能量流转
        let total_energy: f64 = self.completed_tiles.iter().map(|t| t.energy).sum();
        let avg_energy = if self.completed_tiles.is_empty() { 0.0 } 
                         else { total_energy / self.completed_tiles.len() as f64 };
        
        // 分裂数据：能量, 数据特征, 连接（继承父 Tile）
        let mut new_tiles_data: Vec<(f64, Vec<f64>, Vec<String>)> = Vec::new();
        
        self.completed_tiles.retain_mut(|tile| {
            // 能量根据与平均值的差异流动
            let energy_diff = avg_energy - tile.energy;
            tile.energy += energy_diff * 0.1;
            
            // 高能量 Tile 可能分裂
            if tile.energy > avg_energy * 1.5 && new_tiles_data.len() < 10 {
                tile.energy *= 0.5;
                new_tiles_data.push((tile.energy, tile.data_signature.clone(), tile.connections.clone()));
            }
            
            // 低能量 Tile 死亡
            if tile.energy < avg_energy * 0.1 || tile.energy < 0.01 {
                return false;
            }
            
            true
        });
        
        // 添加分裂产生的新 Tile（继承父 Tile 的连接）
        splits = new_tiles_data.len();
        let tick = self.global_tick;
        let op = self.total_operations;
        for (i, (energy, parent_signature, parent_connections)) in new_tiles_data.into_iter().enumerate() {
            let new_id = format!("split_{}_{}", op, i);
            self.completed_tiles.push(CompletedTile {
                id: new_id.clone(),
                mean: parent_signature.first().cloned().unwrap_or(0.0),
                variance: 0.0,
                signal_count: 0,
                created_tick: tick,
                completed_tick: tick,
                data_signature: parent_signature,
                energy,
                // 神经网络属性：继承部分连接
                activation: 0.0,
                weights: vec![0.1; parent_connections.len().min(4)],
                bias: 0.0,
                connections: parent_connections.into_iter().take(4).collect(),
            });
        }
        
        // 连接在交互对话时才建立，训练时不建立
        
        // 4. 前向传播 - 信号沿连接传递
        let prop_result = self.propagate(avg_energy);
        
        // 更新维度
        let signal = vec![self.completed_tiles.len() as f64, avg_energy, splits as f64, merges as f64];
        self.space.adaptive_expand(&signal);
        
        TileCycleResult {
            generation: self.total_operations,
            tile_count: self.completed_tiles.len(),
            merges,
            splits,
            avg_activation: prop_result.avg_activation,
            total_connections: prop_result.total_connections,
            new_connections: prop_result.new_connections,
        }
    }
    
    /// 执行 Tile 聚合合并 - 边际递减判断相似度
    /// 与 StemCell 同样逻辑：排序 + 边际递减检测 + 删除重复保留差异
    fn execute_cluster_merge(&mut self) {
        let n = self.completed_tiles.len();
        if n < 4 {
            return;
        }
        
        // 1. 按 data_signature 排序（信号特征即身份）
        self.completed_tiles.sort_by(|a, b| {
            let sig_a = a.data_signature.first().unwrap_or(&0.0);
            let sig_b = b.data_signature.first().unwrap_or(&0.0);
            sig_a.partial_cmp(sig_b).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // 2. 计算相邻 Tile 的差异序列
        let mut diffs: Vec<f64> = Vec::with_capacity(n - 1);
        for i in 1..n {
            let diff = Self::compute_tile_diff(&self.completed_tiles[i - 1], &self.completed_tiles[i]);
            diffs.push(diff);
        }
        
        if diffs.len() < 3 {
            return;
        }
        
        // 3. 用边际递减标记待合并的 Tile
        use std::collections::HashSet;
        let mut to_merge: HashSet<usize> = HashSet::new();
        
        for i in 2..diffs.len() {
            let d0 = diffs[i];
            let d1 = diffs[i - 1];
            let d2 = diffs[i - 2];
            
            // 边际递减：d0 < d1 < d2 表示趋近收敛（相似）
            if d0 < d1 && d1 < d2 && d0 > 0.0 {
                if i + 1 < n {
                    to_merge.insert(i + 1);
                }
            }
        }
        
        if to_merge.is_empty() {
            return;
        }
        
        // 4. 合并：删除重复部分，保留不同部分
        let merge_data: Vec<_> = to_merge.iter()
            .filter(|&&idx| idx > 0 && idx < self.completed_tiles.len())
            .map(|&idx| {
                let absorbed = &self.completed_tiles[idx];
                (idx - 1, absorbed.energy, absorbed.signal_count, 
                 absorbed.data_signature.clone(), absorbed.variance)
            })
            .collect();
        
        for (target_idx, energy, signals, sig, var) in merge_data {
            let target = &mut self.completed_tiles[target_idx];
            
            // 能量合并
            target.energy += energy;
            target.signal_count += signals;
            
            // 特征合并：保留差异，删除重复
            for (i, &s) in sig.iter().enumerate() {
                if i < target.data_signature.len() {
                    target.data_signature[i] = (target.data_signature[i] + s) / 2.0;
                } else {
                    target.data_signature.push(s);
                }
            }
            
            target.variance = (target.variance + var) / 2.0;
        }
        
        // 5. 一次性移除所有标记的 Tile
        let mut current_idx = 0usize;
        self.completed_tiles.retain(|_| {
            let keep = !to_merge.contains(&current_idx);
            current_idx += 1;
            keep
        });
    }
    
    /// 计算两个 Tile 的信号差异
    fn compute_tile_diff(a: &CompletedTile, b: &CompletedTile) -> f64 {
        let mut diff = 0.0;
        
        // data_signature 差异
        let len = a.data_signature.len().min(b.data_signature.len());
        if len > 0 {
            for i in 0..len {
                diff += (a.data_signature[i] - b.data_signature[i]).abs();
            }
            diff /= len as f64;
        }
        
        // variance 差异
        diff += (a.variance - b.variance).abs();
        
        diff
    }
    
    /// ═══ Fate 被动建立连接 ═══
    /// 在演化周期（交互时）由 Fate 决定建立连接
    /// 高效实现：只处理连接少的 Tile，用相邻比较
    fn fate_passive_connect(&mut self) {
        let n = self.completed_tiles.len();
        if n < 2 {
            return;
        }
        
        // 收集需要建立连接的 Tile 对 (从, 到)
        let mut new_connections: Vec<(usize, usize)> = Vec::new();
        
        // 遍历连接少的 Tile（< 2 个连接）
        for i in 0..n {
            if self.completed_tiles[i].connections.len() >= 2 {
                continue;  // 已有足够连接
            }
            
            // 找相邻的 Tile（已按 data_signature 排序）
            let neighbors = [
                i.saturating_sub(1),
                (i + 1).min(n - 1),
            ];
            
            for &j in &neighbors {
                if i == j {
                    continue;
                }
                
                // 计算差异
                let diff_ij = Self::compute_tile_diff(
                    &self.completed_tiles[i], 
                    &self.completed_tiles[j]
                );
                
                // 用 Fate 方程判断是否连接
                // x = diff / (上一个diff + 1)，避免除零
                let x = diff_ij / (diff_ij + 1.0);
                let result = self.triple_fate.cluster_equation.evaluate(x);
                
                // 方程结果小于阈值时建立连接
                if result.abs() < self.triple_fate.cluster_equation.threshold * 10.0 {
                    new_connections.push((i, j));
                }
            }
        }
        
        // 建立连接（限制每次最多建立 100 个新连接）
        for (i, j) in new_connections.into_iter().take(100) {
            let id_j = self.completed_tiles[j].id.clone();
            let id_i = self.completed_tiles[i].id.clone();
            
            // i -> j
            if !self.completed_tiles[i].connections.contains(&id_j) {
                self.completed_tiles[i].connections.push(id_j);
                self.completed_tiles[i].weights.push(0.1);
            }
            
            // j -> i (双向)
            if !self.completed_tiles[j].connections.contains(&id_i) {
                self.completed_tiles[j].connections.push(id_i);
                self.completed_tiles[j].weights.push(0.1);
            }
        }
    }
    
    /// 查找相似的 Tile（用于自发连接）
    /// 使用边际递减判断相似度
    fn find_similar_tiles(&self, mean: f64, max_count: usize) -> Vec<String> {
        if self.completed_tiles.len() < 4 {
            return self.completed_tiles.iter().map(|t| t.id.clone()).collect();
        }
        
        let new_sig = mean;
        
        // 找到最接近的 Tile
        let mut candidates: Vec<(usize, f64)> = self.completed_tiles.iter()
            .enumerate()
            .filter_map(|(i, t)| {
                t.data_signature.first().map(|&sig| (i, (sig - new_sig).abs()))
            })
            .collect();
        
        candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        
        if candidates.len() < 4 {
            return candidates.iter()
                .take(max_count)
                .map(|(i, _)| self.completed_tiles[*i].id.clone())
                .collect();
        }
        
        // 使用动态方程筛选真正相似的（而不是硬编码 d1 < d2 < d3）
        let mut similar = Vec::new();
        
        for i in 0..candidates.len().min(max_count) {
            if i + 1 < candidates.len() {
                let d1 = candidates[i].1;
                let d2 = candidates[i + 1].1;
                
                // 用动态方程判断相似性
                // x = d1 / d2，如果 x ≈ 1（稳定），则相似
                let x = if d2 > 1e-10 { d1 / d2 } else { 1.0 };
                // 简化的方程评估：(x-1)² < 0.1
                let result = (x - 1.0) * (x - 1.0);
                
                if result < 0.25 {  // 方程判定为相似
                    similar.push(self.completed_tiles[candidates[i].0].id.clone());
                }
            } else {
                // 不够做方程检测，直接加入
                similar.push(self.completed_tiles[candidates[i].0].id.clone());
            }
            
            if similar.len() >= max_count {
                break;
            }
        }
        
        similar
    }
    
    /// 前向传播 - 信号沿连接传递
    pub fn propagate(&mut self, input_signal: f64) -> PropagationResult {
        let mut total_activation = 0.0;
        let mut propagated = 0usize;
        let mut new_connections = 0usize;
        
        // 训练时跳过复杂传播（Tile 太多）
        // 只在交互时（Tile < 10万）做完整传播
        if self.completed_tiles.len() > 100_000 {
            // 只计算平均激活
            for tile in &mut self.completed_tiles {
                tile.activation = (input_signal * tile.energy).tanh();
                total_activation += tile.activation;
            }
            
            let total_conns: usize = self.completed_tiles.iter()
                .map(|t| t.connections.len()).sum();
            
            return PropagationResult {
                avg_activation: total_activation / self.completed_tiles.len().max(1) as f64,
                total_activation,
                total_connections: total_conns,
                propagated_tiles: self.completed_tiles.len(),
                new_connections: 0,
            };
        }
        
        // 收集所有 Tile 的激活和连接信息
        let tile_ids: Vec<String> = self.completed_tiles.iter().map(|t| t.id.clone()).collect();
        let activations: std::collections::HashMap<String, f64> = self.completed_tiles.iter()
            .map(|t| (t.id.clone(), t.activation))
            .collect();
        
        for tile in &mut self.completed_tiles {
            // 计算输入：来自连接的加权和
            let mut connection_input = 0.0;
            for (i, conn_id) in tile.connections.iter().enumerate() {
                if let Some(&conn_activation) = activations.get(conn_id) {
                    let weight = tile.weights.get(i).copied().unwrap_or(0.1);
                    connection_input += conn_activation * weight;
                }
            }
            
            // 加上外部输入信号
            let raw = input_signal * tile.energy + connection_input + tile.bias;
            
            // 激活函数 (tanh)
            tile.activation = raw.tanh();
            
            // 自适应权重调整（Hebbian-like）
            for (i, conn_id) in tile.connections.iter().enumerate() {
                if let Some(&conn_activation) = activations.get(conn_id) {
                    if i < tile.weights.len() {
                        // 同时激活则增强连接
                        let delta = tile.activation * conn_activation * 0.01;
                        tile.weights[i] = (tile.weights[i] + delta).clamp(-1.0, 1.0);
                    }
                }
            }
            
            // 自发形成新连接：高激活 Tile 尝试连接到其他高激活 Tile
            if tile.activation.abs() > 0.5 && tile.connections.len() < 8 {
                for other_id in &tile_ids {
                    if other_id != &tile.id && !tile.connections.contains(other_id) {
                        if let Some(&other_act) = activations.get(other_id) {
                            if other_act.abs() > 0.3 {
                                tile.connections.push(other_id.clone());
                                tile.weights.push(0.1);
                                new_connections += 1;
                                break;  // 每次只添加一个新连接
                            }
                        }
                    }
                }
            }
            
            total_activation += tile.activation;
            propagated += 1;
        }
        
        PropagationResult {
            propagated_tiles: propagated,
            total_activation,
            avg_activation: if propagated > 0 { total_activation / propagated as f64 } else { 0.0 },
            new_connections,
            total_connections: self.completed_tiles.iter().map(|t| t.connections.len()).sum(),
        }
    }
    
    /// 获取 Fate 统计
    pub fn get_fate_stats(&self) -> String {
        self.triple_fate.stats()
    }
    
    /// 检查 Fate 是否收敛（被动模式）
    pub fn has_fate_converged(&self) -> bool {
        self.triple_fate.is_passively_converged()
    }
    
    /// 获取效率历史
    pub fn get_efficiency_history(&self) -> &[(f64, f64)] {
        &self.triple_fate.efficiency_history
    }
}

impl Default for TileQuattroComplete {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 结果类型
// ============================================================================

/// Tile 数据流结果
#[derive(Debug, Clone)]
pub struct TileFlowResult {
    pub tick: u64,
    pub data_received: f64,
    pub tile_converged: bool,
    pub new_tiles: usize,
    pub discoveries: Vec<TripleDiscovery>,
    pub active_tiles: usize,
    pub total_completed: usize,
}

/// 批量 Tile 结果
#[derive(Debug, Clone)]
pub struct TileBatchResult {
    pub data_count: usize,
    pub new_tiles: usize,
    pub tiles_converged: usize,
    pub discoveries: Vec<TripleDiscovery>,
    pub total_completed: usize,
    pub fate_stats: String,
}

/// 系统状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileQuattroStatus {
    pub tile_id: String,
    pub total_operations: u64,
    pub completed_tiles: usize,
    pub active_tiles: usize,
    pub dimension: f64,
    pub fate_tick: u64,
    pub base_generation: u64,
    pub meta_generation: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_micro_tile_convergence() {
        let mut tile = MicroTile::new("test".into(), 100, 0);
        
        for i in 0..20 {
            let value = 10.0 / (i as f64 + 1.0);
            tile.receive_data(value);
        }
        
        println!("Converged: {}", tile.is_converged());
    }
    
    #[test]
    fn test_data_flow() {
        let mut system = TileQuattroComplete::new();
        
        for i in 0..500 {
            let data = (i % 50) as f64 + (i as f64).sin();
            let result = system.receive_data(data);
            
            if result.tile_converged {
                println!("Tile converged at tick {}", result.tick);
            }
        }
        
        let status = system.get_status();
        println!("Final status: {:?}", status);
    }
    
    #[test]
    fn test_quad_processing() {
        let mut system = TileQuattroComplete::new();
        
        let signals = QuadSignals {
            blackhole_distance: 10.0,
            meshbrain_intensity: 0.5,
            multinova_energy: 0.8,
            baseforce_coordination: 0.3,
        };
        
        let results = system.process_quad(signals);
        println!("Results: {:?}", results);
        assert!(results.combined_output > 0.0);
    }
}
