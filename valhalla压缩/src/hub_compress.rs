//! 使用完整 Valhalla Hub 处理数据
//! 测试: 学习模式 (扩张) vs 压缩模式 (收缩)

use std::fs;
use std::io::{Read, Write};
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("用法: {} <输入文件> <输出文件>", args[0]);
        std::process::exit(1);
    }
    
    let input_path = &args[1];
    let output_path = &args[2];
    
    // 读取输入
    let mut input = fs::File::open(input_path).expect("无法打开输入");
    let mut data = Vec::new();
    input.read_to_end(&mut data).expect("读取失败");
    
    let input_size = data.len();
    println!("📥 输入: {} ({} bytes)", input_path, input_size);
    
    // ============================================
    // Valhalla Hub 处理流程
    // ============================================
    
    // 1. 信号特征提取 (模拟 DeformationSignal)
    let features = extract_features(&data);
    println!("🔍 特征: entropy={:.3}, pattern={:.3}, structure={:.3}, density={:.3}",
        features[0], features[1], features[2], features[3]);
    
    // 2. Fate 路由决策
    let routing = fate_decide(&features);
    println!("🎯 Fate路由: BH={:.1}% MB={:.1}% MN={:.1}% BF={:.1}%",
        routing[0]*100.0, routing[1]*100.0, routing[2]*100.0, routing[3]*100.0);
    
    // 3. 四元系统并行处理
    let (bh_out, mb_out, mn_out, bf_out) = process_quad_systems(&data, &routing, &features);
    
    // 4. 输出整合
    let mut output_data = Vec::new();
    
    // 头部: 版本 + 原始大小 + 特征 + 路由
    output_data.extend_from_slice(b"VALH");  // 魔数
    output_data.extend_from_slice(&(input_size as u64).to_le_bytes());
    for f in &features {
        output_data.extend_from_slice(&f.to_le_bytes());
    }
    for r in &routing {
        output_data.extend_from_slice(&r.to_le_bytes());
    }
    
    // 各系统输出 (按路由权重决定是否包含)
    let threshold = 0.2;
    
    if routing[0] > threshold {
        output_data.extend_from_slice(&(bh_out.len() as u32).to_le_bytes());
        output_data.extend_from_slice(&bh_out);
    } else {
        output_data.extend_from_slice(&0u32.to_le_bytes());
    }
    
    if routing[1] > threshold {
        output_data.extend_from_slice(&(mb_out.len() as u32).to_le_bytes());
        output_data.extend_from_slice(&mb_out);
    } else {
        output_data.extend_from_slice(&0u32.to_le_bytes());
    }
    
    if routing[2] > threshold {
        output_data.extend_from_slice(&(mn_out.len() as u32).to_le_bytes());
        output_data.extend_from_slice(&mn_out);
    } else {
        output_data.extend_from_slice(&0u32.to_le_bytes());
    }
    
    if routing[3] > threshold {
        output_data.extend_from_slice(&(bf_out.len() as u32).to_le_bytes());
        output_data.extend_from_slice(&bf_out);
    } else {
        output_data.extend_from_slice(&0u32.to_le_bytes());
    }
    
    // 写出
    let mut out_file = fs::File::create(output_path).expect("无法创建输出");
    out_file.write_all(&output_data).expect("写入失败");
    
    let output_size = output_data.len();
    let ratio = output_size as f64 / input_size as f64 * 100.0;
    
    println!("📤 输出: {} ({} bytes)", output_path, output_size);
    println!("");
    
    if output_size > input_size {
        let expand = (output_size - input_size) as f64 / input_size as f64 * 100.0;
        println!("🎓 学习模式: 扩张 +{:.1}% (Valhalla 在学习/生成)", expand);
    } else {
        let compress = (input_size - output_size) as f64 / input_size as f64 * 100.0;
        println!("📦 压缩模式: 收缩 -{:.1}% (Valhalla 在压缩)", compress);
    }
    
    println!("📊 最终比例: {:.1}%", ratio);
}

/// 提取信号特征 (模拟 DeformationSignal)
fn extract_features(data: &[u8]) -> [f64; 4] {
    if data.is_empty() {
        return [0.0; 4];
    }
    
    // 熵 (信息密度)
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
        .sum::<f64>() / 8.0;  // 归一化到 0-1
    
    // 模式重复度 (相邻相似)
    let mut repeat_count = 0;
    for i in 1..data.len() {
        if data[i] == data[i-1] {
            repeat_count += 1;
        }
    }
    let pattern = 1.0 - (repeat_count as f64 / data.len() as f64);
    
    // 结构性 (字节分布方差)
    let mean = data.iter().map(|&b| b as f64).sum::<f64>() / len;
    let variance = data.iter().map(|&b| (b as f64 - mean).powi(2)).sum::<f64>() / len;
    let structure = (variance / 16384.0).min(1.0);  // 归一化
    
    // 密度 (非零字节比例)
    let non_zero = data.iter().filter(|&&b| b != 0).count() as f64;
    let density = non_zero / len;
    
    [entropy, pattern, structure, density]
}

