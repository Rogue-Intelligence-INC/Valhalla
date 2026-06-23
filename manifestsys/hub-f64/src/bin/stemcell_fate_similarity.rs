//! StemCell + Fate 相似度互动
//!
//! 输入信号 → Fate 计算签名 → 找相似存储 → 输出关联内容
//! 物理值由 Fate 的 ClusterEquation 给定

use std::path::Path;
use std::io::{self, Write, BufRead};

use stem_cell::StemCellComplete;

fn main() {
    println!("╔════════════════════════════════════════════════════════════════════════════╗");
    println!("║      🧬 Fate 相似度互动                                                     ║");
    println!("║      输入 → Fate签名 → 找相似结构 → 输出                                    ║");
    println!("╚════════════════════════════════════════════════════════════════════════════╝");
    println!();

    // 加载（优先 universal，其次 natural）
    println!("📥 加载系统...");
    let slice_path = if Path::new("/tmp/universal_stemcell.stemcell").exists() {
        Path::new("/tmp/universal_stemcell.stemcell")
    } else {
        Path::new("/tmp/natural_language_stemcell.stemcell")
    };
    
    let mut stemcell = match StemCellComplete::load_slice(slice_path) {
        Ok(s) => {
            println!("   ✓ 细胞: {}", s.cells.len());
            println!("   ✓ Fate存储: {}", s.triple_fate.stored_count());
            println!("   ✓ 方程: {}", s.triple_fate.get_equation());
            s
        }
        Err(e) => {
            println!("   ✗ 加载失败: {}", e);
            return;
        }
    };
    
    // 展示方程参数
    let eq = &stemcell.triple_fate.cluster_equation;
    println!();
    println!("📐 Fate 签名公式:");
    println!("   signature = {:.4}×mean² + {:.4}×variance + {:.4}", 
        eq.coef_a, eq.coef_b, eq.coef_c);
    println!("   阈值: {:.4}", eq.threshold);
    println!();
    
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("   输入文字 → 计算签名 → 找相似存储");
    println!("   'q' 退出 | 'scan' 扫描所有签名分布");
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
            "q" | "quit" => {
                println!("\n👋 再见");
                break;
            }
            "scan" => {
                scan_signatures(&stemcell);
                continue;
            }
            "" => continue,
            _ => {}
        }
        
        // 1. 直接用原始输入，让 Fate 计算签名
        let input_bytes = input.as_bytes();
        let input_sig = stemcell.triple_fate.compute_signature(input_bytes);
        
        println!();
        println!("📊 输入分析:");
        println!("   原始: \"{}\"", input);
        println!("   字节数: {}", input_bytes.len());
        
        // 计算 mean 和 variance（展示计算过程）
        let mean: f64 = input_bytes.iter().map(|&b| b as f64).sum::<f64>() / input_bytes.len() as f64;
        let variance: f64 = input_bytes.iter()
            .map(|&b| {
                let d = b as f64 - mean;
                d * d
            })
            .sum::<f64>() / input_bytes.len() as f64;
        
        println!("   mean: {:.2}", mean);
        println!("   variance: {:.2}", variance);
        println!("   → 签名: {:.4}", input_sig);
        println!();
        
        // 2. 用不同阈值找相似
        let thresholds = [1.0, 5.0, 10.0, 50.0, 100.0];
        
        println!("🔍 相似度搜索:");
        for &thresh in &thresholds {
            let similar = stemcell.triple_fate.find_similar(input_sig, thresh);
            if !similar.is_empty() {
                println!("   阈值 {:.1}: {} 条匹配", thresh, similar.len());
                
                // 展示前 3 个
                for id in similar.iter().take(3) {
                    if let Some(entry) = stemcell.triple_fate.load_data(id) {
                        let text = String::from_utf8_lossy(&entry.raw);
                        let short: String = text.chars()
                            .filter(|c| !c.is_control())
                            .take(80)
                            .collect();
                        println!("      [{:.2}] \"{}...\"", entry.signature, short);
                    }
                }
                
                if similar.len() > 3 {
                    println!("      ... 及其他 {} 条", similar.len() - 3);
                }
                break; // 找到就停
            }
        }
        
        // 3. 如果没找到，显示最接近的
        if thresholds.iter().all(|&t| stemcell.triple_fate.find_similar(input_sig, t).is_empty()) {
            println!("   无精确匹配，寻找最近的...");
            find_closest(&stemcell, input_sig, 5);
        }
        
        println!();
    }
}

