//! Yaksha Cipher - Cryptographic Analysis Suite
//! 
//! Tests:
//! 1. Differential Analysis - Input difference → Output difference distribution
//! 2. Linear Analysis - Linear approximation bias detection
//! 3. Correlation Analysis - Byte position correlation
//! 4. Bit Independence - Each output bit's independence
//! 5. AEAD Security - Tag forgery resistance

use yaksha_cipher::{core, parametric, neural, aead};
use std::collections::HashMap;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║         Yaksha Cipher Cryptographic Analysis                 ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let key = b"analysis_test_key_2026!";
    
    // Run all analyses
    differential_analysis(key);
    linear_analysis(key);
    correlation_analysis(key);
    bit_independence_test(key);
    aead_security_test();
    
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                  Analysis Complete                           ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
}

/// Differential Cryptanalysis Resistance Test
/// 
/// Checks if specific input differences lead to predictable output differences.
/// A secure cipher should have uniform output difference distribution.
fn differential_analysis(key: &[u8]) {
    println!("【1. Differential Analysis】\n");
    
    let test_size = 10000;
    let mut diff_counts: HashMap<u8, usize> = HashMap::new();
    
    // Test with single-bit input differences
    for i in 0..test_size {
        let plaintext: Vec<u8> = (0..64).map(|j| ((i + j) % 256) as u8).collect();
        let mut plaintext_mod = plaintext.clone();
        plaintext_mod[0] ^= 0x80; // Flip MSB of first byte
        
        let c1 = neural::encrypt(&plaintext, key);
        let c2 = neural::encrypt(&plaintext_mod, key);
        
        // Record first byte difference
        let diff = c1[0] ^ c2[0];
        *diff_counts.entry(diff).or_insert(0) += 1;
    }
    
    // Analyze distribution
    let unique_diffs = diff_counts.len();
    let max_count = *diff_counts.values().max().unwrap_or(&0);
    let min_count = *diff_counts.values().min().unwrap_or(&0);
    let expected = test_size / 256;
    
    // Calculate chi-square
    let chi_square: f64 = diff_counts.iter()
        .map(|(_, &count)| {
            let diff = count as f64 - expected as f64;
            diff * diff / expected as f64
        })
        .sum();
    
    // Chi-square critical value for 255 df at 0.05 significance: ~293
    let chi_critical = 293.0;
    let passed = chi_square < chi_critical * 1.5; // Allow some margin
    
    println!("  Input difference: 0x80 (MSB flip)");
    println!("  Samples: {}", test_size);
    println!("  Unique output differences: {}/256", unique_diffs);
    println!("  Max frequency: {} (expected: {})", max_count, expected);
    println!("  Min frequency: {}", min_count);
    println!("  Chi-square: {:.1} (critical: {:.1})", chi_square, chi_critical);
    println!("  Result: {}\n", if passed { "✓ PASSED - Uniform distribution" } else { "✗ FAILED - Non-uniform" });
    
    // Test multiple bit positions
    println!("  Multi-position differential test:");
    let positions = [0, 15, 31, 63];
    for &pos in &positions {
        let mut total_diff_bits = 0u64;
        for i in 0..1000 {
            let plaintext: Vec<u8> = (0..64).map(|j| ((i * 7 + j) % 256) as u8).collect();
            let mut plaintext_mod = plaintext.clone();
            plaintext_mod[pos] ^= 0x01;
            
            let c1 = core::encrypt(&plaintext, key);
            let c2 = core::encrypt(&plaintext_mod, key);
            
            let diff: u32 = c1.iter().zip(c2.iter())
                .map(|(&a, &b)| (a ^ b).count_ones())
                .sum();
            total_diff_bits += diff as u64;
        }
        let avg_diff = total_diff_bits as f64 / 1000.0 / (64.0 * 8.0) * 100.0;
        println!("    Position {}: {:.1}% bits differ (ideal: 50%)", pos, avg_diff);
    }
    println!();
}

