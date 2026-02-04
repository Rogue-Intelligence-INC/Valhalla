//! StemCell 细胞分化与器官化深度分析
//!
//! 分析:
//! - 细胞分化层次
//! - 功能聚类
//! - 器官化结构检测
//! - 信号网络拓扑

use std::collections::HashMap;
use stem_cell::{StemCellComplete, CompleteCell as Cell};

fn main() {
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("       StemCell 细胞分化与器官化深度分析");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    // 进行训练以获取分化状态
    println!("🏋️ 进行收敛训练...");
    let mut stemcell = StemCellComplete::new();
    
    // 加载语料
    let corpus = load_corpus();
    println!("   语料: {} 文件", corpus.len());
    
    // 5次收敛迭代
    for iter in 1..=5 {
        let mut round = 0;
        let mut last_count = stemcell.cells.len();
        let mut stable = 0;
        
        loop {
            round += 1;
            for (_, bytes) in &corpus {
                for &b in bytes.iter() {
                    stemcell.receive_data(b as f64);
                }
            }
            
            let current = stemcell.cells.len();
            let change = (current as f64 - last_count as f64).abs() / last_count.max(1) as f64;
            
            if change < 0.01 { stable += 1; } else { stable = 0; }
            if stable >= 3 || round >= 10 { break; }
            last_count = current;
        }
        println!("   迭代 {}: {} 细胞 ({} 轮)", iter, stemcell.cells.len(), round);
    }
    
    let cells = &stemcell.cells;
    println!();
    println!("📊 总细胞数: {}", cells.len());
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 1. 细胞分化层次分析
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("                    1. 细胞分化层次分析");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    // 基于能量、信号、方差的三维分类
    let mut differentiation_map: HashMap<DifferentiationLevel, Vec<usize>> = HashMap::new();
    
    for (i, c) in cells.iter().enumerate() {
        let level = classify_differentiation(c);
        differentiation_map.entry(level).or_default().push(i);
    }
    
    println!("🔬 分化层次分布:");
    println!();
    
    let levels = [
        DifferentiationLevel::Totipotent,
        DifferentiationLevel::Pluripotent,
        DifferentiationLevel::Multipotent,
        DifferentiationLevel::Oligopotent,
        DifferentiationLevel::Unipotent,
        DifferentiationLevel::TerminallyDifferentiated,
    ];
    
    for level in &levels {
        let count = differentiation_map.get(level).map(|v| v.len()).unwrap_or(0);
        let pct = count as f64 / cells.len() as f64 * 100.0;
        let bar = "█".repeat((pct * 0.5) as usize);
        println!("   {:25} {:>7} ({:>5.1}%) {}", 
            format!("{:?}", level), count, pct, bar);
        
        // 详细统计
        if let Some(indices) = differentiation_map.get(level) {
            if !indices.is_empty() {
                let energies: Vec<f64> = indices.iter().map(|&i| cells[i].energy).collect();
                let signals: Vec<f64> = indices.iter().map(|&i| cells[i].signal_count as f64).collect();
                let variances: Vec<f64> = indices.iter().map(|&i| cells[i].variance).collect();
                
                let (e_mean, e_std) = mean_std(&energies);
                let (s_mean, s_std) = mean_std(&signals);
                let (v_mean, v_std) = mean_std(&variances);
                
                println!("      能量: {:.4}±{:.4} | 信号: {:.0}±{:.0} | 方差: {:.1}±{:.1}",
                    e_mean, e_std, s_mean, s_std, v_mean, v_std);
            }
        }
    }
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 2. 功能聚类分析
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("                    2. 功能聚类分析");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    // K-means 风格聚类 (基于特征)
    let clusters = functional_clustering(cells, 8);
    
    println!("🎯 功能聚类 (8 类):");
    println!();
    
    for (i, cluster) in clusters.iter().enumerate() {
        if cluster.members.is_empty() { continue; }
        
        let pct = cluster.members.len() as f64 / cells.len() as f64 * 100.0;
        println!("   聚类 {} [{}]: {} 个 ({:.1}%)", 
            i, cluster.name, cluster.members.len(), pct);
        println!("      中心: 能量={:.4}, 信号={:.0}, 方差={:.1}",
            cluster.centroid.0, cluster.centroid.1, cluster.centroid.2);
        println!("      特征: {}", cluster.description);
        println!();
    }
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 3. 器官化结构检测
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("                    3. 器官化结构检测");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    // 检测器官化特征
    let organs = detect_organ_structures(cells);
    
    println!("🏛️ 检测到的类器官结构:");
    println!();
    
    for organ in &organs {
        println!("   {} [成熟度: {:.1}/10]", organ.name, organ.maturity);
        println!("      细胞数: {} ({:.1}%)", organ.cell_count, 
            organ.cell_count as f64 / cells.len() as f64 * 100.0);
        println!("      功能: {}", organ.function);
        println!("      特征: {}", organ.characteristics);
        println!();
    }
    
    // 器官化指数
    let organ_index = calculate_organ_index(&organs, cells.len());
    println!("   📊 器官化总指数: {:.2}/10", organ_index);
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 4. 信号网络拓扑分析
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("                    4. 信号网络拓扑分析");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    // 基于信号强度的网络分析
    let topology = analyze_signal_topology(cells);
    
    println!("📡 信号网络特征:");
    println!("   总连接强度: {:.0}", topology.total_signal_strength);
    println!("   平均信号/细胞: {:.1}", topology.avg_signal_per_cell);
    println!("   信号集中度 (Gini): {:.4}", topology.gini_coefficient);
    println!();
    
    println!("   🌟 信号枢纽 (Top 10):");
    for (i, hub) in topology.hubs.iter().take(10).enumerate() {
        println!("      {}. 细胞 {} - 信号: {}, 能量: {:.4}", 
            i + 1, hub.0, hub.1, hub.2);
    }
    println!();
    
    println!("   📊 信号分布直方图:");
    for (range, count) in &topology.signal_histogram {
        let pct = *count as f64 / cells.len() as f64 * 100.0;
        let bar = "█".repeat((pct * 0.3) as usize);
        println!("      {:>12}: {:>6} ({:>5.1}%) {}", range, count, pct, bar);
    }
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 5. 数据特征多样性
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("                    5. 数据特征多样性");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    let cells_with_sig: Vec<_> = cells.iter()
        .filter(|c| !c.data_signature.is_empty())
        .collect();
    
    println!("📝 数据特征统计:");
    println!("   有特征细胞: {} ({:.1}%)", 
        cells_with_sig.len(), 
        cells_with_sig.len() as f64 / cells.len() as f64 * 100.0);
    
    if !cells_with_sig.is_empty() {
        // 特征维度
        let dims: Vec<usize> = cells_with_sig.iter().map(|c| c.data_signature.len()).collect();
        let min_dim = *dims.iter().min().unwrap();
        let max_dim = *dims.iter().max().unwrap();
        
        println!("   特征维度: {} ~ {}", min_dim, max_dim);
        
        // 特征均值分布
        let sig_means: Vec<f64> = cells_with_sig.iter()
            .map(|c| c.data_signature.iter().sum::<f64>() / c.data_signature.len() as f64)
            .collect();
        
        let (mean_of_means, std_of_means) = mean_std(&sig_means);
        println!("   特征均值: {:.2} ± {:.2}", mean_of_means, std_of_means);
        
        // 特征聚类数估计 (简化版)
        let unique_patterns = estimate_unique_patterns(&cells_with_sig);
        println!("   估计独立模式数: ~{}", unique_patterns);
    }
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 6. 综合评估
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("                    6. 综合评估");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    // 计算各项指数
    let differentiation_index = calculate_differentiation_index(&differentiation_map, cells.len());
    let clustering_index = calculate_clustering_index(&clusters, cells.len());
    let topology_index = calculate_topology_index(&topology);
    
    println!("🎯 分化与器官化评分:");
    println!();
    println!("   ┌──────────────────────────┬────────┐");
    println!("   │ 指标                     │  得分  │");
    println!("   ├──────────────────────────┼────────┤");
    println!("   │ 分化层次多样性           │ {:>5.2} │", differentiation_index);
    println!("   │ 功能聚类清晰度           │ {:>5.2} │", clustering_index);
    println!("   │ 器官化程度               │ {:>5.2} │", organ_index);
    println!("   │ 信号网络复杂度           │ {:>5.2} │", topology_index);
    println!("   ├──────────────────────────┼────────┤");
    let total = (differentiation_index + clustering_index + organ_index + topology_index) / 4.0;
    println!("   │ 综合得分                 │ {:>5.2} │", total);
    println!("   └──────────────────────────┴────────┘");
    println!();
    
    // 生物学类比
    println!("🧬 生物学类比:");
    if total >= 8.0 {
        println!("   → 类似复杂多细胞生物，具有明确的器官分化");
    } else if total >= 6.0 {
        println!("   → 类似简单多细胞生物，有初步的组织分化");
    } else if total >= 4.0 {
        println!("   → 类似细胞群落，有功能分工但无明确器官");
    } else {
        println!("   → 类似原始细胞群，分化程度较低");
    }
    println!();
    
    // 器官功能总结
    println!("🏛️ 器官功能总结:");
    for organ in &organs {
        if organ.maturity >= 5.0 {
            println!("   ✅ {}: {}", organ.name, organ.function);
        }
    }
    
    println!();
    println!("═══════════════════════════════════════════════════════════════════════════════");
}

