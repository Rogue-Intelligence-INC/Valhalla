//! # Yaksha Core: Basic Encryption Primitive
//!
//! A deterministic symmetric cipher using position-dependent entropy trajectory.
//!
//! ## Properties
//! - **Deterministic**: Same (key, plaintext) always produces same ciphertext
//! - **Reversible**: 100% correct decryption
//! - **Statistical tests**: see crate `crypto_stat_tests` (NIST monobit, key-bit SAC).
//! - **Throughput**: ~800-1000 MB/s (CPU, rayon parallel)
//!
//! ## Algorithm
//!
//! ```text
//! Phase 1: Hybrid 64-bit block + tail bytes
//!   - Full 8-byte blocks: single u64 ARX path (entropy branch: odd multiply or add, mod 2^64)
//!   - Remaining bytes (< 8): legacy per-byte parallel path (same formula as before)
//!
//! Phase 1b: **64-byte** wave–particle Feistel (`cascade::wave_block_phase1_*`)
//! Phase 2: **64-byte** superposition Feistel (`cascade::cascade_*`)
//! ```
//!
//! ## Security Note
//!
//! This is a **deterministic primitive**. For semantic security (IND-CPA),
//! use with stateful key derivation (see `aead` module).

use rayon::prelude::*;
use crate::cascade;
use crate::frac_container::YAKSHA_DIMENSION;

/// Fixed dimension parameter for core mode (re-exported container dimension).
const DIMENSION: f64 = YAKSHA_DIMENSION;

/// Broadcast a byte to all lanes of a u64 (for block-linear mixing).
#[inline]
fn broadcast_u64(b: u8) -> u64 {
    (b as u64).wrapping_mul(0x0101_0101_0101_0101)
}

/// Multiplicative inverse of odd `a` modulo 2^64 (Newton iteration).
#[inline]
fn inv_mul_odd_u64(a: u64) -> u64 {
    debug_assert!(a & 1 == 1, "inverse requires odd multiplier");
    let mut x = 1u64;
    for _ in 0..7 {
        x = x.wrapping_mul(2u64.wrapping_sub(a.wrapping_mul(x)));
    }
    x
}

#[inline]
fn pk_u64_block(
    key: &[u8],
    key_len: usize,
    b: usize,
    frac_dim: f64,
    frac_factor: u8,
    kh: u64,
) -> u64 {
    let base = b * 8;
    let mut v = 0u64;
    for k in 0..8 {
        let i = base + k;
        let frac_influence = ((i as f64 * frac_dim) as u8).wrapping_mul(19);
        let pos_factor = ((i.wrapping_mul(17)) & 0xFF) as u8;
        let pk = key[i % key_len]
            .wrapping_add(pos_factor)
            .wrapping_add(frac_factor)
            .wrapping_add(frac_influence);
        v |= (pk as u64) << (8 * k);
    }
    v ^ kh.rotate_left((b as u32).wrapping_mul(11))
}

#[inline]
fn phase1_encrypt_u64(
    w: u64,
    key: &[u8],
    key_len: usize,
    block_idx: usize,
    membrane: u8,
    frac_dim: f64,
    frac_factor: u8,
    kh: u64,
) -> u64 {
    let base = block_idx * 8;
    let fi_byte = ((base as f64 * frac_dim) as u8).wrapping_mul(19);
    let fi_u = broadcast_u64(fi_byte);
    let membrane_u = broadcast_u64(membrane);
    let pk = pk_u64_block(key, key_len, block_idx, frac_dim, frac_factor, kh);
    let entropy = derive_entropy(key, base);
    let state = w.wrapping_add(membrane_u).wrapping_add(fi_u);
    let r1 = ((DIMENSION as u32) % 4) + 1;
    let r2 = ((DIMENSION as u32) % 3) + 1;
    let activated = if entropy > 127 {
        let odd = pk | 1u64;
        state.wrapping_mul(odd).rotate_left(r1)
    } else {
        state.wrapping_add(pk).rotate_right(r2)
    };
    activated ^ pk ^ fi_u
}

