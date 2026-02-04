//! 🌌 Hub 互动测试
//!
//! 展示 Hub 内部数据流动和 Fate 决策过程

use std::io::{self, Write, BufRead};
use std::fs;
use std::collections::HashMap;

use hub_f64::{
    ValhallaHub, HubRequest, HubConfig, RequestType, Priority,
};

/// 本地对话语料
struct LocalCorpus {
    dialogues: Vec<String>,
    by_pattern: HashMap<String, Vec<String>>,
}

impl LocalCorpus {
    fn load() -> Self {
        let mut dialogues = Vec::new();
        
        // 加载本地对话文件
        let paths = [
            "/home/jouly/AI/Valhalla/valhalla_all_dialogues.txt",
            "/home/jouly/AI/Valhalla/valhalla_enhanced_dialogues.txt",
        ];
        
        for path in paths {
            if let Ok(content) = fs::read_to_string(path) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() && trimmed.len() > 5 {
                        dialogues.push(trimmed.to_string());
                    }
                }
            }
        }
        
        // 按关键词分类
        let mut by_pattern: HashMap<String, Vec<String>> = HashMap::new();
        
        let patterns = [
            ("hello", vec!["你好", "hello", "hi", "嗨"]),
            ("who", vec!["谁", "who", "什么人"]),
            ("what", vec!["什么", "what", "是什么"]),
            ("how", vec!["怎么", "how", "如何"]),
            ("why", vec!["为什么", "why", "为何"]),
            ("valhalla", vec!["valhalla", "瓦尔哈拉"]),
            ("fate", vec!["fate", "命运"]),
            ("learn", vec!["学习", "learn", "训练"]),
            ("help", vec!["帮助", "help", "帮忙"]),
        ];
        
        for (key, keywords) in patterns {
            let mut matches: Vec<String> = Vec::new();
            for d in &dialogues {
                let lower = d.to_lowercase();
                if keywords.iter().any(|k| lower.contains(k)) {
                    matches.push(d.clone());
                }
            }
            if !matches.is_empty() {
                by_pattern.insert(key.to_string(), matches);
            }
        }
        
        Self { dialogues, by_pattern }
    }
    
    fn respond(&self, input: &str, fate_tick: u64, dominant_system: usize) -> String {
        let lower = input.to_lowercase();
        
        // 尝试匹配模式
        let patterns = [
            ("hello", vec!["你好", "hello", "hi", "嗨"]),
            ("who", vec!["谁", "who", "什么人"]),
            ("what", vec!["什么", "what", "是什么"]),
            ("how", vec!["怎么", "how", "如何"]),
            ("why", vec!["为什么", "why", "为何"]),
            ("valhalla", vec!["valhalla", "瓦尔哈拉"]),
            ("fate", vec!["fate", "命运"]),
            ("learn", vec!["学习", "learn", "训练"]),
            ("help", vec!["帮助", "help", "帮忙"]),
        ];
        
        for (key, keywords) in patterns {
            if keywords.iter().any(|k| lower.contains(k)) {
                if let Some(matches) = self.by_pattern.get(key) {
                    if !matches.is_empty() {
                        let idx = (fate_tick as usize + input.len() + dominant_system * 7) % matches.len();
                        return matches[idx].clone();
                    }
                }
            }
        }
        
        // 默认：基于 Fate tick 选择
        if self.dialogues.is_empty() {
            return format!("[系统{}处理完成 tick={}]", 
                ["BlackHole", "MeshBrain", "MultiNova", "BaseForce"][dominant_system],
                fate_tick);
        }
        
        let idx = (fate_tick as usize + input.len()) % self.dialogues.len();
        self.dialogues[idx].clone()
    }
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════════════╗");
    println!("║         🌌 Hub 互动测试                                            ║");
    println!("║         展示 Fate 决策 + 四系统路由 + 内部状态                     ║");
    println!("╚════════════════════════════════════════════════════════════════════╝\n");
    
    // 初始化
    let config = HubConfig {
        resource_percent: 70,
        batch_size: 1024,
        ..Default::default()
    };
    let mut hub = ValhallaHub::with_config(config);
    let corpus = LocalCorpus::load();
    
    println!("📂 本地语料: {} 条对话\n", corpus.dialogues.len());
    println!("命令: quit/exit 退出 | status 查看状态 | fate 查看偏好\n");
    println!("─────────────────────────────────────────────────────────────────────\n");
    
    let stdin = io::stdin();
    let mut counter = 0u64;
    
    loop {
        print!("你> ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        if stdin.lock().read_line(&mut input).is_err() {
            break;
        }
        
        let input = input.trim();
        if input.is_empty() {
            continue;
        }
        
        // 特殊命令
        match input.to_lowercase().as_str() {
            "quit" | "exit" => {
                println!("\n👋 再见！\n");
                break;
            }
            "status" => {
                let stats = hub.get_statistics();
                let status = hub.get_status();
                println!("\n📊 Hub 状态:");
                println!("   总请求: {} | 成功: {} | 失败: {}", 
                    stats.total_requests, stats.successful_routes, stats.failed_routes);
                println!("   平均响应: {:.3}ms", stats.average_response_time_ms);
                println!("   Fate tick: {}", status.fate_signal_tick);
                println!("   活跃系统: {:?}\n", status.active_systems);
                continue;
            }
            "fate" => {
                let prefs = hub.get_fate_preferences();
                let pattern = hub.get_fate_convergence_pattern();
                println!("\n🔮 Fate 状态:");
                println!("   偏好分布:");
                println!("     🕳️ BlackHole: {:>6.2}%", prefs.affinities[0] * 100.0);
                println!("     🧠 MeshBrain: {:>6.2}%", prefs.affinities[1] * 100.0);
                println!("     💫 MultiNova: {:>6.2}%", prefs.affinities[2] * 100.0);
                println!("     ⚡ BaseForce: {:>6.2}%", prefs.affinities[3] * 100.0);
                println!("   收敛模式: {:?}", pattern);
                println!("   主导系统: {}\n", ["BlackHole", "MeshBrain", "MultiNova", "BaseForce"]
                    [prefs.preferred_system()]);
                continue;
            }
            _ => {}
        }
        
        counter += 1;
        
        // 计算信号强度
        let intensity = (input.len() as f64 / 100.0).clamp(0.1, 1.0);
        
        // 分类请求类型
        let req_type = if input.contains('?') || input.contains('？') {
            RequestType::Reasoning
        } else if input.contains("学") || input.contains("learn") {
            RequestType::Learning
        } else if input.contains("代码") || input.contains("code") {
            RequestType::DataProcessing
        } else {
            RequestType::SystemCoordination
        };
        
        // 创建请求
        let request = HubRequest {
            request_id: format!("interact_{}", counter),
            request_type: req_type,
            intensity,
            data: Some(serde_json::json!({ "text": input })),
            priority: Priority::Normal,
        };
        
        // 处理请求
        let response = hub.process(request);
        
        // 获取 Fate 状态
        let prefs = hub.get_fate_preferences();
        let status = hub.get_status();
        let dominant = prefs.preferred_system();
        
        // 从响应中提取路由目标
        let routed_to = if response.message.contains("BlackHole") {
            "🕳️ BlackHole"
        } else if response.message.contains("MeshBrain") {
            "🧠 MeshBrain"
        } else if response.message.contains("MultiNova") {
            "💫 MultiNova"
        } else {
            "⚡ BaseForce"
        };
        
        // 生成响应
        let reply = corpus.respond(input, status.fate_signal_tick, dominant);
        
        // 显示详细信息
        println!("\n┌─ Hub 处理 ─────────────────────────────────────────────────────");
        println!("│ 路由: {} | 强度: {:.2} | 耗时: {:.3}ms", 
            routed_to, intensity, response.processing_time_ms);
        println!("│ Fate偏好: BH={:.1}% MB={:.1}% MN={:.1}% BF={:.1}%",
            prefs.affinities[0] * 100.0, prefs.affinities[1] * 100.0,
            prefs.affinities[2] * 100.0, prefs.affinities[3] * 100.0);
        println!("└────────────────────────────────────────────────────────────────");
        println!("\nValhalla> {}\n", reply);
    }
    
    // 最终统计
    let stats = hub.get_statistics();
    let prefs = hub.get_fate_preferences();
    
    println!("═══════════════════════════════════════════════════════════════════");
    println!("📊 会话统计:");
    println!("   总交互: {} 次", stats.total_requests);
    println!("   平均响应: {:.3}ms", stats.average_response_time_ms);
    println!("   最终Fate偏好: BH={:.1}% MB={:.1}% MN={:.1}% BF={:.1}%",
        prefs.affinities[0] * 100.0, prefs.affinities[1] * 100.0,
        prefs.affinities[2] * 100.0, prefs.affinities[3] * 100.0);
    println!("═══════════════════════════════════════════════════════════════════");
}
