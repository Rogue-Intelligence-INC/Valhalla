//! StemCell 智能性测试
//!
//! 测试维度:
//! 1. 模式识别 - 能否识别重复模式
//! 2. 记忆能力 - 对相同输入的反应一致性
//! 3. 泛化能力 - 对相似输入的迁移反应
//! 4. 异常检测 - 区分正常与异常数据
//! 5. 学习效率 - 处理相同数据的效率变化
//! 6. 关联推理 - 相关数据的关联反应
//! 7. 预测能力 - 对序列数据的预测

use std::collections::HashMap;
use std::time::Instant;
use stem_cell::StemCellComplete;

fn main() {
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("       StemCell 智能性测试");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    // 先进行基础训练
    println!("🏋️ 基础训练...");
    let mut stemcell = StemCellComplete::new();
    
    let corpus = load_corpus();
    println!("   语料: {} 文件", corpus.len());
    
    // 3次收敛训练
    for iter in 1..=3 {
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
            if stable >= 3 || round >= 8 { break; }
            last_count = current;
        }
        println!("   迭代 {}: {} 细胞", iter, stemcell.cells.len());
    }
    
    println!();
    println!("📊 基础状态: {} 细胞, Fate Tick: {}", 
        stemcell.cells.len(), stemcell.triple_fate.global_tick);
    println!();
    
    let mut total_score = 0.0;
    let mut test_count = 0;
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 测试 1: 模式识别
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("  测试 1: 模式识别能力");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    let pattern_score = test_pattern_recognition(&mut stemcell);
    println!("   🎯 模式识别得分: {:.2}/10", pattern_score);
    total_score += pattern_score;
    test_count += 1;
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 测试 2: 记忆一致性
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("  测试 2: 记忆一致性");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    let memory_score = test_memory_consistency(&mut stemcell);
    println!("   🎯 记忆一致性得分: {:.2}/10", memory_score);
    total_score += memory_score;
    test_count += 1;
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 测试 3: 泛化能力
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("  测试 3: 泛化能力");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    let generalization_score = test_generalization(&mut stemcell);
    println!("   🎯 泛化能力得分: {:.2}/10", generalization_score);
    total_score += generalization_score;
    test_count += 1;
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 测试 4: 异常检测
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("  测试 4: 异常检测能力");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    let anomaly_score = test_anomaly_detection(&mut stemcell);
    println!("   🎯 异常检测得分: {:.2}/10", anomaly_score);
    total_score += anomaly_score;
    test_count += 1;
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 测试 5: 学习效率
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("  测试 5: 学习效率");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    let learning_score = test_learning_efficiency(&mut stemcell);
    println!("   🎯 学习效率得分: {:.2}/10", learning_score);
    total_score += learning_score;
    test_count += 1;
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 测试 6: 关联推理
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("  测试 6: 关联推理能力");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    let association_score = test_association(&mut stemcell);
    println!("   🎯 关联推理得分: {:.2}/10", association_score);
    total_score += association_score;
    test_count += 1;
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 测试 7: 序列预测
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("  测试 7: 序列预测能力");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    let prediction_score = test_sequence_prediction(&mut stemcell);
    println!("   🎯 序列预测得分: {:.2}/10", prediction_score);
    total_score += prediction_score;
    test_count += 1;
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 综合评估
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("                    智能性综合评估");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    let avg_score = total_score / test_count as f64;
    
    println!("   ┌────────────────────────────┬────────┐");
    println!("   │ 测试项目                   │  得分  │");
    println!("   ├────────────────────────────┼────────┤");
    println!("   │ 模式识别能力               │ {:>5.2} │", pattern_score);
    println!("   │ 记忆一致性                 │ {:>5.2} │", memory_score);
    println!("   │ 泛化能力                   │ {:>5.2} │", generalization_score);
    println!("   │ 异常检测能力               │ {:>5.2} │", anomaly_score);
    println!("   │ 学习效率                   │ {:>5.2} │", learning_score);
    println!("   │ 关联推理能力               │ {:>5.2} │", association_score);
    println!("   │ 序列预测能力               │ {:>5.2} │", prediction_score);
    println!("   ├────────────────────────────┼────────┤");
    println!("   │ 智能性总分                 │ {:>5.2} │", avg_score);
    println!("   └────────────────────────────┴────────┘");
    println!();
    
    // 智能等级
    let level = if avg_score >= 8.0 {
        ("高级智能", "具备复杂推理和抽象能力")
    } else if avg_score >= 6.0 {
        ("中级智能", "具备基本学习和适应能力")
    } else if avg_score >= 4.0 {
        ("初级智能", "具备简单模式识别能力")
    } else if avg_score >= 2.0 {
        ("反应性行为", "具备基本刺激-反应能力")
    } else {
        ("无智能特征", "仅有随机反应")
    };
    
    println!("   🧠 智能等级: {}", level.0);
    println!("   📝 描述: {}", level.1);
    println!();
    
    // 各维度分析
    println!("   📊 智能特征分析:");
    
    if pattern_score >= 7.0 {
        println!("      ✅ 模式识别: 能有效识别数据中的重复结构");
    } else if pattern_score >= 4.0 {
        println!("      ⚠️ 模式识别: 能识别简单模式，复杂模式能力有限");
    } else {
        println!("      ❌ 模式识别: 模式识别能力较弱");
    }
    
    if memory_score >= 7.0 {
        println!("      ✅ 记忆系统: 对相同输入有稳定一致的反应");
    } else if memory_score >= 4.0 {
        println!("      ⚠️ 记忆系统: 记忆有一定波动");
    } else {
        println!("      ❌ 记忆系统: 记忆不稳定");
    }
    
    if generalization_score >= 7.0 {
        println!("      ✅ 泛化能力: 能将学习迁移到相似情境");
    } else if generalization_score >= 4.0 {
        println!("      ⚠️ 泛化能力: 有限的迁移学习能力");
    } else {
        println!("      ❌ 泛化能力: 泛化能力较弱，倾向于过拟合");
    }
    
    if anomaly_score >= 7.0 {
        println!("      ✅ 异常检测: 能有效识别异常数据");
    } else if anomaly_score >= 4.0 {
        println!("      ⚠️ 异常检测: 对明显异常有感知");
    } else {
        println!("      ❌ 异常检测: 无法有效区分正常与异常");
    }
    
    if learning_score >= 7.0 {
        println!("      ✅ 学习效率: 学习曲线明显，效率持续提升");
    } else if learning_score >= 4.0 {
        println!("      ⚠️ 学习效率: 有一定学习能力");
    } else {
        println!("      ❌ 学习效率: 学习效率提升不明显");
    }
    
    if association_score >= 7.0 {
        println!("      ✅ 关联推理: 能建立数据间的关联");
    } else if association_score >= 4.0 {
        println!("      ⚠️ 关联推理: 能建立简单关联");
    } else {
        println!("      ❌ 关联推理: 关联推理能力较弱");
    }
    
    if prediction_score >= 7.0 {
        println!("      ✅ 序列预测: 能预测序列趋势");
    } else if prediction_score >= 4.0 {
        println!("      ⚠️ 序列预测: 能捕捉简单序列模式");
    } else {
        println!("      ❌ 序列预测: 序列预测能力较弱");
    }
    
    println!();
    println!("═══════════════════════════════════════════════════════════════════════════════");
}

