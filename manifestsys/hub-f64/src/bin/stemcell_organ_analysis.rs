//! StemCell 器官行为与细胞特征分析
//!
//! 分析训练后的 StemCell：
//! - 细胞特征分布
//! - 类器官聚集行为
//! - 功能分化

use std::path::Path;
use std::collections::HashMap;

use stem_cell::{StemCellComplete, CompleteCell as Cell};

fn main() {
    println!("═══════════════════════════════════════════════════════════════════════");
    println!("       StemCell 器官行为与细胞特征分析");
    println!("═══════════════════════════════════════════════════════════════════════");
    println!();
    
    // 先进行训练
    println!("🏋️ 进行训练 (5 轮)...");
    let mut stemcell = StemCellComplete::new();
    
    // 加载语料
    let corpus = load_corpus();
    println!("   语料: {} 文件, {} KB", corpus.len(), 
        corpus.iter().map(|(_, d)| d.len()).sum::<usize>() / 1024);
    
    for round in 1..=5 {
        print!("   第 {} 轮: ", round);
        let mut new_cells = 0;
        for (_, bytes) in &corpus {
            for &b in bytes.iter() {
                let result = stemcell.receive_data(b as f64);
                new_cells += result.new_cells;
            }
        }
        println!("{} 新细胞, 总 {} 细胞", new_cells, stemcell.cells.len());
    }
    println!();
    
    // 基本状态
    let status = stemcell.get_status();
    println!("📊 基本状态:");
    println!("   细胞数: {}", status.cell_count);
    println!("   世代: {}", status.generation);
    println!("   总分裂: {}", status.total_divisions);
    println!("   总合并: {}", status.total_merges);
    println!("   Fate Tick: {}", status.fate_tick);
    println!();
    
    // 获取所有细胞进行分析
    let cells = &stemcell.cells;
    
    if cells.is_empty() {
        println!("⚠️  没有细胞可分析");
        return;
    }
    
    println!("═══════════════════════════════════════════════════════════════════════");
    println!("                    细胞特征分析");
    println!("═══════════════════════════════════════════════════════════════════════");
    println!();
    
    // 1. 能量分布
    let energies: Vec<f64> = cells.iter().map(|c| c.energy).collect();
    let energy_stats = calculate_stats(&energies);
    println!("⚡ 能量分布:");
    println!("   最小: {:.4}", energy_stats.min);
    println!("   最大: {:.4}", energy_stats.max);
    println!("   平均: {:.4}", energy_stats.mean);
    println!("   标准差: {:.4}", energy_stats.std);
    println!();
    
    // 能量区间分布
    let mut energy_buckets = [0usize; 10];
    for &e in &energies {
        let bucket = ((e * 10.0).floor() as usize).min(9);
        energy_buckets[bucket] += 1;
    }
    println!("   能量直方图:");
    for (i, &count) in energy_buckets.iter().enumerate() {
        let bar = "█".repeat((count as f64 / cells.len() as f64 * 50.0) as usize);
        println!("   {:.1}-{:.1}: {:>6} {}", i as f64 / 10.0, (i + 1) as f64 / 10.0, count, bar);
    }
    println!();
    
    // 2. 年龄分布
    let ages: Vec<f64> = cells.iter().map(|c| c.age as f64).collect();
    let age_stats = calculate_stats(&ages);
    println!("🕐 年龄分布:");
    println!("   最小: {}", age_stats.min as u64);
    println!("   最大: {}", age_stats.max as u64);
    println!("   平均: {:.2}", age_stats.mean);
    println!("   标准差: {:.2}", age_stats.std);
    println!();
    
    // 3. 信号计数分布
    let signal_counts: Vec<f64> = cells.iter().map(|c| c.signal_count as f64).collect();
    let signal_stats = calculate_stats(&signal_counts);
    println!("📡 信号计数分布:");
    println!("   最小: {}", signal_stats.min as u64);
    println!("   最大: {}", signal_stats.max as u64);
    println!("   平均: {:.2}", signal_stats.mean);
    println!("   标准差: {:.2}", signal_stats.std);
    println!();
    
    // 4. 方差分布
    let variances: Vec<f64> = cells.iter().map(|c| c.variance).collect();
    let var_stats = calculate_stats(&variances);
    println!("📈 方差分布:");
    println!("   最小: {:.6}", var_stats.min);
    println!("   最大: {:.6}", var_stats.max);
    println!("   平均: {:.6}", var_stats.mean);
    println!("   标准差: {:.6}", var_stats.std);
    println!();
    
    // 5. 世代分布
    let generations: Vec<f64> = cells.iter().map(|c| c.generation as f64).collect();
    let mut gen_counts: HashMap<u32, usize> = HashMap::new();
    for c in cells.iter() {
        *gen_counts.entry(c.generation).or_insert(0) += 1;
    }
    println!("🧬 世代分布:");
    let mut gen_vec: Vec<_> = gen_counts.iter().collect();
    gen_vec.sort_by_key(|(g, _)| *g);
    for (gen, count) in gen_vec.iter().take(10) {
        let pct = **count as f64 / cells.len() as f64 * 100.0;
        println!("   世代 {}: {} 个 ({:.1}%)", gen, count, pct);
    }
    if gen_vec.len() > 10 {
        println!("   ... 还有 {} 个世代", gen_vec.len() - 10);
    }
    println!();
    
    // 6. 数据特征分析
    println!("═══════════════════════════════════════════════════════════════════════");
    println!("                    数据特征分析 (细胞身份)");
    println!("═══════════════════════════════════════════════════════════════════════");
    println!();
    
    let cells_with_signature: Vec<_> = cells.iter()
        .filter(|c| !c.data_signature.is_empty())
        .collect();
    
    println!("📝 有数据特征的细胞: {} / {} ({:.1}%)", 
        cells_with_signature.len(), 
        cells.len(),
        cells_with_signature.len() as f64 / cells.len() as f64 * 100.0
    );
    
    if !cells_with_signature.is_empty() {
        // 特征维度分布
        let sig_lens: Vec<usize> = cells_with_signature.iter()
            .map(|c| c.data_signature.len())
            .collect();
        let min_len = *sig_lens.iter().min().unwrap();
        let max_len = *sig_lens.iter().max().unwrap();
        let avg_len = sig_lens.iter().sum::<usize>() as f64 / sig_lens.len() as f64;
        
        println!("   特征维度: {} ~ {} (平均 {:.1})", min_len, max_len, avg_len);
        
        // 特征聚类分析 (简单 K-means 思想)
        println!();
        println!("🔬 特征聚类分析:");
        
        // 计算特征均值
        let sig_means: Vec<f64> = cells_with_signature.iter()
            .map(|c| {
                if c.data_signature.is_empty() { 0.0 }
                else { c.data_signature.iter().sum::<f64>() / c.data_signature.len() as f64 }
            })
            .collect();
        
        // 特征方差
        let sig_vars: Vec<f64> = cells_with_signature.iter()
            .map(|c| {
                if c.data_signature.len() < 2 { return 0.0; }
                let mean = c.data_signature.iter().sum::<f64>() / c.data_signature.len() as f64;
                c.data_signature.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / c.data_signature.len() as f64
            })
            .collect();
        
        let mean_stats = calculate_stats(&sig_means);
        let var_stats = calculate_stats(&sig_vars);
        
        println!("   特征均值: {:.4} ± {:.4}", mean_stats.mean, mean_stats.std);
        println!("   特征方差: {:.4} ± {:.4}", var_stats.mean, var_stats.std);
        
        // 简单聚类：按特征均值分组
        let mut low_mean = 0;
        let mut mid_mean = 0;
        let mut high_mean = 0;
        
        for &m in &sig_means {
            if m < mean_stats.mean - mean_stats.std {
                low_mean += 1;
            } else if m > mean_stats.mean + mean_stats.std {
                high_mean += 1;
            } else {
                mid_mean += 1;
            }
        }
        
        println!();
        println!("   特征分组 (按均值):");
        println!("   └─ 低均值组: {} 个 ({:.1}%)", low_mean, low_mean as f64 / cells_with_signature.len() as f64 * 100.0);
        println!("   └─ 中均值组: {} 个 ({:.1}%)", mid_mean, mid_mean as f64 / cells_with_signature.len() as f64 * 100.0);
        println!("   └─ 高均值组: {} 个 ({:.1}%)", high_mean, high_mean as f64 / cells_with_signature.len() as f64 * 100.0);
    }
    println!();
    
    // 7. 类器官结构检测
    println!("═══════════════════════════════════════════════════════════════════════");
    println!("                    类器官结构检测");
    println!("═══════════════════════════════════════════════════════════════════════");
    println!();
    
    // 基于能量和信号的功能分化
    let mut functional_groups: HashMap<&str, Vec<usize>> = HashMap::new();
    
    for (i, c) in cells.iter().enumerate() {
        let category = classify_cell(c);
        functional_groups.entry(category).or_insert_with(Vec::new).push(i);
    }
    
    println!("🏛️  功能分化 (类器官):");
    let mut groups: Vec<_> = functional_groups.iter().collect();
    groups.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
    
    for (name, indices) in groups.iter() {
        let pct = indices.len() as f64 / cells.len() as f64 * 100.0;
        let bar = "█".repeat((pct * 0.5) as usize);
        println!("   {:<20}: {:>6} 个 ({:>5.1}%) {}", name, indices.len(), pct, bar);
        
        // 该组的统计
        if indices.len() >= 2 {
            let group_energies: Vec<f64> = indices.iter().map(|&i| cells[i].energy).collect();
            let group_signals: Vec<f64> = indices.iter().map(|&i| cells[i].signal_count as f64).collect();
            let e_stats = calculate_stats(&group_energies);
            let s_stats = calculate_stats(&group_signals);
            println!("      能量: {:.3} ± {:.3}, 信号: {:.0} ± {:.0}", 
                e_stats.mean, e_stats.std, s_stats.mean, s_stats.std);
        }
    }
    println!();
    
    // 8. 协同行为检测
    println!("═══════════════════════════════════════════════════════════════════════");
    println!("                    协同行为检测");
    println!("═══════════════════════════════════════════════════════════════════════");
    println!();
    
    // 能量-信号相关性
    if cells.len() >= 2 {
        let correlation = calculate_correlation(&energies, &signal_counts);
        println!("📊 能量-信号相关性: {:.4}", correlation);
        
        if correlation > 0.5 {
            println!("   → 强正相关: 高能量细胞接收更多信号 (类似神经元)");
        } else if correlation < -0.5 {
            println!("   → 强负相关: 低能量细胞接收更多信号 (类似感受器)");
        } else if correlation.abs() < 0.1 {
            println!("   → 无明显相关: 功能独立分化");
        } else {
            println!("   → 弱相关: 部分功能耦合");
        }
    }
    
    // 年龄-能量相关性
    if cells.len() >= 2 {
        let age_energy_corr = calculate_correlation(&ages, &energies);
        println!("📊 年龄-能量相关性: {:.4}", age_energy_corr);
        
        if age_energy_corr > 0.3 {
            println!("   → 老细胞能量更高 (能量积累)");
        } else if age_energy_corr < -0.3 {
            println!("   → 新细胞能量更高 (快速代谢)");
        }
    }
    println!();
    
    // 9. 总结
    println!("═══════════════════════════════════════════════════════════════════════");
    println!("                    分析总结");
    println!("═══════════════════════════════════════════════════════════════════════");
    println!();
    
    let diversity_score = calculate_diversity(&energies, &signal_counts, &variances);
    let organ_score = calculate_organ_score(&functional_groups, cells.len());
    
    println!("🎯 细胞多样性指数: {:.2}/10", diversity_score);
    println!("🎯 类器官分化指数: {:.2}/10", organ_score);
    println!();
    
    if diversity_score > 7.0 {
        println!("✅ 细胞特征高度分化，形成了明显的功能区分");
    } else if diversity_score > 4.0 {
        println!("⚠️  细胞特征中度分化，有一定的功能区分");
    } else {
        println!("❌ 细胞特征低度分化，尚未形成明显功能区分");
    }
    
    if organ_score > 7.0 {
        println!("✅ 检测到类器官结构，细胞形成了功能聚集");
    } else if organ_score > 4.0 {
        println!("⚠️  检测到部分类器官倾向");
    } else {
        println!("❌ 未检测到明显类器官结构");
    }
    
    println!();
    println!("═══════════════════════════════════════════════════════════════════════");
}

