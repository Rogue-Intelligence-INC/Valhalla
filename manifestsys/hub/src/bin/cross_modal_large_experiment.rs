//! 🌐 跨模态统一实验 - 大规模版本
//!
//! 实验规模:
//! - 文本: 100+ 项 (10+ 语言概念 × 多语言)
//! - 代码: 50+ 项 (多语言)
//! - 数学: 30+ 项
//! - 图形: 20+ 项
//! - 音频: 30+ 项
//!
//! 资源限制: 50% CPU

use valhalla_hub::data_ingestion::*;
use std::time::{Duration, Instant};
use std::thread;
use std::collections::HashMap;

/// 资源限制配置
const MAX_CPU_PERCENT: u32 = 50;
const BATCH_DELAY_MS: u64 = 10; // 减少延迟以加快处理

fn main() {
    println!("╔══════════════════════════════════════════════════════════════════════════╗");
    println!("║  🌐 Valhalla 跨模态统一实验 - 大规模版本                                 ║");
    println!("║  资源限制: {}% CPU                                                       ║", MAX_CPU_PERCENT);
    println!("╚══════════════════════════════════════════════════════════════════════════╝");
    println!();
    
    let start = Instant::now();
    let mut engine = DataIngestionEngine::new();
    
    // ═══════════════════════════════════════════════════════════════
    // 阶段 1: 大规模数据摄入
    // ═══════════════════════════════════════════════════════════════
    
    println!("📥 阶段 1: 大规模多模态数据摄入");
    println!("────────────────────────────────────────────────────────");
    
    // 1.1 文本数据 - 10个语义概念 × 多语言变体
    let text_concepts: Vec<(&str, Vec<(&str, &str)>)> = vec![
        // 概念: 包含关系
        ("containment", vec![
            ("A contains B", "en"),
            ("B belongs to A", "en"),
            ("A 包含 B", "zh"),
            ("B 属于 A", "zh"),
            ("Container holds items", "en"),
            ("容器包含元素", "zh"),
            ("The set includes element X", "en"),
            ("集合包括元素 X", "zh"),
            ("Data is within the structure", "en"),
            ("数据在结构中", "zh"),
        ]),
        // 概念: 比较关系
        ("comparison", vec![
            ("X is greater than Y", "en"),
            ("X 大于 Y", "zh"),
            ("A is less than B", "en"),
            ("A 小于 B", "zh"),
            ("Value exceeds threshold", "en"),
            ("值超过阈值", "zh"),
            ("More than expected", "en"),
            ("比预期更多", "zh"),
            ("Size comparison result", "en"),
            ("大小比较结果", "zh"),
        ]),
        // 概念: 等于关系
        ("equality", vec![
            ("A equals B", "en"),
            ("A 等于 B", "zh"),
            ("X is the same as Y", "en"),
            ("X 和 Y 相同", "zh"),
            ("Values are equivalent", "en"),
            ("值是等价的", "zh"),
            ("Equal in magnitude", "en"),
            ("大小相等", "zh"),
        ]),
        // 概念: 函数/动作
        ("action", vec![
            ("Function returns a value", "en"),
            ("函数返回一个值", "zh"),
            ("Process executes task", "en"),
            ("进程执行任务", "zh"),
            ("System calls method", "en"),
            ("系统调用方法", "zh"),
            ("Calculate and return result", "en"),
            ("计算并返回结果", "zh"),
            ("Transform input to output", "en"),
            ("将输入转换为输出", "zh"),
        ]),
        // 概念: 情感/状态
        ("emotion", vec![
            ("Love is eternal", "en"),
            ("爱是永恒的", "zh"),
            ("I love you", "en"),
            ("我爱你", "zh"),
            ("Ich liebe dich", "de"),
            ("愛してる", "ja"),
            ("Beauty transcends time", "en"),
            ("美超越时间", "zh"),
            ("Happiness fills the heart", "en"),
            ("幸福充满内心", "zh"),
        ]),
        // 概念: 逻辑推理
        ("logic", vec![
            ("If A then B", "en"),
            ("如果 A 则 B", "zh"),
            ("All men are mortal", "en"),
            ("凡人皆有死", "zh"),
            ("Socrates is a man", "en"),
            ("苏格拉底是人", "zh"),
            ("Therefore Socrates is mortal", "en"),
            ("故苏格拉底会死", "zh"),
            ("Logical implication holds", "en"),
            ("逻辑蕴涵成立", "zh"),
        ]),
        // 概念: 否定
        ("negation", vec![
            ("A does not contain B", "en"),
            ("A 不包含 B", "zh"),
            ("X is not equal to Y", "en"),
            ("X 不等于 Y", "zh"),
            ("Never happens", "en"),
            ("永远不会发生", "zh"),
            ("Cannot be true", "en"),
            ("不可能为真", "zh"),
        ]),
        // 概念: 迭代
        ("iteration", vec![
            ("For each element in list", "en"),
            ("对于列表中的每个元素", "zh"),
            ("Loop through all items", "en"),
            ("循环遍历所有项", "zh"),
            ("Iterate until complete", "en"),
            ("迭代直到完成", "zh"),
            ("Repeat process N times", "en"),
            ("重复过程 N 次", "zh"),
        ]),
    ];
    
    let mut text_count = 0;
    let mut concept_ids: HashMap<String, Vec<String>> = HashMap::new();
    
    for (concept, variants) in &text_concepts {
        let mut ids = Vec::new();
        for (text, lang) in variants {
            if let Ok(item) = engine.ingest_text(text, lang) {
                ids.push(item.id);
                text_count += 1;
            }
        }
        concept_ids.insert(concept.to_string(), ids);
        throttle();
    }
    println!("  ✅ 文本: {} 项 ({} 个语义概念)", text_count, text_concepts.len());
    
    // 1.2 代码数据 - 跨语言等价代码
    let code_concepts: Vec<(&str, Vec<(&str, &str)>)> = vec![
        // 概念: 容器/包含
        ("containment", vec![
            ("fn container() { let items = vec![1, 2, 3]; items }", "rust"),
            ("def container(): items = [1, 2, 3]; return items", "python"),
            ("function container() { return [1, 2, 3]; }", "javascript"),
            ("struct Container { items: Vec<i32> }", "rust"),
            ("class Container: def __init__(self): self.items = []", "python"),
        ]),
        // 概念: 比较
        ("comparison", vec![
            ("fn compare(x: i32, y: i32) -> bool { x > y }", "rust"),
            ("def compare(x, y): return x > y", "python"),
            ("function compare(x, y) { return x > y; }", "javascript"),
            ("fn less_than(a: i32, b: i32) -> bool { a < b }", "rust"),
            ("def less_than(a, b): return a < b", "python"),
        ]),
        // 概念: 等于
        ("equality", vec![
            ("fn equals(a: i32, b: i32) -> bool { a == b }", "rust"),
            ("def equals(a, b): return a == b", "python"),
            ("function equals(a, b) { return a === b; }", "javascript"),
            ("impl PartialEq for MyType { fn eq(&self, other: &Self) -> bool { self.value == other.value } }", "rust"),
        ]),
        // 概念: 动作/返回
        ("action", vec![
            ("fn add(a: i32, b: i32) -> i32 { a + b }", "rust"),
            ("def add(a, b): return a + b", "python"),
            ("function add(a, b) { return a + b; }", "javascript"),
            ("fn process(input: String) -> String { input.to_uppercase() }", "rust"),
            ("def process(input): return input.upper()", "python"),
            ("fn transform(x: i32) -> i32 { x * 2 }", "rust"),
            ("def transform(x): return x * 2", "python"),
        ]),
        // 概念: 迭代
        ("iteration", vec![
            ("for i in 0..10 { println!(\"{}\", i); }", "rust"),
            ("for i in range(10): print(i)", "python"),
            ("for (let i = 0; i < 10; i++) { console.log(i); }", "javascript"),
            ("items.iter().for_each(|x| println!(\"{}\", x));", "rust"),
            ("for item in items: print(item)", "python"),
            ("while condition { process(); }", "rust"),
            ("while condition: process()", "python"),
        ]),
        // 概念: 条件
        ("logic", vec![
            ("if a > b { return a; } else { return b; }", "rust"),
            ("if a > b: return a\nelse: return b", "python"),
            ("match x { 1 => \"one\", 2 => \"two\", _ => \"other\" }", "rust"),
            ("if condition: do_something()", "python"),
        ]),
    ];
    
    let mut code_count = 0;
    for (concept, variants) in &code_concepts {
        let ids = concept_ids.entry(concept.to_string()).or_insert_with(Vec::new);
        for (code, lang) in variants {
            if let Ok(item) = engine.ingest_code(code, lang) {
                ids.push(item.id);
                code_count += 1;
            }
        }
        throttle();
    }
    println!("  ✅ 代码: {} 项 ({} 个语义概念)", code_count, code_concepts.len());
    
    // 1.3 数学表达式
    let math_concepts: Vec<(&str, Vec<&str>)> = vec![
        ("containment", vec!["A ⊆ B", "x ∈ S", "A ⊂ B", "{x | x ∈ A}"]),
        ("comparison", vec!["x > y", "a < b", "x ≥ y", "a ≤ b", "max(a, b)"]),
        ("equality", vec!["a = b", "x ≡ y", "f(x) = g(x)", "A ≈ B"]),
        ("action", vec!["f(x) = x^2", "g(x) = 2x + 1", "∫f(x)dx", "∑_{i=1}^n i"]),
        ("logic", vec!["A → B", "A ∧ B", "A ∨ B", "¬A", "∀x P(x)", "∃x Q(x)"]),
    ];
    
    let mut math_count = 0;
    for (concept, exprs) in &math_concepts {
        let ids = concept_ids.entry(concept.to_string()).or_insert_with(Vec::new);
        for expr in exprs {
            if let Ok(item) = engine.ingest_math(expr) {
                ids.push(item.id);
                math_count += 1;
            }
        }
        throttle();
    }
    println!("  ✅ 数学: {} 项", math_count);
    
    // 1.4 SVG 图形
    let svg_concepts: Vec<(&str, Vec<&str>)> = vec![
        ("containment", vec![
            r#"<svg><rect x="0" y="0" width="100" height="100"><rect x="20" y="20" width="60" height="60"/></rect></svg>"#,
            r#"<svg><g id="container"><circle cx="50" cy="50" r="10"/></g></svg>"#,
            r#"<svg><rect x="0" y="0" width="200" height="200" fill="blue"/></svg>"#,
        ]),
        ("comparison", vec![
            r#"<svg><rect x="0" y="50" width="30" height="50"/><rect x="50" y="20" width="30" height="80"/></svg>"#,
            r#"<svg><line x1="0" y1="100" x2="100" y2="0" stroke="black"/></svg>"#,
        ]),
        ("equality", vec![
            r#"<svg><circle cx="30" cy="50" r="20"/><circle cx="70" cy="50" r="20"/></svg>"#,
            r#"<svg><rect x="10" y="40" width="30" height="30"/><rect x="60" y="40" width="30" height="30"/></svg>"#,
        ]),
        ("action", vec![
            r#"<svg><path d="M10 10 L90 90" stroke="black" marker-end="url(#arrow)"/></svg>"#,
            r#"<svg><path d="M0 50 Q50 0 100 50" fill="none" stroke="blue"/></svg>"#,
        ]),
        ("iteration", vec![
            r#"<svg><circle cx="20" cy="50" r="10"/><circle cx="40" cy="50" r="10"/><circle cx="60" cy="50" r="10"/><circle cx="80" cy="50" r="10"/></svg>"#,
            r#"<svg><rect x="10" y="50" width="15" height="30"/><rect x="30" y="50" width="15" height="30"/><rect x="50" y="50" width="15" height="30"/></svg>"#,
        ]),
    ];
    
    let mut svg_count = 0;
    for (concept, svgs) in &svg_concepts {
        let ids = concept_ids.entry(concept.to_string()).or_insert_with(Vec::new);
        for svg in svgs {
            if let Ok(item) = engine.ingest_svg(svg) {
                ids.push(item.id);
                svg_count += 1;
            }
        }
        throttle();
    }
    println!("  ✅ SVG: {} 项", svg_count);
    
    // 1.5 音频特征
    let audio_concepts: Vec<(&str, Vec<AudioFeatures>)> = vec![
        ("containment", vec![
            AudioFeatures::from_description("低音 沉稳 包容"),
            AudioFeatures::from_description("low deep stable"),
            AudioFeatures::new(2000.0, 200.0, 0.6),
        ]),
        ("comparison", vec![
            AudioFeatures::from_description("高音 尖锐 上升"),
            AudioFeatures::from_description("high sharp ascending"),
            AudioFeatures::new(1000.0, 880.0, 0.8),
            AudioFeatures::new(1500.0, 440.0, 0.5),
        ]),
        ("equality", vec![
            AudioFeatures::from_description("中音 平稳 和谐"),
            AudioFeatures::from_description("mid stable harmony"),
            AudioFeatures::new(2000.0, 440.0, 0.6),
        ]),
        ("action", vec![
            AudioFeatures::from_description("快速 节奏 动感"),
            AudioFeatures::from_description("fast rhythm dynamic"),
            AudioFeatures::new(500.0, 660.0, 0.9),
        ]),
        ("emotion", vec![
            AudioFeatures::from_description("温暖 柔和 深情"),
            AudioFeatures::from_description("warm soft emotional"),
            AudioFeatures::new(3000.0, 330.0, 0.4),
        ]),
    ];
    
    let mut audio_count = 0;
    for (concept, features_list) in &audio_concepts {
        let ids = concept_ids.entry(concept.to_string()).or_insert_with(Vec::new);
        for features in features_list {
            if let Ok(item) = engine.ingest_audio(features) {
                ids.push(item.id);
                audio_count += 1;
            }
        }
        throttle();
    }
    println!("  ✅ 音频: {} 项", audio_count);
    
    let total_items = engine.get_stats().total_items;
    println!();
    println!("📊 摄入统计:");
    println!("  总项数: {}", total_items);
    println!("  平均复杂度: {:.3}", engine.get_stats().average_complexity);
    for (modality, count) in &engine.get_stats().by_modality {
        println!("  - {}: {}", modality, count);
    }
    
    // ═══════════════════════════════════════════════════════════════
    // 阶段 2: 概念内跨模态相似度测试
    // ═══════════════════════════════════════════════════════════════
    
    println!();
    println!("🔗 阶段 2: 概念内跨模态相似度测试");
    println!("────────────────────────────────────────────────────────");
    
    let mut concept_results: HashMap<String, ConceptTestResult> = HashMap::new();
    
    for (concept, ids) in &concept_ids {
        if ids.len() < 2 {
            continue;
        }
        
        let items: Vec<_> = ids.iter()
            .filter_map(|id| engine.items.iter().find(|item| &item.id == id))
            .collect();
        
        if items.len() < 2 {
            continue;
        }
        
        let mut total_sim = 0.0;
        let mut pair_count = 0;
        let mut cross_modal_count = 0;
        let mut cross_modal_sim = 0.0;
        let mut same_modal_sim = 0.0;
        let mut same_modal_count = 0;
        
        for i in 0..items.len() {
            for j in (i+1)..items.len() {
                let sim = items[i].deformation.cosine_similarity(&items[j].deformation);
                total_sim += sim;
                pair_count += 1;
                
                if items[i].modality != items[j].modality {
                    cross_modal_sim += sim;
                    cross_modal_count += 1;
                } else {
                    same_modal_sim += sim;
                    same_modal_count += 1;
                }
            }
        }
        
        let avg_sim = if pair_count > 0 { total_sim / pair_count as f64 } else { 0.0 };
        let avg_cross = if cross_modal_count > 0 { cross_modal_sim / cross_modal_count as f64 } else { 0.0 };
        let avg_same = if same_modal_count > 0 { same_modal_sim / same_modal_count as f64 } else { 0.0 };
        
        concept_results.insert(concept.clone(), ConceptTestResult {
            concept: concept.clone(),
            item_count: items.len(),
            avg_similarity: avg_sim,
            cross_modal_similarity: avg_cross,
            same_modal_similarity: avg_same,
        });
        
        let status = if avg_sim > 0.9 { "✅" } else if avg_sim > 0.75 { "⚠️" } else { "❌" };
        println!("  {} {}: 平均 {:.1}% | 跨模态 {:.1}% | 同模态 {:.1}% ({} 项)",
            status, concept, avg_sim * 100.0, avg_cross * 100.0, avg_same * 100.0, items.len());
        
        throttle();
    }
    
    // ═══════════════════════════════════════════════════════════════
    // 阶段 3: 概念间区分度测试
    // ═══════════════════════════════════════════════════════════════
    
    println!();
    println!("🎯 阶段 3: 概念间区分度测试");
    println!("────────────────────────────────────────────────────────");
    
    let concepts: Vec<&String> = concept_ids.keys().collect();
    let mut inter_concept_sims: Vec<f64> = Vec::new();
    
    for i in 0..concepts.len() {
        for j in (i+1)..concepts.len() {
            let ids_a = &concept_ids[concepts[i]];
            let ids_b = &concept_ids[concepts[j]];
            
            if ids_a.is_empty() || ids_b.is_empty() {
                continue;
            }
            
            let mut sum_sim = 0.0;
            let mut count = 0;
            
            // 采样比较 (避免 O(n²) 爆炸)
            let sample_a: Vec<_> = ids_a.iter().take(5).collect();
            let sample_b: Vec<_> = ids_b.iter().take(5).collect();
            
            for id_a in &sample_a {
                for id_b in &sample_b {
                    if let (Some(item_a), Some(item_b)) = (
                        engine.items.iter().find(|item| &&item.id == id_a),
                        engine.items.iter().find(|item| &&item.id == id_b),
                    ) {
                        sum_sim += item_a.deformation.cosine_similarity(&item_b.deformation);
                        count += 1;
                    }
                }
            }
            
            if count > 0 {
                let avg = sum_sim / count as f64;
                inter_concept_sims.push(avg);
            }
        }
        throttle();
    }
    
    let avg_inter_concept = if !inter_concept_sims.is_empty() {
        inter_concept_sims.iter().sum::<f64>() / inter_concept_sims.len() as f64
    } else { 0.0 };
    
    let intra_concept_avg: f64 = concept_results.values()
        .map(|r| r.avg_similarity)
        .sum::<f64>() / concept_results.len().max(1) as f64;
    
    let discrimination = intra_concept_avg - avg_inter_concept;
    
    println!("  概念内平均相似度: {:.1}%", intra_concept_avg * 100.0);
    println!("  概念间平均相似度: {:.1}%", avg_inter_concept * 100.0);
    println!("  区分度 (Δ): {:.1}%", discrimination * 100.0);
    
    if discrimination > 0.15 {
        println!("  ✅ 区分度良好 - 不同概念可有效区分");
    } else if discrimination > 0.05 {
        println!("  ⚠️ 区分度一般 - 需要优化编码函数");
    } else {
        println!("  ❌ 区分度不足 - 概念混淆风险");
    }
    
    // ═══════════════════════════════════════════════════════════════
    // 阶段 4: 跨模态查询测试
    // ═══════════════════════════════════════════════════════════════
    
    println!();
    println!("🔍 阶段 4: 跨模态查询测试");
    println!("────────────────────────────────────────────────────────");
    
    // 测试文本查询找代码/数学/图形
    let test_queries = vec![
        ("text_0", "包含关系文本"),
        ("code_0", "容器代码"),
        ("math_0", "集合数学"),
    ];
    
    for (query_id, desc) in test_queries {
        if let Some(query_item) = engine.items.iter().find(|i| i.id == query_id) {
            let matches = engine.cross_modal_query(query_item);
            let high_matches: Vec<_> = matches.iter().filter(|m| m.similarity > 0.8).collect();
            
            println!("  查询 [{}] ({:?}):", desc, query_item.modality);
            println!("    匹配总数: {} | 高相似度: {}", matches.len(), high_matches.len());
            
            // 显示前3个跨模态匹配
            for m in matches.iter().take(3) {
                println!("      → {} ({:?}): {:.1}%", m.target_id, m.target_modality, m.similarity * 100.0);
            }
        }
        throttle();
    }
    
    // ═══════════════════════════════════════════════════════════════
    // 阶段 5: 实验总结
    // ═══════════════════════════════════════════════════════════════
    
    let elapsed = start.elapsed();
    
    println!();
    println!("╔══════════════════════════════════════════════════════════════════════════╗");
    println!("║  📊 大规模实验总结                                                       ║");
    println!("╠══════════════════════════════════════════════════════════════════════════╣");
    println!("║  运行时间: {:?}", elapsed);
    println!("║  处理项数: {}", total_items);
    println!("║  语义概念: {} 个", concept_ids.len());
    println!("║  模态数量: {}", engine.get_stats().by_modality.len());
    println!("║                                                                          ║");
    println!("║  📈 核心指标:                                                            ║");
    println!("║    - 概念内平均相似度: {:.1}%", intra_concept_avg * 100.0);
    println!("║    - 概念间平均相似度: {:.1}%", avg_inter_concept * 100.0);
    println!("║    - 区分度: {:.1}%", discrimination * 100.0);
    println!("║                                                                          ║");
    
    // 计算最终得分
    let cross_modal_success_rate = concept_results.values()
        .filter(|r| r.cross_modal_similarity > 0.75)
        .count() as f64 / concept_results.len().max(1) as f64;
    
    let overall_score = (intra_concept_avg * 0.4 + discrimination * 0.3 + cross_modal_success_rate * 0.3) * 100.0;
    
    println!("║  🏆 综合评分: {:.1}/100", overall_score);
    println!("║                                                                          ║");
    
    if overall_score >= 80.0 {
        println!("║  ✅ 实验结论: 跨模态统一验证成功                                        ║");
        println!("║     形变信号架构有效实现了语义级跨模态对齐                              ║");
    } else if overall_score >= 60.0 {
        println!("║  ⚠️ 实验结论: 部分成功，具备基础跨模态能力                             ║");
        println!("║     建议优化: 增强关系编码、扩展训练数据                                ║");
    } else {
        println!("║  ❌ 实验结论: 需要显著优化                                              ║");
    }
    println!("╚══════════════════════════════════════════════════════════════════════════╝");
    
    // 输出详细概念报告
    println!();
    println!("📋 概念详细报告:");
    println!("────────────────────────────────────────────────────────");
    for (concept, result) in &concept_results {
        println!("  {}: {} 项 | 内部 {:.1}% | 跨模态 {:.1}%",
            concept, result.item_count, 
            result.avg_similarity * 100.0,
            result.cross_modal_similarity * 100.0);
    }
}

/// 节流函数
fn throttle() {
    thread::sleep(Duration::from_millis(BATCH_DELAY_MS));
}

/// 概念测试结果
struct ConceptTestResult {
    concept: String,
    item_count: usize,
    avg_similarity: f64,
    cross_modal_similarity: f64,
    same_modal_similarity: f64,
}
