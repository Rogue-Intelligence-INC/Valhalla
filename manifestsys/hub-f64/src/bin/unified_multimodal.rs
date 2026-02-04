//! 🌌 Valhalla 多模态统一实验
//!
//! 核心理念: 所有模态 → 统一变形信号 → Fate 处理
//!
//! 模态类型:
//! - 语言 (10种语言的句子/词汇/语法)
//! - 代码 (Rust/Python/Wave/Cosmic)
//! - 数学 (LaTeX/公式)
//! - 对话 (多轮对话)
//! - 叙事 (小说/故事)
//!
//! 关键: Fate 不知道模态类型，只处理变形信号

use hub_f64::{ValhallaHub, HubRequest, RequestType, Priority};
use std::fs::{self, File};
use std::io::{self, Write, BufRead, BufReader};
use std::path::Path;
use std::time::{Duration, Instant};
use std::thread;
use rand::seq::SliceRandom;
use rand::thread_rng;

/// 模态类型 (仅用于统计，不影响 Fate 处理)
#[derive(Debug, Clone, Copy, PartialEq)]
enum Modality {
    Language,    // 自然语言
    Code,        // 程序代码
    Math,        // 数学公式
    Dialogue,    // 对话
    Narrative,   // 叙事
}

/// 统一数据项 - 所有模态转换成相同格式
struct UnifiedItem {
    content: String,
    modality: Modality,
    /// 特征向量 (从内容提取)
    features: Vec<f64>,
}

impl UnifiedItem {
    /// 从内容提取统一特征向量 (10维)
    fn extract_features(content: &str, modality: Modality) -> Vec<f64> {
        let len = content.len() as f64;
        let chars: Vec<char> = content.chars().collect();
        let char_count = chars.len() as f64;
        
        // 基础统计特征
        let avg_word_len = if char_count > 0.0 {
            len / char_count.max(1.0)
        } else {
            0.0
        };
        
        // 字符类型分布
        let alpha_ratio = chars.iter().filter(|c| c.is_alphabetic()).count() as f64 / char_count.max(1.0);
        let digit_ratio = chars.iter().filter(|c| c.is_numeric()).count() as f64 / char_count.max(1.0);
        let space_ratio = chars.iter().filter(|c| c.is_whitespace()).count() as f64 / char_count.max(1.0);
        let punct_ratio = chars.iter().filter(|c| c.is_ascii_punctuation()).count() as f64 / char_count.max(1.0);
        
        // 特殊字符 (代码/数学相关)
        let bracket_ratio = chars.iter().filter(|c| "()[]{}".contains(**c)).count() as f64 / char_count.max(1.0);
        let operator_ratio = chars.iter().filter(|c| "+-*/=<>".contains(**c)).count() as f64 / char_count.max(1.0);
        
        // 行结构
        let line_count = content.lines().count() as f64;
        let avg_line_len = if line_count > 0.0 { len / line_count } else { len };
        
        // 熵估计 (简化)
        let unique_chars = {
            let mut seen = std::collections::HashSet::new();
            for c in chars.iter() {
                seen.insert(*c);
            }
            seen.len() as f64
        };
        let entropy_estimate = if char_count > 0.0 {
            unique_chars / char_count.sqrt()
        } else {
            0.0
        };
        
        // 模态特定调整 (微调，不是硬编码)
        let modality_bias = match modality {
            Modality::Language => 0.1,
            Modality::Code => 0.3,
            Modality::Math => 0.5,
            Modality::Dialogue => 0.2,
            Modality::Narrative => 0.15,
        };
        
        vec![
            (len / 1000.0).min(1.0),           // 长度归一化
            alpha_ratio,                        // 字母比例
            digit_ratio,                        // 数字比例
            space_ratio,                        // 空白比例
            punct_ratio,                        // 标点比例
            bracket_ratio * 10.0,               // 括号比例 (放大)
            operator_ratio * 10.0,              // 运算符比例 (放大)
            (avg_line_len / 100.0).min(1.0),   // 平均行长
            entropy_estimate.min(1.0),          // 熵估计
            modality_bias,                      // 模态偏置 (会被 Fate 学习掉)
        ]
    }
    
    fn new(content: String, modality: Modality) -> Self {
        let features = Self::extract_features(&content, modality);
        Self { content, modality, features }
    }
    
