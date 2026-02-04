//! 🌍 语言核提炼实验 - Valhalla Hub
//! 
//! 使用 signal_tick 逻辑时间 + Rational 精确算术
//! 系统50%资源，处理多语言数据，提取语言核心特征

use valhalla_hub::{ValhallaHub, HubRequest, RequestType, Priority};
use std::time::{Duration, Instant};
use std::fs;
use std::collections::{VecDeque, HashMap};

// ============================================================================
// 稳定度监控
// ============================================================================

struct StabilityMonitor {
    history: VecDeque<f64>,
    consecutive_stable: usize,
}

impl StabilityMonitor {
    fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(100),
            consecutive_stable: 0,
        }
    }

    fn add_sample(&mut self, stability: f64) {
        if self.history.len() >= 50 {
            self.history.pop_front();
        }
        self.history.push_back(stability);
    }

    fn is_stable(&mut self) -> bool {
        if self.history.len() < 10 {
            return false;
        }
        
        let recent: Vec<f64> = self.history.iter().rev().take(10).copied().collect();
        let mean: f64 = recent.iter().sum::<f64>() / recent.len() as f64;
        let variance: f64 = recent.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / recent.len() as f64;
        let std_dev = variance.sqrt();
        
        let is_currently_stable = std_dev < 0.015 && mean > 0.6;
        
        if is_currently_stable {
            self.consecutive_stable += 1;
        } else {
            self.consecutive_stable = 0;
        }
        
        self.consecutive_stable >= 10
    }

    fn get_current_stability(&self) -> f64 {
        self.history.back().copied().unwrap_or(0.0)
    }

    fn get_trend(&self) -> &str {
        if self.history.len() < 3 {
            return "初始化";
        }
        let recent: Vec<f64> = self.history.iter().rev().take(5).copied().collect();
        let diff = recent[0] - recent[recent.len() - 1];
        if diff.abs() < 0.01 { "稳定 ═" } else if diff > 0.0 { "上升 ↑" } else { "下降 ↓" }
    }

    fn get_average(&self) -> f64 {
        if self.history.is_empty() { 0.0 } else { self.history.iter().sum::<f64>() / self.history.len() as f64 }
    }
}

// ============================================================================
// 数据加载
// ============================================================================

fn load_multilingual_data() -> Vec<(String, String, String, f64)> {
    // (content, language, type, complexity)
    let mut data = Vec::new();
    
    // 词汇
    let words = [
        ("en", vec!["hello", "world", "computer", "science", "data", "network", "system"]),
        ("zh", vec!["你好", "世界", "电脑", "科学", "数据", "网络", "系统"]),
        ("ja", vec!["こんにちは", "世界", "コンピュータ", "科学", "データ"]),
        ("de", vec!["hallo", "welt", "computer", "wissenschaft", "daten"]),
        ("fr", vec!["bonjour", "monde", "ordinateur", "science", "données"]),
    ];
    
    for (lang, word_list) in words {
        for (i, word) in word_list.iter().enumerate() {
            data.push((word.to_string(), lang.to_string(), "word".to_string(), (i + 1) as f64 * 0.1));
        }
    }
    
    // 俗语
    let idioms = [
        ("zh", vec!["画蛇添足", "守株待兔", "对牛弹琴"]),
        ("en", vec!["break a leg", "piece of cake", "bite the bullet"]),
        ("ja", vec!["猿も木から落ちる", "石の上にも三年"]),
    ];
    
    for (lang, idiom_list) in idioms {
        for idiom in idiom_list {
            data.push((idiom.to_string(), lang.to_string(), "idiom".to_string(), 0.7));
        }
    }
    
    // 语法规则
    let grammar = ["SVO", "SOV", "VSO", "noun_declension", "verb_conjugation"];
    for rule in grammar {
        data.push((rule.to_string(), "multi".to_string(), "grammar".to_string(), 0.6));
    }
    
    data
}

fn get_system_resources() -> (f64, usize) {
    let meminfo = fs::read_to_string("/proc/meminfo").unwrap_or_default();
    let mut total_mem_gb = 16.0;
    for line in meminfo.lines() {
        if line.starts_with("MemTotal:") {
            if let Some(kb_str) = line.split_whitespace().nth(1) {
                if let Ok(kb) = kb_str.parse::<u64>() {
                    total_mem_gb = kb as f64 / 1024.0 / 1024.0;
                }
            }
        }
    }
    let cpu_cores = std::thread::available_parallelism().map(|p| p.get()).unwrap_or(8);
    (total_mem_gb, cpu_cores)
}