// ============================================================================
// 数据结构
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum DifferentiationLevel {
    Totipotent,            // 全能干细胞
    Pluripotent,           // 多能干细胞
    Multipotent,           // 多向分化
    Oligopotent,           // 寡能分化
    Unipotent,             // 单向分化
    TerminallyDifferentiated, // 终末分化
}

struct Cluster {
    name: &'static str,
    members: Vec<usize>,
    centroid: (f64, f64, f64), // (energy, signal, variance)
    description: String,
}

struct OrganStructure {
    name: &'static str,
    cell_count: usize,
    function: &'static str,
    characteristics: String,
    maturity: f64,
}

struct SignalTopology {
    total_signal_strength: f64,
    avg_signal_per_cell: f64,
    gini_coefficient: f64,
    hubs: Vec<(String, u64, f64)>, // (id, signal, energy)
    signal_histogram: Vec<(String, usize)>,
}

// ============================================================================
// 分类函数
// ============================================================================

fn classify_differentiation(cell: &Cell) -> DifferentiationLevel {
    let energy = cell.energy;
    let signal = cell.signal_count;
    let variance = cell.variance;
    let has_sig = !cell.data_signature.is_empty();
    
    // 基于特征组合判断分化程度
    if energy < 0.3 && signal < 10 && variance < 1.0 {
        DifferentiationLevel::Totipotent // 低活性，未分化
    } else if energy < 0.5 && signal < 100 && !has_sig {
        DifferentiationLevel::Pluripotent // 可向多方向分化
    } else if energy > 0.9 && signal > 10000 && variance > 500.0 {
        DifferentiationLevel::TerminallyDifferentiated // 高度特化
    } else if energy > 0.8 && signal > 1000 {
        DifferentiationLevel::Unipotent // 单向高活性
    } else if signal > 500 || variance > 100.0 {
        DifferentiationLevel::Oligopotent // 有限分化
    } else {
        DifferentiationLevel::Multipotent // 中等分化潜力
    }
}

