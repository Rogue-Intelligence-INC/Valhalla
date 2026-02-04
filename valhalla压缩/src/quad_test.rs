//! 🌌 四元系统独立测试
//!
//! 分别测试: BlackHole, MeshBrain, MultiNova, BaseForce
//! 纯二进制处理，不走 Hub

use std::fs::File;
use std::io::{self, Read, Write};

/// BlackHole - 吸收高密度信息
mod blackhole {
    pub fn process(data: &[u8]) -> Vec<u8> {
        // 黑洞: XOR 折叠 - 将数据压缩到极限
        let mut result = Vec::new();
        
        // 每 8 字节折叠为 1 字节
        for chunk in data.chunks(8) {
            let folded = chunk.iter().fold(0u8, |acc, &b| acc ^ b);
            result.push(folded);
        }
        
        result
    }
    
    pub fn restore(data: &[u8], orig_len: usize) -> Vec<u8> {
        // 黑洞无法完全还原 (信息被吸收)
        // 只能生成"近似"
        let mut result = Vec::with_capacity(orig_len);
        
        for &b in data {
            // 从折叠值展开为 8 字节
            for i in 0..8 {
                result.push(b.rotate_left(i as u32));
            }
        }
        
        result.truncate(orig_len);
        result
    }
}

/// MeshBrain - 网络连接结构
mod meshbrain {
    pub fn process(data: &[u8]) -> Vec<u8> {
        // 膜: 差分编码 - 保留结构关系
        let mut result = Vec::with_capacity(data.len());
        
        let mut prev = 0u8;
        for &b in data {
            result.push(b.wrapping_sub(prev));
            prev = b;
        }
        
        result
    }
    
    pub fn restore(data: &[u8], _orig_len: usize) -> Vec<u8> {
        // 膜可以完全还原
        let mut result = Vec::with_capacity(data.len());
        
        let mut prev = 0u8;
        for &b in data {
            let orig = b.wrapping_add(prev);
            result.push(orig);
            prev = orig;
        }
        
        result
    }
}

/// MultiNova - 多重爆发分布
mod multinova {
    pub fn process(data: &[u8]) -> Vec<u8> {
        // 多星: 位分散 - 将每个字节的位分散到多个位置
        let mut result = vec![0u8; data.len()];
        
        for (i, &b) in data.iter().enumerate() {
            // 每个 bit 分散到不同位置
            for bit in 0..8 {
                let target_idx = (i + bit * data.len() / 8) % data.len();
                if (b >> bit) & 1 == 1 {
                    result[target_idx] ^= 1 << (i % 8);
                }
            }
        }
        
        result
    }
    
    pub fn restore(data: &[u8], _orig_len: usize) -> Vec<u8> {
        // 多星可以还原 (逆向位分散)
        let mut result = vec![0u8; data.len()];
        
        for (i, &b) in data.iter().enumerate() {
            for bit in 0..8 {
                let source_idx = (i + bit * data.len() / 8) % data.len();
                if (data[source_idx] >> (i % 8)) & 1 == 1 {
                    result[i] |= 1 << bit;
                }
            }
        }
        
        result
    }
}

/// BaseForce - 基础力场
mod baseforce {
    pub fn process(data: &[u8]) -> Vec<u8> {
        if data.is_empty() {
            return Vec::new();
        }
        
        // 基底: 提取基准 + 存储偏移
        let base = data.iter().map(|&b| b as u32).sum::<u32>() / data.len() as u32;
        let base = base as u8;
        
        let mut result = Vec::with_capacity(data.len() + 1);
        result.push(base);  // 存储基准
        
        for &b in data {
            result.push(b.wrapping_sub(base));
        }
        
        result
    }
    
