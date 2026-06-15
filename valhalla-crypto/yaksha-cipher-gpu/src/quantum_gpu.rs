//! Neural cipher **body** — CPU only.
//!
//! Transform + cascade run on CPU (`yaksha_cipher::neural`). This crate does not
//! GPU-accelerate the cipher body; see [`crate::shell_gpu`] for the full primitive
//! (ocean XOR on GPU + CPU body).

use crate::GpuContext;

pub fn encrypt_body(_ctx: &GpuContext, plaintext: &[u8], key: &[u8]) -> Vec<u8> {
    yaksha_cipher::neural::encrypt_body(plaintext, key)
}

pub fn decrypt_body(_ctx: &GpuContext, ciphertext: &[u8], key: &[u8]) -> Vec<u8> {
    yaksha_cipher::neural::decrypt_body(ciphertext, key)
}

pub fn encrypt(ctx: &GpuContext, plaintext: &[u8], key: &[u8]) -> Vec<u8> {
    encrypt_body(ctx, plaintext, key)
}

pub fn decrypt(ctx: &GpuContext, ciphertext: &[u8], key: &[u8]) -> Vec<u8> {
    decrypt_body(ctx, ciphertext, key)
}
