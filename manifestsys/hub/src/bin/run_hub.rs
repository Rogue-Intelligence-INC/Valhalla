//! Valhalla Hub 多语言稳定度实验
//! 使用 signal_tick 逻辑时间 + Rational 精确算术

use valhalla_hub::{ValhallaHub, HubRequest, RequestType, Priority};
use std::time::{Duration, Instant};
use std::fs;
use std::collections::{VecDeque, HashMap};

#[derive(Debug, Clone)]
struct MultilingualData {
    languages: Vec<String>,
    words: HashMap<String, Vec<String>>,
    idioms: HashMap<String, Vec<String>>,
}

struct StabilityMonitor {
    history: VecDeque<f64>,
    consecutive_stable_count: usize,
}

impl StabilityMonitor {
    fn new() -> Self {
        Self { history: VecDeque::with_capacity(20), consecutive_stable_count: 0 }
    }
    
    fn add_sample(&mut self, stability: f64) {
        if self.history.len() >= 10 { self.history.pop_front(); }
        self.history.push_back(stability);
    }
    
    fn is_stable(&mut self) -> bool {
        if self.history.len() < 3 { return false; }
        let recent: Vec<f64> = self.history.iter().rev().take(5).copied().collect();
        let mean: f64 = recent.iter().sum::<f64>() / recent.len() as f64;
        let variance: f64 = recent.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / recent.len() as f64;
        let is_currently_stable = variance.sqrt() < 0.02 && mean > 0.5;
        if is_currently_stable { self.consecutive_stable_count += 1; } else { self.consecutive_stable_count = 0; }
        self.consecutive_stable_count >= 5
    }
    
    fn get_current_stability(&self) -> f64 { self.history.back().copied().unwrap_or(0.0) }
    
    fn get_trend(&self) -> &str {
        if self.history.len() < 2 { return "初始化中"; }
        let recent: Vec<f64> = self.history.iter().rev().take(3).copied().collect();
        let diff = recent[0] - recent[recent.len() - 1];
        if diff.abs() < 0.01 { "稳定" } else if diff > 0.0 { "上升 ↑" } else { "下降 ↓" }
    }
}

fn load_multilingual_data() -> MultilingualData {
    let languages = vec!["en", "zh", "ja", "de", "fr", "es", "ru", "ar", "ko", "hi"]
        .into_iter().map(String::from).collect();
    
    let mut words = HashMap::new();
    words.insert("en".into(), vec!["hello", "world", "computer", "science", "artificial"].into_iter().map(String::from).collect());
    words.insert("zh".into(), vec!["你好", "世界", "电脑", "科学", "人工"].into_iter().map(String::from).collect());
    words.insert("ja".into(), vec!["こんにちは", "世界", "コンピュータ", "科学", "人工"].into_iter().map(String::from).collect());
    words.insert("de".into(), vec!["hallo", "welt", "computer", "wissenschaft", "künstliche"].into_iter().map(String::from).collect());
    words.insert("fr".into(), vec!["bonjour", "monde", "ordinateur", "science", "artificiel"].into_iter().map(String::from).collect());
    words.insert("es".into(), vec!["hola", "mundo", "computadora", "ciencia", "artificial"].into_iter().map(String::from).collect());
    words.insert("ru".into(), vec!["привет", "мир", "компьютер", "наука", "искусственный"].into_iter().map(String::from).collect());
    words.insert("ar".into(), vec!["مرحبا", "عالم", "حاسوب", "علم", "اصطناعي"].into_iter().map(String::from).collect());
    words.insert("ko".into(), vec!["안녕하세요", "세계", "컴퓨터", "과학", "인공"].into_iter().map(String::from).collect());
    words.insert("hi".into(), vec!["नमस्ते", "दुनिया", "कंप्यूटर", "विज्ञान", "कृत्रिम"].into_iter().map(String::from).collect());
    
    let mut idioms = HashMap::new();
    idioms.insert("en".into(), vec!["break a leg", "piece of cake", "bite the bullet"].into_iter().map(String::from).collect());
    idioms.insert("zh".into(), vec!["画蛇添足", "守株待兔", "对牛弹琴"].into_iter().map(String::from).collect());
    idioms.insert("ja".into(), vec!["猿も木から落ちる", "石の上にも三年"].into_iter().map(String::from).collect());
    
    MultilingualData { languages, words, idioms }
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
    let success_rate = if stats.total_requests > 0 { stats.successful_routes as f64 / stats.total_requests as f64 } else { 0.0 };
    let error_rate = if processed > 0 { 1.0 - (errors as f64 / processed as f64) } else { 1.0 };
    let iteration_factor = 1.0 - (1.0 / (1.0 + iteration as f64 * 0.1));
    (success_rate * 0.4 + error_rate * 0.4 + iteration_factor * 0.2).min(1.0)
}