    /// 计算信号强度 (基于特征向量)
    fn intensity(&self) -> f64 {
        let sum: f64 = self.features.iter().sum();
        (sum / self.features.len() as f64).min(1.0).max(0.1)
    }
}

/// 加载所有模态数据
fn load_multimodal_corpus() -> Vec<UnifiedItem> {
    let mut items = Vec::new();
    let corpus_root = Path::new("/run/media/jouly/My Book/valhalla_corpus");
    let valhalla_root = Path::new("/home/jouly/AI/Valhalla");
    
    println!("📦 加载多模态语料...\n");
    
    // === 1. 语言模态 ===
    print!("  🗣️  语言模态...");
    io::stdout().flush().unwrap();
    let lang_path = corpus_root.join("languages");
    let languages = ["english", "chinese", "japanese", "korean", "french", 
                     "german", "spanish", "russian", "arabic", "portuguese"];
    
    for lang in &languages {
        let sent_path = lang_path.join(lang).join("sentences");
        if sent_path.exists() {
            for entry in fs::read_dir(&sent_path).into_iter().flatten().flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "txt" || e == "tsv") {
                    if let Ok(file) = File::open(&path) {
                        let reader = BufReader::new(file);
                        for line in reader.lines().take(20000).flatten() {  // 增加到 20000
                            let parts: Vec<&str> = line.split('\t').collect();
                            let sentence = if parts.len() >= 3 {
                                parts[2].trim().to_string()
                            } else {
                                line.trim().to_string()
                            };
                            if sentence.len() > 5 && sentence.len() < 500 {
                                items.push(UnifiedItem::new(sentence, Modality::Language));
                            }
                        }
                    }
                }
            }
        }
    }
    let lang_count = items.len();
    println!(" {} 条", lang_count);
    
    // === 2. 代码模态 ===
    print!("  💻 代码模态...");
    io::stdout().flush().unwrap();
    let code_start = items.len();
    
    // Valhalla 原生代码
    let code_extensions = ["cosmic", "wave", "rs", "c", "vh", "py"];
    for ext in &code_extensions {
        let pattern = valhalla_root.join("examples").join(format!("*.{}", ext));
        if let Ok(entries) = fs::read_dir(valhalla_root.join("examples")) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == *ext) {
                    if let Ok(content) = fs::read_to_string(&path) {
                        // 按函数/块分割
                        for chunk in content.split("\n\n").filter(|s| s.len() > 10) {
                            items.push(UnifiedItem::new(chunk.to_string(), Modality::Code));
                        }
                    }
                }
            }
        }
    }
    
    // Wave 语言源码
    if let Ok(entries) = fs::read_dir(valhalla_root.join("wave")) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "wave") {
                if let Ok(content) = fs::read_to_string(&path) {
                    for chunk in content.split("\n\n").filter(|s| s.len() > 10) {
                        items.push(UnifiedItem::new(chunk.to_string(), Modality::Code));
                    }
                }
            }
        }
    }
    
    // Valhalla Rust 源码样本
    let src_dirs = ["manifestsys/hub-f64/src", "systems/metasystems/fate-f64/src"];
    for src_dir in &src_dirs {
        let src_path = valhalla_root.join(src_dir);
        if src_path.exists() {
            for entry in fs::read_dir(&src_path).into_iter().flatten().flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "rs") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        // 按函数分割
                        for chunk in content.split("\n\nfn ").skip(1) {
                            let fn_code = format!("fn {}", chunk.lines().take(30).collect::<Vec<_>>().join("\n"));
                            if fn_code.len() > 20 {
                                items.push(UnifiedItem::new(fn_code, Modality::Code));
                            }
                        }
                    }
                }
            }
        }
    }
    println!(" {} 条", items.len() - code_start);
    
    // === 3. 数学模态 ===
    print!("  🔢 数学模态...");
    io::stdout().flush().unwrap();
    let math_start = items.len();
    
    // 生成一些数学表达式样本 (因为 tar.gz 需要解压)
    let math_samples = [
        "E = mc^2",
        "\\int_{0}^{\\infty} e^{-x^2} dx = \\frac{\\sqrt{\\pi}}{2}",
        "\\sum_{n=1}^{\\infty} \\frac{1}{n^2} = \\frac{\\pi^2}{6}",
        "f(x) = \\frac{1}{\\sigma\\sqrt{2\\pi}} e^{-\\frac{(x-\\mu)^2}{2\\sigma^2}}",
        "\\nabla \\times \\mathbf{E} = -\\frac{\\partial \\mathbf{B}}{\\partial t}",
        "a^2 + b^2 = c^2",
        "\\lim_{x \\to 0} \\frac{\\sin x}{x} = 1",
        "\\det(A) = \\sum_{\\sigma \\in S_n} \\text{sgn}(\\sigma) \\prod_{i=1}^{n} a_{i,\\sigma(i)}",
        "\\mathcal{L}\\{f(t)\\} = \\int_0^{\\infty} e^{-st} f(t) dt",
        "\\frac{d}{dx}[f(g(x))] = f'(g(x)) \\cdot g'(x)",
    ];
    
    for sample in &math_samples {
        items.push(UnifiedItem::new(sample.to_string(), Modality::Math));
    }
    
    // 从 Valhalla 文档中提取数学相关内容
    let docs_path = valhalla_root.join("docs");
    if docs_path.exists() {
        for entry in fs::read_dir(&docs_path).into_iter().flatten().flatten().take(10) {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "md") {
                if let Ok(content) = fs::read_to_string(&path) {
                    // 提取代码块中的数学内容
                    for line in content.lines() {
                        if line.contains("formula") || line.contains("equation") || 
                           line.contains("=") && line.contains("^") {
                            items.push(UnifiedItem::new(line.to_string(), Modality::Math));
                        }
                    }
                }
            }
        }
    }
    println!(" {} 条", items.len() - math_start);
    
    // === 4. 对话模态 ===
    print!("  💬 对话模态...");
    io::stdout().flush().unwrap();
    let dialogue_start = items.len();
    
    let dialogue_path = corpus_root.join("content/dialogues");
    if dialogue_path.exists() {
        // 加载 CSV 对话
        for entry in fs::read_dir(&dialogue_path).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "csv" || e == "txt") {
                if let Ok(file) = File::open(&path) {
                    let reader = BufReader::new(file);
                    for line in reader.lines().take(10000).flatten() {  // 增加对话数据
                        if line.len() > 10 && line.len() < 500 {
                            items.push(UnifiedItem::new(line, Modality::Dialogue));
                        }
                    }
                }
            }
        }
        
        // 递归加载子目录
        let empathetic_path = dialogue_path.join("empatheticdialogues");
        if empathetic_path.exists() {
            for entry in fs::read_dir(&empathetic_path).into_iter().flatten().flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "csv") {
                    if let Ok(file) = File::open(&path) {
                        let reader = BufReader::new(file);
                        for line in reader.lines().take(2000).flatten() {
                            let parts: Vec<&str> = line.split(',').collect();
                            if parts.len() >= 3 {
                                let utterance = parts[2..].join(",").trim_matches('"').to_string();
                                if utterance.len() > 5 && utterance.len() < 500 {
                                    items.push(UnifiedItem::new(utterance, Modality::Dialogue));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    println!(" {} 条", items.len() - dialogue_start);
    
    // === 5. 叙事模态 ===
    print!("  📖 叙事模态...");
    io::stdout().flush().unwrap();
    let narrative_start = items.len();
    
    let novels_path = corpus_root.join("content/novels");
    if novels_path.exists() {
        for entry in fs::read_dir(&novels_path).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "txt") {
                if let Ok(file) = File::open(&path) {
                    let reader = BufReader::new(file);
                    let mut paragraph = String::new();
                    for line in reader.lines().take(50000).flatten() {  // 增加叙事数据
                        if line.trim().is_empty() && !paragraph.is_empty() {
                            if paragraph.len() > 50 && paragraph.len() < 1000 {
                                items.push(UnifiedItem::new(paragraph.clone(), Modality::Narrative));
                            }
                            paragraph.clear();
                        } else {
                            paragraph.push_str(&line);
                            paragraph.push(' ');
                        }
                    }
                }
            }
        }
    }
    println!(" {} 条", items.len() - narrative_start);
    
    println!("\n📊 总计: {} 条多模态数据", items.len());
    
    items
}

/// 训练配置
struct TrainingConfig {
    cpu_limit: f64,
    total_rounds: usize,
    convergence_window: usize,
    convergence_threshold: f64,
    report_interval_secs: u64,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            cpu_limit: 0.7,  // 70% CPU
            total_rounds: 15,  // 增加到 15 轮
            convergence_window: 30,  // 更严格的收敛窗口
            convergence_threshold: 0.001,  // 更低的阈值
            report_interval_secs: 20,
        }
    }
}

