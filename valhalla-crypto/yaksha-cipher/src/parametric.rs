//! # Yaksha Parametric: Key-Derived Transformation Parameters
//!
//! Encryption with transformation parameters derived from key material.
//! The "dimension" parameter influences rotation amounts, mixing factors,
//! and branch selections throughout the transformation.
//!
//! ## Properties
//! - **Deterministic**: Same (key, plaintext) = same ciphertext
//! - **Parameterized**: Key material derives a continuous parameter space
//! - **64B wave / superposition**: after per-byte phase, `cascade::wave_block_phase1_*` then `cascade_encrypt` (see `cascade` module)
//!
//! ## Algorithm Overview
//!
//! ```text
//! Key → ParametricSpace (dimension ∈ [1.5, 3.5])
//!         ↓
//! Phase 1a: Parallel per-byte transformation (local “particle” track)
//! Phase 1b: **64-byte** Feistel blocks — global coupling inside each chunk (wave–particle)
//! Phase 2: **64-byte** Feistel blocks, distinct domain (superposition); see `cascade`
//! ```

use rayon::prelude::*;
use crate::cascade;
use crate::frac_container::FracDimContainer;
use crate::ocean_pipeline::{decrypt_with_ocean, encrypt_with_ocean, primitive_session_anchor};

/// Parametric encryption state derived from key
///
/// The "dimension" parameter is a continuous value in [1.5, 3.5] derived
/// from key material. It influences transformation behavior but is NOT
/// a true mathematical dimension - it's a key-derived parameter.
#[derive(Debug, Clone)]
pub struct ParametricSpace {
    /// Current dimension parameter (1.5 - 3.5)
    pub dimension: f64,
    /// Base dimension (derived from key, constant for a session)
    pub base_dimension: f64,
    /// Number of dimension adaptations (for debugging)
    pub dimension_changes: u64,
    /// Signal tick counter (for stateful protocols)
    pub signal_tick: u64,
}

// Backward compatibility alias
pub type FateSpace = ParametricSpace;

impl ParametricSpace {
    /// Create ParametricSpace from key material
    ///
    /// Derives a base dimension in [1.5, 3.5] from key bytes.
    pub fn from_key(key: &[u8]) -> Self {
        let key_hash: u64 = key.iter().enumerate()
            .fold(0u64, |acc, (i, &b)| 
                acc.wrapping_add((b as u64).wrapping_mul((i + 1) as u64)));
        
        let base_dim = 1.5 + (key_hash % 2000) as f64 / 1000.0;
        
        Self {
            dimension: base_dim,
            base_dimension: base_dim,
            dimension_changes: 0,
            signal_tick: 0,
        }
    }
    
