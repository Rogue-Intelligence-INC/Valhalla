//! 🧱 TileQuattro 单独测试 (完整语料, 10周期 Fate)

use valhalla_tile_quattro::{TileQuattroComplete, TileQuattroCompleteConfig};
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::time::Instant;

fn main() {
    eprintln!("╔═══════════════════════════════════════════════════════════════════╗");
    eprintln!("║     TileQuattroComplete 单独测试 (完整语料, 10周期 Fate)          ║");
    eprintln!("╚═══════════════════════════════════════════════════════════════════╝\n");

    // 加载完整语料
    eprintln!("📂 加载完整语料...\n");
    let corpus = load_full_corpus();
    eprintln!("\n📊 总计: {} 条\n", corpus.len());

    if corpus.is_empty() {
        eprintln!("❌ 未找到语料！");
        return;
    }

    // 初始化 Tile
    let mut tile = TileQuattroComplete::new();
    eprintln!("🧱 初始化 TileQuattroComplete");
    eprintln!("   含 TripleFate + VoyagerSpace\n");

    let start = Instant::now();

    // ==================== 第1周期：原始数据 → Tile ====================
    eprintln!("═══ 周期 1/10：数据 → Tile化 ═══");
    let cycle1_start = Instant::now();
    let mut last_report = Instant::now();

    for (i, content) in corpus.iter().enumerate() {
        let v = content.len() as f64 + (i as f64).sin() * 10.0;
        let _result = tile.receive_data(v);

        if last_report.elapsed().as_secs() >= 5 {
            let status = tile.get_status();
            let pct = i * 100 / corpus.len();
            let rate = i as f64 / cycle1_start.elapsed().as_secs_f64();
            eprintln!("  {}% [{}/{}] {:.0}/s | Tile:{} 闭合:{}",
                pct, i, corpus.len(), rate,
                status.completed_tiles,
                status.total_operations);
            last_report = Instant::now();
        }
    }

    let status = tile.get_status();
    let t = cycle1_start.elapsed().as_secs_f64();
    eprintln!("  ✓ 第1周期完成 {:.1}s | Tile:{} Fate收敛:{}\n",
        t, status.completed_tiles, tile.has_fate_converged());

    // ==================== 第2-10周期：纯 Tile 处理 ====================
    for cycle in 2..=10 {
        let cycle_start = Instant::now();
        eprintln!("═══ 周期 {}/10：Tile 演化 ═══", cycle);

        let tile_result = tile.process_tiles_cycle();
        
        let status = tile.get_status();
        let t = cycle_start.elapsed().as_secs_f64();
        
        eprintln!("  Tile数:{} | 合并:{} 分裂:{} | 平均激活:{:.3}",
            status.completed_tiles,
            tile_result.merges,
            tile_result.splits,
            tile_result.avg_activation);
        eprintln!("  ✓ 完成 {:.2}s | Fate收敛:{}\n",
            t, tile.has_fate_converged());

        if tile.has_fate_converged() {
            eprintln!("🎯 Fate 收敛！停止于周期 {}\n", cycle);
            break;
        }
    }

    // 最终报告
    let total = start.elapsed();
    let final_status = tile.get_status();
    
    eprintln!("╔═══════════════════════════════════════════════════════════════════╗");
    eprintln!("║                 TileQuattro 10周期 最终报告                       ║");
    eprintln!("╚═══════════════════════════════════════════════════════════════════╝");
    eprintln!();
    eprintln!("⏱️  总时间: {:.1}s", total.as_secs_f64());
    eprintln!();
    eprintln!("🧱 Tile 结构:");
    eprintln!("   Tile数: {}", final_status.completed_tiles);
    eprintln!("   总操作: {}", final_status.total_operations);
    eprintln!("   活跃容器: {}", final_status.active_tiles);
    eprintln!();
    eprintln!("📐 空间:");
    eprintln!("   维度: {:.4}", final_status.dimension);
    eprintln!();
    eprintln!("🎯 TripleFate:");
    eprintln!("   Fate tick: {}", final_status.fate_tick);
    eprintln!("   Base Generation: {}", final_status.base_generation);
    eprintln!("   Meta Generation: {}", final_status.meta_generation);
    eprintln!("   收敛: {}", tile.has_fate_converged());
    eprintln!();
    eprintln!("{}", tile.get_fate_stats());
    
    // 输出效率历史（最后10条）
    let eff_hist = tile.get_efficiency_history();
    eprintln!("\n📈 效率历史 (最后{}条):", eff_hist.len().min(10));
    for (i, (base_eff, _meta_eff)) in eff_hist.iter().rev().take(10).enumerate() {
        eprintln!("   [{:2}] base_efficiency: {:.6}", eff_hist.len() - i, base_eff);
    }
}

fn load_full_corpus() -> Vec<String> {
    let mut items = Vec::new();
    let corpus_root = Path::new("/run/media/jouly/My Book/valhalla_corpus");

    // 语言模态
    eprint!("  🗣️  语言模态...");
    io::stderr().flush().unwrap();
    let lang_path = corpus_root.join("languages");
    let languages = ["english", "chinese", "japanese", "korean", "french", 
                     "german", "spanish", "russian", "arabic", "portuguese"];
    
    let mut lang_count = 0usize;
    
    for lang in &languages {
        let lang_dir = lang_path.join(lang);
        
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
                                items.push(sentence.to_string());
                                lang_count += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    eprintln!(" {} 条", lang_count);

    // 对话模态
    eprint!("  💬 对话模态...");
    io::stderr().flush().unwrap();
    let valhalla_root = Path::new("/home/jouly/AI/Valhalla");
    let dialogue_count_before = items.len();
    for filename in ["valhalla_all_dialogues.txt", "valhalla_enhanced_dialogues.txt"] {
        let path = valhalla_root.join(filename);
        if path.exists() {
            if let Ok(file) = File::open(&path) {
                for line in BufReader::new(file).lines().flatten() {
                    if !line.trim().is_empty() {
                        items.push(line);
                    }
                }
            }
        }
    }
    eprintln!(" {} 条", items.len() - dialogue_count_before);

    items
}
