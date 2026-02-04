//! 🎨 Valhalla 可视化训练监控
//!
//! 实时显示:
//! - 数据聚合形式 (柱状图)
//! - 训练周期进度 (进度条)
//! - Fate 系统偏好 (动态图)
//! - 资源使用状态

use hub_f64::{
    ValhallaHub, HubConfig, HubRequest, RequestType, Priority,
};
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Write, stdout};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use rand::seq::SliceRandom;
use bzip2::read::BzDecoder;
use serde::{Serialize, Deserialize};

// ═══════════════════════════════════════════════════════════════════
//  ANSI 颜色和样式
// ═══════════════════════════════════════════════════════════════════

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";

// 前景色
const FG_RED: &str = "\x1b[91m";
const FG_GREEN: &str = "\x1b[92m";
const FG_YELLOW: &str = "\x1b[93m";
const FG_BLUE: &str = "\x1b[94m";
const FG_MAGENTA: &str = "\x1b[95m";
const FG_CYAN: &str = "\x1b[96m";
const FG_WHITE: &str = "\x1b[97m";
const FG_ORANGE: &str = "\x1b[38;5;208m";
const FG_PINK: &str = "\x1b[38;5;213m";
const FG_LIME: &str = "\x1b[38;5;118m";

// 背景色
const BG_BLACK: &str = "\x1b[40m";
const BG_BLUE: &str = "\x1b[44m";
const BG_CYAN: &str = "\x1b[46m";
const BG_MAGENTA: &str = "\x1b[45m";
const BG_GREEN: &str = "\x1b[42m";

// 光标控制
const CLEAR_SCREEN: &str = "\x1b[2J";
const CURSOR_HOME: &str = "\x1b[H";
const HIDE_CURSOR: &str = "\x1b[?25l";
const SHOW_CURSOR: &str = "\x1b[?25h";

// ═══════════════════════════════════════════════════════════════════
//  数据结构
// ═══════════════════════════════════════════════════════════════════

struct TrainingConfig {
    cpu_percent: u8,
    old_corpus_root: PathBuf,
    new_data_root: PathBuf,
    checkpoint_path: PathBuf,
    convergence_threshold: f64,
    convergence_stable_count: u32,
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
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct TrainingState {
    epoch: u64,
    total_processed: u64,
    stable_count: u32,
    fate_preferences: Vec<f64>,
    fate_signal_tick: u64,
    last_save_time: String,
}

#[derive(Clone)]
enum DataItem {
    Vocabulary { word: String },
    Sentence { text: String },
    Dialogue { text: String },
    Novel { text: String },
    Script { text: String },
    Image { features: Vec<f64> },
    Audio { features: Vec<f64> },
    WikiText { text: String },
}

struct DataStats {
    vocab: usize,
    sentence: usize,
    dialogue: usize,
    novel: usize,
    script: usize,
    image: usize,
    audio: usize,
    wiki: usize,
}

impl DataStats {
    fn new() -> Self {
        Self { vocab: 0, sentence: 0, dialogue: 0, novel: 0, script: 0, image: 0, audio: 0, wiki: 0 }
    }
    
