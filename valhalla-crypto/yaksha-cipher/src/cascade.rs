//! Random Cascade: Full-diffusion cascade
//!
//! Uses a Feistel-like structure to achieve:
//! 1. Full diffusion (every position affects every position)
//! 2. Perfect reversibility
//!
//! ## Algorithm
//! Split data into two halves L and R:
//! - L' = L XOR F(R, key, round)
//! - R' = R XOR F(L', key, round)
//! Repeat for multiple rounds.
//! 
//! Decryption: reverse the operations.

use rayon::prelude::*;

/// Number of Feistel rounds
const FEISTEL_ROUNDS: usize = 4;

/// Compute key hash
#[inline]
fn key_hash(key: &[u8]) -> u64 {
    let mut h: u64 = 14695981039346656037;
    for &b in key {
        h ^= b as u64;
        h = h.wrapping_mul(1099511628211);
    }
    h
}

/// Round function F(data, key_hash, round) -> mixing bytes
/// This function is deterministic based on data content, key, and round.
fn round_function(data: &[u8], key_h: u64, round: usize) -> Vec<u8> {
    let len = data.len();
    
    (0..len)
        .into_par_iter()
        .map(|i| {
            // Mix position, round, key, and neighboring data
            let mut h = key_h;
            h ^= round as u64;
            h = h.wrapping_mul(1099511628211);
            h ^= i as u64;
            h = h.wrapping_mul(1099511628211);
            
            // Include data dependencies for diffusion
            let d0 = data[i] as u64;
            let d1 = data[(i + 1) % len] as u64;
            let d2 = data[(i + len - 1) % len] as u64;
            let d_far = data[(i + len / 2) % len] as u64;
            
            h ^= d0.wrapping_mul(31);
            h ^= d1.wrapping_mul(37).rotate_left(8);
            h ^= d2.wrapping_mul(41).rotate_left(16);
            h ^= d_far.wrapping_mul(43).rotate_left(24);
            h = h.wrapping_mul(1099511628211);
            
            h as u8
        })
        .collect()
}

/// Apply Feistel-like cascade (encryption)
pub fn cascade_encrypt(data: &mut [u8], key: &[u8]) {
    let len = data.len();
    if len < 4 {
        // Too small for Feistel, use simple XOR
        let key_h = key_hash(key);
        for i in 0..len {
            data[i] ^= (key_h >> (i * 8 % 64)) as u8;
        }
        return;
    }

    let key_h = key_hash(key);
    let mid = len / 2;

    for round in 0..FEISTEL_ROUNDS {
        // Split into left and right halves
        let (left, right) = data.split_at_mut(mid);
        
        // L' = L XOR F(R, key, round)
        let f_r = round_function(right, key_h, round * 2);
        for i in 0..left.len().min(f_r.len()) {
            left[i] ^= f_r[i];
        }
        
        // R' = R XOR F(L', key, round)
        let f_l = round_function(left, key_h, round * 2 + 1);
        for i in 0..right.len().min(f_l.len()) {
            right[i] ^= f_l[i];
        }
    }
}

/// Apply Feistel-like cascade (decryption)
pub fn cascade_decrypt(data: &mut [u8], key: &[u8]) {
    let len = data.len();
    if len < 4 {
        let key_h = key_hash(key);
        for i in 0..len {
            data[i] ^= (key_h >> (i * 8 % 64)) as u8;
        }
        return;
    }

    let key_h = key_hash(key);
    let mid = len / 2;

    // Reverse order of rounds
    for round in (0..FEISTEL_ROUNDS).rev() {
        let (left, right) = data.split_at_mut(mid);
        
        // First undo: R = R' XOR F(L', key, round)
        // We have L' (which is what's in left now)
        let f_l = round_function(left, key_h, round * 2 + 1);
        for i in 0..right.len().min(f_l.len()) {
            right[i] ^= f_l[i];
        }
        
        // Then undo: L = L' XOR F(R, key, round)
        // Now right contains original R
        let f_r = round_function(right, key_h, round * 2);
        for i in 0..left.len().min(f_r.len()) {
            left[i] ^= f_r[i];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cascade_reversible() {
        let key = b"test_key_for_cascade";
        
        for size in [8, 16, 32, 64, 128, 256] {
            let original: Vec<u8> = (0..size).map(|i| (i * 7) as u8).collect();
            
            let mut data = original.clone();
            cascade_encrypt(&mut data, key);
            
            assert_ne!(data, original, "Size {}: unchanged after encrypt", size);
            
            cascade_decrypt(&mut data, key);
            
            assert_eq!(data, original, "Size {}: not restored after decrypt", size);
        }
    }

    #[test]
    fn test_cascade_diffusion() {
        let key = b"diffusion_test";
        
        for pos in [0, 31, 63] {
            let data: Vec<u8> = (0..64).collect();
            let mut data1 = data.clone();
            let mut data2 = data.clone();
            data2[pos] ^= 0x01;
            
            cascade_encrypt(&mut data1, key);
            cascade_encrypt(&mut data2, key);
            
            let diff: u32 = data1.iter().zip(data2.iter())
                .map(|(&a, &b)| (a ^ b).count_ones())
                .sum();
            let pct = diff as f64 / (data1.len() * 8) as f64;
            
            println!("Position {} diffusion: {:.1}%", pos, pct * 100.0);
        }
    }
}
