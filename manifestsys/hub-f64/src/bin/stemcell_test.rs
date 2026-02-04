//! StemCellComplete 全量训练 (70% 资源, 10周期)

use stem_cell::StemCellComplete;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::time::Instant;
use rayon;

fn main() {
    eprintln!("╔═══════════════════════════════════════════════════════════════════╗");
    eprintln!("║     StemCellComplete 全量训练 (70%资源, 10周期 Fate)              ║");
    eprintln!("╚═══════════════════════════════════════════════════════════════════╝\n");
    
    // 70% CPU 资源限制
    let num_cpus = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let target_threads = (num_cpus as f64 * 0.7).ceil() as usize;
    rayon::ThreadPoolBuilder::new()
        .num_threads(target_threads)
        .build_global()
        .ok();
    eprintln!("⚙️  CPU: {}/{} 核心 (70%)", target_threads, num_cpus);
    eprintln!("⚙️  GPU: CUDA 加速已启用\n");

    // 加载完整语料
    eprintln!("📂 加载完整语料...\n");
    let corpus = load_full_corpus();
    eprintln!("\n📊 总计: {} 条\n", corpus.len());

    if corpus.is_empty() {
        eprintln!("❌ 未找到语料！");
        return;
    }

    // 初始化 StemCell
    let mut stemcell = StemCellComplete::new();
    eprintln!("🧬 初始化 StemCellComplete");
    eprintln!("   含 TripleFate + VoyagerSpace\n");

    let start = Instant::now();

    // ==================== 第1周期：全语料 → 细胞 ====================
    eprintln!("═══ 周期 1/10：全语料 → 细胞 ═══");
    let cycle1_start = Instant::now();
    let mut total_signals = 0u64;
    let mut last_report = Instant::now();

    for (i, content) in corpus.iter().enumerate() {
        for ch in content.chars() {
            let v = (ch as u32 as f64 / 65536.0) * 2.0 - 1.0;
            stemcell.receive_data(v);
            total_signals += 1;
        }

        if last_report.elapsed().as_secs() >= 5 {
            let status = stemcell.get_status();
            let pct = i * 100 / corpus.len();
            let rate = total_signals as f64 / cycle1_start.elapsed().as_secs_f64();
            eprintln!("  {}% [{}/{}] {:.0} 信号/s | 细胞:{}",
                pct, i, corpus.len(), rate, status.cell_count);
            io::stderr().flush().unwrap();
            last_report = Instant::now();
        }
    }
    
    let status = stemcell.get_status();
    eprintln!("  📊 总信号: {} | 细胞: {}", total_signals, status.cell_count);
    eprintln!("  ✓ 周期1完成 {:.1}s\n", cycle1_start.elapsed().as_secs_f64());

    // ==================== 第2-10周期：纯细胞演化 ====================
    for cycle in 2..=10 {
        let cycle_start = Instant::now();
        eprintln!("═══ 周期 {}/10：细胞演化 ═══", cycle);

        let cell_result = stemcell.process_cells_cycle();
        
        let status = stemcell.get_status();
        let t = cycle_start.elapsed().as_secs_f64();
        
        eprintln!("  🧬 细胞:{} | 合并:{} 分裂:{} 死亡:{} | 能量:{:.3}",
            status.cell_count,
            cell_result.merges,
            cell_result.divisions,
            cell_result.deaths,
            cell_result.avg_energy);
        eprintln!("  ✓ 周期{}完成 {:.2}s | Fate收敛:{}\n",
            cycle, t, stemcell.has_fate_converged());
    }

    // 最终报告
    let total = start.elapsed();
    let final_status = stemcell.get_status();
    
    eprintln!("╔═══════════════════════════════════════════════════════════════════╗");
    eprintln!("║                 StemCell 10周期 最终报告                          ║");
    eprintln!("╚═══════════════════════════════════════════════════════════════════╝");
    eprintln!();
    eprintln!("⏱️  总时间: {:.1}s", total.as_secs_f64());
    eprintln!();
    eprintln!("🧬 细胞结构:");
    eprintln!("   细胞数: {}", final_status.cell_count);
    eprintln!("   容器闭合 (局部收敛→细胞): {}", final_status.total_containers_closed);
    eprintln!("   分裂总数: {}", final_status.total_divisions);
    eprintln!("   死亡总数: {}", final_status.total_deaths);
    eprintln!("   活跃容器: {}", final_status.active_containers);
    eprintln!();
    eprintln!("📐 空间:");
    eprintln!("   维度: {:.4}", final_status.dimension);
    eprintln!();
    eprintln!("🎯 TripleFate:");
    eprintln!("   Fate tick: {}", final_status.fate_tick);
    eprintln!("   Base Generation: {}", final_status.base_generation);
    eprintln!("   Meta Generation: {}", final_status.meta_generation);
    eprintln!("   收敛: {}", stemcell.has_fate_converged());
    eprintln!();
    eprintln!("{}", stemcell.get_fate_stats());
    
    // 输出效率历史（最后10条）
    let eff_hist = stemcell.get_efficiency_history();
    eprintln!("\n📈 效率历史 (最后{}条):", eff_hist.len().min(10));
    for (i, (base_eff, _meta_eff)) in eff_hist.iter().rev().take(10).enumerate() {
        eprintln!("   [{:2}] base_efficiency: {:.6}", eff_hist.len() - i, base_eff);
    }
}

