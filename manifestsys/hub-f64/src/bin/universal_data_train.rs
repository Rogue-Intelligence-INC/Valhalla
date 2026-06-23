//! 全数据类型训练
//! 
//! 不预设，让 Fate 自己发现结构：
//! - 文本（英语、中文）
//! - 图像（JPEG 原始字节）
//! - 音频转录
//! 所有数据都是字节流，Fate 计算签名

use std::time::Instant;
use std::path::Path;
use std::io::{BufRead, BufReader, Read};
use std::fs::File;

use stem_cell::StemCellComplete;

const DATA_BASE: &str = "/run/media/jouly/My Book/valhalla_training_data";

fn main() {
    println!("╔════════════════════════════════════════════════════════════════════════════╗");
    println!("║         🌍 全数据类型训练                                                   ║");
    println!("║         文本 + 图像 + 音频 → Fate 自己找结构                                ║");
    println!("╚════════════════════════════════════════════════════════════════════════════╝");
    println!();

    let start = Instant::now();
    
    // ========== 加载所有类型数据 ==========
    println!("📚 加载数据...");
    let mut corpus: Vec<(String, Vec<u8>)> = Vec::new();
    
    // 1. 英语文本 (LibriSpeech 转录)
    println!("   [文本] LibriSpeech 英语转录...");
    let libri_count = load_transcripts(&mut corpus, 500);
    println!("          {} 条", libri_count);
    
    // 2. 中文文本 (zhwiki)
    println!("   [文本] 中文维基...");
    let wiki_count = load_zhwiki(&mut corpus, 500);
    println!("          {} 条", wiki_count);
    
    // 3. 图像 (COCO)
    println!("   [图像] COCO train2017...");
    let img_count = load_images(&mut corpus, 200);
    println!("          {} 张", img_count);
    
    let total_bytes: usize = corpus.iter().map(|(_, d)| d.len()).sum();
    println!();
    println!("   总计: {} 条数据, {:.2} MB", corpus.len(), total_bytes as f64 / 1024.0 / 1024.0);
    println!("   加载耗时: {:?}", start.elapsed());
    println!();

    // 显示样本
    println!("📝 数据样本:");
    for (name, data) in corpus.iter().take(6) {
        let type_tag = if name.ends_with(".jpg") { "[IMG]" } 
                       else if name.contains("zhwiki") { "[中文]" }
                       else { "[英文]" };
        let preview = if name.ends_with(".jpg") {
            format!("{} bytes, JPEG header: {:02X?}", data.len(), &data[..16.min(data.len())])
        } else {
            let text = String::from_utf8_lossy(data);
            let short: String = text.chars().take(60).collect();
            format!("\"{}...\"", short.replace('\n', " "))
        };
        println!("   {} {}: {}", type_tag, name, preview);
    }
    println!();

    // ========== 创建系统并训练 ==========
    let mut stemcell = StemCellComplete::new();
    println!("🧬 初始: {} 细胞", stemcell.cells.len());
    println!();

    let train_start = Instant::now();
    
    for round in 1..=10 {
        println!("═══════════════════════════════════════════════════════════════════════════");
        println!("                         第 {} 轮", round);
        println!("═══════════════════════════════════════════════════════════════════════════");
        
        let round_start = Instant::now();
        
        for (idx, (name, bytes)) in corpus.iter().enumerate() {
            // 直接存储原始字节，让 Fate 计算签名
            let _id = stemcell.fate_store(bytes);
            
            // 传递信号（采样，不是全部字节）
            let step = (bytes.len() / 100).max(1);
            for i in (0..bytes.len()).step_by(step) {
                stemcell.receive_data(bytes[i] as f64);
            }
            
            if (idx + 1) % 100 == 0 {
                print!("\r   进度: {}/{}", idx + 1, corpus.len());
                std::io::Write::flush(&mut std::io::stdout()).ok();
            }
        }
        
        stemcell.evolve(0.5);
        
        println!("\r   第 {} 轮完成 ({:?}): {} 细胞, {} 存储, {} 连接",
            round, round_start.elapsed(),
            stemcell.cells.len(),
            stemcell.triple_fate.stored_count(),
            stemcell.cells.iter().map(|c| c.connections.len()).sum::<usize>());
    }
    
    println!();
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("                         训练完成");
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!();
    println!("📊 统计:");
    println!("   训练耗时: {:?}", train_start.elapsed());
    println!("   细胞数: {}", stemcell.cells.len());
    println!("   Fate 存储: {}", stemcell.triple_fate.stored_count());
    println!("   连接数: {}", stemcell.cells.iter().map(|c| c.connections.len()).sum::<usize>());
    println!("   方程: {}", stemcell.triple_fate.get_equation());
    println!();
    
    // 签名分布
    println!("📈 签名分布:");
    let mut sigs: Vec<f64> = Vec::new();
    for id in stemcell.triple_fate.list_stored() {
        if let Some(entry) = stemcell.triple_fate.load_data(id) {
            sigs.push(entry.signature);
        }
    }
    sigs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    if !sigs.is_empty() {
        println!("   范围: {:.2} ~ {:.2}", sigs[0], sigs[sigs.len()-1]);
        println!("   中位数: {:.2}", sigs[sigs.len()/2]);
        
        // 按类型统计
        let mut text_sigs = Vec::new();
        let mut img_sigs = Vec::new();
        let mut zh_sigs = Vec::new();
        
        for id in stemcell.triple_fate.list_stored() {
            if let Some(entry) = stemcell.triple_fate.load_data(id) {
                // 判断类型（通过内容特征）
                if entry.raw.len() > 2 && entry.raw[0] == 0xFF && entry.raw[1] == 0xD8 {
                    img_sigs.push(entry.signature);
                } else if entry.raw.iter().any(|&b| b > 127) {
                    zh_sigs.push(entry.signature);
                } else {
                    text_sigs.push(entry.signature);
                }
            }
        }
        
        if !text_sigs.is_empty() {
            text_sigs.sort_by(|a, b| a.partial_cmp(b).unwrap());
            println!("   英文文本 ({}): {:.2} ~ {:.2}", text_sigs.len(), text_sigs[0], text_sigs[text_sigs.len()-1]);
        }
        if !zh_sigs.is_empty() {
            zh_sigs.sort_by(|a, b| a.partial_cmp(b).unwrap());
            println!("   中文/高字节 ({}): {:.2} ~ {:.2}", zh_sigs.len(), zh_sigs[0], zh_sigs[zh_sigs.len()-1]);
        }
        if !img_sigs.is_empty() {
            img_sigs.sort_by(|a, b| a.partial_cmp(b).unwrap());
            println!("   图像JPEG ({}): {:.2} ~ {:.2}", img_sigs.len(), img_sigs[0], img_sigs[img_sigs.len()-1]);
        }
    }
    println!();
    
    // 保存
    let save_path = Path::new("/tmp/universal_stemcell.stemcell");
    match stemcell.save_slice(save_path) {
        Ok(()) => {
            let size = std::fs::metadata(save_path).map(|m| m.len()).unwrap_or(0);
            println!("💾 已保存: {} ({:.2} MB)", save_path.display(), size as f64 / 1024.0 / 1024.0);
        }
        Err(e) => println!("❌ 保存失败: {}", e),
    }
    
    println!();
    println!("✅ 完成");
}

