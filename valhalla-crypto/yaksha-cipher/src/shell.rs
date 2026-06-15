//! Yaksha shell (外壳): **2D ocean interference** + **core transform body**.
//!
//! ```text
//! Shell encrypt:  Ocean₁(enc_key, session) → core body(enc_key) → Ocean₂
//! AEAD seal:      KDF(master) → enc_key + mac_key → Shell → YakshaMac tag
//! ```

use crate::core;
use crate::frac_container::FracDimContainer;
use crate::ocean_pipeline::{decrypt_with_ocean, encrypt_with_ocean, primitive_session_anchor};

/// Shell encrypt: ocean phases bound to `(enc_key, session)` + core body.
pub fn encrypt(plaintext: &[u8], enc_key: &[u8], session: &[u8]) -> Vec<u8> {
    if plaintext.is_empty() || enc_key.is_empty() {
        return plaintext.to_vec();
    }
    encrypt_with_ocean(
        plaintext,
        enc_key,
        session,
        FracDimContainer::standard(),
        |p| core::encrypt_body(p, enc_key),
    )
}

/// Shell decrypt: inverse of [`encrypt`].
pub fn decrypt(ciphertext: &[u8], enc_key: &[u8], session: &[u8]) -> Vec<u8> {
    if ciphertext.is_empty() || enc_key.is_empty() {
        return ciphertext.to_vec();
    }
    decrypt_with_ocean(
        ciphertext,
        enc_key,
        session,
        FracDimContainer::standard(),
        |c| core::decrypt_body(c, enc_key),
    )
}

/// Primitive entry: session anchor derived from key only (no external nonce).
pub fn encrypt_primitive(plaintext: &[u8], key: &[u8]) -> Vec<u8> {
    let session = primitive_session_anchor(key);
    encrypt(plaintext, key, &session)
}

/// Primitive decrypt (see [`encrypt_primitive`]).
pub fn decrypt_primitive(ciphertext: &[u8], key: &[u8]) -> Vec<u8> {
    let session = primitive_session_anchor(key);
    decrypt(ciphertext, key, &session)
}

/// Core transform only — **no ocean** (for GPU / evolve inner pipelines).
pub fn core_body_encrypt(plaintext: &[u8], key: &[u8]) -> Vec<u8> {
    core::encrypt_body(plaintext, key)
}

/// Inverse of [`core_body_encrypt`].
pub fn core_body_decrypt(ciphertext: &[u8], key: &[u8]) -> Vec<u8> {
    core::decrypt_body(ciphertext, key)
}
