//! Chamber Complete (f64 Version) - 高级容器编排系统
//!
//! 完整版本，从 valhalla-rational 移植
//! 自带: TripleFate (多层监控) + VoyagerSpace (自动维度) + 自动扩缩容
//!
//! 架构:
//! - 智能容器编排
//! - 数据流 → Container → Fate 监控
//! - VoyagerSpace 自动维度扩展
//! - 自动扩缩容
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

/// Chamber 系统切片
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChamberSlice {
    pub id: String,
    pub global_tick: u64,
    pub total_operations: u64,
    pub completed_container_count: usize,
    /// 内嵌的 Fate 切片
    pub fate_slice: FateSlice,
}

// ============================================================================
// 配置
// ============================================================================

/// Chamber 配置 (f64)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChamberConfig {
    pub max_containers: usize,
    pub auto_scale_enabled: bool,
    pub scale_up_threshold: f64,
    pub scale_down_threshold: f64,
    /// 小容器容量
    pub container_capacity: usize,
}

impl Default for ChamberConfig {
    fn default() -> Self {
        Self {
            max_containers: 100,
            auto_scale_enabled: true,
            scale_up_threshold: 0.8,
            scale_down_threshold: 0.2,
            container_capacity: 100,
        }
    }
}

// ============================================================================
// 小容器 (MicroContainer) - 数据流入单元
// ============================================================================

/// 小容器 - 数据流入并局部收敛后成为有效 Container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroContainer {
    pub id: String,
    pub data_buffer: Vec<f64>,
    pub capacity: usize,
    pub local_state: LocalContainerState,
    pub created_tick: u64,
    pub container_type: ContainerType,
}

/// 容器类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContainerType {
    Basic,
    Adaptive,
    QuadSystem,
    Fractional,
}

/// 局部容器状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalContainerState {
    pub signal_count: u64,
    pub sum: f64,
    pub sum_sq: f64,
    pub last_values: Vec<f64>,
    pub converged: bool,
    pub load: f64,
}

impl LocalContainerState {
    pub fn new() -> Self {
        Self {
            signal_count: 0,
            sum: 0.0,
            sum_sq: 0.0,
            last_values: Vec::new(),
            converged: false,
            load: 0.0,
        }
    }

    /// 接收数据并检查局部收敛
    pub fn receive(&mut self, value: f64) -> bool {
        self.signal_count += 1;
        self.sum += value;
        self.sum_sq += value * value;
        
        self.last_values.push(value);
        if self.last_values.len() > 10 {
            self.last_values.remove(0);
        }
        
        // 更新负载
        self.load = (self.signal_count as f64 / 100.0).min(1.0);
        
        // 局部收敛检测
        if self.last_values.len() >= 5 {
            let diffs: Vec<f64> = self.last_values.windows(2)
                .map(|w| (w[1] - w[0]).abs())
                .collect();
            
            if diffs.len() >= 3 {
                let decreasing = diffs[diffs.len()-1] < diffs[diffs.len()-2] 
                              && diffs[diffs.len()-2] < diffs[diffs.len()-3];
                if decreasing {
                    self.converged = true;
                }
            }
        }
        
        self.converged
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

impl Default for LocalContainerState {
    fn default() -> Self {
        Self::new()
    }
}

impl MicroContainer {
    pub fn new(id: String, capacity: usize, tick: u64, container_type: ContainerType) -> Self {
        Self {
            id,
            data_buffer: Vec::with_capacity(capacity),
            capacity,
            local_state: LocalContainerState::new(),
            created_tick: tick,
            container_type,
        }
    }
    
    pub fn receive_data(&mut self, data: f64) -> bool {
        self.data_buffer.push(data);
        self.local_state.receive(data)
    }
    
    pub fn is_converged(&self) -> bool {
        self.local_state.converged || self.data_buffer.len() >= self.capacity
    }
    
    pub fn get_load(&self) -> f64 {
        self.local_state.load
    }
}

// ============================================================================
// 已完成的容器
// ============================================================================

/// 已完成的容器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedContainer {
    pub id: String,
    pub container_type: ContainerType,
    pub mean: f64,
    pub variance: f64,
    pub signal_count: u64,
    pub created_tick: u64,
    pub completed_tick: u64,
    pub final_load: f64,
}

// ============================================================================
// Chamber Complete - 完整的容器编排系统
// ============================================================================

/// 完整的 Chamber 编排系统 (f64)
/// 自带: TripleFate (多层监控) + VoyagerSpace (自动维度) + 自动扩缩容 + 系统存储
#[derive(Debug)]
pub struct ChamberComplete {
    pub id: String,
    pub config: ChamberConfig,
    /// 多层 Fate 系统
    pub triple_fate: TripleFateCombined,
    /// 自动维度空间
    pub space: VoyagerSpaceF64,
    /// 活跃的小容器
    pub active_containers: Vec<MicroContainer>,
    /// 已完成的容器
    pub completed_containers: Vec<CompletedContainer>,
    pub total_operations: u64,
    pub global_tick: u64,
    /// 扩缩容历史
    pub scale_history: Vec<ScaleEvent>,
    /// 系统存储 - 完整的文件操作权限
    pub storage: Option<SystemStorage>,
    /// Space GPU 高维计算
    pub space_gpu: SpaceGPU,
}

/// 扩缩容事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaleEvent {
    pub tick: u64,
    pub action: ScaleAction,
    pub reason: String,
}

