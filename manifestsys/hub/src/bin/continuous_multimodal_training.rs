//! 🌌 Valhalla 持续多模态训练
//!
//! 功能:
//! - 50% CPU 资源限制
//! - 多模态数据摄入 (文本/代码/数学/音频/图形)
//! - Fate 自主学习
//! - 后台持续运行
//! - 定期检查点保存

use hub::{
    ValhallaHub, HubRequest, RequestType, Priority,
    DataIngestionEngine, Modality, AudioFeatures,
    FateDiscoveryEngine,
};
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};
use std::thread;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

/// 训练配置
struct TrainingConfig {
    /// CPU 使用率限制 (0.0 - 1.0)
    cpu_limit: f64,
    /// 每批次处理数量
    batch_size: usize,
    /// 检查点保存间隔 (秒)
    checkpoint_interval_secs: u64,
    /// 统计报告间隔 (秒)
    stats_interval_secs: u64,
    /// 语料根目录
    corpus_dir: String,
    /// 收敛检测窗口大小
    convergence_window: usize,
    /// 收敛阈值 (方差变化率)
    convergence_threshold: f64,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            cpu_limit: 0.5,  // 50% CPU
            batch_size: 100,
            checkpoint_interval_secs: 300,  // 5分钟
            stats_interval_secs: 60,  // 1分钟
            corpus_dir: "/home/jouly/AI/Valhalla/manifestsys/hub/data/corpus".to_string(),
            convergence_window: 50,  // 检测最近50次聚类
            convergence_threshold: 0.01,  // 方差变化 < 1% 视为收敛
        }
    }
}

/// 收敛检测器
struct ConvergenceDetector {
    variance_history: Vec<f64>,
    diversity_history: Vec<f64>,
    cluster_count_history: Vec<usize>,
    window_size: usize,
    threshold: f64,
}

impl ConvergenceDetector {
    fn new(window_size: usize, threshold: f64) -> Self {
        Self {
            variance_history: Vec::new(),
            diversity_history: Vec::new(),
            cluster_count_history: Vec::new(),
            window_size,
            threshold,
        }
    }
    
    fn record(&mut self, variance: f64, diversity: f64, cluster_count: usize) {
        self.variance_history.push(variance);
        self.diversity_history.push(diversity);
        self.cluster_count_history.push(cluster_count);
        
        // 保持窗口大小
        if self.variance_history.len() > self.window_size * 2 {
            self.variance_history.remove(0);
            self.diversity_history.remove(0);
            self.cluster_count_history.remove(0);
        }
    }
    
    fn is_converged(&self) -> bool {
        if self.variance_history.len() < self.window_size {
            return false;
        }
        
        let len = self.variance_history.len();
        let recent = &self.variance_history[len - self.window_size..];
        
        // 计算方差的标准差
        let mean: f64 = recent.iter().sum::<f64>() / recent.len() as f64;
        let variance: f64 = recent.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / recent.len() as f64;
        let std_dev = variance.sqrt();
        
        // 相对变化率
        let relative_change = if mean > 0.0 { std_dev / mean } else { 1.0 };
        
        // 聚类数稳定性检查
        let recent_clusters = &self.cluster_count_history[len - self.window_size..];
        let cluster_stable = recent_clusters.iter().all(|&c| c == recent_clusters[0]);
        
        // 收敛条件: 方差变化 < 阈值 且 聚类数稳定
        relative_change < self.threshold && cluster_stable
    }
    
    fn get_stats(&self) -> (f64, f64, usize) {
        let len = self.variance_history.len();
        if len == 0 {
            return (0.0, 0.0, 0);
        }
        (
            self.variance_history[len - 1],
            self.diversity_history[len - 1],
            self.cluster_count_history[len - 1],
        )
    }
}

/// 训练统计
struct TrainingStats {
    text_processed: AtomicU64,
    code_processed: AtomicU64,
    math_processed: AtomicU64,
    audio_processed: AtomicU64,
    visual_processed: AtomicU64,
    total_signals: AtomicU64,
    fate_discoveries: AtomicU64,
    start_time: Instant,
}

