//! StemCell 关联性交互模块
//! 
//! 核心理念：
//! 1. 不是问答模式，而是让系统**自由输出**
//! 2. 分析输出中的**强、中、弱关联**
//! 3. 基于训练后的细胞网络拓扑进行交互
//! 
//! 交互方式：
//! - 刺激网络 → 观察能量传播模式
//! - 系统自由激活 → 输出关联链
//! - 分析关联强度分布

use stem_cell::{StemCellComplete, CompleteCell};
use std::collections::HashSet;
use std::cmp::Ordering;

fn main() {
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("   StemCell 关联性交互 - 自由输出 + 关联强度分析");
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!();
    
    // ═══ 阶段 1: 训练系统 ═══
    println!("【阶段 1】训练系统 - 建立细胞网络\n");
    
    let mut stemcell = StemCellComplete::new();
    
    // 训练语料 - 建立概念关联
    let training_data = include_training_corpus();
    
    for text in &training_data {
        for ch in text.chars() {
            stemcell.receive_data(ch as u8 as f64);
        }
        stemcell.receive_data(0.0);  // 分隔符
    }
    
    // 让网络自组织稳定
    println!("训练数据已输入，开始自组织演化...");
    for i in 0..200 {
        stemcell.evolve(0.0);
        if (i + 1) % 50 == 0 {
            println!("  演化 {} tick, 细胞: {}, 连接: {}", 
                i + 1, 
                stemcell.cells.len(),
                stemcell.cells.iter().map(|c| c.connections.len()).sum::<usize>());
        }
    }
    
    println!("\n网络已稳定！");
    print_network_summary(&stemcell);
    
    // ═══ 阶段 2: 分析网络拓扑 ═══
    println!("\n【阶段 2】网络拓扑分析 - 识别核心与边缘\n");
    
    let topology = analyze_topology(&stemcell);
    print_topology(&topology);
    
    // ═══ 阶段 3: 自由输出模式 ═══
    println!("\n【阶段 3】自由输出模式 - 系统自主激活\n");
    println!("不给定输入，让系统基于内部状态自由输出关联链...\n");
    
    // 从不同起点探索关联
    let start_points = select_diverse_start_points(&stemcell, &topology, 5);
    
    for (idx, start_cell_id) in start_points.iter().enumerate() {
        println!("─────────────────────────────────────────────────────────────────────────────");
        println!("自由关联链 #{}", idx + 1);
        
        let association_chain = generate_free_association(&stemcell, start_cell_id, 8);
        print_association_chain(&association_chain);
        println!();
    }
    
    // ═══ 阶段 4: 关联强度分析 ═══
    println!("\n【阶段 4】关联强度分布分析\n");
    
    let strength_distribution = analyze_association_strengths(&stemcell);
    print_strength_distribution(&strength_distribution);
    
    // ═══ 阶段 5: 刺激-响应模式 ═══
    println!("\n【阶段 5】刺激-响应模式 - 输入信号观察扩散\n");
    
    // 注入刺激并观察传播
    let stimuli = vec!["cat", "code", "run", "数字"];
    
    for stimulus in &stimuli {
        println!("─────────────────────────────────────────────────────────────────────────────");
        println!("刺激信号: \"{}\"", stimulus);
        
        let propagation = inject_and_observe(&stemcell, stimulus);
        print_propagation_result(&propagation);
        println!();
    }
    
    // ═══ 阶段 6: 关联矩阵 ═══
    println!("\n【阶段 6】核心概念关联矩阵\n");
    
    build_association_matrix(&stemcell, &topology);
    
    // ═══ 总结 ═══
    println!("\n═══════════════════════════════════════════════════════════════════════════");
    println!("   关联性交互总结");
    println!("═══════════════════════════════════════════════════════════════════════════\n");
    
    summarize_interaction(&stemcell, &topology, &strength_distribution);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 数据结构
// ═══════════════════════════════════════════════════════════════════════════════

/// 网络拓扑结构
struct NetworkTopology {
    total_cells: usize,
    total_connections: usize,
    core_cells: Vec<String>,      // 高连接度细胞
    peripheral_cells: Vec<String>, // 低连接度细胞
    isolated_cells: Vec<String>,   // 孤立细胞
    hub_cells: Vec<String>,        // 枢纽细胞（连接不同群体）
    clusters: Vec<HashSet<String>>, // 自动发现的簇
}

/// 关联链节点
struct AssociationNode {
    cell_id: String,
    connection_strength: f64,
    strength_class: StrengthClass,
    data_summary: String,
}

/// 关联强度分类
#[derive(Debug, Clone, Copy, PartialEq)]
enum StrengthClass {
    Strong,   // > 0.7
    Medium,   // 0.3 - 0.7
    Weak,     // < 0.3
}

/// 关联强度分布
struct StrengthDistribution {
    strong_count: usize,
    medium_count: usize,
    weak_count: usize,
    strong_avg: f64,
    medium_avg: f64,
    weak_avg: f64,
}

/// 传播结果
struct PropagationResult {
    stimulus: String,
    directly_activated: usize,
    first_hop: usize,
    second_hop: usize,
    third_hop: usize,
    total_reached: usize,
    reach_percentage: f64,
    strongest_associations: Vec<(String, f64, StrengthClass)>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// 核心函数
// ═══════════════════════════════════════════════════════════════════════════════

/// 训练语料
fn include_training_corpus() -> Vec<&'static str> {
    vec![
        // 基础概念
        "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
        "red", "green", "blue", "yellow", "orange", "purple", "black", "white",
        "cat", "dog", "bird", "fish", "lion", "tiger", "elephant", "mouse",
        "run", "walk", "jump", "swim", "fly", "sleep", "eat", "drink", "think",
        "big", "small", "fast", "slow", "hot", "cold", "good", "bad",
        // 代码概念
        "fn main", "let x", "if true", "else false", "for i in", "while loop",
        "struct Cell", "impl Trait", "pub fn", "use std", "mod test", "return value",
        // 关联短语
        "cat runs fast", "dog walks slow", "bird flies high", "fish swims deep",
        "red cat", "blue fish", "green bird", "yellow sun", "black night",
        "big elephant", "small mouse", "fast rabbit", "slow turtle",
        // 抽象概念
        "time flows", "space expands", "energy transforms", "matter exists",
        "thought emerges", "pattern forms", "structure builds", "system evolves",
    ]
}

/// 分析网络拓扑
fn analyze_topology(stemcell: &StemCellComplete) -> NetworkTopology {
    let cells = &stemcell.cells;
    
    // 统计连接度
    let mut conn_counts: Vec<(String, usize)> = cells.iter()
        .map(|c| (c.cell_id.clone(), c.connections.len()))
        .collect();
    conn_counts.sort_by(|a, b| b.1.cmp(&a.1));
    
    let avg_conn = if cells.is_empty() { 0.0 } else {
        conn_counts.iter().map(|(_, c)| *c).sum::<usize>() as f64 / cells.len() as f64
    };
    
    // 分类
    let core_threshold = (avg_conn * 1.5) as usize;
    let peripheral_threshold = (avg_conn * 0.5) as usize;
    
    let mut core_cells = Vec::new();
    let mut peripheral_cells = Vec::new();
    let mut isolated_cells = Vec::new();
    
    for (id, count) in &conn_counts {
        if *count == 0 {
            isolated_cells.push(id.clone());
        } else if *count >= core_threshold {
            core_cells.push(id.clone());
        } else if *count <= peripheral_threshold {
            peripheral_cells.push(id.clone());
        }
    }
    
    // 识别枢纽细胞（连接到多个不同"群体"的细胞）
    let hub_cells = identify_hubs(stemcell);
    
    // 简单聚类（基于连接）
    let clusters = simple_clustering(stemcell);
    
    NetworkTopology {
        total_cells: cells.len(),
        total_connections: cells.iter().map(|c| c.connections.len()).sum(),
        core_cells,
        peripheral_cells,
        isolated_cells,
        hub_cells,
        clusters,
    }
}

/// 识别枢纽细胞
fn identify_hubs(stemcell: &StemCellComplete) -> Vec<String> {
    let mut hubs = Vec::new();
    
    for cell in &stemcell.cells {
        if cell.connections.len() < 5 {
            continue;
        }
        
        // 检查连接的多样性（连接的细胞之间是否也互连）
        let neighbor_ids: HashSet<&str> = cell.connections.iter()
            .map(|(id, _)| id.as_str())
            .collect();
        
        let mut inter_connections = 0;
        let total_possible = cell.connections.len() * (cell.connections.len() - 1) / 2;
        
        for (neighbor_id, _) in &cell.connections {
            if let Some(neighbor) = stemcell.cells.iter().find(|c| &c.cell_id == neighbor_id) {
                for (n2_id, _) in &neighbor.connections {
                    if neighbor_ids.contains(n2_id.as_str()) {
                        inter_connections += 1;
                    }
                }
            }
        }
        
        // 如果邻居之间的互连度低，说明这是一个枢纽
        if total_possible > 0 {
            let clustering_coeff = inter_connections as f64 / (total_possible as f64 * 2.0);
            if clustering_coeff < 0.5 {  // 低聚类系数 = 枢纽特征
                hubs.push(cell.cell_id.clone());
            }
        }
    }
    
    hubs
}

/// 简单聚类
fn simple_clustering(stemcell: &StemCellComplete) -> Vec<HashSet<String>> {
    let mut visited: HashSet<String> = HashSet::new();
    let mut clusters: Vec<HashSet<String>> = Vec::new();
    
    for cell in &stemcell.cells {
        if visited.contains(&cell.cell_id) || cell.connections.is_empty() {
            continue;
        }
        
        // BFS 找到连通分量
        let mut cluster: HashSet<String> = HashSet::new();
        let mut queue = vec![cell.cell_id.clone()];
        
        while let Some(current_id) = queue.pop() {
            if visited.contains(&current_id) {
                continue;
            }
            visited.insert(current_id.clone());
            cluster.insert(current_id.clone());
            
            if let Some(current) = stemcell.cells.iter().find(|c| c.cell_id == current_id) {
                for (neighbor_id, strength) in &current.connections {
                    if !visited.contains(neighbor_id) && *strength > 0.05 {
                        queue.push(neighbor_id.clone());
                    }
                }
            }
        }
        
        if cluster.len() > 1 {
            clusters.push(cluster);
        }
    }
    
    // 按大小排序
    clusters.sort_by(|a, b| b.len().cmp(&a.len()));
    clusters
}

/// 选择多样化的起点
fn select_diverse_start_points(stemcell: &StemCellComplete, topology: &NetworkTopology, count: usize) -> Vec<String> {
    let mut points = Vec::new();
    
    // 从核心选一个
    if !topology.core_cells.is_empty() {
        points.push(topology.core_cells[0].clone());
    }
    
    // 从枢纽选一个
    if !topology.hub_cells.is_empty() && points.len() < count {
        let hub = &topology.hub_cells[0];
        if !points.contains(hub) {
            points.push(hub.clone());
        }
    }
    
    // 从边缘选一个
    if !topology.peripheral_cells.is_empty() && points.len() < count {
        points.push(topology.peripheral_cells[0].clone());
    }
    
    // 从各个簇选一个
    for cluster in &topology.clusters {
        if points.len() >= count {
            break;
        }
        if let Some(id) = cluster.iter().find(|id| !points.contains(*id)) {
            points.push(id.clone());
        }
    }
    
    // 补充随机选择
    for cell in &stemcell.cells {
        if points.len() >= count {
            break;
        }
        if !points.contains(&cell.cell_id) && !cell.connections.is_empty() {
            points.push(cell.cell_id.clone());
        }
    }
    
    points
}

/// 生成自由关联链
fn generate_free_association(stemcell: &StemCellComplete, start_id: &str, max_length: usize) -> Vec<AssociationNode> {
    let mut chain = Vec::new();
    let mut visited: HashSet<String> = HashSet::new();
    let mut current_id = start_id.to_string();
    
    // 添加起点
    if let Some(start_cell) = stemcell.cells.iter().find(|c| c.cell_id == start_id) {
        chain.push(AssociationNode {
            cell_id: start_id.to_string(),
            connection_strength: 1.0,
            strength_class: StrengthClass::Strong,
            data_summary: summarize_cell_data(start_cell),
        });
        visited.insert(start_id.to_string());
    } else {
        return chain;
    }
    
    // 沿最强连接走
    while chain.len() < max_length {
        if let Some(current) = stemcell.cells.iter().find(|c| c.cell_id == current_id) {
            // 找到最强的未访问连接
            let mut best: Option<(&str, f64)> = None;
            
            for (neighbor_id, strength) in &current.connections {
                if !visited.contains(neighbor_id) {
                    if best.is_none() || *strength > best.unwrap().1 {
                        best = Some((neighbor_id.as_str(), *strength));
                    }
                }
            }
            
            if let Some((next_id, strength)) = best {
                visited.insert(next_id.to_string());
                
                if let Some(next_cell) = stemcell.cells.iter().find(|c| c.cell_id == next_id) {
                    let strength_class = classify_strength(strength);
                    chain.push(AssociationNode {
                        cell_id: next_id.to_string(),
                        connection_strength: strength,
                        strength_class,
                        data_summary: summarize_cell_data(next_cell),
                    });
                    current_id = next_id.to_string();
                } else {
                    break;
                }
            } else {
                break;  // 没有更多未访问的连接
            }
        } else {
            break;
        }
    }
    
    chain
}

/// 分类关联强度
fn classify_strength(strength: f64) -> StrengthClass {
    if strength > 0.1 {
        StrengthClass::Strong
    } else if strength > 0.05 {
        StrengthClass::Medium
    } else {
        StrengthClass::Weak
    }
}

/// 总结细胞数据
fn summarize_cell_data(cell: &CompleteCell) -> String {
    if cell.data_signature.is_empty() {
        return format!("空数据, E={:.3}", cell.energy);
    }
    
    // 尝试解码数据特征
    let decoded: String = cell.data_signature.iter()
        .filter_map(|&v| {
            let byte = v as u8;
            if byte >= 32 && byte <= 126 {
                Some(byte as char)
            } else {
                None
            }
        })
        .take(10)
        .collect();
    
    if decoded.is_empty() {
        format!("sig=[{:.2}..], E={:.3}", 
            cell.data_signature.first().unwrap_or(&0.0),
            cell.energy)
    } else {
        format!("\"{}..\" E={:.3}", decoded, cell.energy)
    }
}

/// 分析关联强度分布
fn analyze_association_strengths(stemcell: &StemCellComplete) -> StrengthDistribution {
    let mut strong = Vec::new();
    let mut medium = Vec::new();
    let mut weak = Vec::new();
    
    for cell in &stemcell.cells {
        for (_, strength) in &cell.connections {
            match classify_strength(*strength) {
                StrengthClass::Strong => strong.push(*strength),
                StrengthClass::Medium => medium.push(*strength),
                StrengthClass::Weak => weak.push(*strength),
            }
        }
    }
    
    StrengthDistribution {
        strong_count: strong.len(),
        medium_count: medium.len(),
        weak_count: weak.len(),
        strong_avg: if strong.is_empty() { 0.0 } else { strong.iter().sum::<f64>() / strong.len() as f64 },
        medium_avg: if medium.is_empty() { 0.0 } else { medium.iter().sum::<f64>() / medium.len() as f64 },
        weak_avg: if weak.is_empty() { 0.0 } else { weak.iter().sum::<f64>() / weak.len() as f64 },
    }
}

/// 注入刺激并观察传播
fn inject_and_observe(stemcell: &StemCellComplete, stimulus: &str) -> PropagationResult {
    let stimulus_sig: Vec<f64> = stimulus.chars().map(|c| c as u8 as f64).collect();
    
    // 找到直接相关的细胞
    let mut directly_activated: HashSet<String> = HashSet::new();
    
    for cell in &stemcell.cells {
        let sim = compute_similarity(&stimulus_sig, &cell.data_signature);
        if sim > 0.3 {
            directly_activated.insert(cell.cell_id.clone());
        }
    }
    
    // 传播：第一跳
    let mut first_hop: HashSet<String> = HashSet::new();
    for id in &directly_activated {
        if let Some(cell) = stemcell.cells.iter().find(|c| &c.cell_id == id) {
            for (neighbor_id, _) in &cell.connections {
                if !directly_activated.contains(neighbor_id) {
                    first_hop.insert(neighbor_id.clone());
                }
            }
        }
    }
    
    // 传播：第二跳
    let mut second_hop: HashSet<String> = HashSet::new();
    let visited: HashSet<&String> = directly_activated.iter().chain(first_hop.iter()).collect();
    for id in &first_hop {
        if let Some(cell) = stemcell.cells.iter().find(|c| &c.cell_id == id) {
            for (neighbor_id, _) in &cell.connections {
                if !visited.contains(neighbor_id) {
                    second_hop.insert(neighbor_id.clone());
                }
            }
        }
    }
    
    // 传播：第三跳
    let mut third_hop: HashSet<String> = HashSet::new();
    let visited: HashSet<&String> = directly_activated.iter()
        .chain(first_hop.iter())
        .chain(second_hop.iter())
        .collect();
    for id in &second_hop {
        if let Some(cell) = stemcell.cells.iter().find(|c| &c.cell_id == id) {
            for (neighbor_id, _) in &cell.connections {
                if !visited.contains(neighbor_id) {
                    third_hop.insert(neighbor_id.clone());
                }
            }
        }
    }
    
    let total_reached = directly_activated.len() + first_hop.len() + second_hop.len() + third_hop.len();
    let reach_percentage = if stemcell.cells.is_empty() { 0.0 } else {
        total_reached as f64 / stemcell.cells.len() as f64 * 100.0
    };
    
    // 找出最强关联
    let mut associations: Vec<(String, f64, StrengthClass)> = Vec::new();
    for id in directly_activated.iter().chain(first_hop.iter()) {
        if let Some(cell) = stemcell.cells.iter().find(|c| &c.cell_id == id) {
            for (neighbor_id, strength) in &cell.connections {
                associations.push((neighbor_id.clone(), *strength, classify_strength(*strength)));
            }
        }
    }
    associations.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
    associations.truncate(5);
    
    PropagationResult {
        stimulus: stimulus.to_string(),
        directly_activated: directly_activated.len(),
        first_hop: first_hop.len(),
        second_hop: second_hop.len(),
        third_hop: third_hop.len(),
        total_reached,
        reach_percentage,
        strongest_associations: associations,
    }
}

/// 计算相似度
fn compute_similarity(a: &[f64], b: &[f64]) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    
    let min_len = a.len().min(b.len());
    let mut dot = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;
    
    for i in 0..min_len {
        dot += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }
    
    if norm_a < 1e-10 || norm_b < 1e-10 {
        return 0.0;
    }
    
    ((dot / (norm_a.sqrt() * norm_b.sqrt())) + 1.0) / 2.0
}

