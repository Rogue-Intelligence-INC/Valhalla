//! Generate encrypted data from GPU version for dieharder testing

use std::io::{self, Write};
use yaksha_cipher_gpu::{GpuContext, dimension_gpu, quantum_gpu};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("parametric");
    let size_mb: usize = args.get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);
    
    let size = size_mb * 1024 * 1024;
    
    eprintln!("GPU Version - Generating {} MB (mode: {})", size_mb, mode);
    
    // Initialize GPU context
    eprintln!("Initializing GPU...");
    let ctx = match GpuContext::new() {
        Some(c) => {
            eprintln!("GPU: {:?}", c.adapter_info);
            c
        }
        None => {
            eprintln!("No GPU available, using CPU fallback");
            // Fallback to CPU version
            return run_cpu_fallback(mode, size);
        }
    };
    
    // Random input using LCG
    let mut lcg: u64 = 0x5DEECE66Du64;
    let plaintext: Vec<u8> = (0..size).map(|_| {
        lcg = lcg.wrapping_mul(0x5DEECE66Du64).wrapping_add(0xB);
        (lcg >> 16) as u8
    }).collect();
    
    let key = b"yaksha_test_key_for_dieharder_2026!";
    
    eprintln!("Encrypting...");
    let ciphertext = match mode {
        "parametric" => dimension_gpu::encrypt(&ctx, &plaintext, key),
        "neural" => quantum_gpu::encrypt(&ctx, &plaintext, key),
        _ => {
            eprintln!("Unknown mode: {}. Use: parametric, neural", mode);
            std::process::exit(1);
        }
    };
    
    eprintln!("Done. {} bytes.", ciphertext.len());
    
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(&ciphertext).expect("Failed to write");
}

fn run_cpu_fallback(mode: &str, size: usize) {
    use yaksha_cipher::{parametric, neural};
    
    let mut lcg: u64 = 0x5DEECE66Du64;
    let plaintext: Vec<u8> = (0..size).map(|_| {
        lcg = lcg.wrapping_mul(0x5DEECE66Du64).wrapping_add(0xB);
        (lcg >> 16) as u8
    }).collect();
    
    let key = b"yaksha_test_key_for_dieharder_2026!";
    
    let ciphertext = match mode {
        "parametric" => parametric::encrypt(&plaintext, key),
        "neural" => neural::encrypt(&plaintext, key),
        _ => {
            eprintln!("Unknown mode");
            std::process::exit(1);
        }
    };
    
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(&ciphertext).expect("Failed to write");
}