fn functional_clustering(cells: &[Cell], k: usize) -> Vec<Cluster> {
    // 简化的 K-means 聚类
    let mut clusters: Vec<Cluster> = vec![
        Cluster { name: "信号中枢", members: vec![], centroid: (1.0, 50000.0, 1000.0), description: String::new() },
        Cluster { name: "信号调节", members: vec![], centroid: (0.9, 500.0, 200.0), description: String::new() },
        Cluster { name: "信号接收", members: vec![], centroid: (0.8, 100.0, 50.0), description: String::new() },
        Cluster { name: "能量储存", members: vec![], centroid: (1.0, 10.0, 10.0), description: String::new() },
        Cluster { name: "高变异", members: vec![], centroid: (0.7, 200.0, 500.0), description: String::new() },
        Cluster { name: "休眠", members: vec![], centroid: (0.5, 0.0, 0.0), description: String::new() },
        Cluster { name: "代谢活跃", members: vec![], centroid: (0.95, 50.0, 100.0), description: String::new() },
        Cluster { name: "普通", members: vec![], centroid: (0.6, 20.0, 30.0), description: String::new() },
    ];
    
    // 分配每个细胞到最近的聚类
    for (i, c) in cells.iter().enumerate() {
        let point = (c.energy, c.signal_count as f64, c.variance);
        
        let mut best_cluster = 0;
        let mut best_dist = f64::MAX;
        
        for (j, cluster) in clusters.iter().enumerate() {
            let dist = distance(point, cluster.centroid);
            if dist < best_dist {
                best_dist = dist;
                best_cluster = j;
            }
        }
        
        clusters[best_cluster].members.push(i);
    }
    
    // 更新聚类中心和描述
    for cluster in &mut clusters {
        if !cluster.members.is_empty() {
            let n = cluster.members.len() as f64;
            let sum_e: f64 = cluster.members.iter().map(|&i| cells[i].energy).sum();
            let sum_s: f64 = cluster.members.iter().map(|&i| cells[i].signal_count as f64).sum();
            let sum_v: f64 = cluster.members.iter().map(|&i| cells[i].variance).sum();
            
            cluster.centroid = (sum_e / n, sum_s / n, sum_v / n);
            cluster.description = format!(
                "能量{:.2}, 信号{:.0}, 方差{:.0}",
                cluster.centroid.0, cluster.centroid.1, cluster.centroid.2
            );
        }
    }
    
    clusters.truncate(k);
    clusters
}

