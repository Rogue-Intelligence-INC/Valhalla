//! StemCell 交互测试 - 无预设类型，观察涌现结构
//! 
//! 输入文本，观察细胞网络响应

use std::io::{self, Write, BufRead};

use stem_cell::stemcell_complete::{StemCellComplete, StemCellConfig};

fn main() {
    eprintln!("╔═══════════════════════════════════════════════════════════════════╗");
    eprintln!("║           StemCell 交互测试 - 自由涌现                            ║");
    eprintln!("╚═══════════════════════════════════════════════════════════════════╝");
    eprintln!();
    
    // 配置
    let config = StemCellConfig {
        initial_cell_count: 100,
        mutation_rate: 0.10,
        container_capacity: 100,
    };
    
    let mut stemcell = StemCellComplete::with_config(config);
    
    // 预热：注入一些初始数据
    eprintln!("🔥 预热细胞网络...");
    for i in 0..5000 {
        let data = (i as f64 * 0.01).sin() * 0.5;
        stemcell.receive_data(data);
    }
    
    // 执行几代演化
    for _ in 0..3 {
        stemcell.process_cells_cycle();
    }
    
    let clusters = analyze_clusters(&stemcell);
    eprintln!("   ✓ 预热完成: {} 细胞, {} 个聚类涌现", stemcell.cells.len(), clusters.len());
    for (i, (center, count, variance)) in clusters.iter().enumerate() {
        eprintln!("     聚类{}: 中心={:.3}, 数量={}, 方差={:.4}", i + 1, center, count, variance);
    }
    eprintln!();
    
    eprintln!("💬 输入文本测试细胞网络响应 (输入 'quit' 退出, 'status' 查看状态)");
    eprintln!("═══════════════════════════════════════════════════════════════════");
    eprintln!();
    
    let stdin = io::stdin();
    loop {
        // 显示提示符
        eprint!("你> ");
        io::stderr().flush().unwrap();
        
        // 读取输入
        let mut input = String::new();
        if stdin.lock().read_line(&mut input).is_err() {
            break;
        }
        let input = input.trim();
        
        if input.is_empty() {
            continue;
        }
        
        if input == "quit" || input == "exit" {
            eprintln!("👋 再见!");
            break;
        }
        
        if input == "status" {
            show_status(&stemcell);
            continue;
        }
        
        if input == "evolve" {
            eprintln!("🧬 执行演化...");
            let result = stemcell.process_cells_cycle();
            eprintln!("   细胞={} 分裂={} 死亡={} 合并={} 能量={:.4}",
                result.cell_count, result.divisions, result.deaths, result.merges, result.avg_energy);
            eprintln!();
            continue;
        }
        
        // 记录处理前状态
        let before_cells = stemcell.cells.len();
        let before_energy: f64 = stemcell.cells.iter().map(|c| c.energy).sum();
        
        // 将输入文本转换为信号并输入
        let mut signals_sent = 0;
        for ch in input.chars() {
            let v = (ch as u32 as f64 / 65536.0) * 2.0 - 1.0;
            stemcell.receive_data(v);
            signals_sent += 1;
        }
        
        // 记录处理后状态
        let after_cells = stemcell.cells.len();
        let after_energy: f64 = stemcell.cells.iter().map(|c| c.energy).sum();
        
        let cell_change = after_cells as i64 - before_cells as i64;
        let energy_change = after_energy - before_energy;
        
        // 生成响应
        let response = generate_response(&stemcell, input);
        
        eprintln!();
        eprintln!("🧬 细胞网络响应:");
        eprintln!("   输入信号: {} 个", signals_sent);
        eprintln!("   细胞变化: {:+} (当前: {})", cell_change, after_cells);
        eprintln!("   能量变化: {:+.4} (总能量: {:.2})", energy_change, after_energy);
        eprintln!();
        eprintln!("💭 {}", response);
        eprintln!();
    }
}

