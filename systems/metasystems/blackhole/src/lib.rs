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
}

impl GravityField {
    pub fn new(config: GravityFieldConfig) -> Self {
        Self { config }
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
        let gravity = self.gravity_field.calculate_gravity(data.abs().max(0.001));
        self.triple_fate.receive_passive(gravity);
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
