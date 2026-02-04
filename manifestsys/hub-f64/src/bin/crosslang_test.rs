//! 🌍 跨语言统一能力测试
//!
//! 测试 Hub 在不同语言输入下的路由决策和响应能力

use std::collections::HashMap;
use std::fs;
use std::time::Instant;

use hub_f64::{
    ValhallaHub, HubRequest, HubConfig, RequestType, Priority,
};

/// 语言检测
fn detect_language(text: &str) -> &'static str {
    let has_chinese = text.chars().any(|c| c >= '\u{4e00}' && c <= '\u{9fff}');
    let has_japanese_kana = text.chars().any(|c| 
        (c >= '\u{3040}' && c <= '\u{309f}') ||  // Hiragana
        (c >= '\u{30a0}' && c <= '\u{30ff}')     // Katakana
    );
    let has_korean = text.chars().any(|c| c >= '\u{ac00}' && c <= '\u{d7a3}');
    let has_arabic = text.chars().any(|c| c >= '\u{0600}' && c <= '\u{06ff}');
    let has_cyrillic = text.chars().any(|c| c >= '\u{0400}' && c <= '\u{04ff}');
    let has_thai = text.chars().any(|c| c >= '\u{0e00}' && c <= '\u{0e7f}');
    let has_devanagari = text.chars().any(|c| c >= '\u{0900}' && c <= '\u{097f}');
    
    if has_japanese_kana { return "Japanese"; }
    if has_korean { return "Korean"; }
    if has_chinese { return "Chinese"; }
    if has_arabic { return "Arabic"; }
    if has_cyrillic { return "Russian"; }
    if has_thai { return "Thai"; }
    if has_devanagari { return "Hindi"; }
    
    // 检测欧洲语言特征
    let lower = text.to_lowercase();
    if lower.contains("ñ") || lower.contains("¿") || lower.contains("¡") { return "Spanish"; }
    if lower.contains("ç") || lower.contains("œ") || lower.contains("ê") { return "French"; }
    if lower.contains("ß") || lower.contains("ü") || lower.contains("ö") || lower.contains("ä") { return "German"; }
    
    "English"
}

/// 跨语言测试用例
struct CrossLangTestCase {
    input: &'static str,
    expected_lang: &'static str,
    semantic_category: &'static str,  // greeting, question, statement, etc.
}

fn get_test_cases() -> Vec<CrossLangTestCase> {
    vec![
        // 问候语 - 各语言
        CrossLangTestCase { input: "Hello, how are you?", expected_lang: "English", semantic_category: "greeting" },
        CrossLangTestCase { input: "你好，最近怎么样？", expected_lang: "Chinese", semantic_category: "greeting" },
        CrossLangTestCase { input: "こんにちは、お元気ですか？", expected_lang: "Japanese", semantic_category: "greeting" },
        CrossLangTestCase { input: "안녕하세요, 어떻게 지내세요?", expected_lang: "Korean", semantic_category: "greeting" },
        CrossLangTestCase { input: "Bonjour, comment allez-vous?", expected_lang: "French", semantic_category: "greeting" },
        CrossLangTestCase { input: "Hallo, wie geht es Ihnen?", expected_lang: "German", semantic_category: "greeting" },
        CrossLangTestCase { input: "Привет, как дела?", expected_lang: "Russian", semantic_category: "greeting" },
        CrossLangTestCase { input: "مرحبا، كيف حالك؟", expected_lang: "Arabic", semantic_category: "greeting" },
        CrossLangTestCase { input: "¡Hola! ¿Cómo estás?", expected_lang: "Spanish", semantic_category: "greeting" },
        
        // 疑问句 - 各语言
        CrossLangTestCase { input: "What is artificial intelligence?", expected_lang: "English", semantic_category: "question" },
        CrossLangTestCase { input: "人工智能是什么？", expected_lang: "Chinese", semantic_category: "question" },
        CrossLangTestCase { input: "人工知能とは何ですか？", expected_lang: "Japanese", semantic_category: "question" },
        CrossLangTestCase { input: "인공지능이란 무엇인가요?", expected_lang: "Korean", semantic_category: "question" },
        CrossLangTestCase { input: "Qu'est-ce que l'intelligence artificielle?", expected_lang: "French", semantic_category: "question" },
        CrossLangTestCase { input: "Was ist künstliche Intelligenz?", expected_lang: "German", semantic_category: "question" },
        
        // 陈述句 - 各语言
        CrossLangTestCase { input: "The weather is nice today.", expected_lang: "English", semantic_category: "statement" },
        CrossLangTestCase { input: "今天天气很好。", expected_lang: "Chinese", semantic_category: "statement" },
        CrossLangTestCase { input: "今日は天気がいいですね。", expected_lang: "Japanese", semantic_category: "statement" },
        CrossLangTestCase { input: "오늘 날씨가 좋네요.", expected_lang: "Korean", semantic_category: "statement" },
        
        // 请求/命令 - 各语言
        CrossLangTestCase { input: "Please help me with this.", expected_lang: "English", semantic_category: "request" },
        CrossLangTestCase { input: "请帮我解决这个问题。", expected_lang: "Chinese", semantic_category: "request" },
        CrossLangTestCase { input: "これを手伝ってください。", expected_lang: "Japanese", semantic_category: "request" },
        
        // 混合语言测试
        CrossLangTestCase { input: "Hello 你好 こんにちは", expected_lang: "Japanese", semantic_category: "mixed" },
        CrossLangTestCase { input: "我想学习 machine learning", expected_lang: "Chinese", semantic_category: "mixed" },
        CrossLangTestCase { input: "Valhalla システム test", expected_lang: "Japanese", semantic_category: "mixed" },
    ]
}

