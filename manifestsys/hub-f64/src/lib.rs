//! Valhalla Hub System - f64 GPU-Accelerated Performance Version
//!
//! Unified entry point integrating all systems
//! Uses f64 for high-performance GPU-accelerated processing
//!
//! Features:
//! - Fate-f64 signal coordination
//! - Quad systems processing
//! - Batch operations for GPU acceleration
//! - 70% resource utilization target
//! - **系统存储**: 完整的文件操作权限

use serde::{Deserialize, Serialize};
use log::info;
use std::collections::HashMap;
use std::time::Instant;
use std::path::Path;
use uuid::Uuid;

pub mod cortex;
pub mod dl_front;
pub mod fair_score;
pub mod fair_suite;
pub mod grinder;
pub mod patch_export;
pub mod pipeline;
pub mod task_decompose;
#[cfg(feature = "patch-lm")]
pub mod patch_lm;
pub mod hybrid;
pub mod mcq_fate;
pub mod mcq_pairwise;
pub mod multimodal;
pub mod native_qa;
pub mod signal_ingress;
pub mod smoke_report;

pub use native_qa::{run_native_qa, run_native_qa_batch, NativeQaBatchRequest, NativeQaRequest, NativeQaResponse};
pub use dl_front::{
    encode_modality, build_body_ports, hash_embed, project_port, BodyFrontPorts, DlFrontConfig,
    ModalityInput, DL_EMBED_DIM,
};
pub use signal_ingress::{
    build_model_patch, encode_payload, BodyTarget, EncodingMode, IngressBudget, IngressPayload,
    IngressStats, StemOrganCluster, TriadFeedReport, TriadSession, ValhallaModelPatch,
    PATCH_VECTOR_DIM,
};
pub use smoke_report::{SmokeReport, SMOKE_CORPUS};

// 系统存储
pub use valhalla_storage::{SystemStorage, AllStorages, init_all_storages};

// Fate f64 (GPU-ready)
pub use fate_f64::{
    FateSystem, DeformationSignal, SystemType as FateSystemType,
    SystemPreferences, ResourceLimits, SignalBatch, FateEvaluation,
    TripleFateCombined, FateSlice, FateDataEntry,
};

// Quad systems (f64)
pub use valhalla_metasystems_blackhole::{GravityField, GravityFieldConfig, EventHorizon, EventHorizonConfig};
pub use valhalla_metasystems_meshbrain::{MeshBrainSystem, MeshBrainConfig};
pub use valhalla_metasystems_multinova::{MultinovaProcessor, MultinovaConfig};
pub use valhalla_metasystems_baseforce::{BaseForceCoordinator, CoordinationResult};

// Space GPU 高维计算
pub use valhalla_space_f64_gpu::{SpaceGPU, FractalDimensionCalculatorGPU, HyperVector, DimensionResult, SpaceStatus};

/// Hub configuration (f64)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubConfig {
    pub max_requests_per_second: u64,
    pub routing_threshold: f64,
    pub load_balance_enabled: bool,
    pub resource_percent: u8,
    pub batch_size: usize,
}

impl Default for HubConfig {
    fn default() -> Self {
        Self {
            max_requests_per_second: 100000,
            routing_threshold: 0.5,
            load_balance_enabled: false, // Use Fate routing by default
            resource_percent: 70,
            batch_size: 1024,
        }
    }
}

/// Quad system type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum QuadSystemType {
    BlackHole,
    MeshBrain,
    MultiNova,
    BaseForce,
}

impl From<QuadSystemType> for FateSystemType {
    fn from(q: QuadSystemType) -> Self {
        match q {
            QuadSystemType::BlackHole => FateSystemType::BlackHole,
            QuadSystemType::MeshBrain => FateSystemType::MeshBrain,
            QuadSystemType::MultiNova => FateSystemType::MultiNova,
            QuadSystemType::BaseForce => FateSystemType::BaseForce,
        }
    }
}

/// System status (f64)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub is_initialized: bool,
    pub active_systems: Vec<String>,
    pub system_health: HashMap<String, f64>,
    pub last_update_tick: u64,
    pub fate_signal_tick: u64,
}

/// Hub statistics (f64)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubStatistics {
    pub total_requests: u64,
    pub successful_routes: u64,
    pub failed_routes: u64,
    pub system_usage: HashMap<String, u64>,
    pub average_response_time_ms: f64,
    pub total_processing_time_ms: f64,
}

/// Hub 系统切片
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubSlice {
    pub hub_id: String,
    pub statistics: HubStatistics,
    /// 内嵌的 Fate 切片
    pub fate_slice: FateSlice,
}

/// Valhalla Hub (f64 GPU-Accelerated)
pub struct ValhallaHub {
    pub hub_id: String,
    pub config: HubConfig,
    
