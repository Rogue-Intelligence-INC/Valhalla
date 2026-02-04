//! 🌌 Valhalla 循环 (Cycle)
//!
//! 可逆变换 + Fate 自动决定周期数
//! 当边际收益递减时停止

use std::fs::{self, File};
use std::io::{self, Read, Write, BufWriter};
use std::path::Path;

/// Fate 状态
struct FateState {
    preferences: [f64; 4],
    prev_preferences: [f64; 4],
    stable_count: usize,
    signal_count: u64,
    
    // 边际收益跟踪
    prev_compression_ratio: f64,
    current_compression_ratio: f64,
    marginal_gain: f64,
}

impl FateState {
    fn new() -> Self {
        Self {
            preferences: [0.25, 0.25, 0.25, 0.25],
            prev_preferences: [0.0; 4],
            stable_count: 0,
            signal_count: 0,
            prev_compression_ratio: 1.0,
            current_compression_ratio: 1.0,
            marginal_gain: 1.0,
        }
    }
    
    fn process_signal(&mut self, features: [f64; 4]) {
        let alpha = 0.01;
        for i in 0..4 {
            self.preferences[i] = self.preferences[i] * (1.0 - alpha) + features[i] * alpha;
        }
        self.signal_count += 1;
    }
    
    fn check_convergence(&mut self) -> bool {
        let drift: f64 = self.preferences.iter()
            .zip(self.prev_preferences.iter())
            .map(|(a, b)| (a - b).abs())
            .sum();
        
        if drift < 0.0001 {
            self.stable_count += 1;
        } else {
            self.stable_count = 0;
        }
        self.prev_preferences = self.preferences;
        self.stable_count >= 30
    }
    
    fn reset_for_new_cycle(&mut self) {
        self.stable_count = 0;
        self.signal_count = 0;
    }
    
    /// 更新压缩比，计算边际收益
    fn update_compression(&mut self, ratio: f64) {
        self.prev_compression_ratio = self.current_compression_ratio;
        self.current_compression_ratio = ratio;
        
        // 边际收益 = 本次改善量
        self.marginal_gain = self.prev_compression_ratio - self.current_compression_ratio;
    }
    
    /// Fate 决定是否继续
    /// 当边际收益 < 1% 时停止
    fn should_continue(&self) -> bool {
        self.marginal_gain > 0.01  // 边际收益 > 1%
    }
}

/// Valhalla 循环器
struct ValhallaCycle {
    block_size: usize,
    max_cycles: usize,
}

impl ValhallaCycle {
    fn new(block_size: usize, max_cycles: usize) -> Self {
        Self { block_size, max_cycles }
    }
    
    /// 分析块
    fn analyze_block(&self, block: &[u8]) -> [f64; 4] {
        if block.is_empty() {
            return [0.25; 4];
        }
        
        let len = block.len() as f64;
        
        let mut freq = [0u32; 256];
        for &b in block {
            freq[b as usize] += 1;
        }
        let entropy: f64 = freq.iter()
            .filter(|&&f| f > 0)
            .map(|&f| {
                let p = f as f64 / len;
                -p * p.log2()
            })
            .sum();
        let f1 = (entropy / 8.0).min(1.0);
        
        let mut diff_sum = 0u64;
        for w in block.windows(2) {
            diff_sum += (w[1] as i32 - w[0] as i32).unsigned_abs() as u64;
        }
        let f2 = if block.len() > 1 {
            (diff_sum as f64 / (block.len() - 1) as f64 / 255.0).min(1.0)
        } else { 0.25 };
        
        let mean = block.iter().map(|&b| b as f64).sum::<f64>() / len;
        let variance = block.iter()
            .map(|&b| (b as f64 - mean).powi(2))
            .sum::<f64>() / len;
        let f3 = (variance.sqrt() / 128.0).min(1.0);
        
        let f4 = mean / 255.0;
        
        let sum = f1 + f2 + f3 + f4;
        if sum > 0.0 { [f1/sum, f2/sum, f3/sum, f4/sum] } else { [0.25; 4] }
    }
    
