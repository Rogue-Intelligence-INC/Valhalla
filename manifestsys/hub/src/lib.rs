//! 🌌 Valhalla Hub System - 集成所有系统的统一入口
//!
//! 核心设计:
//! - Fate 动态分流 + 循环反馈
//! - 四元系统完整集成（包含 BaseForce）
//! - 智能路由
//! - signal_tick 逻辑时间（非 SystemTime）
//! - 多源数据摄入（文本/代码/数学/图形）

pub mod routing;
pub mod data_ingestion;
pub mod fate_discovery;

pub use routing::*;
pub use data_ingestion::*;
pub use fate_discovery::*;

use std::collections::HashMap;
use thiserror::Error;
use serde::{Serialize, Deserialize};
use log::info;
use uuid::Uuid;

pub use valhalla_space::{VoyagerSpaceBuilder, SpaceError};
pub use valhalla_mathematics::PassiveMathematicalLibrary;
pub use fate::FateSystem;

#[derive(Error, Debug)]
pub enum HubError {
    #[error("Space容器错误: {0}")]
    SpaceError(#[from] SpaceError),
    #[error("路由错误: {0}")]
    RoutingError(String),
    #[error("系统集成错误: {0}")]
    IntegrationError(String),
}

/// 🌌 Valhalla Hub 系统
pub struct ValhallaHub {
    hub_id: String,
    space_builder: VoyagerSpaceBuilder,
    fate_system: FateSystem,
    mathematics_library: PassiveMathematicalLibrary,
    quad_systems: QuadSystems,
    routing_strategy: RoutingStrategy,
    system_status: SystemStatus,
    statistics: HubStatistics,
}

/// 四元系统集合
pub struct QuadSystems {
    pub blackhole_gravity: valhalla_metasystems_blackhole::GravityField,
    pub blackhole_horizon: valhalla_metasystems_blackhole::EventHorizon,
    pub meshbrain_config: valhalla_metasystems_meshbrain::MeshBrainConfig,
    pub multinova_processor: valhalla_metasystems_multinova::MultinovaProcessor,
    pub baseforce_blackhole: valhalla_metasystems_baseforce::BlackHoleSignalConnector,
    pub baseforce_meshbrain: valhalla_metasystems_baseforce::MeshBrainSignalConnector,
    pub baseforce_multinova: valhalla_metasystems_baseforce::MultiNovaSignalConnector,
}

impl std::fmt::Debug for QuadSystems {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QuadSystems")
            .field("blackhole", &"GravityField + EventHorizon")
            .field("meshbrain", &self.meshbrain_config)
            .field("multinova", &"MultinovaProcessor")
            .field("baseforce", &"SignalConnectors (3)")
            .finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub is_initialized: bool,
    pub active_systems: Vec<String>,
    pub system_health: HashMap<String, f64>,
    /// signal_tick 逻辑时间 (非 SystemTime)
    pub last_update: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubStatistics {
    pub total_requests: u64,
    pub successful_routes: u64,
    pub failed_routes: u64,
    pub system_usage: HashMap<String, u64>,
    pub average_response_time: f64,
}

impl ValhallaHub {
    pub fn new() -> Result<Self, HubError> {
        info!("🌌 初始化 Valhalla Hub 系统...");
        
        let space_builder = VoyagerSpaceBuilder::new().map_err(HubError::SpaceError)?;
        
        let resource_limits = fate::ResourceLimits {
            max_memory_mb: 16384,
            max_cpu_percent: 50,
            max_buffer_size: 1000,
            processing_interval_ms: 100,
        };
        
        let fate_system = FateSystem::new("valhalla_hub".to_string(), resource_limits);
        let mathematics_library = PassiveMathematicalLibrary::new();
        
        // 初始化四元系统
        let gravity_config = valhalla_metasystems_blackhole::GravityFieldConfig::default();
        let blackhole_gravity = valhalla_metasystems_blackhole::GravityField::new(gravity_config);
        
        let horizon_config = valhalla_metasystems_blackhole::EventHorizonConfig::default();
        let blackhole_horizon = valhalla_metasystems_blackhole::EventHorizon::new(
            horizon_config, blackhole_gravity.clone()
        );
        
        let meshbrain_config = valhalla_metasystems_meshbrain::MeshBrainConfig::default();
        let multinova_config = valhalla_metasystems_multinova::MultinovaConfig::default();
        let multinova_processor = valhalla_metasystems_multinova::MultinovaProcessor::new(multinova_config);
        
        let quad_systems = QuadSystems {
            blackhole_gravity,
            blackhole_horizon,
            meshbrain_config,
            multinova_processor,
            baseforce_blackhole: valhalla_metasystems_baseforce::BlackHoleSignalConnector::new(),
            baseforce_meshbrain: valhalla_metasystems_baseforce::MeshBrainSignalConnector::new(),
            baseforce_multinova: valhalla_metasystems_baseforce::MultiNovaSignalConnector::new(),
        };
        
        info!("✅ 四元系统初始化完成（包含 BaseForce）");
        
        Ok(Self {
            hub_id: Uuid::new_v4().to_string(),
            space_builder,
            fate_system,
            mathematics_library,
            quad_systems,
            routing_strategy: RoutingStrategy::default(),
            system_status: SystemStatus {
                is_initialized: true,
                active_systems: vec!["blackhole".into(), "meshbrain".into(), "multinova".into(), "baseforce".into()],
                system_health: HashMap::new(),
                last_update: 0, // signal_tick 逻辑时间
            },
            statistics: HubStatistics {
                total_requests: 0,
                successful_routes: 0,
                failed_routes: 0,
                system_usage: HashMap::new(),
                average_response_time: 0.0,
            },
        })
    }
    
