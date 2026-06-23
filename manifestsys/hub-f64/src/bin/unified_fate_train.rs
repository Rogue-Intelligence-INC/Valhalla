//! 统一 Fate 存储训练
//!
//! 同时启用 StemCell、Tile、Hub 的 Fate 存储功能
//! 语料数据真正存入 Fate，而不只是传递数值信号

use std::time::Instant;
use std::path::Path;

use stem_cell::StemCellComplete;
use valhalla_tile_quattro::TileQuattroComplete;
use hub_f64::ValhallaHub;

const TRAINING_DATA_BASE: &str = "/run/media/jouly/My Book/valhalla_training_data";

fn main() {
    println!("╔════════════════════════════════════════════════════════════════════════════╗");
    println!("║          🔮 统一 Fate 存储训练 - StemCell + Tile + Hub                      ║");
    println!("║          语料真正存入 Fate，启用完整存储功能                                 ║");
    println!("╚════════════════════════════════════════════════════════════════════════════╝");
    println!();

    // ========== 加载语料 ==========
    println!("📚 加载训练语料...");
    let corpus_start = Instant::now();
    let corpus = load_training_corpus();
    let total_bytes: usize = corpus.iter().map(|(_, d)| d.len()).sum();
    println!("   文件数: {}", corpus.len());
    println!("   总大小: {:.2} MB", total_bytes as f64 / 1024.0 / 1024.0);
    println!("   耗时: {:?}", corpus_start.elapsed());
    println!();

    // ========== 创建系统 ==========
    println!("🧬 初始化系统...");
    let mut stemcell = StemCellComplete::new();
    let mut tile = TileQuattroComplete::new();
    let mut hub = ValhallaHub::new();
    
    println!("   StemCell: {} 细胞, Fate tick {}", 
        stemcell.get_status().cell_count,
        stemcell.triple_fate.global_tick);
    println!("   Tile: {} 完成, Fate tick {}", 
        tile.get_status().completed_tiles,
        tile.triple_fate.global_tick);
    println!("   Hub: Fate tick {}", 
        hub.triple_fate.global_tick);
    println!();

    // ========== 10 轮训练 ==========
    let total_start = Instant::now();
    
    for round in 1..=10 {
        println!("═══════════════════════════════════════════════════════════════════════════");
        println!("                         第 {} 轮训练", round);
        println!("═══════════════════════════════════════════════════════════════════════════");
        
        let round_start = Instant::now();
        let mut stem_ops = 0u64;
        let mut tile_ops = 0u64;
        let mut hub_ops = 0u64;
        let mut stored_count = 0usize;
        
        for (idx, (filename, bytes)) in corpus.iter().enumerate() {
            // ═══════════════════════════════════════════════════════════════════
            // 关键：将原始数据存入 Fate（而不只是传递数值）
            // ═══════════════════════════════════════════════════════════════════
            
            // 1. StemCell: 存储 + 信号
            let _stem_id = stemcell.fate_store(bytes);
            for &b in bytes.iter() {
                stemcell.receive_data(b as f64);
                stem_ops += 1;
            }
            
            // 2. Tile: 存储 + 信号
            let _tile_id = tile.fate_store(bytes);
            for &b in bytes.iter() {
                tile.receive_data(b as f64);
                tile_ops += 1;
            }
            
            // 3. Hub: 存储 + 信号
            let _hub_id = hub.fate_store(bytes);
            for &b in bytes.iter() {
                hub.triple_fate.receive_passive(b as f64);
                hub_ops += 1;
            }
            
            stored_count += 3; // 每个文件存入3个系统
            
            // 每 200 个文件输出进度
            if (idx + 1) % 200 == 0 || idx + 1 == corpus.len() {
                let elapsed = round_start.elapsed();
                let total_ops = stem_ops + tile_ops + hub_ops;
                let ops_per_sec = total_ops as f64 / elapsed.as_secs_f64();
                print!("\r   进度: {}/{} ({:.1}%), {:.1}M ops/sec, {} 存储条目    ",
                    idx + 1, corpus.len(),
                    (idx + 1) as f64 / corpus.len() as f64 * 100.0,
                    ops_per_sec / 1_000_000.0,
                    stored_count);
                std::io::Write::flush(&mut std::io::stdout()).ok();
            }
        }
        
        // 每轮后演化
        stemcell.evolve(0.5);
        tile.process_tiles_cycle();
        
        let round_time = round_start.elapsed();
        println!();
        println!();
        println!("   第 {} 轮完成 ({:?}):", round, round_time);
        println!("   ┌─ StemCell:");
        println!("   │  ├─ 操作数: {} ({:.1}M ops/sec)", stem_ops, stem_ops as f64 / round_time.as_secs_f64() / 1e6);
        println!("   │  ├─ 细胞数: {}", stemcell.get_status().cell_count);
        println!("   │  ├─ Fate 存储: {}", stemcell.triple_fate.stored_count());
        println!("   │  └─ Fate tick: {}", stemcell.triple_fate.global_tick);
        println!("   │");
        println!("   ├─ Tile:");
        println!("   │  ├─ 操作数: {}", tile_ops);
        println!("   │  ├─ 完成数: {}", tile.get_status().completed_tiles);
        println!("   │  ├─ Fate 存储: {}", tile.triple_fate.stored_count());
        println!("   │  └─ Fate tick: {}", tile.triple_fate.global_tick);
        println!("   │");
        println!("   └─ Hub:");
        println!("      ├─ 操作数: {}", hub_ops);
        println!("      ├─ Fate 存储: {}", hub.triple_fate.stored_count());
        println!("      └─ Fate tick: {}", hub.triple_fate.global_tick);
        println!();
        
        // 显示方程系数
        println!("   📐 Fate 方程状态:");
        println!("   ├─ StemCell: {}", stemcell.triple_fate.get_equation());
        println!("   ├─ Tile: {}", tile.triple_fate.get_equation());
        println!("   └─ Hub: {}", hub.triple_fate.get_equation());
        println!();
    }
    
    let total_time = total_start.elapsed();
    
    // ========== 最终状态 ==========
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("                         训练完成");
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!();
    println!("📊 训练总结:");
    println!("   总耗时: {:?}", total_time);
    println!("   语料: {} 文件 x 10 轮 = {} 次处理", corpus.len(), corpus.len() * 10);
    println!("   吞吐量: {:.2} MB/sec", 
        (total_bytes * 10 * 3) as f64 / 1024.0 / 1024.0 / total_time.as_secs_f64());
    println!();
    
    // ========== Fate 存储验证 ==========
    println!("📦 Fate 存储验证:");
    println!();
    println!("   StemCell ({} 条):", stemcell.triple_fate.stored_count());
    print_fate_samples(&stemcell.triple_fate, 3);
    println!();
    println!("   Tile ({} 条):", tile.triple_fate.stored_count());
    print_fate_samples(&tile.triple_fate, 3);
    println!();
    println!("   Hub ({} 条):", hub.triple_fate.stored_count());
    print_fate_samples(&hub.triple_fate, 3);
    println!();
    
    // ========== 保存完整系统切片（包含所有细胞物理数据）==========
    println!("💾 保存完整系统切片（含细胞物理数据）:");
    
    let stem_path = Path::new("/tmp/unified_stemcell.stemcell");
    let tile_path = Path::new("/tmp/unified_tile.tile");
    let hub_path = Path::new("/tmp/unified_hub.hub");
    
    // StemCell 完整保存
    match stemcell.save_slice(stem_path) {
        Ok(()) => {
            let size = std::fs::metadata(stem_path).map(|m| m.len()).unwrap_or(0);
            println!("   ✓ StemCell: {} ({} bytes, {} 细胞)", 
                stem_path.display(), size, stemcell.cells.len());
        }
        Err(e) => println!("   ✗ StemCell: {}", e),
    }
    
    // Tile 完整保存
    match tile.save_slice(tile_path) {
        Ok(()) => {
            let size = std::fs::metadata(tile_path).map(|m| m.len()).unwrap_or(0);
            println!("   ✓ Tile: {} ({} bytes)", tile_path.display(), size);
        }
        Err(e) => println!("   ✗ Tile: {}", e),
    }
    
    // Hub 完整保存
    match hub.save_slice(hub_path) {
        Ok(()) => {
            let size = std::fs::metadata(hub_path).map(|m| m.len()).unwrap_or(0);
            println!("   ✓ Hub: {} ({} bytes)", hub_path.display(), size);
        }
        Err(e) => println!("   ✗ Hub: {}", e),
    }
    
    println!();
    println!("╔════════════════════════════════════════════════════════════════════════════╗");
    println!("║  ✅ 训练完成 - Fate 存储已启用                                              ║");
    println!("║  语料已真正存入各系统，可用于后续交互                                        ║");
    println!("╚════════════════════════════════════════════════════════════════════════════╝");
}