fn distance(a: (f64, f64, f64), b: (f64, f64, f64)) -> f64 {
    // 归一化距离
    let e_diff = (a.0 - b.0) * 10.0;
    let s_diff = (a.1 - b.1) / 1000.0;
    let v_diff = (a.2 - b.2) / 100.0;
    (e_diff * e_diff + s_diff * s_diff + v_diff * v_diff).sqrt()
}

fn detect_organ_structures(cells: &[Cell]) -> Vec<OrganStructure> {
    let mut organs = Vec::new();
    
    // 检测各种器官化结构
    
    // 1. 神经中枢 (高信号处理)
    let neural_cells: Vec<_> = cells.iter()
        .enumerate()
        .filter(|(_, c)| c.signal_count > 10000 && c.energy > 0.9)
        .collect();
    
    if !neural_cells.is_empty() {
        let avg_signal = neural_cells.iter().map(|(_, c)| c.signal_count).sum::<u64>() as f64 / neural_cells.len() as f64;
        organs.push(OrganStructure {
            name: "神经处理中枢",
            cell_count: neural_cells.len(),
            function: "高强度信号处理与整合",
            characteristics: format!("平均信号量: {:.0}", avg_signal),
            maturity: (neural_cells.len() as f64 / cells.len() as f64 * 50.0).min(10.0),
        });
    }
    
    // 2. 信号调节网络
    let regulatory_cells: Vec<_> = cells.iter()
        .enumerate()
        .filter(|(_, c)| c.signal_count > 100 && c.signal_count < 10000 && c.variance > 50.0)
        .collect();
    
    if !regulatory_cells.is_empty() {
        let avg_var = regulatory_cells.iter().map(|(_, c)| c.variance).sum::<f64>() / regulatory_cells.len() as f64;
        organs.push(OrganStructure {
            name: "信号调节网络",
            cell_count: regulatory_cells.len(),
            function: "信号过滤、放大与调制",
            characteristics: format!("平均方差: {:.1}", avg_var),
            maturity: (regulatory_cells.len() as f64 / cells.len() as f64 * 20.0).min(10.0),
        });
    }
    
    // 3. 感知层
    let sensory_cells: Vec<_> = cells.iter()
        .enumerate()
        .filter(|(_, c)| c.signal_count > 0 && c.signal_count < 100 && c.energy > 0.8)
        .collect();
    
    if !sensory_cells.is_empty() {
        organs.push(OrganStructure {
            name: "感知层",
            cell_count: sensory_cells.len(),
            function: "初级信号接收与转化",
            characteristics: format!("活跃度: {:.1}%", sensory_cells.len() as f64 / cells.len() as f64 * 100.0),
            maturity: (sensory_cells.len() as f64 / cells.len() as f64 * 30.0).min(10.0),
        });
    }
    
    // 4. 能量储备池
    let storage_cells: Vec<_> = cells.iter()
        .enumerate()
        .filter(|(_, c)| c.energy > 0.95 && c.signal_count < 50 && c.variance < 20.0)
        .collect();
    
    if !storage_cells.is_empty() {
        let avg_energy = storage_cells.iter().map(|(_, c)| c.energy).sum::<f64>() / storage_cells.len() as f64;
        organs.push(OrganStructure {
            name: "能量储备池",
            cell_count: storage_cells.len(),
            function: "能量存储与稳定供应",
            characteristics: format!("平均能量: {:.4}", avg_energy),
            maturity: (storage_cells.len() as f64 / cells.len() as f64 * 40.0).min(10.0),
        });
    }
    
    // 5. 适应性模块 (高变异)
    let adaptive_cells: Vec<_> = cells.iter()
        .enumerate()
        .filter(|(_, c)| c.variance > 200.0)
        .collect();
    
    if !adaptive_cells.is_empty() {
        let max_var = adaptive_cells.iter().map(|(_, c)| c.variance).fold(0.0f64, f64::max);
        organs.push(OrganStructure {
            name: "适应性模块",
            cell_count: adaptive_cells.len(),
            function: "环境适应与快速响应",
            characteristics: format!("最大方差: {:.1}", max_var),
            maturity: (adaptive_cells.len() as f64 / cells.len() as f64 * 100.0).min(10.0),
        });
    }
    
    // 6. 干细胞储备
    let stem_cells: Vec<_> = cells.iter()
        .enumerate()
        .filter(|(_, c)| c.energy == 0.5 && c.signal_count == 0)
        .collect();
    
    if !stem_cells.is_empty() {
        organs.push(OrganStructure {
            name: "干细胞储备",
            cell_count: stem_cells.len(),
            function: "细胞再生与修复潜力",
            characteristics: "原始状态保持".to_string(),
            maturity: 5.0, // 干细胞总是中等成熟度
        });
    }
    
    organs
}

