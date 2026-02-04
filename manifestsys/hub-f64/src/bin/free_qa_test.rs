//! 🗣️ Hub 自由问答测试
//!
//! 不预设期望，让 Hub 自由输出，分析问答关联度

use std::fs;
use std::collections::HashMap;

use hub_f64::{
    ValhallaHub, HubRequest, HubConfig, RequestType, Priority,
};

/// 问答对
struct QAPair {
    question: String,
    answer: String,
    routed_to: String,
    tick: u64,
}

/// 关联度评估
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Correlation {
    Strong,   // 强关联：语言相同 + 语义相关
    Medium,   // 中关联：语言相同 或 有部分语义重叠
    Weak,     // 弱关联：语言不同 且 无语义关联
}

impl Correlation {
    fn icon(&self) -> &'static str {
        match self {
            Correlation::Strong => "🟢",
            Correlation::Medium => "🟡", 
            Correlation::Weak => "🔴",
        }
    }
}

/// 检测语言
fn detect_language(text: &str) -> &'static str {
    let has_chinese = text.chars().any(|c| c >= '\u{4e00}' && c <= '\u{9fff}');
    let has_japanese = text.chars().any(|c| 
        (c >= '\u{3040}' && c <= '\u{309f}') || (c >= '\u{30a0}' && c <= '\u{30ff}')
    );
    let has_korean = text.chars().any(|c| c >= '\u{ac00}' && c <= '\u{d7a3}');
    let has_arabic = text.chars().any(|c| c >= '\u{0600}' && c <= '\u{06ff}');
    let has_cyrillic = text.chars().any(|c| c >= '\u{0400}' && c <= '\u{04ff}');
    
    if has_japanese { return "Japanese"; }
    if has_korean { return "Korean"; }
    if has_chinese { return "Chinese"; }
    if has_arabic { return "Arabic"; }
    if has_cyrillic { return "Russian"; }
    "English"
}

/// 提取关键词（简单分词）
fn extract_keywords(text: &str) -> Vec<String> {
    let text_lower = text.to_lowercase();
    
    // 中文按字切分，其他按空格
    let has_cjk = text.chars().any(|c| c >= '\u{4e00}' && c <= '\u{9fff}');
    
    if has_cjk {
        // 中文：每2-3字为一个词
        let chars: Vec<char> = text_lower.chars()
            .filter(|c| c.is_alphanumeric() || *c >= '\u{4e00}')
            .collect();
        let mut words = Vec::new();
        for i in 0..chars.len() {
            if i + 2 <= chars.len() {
                words.push(chars[i..i+2].iter().collect());
            }
        }
        words
    } else {
        // 英文等：按空格分词
        text_lower
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() > 2)
            .map(|w| w.to_string())
            .collect()
    }
}

/// 评估问答关联度
fn evaluate_correlation(question: &str, answer: &str) -> (Correlation, String) {
    let q_lang = detect_language(question);
    let a_lang = detect_language(answer);
    
    let q_keywords = extract_keywords(question);
    let a_keywords = extract_keywords(answer);
    
    // 计算关键词重叠
    let overlap: usize = q_keywords.iter()
        .filter(|qk| a_keywords.iter().any(|ak| ak.contains(qk.as_str()) || qk.contains(ak.as_str())))
        .count();
    
    let overlap_ratio = if q_keywords.is_empty() { 
        0.0 
    } else { 
        overlap as f64 / q_keywords.len() as f64 
    };
    
    // 语言匹配
    let lang_match = q_lang == a_lang;
    
    // 综合评估
    let (correlation, reason) = if lang_match && overlap_ratio > 0.3 {
        (Correlation::Strong, format!("语言匹配({}) + 关键词重叠{:.0}%", q_lang, overlap_ratio * 100.0))
    } else if lang_match || overlap_ratio > 0.1 {
        let reason = if lang_match {
            format!("语言匹配({})", q_lang)
        } else {
            format!("关键词重叠{:.0}%", overlap_ratio * 100.0)
        };
        (Correlation::Medium, reason)
    } else {
        (Correlation::Weak, format!("语言不匹配({} vs {}) + 无关键词重叠", q_lang, a_lang))
    };
    
    (correlation, reason)
}

