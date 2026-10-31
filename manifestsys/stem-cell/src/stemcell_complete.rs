//! Stem-Cell Complete (f64 Version) - 智能细胞网络
//!
//! 完整版本，从 valhalla-rational 移植
//! 自带: FateSystem + VoyagerSpace (自动维度) + TripleFate (多层监控)
//!
//! 架构:
//! - 数据流 → 小容器 → 局部收敛 → 细胞
//! - Fate 监控所有细胞行为，控制数据存储
//! - VoyagerSpace 自动维度扩展
//! - **Fate 存储**: 数据由 Fate 处理并存储，系统有完全控制权

use serde::{Deserialize, Serialize};
use std::path::Path;

// 系统存储 (旧版兼容)
pub use valhalla_storage::SystemStorage;

// 使用 fate-f64 的完整 Triple-Fate 架构
use fate_f64::{
    TripleFateCombined, FateLoop, TripleSignal, SignalSource,
    TripleDiscovery, DiscoveryType, VoyagerSpaceF64, detect_pattern_f64,
    FateSlice,
};
use valhalla_metasystems_baseforce::BaseForceCoordinator;
use valhalla_metasystems_blackhole::{GravityField, GravityFieldConfig};
use valhalla_metasystems_meshbrain::{MeshBrainConfig, MeshBrainSystem};
use valhalla_metasystems_multinova::{MultinovaConfig, MultinovaProcessor};

// Space GPU 高维计算
use valhalla_space_f64_gpu::{SpaceGPU, FractalDimensionCalculatorGPU, HyperVector};

// ============================================================================
// 系统切片 - 用于冻结/解冻
// ============================================================================

/// StemCell 系统切片
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StemCellSlice {
    pub id: String,
    pub global_tick: u64,
    pub generation_count: u32,
    pub total_divisions: u64,
    pub total_deaths: u64,
    pub total_merges: u64,
    pub cell_count: usize,
    /// 内嵌的 Fate 切片（包含所有存储数据）
    pub fate_slice: FateSlice,
}

// ============================================================================
// 小容器 (MicroContainer) - 数据流入单元
// ============================================================================

/// 小容器 - 数据流入并局部收敛后成为细胞（无冗余缓冲）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroContainer {
    pub id: String,
    pub local_state: LocalConvergenceState,
    pub created_tick: u64,
}

/// 局部收敛状态（固定数组，O(1)操作）
/// 收敛由上层 Fate 的 ClusterEquation 控制
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalConvergenceState {
    pub signal_count: u64,
    pub sum: f64,
    pub sum_sq: f64,
    /// 环形缓冲（供 Fate 分析）
    pub ring: [f64; 10],
    pub ring_idx: usize,
    pub ring_len: usize,
    pub converged: bool,
}

impl LocalConvergenceState {
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

impl Default for LocalConvergenceState {
    fn default() -> Self {
        Self::new()
    }
}

impl MicroContainer {
    pub fn new(id: String, _capacity: usize, tick: u64) -> Self {
        Self {
            id,
            local_state: LocalConvergenceState::new(),
            created_tick: tick,
        }
    }
    
    /// 接收数据（只收集，不判断收敛）
    pub fn receive_data(&mut self, data: f64) {
        self.local_state.receive(data);
    }
    
    /// 获取差异序列（供 Fate 分析）
    pub fn get_diffs(&self) -> [f64; 3] {
        self.local_state.get_diffs()
    }
    
    /// 是否已收敛（由 Fate 设置）
    pub fn is_converged(&self) -> bool {
        self.local_state.converged
    }
    
    /// 由 Fate 设置收敛
    pub fn set_converged(&mut self, converged: bool) {
        self.local_state.set_converged(converged);
    }
    
    /// 转换为细胞
    pub fn to_cell(&self, cell_id: String, current_dim: usize) -> Cell {
        let mean = self.local_state.mean();
        let variance = self.local_state.variance();
        
        // 位置由数据特征决定
        let dim = current_dim.max(2);
        let mut pos = vec![0.0; dim];
        pos[0] = mean * 0.1;
        pos[1] = variance * 0.01;
        for d in 2..dim {
            pos[d] = ((mean + variance) * (d as f64)).sin() * 0.5;
        }
        
        Cell {
            cell_id,
            energy: mean.abs().min(1.0),
            age: 0,
            generation: 0,
            // 从小容器继承的数据特征 - 这就是细胞的"身份"
            data_signature: self.local_state.last_values(),
            variance,
            signal_count: self.local_state.signal_count,
            position: pos,
            field_strength: mean.abs() * 0.1,
            connections: Vec::new(),
            content: Vec::new(),
        }
    }
}

// ============================================================================
// 细胞 (Cell) - 小容器局部收敛后的产物
// ============================================================================

/// 细胞状态 (f64) - 无预设类型，结构自由涌现
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    pub cell_id: String,
    pub energy: f64,
    pub age: u64,
    pub generation: u32,
    /// 数据特征（从小容器继承）- 这是细胞的"身份"
    pub data_signature: Vec<f64>,
    pub variance: f64,
    pub signal_count: u64,
    
    // ═══ 高维物理场 - 由 Fate 自然演化 ═══
    /// 高维位置向量 (维度由 Space 决定，不预设)
    /// Fate 抓取相对位置，能量驱动移动
    #[serde(default)]
    pub position: Vec<f64>,
    /// 能量场强度 (正=吸引源，负=排斥源)
    #[serde(default)]
    pub field_strength: f64,
    /// 与其他细胞的连接强度 (cell_id -> strength)
    /// 完全由能量场自然演化，无预设模式
    #[serde(default)]
    pub connections: Vec<(String, f64)>,
    /// 这个容器收口时组织起来的原文。骨架上的数不是原文。
    #[serde(default)]
    pub content: Vec<u8>,
}