impl TrainingStats {
    fn new() -> Self {
        Self {
            text_processed: AtomicU64::new(0),
            code_processed: AtomicU64::new(0),
            math_processed: AtomicU64::new(0),
            audio_processed: AtomicU64::new(0),
            visual_processed: AtomicU64::new(0),
            total_signals: AtomicU64::new(0),
            fate_discoveries: AtomicU64::new(0),
            start_time: Instant::now(),
        }
    }
    
    fn print_report(&self) {
        let elapsed = self.start_time.elapsed().as_secs();
        let hours = elapsed / 3600;
        let minutes = (elapsed % 3600) / 60;
        let seconds = elapsed % 60;
        
        println!("\n╔════════════════════════════════════════════════════════════╗");
        println!("║             🌌 Valhalla 多模态训练统计                      ║");
        println!("╠════════════════════════════════════════════════════════════╣");
        println!("║ ⏱️  运行时间: {:02}:{:02}:{:02}                                    ║", hours, minutes, seconds);
        println!("╠════════════════════════════════════════════════════════════╣");
        println!("║ 📚 文本处理: {:>10}                                   ║", self.text_processed.load(Ordering::Relaxed));
        println!("║ 💻 代码处理: {:>10}                                   ║", self.code_processed.load(Ordering::Relaxed));
        println!("║ 🔢 数学处理: {:>10}                                   ║", self.math_processed.load(Ordering::Relaxed));
        println!("║ 🎵 音频处理: {:>10}                                   ║", self.audio_processed.load(Ordering::Relaxed));
        println!("║ 🎨 图形处理: {:>10}                                   ║", self.visual_processed.load(Ordering::Relaxed));
        println!("╠════════════════════════════════════════════════════════════╣");
        println!("║ 📊 总信号数: {:>10}                                   ║", self.total_signals.load(Ordering::Relaxed));
        println!("║ 🔍 Fate发现: {:>10}                                   ║", self.fate_discoveries.load(Ordering::Relaxed));
        println!("╚════════════════════════════════════════════════════════════╝\n");
    }
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║     🌌 Valhalla 持续多模态训练系统                         ║");
    println!("║     CPU 限制: 50%                                          ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");
    
    let config = TrainingConfig::default();
    let stats = Arc::new(TrainingStats::new());
    let running = Arc::new(AtomicBool::new(true));
    
    // Ctrl+C 处理
    let running_clone = running.clone();
    ctrlc::set_handler(move || {
        println!("\n⚠️  收到停止信号，正在优雅关闭...");
        running_clone.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");
    
    // 初始化系统
    println!("🔧 初始化 Valhalla Hub...");
    let mut hub = match ValhallaHub::new() {
        Ok(h) => h,
        Err(e) => {
            eprintln!("❌ Hub 初始化失败: {:?}", e);
            return;
        }
    };
    
    println!("🔧 初始化数据摄入引擎...");
    let mut ingestion_engine = DataIngestionEngine::new();
    
    println!("🔧 初始化 Fate 发现引擎...");
    let mut discovery_engine = FateDiscoveryEngine::new();
    
    // 加载语料
    println!("\n📦 加载训练语料...");
    let corpus = load_corpus(&config.corpus_dir);
    println!("   ✅ 加载完成: {} 个数据项\n", corpus.len());
    
    // 启动统计报告线程
    let stats_clone = stats.clone();
    let running_stats = running.clone();
    let stats_interval = config.stats_interval_secs;
    thread::spawn(move || {
        while running_stats.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_secs(stats_interval));
            if running_stats.load(Ordering::Relaxed) {
                stats_clone.print_report();
            }
        }
    });
    
    // 初始化收敛检测器
    let mut convergence_detector = ConvergenceDetector::new(
        config.convergence_window,
        config.convergence_threshold,
    );
    
    println!("🚀 开始训练循环 (Ctrl+C 停止, Fate收敛自动停止)...\n");
    
    let mut iteration = 0u64;
    let mut last_checkpoint = Instant::now();
    let mut corpus_index = 0usize;  // 轮转索引
    
