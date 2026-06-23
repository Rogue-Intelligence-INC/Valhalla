//! StemCell 交互测试模块
//! 
//! 基于训练后的细胞网络进行交互：
//! 1. 输入刺激 → 转换为信号
//! 2. 激活相关细胞 → 能量沿连接传播
//! 3. 收集响应 → 返回激活模式

use stem_cell::StemCellComplete;
use std::collections::{HashMap, HashSet};

fn main() {
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("   StemCell 交互测试 - 刺激-响应模式分析");
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!();
    
    // ═══ 阶段 1: 训练系统 ═══
    println!("【阶段 1】训练系统建立基础网络...\n");
    
    let mut stemcell = StemCellComplete::new();
    
    // 丰富的训练数据（建立概念网络）
    let training_corpus = vec![
        // 数字概念
        ("number", vec!["zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten"]),
        // 颜色概念
        ("color", vec!["red", "green", "blue", "yellow", "orange", "purple", "black", "white", "pink"]),
        // 动物概念
        ("animal", vec!["cat", "dog", "bird", "fish", "lion", "tiger", "elephant", "mouse", "rabbit"]),
        // 动作概念
        ("action", vec!["run", "walk", "jump", "swim", "fly", "sleep", "eat", "drink", "think"]),
        // 代码概念
        ("code", vec!["fn", "let", "if", "else", "for", "while", "struct", "impl", "pub", "mod"]),
        // 关联短语
        ("phrase", vec!["cat runs", "dog walks", "bird flies", "fish swims", "red cat", "blue fish"]),
    ];
    
    for (category, items) in &training_corpus {
        for item in items {
            // 输入类别标签
            for ch in category.chars() {
                stemcell.receive_data(ch as u8 as f64);
            }
            stemcell.receive_data(0.0);  // 分隔
            
            // 输入项目
            for ch in item.chars() {
                stemcell.receive_data(ch as u8 as f64);
            }
            stemcell.receive_data(0.0);  // 分隔
        }
    }
    
    // 演化稳定
    for _ in 0..50 {
        stemcell.evolve(0.0);
    }
    
    // 空跑整理（让网络稳定）
    for _ in 0..100 {
        stemcell.evolve(0.0);
    }
    
    println!("训练完成！");
    println!("  细胞数: {}", stemcell.cells.len());
    println!("  总连接数: {}", stemcell.cells.iter().map(|c| c.connections.len()).sum::<usize>());
    println!("  维度: {:.2}", stemcell.space.current_dimension);
    
    // ═══ 阶段 2: 交互测试 ═══
    println!("\n【阶段 2】交互测试\n");
    
    // 测试不同的刺激
    let test_stimuli = vec![
        "cat",
        "red",
        "run",
        "fn",
        "cat runs",
        "blue fish",
        "unknown",  // 未知词
        "xyz",      // 完全随机
    ];
    
    for stimulus in &test_stimuli {
        println!("─────────────────────────────────────────────────────────────────────────────");
        println!("刺激: \"{}\"", stimulus);
        println!();
        
        let response = stimulate_and_respond(&mut stemcell, stimulus);
        
        println!("响应分析:");
        println!("  直接激活细胞: {}", response.directly_activated);
        println!("  传播激活细胞: {}", response.propagated_activated);
        println!("  总激活能量:   {:.4}", response.total_activation_energy);
        println!("  激活深度:     {}", response.activation_depth);
        
        if !response.activated_signatures.is_empty() {
            println!("  关联概念:");
            for (sig, strength) in response.activated_signatures.iter().take(5) {
                println!("    - {} (强度: {:.4})", format_signature(sig), strength);
            }
        }
        
        println!();
    }
    
    // ═══ 阶段 3: 关联性测试 ═══
    println!("\n【阶段 3】关联性测试 - 检查概念是否形成关联\n");
    
    let association_tests = vec![
        ("cat", "dog"),      // 同类（动物）
        ("cat", "runs"),     // 动作关联
        ("red", "blue"),     // 同类（颜色）
        ("cat", "fn"),       // 不同类
        ("fn", "struct"),    // 同类（代码）
    ];
    
    for (word_a, word_b) in &association_tests {
        let association = measure_association(&stemcell, word_a, word_b);
        let relationship = if association > 0.7 {
            "强关联"
        } else if association > 0.3 {
            "中等关联"
        } else if association > 0.1 {
            "弱关联"
        } else {
            "无关联"
        };
        
        println!("  \"{}\" ↔ \"{}\": {:.4} ({})", word_a, word_b, association, relationship);
    }
    
    // ═══ 阶段 4: 网络拓扑分析 ═══
    println!("\n【阶段 4】网络拓扑分析\n");
    
    analyze_network_topology(&stemcell);
    
    // ═══ 阶段 5: 模拟对话 ═══
    println!("\n【阶段 5】模拟简单对话\n");
    
    let dialogue_inputs = vec![
        "cat",
        "what color",
        "action",
        "code example",
    ];
    
    for input in &dialogue_inputs {
        println!("用户输入: \"{}\"", input);
        
        // 刺激系统
        let response = stimulate_and_respond(&mut stemcell, input);
        
        // 基于激活模式生成简单响应
        let output = generate_response(&response);
        println!("系统响应: {}", output);
        println!();
    }
    
    // ═══ 结论 ═══
    println!("\n═══════════════════════════════════════════════════════════════════════════");
    println!("   交互能力评估");
    println!("═══════════════════════════════════════════════════════════════════════════\n");
    
    // 计算整体交互能力
    let mut total_activation = 0.0;
    let mut total_propagation = 0;
    
    for stimulus in &test_stimuli {
        let r = stimulate_and_respond(&mut stemcell, stimulus);
        total_activation += r.total_activation_energy;
        total_propagation += r.propagated_activated;
    }
    
    let avg_activation = total_activation / test_stimuli.len() as f64;
    let avg_propagation = total_propagation as f64 / test_stimuli.len() as f64;
    
    println!("平均激活能量: {:.4}", avg_activation);
    println!("平均传播范围: {:.1} 细胞", avg_propagation);
    
    if avg_propagation > 10.0 && avg_activation > 1.0 {
        println!("\n评估: 系统具有较好的响应和传播能力");
    } else if avg_propagation > 5.0 || avg_activation > 0.5 {
        println!("\n评估: 系统具有基本的响应能力，但传播有限");
    } else {
        println!("\n评估: 系统响应能力较弱，需要更多训练");
    }
}

