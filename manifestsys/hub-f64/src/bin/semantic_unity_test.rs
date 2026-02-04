//! 🧠 语义统一能力测试
//!
//! 核心问题：同一个问题用不同语言表达，是否路由到同一系统？
//! 对比：与随机轮询分布的差异

use std::collections::HashMap;
use std::time::Instant;
use rand::Rng;

use hub_f64::{
    ValhallaHub, HubRequest, HubConfig, RequestType, Priority,
};

/// 语义等价组 - 同一问题的多语言表达
struct SemanticGroup {
    semantic_id: &'static str,
    description: &'static str,
    expressions: Vec<(&'static str, &'static str)>,  // (language, text)
}

fn get_semantic_groups() -> Vec<SemanticGroup> {
    vec![
        // 问候语
        SemanticGroup {
            semantic_id: "greeting_1",
            description: "问候 - 你好吗",
            expressions: vec![
                ("English", "Hello, how are you?"),
                ("Chinese", "你好，你怎么样？"),
                ("Japanese", "こんにちは、お元気ですか？"),
                ("Korean", "안녕하세요, 어떻게 지내세요?"),
                ("French", "Bonjour, comment allez-vous?"),
                ("German", "Hallo, wie geht es Ihnen?"),
                ("Spanish", "¡Hola! ¿Cómo estás?"),
                ("Russian", "Привет, как дела?"),
            ],
        },
        
        // 定义问题
        SemanticGroup {
            semantic_id: "definition_ai",
            description: "定义 - 什么是人工智能",
            expressions: vec![
                ("English", "What is artificial intelligence?"),
                ("Chinese", "什么是人工智能？"),
                ("Japanese", "人工知能とは何ですか？"),
                ("Korean", "인공지능이란 무엇인가요?"),
                ("French", "Qu'est-ce que l'intelligence artificielle?"),
                ("German", "Was ist künstliche Intelligenz?"),
                ("Spanish", "¿Qué es la inteligencia artificial?"),
                ("Russian", "Что такое искусственный интеллект?"),
            ],
        },
        
        // 天气陈述
        SemanticGroup {
            semantic_id: "weather_nice",
            description: "陈述 - 今天天气好",
            expressions: vec![
                ("English", "The weather is nice today."),
                ("Chinese", "今天天气很好。"),
                ("Japanese", "今日は天気がいいですね。"),
                ("Korean", "오늘 날씨가 좋네요."),
                ("French", "Il fait beau aujourd'hui."),
                ("German", "Das Wetter ist heute schön."),
                ("Spanish", "Hace buen tiempo hoy."),
                ("Russian", "Сегодня хорошая погода."),
            ],
        },
        
        // 帮助请求
        SemanticGroup {
            semantic_id: "help_request",
            description: "请求 - 帮帮我",
            expressions: vec![
                ("English", "Please help me with this problem."),
                ("Chinese", "请帮我解决这个问题。"),
                ("Japanese", "この問題を手伝ってください。"),
                ("Korean", "이 문제를 도와주세요."),
                ("French", "Aidez-moi avec ce problème."),
                ("German", "Bitte helfen Sie mir bei diesem Problem."),
                ("Spanish", "Por favor ayúdame con este problema."),
                ("Russian", "Пожалуйста, помогите мне с этой проблемой."),
            ],
        },
        
        // 学习意愿
        SemanticGroup {
            semantic_id: "want_learn",
            description: "意愿 - 想学习",
            expressions: vec![
                ("English", "I want to learn programming."),
                ("Chinese", "我想学习编程。"),
                ("Japanese", "プログラミングを学びたいです。"),
                ("Korean", "프로그래밍을 배우고 싶어요."),
                ("French", "Je veux apprendre la programmation."),
                ("German", "Ich möchte Programmieren lernen."),
                ("Spanish", "Quiero aprender programación."),
                ("Russian", "Я хочу изучать программирование."),
            ],
        },
        
        // 时间询问
        SemanticGroup {
            semantic_id: "ask_time",
            description: "询问 - 现在几点",
            expressions: vec![
                ("English", "What time is it now?"),
                ("Chinese", "现在几点了？"),
                ("Japanese", "今何時ですか？"),
                ("Korean", "지금 몇 시예요?"),
                ("French", "Quelle heure est-il?"),
                ("German", "Wie spät ist es?"),
                ("Spanish", "¿Qué hora es?"),
                ("Russian", "Который час?"),
            ],
        },
        
        // 感谢
        SemanticGroup {
            semantic_id: "thank_you",
            description: "感谢 - 谢谢",
            expressions: vec![
                ("English", "Thank you very much!"),
                ("Chinese", "非常感谢！"),
                ("Japanese", "どうもありがとうございます！"),
                ("Korean", "정말 감사합니다!"),
                ("French", "Merci beaucoup!"),
                ("German", "Vielen Dank!"),
                ("Spanish", "¡Muchas gracias!"),
                ("Russian", "Большое спасибо!"),
            ],
        },
        
        // 道歉
        SemanticGroup {
            semantic_id: "sorry",
            description: "道歉 - 对不起",
            expressions: vec![
                ("English", "I am sorry for the mistake."),
                ("Chinese", "对不起，我搞错了。"),
                ("Japanese", "間違えてすみません。"),
                ("Korean", "실수해서 죄송합니다."),
                ("French", "Je suis désolé pour l'erreur."),
                ("German", "Es tut mir leid für den Fehler."),
                ("Spanish", "Lo siento por el error."),
                ("Russian", "Извините за ошибку."),
            ],
        },
    ]
}

