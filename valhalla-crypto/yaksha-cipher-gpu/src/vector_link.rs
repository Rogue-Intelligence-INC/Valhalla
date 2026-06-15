//! 向量联结模块 (GPU 加速)
//!
//! 使用 GPU 计算块间向量关系，实现跨块扩散
//! XOR 操作是自逆的，所以 apply 和 apply_inverse 使用相同逻辑

use crate::GpuContext;
use wgpu::util::DeviceExt;
use bytemuck::{Pod, Zeroable};

/// 向量联结参数
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct VectorLinkParams {
    pub block_count: u32,
    pub block_size: u32,
    pub dimension: f32,  // 维度参数 (用于维度版) 或 α (用于量子版)
    pub _padding: u32,
}

/// 向量联结 Compute Shader (WGSL)
const VECTOR_LINK_SHADER: &str = r#"
struct Params {
    block_count: u32,
    block_size: u32,
    dimension: f32,
    _padding: u32,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read_write> data: array<u32>;
@group(0) @binding(2) var<storage, read_write> signatures: array<u32>;

// 提取块签名 (首 + 中 + 尾 的 XOR)
@compute @workgroup_size(1)
fn extract_signatures(@builtin(global_invocation_id) gid: vec3<u32>) {
    let block_idx = gid.x;
    if (block_idx >= params.block_count) { return; }
    
    let words_per_block = params.block_size / 4u;
    let start = block_idx * words_per_block;
    let mid = start + words_per_block / 2u;
    let end = start + words_per_block - 1u;
    
    // XOR 三个位置
    signatures[block_idx] = data[start] ^ data[mid] ^ data[end];
}

// 计算向量联结值并应用 (XOR 是自逆的)
@compute @workgroup_size(256)
fn apply_vector_link(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    let words_per_block = params.block_size / 4u;
    let total_u32 = params.block_count * words_per_block;
    if (idx >= total_u32) { return; }
    
    // 计算当前所在块
    let block_idx = idx / words_per_block;
    
    // 计算全局签名 (所有块签名 XOR)
    var global_sig: u32 = 0u;
    for (var i: u32 = 0u; i < params.block_count; i = i + 1u) {
        global_sig = global_sig ^ signatures[i];
    }
    
    // 计算向量联结值 (基于维度参数)
    let rot = u32(params.dimension * 4.0) % 8u;
    var link_val = global_sig;
    
    // 块索引影响旋转，创建位置相关的联结
    for (var i: u32 = 0u; i < params.block_count; i = i + 1u) {
        if (i != block_idx) {
            let delta = u32(abs(i32(block_idx) - i32(i)));
            let shift = (delta * rot) % 32u;
            let rotated = (signatures[i] << shift) | (signatures[i] >> (32u - shift));
            link_val = link_val ^ rotated;
        }
    }
    
    // 添加块内位置变化
    let pos_in_block = idx % words_per_block;
    let pos_mix = (link_val >> (pos_in_block % 24u)) ^ (link_val << ((32u - pos_in_block) % 24u));
    
    // 应用向量联结 (XOR 是自逆的)
    data[idx] = data[idx] ^ (link_val ^ pos_mix);
}
"#;

/// GPU 向量联结器
pub struct VectorLinker {
    extract_pipeline: wgpu::ComputePipeline,
    apply_pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl VectorLinker {
    pub fn new(ctx: &GpuContext) -> Self {
        let shader = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Vector Link Shader"),
            source: wgpu::ShaderSource::Wgsl(VECTOR_LINK_SHADER.into()),
        });

        let bind_group_layout = ctx.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Vector Link Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Vector Link Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let extract_pipeline = ctx.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Extract Signatures Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "extract_signatures",
        });

        let apply_pipeline = ctx.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Apply Vector Link Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "apply_vector_link",
        });

        Self {
            extract_pipeline,
            apply_pipeline,
            bind_group_layout,
        }
    }

    /// 应用向量联结 (加密方向)
    pub fn apply(&self, ctx: &GpuContext, data: &[u8], dimension: f32) -> Vec<u8> {
        self.link_internal(ctx, data, dimension)
    }

    /// 应用逆向量联结 (解密方向)
    /// XOR 是自逆操作，所以和 apply 使用相同逻辑
    pub fn apply_inverse(&self, ctx: &GpuContext, data: &[u8], dimension: f32) -> Vec<u8> {
        self.link_internal(ctx, data, dimension)
    }

    /// 内部实现
    fn link_internal(&self, ctx: &GpuContext, data: &[u8], dimension: f32) -> Vec<u8> {
        let len = data.len();
        
        // 数据太小，不处理
        if len < 256 {
            return data.to_vec();
        }

        // 确定块数量 (每块至少 64 字节)
        let block_count = (len / 64).min(16) as u32;
        if block_count < 2 {
            return data.to_vec();
        }

        // 对齐到块大小
        let block_size = ((len / block_count as usize) / 4) * 4;
        let aligned_len = block_size * block_count as usize;
        
        // Pad data to 4-byte boundary
        let mut data_padded = data[..aligned_len].to_vec();
        while data_padded.len() % 4 != 0 {
            data_padded.push(0);
        }

        let params = VectorLinkParams {
            block_count,
            block_size: block_size as u32,
            dimension,
            _padding: 0,
        };

        let params_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vector Params"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let data_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Data Buffer"),
            contents: &data_padded,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        });

        let signatures_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Signatures Buffer"),
            size: (block_count * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let staging_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Buffer"),
            size: data_padded.len() as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Vector Link Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: params_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: data_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: signatures_buffer.as_entire_binding() },
            ],
        });

        let mut encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Vector Link Encoder"),
        });

        // Step 1: 提取签名
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Extract Pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.extract_pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(block_count, 1, 1);
        }

        // Step 2: 应用向量联结
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Apply Pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.apply_pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            let workgroups = (data_padded.len() as u32 / 4 + 255) / 256;
            pass.dispatch_workgroups(workgroups.min(65535), 1, 1);
        }

        encoder.copy_buffer_to_buffer(&data_buffer, 0, &staging_buffer, 0, data_padded.len() as u64);
        ctx.queue.submit(Some(encoder.finish()));

        // 读回结果
        let buffer_slice = staging_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        ctx.device.poll(wgpu::Maintain::Wait);
        rx.recv().unwrap().unwrap();

        let mapped = buffer_slice.get_mapped_range();
        let mut result = mapped.to_vec();
        drop(mapped);
        staging_buffer.unmap();

        // 如果原始数据有未对齐部分，附加回去
        if len > aligned_len {
            result.extend_from_slice(&data[aligned_len..]);
        }

        result.truncate(len);
        result
    }
}