// ============================================================================
// 测试 1: 模式识别
// ============================================================================

fn test_pattern_recognition(stemcell: &mut StemCellComplete) -> f64 {
    println!("   测试: 输入重复模式，检测系统响应");
    
    // 测试不同模式
    let patterns = vec![
        vec![1.0, 2.0, 3.0, 1.0, 2.0, 3.0, 1.0, 2.0, 3.0],  // 简单重复
        vec![1.0, 1.0, 2.0, 1.0, 1.0, 2.0, 1.0, 1.0, 2.0],  // AAB 模式
        vec![1.0, 2.0, 4.0, 8.0, 16.0, 32.0],               // 指数增长
        vec![10.0, 5.0, 10.0, 5.0, 10.0, 5.0],              // 振荡
    ];
    
    let mut pattern_responses: Vec<f64> = Vec::new();
    
    for (i, pattern) in patterns.iter().enumerate() {
        // 输入模式3次
        let mut responses = Vec::new();
        for _ in 0..3 {
            let before_tick = stemcell.triple_fate.global_tick;
            let before_cells = stemcell.cells.len();
            
            for &v in pattern {
                stemcell.receive_data(v * 10.0);
            }
            
            let tick_delta = stemcell.triple_fate.global_tick - before_tick;
            let cell_delta = stemcell.cells.len() as i64 - before_cells as i64;
            
            responses.push((tick_delta as f64, cell_delta as f64));
        }
        
        // 检查响应一致性
        let tick_responses: Vec<f64> = responses.iter().map(|(t, _)| *t).collect();
        let (mean, std) = mean_std(&tick_responses);
        let cv = if mean > 0.0 { std / mean } else { 1.0 };
        
        let consistency = (1.0 - cv.min(1.0)) * 10.0;
        pattern_responses.push(consistency);
        
        println!("      模式 {}: 响应一致性 = {:.2}", i + 1, consistency);
    }
    
    let avg = pattern_responses.iter().sum::<f64>() / pattern_responses.len() as f64;
    avg
}

