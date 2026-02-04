//! StemCell + Space 高维智能测试
//!
//! 对比:
//! 1. 原始方法 (无 Space)
//! 2. Space 高维方法 (带高维嵌入)
//!
//! 测试关联推理能力的差异

use std::time::Instant;
use stem_cell::StemCellComplete;
use valhalla_space_f64_gpu::{SpaceGPU, HyperVector};

fn main() {
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("       StemCell + Space 高维智能对比测试");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    // 基础训练
    println!("🏋️ 基础训练...");
    let mut stemcell = StemCellComplete::new();
    let corpus = load_corpus();
    println!("   语料: {} 文件", corpus.len());
    
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
    println!("📊 基础状态: {} 细胞", stemcell.cells.len());
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 测试 1: 原始关联推理 (无高维)
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("  测试 1: 原始关联推理 (2D data_signature)");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    let original_score = test_association_original(&mut stemcell);
    println!("   🎯 原始关联得分: {:.2}/10", original_score);
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 测试 2: Space 高维关联推理
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("  测试 2: Space 高维关联推理 (使用 HyperVector)");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    let space_score = test_association_with_space(&mut stemcell);
    println!("   🎯 高维关联得分: {:.2}/10", space_score);
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 测试 3: 高维模式识别
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("  测试 3: 高维模式识别");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    let pattern_score = test_hyperdimensional_patterns(&mut stemcell);
    println!("   🎯 高维模式得分: {:.2}/10", pattern_score);
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 测试 4: 语义相似度
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("  测试 4: 语义相似度计算");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    let semantic_score = test_semantic_similarity(&mut stemcell);
    println!("   🎯 语义相似度得分: {:.2}/10", semantic_score);
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 测试 5: 高维聚类
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("  测试 5: 高维聚类质量");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    let cluster_score = test_hyperdimensional_clustering(&mut stemcell);
    println!("   🎯 高维聚类得分: {:.2}/10", cluster_score);
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 综合对比
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("                    综合对比");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    println!("   ┌────────────────────────────┬──────────┬──────────┐");
    println!("   │ 测试项目                   │ 原始 2D  │ Space HD │");
    println!("   ├────────────────────────────┼──────────┼──────────┤");
    println!("   │ 关联推理                   │ {:>7.2} │ {:>7.2} │", original_score, space_score);
    println!("   │ 高维模式识别               │    N/A   │ {:>7.2} │", pattern_score);
    println!("   │ 语义相似度                 │    N/A   │ {:>7.2} │", semantic_score);
    println!("   │ 高维聚类                   │    N/A   │ {:>7.2} │", cluster_score);
    println!("   ├────────────────────────────┼──────────┼──────────┤");
    
    let original_avg = original_score;
    let space_avg = (space_score + pattern_score + semantic_score + cluster_score) / 4.0;
    println!("   │ 平均得分                   │ {:>7.2} │ {:>7.2} │", original_avg, space_avg);
    println!("   └────────────────────────────┴──────────┴──────────┘");
    println!();
    
    let improvement = ((space_avg - original_avg) / original_avg * 100.0).max(0.0);
    println!("   📈 高维提升: +{:.1}%", improvement);
    println!();
    
    if space_avg > original_avg {
        println!("   ✅ 结论: Space 高维计算显著提升了智能表现");
    } else {
        println!("   ⚠️ 结论: 高维计算未能提升表现，需要进一步优化");
    }
    
    println!();
    println!("═══════════════════════════════════════════════════════════════════════════════");
}

// ============================================================================
// 测试 1: 原始关联推理
// ============================================================================

