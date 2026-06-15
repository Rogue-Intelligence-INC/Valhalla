//! GPU vs CPU throughput (ocean XOR offload vs pure CPU shell).

use yaksha_cipher_gpu::{GpuContext, encrypt_dimension, encrypt_quantum};
use std::time::Instant;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║           Val-Crypt GPU 性能测试                             ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // 初始化 GPU
    print!("初始化 GPU...");
    let ctx = match GpuContext::new() {
        Some(ctx) => {
            println!(" ✓");
            println!("  GPU: {}\n", ctx.info());
            ctx
        }
        None => {
            println!(" ✗ 无法初始化 GPU");
            return;
        }
    };

    let key = b"TestKey12345678!";
    let sizes = [4096, 16384, 65536, 262144, 1024*1024, 4*1024*1024, 16*1024*1024];

    // ========================================================================
    // 维度版对比
    // ========================================================================
    println!("【维度加密 (Dimension)】\n");
    println!("  {:>10} │ {:>12} │ {:>12} │ {:>10}", "Size", "CPU (MB/s)", "GPU (MB/s)", "加速比");
    println!("  ───────────┼──────────────┼──────────────┼────────────");

    for &size in &sizes {
        let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        
        // CPU 测试
        let iterations = if size > 1024*1024 { 3 } else { 10 };
        
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = yaksha_cipher::parametric::encrypt(&data, key);
        }
        let cpu_time = start.elapsed().as_secs_f64() / iterations as f64;
        let cpu_throughput = (size as f64 / 1024.0 / 1024.0) / cpu_time;

        // GPU 测试
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = encrypt_dimension(&ctx, &data, key);
        }
        let gpu_time = start.elapsed().as_secs_f64() / iterations as f64;
        let gpu_throughput = (size as f64 / 1024.0 / 1024.0) / gpu_time;

        let speedup = gpu_throughput / cpu_throughput;
        let size_str = if size >= 1024*1024 {
            format!("{} MB", size / 1024 / 1024)
        } else {
            format!("{} KB", size / 1024)
        };

        println!("  {:>10} │ {:>12.1} │ {:>12.1} │ {:>10.2}x", 
            size_str, cpu_throughput, gpu_throughput, speedup);
    }

    // ========================================================================
    // 量子版对比
    // ========================================================================
    println!("\n【量子模拟 (Quantum)】\n");
    println!("  {:>10} │ {:>12} │ {:>12} │ {:>10}", "Size", "CPU (MB/s)", "GPU (MB/s)", "加速比");
    println!("  ───────────┼──────────────┼──────────────┼────────────");

    for &size in &sizes {
        let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        
        let iterations = if size > 1024*1024 { 3 } else { 10 };
        
        // CPU 测试
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = yaksha_cipher::neural::encrypt(&data, key);
        }
        let cpu_time = start.elapsed().as_secs_f64() / iterations as f64;
        let cpu_throughput = (size as f64 / 1024.0 / 1024.0) / cpu_time;

        // GPU 测试
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = encrypt_quantum(&ctx, &data, key);
        }
        let gpu_time = start.elapsed().as_secs_f64() / iterations as f64;
        let gpu_throughput = (size as f64 / 1024.0 / 1024.0) / gpu_time;

        let speedup = gpu_throughput / cpu_throughput;
        let size_str = if size >= 1024*1024 {
            format!("{} MB", size / 1024 / 1024)
        } else {
            format!("{} KB", size / 1024)
        };

        println!("  {:>10} │ {:>12.1} │ {:>12.1} │ {:>10.2}x", 
            size_str, cpu_throughput, gpu_throughput, speedup);
    }

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                        测试完成                              ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
}