fn keep_content(into: &mut Vec<u8>, extra: &[u8]) {
    if extra.is_empty() {
        return;
    }
    if into.len() >= extra.len() && into.windows(extra.len()).any(|window| window == extra) {
        return;
    }
    if !into.is_empty() {
        into.push(b'\n');
    }
    into.extend_from_slice(extra);
}

// ============================================================================
// StemCell Complete - 完整的细胞网络
// ============================================================================

/// 细胞网络配置 (f64)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StemCellConfig {
    pub initial_cell_count: usize,
    pub mutation_rate: f64,
    /// 小容器容量（多少数据后强制闭合）
    pub container_capacity: usize,
}

impl Default for StemCellConfig {
    fn default() -> Self {
        Self {
            initial_cell_count: 100,
            mutation_rate: 0.01,
            container_capacity: 100,
        }
    }
}

/// 完整的干细胞网络 (f64)
/// 自带: TripleFate (多层监控) + VoyagerSpace (自动维度) + 系统存储
#[derive(Debug)]
pub struct StemCellComplete {
    pub id: String,
    pub config: StemCellConfig,
    pub cells: Vec<Cell>,
    /// 多层 Fate 系统 (高层监控低层)
    pub triple_fate: TripleFateCombined,
    /// 自动维度空间
    pub space: VoyagerSpaceF64,
    /// 活跃的小容器（数据流入中）
    pub active_containers: Vec<MicroContainer>,
    pub generation_count: u32,
    pub total_divisions: u64,
    pub total_deaths: u64,
    pub total_merges: u64,
    pub total_containers_closed: u64,
    /// 确定性 RNG 状态
    rng_state: u64,
    /// 全局 tick
    global_tick: u64,
    /// 细胞数量历史（用于被动合并检测）
    cell_count_history: [usize; 10],
    history_idx: usize,
    history_len: usize,
    /// 系统存储 - 完整的文件操作权限
    pub storage: Option<SystemStorage>,
    /// Space GPU 高维计算
    pub space_gpu: SpaceGPU,
    /// 四元在细胞网络内部。只在 Fate 宣布收敛后走一次。
    pub blackhole: GravityField,
    pub meshbrain: MeshBrainSystem,
    pub multinova: MultinovaProcessor,
    pub baseforce: BaseForceCoordinator,
    /// 最近一次四元：黑洞、MeshBrain、MultiNova、BaseForce。
    pub last_channels: [f64; 4],
    /// 上一拍物理场走的设备。gpu 与 cpu 用同一套力。
    pub field_device: &'static str,
}

impl StemCellComplete {
    pub fn new() -> Self {
        Self::with_config(StemCellConfig::default())
    }
    
