//! Space-GPU 全系统语料基准测试
//!
//! 测试 StemCell, Tile, Chamber, Hub 处理语料的性能
//! 观察是否有性能退化

use std::time::Instant;
use std::path::Path;

use stem_cell::StemCellComplete;
use valhalla_tile_quattro::TileQuattroComplete;
use chamber::ChamberComplete;
use hub_f64::ValhallaHub;

fn main() {
    println!("═══════════════════════════════════════════════════════════════════════");
    println!("       Space-GPU 全系统语料基准测试");
    println!("═══════════════════════════════════════════════════════════════════════");
    println!();
    
    // 加载语料 (项目 Rust 源码)
    let corpus_path = Path::new("/home/jouly/AI/Valhalla");
    let corpus_data = load_rust_corpus(corpus_path);
    
    println!("📁 语料统计:");
    println!("   文件数: {}", corpus_data.len());
    let total_bytes: usize = corpus_data.iter().map(|(_, d)| d.len()).sum();
    println!("   总字节: {} KB", total_bytes / 1024);
    println!();
    
    // 提取数值数据用于 Space 计算
    let numeric_data: Vec<Vec<f64>> = corpus_data.iter()
        .map(|(_, bytes)| {
            bytes.iter().take(100).map(|&b| b as f64).collect()
        })
        .collect();
    
    println!("═══════════════════════════════════════════════════════════════════════");
    println!("                    性能基准测试");
    println!("═══════════════════════════════════════════════════════════════════════");
    println!();
    
    // =========================================================================
    // StemCell 测试
    // =========================================================================
    println!("🧬 StemCell 测试");
    println!("───────────────────────────────────────────────────────────────────────");
    
    let mut stemcell = StemCellComplete::new();
    
    // 预热
    for i in 0..100 {
        stemcell.receive_data(i as f64);
    }
    
    // 测试数据流处理速度
    let start = Instant::now();
    let mut stemcell_ops = 0u64;
    for (_, bytes) in &corpus_data {
        for &b in bytes.iter().take(1000) {
            stemcell.receive_data(b as f64);
            stemcell_ops += 1;
        }
    }
    let stemcell_data_time = start.elapsed();
    
    // 测试 Space-GPU 计算
    let start = Instant::now();
    let stemcell_space_results = stemcell.space_calculate_batch(&numeric_data);
    let stemcell_space_time = start.elapsed();
    
    println!("   数据流处理: {} 次 in {:?} ({:.0} ops/sec)",
        stemcell_ops,
        stemcell_data_time,
        stemcell_ops as f64 / stemcell_data_time.as_secs_f64()
    );
    println!("   Space 批量: {} 次 in {:?} ({:.0} files/sec)",
        numeric_data.len(),
        stemcell_space_time,
        numeric_data.len() as f64 / stemcell_space_time.as_secs_f64()
    );
    println!("   当前维度: {:.4}", stemcell.space_dimension());
    println!("   Fate 存储: {} 条", stemcell.triple_fate.stored_count());
    println!();
    
    // =========================================================================
    // Tile 测试
    // =========================================================================
    println!("🧱 Tile 测试");
    println!("───────────────────────────────────────────────────────────────────────");
    
    let mut tile = TileQuattroComplete::new();
    
    // 预热
    for i in 0..100 {
        tile.receive_data(i as f64);
    }
    
    // 测试数据流处理速度
    let start = Instant::now();
    let mut tile_ops = 0u64;
    for (_, bytes) in &corpus_data {
        for &b in bytes.iter().take(1000) {
            tile.receive_data(b as f64);
            tile_ops += 1;
        }
    }
    let tile_data_time = start.elapsed();
    
    // 测试 Space-GPU 计算
    let start = Instant::now();
    let tile_space_results = tile.space_calculate_batch(&numeric_data);
    let tile_space_time = start.elapsed();
    
    println!("   数据流处理: {} 次 in {:?} ({:.0} ops/sec)",
        tile_ops,
        tile_data_time,
        tile_ops as f64 / tile_data_time.as_secs_f64()
    );
    println!("   Space 批量: {} 次 in {:?} ({:.0} files/sec)",
        numeric_data.len(),
        tile_space_time,
        numeric_data.len() as f64 / tile_space_time.as_secs_f64()
    );
    println!("   当前维度: {:.4}", tile.space_dimension());
    println!("   Fate 存储: {} 条", tile.triple_fate.stored_count());
    println!();
    
    // =========================================================================
    // Chamber 测试
    // =========================================================================
    println!("📦 Chamber 测试");
    println!("───────────────────────────────────────────────────────────────────────");
    
    let mut chamber = ChamberComplete::new();
    
    // 预热
    for i in 0..100 {
        chamber.receive_data(i as f64);
    }
    
    // 测试数据流处理速度
    let start = Instant::now();
    let mut chamber_ops = 0u64;
    for (_, bytes) in &corpus_data {
        for &b in bytes.iter().take(1000) {
            chamber.receive_data(b as f64);
            chamber_ops += 1;
        }
    }
    let chamber_data_time = start.elapsed();
    
    // 测试 Space-GPU 计算
    let start = Instant::now();
    let chamber_space_results = chamber.space_calculate_batch(&numeric_data);
    let chamber_space_time = start.elapsed();
    
    println!("   数据流处理: {} 次 in {:?} ({:.0} ops/sec)",
        chamber_ops,
        chamber_data_time,
        chamber_ops as f64 / chamber_data_time.as_secs_f64()
    );
    println!("   Space 批量: {} 次 in {:?} ({:.0} files/sec)",
        numeric_data.len(),
        chamber_space_time,
        numeric_data.len() as f64 / chamber_space_time.as_secs_f64()
    );
    println!("   当前维度: {:.4}", chamber.space_dimension());
    println!("   Fate 存储: {} 条", chamber.triple_fate.stored_count());
    println!();
    
    // =========================================================================
    // Hub 测试
    // =========================================================================
    println!("🌌 Hub 测试");
    println!("───────────────────────────────────────────────────────────────────────");
    
    let mut hub = ValhallaHub::new();
    
    // 测试 Space-GPU 计算
    let start = Instant::now();
    let hub_space_results = hub.space_calculate_batch(&numeric_data);
    let hub_space_time = start.elapsed();
    
    println!("   Space 批量: {} 次 in {:?} ({:.0} files/sec)",
        numeric_data.len(),
        hub_space_time,
        numeric_data.len() as f64 / hub_space_time.as_secs_f64()
    );
    println!("   当前维度: {:.4}", hub.space_dimension());
    println!();
    
    // =========================================================================
    // 性能对比
    // =========================================================================
    println!("═══════════════════════════════════════════════════════════════════════");
    println!("                    性能对比总结");
    println!("═══════════════════════════════════════════════════════════════════════");
    println!();
    
    let stemcell_rate = stemcell_ops as f64 / stemcell_data_time.as_secs_f64();
    let tile_rate = tile_ops as f64 / tile_data_time.as_secs_f64();
    let chamber_rate = chamber_ops as f64 / chamber_data_time.as_secs_f64();
    
    println!("📊 数据流处理速度 (ops/sec):");
    println!("   StemCell : {:>12.0} ops/sec", stemcell_rate);
    println!("   Tile     : {:>12.0} ops/sec", tile_rate);
    println!("   Chamber  : {:>12.0} ops/sec", chamber_rate);
    println!();
    
    // 速度对比
    let max_rate = stemcell_rate.max(tile_rate).max(chamber_rate);
    println!("   相对速度:");
    println!("   StemCell : {:.1}%", stemcell_rate / max_rate * 100.0);
    println!("   Tile     : {:.1}%", tile_rate / max_rate * 100.0);
    println!("   Chamber  : {:.1}%", chamber_rate / max_rate * 100.0);
    println!();
    
    let stemcell_space_rate = numeric_data.len() as f64 / stemcell_space_time.as_secs_f64();
    let tile_space_rate = numeric_data.len() as f64 / tile_space_time.as_secs_f64();
    let chamber_space_rate = numeric_data.len() as f64 / chamber_space_time.as_secs_f64();
    let hub_space_rate = numeric_data.len() as f64 / hub_space_time.as_secs_f64();
    
    println!("📊 Space-GPU 批量计算速度 (files/sec):");
    println!("   StemCell : {:>12.0} files/sec", stemcell_space_rate);
    println!("   Tile     : {:>12.0} files/sec", tile_space_rate);
    println!("   Chamber  : {:>12.0} files/sec", chamber_space_rate);
    println!("   Hub      : {:>12.0} files/sec", hub_space_rate);
    println!();
    
    // 维度分析
    println!("📐 维度分析:");
    let avg_dim: f64 = stemcell_space_results.iter().map(|r| r.recommended).sum::<f64>() / stemcell_space_results.len() as f64;
    let max_dim: f64 = stemcell_space_results.iter().map(|r| r.recommended).fold(0.0, f64::max);
    let min_dim: f64 = stemcell_space_results.iter().map(|r| r.recommended).fold(f64::MAX, f64::min);
    println!("   平均维度: {:.4}", avg_dim);
    println!("   最大维度: {:.4}", max_dim);
    println!("   最小维度: {:.4}", min_dim);
    println!();
    
    // 性能退化警告
    println!("═══════════════════════════════════════════════════════════════════════");
    println!("                    性能退化检查");
    println!("═══════════════════════════════════════════════════════════════════════");
    println!();
    
    let baseline_ops = 500_000.0; // 基准: 50万 ops/sec
    
    if stemcell_rate < baseline_ops {
        println!("⚠️  StemCell 可能变慢: {:.0} ops/sec (基准: {:.0})", stemcell_rate, baseline_ops);
    } else {
        println!("✅ StemCell 性能正常: {:.0} ops/sec", stemcell_rate);
    }
    
    if tile_rate < baseline_ops {
        println!("⚠️  Tile 可能变慢: {:.0} ops/sec (基准: {:.0})", tile_rate, baseline_ops);
    } else {
        println!("✅ Tile 性能正常: {:.0} ops/sec", tile_rate);
    }
    
    if chamber_rate < baseline_ops {
        println!("⚠️  Chamber 可能变慢: {:.0} ops/sec (基准: {:.0})", chamber_rate, baseline_ops);
    } else {
        println!("✅ Chamber 性能正常: {:.0} ops/sec", chamber_rate);
    }
    
    println!();
    println!("═══════════════════════════════════════════════════════════════════════");
    println!("                    测试完成");
    println!("═══════════════════════════════════════════════════════════════════════");
}

/// 加载 Rust 源码语料
fn load_rust_corpus(base_path: &Path) -> Vec<(String, Vec<u8>)> {
    let mut corpus = Vec::new();
    load_rust_files_recursive(base_path, &mut corpus, 0);
    corpus
}

fn load_rust_files_recursive(path: &Path, corpus: &mut Vec<(String, Vec<u8>)>, depth: usize) {
    if depth > 10 { return; } // 深度限制
    
    let Ok(entries) = std::fs::read_dir(path) else { return };
    
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        
        // 跳过
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
