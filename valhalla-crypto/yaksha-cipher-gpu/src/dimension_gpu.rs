//! Parametric cipher **body** — CPU only.
//!
//! Transform + cascade run on CPU (`yaksha_cipher::parametric`). GPU acceleration
//! applies only to ocean XOR in [`crate::shell_gpu`].

use crate::GpuContext;

pub fn encrypt_body(_ctx: &GpuContext, plaintext: &[u8], key: &[u8]) -> Vec<u8> {
    yaksha_cipher::parametric::encrypt_entropy_body(plaintext, key)
}

pub fn decrypt_body(_ctx: &GpuContext, ciphertext: &[u8], key: &[u8]) -> Vec<u8> {
    yaksha_cipher::parametric::decrypt_entropy_body(ciphertext, key)
}

pub fn encrypt(ctx: &GpuContext, plaintext: &[u8], key: &[u8]) -> Vec<u8> {
    encrypt_body(ctx, plaintext, key)
}

pub fn decrypt(ctx: &GpuContext, ciphertext: &[u8], key: &[u8]) -> Vec<u8> {
    decrypt_body(ctx, ciphertext, key)
}
