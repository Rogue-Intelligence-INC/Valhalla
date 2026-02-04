//! StemCell 全语料 10 轮训练 (大规模)
//!
//! 加载所有可用语料进行训练:
//! - LibriSpeech 转录文本
//! - COCO 图像标注
//! - Valhalla Rust 源码
//! - Cargo Registry 源码

use std::time::Instant;
use std::path::Path;
use std::io::{BufRead, BufReader};

use stem_cell::StemCellComplete;

const TRAINING_DATA_BASE: &str = "/run/media/jouly/My Book/valhalla_training_data";

fn main() {
    println!("═══════════════════════════════════════════════════════════════════════");
    println!("       StemCell 全语料 10 轮训练 (大规模)");
    println!("═══════════════════════════════════════════════════════════════════════");
    println!();
    
    // 加载所有语料
    let mut all_corpus: Vec<(String, Vec<u8>)> = Vec::new();
    
    // 1. LibriSpeech 转录
    println!("📚 加载 LibriSpeech 转录...");
    let libri_start = Instant::now();
    let libri_corpus = load_librispeech_transcripts();
    let libri_bytes: usize = libri_corpus.iter().map(|(_, d)| d.len()).sum();
    println!("   {} 文件, {} KB, 耗时 {:?}", libri_corpus.len(), libri_bytes / 1024, libri_start.elapsed());
    all_corpus.extend(libri_corpus);
    
    // 2. COCO 标注
    println!("🖼️  加载 COCO 标注...");
    let coco_start = Instant::now();
    let coco_corpus = load_coco_captions();
    let coco_bytes: usize = coco_corpus.iter().map(|(_, d)| d.len()).sum();
    println!("   {} 条, {} KB, 耗时 {:?}", coco_corpus.len(), coco_bytes / 1024, coco_start.elapsed());
    all_corpus.extend(coco_corpus);
    
    // 3. Valhalla 源码
    println!("🦀 加载 Valhalla 源码...");
    let valhalla_start = Instant::now();
    let valhalla_corpus = load_rust_corpus(Path::new("/home/jouly/AI/Valhalla"));
    let valhalla_bytes: usize = valhalla_corpus.iter().map(|(_, d)| d.len()).sum();
    println!("   {} 文件, {} KB, 耗时 {:?}", valhalla_corpus.len(), valhalla_bytes / 1024, valhalla_start.elapsed());
    all_corpus.extend(valhalla_corpus);
    
    // 4. Cargo Registry 源码 (限制数量)
    println!("📦 加载 Cargo Registry 源码...");
    let cargo_start = Instant::now();
    let cargo_corpus = load_cargo_registry(5000); // 限制 5000 个文件
    let cargo_bytes: usize = cargo_corpus.iter().map(|(_, d)| d.len()).sum();
    println!("   {} 文件, {} KB, 耗时 {:?}", cargo_corpus.len(), cargo_bytes / 1024, cargo_start.elapsed());
    all_corpus.extend(cargo_corpus);
    
    // 总计
    let total_bytes: usize = all_corpus.iter().map(|(_, d)| d.len()).sum();
    println!();
    println!("📊 语料总计:");
    println!("   文件数: {}", all_corpus.len());
    println!("   总大小: {:.2} MB", total_bytes as f64 / 1024.0 / 1024.0);
    println!();
    
    // 创建 StemCell
    let mut stemcell = StemCellComplete::new();
    
    println!("🧬 初始状态:");
    print_stemcell_status(&stemcell);
    println!();
    
    // 10 轮训练
    let total_start = Instant::now();
    
    for round in 1..=10 {
        println!("═══════════════════════════════════════════════════════════════════════");
        println!("                    第 {} 轮训练", round);
        println!("═══════════════════════════════════════════════════════════════════════");
        
        let round_start = Instant::now();
        let mut round_ops = 0u64;
        let mut round_new_cells = 0usize;
        let mut round_discoveries = 0usize;
        
        // 遍历所有语料
        for (file_idx, (_filename, bytes)) in all_corpus.iter().enumerate() {
            // 将字节转换为数值并输入
            for &b in bytes.iter() {
                let result = stemcell.receive_data(b as f64);
                round_ops += 1;
                round_new_cells += result.new_cells;
                round_discoveries += result.discoveries.len();
            }
            
            // 每处理 500 个文件输出进度
            if (file_idx + 1) % 500 == 0 || file_idx + 1 == all_corpus.len() {
                let elapsed = round_start.elapsed();
                let ops_per_sec = round_ops as f64 / elapsed.as_secs_f64();
                print!("\r   进度: {}/{} ({:.1}%), {} ops, {:.0}M ops/sec, {} 新细胞    ", 
                    file_idx + 1, all_corpus.len(),
                    (file_idx + 1) as f64 / all_corpus.len() as f64 * 100.0,
                    round_ops, ops_per_sec / 1_000_000.0, round_new_cells);
                std::io::Write::flush(&mut std::io::stdout()).ok();
            }
        }
        
        let round_time = round_start.elapsed();
        
        println!();
        println!("   第 {} 轮完成:", round);
        println!("   ├─ 操作数: {} ({:.0}M ops/sec)", 
            round_ops, 
            round_ops as f64 / round_time.as_secs_f64() / 1_000_000.0
        );
        println!("   ├─ 耗时: {:?}", round_time);
        println!("   ├─ 新细胞: {}", round_new_cells);
        println!("   └─ 发现: {}", round_discoveries);
        println!();
        
        // 轮次后状态
        println!("   轮次后状态:");
        print_stemcell_status(&stemcell);
        
        // Space 维度分析
        let sample_data: Vec<f64> = all_corpus.iter()
            .flat_map(|(_, bytes)| bytes.iter().take(10).map(|&b| b as f64))
            .take(100)
            .collect();
        let dim_result = stemcell.space_calculate_dimension(&sample_data);
        println!("   Space 维度: {:.4} (Hausdorff: {:.4}, Box: {:.4}, Info: {:.4})",
            dim_result.recommended, dim_result.hausdorff, dim_result.box_dim, dim_result.info_dim);
        println!();
        
        // 存储轮次数据到 Fate
        let round_summary = format!(
            "round:{},ops:{},cells:{},discoveries:{},time_ms:{}",
            round, round_ops, round_new_cells, round_discoveries, round_time.as_millis()
        );
        stemcell.triple_fate.store_string(&round_summary);
    }
    
    let total_time = total_start.elapsed();
    
    // 最终状态
    println!("═══════════════════════════════════════════════════════════════════════");
    println!("                    训练完成");
    println!("═══════════════════════════════════════════════════════════════════════");
    println!();
    println!("📊 训练总结:");
    println!("   总耗时: {:?}", total_time);
    println!("   总数据量: {:.2} MB x 10 轮 = {:.2} MB", 
        total_bytes as f64 / 1024.0 / 1024.0,
        total_bytes as f64 * 10.0 / 1024.0 / 1024.0
    );
    println!("   吞吐量: {:.2} MB/sec", 
        (total_bytes * 10) as f64 / 1024.0 / 1024.0 / total_time.as_secs_f64()
    );
    println!();
    
    println!("🧬 最终状态:");
    print_stemcell_status(&stemcell);
    println!();
    
    // Fate 存储统计
    println!("📦 Fate 存储:");
    println!("   存储条目: {}", stemcell.triple_fate.stored_count());
    println!("   总修改次数: {}", stemcell.triple_fate.total_modifications());
    println!();
    
    // 保存切片
    let slice_path = Path::new("/tmp/stemcell_full_trained.stemcell");
    match stemcell.save_slice(slice_path) {
        Ok(()) => println!("💾 切片已保存: {}", slice_path.display()),
        Err(e) => println!("❌ 保存失败: {}", e),
    }
    
    // 分析 Fate 中的训练记录
    println!();
    println!("📈 训练记录:");
    for id in stemcell.triple_fate.list_stored().iter().take(10) {
        if let Some(entry) = stemcell.triple_fate.load_data(id) {
            if let Ok(text) = String::from_utf8(entry.raw.clone()) {
                if text.starts_with("round:") {
                    println!("   {}", text);
                }
            }
        }
    }
    
    println!();
    println!("═══════════════════════════════════════════════════════════════════════");
}