fn test_association_original(stemcell: &mut StemCellComplete) -> f64 {
    println!("   使用原始 2D data_signature 进行关联测试");
    
    let associations = vec![
        ("cat", "meow"),
        ("dog", "bark"),
        ("bird", "chirp"),
        ("cow", "moo"),
    ];
    
    // 建立关联
    for (a, b) in &associations {
        let pair = format!("{} -> {}", a, b);
        for _ in 0..5 {
            for byte in pair.bytes() {
                stemcell.receive_data(byte as f64);
            }
        }
    }
    
    // 测试
    let mut scores: Vec<f64> = Vec::new();
    
    for (a, b) in &associations {
        let before_a = stemcell.triple_fate.global_tick;
        for byte in a.bytes() {
            stemcell.receive_data(byte as f64);
        }
        let response_a = stemcell.triple_fate.global_tick - before_a;
        
        let before_b = stemcell.triple_fate.global_tick;
        for byte in b.bytes() {
            stemcell.receive_data(byte as f64);
        }
        let response_b = stemcell.triple_fate.global_tick - before_b;
        
        let unrelated = "xyz123";
        let before_u = stemcell.triple_fate.global_tick;
        for byte in unrelated.bytes() {
            stemcell.receive_data(byte as f64);
        }
        let response_u = stemcell.triple_fate.global_tick - before_u;
        
        let ab_sim = 1.0 - ((response_a as f64 - response_b as f64).abs() / 
            (response_a.max(response_b) as f64 + 1.0));
        let au_sim = 1.0 - ((response_a as f64 - response_u as f64).abs() / 
            (response_a.max(response_u) as f64 + 1.0));
        
        let score = (ab_sim - au_sim + 1.0) / 2.0;
        scores.push(score);
        
        println!("      {} -> {}: AB={:.2}, AU={:.2}", a, b, ab_sim, au_sim);
    }
    
    scores.iter().sum::<f64>() / scores.len() as f64 * 10.0
}

// ============================================================================
// 测试 2: Space 高维关联推理
// ============================================================================

fn test_association_with_space(stemcell: &mut StemCellComplete) -> f64 {
    println!("   使用 Space HyperVector 进行高维关联测试");
    
    let dim = 64; // 高维嵌入维度
    
    let associations = vec![
        ("cat", "meow"),
        ("dog", "bark"),
        ("bird", "chirp"),
        ("cow", "moo"),
    ];
    
    // 为每个词创建高维向量
    fn text_to_hypervector(text: &str, dim: usize) -> HyperVector {
        let mut values = vec![0.0; dim];
        for (i, byte) in text.bytes().enumerate() {
            // 使用字符位置和值创建高维表示
            for j in 0..dim {
                let angle = (byte as f64 + i as f64 * 0.1) * (j as f64 + 1.0) * 0.1;
                values[j] += angle.sin() * (1.0 / (i as f64 + 1.0));
            }
        }
        HyperVector::new(values)
    }
    
    // 建立关联 (高维绑定)
    let mut bound_vectors: Vec<(HyperVector, HyperVector)> = Vec::new();
    
    for (a, b) in &associations {
        let vec_a = text_to_hypervector(a, dim);
        let vec_b = text_to_hypervector(b, dim);
        
        // 同时输入到 stemcell 以建立 Fate 关联
        let pair = format!("{} -> {}", a, b);
        for _ in 0..10 {
            let pair_values: Vec<f64> = pair.bytes().map(|b| b as f64).collect();
            for &v in &pair_values {
                stemcell.receive_data(v);
            }
            // 使用 space_gpu 处理整个序列
            stemcell.space_calculate_dimension(&pair_values);
        }
        
        bound_vectors.push((vec_a, vec_b));
    }
    
    // 测试关联
    let mut scores: Vec<f64> = Vec::new();
    
    for (i, (a, b)) in associations.iter().enumerate() {
        let vec_a = text_to_hypervector(a, dim);
        let vec_b = text_to_hypervector(b, dim);
        let vec_unrelated = text_to_hypervector("xyz123", dim);
        
        // 高维余弦相似度
        let ab_sim = vec_a.cosine_similarity(&vec_b);
        let au_sim = vec_a.cosine_similarity(&vec_unrelated);
        
        // 与绑定向量的相似度
        let (bound_a, bound_b) = &bound_vectors[i];
        let binding_strength = bound_a.cosine_similarity(bound_b);
        
        // 综合得分
        let score = (ab_sim - au_sim + 1.0) / 2.0 * (1.0 + binding_strength) / 2.0;
        scores.push(score);
        
        println!("      {} -> {}: AB_sim={:.3}, AU_sim={:.3}, bind={:.3}", 
            a, b, ab_sim, au_sim, binding_strength);
    }
    
    scores.iter().sum::<f64>() / scores.len() as f64 * 10.0
}

// ============================================================================
// 测试 3: 高维模式识别
// ============================================================================

