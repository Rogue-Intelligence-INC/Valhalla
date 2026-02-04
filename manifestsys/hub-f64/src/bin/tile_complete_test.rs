//! 🧱 TileQuattroComplete 全量训练 (与 StemCellComplete 同样架构)
//!
//! 周期1: 语料 → Tile (边际递减收敛)
//! 周期2+: Tile演化 (边际递减合并 + 能量流转 + 分裂/死亡)
//!
//! 与 StemCell 相同的核心逻辑:
//! - 边际递减判断收敛和相似
//! - 反向边际效应触发合并
//! - 数据特征作为身份继承

use valhalla_tile_quattro::TileQuattroComplete;
use std::fs::{self, File};
use std::io::{self, Write, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::Instant;

fn find_conllu_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
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

/// 语料加载器 - 每个模态独立返回
struct CorpusLoader {
    corpus_root: PathBuf,
    valhalla_root: PathBuf,
}

impl CorpusLoader {
    fn new() -> Self {
        Self {
            corpus_root: PathBuf::from("/run/media/jouly/My Book/valhalla_corpus"),
            valhalla_root: PathBuf::from("/home/jouly/AI/Valhalla"),
        }
    }
    
    /// 加载语言模态 (按字母分解)
    fn load_language_signals(&self) -> Vec<f64> {
        let mut signals = Vec::new();
        let lang_path = self.corpus_root.join("languages");
        let languages = ["english", "chinese", "japanese", "korean", "french", 
                         "german", "spanish", "russian", "arabic", "portuguese"];
        
        for lang in &languages {
            let lang_dir = lang_path.join(lang);
            
            // 句子
            let sent_path = lang_dir.join("sentences");
            if sent_path.exists() {
                for entry in fs::read_dir(&sent_path).into_iter().flatten().flatten() {
                    let path = entry.path();
                    if path.extension().map_or(false, |e| e == "txt" || e == "tsv") {
                        if let Ok(file) = File::open(&path) {
                            for line in BufReader::new(file).lines().flatten() {
                                let parts: Vec<&str> = line.split('\t').collect();
                                let sentence = if parts.len() >= 3 { parts[2].trim() } else { line.trim() };
                                if sentence.len() > 5 && sentence.len() < 500 {
                                    // 按字母/字符分解
                                    for c in sentence.chars() {
                                        signals.push(c as u32 as f64 / 256.0);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // 词汇
            let vocab_path = lang_dir.join("vocabulary");
            if vocab_path.exists() {
                for entry in fs::read_dir(&vocab_path).into_iter().flatten().flatten() {
                    let path = entry.path();
                    if path.extension().map_or(false, |e| e == "txt") {
                        if let Ok(file) = File::open(&path) {
                            for line in BufReader::new(file).lines().flatten() {
                                let word = line.split_whitespace().next().unwrap_or("").trim();
                                if word.len() > 1 && !word.starts_with('#') {
                                    for c in word.chars() {
                                        signals.push(c as u32 as f64 / 256.0);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // 语法
            let grammar_path = lang_dir.join("grammar");
            if grammar_path.exists() {
                for path in find_conllu_files(&grammar_path) {
                    if let Ok(file) = File::open(&path) {
                        let mut current_text = String::new();
                        for line in BufReader::new(file).lines().flatten() {
                            if line.starts_with("# text = ") {
                                current_text = line[9..].to_string();
                            } else if line.is_empty() && !current_text.is_empty() {
                                if current_text.len() > 5 && current_text.len() < 500 {
                                    for c in current_text.chars() {
                                        signals.push(c as u32 as f64 / 256.0);
                                    }
                                }
                                current_text.clear();
                            }
                        }
                    }
                }
            }
        }
        
        signals
    }
    
    /// 加载对话模态 (按字母分解)
    fn load_dialogue_signals(&self) -> Vec<f64> {
        let mut signals = Vec::new();
        
        let cornell_path = self.corpus_root.join("dialogues/cornell movie-dialogs corpus/movie_lines.txt");
        if cornell_path.exists() {
            if let Ok(file) = File::open(&cornell_path) {
                for line in BufReader::new(file).lines().flatten() {
                    let parts: Vec<&str> = line.split(" +++$+++ ").collect();
                    if parts.len() >= 5 {
                        let content = parts[4].trim();
                        if content.len() > 5 && content.len() < 500 {
                            for c in content.chars() {
                                signals.push(c as u32 as f64 / 256.0);
                            }
                        }
                    }
                }
            }
        }
        
        let convai_path = self.corpus_root.join("dialogues/convai/train_both_original.txt");
        if convai_path.exists() {
            if let Ok(file) = File::open(&convai_path) {
                for line in BufReader::new(file).lines().flatten() {
                    let content = line.trim();
                    if content.len() > 10 && content.len() < 500 && !content.starts_with("your persona:") {
                        for c in content.chars() {
                            signals.push(c as u32 as f64 / 256.0);
                        }
                    }
                }
            }
        }
        
        signals
    }
    
    /// 加载叙事模态 (按字母分解)
    fn load_narrative_signals(&self) -> Vec<f64> {
        let mut signals = Vec::new();
        
        let gutenberg_path = self.corpus_root.join("novels/gutenberg");
        if gutenberg_path.exists() {
            for entry in fs::read_dir(&gutenberg_path).into_iter().flatten().flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "txt") {
                    if let Ok(file) = File::open(&path) {
                        let mut paragraph = String::new();
                        for line in BufReader::new(file).lines().flatten() {
                            if line.trim().is_empty() && !paragraph.is_empty() {
                                if paragraph.len() > 50 && paragraph.len() < 1000 {
                                    for c in paragraph.chars() {
                                        signals.push(c as u32 as f64 / 256.0);
                                    }
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
        
        signals
    }
    
    /// 加载图片描述模态 (按字母分解)
    fn load_image_signals(&self) -> Vec<f64> {
        let mut signals = Vec::new();
        
        let coco_path = self.corpus_root.join("images/coco/annotations/captions_train2017.json");
        if coco_path.exists() {
            if let Ok(content) = fs::read_to_string(&coco_path) {
                for line in content.lines() {
                    if let Some(start) = line.find("\"caption\":") {
                        let rest = &line[start + 11..];
                        if let Some(end) = rest.find('"') {
                            let caption = &rest[..end];
                            if caption.len() > 10 && caption.len() < 500 {
                                for c in caption.chars() {
                                    signals.push(c as u32 as f64 / 256.0);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        signals
    }
}

fn main() {
    println!("╔═══════════════════════════════════════════════════════════════════╗");
    println!("║     TileQuattroComplete 全量训练 (70%资源, 10周期 Fate)           ║");
    println!("╚═══════════════════════════════════════════════════════════════════╝\n");
    
    let num_cpus = num_cpus::get();
    let active_cpus = (num_cpus as f64 * 0.7).ceil() as usize;
    rayon::ThreadPoolBuilder::new().num_threads(active_cpus).build_global().ok();
    
    println!("⚙️  CPU: {}/{} 核心 (70%)", active_cpus, num_cpus);
    println!("⚙️  GPU: CUDA 加速已启用\n");
    
    println!("📂 加载完整语料...\n");
    
    let loader = CorpusLoader::new();
    
    print!("  🗣️  语言模态...");
    io::stdout().flush().unwrap();
    let language_signals = loader.load_language_signals();
    println!(" {} 条", language_signals.len());
    
    print!("  💬 对话模态...");
    io::stdout().flush().unwrap();
    let dialogue_signals = loader.load_dialogue_signals();
    println!(" {} 条", dialogue_signals.len());
    
    print!("  📖 叙事模态...");
    io::stdout().flush().unwrap();
    let narrative_signals = loader.load_narrative_signals();
    println!(" {} 条", narrative_signals.len());
    
    print!("  🖼️  图片描述...");
    io::stdout().flush().unwrap();
    let image_signals = loader.load_image_signals();
    println!(" {} 条", image_signals.len());
    
    // 合并所有信号
    let mut all_signals = Vec::new();
    all_signals.extend(&language_signals);
    all_signals.extend(&dialogue_signals);
    all_signals.extend(&narrative_signals);
    all_signals.extend(&image_signals);
    
    println!("\n📊 总计: {} 条\n", all_signals.len());
    
    if all_signals.is_empty() {
        println!("❌ 无数据！");
        return;
    }
    
    println!("🧱 初始化 TileQuattroComplete");
    println!("   含 TripleFate + VoyagerSpace\n");
    
    let mut tile = TileQuattroComplete::new();
    let target_cycles = 10;
    let start_time = Instant::now();
    
    // ═══ 周期 1：全语料 → Tile ═══
    println!("═══ 周期 1/{}: 全语料 → Tile ═══", target_cycles);
    
    let cycle1_start = Instant::now();
    let total_signals = all_signals.len();
    let mut processed = 0usize;
    let report_interval = total_signals / 20;  // 每5%报告一次
    
    for (i, &signal) in all_signals.iter().enumerate() {
        tile.receive_data(signal);
        processed += 1;
        
        if i % report_interval == 0 && i > 0 {
            let elapsed = cycle1_start.elapsed().as_secs_f64();
            let rate = processed as f64 / elapsed;
            let connections: usize = tile.completed_tiles.iter().map(|t| t.connections.len()).sum();
            println!("  {:>2}% [{}/{}] {:.0} 信号/s | Tile:{} 连接:{}",
                   (i * 100 / total_signals), i, total_signals, rate, 
                   tile.completed_tiles.len(), connections);
        }
    }
    
    let cycle1_time = cycle1_start.elapsed().as_secs_f64();
    println!("\n  📊 总信号: {} | Tile: {}", total_signals, tile.completed_tiles.len());
    println!("  ✓ 周期1完成 {:.1}s\n", cycle1_time);
    
    // ═══ 周期 2-10：Tile 演化 ═══
    for cycle in 2..=target_cycles {
        println!("═══ 周期 {}/{}: Tile演化 ═══", cycle, target_cycles);
        
        let cycle_start = Instant::now();
        
        // 执行 Tile 周期处理
        let result = tile.process_tiles_cycle();
        
        let cycle_time = cycle_start.elapsed().as_secs_f64();
        let converged = tile.has_fate_converged();
        
        println!("  🧱 Tile:{} | 合并:{} 分裂:{} | 激活:{:.4}",
                 result.tile_count, result.merges, result.splits, result.avg_activation);
        println!("  🔗 连接:{} (新+{}) | 自发神经网络形成中",
                 result.total_connections, result.new_connections);
        println!("  ✓ 周期{}完成 {:.2}s | Fate收敛:{}\n", cycle, cycle_time, converged);
    }
    
    // ═══ 训练总结 ═══
    let total_time = start_time.elapsed();
    let status = tile.get_status();
    
    // 计算连接统计
    let total_connections: usize = tile.completed_tiles.iter().map(|t| t.connections.len()).sum();
    let avg_connections = if tile.completed_tiles.is_empty() { 0.0 } 
                          else { total_connections as f64 / tile.completed_tiles.len() as f64 };
    
    println!("╔═══════════════════════════════════════════════════════════════════╗");
    println!("║     📊 TileQuattroComplete 训练完成                               ║");
    println!("╠═══════════════════════════════════════════════════════════════════╣");
    println!("║ ⏱️  总时间: {:.1}s ({:.1}分钟)", total_time.as_secs_f64(), total_time.as_secs_f64() / 60.0);
    println!("║ 🔢 总信号: {}", total_signals);
    println!("║ 🧱 最终 Tile: {}", status.completed_tiles);
    println!("║ 🔗 自发连接: {} (平均 {:.1}/Tile)", total_connections, avg_connections);
    println!("║ 📐 维度: {:.2}", status.dimension);
    println!("║ 🎯 Fate tick: {}", status.fate_tick);
    println!("╠═══════════════════════════════════════════════════════════════════╣");
    println!("║ Fate 统计:");
    for line in tile.get_fate_stats().lines() {
        println!("║   {}", line);
    }
    println!("╚═══════════════════════════════════════════════════════════════════╝");
    
    println!("\n🏁 训练完成！");
}