    pub fn with_config(config: StemCellConfig) -> Self {
        // 预分配大容量，避免频繁重新分配
        let mut cells: Vec<Cell> = Vec::with_capacity(200_000);
        for i in 0..config.initial_cell_count {
            // 初始位置由 Fate 随机分配在高维空间
            let initial_dim = 8; // 初始维度，后续由 Space 自适应
            let mut pos = vec![0.0; initial_dim];
            for (j, p) in pos.iter_mut().enumerate() {
                // 基于索引的确定性初始位置
                *p = ((i * 7 + j * 13) % 1000) as f64 / 500.0 - 1.0;
            }
            
            cells.push(Cell {
                cell_id: format!("cell_{}", i),
                energy: 0.5,
                age: 0,
                generation: 0,
                data_signature: Vec::new(),  // 空数据特征，等待数据流入后获得身份
                variance: 0.0,
                signal_count: 0,
                position: pos,
                field_strength: 0.0,  // 初始无场强
                connections: Vec::new(),  // 连接自然涌现
                content: Vec::new(),
            });
        }
        
        Self {
            id: format!("sc_{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() % 1_000_000),
            config,
            cells,
            triple_fate: TripleFateCombined::new(),
            space: VoyagerSpaceF64::new(),
            active_containers: Vec::with_capacity(10),
            generation_count: 0,
            total_divisions: 0,
            total_deaths: 0,
            total_merges: 0,
            total_containers_closed: 0,
            rng_state: 12345,
            global_tick: 0,
            cell_count_history: [0; 10],
            history_idx: 0,
            history_len: 0,
            storage: None,
            space_gpu: SpaceGPU::new(),
            blackhole: GravityField::new(GravityFieldConfig::default()),
            meshbrain: MeshBrainSystem::new(MeshBrainConfig::default()),
            multinova: MultinovaProcessor::new(MultinovaConfig::default()),
            baseforce: BaseForceCoordinator::new(),
            last_channels: [0.0; 4],
            field_device: "cpu",
        }
    }

    /// Fate 已经收下这个均值。四元只看这一次，不按字节累加。返回 BaseForce 合成的这一拍。
    fn quad_on_fate_close(&mut self, mean: f64) -> f64 {
        let distance = mean.abs().max(1e-6);
        let gravity = self.blackhole.calculate_gravity(distance);
        let mesh = self.meshbrain.generate_signal(mean, [0.0, 0.0, 0.0]);
        let nova = self.multinova.process_evolution(mean.abs());
        let mixed = self.baseforce.coordinate(gravity, mesh.intelligence_level, nova.radiation);
        self.last_channels = [gravity, mesh.intelligence_level, nova.radiation, mixed.total_activity];
        mixed.total_activity
    }

    pub fn quad_keep(&mut self, bytes: &[u8]) {
        let gravity = self.blackhole.keep(bytes);
        let mesh = self.meshbrain.keep(bytes);
        let nova = self.multinova.keep(bytes);
        let mixed = self.baseforce.keep(bytes, gravity, mesh, nova);
        self.last_channels = [gravity, mesh, nova, mixed];
    }

    /// 四元看当前容器。容器先留着，数学和 Voyager 还没走。
    pub fn quad_pulse(&mut self) -> Option<(f64, f64, f64)> {
        if self.active_containers.is_empty() || self.active_containers[0].local_state.signal_count == 0 {
            return None;
        }
        let mean = self.active_containers[0].local_state.mean();
        let variance = self.active_containers[0].local_state.variance();
        let activity = self.quad_on_fate_close(mean);
        Some((mean, variance, activity))
    }

    /// Voyager 看过数学的响应之后，这一拍写成细胞。
    pub fn commit_pulse(&mut self, mean: f64, variance: f64, activity: f64, math_degree: f64) -> bool {
        if self.active_containers.is_empty() || self.active_containers[0].local_state.signal_count == 0 {
            return false;
        }
        let signal_count = self.active_containers[0].local_state.signal_count;
        self.space.adaptive_expand(&[mean, variance, activity, math_degree]);
        let current_dim = self.space.current_dimension.ceil() as usize;
        let current_dim = current_dim.max(2);
        let mut new_pos = vec![0.0; current_dim];
        new_pos[0] = mean * 0.1;
        new_pos[1] = variance * 0.01;
        for d in 2..current_dim {
            new_pos[d] = ((mean + variance) * (d as f64)).sin() * 0.5;
        }
        self.cells.push(Cell {
            cell_id: format!("cell_{}", self.total_containers_closed),
            energy: mean.abs().min(1.0).max(0.1),
            age: 0,
            generation: 0,
            data_signature: vec![mean, variance],
            variance,
            signal_count,
            position: new_pos,
            field_strength: mean.abs() * 0.1,
            connections: Vec::new(),
            content: Vec::new(),
        });
        self.active_containers[0] = MicroContainer::new(String::new(), 0, self.global_tick);
        self.total_containers_closed += 1;
        true
    }

    /// 容器只攒数。收口由外面那一个 Fate 决定。
    pub fn ingest(&mut self, data: f64) {
        self.global_tick += 1;
        if self.active_containers.is_empty() {
            self.active_containers.push(MicroContainer::new(String::new(), 0, self.global_tick));
        }
        self.active_containers[0].receive_data(data);
    }

    /// 全局 Fate 这一拍收口。四元看当前容器的均值。
    pub fn promote_from_fate(&mut self) -> bool {
        let Some((mean, variance, activity)) = self.quad_pulse() else {
            return false;
        };
        self.commit_pulse(mean, variance, activity, 0.0)
    }

    /// 只动物理场和由此长出的连接。年龄在细胞周期里加。形变由调用方送回那一份全局 Fate。
    pub fn settle_field(&mut self) {
        self.update_physics_field();
    }

    /// 周期和落盘都跟这一份全局 Fate，不再各写各的方程。
    pub fn follow_global_fate(&mut self, global: &TripleFateCombined) {
        self.triple_fate.global_tick = global.global_tick;
        self.triple_fate.cross_feedback = global.cross_feedback;
        self.triple_fate.cluster_equation = global.cluster_equation.clone();
        self.triple_fate.base.state = global.base.state.clone();
        self.triple_fate.meta.state = global.meta.state.clone();
        self.triple_fate.efficiency_history = global.efficiency_history.clone();
    }
    
    /// 带存储路径初始化
    pub fn with_storage(base_path: &Path) -> std::io::Result<Self> {
        let mut instance = Self::new();
        instance.init_storage(base_path)?;
        Ok(instance)
    }
    
    /// 初始化系统存储
    pub fn init_storage(&mut self, base_path: &Path) -> std::io::Result<()> {
        self.storage = Some(SystemStorage::new(base_path, "stemcell")?);
        Ok(())
    }
    
    /// 获取存储（如果已初始化）
    pub fn storage(&self) -> Option<&SystemStorage> {
        self.storage.as_ref()
    }
    
    /// 获取可变存储
    pub fn storage_mut(&mut self) -> Option<&mut SystemStorage> {
        self.storage.as_mut()
    }
    
    // ==================== 数据操作快捷方法 ====================
    
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
    
    /// 列出所有存储的内容
    pub fn list_stored(&self) -> std::io::Result<Vec<String>> {
        self.storage.as_ref()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Storage not initialized"))?
            .list_all()
    }
    
    /// 接收并存储内容（数据进入系统）
    pub fn receive_content(&mut self, content: &str) -> std::io::Result<String> {
        let id = format!("data_{}", self.global_tick);
        self.store(&id, content)?;
        
        // 同时产生数值信号给细胞系统
        let signal = content.len() as f64;
        self.receive_data(signal);
        
        Ok(id)
    }
    
    // ═══════════════════════════════════════════════════════════════════════════
    // Fate 存储层 - 委托给内部 triple_fate
    // ═══════════════════════════════════════════════════════════════════════════
    
    /// 通过 Fate 存储数据（推荐）
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
    
    /// 保存系统切片 (.stemcell 二进制)
    pub fn save_slice(&self, path: &Path) -> std::io::Result<()> {
        let slice = StemCellSlice {
            id: self.id.clone(),
            global_tick: self.global_tick,
            generation_count: self.generation_count,
            total_divisions: self.total_divisions,
            total_deaths: self.total_deaths,
            total_merges: self.total_merges,
            cell_count: self.cells.len(),
            fate_slice: self.triple_fate.freeze(),
        };
        let bytes = bincode::serialize(&slice)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, bytes)
    }
    
