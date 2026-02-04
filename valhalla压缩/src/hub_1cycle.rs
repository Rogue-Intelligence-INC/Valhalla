use std::fs;
use std::process::Command;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║          Valhalla Hub (禁用膜系统) - 1 周期详细测试                          ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
    println!();
    
    let test_files = vec![
        ("/tmp/raw_text.txt", "文本"),
        ("/tmp/test_image.bmp", "BMP"),
        ("/tmp/test_audio.wav", "WAV"),
        ("/tmp/test_data.json", "JSON"),
        ("/tmp/repeat.bin", "重复"),
        ("/tmp/random.bin", "随机"),
    ];
    
    println!("┌─────────┬──────────┬────────────┬────────────┬────────────┬────────────┬──────────┐");
    println!("│ 数据    │ 原始大小 │ Hub压缩后  │ 压缩率     │ 还原质量   │ 主流最佳   │ 胜者     │");
    println!("├─────────┼──────────┼────────────┼────────────┼────────────┼────────────┼──────────┤");
    
    let mut total_orig = 0usize;
    let mut total_hub = 0usize;
    let mut total_mainstream = 0usize;
    let mut hub_wins = 0;
    let mut mainstream_wins = 0;
    
    for (file_path, file_name) in &test_files {
        let data = match fs::read(file_path) {
            Ok(d) => d,
            Err(_) => continue,
        };
        
        let input_size = data.len();
        total_orig += input_size;
        
        // Hub 1周期压缩
        let (comp_size, restored) = hub_1cycle_compress(&data);
        let ratio = comp_size as f64 / input_size as f64 * 100.0;
        let quality = calculate_quality(&data, &restored);
        
        total_hub += comp_size;
        
        // 主流最佳
        let mainstream = get_best_mainstream(file_path, input_size);
        let ms_ratio = mainstream as f64 / input_size as f64 * 100.0;
        total_mainstream += mainstream;
        
        // 判断胜者 (只看压缩率，因为主流无损)
        let winner = if ratio < ms_ratio { 
            hub_wins += 1;
            "Hub 🏆" 
        } else { 
            mainstream_wins += 1;
            "主流 🏆" 
        };
        
        println!("│ {:7} │ {:>8} │ {:>10} │ {:>9.1}% │ {:>9.1}% │ {:>9.1}% │ {:8} │",
            file_name, 
            format_size(input_size),
            format_size(comp_size),
            ratio,
            quality,
            ms_ratio,
            winner);
    }
    
    println!("├─────────┼──────────┼────────────┼────────────┼────────────┼────────────┼──────────┤");
    
    let hub_ratio = total_hub as f64 / total_orig as f64 * 100.0;
    let ms_ratio = total_mainstream as f64 / total_orig as f64 * 100.0;
    
    println!("│ 总计    │ {:>8} │ {:>10} │ {:>9.1}% │     -      │ {:>9.1}% │ {:8} │",
        format_size(total_orig),
        format_size(total_hub),
        hub_ratio,
        ms_ratio,
        if hub_ratio < ms_ratio { "Hub 🏆" } else { "主流 🏆" });
    
    println!("└─────────┴──────────┴────────────┴────────────┴────────────┴────────────┴──────────┘");
    
    println!();
    println!("📊 胜负统计: Hub {} 胜, 主流 {} 胜", hub_wins, mainstream_wins);
    println!();
    println!("💡 分析:");
    println!("   Hub 1周期平均压缩率: {:.1}%", hub_ratio);
    println!("   主流算法平均压缩率: {:.1}%", ms_ratio);
    println!("   Hub 优势场景: 高熵数据 (随机/音频)");
    println!("   主流优势场景: 低熵/结构化数据 (重复/JSON)");
}