// ============================================================================
// 测试 2: 记忆一致性
// ============================================================================

fn test_memory_consistency(stemcell: &mut StemCellComplete) -> f64 {
    println!("   测试: 重复输入相同数据，检测响应一致性");
    
    let test_data = "Hello World Test Data 测试数据";
    let bytes: Vec<f64> = test_data.bytes().map(|b| b as f64).collect();
    
    let mut responses: Vec<(u64, usize, usize)> = Vec::new();
    
    for trial in 0..5 {
        let before_tick = stemcell.triple_fate.global_tick;
        let before_cells = stemcell.cells.len();
        let mut new_cells = 0;
        
        for &b in &bytes {
            let result = stemcell.receive_data(b);
            new_cells += result.new_cells;
        }
        
        let tick_delta = stemcell.triple_fate.global_tick - before_tick;
        responses.push((tick_delta, new_cells, stemcell.cells.len() - before_cells));
        
        println!("      试验 {}: tick_delta={}, new_cells={}", trial + 1, tick_delta, new_cells);
    }
    
    // 计算一致性
    let tick_deltas: Vec<f64> = responses.iter().map(|(t, _, _)| *t as f64).collect();
    let (mean, std) = mean_std(&tick_deltas);
    let cv = if mean > 0.0 { std / mean } else { 1.0 };
    
    let score = (1.0 - cv.min(1.0)) * 10.0;
    println!("      tick_delta 变异系数: {:.4}", cv);
    
    score
}

// ============================================================================
// 测试 3: 泛化能力
// ============================================================================

fn test_generalization(stemcell: &mut StemCellComplete) -> f64 {
    println!("   测试: 输入相似但不同的数据，检测响应相似度");
    
    // 原始数据和变体
    let original = "function calculate(x) { return x * 2; }";
    let variants = vec![
        "function calculate(y) { return y * 2; }",      // 变量名变化
        "function compute(x) { return x * 2; }",        // 函数名变化
        "function calculate(x) { return x * 3; }",      // 常数变化
        "fn calculate(x) { return x * 2; }",            // 关键字变化
    ];
    
    // 先输入原始数据，记录响应
    let orig_bytes: Vec<f64> = original.bytes().map(|b| b as f64).collect();
    let before = stemcell.triple_fate.global_tick;
    for &b in &orig_bytes {
        stemcell.receive_data(b);
    }
    let orig_response = stemcell.triple_fate.global_tick - before;
    
    println!("      原始: tick_delta = {}", orig_response);
    
    // 输入变体，比较响应
    let mut similarities: Vec<f64> = Vec::new();
    
    for (i, variant) in variants.iter().enumerate() {
        let var_bytes: Vec<f64> = variant.bytes().map(|b| b as f64).collect();
        let before = stemcell.triple_fate.global_tick;
        for &b in &var_bytes {
            stemcell.receive_data(b);
        }
        let var_response = stemcell.triple_fate.global_tick - before;
        
        // 计算响应相似度
        let diff = (orig_response as f64 - var_response as f64).abs();
        let max_val = orig_response.max(var_response) as f64;
        let similarity = if max_val > 0.0 { 1.0 - (diff / max_val) } else { 1.0 };
        
        similarities.push(similarity);
        println!("      变体 {}: tick_delta = {}, 相似度 = {:.2}", i + 1, var_response, similarity);
    }
    
    let avg_similarity = similarities.iter().sum::<f64>() / similarities.len() as f64;
    avg_similarity * 10.0
}

// ============================================================================
// 测试 4: 异常检测
// ============================================================================

