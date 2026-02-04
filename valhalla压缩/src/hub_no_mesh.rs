//! Hub 全系统测试 (禁用膜系统)
//! Fate 控制路由, 2周期 vs 3周期 横向纵向对比

use std::fs;
use std::process::Command;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════════════════════════════════╗");
    println!("║          Valhalla Hub (禁用膜系统) - Fate 控制 2/3 周期 横向纵向对比                     ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════════════════╝");
    println!();
    
    let test_files = vec![
        ("/tmp/raw_text.txt", "文本"),
        ("/tmp/test_image.bmp", "BMP"),
        ("/tmp/test_audio.wav", "WAV"),
        ("/tmp/test_data.json", "JSON"),
        ("/tmp/repeat.bin", "重复"),
        ("/tmp/random.bin", "随机"),
    ];
    
    // 存储结果: [文件][周期] = (压缩率, 质量)
    // 周期: 0=1周期, 1=2周期, 2=3周期
    let mut hub_results: Vec<[(f64, f64); 3]> = vec![[(0.0, 0.0); 3]; test_files.len()];
    let mut mainstream_results: Vec<f64> = vec![0.0; test_files.len()];
    
    for (fi, (file_path, file_name)) in test_files.iter().enumerate() {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📁 {} ({})", file_name, file_path);
        
        let data = match fs::read(file_path) {
            Ok(d) => d,
            Err(_) => {
                println!("  ⚠️ 文件不存在");
                continue;
            }
        };
        
        let input_size = data.len();
        println!("  原始: {} bytes", input_size);
        
        // Hub (禁用膜) 1/2/3 周期
        for cycles in 1..=3 {
            let (comp_size, restored) = hub_no_mesh_compress_restore(&data, cycles);
            let ratio = comp_size as f64 / input_size as f64 * 100.0;
            let quality = calculate_quality(&data, &restored);
            hub_results[fi][cycles - 1] = (ratio, quality);
            
            println!("  {} 周期: 压缩 {:>6.2}%, 质量 {:>6.2}%", cycles, ratio, quality);
        }
        
        // 主流最佳
        let gz = external_compress(file_path, "gzip", &["-c", "-9"]);
        let bz = external_compress(file_path, "bzip2", &["-c", "-9"]);
        let xz_r = external_compress(file_path, "xz", &["-c", "-9"]);
        let zst = external_compress_zstd(file_path);
        let best = [gz, bz, xz_r, zst].iter().copied().filter(|&x| x > 0).min().unwrap_or(input_size);
        mainstream_results[fi] = best as f64 / input_size as f64 * 100.0;
        
        println!("  主流最佳: {:>6.2}% (无损)", mainstream_results[fi]);
    }
    
    // ================== 横向对比表 ==================
    println!();
    println!("╔══════════════════════════════════════════════════════════════════════════════════════════╗");
    println!("║                           📊 横向对比: 各数据类型表现                                    ║");
    println!("╠═══════╦════════════════════╦════════════════════╦════════════════════╦══════════════════╣");
    println!("║       ║     1 周期         ║     2 周期         ║     3 周期         ║    主流最佳      ║");
    println!("║ 数据  ║  压缩   │   质量   ║  压缩   │   质量   ║  压缩   │   质量   ║  压缩  │  质量  ║");
    println!("╠═══════╬═════════╪══════════╬═════════╪══════════╬═════════╪══════════╬════════╪═════════╣");
    
    for (fi, (_, name)) in test_files.iter().enumerate() {
        let r = &hub_results[fi];
        let ms = mainstream_results[fi];
        
        // 找最佳压缩 (质量 > 50% 时)
        let mut best_comp = ms;
        let mut best_mark = [' ', ' ', ' ', '🏆'];
        
        for i in 0..3 {
            if r[i].1 > 50.0 && r[i].0 < best_comp {
                best_comp = r[i].0;
                best_mark = [' ', ' ', ' ', ' '];
                best_mark[i] = '🏆';
            }
        }
        if ms <= best_comp { best_mark[3] = '🏆'; }
        
        println!("║{:^7}║{:>6.1}%{}│{:>7.1}%  ║{:>6.1}%{}│{:>7.1}%  ║{:>6.1}%{}│{:>7.1}%  ║{:>5.1}%{}│ 100%  ║",
            name,
            r[0].0, best_mark[0], r[0].1,
            r[1].0, best_mark[1], r[1].1,
            r[2].0, best_mark[2], r[2].1,
            ms, best_mark[3]);
    }
    
    println!("╠═══════╬═════════╪══════════╬═════════╪══════════╬═════════╪══════════╬════════╪═════════╣");
    
    // 平均值
    let avg1: (f64, f64) = (
        hub_results.iter().map(|r| r[0].0).sum::<f64>() / test_files.len() as f64,
        hub_results.iter().map(|r| r[0].1).sum::<f64>() / test_files.len() as f64,
    );
    let avg2: (f64, f64) = (
        hub_results.iter().map(|r| r[1].0).sum::<f64>() / test_files.len() as f64,
        hub_results.iter().map(|r| r[1].1).sum::<f64>() / test_files.len() as f64,
    );
    let avg3: (f64, f64) = (
        hub_results.iter().map(|r| r[2].0).sum::<f64>() / test_files.len() as f64,
        hub_results.iter().map(|r| r[2].1).sum::<f64>() / test_files.len() as f64,
    );
    let avg_ms: f64 = mainstream_results.iter().sum::<f64>() / test_files.len() as f64;
    
    println!("║ 平均  ║{:>6.1}% │{:>7.1}%  ║{:>6.1}% │{:>7.1}%  ║{:>6.1}% │{:>7.1}%  ║{:>5.1}% │ 100%  ║",
        avg1.0, avg1.1, avg2.0, avg2.1, avg3.0, avg3.1, avg_ms);
    
    println!("╚═══════╩═════════╧══════════╩═════════╧══════════╩═════════╧══════════╩════════╧═════════╝");
    
    // ================== 纵向对比表 ==================
    println!();
    println!("╔══════════════════════════════════════════════════════════════════════════════════════════╗");
    println!("║                           📈 纵向对比: 周期增加的影响                                    ║");
    println!("╠═══════╦══════════════════════════════════════════════════════════════════════════════════╣");
    println!("║ 周期  ║  压缩率变化                        │  质量变化                                  ║");
    println!("╠═══════╬═════════════════════════════════════╪══════════════════════════════════════════════╣");
    
    println!("║ 1→2   ║  {:>+6.1}% (压缩{})              │  {:>+6.1}% (质量{})                       ║",
        avg2.0 - avg1.0,
        if avg2.0 < avg1.0 { "更好 ✓" } else { "更差 ✗" },
        avg2.1 - avg1.1,
        if avg2.1 > avg1.1 { "更好 ✓" } else { "更差 ✗" });
    
    println!("║ 2→3   ║  {:>+6.1}% (压缩{})              │  {:>+6.1}% (质量{})                       ║",
        avg3.0 - avg2.0,
        if avg3.0 < avg2.0 { "更好 ✓" } else { "更差 ✗" },
        avg3.1 - avg2.1,
        if avg3.1 > avg2.1 { "更好 ✓" } else { "更差 ✗" });
    
    println!("║ 1→3   ║  {:>+6.1}% (压缩{})              │  {:>+6.1}% (质量{})                       ║",
        avg3.0 - avg1.0,
        if avg3.0 < avg1.0 { "更好 ✓" } else { "更差 ✗" },
        avg3.1 - avg1.1,
        if avg3.1 > avg1.1 { "更好 ✓" } else { "更差 ✗" });
    
    println!("╚═══════╩═════════════════════════════════════╧══════════════════════════════════════════════╝");
    
    // ================== 综合得分 ==================
    println!();
    println!("🎯 综合得分 (质量 × 100/压缩率):");
    println!();
    
    let score1 = if avg1.0 > 0.0 { avg1.1 * (100.0 / avg1.0) } else { 0.0 };
    let score2 = if avg2.0 > 0.0 { avg2.1 * (100.0 / avg2.0) } else { 0.0 };
    let score3 = if avg3.0 > 0.0 { avg3.1 * (100.0 / avg3.0) } else { 0.0 };
    let score_ms = if avg_ms > 0.0 { 100.0 * (100.0 / avg_ms) } else { 0.0 };
    
    println!("   ┌─────────────┬─────────┬─────────┬─────────┬─────────┐");
    println!("   │             │ 1 周期  │ 2 周期  │ 3 周期  │主流最佳 │");
    println!("   ├─────────────┼─────────┼─────────┼─────────┼─────────┤");
    println!("   │ 平均压缩率  │{:>7.1}% │{:>7.1}% │{:>7.1}% │{:>7.1}% │", avg1.0, avg2.0, avg3.0, avg_ms);
    println!("   │ 平均质量    │{:>7.1}% │{:>7.1}% │{:>7.1}% │  100.0% │", avg1.1, avg2.1, avg3.1);
    println!("   │ 综合得分    │{:>7.1}  │{:>7.1}  │{:>7.1}  │{:>7.1}  │", score1, score2, score3, score_ms);
    println!("   └─────────────┴─────────┴─────────┴─────────┴─────────┘");
    
    // 最佳推荐
    println!();
    let scores = [(score1, "1周期"), (score2, "2周期"), (score3, "3周期"), (score_ms, "主流")];
    let best = scores.iter().max_by(|a, b| a.0.partial_cmp(&b.0).unwrap()).unwrap();
    println!("   🏆 最佳方案: {} (得分 {:.1})", best.1, best.0);
    
    // 分场景推荐
    println!();
    println!("   📋 场景推荐:");
    println!("      极致压缩 (不在乎质量): 3周期 Hub (压缩率 {:.1}%)", avg3.0);
    println!("      平衡压缩 (中等质量):   2周期 Hub (压缩率 {:.1}%, 质量 {:.1}%)", avg2.0, avg2.1);
    println!("      无损压缩:              主流算法 (压缩率 {:.1}%, 质量 100%)", avg_ms);
}