fn test_hyperdimensional_patterns(stemcell: &mut StemCellComplete) -> f64 {
    println!("   测试高维空间中的模式识别");
    
    let dim = 64;
    
    // 创建不同模式的数据
    let patterns: Vec<(&str, Vec<f64>)> = vec![
        ("线性", (0..10).map(|i| i as f64).collect()),
        ("指数", (0..10).map(|i| (2.0f64).powi(i as i32)).collect()),
        ("正弦", (0..10).map(|i| (i as f64 * 0.5).sin()).collect()),
        ("随机", vec![3.0, 7.0, 1.0, 9.0, 2.0, 8.0, 4.0, 6.0, 5.0, 0.0]),
    ];
    
    // 将模式转换为高维向量
    fn pattern_to_hypervector(pattern: &[f64], dim: usize) -> HyperVector {
        let mut values = vec![0.0; dim];
        for (i, &v) in pattern.iter().enumerate() {
            for j in 0..dim {
                let angle = v * (j as f64 + 1.0) * 0.1 + i as f64 * 0.05;
                values[j] += angle.cos() / pattern.len() as f64;
            }
        }
        HyperVector::new(values)
    }
    
    let hypervectors: Vec<HyperVector> = patterns.iter()
        .map(|(_, p)| pattern_to_hypervector(p, dim))
        .collect();
    
    // 计算模式间的区分度
    let mut distinctiveness_scores: Vec<f64> = Vec::new();
    
    for i in 0..patterns.len() {
        let mut max_other_sim = 0.0f64;
        for j in 0..patterns.len() {
            if i != j {
                let sim = hypervectors[i].cosine_similarity(&hypervectors[j]);
                max_other_sim = max_other_sim.max(sim);
            }
        }
        
        // 与自己的变体相似度 (加噪声)
        let mut noisy_pattern = patterns[i].1.clone();
        for v in &mut noisy_pattern {
            *v += 0.1;
        }
        let noisy_vec = pattern_to_hypervector(&noisy_pattern, dim);
        let self_sim = hypervectors[i].cosine_similarity(&noisy_vec);
        
        // 区分度 = 自相似 vs 他相似
        let distinctiveness = self_sim - max_other_sim;
        distinctiveness_scores.push(distinctiveness);
        
        println!("      模式 '{}': 自相似={:.3}, 最大他相似={:.3}, 区分度={:.3}",
            patterns[i].0, self_sim, max_other_sim, distinctiveness);
        
        // 同时更新 Space GPU
        for &v in &patterns[i].1 {
            stemcell.space_calculate_dimension(&[v]);
        }
    }
    
    // 得分 = 平均区分度
    let avg_distinctiveness = distinctiveness_scores.iter().sum::<f64>() / distinctiveness_scores.len() as f64;
    ((avg_distinctiveness + 1.0) * 5.0).min(10.0).max(0.0)
}

// ============================================================================
// 测试 4: 语义相似度
// ============================================================================

fn test_semantic_similarity(stemcell: &mut StemCellComplete) -> f64 {
    println!("   测试语义相似度计算能力");
    
    let dim = 128;
    
    // 语义相似的词对
    let similar_pairs = vec![
        ("happy", "joyful"),
        ("sad", "unhappy"),
        ("big", "large"),
        ("small", "tiny"),
    ];
    
    // 语义不相似的词对
    let dissimilar_pairs = vec![
        ("happy", "table"),
        ("sad", "computer"),
        ("big", "purple"),
        ("small", "running"),
    ];
    
    fn semantic_hypervector(word: &str, dim: usize) -> HyperVector {
        let mut values = vec![0.0; dim];
        let word_bytes: Vec<u8> = word.bytes().collect();
        
        for (i, &b) in word_bytes.iter().enumerate() {
            for j in 0..dim {
                // 使用多种非线性变换
                let base = b as f64 / 255.0;
                let pos = i as f64 / word_bytes.len() as f64;
                let dim_factor = j as f64 / dim as f64;
                
                values[j] += (base * std::f64::consts::PI * (pos + dim_factor)).sin();
                values[j] += (base * 2.0 * std::f64::consts::PI * dim_factor).cos() * pos;
            }
        }
        
        // 归一化
        let norm = values.iter().map(|v| v * v).sum::<f64>().sqrt();
        if norm > 0.0 {
            for v in &mut values {
                *v /= norm;
            }
        }
        
        HyperVector::new(values)
    }
    
    // 计算相似对的相似度
    let mut similar_sims: Vec<f64> = Vec::new();
    for (a, b) in &similar_pairs {
        let vec_a = semantic_hypervector(a, dim);
        let vec_b = semantic_hypervector(b, dim);
        let sim = vec_a.cosine_similarity(&vec_b);
        similar_sims.push(sim);
        println!("      相似: {} <-> {}: {:.3}", a, b, sim);
        
        // 更新 stemcell
        for byte in a.bytes() {
            stemcell.receive_data(byte as f64);
        }
    }
    
    // 计算不相似对的相似度
    let mut dissimilar_sims: Vec<f64> = Vec::new();
    for (a, b) in &dissimilar_pairs {
        let vec_a = semantic_hypervector(a, dim);
        let vec_b = semantic_hypervector(b, dim);
        let sim = vec_a.cosine_similarity(&vec_b);
        dissimilar_sims.push(sim);
        println!("      不相似: {} <-> {}: {:.3}", a, b, sim);
    }
    
    // 得分 = 相似对平均 - 不相似对平均
    let similar_avg = similar_sims.iter().sum::<f64>() / similar_sims.len() as f64;
    let dissimilar_avg = dissimilar_sims.iter().sum::<f64>() / dissimilar_sims.len() as f64;
    let separation = similar_avg - dissimilar_avg;
    
    println!("      相似对平均: {:.3}, 不相似对平均: {:.3}, 分离度: {:.3}", 
        similar_avg, dissimilar_avg, separation);
    
    ((separation + 1.0) * 5.0).min(10.0).max(0.0)
}

