//! Valhalla 全系统压缩率 + 质量对比
//! 关闭 Chamber/StemCell/Tile 的膜系统 (MeshBrain)
//! 3 轮迭代测试

use std::fs;
use std::process::Command;

fn main() {
    println!("╔════════════════════════════════════════════════════════════════════════════════════════╗");
    println!("║     Valhalla 全系统 压缩率 + 质量 对比 (关闭膜系统, 3轮迭代)                           ║");
    println!("╚════════════════════════════════════════════════════════════════════════════════════════╝");
    println!();
    
    let test_files = vec![
        ("/tmp/raw_text.txt", "文本"),
        ("/tmp/test_image.bmp", "BMP"),
        ("/tmp/test_audio.wav", "WAV"),
        ("/tmp/test_data.json", "JSON"),
        ("/tmp/repeat.bin", "重复"),
        ("/tmp/random.bin", "随机"),
    ];
    
    // 算法列表
    let algorithms = vec![
        ("BlackHole", true),
        ("MultiNova", true),
        ("BaseForce", true),
        ("Tile(无膜)", true),
        ("Chamber(无膜)", true),
        ("StemCell(无膜)", true),
        ("gzip", false),
        ("bzip2", false),
        ("xz", false),
        ("zstd", false),
    ];
    
    // 结果存储: [文件][算法] = (压缩率, 质量)
    let mut results: Vec<Vec<(f64, f64)>> = vec![vec![(0.0, 0.0); algorithms.len()]; test_files.len()];
    
    for (fi, (file_path, file_name)) in test_files.iter().enumerate() {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
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
        
        let features = extract_features(&data);
        
        // Valhalla 系统 (3轮)
        let (bh_comp, bh_rest) = blackhole_compress_restore(&data, features[0], 3);
        let (mn_comp, mn_rest) = multinova_compress_restore(&data, features[0], 3);
        let (bf_comp, bf_rest) = baseforce_compress_restore(&data, 3);
        let (tile_comp, tile_rest) = tile_no_mesh_compress_restore(&data, &features, 3);
        let (chamber_comp, chamber_rest) = chamber_no_mesh_compress_restore(&data, &features, 3);
        let (stem_comp, stem_rest) = stemcell_no_mesh_compress_restore(&data, &features, 3);
        
        let bh_ratio = bh_comp as f64 / input_size as f64 * 100.0;
        let bh_quality = calculate_quality(&data, &bh_rest);
        
        let mn_ratio = mn_comp as f64 / input_size as f64 * 100.0;
        let mn_quality = calculate_quality(&data, &mn_rest);
        
        let bf_ratio = bf_comp as f64 / input_size as f64 * 100.0;
        let bf_quality = calculate_quality(&data, &bf_rest);
        
        let tile_ratio = tile_comp as f64 / input_size as f64 * 100.0;
        let tile_quality = calculate_quality(&data, &tile_rest);
        
        let chamber_ratio = chamber_comp as f64 / input_size as f64 * 100.0;
        let chamber_quality = calculate_quality(&data, &chamber_rest);
        
        let stem_ratio = stem_comp as f64 / input_size as f64 * 100.0;
        let stem_quality = calculate_quality(&data, &stem_rest);
        
        // 主流算法 (无损 = 100% 质量)
        let gz = external_compress(file_path, "gzip", &["-c", "-9"]) as f64 / input_size as f64 * 100.0;
        let bz = external_compress(file_path, "bzip2", &["-c", "-9"]) as f64 / input_size as f64 * 100.0;
        let xz_r = external_compress(file_path, "xz", &["-c", "-9"]) as f64 / input_size as f64 * 100.0;
        let zst = external_compress_zstd(file_path) as f64 / input_size as f64 * 100.0;
        
        results[fi] = vec![
            (bh_ratio, bh_quality),
            (mn_ratio, mn_quality),
            (bf_ratio, bf_quality),
            (tile_ratio, tile_quality),
            (chamber_ratio, chamber_quality),
            (stem_ratio, stem_quality),
            (gz, 100.0),
            (bz, 100.0),
            (xz_r, 100.0),
            (zst, 100.0),
        ];
        
        println!("  ┌─ Valhalla (关闭膜系统, 3轮):");
        println!("  │  BlackHole:  {:>5.1}% 压缩, {:>5.1}% 质量", bh_ratio, bh_quality);
        println!("  │  MultiNova:  {:>5.1}% 压缩, {:>5.1}% 质量", mn_ratio, mn_quality);
        println!("  │  BaseForce:  {:>5.1}% 压缩, {:>5.1}% 质量", bf_ratio, bf_quality);
        println!("  │  Tile(无膜):  {:>5.1}% 压缩, {:>5.1}% 质量", tile_ratio, tile_quality);
        println!("  │  Chamber(无膜):{:>5.1}% 压缩, {:>5.1}% 质量", chamber_ratio, chamber_quality);
        println!("  │  StemCell(无膜):{:>5.1}% 压缩, {:>5.1}% 质量", stem_ratio, stem_quality);
        println!("  └─ 主流算法 (无损):");
        println!("     gzip:{:>5.1}%  bzip2:{:>5.1}%  xz:{:>5.1}%  zstd:{:>5.1}%", gz, bz, xz_r, zst);
    }
    
    // 汇总表格
    println!();
    println!("╔═════════════════════════════════════════════════════════════════════════════════════════════════════════════╗");
    println!("║                               📊 压缩率 vs 质量 对比 (3轮迭代)                                               ║");
    println!("╠═══════╦══════════════╦══════════════╦══════════════╦══════════════╦══════════════╦══════════════╦════════════╣");
    println!("║       ║  BlackHole   ║  MultiNova   ║  BaseForce   ║ Tile(无膜)  ║Chamber(无膜)║StemCell(无膜)║  主流最佳  ║");
    println!("║ 数据  ║ 压缩  │ 质量 ║ 压缩  │ 质量 ║ 压缩  │ 质量 ║ 压缩 │ 质量 ║ 压缩 │ 质量 ║ 压缩 │ 质量  ║ 压缩│质量 ║");
    println!("╠═══════╬═══════╪══════╬═══════╪══════╬═══════╪══════╬══════╪══════╬══════╪══════╬══════╪═══════╬═════╪══════╣");
    
    for (fi, (_, name)) in test_files.iter().enumerate() {
        let r = &results[fi];
        if r.iter().all(|(c, _)| *c == 0.0) { continue; }
        
        // 找主流最佳
        let mainstream_best = r[6..10].iter()
            .map(|(c, _)| *c)
            .fold(f64::INFINITY, f64::min);
        
        print!("║{:^7}║", name);
        for i in 0..6 {
            let (comp, qual) = r[i];
            print!("{:>5.1}%│{:>5.1}%║", comp, qual);
        }
        print!("{:>4.1}%│100% ║", mainstream_best);
        println!();
    }
    
    println!("╠═══════╬═══════╪══════╬═══════╪══════╬═══════╪══════╬══════╪══════╬══════╪══════╬══════╪═══════╬═════╪══════╣");
    
    // 平均值
    print!("║ 平均  ║");
    for i in 0..6 {
        let avg_comp: f64 = results.iter().filter(|r| r[i].0 > 0.0).map(|r| r[i].0).sum::<f64>() / results.len() as f64;
        let avg_qual: f64 = results.iter().filter(|r| r[i].1 > 0.0).map(|r| r[i].1).sum::<f64>() / results.len() as f64;
        print!("{:>5.1}%│{:>5.1}%║", avg_comp, avg_qual);
    }
    let mainstream_avg: f64 = results.iter()
        .map(|r| r[6..10].iter().map(|(c, _)| *c).fold(f64::INFINITY, f64::min))
        .filter(|&x| x < f64::INFINITY)
        .sum::<f64>() / results.len() as f64;
    print!("{:>4.1}%│100% ║", mainstream_avg);
    println!();
    
    println!("╚═══════╩═══════╧══════╩═══════╧══════╩═══════╧══════╩══════╧══════╩══════╧══════╩══════╧═══════╩═════╧══════╝");
    
    // 综合得分 (压缩率 * 质量平衡)
    println!();
    println!("🎯 综合得分 (质量权重 × 压缩效率):");
    println!("   公式: score = 质量% × (100 / 压缩率%)  (越高越好)");
    println!();
    
    let algo_names = ["BlackHole", "MultiNova", "BaseForce", "Tile(无膜)", "Chamber(无膜)", "StemCell(无膜)", "主流最佳"];
    
    for (i, name) in algo_names.iter().enumerate() {
        if i < 6 {
            let avg_comp: f64 = results.iter().filter(|r| r[i].0 > 0.0).map(|r| r[i].0).sum::<f64>() / results.len() as f64;
            let avg_qual: f64 = results.iter().filter(|r| r[i].1 > 0.0).map(|r| r[i].1).sum::<f64>() / results.len() as f64;
            let score = if avg_comp > 0.0 { avg_qual * (100.0 / avg_comp) } else { 0.0 };
            println!("   {:15}: 压缩{:>5.1}% × 质量{:>5.1}% = 得分 {:>6.1}", name, avg_comp, avg_qual, score);
        } else {
            println!("   {:15}: 压缩{:>5.1}% × 质量 100.0% = 得分 {:>6.1}", name, mainstream_avg, 100.0 * (100.0 / mainstream_avg));
        }
    }
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

fn calculate_quality(original: &[u8], restored: &[u8]) -> f64 {
    if original.is_empty() || restored.is_empty() {
        return 0.0;
    }
    
    let min_len = original.len().min(restored.len());
    let mut matches = 0usize;
    
    for i in 0..min_len {
        if original[i] == restored[i] {
            matches += 1;
        }
    }
    
    // 长度惩罚
    let length_penalty = min_len as f64 / original.len() as f64;
    
    (matches as f64 / original.len() as f64) * length_penalty * 100.0
}

// ========== BlackHole 压缩/还原 (3轮) ==========
fn blackhole_compress_restore(data: &[u8], entropy: f64, rounds: usize) -> (usize, Vec<u8>) {
    let mut current = data.to_vec();
    let mut fold_levels = Vec::new();
    
    for _ in 0..rounds {
        let fold = ((entropy * 8.0).ceil() as usize).max(1).min(6);
        let chunk_size = 1 << fold;
        
        let mut compressed = vec![fold as u8];
        for chunk in current.chunks(chunk_size) {
            let folded = chunk.iter().fold(0u8, |acc, &b| acc ^ b);
            compressed.push(folded);
        }
        
        fold_levels.push((fold, current.len()));
        current = compressed;
    }
    
    let compressed_size = current.len();
    
    // 还原 (逆向 XOR 无法完美还原，用近似)
    let mut restored = current;
    for (fold, orig_len) in fold_levels.into_iter().rev() {
        let chunk_size = 1 << fold;
        let mut expanded = Vec::new();
        
        for &b in restored.iter().skip(1) {
            for _ in 0..chunk_size {
                expanded.push(b);
            }
        }
        expanded.truncate(orig_len);
        restored = expanded;
    }
    
    (compressed_size, restored)
}

// ========== MultiNova 压缩/还原 (3轮) ==========
fn multinova_compress_restore(data: &[u8], entropy: f64, rounds: usize) -> (usize, Vec<u8>) {
    let mut current = data.to_vec();
    let mut keep_bits_history = Vec::new();
    
    for _ in 0..rounds {
        let keep_bits = ((1.0 - entropy) * 8.0).ceil() as u8;
        let keep_bits = keep_bits.max(3).min(7);
        
        let shift = 8 - keep_bits;
        let compressed: Vec<u8> = current.iter().map(|&b| b >> shift).collect();
        
        keep_bits_history.push((keep_bits, current.len()));
        current = compressed;
    }
    
    let compressed_size = current.len() + keep_bits_history.len();
    
    // 还原
    let mut restored = current;
    for (keep_bits, _) in keep_bits_history.into_iter().rev() {
        let shift = 8 - keep_bits;
        restored = restored.iter().map(|&b| b << shift).collect();
    }
    
    (compressed_size, restored)
}

// ========== BaseForce 压缩/还原 (3轮) ==========
fn baseforce_compress_restore(data: &[u8], rounds: usize) -> (usize, Vec<u8>) {
    if data.is_empty() { return (0, Vec::new()); }
    
    let mut current = data.to_vec();
    let mut baselines = Vec::new();
    
    for _ in 0..rounds {
        let mut sorted = current.clone();
        sorted.sort();
        let baseline = sorted[sorted.len() / 2];
        
        let offset: Vec<u8> = current.iter().map(|&b| b.wrapping_sub(baseline)).collect();
        
        baselines.push(baseline);
        current = offset;
    }
    
    let compressed_size = current.len() + baselines.len();
    
    // 还原
    let mut restored = current;
    for baseline in baselines.into_iter().rev() {
        restored = restored.iter().map(|&b| b.wrapping_add(baseline)).collect();
    }
    
    (compressed_size, restored)
}

// ========== Tile (无膜) 压缩/还原 (3轮) ==========
fn tile_no_mesh_compress_restore(data: &[u8], features: &[f64; 4], rounds: usize) -> (usize, Vec<u8>) {
    let mut current = data.to_vec();
    let entropy = features[0];
    
    for _ in 0..rounds {
        let mut compressed = Vec::new();
        
        for chunk in current.chunks(64) {
            let size = chunk.len();
            
            // 无膜系统: 只用 BlackHole, MultiNova, BaseForce
            if size > 32 {
                // 高密度 -> BlackHole
                let fold = 3usize;
                let chunk_size = 1 << fold;
                compressed.push(0u8); // 标记: BlackHole
                for sub in chunk.chunks(chunk_size) {
                    compressed.push(sub.iter().fold(0u8, |acc, &b| acc ^ b));
                }
            } else if size > 8 {
                // 中等 -> MultiNova
                compressed.push(1u8); // 标记: MultiNova
                let keep_bits = ((1.0 - entropy) * 8.0).ceil() as u8;
                let keep_bits = keep_bits.max(4).min(7);
                let shift = 8 - keep_bits;
                for &b in chunk {
                    compressed.push(b >> shift);
                }
            } else {
                // 小块 -> BaseForce
                compressed.push(2u8); // 标记: BaseForce
                let mut sorted = chunk.to_vec();
                sorted.sort();
                let baseline = sorted[sorted.len() / 2];
                compressed.push(baseline);
                for &b in chunk {
                    compressed.push(b.wrapping_sub(baseline));
                }
            }
        }
        
        current = compressed;
    }
    
    let compressed_size = current.len();
    
    // 简化还原: 直接从压缩数据重建 (有损)
    let mut restored = Vec::with_capacity(data.len());
    let mut i = 0;
    while i < current.len() && restored.len() < data.len() {
        let tag = current[i];
        i += 1;
        
        match tag {
            0 => { // BlackHole
                for j in 0..8 {
                    if i + j < current.len() {
                        restored.push(current[i + j]);
                    }
                }
                i += 8;
            }
            1 => { // MultiNova
                for j in 0..32 {
                    if i + j < current.len() {
                        restored.push(current[i + j] << 2);
                    }
                }
                i += 32;
            }
            2 => { // BaseForce
                if i < current.len() {
                    let baseline = current[i];
                    i += 1;
                    for j in 0..7 {
                        if i + j < current.len() {
                            restored.push(current[i + j].wrapping_add(baseline));
                        }
                    }
                    i += 7;
                }
            }
            _ => { i += 1; }
        }
    }
    
    restored.truncate(data.len());
    while restored.len() < data.len() { restored.push(0); }
    
    (compressed_size, restored)
}

// ========== Chamber (无膜) 压缩/还原 (3轮) ==========
fn chamber_no_mesh_compress_restore(data: &[u8], features: &[f64; 4], rounds: usize) -> (usize, Vec<u8>) {
    let entropy = features[0];
    let mut current = data.to_vec();
    
    // 容器数量基于熵
    let container_count = if entropy > 0.8 { 4 } else if entropy > 0.5 { 2 } else { 1 };
    
    for _ in 0..rounds {
        let chunk_size = (current.len() + container_count - 1) / container_count;
        let mut compressed = vec![container_count as u8];
        
        for i in 0..container_count {
            let start = i * chunk_size;
            let end = ((i + 1) * chunk_size).min(current.len());
            if start >= current.len() { break; }
            
            let chunk = &current[start..end];
            
            // 无膜: 只用 BlackHole
            let fold = 3usize;
            let cs = 1 << fold;
            for sub in chunk.chunks(cs) {
                compressed.push(sub.iter().fold(0u8, |acc, &b| acc ^ b));
            }
        }
        
        current = compressed;
    }
    
    let compressed_size = current.len();
    
    // 还原
    let mut restored = Vec::with_capacity(data.len());
    let container_count = current.get(0).copied().unwrap_or(1) as usize;
    
    for &b in current.iter().skip(1) {
        for _ in 0..8 {
            restored.push(b);
        }
    }
    
    restored.truncate(data.len());
    while restored.len() < data.len() { restored.push(0); }
    
    (compressed_size, restored)
}

// ========== StemCell (无膜) 压缩/还原 (3轮) ==========
fn stemcell_no_mesh_compress_restore(data: &[u8], features: &[f64; 4], rounds: usize) -> (usize, Vec<u8>) {
    let mut cells: Vec<Vec<u8>> = data.chunks(16).map(|c| c.to_vec()).collect();
    
    for _ in 0..rounds {
        let mut new_cells = Vec::new();
        
        for cell in &cells {
            let cell_entropy = cell_entropy(cell);
            let energy = 1.0 - cell_entropy;
            
            // 死亡阈值 (高熵细胞死亡) - 无膜所以更激进
            if energy < 0.3 {
                // 用 BlackHole 压缩后保留
                let folded = cell.iter().fold(0u8, |acc, &b| acc ^ b);
                new_cells.push(vec![folded]);
                continue;
            }
            
            // 分裂阈值 (低熵细胞分裂)
            if energy > 0.6 && cell.len() > 4 {
                let mid = cell.len() / 2;
                new_cells.push(cell[..mid].to_vec());
                new_cells.push(cell[mid..].to_vec());
            } else {
                // BaseForce 压缩
                let mut sorted = cell.clone();
                sorted.sort();
                let baseline = sorted[sorted.len() / 2];
                let offset: Vec<u8> = cell.iter().map(|&b| b.wrapping_sub(baseline)).collect();
                new_cells.push(offset);
            }
        }
        
        cells = new_cells;
    }
    
    let compressed_size: usize = cells.iter().map(|c| c.len() + 1).sum();
    
    // 还原
    let mut restored = Vec::with_capacity(data.len());
    for cell in &cells {
        for &b in cell {
            restored.push(b);
        }
    }
    
    restored.truncate(data.len());
    while restored.len() < data.len() { restored.push(0); }
    
    (compressed_size, restored)
}

fn cell_entropy(data: &[u8]) -> f64 {
    if data.is_empty() { return 0.0; }
    let mut freq = [0u32; 256];
    for &b in data { freq[b as usize] += 1; }
    let len = data.len() as f64;
    freq.iter()
        .filter(|&&f| f > 0)
        .map(|&f| { let p = f as f64 / len; -p * p.log2() })
        .sum::<f64>() / 8.0
}

fn external_compress(file: &str, cmd: &str, args: &[&str]) -> usize {
    let output = Command::new(cmd).args(args).arg(file).output();
    match output { Ok(o) => o.stdout.len(), Err(_) => 0 }
}

fn external_compress_zstd(file: &str) -> usize {
    let output = Command::new("zstd").args(&["-c", "-19", "-q", file]).output();
    match output { Ok(o) => o.stdout.len(), Err(_) => 0 }
}
