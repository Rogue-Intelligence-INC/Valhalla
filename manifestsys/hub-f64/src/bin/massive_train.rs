//! 大规模训练 - 观察自主涌现
//!
//! 目标：训练到系统自动形成记忆和理解
//! 观察：系统是否自主产生新数据

use std::time::Instant;
use std::path::Path;
use std::io::{BufRead, BufReader, Read};
use std::fs::File;

use stem_cell::StemCellComplete;

const DATA_BASE: &str = "/run/media/jouly/My Book/valhalla_training_data";

fn main() {
    println!("╔════════════════════════════════════════════════════════════════════════════╗");
    println!("║              🚀 大规模训练 - 观察涌现                                        ║");
    println!("╚════════════════════════════════════════════════════════════════════════════╝");
    println!();

    // ========== 加载大量数据 ==========
    println!("📚 加载语料（大规模）...");
    let start = Instant::now();
    let mut corpus: Vec<Vec<u8>> = Vec::new();
    
    // 1. 全部 LibriSpeech 转录
    println!("   [英文] LibriSpeech...");
    let en_count = load_all_transcripts(&mut corpus);
    println!("          {} 条", en_count);
    
    // 2. 中文维基（更多）
    println!("   [中文] 维基百科...");
    let zh_count = load_zhwiki_chunks(&mut corpus, 2000);
    println!("          {} 块", zh_count);
    
    // 3. 图像
    println!("   [图像] COCO...");
    let img_count = load_images(&mut corpus, 500);
    println!("          {} 张", img_count);
    
    let total_bytes: usize = corpus.iter().map(|d| d.len()).sum();
    println!();
    println!("   总计: {} 条, {:.2} MB", corpus.len(), total_bytes as f64 / 1024.0 / 1024.0);
    println!("   加载耗时: {:?}", start.elapsed());
    println!();

    // ========== 加载或创建系统 ==========
    let slice_path = Path::new("/tmp/massive_stemcell.stemcell");
    let mut stemcell = if slice_path.exists() {
        println!("📥 继续之前的训练...");
        match StemCellComplete::load_slice(slice_path) {
            Ok(s) => {
                println!("   已有: {} 细胞, {} 存储", s.cells.len(), s.triple_fate.stored_count());
                s
            }
            Err(_) => StemCellComplete::new()
        }
    } else {
        println!("🆕 从头开始...");
        StemCellComplete::new()
    };
    
    let initial_stored = stemcell.triple_fate.stored_count();
    let initial_internal = count_internal_data(&stemcell);
    
    println!();
    println!("   初始状态:");
    println!("   ├─ 细胞: {}", stemcell.cells.len());
    println!("   ├─ 外部存储: {}", initial_stored);
    println!("   └─ 内部创建: {}", initial_internal);
    println!();

    // ========== 训练循环 ==========
    let rounds = 50; // 更多轮次
    let train_start = Instant::now();
    
    for round in 1..=rounds {
        let round_start = Instant::now();
        
        // 打乱顺序（增加多样性）
        let mut indices: Vec<usize> = (0..corpus.len()).collect();
        shuffle(&mut indices, round as u64);
        
        for (i, &idx) in indices.iter().enumerate() {
            let data = &corpus[idx];
            
            // 存储到 Fate
            stemcell.fate_store(data);
            
            // 批量信号传递（加速：采样而非全部）
            let step = (data.len() / 50).max(1);
            for j in (0..data.len()).step_by(step) {
                stemcell.receive_data(data[j] as f64);
            }
            
            // 进度
            if (i + 1) % 500 == 0 {
                print!("\r   第{:02}轮: {}/{}", round, i + 1, corpus.len());
                std::io::Write::flush(&mut std::io::stdout()).ok();
            }
        }
        
        // 演化
        stemcell.evolve(0.5);
        
        // 统计
        let stored = stemcell.triple_fate.stored_count();
        let internal = count_internal_data(&stemcell);
        let new_internal = internal - initial_internal;
        let connections: usize = stemcell.cells.iter().map(|c| c.connections.len()).sum();
        
        println!("\r   第{:02}轮 ({:?}): {} 细胞, {} 存储, {} 新创建, {} 连接",
            round, round_start.elapsed(),
            stemcell.cells.len(), stored, new_internal, connections);
        
        // 关键观察：系统是否自主产生了新数据？
        if new_internal > 0 {
            println!("   ⚡ 检测到系统自主创建了 {} 条新数据!", new_internal);
        }
        
        // 每10轮保存一次
        if round % 10 == 0 {
            println!("   💾 保存检查点...");
            let _ = stemcell.save_slice(slice_path);
            
            // 分析当前状态
            analyze_emergence(&stemcell, round);
        }
    }
    
    // ========== 最终分析 ==========
    println!();
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("                         训练完成");
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!();
    println!("   总耗时: {:?}", train_start.elapsed());
    println!("   细胞数: {}", stemcell.cells.len());
    println!("   存储: {}", stemcell.triple_fate.stored_count());
    println!("   连接: {}", stemcell.cells.iter().map(|c| c.connections.len()).sum::<usize>());
    
    let final_internal = count_internal_data(&stemcell);
    println!("   系统自主创建: {} 条", final_internal - initial_internal);
    println!();
    
    // 保存
    let _ = stemcell.save_slice(slice_path);
    if let Ok(meta) = std::fs::metadata(slice_path) {
        println!("   💾 已保存: {:.2} MB", meta.len() as f64 / 1024.0 / 1024.0);
    }
}