fn test_anomaly_detection(stemcell: &mut StemCellComplete) -> f64 {
    println!("   测试: 输入正常和异常数据，检测响应差异");
    
    // 正常数据: ASCII 范围
    let normal_data: Vec<f64> = (32..127).map(|x| x as f64).collect();
    
    // 异常数据: 极端值
    let anomaly_data: Vec<f64> = vec![
        0.0, 0.0, 0.0,           // 全零
        255.0, 255.0, 255.0,     // 最大值
        1000.0, 2000.0, 3000.0,  // 超范围
        -100.0, -200.0, -300.0,  // 负值
    ];
    
    // 正常数据响应
    let mut normal_responses: Vec<f64> = Vec::new();
    for chunk in normal_data.chunks(10) {
        let before = stemcell.triple_fate.global_tick;
        let before_cells = stemcell.cells.len();
        for &v in chunk {
            stemcell.receive_data(v);
        }
        let tick_delta = (stemcell.triple_fate.global_tick - before) as f64;
        let cell_delta = (stemcell.cells.len() - before_cells) as f64;
        normal_responses.push(tick_delta + cell_delta * 100.0);
    }
    
    let (normal_mean, normal_std) = mean_std(&normal_responses);
    println!("      正常数据: 均值={:.1}, 标准差={:.1}", normal_mean, normal_std);
    
    // 异常数据响应
    let mut anomaly_responses: Vec<f64> = Vec::new();
    for chunk in anomaly_data.chunks(3) {
        let before = stemcell.triple_fate.global_tick;
        let before_cells = stemcell.cells.len();
        for &v in chunk {
            stemcell.receive_data(v);
        }
        let tick_delta = (stemcell.triple_fate.global_tick - before) as f64;
        let cell_delta = (stemcell.cells.len() - before_cells) as f64;
        anomaly_responses.push(tick_delta + cell_delta * 100.0);
    }
    
    let (anomaly_mean, anomaly_std) = mean_std(&anomaly_responses);
    println!("      异常数据: 均值={:.1}, 标准差={:.1}", anomaly_mean, anomaly_std);
    
    // 检测能力 = 正常和异常的分离度
    let separation = (normal_mean - anomaly_mean).abs() / (normal_std + anomaly_std + 0.01);
    let score = (separation * 2.0).min(10.0);
    
    println!("      分离度: {:.2}", separation);
    
    score
}

// ============================================================================
// 测试 5: 学习效率
// ============================================================================

fn test_learning_efficiency(stemcell: &mut StemCellComplete) -> f64 {
    println!("   测试: 重复输入相同数据，检测处理效率变化");
    
    let test_data = "Learning efficiency test data for repeated processing";
    let bytes: Vec<f64> = test_data.bytes().map(|b| b as f64).collect();
    
    let mut processing_times: Vec<f64> = Vec::new();
    let mut cell_creations: Vec<f64> = Vec::new();
    
    for trial in 0..10 {
        let start = Instant::now();
        let before_cells = stemcell.cells.len();
        
        for _ in 0..100 {
            for &b in &bytes {
                stemcell.receive_data(b);
            }
        }
        
        let elapsed = start.elapsed().as_secs_f64();
        let new_cells = stemcell.cells.len() - before_cells;
        
        processing_times.push(elapsed);
        cell_creations.push(new_cells as f64);
        
        println!("      试验 {}: {:.4}s, +{} cells", trial + 1, elapsed, new_cells);
    }
    
    // 检查是否有学习曲线（效率提升）
    // 比较前半和后半的效率
    let first_half_time = processing_times[..5].iter().sum::<f64>() / 5.0;
    let second_half_time = processing_times[5..].iter().sum::<f64>() / 5.0;
    
    // 细胞创建率下降 = 学习效率提升（已学会的不需要创建新细胞）
    let first_half_cells = cell_creations[..5].iter().sum::<f64>() / 5.0;
    let second_half_cells = cell_creations[5..].iter().sum::<f64>() / 5.0;
    
    println!("      前半平均: {:.4}s, {:.0} cells", first_half_time, first_half_cells);
    println!("      后半平均: {:.4}s, {:.0} cells", second_half_time, second_half_cells);
    
    // 学习效率 = 细胞创建率下降比例
    let cell_efficiency = if first_half_cells > 0.0 {
        1.0 - (second_half_cells / first_half_cells)
    } else {
        0.0
    };
    
    // 时间效率 = 处理时间是否稳定或提升
    let time_efficiency = if first_half_time > 0.0 {
        1.0 - (second_half_time / first_half_time).min(1.0)
    } else {
        0.0
    };
    
    let score = ((cell_efficiency.max(0.0) + time_efficiency.max(0.0)) * 5.0).min(10.0);
    
    println!("      细胞效率提升: {:.1}%, 时间效率提升: {:.1}%", 
        cell_efficiency * 100.0, time_efficiency * 100.0);
    
    score
}

// ============================================================================
// 测试 6: 关联推理
// ============================================================================

