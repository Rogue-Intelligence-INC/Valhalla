//! BlackHole 元系统 - 极端物理形变信号生成器
//!
//! 专长：极端物理边界探索，生成逃逸信号和反重力稳定信号
//! 自带 Fate 存储系统

use serde::{Deserialize, Serialize};
use std::path::Path;
use fate_f64::{TripleFateCombined, FateSlice, FateDataEntry};
use valhalla_space_f64_gpu::SpaceGPU;

/// BlackHole 系统切片
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlackHoleSlice {
    pub config: BlackHoleConfig,
    pub fate_slice: FateSlice,
}

/// 引力场配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GravityFieldConfig {
    pub mass: f64,
    pub critical_radius: f64,
    pub field_strength: f64,
    pub curvature_factor: f64,
}

impl Default for GravityFieldConfig {
    fn default() -> Self {
        Self {
            mass: 10.0,
            critical_radius: 2.0,
            field_strength: 1.0,
            curvature_factor: 0.5,
        }
    }
}

/// 引力场
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GravityField {
    pub config: GravityFieldConfig,
    #[serde(default)]
    pub stored: Vec<Vec<u8>>,
}

impl GravityField {
    pub fn new(config: GravityFieldConfig) -> Self {
        Self { config, stored: Vec::new() }
    }

    /// 原文交给黑洞自己保存。引力仍由黑洞自己的函数算。
    pub fn keep(&mut self, bytes: &[u8]) -> f64 {
        if bytes.is_empty() {
            return 0.0;
        }
        self.stored.push(bytes.to_vec());
        let mean = bytes.iter().map(|byte| *byte as f64).sum::<f64>() / bytes.len() as f64;
        self.calculate_gravity(mean.abs().max(1e-6))
    }

    pub fn stored_count(&self) -> usize {
        self.stored.len()
    }

    pub fn stored_get(&self, index: usize) -> Option<&[u8]> {
        self.stored.get(index).map(|item| item.as_slice())
    }

    /// 改黑洞自己存着的这一条。
    pub fn stored_modify(&mut self, index: usize, new_bytes: &[u8]) -> bool {
        let Some(item) = self.stored.get_mut(index) else {
            return false;
        };
        *item = new_bytes.to_vec();
        true
    }

    /// 删黑洞自己存着的这一条。
    pub fn stored_remove(&mut self, index: usize) -> Option<Vec<u8>> {
        if index >= self.stored.len() {
            return None;
        }
        Some(self.stored.remove(index))
    }

    pub fn stored_clear(&mut self) {
        self.stored.clear();
    }

    /// 黑洞自己的存档：魔数、条数、每条长度加原文。
    pub fn save_store(&self, path: &Path) -> std::io::Result<()> {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"BHSTORE1");
        buf.extend_from_slice(&(self.stored.len() as u64).to_le_bytes());
        for item in &self.stored {
            buf.extend_from_slice(&(item.len() as u64).to_le_bytes());
            buf.extend_from_slice(item);
        }
        std::fs::write(path, buf)
    }

    pub fn load_store(path: &Path) -> std::io::Result<Self> {
        let bytes = std::fs::read(path)?;
        let bad = || std::io::Error::new(std::io::ErrorKind::InvalidData, "黑洞存档格式不对");
        if bytes.len() < 16 || &bytes[..8] != b"BHSTORE1" {
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
        let mut field = Self::new(GravityFieldConfig::default());
        for _ in 0..count {
            let len = u64::from_le_bytes(take(&mut at, 8)?.try_into().map_err(|_| bad())?) as usize;
            field.stored.push(take(&mut at, len)?.to_vec());
        }
        Ok(field)
    }

    /// 计算引力强度
    pub fn calculate_gravity(&self, distance: f64) -> f64 {
        if distance <= 0.0 {
            return f64::INFINITY;
        }
        self.config.mass * self.config.field_strength / (distance * distance)
    }

    /// 判断是否在事件视界内
    pub fn is_within_horizon(&self, distance: f64) -> bool {
        distance < self.config.critical_radius
    }
}

/// 事件视界配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventHorizonConfig {
    pub boundary_radius: f64,
    pub information_retention_time: f64,
    pub radiation_coefficient: f64,
}

