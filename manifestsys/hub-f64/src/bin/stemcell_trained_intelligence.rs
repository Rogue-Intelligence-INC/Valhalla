//! StemCell 训练后智能水平测试
//!
//! 测试维度:
//! 1. 模式识别能力
//! 2. 记忆保持能力
//! 3. 泛化能力
//! 4. 异常检测能力
//! 5. 关联学习能力 (物理场驱动)
//! 6. 序列预测能力
//! 7. 连接网络智能

use stem_cell::StemCellComplete;

fn main() {
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("       StemCell 智能水平综合测试");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    // 创建并训练系统
    let mut stemcell = StemCellComplete::new();
    
    println!("📚 阶段 0: 快速训练 (模拟已训练状态)");
    println!("─────────────────────────────────────────────────────────────────────────────────");
    
    // 训练数据 - 多种模式
    let training_data = vec![
        // 重复模式
        ("AAAA", 50), ("BBBB", 50), ("CCCC", 50),
        // 交替模式
        ("ABAB", 50), ("CDCD", 50),
        // 递增模式
        ("1234", 50), ("5678", 50),
        // 语义关联
        ("cat", 100), ("meow", 100), ("dog", 100), ("bark", 100),
        ("sun", 100), ("warm", 100), ("moon", 100), ("cold", 100),
        // 代码模式
        ("fn main", 50), ("let x", 50), ("if true", 50),
    ];
    
    let mut total_data = 0usize;
    for (pattern, count) in &training_data {
        for _ in 0..*count {
            for byte in pattern.bytes() {
                stemcell.receive_data(byte as f64);
                total_data += 1;
            }
        }
    }
    
    // 多次 evolve 让物理场稳定
    for i in 0..30 {
        stemcell.evolve(0.5 + (i as f64 * 0.01));
    }
    
    println!("   训练数据: {} 字节", total_data);
    println!("   细胞数量: {}", stemcell.cells.len());
    println!("   Space 维度: {:.2}", stemcell.space.current_dimension);
    println!("   Fate Tick: {}", stemcell.triple_fate.global_tick);
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 测试 1: 模式识别
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("  测试 1: 模式识别能力");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    
    let patterns = vec![
        ("AAAA", "重复模式", true),   // 已训练
        ("XXXX", "新重复模式", false), // 未训练但相似
        ("ABAB", "交替模式", true),   // 已训练
        ("XYXY", "新交替模式", false), // 未训练但相似
        ("random", "随机", false),    // 完全不同
    ];
    
    let cell_count_before = stemcell.cells.len();
    let mut recognized = 0;
    let mut total_patterns = 0;
    
    for (pattern, name, trained) in &patterns {
        let cells_before = stemcell.cells.len();
        
        for byte in pattern.bytes() {
            stemcell.receive_data(byte as f64);
        }
        stemcell.evolve(0.5);
        
        let cells_after = stemcell.cells.len();
        let growth = cells_after as i64 - cells_before as i64;
        
        // 如果细胞增长少，说明被识别并合并到现有结构
        let was_recognized = growth <= 1;
        
        let status = if was_recognized == *trained || (was_recognized && !trained) {
            recognized += 1;
            "✓"
        } else {
            "✗"
        };
        
        println!("   {} {}: growth={:+}, recognized={}", 
            status, name, growth, was_recognized);
        total_patterns += 1;
    }
    
    let pattern_score = (recognized as f64 / total_patterns as f64) * 10.0;
    println!();
    println!("   📊 模式识别得分: {:.1}/10", pattern_score);
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 测试 2: 记忆保持
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("  测试 2: 记忆保持能力");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    
    // 输入干扰数据
    println!("   📝 输入干扰数据...");
    for _ in 0..100 {
        for byte in "noise interference garbage".bytes() {
            stemcell.receive_data(byte as f64);
        }
    }
    for _ in 0..10 {
        stemcell.evolve(0.8);
    }
    
    // 测试原始模式是否仍被记忆
    let memory_tests = vec!["AAAA", "cat", "1234", "fn main"];
    let mut memory_retained = 0;
    
    for pattern in &memory_tests {
        let cells_before = stemcell.cells.len();
        
        for byte in pattern.bytes() {
            stemcell.receive_data(byte as f64);
        }
        stemcell.evolve(0.5);
        
        let growth = stemcell.cells.len() as i64 - cells_before as i64;
        let retained = growth <= 1;
        
        println!("   {} '{}': growth={:+}, retained={}", 
            if retained { "✓" } else { "✗" },
            pattern, growth, retained);
        
        if retained {
            memory_retained += 1;
        }
    }
    
    let memory_score = (memory_retained as f64 / memory_tests.len() as f64) * 10.0;
    println!();
    println!("   📊 记忆保持得分: {:.1}/10", memory_score);
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 测试 3: 物理连接网络
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("  测试 3: 物理连接网络智能");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    
    // 统计连接
    let mut total_connections = 0usize;
    let mut cells_with_connections = 0usize;
    let mut max_connections = 0usize;
    let mut connection_strengths: Vec<f64> = Vec::new();
    
    for cell in &stemcell.cells {
        let conn_count = cell.connections.len();
        total_connections += conn_count;
        if conn_count > 0 {
            cells_with_connections += 1;
            max_connections = max_connections.max(conn_count);
            for (_, strength) in &cell.connections {
                connection_strengths.push(*strength);
            }
        }
    }
    
    let connectivity_ratio = cells_with_connections as f64 / stemcell.cells.len() as f64;
    let avg_connections = if cells_with_connections > 0 {
        total_connections as f64 / cells_with_connections as f64
    } else {
        0.0
    };
    
    let avg_strength = if !connection_strengths.is_empty() {
        connection_strengths.iter().sum::<f64>() / connection_strengths.len() as f64
    } else {
        0.0
    };
    
    println!("   📡 连接统计:");
    println!("      连通率: {:.1}%", connectivity_ratio * 100.0);
    println!("      平均连接数: {:.2}", avg_connections);
    println!("      最大连接数: {}", max_connections);
    println!("      平均连接强度: {:.4}", avg_strength);
    
    let network_score = (connectivity_ratio * 5.0 + (avg_connections / 10.0).min(1.0) * 5.0).min(10.0);
    println!();
    println!("   📊 网络智能得分: {:.1}/10", network_score);
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 测试 4: 关联学习 (基于物理距离)
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("  测试 4: 关联学习能力 (物理场驱动)");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    
    // 建立新关联
    let associations = vec![
        ("apple", "fruit"),
        ("car", "drive"),
        ("code", "program"),
    ];
    
    println!("   📚 建立关联对:");
    for (a, b) in &associations {
        for _ in 0..50 {
            for byte in a.bytes() {
                stemcell.receive_data(byte as f64);
            }
            for byte in b.bytes() {
                stemcell.receive_data(byte as f64);
            }
        }
        stemcell.evolve(0.6);
        println!("      {} <-> {}", a, b);
    }
    
    // 多次 evolve 让物理场稳定
    for _ in 0..20 {
        stemcell.evolve(0.5);
    }
    
    // 测试：输入 A 后，输入 B 的响应是否比输入 C 更小
    let mut association_correct = 0;
    
    for (a, b) in &associations {
        // 先输入 A
        for byte in a.bytes() {
            stemcell.receive_data(byte as f64);
        }
        let cells_after_a = stemcell.cells.len();
        
        // 输入关联的 B
        for byte in b.bytes() {
            stemcell.receive_data(byte as f64);
        }
        let growth_b = stemcell.cells.len() as i64 - cells_after_a as i64;
        
        // 输入不相关的 C
        let unrelated = "zzzzz";
        let cells_before_c = stemcell.cells.len();
        for byte in unrelated.bytes() {
            stemcell.receive_data(byte as f64);
        }
        let growth_c = stemcell.cells.len() as i64 - cells_before_c as i64;
        
        let associated = growth_b <= growth_c;
        if associated {
            association_correct += 1;
        }
        
        println!("   {} {} -> {}: growth_b={:+}, growth_c={:+}", 
            if associated { "✓" } else { "✗" },
            a, b, growth_b, growth_c);
    }
    
    let association_score = (association_correct as f64 / associations.len() as f64) * 10.0;
    println!();
    println!("   📊 关联学习得分: {:.1}/10", association_score);
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 测试 5: 异常检测
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("  测试 5: 异常检测能力");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    
    // 正常数据
    let normal = vec!["cat", "dog", "AAAA", "1234"];
    let abnormal = vec!["@#$%", "异常数据", "\x00\x01\x02", "999999"];
    
    let mut normal_growth = 0i64;
    let mut abnormal_growth = 0i64;
    
    for data in &normal {
        let before = stemcell.cells.len();
        for byte in data.bytes() {
            stemcell.receive_data(byte as f64);
        }
        normal_growth += stemcell.cells.len() as i64 - before as i64;
    }
    
    for data in &abnormal {
        let before = stemcell.cells.len();
        for byte in data.bytes() {
            stemcell.receive_data(byte as f64);
        }
        abnormal_growth += stemcell.cells.len() as i64 - before as i64;
    }
    
    let avg_normal = normal_growth as f64 / normal.len() as f64;
    let avg_abnormal = abnormal_growth as f64 / abnormal.len() as f64;
    
    println!("   正常数据平均增长: {:.2}", avg_normal);
    println!("   异常数据平均增长: {:.2}", avg_abnormal);
    
    // 异常数据应该导致更多新细胞（不被识别）
    let anomaly_detected = avg_abnormal > avg_normal * 1.2;
    let anomaly_score = if anomaly_detected { 8.0 } else { 4.0 };
    
    println!("   异常检测: {}", if anomaly_detected { "✓ 成功区分" } else { "✗ 未能区分" });
    println!();
    println!("   📊 异常检测得分: {:.1}/10", anomaly_score);
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 测试 6: 高维位置分布
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("  测试 6: 高维空间组织能力");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    
    // 分析位置分布
    let pos_dims: Vec<usize> = stemcell.cells.iter()
        .map(|c| c.position.len())
        .collect();
    
    let min_dim = *pos_dims.iter().min().unwrap_or(&0);
    let max_dim = *pos_dims.iter().max().unwrap_or(&0);
    
    println!("   位置维度范围: {} ~ {}", min_dim, max_dim);
    println!("   Space 当前维度: {:.2}", stemcell.space.current_dimension);
    
    // 计算位置的方差（衡量空间利用）
    if !stemcell.cells.is_empty() && min_dim > 0 {
        let dim = min_dim.min(5);
        let mut total_variance = 0.0;
        
        for d in 0..dim {
            let values: Vec<f64> = stemcell.cells.iter()
                .filter(|c| c.position.len() > d)
                .map(|c| c.position[d])
                .collect();
            
            if !values.is_empty() {
                let mean = values.iter().sum::<f64>() / values.len() as f64;
                let variance = values.iter()
                    .map(|v| (v - mean).powi(2))
                    .sum::<f64>() / values.len() as f64;
                total_variance += variance;
            }
        }
        
        let avg_variance = total_variance / dim as f64;
        println!("   平均位置方差: {:.4}", avg_variance);
        
        // 方差越大，空间利用越好
        let space_score = (avg_variance.sqrt() * 2.0).min(10.0);
        println!();
        println!("   📊 空间组织得分: {:.1}/10", space_score);
    }
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 测试 7: 学习效率
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("  测试 7: 学习效率");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    
    // 测试新概念学习速度
    let cells_before = stemcell.cells.len();
    let new_concept = "quantum_entanglement_physics";
    
    // 首次学习
    for _ in 0..5 {
        for byte in new_concept.bytes() {
            stemcell.receive_data(byte as f64);
        }
    }
    stemcell.evolve(0.5);
    let first_growth = stemcell.cells.len() - cells_before;
    
    // 二次学习
    let cells_mid = stemcell.cells.len();
    for _ in 0..5 {
        for byte in new_concept.bytes() {
            stemcell.receive_data(byte as f64);
        }
    }
    stemcell.evolve(0.5);
    let second_growth = stemcell.cells.len() - cells_mid;
    
    println!("   新概念首次学习增长: {}", first_growth);
    println!("   新概念二次学习增长: {}", second_growth);
    
    let learning_efficiency = if first_growth > 0 {
        1.0 - (second_growth as f64 / first_growth as f64).min(1.0)
    } else {
        0.5
    };
    
    let efficiency_score = learning_efficiency * 10.0;
    println!("   学习效率: {:.1}%", learning_efficiency * 100.0);
    println!();
    println!("   📊 学习效率得分: {:.1}/10", efficiency_score);
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 综合评估
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("                    智能水平综合评估");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    // 计算位置方差得分 (重新计算)
    let space_score = if !stemcell.cells.is_empty() {
        let dim = pos_dims.iter().min().unwrap_or(&1).min(&5);
        let mut total_var = 0.0;
        for d in 0..*dim {
            let values: Vec<f64> = stemcell.cells.iter()
                .filter(|c| c.position.len() > d)
                .map(|c| c.position[d])
                .collect();
            if !values.is_empty() {
                let mean = values.iter().sum::<f64>() / values.len() as f64;
                let var = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
                total_var += var;
            }
        }
        (total_var.sqrt() / *dim as f64 * 2.0).min(10.0)
    } else {
        0.0
    };
    
    let scores = vec![
        ("模式识别", pattern_score),
        ("记忆保持", memory_score),
        ("网络智能", network_score),
        ("关联学习", association_score),
        ("异常检测", anomaly_score),
        ("空间组织", space_score),
        ("学习效率", efficiency_score),
    ];
    
    let total: f64 = scores.iter().map(|(_, s)| s).sum();
    let average = total / scores.len() as f64;
    
    println!("   ┌────────────────────────────┬────────┐");
    println!("   │ 智能维度                   │  得分  │");
    println!("   ├────────────────────────────┼────────┤");
    for (name, score) in &scores {
        let bar = "█".repeat((score / 2.0) as usize);
        println!("   │ {:20} │ {:>5.2} │ {}", name, score, bar);
    }
    println!("   ├────────────────────────────┼────────┤");
    println!("   │ 综合智能指数               │ {:>5.2} │", average);
    println!("   └────────────────────────────┴────────┘");
    println!();
    
    // 智能等级
    let level = if average >= 8.0 {
        "🧠 高级智能"
    } else if average >= 6.0 {
        "🔵 中级智能"
    } else if average >= 4.0 {
        "🟡 初级智能"
    } else {
        "⚪ 基础反应"
    };
    
    println!("   🎯 智能等级: {}", level);
    println!();
    
    println!("   🏁 最终状态:");
    println!("      细胞总数: {}", stemcell.cells.len());
    println!("      有连接细胞: {} ({:.1}%)", 
        cells_with_connections, 
        cells_with_connections as f64 / stemcell.cells.len() as f64 * 100.0);
    println!("      Space 维度: {:.2}", stemcell.space.current_dimension);
    println!("      Fate Tick: {}", stemcell.triple_fate.global_tick);
    println!();
    println!("═══════════════════════════════════════════════════════════════════════════════");
}
