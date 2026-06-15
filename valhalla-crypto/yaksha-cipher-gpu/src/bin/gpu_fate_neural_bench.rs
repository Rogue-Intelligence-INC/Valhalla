use std::time::Instant;

use yaksha_cipher::{fate_neural_decrypt, fate_neural_encrypt};
use yaksha_cipher_gpu::{fate_evolve_gpu::gpu_fate_neural_decrypt, fate_evolve_gpu::gpu_fate_neural_encrypt, GpuContext};

fn bytes_to_mb(bytes: usize) -> f64 {
    bytes as f64 / 1024.0 / 1024.0
}

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║       Yaksha GPU Encryption Capability Benchmark (Fate)      ║");
    println!("╚══════════════════════════════════════════════════════════════╝");

    let ctx = match GpuContext::new() {
        Some(c) => {
            println!("GPU: {}", c.info());
            c
        }
        None => {
            eprintln!("No GPU available.");
            return;
        }
    };

    let key = b"gpu_fate_bench_master_key_2026";
    let nonce = b"gpu_fate_bench_nonce";

    // sizes: include small and large enough to amortize GPU overhead
    let sizes: [usize; 7] = [4096, 16384, 65536, 262144, 1024 * 1024, 4 * 1024 * 1024, 16 * 1024 * 1024];

    println!(
        "\n  {:>10} │ {:>10} │ {:>10} │ {:>8} │ {:>10} │ {:>10} │ {:>8}",
        "Size", "CPU enc", "GPU enc", "enc x", "CPU dec", "GPU dec", "dec x"
    );
    println!("  ──────────┼────────────┼────────────┼─────────┼────────────┼────────────┼─────────");

    for &size in &sizes {
        let plain: Vec<u8> = (0..size).map(|i| (i % 251) as u8).collect();

        // fewer iters for large sizes to avoid very long runs
        let iterations: u32 = if size >= 4 * 1024 * 1024 { 3 } else { 6 };

        // Warmup (JIT / cache / first-command submission)
        let _ = fate_neural_encrypt(&plain, key, nonce);
        let _ = gpu_fate_neural_encrypt(&plain, key, nonce, &ctx);

        // -------------------------
        // Encrypt benchmark
        // -------------------------
        let start_cpu = Instant::now();
        let mut last_ct_cpu = Vec::new();
        let mut last_ctrl_cpu = Default::default();
        for _ in 0..iterations {
            let (ct, ctrl) = fate_neural_encrypt(&plain, key, nonce);
            last_ct_cpu = ct;
            last_ctrl_cpu = ctrl;
        }
        let cpu_enc_time = start_cpu.elapsed().as_secs_f64() / iterations as f64;
        let cpu_enc_tp = bytes_to_mb(size) / cpu_enc_time;

        let start_gpu = Instant::now();
        let mut last_ct_gpu = Vec::new();
        let mut last_ctrl_gpu = Default::default();
        for _ in 0..iterations {
            let (ct, ctrl) = gpu_fate_neural_encrypt(&plain, key, nonce, &ctx);
            last_ct_gpu = ct;
            last_ctrl_gpu = ctrl;
        }
        let gpu_enc_time = start_gpu.elapsed().as_secs_f64() / iterations as f64;
        let gpu_enc_tp = bytes_to_mb(size) / gpu_enc_time;

        let enc_speedup = gpu_enc_tp / cpu_enc_tp;

        // -------------------------
        // Decrypt benchmark
        // -------------------------
        let start_cpu_dec = Instant::now();
        for _ in 0..iterations {
            let _ = fate_neural_decrypt(&last_ct_cpu, key, nonce, &last_ctrl_cpu);
        }
        let cpu_dec_time = start_cpu_dec.elapsed().as_secs_f64() / iterations as f64;
        let cpu_dec_tp = bytes_to_mb(size) / cpu_dec_time;

        let start_gpu_dec = Instant::now();
        for _ in 0..iterations {
            let _ = gpu_fate_neural_decrypt(&last_ct_gpu, key, nonce, &last_ctrl_gpu, &ctx);
        }
        let gpu_dec_time = start_gpu_dec.elapsed().as_secs_f64() / iterations as f64;
        let gpu_dec_tp = bytes_to_mb(size) / gpu_dec_time;

        let dec_speedup = gpu_dec_tp / cpu_dec_tp;

        let size_str = if size >= 1024 * 1024 {
            format!("{} MB", size / 1024 / 1024)
        } else if size >= 1024 {
            format!("{} KB", size / 1024)
        } else {
            format!("{} B", size)
        };

        println!(
            "  {:>10} │ {:>10.1} │ {:>10.1} │ {:>7.2}x │ {:>10.1} │ {:>10.1} │ {:>7.2}x",
            size_str, cpu_enc_tp, gpu_enc_tp, enc_speedup, cpu_dec_tp, gpu_dec_tp, dec_speedup
        );
    }

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                       Benchmark done                         ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
}

