//! StemCell + Fate 维度驱动的智能测试
//!
//! 利用 Fate 自身的维度判断能力来增强关联学习
//! 而不是使用固定的高维嵌入

use stem_cell::StemCellComplete;
use fate_f64::TripleFateCombined;

fn main() {
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("       Fate 维度驱动的智能测试");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    // 初始化
    let mut stemcell = StemCellComplete::new();
    
    println!("📊 初始状态:");
    println!("   细胞数: {}", stemcell.cells.len());
    println!("   Space 维度: {:.2}", stemcell.space.current_dimension);
    println!("   Fate Tick: {}", stemcell.triple_fate.global_tick);
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 阶段 1: 观察 Fate + evolve 对不同数据的维度响应
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("  阶段 1: Fate 维度响应观察 (使用 evolve 触发)");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    // 简单数据 + evolve
    println!("   📝 输入简单数据 (0-9) + evolve:");
    let dim_before = stemcell.space.current_dimension;
    let tick_before = stemcell.triple_fate.global_tick;
    
    for i in 0..100 {
        let v = (i % 10) as f64;
        stemcell.receive_data(v);
    }
    // 触发 evolve 来激活维度自适应
    for _ in 0..10 {
        stemcell.evolve(0.5);
    }
    
    let dim_after_simple = stemcell.space.current_dimension;
    let tick_after_simple = stemcell.triple_fate.global_tick;
    
    println!("      维度: {:.2} → {:.2} (Δ={:.2})", 
        dim_before, dim_after_simple, dim_after_simple - dim_before);
    println!("      Tick: {} → {} (Δ={})", 
        tick_before, tick_after_simple, tick_after_simple - tick_before);
    println!("      细胞数: {}", stemcell.cells.len());
    println!();
    
    // 大量复杂数据 + evolve
    println!("   📝 输入复杂数据 (多模式混合) + evolve:");
    let dim_before = stemcell.space.current_dimension;
    let tick_before = stemcell.triple_fate.global_tick;
    
    for i in 0..500 {
        // 混合多种模式，使用更大的值
        let v = match i % 4 {
            0 => (i as f64).sin() * 1000.0,  // 更大的值
            1 => (2.0f64).powi(i as i32 % 15),
            2 => (i as f64 * 0.1).cos() * 500.0,
            _ => (i * 7 % 1000) as f64,
        };
        stemcell.receive_data(v);
    }
    // 多次 evolve 触发维度自适应
    for _ in 0..20 {
        stemcell.evolve(0.8);
    }
    
    let dim_after_complex = stemcell.space.current_dimension;
    let tick_after_complex = stemcell.triple_fate.global_tick;
    
    println!("      维度: {:.2} → {:.2} (Δ={:.2})", 
        dim_before, dim_after_complex, dim_after_complex - dim_before);
    println!("      Tick: {} → {} (Δ={})", 
        tick_before, tick_after_complex, tick_after_complex - tick_before);
    println!("      细胞数: {}", stemcell.cells.len());
    println!();
    
    // 随机数据 + 高强度 evolve
    println!("   📝 输入随机数据 + 高强度 evolve:");
    let dim_before = stemcell.space.current_dimension;
    let tick_before = stemcell.triple_fate.global_tick;
    let mut rng = 12345u64;
    
    for _ in 0..500 {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
        let v = (rng % 10000) as f64;  // 更大的值范围
        stemcell.receive_data(v);
    }
    // 高强度 evolve
    for _ in 0..30 {
        stemcell.evolve(1.0);
    }
    
    let dim_after_random = stemcell.space.current_dimension;
    let tick_after_random = stemcell.triple_fate.global_tick;
    
    println!("      维度: {:.2} → {:.2} (Δ={:.2})", 
        dim_before, dim_after_random, dim_after_random - dim_before);
    println!("      Tick: {} → {} (Δ={})", 
        tick_before, tick_after_random, tick_after_random - tick_before);
    println!("      细胞数: {}", stemcell.cells.len());
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 阶段 2: Fate 驱动的关联学习
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("  阶段 2: Fate 驱动的关联学习");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    let associations = vec![
        ("cat", "meow"),
        ("dog", "bark"),
        ("bird", "chirp"),
        ("cow", "moo"),
    ];
    
    println!("   📚 建立关联 (使用 Fate 存储):");
    
    // 使用 Fate 的数据存储能力来建立关联
    for (a, b) in &associations {
        // 存储关联对
        let key = format!("{}_{}", a, b);
        let value = format!("{} -> {}", a, b);
        
        // 输入数据建立神经关联
        for _ in 0..20 {
            // 先输入 A
            for byte in a.bytes() {
                stemcell.receive_data(byte as f64);
            }
            // 紧接着输入 B（时间相关性）
            for byte in b.bytes() {
                stemcell.receive_data(byte as f64);
            }
        }
        
        // 使用 Fate 存储关联
        stemcell.triple_fate.store_string(&value);
        
        let dim_now = stemcell.space.current_dimension;
        println!("      {} -> {}: 维度={:.2}", a, b, dim_now);
    }
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 阶段 3: 测试关联检索
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("  阶段 3: 关联检索测试");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    println!("   🔍 Fate 存储检索:");
    
    // 检查 Fate 存储的数据
    let stored_keys: Vec<String> = stemcell.triple_fate.list_stored().iter().map(|s| s.to_string()).collect();
    println!("      存储条目数: {}", stored_keys.len());
    
    for (i, key) in stored_keys.iter().take(10).enumerate() {
        if let Some(entry) = stemcell.triple_fate.load_data(key) {
            let preview: String = entry.raw.iter().take(50).map(|&b| b as char).collect();
            println!("      {}: {} -> \"{}...\"", i + 1, key, preview);
        }
    }
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 阶段 4: 基于 Fate 状态的关联强度测量
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("  阶段 4: Fate 状态驱动的关联强度");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    let mut association_scores: Vec<f64> = Vec::new();
    
    for (a, b) in &associations {
        // 输入 A，记录 Fate 状态变化
        let state_before_a = capture_fate_state(&stemcell.triple_fate);
        for byte in a.bytes() {
            stemcell.receive_data(byte as f64);
        }
        let state_after_a = capture_fate_state(&stemcell.triple_fate);
        let a_signature = compute_state_delta(&state_before_a, &state_after_a);
        
        // 输入 B，记录 Fate 状态变化
        let state_before_b = capture_fate_state(&stemcell.triple_fate);
        for byte in b.bytes() {
            stemcell.receive_data(byte as f64);
        }
        let state_after_b = capture_fate_state(&stemcell.triple_fate);
        let b_signature = compute_state_delta(&state_before_b, &state_after_b);
        
        // 输入无关词
        let unrelated = "xyz123";
        let state_before_u = capture_fate_state(&stemcell.triple_fate);
        for byte in unrelated.bytes() {
            stemcell.receive_data(byte as f64);
        }
        let state_after_u = capture_fate_state(&stemcell.triple_fate);
        let u_signature = compute_state_delta(&state_before_u, &state_after_u);
        
        // 计算关联强度（基于 Fate 状态相似度）
        let ab_similarity = state_similarity(&a_signature, &b_signature);
        let au_similarity = state_similarity(&a_signature, &u_signature);
        
        let score = (ab_similarity - au_similarity + 1.0) / 2.0;
        association_scores.push(score);
        
        println!("      {} -> {}: AB_sim={:.3}, AU_sim={:.3}, 得分={:.3}",
            a, b, ab_similarity, au_similarity, score);
    }
    
    let avg_association = association_scores.iter().sum::<f64>() / association_scores.len() as f64;
    println!();
    println!("   🎯 Fate 状态关联得分: {:.2}/10", avg_association * 10.0);
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 阶段 5: 维度自适应能力测试
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("  阶段 5: 维度自适应能力");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    // 记录维度变化历史
    let mut dimension_history: Vec<f64> = Vec::new();
    dimension_history.push(stemcell.space.current_dimension);
    
    // 输入不同复杂度的数据，观察维度自适应
    let test_sequences = vec![
        ("常量", vec![1.0; 50]),
        ("线性", (0..50).map(|i| i as f64).collect()),
        ("指数", (0..50).map(|i| (1.5f64).powi(i % 10)).collect()),
        ("混合", (0..50).map(|i| (i as f64).sin() * (i as f64).cos() * 100.0).collect()),
    ];
    
    for (name, seq) in &test_sequences {
        let dim_before = stemcell.space.current_dimension;
        
        for &v in seq {
            stemcell.receive_data(v);
        }
        
        let dim_after = stemcell.space.current_dimension;
        dimension_history.push(dim_after);
        
        let change = dim_after - dim_before;
        let change_type = if change > 0.1 { "升维" } 
            else if change < -0.1 { "降维" } 
            else { "稳定" };
        
        println!("      {}: {:.2} → {:.2} ({}: Δ={:+.2})", 
            name, dim_before, dim_after, change_type, change);
    }
    println!();
    
    // 计算维度自适应得分
    let dim_range = dimension_history.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
        - dimension_history.iter().cloned().fold(f64::INFINITY, f64::min);
    let adaptability_score = (dim_range / 10.0 * 10.0).min(10.0);
    
    println!("   📊 维度变化范围: {:.2}", dim_range);
    println!("   🎯 维度自适应得分: {:.2}/10", adaptability_score);
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 综合评估
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("                    Fate 维度智能综合评估");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    // 收集更多 Fate 状态指标
    let base_state = &stemcell.triple_fate.base.state;
    let meta_state = &stemcell.triple_fate.meta.state;
    
    println!("   📊 Fate 双层状态:");
    println!("      Base: 信号={}, 发现={}, 维度变化={}", 
        base_state.signal_count, base_state.discovery_count, base_state.dimension_changes);
    println!("      Meta: 信号={}, 发现={}, 维度变化={}", 
        meta_state.signal_count, meta_state.discovery_count, meta_state.dimension_changes);
    println!("      Ultimate发现: {}", stemcell.triple_fate.ultimate_discoveries.len());
    println!();
    
    // Fate 规则生成情况
    let base_rules = stemcell.triple_fate.base.rules.len();
    let meta_rules = stemcell.triple_fate.meta.rules.len();
    let emergent_rules = stemcell.triple_fate.meta.emergent_rules.len();
    
    println!("   🔧 Fate 规则:");
    println!("      Base 规则: {}", base_rules);
    println!("      Meta 规则: {}", meta_rules);
    println!("      涌现规则: {}", emergent_rules);
    println!();
    
    // 计算 Fate 智能得分
    let rule_score = ((base_rules + meta_rules + emergent_rules) as f64 / 10.0).min(10.0);
    let discovery_score = ((base_state.discovery_count + meta_state.discovery_count) as f64 / 100.0 * 10.0).min(10.0);
    let dimension_score = adaptability_score;
    let association_score_final = avg_association * 10.0;
    
    let total_score = (rule_score + discovery_score + dimension_score + association_score_final) / 4.0;
    
    println!("   ┌────────────────────────────┬────────┐");
    println!("   │ 指标                       │  得分  │");
    println!("   ├────────────────────────────┼────────┤");
    println!("   │ 规则生成能力               │ {:>5.2} │", rule_score);
    println!("   │ 发现能力                   │ {:>5.2} │", discovery_score);
    println!("   │ 维度自适应                 │ {:>5.2} │", dimension_score);
    println!("   │ 关联推理                   │ {:>5.2} │", association_score_final);
    println!("   ├────────────────────────────┼────────┤");
    println!("   │ Fate 智能总分              │ {:>5.2} │", total_score);
    println!("   └────────────────────────────┴────────┘");
    println!();
    
    // 最终状态
    println!("   🏁 最终 Fate 维度: {:.2}", stemcell.space.current_dimension);
    println!("   🏁 细胞总数: {}", stemcell.cells.len());
    println!("   🏁 Fate Tick: {}", stemcell.triple_fate.global_tick);
    println!();
    println!("═══════════════════════════════════════════════════════════════════════════════");
}

