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
    pub stored: Vec<Vec<u8>>,
}

impl MultinovaProcessor {
    pub fn new(config: MultinovaConfig) -> Self {
        Self {
            config,
            total_events: 0,
            stored: Vec::new(),
        }
    }

    /// 原文交给新星自己保存。演化仍由新星自己的函数做。
    pub fn keep(&mut self, bytes: &[u8]) -> f64 {
        if bytes.is_empty() {
            return 0.0;
        }
        self.stored.push(bytes.to_vec());
        let mean = bytes.iter().map(|byte| *byte as f64).sum::<f64>() / bytes.len() as f64;
        self.process_evolution(mean.abs()).radiation
    }

    pub fn stored_count(&self) -> usize {
        self.stored.len()
    }

    pub fn stored_get(&self, index: usize) -> Option<&[u8]> {
        self.stored.get(index).map(|item| item.as_slice())
    }

    /// 改新星自己存着的这一条。
    pub fn stored_modify(&mut self, index: usize, new_bytes: &[u8]) -> bool {
        let Some(item) = self.stored.get_mut(index) else {
            return false;
        };
        *item = new_bytes.to_vec();
        true
    }

    /// 删新星自己存着的这一条。
    pub fn stored_remove(&mut self, index: usize) -> Option<Vec<u8>> {
        if index >= self.stored.len() {
            return None;
        }
        Some(self.stored.remove(index))
    }

    pub fn stored_clear(&mut self) {
        self.stored.clear();
    }

    /// 新星自己的存档：魔数、条数、每条长度加原文。
    pub fn save_store(&self, path: &Path) -> std::io::Result<()> {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"MNSTORE1");
        buf.extend_from_slice(&(self.stored.len() as u64).to_le_bytes());
        for item in &self.stored {
            buf.extend_from_slice(&(item.len() as u64).to_le_bytes());
            buf.extend_from_slice(item);
        }
        std::fs::write(path, buf)
    }

    pub fn load_store(path: &Path) -> std::io::Result<Self> {
        let bytes = std::fs::read(path)?;
        let bad = || std::io::Error::new(std::io::ErrorKind::InvalidData, "新星存档格式不对");
        if bytes.len() < 16 || &bytes[..8] != b"MNSTORE1" {
            return Err(bad());
        }
        let mut at = 8usize;
        let mut take = |at: &mut usize, n: usize| -> std::io::Result<&[u8]> {
            let end = at.checked_add(n).ok_or_else(bad)?;
            let slice = bytes.get(*at..end).ok_or_else(bad)?;
            *at = end;
            Ok(slice)
        };
        let count = u64::from_le_bytes(take(&mut at, 8)?.try_into().map_err(|_| bad())?);
        let mut processor = Self::new(MultinovaConfig::default());
        for _ in 0..count {
            let len = u64::from_le_bytes(take(&mut at, 8)?.try_into().map_err(|_| bad())?) as usize;
            processor.stored.push(take(&mut at, len)?.to_vec());
        }
        Ok(processor)
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
        let closed = self.triple_fate.receive_passive(energy);
        if !closed {
            return String::new();
        }
        let event = self.processor.process_evolution(energy);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_save_load_and_reprocess() {
        let mut processor = MultinovaProcessor::new(MultinovaConfig::default());
        processor.keep(b"Awake, Watson?");
        processor.keep("錐刀既無棄".as_bytes());
        assert_eq!(processor.stored_count(), 2);
        let path = std::env::temp_dir().join(format!("mn_store_{}.bin", std::process::id()));
        processor.save_store(&path).unwrap();
        let mut loaded = MultinovaProcessor::load_store(&path).unwrap();
        let _ = std::fs::remove_file(&path);
        assert_eq!(loaded.stored_get(0), Some(b"Awake, Watson?".as_slice()));
        assert_eq!(loaded.stored_get(1), Some("錐刀既無棄".as_bytes()));
    }

    #[test]
    fn store_modify_remove_and_clear() {
        let mut processor = MultinovaProcessor::new(MultinovaConfig::default());
        processor.keep(b"first");
        processor.keep(b"second");
        assert!(processor.stored_modify(0, b"changed"));
        assert_eq!(processor.stored_get(0), Some(b"changed".as_slice()));
        assert!(!processor.stored_modify(99, b"nope"));
        assert_eq!(processor.stored_remove(1), Some(b"second".to_vec()));
        assert_eq!(processor.stored_count(), 1);
        assert_eq!(processor.stored_remove(99), None);
        processor.stored_clear();
        assert_eq!(processor.stored_count(), 0);
    }
}
