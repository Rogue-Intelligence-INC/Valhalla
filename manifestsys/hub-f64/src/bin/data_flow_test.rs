//! 数据流测试
//! 
//! 让各系统处理数据，观察 Fate 的动态行为
//! 然后根据结果设计每个容器的存储格式

use std::path::Path;
use hub_f64::ValhallaHub;
use stem_cell::StemCellComplete;
use valhalla_tile_quattro::TileQuattroComplete;
use chamber::ChamberComplete;

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║     🔬 数据流测试 - 观察 Fate 动态行为                          ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    let base_path = Path::new("/tmp/valhalla_data_flow");
    if base_path.exists() {
        std::fs::remove_dir_all(base_path).ok();
    }

    // 测试数据 - 各种类型
    let test_data = vec![
        ("text_en", "Hello World! This is a test."),
        ("text_cn", "你好世界！这是一个测试。"),
        ("text_jp", "こんにちは世界！これはテストです。"),
        ("numbers", "123456789 3.14159 2.71828"),
        ("code", "fn main() { println!(\"Hello\"); }"),
        ("mixed", "Hello 你好 123 @#$%"),
    ];

    // ========== 初始化系统 ==========
    println!("📦 初始化系统...\n");
    
    let mut hub = ValhallaHub::with_storage(base_path).expect("Hub init failed");
    let mut stemcell = StemCellComplete::with_storage(base_path).expect("StemCell init failed");
    let mut tile = TileQuattroComplete::with_storage(base_path).expect("Tile init failed");
    let mut chamber = ChamberComplete::with_storage(base_path).expect("Chamber init failed");

    // ========== 跑数据 ==========
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔄 跑数据通过各系统...\n");

    for (name, content) in &test_data {
        println!("📝 数据: {} ({} bytes)", name, content.len());
        
        // 转为字节
        let bytes = content.as_bytes();
        
        // 1. StemCell 处理
        let sc_id = stemcell.receive_content(content).unwrap();
        let sc_status = stemcell.get_status();
        println!("   StemCell: {} → 细胞数={}, Fate收敛={}", 
                 sc_id, sc_status.cell_count, stemcell.has_fate_converged());
        
        // 2. Tile 处理
        let tile_id = tile.receive_content(content).unwrap();
        let tile_status = tile.get_status();
        println!("   Tile: {} → 完成={}, Fate收敛={}", 
                 tile_id, tile_status.completed_tiles, tile.has_fate_converged());
        
        // 3. Chamber 处理
        let ch_id = chamber.receive_content(content).unwrap();
        let ch_status = chamber.get_status();
        println!("   Chamber: {} → 完成={}, Fate收敛={}", 
                 ch_id, ch_status.completed_containers, chamber.has_fate_converged());
        
        // 4. 记录字节特征
        let byte_stats = analyze_bytes(bytes);
        println!("   字节特征: {:?}", byte_stats);
        
        println!();
    }

    // ========== 多轮循环 ==========
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔁 多轮循环处理...\n");

    for cycle in 1..=5 {
        println!("周期 {}:", cycle);
        
        for (name, content) in &test_data {
            // 每轮都处理
            for byte in content.bytes() {
                stemcell.receive_data(byte as f64);
                tile.receive_data(byte as f64);
                chamber.receive_data(byte as f64);
            }
        }
        
        let sc = stemcell.get_status();
        let ti = tile.get_status();
        let ch = chamber.get_status();
        
        println!("   SC: 细胞={} dim={:.2} Fate收敛={}", 
                 sc.cell_count, sc.dimension, stemcell.has_fate_converged());
        println!("   Tile: 完成={} dim={:.2} Fate收敛={}", 
                 ti.completed_tiles, ti.dimension, tile.has_fate_converged());
        println!("   Chamber: 完成={} dim={:.2} Fate收敛={}", 
                 ch.completed_containers, ch.dimension, chamber.has_fate_converged());
        println!();
    }

    // ========== 观察 Fate 状态 ==========
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔮 Fate 状态观察:\n");
    
    println!("StemCell Fate:\n{}", stemcell.get_fate_stats());
    println!();
    println!("Tile Fate:\n{}", tile.get_fate_stats());
    println!();
    println!("Chamber Status: {:?}", chamber.get_status());

    // ========== 查看存储的文件 ==========
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📂 存储的文件:\n");
    
    println!("StemCell:");
    for f in stemcell.list_stored().unwrap_or_default() {
        let size = stemcell.storage().unwrap().size(&f).unwrap_or(0);
        println!("   {} ({} bytes)", f, size);
    }
    
    println!("\nTile:");
    for f in tile.list_stored().unwrap_or_default() {
        let size = tile.storage().unwrap().size(&f).unwrap_or(0);
        println!("   {} ({} bytes)", f, size);
    }
    
    println!("\nChamber:");
    for f in chamber.list_stored().unwrap_or_default() {
        let size = chamber.storage().unwrap().size(&f).unwrap_or(0);
        println!("   {} ({} bytes)", f, size);
    }

    // ========== 建议存储格式 ==========
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("💡 基于 Fate 行为建议的存储格式:\n");
    
    let sc = stemcell.get_status();
    let ti = tile.get_status();
    let ch = chamber.get_status();
    
    println!("StemCell (.sc):");
    println!("   - 维度: {:.2}", sc.dimension);
    println!("   - 细胞数: {}", sc.cell_count);
    println!("   - 建议: 按细胞分组存储，每个细胞保存其 data_signature");
    
    println!("\nTile (.tile):");
    println!("   - 维度: {:.2}", ti.dimension);
    println!("   - 完成数: {}", ti.completed_tiles);
    println!("   - 建议: 按神经元连接存储，保存 connections 图");
    
    println!("\nChamber (.ch):");
    println!("   - 维度: {:.2}", ch.dimension);
    println!("   - 完成数: {}", ch.completed_containers);
    println!("   - 建议: 按容器类型分组，保存 load 和 scale 历史");

    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║     ✅ 数据流测试完成                                           ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
}

/// 分析字节特征
fn analyze_bytes(bytes: &[u8]) -> ByteStats {
    if bytes.is_empty() {
        return ByteStats::default();
    }
    
    let sum: u64 = bytes.iter().map(|&b| b as u64).sum();
    let mean = sum as f64 / bytes.len() as f64;
    
    let variance: f64 = bytes.iter()
        .map(|&b| {
            let diff = (b as f64) - mean;
            diff * diff
        })
        .sum::<f64>() / bytes.len() as f64;
    
    let min = *bytes.iter().min().unwrap_or(&0);
    let max = *bytes.iter().max().unwrap_or(&0);
    
    // 检测模式
    let has_ascii = bytes.iter().all(|&b| b < 128);
    let has_utf8_multi = bytes.iter().any(|&b| b >= 128);
    
    ByteStats {
        len: bytes.len(),
        mean,
        variance,
        min,
        max,
        has_ascii,
        has_utf8_multi,
    }
}

#[derive(Debug, Default)]
struct ByteStats {
    len: usize,
    mean: f64,
    variance: f64,
    min: u8,
    max: u8,
    has_ascii: bool,
    has_utf8_multi: bool,
}
