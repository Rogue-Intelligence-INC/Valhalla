//! 🌌 Valhalla 训练 + 冻结
//!
//! 一次性加入所有训练资料 → 10周期收敛 → 冻结完整状态
//! 
//! 无预设：不预设 intensity, type, quality_score
//! 让 Fate 从原始数据自己学习

use hub_f64::{ValhallaHub, HubRequest, RequestType, Priority};
use std::io::{self, Write, BufRead, BufReader};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::Instant;
use serde::{Serialize, Deserialize};

/// 冻结状态结构
#[derive(Serialize, Deserialize)]
struct FrozenState {
    cycles: u32,
    total_signals: u64,
    preferences: [f64; 4],
    cycle_history: Vec<CycleSnapshot>,
    internal_digest: Vec<f64>,
    frozen_at: String,
    training_stats: TrainingStats,
}

#[derive(Serialize, Deserialize)]
struct CycleSnapshot {
    cycle: u32,
    signals: u64,
    preferences: [f64; 4],
    convergence_drift: f64,
}

#[derive(Serialize, Deserialize)]
struct TrainingStats {
    sentences: usize,
    words: usize,
    grammar: usize,
    dialogues: usize,
    images: usize,
    total: usize,
}

/// 递归查找所有 .conllu 文件
fn find_conllu_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(find_conllu_files(&path));
            } else if path.extension().map_or(false, |e| e == "conllu") {
                files.push(path);
            }
        }
    }
    files
}

