//! # Yaksha Neural: Neural Network-Inspired Encryption
//!
//! Encryption algorithm inspired by the MeshBrain neural network model.
//! Uses membrane dynamics, network connectivity, and signal propagation
//! concepts as transformation primitives.
//!
//! ## MeshBrain Parameters (from systems/metasystems/meshbrain)
//! - `BASE_MEMBRANE_THICKNESS`: 10.0 (barrier between processing layers)
//! - `NETWORK_DENSITY`: 0.7 (connection density)
//! - `FLUCTUATION_INTENSITY`: 0.3 (random perturbation strength)
//!
//! ## Signal Types
//! - `ContinuousFluctuation`: intensity <= 0.5 (additive transformations)
//! - `StructuralReorganization`: intensity > 0.5 (multiplicative transformations)
//!
//! ## Security Properties
//! - **Deterministic**: Same (key, plaintext) = same ciphertext
//! - **64B blocks**: same wave-then-superposition Feistel layers as `parametric` / `cascade`
//! - **Reversible**: 100% correct decryption

use rayon::prelude::*;
use crate::cascade;
use crate::frac_container::FracDimContainer;
use crate::ocean_pipeline::{decrypt_with_ocean, encrypt_with_ocean, primitive_session_anchor};

// ============================================================================
// MeshBrain Configuration Constants
// ============================================================================

/// Base membrane thickness (barrier parameter)
pub const BASE_MEMBRANE_THICKNESS: f64 = 10.0;

/// Network density (connection strength)
pub const NETWORK_DENSITY: f64 = 0.7;

/// Fluctuation intensity (perturbation factor)
pub const FLUCTUATION_INTENSITY: f64 = 0.3;

/// Core count (for simulation scaling, not used in encryption)
pub const CORE_COUNT: usize = 1000;

/// Signal type determines transformation branch
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SignalType {
    /// Continuous fluctuation mode (intensity <= 0.5)
    /// Uses additive transformations
    ContinuousFluctuation,
    /// Structural reorganization mode (intensity > 0.5)
    /// Uses multiplicative transformations
    StructuralReorganization,
}

// Backward compatibility aliases
pub use SignalType::ContinuousFluctuation as ContinuousFluctuationMode;
pub use SignalType::StructuralReorganization as StructuralReorganizationMode;

impl SignalType {
    /// Determine signal type from intensity value
    #[inline]
    pub fn from_intensity(intensity: f64) -> Self {
        if intensity > 0.5 {
            Self::StructuralReorganization
        } else {
            Self::ContinuousFluctuation
        }
    }
}

// ============================================================================
// Neural Encryption Core
// ============================================================================

/// Encrypt plaintext using neural transformations + global ocean phases.
pub fn encrypt(plaintext: &[u8], key: &[u8]) -> Vec<u8> {
    let session = primitive_session_anchor(key);
    encrypt_with_ocean(
        plaintext,
        key,
        &session,
        FracDimContainer::standard(),
        |p| encrypt_body(p, key),
    )
}

/// Neural transform body (wave + cascade), no ocean.
pub fn encrypt_body(plaintext: &[u8], key: &[u8]) -> Vec<u8> {
    if plaintext.is_empty() || key.is_empty() {
        return plaintext.to_vec();
    }

    let len = plaintext.len();
    let key_len = key.len();
    
    // Derive MeshBrain parameters from key
    let key_sum: u64 = key.iter().map(|&b| b as u64).sum();
    let membrane_base = (key[0] as f64 / 255.0) * BASE_MEMBRANE_THICKNESS + 1.0;
    let density = NETWORK_DENSITY + (key_sum % 100) as f64 / 1000.0;
    let fluctuation = FLUCTUATION_INTENSITY + (key_sum % 50) as f64 / 500.0;

    // Phase 1: Neural transformation (parallel)
    let mut output: Vec<u8> = (0..len)
        .into_par_iter()
        .map(|i| {
            let byte = plaintext[i];
            let k = key[i % key_len];
            
            // Compute 3D position factor (simulates core positions in MeshBrain)
            let pos_x = (i % 16) as f64 / 16.0;
            let pos_y = ((i / 16) % 16) as f64 / 16.0;
            let pos_z = (i / 256) as f64 / 256.0;
            let position_factor = ((pos_x + pos_y + pos_z) * 255.0) as u8;
            
            // Use deterministic entropy to determine signal type (ensures reversibility)
            let entropy = derive_entropy(key, i);
            let intensity = entropy as f64 / 255.0;
            
            // Membrane thickness formula: base × (1 + intensity × 0.1)
            let membrane_thickness = membrane_base * (1.0 + intensity * 0.1);
            let membrane_factor = ((membrane_thickness * 10.0) as u8).wrapping_mul(17);
            
            // Intelligence level formula: intensity × fluctuation_intensity
            let intelligence = intensity * fluctuation;
            let intelligence_factor = ((intelligence * 255.0) as u8).wrapping_mul(23);
            
            // Network connectivity factor
            let connectivity_factor = ((density * 255.0) as u8).wrapping_add(k);
            
            // Determine signal type (based on entropy, consistent with decryption)
            let signal_type = SignalType::from_intensity(intensity);
            
            // Initial state mixing
            let state = byte
                .wrapping_add(membrane_factor)
                .wrapping_add(position_factor);
            
            // Branch based on signal type
            let activated = match signal_type {
                SignalType::StructuralReorganization => {
                    // Structural reorganization: strong nonlinear transformation
                    let pk_odd = (k.wrapping_add(intelligence_factor)) | 1;
                    state.wrapping_mul(pk_odd)
                        .rotate_left(((membrane_thickness as u32) % 5) + 1)
                        .wrapping_add(connectivity_factor)
                }
                SignalType::ContinuousFluctuation => {
                    // Continuous fluctuation: additive transformation
                    state.wrapping_add(k)
                        .wrapping_add(intelligence_factor)
                        .rotate_right(((density * 7.0) as u32) + 1)
                }
            };
            
            // Final mixing
            activated ^ k ^ membrane_factor ^ intelligence_factor
        })
        .collect();

    cascade::wave_block_phase1_encrypt(&mut output, key);
    cascade::cascade_encrypt(&mut output, key);

    output
}