/// 本地语料库（简化版，用于响应生成）
struct MultiLangCorpus {
    by_lang: HashMap<String, Vec<String>>,
}

impl MultiLangCorpus {
    fn load() -> Self {
        let mut by_lang: HashMap<String, Vec<String>> = HashMap::new();
        
        // 预设各语言响应模板
        by_lang.insert("English".to_string(), vec![
            "I understand your message.".to_string(),
            "That's an interesting point.".to_string(),
            "Let me help you with that.".to_string(),
            "Thank you for your input.".to_string(),
        ]);
        
        by_lang.insert("Chinese".to_string(), vec![
            "我理解你的意思。".to_string(),
            "这是个有趣的观点。".to_string(),
            "让我来帮助你。".to_string(),
            "谢谢你的输入。".to_string(),
        ]);
        
        by_lang.insert("Japanese".to_string(), vec![
            "お話を理解しました。".to_string(),
            "それは興味深いですね。".to_string(),
            "お手伝いしましょう。".to_string(),
            "ご入力ありがとうございます。".to_string(),
        ]);
        
        by_lang.insert("Korean".to_string(), vec![
            "말씀을 이해했습니다.".to_string(),
            "흥미로운 관점이네요.".to_string(),
            "도와드리겠습니다.".to_string(),
            "입력해 주셔서 감사합니다.".to_string(),
        ]);
        
        by_lang.insert("French".to_string(), vec![
            "Je comprends votre message.".to_string(),
            "C'est un point intéressant.".to_string(),
            "Laissez-moi vous aider.".to_string(),
            "Merci pour votre contribution.".to_string(),
        ]);
        
        by_lang.insert("German".to_string(), vec![
            "Ich verstehe Ihre Nachricht.".to_string(),
            "Das ist ein interessanter Punkt.".to_string(),
            "Lassen Sie mich Ihnen helfen.".to_string(),
            "Vielen Dank für Ihre Eingabe.".to_string(),
        ]);
        
        by_lang.insert("Russian".to_string(), vec![
            "Я понимаю ваше сообщение.".to_string(),
            "Это интересная точка зрения.".to_string(),
            "Позвольте мне помочь вам.".to_string(),
            "Спасибо за ваш вклад.".to_string(),
        ]);
        
        by_lang.insert("Arabic".to_string(), vec![
            "أفهم رسالتك.".to_string(),
            "هذه نقطة مثيرة للاهتمام.".to_string(),
            "دعني أساعدك.".to_string(),
            "شكرا لمساهمتك.".to_string(),
        ]);
        
        by_lang.insert("Spanish".to_string(), vec![
            "Entiendo tu mensaje.".to_string(),
            "Es un punto interesante.".to_string(),
            "Déjame ayudarte.".to_string(),
            "Gracias por tu aporte.".to_string(),
        ]);
        
        // 加载本地对话文件增强语料
        let paths = [
            "/home/jouly/AI/Valhalla/valhalla_all_dialogues.txt",
            "/home/jouly/AI/Valhalla/valhalla_enhanced_dialogues.txt",
        ];
        
        for path in paths {
            if let Ok(content) = fs::read_to_string(path) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() && trimmed.len() > 10 {
                        let lang = detect_language(trimmed);
                        by_lang.entry(lang.to_string()).or_insert_with(Vec::new).push(trimmed.to_string());
                    }
                }
            }
        }
        
        Self { by_lang }
    }
    
    fn respond(&self, input_lang: &str, category: &str, tick: u64) -> String {
        if let Some(responses) = self.by_lang.get(input_lang) {
            if !responses.is_empty() {
                // 基于语义类别和 tick 选择响应
                let category_offset = match category {
                    "greeting" => 0,
                    "question" => 1,
                    "statement" => 2,
                    "request" => 3,
                    _ => tick as usize,
                };
                let idx = (tick as usize + category_offset) % responses.len();
                return responses[idx].clone();
            }
        }
        
        // 回退到英语
        if let Some(responses) = self.by_lang.get("English") {
            if !responses.is_empty() {
                let idx = tick as usize % responses.len();
                return responses[idx].clone();
            }
        }
        
        format!("[{} response at tick {}]", input_lang, tick)
    }
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════════════╗");
    println!("║         🌍 跨语言统一能力测试                                      ║");
    println!("║         测试 Hub 多语言处理 + Fate 路由 + 语言检测                 ║");
    println!("╚════════════════════════════════════════════════════════════════════╝\n");
    
    let start = Instant::now();
    
    // 初始化
    let config = HubConfig {
        resource_percent: 70,
        batch_size: 1024,
        ..Default::default()
    };
    let mut hub = ValhallaHub::with_config(config);
    let corpus = MultiLangCorpus::load();
    let test_cases = get_test_cases();
    
    println!("📂 语料库语言统计:");
    for (lang, responses) in &corpus.by_lang {
        println!("   {:12} │ {} 条", lang, responses.len());
    }
    println!("\n📝 测试用例: {} 条\n", test_cases.len());
    println!("═══════════════════════════════════════════════════════════════════════\n");
    
    // 统计
    let mut lang_correct = 0;
    let mut total_tests = 0;
    let mut lang_routing: HashMap<String, HashMap<String, u32>> = HashMap::new();  // lang -> system -> count
    let mut category_routing: HashMap<String, HashMap<String, u32>> = HashMap::new();  // category -> system -> count
    
    // 运行测试
    for (i, case) in test_cases.iter().enumerate() {
        total_tests += 1;
        
        // 检测语言
        let detected_lang = detect_language(case.input);
        let lang_match = detected_lang == case.expected_lang;
        if lang_match { lang_correct += 1; }
        
        // 创建请求
        let intensity = (case.input.len() as f64 / 100.0).clamp(0.1, 1.0);
        let req_type = match case.semantic_category {
            "greeting" => RequestType::SystemCoordination,
            "question" => RequestType::Reasoning,
            "statement" => RequestType::DataProcessing,
            "request" => RequestType::Adaptation,
            _ => RequestType::Learning,
        };
        
        let request = HubRequest {
            request_id: format!("crosslang_{}", i),
            request_type: req_type,
            intensity,
            data: Some(serde_json::json!({ 
                "text": case.input,
                "lang": detected_lang,
                "category": case.semantic_category
            })),
            priority: Priority::Normal,
        };
        
        // 处理请求
        let response = hub.process(request);
        
        // 获取路由目标
        let routed_to = if response.message.contains("BlackHole") {
            "BlackHole"
        } else if response.message.contains("MeshBrain") {
            "MeshBrain"
        } else if response.message.contains("MultiNova") {
            "MultiNova"
        } else {
            "BaseForce"
        };
        
        // 记录路由统计
        *lang_routing
            .entry(detected_lang.to_string())
            .or_insert_with(HashMap::new)
            .entry(routed_to.to_string())
            .or_insert(0) += 1;
        
        *category_routing
            .entry(case.semantic_category.to_string())
            .or_insert_with(HashMap::new)
            .entry(routed_to.to_string())
            .or_insert(0) += 1;
        
        // 生成响应
        let status = hub.get_status();
        let reply = corpus.respond(detected_lang, case.semantic_category, status.fate_signal_tick);
        
        // 显示结果
        let lang_icon = if lang_match { "✓" } else { "✗" };
        let lang_flag = match detected_lang {
            "Chinese" => "🇨🇳",
            "Japanese" => "🇯🇵",
            "Korean" => "🇰🇷",
            "English" => "🇬🇧",
            "French" => "🇫🇷",
            "German" => "🇩🇪",
            "Russian" => "🇷🇺",
            "Arabic" => "🇸🇦",
            "Spanish" => "🇪🇸",
            _ => "🌐",
        };
        
        println!("┌─ 测试 #{:<2} ────────────────────────────────────────────────────────", i + 1);
        println!("│ 输入: {}", case.input);
        println!("│ 语言: {} {} [{}] 预期: {} | 类别: {}", 
            lang_flag, detected_lang, lang_icon, case.expected_lang, case.semantic_category);
        println!("│ 路由: {} | 强度: {:.2} | 耗时: {:.3}ms", 
            routed_to, intensity, response.processing_time_ms);
        println!("│ 响应: {}", reply);
        println!("└───────────────────────────────────────────────────────────────────────\n");
    }
    
    // 最终统计
    let elapsed = start.elapsed();
    let prefs = hub.get_fate_preferences();
    
    println!("═══════════════════════════════════════════════════════════════════════");
    println!("                         📊 跨语言测试报告                             ");
    println!("═══════════════════════════════════════════════════════════════════════\n");
    
    // 1. 语言检测准确率
    let accuracy = lang_correct as f64 / total_tests as f64 * 100.0;
    println!("1. 语言检测准确率: {}/{} ({:.1}%)\n", lang_correct, total_tests, accuracy);
    
    // 2. 语言-路由分布
    println!("2. 语言 → 系统路由分布:");
    println!("   ─────────────┬──────────┬──────────┬──────────┬──────────");
    println!("   语言         │ BlackHole│ MeshBrain│ MultiNova│ BaseForce");
    println!("   ─────────────┼──────────┼──────────┼──────────┼──────────");
    
    let mut langs: Vec<_> = lang_routing.keys().collect();
    langs.sort();
    for lang in langs {
        if let Some(systems) = lang_routing.get(lang) {
            let bh = systems.get("BlackHole").unwrap_or(&0);
            let mb = systems.get("MeshBrain").unwrap_or(&0);
            let mn = systems.get("MultiNova").unwrap_or(&0);
            let bf = systems.get("BaseForce").unwrap_or(&0);
            println!("   {:12} │ {:>8} │ {:>8} │ {:>8} │ {:>8}", lang, bh, mb, mn, bf);
        }
    }
    println!();
    
    // 3. 语义类别-路由分布
    println!("3. 语义类别 → 系统路由分布:");
    println!("   ─────────────┬──────────┬──────────┬──────────┬──────────");
    println!("   类别         │ BlackHole│ MeshBrain│ MultiNova│ BaseForce");
    println!("   ─────────────┼──────────┼──────────┼──────────┼──────────");
    
    let mut categories: Vec<_> = category_routing.keys().collect();
    categories.sort();
    for category in categories {
        if let Some(systems) = category_routing.get(category) {
            let bh = systems.get("BlackHole").unwrap_or(&0);
            let mb = systems.get("MeshBrain").unwrap_or(&0);
            let mn = systems.get("MultiNova").unwrap_or(&0);
            let bf = systems.get("BaseForce").unwrap_or(&0);
            println!("   {:12} │ {:>8} │ {:>8} │ {:>8} │ {:>8}", category, bh, mb, mn, bf);
        }
    }
    println!();
    
    // 4. 最终 Fate 偏好
    println!("4. 最终 Fate 偏好:");
    println!("   🕳️ BlackHole: {:>6.2}%", prefs.affinities[0] * 100.0);
    println!("   🧠 MeshBrain: {:>6.2}%", prefs.affinities[1] * 100.0);
    println!("   💫 MultiNova: {:>6.2}%", prefs.affinities[2] * 100.0);
    println!("   ⚡ BaseForce: {:>6.2}%", prefs.affinities[3] * 100.0);
    println!("   主导系统: {}\n", ["BlackHole", "MeshBrain", "MultiNova", "BaseForce"][prefs.preferred_system()]);
    
    // 5. 跨语言统一能力评估
    println!("5. 跨语言统一能力评估:");
    
    // 计算路由一致性（相同语义类别的不同语言是否路由到相似系统）
    let mut consistency_score = 0.0;
    let mut consistency_count = 0;
    
    for (category, systems) in &category_routing {
        let total: u32 = systems.values().sum();
        if total > 1 {
            let max_count = *systems.values().max().unwrap_or(&0);
            consistency_score += max_count as f64 / total as f64;
            consistency_count += 1;
        }
    }
    
    let avg_consistency = if consistency_count > 0 {
        consistency_score / consistency_count as f64 * 100.0
    } else {
        0.0
    };
    
    println!("   语义一致性: {:.1}% (相同类别跨语言路由到同一系统的比例)", avg_consistency);
    println!("   语言覆盖: {} 种语言", lang_routing.len());
    println!("   总测试: {} 条\n", total_tests);
    
    println!("═══════════════════════════════════════════════════════════════════════");
    println!("⏱️  总耗时: {:.2}ms", elapsed.as_secs_f64() * 1000.0);
    println!("═══════════════════════════════════════════════════════════════════════");
}
