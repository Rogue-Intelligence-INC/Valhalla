//! GPU shell: 2D ocean XOR (GPU) + CPU body — matches `yaksha-cipher` primitive API.

use yaksha_cipher::frac_container::FracDimContainer;
use yaksha_cipher::ocean_builtin::ocean_phase;
use yaksha_cipher::{encrypt_with_ocean, primitive_session_anchor};

use crate::ocean_gpu::apply_ocean_phase_gpu;
use crate::GpuContext;

/// Neural primitive: Ocean₁ → neural body → Ocean₂ (same as CPU `neural::encrypt`).
pub fn encrypt_neural(ctx: &GpuContext, plaintext: &[u8], key: &[u8]) -> Vec<u8> {
    encrypt_with_ocean_gpu(ctx, plaintext, key, |p| yaksha_cipher::neural::encrypt_body(p, key))
}

/// Parametric primitive: Ocean₁ → parametric body → Ocean₂.
pub fn encrypt_parametric(ctx: &GpuContext, plaintext: &[u8], key: &[u8]) -> Vec<u8> {
    encrypt_with_ocean_gpu(ctx, plaintext, key, |p| {
        yaksha_cipher::parametric::encrypt_entropy_body(p, key)
    })
}

pub fn decrypt_neural(ctx: &GpuContext, ciphertext: &[u8], key: &[u8]) -> Vec<u8> {
    decrypt_with_ocean_gpu(ctx, ciphertext, key, |c| yaksha_cipher::neural::decrypt_body(c, key))
}

pub fn decrypt_parametric(ctx: &GpuContext, ciphertext: &[u8], key: &[u8]) -> Vec<u8> {
    decrypt_with_ocean_gpu(ctx, ciphertext, key, |c| {
        yaksha_cipher::parametric::decrypt_entropy_body(c, key)
    })
}

fn encrypt_with_ocean_gpu<B>(
    ctx: &GpuContext,
    plaintext: &[u8],
    key: &[u8],
    body: B,
) -> Vec<u8>
where
    B: FnOnce(&[u8]) -> Vec<u8>,
{
    if plaintext.is_empty() || key.is_empty() {
        return plaintext.to_vec();
    }
    let session = primitive_session_anchor(key);
    let container = FracDimContainer::standard();
    let phase1 = apply_ocean_phase_gpu(ctx, plaintext, key, &session, ocean_phase::ONE, container);
    let mid = body(&phase1);
    apply_ocean_phase_gpu(ctx, &mid, key, &session, ocean_phase::TWO, container)
}

fn decrypt_with_ocean_gpu<B>(
    ctx: &GpuContext,
    ciphertext: &[u8],
    key: &[u8],
    body: B,
) -> Vec<u8>
where
    B: FnOnce(&[u8]) -> Vec<u8>,
{
    if ciphertext.is_empty() || key.is_empty() {
        return ciphertext.to_vec();
    }
    let session = primitive_session_anchor(key);
    let container = FracDimContainer::standard();
    let after_p2 = apply_ocean_phase_gpu(ctx, ciphertext, key, &session, ocean_phase::TWO, container);
    let mid = body(&after_p2);
    apply_ocean_phase_gpu(ctx, &mid, key, &session, ocean_phase::ONE, container)
}

/// CPU-equivalent path using CPU ocean (for cross-check in tests).
pub fn encrypt_neural_cpu_ocean(plaintext: &[u8], key: &[u8]) -> Vec<u8> {
    let session = primitive_session_anchor(key);
    encrypt_with_ocean(
        plaintext,
        key,
        &session,
        FracDimContainer::standard(),
        |p| yaksha_cipher::neural::encrypt_body(p, key),
    )
}

pub fn encrypt_parametric_cpu_ocean(plaintext: &[u8], key: &[u8]) -> Vec<u8> {
    let session = primitive_session_anchor(key);
    encrypt_with_ocean(
        plaintext,
        key,
        &session,
        FracDimContainer::standard(),
        |p| yaksha_cipher::parametric::encrypt_entropy_body(p, key),
    )
}