/// Fate 路由决策
fn fate_decide(features: &[f64; 4]) -> [f64; 4] {
    let entropy = features[0];
    let pattern = features[1];
    let structure = features[2];
    let density = features[3];
    
    // 基于特征分配路由权重
    let mut weights = [0.0f64; 4];
    
    // BlackHole: 高熵、高密度 → 极端压缩
    weights[0] = entropy * density;
    
    // MeshBrain: 高模式、高结构 → 关系网络
    weights[1] = pattern * structure;
    
    // MultiNova: 中等熵、高密度 → 并行分解
    weights[2] = (1.0 - entropy.abs()) * density;
    
    // BaseForce: 低熵、低结构 → 基础存储
    weights[3] = (1.0 - entropy) * (1.0 - structure);
    
    // 归一化
    let sum: f64 = weights.iter().sum();
    if sum > 0.0 {
        for w in &mut weights {
            *w /= sum;
        }
    } else {
        weights = [0.25; 4];
    }
    
    weights
}

/// 四元系统并行处理
fn process_quad_systems(data: &[u8], routing: &[f64; 4], features: &[f64; 4]) -> (Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>) {
    let entropy = features[0];
    
    // BlackHole: XOR 折叠压缩 (有损但极致)
    let bh_output = if routing[0] > 0.1 {
        blackhole_process(data, (routing[0] * 8.0).ceil() as usize)
    } else {
        Vec::new()
    };
    
    // MeshBrain: 差分编码 (无损)
    let mb_output = if routing[1] > 0.1 {
        meshbrain_process(data)
    } else {
        Vec::new()
    };
    
    // MultiNova: 位平面分离 (可调损)
    let mn_output = if routing[2] > 0.1 {
        multinova_process(data, entropy)
    } else {
        Vec::new()
    };
    
    // BaseForce: 基线编码 (无损)
    let bf_output = if routing[3] > 0.1 {
        baseforce_process(data)
    } else {
        Vec::new()
    };
    
    println!("  BlackHole: {} bytes", bh_output.len());
    println!("  MeshBrain: {} bytes", mb_output.len());
    println!("  MultiNova: {} bytes", mn_output.len());
    println!("  BaseForce: {} bytes", bf_output.len());
    
    (bh_output, mb_output, mn_output, bf_output)
}

/// BlackHole: XOR 折叠压缩
fn blackhole_process(data: &[u8], fold_level: usize) -> Vec<u8> {
    let fold = fold_level.max(1).min(8);
    let chunk_size = 1 << fold;  // 2^fold
    
    let mut output = Vec::new();
    output.push(fold as u8);  // 存储折叠级别
    
    for chunk in data.chunks(chunk_size) {
        let folded = chunk.iter().fold(0u8, |acc, &b| acc ^ b);
        output.push(folded);
    }
    
    output
}

/// MeshBrain: 差分编码
fn meshbrain_process(data: &[u8]) -> Vec<u8> {
    if data.is_empty() {
        return Vec::new();
    }
    
    let mut output = Vec::with_capacity(data.len());
    output.push(data[0]);  // 首字节原样
    
    for i in 1..data.len() {
        let diff = data[i].wrapping_sub(data[i-1]);
        output.push(diff);
    }
    
    // RLE 压缩差分序列
    rle_compress(&output)
}

/// MultiNova: 位平面分离
fn multinova_process(data: &[u8], entropy: f64) -> Vec<u8> {
    // 保留的位数 (根据熵决定)
    let keep_bits = ((1.0 - entropy) * 8.0).ceil() as u8;
    let keep_bits = keep_bits.max(2).min(8);
    
    let mut output = Vec::new();
    output.push(keep_bits);  // 存储保留位数
    
    let shift = 8 - keep_bits;
    for &b in data {
        output.push(b >> shift);
    }
    
    // 位打包
    if keep_bits < 8 {
        output = bit_pack(&output[1..], keep_bits);
        let mut result = vec![keep_bits];
        result.extend(output);
        result
    } else {
        output
    }
}

/// BaseForce: 基线编码
fn baseforce_process(data: &[u8]) -> Vec<u8> {
    if data.is_empty() {
        return Vec::new();
    }
    
    // 找基线 (中位数)
    let mut sorted = data.to_vec();
    sorted.sort();
    let baseline = sorted[sorted.len() / 2];
    
    let mut output = Vec::with_capacity(data.len() + 1);
    output.push(baseline);
    
    for &b in data {
        output.push(b.wrapping_sub(baseline));
    }
    
    // RLE 压缩偏移序列
    rle_compress(&output)
}

/// RLE 压缩
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
        
        if count >= 3 {
            output.push(0xFF);  // RLE 标记
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

/// 位打包
fn bit_pack(data: &[u8], bits: u8) -> Vec<u8> {
    let bits = bits as usize;
    let total_bits = data.len() * bits;
    let output_len = (total_bits + 7) / 8;
    
    let mut output = vec![0u8; output_len + 4];
    // 存储原始长度
    let len_bytes = (data.len() as u32).to_le_bytes();
    output[0..4].copy_from_slice(&len_bytes);
    
    let mut bit_pos = 0;
    for &b in data {
        let byte_idx = 4 + bit_pos / 8;
        let bit_offset = bit_pos % 8;
        
        if byte_idx < output.len() {
            output[byte_idx] |= (b & ((1 << bits) - 1)) << bit_offset;
            if bit_offset + bits > 8 && byte_idx + 1 < output.len() {
                output[byte_idx + 1] |= b >> (8 - bit_offset);
            }
        }
        bit_pos += bits;
    }
    
    output
}
