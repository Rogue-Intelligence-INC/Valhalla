//! Timing heuristic across key variations (side-channel signal check).
//!
//! This is NOT a formal constant-time proof. It is intended as a practical
//! early-warning test: if seal/open latency changes significantly with key
//! bytes for fixed plaintext/nonce/aad, that can indicate data-dependent paths.

use std::hint::black_box;
use std::time::Instant;

use yaksha_cipher::aead::YakshaAead;

fn main() {
    // Usage:
    //   yaksha_aead_timing_keys [plaintext_kb] [seconds] [sweep]
    // Defaults:
    //   plaintext_kb=64, seconds=0.25, sweep=64 (key[0] from 0..63)
    let args: Vec<String> = std::env::args().collect();
    let plaintext_kb: usize = args
        .get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(64);
    let seconds: f64 = args
        .get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.25);
    let sweep: usize = args
        .get(3)
        .and_then(|s| s.parse().ok())
        .unwrap_or(64);

    let aad = vec![0xA5u8; 32];
    let nonce = b"yaksha_nonce_12b";

    let plaintext_len = plaintext_kb * 1024;
    let plaintext: Vec<u8> = (0..plaintext_len).map(|i| (i % 251) as u8).collect();

    let mut base_key = [0u8; 32];
    base_key.copy_from_slice(b"yaksha_aead_master_key_32bytes!!");
    // Note: above copy should fit exactly; keep as a safe slice fallback.

    println!("key0,seal_ns_op,open_ns_op");

    for key0 in 0..sweep {
        let mut key = base_key;
        key[0] = (key0 & 0xFF) as u8;

        // Precompute seal outputs so open timing isolates verification+decrypt.
        let (ciphertext, tag) = YakshaAead::seal(&key, nonce, &plaintext, &aad);
        let ciphertext = black_box(ciphertext);
        let tag = black_box(tag);

        // Warmup (allocations + any caches).
        let _ = YakshaAead::seal(&key, nonce, &plaintext, &aad);
        let _ = YakshaAead::open(&key, nonce, &ciphertext, &aad, &tag);

        let (seal_ns_op, open_ns_op) = bench_key(&key, nonce, &plaintext, &aad, &ciphertext, &tag, seconds);

        println!("{},{:.0},{:.0}", key0, seal_ns_op, open_ns_op);
    }
}

fn bench_key(
    key: &[u8; 32],
    nonce: &[u8],
    plaintext: &[u8],
    aad: &[u8],
    ciphertext: &[u8],
    tag: &[u8],
    target_seconds: f64,
) -> (f64, f64) {
    // Benchmark seal.
    let mut ops_seal: u64 = 0;
    let mut checksum_seal: u64 = 0;
    let start_seal = Instant::now();
    while start_seal.elapsed().as_secs_f64() < target_seconds {
        let (ct, tg) = YakshaAead::seal(key, nonce, plaintext, aad);
        checksum_seal ^= ct.first().copied().unwrap_or(0) as u64;
        checksum_seal ^= tg.first().copied().unwrap_or(0) as u64;
        black_box(ct);
        black_box(tg);
        ops_seal += 1;
    }
    let seal_secs = start_seal.elapsed().as_secs_f64();
    let seal_ns_op = if ops_seal == 0 {
        0.0
    } else {
        seal_secs * 1e9 / ops_seal as f64
    };
    black_box(checksum_seal);

    // Benchmark open.
    let mut ops_open: u64 = 0;
    let mut checksum_open: u64 = 0;
    let start_open = Instant::now();
    while start_open.elapsed().as_secs_f64() < target_seconds {
        let pt = YakshaAead::open(key, nonce, ciphertext, aad, tag)
            .expect("open should pass for precomputed ciphertext/tag");
        checksum_open ^= pt.first().copied().unwrap_or(0) as u64;
        black_box(pt);
        ops_open += 1;
    }
    let open_secs = start_open.elapsed().as_secs_f64();
    let open_ns_op = if ops_open == 0 {
        0.0
    } else {
        open_secs * 1e9 / ops_open as f64
    };
    black_box(checksum_open);

    (seal_ns_op, open_ns_op)
}

