//! 4元系统 vs 3元系统 对比测试
//! Fate 控制路由，每种配置跑 2 周期

use std::fs;
use std::io::{Read, Write};

#[derive(Clone, Copy, Debug)]
enum SystemConfig {
    Quad,           // 4元: BH + MB + MN + BF
    TriNoBF,        // 3元: BH + MB + MN (无BaseForce)
    TriNoMN,        // 3元: BH + MB + BF (无MultiNova)
    TriNoMB,        // 3元: BH + MN + BF (无MeshBrain)
    TriNoBH,        // 3元: MB + MN + BF (无BlackHole)
}

impl SystemConfig {
    fn name(&self) -> &'static str {
        match self {
            SystemConfig::Quad => "4元(BH+MB+MN+BF)",
            SystemConfig::TriNoBF => "3元(BH+MB+MN)",
            SystemConfig::TriNoMN => "3元(BH+MB+BF)",
            SystemConfig::TriNoMB => "3元(BH+MN+BF)",
            SystemConfig::TriNoBH => "3元(MB+MN+BF)",
        }
    }
    
    fn systems(&self) -> [bool; 4] {
        // [BlackHole, MeshBrain, MultiNova, BaseForce]
        match self {
            SystemConfig::Quad => [true, true, true, true],
            SystemConfig::TriNoBF => [true, true, true, false],
            SystemConfig::TriNoMN => [true, true, false, true],
            SystemConfig::TriNoMB => [true, false, true, true],
            SystemConfig::TriNoBH => [false, true, true, true],
        }
    }
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║     Valhalla 4元 vs 3元系统 压缩对比 (Fate控制, 2周期)         ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();
    
    let test_files = vec![
        ("/tmp/raw_text.txt", "文本(对话)"),
        ("/tmp/test_image.bmp", "BMP图片"),
        ("/tmp/test_audio.wav", "WAV音频"),
        ("/tmp/test_data.json", "JSON数据"),
        ("/tmp/repeat.bin", "重复数据"),
        ("/tmp/random.bin", "随机数据"),
    ];
    
    let configs = vec![
        SystemConfig::Quad,
        SystemConfig::TriNoBF,
        SystemConfig::TriNoMN,
        SystemConfig::TriNoMB,
        SystemConfig::TriNoBH,
    ];
    
    // 存储结果
    let mut results: Vec<Vec<f64>> = vec![vec![0.0; configs.len()]; test_files.len()];
    
    for (fi, (file_path, file_name)) in test_files.iter().enumerate() {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📁 {} ({})", file_name, file_path);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        let data = match fs::read(file_path) {
            Ok(d) => d,
            Err(_) => {
                println!("  ⚠️ 文件不存在，跳过");
                continue;
            }
        };
        
        let input_size = data.len();
        println!("  原始大小: {} bytes", input_size);
        println!();
        
        for (ci, config) in configs.iter().enumerate() {
            let output = process_with_config(&data, *config, 2);
            let ratio = output.len() as f64 / input_size as f64 * 100.0;
            results[fi][ci] = ratio;
            
            let mode = if output.len() > input_size { "🎓学习" } else { "📦压缩" };
            println!("  {:20} → {:>8} bytes ({:>6.1}%) {}", 
                config.name(), output.len(), ratio, mode);
        }
        println!();
    }
    
    // 汇总表格
    println!();
    println!("╔════════════════════════════════════════════════════════════════════════════════╗");
    println!("║                              📊 汇总对比表                                      ║");
    println!("╠════════════════════════════════════════════════════════════════════════════════╣");
    print!("║ {:12}", "数据类型");
    for config in &configs {
        print!(" │{:>13}", config.name().split('(').next().unwrap());
    }
    println!(" ║");
    println!("╠════════════════════════════════════════════════════════════════════════════════╣");
    
    for (fi, (_, file_name)) in test_files.iter().enumerate() {
        print!("║ {:12}", file_name);
        
        // 找出最佳
        let min_ratio = results[fi].iter().cloned().fold(f64::INFINITY, f64::min);
        
        for (ci, _) in configs.iter().enumerate() {
            let ratio = results[fi][ci];
            if ratio == 0.0 {
                print!(" │{:>13}", "-");
            } else if (ratio - min_ratio).abs() < 0.1 {
                print!(" │{:>12}%🏆", format!("{:.1}", ratio));
            } else {
                print!(" │{:>12}%  ", format!("{:.1}", ratio));
            }
        }
        println!(" ║");
    }
    println!("╚════════════════════════════════════════════════════════════════════════════════╝");
    
    // 分析
    println!();
    println!("📈 分析:");
    for (ci, config) in configs.iter().enumerate() {
        let avg: f64 = results.iter()
            .map(|r| r[ci])
            .filter(|&x| x > 0.0)
            .sum::<f64>() / results.len() as f64;
        println!("  {} 平均压缩率: {:.1}%", config.name(), avg);
    }
}

