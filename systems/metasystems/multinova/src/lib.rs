//! MultiNova 元系统 - 大规模演化形变信号生成器
//!
//! 专长：大规模演化模拟，生成高能辐射信号和恒星核心信号
//! 自带 Fate 存储系统

use serde::{Deserialize, Serialize};
use std::path::Path;
use fate_f64::{TripleFateCombined, FateSlice, FateDataEntry};
use valhalla_space_f64_gpu::SpaceGPU;

/// MultiNova 系统切片
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiNovaSlice {
    pub config: MultinovaConfig,
    pub total_events: u64,
    pub fate_slice: FateSlice,
}

/// MultiNova 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultinovaConfig {
    pub galaxy_count: usize,
    pub star_density: f64,
    pub nova_threshold: f64,
    pub radiation_intensity: f64,
}

impl Default for MultinovaConfig {
    fn default() -> Self {
        Self {
            galaxy_count: 100,
            star_density: 0.5,
            nova_threshold: 0.8,
            radiation_intensity: 1.0,
        }
    }
}

/// MultiNova 处理器
#[derive(Debug, Clone)]
pub struct MultinovaProcessor {
    pub config: MultinovaConfig,
    pub total_events: u64,
}

impl MultinovaProcessor {
    pub fn new(config: MultinovaConfig) -> Self {
        Self {
            config,
            total_events: 0,
        }
    }

    /// 处理星系演化
    pub fn process_evolution(&mut self, energy: f64) -> NovaEvent {
        self.total_events += 1;
        let is_nova = energy > self.config.nova_threshold;
        NovaEvent {
            event_id: self.total_events,
            energy,
            is_nova,
            radiation: if is_nova { energy * self.config.radiation_intensity } else { 0.0 },
        }
    }
}

/// Nova 事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NovaEvent {
    pub event_id: u64,
    pub energy: f64,
    pub is_nova: bool,
    pub radiation: f64,
}

// ============================================================================
// MultiNova Complete - 带 Fate 存储的完整系统
// ============================================================================

/// 完整的 MultiNova 系统（带 Fate 存储）
#[derive(Debug)]
pub struct MultiNovaComplete {
    pub config: MultinovaConfig,
    pub processor: MultinovaProcessor,
    pub triple_fate: TripleFateCombined,
    pub global_tick: u64,
    /// Space GPU 高维计算
    pub space_gpu: SpaceGPU,
}

impl MultiNovaComplete {
    pub fn new() -> Self {
        Self::with_config(MultinovaConfig::default())
    }
    
    pub fn with_config(config: MultinovaConfig) -> Self {
        Self {
            processor: MultinovaProcessor::new(config.clone()),
            config,
            triple_fate: TripleFateCombined::new(),
            global_tick: 0,
            space_gpu: SpaceGPU::new(),
        }
    }
    
    // Space GPU 高维计算
    pub fn space_calculate(&mut self, data: &[f64]) -> valhalla_space_f64_gpu::DimensionResult {
        self.space_gpu.process_and_expand(data)
    }
    pub fn space_batch(&mut self, batch: &[Vec<f64>]) -> Vec<valhalla_space_f64_gpu::DimensionResult> {
        self.space_gpu.process_batch(batch)
    }
    pub fn space_dimension(&self) -> f64 { self.space_gpu.get_dimension() }
    
    /// 处理数据并存储
    pub fn process_and_store(&mut self, energy: f64) -> String {
        self.global_tick += 1;
        let event = self.processor.process_evolution(energy);
        self.triple_fate.receive_passive(event.radiation);
        self.triple_fate.store_data(&event.radiation.to_le_bytes())
    }
    
    // Fate 存储方法
    pub fn fate_store(&mut self, data: &[u8]) -> String { self.triple_fate.store_data(data) }
    pub fn fate_store_string(&mut self, content: &str) -> String { self.triple_fate.store_string(content) }
    pub fn fate_create(&mut self, data: Vec<u8>) -> String { self.triple_fate.create_data(data) }
    pub fn fate_modify(&mut self, id: &str, data: Vec<u8>) -> bool { self.triple_fate.modify_data(id, data) }
    pub fn fate_load(&self, id: &str) -> Option<&FateDataEntry> { self.triple_fate.load_data(id) }
    pub fn fate_remove(&mut self, id: &str) -> Option<FateDataEntry> { self.triple_fate.remove_data(id) }
    pub fn fate_list(&self) -> Vec<&String> { self.triple_fate.list_stored() }
    pub fn fate_count(&self) -> usize { self.triple_fate.stored_count() }
    
    /// 保存切片
    pub fn save_slice(&self, path: &Path) -> std::io::Result<()> {
        let slice = MultiNovaSlice { 
            config: self.config.clone(), 
            total_events: self.processor.total_events,
            fate_slice: self.triple_fate.freeze() 
        };
        let bytes = bincode::serialize(&slice).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, bytes)
    }
    
    /// 从切片恢复
    pub fn load_slice(path: &Path) -> std::io::Result<Self> {
        let bytes = std::fs::read(path)?;
        let slice: MultiNovaSlice = bincode::deserialize(&bytes).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let mut instance = Self::with_config(slice.config);
        instance.processor.total_events = slice.total_events;
        instance.triple_fate = TripleFateCombined::thaw(slice.fate_slice);
        Ok(instance)
    }
}

impl Default for MultiNovaComplete {
    fn default() -> Self { Self::new() }
}