#[inline]
fn phase1_decrypt_u64(
    t: u64,
    key: &[u8],
    key_len: usize,
    block_idx: usize,
    membrane: u8,
    frac_dim: f64,
    frac_factor: u8,
    kh: u64,
) -> u64 {
    let base = block_idx * 8;
    let fi_byte = ((base as f64 * frac_dim) as u8).wrapping_mul(19);
    let fi_u = broadcast_u64(fi_byte);
    let membrane_u = broadcast_u64(membrane);
    let pk = pk_u64_block(key, key_len, block_idx, frac_dim, frac_factor, kh);
    let entropy = derive_entropy(key, base);
    let r1 = ((DIMENSION as u32) % 4) + 1;
    let r2 = ((DIMENSION as u32) % 3) + 1;
    let activated = t ^ pk ^ fi_u;
    let state = if entropy > 127 {
        let odd = pk | 1u64;
        let inv = inv_mul_odd_u64(odd);
        activated.rotate_right(r1).wrapping_mul(inv)
    } else {
        activated.rotate_left(r2).wrapping_sub(pk)
    };
    state.wrapping_sub(membrane_u).wrapping_sub(fi_u)
}

#[inline]
fn phase1_encrypt_byte(
    byte: u8,
    i: usize,
    key: &[u8],
    key_len: usize,
    membrane: u8,
    frac_dim: f64,
    frac_factor: u8,
    kh: u64,
) -> u8 {
    let frac_influence = ((i as f64 * frac_dim) as u8).wrapping_mul(19);
    let pos_factor = ((i.wrapping_mul(17)) & 0xFF) as u8;
    let mut pk = key[i % key_len]
        .wrapping_add(pos_factor)
        .wrapping_add(frac_factor)
        .wrapping_add(frac_influence);
    pk ^= (kh.rotate_left((i as u32) % 56) >> 56) as u8;
    let entropy = derive_entropy(key, i);
    let state = membrane.wrapping_add(byte).wrapping_add(frac_influence);
    let activated = if entropy > 127 {
        let pk_odd = pk | 1;
        state.wrapping_mul(pk_odd).rotate_left(((DIMENSION as u32) % 4) + 1)
    } else {
        state.wrapping_add(pk).rotate_right(((DIMENSION as u32) % 3) + 1)
    };
    activated ^ pk ^ frac_influence
}

#[inline]
fn phase1_decrypt_byte(
    t: u8,
    i: usize,
    key: &[u8],
    key_len: usize,
    membrane: u8,
    frac_dim: f64,
    frac_factor: u8,
    kh: u64,
) -> u8 {
    let frac_influence = ((i as f64 * frac_dim) as u8).wrapping_mul(19);
    let pos_factor = ((i.wrapping_mul(17)) & 0xFF) as u8;
    let mut pk = key[i % key_len]
        .wrapping_add(pos_factor)
        .wrapping_add(frac_factor)
        .wrapping_add(frac_influence);
    pk ^= (kh.rotate_left((i as u32) % 56) >> 56) as u8;
    let activated = t ^ pk ^ frac_influence;
    let entropy = derive_entropy(key, i);
    let state = if entropy > 127 {
        let pk_odd = pk | 1;
        let pk_inv = mod_inverse(pk_odd);
        activated.rotate_right(((DIMENSION as u32) % 4) + 1)
            .wrapping_mul(pk_inv)
    } else {
        activated.rotate_left(((DIMENSION as u32) % 3) + 1)
            .wrapping_sub(pk)
    };
    state.wrapping_sub(membrane).wrapping_sub(frac_influence)
}

/// Encrypt plaintext using Yaksha Core (shell: ocean + body).
pub fn encrypt(plaintext: &[u8], key: &[u8]) -> Vec<u8> {
    crate::shell::encrypt_primitive(plaintext, key)
}

/// Encrypt with explicit ocean session (used by AEAD shell).
pub fn encrypt_with_session(plaintext: &[u8], key: &[u8], session: &[u8]) -> Vec<u8> {
    crate::shell::encrypt(plaintext, key, session)
}

/// Core transform without ocean (for composition / GPU inner path).
pub fn encrypt_body(plaintext: &[u8], key: &[u8]) -> Vec<u8> {
    if plaintext.is_empty() || key.is_empty() {
        return plaintext.to_vec();
    }

    let len = plaintext.len();
    let key0 = key[0];
    let key_len = key.len();

    let frac_dim = DIMENSION - DIMENSION.floor();
    let dim_factor = ((DIMENSION * 100.0) as u8).wrapping_mul(17);
    let frac_factor = ((frac_dim * 256.0) as u8).wrapping_mul(23);
    let membrane = key0.wrapping_add(dim_factor);
    let kh = cascade::key_hash(key);

    let full_blocks = len / 8;
    let tail_start = full_blocks * 8;

    let mut output = vec![0u8; len];

    let blocks: Vec<u64> = (0..full_blocks)
        .into_par_iter()
        .map(|b| {
            let off = b * 8;
            let w = u64::from_le_bytes(
                plaintext[off..off + 8]
                    .try_into()
                    .expect("8-byte chunk"),
            );
            phase1_encrypt_u64(w, key, key_len, b, membrane, frac_dim, frac_factor, kh)
        })
        .collect();

    for b in 0..full_blocks {
        output[b * 8..b * 8 + 8].copy_from_slice(&blocks[b].to_le_bytes());
    }

    output[tail_start..]
        .par_iter_mut()
        .enumerate()
        .for_each(|(j, out_b)| {
            let i = tail_start + j;
            *out_b = phase1_encrypt_byte(
                plaintext[i],
                i,
                key,
                key_len,
                membrane,
                frac_dim,
                frac_factor,
                kh,
            );
        });

    cascade::wave_block_phase1_encrypt(&mut output, key);
    cascade::cascade_encrypt(&mut output, key);

    output
}