fn show_status(stemcell: &StemCellComplete) {
    let status = stemcell.get_status();
    let clusters = analyze_clusters(stemcell);
    
    eprintln!();
    eprintln!("📊 当前状态:");
    eprintln!("   细胞总数: {}", stemcell.cells.len());
    eprintln!("   总分裂: {}", status.total_divisions);
    eprintln!("   总死亡: {}", status.total_deaths);
    eprintln!("   总合并: {}", status.total_merges);
    eprintln!("   维度: {:.2}", status.dimension);
    eprintln!();
    eprintln!("   涌现聚类 ({} 个):", clusters.len());
    for (i, (center, count, variance)) in clusters.iter().enumerate() {
        let bar_len = (*count as f64 / stemcell.cells.len() as f64 * 20.0) as usize;
        let bar = "█".repeat(bar_len.max(1));
        eprintln!("     聚类{}: 中心={:+.3} 数量={:4} 方差={:.4} {}", 
            i + 1, center, count, variance, bar);
    }
    eprintln!();
}

fn generate_response(stemcell: &StemCellComplete, input: &str) -> String {
    let cell_count = stemcell.cells.len();
    let total_energy: f64 = stemcell.cells.iter().map(|c| c.energy).sum();
    let avg_energy = if cell_count > 0 { total_energy / cell_count as f64 } else { 0.0 };
    
    // 分析涌现的聚类结构
    let clusters = analyze_clusters(stemcell);
    
    let input_len = input.len();
    
    if cell_count == 0 {
        return "细胞网络已耗尽...".to_string();
    }
    
    // 基于能量生成响应
    let energy_level = if avg_energy > 100.0 { "高能量" }
        else if avg_energy > 10.0 { "正常" }
        else { "低能量" };
    
    // 基于聚类结构生成响应
    let structure_desc = match clusters.len() {
        0 => "无结构",
        1 => "单核",
        2 => "双核",
        3..=5 => "多核",
        _ => "复杂网络",
    };
    
    // 根据输入特征生成不同响应
    let response_type = if input.contains('?') || input.contains('？') {
        "疑问"
    } else if input.contains('!') || input.contains('！') {
        "感叹"
    } else if input_len > 20 {
        "长文本"
    } else {
        "一般"
    };
    
    format!("[{} | {} | {} 细胞 | {} 聚类] 已处理 {} 字符的{}输入",
        energy_level, structure_desc, cell_count, clusters.len(), input_len, response_type)
}

/// 分析细胞的聚类结构（基于数据特征，无预设类型）
/// 返回: Vec<(聚类中心, 细胞数量, 聚类内方差)>
fn analyze_clusters(stemcell: &StemCellComplete) -> Vec<(f64, usize, f64)> {
    if stemcell.cells.is_empty() {
        return Vec::new();
    }
    
    // 提取所有细胞的第一个数据特征值（信号中心）
    let mut signatures: Vec<f64> = stemcell.cells.iter()
        .filter_map(|c| c.data_signature.first().copied())
        .collect();
    
    if signatures.is_empty() {
        return Vec::new();
    }
    
    signatures.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    
    // 用代数方法（边际递减）找聚类边界
    // 相邻值差异呈边际递减的位置 = 聚类内部
    // 差异突然增大的位置 = 聚类边界
    let mut clusters: Vec<(f64, usize, f64)> = Vec::new();
    let mut cluster_start = 0;
    
    for i in 1..signatures.len() {
        let diff = (signatures[i] - signatures[i - 1]).abs();
        
        // 如果差异大于平均差异的2倍，认为是聚类边界
        let avg_diff = if i > 0 {
            (signatures[i] - signatures[0]) / i as f64
        } else {
            diff
        };
        
        if diff > avg_diff.abs() * 2.0 || i == signatures.len() - 1 {
            // 结束当前聚类
            let cluster_values = &signatures[cluster_start..=i.min(signatures.len() - 1)];
            if !cluster_values.is_empty() {
                let center = cluster_values.iter().sum::<f64>() / cluster_values.len() as f64;
                let variance = cluster_values.iter()
                    .map(|v| (v - center).powi(2))
                    .sum::<f64>() / cluster_values.len() as f64;
                clusters.push((center, cluster_values.len(), variance));
            }
            cluster_start = i;
        }
    }
    
    // 合并太小的聚类
    clusters.retain(|(_, count, _)| *count >= 2);
    
    // 按数量排序
    clusters.sort_by(|a, b| b.1.cmp(&a.1));
    
    // 只返回主要聚类
    clusters.truncate(10);
    
    clusters
}
