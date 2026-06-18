//! Lightweight keyed MAC for the fast throughput path.

use rayon::prelude::*;

#[inline]
fn fnv1a64(data: &[u8]) -> u64 {
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
fn mix64(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}

#[inline]
fn chunk_hash(key_h: u64, len_h: u64, bi: usize, chunk: &[u8]) -> u64 {
    let mut h = key_h ^ ((bi as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    h ^= len_h.rotate_left(17);
    for (j, &b) in chunk.iter().enumerate() {
        h ^= (b as u64).wrapping_mul(0x85EB_CA6B).rotate_left((j % 56) as u32);
        h = mix64(h);
    }
    h
}

/// Fast keyed MAC — parallel per-chunk hash, **deterministic** ordered fold.
pub struct YakshaMacFast;

impl YakshaMacFast {
    pub fn compute(key: &[u8], message: &[u8], tag_len: usize) -> Vec<u8> {
        let tag_len = tag_len.min(32);
        let key_h = fnv1a64(key);
        let len_h = message.len() as u64;

        let block_state: u64 = if message.is_empty() {
            mix64(key_h ^ len_h)
        } else {
            let n_chunks = message.len().div_ceil(64);
            let mut per_chunk: Vec<u64> = vec![0; n_chunks];
            per_chunk
                .par_iter_mut()
                .enumerate()
                .for_each(|(bi, slot)| {
                    let start = bi * 64;
                    let end = (start + 64).min(message.len());
                    *slot = chunk_hash(key_h, len_h, bi, &message[start..end]);
                });
            per_chunk
                .iter()
                .fold(0u64, |acc, &h| acc ^ mix64(acc.wrapping_add(h)))
        };

        let mut state = mix64(block_state ^ key_h ^ len_h);
        for round in 0..4u64 {
            state = mix64(state ^ round.wrapping_mul(0xD6E8_FEB8_6659_9349));
        }

        let mut tag = vec![0u8; tag_len];
        let bytes = state.to_le_bytes();
        for i in 0..tag_len {
            tag[i] = bytes[i % 8] ^ bytes[(i + 3) % 8] ^ ((state >> (i * 8)) as u8);
        }
        tag
    }

    pub fn verify(key: &[u8], message: &[u8], expected: &[u8]) -> bool {
        let computed = Self::compute(key, message, expected.len());
        crate::aead::constant_time_compare(&computed, expected)
    }
}