    pub fn process(&mut self, request: HubRequest) -> Result<HubResponse, HubError> {
        self.statistics.total_requests += 1;
        
        // ═══════════════════════════════════════════════════════════════
        // 🔥 Fate 完全自主控制 - 无预设值
        //
        // 设计原则：
        // 1. 第一轮为启动轮，作废（不计入统计，不影响偏好）
        // 2. 四元系统物理模拟值初始为空，由 Fate 自主填写
        // 3. 质量分数由 Fate 自主赋予，基于历史学习
        // ═══════════════════════════════════════════════════════════════
        
        // 🔥 第一轮为启动轮，作废
        if self.statistics.total_requests == 1 {
            // 启动轮：仅触发 Fate 初始化，不计入任何统计
            let bootstrap_signal = fate::DeformationSignal {
                source_system: fate::SystemType::BlackHole, // 任意
                intensity: fate::Rational::zero(),          // 空
                direction: vec![fate::Rational::zero(); 4], // 空
                signal_tick: 0,
                quality_score: fate::Rational::zero(),      // 空
            };
            self.fate_system.receive_signal(bootstrap_signal);
            
            // 返回启动确认，不计入成功统计
            return Ok(HubResponse {
                response_id: format!("bootstrap_{}", request.request_id),
                success: true,
                message: "启动轮 - 初始化 Fate 系统".to_string(),
                data: Some(serde_json::json!({
                    "status": "bootstrap",
                    "signal_tick": self.fate_system.get_logical_time()
                })),
                processing_time_ms: 0.0,
            });
        }
        
        // 强制轮询的最小请求数（确保每个系统获得足够初始信号）
        const MIN_ROUNDROBIN_REQUESTS: u64 = 100;
        
        let preferences = self.fate_system.calculate_system_preferences();
        
        // 从第2轮开始正式处理
        let route = if self.statistics.total_requests <= MIN_ROUNDROBIN_REQUESTS + 1 {
            // 📍 阶段1：轮询建立平等基础（跳过第1轮启动轮）
            self.routing_strategy.route(&request)?
        } else if self.has_sufficient_confidence(&preferences) {
            // 📍 阶段2：Fate 已学习到足够信息，使用其偏好
            self.fate_based_routing(&preferences, &request)
        } else {
            // 📍 继续轮询直到 Fate 有足够置信度
            self.routing_strategy.route(&request)?
        };
        
        let route_name = format!("{:?}", route);
        let response = match route {
            RouteTarget::QuadSystem(system) => {
                let result = self.process_quad_system(system, &request)?;
                self.feedback_to_fate(&system, &result);
                result
            },
            _ => HubResponse::success("Processed".to_string()),
        };
        
        self.statistics.successful_routes += 1;
        *self.statistics.system_usage.entry(route_name).or_insert(0) += 1;
        
        Ok(response)
    }
    
    /// 检查 Fate 是否有足够置信度进行路由
    fn has_sufficient_confidence(&self, prefs: &fate::SystemPreferences) -> bool {
        if prefs.affinities.len() < 4 {
            return false;
        }
        
        // 计算最大亲和力和总和
        let mut max_aff = fate::Rational::zero();
        let mut sum = fate::Rational::zero();
        
        for aff in &prefs.affinities {
            sum = sum.add(aff);
            if aff.numerator * max_aff.denominator > max_aff.numerator * aff.denominator {
                max_aff = *aff;
            }
        }
        
        if sum.is_zero() {
            return false;
        }
        
        // 如果最大亲和力 > 平均值 × 1.2，则有足够置信度
        let avg = sum.div(&fate::Rational::new(4, 1));
        let threshold = avg.mul(&fate::Rational::new(12, 10)); // 1.2 倍
        
        max_aff.numerator * threshold.denominator > threshold.numerator * max_aff.denominator
    }
    
