//! 波粒转换：全局混淆 + 量子涨落时序递进
//!
//! 设计：
//! - 量子涨落：每个位置产生随机"涨落"
//! - 时序递进：涨落逐步向更远的邻居传播
//! - 波粒叠加：波（连续传播）+ 粒子（局部非线性）

use crate::GpuContext;
use wgpu::util::DeviceExt;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Params {
    len: u32,
    seed: u32,
    round: u32,
    step: u32,  // 时序步：决定传播距离
}

const SHADER: &str = r#"
struct Params {
    len: u32,
    seed: u32,
    round: u32,
    step: u32,
}

@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var<storage, read> src: array<u32>;
@group(0) @binding(2) var<storage, read_write> dst: array<u32>;

// 量子涨落生成器
fn fluctuation(seed: u32, pos: u32, step: u32) -> u32 {
    var h = seed ^ (pos * 2654435761u);
    h = h * 16777619u;
    h = h ^ (step * 2246822519u);
    h = h * 16777619u;
    h = h ^ ((pos + step) * 3266489917u);
    return h;
}

@compute @workgroup_size(256)
fn wave_particle(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    let n = (p.len + 3u) / 4u;
    if (idx >= n) { return; }
    
    var val = src[idx];
    
    // 量子涨落：当前位置的随机扰动
    let fluct = fluctuation(p.seed, idx, p.step);
    let fluct_rot = (fluct % 31u) + 1u;
    val = val ^ ((fluct << fluct_rot) | (fluct >> (32u - fluct_rot)));
    
    // 时序递进传播：距离 = step
    // 每个 step 传播到不同距离的邻居
    let dist = p.step + 1u;
    
    // 波传播：从距离 dist 的邻居收集信息
    let left_pos = (idx + n - dist) % n;
    let right_pos = (idx + dist) % n;
    let left_val = src[left_pos];
    let right_val = src[right_pos];
    
    // 相位调制：不同距离使用不同旋转
    let phase_l = ((dist * 7u + p.round * 3u) % 31u) + 1u;
    let phase_r = ((dist * 11u + p.round * 5u) % 31u) + 1u;
    
    // 波干涉
    let wave_l = (left_val << phase_l) | (left_val >> (32u - phase_l));
    let wave_r = (right_val << phase_r) | (right_val >> (32u - phase_r));
    val = val ^ wave_l ^ wave_r;
    
    // 粒子行为：非线性局部混合
    // 使用乘法混合增加非线性
    let particle = val ^ (val >> 17u);
    let mixed = particle * 0x85EBCA6Bu;
    val = val ^ (mixed >> 13u);
    
    // 远距离跳跃（模拟量子隧穿）
    // 跳跃距离基于涨落值，确保全局连接
    let jump_dist = (fluct % (n / 2u)) + 1u;
    let jump_pos = (idx + jump_dist) % n;
    let jump_val = src[jump_pos];
    let jump_rot = ((fluct >> 8u) % 31u) + 1u;
    val = val ^ ((jump_val << jump_rot) | (jump_val >> (32u - jump_rot)));
    
    dst[idx] = val;
}
"#;

pub struct WaveParticle {
    pipeline: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,
}

impl WaveParticle {
    pub fn new(ctx: &GpuContext) -> Self {
        let shader = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("WaveParticle"),
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
            label: Some("WaveParticle"),
            layout: Some(&pipe_layout),
            module: &shader,
            entry_point: "wave_particle",
        });

        Self { pipeline, layout }
    }

    /// 波粒转换：时序递进全局混淆
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

        // 时序步数：log2(n) 确保覆盖所有距离
        let max_steps = (32 - (n as u32).leading_zeros()).max(4);

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
            // 每轮执行多个时序步
            for step in 0..max_steps {
                let params = Params {
                    len: data.len() as u32,
                    seed: seed ^ (round * 0x9E3779B9),
                    round,
                    step,
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
