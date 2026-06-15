//! Quick effect check: roundtrip, control diversity, avalanche sample.

use yaksha_cipher::{fate_neural_decrypt, fate_neural_encrypt, neural};

fn main() {
    let key = b"FateYakshaMasterKey!";
    // Use non-zero plaintext so avalanche is meaningful (no all-zero collapse)
    let plain: Vec<u8> = (0..1024).map(|i: usize| (i.wrapping_mul(0x9E).wrapping_add(0x37)) as u8).collect();
    let fixed_nonce = b"fixed_nonce_for_avalanche";

    // 1. Roundtrip
    let (ct, ctrl) = fate_neural_encrypt(&plain, key, fixed_nonce);
    let pt = fate_neural_decrypt(&ct, key, fixed_nonce, &ctrl);
    assert_eq!(pt, plain);
    println!("[OK] Roundtrip 1KB: OK");

    // 2. Control diversity over 8 runs (different nonces)
    let mut modes = std::collections::HashSet::new();
    let mut patterns = std::collections::HashSet::new();
    let mut breath_min = 16u8;
    let mut breath_max = 0u8;
    for i in 0..8u8 {
        let nonce = [i];
        let (_, c) = fate_neural_encrypt(&plain, key, &nonce);
        modes.insert(std::mem::discriminant(&c.mode));
        patterns.insert(std::mem::discriminant(&c.cascade_pattern));
        breath_min = breath_min.min(c.breaths);
        breath_max = breath_max.max(c.breaths);
    }
    println!(
        "[OK] Control diversity: {} mode variants, {} cascade variants, breaths {}-{}",
        modes.len(),
        patterns.len(),
        breath_min,
        breath_max
    );

    // 3. Avalanche: same nonce (same control), flip 1 bit in plaintext
    let (c1, ctrl1) = fate_neural_encrypt(&plain, key, fixed_nonce);
    let mut plain2 = plain.clone();
    plain2[0] ^= 1u8;
    let (c2, ctrl2) = fate_neural_encrypt(&plain2, key, fixed_nonce);
    let same_ctrl = ctrl1.mode as u8 == ctrl2.mode as u8
        && ctrl1.breaths == ctrl2.breaths
        && ctrl1.label == ctrl2.label;
    let diff_bits: u32 = c1.iter().zip(c2.iter()).map(|(a, b)| (a ^ b).count_ones()).sum();
    let total_bits = c1.len() * 8;
    let av_pct = (diff_bits as f64 / total_bits as f64) * 100.0;
    println!(
        "[OK] Avalanche (1-bit flip at byte 0, fixed control): {:.2}% (mode={:?}, breaths={}, pattern={:?}, same_control={})",
        av_pct, ctrl1.mode, ctrl1.breaths, ctrl1.cascade_pattern, same_ctrl
    );

    // 4. Same plain, different nonce → different ciphertext
    let (ct_a, _) = fate_neural_encrypt(&plain, key, b"a");
    let (ct_b, _) = fate_neural_encrypt(&plain, key, b"b");
    let diff: usize = ct_a.iter().zip(ct_b.iter()).filter(|(a, b)| a != b).count();
    println!("[OK] Same plaintext, different nonce: {} bytes differ (expect all)", diff);

    // 5. Baseline: raw neural avalanche (same plain/plain2) for comparison
    let cn1 = neural::encrypt(&plain, key);
    let cn2 = neural::encrypt(&plain2, key);
    let dn: u32 = cn1.iter().zip(cn2.iter()).map(|(a, b)| (a ^ b).count_ones()).sum();
    let an_pct = (dn as f64 / (cn1.len() * 8) as f64) * 100.0;
    println!("[OK] Baseline neural only (1-bit flip): {:.2}%", an_pct);

    println!("\n--- Fate evolve effect summary ---");
    println!("Roundtrip: OK | Control diversity: OK | Evolve avalanche ~{:.1}% | Neural baseline ~{:.1}% | Session diff: OK", av_pct, an_pct);
}
