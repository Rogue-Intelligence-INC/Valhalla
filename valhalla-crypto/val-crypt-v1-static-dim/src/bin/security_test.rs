//! Security Tests for Val-Crypt
//!
//! ChatGPT's recommended security analysis:
//! 1. Repeated plaintext test (determinism leakage)
//! 2. Linear correlation test (basic distinguisher)
//! 3. Differential trail analysis
//! 4. Known-plaintext cascade recovery
//! 5. Weak key detection

use val_crypt::{vector, entropy};
use std::collections::HashMap;

/// Simple PRNG for reproducible "random" test data
struct SimpleRng(u64);

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self(seed)
    }
    
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0
    }
    
    fn next_u8(&mut self) -> u8 {
        (self.next() >> 56) as u8
    }
    
    fn fill(&mut self, buf: &mut [u8]) {
        for byte in buf.iter_mut() {
            *byte = self.next_u8();
        }
    }
}

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║           Val-Crypt Security Analysis Suite                  ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Test 1: Determinism (same key + plaintext → same ciphertext?)
    test_determinism();
    
    // Test 2: Linear correlation
    test_linear_correlation();
    
    // Test 3: Differential trail analysis
    test_differential_trail();
    
    // Test 4: Known-plaintext cascade recovery
    test_known_plaintext();
    
    // Test 5: Weak key detection
    test_weak_keys();
    
    // Test 6: Distinguisher from random
    test_distinguisher();
    
    // Test 7: Bit position bias
    test_bit_position_bias();

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                    Analysis Complete                         ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
}

/// Test 1: Same key + same plaintext should produce same ciphertext
/// (This is expected for deterministic encryption - it's a "feature" not a bug,
///  but means ECB-like prefix leakage)
fn test_determinism() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Test 1: Determinism Test (ECB-like behavior)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let key = b"TestSecretKey123";
    let plaintext = b"Hello, World! This is a test message.";
    
    // Encrypt same plaintext twice with same key
    let c1_vec = vector::encrypt(plaintext, key);
    let c2_vec = vector::encrypt(plaintext, key);
    
    let c1_ent = entropy::encrypt(plaintext, key);
    let c2_ent = entropy::encrypt(plaintext, key);
    
    let vec_same = c1_vec == c2_vec;
    let ent_same = c1_ent == c2_ent;
    
    println!("  Vector mode: same plaintext → same ciphertext: {}", 
             if vec_same { "YES (deterministic)" } else { "NO (randomized)" });
    println!("  Entropy mode: same plaintext → same ciphertext: {}", 
             if ent_same { "YES (deterministic)" } else { "NO (randomized)" });
    
    // Test prefix leakage
    let p1 = b"Same prefix different ending AAA";
    let p2 = b"Same prefix different ending BBB";
    
    let c1 = vector::encrypt(p1, key);
    let c2 = vector::encrypt(p2, key);
    
    // Check how many bytes are identical at the beginning
    let prefix_match = c1.iter().zip(c2.iter()).take_while(|(a, b)| a == b).count();
    
    println!("\n  Prefix leakage test:");
    println!("    P1: \"{}\"", String::from_utf8_lossy(p1));
    println!("    P2: \"{}\"", String::from_utf8_lossy(p2));
    println!("    Identical ciphertext prefix bytes: {}", prefix_match);
    
    if prefix_match > 0 {
        println!("    ⚠️  PREFIX LEAKAGE DETECTED: {} bytes match", prefix_match);
    } else {
        println!("    ✓ No prefix leakage (cascade diffusion working)");
    }
    
    println!();
}

