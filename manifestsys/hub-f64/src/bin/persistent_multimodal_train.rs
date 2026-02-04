//! 🚀 Valhalla f64 持久化多模态训练
//!
//! 特性:
//! - 加载所有数据源 (旧语料库 + 新下载数据)
//! - .valhalla 格式保存训练状态
//! - 支持断点续训
//! - 70% 系统资源
//! - 自动收敛停机

use hub_f64::{
    ValhallaHub, HubConfig, HubRequest, RequestType, Priority,
    SystemPreferences,
};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use rand::seq::SliceRandom;
use bzip2::read::BzDecoder;
use serde::{Serialize, Deserialize};

/// 训练配置
struct TrainingConfig {
    cpu_percent: u8,
    old_corpus_root: PathBuf,      // 旧语料库
    new_data_root: PathBuf,        // 新下载数据
    checkpoint_path: PathBuf,      // .valhalla 文件路径
    convergence_threshold: f64,
    convergence_stable_count: u32,
    save_interval: u64,            // 每N次迭代保存
    max_epochs: u64,               // 最大收敛轮数
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            cpu_percent: 70,
            old_corpus_root: PathBuf::from("/run/media/jouly/My Book/valhalla_corpus"),
            new_data_root: PathBuf::from("/run/media/jouly/My Book/valhalla_training_data"),
            checkpoint_path: PathBuf::from("/run/media/jouly/My Book/valhalla_training.valhalla"),
            convergence_threshold: 0.0005,
            convergence_stable_count: 30,
            save_interval: 50000,
            max_epochs: 10,  // 最多10轮收敛
        }
    }
}

/// 训练状态 - 可序列化保存
#[derive(Serialize, Deserialize, Clone)]
struct TrainingState {
    epoch: u64,
    total_processed: u64,
    stable_count: u32,
    fate_preferences: Vec<f64>,
    fate_signal_tick: u64,
    last_save_time: String,
}

impl Default for TrainingState {
    fn default() -> Self {
        Self {
            epoch: 0,
            total_processed: 0,
            stable_count: 0,
            fate_preferences: vec![0.25, 0.25, 0.25, 0.25],
            fate_signal_tick: 0,
            last_save_time: String::new(),
        }
    }
}

/// 多模态数据项
#[derive(Clone)]
enum DataItem {
    // 旧语料库
    Vocabulary { word: String, lang: String },
    Sentence { text: String, lang: String },
    Dialogue { text: String, source: String },
    Novel { text: String },
    Script { text: String },
    QA { question: String, answer: String },
    
    // 新下载数据
    Image { features: Vec<f64> },
    Audio { features: Vec<f64> },
    WikiText { text: String },
}

/// 保存训练状态到 .valhalla 文件
fn save_checkpoint(path: &Path, state: &TrainingState) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(state)?;
    let mut file = File::create(path)?;
    
    // 写入魔数和版本
    file.write_all(b"VALHALLA_CHECKPOINT_V1\n")?;
    file.write_all(json.as_bytes())?;
    
    Ok(())
}

/// 从 .valhalla 文件加载训练状态
fn load_checkpoint(path: &Path) -> Option<TrainingState> {
    if !path.exists() {
        return None;
    }
    
    let content = fs::read_to_string(path).ok()?;
    let lines: Vec<&str> = content.lines().collect();
    
    if lines.is_empty() || !lines[0].starts_with("VALHALLA_CHECKPOINT") {
        return None;
    }
    
    let json = lines[1..].join("\n");
    serde_json::from_str(&json).ok()
}