/// 构建关联矩阵
fn build_association_matrix(stemcell: &StemCellComplete, topology: &NetworkTopology) {
    // 选择核心细胞的子集
    let sample_ids: Vec<&String> = topology.core_cells.iter().take(8).collect();
    
    if sample_ids.is_empty() {
        println!("没有足够的核心细胞构建矩阵");
        return;
    }
    
    // 表头
    print!("{:>12} |", "");
    for id in &sample_ids {
        print!(" {:>8} |", &id[..id.len().min(8)]);
    }
    println!();
    println!("{}", "-".repeat(12 + (11 * sample_ids.len())));
    
    // 矩阵内容
    for id_a in &sample_ids {
        print!("{:>12} |", &id_a[..id_a.len().min(12)]);
        
        if let Some(cell_a) = stemcell.cells.iter().find(|c| &c.cell_id == *id_a) {
            for id_b in &sample_ids {
                if id_a == id_b {
                    print!("     ---  |");
                } else {
                    // 查找连接强度
                    let strength = cell_a.connections.iter()
                        .find(|(id, _)| &id == id_b)
                        .map(|(_, s)| *s)
                        .unwrap_or(0.0);
                    
                    let symbol = if strength > 0.1 { "●●●" }
                        else if strength > 0.05 { " ●●" }
                        else if strength > 0.01 { "  ●" }
                        else { "  ·" };
                    
                    print!(" {} {:>4.2} |", symbol, strength);
                }
            }
        }
        println!();
    }
    
    println!();
    println!("图例: ●●● 强关联 (>0.1)  ●● 中关联 (>0.05)  ● 弱关联 (>0.01)  · 无关联");
}