/// 统计系统内部创建的数据
fn count_internal_data(stemcell: &StemCellComplete) -> usize {
    let mut count = 0;
    for id in stemcell.triple_fate.list_stored() {
        if let Some(entry) = stemcell.triple_fate.load_data(id) {
            if entry.origin == "internal" {
                count += 1;
            }
        }
    }
    count
}

/// 分析涌现特性
fn analyze_emergence(stemcell: &StemCellComplete, round: usize) {
    println!();
    println!("   === 第{}轮涌现分析 ===", round);
    
    // 1. 细胞分化
    let energies: Vec<f64> = stemcell.cells.iter().map(|c| c.energy).collect();
    let mean = energies.iter().sum::<f64>() / energies.len() as f64;
    let var = energies.iter().map(|e| (e - mean).powi(2)).sum::<f64>() / energies.len() as f64;
    println!("   能量分布: 均值={:.4}, 方差={:.4}", mean, var);
    
    // 2. 连接度
    let conn_counts: Vec<usize> = stemcell.cells.iter().map(|c| c.connections.len()).collect();
    let max_conn = *conn_counts.iter().max().unwrap_or(&0);
    let avg_conn = conn_counts.iter().sum::<usize>() as f64 / conn_counts.len() as f64;
    println!("   连接: 最大={}, 平均={:.1}", max_conn, avg_conn);
    
    // 3. 检查是否有超级枢纽
    let hub_threshold = (avg_conn * 3.0) as usize;
    let hub_count = stemcell.cells.iter()
        .filter(|c| c.connections.len() > hub_threshold)
        .count();
    if hub_count > 0 {
        println!("   ⭐ 发现 {} 个超级枢纽细胞!", hub_count);
    }
    
    // 4. Fate 收敛状态
    if stemcell.triple_fate.is_passively_converged() {
        println!("   ✓ 系统已收敛");
    }
    
    println!();
}

/// 简单打乱
fn shuffle(arr: &mut [usize], seed: u64) {
    let n = arr.len();
    let mut state = seed;
    for i in (1..n).rev() {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let j = (state as usize) % (i + 1);
        arr.swap(i, j);
    }
}

// ========== 数据加载函数 ==========

fn load_all_transcripts(corpus: &mut Vec<Vec<u8>>) -> usize {
    let libri_path = Path::new(DATA_BASE).join("audio/LibriSpeech");
    let mut count = 0;
    
    fn walk(path: &Path, corpus: &mut Vec<Vec<u8>>, count: &mut usize) {
        let Ok(entries) = std::fs::read_dir(path) else { return };
        
        for entry in entries.flatten() {
            let p = entry.path();
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.starts_with('.') { continue; }
            
            if p.is_dir() {
                walk(&p, corpus, count);
            } else if name.ends_with(".trans.txt") {
                if let Ok(file) = File::open(&p) {
                    let reader = BufReader::new(file);
                    let mut text = String::new();
                    for line in reader.lines().flatten() {
                        if let Some(pos) = line.find(' ') {
                            text.push_str(&line[pos+1..]);
                            text.push('\n');
                        }
                    }
                    if !text.is_empty() {
                        corpus.push(text.into_bytes());
                        *count += 1;
                    }
                }
            }
        }
    }
    
    walk(&libri_path, corpus, &mut count);
    count
}

fn load_zhwiki_chunks(corpus: &mut Vec<Vec<u8>>, max_chunks: usize) -> usize {
    let wiki_xml = Path::new(DATA_BASE).join("text/zhwiki-latest.xml");
    
    let data = if wiki_xml.exists() {
        let mut file = match File::open(&wiki_xml) {
            Ok(f) => f,
            Err(_) => return 0,
        };
        // 读取更多（200MB）
        let mut buf = vec![0u8; 200_000_000];
        let n = file.read(&mut buf).unwrap_or(0);
        buf.truncate(n);
        buf
    } else {
        return 0;
    };
    
    let chunk_size = 100_000;
    let mut count = 0;
    
    for chunk in data.chunks(chunk_size) {
        if count >= max_chunks { break; }
        corpus.push(chunk.to_vec());
        count += 1;
    }
    
    count
}

fn load_images(corpus: &mut Vec<Vec<u8>>, max_count: usize) -> usize {
    let img_path = Path::new(DATA_BASE).join("images/train2017");
    let mut count = 0;
    
    let Ok(entries) = std::fs::read_dir(&img_path) else { return 0 };
    
    for entry in entries.flatten() {
        if count >= max_count { break; }
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("jpg") {
            if let Ok(data) = std::fs::read(&path) {
                corpus.push(data);
                count += 1;
            }
        }
    }
    
    count
}
