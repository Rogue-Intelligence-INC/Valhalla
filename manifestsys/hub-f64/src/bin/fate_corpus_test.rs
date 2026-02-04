//! Fate 语料库完整测试
//! 
//! 加载项目 Rust 源码作为语料，由 Fate 处理存储
//! 使用 bincode 二进制格式存储

use fate_f64::TripleFateCombined;
use std::path::Path;
use std::fs;

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║   🦀 Fate Rust 语料测试 - 加载项目源码                           ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    let mut fate = TripleFateCombined::new();
    let project_root = Path::new("/home/jouly/AI/Valhalla");
    
    println!("📂 项目路径: {:?}\n", project_root);
    
    // ========== 初始状态 ==========
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📦 初始 Fate 状态:");
    println!("   tick: {}", fate.global_tick);
    println!("   系数: [{:.6}, {:.6}, {:.6}]",
             fate.cluster_equation.coef_a,
             fate.cluster_equation.coef_b,
             fate.cluster_equation.coef_c);
    println!();

    // ========== 加载 Rust 源码 ==========
    // 只加载项目中的 .rs 文件
    let rust_dirs = vec![
        ("systems/metasystems/fate-f64/src", "Fate"),
        ("manifestsys/hub-f64/src", "Hub"),
        ("manifestsys/stem-cell/src", "StemCell"),
        ("manifestsys/tile-quattro/src", "Tile"),
        ("manifestsys/chamber/src", "Chamber"),
        ("systems/metasystems/blackhole/src", "BlackHole"),
        ("systems/metasystems/meshbrain/src", "MeshBrain"),
    ];
    
    let mut total_files = 0;
    let mut total_bytes = 0usize;
    let mut category_stats: Vec<(String, usize, usize)> = Vec::new();
    
    for (dir_path, module_name) in &rust_dirs {
        let full_path = project_root.join(dir_path);
        if !full_path.exists() {
            continue;
        }
        
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📥 加载 {} 模块 (.rs)...\n", module_name);
        
        let mut mod_files = 0;
        let mut mod_bytes = 0usize;
        
        // 只加载 .rs 文件
        load_rust_files(&mut fate, &full_path, &mut mod_files, &mut mod_bytes);
        
        total_files += mod_files;
        total_bytes += mod_bytes;
        category_stats.push((module_name.to_string(), mod_files, mod_bytes));
        
        println!("   {} 模块: {} 文件, {} bytes", module_name, mod_files, mod_bytes);
        println!("   当前 Fate tick: {}", fate.global_tick);
        println!("   当前系数: [{:.6}, {:.6}, {:.6}]",
                 fate.cluster_equation.coef_a,
                 fate.cluster_equation.coef_b,
                 fate.cluster_equation.coef_c);
        println!();
    }

    // ========== 系统自主处理 ==========
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔄 系统自主处理（基于已学习的模式）...\n");
    
    // 让系统基于已有数据创建新的派生数据
    let stored_ids: Vec<String> = fate.list_stored().iter().map(|s| (*s).clone()).collect();
    let mut derived_count = 0;
    
    // 对部分数据进行变换
    for id in stored_ids.iter().take(5) {
        if let Some(entry) = fate.load_data(id) {
            if entry.raw.len() > 10 {
                // 创建变换后的派生数据
                let transformed: Vec<u8> = entry.raw.iter()
                    .map(|&b| b.wrapping_add(fate.global_tick as u8 % 10))
                    .collect();
                fate.create_data(transformed);
                derived_count += 1;
            }
        }
    }
    
    // 合并相似签名的数据
    let sigs: Vec<(String, f64)> = fate.list_stored().iter()
        .filter_map(|id| fate.load_data(id).map(|e| ((*id).clone(), e.signature)))
        .collect();
    
    let mut merged_count = 0;
    for i in 0..sigs.len().min(3) {
        // 先获取相似的 ID 列表
        let similar_ids: Vec<String> = fate.find_similar(sigs[i].1, 100.0)
            .iter()
            .take(2)
            .map(|s| (*s).clone())
            .collect();
        
        if similar_ids.len() >= 2 {
            let ids: Vec<&str> = similar_ids.iter().map(|s| s.as_str()).collect();
            if fate.merge_data(&ids).is_some() {
                merged_count += 1;
            }
        }
    }
    
    println!("   派生数据: {} 条", derived_count);
    println!("   合并数据: {} 条", merged_count);
    println!();

    // ========== 最终统计 ==========
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 最终统计\n");

    println!("   语料加载:");
    for (cat, files, bytes) in &category_stats {
        println!("   - {}: {} 文件, {} bytes", cat, files, bytes);
    }
    println!("   ────────────────────────────");
    println!("   总计: {} 文件, {} bytes ({:.2} KB)", 
             total_files, total_bytes, total_bytes as f64 / 1024.0);
    println!();

    println!("   Fate 状态:");
    println!("   - tick: {}", fate.global_tick);
    println!("   - base 信号数: {}", fate.base.state.signal_count);
    println!("   - base 发现数: {}", fate.base.state.discovery_count);
    println!("   - meta 信号数: {}", fate.meta.state.signal_count);
    println!("   - 方程系数: [{:.6}, {:.6}, {:.6}]",
             fate.cluster_equation.coef_a,
             fate.cluster_equation.coef_b,
             fate.cluster_equation.coef_c);
    println!("   - 收敛: {}", fate.cluster_equation.triggered);
    println!();

    println!("   存储统计:");
    println!("   - 总存储: {} 条", fate.stored_count());
    println!("   - 外部导入: {} 条", fate.list_external().len());
    println!("   - 内部创建: {} 条", fate.list_internal().len());
    println!("   - 总修改次数: {}", fate.total_modifications());
    println!();

    // ========== 签名分布 ==========
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📈 签名分布分析\n");

    let signatures: Vec<f64> = fate.list_stored().iter()
        .filter_map(|id| fate.load_data(id).map(|e| e.signature))
        .collect();
    
    if !signatures.is_empty() {
        let min_sig = signatures.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_sig = signatures.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let avg_sig = signatures.iter().sum::<f64>() / signatures.len() as f64;
        let variance: f64 = signatures.iter()
            .map(|&s| (s - avg_sig).powi(2))
            .sum::<f64>() / signatures.len() as f64;
        
        println!("   签名范围: {:.2} ~ {:.2}", min_sig, max_sig);
        println!("   平均签名: {:.2}", avg_sig);
        println!("   签名方差: {:.2}", variance);
        println!("   签名标准差: {:.2}", variance.sqrt());
        
        // 分桶统计
        let buckets = 5;
        let bucket_size = (max_sig - min_sig) / buckets as f64;
        let mut counts = vec![0usize; buckets];
        
        for &sig in &signatures {
            let idx = ((sig - min_sig) / bucket_size).floor() as usize;
            let idx = idx.min(buckets - 1);
            counts[idx] += 1;
        }
        
        println!("\n   签名分布:");
        for i in 0..buckets {
            let low = min_sig + i as f64 * bucket_size;
            let high = low + bucket_size;
            let bar: String = "█".repeat(counts[i].min(40));
            println!("   [{:8.1} ~ {:8.1}]: {:3} {}", low, high, counts[i], bar);
        }
    }
    println!();

    // ========== 保存 Fate 切片 (二进制) ==========
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("❄️  保存 Fate 切片 (bincode 二进制)\n");

    let slice_path = Path::new("/tmp/fate_rust.fate");
    fate.save_slice(slice_path).expect("保存失败");
    let file_size = fs::metadata(slice_path).map(|m| m.len()).unwrap_or(0);
    
    println!("   保存到: {:?}", slice_path);
    println!("   格式: bincode (Rust 原生二进制)");
    println!("   文件大小: {} bytes ({:.2} KB)", file_size, file_size as f64 / 1024.0);

    // ========== 验证恢复 ==========
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔄 验证恢复\n");

    let restored = TripleFateCombined::load_slice(slice_path).expect("加载失败");
    println!("   恢复后 tick: {}", restored.global_tick);
    println!("   恢复后存储: {} 条", restored.stored_count());
    println!("   恢复后系数: [{:.6}, {:.6}, {:.6}]",
             restored.cluster_equation.coef_a,
             restored.cluster_equation.coef_b,
             restored.cluster_equation.coef_c);
    
    // 验证数据完整性
    let orig_count = fate.stored_count();
    let restored_count = restored.stored_count();
    let integrity = if orig_count == restored_count { "✓ 完整" } else { "✗ 不一致" };
    println!("   数据完整性: {}", integrity);

    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║   ✅ Rust 语料测试完成                                           ║");
    println!("║   {} .rs 文件 → Fate → {} 条 → {:.2} KB (.fate)              ║", 
             total_files, fate.stored_count(), file_size as f64 / 1024.0);
    println!("╚════════════════════════════════════════════════════════════════╝");
}

/// 递归加载目录下所有 .rs 文件
fn load_rust_files(fate: &mut TripleFateCombined, dir: &Path, files: &mut usize, bytes: &mut usize) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // 跳过 target 目录
                if path.file_name().map(|n| n == "target").unwrap_or(false) {
                    continue;
                }
                load_rust_files(fate, &path, files, bytes);
            } else if path.is_file() {
                // 只加载 .rs 文件
                if path.extension().map(|e| e == "rs").unwrap_or(false) {
                    if let Ok(content) = fs::read(&path) {
                        let size = content.len();
                        let id = fate.store_data(&content);
                        
                        let filename = path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown");
                        
                        println!("   [{:>6} bytes] {} → {}", size, filename, id);
                        
                        *files += 1;
                        *bytes += size;
                    }
                }
            }
        }
    }
}