    fn total(&self) -> usize {
        self.vocab + self.sentence + self.dialogue + self.novel + 
        self.script + self.image + self.audio + self.wiki
    }
}

// ═══════════════════════════════════════════════════════════════════
//  可视化绘图函数
// ═══════════════════════════════════════════════════════════════════

fn draw_header() {
    println!("{}{}╔══════════════════════════════════════════════════════════════════════════════╗{}", BOLD, FG_CYAN, RESET);
    println!("{}{}║{}  🚀 {}VALHALLA f64 可视化训练监控{}                                             {}║{}", BOLD, FG_CYAN, RESET, FG_YELLOW, RESET, FG_CYAN, RESET);
    println!("{}{}║{}  {}70% CPU | .valhalla 断点续训 | 实时监控 | 按 Ctrl+C 安全退出{}              {}║{}", BOLD, FG_CYAN, RESET, DIM, RESET, FG_CYAN, RESET);
    println!("{}{}╚══════════════════════════════════════════════════════════════════════════════╝{}", BOLD, FG_CYAN, RESET);
}

fn draw_progress_bar(label: &str, current: usize, total: usize, width: usize, color: &str) {
    let pct = if total > 0 { (current as f64 / total as f64 * 100.0) as usize } else { 0 };
    let filled = if total > 0 { (current as f64 / total as f64 * width as f64) as usize } else { 0 };
    let empty = width.saturating_sub(filled);
    
    let bar: String = format!("{}{}{}",
        color,
        "█".repeat(filled),
        format!("{}░{}", DIM, RESET).repeat(empty)
    );
    
    println!("  {}{:<12}{} │{}│ {:>6.1}% ({}/{})", 
        BOLD, label, RESET, bar, pct as f64, current, total);
}

fn draw_horizontal_bar(value: f64, max_width: usize, color: &str, bg_color: &str) -> String {
    let filled = ((value * max_width as f64) as usize).min(max_width);
    let empty = max_width.saturating_sub(filled);
    format!("{}{}{}{}{}", 
        color, "▓".repeat(filled), 
        bg_color, "░".repeat(empty), RESET)
}

fn draw_data_aggregation(stats: &DataStats, loaded: &DataStats) {
    println!("\n{}{}┌─────────────────────────────────────────────────────────────────────────────┐{}", BOLD, FG_BLUE, RESET);
    println!("{}{}│{}  📊 {}数据聚合形式{}                                                           {}│{}", BOLD, FG_BLUE, RESET, FG_WHITE, RESET, FG_BLUE, RESET);
    println!("{}{}├─────────────────────────────────────────────────────────────────────────────┤{}", BOLD, FG_BLUE, RESET);
    
    let max_loaded = [loaded.vocab, loaded.sentence, loaded.dialogue, loaded.novel,
                      loaded.script, loaded.image, loaded.audio, loaded.wiki]
                     .iter().max().copied().unwrap_or(1) as f64;
    
    let bar_width = 35;
    
    // 词汇
    let v_pct = loaded.vocab as f64 / max_loaded;
    let v_processed = if loaded.vocab > 0 { stats.vocab as f64 / loaded.vocab as f64 } else { 0.0 };
    println!("{}{}│{}  {}词汇{}     {:>6} │{} {:>5.1}%", 
        BOLD, FG_BLUE, RESET, FG_YELLOW, RESET, loaded.vocab,
        draw_horizontal_bar(v_pct, bar_width, FG_YELLOW, DIM), v_processed * 100.0);
    
    // 句子
    let s_pct = loaded.sentence as f64 / max_loaded;
    let s_processed = if loaded.sentence > 0 { stats.sentence as f64 / loaded.sentence as f64 } else { 0.0 };
    println!("{}{}│{}  {}句子{}     {:>6} │{} {:>5.1}%",
        BOLD, FG_BLUE, RESET, FG_GREEN, RESET, loaded.sentence,
        draw_horizontal_bar(s_pct, bar_width, FG_GREEN, DIM), s_processed * 100.0);
    
    // 对话
    let d_pct = loaded.dialogue as f64 / max_loaded;
    let d_processed = if loaded.dialogue > 0 { stats.dialogue as f64 / loaded.dialogue as f64 } else { 0.0 };
    println!("{}{}│{}  {}对话{}     {:>6} │{} {:>5.1}%",
        BOLD, FG_BLUE, RESET, FG_CYAN, RESET, loaded.dialogue,
        draw_horizontal_bar(d_pct, bar_width, FG_CYAN, DIM), d_processed * 100.0);
    
    // 小说
    let n_pct = loaded.novel as f64 / max_loaded;
    let n_processed = if loaded.novel > 0 { stats.novel as f64 / loaded.novel as f64 } else { 0.0 };
    println!("{}{}│{}  {}小说{}     {:>6} │{} {:>5.1}%",
        BOLD, FG_BLUE, RESET, FG_MAGENTA, RESET, loaded.novel,
        draw_horizontal_bar(n_pct, bar_width, FG_MAGENTA, DIM), n_processed * 100.0);
    
    // 图片
    let i_pct = loaded.image as f64 / max_loaded;
    let i_processed = if loaded.image > 0 { stats.image as f64 / loaded.image as f64 } else { 0.0 };
    println!("{}{}│{}  {}图片{}     {:>6} │{} {:>5.1}%",
        BOLD, FG_BLUE, RESET, FG_ORANGE, RESET, loaded.image,
        draw_horizontal_bar(i_pct, bar_width, FG_ORANGE, DIM), i_processed * 100.0);
    
    // 音频
    let a_pct = loaded.audio as f64 / max_loaded;
    let a_processed = if loaded.audio > 0 { stats.audio as f64 / loaded.audio as f64 } else { 0.0 };
    println!("{}{}│{}  {}音频{}     {:>6} │{} {:>5.1}%",
        BOLD, FG_BLUE, RESET, FG_PINK, RESET, loaded.audio,
        draw_horizontal_bar(a_pct, bar_width, FG_PINK, DIM), a_processed * 100.0);
    
    // Wiki
    let w_pct = loaded.wiki as f64 / max_loaded;
    let w_processed = if loaded.wiki > 0 { stats.wiki as f64 / loaded.wiki as f64 } else { 0.0 };
    println!("{}{}│{}  {}Wiki{}     {:>6} │{} {:>5.1}%",
        BOLD, FG_BLUE, RESET, FG_LIME, RESET, loaded.wiki,
        draw_horizontal_bar(w_pct, bar_width, FG_LIME, DIM), w_processed * 100.0);
    
    println!("{}{}└─────────────────────────────────────────────────────────────────────────────┘{}", BOLD, FG_BLUE, RESET);
}

fn draw_training_cycle(epoch: u64, progress: f64, total_processed: u64, rate: f64, elapsed: Duration) {
    println!("\n{}{}┌─────────────────────────────────────────────────────────────────────────────┐{}", BOLD, FG_GREEN, RESET);
    println!("{}{}│{}  🔄 {}训练周期{}                                                               {}│{}", BOLD, FG_GREEN, RESET, FG_WHITE, RESET, FG_GREEN, RESET);
    println!("{}{}├─────────────────────────────────────────────────────────────────────────────┤{}", BOLD, FG_GREEN, RESET);
    
    // Epoch 进度条
    let bar_width = 50;
    let filled = ((progress / 100.0 * bar_width as f64) as usize).min(bar_width);
    let empty = bar_width - filled;
    let bar = format!("{}{}{}{}░{}", 
        FG_GREEN, "█".repeat(filled), 
        DIM, "░".repeat(empty), RESET);
    
    println!("{}{}│{}  Epoch {:>3} │{}│ {:>5.1}%",
        BOLD, FG_GREEN, RESET, epoch, bar, progress);
    
    let hours = elapsed.as_secs() / 3600;
    let mins = (elapsed.as_secs() % 3600) / 60;
    let secs = elapsed.as_secs() % 60;
    
    println!("{}{}│{}  已处理: {}{:>12}{} 项  │  速度: {}{:>8.0}{}/s  │  耗时: {}{}h {}m {}s{}",
        BOLD, FG_GREEN, RESET,
        FG_YELLOW, total_processed, RESET,
        FG_CYAN, rate, RESET,
        FG_MAGENTA, hours, mins, secs, RESET);
    
    println!("{}{}└─────────────────────────────────────────────────────────────────────────────┘{}", BOLD, FG_GREEN, RESET);
}

fn draw_fate_system(prefs: &[f64], stable_count: u32, max_stable: u32, drift: f64) {
    println!("\n{}{}┌─────────────────────────────────────────────────────────────────────────────┐{}", BOLD, FG_MAGENTA, RESET);
    println!("{}{}│{}  🌌 {}FATE 系统偏好{}                                                         {}│{}", BOLD, FG_MAGENTA, RESET, FG_WHITE, RESET, FG_MAGENTA, RESET);
    println!("{}{}├─────────────────────────────────────────────────────────────────────────────┤{}", BOLD, FG_MAGENTA, RESET);
    
    let names = ["BlackHole ", "MeshBrain ", "MultiNova ", "BaseForce "];
    let colors = [FG_RED, FG_CYAN, FG_YELLOW, FG_GREEN];
    let bar_width = 30;
    
    for (i, name) in names.iter().enumerate() {
        let val = prefs.get(i).copied().unwrap_or(0.25);
        let pct = val * 100.0;
        let filled = ((val * bar_width as f64) as usize).min(bar_width);
        let empty = bar_width - filled;
        
        let bar = format!("{}{}{}{}░{}", 
            colors[i], "▓".repeat(filled),
            DIM, "░".repeat(empty), RESET);
        
        // 动态指示器
        let indicator = if pct > 26.0 { "▲" } else if pct < 24.0 { "▼" } else { "●" };
        let ind_color = if pct > 26.0 { FG_GREEN } else if pct < 24.0 { FG_RED } else { FG_YELLOW };
        
        println!("{}{}│{}  {}{}{} │{}│ {:>5.2}% {}{}{}",
            BOLD, FG_MAGENTA, RESET,
            colors[i], name, RESET,
            bar, pct, ind_color, indicator, RESET);
    }
    
    println!("{}{}├─────────────────────────────────────────────────────────────────────────────┤{}", BOLD, FG_MAGENTA, RESET);
    
    // 收敛状态
    let conv_bar_width = 30;
    let conv_filled = ((stable_count as f64 / max_stable as f64 * conv_bar_width as f64) as usize).min(conv_bar_width);
    let conv_empty = conv_bar_width - conv_filled;
    let conv_bar = format!("{}{}{}{}░{}", 
        FG_CYAN, "█".repeat(conv_filled),
        DIM, "░".repeat(conv_empty), RESET);
    
    let drift_color = if drift < 0.001 { FG_GREEN } else if drift < 0.005 { FG_YELLOW } else { FG_RED };
    
    println!("{}{}│{}  收敛进度: │{}│ {}/{} │ 漂移: {}{}%.4{}",
        BOLD, FG_MAGENTA, RESET,
        conv_bar, stable_count, max_stable,
        drift_color, drift * 100.0, RESET);
    
    println!("{}{}└─────────────────────────────────────────────────────────────────────────────┘{}", BOLD, FG_MAGENTA, RESET);
}

fn draw_status_bar(checkpoint: &str, last_save: &str) {
    println!("\n{}{}─────────────────────────────────────────────────────────────────────────────────{}", DIM, FG_WHITE, RESET);
    println!("  {}💾 Checkpoint:{} {} │ {}⏱ 上次保存:{} {}", 
        FG_CYAN, RESET, checkpoint,
        FG_GREEN, RESET, if last_save.is_empty() { "尚未保存" } else { last_save });
    println!("  {}按 Ctrl+C 安全退出（自动保存状态）{}", DIM, RESET);
}

// ═══════════════════════════════════════════════════════════════════
//  数据加载函数
// ═══════════════════════════════════════════════════════════════════

fn load_old_corpus(root: &Path) -> (Vec<DataItem>, DataStats) {
    let mut items = Vec::new();
    let mut stats = DataStats::new();
    
    // 语言语料
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
                                    items.push(DataItem::Vocabulary { word: word.to_string() });
                                    stats.vocab += 1;
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
                                    items.push(DataItem::Sentence { text: sentence });
                                    stats.sentence += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // 对话
    let dialogue_dir = root.join("dialogues");
    if dialogue_dir.exists() {
        let daily = dialogue_dir.join("daily_dialog_train.json");
        if daily.exists() {
            if let Ok(content) = fs::read_to_string(&daily) {
                for line in content.lines().take(30000) {
                    if let Some(start) = line.find('"') {
                        if let Some(end) = line[start+1..].find('"') {
                            let text = &line[start+1..start+1+end];
                            if text.len() > 5 && text.len() < 500 {
                                items.push(DataItem::Dialogue { text: text.to_string() });
                                stats.dialogue += 1;
                            }
                        }
                    }
                }
            }
        }
        
        let cornell = dialogue_dir.join("cornell movie-dialogs corpus/movie_lines.txt");
        if cornell.exists() {
            if let Ok(file) = File::open(&cornell) {
                for line in BufReader::new(file).lines().take(30000).flatten() {
                    let parts: Vec<&str> = line.split(" +++$+++ ").collect();
                    if parts.len() >= 5 {
                        let text = parts[4].trim();
                        if text.len() > 5 && text.len() < 500 {
                            items.push(DataItem::Dialogue { text: text.to_string() });
                            stats.dialogue += 1;
                        }
                    }
                }
            }
        }
    }
    
    // 小说
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
                                        stats.novel += 1;
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
    
    (items, stats)
}

fn load_new_data(root: &Path) -> (Vec<DataItem>, DataStats) {
    let mut items = Vec::new();
    let mut stats = DataStats::new();
    
    // COCO 图片
    let images_dir = root.join("images/train2017");
    if images_dir.exists() {
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
                        stats.image += 1;
                    }
                }
            }
        }
    }
    
    // LibriSpeech 音频
    let audio_dir = root.join("audio/LibriSpeech");
    if audio_dir.exists() {
        fn scan_flac(dir: &Path, items: &mut Vec<DataItem>, stats: &mut DataStats, limit: usize) {
            if stats.audio >= limit { return; }
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    if stats.audio >= limit { break; }
                    let path = entry.path();
                    if path.is_dir() {
                        scan_flac(&path, items, stats, limit);
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
                            stats.audio += 1;
                        }
                    }
                }
            }
        }
        scan_flac(&audio_dir, &mut items, &mut stats, 15000);
    }
    
    // Wikipedia
    let wiki_file = root.join("text/zhwiki-latest.xml.bz2");
    if wiki_file.exists() {
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
                            stats.wiki += 1;
                            if stats.wiki >= 50000 { break; }
                        }
                    }
                } else if in_text {
                    current_text.push_str(&line);
                    current_text.push('\n');
                }
                if stats.wiki >= 50000 { break; }
            }
        }
    }
    
    (items, stats)
}