fn analyze_signal_topology(cells: &[Cell]) -> SignalTopology {
    let signals: Vec<u64> = cells.iter().map(|c| c.signal_count).collect();
    let total: u64 = signals.iter().sum();
    let avg = total as f64 / cells.len() as f64;
    
    // Gini 系数计算
    let mut sorted = signals.clone();
    sorted.sort();
    let n = sorted.len() as f64;
    let mut sum_i_y = 0.0;
    for (i, &y) in sorted.iter().enumerate() {
        sum_i_y += (i + 1) as f64 * y as f64;
    }
    let gini = if total > 0 {
        (2.0 * sum_i_y) / (n * total as f64) - (n + 1.0) / n
    } else {
        0.0
    };
    
    // 找出枢纽
    let mut hubs: Vec<(String, u64, f64)> = cells.iter()
        .map(|c| (c.cell_id.clone(), c.signal_count, c.energy))
        .collect();
    hubs.sort_by(|a, b| b.1.cmp(&a.1));
    
    // 信号直方图
    let mut histogram = vec![
        ("0".to_string(), 0),
        ("1-10".to_string(), 0),
        ("11-100".to_string(), 0),
        ("101-1000".to_string(), 0),
        ("1001-10000".to_string(), 0),
        ("10001-100000".to_string(), 0),
        (">100000".to_string(), 0),
    ];
    
    for &s in &signals {
        let idx = match s {
            0 => 0,
            1..=10 => 1,
            11..=100 => 2,
            101..=1000 => 3,
            1001..=10000 => 4,
            10001..=100000 => 5,
            _ => 6,
        };
        histogram[idx].1 += 1;
    }
    
    SignalTopology {
        total_signal_strength: total as f64,
        avg_signal_per_cell: avg,
        gini_coefficient: gini,
        hubs,
        signal_histogram: histogram,
    }
}

