//! StemCell 全语料迭代训练 - Training Guardian 控制
//!
//! 特点:
//! - 使用全部可用语料
//! - Training Guardian 自动控制训练周期
//! - 防止过度训练
//! - 累积学习

use stem_cell::{StemCellComplete, TrainingPhase, TrainingDecision};
use std::path::Path;
use std::fs;
use std::io::Read;

fn main() {
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("       StemCell 全语料迭代训练 - Training Guardian 控制");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    let mut stemcell = StemCellComplete::new();
    
    println!("📊 初始状态:");
    println!("   细胞数: {}", stemcell.cells.len());
    println!("   训练阶段: {:?}", stemcell.training_phase());
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 加载语料
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("  加载语料");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    let mut all_data: Vec<f64> = Vec::new();
    
    // 1. Valhalla 项目源码
    println!("   📁 加载 Valhalla 项目源码...");
    let valhalla_data = load_rust_files("/home/jouly/AI/Valhalla", 500);
    println!("      文件数: {}, 数据量: {} KB", valhalla_data.1, valhalla_data.0.len() / 1024);
    all_data.extend(valhalla_data.0);
    
    // 2. Cargo 注册表
    println!("   📁 加载 Cargo 注册表源码...");
    let cargo_data = load_rust_files("/home/jouly/.cargo/registry/src", 1000);
    println!("      文件数: {}, 数据量: {} KB", cargo_data.1, cargo_data.0.len() / 1024);
    all_data.extend(cargo_data.0);
    
    println!();
    println!("   📊 总语料: {} KB ({} 字节)", all_data.len() / 1024, all_data.len());
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // Guardian 控制的迭代训练
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("  开始 Guardian 控制的迭代训练");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    let max_iterations = 50;  // 最大迭代数 (安全限制)
    let batch_size = 100_000; // 每批数据量
    let mut iteration = 0;
    let mut total_data_processed = 0usize;
    
    // 记录每次迭代的状态
    let mut history: Vec<(usize, usize, f64, TrainingPhase)> = Vec::new();
    
    while iteration < max_iterations {
        iteration += 1;
        stemcell.start_training_epoch();
        
        println!("┌─────────────────────────────────────────────────────────────────────────────┐");
        println!("│  迭代 {}/{}                                                                │", iteration, max_iterations);
        println!("└─────────────────────────────────────────────────────────────────────────────┘");
        
        // 检查 Guardian 是否允许继续
        if !stemcell.should_continue_training() {
            println!("   ⛔ Training Guardian 决定停止训练");
            println!("   原因: {:?}", stemcell.training_phase());
            println!("   状态: {}", stemcell.training_status());
            break;
        }
        
        let cells_before = stemcell.cells.len();
        let lr = stemcell.current_learning_rate();
        
        // 分批输入数据
        let start_idx = ((iteration - 1) * batch_size) % all_data.len();
        let end_idx = (start_idx + batch_size).min(all_data.len());
        let batch_data = &all_data[start_idx..end_idx];
        
        println!("   📝 输入数据: {} ~ {} ({} 字节)", start_idx, end_idx, batch_data.len());
        println!("   📊 学习率: {:.4}", lr);
        
        // 输入数据
        for &value in batch_data {
            stemcell.receive_data(value * lr);
        }
        total_data_processed += batch_data.len();
        
        // 多次 evolve 让数据被处理
        for i in 0..5 {
            stemcell.evolve(0.5 + (i as f64 * 0.05));
            
            // 检查是否应该提前停止
            if !stemcell.should_continue_training() {
                println!("   ⚠️  Evolve {} 时 Guardian 要求停止", i + 1);
                break;
            }
        }
        
        let cells_after = stemcell.cells.len();
        let growth = cells_after as i64 - cells_before as i64;
        let phase = stemcell.training_phase();
        
        // 记录历史
        history.push((iteration, cells_after, lr, phase));
        
        // 显示状态
        println!("   📈 细胞: {} → {} ({:+})", cells_before, cells_after, growth);
        println!("   🎯 阶段: {:?}", phase);
        println!("   📋 {}", stemcell.training_status());
        
        // 根据阶段显示额外信息
        match phase {
            TrainingPhase::Overtraining => {
                println!("   🛑 检测到过度训练!");
                break;
            }
            TrainingPhase::Saturated => {
                println!("   📦 系统饱和，可能需要新类型数据");
            }
            TrainingPhase::Resting => {
                println!("   😴 系统休息中...");
            }
            TrainingPhase::Consolidating => {
                println!("   🔄 巩固阶段");
            }
            _ => {}
        }
        println!();
    }
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 最终分析
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("                    训练完成 - 最终分析");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    // 连接统计
    let mut total_connections = 0usize;
    let mut cells_with_connections = 0usize;
    let mut max_connections = 0usize;
    
    for cell in &stemcell.cells {
        let conn_count = cell.connections.len();
        total_connections += conn_count;
        if conn_count > 0 {
            cells_with_connections += 1;
            max_connections = max_connections.max(conn_count);
        }
    }
    
    let connectivity = if stemcell.cells.is_empty() {
        0.0
    } else {
        cells_with_connections as f64 / stemcell.cells.len() as f64 * 100.0
    };
    
    // 细胞分化统计
    let mut core_count = 0;
    let mut signal_count = 0;
    let mut active_count = 0;
    let mut high_var_count = 0;
    
    for cell in &stemcell.cells {
        if cell.energy > 0.8 && cell.signal_count > 1000 {
            core_count += 1;
        } else if cell.signal_count > 500 {
            signal_count += 1;
        } else if cell.energy > 0.5 {
            active_count += 1;
        } else if cell.variance > 100.0 {
            high_var_count += 1;
        }
    }
    
    println!("   ┌────────────────────────────┬───────────────┐");
    println!("   │ 指标                       │ 值            │");
    println!("   ├────────────────────────────┼───────────────┤");
    println!("   │ 总迭代数                   │ {:>13} │", iteration);
    println!("   │ 处理数据量                 │ {:>10} KB │", total_data_processed / 1024);
    println!("   │ 最终细胞数                 │ {:>13} │", stemcell.cells.len());
    println!("   │ 连通率                     │ {:>12.1}% │", connectivity);
    println!("   │ 总连接数                   │ {:>13} │", total_connections);
    println!("   │ 最大连接数                 │ {:>13} │", max_connections);
    println!("   │ Space 维度                 │ {:>13.2} │", stemcell.space.current_dimension);
    println!("   │ Fate Tick                  │ {:>13} │", stemcell.triple_fate.global_tick);
    println!("   └────────────────────────────┴───────────────┘");
    println!();
    
    println!("   🧬 细胞分化:");
    println!("      核心处理: {} ({:.1}%)", core_count, core_count as f64 / stemcell.cells.len() as f64 * 100.0);
    println!("      信号调节: {} ({:.1}%)", signal_count, signal_count as f64 / stemcell.cells.len() as f64 * 100.0);
    println!("      活跃处理: {} ({:.1}%)", active_count, active_count as f64 / stemcell.cells.len() as f64 * 100.0);
    println!("      高变异: {} ({:.1}%)", high_var_count, high_var_count as f64 / stemcell.cells.len() as f64 * 100.0);
    println!();
    
    // Guardian 状态
    println!("   🛡️ Training Guardian 最终状态:");
    println!("      阶段: {:?}", stemcell.training_phase());
    println!("      学习率: {:.4}", stemcell.current_learning_rate());
    println!("      摘要: {}", stemcell.training_status());
    println!();
    
    // 演化曲线
    println!("   📈 演化曲线:");
    println!("   迭代 │ 细胞数    │ 学习率   │ 阶段");
    println!("   ─────┼───────────┼──────────┼─────────────");
    for (iter, cells, lr, phase) in history.iter().take(20) {
        println!("   {:>4} │ {:>9} │ {:>8.4} │ {:?}", iter, cells, lr, phase);
    }
    if history.len() > 20 {
        println!("   ... (省略 {} 行)", history.len() - 20);
    }
    println!();
    
    // 保存状态
    let save_path = "/tmp/stemcell_guardian_trained.stemcell";
    match stemcell.save_slice(Path::new(save_path)) {
        Ok(_) => println!("   💾 状态已保存: {}", save_path),
        Err(e) => println!("   ⚠️  保存失败: {}", e),
    }
    
    println!();
    println!("═══════════════════════════════════════════════════════════════════════════════");
}

/// 加载 Rust 源码文件
fn load_rust_files(base_path: &str, max_files: usize) -> (Vec<f64>, usize) {
    let mut data = Vec::new();
    let mut file_count = 0;
    
    load_files_recursive(Path::new(base_path), &mut data, &mut file_count, max_files);
    
    (data, file_count)
}

fn load_files_recursive(path: &Path, data: &mut Vec<f64>, count: &mut usize, max: usize) {
    if *count >= max {
        return;
    }
    
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if *count >= max {
                break;
            }
            
            let path = entry.path();
            
            if path.is_dir() {
                // 跳过某些目录
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if name == "target" || name == "node_modules" || name == ".git" {
                    continue;
                }
                load_files_recursive(&path, data, count, max);
            } else if path.extension().map_or(false, |e| e == "rs") {
                // 读取 .rs 文件
                if let Ok(mut file) = fs::File::open(&path) {
                    let mut content = String::new();
                    if file.read_to_string(&mut content).is_ok() {
                        // 转换为 f64 数据
                        for byte in content.bytes() {
                            data.push(byte as f64);
                        }
                        *count += 1;
                    }
                }
            }
        }
    }
}
