//! 🔬 Hub 内部数据聚合分析模块
//!
//! 功能：
//! 1. 导入所有语言语料
//! 2. 追踪数据在四大系统间的流动
//! 3. 聚合分析：收敛模式、系统偏好、数据分布
//! 4. 输出详细报告

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::time::Instant;

use hub_f64::{
    ValhallaHub, HubRequest, HubConfig, RequestType, Priority,
};

/// 数据来源类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum DataSource {
    UniversalDependency,  // conllu 语言文件
    Dialogue,             // 对话文件
    Vocabulary,           // 词汇表
    Grammar,              // 语法模式
    Code,                 // 代码样本
    Math,                 // 数学公式
    Visual,               // 视觉数据
}

impl DataSource {
    fn name(&self) -> &'static str {
        match self {
            DataSource::UniversalDependency => "UD语言",
            DataSource::Dialogue => "对话",
            DataSource::Vocabulary => "词汇",
            DataSource::Grammar => "语法",
            DataSource::Code => "代码",
            DataSource::Math => "数学",
            DataSource::Visual => "视觉",
        }
    }
}

/// 信号统计
#[derive(Debug, Default, Clone)]
struct SignalStats {
    count: u64,
    sum: f64,
    sum_sq: f64,
    min: f64,
    max: f64,
}

impl SignalStats {
    fn new() -> Self {
        Self {
            count: 0,
            sum: 0.0,
            sum_sq: 0.0,
            min: f64::MAX,
            max: f64::MIN,
        }
    }

    fn update(&mut self, value: f64) {
        self.count += 1;
        self.sum += value;
        self.sum_sq += value * value;
        if value < self.min { self.min = value; }
        if value > self.max { self.max = value; }
    }

    fn mean(&self) -> f64 {
        if self.count == 0 { 0.0 } else { self.sum / self.count as f64 }
    }

    fn variance(&self) -> f64 {
        if self.count < 2 { return 0.0; }
        let mean = self.mean();
        self.sum_sq / self.count as f64 - mean * mean
    }

    fn std_dev(&self) -> f64 {
        self.variance().sqrt()
    }
}

/// 系统名称常量
const SYSTEM_NAMES: [&str; 4] = ["BlackHole", "MeshBrain", "MultiNova", "BaseForce"];
const SYSTEM_ICONS: [&str; 4] = ["🕳️", "🧠", "💫", "⚡"];

/// 聚合分析器
struct AggregationAnalyzer {
    hub: ValhallaHub,
    
    // 按来源统计
    source_stats: HashMap<DataSource, SignalStats>,
    
    // 按系统统计 (用索引 0-3 代表四系统)
    system_stats: [SignalStats; 4],
    system_routing: [u64; 4],
    
    // 时序数据 (用于收敛分析)
    preference_history: Vec<[f64; 4]>,
    routing_history: Vec<usize>,  // 0-3 索引
    
    // 聚合指标
    total_signals: u64,
    total_chars: u64,
    
    // 语言分布
    language_stats: HashMap<String, u64>,
    
    // 收敛检测
    convergence_checkpoints: Vec<(u64, String, [f64; 4])>,
}

impl AggregationAnalyzer {
    fn new() -> Self {
        let config = HubConfig {
            resource_percent: 70,
            batch_size: 1024,
            ..Default::default()
        };
        
        Self {
            hub: ValhallaHub::with_config(config),
            source_stats: HashMap::new(),
            system_stats: [SignalStats::new(), SignalStats::new(), SignalStats::new(), SignalStats::new()],
            system_routing: [0, 0, 0, 0],
            preference_history: Vec::new(),
            routing_history: Vec::new(),
            total_signals: 0,
            total_chars: 0,
            language_stats: HashMap::new(),
            convergence_checkpoints: Vec::new(),
        }
    }

