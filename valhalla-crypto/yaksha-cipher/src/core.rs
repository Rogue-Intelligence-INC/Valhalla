//! # Yaksha Core: Basic Encryption Primitive
//!
//! A deterministic symmetric cipher using position-dependent entropy trajectory.
//!
//! ## Properties
//! - **Deterministic**: Same (key, plaintext) always produces same ciphertext
//! - **Reversible**: 100% correct decryption
//! - **Avalanche**: ~50% bit difference on single-bit input change
//! - **Throughput**: ~800-1000 MB/s (CPU, rayon parallel)
//!
//! ## Algorithm
//!
//! ```text
//! Phase 1: Parallel per-byte transformation
//!   - Position-dependent entropy derivation
//!   - Branching based on entropy value
//!   - ARX operations (Add, Rotate, XOR)
//!
//! Phase 2: Random cascade (full diffusion)
//!   - Bidirectional random jump links
//!   - Every position affects every other position
//!   - GPU-acceleratable parallel design
//! ```
//!
//! ## Security Note
//!
//! This is a **deterministic primitive**. For semantic security (IND-CPA),
//! use with stateful key derivation (see `aead` module).

use rayon::prelude::*;
use crate::cascade;

/// Fixed dimension parameter for core mode
const DIMENSION: f64 = 2.20;

/// Encrypt plaintext using Yaksha Core algorithm
///
/// # Arguments
/// * `plaintext` - Data to encrypt
/// * `key` - Encryption key (any length > 0)
///
/// # Returns
/// Ciphertext of same length as plaintext
pub fn encrypt(plaintext: &[u8], key: &[u8]) -> Vec<u8> {
    if plaintext.is_empty() || key.is_empty() {
        return plaintext.to_vec();
    }

    let len = plaintext.len();
    let key0 = key[0];
    let key_len = key.len();
    
    // Derive transformation parameters from fixed dimension
    let frac_dim = DIMENSION - DIMENSION.floor();
    let dim_factor = ((DIMENSION * 100.0) as u8).wrapping_mul(17);
    let frac_factor = ((frac_dim * 256.0) as u8).wrapping_mul(23);
    let membrane = key0.wrapping_add(dim_factor);

    // Phase 1: Parallel transformation with deterministic entropy
    let mut output: Vec<u8> = (0..len)
        .into_par_iter()
        .map(|i| {
            let byte = plaintext[i];
            let frac_influence = ((i as f64 * frac_dim) as u8).wrapping_mul(19);
            let pos_factor = ((i.wrapping_mul(17)) & 0xFF) as u8;
            let pk = key[i % key_len]
                .wrapping_add(pos_factor)
                .wrapping_add(frac_factor)
                .wrapping_add(frac_influence);
            
            // Deterministic entropy determines transformation branch
            let entropy = derive_entropy(key, i);
            let state = membrane.wrapping_add(byte).wrapping_add(frac_influence);
            
            let activated = if entropy > 127 {
                // High entropy: multiply by odd number (always invertible mod 256)
                let pk_odd = pk | 1;
                state.wrapping_mul(pk_odd).rotate_left(((DIMENSION as u32) % 4) + 1)
            } else {
                // Low entropy: addition (always invertible)
                state.wrapping_add(pk).rotate_right(((DIMENSION as u32) % 3) + 1)
            };
            
            activated ^ pk ^ frac_influence
        })
        .collect();

    // Phase 2: Random cascade for full diffusion
    // Uses bidirectional random links for complete avalanche
    cascade::cascade_encrypt(&mut output, key);

    output
}

/// Decrypt ciphertext using Yaksha Core algorithm
///
/// # Arguments
/// * `ciphertext` - Data to decrypt
/// * `key` - Decryption key (must match encryption key)
///
/// # Returns
/// Original plaintext
pub fn decrypt(ciphertext: &[u8], key: &[u8]) -> Vec<u8> {
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

    // Phase 2 inverse: Remove random cascade
    let mut phase1_output = ciphertext.to_vec();
    cascade::cascade_decrypt(&mut phase1_output, key);

    // Phase 1 inverse: Reverse per-byte transformation
    let plaintext: Vec<u8> = (0..len)
        .into_par_iter()
        .map(|i| {
            let t = phase1_output[i];
            let frac_influence = ((i as f64 * frac_dim) as u8).wrapping_mul(19);
            let pos_factor = ((i.wrapping_mul(17)) & 0xFF) as u8;
            let pk = key[i % key_len]
                .wrapping_add(pos_factor)
                .wrapping_add(frac_factor)
                .wrapping_add(frac_influence);

            let activated = t ^ pk ^ frac_influence;
            
            // Same deterministic entropy path as encryption
            let entropy = derive_entropy(key, i);
            
            let state = if entropy > 127 {
                // Inverse of odd multiplication
                let pk_odd = pk | 1;
                let pk_inv = mod_inverse(pk_odd);
                activated.rotate_right(((DIMENSION as u32) % 4) + 1).wrapping_mul(pk_inv)
            } else {
                // Inverse of addition
                activated.rotate_left(((DIMENSION as u32) % 3) + 1).wrapping_sub(pk)
            };
            
            state.wrapping_sub(membrane).wrapping_sub(frac_influence)
        })
        .collect();

    plaintext
}

/// Derive deterministic entropy from key and position
///
/// This function is pure and produces the same output for same inputs,
/// ensuring encryption and decryption follow identical transformation paths.
fn derive_entropy(key: &[u8], position: usize) -> u8 {
    let key_hash: u64 = key.iter().enumerate()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_reversible() {
        let data = b"Hello, Yaksha Core!";
        let key = b"TestKey12345678!";
        
        let encrypted = encrypt(data, key);
        let decrypted = decrypt(&encrypted, key);
        
        assert_eq!(data.to_vec(), decrypted);
    }

    #[test]
    fn test_core_avalanche() {
        let data: Vec<u8> = (0..1024).map(|i| (i % 256) as u8).collect();
        let mut data_mod = data.clone();
        data_mod[0] ^= 0x01;
        
        let key = b"TestKey12345678!";
        let c1 = encrypt(&data, key);
        let c2 = encrypt(&data_mod, key);
        
        let diff: u32 = c1.iter().zip(c2.iter())
            .map(|(&a, &b)| (a ^ b).count_ones())
            .sum();
        let avalanche = diff as f64 / (c1.len() * 8) as f64;
        
        // Avalanche should be close to 50%
        assert!(avalanche > 0.45 && avalanche < 0.55, 
            "Avalanche effect: {:.1}% (expected 45-55%)", avalanche * 100.0);
    }

    #[test]
    fn test_core_deterministic() {
        let data = b"Same input twice";
        let key = b"SameKey";
        
        let c1 = encrypt(data, key);
        let c2 = encrypt(data, key);
        
        // Deterministic: same input always produces same output
        assert_eq!(c1, c2);
    }
}