    /// Adapt dimension based on data complexity (optional, for experimental use)
    ///
    /// This method adjusts the dimension parameter based on input variance.
    /// Not used in standard encryption (which must be deterministic).
    #[inline]
    pub fn adapt(&mut self, values: &[u8]) {
        self.signal_tick += 1;
        
        if values.is_empty() {
            return;
        }
        
        // Compute data complexity (variance)
        let mean = values.iter().map(|&v| v as f64).sum::<f64>() / values.len() as f64;
        let variance = values.iter()
            .map(|&v| ((v as f64) - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        
        // Adjust dimension based on variance
        let complexity = variance.sqrt() / 128.0;
        
        if complexity > 0.5 {
            self.dimension = (self.dimension * 1.01).min(4.0);
            self.dimension_changes += 1;
        } else if complexity < 0.2 {
            self.dimension = (self.dimension * 0.99).max(1.5);
            self.dimension_changes += 1;
        }
    }
    
    /// Get dimension factor (u8 derived from dimension)
    #[inline]
    pub fn dim_factor(&self) -> u8 {
        ((self.dimension * 100.0) as u8).wrapping_mul(17)
    }
    
    /// Get fractional dimension factor
    #[inline]
    pub fn frac_factor(&self) -> u8 {
        let frac = self.dimension - self.dimension.floor();
        ((frac * 256.0) as u8).wrapping_mul(23)
    }
    
    /// Reset to base state (for decryption)
    pub fn reset(&mut self) {
        self.dimension = self.base_dimension;
        self.dimension_changes = 0;
        self.signal_tick = 0;
    }
}

// ============================================================================
// Vector Mode: ARX block cipher with parameterized transformations
// ============================================================================

/// Encrypt using parametric vector (ARX) mode + global ocean.
pub fn encrypt_vector(plaintext: &[u8], key: &[u8]) -> Vec<u8> {
    let session = primitive_session_anchor(key);
    encrypt_with_ocean(
        plaintext,
        key,
        &session,
        FracDimContainer::standard(),
        |p| encrypt_vector_body(p, key),
    )
}

fn encrypt_vector_body(plaintext: &[u8], key: &[u8]) -> Vec<u8> {
    if plaintext.is_empty() || key.is_empty() {
        return plaintext.to_vec();
    }

    let mut output = plaintext.to_vec();
    let len = output.len();
    
    // Create parametric space from key
    let fate = ParametricSpace::from_key(key);
    
    // Process in 4-byte chunks
    let num_chunks = (len + 3) / 4;
    
    for chunk_idx in 0..num_chunks {
        let start = chunk_idx * 4;
        let end = (start + 4).min(len);
        
        if end - start == 4 {
            let chunk_key = derive_chunk_key_fate(key, chunk_idx, &fate);
            arx_quarter_round(&mut output[start..end], chunk_key);
        }
    }

    cascade::wave_block_phase1_encrypt(&mut output, key);
    cascade::cascade_encrypt(&mut output, key);

    output
}

/// Decrypt using parametric vector (ARX) mode + global ocean.
pub fn decrypt_vector(ciphertext: &[u8], key: &[u8]) -> Vec<u8> {
    let session = primitive_session_anchor(key);
    decrypt_with_ocean(
        ciphertext,
        key,
        &session,
        FracDimContainer::standard(),
        |c| decrypt_vector_body(c, key),
    )
}

fn decrypt_vector_body(ciphertext: &[u8], key: &[u8]) -> Vec<u8> {
    if ciphertext.is_empty() || key.is_empty() {
        return ciphertext.to_vec();
    }

    let mut output = ciphertext.to_vec();
    let len = output.len();
    
    let fate = ParametricSpace::from_key(key);

    cascade::cascade_decrypt(&mut output, key);
    cascade::wave_block_phase1_decrypt(&mut output, key);

    // Inverse ARX in reverse order
    let num_chunks = (len + 3) / 4;
    
    for chunk_idx in (0..num_chunks).rev() {
        let start = chunk_idx * 4;
        let end = (start + 4).min(len);
        
        if end - start == 4 {
            let chunk_key = derive_chunk_key_fate(key, chunk_idx, &fate);
            arx_quarter_round_inverse(&mut output[start..end], chunk_key);
        }
    }

    output
}

fn derive_chunk_key_fate(key: &[u8], chunk_idx: usize, fate: &ParametricSpace) -> [u8; 4] {
    let dim_factor = fate.dim_factor();
    let frac_factor = fate.frac_factor();
    
    [
        key[chunk_idx * 4 % key.len()].wrapping_add(dim_factor),
        key[(chunk_idx * 4 + 1) % key.len()].wrapping_add(frac_factor),
        key[(chunk_idx * 4 + 2) % key.len()].wrapping_mul(17),
        key[(chunk_idx * 4 + 3) % key.len()].wrapping_mul(31),
    ]
}

fn arx_quarter_round(chunk: &mut [u8], k: [u8; 4]) {
    if chunk.len() < 4 { return; }
    
    let (mut a, mut b, mut c, mut d) = (chunk[0], chunk[1], chunk[2], chunk[3]);
    
    // Round 1
    a = a.wrapping_add(b).wrapping_add(k[0]);
    d = (d ^ a).rotate_left(4);
    c = c.wrapping_add(d).wrapping_add(k[1]);
    b = (b ^ c).rotate_left(5);
    
    // Round 2
    a = a.wrapping_add(b).wrapping_add(k[2]);
    d = (d ^ a).rotate_left(3);
    c = c.wrapping_add(d).wrapping_add(k[3]);
    b = (b ^ c).rotate_left(7);
    
    chunk[0] = a;
    chunk[1] = b;
    chunk[2] = c;
    chunk[3] = d;
}

fn arx_quarter_round_inverse(chunk: &mut [u8], k: [u8; 4]) {
    if chunk.len() < 4 { return; }
    
    let (mut a, mut b, mut c, mut d) = (chunk[0], chunk[1], chunk[2], chunk[3]);
    
    // Inverse Round 2
    b = b.rotate_right(7) ^ c;
    c = c.wrapping_sub(d).wrapping_sub(k[3]);
    d = d.rotate_right(3) ^ a;
    a = a.wrapping_sub(b).wrapping_sub(k[2]);
    
    // Inverse Round 1
    b = b.rotate_right(5) ^ c;
    c = c.wrapping_sub(d).wrapping_sub(k[1]);
    d = d.rotate_right(4) ^ a;
    a = a.wrapping_sub(b).wrapping_sub(k[0]);
    
    chunk[0] = a;
    chunk[1] = b;
    chunk[2] = c;
    chunk[3] = d;
}

// ============================================================================
// Entropy Mode: Parameterized entropy trajectory encryption
// ============================================================================

/// Encrypt using parametric entropy mode
///
/// This is the primary encryption function for the parametric module.
pub fn encrypt(plaintext: &[u8], key: &[u8]) -> Vec<u8> {
    encrypt_entropy(plaintext, key)
}

/// Decrypt using parametric entropy mode
///
/// This is the primary decryption function for the parametric module.
pub fn decrypt(ciphertext: &[u8], key: &[u8]) -> Vec<u8> {
    decrypt_entropy(ciphertext, key)
}

/// Encrypt using parametric entropy mode + global ocean.
pub fn encrypt_entropy(plaintext: &[u8], key: &[u8]) -> Vec<u8> {
    let session = primitive_session_anchor(key);
    encrypt_with_ocean(
        plaintext,
        key,
        &session,
        FracDimContainer::standard(),
        |p| encrypt_entropy_body(p, key),
    )
}

pub fn encrypt_entropy_body(plaintext: &[u8], key: &[u8]) -> Vec<u8> {
    if plaintext.is_empty() || key.is_empty() {
        return plaintext.to_vec();
    }

    let len = plaintext.len();
    let key0 = key[0];
    let key_len = key.len();
    
    // Create parametric space from key
    let fate = ParametricSpace::from_key(key);
    let dim_factor = fate.dim_factor();
    let frac_factor = fate.frac_factor();
    let membrane = key0.wrapping_add(dim_factor);
    let dim = fate.dimension;

    // Phase 1: Parallel transformation with parameterized operations
    let mut output: Vec<u8> = (0..len)
        .into_par_iter()
        .map(|i| {
            let byte = plaintext[i];
            
            // Position-dependent parameter offset (deterministic)
            let pos_dim_offset = derive_position_dim(key, i, dim);
            let local_frac = (dim + pos_dim_offset) - (dim + pos_dim_offset).floor();
            let local_frac_influence = ((local_frac * 256.0) as u8).wrapping_mul(19);
            
            let pos_factor = ((i.wrapping_mul(17)) & 0xFF) as u8;
            let pk = key[i % key_len]
                .wrapping_add(pos_factor)
                .wrapping_add(frac_factor)
                .wrapping_add(local_frac_influence);
            
            // Deterministic entropy determines branch
            let entropy = derive_entropy(key, i);
            let state = membrane.wrapping_add(byte).wrapping_add(local_frac_influence);
            
            let activated = if entropy > 127 {
                // High entropy: odd multiplication (always invertible mod 256)
                let pk_odd = pk | 1;
                state.wrapping_mul(pk_odd).rotate_left(((dim as u32) % 4) + 1)
            } else {
                // Low entropy: addition
                state.wrapping_add(pk).rotate_right(((dim as u32) % 3) + 1)
            };
            
            activated ^ pk ^ local_frac_influence
        })
        .collect();

    cascade::wave_block_phase1_encrypt(&mut output, key);
    cascade::cascade_encrypt(&mut output, key);

    output
}

/// Decrypt using parametric entropy mode + global ocean.
pub fn decrypt_entropy(ciphertext: &[u8], key: &[u8]) -> Vec<u8> {
    let session = primitive_session_anchor(key);
    decrypt_with_ocean(
        ciphertext,
        key,
        &session,
        FracDimContainer::standard(),
        |c| decrypt_entropy_body(c, key),
    )
}

pub fn decrypt_entropy_body(ciphertext: &[u8], key: &[u8]) -> Vec<u8> {
    if ciphertext.is_empty() || key.is_empty() {
        return ciphertext.to_vec();
    }

    let len = ciphertext.len();
    let key0 = key[0];
    let key_len = key.len();
    
    // Create parametric space from key (same as encryption)
    let fate = ParametricSpace::from_key(key);
    let dim_factor = fate.dim_factor();
    let frac_factor = fate.frac_factor();
    let membrane = key0.wrapping_add(dim_factor);
    let dim = fate.dimension;

    let mut phase1_output = ciphertext.to_vec();
    cascade::cascade_decrypt(&mut phase1_output, key);
    cascade::wave_block_phase1_decrypt(&mut phase1_output, key);

    // Phase 1a inverse: Reverse per-byte transformation
    let plaintext: Vec<u8> = (0..len)
        .into_par_iter()
        .map(|i| {
            let t = phase1_output[i];
            
            // Same position-dependent parameter offset as encryption
            let pos_dim_offset = derive_position_dim(key, i, dim);
            let local_frac = (dim + pos_dim_offset) - (dim + pos_dim_offset).floor();
            let local_frac_influence = ((local_frac * 256.0) as u8).wrapping_mul(19);
            
            let pos_factor = ((i.wrapping_mul(17)) & 0xFF) as u8;
            let pk = key[i % key_len]
                .wrapping_add(pos_factor)
                .wrapping_add(frac_factor)
                .wrapping_add(local_frac_influence);

            let activated = t ^ pk ^ local_frac_influence;
            
            // Same deterministic entropy path as encryption
            let entropy = derive_entropy(key, i);
            
            let state = if entropy > 127 {
                // Inverse of odd multiplication
                let pk_odd = pk | 1;
                let pk_inv = mod_inverse(pk_odd);
                activated.rotate_right(((dim as u32) % 4) + 1).wrapping_mul(pk_inv)
            } else {
                // Inverse of addition
                activated.rotate_left(((dim as u32) % 3) + 1).wrapping_sub(pk)
            };
            
            state.wrapping_sub(membrane).wrapping_sub(local_frac_influence)
        })
        .collect();

    plaintext
}

/// Derive deterministic entropy from key and position
fn derive_entropy(key: &[u8], position: usize) -> u8 {
    let key_hash: u64 = key.iter().enumerate()
        .fold(0u64, |acc, (i, &b)| acc.wrapping_add((b as u64).wrapping_mul((i + 1) as u64)));
    let pos_hash = (position as u64).wrapping_mul(0x9E3779B9);
    ((key_hash ^ pos_hash) % 256) as u8
}

/// Derive position-dependent dimension offset
fn derive_position_dim(key: &[u8], position: usize, base_dim: f64) -> f64 {
    let key_hash: u64 = key.iter().enumerate()
        .fold(0u64, |acc, (i, &b)| acc.wrapping_add((b as u64).wrapping_mul((i + 7) as u64)));
    let pos_hash = (position as u64).wrapping_mul(0x517CC1B7);
    let combined = key_hash ^ pos_hash;
    
    // Small offset (-0.1 to +0.1)
    let offset = ((combined % 200) as f64 - 100.0) / 1000.0;
    offset * base_dim.fract().max(0.1)
}

/// Modular multiplicative inverse for odd numbers mod 256
fn mod_inverse(a: u8) -> u8 {
    let mut x = a;
    x = x.wrapping_mul(2u8.wrapping_sub(a.wrapping_mul(x)));
    x = x.wrapping_mul(2u8.wrapping_sub(a.wrapping_mul(x)));
    x = x.wrapping_mul(2u8.wrapping_sub(a.wrapping_mul(x)));
    x = x.wrapping_mul(2u8.wrapping_sub(a.wrapping_mul(x)));
    x
}