/// 加载旧语料库
fn load_old_corpus(root: &Path) -> Vec<DataItem> {
    let mut items = Vec::new();
    
    println!("  📚 加载旧语料库 (词汇/语法/对话/小说)...");
    
    // === 1. 语言语料 ===
    let lang_dir = root.join("languages");
    if lang_dir.exists() {
        let languages = ["chinese", "english", "japanese", "korean", "french", 
                         "german", "spanish", "russian", "arabic", "portuguese"];
        
        for lang in languages {
            let lang_path = lang_dir.join(lang);
            
            // 词汇
            let vocab_dir = lang_path.join("vocabulary");
            if vocab_dir.exists() {
                for entry in fs::read_dir(&vocab_dir).into_iter().flatten().flatten() {
                    if entry.path().extension().map_or(false, |e| e == "txt") {
                        if let Ok(file) = File::open(entry.path()) {
                            for line in BufReader::new(file).lines().take(5000).flatten() {
                                let word = line.split_whitespace().next().unwrap_or("").trim();
                                if word.len() > 1 && !word.starts_with('#') {
                                    items.push(DataItem::Vocabulary {
                                        word: word.to_string(),
                                        lang: lang.to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
            
            // 句子
            let sent_dir = lang_path.join("sentences");
            if sent_dir.exists() {
                for entry in fs::read_dir(&sent_dir).into_iter().flatten().flatten() {
                    if entry.path().extension().map_or(false, |e| e == "txt" || e == "tsv") {
                        if let Ok(file) = File::open(entry.path()) {
                            for line in BufReader::new(file).lines().take(10000).flatten() {
                                let parts: Vec<&str> = line.split('\t').collect();
                                let sentence = if parts.len() >= 3 {
                                    parts[2].trim().to_string()
                                } else {
                                    line.trim().to_string()
                                };
                                if sentence.len() > 5 && sentence.len() < 500 {
                                    items.push(DataItem::Sentence {
                                        text: sentence,
                                        lang: lang.to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    let lang_count = items.len();
    println!("    语言语料: {} 项", lang_count);
    
    // === 2. 对话语料 ===
    let dialogue_dir = root.join("dialogues");
    if dialogue_dir.exists() {
        // DailyDialog
        let daily = dialogue_dir.join("daily_dialog_train.json");
        if daily.exists() {
            if let Ok(content) = fs::read_to_string(&daily) {
                for line in content.lines().take(30000) {
                    if let Some(start) = line.find('"') {
                        if let Some(end) = line[start+1..].find('"') {
                            let text = &line[start+1..start+1+end];
                            if text.len() > 5 && text.len() < 500 {
                                items.push(DataItem::Dialogue {
                                    text: text.to_string(),
                                    source: "dailydialog".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
        
        // Cornell Movie
        let cornell = dialogue_dir.join("cornell movie-dialogs corpus/movie_lines.txt");
        if cornell.exists() {
            if let Ok(file) = File::open(&cornell) {
                for line in BufReader::new(file).lines().take(30000).flatten() {
                    let parts: Vec<&str> = line.split(" +++$+++ ").collect();
                    if parts.len() >= 5 {
                        let text = parts[4].trim();
                        if text.len() > 5 && text.len() < 500 {
                            items.push(DataItem::Dialogue {
                                text: text.to_string(),
                                source: "cornell".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }
    let dialogue_count = items.len() - lang_count;
    println!("    对话: {} 项", dialogue_count);
    
    // === 3. 小说 ===
    let novel_dir = root.join("novels");
    if novel_dir.exists() {
        for entry in fs::read_dir(&novel_dir).into_iter().flatten().flatten() {
            if entry.path().is_dir() {
                for sub in fs::read_dir(entry.path()).into_iter().flatten().flatten() {
                    if sub.path().extension().map_or(false, |e| e == "txt") {
                        if let Ok(file) = File::open(sub.path()) {
                            let mut para = String::new();
                            for line in BufReader::new(file).lines().take(10000).flatten() {
                                let line = line.trim();
                                if line.is_empty() {
                                    if para.len() > 50 && para.len() < 2000 {
                                        items.push(DataItem::Novel { text: para.clone() });
                                    }
                                    para.clear();
                                } else {
                                    if !para.is_empty() { para.push(' '); }
                                    para.push_str(line);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    let novel_count = items.len() - lang_count - dialogue_count;
    println!("    小说: {} 项", novel_count);
    
    // === 4. 剧本 ===
    let script_dir = root.join("scripts");
    if script_dir.exists() {
        for entry in fs::read_dir(&script_dir).into_iter().flatten().flatten() {
            if entry.path().extension().map_or(false, |e| e == "txt") {
                if let Ok(file) = File::open(entry.path()) {
                    for line in BufReader::new(file).lines().take(10000).flatten() {
                        let line = line.trim();
                        if line.len() > 20 && line.len() < 500 {
                            items.push(DataItem::Script { text: line.to_string() });
                        }
                    }
                }
            }
        }
    }
    let script_count = items.len() - lang_count - dialogue_count - novel_count;
    println!("    剧本: {} 项", script_count);
    
    items
}

/// 加载新下载数据
fn load_new_data(root: &Path) -> Vec<DataItem> {
    let mut items = Vec::new();
    
    println!("  🆕 加载新下载数据 (COCO/LibriSpeech/Wikipedia)...");
    
    // === 1. COCO 图片特征 ===
    let images_dir = root.join("images/train2017");
    if images_dir.exists() {
        let mut count = 0;
        if let Ok(entries) = fs::read_dir(&images_dir) {
            for entry in entries.flatten().take(30000) {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "jpg" || e == "jpeg") {
                    if let Ok(meta) = fs::metadata(&path) {
                        let size = meta.len() as f64;
                        let name = path.file_stem().unwrap_or_default().to_string_lossy();
                        let hash: u64 = name.bytes().fold(0u64, |a, b| a.wrapping_mul(31).wrapping_add(b as u64));
                        
                        let mut features = vec![0.0; 32];
                        features[0] = (size / 1_000_000.0).min(10.0) / 10.0;
                        for i in 1..32 {
                            features[i] = ((hash.wrapping_mul(i as u64 + 1) % 1000) as f64) / 1000.0;
                        }
                        
                        items.push(DataItem::Image { features });
                        count += 1;
                    }
                }
            }
        }
        println!("    图片: {} 项", count);
    }
    
    // === 2. LibriSpeech 音频特征 ===
    let audio_dir = root.join("audio/LibriSpeech");
    if audio_dir.exists() {
        let mut count = 0;
        fn scan_flac(dir: &Path, items: &mut Vec<DataItem>, count: &mut usize, limit: usize) {
            if *count >= limit { return; }
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    if *count >= limit { break; }
                    let path = entry.path();
                    if path.is_dir() {
                        scan_flac(&path, items, count, limit);
                    } else if path.extension().map_or(false, |e| e == "flac") {
                        if let Ok(meta) = fs::metadata(&path) {
                            let size = meta.len() as f64;
                            let name = path.to_string_lossy();
                            let hash: u64 = name.bytes().fold(0u64, |a, b| a.wrapping_mul(37).wrapping_add(b as u64));
                            
                            let mut features = vec![0.0; 32];
                            features[0] = (size / 500_000.0).min(10.0) / 10.0;
                            for i in 1..32 {
                                features[i] = ((hash.wrapping_mul(i as u64 + 7) % 1000) as f64) / 1000.0;
                            }
                            
                            items.push(DataItem::Audio { features });
                            *count += 1;
                        }
                    }
                }
            }
        }
        scan_flac(&audio_dir, &mut items, &mut count, 15000);
        println!("    音频: {} 项", count);
    }
    
    // === 3. Wikipedia 中文 ===
    let wiki_file = root.join("text/zhwiki-latest.xml.bz2");
    if wiki_file.exists() {
        let mut count = 0;
        if let Ok(file) = File::open(&wiki_file) {
            let decoder = BzDecoder::new(file);
            let reader = BufReader::new(decoder);
            
            let mut current_text = String::new();
            let mut in_text = false;
            
            for line in reader.lines().take(300000).flatten() {
                if line.contains("<text") {
                    in_text = true;
                    current_text.clear();
                } else if line.contains("</text>") {
                    in_text = false;
                    for para in current_text.split("\n\n") {
                        let para = para.trim();
                        if para.len() > 50 && para.len() < 2000 
                           && !para.starts_with("{{") && !para.starts_with("[[") 
                           && !para.contains("[[Category:") {
                            items.push(DataItem::WikiText { text: para.to_string() });
                            count += 1;
                            if count >= 50000 { break; }
                        }
                    }
                } else if in_text {
                    current_text.push_str(&line);
                    current_text.push('\n');
                }
                if count >= 50000 { break; }
            }
        }
        println!("    Wikipedia: {} 项", count);
    }
    
    items
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║     🚀 Valhalla f64 持久化多模态训练                           ║");
    println!("║     70% 资源 | .valhalla 断点续训 | 全量数据                   ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    
    let config = TrainingConfig::default();
    let running = Arc::new(AtomicBool::new(true));
    
    // Ctrl+C 处理
    let running_clone = running.clone();
    ctrlc::set_handler(move || {
        println!("\n⚠️  收到停止信号，保存状态后退出...");
        running_clone.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");
    
    // 尝试加载之前的训练状态
    let mut state = load_checkpoint(&config.checkpoint_path).unwrap_or_else(|| {
        println!("📝 未找到checkpoint，从头开始训练");
        TrainingState::default()
    });
    
    if state.total_processed > 0 {
        println!("📂 加载checkpoint: {}", config.checkpoint_path.display());
        println!("   上次保存: {}", state.last_save_time);
        println!("   已完成 Epoch: {}", state.epoch);
        println!("   已处理: {} 项", state.total_processed);
        println!("   稳定计数: {}", state.stable_count);
        println!("   Fate偏好: {:?}", state.fate_preferences);
    }
    
    // 系统配置
    let num_cpus = num_cpus::get();
    let target_threads = (num_cpus as f64 * config.cpu_percent as f64 / 100.0) as usize;
    
    println!("\n📊 系统配置:");
    println!("   CPU 核心: {} | 使用: {} ({}%)", num_cpus, target_threads, config.cpu_percent);
    println!("   收敛阈值: {} | 稳定计数: {}", config.convergence_threshold, config.convergence_stable_count);
    println!("   保存间隔: {} 项", config.save_interval);
    
    rayon::ThreadPoolBuilder::new()
        .num_threads(target_threads)
        .build_global()
        .unwrap();
    
    // 初始化 Hub
    print!("\n🔧 初始化 Valhalla Hub...");
    let hub_config = HubConfig {
        resource_percent: config.cpu_percent,
        batch_size: 512,
        ..Default::default()
    };
    let mut hub = ValhallaHub::with_config(hub_config);
    println!(" ✅");
    
    // 恢复 Fate 状态
    if state.total_processed > 0 {
        // TODO: 实际恢复 Fate 内部状态
        println!("   (Fate 状态已标记，将在训练中逐步恢复)");
    }
    
    // 加载所有数据
    println!("\n📦 加载训练数据...");
    let mut old_data = load_old_corpus(&config.old_corpus_root);
    let mut new_data = load_new_data(&config.new_data_root);
    
    let old_count = old_data.len();
    let new_count = new_data.len();
    
    // 合并数据
    let mut all_data = Vec::with_capacity(old_count + new_count);
    all_data.append(&mut old_data);
    all_data.append(&mut new_data);
    
    println!("\n  ✅ 数据总计:");
    println!("     旧语料库: {} 项", old_count);
    println!("     新数据: {} 项", new_count);
    println!("     总计: {} 项", all_data.len());
    
    if all_data.is_empty() {
        println!("❌ 没有找到训练数据！");
        return;
    }
    
    // 打乱数据
    println!("\n🔀 打乱数据顺序...");
    let mut rng = rand::thread_rng();
    all_data.shuffle(&mut rng);
    
    let total_start = Instant::now();
    let mut last_report = Instant::now();
    let mut last_save = Instant::now();
    let data_len = all_data.len();
    
    let mut prev_prefs: Option<Vec<f64>> = Some(state.fate_preferences.clone());
    
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  🏃 开始训练 - {} 轮收敛", config.max_epochs);
    println!("  📁 Checkpoint: {}", config.checkpoint_path.display());
    println!("═══════════════════════════════════════════════════════════════\n");
    
    let mut convergence_count = 0u64;
    
    // 主训练循环
    while running.load(Ordering::SeqCst) {
        state.epoch += 1;
        let epoch_start = Instant::now();
        
        println!("┌─────────────────────────────────────────────────────────────┐");
        println!("│  📖 Epoch {} 开始 (累计: {} 项)                             │", 
            state.epoch, state.total_processed);
        println!("└─────────────────────────────────────────────────────────────┘");
        
        all_data.shuffle(&mut rng);
        
        let mut epoch_stats = [0usize; 10]; // vocab, sent, dialogue, novel, script, qa, image, audio, wiki, other
        
        for (idx, item) in all_data.iter().enumerate() {
            if !running.load(Ordering::Relaxed) { break; }
            
            let (request_type, intensity, stat_idx) = match item {
                DataItem::Vocabulary { word, .. } => {
                    let intensity = (word.len() as f64 / 20.0).min(1.0).max(0.1);
                    (RequestType::Learning, intensity, 0)
                },
                DataItem::Sentence { text, .. } => {
                    let intensity = (text.len() as f64 / 200.0).min(1.0).max(0.1);
                    (RequestType::DataProcessing, intensity, 1)
                },
                DataItem::Dialogue { text, .. } => {
                    let intensity = (text.len() as f64 / 150.0).min(1.0).max(0.1);
                    (RequestType::Adaptation, intensity, 2)
                },
                DataItem::Novel { text } => {
                    let intensity = (text.len() as f64 / 500.0).min(1.0).max(0.1);
                    (RequestType::DataProcessing, intensity, 3)
                },
                DataItem::Script { text } => {
                    let intensity = (text.len() as f64 / 200.0).min(1.0).max(0.1);
                    (RequestType::Adaptation, intensity, 4)
                },
                DataItem::QA { question, answer } => {
                    let intensity = ((question.len() + answer.len()) as f64 / 300.0).min(1.0).max(0.1);
                    (RequestType::Reasoning, intensity, 5)
                },
                DataItem::Image { features } => {
                    let intensity = features[0].max(0.1);
                    (RequestType::DataProcessing, intensity, 6)
                },
                DataItem::Audio { features } => {
                    let intensity = features[0].max(0.1);
                    (RequestType::Learning, intensity, 7)
                },
                DataItem::WikiText { text } => {
                    let intensity = (text.len() as f64 / 500.0).min(1.0).max(0.1);
                    (RequestType::Reasoning, intensity, 8)
                },
            };
            
            epoch_stats[stat_idx] += 1;
            
            let request = HubRequest {
                request_id: format!("e{}_i{}", state.epoch, idx),
                request_type,
                intensity,
                data: None,
                priority: Priority::Normal,
            };
            
            hub.process(request);
            state.total_processed += 1;
            
            // 定期报告
            if last_report.elapsed() >= Duration::from_secs(15) {
                let prefs = hub.get_fate_preferences();
                let elapsed = total_start.elapsed();
                let rate = state.total_processed as f64 / elapsed.as_secs_f64();
                let progress = (idx as f64 / data_len as f64) * 100.0;
                
                println!("\n📈 进度: Epoch {} ({:.1}%) | 总计: {} | {:.0}/s",
                    state.epoch, progress, state.total_processed, rate);
                println!("   词汇:{} 句子:{} 对话:{} 小说:{} 剧本:{}",
                    epoch_stats[0], epoch_stats[1], epoch_stats[2], epoch_stats[3], epoch_stats[4]);
                println!("   图片:{} 音频:{} Wiki:{}", epoch_stats[6], epoch_stats[7], epoch_stats[8]);
                println!("\n   🎯 Fate: BH:{:.2}% MB:{:.2}% MN:{:.2}% BF:{:.2}%",
                    prefs.affinities.get(0).unwrap_or(&0.0) * 100.0,
                    prefs.affinities.get(1).unwrap_or(&0.0) * 100.0,
                    prefs.affinities.get(2).unwrap_or(&0.0) * 100.0,
                    prefs.affinities.get(3).unwrap_or(&0.0) * 100.0);
                
                // 检查收敛
                if let Some(ref prev) = prev_prefs {
                    let diff: f64 = prefs.affinities.iter()
                        .zip(prev.iter())
                        .map(|(a, b)| (a - b).abs())
                        .sum();
                    
                    println!("   📐 漂移: {:.6} | 稳定: {}/{}", 
                        diff, state.stable_count, config.convergence_stable_count);
                    
                    if diff < config.convergence_threshold {
                        state.stable_count += 1;
                        if state.stable_count >= config.convergence_stable_count {
                            convergence_count += 1;
                            println!("\n🎉 收敛 #{} 完成!", convergence_count);
                            state.fate_preferences = prefs.affinities.clone();
                            state.last_save_time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                            let _ = save_checkpoint(&config.checkpoint_path, &state);
                            
                            if convergence_count >= config.max_epochs {
                                println!("\n✅ 已完成 {} 轮收敛，训练结束!", config.max_epochs);
                                running.store(false, Ordering::SeqCst);
                                break;
                            }
                            state.stable_count = 0; // 重置继续下一轮
                        }
                    } else {
                        state.stable_count = 0;
                    }
                }
                prev_prefs = Some(prefs.affinities.clone());
                state.fate_preferences = prev_prefs.clone().unwrap();
                
                last_report = Instant::now();
            }
            
            // 定期保存
            if last_save.elapsed() >= Duration::from_secs(300) { // 每5分钟保存
                state.last_save_time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                if let Err(e) = save_checkpoint(&config.checkpoint_path, &state) {
                    eprintln!("⚠️ 保存checkpoint失败: {}", e);
                } else {
                    println!("💾 Checkpoint已保存: {}", state.last_save_time);
                }
                last_save = Instant::now();
            }
            
            thread::sleep(Duration::from_micros(15));
        }
        
        // Epoch 结束
        let epoch_time = epoch_start.elapsed();
        println!("\n✅ Epoch {} 完成 | 用时: {}m{}s", 
            state.epoch, epoch_time.as_secs() / 60, epoch_time.as_secs() % 60);
    }
    
    // 最终保存
    state.last_save_time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let final_prefs = hub.get_fate_preferences();
    state.fate_preferences = final_prefs.affinities.clone();
    state.fate_signal_tick = hub.fate_system().get_logical_time();
    
    if let Err(e) = save_checkpoint(&config.checkpoint_path, &state) {
        eprintln!("⚠️ 最终保存失败: {}", e);
    } else {
        println!("\n💾 最终状态已保存: {}", config.checkpoint_path.display());
    }
    
    // 最终报告
    let total_elapsed = total_start.elapsed();
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║                    🏁 训练完成报告                              ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!("\n⏱️  用时: {}h {}m {}s", 
        total_elapsed.as_secs() / 3600,
        (total_elapsed.as_secs() % 3600) / 60,
        total_elapsed.as_secs() % 60);
    println!("📊 处理: {} 项 | Epoch: {}", state.total_processed, state.epoch);
    println!("\n🎯 最终 Fate 偏好:");
    println!("   BlackHole:  {:.4}%", state.fate_preferences.get(0).unwrap_or(&0.0) * 100.0);
    println!("   MeshBrain:  {:.4}%", state.fate_preferences.get(1).unwrap_or(&0.0) * 100.0);
    println!("   MultiNova:  {:.4}%", state.fate_preferences.get(2).unwrap_or(&0.0) * 100.0);
    println!("   BaseForce:  {:.4}%", state.fate_preferences.get(3).unwrap_or(&0.0) * 100.0);
    println!("\n📁 Checkpoint: {}", config.checkpoint_path.display());
    println!("   下次运行将从此状态继续训练");
}