/// 扩缩容动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScaleAction {
    None,
    ScaleUp(String),
    ScaleDown(String),
}

impl ChamberComplete {
    pub fn new() -> Self {
        Self::with_config(ChamberConfig::default())
    }
    
    pub fn with_config(config: ChamberConfig) -> Self {
        Self {
            id: format!("chamber_complete_{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() % 1_000_000),
            config,
            triple_fate: TripleFateCombined::new(),
            space: VoyagerSpaceF64::new(),
            active_containers: Vec::new(),
            completed_containers: Vec::new(),
            total_operations: 0,
            global_tick: 0,
            scale_history: Vec::new(),
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
        self.storage = Some(SystemStorage::new(base_path, "chamber")?);
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
    
    /// 保存系统切片 (.chamber 二进制)
    pub fn save_slice(&self, path: &Path) -> std::io::Result<()> {
        let slice = ChamberSlice {
            id: self.id.clone(),
            global_tick: self.global_tick,
            total_operations: self.total_operations,
            completed_container_count: self.completed_containers.len(),
            fate_slice: self.triple_fate.freeze(),
        };
        let bytes = bincode::serialize(&slice)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, bytes)
    }
    
    /// 从切片恢复
    pub fn load_slice(path: &Path) -> std::io::Result<Self> {
        let bytes = std::fs::read(path)?;
        let slice: ChamberSlice = bincode::deserialize(&bytes)
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
    pub fn receive_data(&mut self, data: f64) -> ContainerFlowResult {
        self.global_tick += 1;
        let mut new_containers = 0usize;
        let mut discoveries = Vec::new();
        
        // 找到或创建活跃的容器
        let container_idx = if let Some(idx) = self.active_containers.iter()
            .position(|c| !c.is_converged()) 
        {
            idx
        } else {
            // 检查是否需要自动扩容
            if self.active_containers.len() >= self.config.max_containers {
                // 已达最大，尝试缩容已完成的
                self.try_auto_scale();
            }
            
            let container_type = if self.completed_containers.len() % 4 == 0 {
                ContainerType::Adaptive
            } else {
                ContainerType::Basic
            };
            
            let new_container = MicroContainer::new(
                format!("container_{}", self.global_tick),
                self.config.container_capacity,
                self.global_tick,
                container_type,
            );
            self.active_containers.push(new_container);
            self.active_containers.len() - 1
        };
        
        // 数据流入
        let converged = self.active_containers[container_idx].receive_data(data);
        
        // 如果局部收敛，闭合
        if converged {
            let container = self.active_containers.remove(container_idx);
            let completed = CompletedContainer {
                id: container.id.clone(),
                container_type: container.container_type,
                mean: container.local_state.mean(),
                variance: container.local_state.variance(),
                signal_count: container.local_state.signal_count,
                created_tick: container.created_tick,
                completed_tick: self.global_tick,
                final_load: container.local_state.load,
            };
            
            // 发送信号给 Triple-Fate
            let signal_values = vec![
                completed.mean,
                completed.variance,
                completed.signal_count as f64,
                completed.final_load,
            ];
            let fate_discoveries = self.triple_fate.tick(Some(signal_values.clone()));
            discoveries.extend(fate_discoveries);
            
            // 自动维度扩展
            self.space.adaptive_expand(&signal_values);
            
            self.completed_containers.push(completed);
            self.total_operations += 1;
            new_containers = 1;
            
            // 检查自动扩缩容
            self.try_auto_scale();
        }
        
        ContainerFlowResult {
            tick: self.global_tick,
            data_received: data,
            container_converged: converged,
            new_containers,
            discoveries,
            active_containers: self.active_containers.len(),
            total_completed: self.completed_containers.len(),
        }
    }
    
    /// 批量接收数据
    pub fn receive_batch(&mut self, data: &[f64]) -> ContainerBatchResult {
        let mut total_new_containers = 0usize;
        let mut total_discoveries = Vec::new();
        let mut containers_converged = 0usize;
        
        for &d in data {
            let result = self.receive_data(d);
            total_new_containers += result.new_containers;
            total_discoveries.extend(result.discoveries);
            if result.container_converged {
                containers_converged += 1;
            }
        }
        
        ContainerBatchResult {
            data_count: data.len(),
            new_containers: total_new_containers,
            containers_converged,
            discoveries: total_discoveries,
            total_completed: self.completed_containers.len(),
            fate_stats: self.triple_fate.stats(),
        }
    }
    
    /// 尝试自动扩缩容
    fn try_auto_scale(&mut self) {
        if !self.config.auto_scale_enabled {
            return;
        }
        
        let current_load = self.get_average_load();
        
        // 扩容
        if current_load > self.config.scale_up_threshold 
           && self.active_containers.len() < self.config.max_containers {
            let new_container = MicroContainer::new(
                format!("scaled_up_{}", self.global_tick),
                self.config.container_capacity,
                self.global_tick,
                ContainerType::Adaptive,
            );
            let id = new_container.id.clone();
            self.active_containers.push(new_container);
            self.scale_history.push(ScaleEvent {
                tick: self.global_tick,
                action: ScaleAction::ScaleUp(id.clone()),
                reason: format!("High load: {:.2}", current_load),
            });
        }
        
        // 缩容
        if current_load < self.config.scale_down_threshold 
           && self.active_containers.len() > 1 {
            // 移除最空闲的容器
            if let Some(idx) = self.active_containers.iter()
                .enumerate()
                .min_by(|a, b| a.1.get_load().partial_cmp(&b.1.get_load()).unwrap())
                .map(|(i, _)| i)
            {
                let removed = self.active_containers.remove(idx);
                self.scale_history.push(ScaleEvent {
                    tick: self.global_tick,
                    action: ScaleAction::ScaleDown(removed.id),
                    reason: format!("Low load: {:.2}", current_load),
                });
            }
        }
    }
    
    /// 获取平均负载
    pub fn get_average_load(&self) -> f64 {
        if self.active_containers.is_empty() {
            return 0.0;
        }
        self.active_containers.iter().map(|c| c.get_load()).sum::<f64>() 
            / self.active_containers.len() as f64
    }
    
    /// 创建新容器（手动）
    pub fn create_container(&mut self, container_type: ContainerType) -> Result<String, ChamberError> {
        if self.active_containers.len() >= self.config.max_containers {
            return Err(ChamberError::MaxContainersReached);
        }
        
        let container = MicroContainer::new(
            format!("manual_{}", self.global_tick),
            self.config.container_capacity,
            self.global_tick,
            container_type,
        );
        let id = container.id.clone();
        self.active_containers.push(container);
        self.total_operations += 1;
        
        Ok(id)
    }
    
    /// 销毁容器（手动）
    pub fn destroy_container(&mut self, container_id: &str) -> Result<(), ChamberError> {
        if let Some(pos) = self.active_containers.iter().position(|c| c.id == container_id) {
            self.active_containers.remove(pos);
            self.total_operations += 1;
            Ok(())
        } else {
            Err(ChamberError::ContainerNotFound)
        }
    }
    
    /// 检查 Fate 是否收敛
    pub fn has_fate_converged(&self) -> bool {
        self.triple_fate.has_converged()
    }
    
    /// 获取当前维度
    pub fn get_dimension(&self) -> f64 {
        self.space.current_dimension
    }
    
    /// 获取状态
    pub fn get_status(&self) -> ChamberStatus {
        ChamberStatus {
            chamber_id: self.id.clone(),
            active_containers: self.active_containers.len(),
            completed_containers: self.completed_containers.len(),
            total_operations: self.total_operations,
            auto_scale_enabled: self.config.auto_scale_enabled,
            average_load: self.get_average_load(),
            dimension: self.space.current_dimension,
            fate_tick: self.triple_fate.global_tick,
            scale_events: self.scale_history.len(),
        }
    }
}

impl Default for ChamberComplete {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 错误和结果类型
// ============================================================================

/// Chamber 错误
#[derive(Debug, Clone)]
pub enum ChamberError {
    MaxContainersReached,
    ContainerNotFound,
    SpaceError(String),
}

/// 容器数据流结果
#[derive(Debug, Clone)]
pub struct ContainerFlowResult {
    pub tick: u64,
    pub data_received: f64,
    pub container_converged: bool,
    pub new_containers: usize,
    pub discoveries: Vec<TripleDiscovery>,
    pub active_containers: usize,
    pub total_completed: usize,
}

/// 批量容器结果
#[derive(Debug, Clone)]
pub struct ContainerBatchResult {
    pub data_count: usize,
    pub new_containers: usize,
    pub containers_converged: usize,
    pub discoveries: Vec<TripleDiscovery>,
    pub total_completed: usize,
    pub fate_stats: String,
}

/// Chamber 状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChamberStatus {
    pub chamber_id: String,
    pub active_containers: usize,
    pub completed_containers: usize,
    pub total_operations: u64,
    pub auto_scale_enabled: bool,
    pub average_load: f64,
    pub dimension: f64,
    pub fate_tick: u64,
    pub scale_events: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_chamber_creation() {
        let chamber = ChamberComplete::new();
        assert_eq!(chamber.active_containers.len(), 0);
    }
    
    #[test]
    fn test_data_flow() {
        let mut chamber = ChamberComplete::new();
        
        for i in 0..500 {
            let data = (i % 50) as f64 + (i as f64).sin();
            let result = chamber.receive_data(data);
            
            if result.container_converged {
                println!("Container converged at tick {}", result.tick);
            }
        }
        
        let status = chamber.get_status();
        println!("Final status: {:?}", status);
    }
    
    #[test]
    fn test_auto_scale() {
        let mut chamber = ChamberComplete::new();
        
        // 高负载数据流
        for i in 0..200 {
            chamber.receive_data(i as f64);
        }
        
        let status = chamber.get_status();
        println!("Scale events: {}", status.scale_events);
    }
    
    #[test]
    fn test_manual_container() {
        let mut chamber = ChamberComplete::new();
        
        let id = chamber.create_container(ContainerType::Adaptive).unwrap();
        assert_eq!(chamber.active_containers.len(), 1);
        
        chamber.destroy_container(&id).unwrap();
        assert_eq!(chamber.active_containers.len(), 0);
    }
}
