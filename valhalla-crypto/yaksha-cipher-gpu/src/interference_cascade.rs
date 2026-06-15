//! Quantum Interference Cascade: 模拟量子干涉扩散
//!
//! C 版本：多条路径的贡献相互叠加/抵消（干涉效应）
//!
//! ## 算法
//! ```text
//! 对于每个位置 i:
//!   paths[i] = [路径0, 路径1, ..., 路径N-1]
//!   每条路径有正负相位 (+1 或 -1)
//!   干涉结果 = Σ (phase[k] * path_value[k])
//!   相消干涉：某些路径抵消
//!   相长干涉：某些路径增强
//! ```

use crate::GpuContext;
use wgpu::util::DeviceExt;
use bytemuck::{Pod, Zeroable};

/// 干涉路径数量
const NUM_PATHS: u32 = 16;

/// 参数
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct InterferenceParams {
    data_len: u32,
    key_hash: u32,
    round: u32,
    _padding: u32,
}

/// Quantum Interference WGSL Shader
const INTERFERENCE_SHADER: &str = r#"
struct Params {
    data_len: u32,
    key_hash: u32,
    round: u32,
    _padding: u32,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> src: array<u32>;
@group(0) @binding(2) var<storage, read_write> dst: array<u32>;

// 量子哈希
fn quantum_hash(seed: u32, pos: u32, path_idx: u32) -> u32 {
    var h = seed ^ 2166136261u;
    h = h ^ pos;
    h = h * 16777619u;
    h = h ^ path_idx;
    h = h * 16777619u;
    h = h ^ (pos * 0x85EBCA6Bu);
    h = h * 16777619u;
    return h;
}

// 获取路径链接位置
fn get_path_link(pos: u32, path_idx: u32, len: u32, seed: u32) -> u32 {
    let h = quantum_hash(seed, pos, path_idx);
    var link = h % len;
    if (link == pos) {
        link = (link + 1u) % len;
    }
    return link;
}

// 获取路径相位 (+1 或 -1，用 bool 表示)
// true = 正相位（加法），false = 负相位（减法/XOR 不同模式）
fn get_phase(seed: u32, pos: u32, path_idx: u32) -> bool {
    let h = quantum_hash(seed ^ 0xCAFEBABEu, pos, path_idx);
    return (h & 1u) == 0u;
}

// 干涉混合函数
// 正相位：val = val XOR rotate(link_val, rot)
// 负相位：val = val XOR rotate(~link_val, rot)  (取反后 XOR，模拟相消)
fn interfere(val: u32, link_val: u32, phase: bool, rot: u32) -> u32 {
    let rotated_link = (link_val << rot) | (link_val >> (32u - rot));
    if (phase) {
        // 相长干涉：直接 XOR
        return val ^ rotated_link;
    } else {
        // 相消干涉：XOR 取反值（模拟抵消效应）
        return val ^ ~rotated_link;
    }
}

@compute @workgroup_size(256)
fn interference_forward(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    let word_count = (params.data_len + 3u) / 4u;
    if (idx >= word_count) { return; }
    
    var val = src[idx];
    let seed = params.key_hash ^ (params.round * 0x9E3779B9u);
    
    // 16 条干涉路径
    for (var p: u32 = 0u; p < 16u; p = p + 1u) {
        let link_pos = get_path_link(idx, p, word_count, seed);
        let link_val = src[link_pos];
        let phase = get_phase(seed, idx, p);
        let rot = (p * 5u + 3u) % 32u;
        
        val = interfere(val, link_val, phase, rot);
    }
    
    // 额外的全局干涉：与远端位置交互
    let far_pos = (idx + word_count / 2u) % word_count;
    let far_val = src[far_pos];
    val = val ^ ((far_val << 13u) | (far_val >> 19u));
    
    dst[idx] = val;
}

@compute @workgroup_size(256)
fn interference_backward(@builtin(global_invocation_id) gid: vec3<u32>) {
    let word_count = (params.data_len + 3u) / 4u;
    let idx = word_count - 1u - gid.x;
    if (gid.x >= word_count) { return; }
    
    var val = src[idx];
    let seed = params.key_hash ^ (params.round * 0x9E3779B9u) ^ 0xDEADBEEFu;
    
    // 16 条反向干涉路径（使用不同的 seed）
    for (var p: u32 = 0u; p < 16u; p = p + 1u) {
        let link_pos = get_path_link(idx, p + 16u, word_count, seed);
        let link_val = src[link_pos];
        let phase = get_phase(seed, idx, p + 16u);
        let rot = ((p + 16u) * 5u + 7u) % 32u;
        
        val = interfere(val, link_val, phase, rot);
    }
    
    // 反向全局干涉
    let far_pos = (idx + word_count / 3u) % word_count;
    let far_val = src[far_pos];
    val = val ^ ((far_val << 17u) | (far_val >> 15u));
    
    dst[idx] = val;
}

// 波函数扩散：模拟量子波函数的空间扩散
@compute @workgroup_size(256)
fn wave_spread(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    let word_count = (params.data_len + 3u) / 4u;
    if (idx >= word_count) { return; }
    
    let val = src[idx];
    
    // 波函数向邻近位置扩散
    let left = src[(idx + word_count - 1u) % word_count];
    let right = src[(idx + 1u) % word_count];
    let far_left = src[(idx + word_count - word_count/4u) % word_count];
    let far_right = src[(idx + word_count/4u) % word_count];
    
    // 干涉叠加
    var result = val;
    result = result ^ ((left << 3u) | (left >> 29u));
    result = result ^ ((right << 5u) | (right >> 27u));
    result = result ^ ((far_left << 11u) | (far_left >> 21u));
    result = result ^ ((far_right << 13u) | (far_right >> 19u));
    
    dst[idx] = result;
}
"#;

/// Quantum Interference Cascade
pub struct InterferenceCascade {
    forward_pipeline: wgpu::ComputePipeline,
    backward_pipeline: wgpu::ComputePipeline,
    wave_pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl InterferenceCascade {
    pub fn new(ctx: &GpuContext) -> Self {
        let shader = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Interference Cascade Shader"),
            source: wgpu::ShaderSource::Wgsl(INTERFERENCE_SHADER.into()),
        });