// ═══════════════════════════════════════════════════════════════════════════════
// 输出函数
// ═══════════════════════════════════════════════════════════════════════════════

fn print_network_summary(stemcell: &StemCellComplete) {
    println!("细胞数:     {}", stemcell.cells.len());
    println!("总连接数:   {}", stemcell.cells.iter().map(|c| c.connections.len()).sum::<usize>());
    println!("维度:       {:.2}", stemcell.space.current_dimension);
    
    let energies: Vec<f64> = stemcell.cells.iter().map(|c| c.energy).collect();
    let avg_energy = energies.iter().sum::<f64>() / energies.len().max(1) as f64;
    println!("平均能量:   {:.4}", avg_energy);
}

fn print_topology(topology: &NetworkTopology) {
    println!("网络拓扑结构:");
    println!("  总细胞:     {}", topology.total_cells);
    println!("  总连接:     {}", topology.total_connections);
    println!("  核心细胞:   {} ({:.1}%)", 
        topology.core_cells.len(),
        topology.core_cells.len() as f64 / topology.total_cells as f64 * 100.0);
    println!("  边缘细胞:   {} ({:.1}%)",
        topology.peripheral_cells.len(),
        topology.peripheral_cells.len() as f64 / topology.total_cells as f64 * 100.0);
    println!("  孤立细胞:   {} ({:.1}%)",
        topology.isolated_cells.len(),
        topology.isolated_cells.len() as f64 / topology.total_cells as f64 * 100.0);
    println!("  枢纽细胞:   {}", topology.hub_cells.len());
    println!("  连通簇数:   {}", topology.clusters.len());
    
    if !topology.clusters.is_empty() {
        println!("\n  前 3 个最大簇:");
        for (i, cluster) in topology.clusters.iter().take(3).enumerate() {
            println!("    簇 {}: {} 个细胞", i + 1, cluster.len());
        }
    }
}

