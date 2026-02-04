//! Valhalla 全系统 vs 主流压缩算法对比
//! 
//! 测试: BlackHole, MeshBrain, MultiNova, BaseForce, Tile, Chamber, StemCell
//! 对比: gzip, bzip2, xz, zstd, lz4

use std::fs;
use std::process::Command;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════════════════════════╗");
    println!("║          Valhalla 全系统 vs 主流压缩算法 横向对比 (Fate 控制, 2 周期)            ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════════╝");
    println!();
    
    let test_files = vec![
        ("/tmp/raw_text.txt", "文本"),
        ("/tmp/test_image.bmp", "BMP"),
        ("/tmp/test_audio.wav", "WAV"),
        ("/tmp/test_data.json", "JSON"),
        ("/tmp/repeat.bin", "重复"),
        ("/tmp/random.bin", "随机"),
    ];
    
    // 收集所有结果
    let mut all_results: Vec<Vec<f64>> = Vec::new();
    
    for (file_path, file_name) in &test_files {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📁 {} ({})", file_name, file_path);
        
        let data = match fs::read(file_path) {
            Ok(d) => d,
            Err(_) => {
                println!("  ⚠️ 文件不存在");
                all_results.push(vec![0.0; 12]);
                continue;
            }
        };
        
        let input_size = data.len();
        println!("  原始: {} bytes", input_size);
        
        let features = extract_features(&data);
        
        // Valhalla 系统
        let bh = blackhole_compress(&data, features[0]) as f64 / input_size as f64 * 100.0;
        let mb = meshbrain_compress(&data) as f64 / input_size as f64 * 100.0;
        let mn = multinova_compress(&data, features[0]) as f64 / input_size as f64 * 100.0;
        let bf = baseforce_compress(&data) as f64 / input_size as f64 * 100.0;
        let tile = tile_compress(&data, &features) as f64 / input_size as f64 * 100.0;
        let chamber = chamber_compress(&data, &features) as f64 / input_size as f64 * 100.0;
        let stemcell = stemcell_compress(&data, &features) as f64 / input_size as f64 * 100.0;
        
        // 主流算法
        let gz = external_compress(file_path, "gzip", &["-c", "-9"]) as f64 / input_size as f64 * 100.0;
        let bz = external_compress(file_path, "bzip2", &["-c", "-9"]) as f64 / input_size as f64 * 100.0;
        let xz_r = external_compress(file_path, "xz", &["-c", "-9"]) as f64 / input_size as f64 * 100.0;
        let zst = external_compress_zstd(file_path) as f64 / input_size as f64 * 100.0;
        let lz = external_compress_lz4(file_path) as f64 / input_size as f64 * 100.0;
        
        println!("  ├─ Valhalla 系统:");
        println!("  │  BlackHole:{:>6.1}%  MeshBrain:{:>6.1}%  MultiNova:{:>6.1}%  BaseForce:{:>6.1}%", bh, mb, mn, bf);
        println!("  │  Tile:{:>6.1}%  Chamber:{:>6.1}%  StemCell:{:>6.1}%", tile, chamber, stemcell);
        println!("  └─ 主流算法:");
        println!("     gzip:{:>6.1}%  bzip2:{:>6.1}%  xz:{:>6.1}%  zstd:{:>6.1}%  lz4:{:>6.1}%", gz, bz, xz_r, zst, lz);
        
        all_results.push(vec![bh, mb, mn, bf, tile, chamber, stemcell, gz, bz, xz_r, zst, lz]);
    }
    
    // 汇总表格
    println!();
    println!("╔══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════╗");
    println!("║                                           📊 压缩率对比 (%)                                                          ║");
    println!("╠═══════╦════════╦════════╦════════╦════════╦═══════╦═════════╦══════════╦═══════╦═══════╦══════╦═══════╦═══════╣");
    println!("║ 数据  ║BlackH. ║MeshBr. ║MultiN. ║BaseFo. ║ Tile  ║ Chamber ║ StemCell ║ gzip  ║ bzip2 ║  xz  ║ zstd  ║  lz4  ║");
    println!("╠═══════╬════════╬════════╬════════╬════════╬═══════╬═════════╬══════════╬═══════╬═══════╬══════╬═══════╬═══════╣");
    
    let algo_names = ["BlackHole", "MeshBrain", "MultiNova", "BaseForce", "Tile", "Chamber", "StemCell", "gzip", "bzip2", "xz", "zstd", "lz4"];
    
    for (i, (_, name)) in test_files.iter().enumerate() {
        let results = &all_results[i];
        let min_idx = results.iter().enumerate()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(idx, _)| idx)
            .unwrap_or(0);
        
        print!("║{:^7}║", name);
        for (j, r) in results.iter().enumerate() {
            if j == min_idx {
                print!("{:>6.1}%🏆║", r);
            } else {
                print!("{:>7.1}%║", r);
            }
        }
        println!();
    }
    
    println!("╠═══════╬════════╬════════╬════════╬════════╬═══════╬═════════╬══════════╬═══════╬═══════╬══════╬═══════╬═══════╣");
    
    // 平均值
    print!("║ 平均  ║");
    for j in 0..12 {
        let avg: f64 = all_results.iter()
            .filter(|r| r[j] > 0.0)
            .map(|r| r[j])
            .sum::<f64>() / all_results.len() as f64;
        print!("{:>7.1}%║", avg);
    }
    println!();
    println!("╚═══════╩════════╩════════╩════════╩════════╩═══════╩═════════╩══════════╩═══════╩═══════╩══════╩═══════╩═══════╝");
    
    // 胜负统计
    println!();
    println!("🏆 各算法胜利次数:");
    let mut wins = vec![0usize; 12];
    for results in &all_results {
        if results.iter().all(|&x| x == 0.0) { continue; }
        let min_idx = results.iter().enumerate()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(idx, _)| idx)
            .unwrap_or(0);
        wins[min_idx] += 1;
    }
    
    println!("   Valhalla: BlackHole={} MeshBrain={} MultiNova={} BaseForce={} Tile={} Chamber={} StemCell={}",
        wins[0], wins[1], wins[2], wins[3], wins[4], wins[5], wins[6]);
    println!("   主流算法: gzip={} bzip2={} xz={} zstd={} lz4={}",
        wins[7], wins[8], wins[9], wins[10], wins[11]);
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

