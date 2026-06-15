//! Yaksha Cipher comprehensive test suite

use yaksha_cipher::{core, parametric, neural};

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║           Yaksha Cipher Test Suite                           ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    test_correctness();
    test_avalanche();
    test_key_sensitivity();

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                    All Tests Passed                          ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
}

fn test_correctness() {
    println!("【Correctness Tests】\n");
    
    let test_data = b"Hello, Yaksha Cipher! This is a test message for encryption.";
    let key = b"secret_key_1234!";
    
    // Core mode
    let encrypted = core::encrypt(test_data, key);
    let decrypted = core::decrypt(&encrypted, key);
    let core_ok = test_data.to_vec() == decrypted;
    println!("  Core mode:       {}", if core_ok { "✓ PASS" } else { "✗ FAIL" });
    
    // Parametric mode
    let encrypted = parametric::encrypt(test_data, key);
    let decrypted = parametric::decrypt(&encrypted, key);
    let param_ok = test_data.to_vec() == decrypted;
    println!("  Parametric mode: {}", if param_ok { "✓ PASS" } else { "✗ FAIL" });
    
    // Neural mode
    let encrypted = neural::encrypt(test_data, key);
    let decrypted = neural::decrypt(&encrypted, key);
    let neural_ok = test_data.to_vec() == decrypted;
    println!("  Neural mode:     {}", if neural_ok { "✓ PASS" } else { "✗ FAIL" });
    
    assert!(core_ok && param_ok && neural_ok, "Correctness test failed");
}

fn test_avalanche() {
    println!("\n【Avalanche Effect Tests】\n");
    
    let data: Vec<u8> = (0..1024).map(|i| (i % 256) as u8).collect();
    let mut data_flipped = data.clone();
    data_flipped[0] ^= 0x01; // Flip one bit
    
    let key = b"avalanche_test_key!";
    
    // Core mode
    let c1 = core::encrypt(&data, key);
    let c2 = core::encrypt(&data_flipped, key);
    let core_avalanche = compute_avalanche(&c1, &c2);
    println!("  Core mode:       {:.1}% (ideal: 50%)", core_avalanche * 100.0);
    
    // Parametric mode
    let c1 = parametric::encrypt(&data, key);
    let c2 = parametric::encrypt(&data_flipped, key);
    let param_avalanche = compute_avalanche(&c1, &c2);
    println!("  Parametric mode: {:.1}% (ideal: 50%)", param_avalanche * 100.0);
    
    // Neural mode
    let c1 = neural::encrypt(&data, key);
    let c2 = neural::encrypt(&data_flipped, key);
    let neural_avalanche = compute_avalanche(&c1, &c2);
    println!("  Neural mode:     {:.1}% (ideal: 50%)", neural_avalanche * 100.0);
    
    // Check all are in acceptable range (40-60%)
    assert!(core_avalanche > 0.40 && core_avalanche < 0.60, "Core avalanche out of range");
    assert!(param_avalanche > 0.40 && param_avalanche < 0.60, "Parametric avalanche out of range");
    assert!(neural_avalanche > 0.40 && neural_avalanche < 0.60, "Neural avalanche out of range");
}

fn test_key_sensitivity() {
    println!("\n【Key Sensitivity Tests】\n");
    
    let data = b"Key sensitivity test data for Yaksha cipher";
    let key1 = b"original_key_123";
    let key2 = b"original_key_124"; // One byte different
    
    // Core mode
    let c1 = core::encrypt(data, key1);
    let c2 = core::encrypt(data, key2);
    let core_sensitivity = compute_avalanche(&c1, &c2);
    println!("  Core mode:       {:.1}% (ideal: 50%)", core_sensitivity * 100.0);
    
    // Parametric mode
    let c1 = parametric::encrypt(data, key1);
    let c2 = parametric::encrypt(data, key2);
    let param_sensitivity = compute_avalanche(&c1, &c2);
    println!("  Parametric mode: {:.1}% (ideal: 50%)", param_sensitivity * 100.0);
    
    // Neural mode
    let c1 = neural::encrypt(data, key1);
    let c2 = neural::encrypt(data, key2);
    let neural_sensitivity = compute_avalanche(&c1, &c2);
    println!("  Neural mode:     {:.1}% (ideal: 50%)", neural_sensitivity * 100.0);
    
    // Check all are in acceptable range (40-60%)
    assert!(core_sensitivity > 0.40 && core_sensitivity < 0.60, "Core key sensitivity out of range");
    assert!(param_sensitivity > 0.40 && param_sensitivity < 0.60, "Parametric key sensitivity out of range");
    assert!(neural_sensitivity > 0.40 && neural_sensitivity < 0.60, "Neural key sensitivity out of range");
}

fn compute_avalanche(a: &[u8], b: &[u8]) -> f64 {
    let diff: u32 = a.iter().zip(b.iter())
        .map(|(&x, &y)| (x ^ y).count_ones())
        .sum();
    diff as f64 / (a.len() * 8) as f64
}