/// Hub (禁用膜系统) 压缩和还原
fn hub_no_mesh_compress_restore(data: &[u8], cycles: usize) -> (usize, Vec<u8>) {
    let mut current = data.to_vec();
    let mut cycle_meta: Vec<CycleMeta> = Vec::new();
    
    for cycle in 0..cycles {
        let features = extract_features(&current);
        let routing = fate_decide_no_mesh(&features);
        
        // 头部
        let mut compressed = Vec::new();
        compressed.push(cycle as u8);
        compressed.extend_from_slice(&(current.len() as u32).to_le_bytes());
        
        // 按路由分配到三个系统 (无膜)
        let mut bh_data = Vec::new();
        let mut mn_data = Vec::new();
        let mut bf_data = Vec::new();
        
        for (i, chunk) in current.chunks(16).enumerate() {
            // Fate 决定这个块去哪个系统
            let selector = (i as f64 / (current.len() / 16 + 1) as f64 + features[0]) % 1.0;
            
            if selector < routing[0] {
                // BlackHole
                let folded = chunk.iter().fold(0u8, |acc, &b| acc ^ b);
                bh_data.push(folded);
            } else if selector < routing[0] + routing[1] {
                // MultiNova
                let keep_bits = 5u8;
                let shift = 8 - keep_bits;
                for &b in chunk {
                    mn_data.push(b >> shift);
                }
            } else {
                // BaseForce
                let baseline = if chunk.is_empty() { 0 } else {
                    let mut sorted = chunk.to_vec();
                    sorted.sort();
                    sorted[sorted.len() / 2]
                };
                bf_data.push(baseline);
                for &b in chunk {
                    bf_data.push(b.wrapping_sub(baseline));
                }
            }
        }
        
        // 写入各系统数据
        compressed.extend_from_slice(&(bh_data.len() as u32).to_le_bytes());
        compressed.extend_from_slice(&bh_data);
        compressed.extend_from_slice(&(mn_data.len() as u32).to_le_bytes());
        compressed.extend_from_slice(&mn_data);
        compressed.extend_from_slice(&(bf_data.len() as u32).to_le_bytes());
        compressed.extend_from_slice(&bf_data);
        
        cycle_meta.push(CycleMeta {
            original_len: current.len(),
            routing,
            bh_len: bh_data.len(),
            mn_len: mn_data.len(),
            bf_len: bf_data.len(),
        });
        
        current = compressed;
    }
    
    let compressed_size = current.len();
    
    // 还原
    let mut restored = current;
    for meta in cycle_meta.into_iter().rev() {
        restored = restore_cycle(&restored, &meta);
    }
    
    (compressed_size, restored)
}

