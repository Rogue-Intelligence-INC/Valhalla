//! Global two-phase ocean wrapper around cipher bodies.
//!
//! ```text
//! encrypt:  Ocean₁ → body(key) → Ocean₂
//! decrypt:  Ocean₂⁻¹ → body⁻¹(key) → Ocean₁⁻¹
//! ```

use crate::frac_container::FracDimContainer;
use crate::ocean_builtin::{ocean_apply_inplace, ocean_phase};

/// Derive a stable session anchor when no explicit nonce is supplied (primitive APIs).
pub fn primitive_session_anchor(key: &[u8]) -> [u8; 16] {
    let a = crate::ocean_builtin::fnv1a64(b"yaksha-ocean-session-v2");
    let b = crate::ocean_builtin::fnv1a64(key);
    let mut out = [0u8; 16];
    for i in 0..8 {
        out[i] = (a >> (i * 8)) as u8;
        out[i + 8] = (b >> (i * 8)) as u8;
    }
    out
}

/// Encrypt with global ocean phases around `body`.
pub fn encrypt_with_ocean<B>(
    plaintext: &[u8],
    ocean_key: &[u8],
    ocean_session: &[u8],
    container: FracDimContainer,
    body: B,
) -> Vec<u8>
where
    B: FnOnce(&[u8]) -> Vec<u8>,
{
    if plaintext.is_empty() || ocean_key.is_empty() {
        return plaintext.to_vec();
    }
    let mut buf = plaintext.to_vec();
    ocean_apply_inplace(
        &mut buf,
        ocean_key,
        ocean_session,
        ocean_phase::ONE,
        container,
    );
    let mut out = body(&buf);
    ocean_apply_inplace(
        &mut out,
        ocean_key,
        ocean_session,
        ocean_phase::TWO,
        container,
    );
    out
}

/// Decrypt: inverse ocean order around `body`.
pub fn decrypt_with_ocean<B>(
    ciphertext: &[u8],
    ocean_key: &[u8],
    ocean_session: &[u8],
    container: FracDimContainer,
    body: B,
) -> Vec<u8>
where
    B: FnOnce(&[u8]) -> Vec<u8>,
{
    if ciphertext.is_empty() || ocean_key.is_empty() {
        return ciphertext.to_vec();
    }
    let mut buf = ciphertext.to_vec();
    ocean_apply_inplace(
        &mut buf,
        ocean_key,
        ocean_session,
        ocean_phase::TWO,
        container,
    );
    let mut plain = body(&buf);
    ocean_apply_inplace(
        &mut plain,
        ocean_key,
        ocean_session,
        ocean_phase::ONE,
        container,
    );
    plain
}