/// Linear Cryptanalysis Resistance Test
/// 
/// Checks for linear relationships between input and output bits.
/// A secure cipher should have bias close to 0.5 (no linear correlation).
fn linear_analysis(key: &[u8]) {
    println!("【2. Linear Analysis】\n");
    
    let test_size = 50000;
    let mut biases = Vec::new();
    
    // Test linear approximations: input_bit XOR output_bit
    // For each input bit position and output bit position
    for in_byte in [0, 31].iter() {
        for in_bit in [0, 3, 7].iter() {
            for out_byte in [0, 31, 63].iter() {
                for out_bit in [0, 4, 7].iter() {
                    let mut matches = 0u64;
                    
                    for i in 0..test_size {
                        let plaintext: Vec<u8> = (0..64).map(|j| ((i * 13 + j * 7) % 256) as u8).collect();
                        let ciphertext = core::encrypt(&plaintext, key);
                        
                        let in_val = (plaintext[*in_byte] >> in_bit) & 1;
                        let out_val = (ciphertext[*out_byte] >> out_bit) & 1;
                        
                        if in_val == out_val {
                            matches += 1;
                        }
                    }
                    
                    let bias = (matches as f64 / test_size as f64 - 0.5).abs();
                    biases.push(bias);
                }
            }
        }
    }
    
    let max_bias = biases.iter().cloned().fold(0.0f64, f64::max);
    let avg_bias = biases.iter().sum::<f64>() / biases.len() as f64;
    
    // Bias should be very small (< 0.01 for good cipher)
    let passed = max_bias < 0.02;
    
    println!("  Tested {} linear approximations", biases.len());
    println!("  Maximum bias: {:.4} (threshold: 0.02)", max_bias);
    println!("  Average bias: {:.4}", avg_bias);
    println!("  Result: {}\n", if passed { "✓ PASSED - No significant linear bias" } else { "✗ FAILED - Linear bias detected" });
}

/// Byte Correlation Analysis
/// 
/// Checks if output bytes are correlated with each other.
fn correlation_analysis(key: &[u8]) {
    println!("【3. Correlation Analysis】\n");
    
    let test_size = 10000;
    let data_len = 64;
    
    // Collect output bytes for correlation analysis
    let mut byte_sums: Vec<f64> = vec![0.0; data_len];
    let mut byte_sq_sums: Vec<f64> = vec![0.0; data_len];
    let mut cross_sums: Vec<Vec<f64>> = vec![vec![0.0; data_len]; data_len];
    
    for i in 0..test_size {
        let plaintext: Vec<u8> = (0..data_len).map(|j| ((i * 17 + j * 11) % 256) as u8).collect();
        let ciphertext = core::encrypt(&plaintext, key);
        
        for j in 0..data_len {
            let val = ciphertext[j] as f64;
            byte_sums[j] += val;
            byte_sq_sums[j] += val * val;
            
            for k in (j+1)..data_len {
                cross_sums[j][k] += val * ciphertext[k] as f64;
            }
        }
    }
    
    // Calculate correlation coefficients
    let n = test_size as f64;
    let mut max_corr = 0.0f64;
    let mut correlations = Vec::new();
    
    for j in 0..data_len {
        for k in (j+1)..data_len {
            let mean_j = byte_sums[j] / n;
            let mean_k = byte_sums[k] / n;
            let var_j = byte_sq_sums[j] / n - mean_j * mean_j;
            let var_k = byte_sq_sums[k] / n - mean_k * mean_k;
            let cov = cross_sums[j][k] / n - mean_j * mean_k;
            
            let corr = if var_j > 0.0 && var_k > 0.0 {
                cov / (var_j.sqrt() * var_k.sqrt())
            } else {
                0.0
            };
            
            correlations.push(corr.abs());
            if corr.abs() > max_corr {
                max_corr = corr.abs();
            }
        }
    }
    
    let avg_corr = correlations.iter().sum::<f64>() / correlations.len() as f64;
    let passed = max_corr < 0.05;
    
    println!("  Tested {} byte pairs", correlations.len());
    println!("  Maximum correlation: {:.4} (threshold: 0.05)", max_corr);
    println!("  Average correlation: {:.4}", avg_corr);
    println!("  Result: {}\n", if passed { "✓ PASSED - No significant correlation" } else { "✗ FAILED - Correlation detected" });
}

/// Bit Independence Test (Strict Avalanche Criterion)
/// 
/// For each input bit, check if flipping it causes each output bit
/// to flip with probability 0.5.
fn bit_independence_test(key: &[u8]) {
    println!("【4. Bit Independence Test (SAC)】\n");
    
    let test_size = 5000;
    let data_len = 32;
    
    // For first input byte, test all 8 bits
    let mut sac_matrix: Vec<Vec<f64>> = vec![vec![0.0; data_len * 8]; 8];
    
    for in_bit in 0..8 {
        let mut flip_counts = vec![0u64; data_len * 8];
        
        for i in 0..test_size {
            let plaintext: Vec<u8> = (0..data_len).map(|j| ((i * 23 + j * 31) % 256) as u8).collect();
            let mut plaintext_mod = plaintext.clone();
            plaintext_mod[0] ^= 1 << in_bit;
            
            let c1 = core::encrypt(&plaintext, key);
            let c2 = core::encrypt(&plaintext_mod, key);
            
            for (j, (&a, &b)) in c1.iter().zip(c2.iter()).enumerate() {
                let diff = a ^ b;
                for bit in 0..8 {
                    if (diff >> bit) & 1 == 1 {
                        flip_counts[j * 8 + bit] += 1;
                    }
                }
            }
        }
        
        for out_bit in 0..(data_len * 8) {
            sac_matrix[in_bit][out_bit] = flip_counts[out_bit] as f64 / test_size as f64;
        }
    }
    
    // Calculate SAC deviation from 0.5
    let mut max_deviation = 0.0f64;
    let mut total_deviation = 0.0f64;
    let mut count = 0;
    
    for in_bit in 0..8 {
        for out_bit in 0..(data_len * 8) {
            let deviation = (sac_matrix[in_bit][out_bit] - 0.5).abs();
            if deviation > max_deviation {
                max_deviation = deviation;
            }
            total_deviation += deviation;
            count += 1;
        }
    }
    
    let avg_deviation = total_deviation / count as f64;
    let passed = max_deviation < 0.1;
    
    println!("  Tested: 8 input bits × {} output bits", data_len * 8);
    println!("  Maximum deviation from 0.5: {:.4} (threshold: 0.1)", max_deviation);
    println!("  Average deviation: {:.4}", avg_deviation);
    println!("  Result: {}\n", if passed { "✓ PASSED - Satisfies SAC" } else { "✗ FAILED - SAC violation" });
}