/// Test 2: Linear Correlation Test
/// Flip 1 bit in plaintext, check if C XOR C' distribution is random
fn test_linear_correlation() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Test 2: Linear Correlation Test");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let key = b"LinearCorrKey123";
    let num_samples = 10000;
    let mut rng = SimpleRng::new(12345);
    
    let mut xor_diff_histogram: [u32; 256] = [0; 256];
    
    for _ in 0..num_samples {
        // Actually random plaintext
        let mut plaintext = vec![0u8; 64];
        rng.fill(&mut plaintext);
        let mut plaintext_mod = plaintext.clone();
        
        // Flip random bit in first byte
        let bit_pos = rng.next_u8() % 8;
        plaintext_mod[0] ^= 1 << bit_pos;
        
        let c1 = vector::encrypt(&plaintext, key);
        let c2 = vector::encrypt(&plaintext_mod, key);
        
        // Record C1 XOR C2 for first byte
        if !c1.is_empty() && !c2.is_empty() {
            let diff = c1[0] ^ c2[0];
            xor_diff_histogram[diff as usize] += 1;
        }
    }
    
    // Check if distribution is uniform (chi-square test)
    let expected = num_samples as f64 / 256.0;
    let chi_square: f64 = xor_diff_histogram.iter()
        .map(|&observed| {
            let diff = observed as f64 - expected;
            diff * diff / expected
        })
        .sum();
    
    println!("  Samples: {}", num_samples);
    println!("  Chi-square statistic: {:.2}", chi_square);
    println!("  Expected (uniform): {:.2} per bin", expected);
    
    // Check for dominant values
    let max_count = *xor_diff_histogram.iter().max().unwrap();
    let min_count_nonzero = *xor_diff_histogram.iter().filter(|&&x| x > 0).min().unwrap_or(&1);
    let max_idx = xor_diff_histogram.iter().position(|&x| x == max_count).unwrap();
    let zero_bins = xor_diff_histogram.iter().filter(|&&x| x == 0).count();
    
    println!("  Max count: {} (value 0x{:02X})", max_count, max_idx);
    println!("  Min count (non-zero): {}", min_count_nonzero);
    println!("  Zero bins: {} / 256", zero_bins);
    println!("  Uniformity ratio: {:.3}", min_count_nonzero as f64 / max_count as f64);
    
    if chi_square > 350.0 {
        println!("  ⚠️  LINEAR BIAS DETECTED (chi-square too high)");
    } else {
        println!("  ✓ No significant linear correlation detected");
    }
    
    println!();
}

/// Test 3: Differential Trail Analysis for ARX rounds
fn test_differential_trail() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Test 3: Differential Trail Analysis (ARX)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let key = b"DifferentialKey!";
    
    // Test single-bit differences at each position
    let base_chunk = [0x00u8, 0x00, 0x00, 0x00];
    
    println!("  Input difference → Output difference propagation:");
    println!("  (Testing 4-byte chunks through Phase 1 ARX only)\n");
    
    let mut total_active_bytes = 0;
    let mut test_count = 0;
    
    for byte_pos in 0..4 {
        for bit_pos in 0..8 {
            let mut diff_chunk = base_chunk.clone();
            diff_chunk[byte_pos] ^= 1 << bit_pos;
            
            // Encrypt both through vector mode
            let c1 = vector::encrypt(&base_chunk, key);
            let c2 = vector::encrypt(&diff_chunk, key);
            
            // Count active bytes in output difference
            let active_bytes: u32 = c1.iter().zip(c2.iter())
                .map(|(&a, &b)| if a != b { 1 } else { 0 })
                .sum();
            
            total_active_bytes += active_bytes;
            test_count += 1;
        }
    }
    
    let avg_active = total_active_bytes as f64 / test_count as f64;
    println!("  Average active output bytes per 1-bit input diff: {:.2} / {}",
             avg_active, c1_len_or_4());
    
    // High probability differential search
    println!("\n  Searching for high-probability differentials...");
    
    let mut diff_counts: HashMap<Vec<u8>, u32> = HashMap::new();
    
    // Test many random plaintexts with same input difference
    let input_diff = [0x01u8, 0x00, 0x00, 0x00]; // 1-bit diff in first byte
    
    for base in 0..=255u8 {
        let p1 = [base, 0x00, 0x00, 0x00];
        let p2 = [base ^ input_diff[0], 0x00, 0x00, 0x00];
        
        let c1 = vector::encrypt(&p1, key);
        let c2 = vector::encrypt(&p2, key);
        
        let output_diff: Vec<u8> = c1.iter().zip(c2.iter())
            .take(4)
            .map(|(&a, &b)| a ^ b)
            .collect();
        
        *diff_counts.entry(output_diff).or_insert(0) += 1;
    }
    
    // Find most frequent output differential
    let mut sorted: Vec<_> = diff_counts.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));
    
    let (most_common_diff, count) = sorted[0];
    let probability = *count as f64 / 256.0;
    
    println!("  Most common output diff: {:02X?}", most_common_diff);
    println!("  Probability: {:.4} ({}/256)", probability, count);
    
    if probability > 0.1 {
        println!("  ⚠️  HIGH PROBABILITY DIFFERENTIAL FOUND: {:.1}%", probability * 100.0);
    } else {
        println!("  ✓ No high-probability differentials (max {:.2}%)", probability * 100.0);
    }
    
    println!();
}

