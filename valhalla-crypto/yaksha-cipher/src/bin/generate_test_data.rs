//! Generate encrypted data for dieharder / NIST STS / PractRand testing

use std::io::{self, Write};
use yaksha_cipher::{core, fate_neural_encrypt, parametric, neural};

const EVOLVE_CHUNK: usize = 65536; // 64 KB per chunk for evolve stream

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("core");
    let size_mb: usize = args
        .get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);
    let input_type = args.get(3).map(|s| s.as_str()).unwrap_or("random");

    let size = size_mb * 1024 * 1024;

    eprintln!("Generating {} MB of encrypted data", size_mb);
    eprintln!("  Mode: {}", mode);
    eprintln!("  Input: {}", input_type);

    let key = b"yaksha_test_key_for_dieharder_2026!";

    if mode == "evolve" {
        // Stream mode: many chunks, each with fate_neural_encrypt and unique nonce
        let mut written: usize = 0;
        let mut lcg: u64 = 0x5DEECE66Du64;
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        let mut chunk_idx: u64 = 0;
        while written < size {
            let chunk_len = EVOLVE_CHUNK.min(size - written);
            let plain_chunk: Vec<u8> = (0..chunk_len)
                .map(|_| {
                    lcg = lcg.wrapping_mul(0x5DEECE66Du64).wrapping_add(0xB);
                    (lcg >> 16) as u8
                })
                .collect();
            let nonce = chunk_idx.to_le_bytes();
            let (ct, _) = fate_neural_encrypt(&plain_chunk, key, &nonce);
            handle.write_all(&ct).expect("write");
            written += ct.len();
            chunk_idx += 1;
        }
        eprintln!("Encryption complete. Writing to stdout...");
        eprintln!("Done. {} bytes written.", written);
        return;
    }

    // Generate plaintext based on input type
    let plaintext: Vec<u8> = match input_type {
        "sequential" => (0..size).map(|i| (i % 256) as u8).collect(),
        "random" => {
            let mut lcg: u64 = 0x5DEECE66Du64;
            (0..size)
                .map(|_| {
                    lcg = lcg.wrapping_mul(0x5DEECE66Du64).wrapping_add(0xB);
                    (lcg >> 16) as u8
                })
                .collect()
        }
        "zeros" => vec![0u8; size],
        "text" => {
            let pattern = b"The quick brown fox jumps over the lazy dog. ";
            (0..size).map(|i| pattern[i % pattern.len()]).collect()
        }
        _ => {
            eprintln!("Unknown input type: {}. Use: sequential, random, zeros, text", input_type);
            std::process::exit(1);
        }
    };

    let ciphertext = match mode {
        "core" => core::encrypt(&plaintext, key),
        "parametric" => parametric::encrypt(&plaintext, key),
        "neural" => neural::encrypt(&plaintext, key),
        _ => {
            eprintln!("Unknown mode: {}. Use: core, parametric, neural, evolve", mode);
            std::process::exit(1);
        }
    };

    eprintln!("Encryption complete. Writing to stdout...");
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(&ciphertext).expect("Failed to write");
    eprintln!("Done. {} bytes written.", ciphertext.len());
}