#[derive(Clone)]
struct CycleMeta {
    original_len: usize,
    routing: [f64; 3],
    bh_len: usize,
    mn_len: usize,
    bf_len: usize,
}

fn restore_cycle(data: &[u8], meta: &CycleMeta) -> Vec<u8> {
    if data.len() < 5 { return Vec::new(); }
    
    let mut pos = 5; // 跳过 cycle + original_len
    
    // 读取 BlackHole 数据
    if pos + 4 > data.len() { return vec![0; meta.original_len]; }
    let bh_len = u32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]) as usize;
    pos += 4;
    let bh_data: Vec<u8> = data[pos..pos.min(data.len())].iter().take(bh_len).copied().collect();
    pos += bh_len;
    
    // 读取 MultiNova 数据
    if pos + 4 > data.len() { return vec![0; meta.original_len]; }
    let mn_len = u32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]) as usize;
    pos += 4;
    let mn_data: Vec<u8> = data[pos..pos.min(data.len())].iter().take(mn_len).copied().collect();
    pos += mn_len;
    
    // 读取 BaseForce 数据
    if pos + 4 > data.len() { return vec![0; meta.original_len]; }
    let bf_len = u32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]) as usize;
    pos += 4;
    let bf_data: Vec<u8> = data[pos..pos.min(data.len())].iter().take(bf_len).copied().collect();
    
    // 重建原始数据
    let mut restored = Vec::with_capacity(meta.original_len);
    let mut bh_idx = 0;
    let mut mn_idx = 0;
    let mut bf_idx = 0;
    
    let chunk_count = (meta.original_len + 15) / 16;
    let features = [0.5f64; 4]; // 近似
    
    for i in 0..chunk_count {
        let selector = (i as f64 / chunk_count as f64 + features[0]) % 1.0;
        
        if selector < meta.routing[0] {
            // BlackHole: 扩展单字节到 16 字节
            if bh_idx < bh_data.len() {
                for _ in 0..16 {
                    restored.push(bh_data[bh_idx]);
                }
                bh_idx += 1;
            }
        } else if selector < meta.routing[0] + meta.routing[1] {
            // MultiNova: 左移恢复
            for _ in 0..16 {
                if mn_idx < mn_data.len() {
                    restored.push(mn_data[mn_idx] << 3);
                    mn_idx += 1;
                }
            }
        } else {
            // BaseForce: 加回基线
            if bf_idx < bf_data.len() {
                let baseline = bf_data[bf_idx];
                bf_idx += 1;
                for _ in 0..16 {
                    if bf_idx < bf_data.len() {
                        restored.push(bf_data[bf_idx].wrapping_add(baseline));
                        bf_idx += 1;
                    }
                }
            }
        }
    }
    
    restored.truncate(meta.original_len);
    while restored.len() < meta.original_len { restored.push(0); }
    
    restored
}

