//! Yaksha Cipher performance benchmark

use yaksha_cipher::{core, parametric, neural};
use std::time::Instant;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║           Yaksha Cipher Performance Benchmark                ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let sizes = [
        (4 * 1024, "4 KB"),
        (64 * 1024, "64 KB"),
        (256 * 1024, "256 KB"),
        (1024 * 1024, "1 MB"),
        (4 * 1024 * 1024, "4 MB"),
        (16 * 1024 * 1024, "16 MB"),
    ];
    
    let key = b"benchmark_key_1234567890!";
    
    println!("      Size │     Core │  Parametric │    Neural");
    println!("  ─────────┼──────────┼─────────────┼──────────");
    
    for (size, label) in sizes.iter() {
        let data: Vec<u8> = (0..*size).map(|i| (i % 256) as u8).collect();
        
        // Warmup
        let _ = core::encrypt(&data, key);
        let _ = parametric::encrypt(&data, key);
        let _ = neural::encrypt(&data, key);
        
        // Core benchmark
        let start = Instant::now();
        let iterations = 3;
        for _ in 0..iterations {
            let _ = core::encrypt(&data, key);
        }
        let core_time = start.elapsed().as_secs_f64() / iterations as f64;
        let core_throughput = (*size as f64) / core_time / (1024.0 * 1024.0);
        
        // Parametric benchmark
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = parametric::encrypt(&data, key);
        }
        let param_time = start.elapsed().as_secs_f64() / iterations as f64;
        let param_throughput = (*size as f64) / param_time / (1024.0 * 1024.0);
        
        // Neural benchmark
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = neural::encrypt(&data, key);
        }
        let neural_time = start.elapsed().as_secs_f64() / iterations as f64;
        let neural_throughput = (*size as f64) / neural_time / (1024.0 * 1024.0);
        
        println!("  {:>8} │ {:>6.0} MB/s │ {:>7.0} MB/s │ {:>6.0} MB/s", 
            label, core_throughput, param_throughput, neural_throughput);
    }
    
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                    Benchmark Complete                        ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
}
