//! Random Cascade: Feistel diffusion in **64-byte blocks**
//!
//! - **Phase 1 (wave–particle, elsewhere)**: `wave_block_phase1_*` — within each 64B chunk,
//!   a Feistel network couples all bytes (simplified global mutual influence).
//! - **Phase 2 (superposition)**: `cascade_encrypt` / `cascade_decrypt` — same block size,
//!   different domain tag and round count so layers do not commute trivially.
//!
//! Tail chunks shorter than 64 bytes are still processed as one Feistel unit (or XOR if `< 4`).
//!
//! **Note:** `round_function` is **sequential**. Feistel already runs per 64B chunk; nested
//! `rayon` here caused millions of tiny parallel jobs on large buffers (looks like a hang).

use std::cmp::min;

/// Block size for wave / superposition simulation (64 bytes).
pub const WAVE_BLOCK: usize = 64;

/// Feistel rounds inside each 64B chunk for phase-1 (wave–particle coupling).
pub const PHASE1_FEISTEL_ROUNDS: usize = 6;

/// Feistel rounds per chunk for phase-2 (superposition scale).
pub const FEISTEL_ROUNDS: usize = 8;

/// Domain separation: phase-1 vs phase-2 round functions must differ.
const DOMAIN_WAVE_PARTICLE: u64 = 0xB10C_BADE_D15C_0001;
const DOMAIN_SUPERPOSITION: u64 = 0x50F4_7C00_5A10_0002;

/// Number of Feistel rounds (alias for phase-2; kept for older docs).
pub const FEISTEL_ROUNDS_PHASE2: usize = FEISTEL_ROUNDS;

#[inline]
pub fn key_hash(key: &[u8]) -> u64 {
    let mut h: u64 = 14695981039346656037;
    for &b in key {
        h ^= b as u64;
        h = h.wrapping_mul(1099511628211);
    }
    h
}

fn round_function(data: &[u8], key_h: u64, round: usize, domain_tag: u64) -> Vec<u8> {
    let len = data.len();
    let mut out = Vec::with_capacity(len);
    for i in 0..len {
        let mut h = key_h ^ domain_tag;
        h ^= round as u64;
        h = h.wrapping_mul(1099511628211);
        h ^= i as u64;
        h = h.wrapping_mul(1099511628211);

        let d0 = data[i] as u64;
        let d1 = data[(i + 1) % len] as u64;
        let d2 = data[(i + len - 1) % len] as u64;
        let d_far = data[(i + len / 2) % len] as u64;

        h ^= d0.wrapping_mul(31);
        h ^= d1.wrapping_mul(37).rotate_left(8);
        h ^= d2.wrapping_mul(41).rotate_left(16);
        h ^= d_far.wrapping_mul(43).rotate_left(24);
        h = h.wrapping_mul(1099511628211);

        out.push(h as u8);
    }
    out
}

/// Single contiguous buffer Feistel (variable length ≥ 4).
fn feistel_encrypt_chunk(data: &mut [u8], key: &[u8], rounds: usize, domain_tag: u64) {
    let len = data.len();
    if len < 4 {
        let key_h = key_hash(key) ^ domain_tag;
        for i in 0..len {
            data[i] ^= (key_h >> (i * 8 % 64)) as u8;
        }
        return;
    }

    let key_h = key_hash(key);
    let mid = len / 2;

    for round in 0..rounds {
        let (left, right) = data.split_at_mut(mid);

        let f_r = round_function(right, key_h, round * 2, domain_tag);
        for i in 0..left.len().min(f_r.len()) {
            left[i] ^= f_r[i];
        }

        let f_l = round_function(left, key_h, round * 2 + 1, domain_tag);
        for i in 0..right.len().min(f_l.len()) {
            right[i] ^= f_l[i];
        }
    }
}

fn feistel_decrypt_chunk(data: &mut [u8], key: &[u8], rounds: usize, domain_tag: u64) {
    let len = data.len();
    if len < 4 {
        let key_h = key_hash(key) ^ domain_tag;
        for i in 0..len {
            data[i] ^= (key_h >> (i * 8 % 64)) as u8;
        }
        return;
    }

    let key_h = key_hash(key);
    let mid = len / 2;

    for round in (0..rounds).rev() {
        let (left, right) = data.split_at_mut(mid);

        let f_l = round_function(left, key_h, round * 2 + 1, domain_tag);
        for i in 0..right.len().min(f_l.len()) {
            right[i] ^= f_l[i];
        }

        let f_r = round_function(right, key_h, round * 2, domain_tag);
        for i in 0..left.len().min(f_r.len()) {
            left[i] ^= f_r[i];
        }
    }
}

fn for_each_wave_chunk<F>(data: &mut [u8], mut f: F)
where
    F: FnMut(&mut [u8]),
{
    let len = data.len();
    let mut i = 0;
    while i < len {
        let end = min(i + WAVE_BLOCK, len);
        f(&mut data[i..end]);
        i = end;
    }
}

/// Phase 1: per **64-byte** (or tail) chunk — global coupling inside chunk (wave–particle).
pub fn wave_block_phase1_encrypt(data: &mut [u8], key: &[u8]) {
    for_each_wave_chunk(data, |chunk| {
        feistel_encrypt_chunk(chunk, key, PHASE1_FEISTEL_ROUNDS, DOMAIN_WAVE_PARTICLE);
    });
}

/// Inverse of [`wave_block_phase1_encrypt`].
pub fn wave_block_phase1_decrypt(data: &mut [u8], key: &[u8]) {
    for_each_wave_chunk(data, |chunk| {
        feistel_decrypt_chunk(chunk, key, PHASE1_FEISTEL_ROUNDS, DOMAIN_WAVE_PARTICLE);
    });
}

/// Phase 2: superposition — same **64-byte** alignment, distinct domain tag + rounds.
pub fn cascade_encrypt(data: &mut [u8], key: &[u8]) {
    for_each_wave_chunk(data, |chunk| {
        feistel_encrypt_chunk(chunk, key, FEISTEL_ROUNDS, DOMAIN_SUPERPOSITION);
    });
}

/// Inverse of [`cascade_encrypt`].
pub fn cascade_decrypt(data: &mut [u8], key: &[u8]) {
    for_each_wave_chunk(data, |chunk| {
        feistel_decrypt_chunk(chunk, key, FEISTEL_ROUNDS, DOMAIN_SUPERPOSITION);
    });
}
