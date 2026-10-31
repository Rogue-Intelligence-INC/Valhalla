//! # Yaksha Core: Basic Encryption Primitive

use rayon::prelude::*;
use crate::cascade;

const DIMENSION: f64 = 2.20;

/// Rotation amounts derived from the *scaled* dimension so the fractional
/// part participates (a plain `as u32` cast discarded it entirely).
#[inline]
fn dim_rot_mul() -> u32 {
    (((DIMENSION * 100.0) as u32) % 4) + 1
}

#[inline]
fn dim_rot_add() -> u32 {
    (((DIMENSION * 100.0) as u32) % 3) + 1
}

/// Phase-1 parallel transform only (no cascade).
pub fn encrypt_phase1(plaintext: &[u8], key: &[u8]) -> Vec<u8> {
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

    (0..len)
        .into_par_iter()
        .map(|i| {
            let byte = plaintext[i];
            // Wrap modulo 256 instead of a saturating `as u8` cast, which
            // froze this channel at 255 for all positions beyond ~1275.
            let frac_influence = (((i as f64 * frac_dim) % 256.0) as u8).wrapping_mul(19);
            let pos_factor = ((i.wrapping_mul(17)) & 0xFF) as u8;
            let pk = key[i % key_len]
                .wrapping_add(pos_factor)
                .wrapping_add(frac_factor)
                .wrapping_add(frac_influence);

            let entropy = derive_entropy(key, i);
            let state = membrane.wrapping_add(byte).wrapping_add(frac_influence);

            let activated = if entropy > 127 {
                let pk_odd = pk | 1;
                state.wrapping_mul(pk_odd).rotate_left(dim_rot_mul())
            } else {
                state.wrapping_add(pk).rotate_right(dim_rot_add())
            };

            activated ^ pk ^ frac_influence
        })
        .collect()
}

/// Core body: phase1 + cascade diffusion.
pub fn encrypt_body(plaintext: &[u8], key: &[u8]) -> Vec<u8> {
    let mut output = encrypt_phase1(plaintext, key);
    cascade::cascade_encrypt(&mut output, key);
    output
}

/// Phase-1 inverse (input is phase1 buffer, not post-cascade).
pub fn decrypt_phase1(phase1_output: &[u8], key: &[u8]) -> Vec<u8> {
    if phase1_output.is_empty() || key.is_empty() {
        return phase1_output.to_vec();
    }

    let len = phase1_output.len();
    let key0 = key[0];
    let key_len = key.len();

    let frac_dim = DIMENSION - DIMENSION.floor();
    let dim_factor = ((DIMENSION * 100.0) as u8).wrapping_mul(17);
    let frac_factor = ((frac_dim * 256.0) as u8).wrapping_mul(23);
    let membrane = key0.wrapping_add(dim_factor);

    (0..len)
        .into_par_iter()
        .map(|i| {
            let t = phase1_output[i];
            let frac_influence = (((i as f64 * frac_dim) % 256.0) as u8).wrapping_mul(19);
            let pos_factor = ((i.wrapping_mul(17)) & 0xFF) as u8;
            let pk = key[i % key_len]
                .wrapping_add(pos_factor)
                .wrapping_add(frac_factor)
                .wrapping_add(frac_influence);

            let activated = t ^ pk ^ frac_influence;
            let entropy = derive_entropy(key, i);

            let state = if entropy > 127 {
                let pk_odd = pk | 1;
                let pk_inv = mod_inverse(pk_odd);
                activated
                    .rotate_right(dim_rot_mul())
                    .wrapping_mul(pk_inv)
            } else {
                activated
                    .rotate_left(dim_rot_add())
                    .wrapping_sub(pk)
            };

            state.wrapping_sub(membrane).wrapping_sub(frac_influence)
        })
        .collect()
}

/// Inverse of [`encrypt_body`].
pub fn decrypt_body(ciphertext: &[u8], key: &[u8]) -> Vec<u8> {
    if ciphertext.is_empty() || key.is_empty() {
        return ciphertext.to_vec();
    }

    let mut phase1_output = ciphertext.to_vec();
    cascade::cascade_decrypt(&mut phase1_output, key);
    decrypt_phase1(&phase1_output, key)
}

/// Encrypt: body only, or shell+ocean when `YAKSHA_SHELL=1` (default on).
pub fn encrypt(plaintext: &[u8], key: &[u8]) -> Vec<u8> {
    if crate::config::shell_enabled() {
        crate::shell::encrypt_primitive(plaintext, key)
    } else {
        encrypt_body(plaintext, key)
    }
}

/// Decrypt: matches [`encrypt`].
pub fn decrypt(ciphertext: &[u8], key: &[u8]) -> Vec<u8> {
    if crate::config::shell_enabled() {
        crate::shell::decrypt_primitive(ciphertext, key)
    } else {
        decrypt_body(ciphertext, key)
    }
}

/// Encrypt with explicit ocean session (AEAD).
pub fn encrypt_with_session(plaintext: &[u8], key: &[u8], session: &[u8]) -> Vec<u8> {
    crate::shell::encrypt(plaintext, key, session)
}

/// Decrypt with explicit ocean session (AEAD).
pub fn decrypt_with_session(ciphertext: &[u8], key: &[u8], session: &[u8]) -> Vec<u8> {
    crate::shell::decrypt(ciphertext, key, session)
}

fn derive_entropy(key: &[u8], position: usize) -> u8 {
    let key_hash: u64 = key
        .iter()
        .enumerate()
        .fold(0u64, |acc, (i, &b)| acc.wrapping_add((b as u64).wrapping_mul((i + 1) as u64)));
    let pos_hash = (position as u64).wrapping_mul(0x9E3779B9);
    ((key_hash ^ pos_hash) % 256) as u8
}

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
    fn test_body_reversible() {
        let data: Vec<u8> = (0..4096).map(|i| (i % 256) as u8).collect();
        let key = b"body_key_test_1234";
        let ct = encrypt_body(&data, key);
        let pt = decrypt_body(&ct, key);
        assert_eq!(data, pt);
    }
}