fn estimate_unique_patterns(cells: &[&Cell]) -> usize {
    // 简化估计：基于特征均值分桶
    let mut buckets: HashMap<i32, usize> = HashMap::new();
    
    for c in cells {
        if !c.data_signature.is_empty() {
            let mean = c.data_signature.iter().sum::<f64>() / c.data_signature.len() as f64;
            let bucket = (mean / 100.0) as i32;
            *buckets.entry(bucket).or_insert(0) += 1;
        }
    }
    
    buckets.len()
}

// ============================================================================
// 指数计算
// ============================================================================

fn calculate_organ_index(organs: &[OrganStructure], total_cells: usize) -> f64 {
    if organs.is_empty() || total_cells == 0 {
        return 0.0;
    }
    
    // 综合器官成熟度和覆盖率
    let avg_maturity = organs.iter().map(|o| o.maturity).sum::<f64>() / organs.len() as f64;
    let coverage: usize = organs.iter().map(|o| o.cell_count).sum();
    let coverage_ratio = coverage as f64 / total_cells as f64;
    
    (avg_maturity * 0.6 + coverage_ratio * 4.0).min(10.0)
}

fn calculate_differentiation_index(
    map: &HashMap<DifferentiationLevel, Vec<usize>>, 
    total: usize
) -> f64 {
    if map.is_empty() || total == 0 {
        return 0.0;
    }
    
    // Shannon 熵
    let mut entropy = 0.0;
    for (_, indices) in map.iter() {
        let p = indices.len() as f64 / total as f64;
        if p > 0.0 {
            entropy -= p * p.ln();
        }
    }
    
    let max_entropy = (map.len() as f64).ln();
    let normalized = if max_entropy > 0.0 { entropy / max_entropy } else { 0.0 };
    
    normalized * 10.0
}

fn calculate_clustering_index(clusters: &[Cluster], total: usize) -> f64 {
    if clusters.is_empty() || total == 0 {
        return 0.0;
    }
    
    // 聚类清晰度：基于聚类大小分布的均匀度
    let non_empty: Vec<_> = clusters.iter().filter(|c| !c.members.is_empty()).collect();
    if non_empty.is_empty() {
        return 0.0;
    }
    
    let sizes: Vec<f64> = non_empty.iter().map(|c| c.members.len() as f64).collect();
    let (mean, std) = mean_std(&sizes);
    
    // 变异系数越低，聚类越清晰
    let cv = if mean > 0.0 { std / mean } else { 1.0 };
    let clarity = (1.0 - cv.min(1.0)) * 10.0;
    
    // 加上聚类数量奖励
    let num_bonus = (non_empty.len() as f64 / 8.0).min(1.0) * 2.0;
    
    (clarity + num_bonus).min(10.0)
}

fn calculate_topology_index(topology: &SignalTopology) -> f64 {
    // 基于 Gini 系数和信号分布
    // 高 Gini = 信号集中 = 有明确的枢纽结构
    let structure_score = topology.gini_coefficient * 8.0;
    
    // 信号活跃度
    let activity_score = if topology.avg_signal_per_cell > 1000.0 { 2.0 } 
        else if topology.avg_signal_per_cell > 100.0 { 1.0 } 
        else { 0.5 };
    
    (structure_score + activity_score).min(10.0)
}

// ============================================================================
// 工具函数
// ============================================================================

fn mean_std(values: &[f64]) -> (f64, f64) {
    if values.is_empty() {
        return (0.0, 0.0);
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
    (mean, variance.sqrt())
}

fn load_corpus() -> Vec<(String, Vec<u8>)> {
    let base_path = std::path::Path::new("/home/jouly/AI/Valhalla");
    let mut corpus = Vec::new();
    load_files_recursive(base_path, &mut corpus, 0);
    
    // 加一些 Cargo registry
    let cargo_path = std::path::Path::new("/home/jouly/.cargo/registry/src");
    load_files_recursive(cargo_path, &mut corpus, 0);
    
    if corpus.len() > 2000 {
        corpus.truncate(2000);
    }
    corpus
}

fn load_files_recursive(path: &std::path::Path, corpus: &mut Vec<(String, Vec<u8>)>, depth: usize) {
    if depth > 5 { return; }
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