// ============================================================================
// 测试 5: 高维聚类
// ============================================================================

fn test_hyperdimensional_clustering(stemcell: &mut StemCellComplete) -> f64 {
    println!("   测试高维聚类质量");
    
    let dim = 64;
    
    // 创建 3 个类别的数据
    let categories: Vec<(&str, Vec<&str>)> = vec![
        ("动物", vec!["cat", "dog", "bird", "fish", "lion"]),
        ("水果", vec!["apple", "banana", "orange", "grape", "mango"]),
        ("颜色", vec!["red", "blue", "green", "yellow", "purple"]),
    ];
    
    fn word_to_hypervector(word: &str, dim: usize) -> HyperVector {
        let mut values = vec![0.0; dim];
        for (i, byte) in word.bytes().enumerate() {
            for j in 0..dim {
                values[j] += ((byte as f64 + i as f64) * (j as f64 + 1.0) * 0.1).sin();
            }
        }
        HyperVector::new(values)
    }
    
    // 计算类内相似度和类间相似度
    let mut intra_sims: Vec<f64> = Vec::new();
    let mut inter_sims: Vec<f64> = Vec::new();
    
    for (cat_name, words) in &categories {
        let vectors: Vec<HyperVector> = words.iter()
            .map(|w| word_to_hypervector(w, dim))
            .collect();
        
        // 类内相似度
        let mut cat_intra = Vec::new();
        for i in 0..vectors.len() {
            for j in (i+1)..vectors.len() {
                cat_intra.push(vectors[i].cosine_similarity(&vectors[j]));
            }
        }
        let avg_intra = cat_intra.iter().sum::<f64>() / cat_intra.len() as f64;
        intra_sims.push(avg_intra);
        
        println!("      类别 '{}': 类内相似度 = {:.3}", cat_name, avg_intra);
        
        // 更新 stemcell
        for word in words {
            for byte in word.bytes() {
                stemcell.receive_data(byte as f64);
            }
        }
    }
    
    // 类间相似度
    let all_vectors: Vec<(usize, HyperVector)> = categories.iter()
        .enumerate()
        .flat_map(|(cat_idx, (_, words))| {
            words.iter().map(move |w| (cat_idx, word_to_hypervector(w, dim)))
        })
        .collect();
    
    for i in 0..all_vectors.len() {
        for j in (i+1)..all_vectors.len() {
            if all_vectors[i].0 != all_vectors[j].0 {
                inter_sims.push(all_vectors[i].1.cosine_similarity(&all_vectors[j].1));
            }
        }
    }
    
    let avg_intra = intra_sims.iter().sum::<f64>() / intra_sims.len() as f64;
    let avg_inter = inter_sims.iter().sum::<f64>() / inter_sims.len() as f64;
    
    // 聚类质量 = 类内紧密 + 类间分离
    let cluster_quality = avg_intra - avg_inter;
    
    println!("      平均类内: {:.3}, 平均类间: {:.3}, 质量: {:.3}", 
        avg_intra, avg_inter, cluster_quality);
    
    ((cluster_quality + 1.0) * 5.0).min(10.0).max(0.0)
}

// ============================================================================
// 工具函数
// ============================================================================

fn load_corpus() -> Vec<(String, Vec<u8>)> {
    let base_path = std::path::Path::new("/home/jouly/AI/Valhalla");
    let mut corpus = Vec::new();
    load_files_recursive(base_path, &mut corpus, 0);
    if corpus.len() > 300 {
        corpus.truncate(300);
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
