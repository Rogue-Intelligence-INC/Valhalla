//! Chamber - 高级容器编排系统
//!
//! 基于 Space 和 Fate 的智能容器编排
//!
//! 完整版本 (chamber_complete):
//! - TripleFate: 多层 Fate 监控
//! - VoyagerSpace: 自动维度扩展
//! - MicroContainer: 小容器 → 局部收敛 → Container
//! - 自动扩缩容

// 完整版本 (带 Triple-Fate + VoyagerSpace)
pub mod chamber_complete;
pub use chamber_complete::{
    ChamberComplete, ChamberConfig as ChamberCompleteConfig,
    MicroContainer as CompleteMicroContainer, LocalContainerState, CompletedContainer,
    ContainerType as CompleteContainerType, ScaleAction, ScaleEvent,
    ContainerFlowResult, ContainerBatchResult, ChamberStatus as CompleteStatus,
    ChamberError,
};

use serde::{Deserialize, Serialize};
use log::info;
pub use valhalla_space::{VoyagerSpaceBuilder, SpaceContainer, ContainerType, ContainerStatus, SpaceError};
pub use fate::{FateSystem, SystemPreferences};

/// Chamber 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChamberConfig {
    pub max_containers: usize,
    pub auto_scale_enabled: bool,
    pub scale_up_threshold: f64,
    pub scale_down_threshold: f64,
}

impl Default for ChamberConfig {
    fn default() -> Self {
        Self {
            max_containers: 100,
            auto_scale_enabled: true,
            scale_up_threshold: 0.8,
            scale_down_threshold: 0.2,
        }
    }
}

/// Chamber 编排系统
#[derive(Debug)]
pub struct Chamber {
    pub id: String,
    pub config: ChamberConfig,
    pub space_builder: VoyagerSpaceBuilder,
    pub container_ids: Vec<String>,
    pub total_operations: u64,
}

impl Chamber {
    pub fn new() -> Result<Self, SpaceError> {
        Self::with_config(ChamberConfig::default())
    }
    
    pub fn with_config(config: ChamberConfig) -> Result<Self, SpaceError> {
        info!("🏛️ 初始化 Chamber 编排系统");
        
        let space_builder = VoyagerSpaceBuilder::new()?;
        
        Ok(Self {
            id: uuid::Uuid::new_v4().to_string(),
            config,
            space_builder,
            container_ids: Vec::new(),
            total_operations: 0,
        })
    }
    
    /// 创建新容器
    pub fn create_container(&mut self, container_type: ContainerType) -> Result<String, SpaceError> {
        if self.container_ids.len() >= self.config.max_containers {
            return Err(SpaceError::ContainerCreationError(
                "达到最大容器数量限制".to_string()
            ));
        }
        
        let container_id = self.space_builder.create_container(container_type)?;
        self.container_ids.push(container_id.clone());
        self.total_operations += 1;
        
        info!("📦 Chamber 创建容器: {}", container_id);
        Ok(container_id)
    }
    
    /// 获取 Fate 偏好
    pub fn get_fate_preferences(&self) -> SystemPreferences {
        self.space_builder.fate_system().calculate_system_preferences()
    }
    
    /// 获取状态
    pub fn get_status(&self) -> ChamberStatus {
        ChamberStatus {
            chamber_id: self.id.clone(),
            container_count: self.container_ids.len(),
            total_operations: self.total_operations,
            auto_scale_enabled: self.config.auto_scale_enabled,
        }
    }
}

impl Default for Chamber {
    fn default() -> Self {
        Self::new().expect("Chamber initialization failed")
    }
}

/// Chamber 状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChamberStatus {
    pub chamber_id: String,
    pub container_count: usize,
    pub total_operations: u64,
    pub auto_scale_enabled: bool,
}