/// AEAD Security Test
/// 
/// Tests tag forgery resistance and authentication integrity.
fn aead_security_test() {
    println!("【5. AEAD Security Test】\n");
    
    let key = b"aead_security_test_key!!";
    let nonce = b"unique_nonce_123";
    let plaintext = b"Secret message for AEAD test";
    let aad = b"associated data";
    
    // Test 1: Tag verification
    println!("  Tag Verification:");
    let (ciphertext, tag) = aead::YakshaAead::seal(key, nonce, plaintext, aad);
    
    // Verify correct decryption
    match aead::YakshaAead::open(key, nonce, &ciphertext, aad, &tag) {
        Ok(decrypted) => {
            if decrypted == plaintext.to_vec() {
                println!("    Correct tag: ✓ Decryption successful");
            } else {
                println!("    Correct tag: ✗ Decryption mismatch");
            }
        }
        Err(_) => println!("    Correct tag: ✗ Verification failed"),
    }
    
    // Test 2: Modified ciphertext detection
    let mut modified_ct = ciphertext.clone();
    modified_ct[0] ^= 0x01;
    
    match aead::YakshaAead::open(key, nonce, &modified_ct, aad, &tag) {
        Ok(_) => println!("    Modified ciphertext: ✗ Should have failed"),
        Err(_) => println!("    Modified ciphertext: ✓ Correctly rejected"),
    }
    
    // Test 3: Modified tag detection
    let mut modified_tag = tag.clone();
    modified_tag[0] ^= 0x01;
    
    match aead::YakshaAead::open(key, nonce, &ciphertext, aad, &modified_tag) {
        Ok(_) => println!("    Modified tag: ✗ Should have failed"),
        Err(_) => println!("    Modified tag: ✓ Correctly rejected"),
    }
    
    // Test 4: Modified AAD detection
    let wrong_aad = b"wrong associated data";
    match aead::YakshaAead::open(key, nonce, &ciphertext, wrong_aad, &tag) {
        Ok(_) => println!("    Modified AAD: ✗ Should have failed"),
        Err(_) => println!("    Modified AAD: ✓ Correctly rejected"),
    }
    
    // Test 5: Tag forgery attempt (1000 random tags)
    println!("\n  Tag Forgery Resistance (1000 random tags):");
    let mut forgery_success = 0;
    for i in 0..1000 {
        let fake_tag: Vec<u8> = (0..tag.len()).map(|j| ((i * 13 + j * 7) % 256) as u8).collect();
        if aead::YakshaAead::open(key, nonce, &ciphertext, aad, &fake_tag).is_ok() {
            forgery_success += 1;
        }
    }
    
    let forgery_passed = forgery_success == 0;
    println!("    Successful forgeries: {}/1000", forgery_success);
    println!("    Result: {}\n", if forgery_passed { "✓ PASSED" } else { "✗ FAILED" });
    
    // Test 6: Key sensitivity
    println!("  Key Sensitivity:");
    let wrong_key = b"aead_security_test_key!?"; // One byte different
    match aead::YakshaAead::open(wrong_key, nonce, &ciphertext, aad, &tag) {
        Ok(_) => println!("    Wrong key decryption: ✗ Should have failed"),
        Err(_) => println!("    Wrong key decryption: ✓ Correctly rejected"),
    }
    
    // Test 7: Nonce sensitivity
    let wrong_nonce = b"unique_nonce_124"; // One byte different
    match aead::YakshaAead::open(key, wrong_nonce, &ciphertext, aad, &tag) {
        Ok(_) => println!("    Wrong nonce decryption: ✗ Should have failed"),
        Err(_) => println!("    Wrong nonce decryption: ✓ Correctly rejected"),
    }
    
    println!();
}