    while running.load(Ordering::Relaxed) {
        iteration += 1;
        
        // 处理一批数据 - 轮转遍历所有语料
        for _ in 0..config.batch_size {
            let item = &corpus[corpus_index % corpus.len()];
            corpus_index += 1;
            if !running.load(Ordering::Relaxed) {
                break;
            }
            
            // 根据模态处理
            match &item.modality {
                CorpusModality::Text { content, language } => {
                    if let Ok(_) = ingestion_engine.ingest_text(content, language) {
                        stats.text_processed.fetch_add(1, Ordering::Relaxed);
                    }
                    discovery_engine.ingest(content);
                }
                CorpusModality::Code { content, language } => {
                    if let Ok(_) = ingestion_engine.ingest_code(content, language) {
                        stats.code_processed.fetch_add(1, Ordering::Relaxed);
                    }
                    discovery_engine.ingest(content);
                }
                CorpusModality::Math { content } => {
                    if let Ok(_) = ingestion_engine.ingest_math(content) {
                        stats.math_processed.fetch_add(1, Ordering::Relaxed);
                    }
                    discovery_engine.ingest(content);
                }
                CorpusModality::Audio { features } => {
                    if let Ok(_) = ingestion_engine.ingest_audio(features) {
                        stats.audio_processed.fetch_add(1, Ordering::Relaxed);
                    }
                }
                CorpusModality::Visual { content } => {
                    if let Ok(_) = ingestion_engine.ingest_svg(content) {
                        stats.visual_processed.fetch_add(1, Ordering::Relaxed);
                    }
                    discovery_engine.ingest(content);
                }
            }
            
            // 发送到 Hub 处理
            let request = HubRequest {
                request_id: format!("train_{}_{}", iteration, stats.total_signals.load(Ordering::Relaxed)),
                request_type: RequestType::Learning,
                data: serde_json::json!({"source": item.source.clone()}),
                priority: Priority::Normal,
            };
            
            if let Ok(_) = hub.process(request) {
                stats.total_signals.fetch_add(1, Ordering::Relaxed);
            }
            
            // CPU 限制 - 通过 sleep 控制
            // 50% CPU = 处理时间和休息时间 1:1
            thread::sleep(Duration::from_micros((100.0 / config.cpu_limit) as u64));
        }
        
        // Fate 发现处理
        if iteration % 10 == 0 {
            discovery_engine.fate_process();
            let mut clusters = discovery_engine.fate_discover_clusters();
            discovery_engine.fate_analyze_clusters(&mut clusters);
            stats.fate_discoveries.fetch_add(clusters.len() as u64, Ordering::Relaxed);
            
            // 提取聚类统计并记录到收敛检测器
            if !clusters.is_empty() {
                // 计算方差和多样性
                let cluster_sizes: Vec<f64> = clusters.iter()
                    .map(|c| c.member_indices.len() as f64)
                    .collect();
                let mean_size = cluster_sizes.iter().sum::<f64>() / cluster_sizes.len() as f64;
                let variance = cluster_sizes.iter()
                    .map(|s| (s - mean_size).powi(2))
                    .sum::<f64>() / cluster_sizes.len() as f64;
                let diversity = cluster_sizes.iter()
                    .map(|s| if *s > 0.0 { s / mean_size } else { 0.0 })
                    .sum::<f64>() / cluster_sizes.len() as f64;
                
                convergence_detector.record(variance, diversity, clusters.len());
                
                // 检查收敛
                if convergence_detector.is_converged() {
                    let (v, d, c) = convergence_detector.get_stats();
                    println!("\n╔════════════════════════════════════════════════════════════╗");
                    println!("║     🎯 Fate 检测到收敛！训练自动停止                        ║");
                    println!("╠════════════════════════════════════════════════════════════╣");
                    println!("║ 聚类数: {}                                                 ║", c);
                    println!("║ 方差:   {:.4}                                             ║", v);
                    println!("║ 多样性: {:.4}                                             ║", d);
                    println!("║ 迭代数: {}                                              ║", iteration);
                    println!("╚════════════════════════════════════════════════════════════╝\n");
                    running.store(false, Ordering::SeqCst);
                    break;
                }
            }
        }
        
        // 检查点保存
        if last_checkpoint.elapsed().as_secs() >= config.checkpoint_interval_secs {
            save_checkpoint(&stats, &ingestion_engine, iteration);
            last_checkpoint = Instant::now();
        }
    }
    
