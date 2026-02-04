//! StemCell 全语料 10 轮训练
//!
//! 加载项目 Rust 源码，对 StemCell 进行多轮训练

use std::time::Instant;
use std::path::Path;

use stem_cell::StemCellComplete;

fn main() {
    println!("═══════════════════════════════════════════════════════════════════════");
    println!("       StemCell 全语料 10 轮训练");
    println!("═══════════════════════════════════════════════════════════════════════");
    println!();
    
    // 加载语料
    let corpus_path = Path::new("/home/jouly/AI/Valhalla");
    let corpus_data = load_rust_corpus(corpus_path);
    
    println!("📁 语料统计:");
    println!("   文件数: {}", corpus_data.len());
    let total_bytes: usize = corpus_data.iter().map(|(_, d)| d.len()).sum();
    println!("   总字节: {} KB ({:.2} MB)", total_bytes / 1024, total_bytes as f64 / 1024.0 / 1024.0);
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
        for (file_idx, (filename, bytes)) in corpus_data.iter().enumerate() {
            // 将字节转换为数值并输入
            for &b in bytes.iter() {
                let result = stemcell.receive_data(b as f64);
                round_ops += 1;
                round_new_cells += result.new_cells;
                round_discoveries += result.discoveries.len();
            }
            
            // 每处理 50 个文件输出进度
            if (file_idx + 1) % 50 == 0 {
                print!("\r   进度: {}/{} 文件, {} ops, {} 新细胞", 
                    file_idx + 1, corpus_data.len(), round_ops, round_new_cells);
                std::io::Write::flush(&mut std::io::stdout()).ok();
            }
        }
        
        let round_time = round_start.elapsed();
        
        println!("\r                                                                        ");
        println!("   第 {} 轮完成:", round);
        println!("   ├─ 操作数: {} ({:.0} ops/sec)", 
            round_ops, 
            round_ops as f64 / round_time.as_secs_f64()
        );
        println!("   ├─ 耗时: {:?}", round_time);
        println!("   ├─ 新细胞: {}", round_new_cells);
        println!("   └─ 发现: {}", round_discoveries);
        println!();
        
        // 轮次后状态
        println!("   轮次后状态:");
        print_stemcell_status(&stemcell);
        
        // Space 维度分析
        let sample_data: Vec<f64> = corpus_data.iter()
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
    println!("   总数据量: {} MB", total_bytes as f64 / 1024.0 / 1024.0);
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
    let slice_path = Path::new("/tmp/stemcell_trained.stemcell");
    match stemcell.save_slice(slice_path) {
        Ok(()) => println!("💾 切片已保存: {}", slice_path.display()),
        Err(e) => println!("❌ 保存失败: {}", e),
    }
    
    // 分析 Fate 中的训练记录
    println!();
    println!("📈 训练记录 (Fate 存储):");
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

/// 加载 Rust 源码语料
fn load_rust_corpus(base_path: &Path) -> Vec<(String, Vec<u8>)> {
    let mut corpus = Vec::new();
    load_rust_files_recursive(base_path, &mut corpus, 0);
    corpus
}

fn load_rust_files_recursive(path: &Path, corpus: &mut Vec<(String, Vec<u8>)>, depth: usize) {
    if depth > 10 { return; }
    
    let Ok(entries) = std::fs::read_dir(path) else { return };
    
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        
        if name.starts_with('.') || name == "target" || name == "node_modules" {
            continue;
        }
        
        if path.is_dir() {
            load_rust_files_recursive(&path, corpus, depth + 1);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            if let Ok(content) = std::fs::read(&path) {
                corpus.push((path.display().to_string(), content));
            }
        }
    }
}
