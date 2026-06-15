//! 量子超级位置扩散
//!
//! 职责：建立随机不相邻的远距离连接
//! 类比：量子纠缠 - 相距很远的位置瞬间关联

use crate::GpuContext;
use wgpu::util::DeviceExt;
use bytemuck::{Pod, Zeroable};

/// 每个位置的随机链接数
const NUM_LINKS: u32 = 8;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Params {
    len: u32,
    seed: u32,
    round: u32,
    _pad: u32,
}

const SHADER: &str = r#"
struct Params {
    len: u32,
    seed: u32,
    round: u32,
    _pad: u32,
}

@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var<storage, read> src: array<u32>;
@group(0) @binding(2) var<storage, read_write> dst: array<u32>;

// 哈希函数：生成随机链接位置
fn hash_link(seed: u32, pos: u32, link_idx: u32, round: u32) -> u32 {
    var h = seed;
    h = h ^ (pos * 2654435761u);
    h = h * 16777619u;
    h = h ^ (link_idx * 2246822519u);
    h = h * 16777619u;
    h = h ^ (round * 3266489917u);  // 每轮链接位置变化
    h = h * 16777619u;
    return h;
}

// 生成远距离链接位置（确保不相邻）
fn get_remote_link(pos: u32, link_idx: u32, n: u32, seed: u32, round: u32) -> u32 {
    let h = hash_link(seed, pos, link_idx, round);
    
    // 最小距离：数据长度的 1/8，确保真正"远距离"
    let min_dist = max(n / 8u, 2u);
    
    // 在 [min_dist, n - min_dist] 范围内选择偏移
    let range = n - 2u * min_dist;
    var offset = min_dist + (h % max(range, 1u));
    
    // 随机决定正负方向
    if ((h >> 16u) & 1u) == 1u {
        return (pos + offset) % n;
    } else {
        return (pos + n - offset) % n;
    }
}

@compute @workgroup_size(256)
fn superposition(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    let n = (p.len + 3u) / 4u;
    if (idx >= n) { return; }
    
    var val = src[idx];
    
    // 与 NUM_LINKS 个远距离位置建立纠缠
    for (var k = 0u; k < 8u; k = k + 1u) {
        let remote_pos = get_remote_link(idx, k, n, p.seed, p.round);
        let remote_val = src[remote_pos];
        
        // 每个链接使用不同的旋转量（基于链接索引和轮次）
        let rot = ((k * 7u + p.round * 11u) % 31u) + 1u;
        let rotated = (remote_val << rot) | (remote_val >> (32u - rot));
        
        // XOR 混合
        val = val ^ rotated;
    }
    
    // 最终非线性混合
    val = val ^ ((val >> 17u) * 0x85EBCA6Bu);
    val = val ^ ((val >> 13u) * 0xC2B2AE35u);
    
    dst[idx] = val;
}
"#;

pub struct Superposition {
    pipeline: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,
}

impl Superposition {
    pub fn new(ctx: &GpuContext) -> Self {
        let shader = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Superposition"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });

        let layout = ctx.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
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
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
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

        let pipe_layout = ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });

        let pipeline = ctx.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Superposition"),
            layout: Some(&pipe_layout),
            module: &shader,
            entry_point: "superposition",
        });

        Self { pipeline, layout }
    }

    /// 应用超级位置扩散
    pub fn apply(&self, ctx: &GpuContext, data: &[u8], key: &[u8], rounds: u32) -> Vec<u8> {
        if data.len() < 16 {
            return data.to_vec();
        }

        let seed = key.iter().fold(2166136261u32, |h, &b| {
            h.wrapping_mul(16777619) ^ b as u32
        });

        let mut padded = data.to_vec();
        while padded.len() % 4 != 0 {
            padded.push(0);
        }

        let n = padded.len() / 4;
        let wg = ((n + 255) / 256).min(65535) as u32;

        // 双缓冲
        let buf_a = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: &padded,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
        });
        let buf_b = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: padded.len() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let staging = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: padded.len() as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut encoder = ctx.device.create_command_encoder(&Default::default());
        let mut src_a = true;

        for round in 0..rounds {
            let params = Params {
                len: data.len() as u32,
                seed,
                round,
                _pad: 0,
            };
            let params_buf = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::bytes_of(&params),
                usage: wgpu::BufferUsages::UNIFORM,
            });

            let (src_buf, dst_buf) = if src_a {
                (&buf_a, &buf_b)
            } else {
                (&buf_b, &buf_a)
            };

            let bg = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: params_buf.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: src_buf.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 2, resource: dst_buf.as_entire_binding() },
                ],
            });

            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bg, &[]);
            pass.dispatch_workgroups(wg, 1, 1);
            drop(pass);

            src_a = !src_a;
        }

        let result_buf = if src_a { &buf_a } else { &buf_b };
        encoder.copy_buffer_to_buffer(result_buf, 0, &staging, 0, padded.len() as u64);
        ctx.queue.submit(Some(encoder.finish()));

        let slice = staging.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            tx.send(r).unwrap();
        });
        ctx.device.poll(wgpu::Maintain::Wait);
        rx.recv().unwrap().unwrap();

        let mut result = slice.get_mapped_range().to_vec();
        staging.unmap();
        result.truncate(data.len());
        result
    }
}
