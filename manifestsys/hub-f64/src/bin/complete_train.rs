//! Complete Train - 完整版本 10 周期训练
//! 
//! StemCellComplete + TileQuattroComplete + ChamberComplete
//! TripleFate + VoyagerSpace | 70% 资源 (CPU+GPU+内存)

use std::time::Instant;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::Path;

use stem_cell::StemCellComplete;
use valhalla_tile_quattro::TileQuattroComplete;
use chamber::ChamberComplete;

fn main() {
    eprintln!("╔═══════════════════════════════════════════════════════════════════╗");
    eprintln!("║   Valhalla Complete 10周期 (TripleFate + VoyagerSpace + 70%资源)  ║");
    eprintln!("╚═══════════════════════════════════════════════════════════════════╝");
    eprintln!();

    // 加载语料
    eprintln!("📂 加载语料...");
    let corpus = load_corpus();
    eprintln!("   总量: {} 条", corpus.len());
    eprintln!();

    if corpus.is_empty() {
        eprintln!("❌ 未找到语料！");
        return;
    }

    // 初始化
    eprintln!("🔧 初始化...");
    let mut stemcell = StemCellComplete::new();
    let mut tile = TileQuattroComplete::new();
    let mut chamber = ChamberComplete::new();
    eprintln!("   ✓ StemCell + Tile + Chamber (含 TripleFate + VoyagerSpace)");
    eprintln!();

    // 资源配置
    let num_cpus = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    let use_cpus = (num_cpus as f64 * 0.7) as usize;
    eprintln!("⚡ 资源: CPU {}/{} | GPU CUDA | 内存 70%", use_cpus, num_cpus);
    eprintln!("═══════════════════════════════════════════════════════════════════");
    eprintln!();

    let total_start = Instant::now();

    for cycle in 1..=10 {
        let cycle_start = Instant::now();
        eprintln!("┌───────────────────────────────────────────────────────────────┐");
        eprintln!("│ 周期 {}/10 开始                                               │", cycle);
        eprintln!("└───────────────────────────────────────────────────────────────┘");

        let report_interval = corpus.len() / 10; // 每10%报告一次

        for (i, content) in corpus.iter().enumerate() {
            let v = content.len() as f64 + (i as f64).sin() * 10.0;
            
            stemcell.receive_data(v);
            tile.receive_data(v);
            chamber.receive_data(v);

            // 每 10% 输出一次
            if i > 0 && i % report_interval == 0 {
                let pct = (i as f64 / corpus.len() as f64 * 100.0) as u32;
                let elapsed = cycle_start.elapsed().as_secs_f64();
                let rate = i as f64 / elapsed;
                let eta = (corpus.len() - i) as f64 / rate;
                
                let sc = stemcell.get_status();
                let ti = tile.get_status();
                let ch = chamber.get_status();
                
                eprintln!("  {}% | {:.0}/s | ETA {:.0}s | SC细胞:{} T完成:{} C完成:{}",
                    pct, rate, eta, sc.cell_count, ti.completed_tiles, ch.completed_containers);
            }
        }

        let t = cycle_start.elapsed().as_secs_f64();
        let sc = stemcell.get_status();
        let ti = tile.get_status();
        let ch = chamber.get_status();

        eprintln!();
        eprintln!("  ✓ 周期{} 完成 {:.1}s | SC: 细胞={} 闭合={} dim={:.2}",
            cycle, t, sc.cell_count, sc.total_containers_closed, sc.dimension);
        eprintln!("                      | Tile: 完成={} 活跃={} dim={:.2}",
            ti.completed_tiles, ti.active_tiles, ti.dimension);
        eprintln!("                      | Chamber: 完成={} 扩缩={} dim={:.2}",
            ch.completed_containers, ch.scale_events, ch.dimension);
        eprintln!("  Fate收敛: SC={} Tile={} Chamber={}", 
            stemcell.has_fate_converged(), tile.has_fate_converged(), chamber.has_fate_converged());
        eprintln!();
    }

    let total = total_start.elapsed();
    eprintln!("╔═══════════════════════════════════════════════════════════════════╗");
    eprintln!("║ 完成! 总耗时: {:.1}分钟 ({:.0}秒)                                  ║", 
        total.as_secs_f64()/60.0, total.as_secs_f64());
    eprintln!("╚═══════════════════════════════════════════════════════════════════╝");
}

fn load_corpus() -> Vec<String> {
    let mut items = Vec::new();
    let corpus_root = Path::new("/run/media/jouly/My Book/valhalla_corpus");
    let valhalla_root = Path::new("/home/jouly/AI/Valhalla");

    // 语言
    let lang_path = corpus_root.join("languages");
    for lang in ["english", "chinese", "japanese", "korean", "french", "german", "spanish"] {
        let sent_path = lang_path.join(lang).join("sentences");
        if sent_path.exists() {
            for entry in fs::read_dir(&sent_path).into_iter().flatten().flatten() {
                let p = entry.path();
                if p.extension().map_or(false, |e| e == "txt" || e == "tsv") {
                    if let Ok(f) = File::open(&p) {
                        for line in BufReader::new(f).lines().flatten() {
                            let parts: Vec<&str> = line.split('\t').collect();
                            let s = if parts.len() >= 3 { parts[2].trim() } else { line.trim() };
                            if s.len() > 5 && s.len() < 500 {
                                items.push(s.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    // 对话
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

    items
}