fn print_association_chain(chain: &[AssociationNode]) {
    if chain.is_empty() {
        println!("  (空链)");
        return;
    }
    
    for (i, node) in chain.iter().enumerate() {
        let strength_symbol = match node.strength_class {
            StrengthClass::Strong => "●●●",
            StrengthClass::Medium => " ●●",
            StrengthClass::Weak => "  ●",
        };
        
        if i == 0 {
            println!("  起点: {} | {}", node.cell_id, node.data_summary);
        } else {
            let arrow = match node.strength_class {
                StrengthClass::Strong => "═══>",
                StrengthClass::Medium => "───>",
                StrengthClass::Weak => "···>",
            };
            println!("    {} [{}] {} | {} | 强度: {:.4}", 
                arrow, strength_symbol, node.cell_id, node.data_summary, node.connection_strength);
        }
    }
}

fn print_strength_distribution(dist: &StrengthDistribution) {
    let total = dist.strong_count + dist.medium_count + dist.weak_count;
    if total == 0 {
        println!("  无连接数据");
        return;
    }
    
    println!("关联强度分布:");
    println!();
    
    let strong_pct = dist.strong_count as f64 / total as f64 * 100.0;
    let medium_pct = dist.medium_count as f64 / total as f64 * 100.0;
    let weak_pct = dist.weak_count as f64 / total as f64 * 100.0;
    
    // 绘制条形图
    let bar_width = 40;
    let strong_bar = "█".repeat((strong_pct / 100.0 * bar_width as f64) as usize);
    let medium_bar = "▓".repeat((medium_pct / 100.0 * bar_width as f64) as usize);
    let weak_bar = "░".repeat((weak_pct / 100.0 * bar_width as f64) as usize);
    
    println!("  强关联 (>0.1):  {:>6} ({:>5.1}%) {} 平均: {:.4}",
        dist.strong_count, strong_pct, strong_bar, dist.strong_avg);
    println!("  中关联 (0.05-0.1): {:>6} ({:>5.1}%) {} 平均: {:.4}",
        dist.medium_count, medium_pct, medium_bar, dist.medium_avg);
    println!("  弱关联 (<0.05): {:>6} ({:>5.1}%) {} 平均: {:.4}",
        dist.weak_count, weak_pct, weak_bar, dist.weak_avg);
    
    println!();
    println!("  总连接: {}", total);
}

