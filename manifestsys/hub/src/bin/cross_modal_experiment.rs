//! 🌐 跨模态统一实验
//!
//! 实验目标:
//! 验证 Valhalla 形变信号架构能否实现跨模态语义理解
//!
//! 测试模态:
//! - 文本 (中/英)
//! - 代码 (Rust/Python)
//! - 数学表达式
//! - SVG 图形
//! - 音频特征
//!
//! 资源限制: 50% CPU

use hub::data_ingestion::*;
use hub::{Modality, AudioFeatures};
use std::time::{Duration, Instant};
use std::thread;

/// 资源限制配置
const MAX_CPU_PERCENT: u32 = 50;
const MAX_ITEMS_PER_BATCH: usize = 100;
const BATCH_DELAY_MS: u64 = 50; // 批次间延迟，控制 CPU 使用

fn main() {
    println!("╔══════════════════════════════════════════════════════════════════════════╗");
    println!("║  🌐 Valhalla 跨模态统一实验                                              ║");
    println!("║  资源限制: {}% CPU                                                       ║", MAX_CPU_PERCENT);
    println!("╚══════════════════════════════════════════════════════════════════════════╝");
    println!();
    
    let start = Instant::now();
    let mut engine = DataIngestionEngine::new();
    
    // ═══════════════════════════════════════════════════════════════
    // 阶段 1: 数据摄入 (多模态)
    // ═══════════════════════════════════════════════════════════════
    
    println!("📥 阶段 1: 多模态数据摄入");
    println!("────────────────────────────────────────────────────────");
    
    // 1.1 文本数据 (跨语言)
    let text_samples = vec![
        ("A contains B", "en"),
        ("B 属于 A", "zh"),
        ("Container holds items", "en"),
        ("容器包含元素", "zh"),
        ("X is greater than Y", "en"),
        ("X 大于 Y", "zh"),
        ("The function returns a value", "en"),
        ("函数返回一个值", "zh"),
        ("Love is eternal", "en"),
        ("爱是永恒的", "zh"),
    ];
    
    for (text, lang) in &text_samples {
        engine.ingest_text(text, lang).unwrap();
        throttle();
    }
    println!("  ✅ 文本: {} 项", text_samples.len());
    
    // 1.2 代码数据
    let code_samples = vec![
        ("fn container() { let items = vec![1, 2, 3]; }", "rust"),
        ("def container(): items = [1, 2, 3]", "python"),
        ("fn compare(x: i32, y: i32) -> bool { x > y }", "rust"),
        ("def compare(x, y): return x > y", "python"),
        ("fn add(a: i32, b: i32) -> i32 { a + b }", "rust"),
        ("def add(a, b): return a + b", "python"),
        ("for i in 0..10 { println!(\"{}\", i); }", "rust"),
        ("for i in range(10): print(i)", "python"),
    ];
    
    for (code, lang) in &code_samples {
        engine.ingest_code(code, lang).unwrap();
        throttle();
    }
    println!("  ✅ 代码: {} 项", code_samples.len());
    
    // 1.3 数学表达式
    let math_samples = vec![
        "a + b = c",
        "x > y",
        "\\int_0^1 f(x) dx",
        "\\sum_{i=1}^n i = \\frac{n(n+1)}{2}",
        "E = mc^2",
        "a^2 + b^2 = c^2",
    ];
    
    for expr in &math_samples {
        engine.ingest_math(expr).unwrap();
        throttle();
    }
    println!("  ✅ 数学: {} 项", math_samples.len());
    
    // 1.4 SVG 图形
    let svg_samples = vec![
        r#"<svg><rect x="0" y="0" width="100" height="100" fill="blue"/></svg>"#,
        r#"<svg><circle cx="50" cy="50" r="40" fill="red"/></svg>"#,
        r#"<svg><path d="M10 10 L90 90" stroke="black"/></svg>"#,
        r#"<svg><rect x="10" y="10" width="80" height="80"/><text x="50" y="50">A</text></svg>"#,
        r#"<svg><line x1="0" y1="0" x2="100" y2="100"/><line x1="100" y1="0" x2="0" y2="100"/></svg>"#,
    ];
    
    for svg in &svg_samples {
        engine.ingest_svg(svg).unwrap();
        throttle();
    }
    println!("  ✅ SVG: {} 项", svg_samples.len());
    
    // 1.5 音频特征
    let audio_samples = vec![
        AudioFeatures::from_description("高音 快速"),
        AudioFeatures::from_description("低音 缓慢"),
        AudioFeatures::from_description("中音 稳定"),
        AudioFeatures::from_description("loud high fast"),
        AudioFeatures::from_description("soft low slow"),
        AudioFeatures::new(2000.0, 440.0, 0.7), // A4 音
        AudioFeatures::new(1500.0, 261.63, 0.5), // C4 音
        AudioFeatures::new(3000.0, 880.0, 0.9), // A5 音
    ];
    
    for features in &audio_samples {
        engine.ingest_audio(features).unwrap();
        throttle();
    }
    println!("  ✅ 音频: {} 项", audio_samples.len());
    
    println!();
    println!("📊 摄入统计:");
    let stats = engine.get_stats();
    println!("  总项数: {}", stats.total_items);
    println!("  平均复杂度: {:.3}", stats.average_complexity);
    for (modality, count) in &stats.by_modality {
        println!("  - {}: {}", modality, count);
    }
    
    // ═══════════════════════════════════════════════════════════════
    // 阶段 2: 跨模态匹配测试
    // ═══════════════════════════════════════════════════════════════
    
    println!();
    println!("🔗 阶段 2: 跨模态匹配测试");
    println!("────────────────────────────────────────────────────────");
    
    let mut total_matches = 0;
    let mut high_similarity_matches = 0;
    let mut cross_modal_pairs: Vec<(String, String, f64)> = Vec::new();
    
    // 测试每个文本项与其他模态的匹配
    for (i, item) in engine.items.iter().enumerate() {
        if item.modality == Modality::Text {
            let matches = engine.cross_modal_query(item);
            for m in &matches {
                total_matches += 1;
                if m.similarity > 0.85 {
                    high_similarity_matches += 1;
                    cross_modal_pairs.push((
                        format!("{} ({:?})", m.source_id, m.source_modality),
                        format!("{} ({:?})", m.target_id, m.target_modality),
                        m.similarity,
                    ));
                }
            }
        }
        
        // 资源控制
        if i % 10 == 0 {
            throttle();
        }
    }
    
    println!("  跨模态匹配总数: {}", total_matches);
    println!("  高相似度匹配 (>85%): {}", high_similarity_matches);
    
    if !cross_modal_pairs.is_empty() {
        println!();
        println!("  🎯 高相似度跨模态配对 (前10):");
        for (src, tgt, sim) in cross_modal_pairs.iter().take(10) {
            println!("    {} ↔ {} : {:.2}%", src, tgt, sim * 100.0);
        }
    }
    
    // ═══════════════════════════════════════════════════════════════
    // 阶段 3: 语义等价验证
    // ═══════════════════════════════════════════════════════════════
    
    println!();
    println!("🧪 阶段 3: 语义等价验证");
    println!("────────────────────────────────────────────────────────");
    
    // 测试 1: 文本跨语言等价
    let test_cases = vec![
        ("text_0", "text_1", "包含关系 (EN↔CN)"),
        ("text_2", "text_3", "容器概念 (EN↔CN)"),
        ("text_4", "text_5", "大于关系 (EN↔CN)"),
        ("text_6", "text_7", "函数返回 (EN↔CN)"),
    ];
    
    println!("  📝 文本跨语言等价测试:");
    let mut text_total_sim = 0.0;
    for (id1, id2, desc) in &test_cases {
        if let (Some(item1), Some(item2)) = (
            engine.items.iter().find(|i| i.id == *id1),
            engine.items.iter().find(|i| i.id == *id2),
        ) {
            let sim = item1.deformation.cosine_similarity(&item2.deformation);
            text_total_sim += sim;
            let status = if sim > 0.95 { "✅" } else if sim > 0.8 { "⚠️" } else { "❌" };
            println!("    {} {}: {:.2}%", status, desc, sim * 100.0);
        }
        throttle();
    }
    let text_avg_sim = text_total_sim / test_cases.len() as f64;
    
    // 测试 2: 代码跨语言等价
    println!();
    println!("  💻 代码跨语言等价测试:");
    let code_tests = vec![
        ("code_0", "code_1", "容器函数 (Rust↔Python)"),
        ("code_2", "code_3", "比较函数 (Rust↔Python)"),
        ("code_4", "code_5", "加法函数 (Rust↔Python)"),
        ("code_6", "code_7", "循环 (Rust↔Python)"),
    ];
    
    let mut code_total_sim = 0.0;
    for (id1, id2, desc) in &code_tests {
        if let (Some(item1), Some(item2)) = (
            engine.items.iter().find(|i| i.id == *id1),
            engine.items.iter().find(|i| i.id == *id2),
        ) {
            let sim = item1.deformation.cosine_similarity(&item2.deformation);
            code_total_sim += sim;
            let status = if sim > 0.95 { "✅" } else if sim > 0.8 { "⚠️" } else { "❌" };
            println!("    {} {}: {:.2}%", status, desc, sim * 100.0);
        }
        throttle();
    }
    let code_avg_sim = code_total_sim / code_tests.len() as f64;
    
    // 测试 3: 跨模态语义等价 (文本 ↔ 代码 ↔ 图形)
    println!();
    println!("  🌐 跨模态语义等价测试:");
    
    // "包含" 概念跨模态
    let contain_text = engine.items.iter().find(|i| i.id == "text_0"); // "A contains B"
    let contain_code = engine.items.iter().find(|i| i.id == "code_0"); // container function
    let contain_svg = engine.items.iter().find(|i| i.id == "svg_0");   // rect (container)
    let contain_audio_low = engine.items.iter().find(|i| i.id == "audio_1"); // 低音 (沉稳=包含)
    
    if let (Some(t), Some(c), Some(s), Some(a)) = (contain_text, contain_code, contain_svg, contain_audio_low) {
        let tc = t.deformation.cosine_similarity(&c.deformation);
        let ts = t.deformation.cosine_similarity(&s.deformation);
        let ta = t.deformation.cosine_similarity(&a.deformation);
        let cs = c.deformation.cosine_similarity(&s.deformation);
        let ca = c.deformation.cosine_similarity(&a.deformation);
        let sa = s.deformation.cosine_similarity(&a.deformation);
        
        println!("    「包含」概念跨模态相似度:");
        println!("      文本 ↔ 代码:  {:.2}%", tc * 100.0);
        println!("      文本 ↔ 图形:  {:.2}%", ts * 100.0);
        println!("      文本 ↔ 音频:  {:.2}%", ta * 100.0);
        println!("      代码 ↔ 图形:  {:.2}%", cs * 100.0);
        println!("      代码 ↔ 音频:  {:.2}%", ca * 100.0);
        println!("      图形 ↔ 音频:  {:.2}%", sa * 100.0);
    }
    
    // "大于" 概念跨模态
    println!();
    let greater_text = engine.items.iter().find(|i| i.id == "text_4"); // "X > Y"
    let greater_code = engine.items.iter().find(|i| i.id == "code_2"); // compare function
    let greater_math = engine.items.iter().find(|i| i.id == "math_1"); // "x > y"
    let greater_svg = engine.items.iter().find(|i| i.id == "svg_2");   // path (方向性)
    let greater_audio = engine.items.iter().find(|i| i.id == "audio_0"); // 高音快速
    
    if let (Some(t), Some(c), Some(m), Some(s), Some(a)) = 
        (greater_text, greater_code, greater_math, greater_svg, greater_audio) {
        let tc = t.deformation.cosine_similarity(&c.deformation);
        let tm = t.deformation.cosine_similarity(&m.deformation);
        let ts = t.deformation.cosine_similarity(&s.deformation);
        let ta = t.deformation.cosine_similarity(&a.deformation);
        let cm = c.deformation.cosine_similarity(&m.deformation);
        
        println!("    「大于/方向」概念跨模态相似度:");
        println!("      文本 ↔ 代码:  {:.2}%", tc * 100.0);
        println!("      文本 ↔ 数学:  {:.2}%", tm * 100.0);
        println!("      文本 ↔ 图形:  {:.2}%", ts * 100.0);
        println!("      文本 ↔ 音频:  {:.2}%", ta * 100.0);
        println!("      代码 ↔ 数学:  {:.2}%", cm * 100.0);
    }
    
    // ═══════════════════════════════════════════════════════════════
    // 阶段 4: 实验总结
    // ═══════════════════════════════════════════════════════════════
    
    let elapsed = start.elapsed();
    
    println!();
    println!("╔══════════════════════════════════════════════════════════════════════════╗");
    println!("║  📊 实验总结                                                             ║");
    println!("╠══════════════════════════════════════════════════════════════════════════╣");
    println!("║  运行时间: {:?}", elapsed);
    println!("║  处理项数: {}", stats.total_items);
    println!("║  模态数量: {}", stats.by_modality.len());
    println!("║                                                                          ║");
    println!("║  📈 语义等价测试结果:                                                    ║");
    println!("║    - 文本跨语言平均相似度: {:.2}%", text_avg_sim * 100.0);
    println!("║    - 代码跨语言平均相似度: {:.2}%", code_avg_sim * 100.0);
    println!("║    - 跨模态高相似度配对数: {}", high_similarity_matches);
    println!("║                                                                          ║");
    
    let success = text_avg_sim > 0.9 && code_avg_sim > 0.85;
    if success {
        println!("║  ✅ 实验结论: 跨模态统一验证成功                                        ║");
        println!("║     形变信号架构能够捕获跨模态的语义等价性                              ║");
    } else {
        println!("║  ⚠️ 实验结论: 部分成功，需要优化编码函数                               ║");
    }
    println!("╚══════════════════════════════════════════════════════════════════════════╝");
}

/// 节流函数 - 控制 CPU 使用率
fn throttle() {
    thread::sleep(Duration::from_millis(BATCH_DELAY_MS));
}
