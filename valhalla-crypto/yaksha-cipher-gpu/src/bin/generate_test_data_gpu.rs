//! Generate encrypted data for dieharder / NIST STS using Yaksha GPU backend.
//!
//! CLI mirrors `yaksha-cipher/src/bin/generate_test_data.rs`:
//!   cargo run --release --bin generate_test_data_gpu -- <mode> <size_mb> <input_type>
//!
//! Notes:
//! - GPU backend supports `parametric`, `neural`, and `evolve` fully.
//! - For `core`, this binary currently falls back to CPU (no GPU core implementation in this crate).
//! - Output is a raw binary stream to stdout (ciphertext bytes).

use std::io::{self, Write};

use yaksha_cipher_gpu::{
    encrypt_dimension, encrypt_quantum, fate_evolve_gpu::gpu_fate_neural_encrypt, GpuContext,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("core");
    let size_mb: usize = args
        .get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);
    let input_type = args.get(3).map(|s| s.as_str()).unwrap_or("random");

    let size = size_mb * 1024 * 1024;

    eprintln!("GPU Generating {} MB of encrypted data", size_mb);
    eprintln!("  Mode: {}", mode);
    eprintln!("  Input: {}", input_type);

    let key = b"yaksha_test_key_for_dieharder_2026!";

    // Core fallback is CPU-only.
    if mode == "core" {
        let plaintext = make_plaintext(size, input_type);
        let ciphertext = yaksha_cipher::core::encrypt(&plaintext, key);
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        handle.write_all(&ciphertext).expect("Failed to write");
        eprintln!("Done. {} bytes written.", ciphertext.len());
        return;
    }

    // Initialize GPU once for all other supported modes.
    let ctx = match GpuContext::new() {
        Some(c) => c,
        None => {
            eprintln!("No GPU available; falling back to CPU.");
            // Fallback for core/parametric/neural/evolve
            fallback_cpu(mode, size_mb, input_type);
        }
    };

    if mode == "evolve" {
        // Stream mode: many chunks, each with gpu_fate_neural_encrypt and unique nonce.
        const EVOLVE_CHUNK: usize = 65536; // 64 KB per chunk

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
            let (ct, _ctrl) = gpu_fate_neural_encrypt(&plain_chunk, key, &nonce, &ctx);
            handle.write_all(&ct).expect("write");

            written += ct.len();
            chunk_idx += 1;
        }

        eprintln!("Encryption complete. Writing to stdout...");
        eprintln!("Done. {} bytes written.", written);
        return;
    }

    // For parametric/neural: encrypt the whole plaintext in one go.
    let plaintext = make_plaintext(size, input_type);
    let ciphertext = match mode {
        "parametric" => encrypt_dimension(&ctx, &plaintext, key),
        "neural" => encrypt_quantum(&ctx, &plaintext, key),
        _ => {
            eprintln!("Unknown mode: {}. Use: core, parametric, neural, evolve", mode);
            std::process::exit(1);
        }
    };

    eprintln!("Done. {} bytes written.", ciphertext.len());
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(&ciphertext).expect("Failed to write");
}

fn fallback_cpu(mode: &str, size_mb: usize, input_type: &str) -> ! {
    let size = size_mb * 1024 * 1024;
    let key = b"yaksha_test_key_for_dieharder_2026!";

    if mode == "evolve" {
        // Use CPU evolve stream identical to yaksha-cipher's generator.
        const EVOLVE_CHUNK: usize = 65536;

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
            let (ct, _) = yaksha_cipher::fate_neural_encrypt(&plain_chunk, key, &nonce);
            handle.write_all(&ct).expect("write");
            written += ct.len();
            chunk_idx += 1;
        }
        std::process::exit(0);
    }

    let plaintext = make_plaintext(size, input_type);
    let ciphertext = match mode {
        "parametric" => yaksha_cipher::parametric::encrypt(&plaintext, key),
        "neural" => yaksha_cipher::neural::encrypt(&plaintext, key),
        _ => yaksha_cipher::core::encrypt(&plaintext, key),
    };
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(&ciphertext).expect("Failed to write");
    std::process::exit(0);
}

fn make_plaintext(size: usize, input_type: &str) -> Vec<u8> {
    match input_type {
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
            eprintln!(
                "Unknown input type: {}. Use: sequential, random, zeros, text",
                input_type
            );
            std::process::exit(1);
        }
    }
}

