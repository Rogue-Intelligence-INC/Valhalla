//! 存储系统验证测试
//! 
//! 验证所有系统的文件操作能力

use std::path::Path;
use hub_f64::ValhallaHub;
use stem_cell::StemCellComplete;
use valhalla_tile_quattro::TileQuattroComplete;
use chamber::ChamberComplete;

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║     🧪 Valhalla 存储系统验证                                    ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    let base_path = std::env::temp_dir().join("valhalla_storage_test");
    
    // 清理旧测试数据
    if base_path.exists() {
        std::fs::remove_dir_all(&base_path).ok();
    }

    println!("📁 测试路径: {:?}\n", base_path);

    // ========== 测试 Hub ==========
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🌌 测试 Hub 存储...");
    
    match ValhallaHub::with_storage(&base_path) {
        Ok(hub) => {
            // 写入
            hub.store("test.txt", "Hello from Hub!").unwrap();
            println!("   ✓ 写入 test.txt");
            
            // 读取
            let content = hub.load("test.txt").unwrap();
            assert_eq!(content, "Hello from Hub!");
            println!("   ✓ 读取 test.txt: {}", content);
            
            // 写入子目录
            hub.storage().unwrap().write("subdir/nested.txt", "Nested content").unwrap();
            println!("   ✓ 写入 subdir/nested.txt");
            
            // 列出所有
            let files = hub.list_stored().unwrap();
            println!("   ✓ 列出所有文件: {:?}", files);
            
            // 删除
            hub.remove("test.txt").unwrap();
            println!("   ✓ 删除 test.txt");
            
            // 验证四元系统存储目录
            println!("\n   四元系统存储:");
            for system in ["blackhole", "meshbrain", "multinova", "baseforce"] {
                let dir = base_path.join(system);
                println!("   ├── {} 目录: {}", system, if dir.exists() { "✓" } else { "✗" });
            }
            
            println!("   ✅ Hub 存储测试通过\n");
        }
        Err(e) => println!("   ❌ Hub 初始化失败: {}\n", e),
    }

    // ========== 测试 StemCell ==========
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🧬 测试 StemCell 存储...");
    
    match StemCellComplete::with_storage(&base_path) {
        Ok(mut stemcell) => {
            // 存储原始内容
            stemcell.store("cell_data.txt", "Cell原始数据").unwrap();
            println!("   ✓ 写入 cell_data.txt");
            
            // 使用 receive_content 同时存储和产生信号
            let id = stemcell.receive_content("这是一段测试语料").unwrap();
            println!("   ✓ receive_content -> {}", id);
            
            // 读取
            let content = stemcell.load("cell_data.txt").unwrap();
            println!("   ✓ 读取: {}", content);
            
            // 修改
            stemcell.store("cell_data.txt", "Cell修改后的数据").unwrap();
            let modified = stemcell.load("cell_data.txt").unwrap();
            println!("   ✓ 修改后: {}", modified);
            
            // 列出
            let files = stemcell.list_stored().unwrap();
            println!("   ✓ 文件列表: {:?}", files);
            
            println!("   ✅ StemCell 存储测试通过\n");
        }
        Err(e) => println!("   ❌ StemCell 初始化失败: {}\n", e),
    }

    // ========== 测试 Tile ==========
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🧱 测试 Tile 存储...");
    
    match TileQuattroComplete::with_storage(&base_path) {
        Ok(mut tile) => {
            tile.store("tile_data.json", r#"{"type": "tile", "value": 42}"#).unwrap();
            println!("   ✓ 写入 tile_data.json");
            
            let id = tile.receive_content("Tile 测试内容").unwrap();
            println!("   ✓ receive_content -> {}", id);
            
            let files = tile.list_stored().unwrap();
            println!("   ✓ 文件列表: {:?}", files);
            
            println!("   ✅ Tile 存储测试通过\n");
        }
        Err(e) => println!("   ❌ Tile 初始化失败: {}\n", e),
    }

    // ========== 测试 Chamber ==========
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🏛️  测试 Chamber 存储...");
    
    match ChamberComplete::with_storage(&base_path) {
        Ok(mut chamber) => {
            chamber.store("container_log.txt", "Container 日志").unwrap();
            println!("   ✓ 写入 container_log.txt");
            
            let id = chamber.receive_content("Chamber 测试数据").unwrap();
            println!("   ✓ receive_content -> {}", id);
            
            let files = chamber.list_stored().unwrap();
            println!("   ✓ 文件列表: {:?}", files);
            
            println!("   ✅ Chamber 存储测试通过\n");
        }
        Err(e) => println!("   ❌ Chamber 初始化失败: {}\n", e),
    }

    // ========== 检查最终目录结构 ==========
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📂 最终目录结构:");
    print_dir(&base_path, 0);

    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║     ✅ 所有存储测试完成！                                       ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
}

fn print_dir(path: &Path, depth: usize) {
    let indent = "   ".repeat(depth);
    
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let entry_path = entry.path();
            
            if entry_path.is_dir() {
                println!("{}├── 📁 {}/", indent, name);
                print_dir(&entry_path, depth + 1);
            } else {
                let size = std::fs::metadata(&entry_path)
                    .map(|m| m.len())
                    .unwrap_or(0);
                println!("{}├── 📄 {} ({} bytes)", indent, name, size);
            }
        }
    }
}
