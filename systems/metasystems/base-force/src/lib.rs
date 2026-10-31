//! BaseForce 元系统 - 形变信号检测与协调器
//!
//! 专长：检测各元系统的形变信号，协调三系统协同工作
//! 自带 Fate 存储系统

use serde::{Deserialize, Serialize};
use std::path::Path;
use fate_f64::{TripleFateCombined, FateSlice, FateDataEntry};
use valhalla_space_f64_gpu::SpaceGPU;

/// BaseForce 系统切片
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseForceSlice {
    pub fate_slice: FateSlice,
}

/// 信号连接配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalConnectionConfig {
    pub buffer_size: usize,
    pub sensitivity: f64,
}

impl Default for SignalConnectionConfig {
    fn default() -> Self {
        Self {
            buffer_size: 100,
            sensitivity: 0.5,
        }
    }
}

/// BlackHole 信号连接器
#[derive(Debug, Clone)]
pub struct BlackHoleSignalConnector {
    pub config: SignalConnectionConfig,
    pub total_signals: u64,
}

impl BlackHoleSignalConnector {
    pub fn new() -> Self {
        Self {
            config: SignalConnectionConfig::default(),
            total_signals: 0,
        }
    }

    pub fn connect(&mut self, signal_strength: f64) -> bool {
        self.total_signals += 1;
        signal_strength > self.config.sensitivity
    }
}

impl Default for BlackHoleSignalConnector {
    fn default() -> Self {
        Self::new()
    }
}

/// MeshBrain 信号连接器
#[derive(Debug, Clone)]
pub struct MeshBrainSignalConnector {
    pub config: SignalConnectionConfig,
    pub total_signals: u64,
}

impl MeshBrainSignalConnector {
    pub fn new() -> Self {
        Self {
            config: SignalConnectionConfig::default(),
            total_signals: 0,
        }
    }

    pub fn connect(&mut self, signal_strength: f64) -> bool {
        self.total_signals += 1;
        signal_strength > self.config.sensitivity
    }
}

impl Default for MeshBrainSignalConnector {
    fn default() -> Self {
        Self::new()
    }
}

/// MultiNova 信号连接器
#[derive(Debug, Clone)]
pub struct MultiNovaSignalConnector {
    pub config: SignalConnectionConfig,
    pub total_signals: u64,
}

impl MultiNovaSignalConnector {
    pub fn new() -> Self {
        Self {
            config: SignalConnectionConfig::default(),
            total_signals: 0,
        }
    }

    pub fn connect(&mut self, signal_strength: f64) -> bool {
        self.total_signals += 1;
        signal_strength > self.config.sensitivity
    }
}

impl Default for MultiNovaSignalConnector {
    fn default() -> Self {
        Self::new()
    }
}

/// BaseForce 协调器
#[derive(Debug, Clone)]
pub struct BaseForceCoordinator {
    pub blackhole: BlackHoleSignalConnector,
    pub meshbrain: MeshBrainSignalConnector,
    pub multinova: MultiNovaSignalConnector,
    pub stored: Vec<Vec<u8>>,
}

impl BaseForceCoordinator {
    pub fn new() -> Self {
        Self {
            blackhole: BlackHoleSignalConnector::new(),
            meshbrain: MeshBrainSignalConnector::new(),
            multinova: MultiNovaSignalConnector::new(),
            stored: Vec::new(),
        }
    }

    /// 原文交给底力自己保存。协调仍由底力自己的函数做。
    pub fn keep(&mut self, bytes: &[u8], gravity: f64, mesh: f64, nova: f64) -> f64 {
        if bytes.is_empty() {
            return 0.0;
        }
        self.stored.push(bytes.to_vec());
        self.coordinate(gravity, mesh, nova).total_activity
    }

    pub fn stored_count(&self) -> usize {
        self.stored.len()
    }

    pub fn stored_get(&self, index: usize) -> Option<&[u8]> {
        self.stored.get(index).map(|item| item.as_slice())
    }

    /// 改底力自己存着的这一条。
    pub fn stored_modify(&mut self, index: usize, new_bytes: &[u8]) -> bool {
        let Some(item) = self.stored.get_mut(index) else {
            return false;
        };
        *item = new_bytes.to_vec();
        true
    }

    /// 删底力自己存着的这一条。
    pub fn stored_remove(&mut self, index: usize) -> Option<Vec<u8>> {
        if index >= self.stored.len() {
            return None;
        }
        Some(self.stored.remove(index))
    }

    pub fn stored_clear(&mut self) {
        self.stored.clear();
    }