/// Decrypt ciphertext using neural transformations + global ocean phases.
pub fn decrypt(ciphertext: &[u8], key: &[u8]) -> Vec<u8> {
    let session = primitive_session_anchor(key);
    decrypt_with_ocean(
        ciphertext,
        key,
        &session,
        FracDimContainer::standard(),
        |c| decrypt_body(c, key),
    )
}

/// Neural inverse body, no ocean.
pub fn decrypt_body(ciphertext: &[u8], key: &[u8]) -> Vec<u8> {
    if ciphertext.is_empty() || key.is_empty() {
        return ciphertext.to_vec();
    }

    let len = ciphertext.len();
    let key_len = key.len();
    
    // Derive MeshBrain parameters (must match encryption exactly)
    let key_sum: u64 = key.iter().map(|&b| b as u64).sum();
    let membrane_base = (key[0] as f64 / 255.0) * BASE_MEMBRANE_THICKNESS + 1.0;
    let density = NETWORK_DENSITY + (key_sum % 100) as f64 / 1000.0;
    let fluctuation = FLUCTUATION_INTENSITY + (key_sum % 50) as f64 / 500.0;

    let mut phase1_output = ciphertext.to_vec();
    cascade::cascade_decrypt(&mut phase1_output, key);
    cascade::wave_block_phase1_decrypt(&mut phase1_output, key);

    // Phase 1 inverse: Neural reverse transformation (parallel)
    let plaintext: Vec<u8> = (0..len)
        .into_par_iter()
        .map(|i| {
            let t = phase1_output[i];
            let k = key[i % key_len];
            
            // Recompute all factors (must match encryption exactly)
            let pos_x = (i % 16) as f64 / 16.0;
            let pos_y = ((i / 16) % 16) as f64 / 16.0;
            let pos_z = (i / 256) as f64 / 256.0;
            let position_factor = ((pos_x + pos_y + pos_z) * 255.0) as u8;
            
            // Same entropy-based signal type as encryption
            let entropy = derive_entropy(key, i);
            let intensity = entropy as f64 / 255.0;
            
            // Recompute all factors
            let membrane_thickness = membrane_base * (1.0 + intensity * 0.1);
            let membrane_factor = ((membrane_thickness * 10.0) as u8).wrapping_mul(17);
            let intelligence = intensity * fluctuation;
            let intelligence_factor = ((intelligence * 255.0) as u8).wrapping_mul(23);
            let connectivity_factor = ((density * 255.0) as u8).wrapping_add(k);
            let signal_type = SignalType::from_intensity(intensity);
            
            // Remove final mixing
            let activated = t ^ k ^ membrane_factor ^ intelligence_factor;
            
            // Inverse transformation based on signal type
            let state = match signal_type {
                SignalType::StructuralReorganization => {
                    let pk_odd = (k.wrapping_add(intelligence_factor)) | 1;
                    let pk_inv = mod_inverse(pk_odd);
                    
                    activated
                        .wrapping_sub(connectivity_factor)
                        .rotate_right(((membrane_thickness as u32) % 5) + 1)
                        .wrapping_mul(pk_inv)
                }
                SignalType::ContinuousFluctuation => {
                    activated
                        .rotate_left(((density * 7.0) as u32) + 1)
                        .wrapping_sub(intelligence_factor)
                        .wrapping_sub(k)
                }
            };
            
            // Remove initial state mixing
            state
                .wrapping_sub(position_factor)
                .wrapping_sub(membrane_factor)
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

/// Modular multiplicative inverse for odd numbers mod 256
fn mod_inverse(a: u8) -> u8 {
    let mut x = a;
    x = x.wrapping_mul(2u8.wrapping_sub(a.wrapping_mul(x)));
    x = x.wrapping_mul(2u8.wrapping_sub(a.wrapping_mul(x)));
    x = x.wrapping_mul(2u8.wrapping_sub(a.wrapping_mul(x)));
    x = x.wrapping_mul(2u8.wrapping_sub(a.wrapping_mul(x)));
    x
}

// ============================================================================
// Neural State Structure
// ============================================================================

/// Neural encryption state derived from key
#[derive(Debug, Clone)]
pub struct NeuralState {
    pub membrane_base: f64,
    pub network_density: f64,
    pub fluctuation_intensity: f64,
}

// Backward compatibility alias
pub type MeshBrainState = NeuralState;

impl NeuralState {
    /// Create neural state from key material
    pub fn from_key(key: &[u8]) -> Self {
        let key_sum: u64 = key.iter().map(|&b| b as u64).sum();
        Self {
            membrane_base: (key[0] as f64 / 255.0) * BASE_MEMBRANE_THICKNESS + 1.0,
            network_density: NETWORK_DENSITY + (key_sum % 100) as f64 / 1000.0,
            fluctuation_intensity: FLUCTUATION_INTENSITY + (key_sum % 50) as f64 / 500.0,
        }
    }
}
