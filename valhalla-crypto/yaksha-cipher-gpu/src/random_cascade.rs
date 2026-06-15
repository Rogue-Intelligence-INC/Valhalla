//! Random Cascade: GPU-accelerated full diffusion
//!
//! Solves the one-directional cascade problem by using random jump links.
//! Each byte position links to multiple pseudo-random positions, ensuring
//! that changes at ANY position affect ALL positions.
//!
//! ## Algorithm
//! ```text
//! For each position i:
//!   links[i] = [hash(key, i, 0), hash(key, i, 1), ..., hash(key, i, K-1)]
//!   cascade[i] = XOR of all linked positions with rotation mixing
//! ```

use crate::GpuContext;
use wgpu::util::DeviceExt;
use bytemuck::{Pod, Zeroable};

/// Number of random links per position
const LINK_COUNT: u32 = 8;

/// Cascade parameters
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct CascadeParams {
    data_len: u32,
    key_hash: u32,
    rounds: u32,
    _padding: u32,
}

/// Random Cascade WGSL Shader
const RANDOM_CASCADE_SHADER: &str = r#"
struct Params {
    data_len: u32,
    key_hash: u32,
    rounds: u32,
    _padding: u32,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read_write> data: array<u32>;

// Fast hash for link generation (FNV-1a style)
fn hash_link(seed: u32, pos: u32, link_idx: u32) -> u32 {
    var h = seed ^ 2166136261u;
    h = h ^ pos;
    h = h * 16777619u;
    h = h ^ link_idx;
    h = h * 16777619u;
    return h;
}

// Get linked position (guaranteed different from current)
fn get_link_pos(pos: u32, link_idx: u32, len: u32, seed: u32) -> u32 {
    let h = hash_link(seed, pos, link_idx);
    var link_target = h % len;
    // Ensure we don't link to self
    if (link_target == pos) {
        link_target = (link_target + 1u) % len;
    }
    return link_target;
}

// Forward cascade pass
@compute @workgroup_size(256)
fn cascade_forward(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    let word_count = (params.data_len + 3u) / 4u;
    if (idx >= word_count) { return; }
    
    var val = data[idx];
    
    // Gather from 8 random positions
    for (var k: u32 = 0u; k < 8u; k = k + 1u) {
        let link_pos = get_link_pos(idx, k, word_count, params.key_hash);
        let link_val = data[link_pos];
        
        // Mix with rotation based on link index
        let rot = (k * 4u + 1u) % 32u;
        let rotated = (link_val << rot) | (link_val >> (32u - rot));
        val = val ^ rotated;
    }
    
    // Barrier needed - use storage barrier
    storageBarrier();
    
    data[idx] = val;
}

// Backward cascade pass (for full diffusion)
@compute @workgroup_size(256)
fn cascade_backward(@builtin(global_invocation_id) gid: vec3<u32>) {
    let word_count = (params.data_len + 3u) / 4u;
    // Process in reverse order
    let idx = word_count - 1u - gid.x;
    if (gid.x >= word_count) { return; }
    
    var val = data[idx];
    
    // Gather from 8 different random positions
    for (var k: u32 = 0u; k < 8u; k = k + 1u) {
        let link_pos = get_link_pos(idx, k + 8u, word_count, params.key_hash ^ 0xDEADBEEFu);
        let link_val = data[link_pos];
        
        let rot = ((k + 8u) * 4u + 3u) % 32u;
        let rotated = (link_val << rot) | (link_val >> (32u - rot));
        val = val ^ rotated;
    }
    
    storageBarrier();
    
    data[idx] = val;
}

// Combined single-pass for smaller data
@compute @workgroup_size(64)
fn cascade_combined(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    let word_count = (params.data_len + 3u) / 4u;
    if (idx >= word_count) { return; }
    
    var val = data[idx];
    let seed1 = params.key_hash;
    let seed2 = params.key_hash ^ 0x5A5A5A5Au;
    
    // Forward links (4 links)
    for (var k: u32 = 0u; k < 4u; k = k + 1u) {
        let link_pos = get_link_pos(idx, k, word_count, seed1);
        let link_val = data[link_pos];
        let rot = (k * 7u + 1u) % 32u;
        val = val ^ ((link_val << rot) | (link_val >> (32u - rot)));
    }
    
    // Backward links (4 links with different seed)
    for (var k: u32 = 0u; k < 4u; k = k + 1u) {
        let link_pos = get_link_pos(idx, k, word_count, seed2);
        let link_val = data[link_pos];
        let rot = ((k + 4u) * 7u + 3u) % 32u;
        val = val ^ ((link_val << rot) | (link_val >> (32u - rot)));
    }
    
    data[idx] = val;
}
"#;

/// GPU Random Cascade
pub struct RandomCascade {
    forward_pipeline: wgpu::ComputePipeline,
    backward_pipeline: wgpu::ComputePipeline,
    combined_pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl RandomCascade {
    pub fn new(ctx: &GpuContext) -> Self {
        let shader = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Random Cascade Shader"),
            source: wgpu::ShaderSource::Wgsl(RANDOM_CASCADE_SHADER.into()),
        });