    /// 处理单个信号
    fn process_signal(&mut self, value: f64, source: DataSource, lang: Option<&str>) {
        self.total_signals += 1;
        
        // 来源统计
        self.source_stats.entry(source).or_insert_with(SignalStats::new).update(value);
        
        // 语言统计
        if let Some(l) = lang {
            *self.language_stats.entry(l.to_string()).or_insert(0) += 1;
        }
        
        // 创建请求
        let request = HubRequest {
            request_id: format!("agg_{}", self.total_signals),
            request_type: classify_source(source),
            intensity: (value / 255.0).clamp(0.0, 1.0),
            data: None,
            priority: Priority::Normal,
        };
        
        // 处理并追踪路由
        let response = self.hub.process(request);
        
        // 从响应消息中提取路由目标 (索引)
        let routed_idx = if response.message.contains("BlackHole") {
            0
        } else if response.message.contains("MeshBrain") {
            1
        } else if response.message.contains("MultiNova") {
            2
        } else {
            3
        };
        
        // 系统统计
        self.system_stats[routed_idx].update(value);
        self.system_routing[routed_idx] += 1;
        self.routing_history.push(routed_idx);
        
        // 定期记录偏好历史
        if self.total_signals % 1000 == 0 {
            let prefs = self.hub.get_fate_preferences();
            let arr = [prefs.affinities[0], prefs.affinities[1], 
                       prefs.affinities[2], prefs.affinities[3]];
            self.preference_history.push(arr);
            
            // 检测收敛
            if let Some(pattern) = self.hub.get_fate_convergence_pattern() {
                self.convergence_checkpoints.push((self.total_signals, pattern.to_string(), arr));
            }
        }
    }

    /// 处理字符串 (字符级)
    fn process_text(&mut self, text: &str, source: DataSource, lang: Option<&str>) {
        for ch in text.chars() {
            let value = ch as u32 as f64;
            self.total_chars += 1;
            self.process_signal(value, source, lang);
        }
    }

    /// 生成聚合报告
    fn generate_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("╔════════════════════════════════════════════════════════════════════╗\n");
        report.push_str("║              🔬 Hub 内部数据聚合分析报告                           ║\n");
        report.push_str("╚════════════════════════════════════════════════════════════════════╝\n\n");
        
        // 1. 总体统计
        report.push_str("┌─────────────────────────────────────────────────────────────────────┐\n");
        report.push_str("│ 1. 总体统计                                                         │\n");
        report.push_str("└─────────────────────────────────────────────────────────────────────┘\n");
        report.push_str(&format!("   总信号数: {}\n", self.total_signals));
        report.push_str(&format!("   总字符数: {}\n", self.total_chars));
        report.push_str(&format!("   数据来源: {} 种\n", self.source_stats.len()));
        report.push_str(&format!("   语言种类: {} 种\n\n", self.language_stats.len()));
        
        // 2. 数据来源分布
        report.push_str("┌─────────────────────────────────────────────────────────────────────┐\n");
        report.push_str("│ 2. 数据来源分布                                                     │\n");
        report.push_str("└─────────────────────────────────────────────────────────────────────┘\n");
        let mut sources: Vec<_> = self.source_stats.iter().collect();
        sources.sort_by(|a, b| b.1.count.cmp(&a.1.count));
        for (source, stats) in &sources {
            let pct = stats.count as f64 / self.total_signals as f64 * 100.0;
            report.push_str(&format!("   {:12} │ {:>10} 信号 ({:>5.1}%) │ μ={:>8.2} σ={:>8.2}\n",
                source.name(), stats.count, pct, stats.mean(), stats.std_dev()));
        }
        report.push_str("\n");
        
        // 3. 语言分布
        report.push_str("┌─────────────────────────────────────────────────────────────────────┐\n");
        report.push_str("│ 3. 语言分布                                                         │\n");
        report.push_str("└─────────────────────────────────────────────────────────────────────┘\n");
        let mut langs: Vec<_> = self.language_stats.iter().collect();
        langs.sort_by(|a, b| b.1.cmp(a.1));
        for (lang, count) in langs.iter().take(15) {
            let c = **count;
            let pct = c as f64 / self.total_signals as f64 * 100.0;
            let bar_len = (pct / 2.0) as usize;
            let bar: String = "█".repeat(bar_len.min(30));
            report.push_str(&format!("   {:6} │ {:>10} │ {:>5.1}% │ {}\n", lang, c, pct, bar));
        }
        if langs.len() > 15 {
            report.push_str(&format!("   ... 还有 {} 种语言\n", langs.len() - 15));
        }
        report.push_str("\n");
        