    // 最终保存
    println!("\n💾 保存最终检查点...");
    save_checkpoint(&stats, &ingestion_engine, iteration);
    
    // 最终报告
    stats.print_report();
    
    println!("✅ 训练已停止");
}

/// 语料项
struct CorpusItem {
    source: String,
    modality: CorpusModality,
}

enum CorpusModality {
    Text { content: String, language: String },
    Code { content: String, language: String },
    Math { content: String },
    Audio { features: AudioFeatures },
    Visual { content: String },
}

/// 加载语料
fn load_corpus(corpus_dir: &str) -> Vec<CorpusItem> {
    let mut items = Vec::new();
    let corpus_path = Path::new(corpus_dir);
    
    if !corpus_path.exists() {
        println!("⚠️  语料目录不存在: {}", corpus_dir);
        println!("   请先运行: bash download_all_corpora.sh");
        return items;
    }
    
    // 加载文本
    let text_dir = corpus_path.join("text");
    if text_dir.exists() {
        load_text_corpus(&text_dir, &mut items);
    }
    
    // 加载代码
    let code_dir = corpus_path.join("code");
    if code_dir.exists() {
        load_code_corpus(&code_dir, &mut items);
    }
    
    // 加载数学
    let math_dir = corpus_path.join("math");
    if math_dir.exists() {
        load_math_corpus(&math_dir, &mut items);
    }
    
    // 加载音频特征
    let audio_dir = corpus_path.join("audio");
    if audio_dir.exists() {
        load_audio_corpus(&audio_dir, &mut items);
    }
    
    // 加载图形
    let visual_dir = corpus_path.join("visual");
    if visual_dir.exists() {
        load_visual_corpus(&visual_dir, &mut items);
    }
    
    // 同时加载已有的语言数据
    let lang_dir = Path::new(corpus_dir).parent().unwrap().join("languages");
    if lang_dir.exists() {
        load_language_corpus(&lang_dir, &mut items);
    }
    
    items
}

fn load_text_corpus(dir: &Path, items: &mut Vec<CorpusItem>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(content) = fs::read_to_string(&path) {
                    let lang = path.file_stem()
                        .and_then(|s| s.to_str())
                        .map(|s| s.split('_').last().unwrap_or("en"))
                        .unwrap_or("en");
                    
                    for line in content.lines().filter(|l| !l.trim().is_empty() && !l.starts_with('#')) {
                        items.push(CorpusItem {
                            source: format!("text:{}", path.display()),
                            modality: CorpusModality::Text {
                                content: line.to_string(),
                                language: lang.to_string(),
                            },
                        });
                    }
                }
            } else if path.is_dir() {
                load_text_corpus(&path, items);
            }
        }
    }
}