        let bind_group_layout = ctx.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Interference Layout"),
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
            label: Some("Interference Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let forward_pipeline = ctx.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Interference Forward"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "interference_forward",
        });

        let backward_pipeline = ctx.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Interference Backward"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "interference_backward",
        });

        let wave_pipeline = ctx.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Wave Spread"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "wave_spread",
        });

        Self {
            forward_pipeline,
            backward_pipeline,
            wave_pipeline,
            bind_group_layout,
        }
    }

    /// 应用干涉扩散（使用双缓冲）
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
            label: Some("Interference Encoder"),
        });

        let mut src_is_a = true;

        for round in 0..rounds {
            let params = InterferenceParams {
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

            // Forward pass
            {
                let (src_buf, dst_buf) = if src_is_a {
                    (&buffer_a, &buffer_b)
                } else {
                    (&buffer_b, &buffer_a)
                };

                let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Forward Bind"),
                    layout: &self.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry { binding: 0, resource: params_buffer.as_entire_binding() },
                        wgpu::BindGroupEntry { binding: 1, resource: src_buf.as_entire_binding() },
                        wgpu::BindGroupEntry { binding: 2, resource: dst_buf.as_entire_binding() },
                    ],
                });

                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Forward"),
                    timestamp_writes: None,
                });
                pass.set_pipeline(&self.forward_pipeline);
                pass.set_bind_group(0, &bind_group, &[]);
                pass.dispatch_workgroups(workgroups, 1, 1);
            }
            src_is_a = !src_is_a;

            // Wave spread pass
            {
                let (src_buf, dst_buf) = if src_is_a {
                    (&buffer_a, &buffer_b)
                } else {
                    (&buffer_b, &buffer_a)
                };

                let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Wave Bind"),
                    layout: &self.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry { binding: 0, resource: params_buffer.as_entire_binding() },
                        wgpu::BindGroupEntry { binding: 1, resource: src_buf.as_entire_binding() },
                        wgpu::BindGroupEntry { binding: 2, resource: dst_buf.as_entire_binding() },
                    ],
                });

                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Wave"),
                    timestamp_writes: None,
                });
                pass.set_pipeline(&self.wave_pipeline);
                pass.set_bind_group(0, &bind_group, &[]);
                pass.dispatch_workgroups(workgroups, 1, 1);
            }
            src_is_a = !src_is_a;

            // Backward pass
            {
                let (src_buf, dst_buf) = if src_is_a {
                    (&buffer_a, &buffer_b)
                } else {
                    (&buffer_b, &buffer_a)
                };

                let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Backward Bind"),
                    layout: &self.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry { binding: 0, resource: params_buffer.as_entire_binding() },
                        wgpu::BindGroupEntry { binding: 1, resource: src_buf.as_entire_binding() },
                        wgpu::BindGroupEntry { binding: 2, resource: dst_buf.as_entire_binding() },
                    ],
                });

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