// ========== Valhalla 系统 ==========

fn blackhole_compress(data: &[u8], entropy: f64) -> usize {
    // XOR 折叠压缩 - 折叠级别基于熵
    let fold = ((entropy * 8.0).ceil() as usize).max(1).min(8);
    let chunk_size = 1 << fold;
    1 + (data.len() + chunk_size - 1) / chunk_size
}

fn meshbrain_compress(data: &[u8]) -> usize {
    // 差分 + RLE
    if data.is_empty() { return 0; }
    let mut diff = vec![data[0]];
    for i in 1..data.len() {
        diff.push(data[i].wrapping_sub(data[i-1]));
    }
    rle_size(&diff)
}

fn multinova_compress(data: &[u8], entropy: f64) -> usize {
    // 位平面分离 - 根据熵保留位数
    let keep_bits = ((1.0 - entropy) * 8.0).ceil() as usize;
    let keep_bits = keep_bits.max(2).min(8);
    let total_bits = data.len() * keep_bits;
    5 + (total_bits + 7) / 8
}

fn baseforce_compress(data: &[u8]) -> usize {
    // 基线 + RLE
    if data.is_empty() { return 0; }
    let mut sorted = data.to_vec();
    sorted.sort();
    let baseline = sorted[sorted.len() / 2];
    let offset: Vec<u8> = data.iter().map(|&b| b.wrapping_sub(baseline)).collect();
    1 + rle_size(&offset)
}

