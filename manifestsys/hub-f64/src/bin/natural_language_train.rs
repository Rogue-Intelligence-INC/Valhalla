//! 自然语言训练
//! 
//! 只用自然语言数据，不用代码：
//! - LibriSpeech 英语转录
//! - 中文维基百科

use std::time::Instant;
use std::path::Path;
use std::io::{BufRead, BufReader};
use std::fs::File;

use stem_cell::StemCellComplete;

const TRAINING_DATA_BASE: &str = "/run/media/jouly/My Book/valhalla_training_data";

fn main() {
    println!("╔════════════════════════════════════════════════════════════════════════════╗");
    println!("║          🗣️ 自然语言训练 - 只用对话/文本数据                                ║");
    println!("╚════════════════════════════════════════════════════════════════════════════╝");
    println!();

    // ========== 加载自然语言语料 ==========
    println!("📚 加载自然语言语料...");
    let corpus_start = Instant::now();
    let corpus = load_natural_language_corpus();
    let total_bytes: usize = corpus.iter().map(|(_, d)| d.len()).sum();
    println!("   文件数: {}", corpus.len());
    println!("   总大小: {:.2} MB", total_bytes as f64 / 1024.0 / 1024.0);
    println!("   耗时: {:?}", corpus_start.elapsed());
    println!();

    // 显示样例
    println!("📝 语料样例:");
    for (i, (name, content)) in corpus.iter().take(3).enumerate() {
        let preview = String::from_utf8_lossy(content);
        let short: String = preview.chars().take(100).collect();
        println!("   {}. {}: \"{}...\"", i+1, name, short.replace('\n', " "));
    }
    println!();

    // ========== 创建系统 ==========
    let mut stemcell = StemCellComplete::new();
    println!("🧬 初始状态: {} 细胞", stemcell.cells.len());
    println!();

    // ========== 10 轮训练 ==========
    let total_start = Instant::now();
    
    for round in 1..=10 {
        println!("═══════════════════════════════════════════════════════════════════════════");
        println!("                         第 {} 轮训练", round);
        println!("═══════════════════════════════════════════════════════════════════════════");
        
        let round_start = Instant::now();
        let mut ops = 0u64;
        let mut stored = 0usize;
        
        for (idx, (_name, bytes)) in corpus.iter().enumerate() {
            // 存储原始数据到 Fate
            let _id = stemcell.fate_store(bytes);
            stored += 1;
            
            // 传递数值信号
            for &b in bytes.iter() {
                stemcell.receive_data(b as f64);
                ops += 1;
            }
            
            // 进度
            if (idx + 1) % 100 == 0 || idx + 1 == corpus.len() {
                let elapsed = round_start.elapsed();
                let ops_per_sec = ops as f64 / elapsed.as_secs_f64();
                print!("\r   进度: {}/{} ({:.1}%), {:.1}M ops/sec, {} 存储    ",
                    idx + 1, corpus.len(),
                    (idx + 1) as f64 / corpus.len() as f64 * 100.0,
                    ops_per_sec / 1_000_000.0,
                    stored);
                std::io::Write::flush(&mut std::io::stdout()).ok();
            }
        }
        
        // 演化
        stemcell.evolve(0.5);
        
        let round_time = round_start.elapsed();
        println!();
        println!("   第 {} 轮完成 ({:?}):", round, round_time);
        println!("   ├─ 细胞数: {}", stemcell.cells.len());
        println!("   ├─ Fate 存储: {}", stemcell.triple_fate.stored_count());
        println!("   └─ 总连接: {}", stemcell.cells.iter().map(|c| c.connections.len()).sum::<usize>());
        println!();
    }
    
    let total_time = total_start.elapsed();
    
    // ========== 保存 ==========
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("                         训练完成");
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!();
    println!("📊 总结:");
    println!("   总耗时: {:?}", total_time);
    println!("   细胞数: {}", stemcell.cells.len());
    println!("   Fate 存储: {}", stemcell.triple_fate.stored_count());
    println!("   连接数: {}", stemcell.cells.iter().map(|c| c.connections.len()).sum::<usize>());
    println!();
    
    // 存储样例
    println!("📦 存储内容样例:");
    for id in stemcell.triple_fate.list_stored().iter().take(5) {
        if let Some(entry) = stemcell.triple_fate.load_data(id) {
            let preview = String::from_utf8_lossy(&entry.raw);
            let short: String = preview.chars().take(80).collect();
            println!("   - \"{}\": \"{}...\"", id, short.replace('\n', " "));
        }
    }
    println!();
    
    // 保存
    let save_path = Path::new("/tmp/natural_language_stemcell.stemcell");
    match stemcell.save_slice(save_path) {
        Ok(()) => {
            let size = std::fs::metadata(save_path).map(|m| m.len()).unwrap_or(0);
            println!("💾 已保存: {} ({:.2} MB)", save_path.display(), size as f64 / 1024.0 / 1024.0);
        }
        Err(e) => println!("❌ 保存失败: {}", e),
    }
    
    println!();
    println!("╔════════════════════════════════════════════════════════════════════════════╗");
    println!("║  ✅ 自然语言训练完成                                                        ║");
    println!("╚════════════════════════════════════════════════════════════════════════════╝");
}

/// 加载自然语言语料（不包含代码）
fn load_natural_language_corpus() -> Vec<(String, Vec<u8>)> {
    let mut corpus = Vec::new();
    
    // 1. LibriSpeech 英语转录
    println!("   加载 LibriSpeech 转录...");
    let libri_path = Path::new(TRAINING_DATA_BASE).join("audio/LibriSpeech");
    load_transcripts_recursive(&libri_path, &mut corpus, 0);
    println!("   LibriSpeech: {} 文件", corpus.len());
    
    // 2. 提取中文维基百科（只取前 10MB）
    println!("   加载中文维基百科...");
    let wiki_path = Path::new(TRAINING_DATA_BASE).join("text/zhwiki-latest.xml.bz2");
    if wiki_path.exists() {
        // 只读取前 10MB 的压缩数据作为样本
        if let Ok(data) = std::fs::read(&wiki_path) {
            let sample_size = 10_000_000.min(data.len()); // 10MB
            corpus.push(("zhwiki_sample".to_string(), data[..sample_size].to_vec()));
            println!("   中文维基: {:.2} MB 样本", sample_size as f64 / 1024.0 / 1024.0);
        }
    }
    
    corpus
}

/// 递归加载转录文件
fn load_transcripts_recursive(path: &Path, corpus: &mut Vec<(String, Vec<u8>)>, depth: usize) {
    if depth > 10 { return; }
    
    let Ok(entries) = std::fs::read_dir(path) else { return };
    
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        
        if name.starts_with('.') { continue; }
        
        if path.is_dir() {
            load_transcripts_recursive(&path, corpus, depth + 1);
        } else if name.ends_with(".trans.txt") {
            // 读取转录文件并提取纯文本
            if let Ok(file) = File::open(&path) {
                let reader = BufReader::new(file);
                let mut text = String::new();
                
                for line in reader.lines().flatten() {
                    // 格式: ID TEXT
                    if let Some(pos) = line.find(' ') {
                        text.push_str(&line[pos+1..]);
                        text.push('\n');
                    }
                }
                
                if !text.is_empty() {
                    corpus.push((name.to_string(), text.into_bytes()));
                }
            }
        }
    }
}
