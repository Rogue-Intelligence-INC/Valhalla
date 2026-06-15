//! Quantum Superposition Cascade: 模拟量子叠加态扩散
//!
//! B 版本：每个位置同时处于多个"态"的叠加，最后坍缩为一个值
//!
//! ## 算法
//! ```text
//! 对于每个位置 i:
//!   states[i] = [态0, 态1, 态2, ..., 态N-1]  // N 个叠加态
//!   每个态基于不同的量子路径计算
//!   最终值 = collapse(states[i])  // 坍缩函数
//! ```

use crate::GpuContext;
use wgpu::util::DeviceExt;
use bytemuck::{Pod, Zeroable};

/// 叠加态数量
const NUM_STATES: u32 = 8;

/// 参数
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct SuperpositionParams {
    data_len: u32,
    key_hash: u32,
    round: u32,
    _padding: u32,
}

/// Quantum Superposition WGSL Shader
const SUPERPOSITION_SHADER: &str = r#"
struct Params {
    data_len: u32,
    key_hash: u32,
    round: u32,
    _padding: u32,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> src: array<u32>;
@group(0) @binding(2) var<storage, read_write> dst: array<u32>;

// 量子哈希 - 生成叠加态的基向量
fn quantum_hash(seed: u32, pos: u32, state_idx: u32) -> u32 {
    var h = seed ^ 2166136261u;
    h = h ^ pos;
    h = h * 16777619u;
    h = h ^ state_idx;
    h = h * 16777619u;
    h = h ^ (pos >> 16u);
    h = h * 16777619u;
    return h;
}

// 获取量子态的链接位置
fn get_quantum_link(pos: u32, state_idx: u32, len: u32, seed: u32) -> u32 {
    let h = quantum_hash(seed, pos, state_idx);
    var link = h % len;
    if (link == pos) {
        link = (link + 1u) % len;
    }
    return link;
}

// 量子相位旋转（模拟相位）
fn phase_rotate(val: u32, phase: u32) -> u32 {
    let rot = (phase * 7u + 3u) % 32u;
    return (val << rot) | (val >> (32u - rot));
}

// 叠加态坍缩函数：将多个态组合为最终值
fn collapse_states(s0: u32, s1: u32, s2: u32, s3: u32, s4: u32, s5: u32, s6: u32, s7: u32) -> u32 {
    // 量子干涉式坍缩：不同态之间相互影响
    var result = s0;
    
    // 相邻态 XOR（模拟干涉）
    result = result ^ phase_rotate(s1, 1u);
    result = result ^ phase_rotate(s2, 2u);
    result = result ^ phase_rotate(s3, 3u);
    result = result ^ phase_rotate(s4, 4u);
    result = result ^ phase_rotate(s5, 5u);
    result = result ^ phase_rotate(s6, 6u);
    result = result ^ phase_rotate(s7, 7u);
    
    // 非线性混合（增强扩散）
    let mix = (s0 & s1) ^ (s2 | s3) ^ (s4 & s5) ^ (s6 | s7);
    result = result ^ phase_rotate(mix, 11u);
    
    return result;
}

@compute @workgroup_size(256)
fn superposition_forward(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    let word_count = (params.data_len + 3u) / 4u;
    if (idx >= word_count) { return; }
    
    let original = src[idx];
    let seed = params.key_hash ^ (params.round * 0x9E3779B9u);
    
    // 计算 8 个叠加态
    var states: array<u32, 8>;
    for (var s: u32 = 0u; s < 8u; s = s + 1u) {
        let link_pos = get_quantum_link(idx, s, word_count, seed ^ s);
        let link_val = src[link_pos];
        
        // 每个态 = 原始值 XOR 链接值的相位旋转
        states[s] = original ^ phase_rotate(link_val, s);
    }
    
    // 坍缩所有叠加态
    let collapsed = collapse_states(
        states[0], states[1], states[2], states[3],
        states[4], states[5], states[6], states[7]
    );
    
    dst[idx] = collapsed;
}

@compute @workgroup_size(256)
fn superposition_backward(@builtin(global_invocation_id) gid: vec3<u32>) {
    let word_count = (params.data_len + 3u) / 4u;
    let idx = word_count - 1u - gid.x;
    if (gid.x >= word_count) { return; }
    
    let original = src[idx];
    // 使用不同的 seed 进行反向传播
    let seed = params.key_hash ^ (params.round * 0x9E3779B9u) ^ 0xDEADBEEFu;
    
    var states: array<u32, 8>;
    for (var s: u32 = 0u; s < 8u; s = s + 1u) {
        let link_pos = get_quantum_link(idx, s + 8u, word_count, seed ^ s);
        let link_val = src[link_pos];
        states[s] = original ^ phase_rotate(link_val, s + 8u);
    }
    
    let collapsed = collapse_states(
        states[0], states[1], states[2], states[3],
        states[4], states[5], states[6], states[7]
    );
    
    dst[idx] = collapsed;
}
"#;

/// Quantum Superposition Cascade
pub struct SuperpositionCascade {
    forward_pipeline: wgpu::ComputePipeline,
    backward_pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl SuperpositionCascade {
    pub fn new(ctx: &GpuContext) -> Self {
        let shader = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Superposition Cascade Shader"),
            source: wgpu::ShaderSource::Wgsl(SUPERPOSITION_SHADER.into()),
        });