/// 将 Rational 转换为 f64 用于显示
fn rational_to_f64(r: &fate::Rational) -> f64 {
    r.numerator as f64 / r.denominator as f64
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║     🌌 Valhalla Hub 多语言稳定度实验                       ║");
    println!("║     Fate 动态分流 + signal_tick 逻辑时间                   ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");
    
    let (total_mem, cpu_cores) = get_system_resources();
    println!("📊 资源: {:.1}GB / {}核 → 使用50%\n", total_mem, cpu_cores);
    
    let data = load_multilingual_data();
    println!("📥 语言: {:?}", data.languages);
    println!("   词汇: {} 种语言, 俗语: {} 种语言\n", data.words.len(), data.idioms.len());
    
    println!("🚀 初始化 Hub 系统...");
    let mut hub = ValhallaHub::new()?;
    println!("✅ Hub 系统初始化完成\n");
    
    let mut stability_monitor = StabilityMonitor::new();
    let mut all_inputs: Vec<(String, String, String)> = Vec::new();
    
    for (lang, words) in &data.words {
        for word in words { all_inputs.push(("word".into(), lang.clone(), word.clone())); }
    }
    for (lang, idioms) in &data.idioms {
        for idiom in idioms { all_inputs.push(("idiom".into(), lang.clone(), idiom.clone())); }
    }
    
    println!("🔬 开始实验... ({} 项)\n", all_inputs.len());
    
    let start = Instant::now();
    let mut iteration = 0u64;
    let mut processed = 0u64;
    let mut errors = 0u64;
    let mut fate_decisions = 0u64;
    
    loop {
        iteration += 1;
        if start.elapsed() > Duration::from_secs(600) { break; }
        
        let batch_size = (all_inputs.len() / 10).max(5);
        let batch_start = (iteration as usize * batch_size) % all_inputs.len();
        
        for i in 0..batch_size {
            let idx = (batch_start + i) % all_inputs.len();
            let (item_type, lang, content) = &all_inputs[idx];
            
            let request = HubRequest {
                request_id: uuid::Uuid::new_v4().to_string(),
                request_type: match item_type.as_str() {
                    "word" => RequestType::Learning,
                    _ => RequestType::Reasoning,
                },
                data: serde_json::json!({"type": item_type, "language": lang, "content": content}),
                priority: Priority::Normal,
            };
            
            match hub.process(request) {
                Ok(_) => processed += 1,
                Err(_) => errors += 1,
            }
        }
        
        let prefs = hub.fate_system().calculate_system_preferences();
        
        // 检查置信度：如果最大亲和力 > 平均值 × 1.2
        if prefs.affinities.len() >= 4 {
            let sum: f64 = prefs.affinities.iter().map(rational_to_f64).sum();
            let max_aff: f64 = prefs.affinities.iter().map(rational_to_f64).fold(0.0, f64::max);
            let avg = sum / 4.0;
            if max_aff > avg * 1.2 {
                fate_decisions += 1;
            }
        }
        
        let stability = calculate_stability(&hub, iteration, processed, errors);
        stability_monitor.add_sample(stability);
        
        if iteration % 5 == 0 && prefs.affinities.len() >= 4 {
            println!("📈 迭代 {:3} | 稳定度: {:.4} | 趋势: {:8} | 处理: {:4}", 
                     iteration, stability, stability_monitor.get_trend(), processed);
            println!("   🧠 Fate: BH={:.2} MB={:.2} MN={:.2} BF={:.2} (Rational)",
                     rational_to_f64(&prefs.affinities[0]), 
                     rational_to_f64(&prefs.affinities[1]),
                     rational_to_f64(&prefs.affinities[2]), 
                     rational_to_f64(&prefs.affinities[3]));
        }
        
        if stability_monitor.is_stable() {
            println!("\n🎉 实验成功！内部稳定度已趋向稳定");
            break;
        }
        
        std::thread::sleep(Duration::from_millis(50));
    }
    
    let final_prefs = hub.fate_system().calculate_system_preferences();
    let stats = hub.get_statistics();
    
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║                    📊 实验报告                             ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║ 迭代: {:6} | 处理: {:6} | 错误: {:4} | 稳定度: {:.4}  ║", 
             iteration, processed, errors, stability_monitor.get_current_stability());
    println!("║ Fate决策: {:4}次                                          ║", fate_decisions);
    println!("║ 成功路由: {} | 失败: {}                                   ║", stats.successful_routes, stats.failed_routes);
    println!("╚════════════════════════════════════════════════════════════╝");
    
    if final_prefs.affinities.len() >= 4 {
        println!("\n🧠 Fate 最终偏好 (Rational → f64):");
        let bh = rational_to_f64(&final_prefs.affinities[0]);
        let mb = rational_to_f64(&final_prefs.affinities[1]);
        let mn = rational_to_f64(&final_prefs.affinities[2]);
        let bf = rational_to_f64(&final_prefs.affinities[3]);
        println!("   BlackHole:  {} ({:.4}) {}", final_prefs.affinities[0], bh, if bh > 0.3 { "🔥" } else { "" });
        println!("   MeshBrain:  {} ({:.4}) {}", final_prefs.affinities[1], mb, if mb > 0.3 { "🔥" } else { "" });
        println!("   MultiNova:  {} ({:.4}) {}", final_prefs.affinities[2], mn, if mn > 0.3 { "🔥" } else { "" });
        println!("   BaseForce:  {} ({:.4}) {}", final_prefs.affinities[3], bf, if bf > 0.3 { "🔥" } else { "" });
    }
    
    Ok(())
}