fn save_checkpoint(path: &Path, state: &TrainingState) -> io::Result<()> {
    let json = serde_json::to_string_pretty(state)?;
    let mut file = File::create(path)?;
    file.write_all(b"VALHALLA_CHECKPOINT_V1\n")?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

fn load_checkpoint(path: &Path) -> Option<TrainingState> {
    if !path.exists() { return None; }
    let content = fs::read_to_string(path).ok()?;
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() || !lines[0].starts_with("VALHALLA_CHECKPOINT") { return None; }
    let json = lines[1..].join("\n");
    serde_json::from_str(&json).ok()
}

// ═══════════════════════════════════════════════════════════════════
//  主函数
// ═══════════════════════════════════════════════════════════════════

fn main() {
    let config = TrainingConfig::default();
    let running = Arc::new(AtomicBool::new(true));
    
    // Ctrl+C 处理
    let running_clone = running.clone();
    ctrlc::set_handler(move || {
        print!("{}", SHOW_CURSOR);
        running_clone.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");
    
    // 初始显示
    print!("{}{}", CLEAR_SCREEN, CURSOR_HOME);
    print!("{}", HIDE_CURSOR);
    
    draw_header();
    println!("\n  {}⏳ 正在加载数据...{}", FG_YELLOW, RESET);
    stdout().flush().ok();
    
    // 加载训练状态
    let mut state = load_checkpoint(&config.checkpoint_path).unwrap_or_default();
    
    // 系统配置
    let num_cpus = num_cpus::get();
    let target_threads = (num_cpus as f64 * config.cpu_percent as f64 / 100.0) as usize;
    rayon::ThreadPoolBuilder::new()
        .num_threads(target_threads)
        .build_global()
        .unwrap();
    
    // 初始化 Hub
    let hub_config = HubConfig {
        resource_percent: config.cpu_percent,
        batch_size: 512,
        ..Default::default()
    };
    let mut hub = ValhallaHub::with_config(hub_config);
    
    // 加载数据
    let (mut old_data, old_stats) = load_old_corpus(&config.old_corpus_root);
    let (mut new_data, new_stats) = load_new_data(&config.new_data_root);
    
    let mut loaded_stats = DataStats {
        vocab: old_stats.vocab,
        sentence: old_stats.sentence,
        dialogue: old_stats.dialogue,
        novel: old_stats.novel,
        script: old_stats.script,
        image: new_stats.image,
        audio: new_stats.audio,
        wiki: new_stats.wiki,
    };
    
    // 合并数据
    let mut all_data = Vec::with_capacity(old_data.len() + new_data.len());
    all_data.append(&mut old_data);
    all_data.append(&mut new_data);
    
    if all_data.is_empty() {
        print!("{}", SHOW_CURSOR);
        println!("\n{}❌ 没有找到训练数据！{}", FG_RED, RESET);
        return;
    }
    
    // 打乱数据
    let mut rng = rand::thread_rng();
    all_data.shuffle(&mut rng);
    
    let total_start = Instant::now();
    let mut last_render = Instant::now();
    let mut last_save = Instant::now();
    let data_len = all_data.len();
    
    let mut current_stats = DataStats::new();
    let mut prev_prefs: Option<Vec<f64>> = Some(state.fate_preferences.clone());
    let mut current_drift = 0.0f64;
    
    // 主训练循环
    while running.load(Ordering::SeqCst) {
        state.epoch += 1;
        
        all_data.shuffle(&mut rng);
        current_stats = DataStats::new();
        
        for (idx, item) in all_data.iter().enumerate() {
            if !running.load(Ordering::Relaxed) { break; }
            
            let (request_type, intensity, stat_update) = match item {
                DataItem::Vocabulary { word } => {
                    current_stats.vocab += 1;
                    (RequestType::Learning, (word.len() as f64 / 20.0).min(1.0).max(0.1), 0)
                },
                DataItem::Sentence { text } => {
                    current_stats.sentence += 1;
                    (RequestType::DataProcessing, (text.len() as f64 / 200.0).min(1.0).max(0.1), 1)
                },
                DataItem::Dialogue { text } => {
                    current_stats.dialogue += 1;
                    (RequestType::Adaptation, (text.len() as f64 / 150.0).min(1.0).max(0.1), 2)
                },
                DataItem::Novel { text } => {
                    current_stats.novel += 1;
                    (RequestType::DataProcessing, (text.len() as f64 / 500.0).min(1.0).max(0.1), 3)
                },
                DataItem::Script { text } => {
                    current_stats.script += 1;
                    (RequestType::Adaptation, (text.len() as f64 / 200.0).min(1.0).max(0.1), 4)
                },
                DataItem::Image { features } => {
                    current_stats.image += 1;
                    (RequestType::DataProcessing, features[0].max(0.1), 6)
                },
                DataItem::Audio { features } => {
                    current_stats.audio += 1;
                    (RequestType::Learning, features[0].max(0.1), 7)
                },
                DataItem::WikiText { text } => {
                    current_stats.wiki += 1;
                    (RequestType::Reasoning, (text.len() as f64 / 500.0).min(1.0).max(0.1), 8)
                },
            };
            
            let request = HubRequest {
                request_id: format!("e{}_i{}", state.epoch, idx),
                request_type,
                intensity,
                data: None,
                priority: Priority::Normal,
            };
            
            hub.process(request);
            state.total_processed += 1;
            
            // 更新显示 (每500ms)
            if last_render.elapsed() >= Duration::from_millis(500) {
                let prefs = hub.get_fate_preferences();
                let elapsed = total_start.elapsed();
                let rate = state.total_processed as f64 / elapsed.as_secs_f64().max(0.001);
                let progress = (idx as f64 / data_len as f64) * 100.0;
                
                // 计算漂移
                if let Some(ref prev) = prev_prefs {
                    current_drift = prefs.affinities.iter()
                        .zip(prev.iter())
                        .map(|(a, b)| (a - b).abs())
                        .sum();
                    
                    if current_drift < config.convergence_threshold {
                        state.stable_count += 1;
                        if state.stable_count >= config.convergence_stable_count {
                            running.store(false, Ordering::SeqCst);
                        }
                    } else {
                        state.stable_count = 0;
                    }
                }
                prev_prefs = Some(prefs.affinities.clone());
                state.fate_preferences = prev_prefs.clone().unwrap();
                
                // 绘制界面
                print!("{}{}", CLEAR_SCREEN, CURSOR_HOME);
                draw_header();
                draw_data_aggregation(&current_stats, &loaded_stats);
                draw_training_cycle(state.epoch, progress, state.total_processed, rate, elapsed);
                draw_fate_system(&state.fate_preferences, state.stable_count, config.convergence_stable_count, current_drift);
                draw_status_bar(&config.checkpoint_path.to_string_lossy(), &state.last_save_time);
                stdout().flush().ok();
                
                last_render = Instant::now();
            }
            
            // 自动保存 (每5分钟)
            if last_save.elapsed() >= Duration::from_secs(300) {
                state.last_save_time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                let _ = save_checkpoint(&config.checkpoint_path, &state);
                last_save = Instant::now();
            }
            
            thread::sleep(Duration::from_micros(15));
        }
    }
    
    // 最终保存
    state.last_save_time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let final_prefs = hub.get_fate_preferences();
    state.fate_preferences = final_prefs.affinities.clone();
    state.fate_signal_tick = hub.fate_system().get_logical_time();
    let _ = save_checkpoint(&config.checkpoint_path, &state);
    
    print!("{}", SHOW_CURSOR);
    print!("{}{}", CLEAR_SCREEN, CURSOR_HOME);
    
    println!("\n{}{}╔════════════════════════════════════════════════════════════════╗{}", BOLD, FG_GREEN, RESET);
    println!("{}{}║               ✅ 训练已安全停止并保存                          ║{}", BOLD, FG_GREEN, RESET);
    println!("{}{}╚════════════════════════════════════════════════════════════════╝{}", BOLD, FG_GREEN, RESET);
    
    let elapsed = total_start.elapsed();
    println!("\n⏱️  总耗时: {}h {}m {}s", 
        elapsed.as_secs() / 3600,
        (elapsed.as_secs() % 3600) / 60,
        elapsed.as_secs() % 60);
    println!("📊 处理: {} 项 | Epoch: {}", state.total_processed, state.epoch);
    println!("\n🎯 Fate 偏好:");
    println!("   BlackHole:  {:.4}%", state.fate_preferences.get(0).unwrap_or(&0.0) * 100.0);
    println!("   MeshBrain:  {:.4}%", state.fate_preferences.get(1).unwrap_or(&0.0) * 100.0);
    println!("   MultiNova:  {:.4}%", state.fate_preferences.get(2).unwrap_or(&0.0) * 100.0);
    println!("   BaseForce:  {:.4}%", state.fate_preferences.get(3).unwrap_or(&0.0) * 100.0);
    println!("\n💾 Checkpoint: {}", config.checkpoint_path.display());
}