fn print_stemcell_status(stemcell: &StemCellComplete) {
    let status = stemcell.get_status();
    println!("   ├─ 细胞数: {}", status.cell_count);
    println!("   ├─ 世代: {}", status.generation);
    println!("   ├─ 总分裂: {}", status.total_divisions);
    println!("   ├─ 总死亡: {}", status.total_deaths);
    println!("   ├─ 总合并: {}", status.total_merges);
    println!("   ├─ 已关闭容器: {}", status.total_containers_closed);
    println!("   ├─ 活跃容器: {}", status.active_containers);
    println!("   ├─ 维度: {:.4}", status.dimension);
    println!("   └─ Fate Tick: {}", status.fate_tick);
}

/// 加载 LibriSpeech 转录文本
fn load_librispeech_transcripts() -> Vec<(String, Vec<u8>)> {
    let base = Path::new(TRAINING_DATA_BASE).join("audio/LibriSpeech");
    let mut corpus = Vec::new();
    
    // 递归查找所有 .trans.txt 文件
    load_txt_files_recursive(&base, &mut corpus, 0, ".trans.txt");
    
    // 也加载元数据文件
    for name in &["BOOKS.TXT", "CHAPTERS.TXT", "SPEAKERS.TXT", "README.TXT"] {
        let path = base.join(name);
        if let Ok(content) = std::fs::read(&path) {
            corpus.push((path.display().to_string(), content));
        }
    }
    
    corpus
}

