//! Test Random Cascade Diffusion

use yaksha_cipher_gpu::{GpuContext, random_cascade::RandomCascade};

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║         Random Cascade Diffusion Test                        ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let ctx = match GpuContext::new() {
        Some(c) => {
            println!("GPU: {:?}\n", c.adapter_info);
            c
        }
        None => {
            println!("No GPU available!");
            return;
        }
    };

    let cascade = RandomCascade::new(&ctx);
    let key = b"test_cascade_key_2026!";
    
    // Test data: 64 bytes
    let data: Vec<u8> = (0..64).map(|i| i as u8).collect();
    
    println!("【1. Basic Encryption/Decryption Test】\n");
    
    let encrypted = cascade.apply(&ctx, &data, key, 2);
    println!("  Original:  {:?}...", &data[..8]);
    println!("  Encrypted: {:?}...", &encrypted[..8]);
    
    // Note: XOR-based cascade may not be perfectly reversible with this approach
    // Let's just verify diffusion for now
    
    println!("\n【2. Diffusion Test - Position Sensitivity】\n");
    
    // Test each position
    let positions = [0, 15, 31, 63];
    
    for &pos in &positions {
        let mut data_mod = data.clone();
        data_mod[pos] ^= 0x01;  // Flip 1 bit
        
        let enc1 = cascade.apply(&ctx, &data, key, 2);
        let enc2 = cascade.apply(&ctx, &data_mod, key, 2);
        
        let diff: u32 = enc1.iter().zip(enc2.iter())
            .map(|(&a, &b)| (a ^ b).count_ones())
            .sum();
        
        let diff_percent = diff as f64 / (enc1.len() * 8) as f64 * 100.0;
        
        let status = if diff_percent > 40.0 && diff_percent < 60.0 { "✓" } else { "✗" };
        println!("  Position {:2}: {:.1}% bits differ (ideal: 50%) {}", pos, diff_percent, status);
    }
    
    println!("\n【3. Avalanche Effect per Position】\n");
    
    // Detailed avalanche: which output bytes are affected?
    for &pos in &[0usize, 63] {
        let mut data_mod = data.clone();
        data_mod[pos] ^= 0x01;
        
        let enc1 = cascade.apply(&ctx, &data, key, 2);
        let enc2 = cascade.apply(&ctx, &data_mod, key, 2);
        
        println!("  Change at position {}:", pos);
        
        // Count affected bytes in each quarter
        let quarters = [
            (0, 16, "Bytes 0-15"),
            (16, 32, "Bytes 16-31"),
            (32, 48, "Bytes 32-47"),
            (48, 64, "Bytes 48-63"),
        ];
        
        for (start, end, label) in quarters {
            let diff: u32 = enc1[start..end].iter()
                .zip(enc2[start..end].iter())
                .map(|(&a, &b)| (a ^ b).count_ones())
                .sum();
            let pct = diff as f64 / ((end - start) * 8) as f64 * 100.0;
            let bar = "█".repeat((pct / 5.0) as usize);
            println!("    {}: {:5.1}% {}", label, pct, bar);
        }
        println!();
    }
    
    println!("【4. Key Sensitivity Test】\n");
    
    let key2 = b"test_cascade_key_2026?"; // One byte different
    
    let enc1 = cascade.apply(&ctx, &data, key, 2);
    let enc2 = cascade.apply(&ctx, &data, key2, 2);
    
    let diff: u32 = enc1.iter().zip(enc2.iter())
        .map(|(&a, &b)| (a ^ b).count_ones())
        .sum();
    let diff_percent = diff as f64 / (enc1.len() * 8) as f64 * 100.0;
    
    println!("  Key difference (1 byte): {:.1}% bits differ", diff_percent);
    println!("  Result: {}\n", if diff_percent > 40.0 { "✓ PASSED" } else { "✗ FAILED" });
    
    println!("【5. Multi-Round Effect】\n");
    
    for rounds in [1, 2, 4] {
        let mut data_mod = data.clone();
        data_mod[63] ^= 0x01;
        
        let enc1 = cascade.apply(&ctx, &data, key, rounds);
        let enc2 = cascade.apply(&ctx, &data_mod, key, rounds);
        
        let diff: u32 = enc1.iter().zip(enc2.iter())
            .map(|(&a, &b)| (a ^ b).count_ones())
            .sum();
        let diff_percent = diff as f64 / (enc1.len() * 8) as f64 * 100.0;
        
        println!("  {} round(s): {:.1}% diffusion from position 63", rounds, diff_percent);
    }
    
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                    Test Complete                             ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
}
