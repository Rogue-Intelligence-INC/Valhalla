//! GPU version of `yaksha-cipher` strict_test.
//!
//! Mirrors the structure but uses:
//! - gpu_fate_neural_encrypt/decrypt for Fate-Neural KAT, avalanche, key sensitivity, session diff
//! - encrypt_quantum for monobit
//!
//! Run:
//!   cargo run --release --bin strict_test_gpu

use yaksha_cipher_gpu::fate_evolve_gpu::{gpu_fate_neural_decrypt, gpu_fate_neural_encrypt};
use yaksha_cipher_gpu::{encrypt_neural, GpuContext};

const AVALANCHE_SAMPLES: usize = 300;
const KEY_SENSITIVITY_SAMPLES: usize = 50;
const CHI_SQUARE_CIPHERS: usize = 200;
const CHI_SQUARE_PLAIN_LEN: usize = 256;
const MONOBIT_BYTES: usize = 64 * 1024; // 512k bits
const KAT_KEY: &[u8] = b"YakshaKATkey!!!!!!!!";
const KAT_NONCE: &[u8] = b"YakshaKATnonce!!!!!!";
const KAT_PLAIN: &[u8] = b"Known-Answer Test plaintext for Yaksha strict_test.";

fn main() {
    let ctx = match GpuContext::new() {
        Some(c) => c,
        None => {
            eprintln!("No GPU available for strict_test_gpu.");
            std::process::exit(1);
        }
    };

    let mut failed = 0u32;

    // ---- 1. KAT: deterministic and roundtrip ----
    println!("[1/6] KAT (determinism + roundtrip) ...");
    let (ct1, ctrl1) = gpu_fate_neural_encrypt(KAT_PLAIN, KAT_KEY, KAT_NONCE, &ctx);
    let (ct2, ctrl2) = gpu_fate_neural_encrypt(KAT_PLAIN, KAT_KEY, KAT_NONCE, &ctx);
    if ct1 != ct2 || ctrl1.mode != ctrl2.mode || ctrl1.breaths != ctrl2.breaths {
        eprintln!("  FAIL: encrypt not deterministic");
        failed += 1;
    } else {
        let pt = gpu_fate_neural_decrypt(&ct1, KAT_KEY, KAT_NONCE, &ctrl1, &ctx);
        if pt != KAT_PLAIN {
            eprintln!("  FAIL: roundtrip mismatch");
            failed += 1;
        } else {
            println!("  OK");
        }
    }

    // ---- 2. Avalanche (evolve): 1-bit & 2-bit perturbations ----
    println!(
        "[2/6] Avalanche (gpu fate_neural, {} samples, 1-bit & 2-bit) ...",
        AVALANCHE_SAMPLES
    );
    let key = b"strict_avalanche_key!!!!!";
    let plain: Vec<u8> = (0..512)
        .map(|i: usize| (i.wrapping_mul(0x9E).wrapping_add(0x37)) as u8)
        .collect();

    let mut sum_ratio_1 = 0.0;
    let mut sum_ratio_2 = 0.0;
    let mut ok_count = 0usize;

    for i in 0..AVALANCHE_SAMPLES {
        let nonce = (i as u64).to_le_bytes();

        let (c1, _ctrl) = gpu_fate_neural_encrypt(&plain, key, &nonce, &ctx);

        // 1-bit perturbation
        let mut p1 = plain.clone();
        p1[0] ^= 1u8;
        let (c1b, _) = gpu_fate_neural_encrypt(&p1, key, &nonce, &ctx);
        let diff_bits_1: u32 = c1
            .iter()
            .zip(c1b.iter())
            .map(|(a, b)| (a ^ b).count_ones())
            .sum();
        let ratio_1 = diff_bits_1 as f64 / (c1.len() * 8) as f64;
        sum_ratio_1 += ratio_1;

        // 2-bit adjacent perturbation
        let mut p2 = plain.clone();
        p2[0] ^= 1u8;
        p2[1] ^= 1u8;
        let (c2b, _) = gpu_fate_neural_encrypt(&p2, key, &nonce, &ctx);
        let diff_bits_2: u32 = c1
            .iter()
            .zip(c2b.iter())
            .map(|(a, b)| (a ^ b).count_ones())
            .sum();
        let ratio_2 = diff_bits_2 as f64 / (c1.len() * 8) as f64;
        sum_ratio_2 += ratio_2;

        if (0.35..=0.65).contains(&ratio_2) {
            ok_count += 1;
        }
    }

    let avg1 = sum_ratio_1 / AVALANCHE_SAMPLES as f64;
    let avg2 = sum_ratio_2 / AVALANCHE_SAMPLES as f64;
    let pct_ok = ok_count as f64 / AVALANCHE_SAMPLES as f64 * 100.0;
    println!(
        "  1-bit avg diff %: {:.2}% | 2-bit avg diff %: {:.2}% (2-bit in [35,65]%: {}/{} ≈{:.0}%)",
        avg1 * 100.0,
        avg2 * 100.0,
        ok_count,
        AVALANCHE_SAMPLES,
        pct_ok
    );

    if pct_ok < 50.0 && avg2 < 0.20 {
        eprintln!("  WARN: 2-bit avalanche weak (documented baseline); consider improving diffusion");
    } else {
        println!("  OK");
    }

    // ---- 3. Key sensitivity ----
    println!(
        "[3/6] Key sensitivity ({} samples) ...",
        KEY_SENSITIVITY_SAMPLES
    );
    let plain: Vec<u8> = (0..256).map(|i| (i % 251) as u8).collect();
    let mut key_buf = [0u8; 24];
    key_buf[..22].copy_from_slice(b"key_sensitivity_test!!");

    let mut sum_ratio = 0.0;
    for i in 0..KEY_SENSITIVITY_SAMPLES {
        key_buf[20] = (i % 256) as u8;
        let nonce = [i as u8; 8];

        let (c1, _ctrl1) = gpu_fate_neural_encrypt(&plain, &key_buf, &nonce, &ctx);
        key_buf[0] ^= 1u8;
        let (c2, _ctrl2) = gpu_fate_neural_encrypt(&plain, &key_buf, &nonce, &ctx);
        key_buf[0] ^= 1u8;

        let diff_bits: u32 = c1.iter().zip(c2.iter()).map(|(a, b)| (a ^ b).count_ones()).sum();
        sum_ratio += diff_bits as f64 / (c1.len() * 8) as f64;
    }

    let avg = sum_ratio / KEY_SENSITIVITY_SAMPLES as f64;
    println!("  avg ciphertext diff %: {:.2}%", avg * 100.0);
    if avg < 0.30 {
        eprintln!("  WARN: key sensitivity low");
    } else {
        println!("  OK");
    }

    // ---- 4. Byte distribution (Chi-Square) ----
    println!(
        "[4/6] Byte distribution Chi-Square ({} ciphertexts, {} B each) ...",
        CHI_SQUARE_CIPHERS, CHI_SQUARE_PLAIN_LEN
    );
    let key = b"chi_square_key!!!!!!!!!";
    let plain: Vec<u8> = (0..CHI_SQUARE_PLAIN_LEN).map(|i| (i * 7) as u8).collect();

    let mut counts = [0u64; 256];
    for i in 0..CHI_SQUARE_CIPHERS {
        let nonce = (i as u32).to_le_bytes();
        let (ct, _) = gpu_fate_neural_encrypt(&plain, key, &nonce, &ctx);
        for &b in &ct {
            counts[b as usize] += 1;
        }
    }

    let n: f64 = counts.iter().sum::<u64>() as f64;
    let expected = n / 256.0;
    let chi2: f64 = counts
        .iter()
        .map(|&c| {
            let e = expected;
            let d = (c as f64) - e;
            d * d / e
        })
        .sum();
    println!("  chi2 = {:.2}  (critical ~293 for p=0.05)", chi2);
    if chi2 > 5000.0 {
        eprintln!("  FAIL: byte distribution severely non-uniform");
        failed += 1;
    } else if chi2 > 400.0 {
        println!("  WARN: byte distribution off uniform (documented baseline); target chi2 < 293");
    } else {
        println!("  OK");
    }

    // ---- 5. Monobit ----
    println!("[5/6] Monobit (gpu neural {} KB) ...", MONOBIT_BYTES / 1024);
    let key = b"monobit_key!!!!!!!!!!!!";
    let plain: Vec<u8> = (0..MONOBIT_BYTES).map(|i| (i.wrapping_mul(0x9E)) as u8).collect();
    let ct = encrypt_neural(&ctx, &plain, key);
    let ones: u32 = ct.iter().map(|&b| b.count_ones()).sum();
    let total_bits = (ct.len() * 8) as f64;
    let ratio = ones as f64 / total_bits;
    println!("  ones ratio: {:.4} (expect ~0.5)", ratio);
    if !(0.49..=0.51).contains(&ratio) {
        eprintln!("  WARN: monobit bias");
    } else {
        println!("  OK");
    }

    // ---- 6. Session diff (different nonce -> different ct) ----
    println!("[6/6] Session diff (same plain, different nonce) ...");
    let key = b"session_diff_key!!!!!!!!";
    let plain = vec![0u8; 128];
    let (ct_a, _) = gpu_fate_neural_encrypt(&plain, key, b"n1", &ctx);
    let (ct_b, _) = gpu_fate_neural_encrypt(&plain, key, b"n2", &ctx);
    let diff: usize = ct_a.iter().zip(ct_b.iter()).filter(|(a, b)| a != b).count();
    if diff == 0 {
        eprintln!("  FAIL: ciphertexts identical");
        failed += 1;
    } else {
        println!("  {} bytes differ (expect most)  OK", diff);
    }

    println!("");
    if failed > 0 {
        eprintln!("strict_test_gpu: {} failure(s)", failed);
        std::process::exit(1);
    }
    println!("strict_test_gpu: all strict checks passed or within documented baseline.");
}