fn print_propagation_result(result: &PropagationResult) {
    println!("传播分析:");
    println!("  直接激活:   {} 细胞", result.directly_activated);
    println!("  第 1 跳:    {} 细胞", result.first_hop);
    println!("  第 2 跳:    {} 细胞", result.second_hop);
    println!("  第 3 跳:    {} 细胞", result.third_hop);
    println!("  总覆盖:     {} 细胞 ({:.1}%)", result.total_reached, result.reach_percentage);
    
    if !result.strongest_associations.is_empty() {
        println!();
        println!("  最强关联:");
        for (id, strength, class) in &result.strongest_associations {
            let symbol = match class {
                StrengthClass::Strong => "●●●",
                StrengthClass::Medium => " ●●",
                StrengthClass::Weak => "  ●",
            };
            println!("    [{}] {} (强度: {:.4})", symbol, id, strength);
        }
    }
}

fn summarize_interaction(
    stemcell: &StemCellComplete, 
    topology: &NetworkTopology,
    strength_dist: &StrengthDistribution
) {
    println!("1. 网络结构:");
    if topology.clusters.len() == 1 && topology.clusters[0].len() > topology.total_cells * 80 / 100 {
        println!("   → 高度集成：大部分细胞形成单一核心簇");
    } else if topology.isolated_cells.len() > topology.total_cells * 10 / 100 {
        println!("   → 双峰分布：存在显著的核心-边缘分离");
    } else {
        println!("   → 多簇结构：网络形成多个独立群体");
    }
    
    println!();
    println!("2. 关联强度:");
    let total = strength_dist.strong_count + strength_dist.medium_count + strength_dist.weak_count;
    if total > 0 {
        let strong_ratio = strength_dist.strong_count as f64 / total as f64;
        if strong_ratio > 0.5 {
            println!("   → 强关联主导 ({:.1}%)：概念间联系紧密", strong_ratio * 100.0);
        } else if strong_ratio > 0.2 {
            println!("   → 混合关联：强 {:.1}%, 中 {:.1}%, 弱 {:.1}%",
                strength_dist.strong_count as f64 / total as f64 * 100.0,
                strength_dist.medium_count as f64 / total as f64 * 100.0,
                strength_dist.weak_count as f64 / total as f64 * 100.0);
        } else {
            println!("   → 弱关联主导：概念间联系较松散");
        }
    }
    
    println!();
    println!("3. 交互能力评估:");
    if topology.hub_cells.len() > topology.total_cells * 5 / 100 {
        println!("   → 具有枢纽节点：能够桥接不同概念群体");
    }
    
    let avg_conn = topology.total_connections as f64 / topology.total_cells.max(1) as f64;
    if avg_conn > 10.0 {
        println!("   → 高连接密度 ({:.1}/细胞)：信息传播能力强", avg_conn);
    } else if avg_conn > 5.0 {
        println!("   → 中等连接密度 ({:.1}/细胞)：具备基本传播能力", avg_conn);
    } else {
        println!("   → 低连接密度 ({:.1}/细胞)：传播能力有限", avg_conn);
    }
}