    pub fn restore(data: &[u8], _orig_len: usize) -> Vec<u8> {
        if data.is_empty() {
            return Vec::new();
        }
        
        // 基底可以完全还原
        let base = data[0];
        let mut result = Vec::with_capacity(data.len() - 1);
        
        for &b in &data[1..] {
            result.push(b.wrapping_add(base));
        }
        
        result
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║     🌌 四元系统独立测试                                        ║");
    println!("║     BlackHole | MeshBrain | MultiNova | BaseForce              ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    
    if args.len() < 2 {
        println!("用法: {} <input_file>", args[0]);
        return Ok(());
    }
    
    // 读取文件为纯二进制
    let mut file = File::open(&args[1])?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    
    println!("📥 输入: {}", args[1]);
    println!("📊 原始大小: {} bytes", data.len());
    println!("📊 前 16 字节: {:02x?}", &data[..16.min(data.len())]);
    println!("");
    
    // 测试每个系统
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    // 1. BlackHole
    println!("\n🕳️  BlackHole (黑洞 - 极限压缩)");
    let bh_out = blackhole::process(&data);
    let bh_restore = blackhole::restore(&bh_out, data.len());
    let bh_match = bh_restore == data;
    println!("   {} → {} bytes ({:.1}%)", data.len(), bh_out.len(), 
             bh_out.len() as f64 / data.len() as f64 * 100.0);
    println!("   还原: {} ({})", if bh_match { "✅" } else { "❌ 有损" }, 
             if bh_match { "完美" } else { "信息被吸收" });
    
    // 2. MeshBrain
    println!("\n🧠 MeshBrain (膜 - 差分结构)");
    let mb_out = meshbrain::process(&data);
    let mb_restore = meshbrain::restore(&mb_out, data.len());
    let mb_match = mb_restore == data;
    println!("   {} → {} bytes ({:.1}%)", data.len(), mb_out.len(),
             mb_out.len() as f64 / data.len() as f64 * 100.0);
    println!("   还原: {} ({})", if mb_match { "✅" } else { "❌" },
             if mb_match { "完美" } else { "错误" });
    
    // 3. MultiNova
    println!("\n⭐ MultiNova (多星 - 位分散)");
    let mn_out = multinova::process(&data);
    let mn_restore = multinova::restore(&mn_out, data.len());
    let mn_match = mn_restore == data;
    println!("   {} → {} bytes ({:.1}%)", data.len(), mn_out.len(),
             mn_out.len() as f64 / data.len() as f64 * 100.0);
    println!("   还原: {} ({})", if mn_match { "✅" } else { "❌" },
             if mn_match { "完美" } else { "错误" });
    
    // 4. BaseForce
    println!("\n⚡ BaseForce (基底 - 基准偏移)");
    let bf_out = baseforce::process(&data);
    let bf_restore = baseforce::restore(&bf_out, data.len());
    let bf_match = bf_restore == data;
    println!("   {} → {} bytes ({:.1}%)", data.len(), bf_out.len(),
             bf_out.len() as f64 / data.len() as f64 * 100.0);
    println!("   还原: {} ({})", if bf_match { "✅" } else { "❌" },
             if bf_match { "完美" } else { "错误" });
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    // 总结
    println!("\n📊 总结:");
    println!("   ┌─────────────┬──────────┬─────────┬────────┐");
    println!("   │ 系统        │ 压缩比   │ 可还原  │ 特性   │");
    println!("   ├─────────────┼──────────┼─────────┼────────┤");
    println!("   │ BlackHole   │ {:.1}%    │ {}      │ 有损   │", 
             bh_out.len() as f64 / data.len() as f64 * 100.0,
             if bh_match { "✅" } else { "❌" });
    println!("   │ MeshBrain   │ {:.1}%   │ {}      │ 无损   │",
             mb_out.len() as f64 / data.len() as f64 * 100.0,
             if mb_match { "✅" } else { "❌" });
    println!("   │ MultiNova   │ {:.1}%   │ {}      │ 无损   │",
             mn_out.len() as f64 / data.len() as f64 * 100.0,
             if mn_match { "✅" } else { "❌" });
    println!("   │ BaseForce   │ {:.1}%   │ {}      │ 无损   │",
             bf_out.len() as f64 / data.len() as f64 * 100.0,
             if bf_match { "✅" } else { "❌" });
    println!("   └─────────────┴──────────┴─────────┴────────┘");
    
    // 保存各系统输出
    let base_name = args[1].rsplit('/').next().unwrap_or("data");
    std::fs::write(format!("/tmp/{}.blackhole", base_name), &bh_out)?;
    std::fs::write(format!("/tmp/{}.meshbrain", base_name), &mb_out)?;
    std::fs::write(format!("/tmp/{}.multinova", base_name), &mn_out)?;
    std::fs::write(format!("/tmp/{}.baseforce", base_name), &bf_out)?;
    
    println!("\n📁 输出文件:");
    println!("   /tmp/{}.blackhole  ({} bytes)", base_name, bh_out.len());
    println!("   /tmp/{}.meshbrain  ({} bytes)", base_name, mb_out.len());
    println!("   /tmp/{}.multinova  ({} bytes)", base_name, mn_out.len());
    println!("   /tmp/{}.baseforce  ({} bytes)", base_name, bf_out.len());
    
    Ok(())
}
