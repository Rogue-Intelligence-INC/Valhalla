//! 测试量子扩散机制：B 版本（叠加态）和 C 版本（干涉效应）

use yaksha_cipher_gpu::{
    GpuContext,
    superposition_cascade::SuperpositionCascade,
    interference_cascade::InterferenceCascade,
};

fn test_diffusion<F>(name: &str, apply_fn: F)
where
    F: Fn(&[u8], &[u8], u32) -> Vec<u8>,
{
    println!("\n【{}】\n", name);
    
    let key = b"test_quantum_key_2026!";
    
    // 测试不同数据大小
    for &size in &[64usize, 256, 1024, 4096, 16384] {
        let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        let mut data_mod = data.clone();
        data_mod[0] ^= 0x01;  // 改变第一个字节的一位
        
        let enc1 = apply_fn(&data, key, 2);
        let enc2 = apply_fn(&data_mod, key, 2);
        
        let diff: u32 = enc1.iter().zip(enc2.iter())
            .map(|(&a, &b)| (a ^ b).count_ones())
            .sum();
        let diff_pct = diff as f64 / (enc1.len() * 8) as f64 * 100.0;
        
        let status = if diff_pct > 40.0 && diff_pct < 60.0 { "✓" } else { "✗" };
        println!("  {:>6} bytes: {:.1}% diffusion {}", size, diff_pct, status);
    }
    
    // 位置敏感性测试
    println!("\n  位置敏感性 (16KB 数据):");
    let size = 16384;
    let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
    
    for &pos in &[0, size/4, size/2, size-1] {
        let mut data_mod = data.clone();
        data_mod[pos] ^= 0x01;
        
        let enc1 = apply_fn(&data, key, 2);
        let enc2 = apply_fn(&data_mod, key, 2);
        
        let diff: u32 = enc1.iter().zip(enc2.iter())
            .map(|(&a, &b)| (a ^ b).count_ones())
            .sum();
        let diff_pct = diff as f64 / (enc1.len() * 8) as f64 * 100.0;
        
        let status = if diff_pct > 40.0 && diff_pct < 60.0 { "✓" } else { "✗" };
        println!("    Position {:>5}: {:.1}% {}", pos, diff_pct, status);
    }
    
    // 轮次效果测试
    println!("\n  轮次效果 (16KB 数据, 位置 0):");
    for rounds in [1, 2, 3, 4] {
        let mut data_mod = data.clone();
        data_mod[0] ^= 0x01;
        
        let enc1 = apply_fn(&data, key, rounds);
        let enc2 = apply_fn(&data_mod, key, rounds);
        
        let diff: u32 = enc1.iter().zip(enc2.iter())
            .map(|(&a, &b)| (a ^ b).count_ones())
            .sum();
        let diff_pct = diff as f64 / (enc1.len() * 8) as f64 * 100.0;
        
        println!("    {} round(s): {:.1}%", rounds, diff_pct);
    }
    
    // 密钥敏感性
    println!("\n  密钥敏感性:");
    let key2 = b"test_quantum_key_2026?";
    let enc1 = apply_fn(&data, key, 2);
    let enc2 = apply_fn(&data, key2, 2);
    
    let diff: u32 = enc1.iter().zip(enc2.iter())
        .map(|(&a, &b)| (a ^ b).count_ones())
        .sum();
    let diff_pct = diff as f64 / (enc1.len() * 8) as f64 * 100.0;
    let status = if diff_pct > 40.0 { "✓" } else { "✗" };
    println!("    Key diff (1 byte): {:.1}% {}", diff_pct, status);
}

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║       Quantum Diffusion Mechanism Test                       ║");
    println!("║       B: Superposition  |  C: Interference                   ║");
    println!("╚══════════════════════════════════════════════════════════════╝");

    let ctx = match GpuContext::new() {
        Some(c) => {
            println!("\nGPU: {}\n", c.info());
            c
        }
        None => {
            println!("No GPU available!");
            return;
        }
    };

    // ========================================================================
    // B 版本：叠加态坍缩
    // ========================================================================
    let superposition = SuperpositionCascade::new(&ctx);
    test_diffusion("B: Superposition (叠加态坍缩)", |data, key, rounds| {
        superposition.apply(&ctx, data, key, rounds)
    });

    // ========================================================================
    // C 版本：干涉效应
    // ========================================================================
    let interference = InterferenceCascade::new(&ctx);
    test_diffusion("C: Interference (干涉效应)", |data, key, rounds| {
        interference.apply(&ctx, data, key, rounds)
    });

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                    Test Complete                             ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
}