/// Decrypt ciphertext using Yaksha Core (shell: ocean + body).
pub fn decrypt(ciphertext: &[u8], key: &[u8]) -> Vec<u8> {
    crate::shell::decrypt_primitive(ciphertext, key)
}

/// Decrypt with explicit ocean session (used by AEAD shell).
pub fn decrypt_with_session(ciphertext: &[u8], key: &[u8], session: &[u8]) -> Vec<u8> {
    crate::shell::decrypt(ciphertext, key, session)
}

/// Core inverse without ocean.
pub fn decrypt_body(ciphertext: &[u8], key: &[u8]) -> Vec<u8> {
    if ciphertext.is_empty() || key.is_empty() {
        return ciphertext.to_vec();
    }

    let len = ciphertext.len();
    let key0 = key[0];
    let key_len = key.len();

    let frac_dim = DIMENSION - DIMENSION.floor();
    let dim_factor = ((DIMENSION * 100.0) as u8).wrapping_mul(17);
    let frac_factor = ((frac_dim * 256.0) as u8).wrapping_mul(23);
    let membrane = key0.wrapping_add(dim_factor);
    let kh = cascade::key_hash(key);

    let mut phase1_output = ciphertext.to_vec();
    cascade::cascade_decrypt(&mut phase1_output, key);
    cascade::wave_block_phase1_decrypt(&mut phase1_output, key);

    let full_blocks = len / 8;
    let tail_start = full_blocks * 8;

    let mut plaintext = vec![0u8; len];

    let blocks: Vec<u64> = (0..full_blocks)
        .into_par_iter()
        .map(|b| {
            let off = b * 8;
            let t = u64::from_le_bytes(
                phase1_output[off..off + 8]
                    .try_into()
                    .expect("8-byte chunk"),
            );
            phase1_decrypt_u64(t, key, key_len, b, membrane, frac_dim, frac_factor, kh)
        })
        .collect();

    for b in 0..full_blocks {
        plaintext[b * 8..b * 8 + 8].copy_from_slice(&blocks[b].to_le_bytes());
    }

    plaintext[tail_start..]
        .par_iter_mut()
        .enumerate()
        .for_each(|(j, pb)| {
            let i = tail_start + j;
            *pb = phase1_decrypt_byte(
                phase1_output[i],
                i,
                key,
                key_len,
                membrane,
                frac_dim,
                frac_factor,
                kh,
            );
        });

    plaintext
}

/// Derive deterministic entropy from key and position
///
/// This function is pure and produces the same output for same inputs,
/// ensuring encryption and decryption follow identical transformation paths.
fn derive_entropy(key: &[u8], position: usize) -> u8 {
    let key_hash: u64 = key
        .iter()
        .enumerate()
        .fold(0u64, |acc, (i, &b)| acc.wrapping_add((b as u64).wrapping_mul((i + 1) as u64)));
    let pos_hash = (position as u64).wrapping_mul(0x9E3779B9);
    ((key_hash ^ pos_hash) % 256) as u8
}

/// Compute modular multiplicative inverse for odd numbers mod 256
///
/// Uses Newton-Raphson iteration: x = x * (2 - a * x)
/// Converges in 4 iterations for 8-bit values.
fn mod_inverse(a: u8) -> u8 {
    let mut x = a;
    x = x.wrapping_mul(2u8.wrapping_sub(a.wrapping_mul(x)));
    x = x.wrapping_mul(2u8.wrapping_sub(a.wrapping_mul(x)));
    x = x.wrapping_mul(2u8.wrapping_sub(a.wrapping_mul(x)));
    x = x.wrapping_mul(2u8.wrapping_sub(a.wrapping_mul(x)));
    x
}