fn c1_len_or_4() -> usize {
    // Vector mode adds 2 bytes for length, so 4-byte input becomes 6
    6
}

/// Test 4: Known-Plaintext Cascade State Recovery
fn test_known_plaintext() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Test 4: Known-Plaintext Attack (Cascade Recovery)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    // Simulate known-plaintext scenario
    let key = b"KnownPlaintxtKey";
    
    // Attacker knows plaintext and can see ciphertext
    let known_plaintext = b"AAAAAAAAAAAAAAAA"; // Predictable plaintext
    let ciphertext = entropy::encrypt(known_plaintext, key);
    
    println!("  Scenario: Attacker knows plaintext and observes ciphertext");
    println!("  Known plaintext: {:?}", String::from_utf8_lossy(known_plaintext));
    println!("  Ciphertext: {:02X?}\n", &ciphertext[..16.min(ciphertext.len())]);
    
    // If attacker can recover T values from Phase 1 (which is deterministic)
    // they can potentially reconstruct the cascade
    
    // Test if cascade state can be derived
    // cascade[0] is the initial state
    // C[i] = T[i] ^ cascade[i]
    // cascade[i+1] = rotl(cascade[i] + T[i], 1)
    
    // Try to brute-force initial cascade value
    println!("  Attempting to brute-force initial cascade state...");
    
    let mut found_cascade = false;
    let mut recovered_key0_candidates = 0;
    
    // The initial cascade is: key[0] + dim_factor
    // dim_factor = ((2.20 * 100.0) as u8).wrapping_mul(17) = 220 * 17 mod 256
    let dim_factor = (220u8).wrapping_mul(17);
    
    for candidate_key0 in 0..=255u8 {
        let _candidate_cascade = candidate_key0.wrapping_add(dim_factor);
        
        // Try to decrypt with this cascade and check if it produces valid output
        let decrypted = entropy::decrypt(&ciphertext, &[candidate_key0; 16]);
        
        // Check if decryption looks "correct" (we know it's all 'A's)
        if decrypted.iter().all(|&b| b == b'A') {
            recovered_key0_candidates += 1;
            found_cascade = true;
        }
    }
    
    if found_cascade {
        println!("  ⚠️  FOUND {} key[0] candidates that produce valid decryption", 
                 recovered_key0_candidates);
        println!("      (This is expected - known plaintext helps narrow key space)");
    } else {
        println!("  ✓ Brute-force cascade recovery failed");
    }
    
    // Calculate effective key bits exposed
    println!("\n  Information leakage analysis:");
    println!("    Original key[0] entropy: 8 bits (256 possibilities)");
    println!("    After known-plaintext: {} candidates", recovered_key0_candidates);
    let bits_exposed = 8.0 - (recovered_key0_candidates as f64).log2();
    println!("    Effective bits exposed: {:.2}", bits_exposed);
    
    println!();
}