/// 扫描所有存储的签名分布
fn scan_signatures(stemcell: &StemCellComplete) {
    println!("\n📈 签名分布扫描:");
    
    let mut signatures: Vec<(String, f64)> = Vec::new();
    
    for id in stemcell.triple_fate.list_stored() {
        if let Some(entry) = stemcell.triple_fate.load_data(id) {
            signatures.push((id.clone(), entry.signature));
        }
    }
    
    if signatures.is_empty() {
        println!("   无存储数据");
        return;
    }
    
    signatures.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    
    let min = signatures.first().unwrap().1;
    let max = signatures.last().unwrap().1;
    let mean = signatures.iter().map(|s| s.1).sum::<f64>() / signatures.len() as f64;
    
    println!("   总数: {}", signatures.len());
    println!("   范围: {:.2} ~ {:.2}", min, max);
    println!("   均值: {:.2}", mean);
    
    // 分桶统计
    let bucket_count = 10;
    let bucket_size = (max - min) / bucket_count as f64;
    let mut buckets = vec![0usize; bucket_count];
    
    for (_, sig) in &signatures {
        let idx = ((sig - min) / bucket_size).floor() as usize;
        let idx = idx.min(bucket_count - 1);
        buckets[idx] += 1;
    }
    
    println!("\n   分布直方图:");
    let max_count = *buckets.iter().max().unwrap_or(&1);
    for (i, &count) in buckets.iter().enumerate() {
        let start = min + i as f64 * bucket_size;
        let end = start + bucket_size;
        let bar_len = (count as f64 / max_count as f64 * 30.0) as usize;
        let bar: String = "█".repeat(bar_len);
        println!("   [{:7.1}-{:7.1}] {:4} {}", start, end, count, bar);
    }
    
    // 展示几个样本
    println!("\n   样本:");
    println!("   最小: [{:.2}]", signatures[0].1);
    if let Some(entry) = stemcell.triple_fate.load_data(&signatures[0].0) {
        let text: String = String::from_utf8_lossy(&entry.raw).chars().take(60).collect();
        println!("         \"{}...\"", text.replace('\n', " "));
    }
    
    let mid = signatures.len() / 2;
    println!("   中间: [{:.2}]", signatures[mid].1);
    if let Some(entry) = stemcell.triple_fate.load_data(&signatures[mid].0) {
        let text: String = String::from_utf8_lossy(&entry.raw).chars().take(60).collect();
        println!("         \"{}...\"", text.replace('\n', " "));
    }
    
    println!("   最大: [{:.2}]", signatures.last().unwrap().1);
    if let Some(entry) = stemcell.triple_fate.load_data(&signatures.last().unwrap().0) {
        let text: String = String::from_utf8_lossy(&entry.raw).chars().take(60).collect();
        println!("         \"{}...\"", text.replace('\n', " "));
    }
    
    println!();
}

/// 找最接近的 N 个
fn find_closest(stemcell: &StemCellComplete, target_sig: f64, count: usize) {
    let mut distances: Vec<(String, f64, f64)> = Vec::new();
    
    for id in stemcell.triple_fate.list_stored() {
        if let Some(entry) = stemcell.triple_fate.load_data(id) {
            let dist = (entry.signature - target_sig).abs();
            distances.push((id.clone(), entry.signature, dist));
        }
    }
    
    distances.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());
    
    println!("   最近 {} 个:", count);
    for (id, sig, dist) in distances.iter().take(count) {
        if let Some(entry) = stemcell.triple_fate.load_data(id) {
            let text: String = String::from_utf8_lossy(&entry.raw)
                .chars()
                .filter(|c| !c.is_control())
                .take(70)
                .collect();
            println!("   [{:.2}] (距离:{:.2}) \"{}...\"", sig, dist, text);
        }
    }
}
