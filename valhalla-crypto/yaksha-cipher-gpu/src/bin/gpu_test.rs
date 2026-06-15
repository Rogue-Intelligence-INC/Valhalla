//! GPU 版本正确性和雪崩测试

use yaksha_cipher_gpu::{GpuContext, encrypt_dimension, decrypt_dimension, encrypt_quantum, decrypt_quantum};

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║           Val-Crypt GPU 正确性测试                           ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // 初始化 GPU
    let ctx = match GpuContext::new() {
        Some(ctx) => {
            println!("GPU: {}\n", ctx.info());
            ctx
        }
        None => {
            println!("✗ 无法初始化 GPU");
            return;
        }
    };

    let key = b"TestKey12345678!";
    let sizes = [4096, 16384, 65536, 262144];

    // ========================================================================
    // 正确性测试
    // ========================================================================
    println!("【正确性测试】\n");

    for &size in &sizes {
        let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        
        // GPU 维度版
        let enc_dim_gpu = encrypt_dimension(&ctx, &data, key);
        let dec_dim_gpu = decrypt_dimension(&ctx, &enc_dim_gpu, key);

        // GPU 量子版
        let enc_q_gpu = encrypt_quantum(&ctx, &data, key);
        let dec_q_gpu = decrypt_quantum(&ctx, &enc_q_gpu, key);

        let size_str = if size >= 1024 * 1024 {
            format!("{} MB", size / 1024 / 1024)
        } else {
            format!("{} KB", size / 1024)
        };

        // roundtrip correctness
        let dim_rt_ok = dec_dim_gpu == data;
        let q_rt_ok = dec_q_gpu == data;

        println!(
            "  {:>10}: dimension roundtrip {} | quantum roundtrip {}",
            size_str,
            if dim_rt_ok { "✓" } else { "✗" },
            if q_rt_ok { "✓" } else { "✗" }
        );
    }

    // ========================================================================
    // 雪崩效应测试
    // ========================================================================
    println!("\n【雪崩效应测试】\n");

    let size = 16384;
    let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
    let mut data_mod = data.clone();
    data_mod[0] ^= 0x01;

    // 维度版 GPU
    let c1_dim = encrypt_dimension(&ctx, &data, key);
    let c2_dim = encrypt_dimension(&ctx, &data_mod, key);
    let diff_dim: u32 = c1_dim.iter().zip(c2_dim.iter())
        .map(|(&a, &b)| (a ^ b).count_ones())
        .sum();
    let avalanche_dim = diff_dim as f64 / (c1_dim.len() * 8) as f64;

    // 量子版 GPU
    let c1_q = encrypt_quantum(&ctx, &data, key);
    let c2_q = encrypt_quantum(&ctx, &data_mod, key);
    let diff_bytes_q = c1_q
        .iter()
        .zip(c2_q.iter())
        .filter(|(a, b)| a != b)
        .count();
    let diff_q: u32 = c1_q.iter().zip(c2_q.iter())
        .map(|(&a, &b)| (a ^ b).count_ones())
        .sum();
    let avalanche_q = diff_q as f64 / (c1_q.len() * 8) as f64;

    println!("  维度版 GPU: {:.1}% (理想 50%)", avalanche_dim * 100.0);
    println!("  量子版 GPU: {:.1}% (理想 50%)", avalanche_q * 100.0);
    println!("  quantum ciphertext diff bytes: {}", diff_bytes_q);

    // ========================================================================
    // 分块雪崩分析
    // ========================================================================
    println!("\n【分块雪崩分析】\n");

    let block_size = size / 4;
    
    println!("  维度版 GPU:");
    for i in 0..4 {
        let start = i * block_size;
        let end = start + block_size;
        let diff: u32 = c1_dim[start..end].iter().zip(c2_dim[start..end].iter())
            .map(|(&a, &b)| (a ^ b).count_ones())
            .sum();
        let av = diff as f64 / (block_size * 8) as f64;
        let bar = "█".repeat((av * 50.0) as usize);
        println!("    Block {}: {:>5.1}% {}", i, av * 100.0, bar);
    }

    println!("\n  量子版 GPU:");
    for i in 0..4 {
        let start = i * block_size;
        let end = start + block_size;
        let diff: u32 = c1_q[start..end].iter().zip(c2_q[start..end].iter())
            .map(|(&a, &b)| (a ^ b).count_ones())
            .sum();
        let av = diff as f64 / (block_size * 8) as f64;
        let bar = "█".repeat((av * 50.0) as usize);
        println!("    Block {}: {:>5.1}% {}", i, av * 100.0, bar);
    }

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                        测试完成                              ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
}