/// Test 5: Weak Key Detection
fn test_weak_keys() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Test 5: Weak Key Detection");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let test_data: Vec<u8> = (0..256).map(|i| i as u8).collect();
    
    // Test all-zero key
    let zero_key = [0u8; 16];
    let c_zero = vector::encrypt(&test_data, &zero_key);
    let avalanche_zero = calculate_avalanche(&test_data, &c_zero);
    
    // Test all-ones key
    let ones_key = [0xFFu8; 16];
    let c_ones = vector::encrypt(&test_data, &ones_key);
    let avalanche_ones = calculate_avalanche(&test_data, &c_ones);
    
    // Test repeating pattern key
    let pattern_key = [0xAAu8, 0x55u8, 0xAAu8, 0x55u8, 0xAAu8, 0x55u8, 0xAAu8, 0x55u8,
                       0xAAu8, 0x55u8, 0xAAu8, 0x55u8, 0xAAu8, 0x55u8, 0xAAu8, 0x55u8];
    let c_pattern = vector::encrypt(&test_data, &pattern_key);
    let avalanche_pattern = calculate_avalanche(&test_data, &c_pattern);
    
    println!("  Key              Avalanche    Status");
    println!("  ────────────────────────────────────────");
    println!("  All zeros        {:.2}%       {}", 
             avalanche_zero * 100.0, 
             if avalanche_zero < 0.4 { "⚠️ WEAK" } else { "✓ OK" });
    println!("  All ones         {:.2}%       {}",
             avalanche_ones * 100.0,
             if avalanche_ones < 0.4 { "⚠️ WEAK" } else { "✓ OK" });
    println!("  AA55 pattern     {:.2}%       {}",
             avalanche_pattern * 100.0,
             if avalanche_pattern < 0.4 { "⚠️ WEAK" } else { "✓ OK" });
    
    // Test for related keys
    println!("\n  Related-key test:");
    let key1 = b"TestKey000000001";
    let key2 = b"TestKey000000002"; // Differs in last byte only
    
    let c1 = vector::encrypt(&test_data, key1);
    let c2 = vector::encrypt(&test_data, key2);
    
    let related_key_diff: u32 = c1.iter().zip(c2.iter())
        .map(|(&a, &b)| (a ^ b).count_ones())
        .sum();
    let related_key_avalanche = related_key_diff as f64 / (c1.len() * 8) as f64;
    
    println!("    Key1 vs Key2 (1 byte diff): {:.2}% bit change",
             related_key_avalanche * 100.0);
    
    if related_key_avalanche < 0.4 {
        println!("    ⚠️  RELATED KEY WEAKNESS DETECTED");
    } else {
        println!("    ✓ Good related-key resistance");
    }
    
    println!();
}

fn calculate_avalanche(plaintext: &[u8], ciphertext: &[u8]) -> f64 {
    let different_bits: u32 = plaintext.iter().zip(ciphertext.iter())
        .map(|(&p, &c)| (p ^ c).count_ones())
        .sum();
    different_bits as f64 / (plaintext.len() * 8) as f64
}