fn print_fate_samples(fate: &fate_f64::TripleFateCombined, count: usize) {
    let ids = fate.list_stored();
    for id in ids.iter().take(count) {
        if let Some(entry) = fate.load_data(id) {
            let preview = if entry.raw.len() > 60 {
                format!("{}...", String::from_utf8_lossy(&entry.raw[..60]).replace('\n', " "))
            } else {
                String::from_utf8_lossy(&entry.raw).replace('\n', " ")
            };
            println!("   - {}: {} bytes, sig={:.4}, \"{}\"",
                id, entry.raw.len(), entry.signature, preview);
        }
    }
}

/// 加载训练语料
fn load_training_corpus() -> Vec<(String, Vec<u8>)> {
    let mut corpus = Vec::new();
    
    // 1. LibriSpeech 转录
    let libri_path = Path::new(TRAINING_DATA_BASE).join("audio/LibriSpeech");
    load_files_recursive(&libri_path, &mut corpus, 0, ".trans.txt", 1000);
    println!("   LibriSpeech: {} 文件", corpus.len());
    let libri_count = corpus.len();
    
    // 2. COCO 标注
    let coco_path = Path::new(TRAINING_DATA_BASE).join("images/annotations");
    for name in &["captions_train2017.json", "captions_val2017.json"] {
        let path = coco_path.join(name);
        if let Ok(content) = std::fs::read(&path) {
            // 限制大小
            let limited = if content.len() > 10_000_000 {
                content[..10_000_000].to_vec()
            } else {
                content
            };
            corpus.push((path.display().to_string(), limited));
        }
    }
    println!("   COCO: {} 文件", corpus.len() - libri_count);
    let coco_count = corpus.len();
    
    // 3. Valhalla 源码
    load_files_recursive(Path::new("/home/jouly/AI/Valhalla"), &mut corpus, 0, ".rs", 500);
    println!("   Valhalla: {} 文件", corpus.len() - coco_count);
    let valhalla_count = corpus.len();
    
    // 4. Cargo Registry
    load_files_recursive(Path::new("/home/jouly/.cargo/registry/src"), &mut corpus, 0, ".rs", 1000);
    println!("   Cargo: {} 文件", corpus.len() - valhalla_count);
    
    corpus
}

fn load_files_recursive(path: &Path, corpus: &mut Vec<(String, Vec<u8>)>, depth: usize, ext: &str, max: usize) {
    if depth > 10 || corpus.len() >= max { return; }
    
    let Ok(entries) = std::fs::read_dir(path) else { return };
    
    for entry in entries.flatten() {
        if corpus.len() >= max { return; }
        
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        
        if name.starts_with('.') || name == "target" || name == "node_modules" {
            continue;
        }
        
        if path.is_dir() {
            load_files_recursive(&path, corpus, depth + 1, ext, max);
        } else if name.ends_with(ext) {
            if let Ok(content) = std::fs::read(&path) {
                // 限制单文件大小
                let limited = if content.len() > 100_000 {
                    content[..100_000].to_vec()
                } else {
                    content
                };
                corpus.push((path.display().to_string(), limited));
            }
        }
    }
}