/// 细胞分类
fn classify_cell(cell: &Cell) -> &'static str {
    let energy = cell.energy;
    let signals = cell.signal_count;
    let variance = cell.variance;
    
    // 基于能量和信号的分类
    if energy > 0.8 && signals > 1000 {
        "核心处理单元"  // 类似神经元核心
    } else if energy > 0.6 && variance > 100.0 {
        "信号调节单元"  // 类似中间神经元
    } else if signals > 500 && variance < 10.0 {
        "信号接收单元"  // 类似感受器
    } else if energy < 0.3 && signals < 100 {
        "休眠/储备单元"  // 类似干细胞
    } else if variance > 50.0 {
        "高变异单元"    // 类似免疫细胞
    } else if energy > 0.5 {
        "活跃处理单元"  // 通用处理
    } else {
        "普通单元"      // 基础细胞
    }
}

struct Stats {
    min: f64,
    max: f64,
    mean: f64,
    std: f64,
}

fn calculate_stats(values: &[f64]) -> Stats {
    if values.is_empty() {
        return Stats { min: 0.0, max: 0.0, mean: 0.0, std: 0.0 };
    }
    
    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
    let std = variance.sqrt();
    
    Stats { min, max, mean, std }
}

fn calculate_correlation(x: &[f64], y: &[f64]) -> f64 {
    if x.len() != y.len() || x.len() < 2 {
        return 0.0;
    }
    
    let n = x.len() as f64;
    let x_mean = x.iter().sum::<f64>() / n;
    let y_mean = y.iter().sum::<f64>() / n;
    
    let mut cov = 0.0;
    let mut x_var = 0.0;
    let mut y_var = 0.0;
    
    for i in 0..x.len() {
        let dx = x[i] - x_mean;
        let dy = y[i] - y_mean;
        cov += dx * dy;
        x_var += dx * dx;
        y_var += dy * dy;
    }
    
    if x_var < 1e-10 || y_var < 1e-10 {
        return 0.0;
    }
    
    cov / (x_var.sqrt() * y_var.sqrt())
}