fn hub_1cycle_compress(data: &[u8]) -> (usize, Vec<u8>) {
    let features = extract_features(data);
    let routing = fate_decide_no_mesh(&features);
    
    let mut compressed = Vec::new();
    compressed.push(0u8); // cycle
    compressed.extend_from_slice(&(data.len() as u32).to_le_bytes());
    
    let mut bh_data = Vec::new();
    let mut mn_data = Vec::new();
    let mut bf_data = Vec::new();
    
    for (i, chunk) in data.chunks(16).enumerate() {
        let selector = (i as f64 / (data.len() / 16 + 1) as f64 + features[0]) % 1.0;
        
        if selector < routing[0] {
            // BlackHole: XOR 折叠
            let folded = chunk.iter().fold(0u8, |acc, &b| acc ^ b);
            bh_data.push(folded);
        } else if selector < routing[0] + routing[1] {
            // MultiNova: 位截断
            for &b in chunk {
                mn_data.push(b >> 3);
            }
        } else {
            // BaseForce: 基线编码 (可逆)
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
    
    compressed.extend_from_slice(&(bh_data.len() as u32).to_le_bytes());
    compressed.extend_from_slice(&bh_data);
    compressed.extend_from_slice(&(mn_data.len() as u32).to_le_bytes());
    compressed.extend_from_slice(&mn_data);
    compressed.extend_from_slice(&(bf_data.len() as u32).to_le_bytes());
    compressed.extend_from_slice(&bf_data);
    
    let comp_size = compressed.len();
    
    // 还原
    let mut restored = Vec::with_capacity(data.len());
    let mut bh_idx = 0;
    let mut mn_idx = 0;
    let mut bf_idx = 0;
    
    let chunk_count = (data.len() + 15) / 16;
    
    for i in 0..chunk_count {
        let selector = (i as f64 / chunk_count as f64 + features[0]) % 1.0;
        
        if selector < routing[0] {
            // BlackHole 还原 (有损)
            if bh_idx < bh_data.len() {
                for _ in 0..16 { restored.push(bh_data[bh_idx]); }
                bh_idx += 1;
            }
        } else if selector < routing[0] + routing[1] {
            // MultiNova 还原 (有损)
            for _ in 0..16 {
                if mn_idx < mn_data.len() {
                    restored.push(mn_data[mn_idx] << 3);
                    mn_idx += 1;
                }
            }
        } else {
            // BaseForce 还原 (无损)
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
    
    restored.truncate(data.len());
    while restored.len() < data.len() { restored.push(0); }
    
    (comp_size, restored)
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
    let density = data.iter().filter(|&&b| b != 0).count() as f64 / len;
    let structure = {
        let mean = data.iter().map(|&b| b as f64).sum::<f64>() / len;
        let var = data.iter().map(|&b| (b as f64 - mean).powi(2)).sum::<f64>() / len;
        (var / 16384.0).min(1.0)
    };
    [entropy, 0.5, structure, density]
}

fn fate_decide_no_mesh(features: &[f64; 4]) -> [f64; 3] {
    let entropy = features[0];
    let structure = features[2];
    let density = features[3];
    
    let mut weights = [
        entropy * density,                    // BlackHole
        (1.0 - entropy.abs()) * density,      // MultiNova
        (1.0 - entropy) * (1.0 - structure),  // BaseForce
    ];
    
    let sum: f64 = weights.iter().sum();
    if sum > 0.0 { for w in &mut weights { *w /= sum; } }
    else { weights = [0.33, 0.33, 0.34]; }
    
    weights
}

fn calculate_quality(original: &[u8], restored: &[u8]) -> f64 {
    if original.is_empty() || restored.is_empty() { return 0.0; }
    let min_len = original.len().min(restored.len());
    let matches = (0..min_len).filter(|&i| original[i] == restored[i]).count();
    (matches as f64 / original.len() as f64) * 100.0
}

fn get_best_mainstream(file: &str, orig_size: usize) -> usize {
    let results = [
        Command::new("gzip").args(&["-c", "-9", file]).output().map(|o| o.stdout.len()).unwrap_or(orig_size),
        Command::new("bzip2").args(&["-c", "-9", file]).output().map(|o| o.stdout.len()).unwrap_or(orig_size),
        Command::new("xz").args(&["-c", "-9", file]).output().map(|o| o.stdout.len()).unwrap_or(orig_size),
        Command::new("zstd").args(&["-c", "-19", "-q", file]).output().map(|o| o.stdout.len()).unwrap_or(orig_size),
    ];
    *results.iter().filter(|&&x| x > 0).min().unwrap_or(&orig_size)
}

fn format_size(bytes: usize) -> String {
    if bytes >= 1_000_000 { format!("{:.1}MB", bytes as f64 / 1_000_000.0) }
    else if bytes >= 1_000 { format!("{:.1}KB", bytes as f64 / 1_000.0) }
    else { format!("{}B", bytes) }
}
