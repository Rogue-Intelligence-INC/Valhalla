//! Hub + Triple-Fate 训练
//! 10 周期（每周期 = Fate 收敛一次）
//! 收敛 = 代数形式边际效应递减

use std::fs::{self, File};
use std::io::{self, Write, BufRead, BufReader};
use std::path::Path;
use std::time::Instant;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use fate::triple_fate::TripleFateCombined;

fn load_training_data() -> Vec<Vec<f64>> {
    let mut items = Vec::new();
    let corpus_root = Path::new("/run/media/jouly/My Book/valhalla_corpus");
    
    println!("📦 加载训练数据...\n");
    
    print!("  🗣️  语言...");
    io::stdout().flush().unwrap();
    let lang_path = corpus_root.join("languages");
    
    for lang in &["english", "chinese", "japanese"] {
        let sent_path = lang_path.join(lang).join("sentences");
        if sent_path.exists() {
            for entry in fs::read_dir(&sent_path).into_iter().flatten().flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "txt" || e == "tsv") {
                    if let Ok(file) = File::open(&path) {
                        let reader = BufReader::new(file);
                        for line in reader.lines().flatten() {
                            let parts: Vec<&str> = line.split('\t').collect();
                            let text = if parts.len() >= 3 { parts[2] } else { &line };
                            if text.len() > 5 && text.len() < 500 {
                                let signal = text_to_signal(text);
                                items.push(signal);
                            }
                        }
                    }
                }
            }
        }
    }
    println!(" {} 条", items.len());
    
    print!("  💬 对话...");
    io::stdout().flush().unwrap();
    let dialogue_start = items.len();
    
    let cornell_path = corpus_root.join("dialogues/cornell movie-dialogs corpus/movie_lines.txt");
    if cornell_path.exists() {
        if let Ok(file) = File::open(&cornell_path) {
            let reader = BufReader::new(file);
            for line in reader.lines().flatten() {
                let parts: Vec<&str> = line.split(" +++$+++ ").collect();
                if parts.len() >= 5 {
                    let content = parts[4].trim();
                    if content.len() > 5 && content.len() < 500 {
                        let signal = text_to_signal(content);
                        items.push(signal);
                    }
                }
            }
        }
    }
    println!(" {} 条", items.len() - dialogue_start);
    
    print!("  📖 小说...");
    io::stdout().flush().unwrap();
    let novel_start = items.len();
    
    let gutenberg_path = corpus_root.join("novels/gutenberg");
    if gutenberg_path.exists() {
        for entry in fs::read_dir(&gutenberg_path).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "txt") {
                if let Ok(file) = File::open(&path) {
                    let reader = BufReader::new(file);
                    let mut paragraph = String::new();
                    for line in reader.lines().flatten() {
                        if line.trim().is_empty() && !paragraph.is_empty() {
                            if paragraph.len() > 50 && paragraph.len() < 800 {
                                let signal = text_to_signal(&paragraph);
                                items.push(signal);
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
    println!(" {} 条", items.len() - novel_start);
    
    println!("\n📊 总计: {} 条数据\n", items.len());
    items
}

fn text_to_signal(text: &str) -> Vec<f64> {
    let len = text.len() as f64;
    let words = text.split_whitespace().count() as f64;
    let chars: Vec<char> = text.chars().collect();
    
    let avg_word_len = if words > 0.0 { len / words } else { 0.0 };
    let punctuation = chars.iter().filter(|c| c.is_ascii_punctuation()).count() as f64;
    let uppercase = chars.iter().filter(|c| c.is_uppercase()).count() as f64;
    
    vec![
        len / 100.0,
        words / 20.0,
        avg_word_len / 10.0,
        punctuation / 10.0,
        uppercase / 10.0,
    ]
}

/// 代数形式收敛检测
/// 基于效率序列的边际效应递减
struct ConvergenceDetector {
    efficiency_history: Vec<f64>,
}

impl ConvergenceDetector {
    fn new() -> Self {
        Self {
            efficiency_history: Vec::new(),
        }
    }
    
    fn record(&mut self, efficiency: f64) {
        self.efficiency_history.push(efficiency);
        if self.efficiency_history.len() > 100 {
            self.efficiency_history.remove(0);
        }
    }
    
    /// 代数形式判定: diff2 < diff1
    fn has_converged(&self) -> bool {
        let len = self.efficiency_history.len();
        if len < 3 {
            return false;
        }
        
        let h = &self.efficiency_history;
        let diff1 = (h[len-2] - h[len-3]).abs();
        let diff2 = (h[len-1] - h[len-2]).abs();
        
        // 边际递减 且 已有足够历史
        diff2 < diff1 && diff1 > 0.0 && len >= 10
    }
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║     Hub + Triple-Fate 训练                                     ║");
    println!("║     10 周期 | 代数形式收敛判定                                 ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        println!("\n⚠️  停止...");
        r.store(false, Ordering::SeqCst);
    }).expect("Ctrl-C handler");
    
    let data = load_training_data();
    if data.is_empty() {
        println!("❌ 无数据");
        return;
    }
    
    println!("🔧 初始化 TripleFateCombined...");
    let mut fate = TripleFateCombined::new();
    println!("   ✅ 完成\n");
    
    let max_cycles = 10;
    let mut data_index = 0usize;
    let global_start = Instant::now();
    let report_interval = 10000u64;
    
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  🚀 开始训练 - 10 周期");
    println!("═══════════════════════════════════════════════════════════════════\n");
    
    for cycle in 1..=max_cycles {
        if !running.load(Ordering::Relaxed) {
            break;
        }
        
        println!("┌─────────────────────────────────────────────────────────────┐");
        println!("│  📍 周期 {}/{}                                              │", cycle, max_cycles);
        println!("└─────────────────────────────────────────────────────────────┘");
        
        let cycle_start = Instant::now();
        let mut cycle_signals = 0u64;
        let mut detector = ConvergenceDetector::new();
        
        loop {
            if !running.load(Ordering::Relaxed) {
                break;
            }
            
            let signal = data[data_index % data.len()].clone();
            data_index += 1;
            cycle_signals += 1;
            
            fate.tick(Some(signal));
            
            // 每隔一定信号数记录效率
            if cycle_signals % report_interval == 0 {
                let combined_efficiency = (fate.base.state.efficiency + fate.meta.state.efficiency) / 2.0;
                detector.record(combined_efficiency);
                
                let elapsed = cycle_start.elapsed().as_secs_f64();
                let rate = cycle_signals as f64 / elapsed;
                
                println!("  信号: {} | {:.0}/s | 效率: {:.2}%",
                    cycle_signals, rate, combined_efficiency * 100.0);
                
                // 代数形式收敛检测
                if detector.has_converged() {
                    println!("\n  🎉 周期 {} 收敛（边际递减）！", cycle);
                    break;
                }
            }
        }
        
        let cycle_elapsed = cycle_start.elapsed();
        println!("  信号: {} | 时间: {}m {}s",
            cycle_signals,
            cycle_elapsed.as_secs() / 60,
            cycle_elapsed.as_secs() % 60
        );
        println!("  Base: 发现={}, 效率={:.2}%",
            fate.base.discoveries.len(),
            fate.base.state.efficiency * 100.0
        );
        println!("  Meta: 发现={}, 效率={:.2}%\n",
            fate.meta.discoveries.len(),
            fate.meta.state.efficiency * 100.0
        );
        
        if !running.load(Ordering::Relaxed) {
            break;
        }
    }
    
    let total_elapsed = global_start.elapsed();
    
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║                    📊 训练完成                                  ║");
    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("║  ⏱️  总时间: {}m {}s", total_elapsed.as_secs() / 60, total_elapsed.as_secs() % 60);
    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("{}", fate.stats());
    println!("╚════════════════════════════════════════════════════════════════╝\n");
}
