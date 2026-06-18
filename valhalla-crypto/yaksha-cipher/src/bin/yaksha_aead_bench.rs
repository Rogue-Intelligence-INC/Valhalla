//! Yaksha Cipher AEAD benchmark (seal/open throughput).

use std::hint::black_box;
use std::time::Instant;

use yaksha_cipher::aead::YakshaAead;
use yaksha_cipher::config;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║          Yaksha Cipher AEAD Benchmark (seal/open)          ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!(
        "  profile={:?} mac={:?} ocean_rounds={}",
        config::profile(),
        config::mac_mode(),
        config::ocean_mix_rounds()
    );
    println!();

    let key = b"yaksha_aead_master_key_32bytes!!";
    let nonce = b"yaksha_nonce_12b";
    let aad = vec![0xA5u8; 32];

    let sizes: &[(usize, &str)] = &[
        (4 * 1024, "4 KB"),
        (64 * 1024, "64 KB"),
        (256 * 1024, "256 KB"),
        (1024 * 1024, "1 MB"),
        (4 * 1024 * 1024, "4 MB"),
        (16 * 1024 * 1024, "16 MB"),
    ];

    println!("      Size │   seal MB/s │   seal ns/op │   open MB/s │   open ns/op");
    println!("  ─────────┼─────────────┼─────────────┼─────────────┼─────────────");

    for &(size, _) in sizes {
        let plaintext = make_plaintext(size);
        let _ = YakshaAead::seal(key, nonce, &plaintext, &aad);
    }

    for &(size, label) in sizes {
        let plaintext = make_plaintext(size);
        let (ciphertext0, tag0) = YakshaAead::seal(key, nonce, &plaintext, &aad);
        let ciphertext0 = black_box(ciphertext0);
        let tag0 = black_box(tag0);
        let (seal_mb_s, seal_ns_op) = bench_seal(key, nonce, &plaintext, &aad);
        let (open_mb_s, open_ns_op) =
            bench_open(key, nonce, &ciphertext0, &plaintext, &aad, &tag0);
        println!(
            "  {:>8} │ {:>11.0} │ {:>11.0} │ {:>11.0} │ {:>11.0}",
            label, seal_mb_s, seal_ns_op, open_mb_s, open_ns_op
        );
    }

    println!("\nAEAD benchmark complete.");
}

fn make_plaintext(size: usize) -> Vec<u8> {
    (0..size).map(|i| (i % 251) as u8).collect()
}

fn bench_seal(key: &[u8], nonce: &[u8], plaintext: &[u8], aad: &[u8]) -> (f64, f64) {
    let target_seconds = 0.35_f64;
    let max_ops: u64 = 50_000;
    let mut ops: u64 = 0;
    let mut checksum: u64 = 0;
    let start = Instant::now();
    while start.elapsed().as_secs_f64() < target_seconds && ops < max_ops {
        let (ct, tag) = YakshaAead::seal(key, nonce, plaintext, aad);
        checksum ^= ct.first().copied().unwrap_or(0) as u64;
        checksum ^= tag.first().copied().unwrap_or(0) as u64;
        black_box(ct);
        black_box(tag);
        ops += 1;
    }
    let total_secs = start.elapsed().as_secs_f64();
    let bytes = plaintext.len() as f64;
    let mb_s = bytes * ops as f64 / total_secs / (1024.0 * 1024.0);
    let ns_op = if ops == 0 { 0.0 } else { total_secs * 1e9 / ops as f64 };
    black_box(checksum);
    (mb_s, ns_op)
}

fn bench_open(
    key: &[u8],
    nonce: &[u8],
    ciphertext: &[u8],
    expected_plaintext: &[u8],
    aad: &[u8],
    tag: &[u8],
) -> (f64, f64) {
    let target_seconds = 0.35_f64;
    let max_ops: u64 = 50_000;
    let mut ops: u64 = 0;
    let mut checksum: u64 = 0;
    let start = Instant::now();
    while start.elapsed().as_secs_f64() < target_seconds && ops < max_ops {
        let pt = YakshaAead::open(key, nonce, ciphertext, aad, tag)
            .expect("open: authentication should pass in benchmark");
        checksum ^= pt.first().copied().unwrap_or(0) as u64;
        black_box(&pt);
        debug_assert_eq!(pt.as_slice(), expected_plaintext);
        ops += 1;
    }
    let total_secs = start.elapsed().as_secs_f64();
    let bytes = expected_plaintext.len() as f64;
    let mb_s = bytes * ops as f64 / total_secs / (1024.0 * 1024.0);
    let ns_op = if ops == 0 { 0.0 } else { total_secs * 1e9 / ops as f64 };
    black_box(checksum);
    (mb_s, ns_op)
}
