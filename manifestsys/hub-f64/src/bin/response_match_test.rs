//! 🎯 问答语义匹配测试
//!
//! 核心问题：回答是否与问题语义相符？还是随机的？

use std::collections::HashMap;
use std::fs;

use hub_f64::{
    ValhallaHub, HubRequest, HubConfig, RequestType, Priority,
};

/// 测试用例：问题 + 期望回答应包含的关键词
struct QATestCase {
    question: &'static str,
    lang: &'static str,
    category: &'static str,
    // 如果回答语义相符，应该包含这些关键词之一
    expected_keywords: Vec<&'static str>,
}

fn get_test_cases() -> Vec<QATestCase> {
    vec![
        // 问候 - 回答应该是问候相关
        QATestCase {
            question: "Hello, how are you?",
            lang: "English",
            category: "greeting",
            expected_keywords: vec!["hello", "hi", "fine", "good", "well", "thank", "nice", "meet"],
        },
        QATestCase {
            question: "你好，你怎么样？",
            lang: "Chinese",
            category: "greeting",
            expected_keywords: vec!["你好", "好", "谢", "很好", "不错", "您好"],
        },
        QATestCase {
            question: "こんにちは",
            lang: "Japanese",
            category: "greeting",
            expected_keywords: vec!["こんにち", "ありがとう", "元気", "よろしく"],
        },
        
        // 感谢 - 回答应该是客气相关
        QATestCase {
            question: "Thank you very much!",
            lang: "English",
            category: "thanks",
            expected_keywords: vec!["welcome", "pleasure", "glad", "help", "anytime"],
        },
        QATestCase {
            question: "非常感谢！",
            lang: "Chinese",
            category: "thanks",
            expected_keywords: vec!["不客气", "谢", "没关系", "应该", "乐意"],
        },
        
        // 问题 - 回答应该是解释/说明
        QATestCase {
            question: "What is artificial intelligence?",
            lang: "English",
            category: "question",
            expected_keywords: vec!["intelligence", "computer", "machine", "learn", "system", "ai", "data"],
        },
        QATestCase {
            question: "什么是人工智能？",
            lang: "Chinese",
            category: "question",
            expected_keywords: vec!["智能", "计算", "机器", "学习", "系统", "数据", "技术"],
        },
        
        // 天气 - 回答应该是天气相关
        QATestCase {
            question: "How is the weather today?",
            lang: "English",
            category: "weather",
            expected_keywords: vec!["weather", "sunny", "rain", "cloud", "warm", "cold", "nice", "today"],
        },
        QATestCase {
            question: "今天天气怎么样？",
            lang: "Chinese",
            category: "weather",
            expected_keywords: vec!["天气", "晴", "雨", "云", "冷", "热", "好"],
        },
        
        // 帮助请求 - 回答应该是提供帮助
        QATestCase {
            question: "Can you help me?",
            lang: "English",
            category: "help",
            expected_keywords: vec!["help", "sure", "yes", "of course", "glad", "assist", "what"],
        },
        QATestCase {
            question: "你能帮我吗？",
            lang: "Chinese",
            category: "help",
            expected_keywords: vec!["帮", "可以", "好", "当然", "什么", "需要"],
        },
        
        // 时间询问 - 回答应该是时间相关
        QATestCase {
            question: "What time is it?",
            lang: "English",
            category: "time",
            expected_keywords: vec!["time", "o'clock", "hour", "minute", "now", "am", "pm"],
        },
        QATestCase {
            question: "现在几点了？",
            lang: "Chinese",
            category: "time",
            expected_keywords: vec!["点", "时", "分", "现在"],
        },
        
        // 名字询问
        QATestCase {
            question: "What is your name?",
            lang: "English",
            category: "name",
            expected_keywords: vec!["name", "i am", "call", "my", "valhalla"],
        },
        QATestCase {
            question: "你叫什么名字？",
            lang: "Chinese",
            category: "name",
            expected_keywords: vec!["名字", "叫", "我是", "瓦尔哈拉"],
        },
        
        // 学习意愿
        QATestCase {
            question: "I want to learn programming",
            lang: "English",
            category: "learn",
            expected_keywords: vec!["learn", "programming", "code", "good", "start", "practice", "language"],
        },
        QATestCase {
            question: "我想学习编程",
            lang: "Chinese",
            category: "learn",
            expected_keywords: vec!["学习", "编程", "代码", "语言", "开始", "练习"],
        },
    ]
}

/// 加载本地对话语料
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

/// 简单的响应生成（模拟当前系统行为）
fn generate_response(corpus: &[String], tick: u64, input_len: usize) -> String {
    if corpus.is_empty() {
        return "[empty corpus]".to_string();
    }
    // 当前系统的响应选择逻辑：基于 tick 和输入长度
    let idx = (tick as usize + input_len) % corpus.len();
    corpus[idx].clone()
}