/// 交互响应结构
struct InteractionResponse {
    directly_activated: usize,
    propagated_activated: usize,
    total_activation_energy: f64,
    activation_depth: usize,
    activated_signatures: Vec<(Vec<f64>, f64)>,
}

/// 刺激系统并获取响应
fn stimulate_and_respond(stemcell: &mut StemCellComplete, stimulus: &str) -> InteractionResponse {
    // 1. 将刺激转换为信号特征
    let stimulus_signature: Vec<f64> = stimulus.chars()
        .map(|c| c as u8 as f64)
        .collect();
    
    // 2. 找到直接相关的细胞（基于特征相似度）
    let mut activation_map: HashMap<String, f64> = HashMap::new();
    let mut directly_activated = 0;
    
    for cell in &stemcell.cells {
        let similarity = compute_similarity(&stimulus_signature, &cell.data_signature);
        if similarity > 0.1 {  // 相似度阈值
            activation_map.insert(cell.cell_id.clone(), similarity * cell.energy);
            directly_activated += 1;
        }
    }
    
    // 3. 沿连接传播激活（模拟能量传播）
    let mut propagated: HashSet<String> = HashSet::new();
    let mut current_layer: Vec<String> = activation_map.keys().cloned().collect();
    let mut depth = 0;
    let max_depth = 3;  // 最大传播深度
    
    while !current_layer.is_empty() && depth < max_depth {
        let mut next_layer: Vec<String> = Vec::new();
        
        for cell_id in &current_layer {
            // 找到这个细胞
            if let Some(cell) = stemcell.cells.iter().find(|c| &c.cell_id == cell_id) {
                let source_activation = *activation_map.get(cell_id).unwrap_or(&0.0);
                
                // 沿连接传播
                for (connected_id, connection_strength) in &cell.connections {
                    if !activation_map.contains_key(connected_id) {
                        // 传播的激活 = 源激活 * 连接强度 * 衰减
                        let propagated_activation = source_activation * connection_strength * 0.5;
                        
                        if propagated_activation > 0.01 {  // 激活阈值
                            activation_map.insert(connected_id.clone(), propagated_activation);
                            propagated.insert(connected_id.clone());
                            next_layer.push(connected_id.clone());
                        }
                    }
                }
            }
        }
        
        current_layer = next_layer;
        depth += 1;
    }
    
    // 4. 收集激活的细胞特征
    let mut activated_signatures: Vec<(Vec<f64>, f64)> = Vec::new();
    for (cell_id, activation) in &activation_map {
        if let Some(cell) = stemcell.cells.iter().find(|c| &c.cell_id == cell_id) {
            if !cell.data_signature.is_empty() {
                activated_signatures.push((cell.data_signature.clone(), *activation));
            }
        }
    }
    
    // 按激活强度排序
    activated_signatures.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    
    let total_activation_energy: f64 = activation_map.values().sum();
    
    InteractionResponse {
        directly_activated,
        propagated_activated: propagated.len(),
        total_activation_energy,
        activation_depth: depth,
        activated_signatures,
    }
}

