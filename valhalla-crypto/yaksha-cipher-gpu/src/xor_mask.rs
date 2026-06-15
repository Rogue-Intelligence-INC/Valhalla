//! GPU helper: apply XOR mask byte-wise.
//!
//! Constraint: this kernel only performs XOR/bit ops on GPU.
//! Any PRG generation (which may contain additions) must remain CPU-side.

use crate::GpuContext;
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Params {
    // Number of u32 words (padded to 4-byte boundary on the caller side).
    n_words: u32,
}

const SHADER: &str = r#"
struct Params {
    n_words: u32,
}

@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var<storage, read> src: array<u32>;
@group(0) @binding(2) var<storage, read> mask: array<u32>;
@group(0) @binding(3) var<storage, read_write> dst: array<u32>;

@compute @workgroup_size(256)
fn xor_mask(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    if (idx >= p.n_words) { return; }
    dst[idx] = src[idx] ^ mask[idx];
}
"#;

/// Apply `data[i] ^= mask[i]` on GPU.
///
/// Returns `None` if input lengths mismatch.
pub fn apply_xor_mask(
    ctx: &GpuContext,
    data: &[u8],
    mask: &[u8],
) -> Option<Vec<u8>> {
    if data.len() != mask.len() {
        return None;
    }
    if data.is_empty() {
        return Some(vec![]);
    }

    let len_bytes = data.len();
    let padded_len = (len_bytes + 3) & !3;
    let n_words = padded_len / 4;

    // Pad with zeros so that truncation after GPU readback is safe.
    let mut data_pad = Vec::with_capacity(padded_len);
    data_pad.extend_from_slice(data);
    data_pad.resize(padded_len, 0);

    let mut mask_pad = Vec::with_capacity(padded_len);
    mask_pad.extend_from_slice(mask);
    mask_pad.resize(padded_len, 0);

    // Little-endian packing to u32 words.
    let mut src_words = vec![0u32; n_words];
    let mut mask_words = vec![0u32; n_words];
    for i in 0..n_words {
        let j = i * 4;
        src_words[i] = u32::from_le_bytes([data_pad[j], data_pad[j + 1], data_pad[j + 2], data_pad[j + 3]]);
        mask_words[i] = u32::from_le_bytes([mask_pad[j], mask_pad[j + 1], mask_pad[j + 2], mask_pad[j + 3]]);
    }

    let params = Params { n_words: n_words as u32 };

    let shader = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("xor_mask shader"),
        source: wgpu::ShaderSource::Wgsl(SHADER.into()),
    });

    let bind_group_layout = ctx.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("xor_mask layout"),
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
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
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
        label: Some("xor_mask pipeline_layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = ctx.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("xor_mask pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: "xor_mask",
    });

    let params_buf = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("xor_mask params"),
        contents: bytemuck::bytes_of(&params),
        usage: wgpu::BufferUsages::UNIFORM,
    });

    let src_buf = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("xor_mask src"),
        contents: bytemuck::cast_slice(&src_words),
        usage: wgpu::BufferUsages::STORAGE,
    });

    let mask_buf = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("xor_mask mask"),
        contents: bytemuck::cast_slice(&mask_words),
        usage: wgpu::BufferUsages::STORAGE,
    });

    let dst_buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("xor_mask dst"),
        size: (n_words as u64) * 4,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let staging = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("xor_mask staging"),
        size: (n_words as u64) * 4,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("xor_mask bind_group"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: params_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: src_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: mask_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: dst_buf.as_entire_binding(),
            },
        ],
    });

    let mut encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("xor_mask encoder"),
    });

    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("xor_mask pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        let wg: u32 = (((n_words + 255) / 256).min(65535)) as u32;
        pass.dispatch_workgroups(wg, 1, 1);
    }

    encoder.copy_buffer_to_buffer(&dst_buf, 0, &staging, 0, (n_words as u64) * 4);
    ctx.queue.submit(Some(encoder.finish()));

    let staging_slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    staging_slice.map_async(wgpu::MapMode::Read, move |result| {
        tx.send(result).unwrap();
    });
    ctx.device.poll(wgpu::Maintain::Wait);
    rx.recv().unwrap().unwrap();

    let mapped = staging_slice.get_mapped_range();
    let words: &[u32] = bytemuck::cast_slice(&mapped);
    let mut out_pad = vec![0u8; padded_len];
    for i in 0..n_words {
        let v = words[i].to_le_bytes();
        let j = i * 4;
        out_pad[j] = v[0];
        out_pad[j + 1] = v[1];
        out_pad[j + 2] = v[2];
        out_pad[j + 3] = v[3];
    }

    drop(mapped);
    staging.unmap();
    out_pad.truncate(len_bytes);
    Some(out_pad)
}

