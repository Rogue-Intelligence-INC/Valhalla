//! StemCell 自由互动模块
//!
//! 特点：
//! - 任意输入都能激活系统
//! - 基于关联传播，不是问答匹配
//! - 自由输出语言/文字
//! - 系统可自发产生联想

use std::path::Path;
use std::io::{self, Write, BufRead};
use std::collections::{HashSet, VecDeque};

use stem_cell::StemCellComplete;
use fate_f64::TripleFateCombined;

fn main() {
    println!("╔════════════════════════════════════════════════════════════════════════════╗");
    println!("║          🧬 StemCell 自由互动                                               ║");
    println!("║          关联激活 · 自由输出 · 文字通道                                     ║");
    println!("╚════════════════════════════════════════════════════════════════════════════╝");
    println!();

    // 加载训练后的完整状态（包含所有细胞物理数据）
    println!("📥 加载完整系统状态...");
    // 优先加载 universal 训练结果
    let slice_path = if Path::new("/tmp/universal_stemcell.stemcell").exists() {
        Path::new("/tmp/universal_stemcell.stemcell")
    } else {
        Path::new("/tmp/natural_language_stemcell.stemcell")
    };
    
    let mut stemcell = match StemCellComplete::load_slice(slice_path) {
        Ok(s) => {
            println!("   ✓ 完整恢复成功");
            println!("   ✓ 细胞数: {}", s.cells.len());
            println!("   ✓ Fate 存储: {} 条", s.triple_fate.stored_count());
            println!("   ✓ 方程: {}", s.triple_fate.get_equation());
            
            // 统计连接
            let total_conn: usize = s.cells.iter().map(|c| c.connections.len()).sum();
            println!("   ✓ 总连接: {}", total_conn);
            s
        }
        Err(e) => {
            println!("   ✗ 加载失败: {}", e);
            println!("   尝试旧格式...");
            
            // 尝试旧的 Fate 格式
            let old_path = Path::new("/tmp/unified_stemcell.fate");
            if let Ok(fate) = TripleFateCombined::load_slice(old_path) {
                println!("   ✓ 旧格式 Fate 加载成功");
                let mut s = StemCellComplete::new();
                s.triple_fate = fate;
                s
            } else {
                println!("   使用空系统");
                StemCellComplete::new()
            }
        }
    };
    println!();

    // 构建 Fate ID 索引（用于快速查找）
    let fate_ids: Vec<String> = stemcell.triple_fate.list_stored()
        .iter()
        .map(|s| (*s).clone())
        .collect();
    
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("   命令：");
    println!("   - 输入任意文字 → 触发关联激活");
    println!("   - 空回车 → 系统自发联想");
    println!("   - 'q' 或 'quit' → 退出");
    println!("   - 'status' → 显示系统状态");
    println!("   - 'flow' → 连续自由流动模式");
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!();

    let stdin = io::stdin();
    let mut input_buffer = String::new();
    
    loop {
        print!("💭 > ");
        io::stdout().flush().ok();
        
        input_buffer.clear();
        if stdin.lock().read_line(&mut input_buffer).is_err() {
            break;
        }
        
        let input = input_buffer.trim();
        
        match input {
            "q" | "quit" | "exit" => {
                println!("\n👋 再见");
                break;
            }
            "status" => {
                print_status(&stemcell);
                continue;
            }
            "flow" => {
                free_flow_mode(&mut stemcell, &fate_ids, 10);
                continue;
            }
            _ => {}
        }
        
        // 激活并输出
        let output = activate_and_generate(&mut stemcell, &fate_ids, input);
        println!();
        println!("🔮 输出：");
        println!("────────────────────────────────────────────────────────────────");
        println!("{}", output);
        println!("────────────────────────────────────────────────────────────────");
        println!();
    }
}