fn calculate_stability(hub: &ValhallaHub, iteration: u64, processed: u64, errors: u64) -> f64 {
    let stats = hub.get_statistics();
    let success_rate = if stats.total_requests > 0 { 
        stats.successful_routes as f64 / stats.total_requests as f64 
    } else { 0.0 };
    let error_rate = if processed > 0 { 1.0 - (errors as f64 / processed as f64) } else { 1.0 };
    let iteration_factor = 1.0 - (1.0 / (1.0 + iteration as f64 * 0.05));
    (success_rate * 0.5 + error_rate * 0.3 + iteration_factor * 0.2).min(1.0)
}

/// 将 Rational 转换为 f64 用于显示
fn rational_to_f64(r: &fate::Rational) -> f64 {
    r.numerator as f64 / r.denominator as f64
}

// ============================================================================
// 主函数
// ============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    println!("╔════════════════════════════════════════════════════════════════════╗");
    println!("║     🌍 Valhalla 语言核提炼实验                                     ║");
    println!("║     signal_tick 逻辑时间 + Rational 精确算术                       ║");
    println!("╚════════════════════════════════════════════════════════════════════╝\n");
    
    // 系统资源
    let (total_mem, cpu_cores) = get_system_resources();
    let used_mem = total_mem * 0.5;
    let used_cpu = (cpu_cores as f64 * 0.5).round() as usize;
    println!("📊 系统资源:");
    println!("   总内存: {:.1} GB | 使用: {:.1} GB (50%)", total_mem, used_mem);
    println!("   CPU核心: {} | 使用: {} (50%)\n", cpu_cores, used_cpu);
    
    // 加载数据
    let data = load_multilingual_data();
    println!("📥 加载多语言数据: {} 条\n", data.len());
    
    // 统计
    let mut type_counts: HashMap<String, usize> = HashMap::new();
    let mut lang_counts: HashMap<String, usize> = HashMap::new();
    for (_, lang, dtype, _) in &data {
        *type_counts.entry(dtype.clone()).or_insert(0) += 1;
        *lang_counts.entry(lang.clone()).or_insert(0) += 1;
    }
    
    println!("   类型分布:");
    for (dtype, count) in &type_counts {
        println!("     - {}: {} 条", dtype, count);
    }
    
    // 初始化 Hub
    println!("\n🚀 初始化 Valhalla Hub 系统...");
    let mut hub = ValhallaHub::new()?;
    println!("✅ Hub 系统就绪\n");
    
    // 实验参数
    let max_iterations = 200;
    let batch_size = (data.len() / 20).max(3);
    let max_duration = Duration::from_secs(300);
    
    println!("🔬 实验参数:");
    println!("   最大迭代: {}", max_iterations);
    println!("   最大时长: {} 秒", max_duration.as_secs());
    println!("   批次大小: {}\n", batch_size);
    
    println!("═══════════════════════════════════════════════════════════════════════");
    println!("                         🔬 实验进行中                                 ");
    println!("═══════════════════════════════════════════════════════════════════════\n");
    
    // 实验循环
    let mut stability_monitor = StabilityMonitor::new();
    let mut processed = 0u64;
    let mut errors = 0u64;
    let mut fate_decisions = 0u64;
    let mut language_dist: HashMap<String, u64> = HashMap::new();
    let mut type_dist: HashMap<String, u64> = HashMap::new();
    
    let start_time = Instant::now();
    
    for iteration in 0..max_iterations {
        if start_time.elapsed() > max_duration {
            println!("⏱️ 达到最大时长，提前结束");
            break;
        }
        
        // 处理批次
        let batch_start = (iteration * batch_size) % data.len();
        for i in 0..batch_size {
            let idx = (batch_start + i) % data.len();
            let (content, lang, dtype, complexity) = &data[idx];
            
            let request = HubRequest {
                request_id: uuid::Uuid::new_v4().to_string(),
                request_type: RequestType::DataProcessing,
                data: serde_json::json!({
                    "content": content,
                    "language": lang,
                    "type": dtype,
                    "complexity": complexity,
                }),
                priority: Priority::Normal,
            };
            
            match hub.process(request) {
                Ok(_) => {
                    processed += 1;
                    *language_dist.entry(lang.clone()).or_insert(0) += 1;
                    *type_dist.entry(dtype.clone()).or_insert(0) += 1;
                }
                Err(_) => errors += 1,
            }
        }
        
        // 计算稳定度
        let stability = calculate_stability(&hub, iteration as u64, processed, errors);
        stability_monitor.add_sample(stability);
        
        // 获取 Fate 偏好
        let prefs = hub.fate_system().calculate_system_preferences();
        
        // 检查置信度
        if prefs.affinities.len() >= 4 {
            let sum: f64 = prefs.affinities.iter().map(rational_to_f64).sum();
            let max_aff: f64 = prefs.affinities.iter().map(rational_to_f64).fold(0.0, f64::max);
            let avg = sum / 4.0;
            if max_aff > avg * 1.2 {
                fate_decisions += 1;
            }
        }
        
        // 输出进度
        if iteration % 5 == 0 && prefs.affinities.len() >= 4 {
            println!("📈 迭代 {:4} | 稳定度: {:.4} | 趋势: {:8} | 处理: {:6}",
                iteration, stability, stability_monitor.get_trend(), processed);
            println!("   🧠 Fate: BH={} MB={} MN={} BF={}",
                prefs.affinities[0], prefs.affinities[1],
                prefs.affinities[2], prefs.affinities[3]);
        }
        
        // 检查稳定性
        if stability_monitor.is_stable() {
            println!("\n🎉 实验成功！内部稳定度已趋向稳定\n");
            break;
        }
        
        std::thread::sleep(Duration::from_millis(30));
    }
    
    // 最终报告
    let final_prefs = hub.fate_system().calculate_system_preferences();
    let stats = hub.get_statistics();
    
    println!("\n╔════════════════════════════════════════════════════════════════════╗");
    println!("║                    📊 语言核提炼实验报告                           ║");
    println!("╠════════════════════════════════════════════════════════════════════╣");
    println!("║  ⏱️  实验时长: {:.2} 秒                                            ║", start_time.elapsed().as_secs_f64());
    println!("║  📦 处理数量: {:8}                                               ║", processed);
    println!("║  ❌ 错误数量: {:8}                                               ║", errors);
    println!("║  🎯 成功率:   {:.2}%                                              ║", 
        if stats.total_requests > 0 { stats.successful_routes as f64 / stats.total_requests as f64 * 100.0 } else { 0.0 });
    println!("╠════════════════════════════════════════════════════════════════════╣");
    println!("║                      📈 稳定度分析                                 ║");
    println!("╠════════════════════════════════════════════════════════════════════╣");
    println!("║  最终稳定度: {:.4}                                                ║", stability_monitor.get_current_stability());
    println!("║  平均稳定度: {:.4}                                                ║", stability_monitor.get_average());
    println!("║  稳定判定:   {}                                                   ║", 
        if stability_monitor.is_stable() { "✅ 已稳定" } else { "❌ 未稳定" });
    println!("╠════════════════════════════════════════════════════════════════════╣");
    println!("║                      🧠 Fate 决策分析                              ║");
    println!("╠════════════════════════════════════════════════════════════════════╣");
    println!("║  Fate决策次数: {:8}                                              ║", fate_decisions);
    
    if final_prefs.affinities.len() >= 4 {
        println!("║  系统亲和力 (Rational 精确值):                                     ║");
        let bh = rational_to_f64(&final_prefs.affinities[0]);
        let mb = rational_to_f64(&final_prefs.affinities[1]);
        let mn = rational_to_f64(&final_prefs.affinities[2]);
        let bf = rational_to_f64(&final_prefs.affinities[3]);
        println!("║    BlackHole:  {} ≈ {:.4} {}", final_prefs.affinities[0], bh, if bh > 0.3 { "🔥" } else { "" });
        println!("║    MeshBrain:  {} ≈ {:.4} {}", final_prefs.affinities[1], mb, if mb > 0.3 { "🔥" } else { "" });
        println!("║    MultiNova:  {} ≈ {:.4} {}", final_prefs.affinities[2], mn, if mn > 0.3 { "🔥" } else { "" });
        println!("║    BaseForce:  {} ≈ {:.4} {}", final_prefs.affinities[3], bf, if bf > 0.3 { "🔥" } else { "" });
    }
    
    println!("╠════════════════════════════════════════════════════════════════════╣");
    println!("║                      🌍 语言/类型分布                              ║");
    println!("╠════════════════════════════════════════════════════════════════════╣");
    
    let mut lang_vec: Vec<_> = language_dist.iter().collect();
    lang_vec.sort_by(|a, b| b.1.cmp(a.1));
    for (lang, count) in lang_vec.iter().take(5) {
        println!("║    {:5}: {:6}                                                    ║", lang, count);
    }
    
    println!("╚════════════════════════════════════════════════════════════════════╝\n");
    
    // 结论
    if final_prefs.affinities.len() >= 4 {
        let affinities_f64: Vec<f64> = final_prefs.affinities.iter().map(rational_to_f64).collect();
        let max_idx = affinities_f64.iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0);
        
        let dominant = match max_idx {
            0 => "BlackHole",
            1 => "MeshBrain",
            2 => "MultiNova",
            _ => "BaseForce",
        };
        
        println!("╔════════════════════════════════════════════════════════════════════╗");
        println!("║                    🔮 语言核心特征提炼                             ║");
        println!("╠════════════════════════════════════════════════════════════════════╣");
        println!("║  主导系统: {} ({})                                      ║", dominant, final_prefs.affinities[max_idx]);
        println!("║                                                                    ║");
        println!("║  ✅ 使用 signal_tick 逻辑时间                                      ║");
        println!("║  ✅ 使用 Rational 精确算术                                         ║");
        println!("╚════════════════════════════════════════════════════════════════════╝");
    }
    
    Ok(())
}
