//! Built-in ocean (海流) on the **2D** slice of the fractional-dimension container.
//!
//! **Adaptive mask:** per-lattice-cell mixing rounds `R(x,y) ∈ [R_MIN, R_MAX]` derived from
//! fractional-dimension coupling (not a flat constant). Phase base seed is **precomputed once**
//! per buffer (`OceanMaskContext`).
//!
//! Stream byte `S = F(key, session, phase, x, y, frac)` — self-inverse XOR (involution).

use rayon::prelude::*;

use crate::frac_container::{FracDimContainer, YAKSHA_DIMENSION};

/// Minimum adaptive mixing rounds per stream byte (floor after lattice tag).
pub const OCEAN_MIX_ROUNDS_MIN: usize = 10;

/// Maximum adaptive mixing rounds per stream byte.
pub const OCEAN_MIX_ROUNDS_MAX: usize = 28;

/// Legacy name — average of adaptive range (docs / FFI compatibility note only).
pub const OCEAN_MIX_ROUNDS: usize = (OCEAN_MIX_ROUNDS_MIN + OCEAN_MIX_ROUNDS_MAX) / 2;

/// Ocean phase tags (global two-phase pipeline).
pub mod ocean_phase {
    pub const ONE: u8 = 1;
    pub const TWO: u8 = 2;
}

/// Precomputed phase state: hoists `ocean_seed(key, session, phase, container)` once.
#[derive(Debug, Clone, Copy)]
pub struct OceanMaskContext {
    pub base_seed: u64,
    pub phase: u8,
    pub container: FracDimContainer,
}

impl OceanMaskContext {
    /// Build context — call once per `(key, session, phase)` before masking a buffer.
    #[inline]
    pub fn new(
        key: &[u8],
        session: &[u8],
        phase: u8,
        container: FracDimContainer,
    ) -> Self {
        Self {
            base_seed: ocean_seed(key, session, phase, container),
            phase,
            container,
        }
    }

    /// One stream byte at linear index `i`.
    #[inline]
    pub fn stream_byte(&self, index: u64) -> u8 {
        let (x, y) = self.container.index_to_2d(index);
        self.stream_byte_2d(x, y)
    }

    /// One stream byte at 2D coordinate `(x, y)`.
    #[inline]
    pub fn stream_byte_2d(&self, x: u32, y: u32) -> u8 {
        ocean_stream_byte_from_base(self.base_seed, self.phase, x, y, self.container)
    }
}

/// FNV-1a 64-bit over `data`.
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

/// Fold key, session, phase, and container dimension into a 64-bit ocean seed.
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

/// Adaptive round count from 2D lattice + fractional coupling (deterministic).
#[inline]
fn adaptive_mix_rounds(base_seed: u64, x: u32, y: u32, container: FracDimContainer) -> usize {
    let coupling = container.frac_coupling(x, y);
    let frac_q = (container.frac * 256.0) as u64;
    let tag = base_seed
        ^ coupling.wrapping_mul(0xC2B2_AE3D_27D4_EB4F)
        ^ (x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (y as u64).wrapping_mul(0x85EB_CA6B)
        ^ frac_q.rotate_left(33);
    let span = OCEAN_MIX_ROUNDS_MAX - OCEAN_MIX_ROUNDS_MIN + 1;
    OCEAN_MIX_ROUNDS_MIN + (mix64(tag) as usize % span)
}

#[inline]
fn ocean_stream_byte_from_base(
    base_seed: u64,
    phase: u8,
    x: u32,
    y: u32,
    container: FracDimContainer,
) -> u8 {
    let mut s = base_seed;
    s ^= (x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    s ^= (y as u64).wrapping_mul(0x85EB_CA6B);
    s ^= container.frac_coupling(x, y).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    let linear = u64::from(x) | (u64::from(y) << 32);
    s ^= linear.wrapping_mul(0xD6E8_FEB8_6659_9349);
    s ^= (phase as u64).rotate_left(41);
    let rounds = adaptive_mix_rounds(base_seed, x, y, container);
    for _ in 0..rounds {
        s = mix64(s);
    }
    (s as u8) ^ ((s >> 8) as u8) ^ ((s >> 16) as u8) ^ ((s >> 24) as u8)
}

/// One ocean stream byte at **2D** lattice coordinate `(x, y)` inside `container`.
#[inline]
pub fn ocean_stream_byte_2d(
    key: &[u8],
    session: &[u8],
    phase: u8,
    x: u32,
    y: u32,
    container: FracDimContainer,
) -> u8 {
    let base = ocean_seed(key, session, phase, container);
    ocean_stream_byte_from_base(base, phase, x, y, container)
}

/// Stream byte at linear index `i` (maps through 2D container).
#[inline]
pub fn ocean_stream_byte(
    key: &[u8],
    session: &[u8],
    phase: u8,
    index: u64,
    container: FracDimContainer,
) -> u8 {
    OceanMaskContext::new(key, session, phase, container).stream_byte(index)
}

/// XOR each byte with the adaptive 2D ocean stream (encrypt and decrypt — same op).
pub fn ocean_apply_inplace(
    buf: &mut [u8],
    key: &[u8],
    session: &[u8],
    phase: u8,
    container: FracDimContainer,
) {
    let ctx = OceanMaskContext::new(key, session, phase, container);
    buf.par_iter_mut()
        .enumerate()
        .for_each(|(i, b)| *b ^= ctx.stream_byte(i as u64));
}

/// Standard dimension constant re-export for FFI/docs.
pub const OCEAN_CONTAINER_DIMENSION: f64 = YAKSHA_DIMENSION;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ocean_mask_involution() {
        let key = b"key";
        let session = b"session123456789";
        let container = FracDimContainer::standard();
        let mut buf: Vec<u8> = (0..512).map(|i| i as u8).collect();
        let orig = buf.clone();
        ocean_apply_inplace(&mut buf, key, session, ocean_phase::ONE, container);
        assert_ne!(buf, orig);
        ocean_apply_inplace(&mut buf, key, session, ocean_phase::ONE, container);
        assert_eq!(buf, orig);
    }

    #[test]
    fn context_matches_scalar_stream() {
        let key = b"k";
        let session = b"sess";
        let container = FracDimContainer::standard();
        let ctx = OceanMaskContext::new(key, session, ocean_phase::TWO, container);
        for i in 0..200u64 {
            assert_eq!(
                ctx.stream_byte(i),
                ocean_stream_byte(key, session, ocean_phase::TWO, i, container)
            );
        }
    }
}