fn process_with_config(data: &[u8], config: SystemConfig, cycles: usize) -> Vec<u8> {
    let enabled = config.systems();
    let mut current_data = data.to_vec();
    
    for cycle in 0..cycles {
        let features = extract_features(&current_data);
        let routing = fate_decide(&features, &enabled);
        
        let mut cycle_output = Vec::new();
        
        // 头部
        cycle_output.extend_from_slice(&(current_data.len() as u32).to_le_bytes());
        cycle_output.push(config as u8);
        cycle_output.push(cycle as u8);
        
        // 处理各系统
        let threshold = 0.15;
        
        if enabled[0] && routing[0] > threshold {
            let out = blackhole_process(&current_data, routing[0]);
            cycle_output.extend_from_slice(&(out.len() as u32).to_le_bytes());
            cycle_output.extend_from_slice(&out);
        } else {
            cycle_output.extend_from_slice(&0u32.to_le_bytes());
        }
        
        if enabled[1] && routing[1] > threshold {
            let out = meshbrain_process(&current_data);
            cycle_output.extend_from_slice(&(out.len() as u32).to_le_bytes());
            cycle_output.extend_from_slice(&out);
        } else {
            cycle_output.extend_from_slice(&0u32.to_le_bytes());
        }
        
        if enabled[2] && routing[2] > threshold {
            let out = multinova_process(&current_data, features[0]);
            cycle_output.extend_from_slice(&(out.len() as u32).to_le_bytes());
            cycle_output.extend_from_slice(&out);
        } else {
            cycle_output.extend_from_slice(&0u32.to_le_bytes());
        }
        
        if enabled[3] && routing[3] > threshold {
            let out = baseforce_process(&current_data);
            cycle_output.extend_from_slice(&(out.len() as u32).to_le_bytes());
            cycle_output.extend_from_slice(&out);
        } else {
            cycle_output.extend_from_slice(&0u32.to_le_bytes());
        }
        
        current_data = cycle_output;
    }
    
    current_data
}

fn extract_features(data: &[u8]) -> [f64; 4] {
    if data.is_empty() {
        return [0.0; 4];
    }
    
    let mut freq = [0u32; 256];
    for &b in data {
        freq[b as usize] += 1;
    }
    let len = data.len() as f64;
    let entropy: f64 = freq.iter()
        .filter(|&&f| f > 0)
        .map(|&f| {
            let p = f as f64 / len;
            -p * p.log2()
        })
        .sum::<f64>() / 8.0;
    
    let mut repeat_count = 0;
    for i in 1..data.len() {
        if data[i] == data[i-1] {
            repeat_count += 1;
        }
    }
    let pattern = 1.0 - (repeat_count as f64 / data.len() as f64);
    
    let mean = data.iter().map(|&b| b as f64).sum::<f64>() / len;
    let variance = data.iter().map(|&b| (b as f64 - mean).powi(2)).sum::<f64>() / len;
    let structure = (variance / 16384.0).min(1.0);
    
    let non_zero = data.iter().filter(|&&b| b != 0).count() as f64;
    let density = non_zero / len;
    
    [entropy, pattern, structure, density]
}