    // Fate System (f64)
    pub fate_system: FateSystem,
    
    /// Triple-Fate 存储系统
    pub triple_fate: TripleFateCombined,
    
    // Quad Systems (f64)
    pub gravity_field: GravityField,
    pub event_horizon: EventHorizon,
    pub meshbrain: MeshBrainSystem,
    pub multinova: MultinovaProcessor,
    pub baseforce: BaseForceCoordinator,
    
    pub system_status: SystemStatus,
    pub statistics: HubStatistics,
    
    /// 系统存储 - Hub 自身的存储
    pub storage: Option<SystemStorage>,
    /// 所有子系统的存储
    pub all_storages: Option<AllStorages>,
    /// Space GPU 高维计算
    pub space_gpu: SpaceGPU,
}

impl ValhallaHub {
    pub fn new() -> Self {
        Self::with_config(HubConfig::default())
    }
    
    pub fn with_config(config: HubConfig) -> Self {
        info!("🌌 Initializing Valhalla Hub (f64 GPU-Accelerated)");
        
        // Initialize Fate with resource limits
        let fate_limits = ResourceLimits {
            max_memory_mb: 4096,
            max_cpu_percent: config.resource_percent,
            max_buffer_size: 100000,
            processing_interval_ms: 10,
            gpu_enabled: true,
            batch_size: config.batch_size,
        };
        let fate_system = FateSystem::new("hub_fate".to_string(), fate_limits);
        
        // Initialize Quad systems
        let gravity_config = GravityFieldConfig::default();
        let gravity_field = GravityField::new(gravity_config.clone());
        let horizon_config = EventHorizonConfig::default();
        let event_horizon = EventHorizon::new(horizon_config, gravity_field.clone());
        
        Self {
            hub_id: Uuid::new_v4().to_string(),
            config,
            fate_system,
            triple_fate: TripleFateCombined::new(),
            gravity_field,
            event_horizon,
            meshbrain: MeshBrainSystem::new(MeshBrainConfig::default()),
            multinova: MultinovaProcessor::new(MultinovaConfig::default()),
            baseforce: BaseForceCoordinator::new(),
            system_status: SystemStatus {
                is_initialized: true,
                active_systems: vec!["fate".into(), "blackhole".into(), "meshbrain".into(), "multinova".into(), "baseforce".into()],
                system_health: HashMap::new(),
                last_update_tick: 0,
                fate_signal_tick: 0,
            },
            statistics: HubStatistics {
                total_requests: 0,
                successful_routes: 0,
                failed_routes: 0,
                system_usage: HashMap::new(),
                average_response_time_ms: 0.0,
                total_processing_time_ms: 0.0,
            },
            storage: None,
            all_storages: None,
            space_gpu: SpaceGPU::new(),
        }
    }
    
    /// 带存储路径初始化
    pub fn with_storage(base_path: &Path) -> std::io::Result<Self> {
        let mut instance = Self::new();
        instance.init_storage(base_path)?;
        Ok(instance)
    }
    
    /// 初始化所有系统存储
    pub fn init_storage(&mut self, base_path: &Path) -> std::io::Result<()> {
        self.storage = Some(SystemStorage::new(base_path, "hub")?);
        self.all_storages = Some(init_all_storages(base_path)?);
        info!("📁 Storage initialized at {:?}", base_path);
        Ok(())
    }
    
    /// 获取 Hub 存储
    pub fn storage(&self) -> Option<&SystemStorage> {
        self.storage.as_ref()
    }
    
    /// 获取指定系统的存储
    pub fn system_storage(&self, system: QuadSystemType) -> Option<&SystemStorage> {
        self.all_storages.as_ref().map(|s| match system {
            QuadSystemType::BlackHole => &s.blackhole,
            QuadSystemType::MeshBrain => &s.meshbrain,
            QuadSystemType::MultiNova => &s.multinova,
            QuadSystemType::BaseForce => &s.baseforce,
        })
    }
    
    /// 获取 StemCell 存储
    pub fn stemcell_storage(&self) -> Option<&SystemStorage> {
        self.all_storages.as_ref().map(|s| &s.stemcell)
    }
    
    /// 获取 Tile 存储
    pub fn tile_storage(&self) -> Option<&SystemStorage> {
        self.all_storages.as_ref().map(|s| &s.tile)
    }
    
    /// 获取 Chamber 存储
    pub fn chamber_storage(&self) -> Option<&SystemStorage> {
        self.all_storages.as_ref().map(|s| &s.chamber)
    }
    
    // ==================== 数据操作快捷方法 ====================
    