/// 激活并生成文字输出（基于 Fate 签名相似度）
fn activate_and_generate(
    stemcell: &mut StemCellComplete,
    _fate_ids: &[String],
    input: &str,
) -> String {
    // 1. 计算输入的 Fate 签名
    let input_bytes = if input.is_empty() {
        // 空输入：用系统 tick 生成随机种子
        let seed = stemcell.triple_fate.global_tick;
        vec![(seed % 256) as u8, ((seed / 256) % 256) as u8, ((seed / 65536) % 256) as u8]
    } else {
        input.as_bytes().to_vec()
    };
    
    let input_sig = stemcell.triple_fate.compute_signature(&input_bytes);
    
    // 2. 让信号流过系统
    for &b in &input_bytes {
        stemcell.receive_data(b as f64);
    }
    stemcell.evolve(0.3);
    
    // 3. 用 Fate 签名相似度找最近的存储内容
    let mut matches: Vec<(f64, String, String)> = Vec::new(); // (距离, id, 内容)
    
    for id in stemcell.triple_fate.list_stored() {
        if let Some(entry) = stemcell.triple_fate.load_data(id) {
            let dist = (entry.signature - input_sig).abs();
            if let Some(text) = extract_text_fragment(&entry.raw, 200) {
                matches.push((dist, id.clone(), text));
            }
        }
    }
    
    // 按距离排序（最近的在前）
    matches.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    
    if matches.is_empty() {
        return format!("〔系统沉默〕\n（签名: {:.2}）", input_sig);
    }
    
    // 4. 构建输出
    let mut output = String::new();
    output.push_str(&format!("〔签名: {:.2}〕\n\n", input_sig));
    
    // 主要输出（最相似）
    if let Some((dist, _id, text)) = matches.first() {
        output.push_str(&format!("▸ [距离:{:.1}] {}\n", dist, text));
    }
    
    // 次要输出
    for (dist, _id, text) in matches.iter().skip(1).take(2) {
        output.push_str(&format!("\n  ▹ [距离:{:.1}] {}", dist, text));
    }
    
    output
}

/// 自由流动模式 - 连续自发输出
fn free_flow_mode(
    stemcell: &mut StemCellComplete,
    fate_ids: &[String],
    cycles: usize,
) {
    println!("\n🌊 自由流动模式 ({} 周期):\n", cycles);
    println!("════════════════════════════════════════════════════════════════════");
    
    for i in 1..=cycles {
        // 无输入，纯内部激活
        let output = activate_and_generate(stemcell, fate_ids, "");
        
        println!("〔流动 {}〕", i);
        println!("{}", output);
        println!();
        
        // 小延迟让系统演化
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    
    println!("════════════════════════════════════════════════════════════════════");
}

/// 从原始数据提取文字片段（安全处理 UTF-8）
fn extract_text_fragment(raw: &[u8], max_chars: usize) -> Option<String> {
    // 尝试解析为 UTF-8
    let text = String::from_utf8_lossy(raw);
    
    // 清理并截取（按字符数，不是字节数）
    let cleaned: String = text
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '。' || *c == '，' || *c == '、')
        .take(max_chars)
        .collect();
    
    let trimmed = cleaned.trim();
    
    if trimmed.is_empty() || trimmed.len() < 5 {
        return None;
    }
    
    Some(trimmed.to_string())
}

/// 显示系统状态
fn print_status(stemcell: &StemCellComplete) {
    println!();
    println!("📊 系统状态:");
    println!("   ├─ 细胞数: {}", stemcell.cells.len());
    println!("   ├─ Fate 存储: {}", stemcell.triple_fate.stored_count());
    println!("   ├─ Fate tick: {}", stemcell.triple_fate.global_tick);
    println!("   ├─ 方程: {}", stemcell.triple_fate.get_equation());
    println!("   └─ 收敛: {}", if stemcell.triple_fate.is_passively_converged() { "是" } else { "否" });
    
    // 连接统计
    let total_conn: usize = stemcell.cells.iter().map(|c| c.connections.len()).sum();
    let avg_conn = total_conn as f64 / stemcell.cells.len().max(1) as f64;
    println!("   连接统计: 总{}, 平均{:.1}/细胞", total_conn, avg_conn);
    println!();
}
