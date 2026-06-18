//! Yaksha shell: 2D ocean + core body.

use crate::core;
use crate::frac_container::FracDimContainer;
use crate::ocean_pipeline::{decrypt_with_ocean, encrypt_with_ocean, primitive_session_anchor};

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

pub fn encrypt_primitive(plaintext: &[u8], key: &[u8]) -> Vec<u8> {
    let session = primitive_session_anchor(key);
    encrypt(plaintext, key, &session)
}

pub fn decrypt_primitive(ciphertext: &[u8], key: &[u8]) -> Vec<u8> {
    let session = primitive_session_anchor(key);
    decrypt(ciphertext, key, &session)
}