/// 计算两个特征向量的相似度
fn compute_similarity(a: &[f64], b: &[f64]) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    
    // 使用余弦相似度
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
    
    // 归一化到 [0, 1]
    ((dot / (norm_a.sqrt() * norm_b.sqrt())) + 1.0) / 2.0
}

/// 测量两个概念之间的关联度
fn measure_association(stemcell: &StemCellComplete, word_a: &str, word_b: &str) -> f64 {
    let sig_a: Vec<f64> = word_a.chars().map(|c| c as u8 as f64).collect();
    let sig_b: Vec<f64> = word_b.chars().map(|c| c as u8 as f64).collect();
    
    // 找到与 word_a 最相关的细胞
    let mut cells_a: Vec<(&str, f64)> = Vec::new();
    for cell in &stemcell.cells {
        let sim = compute_similarity(&sig_a, &cell.data_signature);
        if sim > 0.1 {
            cells_a.push((&cell.cell_id, sim));
        }
    }
    
    // 找到与 word_b 最相关的细胞
    let mut cells_b: Vec<(&str, f64)> = Vec::new();
    for cell in &stemcell.cells {
        let sim = compute_similarity(&sig_b, &cell.data_signature);
        if sim > 0.1 {
            cells_b.push((&cell.cell_id, sim));
        }
    }
    
    if cells_a.is_empty() || cells_b.is_empty() {
        return 0.0;
    }
    
    // 检查是否有直接连接
    let mut max_connection_strength = 0.0;
    
    for (id_a, _) in &cells_a {
        if let Some(cell_a) = stemcell.cells.iter().find(|c| &c.cell_id == *id_a) {
            for (id_b, _) in &cells_b {
                // 检查 cell_a 是否连接到 cell_b
                for (conn_id, strength) in &cell_a.connections {
                    if conn_id == *id_b {
                        max_connection_strength = max_connection_strength.max(*strength);
                    }
                }
            }
        }
    }
    
    // 也检查位置邻近度
    let mut min_distance = f64::MAX;
    for (id_a, _) in &cells_a {
        if let Some(cell_a) = stemcell.cells.iter().find(|c| &c.cell_id == *id_a) {
            for (id_b, _) in &cells_b {
                if let Some(cell_b) = stemcell.cells.iter().find(|c| &c.cell_id == *id_b) {
                    let dist = hyperdim_distance(&cell_a.position, &cell_b.position);
                    min_distance = min_distance.min(dist);
                }
            }
        }
    }
    
    // 综合连接强度和距离来计算关联度
    let distance_factor = if min_distance < f64::MAX {
        1.0 / (1.0 + min_distance)
    } else {
        0.0
    };
    
    (max_connection_strength + distance_factor) / 2.0
}

/// 高维距离计算
fn hyperdim_distance(a: &[f64], b: &[f64]) -> f64 {
    let min_len = a.len().min(b.len());
    let mut sum = 0.0;
    for i in 0..min_len {
        let diff = a[i] - b[i];
        sum += diff * diff;
    }
    sum.sqrt()
}