const SYSTEMS: [&str; 4] = ["BlackHole", "MeshBrain", "MultiNova", "BaseForce"];

fn main() {
    println!("╔════════════════════════════════════════════════════════════════════╗");
    println!("║         🧠 语义统一能力测试                                        ║");
    println!("║         同一问题 × 多语言 → 是否路由到同一系统？                   ║");
    println!("╚════════════════════════════════════════════════════════════════════╝\n");
    
    let start = Instant::now();
    
    // 初始化 Hub
    let config = HubConfig {
        resource_percent: 70,
        batch_size: 1024,
        ..Default::default()
    };
    let mut hub = ValhallaHub::with_config(config);
    let groups = get_semantic_groups();
    
    println!("📝 测试设计:");
    println!("   语义组数: {}", groups.len());
    println!("   每组语言: {} 种", groups[0].expressions.len());
    println!("   总测试数: {}\n", groups.len() * groups[0].expressions.len());
    
    // 存储结果
    let mut group_results: Vec<(String, String, HashMap<String, u32>)> = Vec::new();  // (id, desc, routing)
    let mut all_routings: Vec<usize> = Vec::new();
    
    println!("═══════════════════════════════════════════════════════════════════════\n");
    
    // 对每个语义组进行测试
    for group in &groups {
        println!("┌─ {} ─────────────────────────────────────────────────", group.description);
        
        let mut routing_counts: HashMap<String, u32> = HashMap::new();
        let mut routing_detail: Vec<(String, String)> = Vec::new();
        
        for (lang, text) in &group.expressions {
            // 创建请求
            let intensity = (text.len() as f64 / 100.0).clamp(0.1, 1.0);
            let request = HubRequest {
                request_id: format!("{}_{}", group.semantic_id, lang),
                request_type: RequestType::Reasoning,  // 统一使用推理类型
                intensity,
                data: Some(serde_json::json!({ "text": text, "lang": lang })),
                priority: Priority::Normal,
            };
            
            // 处理请求
            let response = hub.process(request);
            
            // 提取路由目标
            let routed_to = if response.message.contains("BlackHole") {
                "BlackHole"
            } else if response.message.contains("MeshBrain") {
                "MeshBrain"
            } else if response.message.contains("MultiNova") {
                "MultiNova"
            } else {
                "BaseForce"
            };
            
            let sys_idx = SYSTEMS.iter().position(|&s| s == routed_to).unwrap_or(0);
            all_routings.push(sys_idx);
            
            *routing_counts.entry(routed_to.to_string()).or_insert(0) += 1;
            routing_detail.push((lang.to_string(), routed_to.to_string()));
            
            // 安全截取 (处理 UTF-8 多字节字符)
            let display_text: String = text.chars().take(20).collect();
            println!("│ {:10} │ {} → {}", lang, display_text, routed_to);
        }
        
        // 计算该组的一致性
        let max_count = *routing_counts.values().max().unwrap_or(&0);
        let total = group.expressions.len() as u32;
        let consistency = max_count as f64 / total as f64 * 100.0;
        let dominant_system = routing_counts.iter()
            .max_by_key(|(_, &count)| count)
            .map(|(sys, _)| sys.clone())
            .unwrap_or_default();
        
        println!("│");
        println!("│ 路由分布: BH={} MB={} MN={} BF={}",
            routing_counts.get("BlackHole").unwrap_or(&0),
            routing_counts.get("MeshBrain").unwrap_or(&0),
            routing_counts.get("MultiNova").unwrap_or(&0),
            routing_counts.get("BaseForce").unwrap_or(&0));
        println!("│ 一致性: {:.1}% (主导: {})", consistency, dominant_system);
        println!("└───────────────────────────────────────────────────────────────────────\n");
        
        group_results.push((group.semantic_id.to_string(), group.description.to_string(), routing_counts));
    }
    
    // ═══════════════════════════════════════════════════════════════════
    // 统计分析
    // ═══════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════");
    println!("                         📊 统计分析报告                               ");
    println!("═══════════════════════════════════════════════════════════════════════\n");
    
    // 1. 实际路由分布
    let mut actual_dist: [u32; 4] = [0, 0, 0, 0];
    for &idx in &all_routings {
        actual_dist[idx] += 1;
    }
    let total_tests = all_routings.len() as f64;
    
    println!("1. 实际路由分布:");
    for (i, name) in SYSTEMS.iter().enumerate() {
        let count = actual_dist[i];
        let pct = count as f64 / total_tests * 100.0;
        let bar_len = (pct / 2.5) as usize;
        let bar: String = "█".repeat(bar_len);
        println!("   {:10} │ {:>3} ({:>5.1}%) │ {}", name, count, pct, bar);
    }
    println!();
    
    // 2. 期望随机分布 (均匀)
    let expected_per_system = total_tests / 4.0;
    println!("2. 期望随机分布 (均匀):");
    for name in &SYSTEMS {
        println!("   {:10} │ {:>5.1} ({:>5.1}%)", name, expected_per_system, 25.0);
    }
    println!();
    
    // 3. 卡方检验
    let mut chi_square = 0.0;
    for i in 0..4 {
        let observed = actual_dist[i] as f64;
        let expected = expected_per_system;
        chi_square += (observed - expected).powi(2) / expected;
    }
    
    // 自由度 = 4-1 = 3, 临界值 (p=0.05) ≈ 7.815
    let critical_value = 7.815;
    let is_significant = chi_square > critical_value;
    
    println!("3. 卡方检验 (χ²):");
    println!("   χ² = {:.4}", chi_square);
    println!("   临界值 (df=3, p=0.05) = {:.3}", critical_value);
    println!("   结论: {} ({})", 
        if is_significant { "显著偏离随机" } else { "接近随机分布" },
        if is_significant { "p < 0.05" } else { "p ≥ 0.05" });
    println!();
    
    // 4. 语义一致性分析
    println!("4. 语义一致性分析 (同一语义组内):");
    println!("   ─────────────────────┬────────┬────────┬─────────────");
    println!("   语义组               │ 一致性 │ 主导系统│ 分布");
    println!("   ─────────────────────┼────────┼────────┼─────────────");
    
    let mut total_consistency = 0.0;
    let mut perfect_groups = 0;
    
    for (id, desc, counts) in &group_results {
        let total: u32 = counts.values().sum();
        let max_count = *counts.values().max().unwrap_or(&0);
        let consistency = max_count as f64 / total as f64 * 100.0;
        total_consistency += consistency;
        
        if consistency == 100.0 {
            perfect_groups += 1;
        }
        
        let dominant = counts.iter()
            .max_by_key(|(_, &c)| c)
            .map(|(s, _)| s.as_str())
            .unwrap_or("-");
        
        let dist_str = format!("{}:{}:{}:{}", 
            counts.get("BlackHole").unwrap_or(&0),
            counts.get("MeshBrain").unwrap_or(&0),
            counts.get("MultiNova").unwrap_or(&0),
            counts.get("BaseForce").unwrap_or(&0));
        
        let desc_short = if desc.len() > 18 { &desc[..18] } else { desc };
        println!("   {:20} │ {:>5.1}% │ {:8} │ {}", desc_short, consistency, dominant, dist_str);
    }
    
    let avg_consistency = total_consistency / group_results.len() as f64;
    println!("   ─────────────────────┴────────┴────────┴─────────────");
    println!("   平均一致性: {:.1}%", avg_consistency);
    println!("   完美一致组: {}/{}", perfect_groups, group_results.len());
    println!();
    
    // 5. 对比随机基线
    println!("5. 与随机基线对比:");
    
    // 如果是纯随机，每组8个样本，期望最大频次分布
    // 二项分布 B(8, 0.25)
    let random_expected_consistency = calculate_random_expected_consistency(8, 4);
    
    println!("   实际平均一致性: {:.1}%", avg_consistency);
    println!("   随机期望一致性: {:.1}%", random_expected_consistency);
    println!("   差异: {:+.1}%", avg_consistency - random_expected_consistency);
    
    let improvement_ratio = avg_consistency / random_expected_consistency;
    println!("   改善倍数: {:.2}x", improvement_ratio);
    println!();
    
    // 6. 结论
    println!("6. 结论:");
    if avg_consistency > random_expected_consistency * 1.5 {
        println!("   ✓ 系统展现出语义统一能力");
        println!("     同一语义在不同语言间有较高路由一致性");
    } else if avg_consistency > random_expected_consistency * 1.2 {
        println!("   △ 系统有轻微语义倾向");
        println!("     但未达到显著语义理解水平");
    } else {
        println!("   ✗ 系统路由接近随机");
        println!("     当前 Fate 处于探索/轮询阶段，尚未形成语义偏好");
    }
    
    // 轮询验证
    println!("\n7. 轮询模式验证:");
    let mut is_round_robin = true;
    for i in 0..all_routings.len().saturating_sub(4) {
        let window: Vec<usize> = (0..4).map(|j| all_routings[i + j]).collect();
        let mut sorted = window.clone();
        sorted.sort();
        sorted.dedup();
        if sorted.len() == 4 {
            // 连续4个都不同 = 可能是轮询
        } else {
            is_round_robin = false;
            break;
        }
    }
    
    // 检查是否严格轮询 (0,1,2,3,0,1,2,3...)
    let mut strict_round_robin = true;
    for (i, &routing) in all_routings.iter().enumerate() {
        if routing != i % 4 {
            strict_round_robin = false;
            break;
        }
    }
    
    if strict_round_robin {
        println!("   检测到严格轮询模式 (0→1→2→3→0...)");
        println!("   这是 Fate 探索阶段的预期行为");
    } else {
        // 计算与轮询的偏差
        let mut round_robin_match = 0;
        for (i, &routing) in all_routings.iter().enumerate() {
            if routing == i % 4 {
                round_robin_match += 1;
            }
        }
        let round_robin_pct = round_robin_match as f64 / all_routings.len() as f64 * 100.0;
        println!("   与严格轮询的匹配度: {:.1}%", round_robin_pct);
        
        if round_robin_pct > 90.0 {
            println!("   → 非常接近轮询模式");
        } else if round_robin_pct > 70.0 {
            println!("   → 主要是轮询，有轻微偏离");
        } else {
            println!("   → 已偏离轮询模式");
        }
    }
    
    println!("\n═══════════════════════════════════════════════════════════════════════");
    println!("⏱️  总耗时: {:.2}ms", start.elapsed().as_secs_f64() * 1000.0);
    println!("═══════════════════════════════════════════════════════════════════════");
}

/// 计算随机分布下的期望一致性
/// n = 每组样本数, k = 系统数
fn calculate_random_expected_consistency(n: usize, k: usize) -> f64 {
    // 蒙特卡洛模拟
    let mut rng = rand::thread_rng();
    let trials = 10000;
    let mut total_max = 0.0;
    
    for _ in 0..trials {
        let mut counts = vec![0u32; k];
        for _ in 0..n {
            let choice = rng.gen_range(0..k);
            counts[choice] += 1;
        }
        let max_count = *counts.iter().max().unwrap_or(&0);
        total_max += max_count as f64;
    }
    
    (total_max / trials as f64) / n as f64 * 100.0
}
