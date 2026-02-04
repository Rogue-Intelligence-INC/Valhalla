//! 🌌 Valhalla 还原 (Restore)
//!
//! 可逆还原 - 反向执行变换

use std::fs::File;
use std::io::{self, Read, Write, BufReader};
use std::path::Path;

struct ValhallaRestore;

impl ValhallaRestore {
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
    
    /// 还原文件
    fn restore_file(input_dir: &str, output_path: &str) -> io::Result<()> {
        println!("📥 输入: {}", input_dir);
        println!("📦 输出: {}", output_path);
        
        let input_dir = Path::new(input_dir);
        
        // 读取 manifest
        let manifest_path = input_dir.join("manifest.valhalla");
        let mut manifest = BufReader::new(File::open(&manifest_path)?);
        
        let mut magic = [0u8; 8];
        manifest.read_exact(&mut magic)?;
        if &magic != b"VALHALLA" {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "无效的 Valhalla 文件"));
        }
        
        let mut version = [0u8; 1];
        manifest.read_exact(&mut version)?;
        println!("📋 版本: {}", version[0]);
        
        let mut orig_size_bytes = [0u8; 8];
        manifest.read_exact(&mut orig_size_bytes)?;
        let orig_size = u64::from_le_bytes(orig_size_bytes) as usize;
        
        let mut cycles_bytes = [0u8; 4];
        manifest.read_exact(&mut cycles_bytes)?;
        let num_cycles = u32::from_le_bytes(cycles_bytes) as usize;
        
        // 读取 Fate 偏好
        let mut fate_prefs = [0.0f64; 4];
        for i in 0..4 {
            let mut pref_bytes = [0u8; 8];
            manifest.read_exact(&mut pref_bytes)?;
            fate_prefs[i] = f64::from_le_bytes(pref_bytes);
        }
        
        println!("📊 原始大小: {} bytes", orig_size);
        println!("📊 周期数: {}", num_cycles);
        println!("📊 Fate: [{:.1}%, {:.1}%, {:.1}%, {:.1}%]",
                 fate_prefs[0] * 100.0,
                 fate_prefs[1] * 100.0,
                 fate_prefs[2] * 100.0,
                 fate_prefs[3] * 100.0);
        
        // 读取压缩数据
        let data_path = input_dir.join("data.vbin");
        let mut data_file = File::open(&data_path)?;
        let mut current_data = Vec::new();
        data_file.read_to_end(&mut current_data)?;
        
        println!("📊 压缩数据: {} bytes", current_data.len());
        println!("");
        
        // 反向还原
        println!("🔄 开始反向还原 ({} 周期)...", num_cycles);
        
        for cycle in (1..=num_cycles).rev() {
            let input_size = current_data.len();
            
            // 先 RLE 解压
            let decompressed = Self::rle_decompress(&current_data);
            
            // 再逆变换
            let restored = Self::reverse_transform(&decompressed);
            
            println!("   周期 {}: {} → {} bytes", 
                     cycle, 
                     input_size, 
                     restored.len());
            
            current_data = restored;
        }
        
        // 确保大小正确
        current_data.truncate(orig_size);
        while current_data.len() < orig_size {
            current_data.push(0);
        }
        
        println!("");
        println!("📊 还原结果: {} bytes", current_data.len());
        
        // 写入输出
        let mut output_file = File::create(output_path)?;
        output_file.write_all(&current_data)?;
        
        Ok(())
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║     🌌 Valhalla 还原 (可逆)                                    ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    
    if args.len() < 3 {
        println!("用法: {} <input_dir> <output_file>", args[0]);
        return Ok(());
    }
    
    let input = &args[1];
    let output = &args[2];
    
    ValhallaRestore::restore_file(input, output)?;
    
    println!("\n✅ 还原完成!");
    
    Ok(())
}
