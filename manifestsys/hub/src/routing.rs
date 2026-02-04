//! 智能路由系统
//!
//! ⚠️ 重要：四元系统没有硬编码分工！
//! 路由完全由 Fate 动态决定，基于实际处理结果反馈

use crate::{HubRequest, HubError, RequestType};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingStrategy {
    /// 初始路由模式：轮询（无预设偏好）
    pub initial_mode: InitialRoutingMode,
    /// 轮询计数器
    pub round_robin_counter: usize,
}

/// 初始路由模式（Fate 置信度低时使用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InitialRoutingMode {
    /// 轮询：平等分配到四个系统（无偏好）
    RoundRobin,
    /// 随机：随机选择系统（无偏好）
    Random,
    /// 全部：同时发送到所有系统（测试用）
    Broadcast,
}

impl Default for RoutingStrategy {
    fn default() -> Self {
        Self {
            // ✅ 使用轮询模式，无任何预设偏好
            initial_mode: InitialRoutingMode::RoundRobin,
            round_robin_counter: 0,
        }
    }
}

impl RoutingStrategy {
    /// 无预设偏好的路由
    /// 当 Fate 置信度不足时，使用此方法平等分配请求
    pub fn route(&mut self, _request: &HubRequest) -> Result<RouteTarget, HubError> {
        match self.initial_mode {
            InitialRoutingMode::RoundRobin => {
                // ✅ 轮询：每个系统轮流处理，完全平等
                let systems = [
                    QuadSystemType::BlackHole,
                    QuadSystemType::MeshBrain,
                    QuadSystemType::MultiNova,
                    QuadSystemType::BaseForce,
                ];
                let selected = systems[self.round_robin_counter % 4];
                self.round_robin_counter += 1;
                Ok(RouteTarget::QuadSystem(selected))
            }
            InitialRoutingMode::Random => {
                // ✅ 随机：随机选择，完全平等
                use std::time::{SystemTime, UNIX_EPOCH};
                let seed = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .subsec_nanos() as usize;
                let systems = [
                    QuadSystemType::BlackHole,
                    QuadSystemType::MeshBrain,
                    QuadSystemType::MultiNova,
                    QuadSystemType::BaseForce,
                ];
                Ok(RouteTarget::QuadSystem(systems[seed % 4]))
            }
            InitialRoutingMode::Broadcast => {
                // 广播模式：默认先发到第一个系统（实际实现需要并行处理）
                Ok(RouteTarget::QuadSystem(QuadSystemType::BlackHole))
            }
        }
    }
    
    /// 重置计数器
    pub fn reset(&mut self) {
        self.round_robin_counter = 0;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RouteTarget {
    QuadSystem(QuadSystemType),
    Chamber,
    TileQuattro,
    StemCell,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum QuadSystemType {
    BlackHole,
    MeshBrain,
    MultiNova,
    BaseForce,
}