fn load_code_corpus(dir: &Path, items: &mut Vec<CorpusItem>) {
    let code_langs = [
        ("rust", vec!["rs"]),
        ("python", vec!["py"]),
        ("wave", vec!["wave"]),
        ("javascript", vec!["js", "ts"]),
    ];
    
    for (lang, extensions) in &code_langs {
        let lang_dir = dir.join(lang);
        if lang_dir.exists() {
            if let Ok(entries) = fs::read_dir(&lang_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                        if extensions.contains(&ext) {
                            if let Ok(content) = fs::read_to_string(&path) {
                                // 分割成函数/类块
                                for chunk in split_code_chunks(&content) {
                                    items.push(CorpusItem {
                                        source: format!("code:{}:{}", lang, path.display()),
                                        modality: CorpusModality::Code {
                                            content: chunk,
                                            language: lang.to_string(),
                                        },
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn split_code_chunks(code: &str) -> Vec<String> {
    // 简单分割 - 按函数/类分块
    let mut chunks = Vec::new();
    let mut current_chunk = String::new();
    let mut brace_depth = 0;
    
    for line in code.lines() {
        current_chunk.push_str(line);
        current_chunk.push('\n');
        
        brace_depth += line.matches('{').count();
        brace_depth = brace_depth.saturating_sub(line.matches('}').count());
        
        // 当括号平衡且有内容时，保存块
        if brace_depth == 0 && current_chunk.trim().len() > 10 {
            chunks.push(current_chunk.clone());
            current_chunk.clear();
        }
    }
    
    if !current_chunk.trim().is_empty() {
        chunks.push(current_chunk);
    }
    
    if chunks.is_empty() {
        chunks.push(code.to_string());
    }
    
    chunks
}

fn load_math_corpus(dir: &Path, items: &mut Vec<CorpusItem>) {
    // 加载 LaTeX
    let latex_dir = dir.join("latex");
    if latex_dir.exists() {
        if let Ok(entries) = fs::read_dir(&latex_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("tex") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        // 提取公式
                        for formula in extract_latex_formulas(&content) {
                            items.push(CorpusItem {
                                source: format!("math:latex:{}", path.display()),
                                modality: CorpusModality::Math { content: formula },
                            });
                        }
                    }
                }
            }
        }
    }
    
    // 加载公式-文本对
    let pairs_path = dir.join("formulas/formula_text_pairs.json");
    if pairs_path.exists() {
        if let Ok(content) = fs::read_to_string(&pairs_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(pairs) = json.get("pairs").and_then(|p| p.as_array()) {
                    for pair in pairs {
                        if let Some(latex) = pair.get("latex").and_then(|l| l.as_str()) {
                            items.push(CorpusItem {
                                source: "math:pairs".to_string(),
                                modality: CorpusModality::Math { content: latex.to_string() },
                            });
                        }
                    }
                }
            }
        }
    }
}

fn extract_latex_formulas(content: &str) -> Vec<String> {
    let mut formulas = Vec::new();
    
    // 提取 equation 环境
    let mut in_equation = false;
    let mut current = String::new();
    
    for line in content.lines() {
        if line.contains("\\begin{equation}") || line.contains("\\begin{align}") {
            in_equation = true;
            current.clear();
        } else if line.contains("\\end{equation}") || line.contains("\\end{align}") {
            in_equation = false;
            if !current.trim().is_empty() {
                formulas.push(current.clone());
            }
        } else if in_equation {
            current.push_str(line.trim());
            current.push(' ');
        }
        
        // 行内公式 $...$
        let mut chars = line.chars().peekable();
        let mut inline = String::new();
        let mut in_inline = false;
        
        while let Some(c) = chars.next() {
            if c == '$' && chars.peek() != Some(&'$') {
                if in_inline {
                    if !inline.trim().is_empty() {
                        formulas.push(inline.clone());
                    }
                    inline.clear();
                }
                in_inline = !in_inline;
            } else if in_inline {
                inline.push(c);
            }
        }
    }
    
    formulas
}

fn load_audio_corpus(dir: &Path, items: &mut Vec<CorpusItem>) {
    let features_path = dir.join("audio_features.json");
    if features_path.exists() {
        if let Ok(content) = fs::read_to_string(&features_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(samples) = json.get("samples").and_then(|s| s.as_array()) {
                    for sample in samples {
                        if let Some(features) = sample.get("features") {
                            let audio_features = AudioFeatures {
                                duration_ms: features.get("duration_ms")
                                    .and_then(|v| v.as_f64()).unwrap_or(1000.0),
                                frequency_hz: features.get("frequency_hz")
                                    .and_then(|v| v.as_f64()).unwrap_or(440.0),
                                amplitude: features.get("amplitude")
                                    .and_then(|v| v.as_f64()).unwrap_or(0.5),
                                tempo_bpm: None,
                                pitch_class: None,
                                sample_rate: None,
                                bits_per_sample: None,
                                channels: None,
                            };
                            
                            items.push(CorpusItem {
                                source: format!("audio:{}", sample.get("id").and_then(|i| i.as_str()).unwrap_or("unknown")),
                                modality: CorpusModality::Audio { features: audio_features },
                            });
                        }
                    }
                }
            }
        }
    }
}

fn load_visual_corpus(dir: &Path, items: &mut Vec<CorpusItem>) {
    fn load_svg_recursive(dir: &Path, items: &mut Vec<CorpusItem>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("svg") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        items.push(CorpusItem {
                            source: format!("visual:{}", path.display()),
                            modality: CorpusModality::Visual { content },
                        });
                    }
                } else if path.is_dir() {
                    load_svg_recursive(&path, items);
                }
            }
        }
    }
    
    load_svg_recursive(dir, items);
}