        // 4. 系统路由分布
        report.push_str("┌─────────────────────────────────────────────────────────────────────┐\n");
        report.push_str("│ 4. 四系统路由分布                                                   │\n");
        report.push_str("└─────────────────────────────────────────────────────────────────────┘\n");
        for i in 0..4 {
            let count = self.system_routing[i];
            let pct = count as f64 / self.total_signals.max(1) as f64 * 100.0;
            let bar_len = (pct / 2.0) as usize;
            let bar: String = "█".repeat(bar_len.min(40));
            report.push_str(&format!("   {} {:10} │ {:>10} │ {:>5.1}% │ {}\n", 
                SYSTEM_ICONS[i], SYSTEM_NAMES[i], count, pct, bar));
            
            let stats = &self.system_stats[i];
            if stats.count > 0 {
                report.push_str(&format!("                  │ μ={:>8.2} σ={:>8.2} │ range=[{:.0}, {:.0}]\n",
                    stats.mean(), stats.std_dev(), stats.min, stats.max));
            }
        }
        report.push_str("\n");
        
        // 5. Fate 偏好演化
        report.push_str("┌─────────────────────────────────────────────────────────────────────┐\n");
        report.push_str("│ 5. Fate 偏好演化轨迹                                                │\n");
        report.push_str("└─────────────────────────────────────────────────────────────────────┘\n");
        if !self.preference_history.is_empty() {
            let checkpoints = [0, 
                self.preference_history.len() / 4,
                self.preference_history.len() / 2,
                self.preference_history.len() * 3 / 4,
                self.preference_history.len() - 1];
            
            report.push_str("   时间点    │ BlackHole │ MeshBrain │ MultiNova │ BaseForce │ 主导系统\n");
            report.push_str("   ──────────┼───────────┼───────────┼───────────┼───────────┼──────────\n");
            
            for &idx in &checkpoints {
                if idx < self.preference_history.len() {
                    let prefs = &self.preference_history[idx];
                    let signal_at = (idx + 1) * 1000;
                    let dominant_idx = prefs.iter()
                        .enumerate()
                        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    let dominant = ["BlackHole", "MeshBrain", "MultiNova", "BaseForce"][dominant_idx];
                    
                    report.push_str(&format!("   {:>8}K │ {:>8.2}% │ {:>8.2}% │ {:>8.2}% │ {:>8.2}% │ {}\n",
                        signal_at / 1000,
                        prefs[0] * 100.0, prefs[1] * 100.0, prefs[2] * 100.0, prefs[3] * 100.0,
                        dominant));
                }
            }
            
            // 偏好变化量
            if self.preference_history.len() >= 2 {
                let first = &self.preference_history[0];
                let last = &self.preference_history[self.preference_history.len() - 1];
                let total_change: f64 = first.iter().zip(last.iter())
                    .map(|(a, b)| (a - b).abs())
                    .sum();
                report.push_str(&format!("\n   偏好总变化量: {:.4}\n", total_change));
                
                // 判断稳定性
                let stability = if total_change < 0.01 {
                    "非常稳定 ✓"
                } else if total_change < 0.05 {
                    "相对稳定"
                } else if total_change < 0.1 {
                    "有波动"
                } else {
                    "持续变化"
                };
                report.push_str(&format!("   稳定性评估: {}\n", stability));
            }
        }
        report.push_str("\n");
        
        // 6. 收敛检测
        report.push_str("┌─────────────────────────────────────────────────────────────────────┐\n");
        report.push_str("│ 6. Fate 收敛检测                                                    │\n");
        report.push_str("└─────────────────────────────────────────────────────────────────────┘\n");
        if self.convergence_checkpoints.is_empty() {
            report.push_str("   未检测到收敛模式 (仍在探索中)\n");
        } else {
            for (signal_count, pattern, prefs) in &self.convergence_checkpoints {
                report.push_str(&format!("   @{:>8} 信号: 模式=\"{}\" │ 偏好=[{:.2}, {:.2}, {:.2}, {:.2}]\n",
                    signal_count, pattern, prefs[0], prefs[1], prefs[2], prefs[3]));
            }
        }
        
