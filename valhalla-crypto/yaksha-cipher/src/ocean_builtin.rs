//! Built-in 2D ocean stream — parallel mask build + configurable mixing rounds.

use rayon::prelude::*;

use crate::config;
use crate::frac_container::{FracDimContainer, YAKSHA_DIMENSION};

pub mod ocean_phase {
    pub const ONE: u8 = 1;
    pub const TWO: u8 = 2;
}

#[inline]
pub fn fnv1a64(data: &[u8]) -> u64 {
    const OFFSET: u64 = 14695981039346656037;
    const PRIME: u64 = 1099511628211;
    let mut h = OFFSET;
    for &b in data {
        h ^= b as u64;
        h = h.wrapping_mul(PRIME);
    }
    h
}

#[inline]
pub fn ocean_seed(key: &[u8], session: &[u8], phase: u8, container: FracDimContainer) -> u64 {
    let a = fnv1a64(key);
    let b = fnv1a64(session);
    let dim_bits = (container.dimension.to_bits()) ^ (container.frac.to_bits());
    a ^ b.rotate_left(17) ^ ((phase as u64) << 56) ^ dim_bits.rotate_left(31)
}

#[inline]
fn mix64(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}

#[inline]
pub fn ocean_stream_byte_2d(
    key: &[u8],
    session: &[u8],
    phase: u8,
    x: u32,
    y: u32,
    container: FracDimContainer,
    mix_rounds: usize,
) -> u8 {
    let mut s = ocean_seed(key, session, phase, container);
    s ^= (x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    s ^= (y as u64).wrapping_mul(0x85EB_CA6B);
    s ^= container.frac_coupling(x, y).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    let linear = u64::from(x) | (u64::from(y) << 32);
    s ^= linear.wrapping_mul(0xD6E8_FEB8_6659_9349);
    for _ in 0..mix_rounds {
        s = mix64(s);
    }
    // Fold all 8 bytes of the 64-bit mixed state (previously the high
    // 32 bits were silently discarded).
    (s as u8)
        ^ ((s >> 8) as u8)
        ^ ((s >> 16) as u8)
        ^ ((s >> 24) as u8)
        ^ ((s >> 32) as u8)
        ^ ((s >> 40) as u8)
        ^ ((s >> 48) as u8)
        ^ ((s >> 56) as u8)
}

#[inline]
pub fn ocean_stream_byte(
    key: &[u8],
    session: &[u8],
    phase: u8,
    index: u64,
    container: FracDimContainer,
) -> u8 {
    let mix_rounds = config::ocean_mix_rounds();
    let (x, y) = container.index_to_2d(index);
    ocean_stream_byte_2d(key, session, phase, x, y, container, mix_rounds)
}

/// Precompute full ocean XOR mask in parallel (batch path for CPU / GPU upload).
pub fn build_ocean_mask_parallel(
    key: &[u8],
    session: &[u8],
    phase: u8,
    len: usize,
    container: FracDimContainer,
) -> Vec<u8> {
    let mix_rounds = config::ocean_mix_rounds();
    let chunk = 4096usize;
    let n_chunks = len.div_ceil(chunk);
    let mut mask = vec![0u8; len];
    mask.par_chunks_mut(chunk)
        .enumerate()
        .for_each(|(ci, out_chunk)| {
            let base = ci * chunk;
            for (j, m) in out_chunk.iter_mut().enumerate() {
                let idx = (base + j) as u64;
                let (x, y) = container.index_to_2d(idx);
                *m = ocean_stream_byte_2d(key, session, phase, x, y, container, mix_rounds);
            }
        });
    let _ = n_chunks;
    mask
}

/// XOR with precomputed mask (vectorized apply).
#[inline]
pub fn ocean_apply_mask(buf: &mut [u8], mask: &[u8]) {
    debug_assert_eq!(buf.len(), mask.len());
    buf.par_iter_mut()
        .zip(mask.par_iter())
        .for_each(|(b, m)| *b ^= m);
}

/// Parallel ocean XOR: batch mask + apply.
pub fn ocean_apply_inplace(
    buf: &mut [u8],
    key: &[u8],
    session: &[u8],
    phase: u8,
    container: FracDimContainer,
) {
    let mask = build_ocean_mask_parallel(key, session, phase, buf.len(), container);
    ocean_apply_mask(buf, &mask);
}

pub const OCEAN_CONTAINER_DIMENSION: f64 = YAKSHA_DIMENSION;

/// Industry Gate audit rounds (documented constant).
pub const OCEAN_MIX_ROUNDS_AUDIT: usize = 48;