    /// 存储内容到 Hub
    pub fn store(&self, name: &str, content: &str) -> std::io::Result<()> {
        self.storage.as_ref()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Storage not initialized"))?
            .write(name, content)
    }
    
    /// 读取 Hub 内容
    pub fn load(&self, name: &str) -> std::io::Result<String> {
        self.storage.as_ref()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Storage not initialized"))?
            .read(name)
    }
    
    /// 删除 Hub 内容
    pub fn remove(&self, name: &str) -> std::io::Result<()> {
        self.storage.as_ref()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Storage not initialized"))?
            .delete(name)
    }
    
    /// 列出 Hub 所有内容
    pub fn list_stored(&self) -> std::io::Result<Vec<String>> {
        self.storage.as_ref()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Storage not initialized"))?
            .list_all()
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
    pub fn fate_load(&self, id: &str) -> Option<&FateDataEntry> {
        self.triple_fate.load_data(id)
    }
    
    /// 删除 Fate 存储的数据
    pub fn fate_remove(&mut self, id: &str) -> Option<FateDataEntry> {
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
    
    /// 保存系统切片 (.hub 二进制)
    pub fn save_slice(&self, path: &Path) -> std::io::Result<()> {
        let slice = HubSlice {
            hub_id: self.hub_id.clone(),
            statistics: self.statistics.clone(),
            fate_slice: self.triple_fate.freeze(),
        };
        let bytes = bincode::serialize(&slice)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, bytes)
    }
    
    /// 从切片恢复
    pub fn load_slice(path: &Path) -> std::io::Result<Self> {
        let bytes = std::fs::read(path)?;
        let slice: HubSlice = bincode::deserialize(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        
        let mut instance = Self::new();
        instance.hub_id = slice.hub_id;
        instance.statistics = slice.statistics;
        instance.triple_fate = TripleFateCombined::thaw(slice.fate_slice);
        Ok(instance)
    }
    
    // ═══════════════════════════════════════════════════════════════════════════
    // Space GPU 高维计算
    // ═══════════════════════════════════════════════════════════════════════════
    
    /// 计算维度 (GPU 加速)
    pub fn space_calculate_dimension(&mut self, data: &[f64]) -> DimensionResult {
        self.space_gpu.process_and_expand(data)
    }
    
    /// 批量计算维度 (GPU 加速)
    pub fn space_calculate_batch(&mut self, data_batch: &[Vec<f64>]) -> Vec<DimensionResult> {
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
    pub fn space_status(&self) -> SpaceStatus {
        self.space_gpu.get_status()
    }
    
    /// Process request with Fate routing (f64)
    pub fn process(&mut self, request: HubRequest) -> HubResponse {
        let start = Instant::now();
        self.statistics.total_requests += 1;
        
        // Route based on request type
        let target = self.route_request(&request);
        let target_name = format!("{:?}", target);
        
        // Process based on target
        let result = match target {
            QuadSystemType::BlackHole => {
                let gravity = self.gravity_field.calculate_gravity(request.intensity);
                format!("BlackHole: gravity={:.6}", gravity)
            },
            QuadSystemType::MeshBrain => {
                let signal = self.meshbrain.generate_signal(request.intensity, [0.0, 0.0, 0.0]);
                format!("MeshBrain: intelligence={:.6}", signal.intelligence_level)
            },
            QuadSystemType::MultiNova => {
                let event = self.multinova.process_evolution(request.intensity);
                format!("MultiNova: is_nova={}, radiation={:.6}", event.is_nova, event.radiation)
            },
            QuadSystemType::BaseForce => {
                let coord = self.baseforce.coordinate(request.intensity, request.intensity, request.intensity);
                format!("BaseForce: activity={:.6}", coord.total_activity)
            },
        };
        
        // Feedback to Fate
        self.feedback_to_fate(&target, request.intensity, true);
        
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        self.statistics.successful_routes += 1;
        self.statistics.total_processing_time_ms += elapsed;
        self.statistics.average_response_time_ms = 
            self.statistics.total_processing_time_ms / self.statistics.successful_routes as f64;
        *self.statistics.system_usage.entry(target_name).or_insert(0) += 1;
        
        self.system_status.fate_signal_tick = self.fate_system.get_logical_time();
        
        HubResponse {
            response_id: Uuid::new_v4().to_string(),
            success: true,
            message: result,
            data: None,
            processing_time_ms: elapsed,
        }
    }
    
    /// Feedback to Fate (f64)
    fn feedback_to_fate(&mut self, system: &QuadSystemType, intensity: f64, _success: bool) {
        let mut signal = DeformationSignal::new(
            (*system).into(),
            intensity,
            vec![0.0; 4],
        );
        signal.quality_score = 1.0;  // 有效信号
        self.fate_system.receive_signal(signal);
    }
    
    /// Route request using Fate preferences (f64)
    fn route_request(&mut self, _request: &HubRequest) -> QuadSystemType {
        // 定期记录快照以支持收敛检测
        if self.statistics.total_requests % 1000 == 0 {
            self.fate_system.record_preference_snapshot();
        }
        
        // 检查 Fate 收敛模式 - 如果还在探索中就轮询
        let pattern = self.fate_system.detect_convergence_pattern();
        
        // 只有当 Fate 检测到 "converging" 或 "fixed_point" 模式时，才使用偏好路由
        // 否则继续轮询探索
        let fate_ready = matches!(pattern, Some("converging") | Some("fixed_point"));
        
        if !fate_ready {
            // Fate 尚未形成稳定偏好，轮询分配让所有系统获得信号
            let idx = (self.statistics.total_requests as usize) % 4;
            return match idx {
                0 => QuadSystemType::BlackHole,
                1 => QuadSystemType::MeshBrain,
                2 => QuadSystemType::MultiNova,
                _ => QuadSystemType::BaseForce,
            };
        }
        
        // Fate 已形成稳定偏好，由 Fate 决定路由
        let prefs = self.fate_system.calculate_system_preferences();
        match prefs.preferred_system() {
            0 => QuadSystemType::BlackHole,
            1 => QuadSystemType::MeshBrain,
            2 => QuadSystemType::MultiNova,
            _ => QuadSystemType::BaseForce,
        }
    }
    
    /// Get Fate preferences
    pub fn get_fate_preferences(&self) -> SystemPreferences {
        self.fate_system.calculate_system_preferences()
    }
    
    /// Get system status
    pub fn get_status(&self) -> &SystemStatus {
        &self.system_status
    }
    
    /// Get statistics
    pub fn get_statistics(&self) -> &HubStatistics {
        &self.statistics
    }
    
    /// Get Fate system reference
    pub fn fate_system(&self) -> &FateSystem {
        &self.fate_system
    }
    
    /// 记录当前 Fate 偏好快照 (用于收敛检测)
    pub fn record_fate_snapshot(&mut self) {
        self.fate_system.record_preference_snapshot();
    }
    
    /// 标记一个 epoch (周期) 完成
    /// 只有调用此方法后，Fate 才能判断是否收敛
    /// 这确保了：数据必须处理完毕才能触发收敛
    pub fn mark_epoch_complete(&mut self) {
        self.fate_system.mark_epoch_complete();
    }
    
    /// 检查数据是否还在流入
    pub fn is_data_flowing(&self) -> bool {
        self.fate_system.is_data_flowing()
    }
    
    /// Fate 自主判断是否收敛 (代数形式边际效应)
    /// 前提：数据必须处理完毕 (调用过 mark_epoch_complete)
    pub fn has_fate_converged(&self) -> bool {
        self.fate_system.has_converged()
    }
    
    /// 获取 Fate 收敛模式
    pub fn get_fate_convergence_pattern(&self) -> Option<&'static str> {
        self.fate_system.detect_convergence_pattern()
    }
}

impl Default for ValhallaHub {
    fn default() -> Self {
        Self::new()
    }
}

/// Hub request (f64)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubRequest {
    pub request_id: String,
    pub request_type: RequestType,
    pub intensity: f64,
    pub data: Option<serde_json::Value>,
    pub priority: Priority,
}

/// Request type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestType {
    Reasoning,
    Learning,
    Adaptation,
    DataProcessing,
    SystemCoordination,
}

/// Priority
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

/// Hub response (f64)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubResponse {
    pub response_id: String,
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub processing_time_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hub_with_fate() {
        let mut hub = ValhallaHub::new();
        
        // First request is bootstrap
        let req1 = HubRequest {
            request_id: "test_0".into(),
            request_type: RequestType::Reasoning,
            intensity: 0.5,
            data: None,
            priority: Priority::Normal,
        };
        let resp1 = hub.process(req1);
        assert!(resp1.message.contains("Bootstrap"));
        
        // Second request is real
        let req2 = HubRequest {
            request_id: "test_1".into(),
            request_type: RequestType::Reasoning,
            intensity: 0.5,
            data: None,
            priority: Priority::Normal,
        };
        let resp2 = hub.process(req2);
        assert!(resp2.success);
    }
    
    #[test]
    fn test_fate_integration() {
        let mut hub = ValhallaHub::new();
        
        for i in 0..10 {
            let request = HubRequest {
                request_id: format!("test_{}", i),
                request_type: RequestType::DataProcessing,
                intensity: 0.5,
                data: None,
                priority: Priority::Normal,
            };
            hub.process(request);
        }
        
        let prefs = hub.get_fate_preferences();
        assert_eq!(prefs.affinities.len(), 4);
    }
}