fn calculate_diversity(energies: &[f64], signals: &[f64], variances: &[f64]) -> f64 {
    let e_stats = calculate_stats(energies);
    let s_stats = calculate_stats(signals);
    let v_stats = calculate_stats(variances);
    
    // 基于标准差/均值的变异系数
    let e_cv = if e_stats.mean > 0.001 { e_stats.std / e_stats.mean } else { 0.0 };
    let s_cv = if s_stats.mean > 0.001 { s_stats.std / s_stats.mean } else { 0.0 };
    let v_cv = if v_stats.mean > 0.001 { v_stats.std / v_stats.mean } else { 0.0 };
    
    // 综合多样性指数 (0-10)
    ((e_cv + s_cv + v_cv) / 3.0 * 10.0).min(10.0)
}

/// 加载语料 (Valhalla 源码)
fn load_corpus() -> Vec<(String, Vec<u8>)> {
    let base_path = Path::new("/home/jouly/AI/Valhalla");
    let mut corpus = Vec::new();
    load_files_recursive(base_path, &mut corpus, 0);
    corpus
}

fn load_files_recursive(path: &Path, corpus: &mut Vec<(String, Vec<u8>)>, depth: usize) {
    if depth > 10 { return; }
    let Ok(entries) = std::fs::read_dir(path) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name.starts_with('.') || name == "target" || name == "node_modules" { continue; }
        if path.is_dir() {
            load_files_recursive(&path, corpus, depth + 1);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            if let Ok(content) = std::fs::read(&path) {
                corpus.push((path.display().to_string(), content));
            }
        }
    }
}

fn calculate_organ_score(groups: &HashMap<&str, Vec<usize>>, total: usize) -> f64 {
    if groups.is_empty() || total == 0 {
        return 0.0;
    }
    
    // 基于分组数量和均匀度
    let num_groups = groups.len() as f64;
    
    // Shannon 熵
    let mut entropy = 0.0;
    for (_, indices) in groups.iter() {
        let p = indices.len() as f64 / total as f64;
        if p > 0.0 {
            entropy -= p * p.ln();
        }
    }
    
    // 归一化 (最大熵 = ln(num_groups))
    let max_entropy = num_groups.ln();
    let normalized_entropy = if max_entropy > 0.0 { entropy / max_entropy } else { 0.0 };
    
    // 分组数量贡献
    let group_score = (num_groups / 7.0).min(1.0); // 最多 7 种功能分化
    
    // 综合得分 (0-10)
    ((normalized_entropy * 0.6 + group_score * 0.4) * 10.0).min(10.0)
}