/// 加载 LibriSpeech 转录
fn load_transcripts(corpus: &mut Vec<(String, Vec<u8>)>, max_count: usize) -> usize {
    let libri_path = Path::new(DATA_BASE).join("audio/LibriSpeech");
    let mut count = 0;
    
    fn walk(path: &Path, corpus: &mut Vec<(String, Vec<u8>)>, count: &mut usize, max: usize) {
        if *count >= max { return; }
        
        let Ok(entries) = std::fs::read_dir(path) else { return };
        
        for entry in entries.flatten() {
            if *count >= max { return; }
            
            let p = entry.path();
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            
            if name.starts_with('.') { continue; }
            
            if p.is_dir() {
                walk(&p, corpus, count, max);
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
                        corpus.push((name.to_string(), text.into_bytes()));
                        *count += 1;
                    }
                }
            }
        }
    }
    
    walk(&libri_path, corpus, &mut count, max_count);
    count
}

/// 加载中文维基（分块）
fn load_zhwiki(corpus: &mut Vec<(String, Vec<u8>)>, max_chunks: usize) -> usize {
    // 先尝试解压后的文件
    let wiki_xml = Path::new(DATA_BASE).join("text/zhwiki-latest.xml");
    let wiki_bz2 = Path::new(DATA_BASE).join("text/zhwiki-latest.xml.bz2");
    
    let data = if wiki_xml.exists() {
        // 读取解压后的 XML（只读前 50MB）
        if let Ok(mut file) = File::open(&wiki_xml) {
            let mut buf = vec![0u8; 50_000_000];
            let n = file.read(&mut buf).unwrap_or(0);
            buf.truncate(n);
            buf
        } else {
            return 0;
        }
    } else if wiki_bz2.exists() {
        // 读取压缩文件的前 10MB
        if let Ok(data) = std::fs::read(&wiki_bz2) {
            data[..10_000_000.min(data.len())].to_vec()
        } else {
            return 0;
        }
    } else {
        return 0;
    };
    
    // 分块存储
    let chunk_size = 100_000; // 100KB 每块
    let mut count = 0;
    
    for (i, chunk) in data.chunks(chunk_size).enumerate() {
        if count >= max_chunks { break; }
        corpus.push((format!("zhwiki_chunk_{}", i), chunk.to_vec()));
        count += 1;
    }
    
    count
}

/// 加载图像（原始 JPEG 字节）
fn load_images(corpus: &mut Vec<(String, Vec<u8>)>, max_count: usize) -> usize {
    let img_path = Path::new(DATA_BASE).join("images/train2017");
    let mut count = 0;
    
    let Ok(entries) = std::fs::read_dir(&img_path) else { return 0 };
    
    for entry in entries.flatten() {
        if count >= max_count { break; }
        
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        
        if !name.ends_with(".jpg") { continue; }
        
        // 读取整个图像文件
        if let Ok(data) = std::fs::read(&path) {
            corpus.push((name.to_string(), data));
            count += 1;
        }
    }
    
    count
}