/// 加载语料库
fn load_corpus() -> Vec<String> {
    let mut corpus = Vec::new();
    let paths = [
        "/home/jouly/AI/Valhalla/valhalla_all_dialogues.txt",
        "/home/jouly/AI/Valhalla/valhalla_enhanced_dialogues.txt",
    ];
    
    for path in paths {
        if let Ok(content) = fs::read_to_string(path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() && trimmed.len() > 5 {
                    corpus.push(trimmed.to_string());
                }
            }
        }
    }
    corpus
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════════════╗");
    println!("║         🗣️ Hub 自由问答测试                                        ║");
    println!("║         不预设期望，分析问答关联度                                 ║");
    println!("╚════════════════════════════════════════════════════════════════════╝\n");
    
    // 加载语料
    let corpus = load_corpus();
    println!("📂 语料库: {} 条\n", corpus.len());
    
    // 初始化 Hub
    let config = HubConfig::default();
    let mut hub = ValhallaHub::with_config(config);
    
    // 测试问题（多语言、多类型）
    let questions = vec![
        // 中文
        "你好",
        "今天天气怎么样？",
        "什么是人工智能？",
        "你叫什么名字？",
        "帮我写一段代码",
        "谢谢你的帮助",
        "我想学习编程",
        "晚安",
        
        // 英文
        "Hello",
        "How are you today?",
        "What is machine learning?",
        "Tell me a story",
        "I need help",
        "Thank you",
        "Good morning",
        "Goodbye",
        
        // 日文
        "こんにちは",
        "お元気ですか？",
        "ありがとう",
        
        // 韩文
        "안녕하세요",
        "감사합니다",
        
        // 混合
        "Hello 你好",
        "AI 人工智能是什么？",
    ];
    
    println!("📝 测试问题: {} 条\n", questions.len());
    println!("═══════════════════════════════════════════════════════════════════════\n");
    
    let mut results: Vec<(String, String, Correlation, String)> = Vec::new();
    let mut correlation_counts: HashMap<Correlation, u32> = HashMap::new();
    
    for (i, question) in questions.iter().enumerate() {
        // 发送到 Hub
        let request = HubRequest {
            request_id: format!("free_{}", i),
            request_type: RequestType::Reasoning,
            intensity: (question.len() as f64 / 100.0).clamp(0.1, 1.0),
            data: Some(serde_json::json!({ "text": question })),
            priority: Priority::Normal,
        };
        
        let response = hub.process(request);
        let status = hub.get_status();
        
        // 从语料中选择回答（模拟当前系统行为）
        let idx = (status.fate_signal_tick as usize + question.len()) % corpus.len();
        let answer = &corpus[idx];
        
        // 评估关联度
        let (correlation, reason) = evaluate_correlation(question, answer);
        *correlation_counts.entry(correlation).or_insert(0) += 1;
        
        // 显示
        let q_display: String = question.chars().take(25).collect();
        let a_display: String = answer.chars().take(40).collect();
        
        let routed_to = if response.message.contains("BlackHole") {
            "BH"
        } else if response.message.contains("MeshBrain") {
            "MB"
        } else if response.message.contains("MultiNova") {
            "MN"
        } else {
            "BF"
        };
        
        println!("┌─ #{:<2} ────────────────────────────────────────────────────────────", i + 1);
        println!("│ 问: {} [{}]", q_display, detect_language(question));
        println!("│ 答: {}...", a_display);
        println!("│ 路由: {} | Tick: {}", routed_to, status.fate_signal_tick);
        println!("│ 关联: {} {:?} - {}", correlation.icon(), correlation, reason);
        println!("└───────────────────────────────────────────────────────────────────────\n");
        
        results.push((question.to_string(), answer.clone(), correlation, reason));
    }
    
    // 统计报告
    println!("═══════════════════════════════════════════════════════════════════════");
    println!("                         📊 关联度统计报告                             ");
    println!("═══════════════════════════════════════════════════════════════════════\n");
    
    let total = questions.len() as f64;
    let strong = *correlation_counts.get(&Correlation::Strong).unwrap_or(&0) as f64;
    let medium = *correlation_counts.get(&Correlation::Medium).unwrap_or(&0) as f64;
    let weak = *correlation_counts.get(&Correlation::Weak).unwrap_or(&0) as f64;
    
    println!("1. 关联度分布:");
    println!();
    
    // 强关联
    let strong_pct = strong / total * 100.0;
    let strong_bar: String = "█".repeat((strong_pct / 2.5) as usize);
    println!("   🟢 强关联 │ {:>2} ({:>5.1}%) │ {}", strong as u32, strong_pct, strong_bar);
    
    // 中关联
    let medium_pct = medium / total * 100.0;
    let medium_bar: String = "█".repeat((medium_pct / 2.5) as usize);
    println!("   🟡 中关联 │ {:>2} ({:>5.1}%) │ {}", medium as u32, medium_pct, medium_bar);
    
    // 弱关联
    let weak_pct = weak / total * 100.0;
    let weak_bar: String = "█".repeat((weak_pct / 2.5) as usize);
    println!("   🔴 弱关联 │ {:>2} ({:>5.1}%) │ {}", weak as u32, weak_pct, weak_bar);
    
    println!();
    
    // 2. 按语言统计
    println!("2. 按问题语言的关联度:");
    println!();
    
    let mut lang_stats: HashMap<&str, (u32, u32, u32)> = HashMap::new(); // (strong, medium, weak)
    for (q, _, corr, _) in &results {
        let lang = detect_language(q);
        let entry = lang_stats.entry(lang).or_insert((0, 0, 0));
        match corr {
            Correlation::Strong => entry.0 += 1,
            Correlation::Medium => entry.1 += 1,
            Correlation::Weak => entry.2 += 1,
        }
    }
    
    println!("   语言       │ 强  │ 中  │ 弱  │ 中+强比例");
    println!("   ───────────┼─────┼─────┼─────┼──────────");
    for (lang, (s, m, w)) in &lang_stats {
        let total_lang = s + m + w;
        let good_ratio = (s + m) as f64 / total_lang as f64 * 100.0;
        println!("   {:10} │ {:>3} │ {:>3} │ {:>3} │ {:>5.1}%", lang, s, m, w, good_ratio);
    }
    println!();
    
    // 3. 语料库语言分布（影响回答）
    println!("3. 语料库语言分布:");
    let mut corpus_lang: HashMap<&str, u32> = HashMap::new();
    for item in &corpus {
        let lang = detect_language(item);
        *corpus_lang.entry(lang).or_insert(0) += 1;
    }
    
    let mut langs: Vec<_> = corpus_lang.iter().collect();
    langs.sort_by(|a, b| b.1.cmp(a.1));
    
    for (lang, count) in &langs {
        let c = **count;
        let pct = c as f64 / corpus.len() as f64 * 100.0;
        println!("   {:10} │ {:>4} ({:>5.1}%)", lang, c, pct);
    }
    println!();
    
    // 4. 分析
    println!("4. 分析:");
    println!();
    
    let good_ratio = (strong + medium) / total * 100.0;
    println!("   中+强关联率: {:.1}%", good_ratio);
    println!();
    
    if good_ratio > 50.0 {
        println!("   ✓ 超过半数回答有一定关联性");
    } else if good_ratio > 30.0 {
        println!("   △ 约三分之一回答有关联性");
    } else {
        println!("   ✗ 大部分回答无关联");
    }
    
    // 语言匹配分析
    let lang_match_count: u32 = results.iter()
        .filter(|(q, a, _, _)| detect_language(q) == detect_language(a))
        .count() as u32;
    let lang_match_ratio = lang_match_count as f64 / total * 100.0;
    
    println!();
    println!("   语言匹配率: {:.1}% ({}/{})", lang_match_ratio, lang_match_count, total as u32);
    
    // 期望的语言匹配率（基于语料分布）
    println!();
    println!("5. 与随机对比:");
    
    // 计算问题语言分布
    let mut q_lang_dist: HashMap<&str, f64> = HashMap::new();
    for q in &questions {
        let lang = detect_language(q);
        *q_lang_dist.entry(lang).or_insert(0.0) += 1.0;
    }
    for v in q_lang_dist.values_mut() {
        *v /= total;
    }
    
    // 计算期望语言匹配率
    let mut expected_lang_match = 0.0;
    for (lang, q_ratio) in &q_lang_dist {
        let a_count = corpus_lang.get(lang).copied().unwrap_or(0);
        let a_ratio = a_count as f64 / corpus.len() as f64;
        expected_lang_match += q_ratio * a_ratio;
    }
    expected_lang_match *= 100.0;
    
    println!("   实际语言匹配率: {:.1}%", lang_match_ratio);
    println!("   随机期望匹配率: {:.1}%", expected_lang_match);
    println!("   差异: {:+.1}%", lang_match_ratio - expected_lang_match);
    
    println!("\n═══════════════════════════════════════════════════════════════════════");
    
    // 列出所有强关联的例子
    let strong_examples: Vec<_> = results.iter()
        .filter(|(_, _, corr, _)| *corr == Correlation::Strong)
        .collect();
    
    if !strong_examples.is_empty() {
        println!("\n🟢 强关联示例:");
        for (q, a, _, reason) in strong_examples.iter().take(5) {
            let q_short: String = q.chars().take(20).collect();
            let a_short: String = a.chars().take(30).collect();
            println!("   问: {} → 答: {}...", q_short, a_short);
            println!("   原因: {}", reason);
            println!();
        }
    }
    
    // 列出弱关联的例子
    let weak_examples: Vec<_> = results.iter()
        .filter(|(_, _, corr, _)| *corr == Correlation::Weak)
        .collect();
    
    if !weak_examples.is_empty() {
        println!("🔴 弱关联示例:");
        for (q, a, _, reason) in weak_examples.iter().take(3) {
            let q_short: String = q.chars().take(20).collect();
            let a_short: String = a.chars().take(30).collect();
            println!("   问: {} → 答: {}...", q_short, a_short);
            println!("   原因: {}", reason);
            println!();
        }
    }
    
    println!("═══════════════════════════════════════════════════════════════════════");
}