    /// 从切片恢复
    pub fn load_slice(path: &Path) -> std::io::Result<Self> {
        let bytes = std::fs::read(path)?;
        let slice: StemCellSlice = bincode::deserialize(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        
        let mut instance = Self::new();
        instance.id = slice.id;
        instance.global_tick = slice.global_tick;
        instance.generation_count = slice.generation_count;
        instance.total_divisions = slice.total_divisions;
        instance.total_deaths = slice.total_deaths;
        instance.total_merges = slice.total_merges;
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
    
    /// 确定性 RNG
    fn next_random(&mut self) -> f64 {
        self.rng_state = self.rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.rng_state % 1000) as f64 / 1000.0
    }
    
    /// 接收数据流（核心方法）
    /// 收敛由上层 Fate 的 ClusterEquation（二元一次方程）控制
    pub fn receive_data(&mut self, data: f64) -> DataFlowResult {
        self.global_tick += 1;
        let mut new_cells = 0usize;
        
        // 使用单一容器
        if self.active_containers.is_empty() {
            self.active_containers.push(MicroContainer::new(
                String::new(), 0, self.global_tick,
            ));
        }
        
        // 数据流入（只收集）
        self.active_containers[0].receive_data(data);
        
        // ═══ Fate 被动捕捉收敛信号 ═══
        let converged = self.triple_fate.receive_passive(data);
        
        // 如果 Fate 捕捉到收敛信号
        if converged {
            let container = &self.active_containers[0];
            let mean = container.local_state.mean();
            let variance = container.local_state.variance();
            let signal_count = container.local_state.signal_count;
            
            self.quad_on_fate_close(mean);
            
            // 直接创建细胞
            // 新细胞位置 - 基于数据特征在高维空间中的位置
            let current_dim = self.space.current_dimension.ceil() as usize;
            let current_dim = current_dim.max(2);
            let mut new_pos = vec![0.0; current_dim];
            // 位置由 Fate 的数据特征决定
            new_pos[0] = mean * 0.1;
            new_pos[1] = variance * 0.01;
            for d in 2..current_dim {
                new_pos[d] = ((mean + variance) * (d as f64)).sin() * 0.5;
            }
            
            let new_cell = Cell {
                cell_id: format!("cell_{}", self.total_containers_closed),
                energy: mean.abs().min(1.0).max(0.1),
                age: 0,
                generation: 0,
                data_signature: vec![mean, variance],
                variance,
                signal_count,
                position: new_pos,
                field_strength: mean.abs() * 0.1,  // 场强由数据决定
                connections: Vec::new(),  // 连接自然涌现
                content: Vec::new(),
            };
            self.cells.push(new_cell);
            new_cells = 1;
            
            // 重置容器
            self.active_containers[0] = MicroContainer::new(
                String::new(), 0, self.global_tick,
            );
            self.total_containers_closed += 1;
        }
        
        DataFlowResult {
            tick: self.global_tick,
            data_received: data,
            container_converged: converged,
            new_cells,
            discoveries: Vec::new(),
            active_containers: 1,
            total_cells: self.cells.len(),
        }
    }
    
    /// 细胞生命周期行为 - 由 Fate 驱动，无硬编码
    fn cell_lifecycle(&mut self, _environment_signal: f64) {
        // 细胞的分裂、死亡、合并完全由 Fate 的代数方程驱动
        // 这里只更新年龄
        for cell in &mut self.cells {
            cell.age += 1;
        }
    }
    
    /// 执行细胞聚合合并 - 纯信号相似度合并，无预设类型
    /// 相似度由代数方程（边际递减）动态判断
    /// 优化版：O(n log n) 排序 + O(n) 合并
    fn execute_cluster_merge(&mut self) {
        let n = self.cells.len();
        if n < 4 {
            return;
        }
        
        // 1. 按 data_signature 排序（信号特征即身份）
        self.cells.sort_by(|a, b| {
            let sig_a = a.data_signature.first().unwrap_or(&0.0);
            let sig_b = b.data_signature.first().unwrap_or(&0.0);
            sig_a.partial_cmp(sig_b).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // 2. 计算相邻细胞的差异序列
        let mut diffs: Vec<f64> = Vec::with_capacity(n - 1);
        for i in 1..n {
            let diff = Self::compute_cell_diff(&self.cells[i - 1], &self.cells[i]);
            diffs.push(diff);
        }
        
        if diffs.len() < 3 {
            return;
        }
        
        // 3. 用代数方程（边际递减）标记待合并的细胞
        use std::collections::HashSet;
        let mut to_merge: HashSet<usize> = HashSet::new();
        
        for i in 2..diffs.len() {
            let d0 = diffs[i];
            let d1 = diffs[i - 1];

            // 纯代数判定：相邻差异的比率交给 Fate 的簇方程（check_diffs），
            // 与 try_merge_into_similar 同一条方程路，
            // 不再用写死的 d0<d1<d2 顺序不等式（那对随机间隙也有 1/6 命中）。
            if d0 > 0.0 && self.triple_fate.cluster_equation.check_diffs(&[d0, d1]) {
                if i + 1 < n {
                    to_merge.insert(i + 1);
                }
            }
        }
        
        if to_merge.is_empty() {
            return;
        }
        
        // 4. 合并：删除重复部分，保留不同部分
        // 先收集要合并的数据
        let merge_data: Vec<_> = to_merge.iter()
            .filter(|&&idx| idx > 0 && idx < self.cells.len())
            .map(|&idx| {
                let absorbed = &self.cells[idx];
                (idx - 1, absorbed.energy, absorbed.signal_count, 
                 absorbed.data_signature.clone(), absorbed.variance, absorbed.content.clone())
            })
            .collect();
        
        // 再执行合并
        for (target_idx, energy, signals, sig, var, content) in merge_data {
            let target = &mut self.cells[target_idx];
            keep_content(&mut target.content, &content);
            
            // 能量合并
            target.energy += energy;
            target.signal_count += signals;
            
            // 特征合并：保留差异，删除重复
            for (i, &s) in sig.iter().enumerate() {
                if i < target.data_signature.len() {
                    // 已有位置：取平均（删除重复）
                    target.data_signature[i] = (target.data_signature[i] + s) / 2.0;
                } else {
                    // 新位置：保留差异
                    target.data_signature.push(s);
                }
            }
            
            // variance 取平均
            target.variance = (target.variance + var) / 2.0;
        }
        
        // 5. 使用 retain 一次性移除所有标记的细胞 O(n)
        let mut current_idx = 0usize;
        self.cells.retain(|_| {
            let keep = !to_merge.contains(&current_idx);
            current_idx += 1;
            keep
        });
        
        self.total_merges += to_merge.len() as u64;
    }
    
    /// 计算两个细胞的信号差异（用于相似度判断）
    fn compute_cell_diff(a: &Cell, b: &Cell) -> f64 {
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
    
    /// 批量接收数据
    pub fn receive_batch(&mut self, data: &[f64]) -> BatchFlowResult {
        let mut total_new_cells = 0usize;
        let mut total_discoveries = Vec::new();
        let mut containers_converged = 0usize;
        
        for &d in data {
            let result = self.receive_data(d);
            total_new_cells += result.new_cells;
            total_discoveries.extend(result.discoveries);
            if result.container_converged {
                containers_converged += 1;
            }
        }
        
        BatchFlowResult {
            data_count: data.len(),
            new_cells: total_new_cells,
            containers_converged,
            discoveries: total_discoveries,
            total_cells: self.cells.len(),
            fate_stats: self.triple_fate.stats(),
        }
    }
    
    /// 纯细胞周期处理（第2周期+使用）
    /// 细胞作为整体单位进行演化、合并、分裂、死亡
    pub fn process_cells_cycle(&mut self) -> CellCycleResult {
        self.generation_count += 1;
        let initial_count = self.cells.len();
        let mut divisions = 0usize;
        let mut deaths = 0usize;
        
        // 1. Fate 从"上帝视角"看所有细胞（不逐个 tick）
        //    只需要整体统计信息
        let total_energy: f64 = self.cells.iter().map(|c| c.energy).sum();
        let avg_energy = if self.cells.is_empty() { 0.0 } else { total_energy / self.cells.len() as f64 };
        
        // 让 Fate 感知整体状态（忽略返回值）
        let _ = self.triple_fate.receive_passive(avg_energy);
        
        // 2. 执行合并：排序 + 边际递减检测 + 删除重复保留差异
        self.execute_cluster_merge();
        let merges = initial_count.saturating_sub(self.cells.len());
        
        // 3. 细胞演化：能量流转
        let total_energy: f64 = self.cells.iter().map(|c| c.energy).sum();
        let avg_energy = if self.cells.is_empty() { 0.0 } else { total_energy / self.cells.len() as f64 };
        
        // 分裂数据：能量, 代数, 数据特征（继承父细胞身份）
        let mut new_cells_data: Vec<(f64, u32, Vec<f64>, Vec<u8>)> = Vec::new();
        
        // 死亡回到原方案：身体自己的能量线，回流之后判，不加代谢、
        // 不加濒死缓冲、不搞抓取删除。死亡是物质变换的残差通道，
        // 小规模下不出现是正常现象，不人为追求它发生；死亡数随
        // 本周期状态信号进 Fate（下方 tick），Fate 只观察不执行。
        self.cells.retain_mut(|cell| {
            cell.age += 1;

            // 能量根据与平均值的差异流动
            let energy_diff = avg_energy - cell.energy;
            cell.energy += energy_diff * 0.1; // 趋向平均

            // 高能量细胞可能分裂
            if cell.energy > avg_energy * 1.5 && new_cells_data.len() < 10 {
                cell.energy *= 0.5;
                // 继承父细胞的数据特征（身份）
                new_cells_data.push((cell.energy, cell.generation, cell.data_signature.clone(), cell.content.clone()));
            }

            // 低能量细胞死亡
            if cell.energy < avg_energy * 0.1 || cell.energy < 0.01 {
                deaths += 1;
                return false;
            }

            true
        });
        
        // 添加分裂产生的新细胞（继承父细胞的数据特征）
        divisions = new_cells_data.len();
        let gen = self.generation_count;
        for (i, (energy, generation, parent_signature, parent_content)) in new_cells_data.into_iter().enumerate() {
            // 新细胞位置 = 父细胞位置 + 随机偏移 (在当前维度空间)
            let current_dim = self.space.current_dimension.ceil() as usize;
            let mut new_pos = vec![0.0; current_dim.max(1)];
            // 基于确定性 RNG 的偏移
            for (j, p) in new_pos.iter_mut().enumerate() {
                let offset = ((self.rng_state.wrapping_add(j as u64) % 1000) as f64 / 500.0 - 1.0) * 0.1;
                *p = offset;
            }
            
            self.cells.push(Cell {
                cell_id: format!("div_{}_{}", gen, i),
                energy,
                age: 0,
                generation: generation + 1,
                data_signature: parent_signature,  // 继承父细胞的身份
                variance: 0.0,
                signal_count: 0,
                position: new_pos,
                field_strength: energy * 0.5,  // 场强与能量相关
                connections: Vec::new(),  // 连接自然涌现
                content: parent_content,
            });
        }
        
        self.total_divisions += divisions as u64;
        self.total_deaths += deaths as u64;
        self.total_merges += merges as u64;

        // 更新维度
        let signal = vec![self.cells.len() as f64, avg_energy, divisions as f64, deaths as f64];
        self.space.adaptive_expand(&signal);

        // 本周期状态（含死亡数）进 Fate 的多层 tick 供 Meta 观察
        // （与 evolve 的信号同构）——Fate 观察死亡，不执行死亡。
        let _ = self.triple_fate.tick(Some(signal));
        
        CellCycleResult {
            generation: self.generation_count,
            cell_count: self.cells.len(),
            merges,
            divisions,
            deaths,
            avg_energy,
        }
    }
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 高维物理场演化 - 由 Fate 驱动，无预设模式
    // ═══════════════════════════════════════════════════════════════════════════
    
    /// 场吸引的中性点：Fate 簇方程 ax²+bx+c 的小根（无实根时取顶点横坐标）。
    /// 相似度在根之上吸引、之下排斥——中性点由方程给，不写死任何相似度数值。
    fn equation_neutral(&self) -> f64 {
        let eq = &self.triple_fate.cluster_equation;
        let (a, b, c) = (eq.coef_a, eq.coef_b, eq.coef_c);
        if a.abs() < 1e-12 {
            if b.abs() < 1e-12 {
                return 1.0;
            }
            return -c / b;
        }
        let disc = b * b - 4.0 * a * c;
        if disc < 0.0 {
            -b / (2.0 * a)
        } else {
            let sq = disc.sqrt();
            let r1 = (-b - sq) / (2.0 * a);
            let r2 = (-b + sq) / (2.0 * a);
            r1.min(r2)
        }
    }

    /// 更新高维物理场 - 能量驱动的自由吸引/排斥
    /// 完全由数据特征和 Fate 状态决定，无硬编码规则
    fn update_physics_field(&mut self) {
        let n = self.cells.len();
        if n < 2 {
            return;
        }
        
        // 当前空间维度 (由 Fate/Space 动态决定)
        let current_dim = self.space.current_dimension.ceil() as usize;
        let current_dim = current_dim.max(2);
        
        // 确保所有细胞的位置向量维度一致
        for cell in &mut self.cells {
            while cell.position.len() < current_dim {
                // 扩展维度时，新维度初始化为基于 data_signature 的值
                let new_val = if !cell.data_signature.is_empty() {
                    cell.data_signature[cell.position.len() % cell.data_signature.len()] * 0.1
                } else {
                    0.0
                };
                cell.position.push(new_val);
            }
            if cell.position.len() > current_dim {
                cell.position.truncate(current_dim);
            }
        }
        
        // 计算场强 - 基于数据特征的能量
        for cell in &mut self.cells {
            // 场强 = 能量 * 数据特征复杂度
            let complexity = if cell.data_signature.is_empty() {
                0.1
            } else {
                cell.data_signature.iter().map(|v| v.abs()).sum::<f64>() 
                    / cell.data_signature.len() as f64
            };
            cell.field_strength = cell.energy * complexity.sqrt();
        }

        if self.gpu_field_step(current_dim) {
            return;
        }
        self.field_device = "cpu";
        // 场吸引的中性点：簇方程的小根（GPU 路在 gpu_field_step 里同样取根）
        let neutral = self.equation_neutral();
        
        // 收集数据用于位置更新（避免借用冲突）
        let cell_data: Vec<(usize, Vec<f64>, Vec<f64>, f64)> = self.cells.iter()
            .enumerate()
            .map(|(i, c)| (i, c.position.clone(), c.data_signature.clone(), c.field_strength))
            .collect();
        
        // 计算相互作用力并更新位置
        for i in 0..n {
            let (_, pos_i, sig_i, field_i) = &cell_data[i];
            let mut force = vec![0.0; current_dim];
            
            for j in 0..n {
                if i == j { continue; }
                let (_, pos_j, sig_j, field_j) = &cell_data[j];
                
                // 计算高维距离
                let mut dist_sq = 0.0;
                for d in 0..current_dim.min(pos_i.len()).min(pos_j.len()) {
                    let diff = pos_i.get(d).unwrap_or(&0.0) - pos_j.get(d).unwrap_or(&0.0);
                    dist_sq += diff * diff;
                }
                let dist = (dist_sq + 0.01).sqrt();  // 避免除零
                
                // 计算数据特征相似度 (Fate 判定)
                let similarity = Self::compute_signature_similarity(sig_i, sig_j);

                // 力的方向：指向 j
                // 吸引/排斥的中性点是簇方程的小根：相似度在根之上吸引、之下排斥，
                // 不再写死 0.3。公式：F = (similarity - 根) * field_i * field_j / dist^2
                let attraction = (similarity - neutral) * field_i * field_j / (dist_sq + 0.1);
                
                for d in 0..current_dim.min(pos_i.len()).min(pos_j.len()) {
                    let direction = (pos_j.get(d).unwrap_or(&0.0) - pos_i.get(d).unwrap_or(&0.0)) / dist;
                    force[d] += attraction * direction * 0.01;  // 小步长
                }
            }
            
            // 更新位置
            for d in 0..current_dim.min(self.cells[i].position.len()) {
                self.cells[i].position[d] += force[d];
                // 限制位置范围
                self.cells[i].position[d] = self.cells[i].position[d].clamp(-100.0, 100.0);
            }
        }
        
        // 更新连接 - 基于高维距离，自然涌现
        self.update_connections_by_distance();
    }
    
    /// 计算数据特征相似度
    fn compute_signature_similarity(sig_a: &[f64], sig_b: &[f64]) -> f64 {
        if sig_a.is_empty() || sig_b.is_empty() {
            return 0.0;
        }
        
        // 余弦相似度
        let min_len = sig_a.len().min(sig_b.len());
        let mut dot = 0.0;
        let mut norm_a = 0.0;
        let mut norm_b = 0.0;
        
        for i in 0..min_len {
            dot += sig_a[i] * sig_b[i];
            norm_a += sig_a[i] * sig_a[i];
            norm_b += sig_b[i] * sig_b[i];
        }
        
        if norm_a < 1e-10 || norm_b < 1e-10 {
            return 0.0;
        }
        
        // 归一化到 [0, 1]
        (dot / (norm_a.sqrt() * norm_b.sqrt()) + 1.0) / 2.0
    }
    
    /// 同一套吸引公式。GPU 用 P Pᵀ 和 W P，算完再按新位置重连。
    fn gpu_field_step(&mut self, current_dim: usize) -> bool {
        #[cfg(feature = "native_cuda")]
        {
            let n = self.cells.len();
            if n < 2 || current_dim == 0 {
                return false;
            }
            let accel = match fate_f64::native_cuda::NativeCudaAccelerator::new() {
                Ok(accel) => accel,
                Err(_) => return false,
            };
            let mut packed = vec![0.0f64; n * current_dim];
            for (i, cell) in self.cells.iter().enumerate() {
                let len = current_dim.min(cell.position.len());
                packed[i * current_dim..i * current_dim + len].copy_from_slice(&cell.position[..len]);
            }
            let gram = match accel.row_gram(&packed, n, current_dim) {
                Ok(gram) => gram,
                Err(_) => return false,
            };
            // 场吸引的中性点：簇方程的小根，与 CPU 路一致
            let neutral = self.equation_neutral();
            let mut weights = vec![0.0f64; n * n];
            for i in 0..n {
                let field_i = self.cells[i].field_strength;
                for j in 0..n {
                    if i == j {
                        continue;
                    }
                    let dist_sq = (gram[i * n + i] + gram[j * n + j] - 2.0 * gram[i * n + j]).max(0.0);
                    let dist = (dist_sq + 0.01).sqrt();
                    let similarity = Self::compute_signature_similarity(
                        &self.cells[i].data_signature,
                        &self.cells[j].data_signature,
                    );
                    let attraction = (similarity - neutral) * field_i * self.cells[j].field_strength / (dist_sq + 0.1);
                    weights[i * n + j] = attraction / dist * 0.01;
                }
            }
            let forced = match accel.row_mul(&weights, &packed, n, current_dim) {
                Ok(forced) => forced,
                Err(_) => return false,
            };
            for i in 0..n {
                let mut row_sum = 0.0;
                for j in 0..n {
                    row_sum += weights[i * n + j];
                }
                for d in 0..current_dim {
                    let delta = forced[i * current_dim + d] - packed[i * current_dim + d] * row_sum;
                    self.cells[i].position[d] = (packed[i * current_dim + d] + delta).clamp(-100.0, 100.0);
                }
            }
            let mut repacked = vec![0.0f64; n * current_dim];
            for (i, cell) in self.cells.iter().enumerate() {
                let len = current_dim.min(cell.position.len());
                repacked[i * current_dim..i * current_dim + len].copy_from_slice(&cell.position[..len]);
            }
            let linked = match accel.row_gram(&repacked, n, current_dim) {
                Ok(gram) => gram,
                Err(_) => {
                    self.update_connections_by_distance();
                    self.field_device = "gpu";
                    return true;
                }
            };
            self.rewrite_connections(|i, j| {
                let dist_sq = (linked[i * n + i] + linked[j * n + j] - 2.0 * linked[i * n + j]).max(0.0);
                dist_sq.sqrt()
            });
            self.field_device = "gpu";
            return true;
        }
        #[cfg(not(feature = "native_cuda"))]
        {
            let _ = current_dim;
            false
        }
    }

    /// 更新连接 - 基于高维距离自然涌现，无预设阈值
    fn update_connections_by_distance(&mut self) {
        let n = self.cells.len();
        if n < 2 {
            return;
        }
        let positions: Vec<Vec<f64>> = self.cells.iter().map(|cell| cell.position.clone()).collect();
        self.rewrite_connections(|i, j| Self::hyperdim_distance(&positions[i], &positions[j]));
    }

    fn rewrite_connections(&mut self, dist_of: impl Fn(usize, usize) -> f64) {
        let n = self.cells.len();
        if n < 2 {
            return;
        }
        let mut all_distances: Vec<f64> = Vec::new();
        for i in 0..n {
            for j in (i + 1)..n {
                all_distances.push(dist_of(i, j));
            }
        }
        if all_distances.is_empty() {
            return;
        }
        all_distances.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let fate_efficiency = self.triple_fate.base.state.efficiency;
        let neighbor_ratio = 0.1 + fate_efficiency * 0.3;
        let threshold_idx = ((all_distances.len() as f64 * neighbor_ratio) as usize).min(all_distances.len() - 1);
        let distance_threshold = all_distances[threshold_idx];
        let ids: Vec<String> = self.cells.iter().map(|cell| cell.cell_id.clone()).collect();
        let fields: Vec<f64> = self.cells.iter().map(|cell| cell.field_strength).collect();
        for i in 0..n {
            let mut new_connections: Vec<(String, f64)> = Vec::new();
            for j in 0..n {
                if i == j {
                    continue;
                }
                let dist = dist_of(i, j);
                if dist <= distance_threshold {
                    // 连接强度 = 1 / 距离 * 场强乘积
                    let strength = (1.0 / (dist + 0.1)) * (fields[i] * fields[j]).sqrt();
                    new_connections.push((ids[j].clone(), strength));
                }
            }
            // 连接集合由距离分位阈值（Fate 效率驱动）决定，不再有写死的条数上限
            new_connections.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            
            self.cells[i].connections = new_connections;
        }
    }
    
    /// 计算高维距离
    fn hyperdim_distance(a: &[f64], b: &[f64]) -> f64 {
        let min_len = a.len().min(b.len());
        let mut sum = 0.0;
        for i in 0..min_len {
            let diff = a[i] - b[i];
            sum += diff * diff;
        }
        sum.sqrt()
    }
    
    /// 进化一代（处理现有细胞）- 无硬编码，由 Fate 驱动
    pub fn evolve(&mut self, environment_signal: f64) -> EvolutionResult {
        self.generation_count += 1;
        
        // 更新细胞年龄
        for cell in &mut self.cells {
            cell.age += 1;
        }
        
        // ═══ 更新高维物理场 - 能量自由吸引 ═══
        self.update_physics_field();
        
        // 发送信号给 Triple-Fate
        let signal_values = vec![
            self.cells.len() as f64,
            self.total_divisions as f64,
            self.total_deaths as f64,
            environment_signal,
        ];
        let discoveries = self.triple_fate.tick(Some(signal_values.clone()));
        
        // 自动维度扩展
        self.space.adaptive_expand(&signal_values);
        
        // 计算平均能量
        let average_energy = if self.cells.is_empty() {
            0.0
        } else {
            self.cells.iter().map(|c| c.energy).sum::<f64>() / self.cells.len() as f64
        };
        
        EvolutionResult {
            generation: self.generation_count,
            cell_count: self.cells.len(),
            divisions: self.total_divisions,
            deaths: self.total_deaths,
            average_energy,
            discoveries,
        }
    }
    
    /// 尝试将新数据合并到相似的现有细胞（使用动态方程）
    /// 如果 Fate 方程判定相似，合并并返回 true
    fn try_merge_into_similar(&mut self, mean: f64, _variance: f64, signal_count: u64) -> bool {
        if self.cells.len() < 3 {
            return false;
        }
        
        let new_sig = mean;  // 新数据的主要特征
        
        // 找到最接近的细胞（按数据特征排序）
        let mut candidates: Vec<(usize, f64)> = self.cells.iter()
            .enumerate()
            .filter_map(|(i, c)| {
                c.data_signature.first().map(|&sig| (i, (sig - new_sig).abs()))
            })
            .collect();
        
        candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        
        if candidates.len() < 2 {
            return false;
        }
        
        // 取最近的2个细胞的差异
        let d1 = candidates[0].1;  // 最近
        let d2 = candidates[1].1;
        
        // 用 Fate 的动态方程判断（而不是硬编码 d1 < d2 < d3 < d4）
        let diffs = [d1, d2];
        if self.triple_fate.cluster_equation.check_diffs(&diffs) && d1 > 0.0 {
            // 合并到最近的细胞
            let target_idx = candidates[0].0;
            self.cells[target_idx].energy += mean.abs().min(1.0).max(0.1);
            self.cells[target_idx].signal_count += signal_count;
            // 更新数据特征为平均值
            if let Some(sig) = self.cells[target_idx].data_signature.first_mut() {
                *sig = (*sig + mean) / 2.0;
            }
            return true;
        }
        
        false
    }
    
    /// 检查 Fate 是否收敛（被动模式）
    pub fn has_fate_converged(&self) -> bool {
        self.triple_fate.is_passively_converged()
    }
    
    /// 获取效率历史
    pub fn get_efficiency_history(&self) -> &[(f64, f64)] {
        &self.triple_fate.efficiency_history
    }
    
    /// 获取当前维度
    pub fn get_dimension(&self) -> f64 {
        self.space.current_dimension
    }
    
    /// 获取网络状态
    pub fn get_status(&self) -> StemCellStatus {
        StemCellStatus {
            network_id: self.id.clone(),
            cell_count: self.cells.len(),
            generation: self.generation_count,
            total_divisions: self.total_divisions,
            total_deaths: self.total_deaths,
            total_merges: self.total_merges,
            total_containers_closed: self.total_containers_closed,
            active_containers: self.active_containers.len(),
            dimension: self.space.current_dimension,
            fate_tick: self.triple_fate.global_tick,
            base_generation: self.triple_fate.base.generation,
            meta_generation: self.triple_fate.meta.generation,
        }
    }
    
    /// 获取 Fate 统计
    pub fn get_fate_stats(&self) -> String {
        self.triple_fate.stats()
    }
}

impl Default for StemCellComplete {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 结果类型
// ============================================================================

/// 数据流结果
#[derive(Debug, Clone)]
pub struct DataFlowResult {
    pub tick: u64,
    pub data_received: f64,
    pub container_converged: bool,
    pub new_cells: usize,
    pub discoveries: Vec<TripleDiscovery>,
    pub active_containers: usize,
    pub total_cells: usize,
}

/// 批量数据流结果
#[derive(Debug, Clone)]
pub struct BatchFlowResult {
    pub data_count: usize,
    pub new_cells: usize,
    pub containers_converged: usize,
    pub discoveries: Vec<TripleDiscovery>,
    pub total_cells: usize,
    pub fate_stats: String,
}

/// 进化结果 (f64)
#[derive(Debug, Clone)]
pub struct EvolutionResult {
    pub generation: u32,
    pub cell_count: usize,
    pub divisions: u64,
    pub deaths: u64,
    pub average_energy: f64,
    pub discoveries: Vec<TripleDiscovery>,
}

/// 细胞周期结果 (第2周期+用，纯细胞处理)
#[derive(Debug, Clone)]
pub struct CellCycleResult {
    pub generation: u32,
    pub cell_count: usize,
    pub merges: usize,
    pub divisions: usize,
    pub deaths: usize,
    pub avg_energy: f64,
}

/// 网络状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StemCellStatus {
    pub network_id: String,
    pub cell_count: usize,
    pub generation: u32,
    pub total_divisions: u64,
    pub total_deaths: u64,
    pub total_merges: u64,
    pub total_containers_closed: u64,
    pub active_containers: usize,
    pub dimension: f64,
    pub fate_tick: u64,
    pub base_generation: u64,
    pub meta_generation: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_micro_container_convergence() {
        let mut container = MicroContainer::new("test".into(), 100, 0);
        
        // 模拟收敛数据
        for i in 0..20 {
            let value = 10.0 / (i as f64 + 1.0);  // 递减序列
            container.receive_data(value);
        }
        
        println!("Converged: {}", container.is_converged());
        println!("Mean: {}", container.local_state.mean());
        println!("Variance: {}", container.local_state.variance());
    }
    
    #[test]
    fn test_data_flow() {
        let mut network = StemCellComplete::new();
        
        // 模拟数据流
        for i in 0..500 {
            let data = (i % 50) as f64 + (i as f64).sin();
            let result = network.receive_data(data);
            
            if result.container_converged {
                println!("Container converged at tick {}, new cells: {}", 
                         result.tick, result.new_cells);
            }
        }
        
        let status = network.get_status();
        println!("Final status: {:?}", status);
        println!("Fate stats:\n{}", network.get_fate_stats());
    }
    
    #[test]
    fn test_evolution_with_fate() {
        let mut network = StemCellComplete::new();
        
        for _ in 0..10 {
            let result = network.evolve(0.5);
            println!("Gen {}: cells={}, div={}, deaths={}, discoveries={}", 
                     result.generation, result.cell_count, 
                     result.divisions, result.deaths, result.discoveries.len());
        }
        
        assert!(network.cells.len() > 0);
    }
}