fn test_association(stemcell: &mut StemCellComplete) -> f64 {
    println!("   测试: 输入关联数据对，检测关联建立");
    
    // 关联对
    let associations = vec![
        ("cat", "meow"),
        ("dog", "bark"),
        ("bird", "chirp"),
        ("cow", "moo"),
    ];
    
    // 先建立关联
    println!("      建立关联...");
    for (a, b) in &associations {
        let pair = format!("{} -> {}", a, b);
        for _ in 0..5 {
            for byte in pair.bytes() {
                stemcell.receive_data(byte as f64);
            }
        }
    }
    
    // 测试关联
    println!("      测试关联...");
    let mut association_scores: Vec<f64> = Vec::new();
    
    for (a, b) in &associations {
        // 输入 A，记录状态
        let before_a = stemcell.triple_fate.global_tick;
        for byte in a.bytes() {
            stemcell.receive_data(byte as f64);
        }
        let response_a = stemcell.triple_fate.global_tick - before_a;
        
        // 输入 B，记录状态
        let before_b = stemcell.triple_fate.global_tick;
        for byte in b.bytes() {
            stemcell.receive_data(byte as f64);
        }
        let response_b = stemcell.triple_fate.global_tick - before_b;
        
        // 输入无关词
        let unrelated = "xyz123";
        let before_u = stemcell.triple_fate.global_tick;
        for byte in unrelated.bytes() {
            stemcell.receive_data(byte as f64);
        }
        let response_u = stemcell.triple_fate.global_tick - before_u;
        
        // 关联强度 = A->B 的响应相似度 vs A->U 的响应相似度
        let ab_similarity = 1.0 - ((response_a as f64 - response_b as f64).abs() / 
            (response_a.max(response_b) as f64 + 1.0));
        let au_similarity = 1.0 - ((response_a as f64 - response_u as f64).abs() / 
            (response_a.max(response_u) as f64 + 1.0));
        
        // 关联得分 = AB相似度应该高于AU相似度
        let score = (ab_similarity - au_similarity + 1.0) / 2.0;
        association_scores.push(score);
        
        println!("      {} -> {}: AB={:.2}, AU={:.2}, 得分={:.2}", 
            a, b, ab_similarity, au_similarity, score);
    }
    
    let avg = association_scores.iter().sum::<f64>() / association_scores.len() as f64;
    avg * 10.0
}

// ============================================================================
// 测试 7: 序列预测
// ============================================================================

fn test_sequence_prediction(stemcell: &mut StemCellComplete) -> f64 {
    println!("   测试: 输入序列，检测预测能力");
    
    // 测试序列
    let sequences = vec![
        vec![1.0, 2.0, 3.0, 4.0, 5.0],           // 线性
        vec![1.0, 2.0, 4.0, 8.0, 16.0],          // 指数
        vec![1.0, 1.0, 2.0, 3.0, 5.0, 8.0],      // 斐波那契
        vec![1.0, 0.0, 1.0, 0.0, 1.0, 0.0],      // 振荡
    ];
    
    let mut prediction_scores: Vec<f64> = Vec::new();
    
    for (i, seq) in sequences.iter().enumerate() {
        // 训练序列 (多次输入)
        for _ in 0..10 {
            for &v in seq {
                stemcell.receive_data(v * 10.0);
            }
        }
        
        // 记录序列前部分的响应模式
        let mut responses: Vec<u64> = Vec::new();
        for &v in &seq[..seq.len()-1] {
            let before = stemcell.triple_fate.global_tick;
            stemcell.receive_data(v * 10.0);
            responses.push(stemcell.triple_fate.global_tick - before);
        }
        
        // 预测下一个的响应
        let before = stemcell.triple_fate.global_tick;
        let predicted_response = responses.last().copied().unwrap_or(0);
        
        // 实际响应
        let last_val = seq.last().copied().unwrap_or(0.0);
        stemcell.receive_data(last_val * 10.0);
        let actual_response = stemcell.triple_fate.global_tick - before;
        
        // 预测准确度
        let diff = (predicted_response as f64 - actual_response as f64).abs();
        let max_val = predicted_response.max(actual_response) as f64 + 1.0;
        let accuracy = 1.0 - (diff / max_val);
        
        prediction_scores.push(accuracy);
        println!("      序列 {}: 预测={}, 实际={}, 准确度={:.2}", 
            i + 1, predicted_response, actual_response, accuracy);
    }
    
    let avg = prediction_scores.iter().sum::<f64>() / prediction_scores.len() as f64;
    avg * 10.0
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
    if corpus.len() > 500 {
        corpus.truncate(500);
    }
    corpus
}

fn load_files_recursive(path: &std::path::Path, corpus: &mut Vec<(String, Vec<u8>)>, depth: usize) {
    if depth > 4 { return; }
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