    /// 底力自己的存档：魔数、条数、每条长度加原文。
    pub fn save_store(&self, path: &Path) -> std::io::Result<()> {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"BFSTORE1");
        buf.extend_from_slice(&(self.stored.len() as u64).to_le_bytes());
        for item in &self.stored {
            buf.extend_from_slice(&(item.len() as u64).to_le_bytes());
            buf.extend_from_slice(item);
        }
        std::fs::write(path, buf)
    }

    pub fn load_store(path: &Path) -> std::io::Result<Self> {
        let bytes = std::fs::read(path)?;
        let bad = || std::io::Error::new(std::io::ErrorKind::InvalidData, "底力存档格式不对");
        if bytes.len() < 16 || &bytes[..8] != b"BFSTORE1" {
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
        let mut coordinator = Self::new();
        for _ in 0..count {
            let len = u64::from_le_bytes(take(&mut at, 8)?.try_into().map_err(|_| bad())?) as usize;
            coordinator.stored.push(take(&mut at, len)?.to_vec());
        }
        Ok(coordinator)
    }

    /// 协调三系统
    pub fn coordinate(&mut self, bh_signal: f64, mb_signal: f64, mn_signal: f64) -> CoordinationResult {
        CoordinationResult {
            blackhole_active: self.blackhole.connect(bh_signal),
            meshbrain_active: self.meshbrain.connect(mb_signal),
            multinova_active: self.multinova.connect(mn_signal),
            total_activity: (bh_signal + mb_signal + mn_signal) / 3.0,
        }
    }
}

impl Default for BaseForceCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

/// 协调结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationResult {
    pub blackhole_active: bool,
    pub meshbrain_active: bool,
    pub multinova_active: bool,
    pub total_activity: f64,
}

// ============================================================================
// BaseForce Complete - 带 Fate 存储的完整系统
// ============================================================================

/// 完整的 BaseForce 系统（带 Fate 存储）
#[derive(Debug)]
pub struct BaseForceComplete {
    pub coordinator: BaseForceCoordinator,
    pub triple_fate: TripleFateCombined,
    pub global_tick: u64,
    /// Space GPU 高维计算
    pub space_gpu: SpaceGPU,
}

impl BaseForceComplete {
    pub fn new() -> Self {
        Self {
            coordinator: BaseForceCoordinator::new(),
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
    pub fn process_and_store(&mut self, bh: f64, mb: f64, mn: f64) -> String {
        self.global_tick += 1;
        let closed = self.triple_fate.receive_passive((bh + mb + mn) / 3.0);
        if !closed {
            return String::new();
        }
        let result = self.coordinator.coordinate(bh, mb, mn);
        self.triple_fate.store_data(&result.total_activity.to_le_bytes())
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
        let slice = BaseForceSlice { fate_slice: self.triple_fate.freeze() };
        let bytes = bincode::serialize(&slice).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, bytes)
    }
    
    /// 从切片恢复
    pub fn load_slice(path: &Path) -> std::io::Result<Self> {
        let bytes = std::fs::read(path)?;
        let slice: BaseForceSlice = bincode::deserialize(&bytes).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let mut instance = Self::new();
        instance.triple_fate = TripleFateCombined::thaw(slice.fate_slice);
        Ok(instance)
    }
}

impl Default for BaseForceComplete {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_save_load_and_reprocess() {
        let mut coordinator = BaseForceCoordinator::new();
        coordinator.keep(b"Awake, Watson?", 1.0, 2.0, 3.0);
        coordinator.keep("錐刀既無棄".as_bytes(), 1.0, 2.0, 3.0);
        assert_eq!(coordinator.stored_count(), 2);
        let path = std::env::temp_dir().join(format!("bf_store_{}.bin", std::process::id()));
        coordinator.save_store(&path).unwrap();
        let mut loaded = BaseForceCoordinator::load_store(&path).unwrap();
        let _ = std::fs::remove_file(&path);
        assert_eq!(loaded.stored_get(0), Some(b"Awake, Watson?".as_slice()));
        assert_eq!(loaded.stored_get(1), Some("錐刀既無棄".as_bytes()));
    }

    #[test]
    fn store_modify_remove_and_clear() {
        let mut coordinator = BaseForceCoordinator::new();
        coordinator.keep(b"first", 1.0, 2.0, 3.0);
        coordinator.keep(b"second", 1.0, 2.0, 3.0);
        assert!(coordinator.stored_modify(0, b"changed"));
        assert_eq!(coordinator.stored_get(0), Some(b"changed".as_slice()));
        assert!(!coordinator.stored_modify(99, b"nope"));
        assert_eq!(coordinator.stored_remove(1), Some(b"second".to_vec()));
        assert_eq!(coordinator.stored_count(), 1);
        assert_eq!(coordinator.stored_remove(99), None);
        coordinator.stored_clear();
        assert_eq!(coordinator.stored_count(), 0);
    }
}
