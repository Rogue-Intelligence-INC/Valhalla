//! 测试前缀泄露的影响范围
//! 
//! 问题：相同前缀的数据，密文前缀会相同多久？
//! 随着数据变大，cascade 传播会让差异扩散吗？

use val_crypt::{vector, entropy};

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║          前缀泄露深度分析 / Prefix Leakage Depth             ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let key = b"TestSecretKey123";
    
    // 测试不同前缀长度
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Test 1: 固定差异位置，改变数据大小");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    // 差异在第一个字节
    println!("\n差异位置: 第1字节\n");
    println!("  数据大小    密文相同前缀    差异开始传播位置");
    println!("  ─────────────────────────────────────────────");
    
    for &size in &[32, 64, 128, 256, 512, 1024, 4096, 16384] {
        let mut p1: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        let mut p2 = p1.clone();
        p2[0] ^= 0x01; // 第一字节差1 bit
        
        let c1 = vector::encrypt(&p1, key);
        let c2 = vector::encrypt(&p2, key);
        
        let same_prefix = c1.iter().zip(c2.iter())
            .take_while(|(a, b)| a == b)
            .count();
        
        // 找到第一个不同的位置
        let first_diff = c1.iter().zip(c2.iter())
            .position(|(a, b)| a != b)
            .unwrap_or(c1.len());
        
        println!("  {:>6} B    {:>6} B ({:>5.1}%)    位置 {}", 
                 size, same_prefix, 
                 same_prefix as f64 / c1.len() as f64 * 100.0,
                 first_diff);
    }
    
    // 差异在中间位置
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Test 2: 差异在数据中间位置");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let size = 1024;
    println!("\n数据大小: {} 字节\n", size);
    println!("  差异位置    密文相同前缀    前缀比例    差异扩散");
    println!("  ─────────────────────────────────────────────────────");
    
    for diff_pos in [0, 100, 256, 500, 512, 800, 900, 1000, 1023] {
        let mut p1: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        let mut p2 = p1.clone();
        p2[diff_pos] ^= 0x01;
        
        let c1 = vector::encrypt(&p1, key);
        let c2 = vector::encrypt(&p2, key);
        
        let same_prefix = c1.iter().zip(c2.iter())
            .take_while(|(a, b)| a == b)
            .count();
        
        // 统计差异字节数
        let diff_bytes: usize = c1.iter().zip(c2.iter())
            .filter(|(a, b)| a != b)
            .count();
        
        // 雪崩效应（从差异位置开始）
        let diff_bits: u32 = c1.iter().zip(c2.iter())
            .skip(diff_pos)
            .map(|(a, b)| (a ^ b).count_ones())
            .sum();
        let remaining_bytes = c1.len() - diff_pos;
        let avalanche = diff_bits as f64 / (remaining_bytes * 8) as f64;
        
        println!("  {:>6}      {:>6} B        {:>5.1}%      {:.1}% 雪崩 ({} bytes differ)", 
                 diff_pos, same_prefix, 
                 same_prefix as f64 / c1.len() as f64 * 100.0,
                 avalanche * 100.0,
                 diff_bytes);
    }
    
    // 测试实际场景：相似文件
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Test 3: 实际场景 - 相似文件（如版本差异）");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    // 模拟两个版本的配置文件
    let file1 = b"config_version=1.0.0\nserver_host=localhost\nserver_port=8080\ndebug_mode=false\nlog_level=info\nmax_connections=100\ntimeout=30\n";
    let file2 = b"config_version=1.0.1\nserver_host=localhost\nserver_port=8080\ndebug_mode=false\nlog_level=info\nmax_connections=100\ntimeout=30\n";
    
    let c1 = vector::encrypt(file1, key);
    let c2 = vector::encrypt(file2, key);
    
    let same_prefix = c1.iter().zip(c2.iter())
        .take_while(|(a, b)| a == b)
        .count();
    
    println!("\n  文件1: {:?}...", String::from_utf8_lossy(&file1[..40]));
    println!("  文件2: {:?}...", String::from_utf8_lossy(&file2[..40]));
    println!("\n  文件大小: {} vs {} 字节", file1.len(), file2.len());
    println!("  密文相同前缀: {} 字节 ({:.1}%)", 
             same_prefix, same_prefix as f64 / c1.len() as f64 * 100.0);
    
    // 差异开始的位置
    let diff_start_plaintext = file1.iter().zip(file2.iter())
        .position(|(a, b)| a != b)
        .unwrap_or(0);
    println!("  明文差异开始: 位置 {} (\"{}\")", 
             diff_start_plaintext,
             String::from_utf8_lossy(&file1[diff_start_plaintext..diff_start_plaintext+5]));
    
    if same_prefix > 0 {
        println!("\n  ⚠️  攻击者可以推断：这两个密文对应的明文有相同的前 {} 字节", same_prefix);
    }
    
    // Vector vs Entropy 对比
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Test 4: Vector vs Entropy 前缀泄露对比");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let size = 1024;
    println!("\n数据大小: {} 字节, 差异在位置 0\n", size);
    
    let mut p1: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
    let mut p2 = p1.clone();
    p2[0] ^= 0x01;
    
    let cv1 = vector::encrypt(&p1, key);
    let cv2 = vector::encrypt(&p2, key);
    let vec_same = cv1.iter().zip(cv2.iter()).take_while(|(a, b)| a == b).count();
    
    let ce1 = entropy::encrypt(&p1, key);
    let ce2 = entropy::encrypt(&p2, key);
    let ent_same = ce1.iter().zip(ce2.iter()).take_while(|(a, b)| a == b).count();
    
    println!("  模式        密文相同前缀    比例");
    println!("  ─────────────────────────────────");
    println!("  Vector      {:>6} B        {:.1}%", vec_same, vec_same as f64 / cv1.len() as f64 * 100.0);
    println!("  Entropy     {:>6} B        {:.1}%", ent_same, ent_same as f64 / ce1.len() as f64 * 100.0);
    
    // 关键结论
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("结论 / Conclusion");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("  1. 当差异在数据开头时，密文从第1字节就开始不同");
    println!("  2. 当差异在数据中间时，密文前缀与明文前缀直接对应");
    println!("  3. Cascade 只向前传播，不向后传播");
    println!("  4. 这是 ECB-like 的前缀泄露，但不是块级别的");
    println!();
}