// ============================================================================
// Fate 状态捕获
// ============================================================================

#[derive(Clone)]
struct FateStateSnapshot {
    tick: u64,
    dimension: f64,
    signal_count: u64,
    discovery_count: u64,
    dimension_changes: u64,
    efficiency: f64,
}

fn capture_fate_state(fate: &TripleFateCombined) -> FateStateSnapshot {
    FateStateSnapshot {
        tick: fate.global_tick,
        dimension: fate.base.space.current_dimension,
        signal_count: fate.base.state.signal_count,
        discovery_count: fate.base.state.discovery_count,
        dimension_changes: fate.base.state.dimension_changes,
        efficiency: fate.base.state.efficiency,
    }
}

fn compute_state_delta(before: &FateStateSnapshot, after: &FateStateSnapshot) -> Vec<f64> {
    vec![
        (after.tick - before.tick) as f64,
        after.dimension - before.dimension,
        (after.signal_count - before.signal_count) as f64,
        (after.discovery_count - before.discovery_count) as f64,
        (after.dimension_changes - before.dimension_changes) as f64,
        after.efficiency - before.efficiency,
    ]
}

fn state_similarity(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    
    // 余弦相似度
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    
    if norm_a < 1e-10 || norm_b < 1e-10 {
        return 0.0;
    }
    
    dot / (norm_a * norm_b)
}