/// Test 6: Distinguisher from Random
fn test_distinguisher() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Test 6: Distinguisher from Random");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let key = b"DistinguisherKey";
    let mut rng = SimpleRng::new(67890);
    
    // Generate ciphertexts from RANDOM plaintexts
    let mut byte_histogram: [u32; 256] = [0; 256];
    let num_samples = 10000;
    
    for _ in 0..num_samples {
        let mut plaintext = vec![0u8; 64];
        rng.fill(&mut plaintext);
        let ciphertext = vector::encrypt(&plaintext, key);
        
        for &byte in ciphertext.iter() {
            byte_histogram[byte as usize] += 1;
        }
    }
    
    let total_bytes: u32 = byte_histogram.iter().sum();
    let expected = total_bytes as f64 / 256.0;
    
    // Chi-square test
    let chi_square: f64 = byte_histogram.iter()
        .map(|&observed| {
            let diff = observed as f64 - expected;
            diff * diff / expected
        })
        .sum();
    
    // Shannon entropy
    let entropy: f64 = byte_histogram.iter()
        .filter(|&&x| x > 0)
        .map(|&x| {
            let p = x as f64 / total_bytes as f64;
            -p * p.log2()
        })
        .sum();
    
    println!("  Samples: {} random plaintexts", num_samples);
    println!("  Total ciphertext bytes: {}", total_bytes);
    println!("  Chi-square: {:.2} (ideal: ~255)", chi_square);
    println!("  Shannon entropy: {:.4} bits/byte (ideal: 8.0)", entropy);
    
    // Find most/least common bytes
    let max_count = *byte_histogram.iter().max().unwrap();
    let min_count = *byte_histogram.iter().filter(|&&x| x > 0).min().unwrap_or(&1);
    
    println!("  Max byte frequency: {}", max_count);
    println!("  Min byte frequency: {}", min_count);
    println!("  Frequency ratio: {:.3}", max_count as f64 / min_count as f64);
    
    if chi_square > 350.0 {
        println!("  ⚠️  DISTRIBUTION BIAS DETECTED (distinguisher possible)");
    } else if entropy < 7.9 {
        println!("  ⚠️  LOW ENTROPY (distinguisher possible)");
    } else {
        println!("  ✓ Output appears indistinguishable from random");
    }
    
    println!();
}

/// Test 7: Bit Position Bias
fn test_bit_position_bias() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Test 7: Bit Position Bias Analysis");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let key = b"BitBiasTestKey!1";
    let num_samples = 10000;
    let mut rng = SimpleRng::new(11111);
    
    // Count '1' bits at each position (first 8 bytes = 64 bits)
    let mut bit_counts = [0u32; 64];
    
    for _ in 0..num_samples {
        let mut plaintext = vec![0u8; 64];
        rng.fill(&mut plaintext);
        let ciphertext = vector::encrypt(&plaintext, key);
        
        for (byte_idx, &byte) in ciphertext.iter().take(8).enumerate() {
            for bit_idx in 0..8 {
                if byte & (1 << bit_idx) != 0 {
                    bit_counts[byte_idx * 8 + bit_idx] += 1;
                }
            }
        }
    }
    
    let expected = num_samples as f64 / 2.0;
    
    // Find max deviation from 50%
    let max_bias = bit_counts.iter()
        .map(|&count| ((count as f64 - expected) / expected).abs())
        .fold(0.0f64, f64::max);
    
    let avg_bias: f64 = bit_counts.iter()
        .map(|&count| ((count as f64 - expected) / expected).abs())
        .sum::<f64>() / 64.0;
    
    println!("  Samples: {} random plaintexts", num_samples);
    println!("  Expected '1' count per bit: {} (50%)", num_samples / 2);
    println!("  Max bit bias: {:.2}%", max_bias * 100.0);
    println!("  Avg bit bias: {:.2}%", avg_bias * 100.0);
    
    // Show most biased positions
    let mut biases: Vec<(usize, f64)> = bit_counts.iter()
        .enumerate()
        .map(|(i, &count)| (i, ((count as f64 - expected) / expected).abs()))
        .collect();
    biases.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    
    println!("\n  Most biased bit positions:");
    for (pos, bias) in biases.iter().take(3) {
        let byte_pos = pos / 8;
        let bit_pos = pos % 8;
        let actual_count = bit_counts[*pos];
        let pct = actual_count as f64 / num_samples as f64 * 100.0;
        println!("    Byte {} bit {}: {:.2}% bias (actual: {:.1}%)", byte_pos, bit_pos, bias * 100.0, pct);
    }
    
    if max_bias > 0.1 {
        println!("\n  ⚠️  SIGNIFICANT BIT BIAS DETECTED (>10% deviation from 50%)");
    } else {
        println!("\n  ✓ No significant bit position bias");
    }
    
    println!();
}