impl Default for EventHorizonConfig {
    fn default() -> Self {
        Self {
            boundary_radius: 2.0,
            information_retention_time: 1000.0,
            radiation_coefficient: 0.1,
        }
    }
}

/// 事件视界
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventHorizon {
    pub config: EventHorizonConfig,
    pub gravity_field: GravityField,
}

impl EventHorizon {
    pub fn new(config: EventHorizonConfig, gravity_field: GravityField) -> Self {
        Self { config, gravity_field }
    }

    /// 检测逃逸信号
    pub fn detect_escape_signal(&self, energy: f64, distance: f64) -> bool {
        let escape_velocity = (2.0 * self.gravity_field.calculate_gravity(distance) * distance).sqrt();
        energy > escape_velocity
    }
}

/// BlackHole 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlackHoleConfig {
    pub gravity_config: GravityFieldConfig,
    pub horizon_config: EventHorizonConfig,
}

impl Default for BlackHoleConfig {
    fn default() -> Self {
        Self {
            gravity_config: GravityFieldConfig::default(),
            horizon_config: EventHorizonConfig::default(),
        }
    }
}

// ============================================================================
// BlackHole Complete - 带 Fate 存储的完整系统
// ============================================================================

/// 完整的 BlackHole 系统（带 Fate 存储）
#[derive(Debug)]
pub struct BlackHoleComplete {
    pub config: BlackHoleConfig,
    pub gravity_field: GravityField,
    pub event_horizon: EventHorizon,
    pub triple_fate: TripleFateCombined,
    pub global_tick: u64,
    /// Space GPU 高维计算
    pub space_gpu: SpaceGPU,
}

impl BlackHoleComplete {
    pub fn new() -> Self {
        Self::with_config(BlackHoleConfig::default())
    }
    
    pub fn with_config(config: BlackHoleConfig) -> Self {
        let gravity_field = GravityField::new(config.gravity_config.clone());
        let event_horizon = EventHorizon::new(config.horizon_config.clone(), gravity_field.clone());
        Self {
            config,
            gravity_field,
            event_horizon,
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
    pub fn process_and_store(&mut self, data: f64) -> String {
        self.global_tick += 1;
        let closed = self.triple_fate.receive_passive(data);
        if !closed {
            return String::new();
        }
        let gravity = self.gravity_field.calculate_gravity(data.abs().max(0.001));
        self.triple_fate.store_data(&gravity.to_le_bytes())
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
        let slice = BlackHoleSlice { config: self.config.clone(), fate_slice: self.triple_fate.freeze() };
        let bytes = bincode::serialize(&slice).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, bytes)
    }
    
    /// 从切片恢复
    pub fn load_slice(path: &Path) -> std::io::Result<Self> {
        let bytes = std::fs::read(path)?;
        let slice: BlackHoleSlice = bincode::deserialize(&bytes).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let mut instance = Self::with_config(slice.config);
        instance.triple_fate = TripleFateCombined::thaw(slice.fate_slice);
        Ok(instance)
    }
}

impl Default for BlackHoleComplete {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_save_load_and_reprocess() {
        let mut field = GravityField::new(GravityFieldConfig::default());
        field.keep(b"Awake, Watson?");
        field.keep("錐刀既無棄".as_bytes());
        assert_eq!(field.stored_count(), 2);
        let path = std::env::temp_dir().join(format!("bh_store_{}.bin", std::process::id()));
        field.save_store(&path).unwrap();
        let mut loaded = GravityField::load_store(&path).unwrap();
        let _ = std::fs::remove_file(&path);
        assert_eq!(loaded.stored_get(0), Some(b"Awake, Watson?".as_slice()));
        assert_eq!(loaded.stored_get(1), Some("錐刀既無棄".as_bytes()));
    }

    #[test]
    fn store_modify_remove_and_clear() {
        let mut field = GravityField::new(GravityFieldConfig::default());
        field.keep(b"first");
        field.keep(b"second");
        assert!(field.stored_modify(0, b"changed"));
        assert_eq!(field.stored_get(0), Some(b"changed".as_slice()));
        assert!(!field.stored_modify(99, b"nope"));
        assert_eq!(field.stored_remove(1), Some(b"second".to_vec()));
        assert_eq!(field.stored_count(), 1);
        assert_eq!(field.stored_remove(99), None);
        field.stored_clear();
        assert_eq!(field.stored_count(), 0);
    }
}