fn load_full_corpus() -> Vec<String> {
    let mut items = Vec::new();
    let corpus_root = Path::new("/run/media/jouly/My Book/valhalla_corpus");
    let valhalla_root = Path::new("/home/jouly/AI/Valhalla");

    // === 1. 语言模态 (句子+词汇+语法) ===
    eprint!("  🗣️  语言模态...");
    io::stderr().flush().unwrap();
    let lang_path = corpus_root.join("languages");
    let languages = ["english", "chinese", "japanese", "korean", "french", 
                     "german", "spanish", "russian", "arabic", "portuguese"];
    
    let mut lang_count = 0usize;
    
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
                            let s = if parts.len() >= 3 { parts[2].trim() } else { line.trim() };
                            if s.len() > 5 && s.len() < 500 {
                                items.push(s.to_string());
                                lang_count += 1;
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
                                items.push(word.to_string());
                                lang_count += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    eprintln!(" {} 条", lang_count);

    // === 2. 对话模态 ===
    eprint!("  💬 对话模态...");
    io::stderr().flush().unwrap();
    let dialogue_start = items.len();

    // Cornell Movie Dialogs
    let cornell_path = corpus_root.join("dialogues/cornell movie-dialogs corpus/movie_lines.txt");
    if cornell_path.exists() {
        if let Ok(file) = File::open(&cornell_path) {
            for line in BufReader::new(file).lines().flatten() {
                let parts: Vec<&str> = line.split(" +++$+++ ").collect();
                if parts.len() >= 5 {
                    let content = parts[4].trim();
                    if content.len() > 5 && content.len() < 500 {
                        items.push(content.to_string());
                    }
                }
            }
        }
    }

    // ConvAI2
    let convai_path = corpus_root.join("dialogues/convai/train_both_original.txt");
    if convai_path.exists() {
        if let Ok(file) = File::open(&convai_path) {
            for line in BufReader::new(file).lines().flatten() {
                let content = line.trim();
                if content.len() > 10 && content.len() < 500 && !content.starts_with("your persona:") {
                    items.push(content.to_string());
                }
            }
        }
    }

    // 原有对话
    for name in ["valhalla_all_dialogues.txt", "valhalla_enhanced_dialogues.txt"] {
        let p = valhalla_root.join(name);
        if p.exists() {
            if let Ok(c) = fs::read_to_string(&p) {
                for line in c.lines() {
                    if line.trim().len() > 3 { items.push(line.trim().to_string()); }
                }
            }
        }
    }
    eprintln!(" {} 条", items.len() - dialogue_start);

    // === 3. 叙事模态 ===
    eprint!("  📖 叙事模态...");
    io::stderr().flush().unwrap();
    let narrative_start = items.len();

    // Gutenberg
    let gutenberg_path = corpus_root.join("novels/gutenberg");
    if gutenberg_path.exists() {
        for entry in fs::read_dir(&gutenberg_path).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "txt") {
                if let Ok(file) = File::open(&path) {
                    let mut paragraph = String::new();
                    for line in BufReader::new(file).lines().flatten() {
                        if line.trim().is_empty() && !paragraph.is_empty() {
                            if paragraph.len() > 50 && paragraph.len() < 1000 {
                                items.push(paragraph.clone());
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
    eprintln!(" {} 条", items.len() - narrative_start);

    // === 4. 图片描述 ===
    eprint!("  🖼️  图片描述...");
    io::stderr().flush().unwrap();
    let image_start = items.len();

    // Flickr30k
    let flickr_path = corpus_root.join("images/flickr30k/Sentences");
    if flickr_path.exists() {
        for entry in fs::read_dir(&flickr_path).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "txt") {
                if let Ok(file) = File::open(&path) {
                    for line in BufReader::new(file).lines().flatten() {
                        let content = line.trim();
                        if content.len() > 10 && content.len() < 500 && !content.starts_with('#') {
                            items.push(content.to_string());
                        }
                    }
                }
            }
        }
    }
    eprintln!(" {} 条", items.len() - image_start);

    items
}