        // 最终状态
        let final_prefs = self.hub.get_fate_preferences();
        let final_pattern = self.hub.get_fate_convergence_pattern();
        report.push_str(&format!("\n   最终状态:\n"));
        report.push_str(&format!("   - 收敛模式: {:?}\n", final_pattern));
        report.push_str(&format!("   - 最终偏好: BH={:.2}% MB={:.2}% MN={:.2}% BF={:.2}%\n",
            final_prefs.affinities[0] * 100.0,
            final_prefs.affinities[1] * 100.0,
            final_prefs.affinities[2] * 100.0,
            final_prefs.affinities[3] * 100.0));
        report.push_str("\n");
        
        // 7. 聚合洞察
        report.push_str("┌─────────────────────────────────────────────────────────────────────┐\n");
        report.push_str("│ 7. 聚合洞察                                                         │\n");
        report.push_str("└─────────────────────────────────────────────────────────────────────┘\n");
        
        // 路由模式分析
        let mut routing_runs: Vec<(usize, usize)> = Vec::new();  // (系统索引, 连续长度)
        let mut current_run: Option<(usize, usize)> = None;
        
        for &sys_idx in &self.routing_history {
            match &mut current_run {
                Some((last_idx, count)) if *last_idx == sys_idx => {
                    *count += 1;
                }
                _ => {
                    if let Some(run) = current_run.take() {
                        routing_runs.push(run);
                    }
                    current_run = Some((sys_idx, 1));
                }
            }
        }
        if let Some(run) = current_run {
            routing_runs.push(run);
        }
        
        let avg_run_length: f64 = if routing_runs.is_empty() {
            0.0
        } else {
            routing_runs.iter().map(|(_, len)| *len as f64).sum::<f64>() / routing_runs.len() as f64
        };
        
        report.push_str(&format!("   路由切换次数: {}\n", routing_runs.len().saturating_sub(1)));
        report.push_str(&format!("   平均连续路由长度: {:.1}\n", avg_run_length));
        
        // 数据熵估算
        let mut total_variance = 0.0;
        for (_, stats) in &self.source_stats {
            total_variance += stats.variance();
        }
        let avg_variance = total_variance / self.source_stats.len().max(1) as f64;
        report.push_str(&format!("   数据方差(平均): {:.2}\n", avg_variance));
        
        // 主导来源
        if let Some((dominant_source, _)) = sources.first() {
            report.push_str(&format!("   主导数据来源: {}\n", dominant_source.name()));
        }
        
        // 主导语言
        if let Some((dominant_lang, _)) = langs.first() {
            report.push_str(&format!("   主导语言: {}\n", dominant_lang));
        }
        
        report.push_str("\n");
        report.push_str("════════════════════════════════════════════════════════════════════════\n");
        
        report
    }
}

fn classify_source(source: DataSource) -> RequestType {
    match source {
        DataSource::UniversalDependency => RequestType::DataProcessing,
        DataSource::Dialogue => RequestType::SystemCoordination,
        DataSource::Vocabulary => RequestType::Learning,
        DataSource::Grammar => RequestType::Reasoning,
        DataSource::Code => RequestType::DataProcessing,
        DataSource::Math => RequestType::Reasoning,
        DataSource::Visual => RequestType::Adaptation,
    }
}