/// 收敛检测
struct ConvergenceDetector {
    history: Vec<f64>,
    window: usize,
    threshold: f64,
    stable_count: usize,
}

impl ConvergenceDetector {
    fn new(window: usize, threshold: f64) -> Self {
        Self { history: Vec::new(), window, threshold, stable_count: 0 }
    }
    
    fn record(&mut self, drift: f64) -> bool {
        self.history.push(drift);
        if self.history.len() > self.window {
            self.history.remove(0);
        }
        
        if drift < self.threshold {
            self.stable_count += 1;
        } else {
            self.stable_count = 0;
        }
        
        self.stable_count >= self.window
    }
}

/// 统计
struct Stats {
    language: usize,
    code: usize,
    math: usize,
    dialogue: usize,
    narrative: usize,
}

impl Stats {
    fn new() -> Self {
        Self { language: 0, code: 0, math: 0, dialogue: 0, narrative: 0 }
    }
    
    fn record(&mut self, modality: Modality) {
        match modality {
            Modality::Language => self.language += 1,
            Modality::Code => self.code += 1,
            Modality::Math => self.math += 1,
            Modality::Dialogue => self.dialogue += 1,
            Modality::Narrative => self.narrative += 1,
        }
    }
    
    fn total(&self) -> usize {
        self.language + self.code + self.math + self.dialogue + self.narrative
    }
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║     🌌 Valhalla 多模态统一实验                                 ║");
    println!("║     所有模态 → 统一变形信号 → Fate 处理                        ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    
    let config = TrainingConfig::default();
    
    // 加载多模态数据
    let mut corpus = load_multimodal_corpus();
    
    if corpus.is_empty() {
        println!("❌ 无数据可用！");
        return;
    }
    
    // 打乱顺序 (关键: 让 Fate 看到混合的模态)
    println!("\n🔀 打乱模态顺序 (统一处理)...");
    corpus.shuffle(&mut thread_rng());
    
    // 初始化 Hub
    println!("🔧 初始化 ValhallaHub (f64)...\n");
    let mut hub = ValhallaHub::new();
    
    let mut convergence = ConvergenceDetector::new(config.convergence_window, config.convergence_threshold);
    let mut stats = Stats::new();
    let mut total_signals = 0u64;
    let start_time = Instant::now();
    let mut last_report = Instant::now();
    let mut last_prefs = hub.get_fate_preferences();
    
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║  🚀 开始多模态统一训练                                         ║");
    println!("║  CPU: {}% | 轮数: {} | 收敛阈值: {}%                           ║",
             (config.cpu_limit * 100.0) as u8, config.total_rounds, config.convergence_threshold * 100.0);
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    
    let sleep_duration = Duration::from_micros(((1.0 / config.cpu_limit - 1.0) * 100.0) as u64);
    
    for round in 1..=config.total_rounds {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📍 轮次 {}/{}", round, config.total_rounds);
        
        // 每轮重新打乱
        corpus.shuffle(&mut thread_rng());
        
        for (i, item) in corpus.iter().enumerate() {
            total_signals += 1;
            stats.record(item.modality);
            
            // 构建请求 (Fate 不知道模态类型!)
            let request = HubRequest {
                request_id: format!("unified_{}_{}", round, i),
                request_type: RequestType::Learning,
                intensity: item.intensity(),
                data: Some(serde_json::json!({
                    "features": item.features,
                    "content_hash": item.content.len(),  // 不传内容，只传特征
                })),
                priority: Priority::Normal,
            };
            
            let _response = hub.process(request);
            
            // CPU 限制
            if sleep_duration.as_micros() > 0 {
                thread::sleep(sleep_duration);
            }
            
            // 定期报告
            if last_report.elapsed().as_secs() >= config.report_interval_secs {
                let elapsed = start_time.elapsed().as_secs_f64();
                let rate = total_signals as f64 / elapsed;
                let prefs = hub.get_fate_preferences();
                
                // 计算偏好漂移
                let drift: f64 = prefs.affinities.iter()
                    .zip(last_prefs.affinities.iter())
                    .map(|(a, b)| (a - b).abs())
                    .sum::<f64>() / 4.0;
                
                let converged = convergence.record(drift);
                
                println!("\n┌─────────────────────────────────────────────────────────────┐");
                println!("│ 📊 进度: 轮次 {}/{} | 信号 {}/{} | 速率 {:.0}/s",
                         round, config.total_rounds, i + 1, corpus.len(), rate);
                println!("│ ⏱️  已用: {:.1}s | 总信号: {}", elapsed, total_signals);
                println!("├─────────────────────────────────────────────────────────────┤");
                println!("│ 🎯 Fate 偏好:");
                println!("│    BlackHole:  {:>6.2}%", prefs.affinities.get(0).unwrap_or(&0.0) * 100.0);
                println!("│    MeshBrain:  {:>6.2}%", prefs.affinities.get(1).unwrap_or(&0.0) * 100.0);
                println!("│    MultiNova:  {:>6.2}%", prefs.affinities.get(2).unwrap_or(&0.0) * 100.0);
                println!("│    BaseForce:  {:>6.2}%", prefs.affinities.get(3).unwrap_or(&0.0) * 100.0);
                println!("├─────────────────────────────────────────────────────────────┤");
                println!("│ 📈 模态统计:");
                println!("│    语言: {} | 代码: {} | 数学: {} | 对话: {} | 叙事: {}",
                         stats.language, stats.code, stats.math, stats.dialogue, stats.narrative);
                println!("│ 📉 偏好漂移: {:.4} | 稳定: {}/{}",
                         drift, convergence.stable_count, config.convergence_window);
                println!("└─────────────────────────────────────────────────────────────┘\n");
                
                last_prefs = prefs;
                last_report = Instant::now();
                
                if converged {
                    println!("\n🎉 Fate 已收敛！提前结束训练。\n");
                    break;
                }
            }
        }
        
        if convergence.stable_count >= config.convergence_window {
            break;
        }
    }
    
    // 最终报告
    let final_elapsed = start_time.elapsed().as_secs_f64();
    let final_prefs = hub.get_fate_preferences();
    
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║                 📊 多模态统一训练完成                           ║");
    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("║ ⏱️  总时间: {:.2}s ({:.1}分钟)", final_elapsed, final_elapsed / 60.0);
    println!("║ 🔢 总信号: {}", total_signals);
    println!("║ 📈 速率: {:.0} 信号/秒", total_signals as f64 / final_elapsed);
    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("║ 📊 模态分布:");
    println!("║    🗣️  语言:   {:>8} ({:.1}%)", stats.language, stats.language as f64 / stats.total() as f64 * 100.0);
    println!("║    💻 代码:   {:>8} ({:.1}%)", stats.code, stats.code as f64 / stats.total() as f64 * 100.0);
    println!("║    🔢 数学:   {:>8} ({:.1}%)", stats.math, stats.math as f64 / stats.total() as f64 * 100.0);
    println!("║    💬 对话:   {:>8} ({:.1}%)", stats.dialogue, stats.dialogue as f64 / stats.total() as f64 * 100.0);
    println!("║    📖 叙事:   {:>8} ({:.1}%)", stats.narrative, stats.narrative as f64 / stats.total() as f64 * 100.0);
    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("║ 🎯 最终 Fate 偏好:");
    println!("║    BlackHole:  {:>6.4} ({:.2}%)", 
             final_prefs.affinities.get(0).unwrap_or(&0.0),
             final_prefs.affinities.get(0).unwrap_or(&0.0) * 100.0);
    println!("║    MeshBrain:  {:>6.4} ({:.2}%)",
             final_prefs.affinities.get(1).unwrap_or(&0.0),
             final_prefs.affinities.get(1).unwrap_or(&0.0) * 100.0);
    println!("║    MultiNova:  {:>6.4} ({:.2}%)",
             final_prefs.affinities.get(2).unwrap_or(&0.0),
             final_prefs.affinities.get(2).unwrap_or(&0.0) * 100.0);
    println!("║    BaseForce:  {:>6.4} ({:.2}%)",
             final_prefs.affinities.get(3).unwrap_or(&0.0),
             final_prefs.affinities.get(3).unwrap_or(&0.0) * 100.0);
    println!("╚════════════════════════════════════════════════════════════════╝");
    
    println!("\n💡 关键洞察: Fate 不知道模态类型，只处理统一的变形信号。");
    println!("   偏好的形成完全来自数据的统计特性，而非语义标签。\n");
}