/// 检查响应是否包含期望关键词
fn check_match(response: &str, keywords: &[&str]) -> bool {
    let response_lower = response.to_lowercase();
    keywords.iter().any(|kw| response_lower.contains(&kw.to_lowercase()))
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════════════╗");
    println!("║         🎯 问答语义匹配测试                                        ║");
    println!("║         回答是否与问题语义相符？还是随机选择？                     ║");
    println!("╚════════════════════════════════════════════════════════════════════╝\n");
    
    // 加载语料
    let corpus = load_corpus();
    println!("📂 语料库: {} 条\n", corpus.len());
    
    let test_cases = get_test_cases();
    println!("📝 测试用例: {} 条\n", test_cases.len());
    
    // Hub 初始化
    let config = HubConfig::default();
    let mut hub = ValhallaHub::with_config(config);
    
    println!("═══════════════════════════════════════════════════════════════════════\n");
    
    let mut total_match = 0;
    let mut total_tests = 0;
    let mut category_results: HashMap<String, (u32, u32)> = HashMap::new(); // (match, total)
    
    for (i, case) in test_cases.iter().enumerate() {
        total_tests += 1;
        
        // 发送请求到 Hub
        let request = HubRequest {
            request_id: format!("qa_{}", i),
            request_type: RequestType::Reasoning,
            intensity: (case.question.len() as f64 / 100.0).clamp(0.1, 1.0),
            data: Some(serde_json::json!({ "text": case.question })),
            priority: Priority::Normal,
        };
        
        let _response = hub.process(request);
        let status = hub.get_status();
        
        // 生成响应（模拟当前系统）
        let reply = generate_response(&corpus, status.fate_signal_tick, case.question.len());
        
        // 检查语义匹配
        let is_match = check_match(&reply, &case.expected_keywords);
        if is_match {
            total_match += 1;
        }
        
        // 分类统计
        let entry = category_results.entry(case.category.to_string()).or_insert((0, 0));
        if is_match { entry.0 += 1; }
        entry.1 += 1;
        
        // 显示
        let match_icon = if is_match { "✓" } else { "✗" };
        let question_display: String = case.question.chars().take(30).collect();
        let reply_display: String = reply.chars().take(40).collect();
        
        println!("┌─ 测试 #{:<2} [{}] ─────────────────────────────────────────────", i + 1, case.category);
        println!("│ 问: {} [{}]", question_display, case.lang);
        println!("│ 答: {}...", reply_display);
        println!("│ 期望关键词: {:?}", &case.expected_keywords[..case.expected_keywords.len().min(5)]);
        println!("│ 匹配: {}", match_icon);
        println!("└───────────────────────────────────────────────────────────────────────\n");
    }
    
    // 统计报告
    println!("═══════════════════════════════════════════════════════════════════════");
    println!("                         📊 语义匹配分析报告                           ");
    println!("═══════════════════════════════════════════════════════════════════════\n");
    
    // 1. 实际匹配率
    let actual_rate = total_match as f64 / total_tests as f64 * 100.0;
    println!("1. 实际语义匹配率: {}/{} ({:.1}%)\n", total_match, total_tests, actual_rate);
    
    // 2. 按类别统计
    println!("2. 按类别统计:");
    println!("   ─────────────┬────────┬────────┬────────");
    println!("   类别         │ 匹配数 │ 总数   │ 匹配率");
    println!("   ─────────────┼────────┼────────┼────────");
    
    let mut categories: Vec<_> = category_results.iter().collect();
    categories.sort_by_key(|(k, _)| k.as_str());
    
    for (cat, (matched, total)) in &categories {
        let rate = *matched as f64 / *total as f64 * 100.0;
        println!("   {:12} │ {:>6} │ {:>6} │ {:>5.1}%", cat, matched, total, rate);
    }
    println!();
    
    // 3. 随机基线估算
    // 如果是随机选择语料，关键词匹配的概率
    println!("3. 随机基线估算:");
    
    // 计算语料中包含各类关键词的比例
    let all_keywords: Vec<&str> = test_cases.iter()
        .flat_map(|c| c.expected_keywords.iter().copied())
        .collect();
    
    let mut keyword_hits = 0;
    for item in &corpus {
        let item_lower = item.to_lowercase();
        if all_keywords.iter().any(|kw| item_lower.contains(&kw.to_lowercase())) {
            keyword_hits += 1;
        }
    }
    let random_baseline = keyword_hits as f64 / corpus.len() as f64 * 100.0;
    
    println!("   语料中包含任一关键词的比例: {}/{} ({:.1}%)", 
        keyword_hits, corpus.len(), random_baseline);
    println!("   (这是随机选择时的期望匹配率)\n");
    
    // 4. 对比分析
    println!("4. 对比分析:");
    println!("   实际匹配率: {:.1}%", actual_rate);
    println!("   随机期望率: {:.1}%", random_baseline);
    let diff = actual_rate - random_baseline;
    println!("   差异: {:+.1}%", diff);
    
    let ratio = actual_rate / random_baseline.max(0.1);
    println!("   比值: {:.2}x\n", ratio);
    
    // 5. 结论
    println!("5. 结论:");
    if diff.abs() < 5.0 {
        println!("   ✗ 实际匹配率 ≈ 随机期望率");
        println!("   → 回答选择是随机的，没有语义匹配");
    } else if diff > 10.0 {
        println!("   ✓ 实际匹配率显著高于随机");
        println!("   → 系统有一定的语义匹配能力");
    } else if diff < -10.0 {
        println!("   ⚠ 实际匹配率低于随机");
        println!("   → 系统可能存在负向偏好");
    } else {
        println!("   △ 有轻微差异，但不显著");
    }
    
    println!("\n═══════════════════════════════════════════════════════════════════════");
    
    // 6. 补充：展示当前响应选择机制
    println!("\n6. 当前响应选择机制分析:");
    println!("   选择公式: idx = (fate_tick + input_length) %% corpus_size");
    println!("   这是一个确定性函数，基于:");
    println!("   - fate_tick: 递增计数器");
    println!("   - input_length: 输入字符数");
    println!("   → 选择与问题内容无关，只与输入长度相关");
    println!("   → 本质上是伪随机/顺序选择，不是语义匹配");
    
    println!("\n═══════════════════════════════════════════════════════════════════════");
}
