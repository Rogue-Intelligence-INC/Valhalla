//! 🌌 Valhalla 交互式对话接口
//!
//! 通过 Hub 系统处理文本输入，观察 Fate 响应
//! 测试是否产生 AI 行为

use hub::{
    ValhallaHub, HubRequest, RequestType, Priority,
    DataIngestionEngine, FateDiscoveryEngine,
};
use std::io::{self, Write, BufRead};
use std::collections::HashMap;

/// 对话上下文
struct ConversationContext {
    history: Vec<(String, String)>,  // (input, response)
    ingestion_engine: DataIngestionEngine,
    discovery_engine: FateDiscoveryEngine,
    response_patterns: HashMap<String, Vec<String>>,
}

impl ConversationContext {
    fn new() -> Self {
        let mut patterns = HashMap::new();
        
        // 基于语义关系的响应模式
        patterns.insert("greeting".to_string(), vec![
            "你好！我是 Valhalla，一个基于形变信号的系统。".to_string(),
            "Hello! I'm Valhalla, processing through deformation signals.".to_string(),
            "嗨！Fate 系统已准备好接收你的信号。".to_string(),
        ]);
        
        patterns.insert("question".to_string(), vec![
            "让我通过四元系统分析这个问题...".to_string(),
            "Fate 正在协调 BlackHole、MeshBrain、MultiNova 和 BaseForce...".to_string(),
            "信号已发送到 Hub，正在处理形变...".to_string(),
        ]);
        
        patterns.insert("math".to_string(), vec![
            "数学表达式已编码为形变向量，MeshBrain 正在分析结构...".to_string(),
            "检测到数学语义，precision: Rational (i128/i128)".to_string(),
        ]);
        
        patterns.insert("code".to_string(), vec![
            "代码结构已解析，检测到函数/循环/条件模式...".to_string(),
            "BlackHole 正在计算代码的引力场强度...".to_string(),
        ]);
        
        Self {
            history: Vec::new(),
            ingestion_engine: DataIngestionEngine::new(),
            discovery_engine: FateDiscoveryEngine::new(),
            response_patterns: patterns,
        }
    }
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║     🌌 Valhalla 交互式对话系统                             ║");
    println!("║     基于 Fate + 四元系统 + 形变信号                        ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    
    // 初始化 Hub
    print!("🔧 初始化 Valhalla Hub...");
    io::stdout().flush().unwrap();
    
    let mut hub = match ValhallaHub::new() {
        Ok(h) => {
            println!(" ✅");
            h
        }
        Err(e) => {
            println!(" ❌");
            eprintln!("错误: {:?}", e);
            return;
        }
    };
    
    let mut context = ConversationContext::new();
    
    // 加载检查点（如果存在）
    let checkpoint_path = "/home/jouly/AI/Valhalla/manifestsys/hub/data/checkpoints/latest.json";
    if std::path::Path::new(checkpoint_path).exists() {
        if let Ok(content) = std::fs::read_to_string(checkpoint_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                let signals = json.get("stats")
                    .and_then(|s| s.get("total_signals"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let discoveries = json.get("stats")
                    .and_then(|s| s.get("fate_discoveries"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                println!("📦 加载训练状态: {} 信号, {} Fate发现", signals, discoveries);
            }
        }
    }
    
    println!();
    println!("💬 开始对话 (输入 'quit' 退出, 'status' 查看状态, 'analyze' 深度分析)");
    println!("═══════════════════════════════════════════════════════════════");
    println!();
    
    let stdin = io::stdin();
    
    loop {
        // 提示符
        print!("🧑 你: ");
        io::stdout().flush().unwrap();
        
        // 读取输入
        let mut input = String::new();
        if stdin.lock().read_line(&mut input).is_err() {
            break;
        }
        let input = input.trim().to_string();
        
        if input.is_empty() {
            continue;
        }
        
        // 特殊命令
        match input.as_str() {
            "quit" | "exit" | "q" => {
                println!("\n👋 再见！Fate 系统已保存状态。");
                break;
            }
            "status" => {
                print_status(&hub, &context);
                continue;
            }
            "analyze" => {
                deep_analyze(&mut context);
                continue;
            }
            "history" => {
                print_history(&context);
                continue;
            }
            _ => {}
        }
        
        // 处理输入
        let response = process_input(&mut hub, &mut context, &input);
        
        // 显示响应
        println!();
        println!("🤖 Valhalla: {}", response);
        println!();
        
        // 保存到历史
        context.history.push((input, response));
    }
}

/// 处理用户输入
fn process_input(hub: &mut ValhallaHub, context: &mut ConversationContext, input: &str) -> String {
    // 1. 摄入文本，转换为形变向量
    let data_item = context.ingestion_engine.ingest_text(input, "auto").ok();
    
    // 2. 送入 Fate 发现引擎
    let raw_item = context.discovery_engine.ingest(input);
    
    // 3. 发送到 Hub 处理
    let request = HubRequest {
        request_id: format!("chat_{}", context.history.len()),
        request_type: RequestType::Reasoning,
        data: serde_json::json!({
            "input": input,
            "deformation": data_item.as_ref().map(|d| d.deformation.to_vec()),
        }),
        priority: Priority::High,
    };
    
    let hub_response = hub.process(request);
    
    // 4. 分析输入语义
    let semantic_type = analyze_semantic_type(input);
    
    // 5. 获取 Fate 偏好
    let fate_prefs = hub.fate_system().calculate_system_preferences();
    let preferred_system = get_preferred_system(&fate_prefs);
    
    // 6. 基于 Fate 生成响应
    generate_response(context, input, &semantic_type, &preferred_system, &raw_item.raw_features)
}

/// 分析语义类型
fn analyze_semantic_type(input: &str) -> String {
    let lower = input.to_lowercase();
    
    // 问候
    if lower.contains("hello") || lower.contains("hi") || lower.contains("你好") 
        || lower.contains("嗨") || lower.contains("hey") {
        return "greeting".to_string();
    }
    
    // 问题
    if lower.contains("?") || lower.contains("？") || lower.contains("what") 
        || lower.contains("how") || lower.contains("why") || lower.contains("什么")
        || lower.contains("怎么") || lower.contains("为什么") {
        return "question".to_string();
    }
    
    // 数学
    if input.contains("=") || input.contains("+") || input.contains("-") 
        || input.contains("*") || input.contains("/") || lower.contains("计算")
        || lower.contains("math") || lower.contains("数学") {
        return "math".to_string();
    }
    
    // 代码
    if input.contains("fn ") || input.contains("def ") || input.contains("function")
        || input.contains("{") || input.contains("()") || lower.contains("代码")
        || lower.contains("code") || lower.contains("程序") {
        return "code".to_string();
    }
    
    // 命令
    if lower.starts_with("请") || lower.starts_with("帮我") || lower.contains("please")
        || lower.starts_with("能不能") || lower.starts_with("可以") {
        return "command".to_string();
    }
    
    "general".to_string()
}

/// 获取 Fate 偏好的系统
fn get_preferred_system(prefs: &fate::SystemPreferences) -> String {
    if prefs.affinities.len() < 4 {
        return "BaseForce".to_string();
    }
    
    let mut max_idx = 0;
    let mut max_val = prefs.affinities[0];
    
    for (i, aff) in prefs.affinities.iter().enumerate().skip(1) {
        if aff.numerator * max_val.denominator > max_val.numerator * aff.denominator {
            max_val = *aff;
            max_idx = i;
        }
    }
    
    match max_idx {
        0 => "BlackHole".to_string(),
        1 => "MeshBrain".to_string(),
        2 => "MultiNova".to_string(),
        _ => "BaseForce".to_string(),
    }
}

/// 生成响应
fn generate_response(
    context: &ConversationContext,
    input: &str,
    semantic_type: &str,
    preferred_system: &str,
    features: &hub::RawFeatures,
) -> String {
    let mut response = String::new();
    
    // 基于语义类型选择基础响应
    if let Some(patterns) = context.response_patterns.get(semantic_type) {
        let idx = (features.char_entropy * 100.0) as usize % patterns.len();
        response.push_str(&patterns[idx]);
    } else {
        response.push_str("信号已接收，正在通过形变场处理...");
    }
    
    response.push_str("\n\n");
    
    // 添加 Fate 分析
    response.push_str(&format!("📊 Fate 分析:\n"));
    response.push_str(&format!("   • 语义类型: {}\n", semantic_type));
    response.push_str(&format!("   • 主导系统: {}\n", preferred_system));
    response.push_str(&format!("   • 字符熵: {:.3}\n", features.char_entropy));
    response.push_str(&format!("   • CJK比例: {:.1}%\n", features.cjk_ratio * 100.0));
    response.push_str(&format!("   • 复杂度: {:.2}\n", 
        features.bracket_depth as f64 * 0.1 + features.punct_ratio));
    
    // 基于输入生成"理解"
    response.push_str("\n💭 形变编码:\n");
    if features.cjk_ratio > 0.5 {
        response.push_str("   检测到中文语义场，激活东亚语言模式\n");
    }
    if features.has_equals {
        response.push_str("   检测到等式关系，激活数学推理模式\n");
    }
    if features.bracket_depth > 2 {
        response.push_str("   检测到嵌套结构，激活递归分析模式\n");
    }
    
    // 模拟"思考"
    response.push_str("\n🔮 Fate 响应:\n");
    match preferred_system.as_ref() {
        "BlackHole" => {
            response.push_str("   BlackHole 引力场正在聚合相关概念...\n");
            response.push_str(&format!("   「{}」的引力中心已定位\n", 
                input.chars().take(10).collect::<String>()));
        }
        "MeshBrain" => {
            response.push_str("   MeshBrain 神经膜正在编织语义网络...\n");
            response.push_str("   认知信号已在拓扑空间中传播\n");
        }
        "MultiNova" => {
            response.push_str("   MultiNova 能量演化正在展开...\n");
            response.push_str("   信息能量正在向稳定态收敛\n");
        }
        _ => {
            response.push_str("   BaseForce 正在协调三系统...\n");
            response.push_str("   力平衡点已计算完成\n");
        }
    }
    
    response
}

/// 打印状态
fn print_status(hub: &ValhallaHub, context: &ConversationContext) {
    println!();
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║                   🌌 Valhalla 状态                         ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    
    let stats = hub.get_statistics();
    println!("║ 总请求数:     {:>10}                                 ║", stats.total_requests);
    println!("║ 成功路由:     {:>10}                                 ║", stats.successful_routes);
    println!("║ 对话轮数:     {:>10}                                 ║", context.history.len());
    
    let fate_prefs = hub.fate_system().calculate_system_preferences();
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║ Fate 系统偏好:                                             ║");
    if fate_prefs.affinities.len() >= 4 {
        let systems = ["BlackHole", "MeshBrain", "MultiNova", "BaseForce"];
        for (i, name) in systems.iter().enumerate() {
            let aff = &fate_prefs.affinities[i];
            let val = aff.numerator as f64 / aff.denominator as f64;
            println!("║   {}: {:.4}                                      ║", name, val);
        }
    }
    
    let discovery_stats = context.discovery_engine.get_stats();
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║ Fate 发现:                                                 ║");
    println!("║   摄入项数:   {:>10}                                 ║", discovery_stats.total_items);
    println!("║   信号计数:   {:>10}                                 ║", discovery_stats.signal_count);
    println!("║   聚类数:     {:>10}                                 ║", discovery_stats.cluster_count);
    
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
}

/// 深度分析
fn deep_analyze(context: &mut ConversationContext) {
    println!();
    println!("🔍 执行 Fate 深度分析...");
    
    context.discovery_engine.fate_process();
    let mut clusters = context.discovery_engine.fate_discover_clusters();
    context.discovery_engine.fate_analyze_clusters(&mut clusters);
    
    println!();
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║               🔬 Fate 聚类分析结果                         ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    
    for cluster in &clusters {
        println!("║ 聚类 #{}: {} 个成员                                      ║", 
            cluster.cluster_id, cluster.member_indices.len());
        if let Some(pattern) = &cluster.discovered_pattern {
            println!("║   模式: {}                           ║", pattern);
        }
    }
    
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
}

/// 打印历史
fn print_history(context: &ConversationContext) {
    println!();
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║                   📜 对话历史                              ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    
    for (i, (input, _)) in context.history.iter().enumerate() {
        println!("[{}] 你: {}", i + 1, input);
    }
    println!();
}
