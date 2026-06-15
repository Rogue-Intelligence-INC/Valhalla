//! GPU XOR offload for 2D ocean masks (mask built on CPU, XOR on GPU).

use yaksha_cipher::frac_container::FracDimContainer;
use yaksha_cipher::ocean_builtin::OceanMaskContext;

use crate::{apply_xor_mask, GpuContext};

/// Build adaptive ocean XOR mask (phase seed precomputed once).
pub fn build_ocean_mask(
    key: &[u8],
    session: &[u8],
    phase: u8,
    len: usize,
    container: FracDimContainer,
) -> Vec<u8> {
    let ctx = OceanMaskContext::new(key, session, phase, container);
    let mut mask = vec![0u8; len];
    for (i, b) in mask.iter_mut().enumerate() {
        *b = ctx.stream_byte(i as u64);
    }
    mask
}

/// Apply one ocean phase: CPU mask + GPU XOR.
pub fn apply_ocean_phase_gpu(
    ctx: &GpuContext,
    buf: &[u8],
    key: &[u8],
    session: &[u8],
    phase: u8,
    container: FracDimContainer,
) -> Vec<u8> {
    let mask = build_ocean_mask(key, session, phase, buf.len(), container);
    apply_xor_mask(ctx, buf, &mask).expect("xor mask: len match")
}