/// 加载所有训练数据（纯文本，无预设）
fn load_all_training_data() -> (Vec<String>, TrainingStats) {
    let base = Path::new("/run/media/jouly/My Book/valhalla_corpus/languages");
    let corpus_root = Path::new("/run/media/jouly/My Book/valhalla_corpus");
    
    let mut items: Vec<String> = Vec::new();
    let mut stats = TrainingStats {
        sentences: 0,
        words: 0,
        grammar: 0,
        dialogues: 0,
        images: 0,
        total: 0,
    };
    
    println!("📂 加载训练数据 (无预设)...");
    
    let languages = [
        "english", "chinese", "japanese", "korean", "french", 
        "german", "spanish", "russian", "arabic", "portuguese",
    ];
    
    // 加载句子
    print!("   句子: ");
    io::stdout().flush().unwrap();
    for lang in &languages {
        let sent_dir = base.join(lang).join("sentences");
        if sent_dir.exists() {
            for entry in std::fs::read_dir(&sent_dir).into_iter().flatten().flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "txt" || e == "tsv") {
                    if let Ok(file) = File::open(&path) {
                        let reader = BufReader::new(file);
                        for line in reader.lines().flatten() {
                            let parts: Vec<&str> = line.split('\t').collect();
                            let sentence = if parts.len() >= 3 {
                                parts[2].trim().to_string()
                            } else {
                                line.trim().to_string()
                            };
                            if sentence.len() > 3 && sentence.len() < 500 {
                                items.push(sentence);
                                stats.sentences += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    println!("{}", stats.sentences);
    
    // 加载词汇
    print!("   词汇: ");
    io::stdout().flush().unwrap();
    for lang in &languages {
        let vocab_dir = base.join(lang).join("vocabulary");
        if vocab_dir.exists() {
            for entry in std::fs::read_dir(&vocab_dir).into_iter().flatten().flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "txt") {
                    if let Ok(file) = File::open(&path) {
                        let reader = BufReader::new(file);
                        for line in reader.lines().flatten() {
                            let word = line.split_whitespace().next().unwrap_or("").trim();
                            if word.len() > 1 && !word.starts_with('#') {
                                items.push(word.to_string());
                                stats.words += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    println!("{}", stats.words);
    
    // 加载语法
    print!("   语法: ");
    io::stdout().flush().unwrap();
    for lang in &languages {
        let grammar_dir = base.join(lang).join("grammar");
        if grammar_dir.exists() {
            let conllu_files = find_conllu_files(&grammar_dir);
            for path in conllu_files {
                if let Ok(file) = File::open(&path) {
                    let reader = BufReader::new(file);
                    let mut current_text = String::new();
                    
                    for line in reader.lines().flatten() {
                        if line.starts_with("# text = ") {
                            current_text = line[9..].to_string();
                        } else if line.is_empty() && !current_text.is_empty() {
                            if current_text.len() > 5 {
                                items.push(current_text.clone());
                                stats.grammar += 1;
                            }
                            current_text.clear();
                        }
                    }
                }
            }
        }
    }
    println!("{}", stats.grammar);
    
    // 加载对话
    print!("   对话: ");
    io::stdout().flush().unwrap();
    
    let cornell_path = corpus_root.join("dialogues/cornell movie-dialogs corpus/movie_lines.txt");
    if cornell_path.exists() {
        if let Ok(file) = File::open(&cornell_path) {
            let reader = BufReader::new(file);
            for line in reader.lines().flatten() {
                let parts: Vec<&str> = line.split(" +++$+++ ").collect();
                if parts.len() >= 5 {
                    let content = parts[4].trim();
                    if content.len() > 5 && content.len() < 500 {
                        items.push(content.to_string());
                        stats.dialogues += 1;
                    }
                }
            }
        }
    }
    
    let convai_path = corpus_root.join("dialogues/convai/train_both_original.txt");
    if convai_path.exists() {
        if let Ok(file) = File::open(&convai_path) {
            let reader = BufReader::new(file);
            for line in reader.lines().flatten() {
                let content = line.trim();
                if content.len() > 10 && content.len() < 500 && !content.starts_with("your persona:") {
                    items.push(content.to_string());
                    stats.dialogues += 1;
                }
            }
        }
    }
    
    let empathetic_path = corpus_root.join("dialogues/empatheticdialogues/train.csv");
    if empathetic_path.exists() {
        if let Ok(file) = File::open(&empathetic_path) {
            let reader = BufReader::new(file);
            for line in reader.lines().skip(1).flatten() {
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() >= 3 {
                    let content = parts[parts.len() - 1].trim_matches('"').trim();
                    if content.len() > 5 && content.len() < 500 {
                        items.push(content.to_string());
                        stats.dialogues += 1;
                    }
                }
            }
        }
    }
    println!("{}", stats.dialogues);
    
    // 加载图片描述
    print!("   图片: ");
    io::stdout().flush().unwrap();
    
    let coco_path = corpus_root.join("images/coco/annotations/captions_train2017.json");
    if coco_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&coco_path) {
            for line in content.lines() {
                if let Some(start) = line.find("\"caption\":") {
                    let rest = &line[start + 11..];
                    if let Some(end) = rest.find('"') {
                        let caption = &rest[..end];
                        if caption.len() > 10 && caption.len() < 500 {
                            items.push(caption.to_string());
                            stats.images += 1;
                        }
                    }
                }
            }
        }
    }
    
    let flickr_dir = corpus_root.join("images/flickr30k/Sentences");
    if flickr_dir.exists() {
        for entry in std::fs::read_dir(&flickr_dir).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "txt") {
                if let Ok(file) = File::open(&path) {
                    let reader = BufReader::new(file);
                    for line in reader.lines().flatten() {
                        let content = line.trim();
                        if content.len() > 10 && content.len() < 500 && !content.starts_with('#') {
                            items.push(content.to_string());
                            stats.images += 1;
                        }
                    }
                }
            }
        }
    }
    
    let cc_path = corpus_root.join("images/conceptual_captions/GCC-1.1.0-Validation.tsv");
    if cc_path.exists() {
        if let Ok(file) = File::open(&cc_path) {
            let reader = BufReader::new(file);
            for line in reader.lines().flatten() {
                let parts: Vec<&str> = line.split('\t').collect();
                if !parts.is_empty() {
                    let caption = parts[0].trim();
                    if caption.len() > 10 && caption.len() < 500 {
                        items.push(caption.to_string());
                        stats.images += 1;
                    }
                }
            }
        }
    }
    println!("{}", stats.images);
    
    stats.total = items.len();
    println!("   ────────────────");
    println!("   总计: {} 条", stats.total);
    
    (items, stats)
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║     🌌 Valhalla 训练 + 冻结 (无预设版)                         ║");
    println!("║     原始数据直接输入，Fate 自己决定一切                        ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    
    let start_time = Instant::now();
    
    // 1. 加载所有训练数据
    let (training_data, stats) = load_all_training_data();
    
    if training_data.is_empty() {
        eprintln!("❌ 没有加载到训练数据！");
        return;
    }
    
    println!("\n🚀 开始训练 (10周期，70% CPU，无预设)...\n");
    
    // 2. 初始化 Hub
    let mut hub = ValhallaHub::new();
    
    // 3. 训练 10 个周期
    let max_cycles = 10;
    let mut cycle_history = Vec::new();
    let mut total_signals = 0u64;
    
    // 设置 CPU 限制
    let num_cpus = num_cpus::get();
    let active_cpus = (num_cpus as f64 * 0.7).ceil() as usize;
    rayon::ThreadPoolBuilder::new()
        .num_threads(active_cpus)
        .build_global()
        .ok();
    
    println!("   CPU: {} / {} 核心 (70%)", active_cpus, num_cpus);
    println!("   数据: {} 条", training_data.len());
    println!();
    
    for cycle in 1..=max_cycles {
        let cycle_start = Instant::now();
        let mut cycle_signals = 0u64;
        
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📍 周期 {}/{}", cycle, max_cycles);
        
        // 处理所有训练数据 - 无预设，只传原始内容
        for (i, content) in training_data.iter().enumerate() {
            // 只传递原始数据，不设置 intensity/type/quality
            let request = HubRequest {
                request_id: format!("{}_{}", cycle, i),
                request_type: RequestType::SystemCoordination, // 统一类型，让 Fate 内部决定
                intensity: 1.0, // 统一强度，让数据本身决定
                data: Some(serde_json::json!({ "raw": content })),
                priority: Priority::Normal,
            };
            
            let _response = hub.process(request);
            cycle_signals += 1;
            total_signals += 1;
            
            // 进度报告 - 每1万条显示一次
            if cycle_signals % 10000 == 0 {
                let prefs = hub.get_fate_preferences();
                // 截取内容显示（最多40字符）
                let display_content: String = content.chars().take(40).collect();
                let display_content = display_content.replace('\n', " ");
                
                println!("\r   [{:>8}] BH={:.1}% MB={:.1}% MN={:.1}% BF={:.1}% │ {}{}",
                       cycle_signals,
                       prefs.affinities[0] * 100.0,
                       prefs.affinities[1] * 100.0,
                       prefs.affinities[2] * 100.0,
                       prefs.affinities[3] * 100.0,
                       display_content,
                       if content.len() > 40 { "..." } else { "" });
                io::stdout().flush().unwrap();
            }
        }
        
        let prefs = hub.get_fate_preferences();
        let cycle_time = cycle_start.elapsed();
        
        let drift = if cycle_history.is_empty() {
            1.0
        } else {
            let last: &CycleSnapshot = &cycle_history[cycle_history.len() - 1];
            let mut d = 0.0;
            for i in 0..4 {
                d += (prefs.affinities[i] - last.preferences[i]).abs();
            }
            d
        };
        
        cycle_history.push(CycleSnapshot {
            cycle,
            signals: cycle_signals,
            preferences: [
                prefs.affinities[0],
                prefs.affinities[1],
                prefs.affinities[2],
                prefs.affinities[3],
            ],
            convergence_drift: drift,
        });
        
        println!("\r   ✅ 周期 {} | {} 信号 | {:.1}s | 漂移: {:.6}",
                 cycle, cycle_signals, cycle_time.as_secs_f64(), drift);
        println!("      BH={:.2}% MB={:.2}% MN={:.2}% BF={:.2}%",
                 prefs.affinities[0] * 100.0,
                 prefs.affinities[1] * 100.0,
                 prefs.affinities[2] * 100.0,
                 prefs.affinities[3] * 100.0);
    }
    
    // 4. 冻结状态
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("❄️  冻结...");
    
    let final_prefs = hub.get_fate_preferences();
    let status = hub.get_status();
    
    let frozen = FrozenState {
        cycles: max_cycles,
        total_signals,
        preferences: [
            final_prefs.affinities[0],
            final_prefs.affinities[1],
            final_prefs.affinities[2],
            final_prefs.affinities[3],
        ],
        cycle_history,
        internal_digest: vec![
            final_prefs.affinities[0],
            final_prefs.affinities[1],
            final_prefs.affinities[2],
            final_prefs.affinities[3],
            status.fate_signal_tick as f64,
            total_signals as f64,
        ],
        frozen_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        training_stats: stats,
    };
    
    let frozen_dir = Path::new("/run/media/jouly/My Book/valhalla_hub");
    std::fs::create_dir_all(frozen_dir).ok();
    
    let frozen_path = frozen_dir.join("frozen_state.json");
    let json = serde_json::to_string_pretty(&frozen).unwrap();
    std::fs::write(&frozen_path, &json).expect("无法写入");
    
    let total_time = start_time.elapsed();
    
    println!("   → {}", frozen_path.display());
    println!();
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║ ❄️  完成 | {} 周期 | {} 信号 | {:.1}分钟",
             max_cycles, total_signals, total_time.as_secs_f64() / 60.0);
    println!("║ 偏好: BH={:.2}% MB={:.2}% MN={:.2}% BF={:.2}%",
             frozen.preferences[0] * 100.0,
             frozen.preferences[1] * 100.0,
             frozen.preferences[2] * 100.0,
             frozen.preferences[3] * 100.0);
    println!("╚════════════════════════════════════════════════════════════════╝");
}