        let bind_group_layout = ctx.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Superposition Layout"),
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

        let pipeline_layout = ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Superposition Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let forward_pipeline = ctx.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Superposition Forward"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "superposition_forward",
        });

        let backward_pipeline = ctx.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Superposition Backward"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "superposition_backward",
        });

        Self {
            forward_pipeline,
            backward_pipeline,
            bind_group_layout,
        }
    }

    /// 应用叠加态扩散（使用双缓冲确保正确性）
    pub fn apply(&self, ctx: &GpuContext, data: &[u8], key: &[u8], rounds: u32) -> Vec<u8> {
        if data.len() < 16 {
            return data.to_vec();
        }

        let key_hash = Self::compute_key_hash(key);
        
        let mut padded = data.to_vec();
        while padded.len() % 4 != 0 {
            padded.push(0);
        }

        let word_count = padded.len() / 4;
        let workgroups = ((word_count + 255) / 256).min(65535) as u32;

        // 双缓冲
        let buffer_a = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Buffer A"),
            contents: &padded,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
        });

        let buffer_b = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Buffer B"),
            size: padded.len() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let staging = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging"),
            size: padded.len() as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Superposition Encoder"),
        });

        let mut src_is_a = true;

        for round in 0..rounds {
            let params = SuperpositionParams {
                data_len: data.len() as u32,
                key_hash,
                round,
                _padding: 0,
            };

            let params_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Params"),
                contents: bytemuck::bytes_of(&params),
                usage: wgpu::BufferUsages::UNIFORM,
            });

            let (src_buf, dst_buf) = if src_is_a {
                (&buffer_a, &buffer_b)
            } else {
                (&buffer_b, &buffer_a)
            };

            let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Bind Group"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: params_buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: src_buf.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 2, resource: dst_buf.as_entire_binding() },
                ],
            });

            // Forward pass
            {
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Forward"),
                    timestamp_writes: None,
                });
                pass.set_pipeline(&self.forward_pipeline);
                pass.set_bind_group(0, &bind_group, &[]);
                pass.dispatch_workgroups(workgroups, 1, 1);
            }

            src_is_a = !src_is_a;

            // Backward pass (交换 buffer)
            let (src_buf, dst_buf) = if src_is_a {
                (&buffer_a, &buffer_b)
            } else {
                (&buffer_b, &buffer_a)
            };

            let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Bind Group Back"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: params_buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: src_buf.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 2, resource: dst_buf.as_entire_binding() },
                ],
            });

            {
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Backward"),
                    timestamp_writes: None,
                });
                pass.set_pipeline(&self.backward_pipeline);
                pass.set_bind_group(0, &bind_group, &[]);
                pass.dispatch_workgroups(workgroups, 1, 1);
            }

            src_is_a = !src_is_a;
        }

        // 结果在哪个 buffer
        let result_buf = if src_is_a { &buffer_a } else { &buffer_b };
        encoder.copy_buffer_to_buffer(result_buf, 0, &staging, 0, padded.len() as u64);
        ctx.queue.submit(Some(encoder.finish()));

        let buffer_slice = staging.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        ctx.device.poll(wgpu::Maintain::Wait);
        rx.recv().unwrap().unwrap();

        let mapped = buffer_slice.get_mapped_range();
        let mut result = mapped.to_vec();
        drop(mapped);
        staging.unmap();

        result.truncate(data.len());
        result
    }

    fn compute_key_hash(key: &[u8]) -> u32 {
        let mut h: u32 = 2166136261;
        for &b in key {
            h ^= b as u32;
            h = h.wrapping_mul(16777619);
        }
        h
    }
}