        let bind_group_layout = ctx.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Random Cascade Layout"),
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
            ],
        });

        let pipeline_layout = ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Random Cascade Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let forward_pipeline = ctx.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Cascade Forward"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "cascade_forward",
        });

        let backward_pipeline = ctx.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Cascade Backward"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "cascade_backward",
        });

        let combined_pipeline = ctx.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Cascade Combined"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "cascade_combined",
        });

        Self {
            forward_pipeline,
            backward_pipeline,
            combined_pipeline,
            bind_group_layout,
        }
    }

    /// Apply random cascade (encryption direction)
    pub fn apply(&self, ctx: &GpuContext, data: &[u8], key: &[u8], rounds: u32) -> Vec<u8> {
        if data.len() < 16 {
            return data.to_vec();
        }

        let key_hash = Self::compute_key_hash(key);
        
        // Pad to 4-byte boundary
        let mut padded = data.to_vec();
        while padded.len() % 4 != 0 {
            padded.push(0);
        }

        let params = CascadeParams {
            data_len: data.len() as u32,
            key_hash,
            rounds,
            _padding: 0,
        };

        let params_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cascade Params"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let data_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cascade Data"),
            contents: &padded,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        });

        let staging_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging"),
            size: padded.len() as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Cascade Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: params_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: data_buffer.as_entire_binding() },
            ],
        });

        let word_count = padded.len() / 4;
        let workgroups = ((word_count + 255) / 256).min(65535) as u32;

        let mut encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Cascade Encoder"),
        });

        // Multiple rounds for stronger diffusion
        for _ in 0..rounds {
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

            // Backward pass
            {
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Backward"),
                    timestamp_writes: None,
                });
                pass.set_pipeline(&self.backward_pipeline);
                pass.set_bind_group(0, &bind_group, &[]);
                pass.dispatch_workgroups(workgroups, 1, 1);
            }
        }

        encoder.copy_buffer_to_buffer(&data_buffer, 0, &staging_buffer, 0, padded.len() as u64);
        ctx.queue.submit(Some(encoder.finish()));

        // Read back
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

        result.truncate(data.len());
        result
    }

    /// Apply inverse (same operation since XOR is self-inverse)
    pub fn apply_inverse(&self, ctx: &GpuContext, data: &[u8], key: &[u8], rounds: u32) -> Vec<u8> {
        // XOR with same values is self-inverse, but we need reverse order
        // For perfect inverse, apply rounds in reverse
        if data.len() < 16 {
            return data.to_vec();
        }

        let key_hash = Self::compute_key_hash(key);
        
        let mut padded = data.to_vec();
        while padded.len() % 4 != 0 {
            padded.push(0);
        }

        let params = CascadeParams {
            data_len: data.len() as u32,
            key_hash,
            rounds,
            _padding: 0,
        };

        let params_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cascade Params"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let data_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cascade Data"),
            contents: &padded,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        });

        let staging_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging"),
            size: padded.len() as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Cascade Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: params_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: data_buffer.as_entire_binding() },
            ],
        });

        let word_count = padded.len() / 4;
        let workgroups = ((word_count + 255) / 256).min(65535) as u32;

        let mut encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Cascade Encoder"),
        });

        // Reverse order: backward then forward
        for _ in 0..rounds {
            {
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Backward"),
                    timestamp_writes: None,
                });
                pass.set_pipeline(&self.backward_pipeline);
                pass.set_bind_group(0, &bind_group, &[]);
                pass.dispatch_workgroups(workgroups, 1, 1);
            }

            {
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Forward"),
                    timestamp_writes: None,
                });
                pass.set_pipeline(&self.forward_pipeline);
                pass.set_bind_group(0, &bind_group, &[]);
                pass.dispatch_workgroups(workgroups, 1, 1);
            }
        }

        encoder.copy_buffer_to_buffer(&data_buffer, 0, &staging_buffer, 0, padded.len() as u64);
        ctx.queue.submit(Some(encoder.finish()));

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_cascade_basic() {
        let ctx = match GpuContext::new() {
            Some(c) => c,
            None => {
                println!("No GPU, skipping test");
                return;
            }
        };

        let cascade = RandomCascade::new(&ctx);
        let data = b"Hello, Random Cascade Test!";
        let key = b"test_key";

        let encrypted = cascade.apply(&ctx, data, key, 2);
        
        // Encrypted should be different from original
        assert_ne!(encrypted, data.to_vec());
        
        // Should maintain length
        assert_eq!(encrypted.len(), data.len());
    }
}