    fn fate_based_routing(&self, prefs: &fate::SystemPreferences, _request: &HubRequest) -> RouteTarget {
        // 使用 Rational 精确比较
        if prefs.affinities.len() < 4 {
            return RouteTarget::QuadSystem(QuadSystemType::BaseForce);
        }
        
        let mut max_idx = 0;
        let mut max_val = prefs.affinities[0];
        
        for (i, aff) in prefs.affinities.iter().enumerate().skip(1) {
            // Rational 比较: a/b > c/d ⟺ a×d > c×b
            if aff.numerator * max_val.denominator > max_val.numerator * aff.denominator {
                max_val = *aff;
                max_idx = i;
            }
        }
        
        let best = match max_idx {
            0 => QuadSystemType::BlackHole,
            1 => QuadSystemType::MeshBrain,
            2 => QuadSystemType::MultiNova,
            _ => QuadSystemType::BaseForce,
        };
        RouteTarget::QuadSystem(best)
    }
    
    fn feedback_to_fate(&mut self, system: &QuadSystemType, _response: &HubResponse) {
        // ═══════════════════════════════════════════════════════════════
        // 🔥 Fate 完全自主控制
        //
        // 设计原则：
        // 1. 质量分数 (quality_score) - 由 Fate 自主赋予，初始为空 (0)
        // 2. 强度 (intensity) - 由 Fate 自主赋予，初始为空 (0)
        // 3. 方向 (direction) - 仅标识来源系统，值由 Fate 学习填充
        // 4. 第一轮为启动轮，信号被标记但不参与偏好计算
        //
        // Fate 通过观察信号的自然涨落来学习，而非预设值
        // ═══════════════════════════════════════════════════════════════
        
        let tick = self.fate_system.get_logical_time();
        
        // 方向向量 - 仅标识来源，由 Fate 后续填充权重
        // 初始全为空 (0)，Fate 根据学习结果自主调整
        let direction = vec![
            fate::Rational::zero(),  // BlackHole 槽位
            fate::Rational::zero(),  // MeshBrain 槽位
            fate::Rational::zero(),  // MultiNova 槽位
            fate::Rational::zero(),  // BaseForce 槽位
        ];
        
        // 来源系统索引
        let system_index = match system {
            QuadSystemType::BlackHole => 0,
            QuadSystemType::MeshBrain => 1,
            QuadSystemType::MultiNova => 2,
            QuadSystemType::BaseForce => 3,
        };
        
        // ═══════════════════════════════════════════════════════════════
        // Fate 自主赋值逻辑
        //
        // 不预设 quality_score 和 intensity
        // 而是让 Fate 根据信号历史自主计算
        // ═══════════════════════════════════════════════════════════════
        
        // 获取 Fate 当前对该系统的评估 (从历史学习)
        let fate_evaluation = self.fate_system.evaluate_system(system_index);
        
        // quality_score: Fate 自主赋予，基于历史表现
        // 如果没有历史，返回空 (0)，Fate 会在后续学习中填充
        let quality_score = fate_evaluation.quality;
        
        // intensity: Fate 自主赋予，基于系统活跃度
        // 如果没有历史，返回空 (0)
        let intensity = fate_evaluation.intensity;
        
        let signal = fate::DeformationSignal {
            source_system: match system {
                QuadSystemType::BlackHole => fate::SystemType::BlackHole,
                QuadSystemType::MeshBrain => fate::SystemType::MeshBrain,
                QuadSystemType::MultiNova => fate::SystemType::MultiNova,
                QuadSystemType::BaseForce => fate::SystemType::BaseForce,
            },
            intensity,
            direction,
            signal_tick: tick,
            quality_score,
        };
        
        self.fate_system.receive_signal(signal);
        self.system_status.last_update = tick + 1;
    }
    
    fn process_quad_system(&mut self, system: QuadSystemType, request: &HubRequest) -> Result<HubResponse, HubError> {
        // ✅ 四元系统只负责处理，不评判质量
        // 质量分数由 Fate 统一计算
        Ok(HubResponse::success(format!("{:?}: {}", system, request.request_id)))
    }
    
    pub fn get_status(&self) -> &SystemStatus { &self.system_status }
    pub fn get_statistics(&self) -> &HubStatistics { &self.statistics }
    pub fn fate_system(&self) -> &FateSystem { &self.fate_system }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubRequest {
    pub request_id: String,
    pub request_type: RequestType,
    pub data: serde_json::Value,
    pub priority: Priority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestType { Reasoning, Learning, Adaptation, DataProcessing, SystemCoordination }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority { Low, Normal, High, Critical }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubResponse {
    pub response_id: String,
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub processing_time_ms: f64,
}

impl HubResponse {
    pub fn success(message: String) -> Self {
        Self {
            response_id: Uuid::new_v4().to_string(),
            success: true,
            message,
            data: None,
            processing_time_ms: 0.0,
        }
    }
}