fn load_language_corpus(dir: &Path, items: &mut Vec<CorpusItem>) {
    // 加载 Universal Dependencies
    let ud_dir = dir.join("universal_dependencies");
    if ud_dir.exists() {
        if let Ok(entries) = fs::read_dir(&ud_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("conllu") {
                    let lang = path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown");
                    
                    if let Ok(content) = fs::read_to_string(&path) {
                        // 提取句子
                        let mut sentence = String::new();
                        for line in content.lines() {
                            if line.starts_with("# text = ") {
                                sentence = line.replace("# text = ", "");
                            } else if line.is_empty() && !sentence.is_empty() {
                                items.push(CorpusItem {
                                    source: format!("ud:{}", lang),
                                    modality: CorpusModality::Text {
                                        content: sentence.clone(),
                                        language: lang.to_string(),
                                    },
                                });
                                sentence.clear();
                            }
                        }
                    }
                }
            }
        }
    }
    
    // 加载词汇表
    let vocab_path = dir.join("vocabulary/common_words.json");
    if vocab_path.exists() {
        if let Ok(content) = fs::read_to_string(&vocab_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(obj) = json.as_object() {
                    for (lang, categories) in obj {
                        if let Some(cats) = categories.as_object() {
                            for (_, words) in cats {
                                if let Some(word_list) = words.as_array() {
                                    for word in word_list {
                                        if let Some(w) = word.as_str() {
                                            items.push(CorpusItem {
                                                source: format!("vocab:{}", lang),
                                                modality: CorpusModality::Text {
                                                    content: w.to_string(),
                                                    language: lang.clone(),
                                                },
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn save_checkpoint(stats: &TrainingStats, _engine: &DataIngestionEngine, iteration: u64) {
    let checkpoint_dir = "/home/jouly/AI/Valhalla/manifestsys/hub/data/checkpoints";
    fs::create_dir_all(checkpoint_dir).ok();
    
    let checkpoint = serde_json::json!({
        "iteration": iteration,
        "timestamp": chrono::Local::now().to_rfc3339(),
        "stats": {
            "text_processed": stats.text_processed.load(Ordering::Relaxed),
            "code_processed": stats.code_processed.load(Ordering::Relaxed),
            "math_processed": stats.math_processed.load(Ordering::Relaxed),
            "audio_processed": stats.audio_processed.load(Ordering::Relaxed),
            "visual_processed": stats.visual_processed.load(Ordering::Relaxed),
            "total_signals": stats.total_signals.load(Ordering::Relaxed),
            "fate_discoveries": stats.fate_discoveries.load(Ordering::Relaxed),
            "runtime_secs": stats.start_time.elapsed().as_secs(),
        }
    });
    
    let path = format!("{}/checkpoint_{}.json", checkpoint_dir, iteration);
    if let Ok(content) = serde_json::to_string_pretty(&checkpoint) {
        fs::write(&path, content).ok();
        println!("💾 检查点已保存: {}", path);
    }
    
    // 同时保存最新检查点
    let latest_path = format!("{}/latest.json", checkpoint_dir);
    if let Ok(content) = serde_json::to_string_pretty(&checkpoint) {
        fs::write(&latest_path, content).ok();
    }
}