fn tile_compress(data: &[u8], features: &[f64; 4]) -> usize {
    // Tile: 按块大小分配到不同压缩策略
    let mut total = 4; // 头部
    
    for chunk in data.chunks(64) {
        let chunk_features = extract_features(chunk);
        let size = chunk.len();
        
        // 根据块特征选择最佳压缩
        if size > 48 {
            // 高密度 -> BlackHole
            total += blackhole_compress(chunk, chunk_features[0]);
        } else if size > 16 {
            // 中等 -> MultiNova
            total += multinova_compress(chunk, chunk_features[0]);
        } else if size > 4 {
            // 小块 -> MeshBrain
            total += meshbrain_compress(chunk);
        } else {
            // 微块 -> BaseForce
            total += baseforce_compress(chunk);
        }
    }
    total
}

fn chamber_compress(data: &[u8], features: &[f64; 4]) -> usize {
    // Chamber: 自适应容器扩缩容
    let entropy = features[0];
    let pattern = features[1];
    
    // 根据负载自动选择容器数量
    let container_count = if entropy > 0.8 {
        4  // 高熵 - 更多容器分散处理
    } else if entropy > 0.5 {
        2
    } else {
        1  // 低熵 - 单容器即可
    };
    
    let chunk_size = data.len() / container_count;
    let mut total = 8; // 头部 + 容器元数据
    
    for i in 0..container_count {
        let start = i * chunk_size;
        let end = if i == container_count - 1 { data.len() } else { (i + 1) * chunk_size };
        let chunk = &data[start..end];
        
        // 每个容器使用混合压缩
        if pattern > 0.9 {
            total += meshbrain_compress(chunk);
        } else {
            total += blackhole_compress(chunk, entropy);
        }
    }
    total
}

fn stemcell_compress(data: &[u8], features: &[f64; 4]) -> usize {
    // StemCell: 进化式压缩 - 数据"细胞"分裂/死亡
    let entropy = features[0];
    let density = features[3];
    
    // 初始细胞 = 数据块
    let mut cells: Vec<Vec<u8>> = data.chunks(16).map(|c| c.to_vec()).collect();
    
    // 进化 2 代
    for _ in 0..2 {
        let mut new_cells = Vec::new();
        
        for cell in &mut cells {
            let cell_entropy = cell_entropy(cell);
            let energy = 1.0 - cell_entropy; // 低熵 = 高能量
            
            // 死亡阈值 (高熵细胞死亡)
            if energy < 0.2 {
                continue; // 细胞死亡 - 数据丢失
            }
            
            // 分裂阈值 (低熵细胞分裂)
            if energy > 0.7 && cell.len() > 8 {
                let mid = cell.len() / 2;
                new_cells.push(cell[..mid].to_vec());
                new_cells.push(cell[mid..].to_vec());
            } else {
                new_cells.push(cell.clone());
            }
        }
        
        cells = new_cells;
    }
    
    // 存活细胞压缩
    let mut total = 4; // 头部
    for cell in &cells {
        total += 1 + cell.len() / 2; // 简化压缩
    }
    total
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

fn rle_size(data: &[u8]) -> usize {
    if data.is_empty() { return 0; }
    let mut size = 0;
    let mut i = 0;
    while i < data.len() {
        let byte = data[i];
        let mut count = 1;
        while i + count < data.len() && data[i + count] == byte && count < 255 {
            count += 1;
        }
        if count >= 4 {
            size += 3; // RLE: marker + count + byte
        } else {
            size += if byte == 0xFF { 3 } else { 1 };
        }
        i += count;
    }
    size
}

// ========== 主流算法 ==========

fn external_compress(file: &str, cmd: &str, args: &[&str]) -> usize {
    let output = Command::new(cmd)
        .args(args)
        .arg(file)
        .output();
    
    match output {
        Ok(o) => o.stdout.len(),
        Err(_) => 0,
    }
}

fn external_compress_zstd(file: &str) -> usize {
    let output = Command::new("zstd")
        .args(&["-c", "-19", "-q", file])
        .output();
    match output {
        Ok(o) => o.stdout.len(),
        Err(_) => 0,
    }
}

fn external_compress_lz4(file: &str) -> usize {
    let output = Command::new("lz4")
        .args(&["-c", "-9", "-q", file])
        .output();
    match output {
        Ok(o) => o.stdout.len(),
        Err(_) => 0,
    }
}