/// 分析网络拓扑
fn analyze_network_topology(stemcell: &StemCellComplete) {
    let cells = &stemcell.cells;
    if cells.is_empty() {
        println!("无细胞");
        return;
    }
    
    // 计算连接度分布
    let mut conn_counts: Vec<usize> = cells.iter().map(|c| c.connections.len()).collect();
    conn_counts.sort();
    
    let total_connections: usize = conn_counts.iter().sum();
    let avg_connections = total_connections as f64 / cells.len() as f64;
    let max_connections = *conn_counts.last().unwrap_or(&0);
    let min_connections = *conn_counts.first().unwrap_or(&0);
    
    println!("连接统计:");
    println!("  平均连接数: {:.2}", avg_connections);
    println!("  最大连接数: {}", max_connections);
    println!("  最小连接数: {}", min_connections);
    
    // 识别核心节点（高连接度）
    let core_threshold = (avg_connections * 1.5) as usize;
    let core_count = conn_counts.iter().filter(|&&c| c >= core_threshold).count();
    
    println!("  核心节点数 (连接 >= {}): {} ({:.1}%)", 
        core_threshold, core_count, core_count as f64 / cells.len() as f64 * 100.0);
    
    // 识别孤立节点
    let isolated_count = conn_counts.iter().filter(|&&c| c == 0).count();
    println!("  孤立节点数: {} ({:.1}%)", 
        isolated_count, isolated_count as f64 / cells.len() as f64 * 100.0);
    
    // 计算聚类系数（简化版）
    let mut total_clustering = 0.0;
    let mut counted = 0;
    
    for cell in cells {
        if cell.connections.len() >= 2 {
            // 检查这个细胞的邻居之间是否也互连
            let neighbors: HashSet<&str> = cell.connections.iter()
                .map(|(id, _)| id.as_str())
                .collect();
            
            let mut neighbor_connections = 0;
            for (neighbor_id, _) in &cell.connections {
                if let Some(neighbor) = cells.iter().find(|c| &c.cell_id == neighbor_id) {
                    for (n2_id, _) in &neighbor.connections {
                        if neighbors.contains(n2_id.as_str()) {
                            neighbor_connections += 1;
                        }
                    }
                }
            }
            
            let possible_connections = cell.connections.len() * (cell.connections.len() - 1);
            if possible_connections > 0 {
                total_clustering += neighbor_connections as f64 / possible_connections as f64;
                counted += 1;
            }
        }
    }
    
    let avg_clustering = if counted > 0 { total_clustering / counted as f64 } else { 0.0 };
    println!("  平均聚类系数: {:.4}", avg_clustering);
}

/// 格式化特征向量为可读形式
fn format_signature(sig: &[f64]) -> String {
    // 尝试将数值转换回字符
    let chars: String = sig.iter()
        .filter_map(|&v| {
            let byte = v as u8;
            if byte >= 32 && byte <= 126 {
                Some(byte as char)
            } else {
                None
            }
        })
        .collect();
    
    if chars.is_empty() {
        format!("[{:.2}, ...]", sig.first().unwrap_or(&0.0))
    } else {
        chars
    }
}

/// 基于激活模式生成简单响应
fn generate_response(response: &InteractionResponse) -> String {
    if response.directly_activated == 0 {
        return "我不理解这个输入。".to_string();
    }
    
    if response.activated_signatures.is_empty() {
        return "我感知到了刺激，但没有相关记忆。".to_string();
    }
    
    // 收集最强激活的概念
    let concepts: Vec<String> = response.activated_signatures.iter()
        .take(3)
        .map(|(sig, _)| format_signature(sig))
        .filter(|s| !s.is_empty() && !s.starts_with('['))
        .collect();
    
    if concepts.is_empty() {
        format!("激活了 {} 个细胞，传播了 {} 层。", 
            response.directly_activated, response.activation_depth)
    } else {
        format!("相关概念: {} | 激活强度: {:.2}", 
            concepts.join(", "), response.total_activation_energy)
    }
}