/// 加载 COCO 标注 (仅加载 captions)
fn load_coco_captions() -> Vec<(String, Vec<u8>)> {
    let base = Path::new(TRAINING_DATA_BASE).join("images/annotations");
    let mut corpus = Vec::new();
    
    // 加载 captions JSON
    for name in &["captions_train2017.json", "captions_val2017.json"] {
        let path = base.join(name);
        if let Ok(content) = std::fs::read(&path) {
            // JSON 可能很大，我们提取文本部分
            if let Ok(text) = String::from_utf8(content.clone()) {
                // 简单提取 caption 文本
                let captions: Vec<u8> = extract_captions_from_json(&text)
                    .into_bytes();
                if !captions.is_empty() {
                    corpus.push((path.display().to_string(), captions));
                } else {
                    corpus.push((path.display().to_string(), content));
                }
            }
        }
    }
    
    corpus
}

/// 从 COCO JSON 提取 caption 文本
fn extract_captions_from_json(json: &str) -> String {
    let mut captions = String::new();
    
    // 简单的文本提取：查找 "caption": "..." 模式
    let mut in_caption = false;
    let mut chars = json.chars().peekable();
    
    while let Some(c) = chars.next() {
        if !in_caption {
            // 查找 "caption"
            if c == '"' {
                let mut word = String::new();
                while let Some(&nc) = chars.peek() {
                    if nc == '"' { chars.next(); break; }
                    word.push(nc);
                    chars.next();
                }
                if word == "caption" {
                    // 跳过 : 和空格
                    while let Some(&nc) = chars.peek() {
                        if nc == '"' { in_caption = true; chars.next(); break; }
                        chars.next();
                    }
                }
            }
        } else {
            // 读取 caption 内容
            if c == '"' {
                captions.push('\n');
                in_caption = false;
            } else if c != '\\' {
                captions.push(c);
            }
        }
        
        // 限制大小
        if captions.len() > 50_000_000 { break; } // 50MB 限制
    }
    
    captions
}

/// 加载 Rust 源码语料
fn load_rust_corpus(base_path: &Path) -> Vec<(String, Vec<u8>)> {
    let mut corpus = Vec::new();
    load_files_recursive(base_path, &mut corpus, 0, ".rs");
    corpus
}

/// 加载 Cargo Registry 源码
fn load_cargo_registry(max_files: usize) -> Vec<(String, Vec<u8>)> {
    let base = Path::new("/home/jouly/.cargo/registry/src");
    let mut corpus = Vec::new();
    load_files_recursive_limited(base, &mut corpus, 0, ".rs", max_files);
    corpus
}

fn load_txt_files_recursive(path: &Path, corpus: &mut Vec<(String, Vec<u8>)>, depth: usize, suffix: &str) {
    if depth > 15 { return; }
    
    let Ok(entries) = std::fs::read_dir(path) else { return };
    
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        
        if name.starts_with('.') { continue; }
        
        if path.is_dir() {
            load_txt_files_recursive(&path, corpus, depth + 1, suffix);
        } else if name.ends_with(suffix) {
            if let Ok(content) = std::fs::read(&path) {
                corpus.push((path.display().to_string(), content));
            }
        }
    }
}

fn load_files_recursive(path: &Path, corpus: &mut Vec<(String, Vec<u8>)>, depth: usize, ext: &str) {
    if depth > 10 { return; }
    
    let Ok(entries) = std::fs::read_dir(path) else { return };
    
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        
        if name.starts_with('.') || name == "target" || name == "node_modules" {
            continue;
        }
        
        if path.is_dir() {
            load_files_recursive(&path, corpus, depth + 1, ext);
        } else if path.extension().and_then(|e| e.to_str()) == Some(&ext[1..]) {
            if let Ok(content) = std::fs::read(&path) {
                corpus.push((path.display().to_string(), content));
            }
        }
    }
}

fn load_files_recursive_limited(path: &Path, corpus: &mut Vec<(String, Vec<u8>)>, depth: usize, ext: &str, max: usize) {
    if depth > 10 || corpus.len() >= max { return; }
    
    let Ok(entries) = std::fs::read_dir(path) else { return };
    
    for entry in entries.flatten() {
        if corpus.len() >= max { return; }
        
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        
        if name.starts_with('.') { continue; }
        
        if path.is_dir() {
            load_files_recursive_limited(&path, corpus, depth + 1, ext, max);
        } else if path.extension().and_then(|e| e.to_str()) == Some(&ext[1..]) {
            if let Ok(content) = std::fs::read(&path) {
                corpus.push((path.display().to_string(), content));
            }
        }
    }
}
