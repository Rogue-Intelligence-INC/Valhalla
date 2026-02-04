//! 🌌 Valhalla 继续训练
//!
//! 从冻结状态继续，每周期加入全部资料循环10次

use hub_f64::{ValhallaHub, HubRequest, RequestType, Priority};
use std::io::{self, Write, BufRead, BufReader};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::Instant;
use serde::{Serialize, Deserialize};

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

#[derive(Serialize, Deserialize, Clone)]
struct CycleSnapshot {
    cycle: u32,
    signals: u64,
    preferences: [f64; 4],
    convergence_drift: f64,
}

#[derive(Serialize, Deserialize, Clone)]
struct TrainingStats {
    sentences: usize,
    words: usize,
    grammar: usize,
    dialogues: usize,
    images: usize,
    total: usize,
}

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

fn load_all_training_data() -> (Vec<String>, TrainingStats) {
    let base = Path::new("/run/media/jouly/My Book/valhalla_corpus/languages");
    let corpus_root = Path::new("/run/media/jouly/My Book/valhalla_corpus");
    
    let mut items: Vec<String> = Vec::new();
    let mut stats = TrainingStats {
        sentences: 0, words: 0, grammar: 0, dialogues: 0, images: 0, total: 0,
    };
    
    println!("📂 加载训练数据...");
    
    let languages = [
        "english", "chinese", "japanese", "korean", "french", 
        "german", "spanish", "russian", "arabic", "portuguese",
    ];
    
    // 句子
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
                            let sentence = if parts.len() >= 3 { parts[2].trim().to_string() } else { line.trim().to_string() };
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
    
    // 词汇
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
    
    // 语法
    print!("   语法: ");
    io::stdout().flush().unwrap();
    for lang in &languages {
        let grammar_dir = base.join(lang).join("grammar");
        if grammar_dir.exists() {
            for path in find_conllu_files(&grammar_dir) {
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
    
    // 对话
    print!("   对话: ");
    io::stdout().flush().unwrap();
    let cornell_path = corpus_root.join("dialogues/cornell movie-dialogs corpus/movie_lines.txt");
    if cornell_path.exists() {
        if let Ok(file) = File::open(&cornell_path) {
            for line in BufReader::new(file).lines().flatten() {
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
            for line in BufReader::new(file).lines().flatten() {
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
            for line in BufReader::new(file).lines().skip(1).flatten() {
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
    
    // 图片
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
                    for line in BufReader::new(file).lines().flatten() {
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
            for line in BufReader::new(file).lines().flatten() {
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
    println!("║     🌌 Valhalla 继续训练                                       ║");
    println!("║     从精炼状态继续 + 10新周期                                  ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    
    let start_time = Instant::now();
    
    // 加载之前的冻结状态
    let frozen_path = Path::new("/run/media/jouly/My Book/valhalla_hub/frozen_state.json");
    let previous_state: Option<FrozenState> = if frozen_path.exists() {
        std::fs::read_to_string(frozen_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
    } else {
        None
    };
    
    if let Some(ref state) = previous_state {
        println!("📂 加载之前的训练状态:");
        println!("   周期: {} | 信号: {}", state.cycles, state.total_signals);
        println!("   偏好: BH={:.2}% MB={:.2}% MN={:.2}% BF={:.2}%\n",
                 state.preferences[0] * 100.0, state.preferences[1] * 100.0,
                 state.preferences[2] * 100.0, state.preferences[3] * 100.0);
    }
    
    // 加载训练数据
    let (training_data, stats) = load_all_training_data();
    
    if training_data.is_empty() {
        eprintln!("❌ 没有加载到训练数据！");
        return;
    }
    
    let loops_per_cycle = 1;   // 每周期循环1次
    let new_cycles = 10;       // 新增10周期
    let start_cycle = previous_state.as_ref().map(|s| s.cycles).unwrap_or(0);
    
    println!("\n🚀 继续训练: {} 新周期，每周期 {} 次循环\n", new_cycles, loops_per_cycle);
    println!("   每周期信号: {} × {} = {}", 
             training_data.len(), loops_per_cycle, 
             training_data.len() * loops_per_cycle);
    
    // 初始化 Hub
    let mut hub = ValhallaHub::new();
    
    // 设置 CPU
    let num_cpus = num_cpus::get();
    let active_cpus = (num_cpus as f64 * 0.7).ceil() as usize;
    rayon::ThreadPoolBuilder::new().num_threads(active_cpus).build_global().ok();
    println!("   CPU: {} / {} 核心 (70%)\n", active_cpus, num_cpus);
    
    let mut cycle_history: Vec<CycleSnapshot> = previous_state.as_ref()
        .map(|s| s.cycle_history.clone())
        .unwrap_or_default();
    let mut total_signals = previous_state.as_ref().map(|s| s.total_signals).unwrap_or(0);
    
    for cycle in 1..=new_cycles {
        let actual_cycle = start_cycle + cycle;
        let cycle_start = Instant::now();
        let mut cycle_signals = 0u64;
        
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📍 周期 {}/{} (总第 {} 周期)", cycle, new_cycles, actual_cycle);
        
        // 每周期循环10次全部数据
        for loop_idx in 1..=loops_per_cycle {
            println!("   ┌─ 循环 {}/{}", loop_idx, loops_per_cycle);
            
            for (i, content) in training_data.iter().enumerate() {
                let request = HubRequest {
                    request_id: format!("{}_{}_{}", actual_cycle, loop_idx, i),
                    request_type: RequestType::SystemCoordination,
                    intensity: 1.0,
                    data: Some(serde_json::json!({ "raw": content })),
                    priority: Priority::Normal,
                };
                
                let _response = hub.process(request);
                cycle_signals += 1;
                total_signals += 1;
                
                // 每100万显示进度
                if cycle_signals % 1000000 == 0 {
                    let prefs = hub.get_fate_preferences();
                    let display: String = content.chars().take(30).collect();
                    println!("   │  [{:>10}] BH={:.1}% MB={:.1}% MN={:.1}% BF={:.1}% │ {}{}",
                           cycle_signals,
                           prefs.affinities[0] * 100.0,
                           prefs.affinities[1] * 100.0,
                           prefs.affinities[2] * 100.0,
                           prefs.affinities[3] * 100.0,
                           display.replace('\n', " "),
                           if content.len() > 30 { "..." } else { "" });
                }
            }
            
            let prefs = hub.get_fate_preferences();
            println!("   └─ 循环 {} 完成 | BH={:.2}% MB={:.2}% MN={:.2}% BF={:.2}%",
                     loop_idx,
                     prefs.affinities[0] * 100.0, prefs.affinities[1] * 100.0,
                     prefs.affinities[2] * 100.0, prefs.affinities[3] * 100.0);
        }
        
        let prefs = hub.get_fate_preferences();
        let cycle_time = cycle_start.elapsed();
        
        let drift = if cycle_history.is_empty() {
            1.0
        } else {
            let last = &cycle_history[cycle_history.len() - 1];
            (0..4).map(|i| (prefs.affinities[i] - last.preferences[i]).abs()).sum()
        };
        
        cycle_history.push(CycleSnapshot {
            cycle: actual_cycle,
            signals: cycle_signals,
            preferences: [prefs.affinities[0], prefs.affinities[1], prefs.affinities[2], prefs.affinities[3]],
            convergence_drift: drift,
        });
        
        println!("   ✅ 周期 {} 完成 | {} 信号 | {:.1}分钟 | 漂移: {:.6}",
                 actual_cycle, cycle_signals, cycle_time.as_secs_f64() / 60.0, drift);
        println!("      偏好: BH={:.2}% MB={:.2}% MN={:.2}% BF={:.2}%\n",
                 prefs.affinities[0] * 100.0, prefs.affinities[1] * 100.0,
                 prefs.affinities[2] * 100.0, prefs.affinities[3] * 100.0);
    }
    
    // 冻结
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("❄️  冻结...");
    
    let final_prefs = hub.get_fate_preferences();
    let status = hub.get_status();
    
    let frozen = FrozenState {
        cycles: start_cycle + new_cycles,
        total_signals,
        preferences: [final_prefs.affinities[0], final_prefs.affinities[1], 
                      final_prefs.affinities[2], final_prefs.affinities[3]],
        cycle_history,
        internal_digest: vec![
            final_prefs.affinities[0], final_prefs.affinities[1],
            final_prefs.affinities[2], final_prefs.affinities[3],
            status.fate_signal_tick as f64, total_signals as f64,
        ],
        frozen_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        training_stats: stats,
    };
    
    let frozen_dir = Path::new("/run/media/jouly/My Book/valhalla_hub");
    std::fs::create_dir_all(frozen_dir).ok();
    let frozen_path = frozen_dir.join("frozen_state.json");
    std::fs::write(&frozen_path, serde_json::to_string_pretty(&frozen).unwrap()).expect("无法写入");
    
    let total_time = start_time.elapsed();
    
    println!("   → {}", frozen_path.display());
    println!();
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║ ❄️  完成 | 总周期: {} | 总信号: {} | {:.1}分钟",
             frozen.cycles, total_signals, total_time.as_secs_f64() / 60.0);
    println!("║ 偏好: BH={:.2}% MB={:.2}% MN={:.2}% BF={:.2}%",
             frozen.preferences[0] * 100.0, frozen.preferences[1] * 100.0,
             frozen.preferences[2] * 100.0, frozen.preferences[3] * 100.0);
    println!("╚════════════════════════════════════════════════════════════════╝");
}
