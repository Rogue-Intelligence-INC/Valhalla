//! MeshBrain 元系统 - 智能认知形变信号生成器
//!
//! 专长：分布式认知处理，生成形变涨落和拓扑重构信号
//! 自带 Fate 存储系统

use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use std::path::Path;
use fate_f64::{TripleFateCombined, FateSlice, FateDataEntry};
use valhalla_space_f64_gpu::SpaceGPU;

/// MeshBrain 系统切片
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshBrainSlice {
    pub config: MeshBrainConfig,
    pub fate_slice: FateSlice,
}

/// 膜脑配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshBrainConfig {
    pub core_count: usize,
    pub base_membrane_thickness: f64,
    pub network_density: f64,
    pub fluctuation_intensity: f64,
}

impl Default for MeshBrainConfig {
    fn default() -> Self {
        Self {
            core_count: 1000,
            base_membrane_thickness: 10.0,
            network_density: 0.7,
            fluctuation_intensity: 0.3,
        }
    }
}

/// 膜脑信号类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MeshBrainSignalType {
    ContinuousFluctuationMode,
    StructuralReorganizationMode,
}

/// 膜脑形变信号
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshBrainDeformationSignal {
    pub signal_type: MeshBrainSignalType,
    pub deformation_intensity: f64,
    pub core_position: [f64; 3],
    pub timestamp: SystemTime,
    pub membrane_thickness: f64,
    pub network_connectivity: f64,
    pub intelligence_level: f64,
}

/// 膜脑系统
#[derive(Debug, Clone)]
pub struct MeshBrainSystem {
    pub config: MeshBrainConfig,
    pub signal_buffer: Vec<MeshBrainDeformationSignal>,
    pub stored: Vec<Vec<u8>>,
}

impl MeshBrainSystem {
    pub fn new(config: MeshBrainConfig) -> Self {
        Self {
            config,
            signal_buffer: Vec::new(),
            stored: Vec::new(),
        }
    }

    /// 原文交给膜脑自己保存。信号仍由膜脑自己的函数生成。
    pub fn keep(&mut self, bytes: &[u8]) -> f64 {
        if bytes.is_empty() {
            return 0.0;
        }
        self.stored.push(bytes.to_vec());
        let mean = bytes.iter().map(|byte| *byte as f64).sum::<f64>() / bytes.len() as f64;
        self.generate_signal(mean, [0.0, 0.0, 0.0]).intelligence_level
    }

    pub fn stored_count(&self) -> usize {
        self.stored.len()
    }

    pub fn stored_get(&self, index: usize) -> Option<&[u8]> {
        self.stored.get(index).map(|item| item.as_slice())
    }

    /// 改膜脑自己存着的这一条。
    pub fn stored_modify(&mut self, index: usize, new_bytes: &[u8]) -> bool {
        let Some(item) = self.stored.get_mut(index) else {
            return false;
        };
        *item = new_bytes.to_vec();
        true
    }

    /// 删膜脑自己存着的这一条。
    pub fn stored_remove(&mut self, index: usize) -> Option<Vec<u8>> {
        if index >= self.stored.len() {
            return None;
        }
        Some(self.stored.remove(index))
    }

    pub fn stored_clear(&mut self) {
        self.stored.clear();
    }