fn fate_decide(features: &[f64; 4], enabled: &[bool; 4]) -> [f64; 4] {
    let entropy = features[0];
    let pattern = features[1];
    let structure = features[2];
    let density = features[3];
    
    let mut weights = [0.0f64; 4];
    
    if enabled[0] { weights[0] = entropy * density; }
    if enabled[1] { weights[1] = pattern * structure; }
    if enabled[2] { weights[2] = (1.0 - entropy.abs()) * density; }
    if enabled[3] { weights[3] = (1.0 - entropy) * (1.0 - structure); }
    
    let sum: f64 = weights.iter().sum();
    if sum > 0.0 {
        for w in &mut weights {
            *w /= sum;
        }
    }
    
    weights
}

fn blackhole_process(data: &[u8], weight: f64) -> Vec<u8> {
    let fold = ((weight * 8.0).ceil() as usize).max(1).min(8);
    let chunk_size = 1 << fold;
    
    let mut output = vec![fold as u8];
    for chunk in data.chunks(chunk_size) {
        let folded = chunk.iter().fold(0u8, |acc, &b| acc ^ b);
        output.push(folded);
    }
    output
}

fn meshbrain_process(data: &[u8]) -> Vec<u8> {
    if data.is_empty() {
        return Vec::new();
    }
    
    let mut output = Vec::with_capacity(data.len());
    output.push(data[0]);
    
    for i in 1..data.len() {
        output.push(data[i].wrapping_sub(data[i-1]));
    }
    
    rle_compress(&output)
}

fn multinova_process(data: &[u8], entropy: f64) -> Vec<u8> {
    let keep_bits = ((1.0 - entropy) * 8.0).ceil() as u8;
    let keep_bits = keep_bits.max(2).min(8);
    
    let shift = 8 - keep_bits;
    let mut packed = vec![keep_bits];
    
    for &b in data {
        packed.push(b >> shift);
    }
    
    if keep_bits < 8 {
        let bits_data = &packed[1..];
        let total_bits = bits_data.len() * keep_bits as usize;
        let output_len = (total_bits + 7) / 8;
        
        let mut result = vec![0u8; output_len + 5];
        result[0] = keep_bits;
        result[1..5].copy_from_slice(&(bits_data.len() as u32).to_le_bytes());
        
        let mut bit_pos = 0;
        for &b in bits_data {
            let byte_idx = 5 + bit_pos / 8;
            let bit_offset = bit_pos % 8;
            
            if byte_idx < result.len() {
                result[byte_idx] |= (b & ((1 << keep_bits) - 1)) << bit_offset;
                if bit_offset + keep_bits as usize > 8 && byte_idx + 1 < result.len() {
                    result[byte_idx + 1] |= b >> (8 - bit_offset);
                }
            }
            bit_pos += keep_bits as usize;
        }
        result
    } else {
        packed
    }
}

fn baseforce_process(data: &[u8]) -> Vec<u8> {
    if data.is_empty() {
        return Vec::new();
    }
    
    let mut sorted = data.to_vec();
    sorted.sort();
    let baseline = sorted[sorted.len() / 2];
    
    let mut output = vec![baseline];
    for &b in data {
        output.push(b.wrapping_sub(baseline));
    }
    
    rle_compress(&output)
}

fn rle_compress(data: &[u8]) -> Vec<u8> {
    if data.is_empty() {
        return Vec::new();
    }
    
    let mut output = Vec::new();
    let mut i = 0;
    
    while i < data.len() {
        let byte = data[i];
        let mut count = 1u8;
        
        while i + (count as usize) < data.len() 
            && data[i + count as usize] == byte 
            && count < 255 
        {
            count += 1;
        }
        
        if count >= 4 {
            output.push(0xFF);
            output.push(count);
            output.push(byte);
            i += count as usize;
        } else {
            if byte == 0xFF {
                output.push(0xFF);
                output.push(1);
                output.push(byte);
            } else {
                output.push(byte);
            }
            i += 1;
        }
    }
    
    output
}
