//! GPU 上下文管理

use wgpu::{Device, Queue, Instance};
use std::sync::Arc;

/// GPU 计算上下文
pub struct GpuContext {
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub adapter_info: wgpu::AdapterInfo,
}

impl GpuContext {
    /// 创建新的 GPU 上下文
    pub fn new() -> Option<Self> {
        pollster::block_on(Self::new_async())
    }

    async fn new_async() -> Option<Self> {
        let instance = Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        }).await?;

        let adapter_info = adapter.get_info();
        
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Val-Crypt GPU Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
            },
            None,
        ).await.ok()?;

        Some(Self {
            device: Arc::new(device),
            queue: Arc::new(queue),
            adapter_info,
        })
    }

    /// 获取 GPU 信息
    pub fn info(&self) -> String {
        format!(
            "{} ({:?})",
            self.adapter_info.name,
            self.adapter_info.backend
        )
    }
}

impl Default for GpuContext {
    fn default() -> Self {
        Self::new().expect("Failed to create GPU context")
    }
}