/// 加载 Universal Dependencies 语料
fn load_ud_corpus(analyzer: &mut AggregationAnalyzer) {
    let ud_path = Path::new("/home/jouly/AI/Valhalla/manifestsys/hub/data/languages/universal_dependencies");
    
    if !ud_path.exists() {
        println!("   ⚠️  UD 语料路径不存在");
        return;
    }
    
    let lang_map: HashMap<&str, &str> = [
        ("ar", "Arabic"), ("de", "German"), ("en", "English"),
        ("es", "Spanish"), ("fr", "French"), ("hi", "Hindi"),
        ("it", "Italian"), ("ja", "Japanese"), ("ko", "Korean"),
        ("nl", "Dutch"), ("pl", "Polish"), ("pt", "Portuguese"),
        ("ru", "Russian"), ("tr", "Turkish"), ("zh", "Chinese"),
    ].into_iter().collect();
    
    if let Ok(entries) = fs::read_dir(ud_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "conllu") {
                let lang_code = path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown");
                let lang_name = lang_map.get(lang_code).unwrap_or(&lang_code);
                
                print!("      {} ({})... ", lang_name, lang_code);
                std::io::Write::flush(&mut std::io::stdout()).ok();
                
                if let Ok(file) = File::open(&path) {
                    let reader = BufReader::new(file);
                    let mut line_count = 0;
                    
                    for line in reader.lines().flatten() {
                        // conllu 格式: 忽略注释行和空行
                        if !line.starts_with('#') && !line.trim().is_empty() {
                            let parts: Vec<&str> = line.split('\t').collect();
                            if parts.len() >= 2 {
                                // 第二列是词形
                                analyzer.process_text(parts[1], DataSource::UniversalDependency, Some(lang_name));
                                line_count += 1;
                            }
                        }
                    }
                    println!("{} 词", line_count);
                }
            }
        }
    }
}

/// 加载对话语料
fn load_dialogue_corpus(analyzer: &mut AggregationAnalyzer) {
    let dialogue_files = [
        "/home/jouly/AI/Valhalla/valhalla_all_dialogues.txt",
        "/home/jouly/AI/Valhalla/valhalla_enhanced_dialogues.txt",
        "/home/jouly/AI/Valhalla/valhalla_afternoon_dialogues.txt",
    ];
    
    for path_str in dialogue_files {
        let path = Path::new(path_str);
        if path.exists() {
            print!("      {}... ", path.file_name().unwrap_or_default().to_string_lossy());
            std::io::Write::flush(&mut std::io::stdout()).ok();
            
            if let Ok(content) = fs::read_to_string(path) {
                let mut line_count = 0;
                for line in content.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() && trimmed.len() > 3 {
                        // 检测语言
                        let lang = detect_language(trimmed);
                        analyzer.process_text(trimmed, DataSource::Dialogue, Some(lang));
                        line_count += 1;
                    }
                }
                println!("{} 行", line_count);
            }
        }
    }
}

/// 加载词汇表
fn load_vocabulary(analyzer: &mut AggregationAnalyzer) {
    let vocab_path = Path::new("/home/jouly/AI/Valhalla/manifestsys/hub/data/languages/vocabulary/common_words.json");
    
    if vocab_path.exists() {
        print!("      common_words.json... ");
        std::io::Write::flush(&mut std::io::stdout()).ok();
        
        if let Ok(content) = fs::read_to_string(vocab_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                let mut count = 0;
                if let Some(obj) = json.as_object() {
                    for (lang, words) in obj {
                        if let Some(arr) = words.as_array() {
                            for word in arr {
                                if let Some(w) = word.as_str() {
                                    analyzer.process_text(w, DataSource::Vocabulary, Some(lang));
                                    count += 1;
                                }
                            }
                        }
                    }
                }
                println!("{} 词", count);
            }
        }
    }
}

/// 加载语法模式
fn load_grammar(analyzer: &mut AggregationAnalyzer) {
    let grammar_path = Path::new("/home/jouly/AI/Valhalla/manifestsys/hub/data/languages/grammar/grammar_patterns.json");
    
    if grammar_path.exists() {
        print!("      grammar_patterns.json... ");
        std::io::Write::flush(&mut std::io::stdout()).ok();
        
        if let Ok(content) = fs::read_to_string(grammar_path) {
            let mut count = 0;
            // 简单处理: 将整个文件作为文本处理
            for line in content.lines() {
                if !line.trim().is_empty() {
                    analyzer.process_text(line, DataSource::Grammar, None);
                    count += 1;
                }
            }
            println!("{} 行", count);
        }
    }
}