    /// 膜脑自己的存档：魔数、条数、每条长度加原文。
    pub fn save_store(&self, path: &Path) -> std::io::Result<()> {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"MBSTORE1");
        buf.extend_from_slice(&(self.stored.len() as u64).to_le_bytes());
        for item in &self.stored {
            buf.extend_from_slice(&(item.len() as u64).to_le_bytes());
            buf.extend_from_slice(item);
        }
        std::fs::write(path, buf)
    }

    pub fn load_store(path: &Path) -> std::io::Result<Self> {
        let bytes = std::fs::read(path)?;
        let bad = || std::io::Error::new(std::io::ErrorKind::InvalidData, "膜脑存档格式不对");
        if bytes.len() < 16 || &bytes[..8] != b"MBSTORE1" {
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
        let mut system = Self::new(MeshBrainConfig::default());
        for _ in 0..count {
            let len = u64::from_le_bytes(take(&mut at, 8)?.try_into().map_err(|_| bad())?) as usize;
            system.stored.push(take(&mut at, len)?.to_vec());
        }
        Ok(system)
    }

    /// 生成认知信号
    pub fn generate_signal(&mut self, intensity: f64, position: [f64; 3]) -> MeshBrainDeformationSignal {
        let signal = MeshBrainDeformationSignal {
            signal_type: if intensity > 0.5 {
                MeshBrainSignalType::StructuralReorganizationMode
            } else {
                MeshBrainSignalType::ContinuousFluctuationMode
            },
            deformation_intensity: intensity,
            core_position: position,
            timestamp: SystemTime::now(),
            membrane_thickness: self.config.base_membrane_thickness * (1.0 + intensity * 0.1),
            network_connectivity: self.config.network_density,
            intelligence_level: intensity * self.config.fluctuation_intensity,
        };
        self.signal_buffer.push(signal.clone());
        signal
    }
}

// ============================================================================
// MeshBrain Complete - 带 Fate 存储的完整系统
// ============================================================================

/// 完整的 MeshBrain 系统（带 Fate 存储）
#[derive(Debug)]
pub struct MeshBrainComplete {
    pub config: MeshBrainConfig,
    pub system: MeshBrainSystem,
    pub triple_fate: TripleFateCombined,
    pub global_tick: u64,
    /// Space GPU 高维计算
    pub space_gpu: SpaceGPU,
}

impl MeshBrainComplete {
    pub fn new() -> Self {
        Self::with_config(MeshBrainConfig::default())
    }
    
    pub fn with_config(config: MeshBrainConfig) -> Self {
        Self {
            system: MeshBrainSystem::new(config.clone()),
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
    pub fn process_and_store(&mut self, intensity: f64, position: [f64; 3]) -> String {
        self.global_tick += 1;
        let closed = self.triple_fate.receive_passive(intensity);
        if !closed {
            return String::new();
        }
        let signal = self.system.generate_signal(intensity, position);
        self.triple_fate.store_data(&signal.intelligence_level.to_le_bytes())
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
        let slice = MeshBrainSlice { config: self.config.clone(), fate_slice: self.triple_fate.freeze() };
        let bytes = bincode::serialize(&slice).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, bytes)
    }
    
    /// 从切片恢复
    pub fn load_slice(path: &Path) -> std::io::Result<Self> {
        let bytes = std::fs::read(path)?;
        let slice: MeshBrainSlice = bincode::deserialize(&bytes).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let mut instance = Self::with_config(slice.config);
        instance.triple_fate = TripleFateCombined::thaw(slice.fate_slice);
        Ok(instance)
    }
}

impl Default for MeshBrainComplete {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_save_load_and_reprocess() {
        let mut system = MeshBrainSystem::new(MeshBrainConfig::default());
        system.keep(b"Awake, Watson?");
        system.keep("錐刀既無棄".as_bytes());
        assert_eq!(system.stored_count(), 2);
        let path = std::env::temp_dir().join(format!("mb_store_{}.bin", std::process::id()));
        system.save_store(&path).unwrap();
        let mut loaded = MeshBrainSystem::load_store(&path).unwrap();
        let _ = std::fs::remove_file(&path);
        assert_eq!(loaded.stored_get(0), Some(b"Awake, Watson?".as_slice()));
        assert_eq!(loaded.stored_get(1), Some("錐刀既無棄".as_bytes()));
    }

    #[test]
    fn store_modify_remove_and_clear() {
        let mut system = MeshBrainSystem::new(MeshBrainConfig::default());
        system.keep(b"first");
        system.keep(b"second");
        assert!(system.stored_modify(0, b"changed"));
        assert_eq!(system.stored_get(0), Some(b"changed".as_slice()));
        assert!(!system.stored_modify(99, b"nope"));
        assert_eq!(system.stored_remove(1), Some(b"second".to_vec()));
        assert_eq!(system.stored_count(), 1);
        assert_eq!(system.stored_remove(99), None);
        system.stored_clear();
        assert_eq!(system.stored_count(), 0);
    }
}
