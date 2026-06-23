//! Tile 和 Hub 自主创建能力测试
//!
//! 使用相同的训练逻辑，分别测试两个系统

use std::path::Path;
use std::time::Instant;

use valhalla_tile_quattro::TileQuattroComplete;
use hub_f64::ValhallaHub;

fn load_corpus() -> Vec<Vec<u8>> {
    let mut corpus = Vec::new();
    
    // 英文 - LibriSpeech (遍历子目录)
    let libri_path = Path::new("/run/media/jouly/My Book/valhalla_training_data/audio/LibriSpeech/train-clean-100");
    if libri_path.exists() {
        let mut count = 0;
        fn visit_dir(path: &Path, corpus: &mut Vec<Vec<u8>>, count: &mut usize, max: usize) {
            if *count >= max {
                return;
            }
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    if *count >= max {
                        return;
                    }
                    let p = entry.path();
                    if p.is_dir() {
                        visit_dir(&p, corpus, count, max);
                    } else if p.extension().map(|x| x == "txt").unwrap_or(false) {
                        if let Ok(content) = std::fs::read_to_string(&p) {
                            corpus.push(content.into_bytes());
                            *count += 1;
                        }
                    }
                }
            }
        }
        visit_dir(libri_path, &mut corpus, &mut count, 200);
        println!("   [英文] {} 条", count);
    }
    
    // 中文 - 维基百科 XML (分块读取)
    let wiki_path = Path::new("/run/media/jouly/My Book/valhalla_training_data/text/zhwiki-latest.xml");
    if wiki_path.exists() {
        let mut count = 0;
        if let Ok(content) = std::fs::read(wiki_path) {
            // 分块，每块 50KB
            for chunk in content.chunks(50000) {
                corpus.push(chunk.to_vec());
                count += 1;
                if count >= 500 {
                    break;
                }
            }
        }
        println!("   [中文] {} 块", count);
    }
    
    // 图像 - COCO train2017
    let coco_path = Path::new("/run/media/jouly/My Book/valhalla_training_data/images/train2017");
    if coco_path.exists() {
        let mut count = 0;
        for entry in std::fs::read_dir(coco_path).into_iter().flatten().flatten() {
            if count >= 200 {
                break;
            }
            if let Ok(data) = std::fs::read(&entry.path()) {
                corpus.push(data);
                count += 1;
            }
        }
        println!("   [图像] {} 张", count);
    }
    
    corpus
}

fn train_tile(corpus: &[Vec<u8>], rounds: usize) {
    println!();
    println!("╔════════════════════════════════════════════════════════════════════════════╗");
    println!("║                     🔷 Tile 系统训练                                        ║");
    println!("╚════════════════════════════════════════════════════════════════════════════╝");
    
    let mut tile = TileQuattroComplete::new();
    let initial_spontaneous = tile.triple_fate.spontaneous_create_count;
    
    println!("   初始状态: {} Tile, {} 存储", tile.completed_tiles.len(), tile.fate_count());
    
    let start = Instant::now();
    
    for round in 1..=rounds {
        let round_start = Instant::now();
        
        for data in corpus {
            // 存储到 Fate
            tile.fate_store(data);
            
            // 信号传递（采样）
            let step = (data.len() / 30).max(1);
            for j in (0..data.len()).step_by(step) {
                tile.receive_data(data[j] as f64);
            }
        }
        
        // 演化
        tile.process_tiles_cycle();
        
        let spontaneous = tile.triple_fate.spontaneous_create_count - initial_spontaneous;
        println!("   第{:02}轮 ({:?}): {} Tile, {} 存储, {} 自主创建",
            round, round_start.elapsed(),
            tile.completed_tiles.len(), tile.fate_count(), spontaneous);
    }
    
    println!();
    println!("   ═══ Tile 训练完成 ═══");
    println!("   耗时: {:?}", start.elapsed());
    println!("   Tile 数: {}", tile.completed_tiles.len());
    println!("   存储数: {}", tile.fate_count());
    println!("   自主创建: {} 条", tile.triple_fate.spontaneous_create_count - initial_spontaneous);
    println!("   方程: {:.4}x² + {:.4}x + {:.4}",
        tile.triple_fate.cluster_equation.coef_a,
        tile.triple_fate.cluster_equation.coef_b,
        tile.triple_fate.cluster_equation.coef_c);
    
    // 保存
    let _ = tile.save_slice(Path::new("/tmp/tile_train.tile"));
}

fn train_hub(corpus: &[Vec<u8>], rounds: usize) {
    println!();
    println!("╔════════════════════════════════════════════════════════════════════════════╗");
    println!("║                     🔶 Hub 系统训练                                         ║");
    println!("╚════════════════════════════════════════════════════════════════════════════╝");
    
    let mut hub = ValhallaHub::new();
    let initial_spontaneous = hub.spontaneous_count();
    
    println!("   初始状态: {} 存储", hub.fate_count());
    
    let start = Instant::now();
    
    for round in 1..=rounds {
        let round_start = Instant::now();
        
        for data in corpus {
            // 存储到 Fate
            hub.fate_store(data);
            
            // 信号传递（采样）
            let step = (data.len() / 30).max(1);
            for j in (0..data.len()).step_by(step) {
                hub.receive_data(data[j] as f64);
            }
        }
        
        // 演化
        hub.fate_tick();
        
        let spontaneous = hub.spontaneous_count() - initial_spontaneous;
        println!("   第{:02}轮 ({:?}): {} 存储, {} 自主创建",
            round, round_start.elapsed(),
            hub.fate_count(), spontaneous);
    }
    
    println!();
    println!("   ═══ Hub 训练完成 ═══");
    println!("   耗时: {:?}", start.elapsed());
    println!("   存储数: {}", hub.fate_count());
    println!("   自主创建: {} 条", hub.spontaneous_count() - initial_spontaneous);
    println!("   方程: {:.4}x² + {:.4}x + {:.4}",
        hub.triple_fate.cluster_equation.coef_a,
        hub.triple_fate.cluster_equation.coef_b,
        hub.triple_fate.cluster_equation.coef_c);
    
    // 保存
    let _ = hub.save_slice(Path::new("/tmp/hub_train.hub"));
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════════════════════╗");
    println!("║              🧪 Tile/Hub 自主创建能力对比测试                               ║");
    println!("╚════════════════════════════════════════════════════════════════════════════╝");
    println!();
    
    println!("📚 加载语料...");
    let corpus = load_corpus();
    let total_bytes: usize = corpus.iter().map(|d| d.len()).sum();
    println!("   总计: {} 条, {:.2} MB", corpus.len(), total_bytes as f64 / 1024.0 / 1024.0);
    
    let rounds = 10;
    
    // 分别训练
    train_tile(&corpus, rounds);
    train_hub(&corpus, rounds);
    
    println!();
    println!("╔════════════════════════════════════════════════════════════════════════════╗");
    println!("║                     ✅ 测试完成                                            ║");
    println!("╚════════════════════════════════════════════════════════════════════════════╝");
}