/// 加载代码样本
fn load_code_samples(analyzer: &mut AggregationAnalyzer) {
    let code_paths = [
        ("/home/jouly/AI/Valhalla/manifestsys/hub/data/corpus/code/python/samples.py", "Python"),
        ("/home/jouly/AI/Valhalla/manifestsys/hub/data/corpus/code/rust/samples.rs", "Rust"),
        ("/home/jouly/AI/Valhalla/manifestsys/hub/data/corpus/code/javascript/samples.js", "JavaScript"),
    ];
    
    for (path_str, lang) in code_paths {
        let path = Path::new(path_str);
        if path.exists() {
            print!("      {}... ", lang);
            std::io::Write::flush(&mut std::io::stdout()).ok();
            
            if let Ok(content) = fs::read_to_string(path) {
                let char_count = content.chars().count();
                analyzer.process_text(&content, DataSource::Code, Some(lang));
                println!("{} 字符", char_count);
            }
        }
    }
}

/// 加载数学公式
fn load_math(analyzer: &mut AggregationAnalyzer) {
    let math_path = Path::new("/home/jouly/AI/Valhalla/manifestsys/hub/data/corpus/math/latex/formulas.tex");
    
    if math_path.exists() {
        print!("      formulas.tex... ");
        std::io::Write::flush(&mut std::io::stdout()).ok();
        
        if let Ok(content) = fs::read_to_string(math_path) {
            let char_count = content.chars().count();
            analyzer.process_text(&content, DataSource::Math, Some("LaTeX"));
            println!("{} 字符", char_count);
        }
    }
}

/// 简单语言检测
fn detect_language(text: &str) -> &'static str {
    let has_chinese = text.chars().any(|c| c >= '\u{4e00}' && c <= '\u{9fff}');
    let has_japanese = text.chars().any(|c| (c >= '\u{3040}' && c <= '\u{309f}') || (c >= '\u{30a0}' && c <= '\u{30ff}'));
    let has_korean = text.chars().any(|c| c >= '\u{ac00}' && c <= '\u{d7a3}');
    let has_arabic = text.chars().any(|c| c >= '\u{0600}' && c <= '\u{06ff}');
    let has_cyrillic = text.chars().any(|c| c >= '\u{0400}' && c <= '\u{04ff}');
    
    if has_chinese { return "Chinese"; }
    if has_japanese { return "Japanese"; }
    if has_korean { return "Korean"; }
    if has_arabic { return "Arabic"; }
    if has_cyrillic { return "Russian"; }
    "English"
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════════════╗");
    println!("║         🔬 Hub 内部数据聚合分析模块                                ║");
    println!("║         导入所有语料 → 四系统处理 → 聚合分析                       ║");
    println!("╚════════════════════════════════════════════════════════════════════╝\n");
    
    let start = Instant::now();
    let mut analyzer = AggregationAnalyzer::new();
    
    // 阶段 1: 加载所有语料
    println!("📂 阶段 1: 加载语料数据\n");
    
    println!("   🌍 Universal Dependencies 语料:");
    load_ud_corpus(&mut analyzer);
    
    println!("\n   💬 对话语料:");
    load_dialogue_corpus(&mut analyzer);
    
    println!("\n   📚 词汇表:");
    load_vocabulary(&mut analyzer);
    
    println!("\n   📖 语法模式:");
    load_grammar(&mut analyzer);
    
    println!("\n   💻 代码样本:");
    load_code_samples(&mut analyzer);
    
    println!("\n   ➗ 数学公式:");
    load_math(&mut analyzer);
    
    let load_time = start.elapsed();
    println!("\n   ✓ 数据加载完成: {:.2}s\n", load_time.as_secs_f64());
    
    // 阶段 2: 生成报告
    println!("📊 阶段 2: 生成聚合分析报告\n");
    
    let report = analyzer.generate_report();
    println!("{}", report);
    
    // 保存报告
    let report_path = "/home/jouly/AI/Valhalla/HUB_AGGREGATION_ANALYSIS_REPORT.md";
    if fs::write(report_path, &report).is_ok() {
        println!("📄 报告已保存: {}\n", report_path);
    }
    
    let total_time = start.elapsed();
    println!("⏱️  总耗时: {:.2}s", total_time.as_secs_f64());
}
