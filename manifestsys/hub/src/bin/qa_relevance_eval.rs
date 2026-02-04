//! 问答关联度评估 (使用 Hub 形变编码)

use hub::data_ingestion::DataIngestionEngine;

fn main() {
    println!("╔═══════════════════════════════════════════════════════════════════╗");
    println!("║     问答关联度评估 (Hub 形变编码 + cosine_similarity)             ║");
    println!("╚═══════════════════════════════════════════════════════════════════╝\n");

    // 模拟对话数据
    let qa_pairs = vec![
        ("你好", "💭 ǃ;"),
        ("Hello", "💭 v!"),
        ("今天天气怎么样?", "我喜欢旅行。"),
        ("こんにちは", "我收到了请帖。"),
        ("学习", "我叫Jack。"),
        ("代码", "我会说法语。"),
        ("什么是Valhalla", "我已经看过这部电影了。"),
        ("帮我写代码", "你应该休息一下。"),
        ("你叫什么名字", "Tom很高。"),
        ("今天星期几", "我不知道。"),
        // 添加更多测试
        ("爱是什么", "爱是永恒的。"),
        ("容器是什么", "容器包含元素。"),
        ("1+1等于几", "2"),
        ("X大于Y吗", "X > Y"),
    ];

    println!("📥 编码问答对...\n");

    let mut strong = 0;
    let mut medium = 0;
    let mut weak = 0;
    let mut none = 0;
    let mut total_sim = 0.0;

    println!("┌─────────────────────────────────────────────────────────────────────┐");
    println!("│ 问题              │ 回答              │ 相似度  │ 等级 │");
    println!("├─────────────────────────────────────────────────────────────────────┤");

    for (q, a) in &qa_pairs {
        // 为每个问答对创建独立的引擎来编码
        let mut q_engine = DataIngestionEngine::new();
        let mut a_engine = DataIngestionEngine::new();
        
        // 摄入并获取形变向量
        let q_item = q_engine.ingest_text(q, "auto").unwrap();
        let a_item = a_engine.ingest_text(a, "auto").unwrap();
        
        // 计算余弦相似度
        let sim = q_item.deformation.cosine_similarity(&a_item.deformation);
        total_sim += sim;

        let (level, icon) = if sim >= 0.85 {
            strong += 1;
            ("强", "🟢")
        } else if sim >= 0.60 {
            medium += 1;
            ("中", "🟡")
        } else if sim >= 0.30 {
            weak += 1;
            ("弱", "🟠")
        } else {
            none += 1;
            ("无", "🔴")
        };

        let q_display: String = q.chars().take(16).collect();
        let a_display: String = a.chars().take(16).collect();
        println!("│ {:16} │ {:16} │ {:6.1}% │  {}{}  │",
            q_display, a_display, sim * 100.0, icon, level);
    }

    println!("└─────────────────────────────────────────────────────────────────────┘\n");

    let total = qa_pairs.len();
    let avg = total_sim / total as f64;

    println!("═══════════════════════════════════════════════════════════════════════");
    println!("📊 Hub 形变编码关联度统计:");
    println!("───────────────────────────────────────────────────────────────────────");
    println!("   🟢 强关联 (≥85%): {:3} 条 ({:.1}%)", strong, strong as f64 / total as f64 * 100.0);
    println!("   🟡 中关联 (60-85%): {:3} 条 ({:.1}%)", medium, medium as f64 / total as f64 * 100.0);
    println!("   🟠 弱关联 (30-60%): {:3} 条 ({:.1}%)", weak, weak as f64 / total as f64 * 100.0);
    println!("   🔴 无关联 (<30%): {:3} 条 ({:.1}%)", none, none as f64 / total as f64 * 100.0);
    println!("───────────────────────────────────────────────────────────────────────");
    println!("   📈 平均相似度: {:.1}%", avg * 100.0);
    println!("   📝 总问答数:   {} 条", total);
    println!("═══════════════════════════════════════════════════════════════════════");
}