    /// 可逆变换: 重排序 + 差分编码
    fn reversible_transform(&self, data: &[u8], fate_prefs: &[f64; 4]) -> Vec<u8> {
        if data.is_empty() {
            return Vec::new();
        }
        
        // 计算主导系统
        let dominant = fate_prefs.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0);
        
        let mut result = Vec::with_capacity(data.len() + 8);
        
        // 头部: 标记 + 原始长度 + 主导系统
        result.push(0x56);  // 'V' for Valhalla
        result.push(dominant as u8);
        result.extend(&(data.len() as u32).to_le_bytes());
        
        // 统一使用简单的可逆变换: 差分编码
        // 差分编码是完全可逆的
        let mut prev = 0u8;
        for &b in data {
            result.push(b.wrapping_sub(prev));
            prev = b;
        }
        
        result
    }
    
    /// 逆变换 (差分解码)
    fn reverse_transform(data: &[u8]) -> Vec<u8> {
        if data.len() < 6 {
            return data.to_vec();
        }
        
        // 检查标记
        if data[0] != 0x56 {
            return data.to_vec();
        }
        
        let orig_len = u32::from_le_bytes([data[2], data[3], data[4], data[5]]) as usize;
        let payload = &data[6..];
        
        let mut result = Vec::with_capacity(orig_len);
        
        // 差分解码: 累加
        let mut prev = 0u8;
        for &b in payload {
            let orig = b.wrapping_add(prev);
            result.push(orig);
            prev = orig;
        }
        
        result.truncate(orig_len);
        result
    }
    
    /// RLE 压缩 (可逆)
    fn rle_compress(data: &[u8]) -> Vec<u8> {
        if data.is_empty() {
            return Vec::new();
        }
        
        let mut result = Vec::new();
        let mut i = 0;
        
        while i < data.len() {
            let current = data[i];
            let mut count = 1usize;
            
            while i + count < data.len() && data[i + count] == current && count < 255 {
                count += 1;
            }
            
            if count >= 4 {
                result.push(0xFF);
                result.push(current);
                result.push(count as u8);
            } else {
                for _ in 0..count {
                    if current == 0xFF {
                        result.push(0xFF);
                        result.push(0xFE);  // 转义
                    } else {
                        result.push(current);
                    }
                }
            }
            
            i += count;
        }
        
        result
    }
    
    /// RLE 解压
    fn rle_decompress(data: &[u8]) -> Vec<u8> {
        let mut result = Vec::new();
        let mut i = 0;
        
        while i < data.len() {
            if data[i] == 0xFF && i + 1 < data.len() {
                if data[i + 1] == 0xFE {
                    result.push(0xFF);
                    i += 2;
                } else if i + 2 < data.len() {
                    let val = data[i + 1];
                    let count = data[i + 2] as usize;
                    for _ in 0..count {
                        result.push(val);
                    }
                    i += 3;
                } else {
                    result.push(data[i]);
                    i += 1;
                }
            } else {
                result.push(data[i]);
                i += 1;
            }
        }
        
        result
    }
    
    /// 执行循环处理 - Fate 决定周期数
    fn cycle_process(&self, data: &[u8]) -> (Vec<u8>, FateState, usize) {
        let mut fate = FateState::new();
        let mut current_data = data.to_vec();
        let original_size = data.len();
        
        println!("🔄 开始循环 (Fate 自动决定周期数, 最大 {})", self.max_cycles);
        println!("   初始: {} bytes", original_size);
        println!("");
        
        let mut actual_cycles = 0;
        
        for cycle in 1..=self.max_cycles {
            fate.reset_for_new_cycle();
            let input_size = current_data.len();
            
            // Fate 分析
            let total_blocks = (current_data.len() + self.block_size - 1) / self.block_size;
            for i in 0..total_blocks.min(1000) {
                let start = i * self.block_size;
                let end = (start + self.block_size).min(current_data.len());
                let features = self.analyze_block(&current_data[start..end]);
                fate.process_signal(features);
                
                if fate.signal_count % 100 == 0 && fate.check_convergence() {
                    break;
                }
            }
            
            // 可逆变换
            let transformed = self.reversible_transform(&current_data, &fate.preferences);
            
            // RLE 压缩
            let compressed = Self::rle_compress(&transformed);
            
            let output_size = compressed.len();
            let ratio = output_size as f64 / original_size as f64;
            
            fate.update_compression(ratio);
            
            // 检测模式: 学习 vs 压缩
            let mode = if output_size > input_size {
                "学习/生成"
            } else if output_size < input_size {
                "压缩"
            } else {
                "平衡"
            };
            
            println!("   周期 {}: {} → {} bytes ({:.1}%), 模式=[{}], Fate=[{:.0}%,{:.0}%,{:.0}%,{:.0}%]",
                     cycle,
                     input_size,
                     output_size,
                     ratio * 100.0,
                     mode,
                     fate.preferences[0] * 100.0,
                     fate.preferences[1] * 100.0,
                     fate.preferences[2] * 100.0,
                     fate.preferences[3] * 100.0);
            
            current_data = compressed;
            actual_cycles = cycle;
            
            // Fate 决定是否继续
            if !fate.should_continue() {
                println!("");
                println!("   🛑 Fate 决定停止: 边际收益 < 1%");
                break;
            }
        }
        
        println!("");
        println!("   最终: {} bytes (压缩比 {:.2}%)", 
                 current_data.len(),
                 current_data.len() as f64 / original_size as f64 * 100.0);
        
        (current_data, fate, actual_cycles)
    }
    
    /// 处理文件
    fn process_file(&self, input_path: &str, output_dir: &str) -> io::Result<()> {
        println!("📥 输入: {}", input_path);
        println!("📦 输出: {}", output_dir);
        
        let mut file = File::open(input_path)?;
        let file_size = file.metadata()?.len();
        let mut data = Vec::with_capacity(file_size as usize);
        file.read_to_end(&mut data)?;
        
        println!("📊 大小: {} bytes", data.len());
        println!("");
        
        fs::create_dir_all(output_dir)?;
        
        // 循环处理
        let (final_data, fate, actual_cycles) = self.cycle_process(&data);
        
        // 写入 manifest
        let manifest_path = Path::new(output_dir).join("manifest.valhalla");
        let mut manifest = File::create(&manifest_path)?;
        
        manifest.write_all(b"VALHALLA")?;
        manifest.write_all(&[4u8])?;  // version 4 (可逆)
        manifest.write_all(&(data.len() as u64).to_le_bytes())?;
        manifest.write_all(&(actual_cycles as u32).to_le_bytes())?;
        for pref in &fate.preferences {
            manifest.write_all(&pref.to_le_bytes())?;
        }
        
        // 写入数据
        let data_path = Path::new(output_dir).join("data.vbin");
        let mut f = BufWriter::new(File::create(&data_path)?);
        f.write_all(&final_data)?;
        
        let manifest_size = fs::metadata(&manifest_path)?.len();
        let total_size = final_data.len() as u64 + manifest_size;
        
        println!("🌌 输出:");
        println!("   data.vbin: {} bytes", final_data.len());
        println!("   manifest: {} bytes", manifest_size);
        println!("");
        println!("📊 结果:");
        println!("   原始: {} bytes", file_size);
        println!("   输出: {} bytes", total_size);
        println!("   比率: {:.2}%", total_size as f64 / file_size as f64 * 100.0);
        println!("   周期: {} (Fate 决定)", actual_cycles);
        
        Ok(())
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║     🌌 Valhalla 循环 (可逆 + Fate 自动)                        ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    
    if args.len() < 3 {
        println!("用法: {} <input_file> <output_dir> [max_cycles]", args[0]);
        return Ok(());
    }
    
    let input = &args[1];
    let output = &args[2];
    let max_cycles = args.get(3)
        .and_then(|s| s.parse().ok())
        .unwrap_or(20);
    
    let cycler = ValhallaCycle::new(4096, max_cycles);
    cycler.process_file(input, output)?;
    
    println!("\n✅ 完成!");
    
    Ok(())
}