fn extract_features(data: &[u8]) -> [f64; 4] {
    if data.is_empty() { return [0.0; 4]; }
    
    let mut freq = [0u32; 256];
    for &b in data { freq[b as usize] += 1; }
    let len = data.len() as f64;
    let entropy: f64 = freq.iter()
        .filter(|&&f| f > 0)
        .map(|&f| { let p = f as f64 / len; -p * p.log2() })
        .sum::<f64>() / 8.0;
    
    let mut repeat = 0;
    for i in 1..data.len() { if data[i] == data[i-1] { repeat += 1; } }
    let pattern = 1.0 - (repeat as f64 / data.len() as f64);
    
    let mean = data.iter().map(|&b| b as f64).sum::<f64>() / len;
    let var = data.iter().map(|&b| (b as f64 - mean).powi(2)).sum::<f64>() / len;
    let structure = (var / 16384.0).min(1.0);
    
    let density = data.iter().filter(|&&b| b != 0).count() as f64 / len;
    
    [entropy, pattern, structure, density]
}

/// Fate 路由决策 (禁用膜系统)
fn fate_decide_no_mesh(features: &[f64; 4]) -> [f64; 3] {
    let entropy = features[0];
    let structure = features[2];
    let density = features[3];
    
    // 只有三个系统: BlackHole, MultiNova, BaseForce
    let mut weights = [0.0f64; 3];
    
    // BlackHole: 高熵、高密度 → 极端压缩
    weights[0] = entropy * density;
    
    // MultiNova: 中等熵 → 位平面分离
    weights[1] = (1.0 - entropy.abs()) * density;
    
    // BaseForce: 低熵、低结构 → 无损基线
    weights[2] = (1.0 - entropy) * (1.0 - structure);
    
    // 归一化
    let sum: f64 = weights.iter().sum();
    if sum > 0.0 {
        for w in &mut weights { *w /= sum; }
    } else {
        weights = [0.33, 0.33, 0.34];
    }
    
    weights
}

fn calculate_quality(original: &[u8], restored: &[u8]) -> f64 {
    if original.is_empty() || restored.is_empty() { return 0.0; }
    
    let min_len = original.len().min(restored.len());
    let mut matches = 0usize;
    
    for i in 0..min_len {
        if original[i] == restored[i] { matches += 1; }
    }
    
    let length_penalty = min_len as f64 / original.len() as f64;
    (matches as f64 / original.len() as f64) * length_penalty * 100.0
}

fn external_compress(file: &str, cmd: &str, args: &[&str]) -> usize {
    match Command::new(cmd).args(args).arg(file).output() {
        Ok(o) => o.stdout.len(),
        Err(_) => 0
    }
}

fn external_compress_zstd(file: &str) -> usize {
    match Command::new("zstd").args(&["-c", "-19", "-q", file]).output() {
        Ok(o) => o.stdout.len(),
        Err(_) => 0
    }
}
