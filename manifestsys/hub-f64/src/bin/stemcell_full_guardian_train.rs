//! StemCell 完整全语料迭代训练 - Training Guardian 控制
//!
//! 大规模训练版本

use stem_cell::{StemCellComplete, TrainingPhase};
use std::path::Path;
use std::fs;
use std::io::Read;
use std::time::Instant;

fn main() {
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("       StemCell 完整全语料训练 - Training Guardian 控制");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    let start_time = Instant::now();
    let mut stemcell = StemCellComplete::new();
    
    println!("📊 初始状态:");
    println!("   细胞数: {}", stemcell.cells.len());
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 加载全部语料
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("  加载全部语料");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    let mut all_data: Vec<f64> = Vec::new();
    
    // 1. Valhalla 项目源码 (无限制)
    println!("   📁 加载 Valhalla 项目源码...");
    let valhalla_data = load_rust_files("/home/jouly/AI/Valhalla", 10000);
    println!("      文件数: {}, 数据量: {} KB", valhalla_data.1, valhalla_data.0.len() / 1024);
    all_data.extend(valhalla_data.0);
    
    // 2. Cargo 注册表 (加载更多)
    println!("   📁 加载 Cargo 注册表源码...");
    let cargo_data = load_rust_files("/home/jouly/.cargo/registry/src", 5000);
    println!("      文件数: {}, 数据量: {} KB", cargo_data.1, cargo_data.0.len() / 1024);
    all_data.extend(cargo_data.0);
    
    let total_kb = all_data.len() / 1024;
    println!();
    println!("   📊 总语料: {} KB ({} MB)", total_kb, total_kb / 1024);
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // Guardian 控制的大规模迭代训练
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("  开始大规模训练 (Guardian 控制)");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    let max_iterations = 100;
    let batch_size = 500_000;  // 500KB 每批
    let evolve_rounds = 10;    // 每次迭代 evolve 次数
    let mut iteration = 0;
    let mut total_data_processed = 0usize;
    let mut stopped_by_guardian = false;
    
    while iteration < max_iterations {
        iteration += 1;
        stemcell.start_training_epoch();
        
        // 检查 Guardian
        if !stemcell.should_continue_training() {
            println!("┌─────────────────────────────────────────────────────────────────────────────┐");
            println!("│  ⛔ Training Guardian 停止训练                                              │");
            println!("└─────────────────────────────────────────────────────────────────────────────┘");
            println!("   阶段: {:?}", stemcell.training_phase());
            println!("   状态: {}", stemcell.training_status());
            stopped_by_guardian = true;
            break;
        }
        
        let cells_before = stemcell.cells.len();
        let lr = stemcell.current_learning_rate();
        
        // 循环遍历数据
        let start_idx = ((iteration - 1) * batch_size) % all_data.len();
        let end_idx = (start_idx + batch_size).min(all_data.len());
        let batch = &all_data[start_idx..end_idx];
        
        // 每 10 次迭代显示详细进度
        if iteration % 10 == 1 || iteration <= 5 {
            println!("┌─────────────────────────────────────────────────────────────────────────────┐");
            println!("│  迭代 {}/{}  |  数据: {} KB  |  LR: {:.4}                    │", 
                iteration, max_iterations, batch.len() / 1024, lr);
            println!("└─────────────────────────────────────────────────────────────────────────────┘");
        }
        
        // 输入数据
        for &value in batch {
            stemcell.receive_data(value * lr);
        }
        total_data_processed += batch.len();
        
        // 多次 evolve
        for i in 0..evolve_rounds {
            stemcell.evolve(0.5 + (i as f64 * 0.03));
            
            if !stemcell.should_continue_training() {
                println!("   ⚠️  Guardian 在 evolve {} 时要求停止", i + 1);
                stopped_by_guardian = true;
                break;
            }
        }
        
        if stopped_by_guardian {
            break;
        }
        
        let cells_after = stemcell.cells.len();
        let growth = cells_after as i64 - cells_before as i64;
        let phase = stemcell.training_phase();
        
        // 简洁输出
        if iteration % 10 == 0 {
            println!("   迭代 {:>3}: 细胞 {:>6} ({:+5}) | {:?} | LR: {:.4}", 
                iteration, cells_after, growth, phase, lr);
        }
        
        // 特殊阶段处理
        match phase {
            TrainingPhase::Overtraining => {
                println!("   🛑 过度训练!");
                stopped_by_guardian = true;
                break;
            }
            TrainingPhase::Saturated => {
                if iteration % 10 == 0 {
                    println!("   📦 系统饱和");
                }
            }
            _ => {}
        }
    }
    
    let elapsed = start_time.elapsed();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 最终分析
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!();
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
    
    // 细胞分化
    let mut differentiation = [0usize; 5];  // core, signal, active, high_var, normal
    for cell in &stemcell.cells {
        if cell.energy > 0.8 && cell.signal_count > 1000 {
            differentiation[0] += 1;  // core
        } else if cell.signal_count > 500 {
            differentiation[1] += 1;  // signal
        } else if cell.energy > 0.5 {
            differentiation[2] += 1;  // active
        } else if cell.variance > 100.0 {
            differentiation[3] += 1;  // high_var
        } else {
            differentiation[4] += 1;  // normal
        }
    }
    
    let throughput = total_data_processed as f64 / elapsed.as_secs_f64() / 1024.0 / 1024.0;
    
    println!("   ┌────────────────────────────┬───────────────────┐");
    println!("   │ 指标                       │ 值                │");
    println!("   ├────────────────────────────┼───────────────────┤");
    println!("   │ 总迭代数                   │ {:>17} │", iteration);
    println!("   │ 处理数据量                 │ {:>14} MB │", total_data_processed / 1024 / 1024);
    println!("   │ 训练时间                   │ {:>14.2}s │", elapsed.as_secs_f64());
    println!("   │ 吞吐量                     │ {:>12.2} MB/s │", throughput);
    println!("   │ Guardian 停止              │ {:>17} │", if stopped_by_guardian { "是" } else { "否" });
    println!("   ├────────────────────────────┼───────────────────┤");
    println!("   │ 最终细胞数                 │ {:>17} │", stemcell.cells.len());
    println!("   │ 连通率                     │ {:>16.1}% │", connectivity);
    println!("   │ 总连接数                   │ {:>17} │", total_connections);
    println!("   │ 最大连接数                 │ {:>17} │", max_connections);
    println!("   │ Space 维度                 │ {:>17.2} │", stemcell.space.current_dimension);
    println!("   │ Fate Tick                  │ {:>17} │", stemcell.triple_fate.global_tick);
    println!("   └────────────────────────────┴───────────────────┘");
    println!();
    
    println!("   🧬 细胞分化:");
    let total = stemcell.cells.len() as f64;
    println!("      核心处理: {:>6} ({:>5.1}%)", differentiation[0], differentiation[0] as f64 / total * 100.0);
    println!("      信号调节: {:>6} ({:>5.1}%)", differentiation[1], differentiation[1] as f64 / total * 100.0);
    println!("      活跃处理: {:>6} ({:>5.1}%)", differentiation[2], differentiation[2] as f64 / total * 100.0);
    println!("      高变异:   {:>6} ({:>5.1}%)", differentiation[3], differentiation[3] as f64 / total * 100.0);
    println!("      普通:     {:>6} ({:>5.1}%)", differentiation[4], differentiation[4] as f64 / total * 100.0);
    println!();
    
    println!("   🛡️ Training Guardian:");
    println!("      最终阶段: {:?}", stemcell.training_phase());
    println!("      学习率: {:.4}", stemcell.current_learning_rate());
    println!("      {}", stemcell.training_status());
    println!();
    
    // 保存
    let save_path = "/tmp/stemcell_full_trained.stemcell";
    match stemcell.save_slice(Path::new(save_path)) {
        Ok(_) => println!("   💾 已保存: {}", save_path),
        Err(e) => println!("   ⚠️  保存失败: {}", e),
    }
    
    println!();
    println!("═══════════════════════════════════════════════════════════════════════════════");
}

fn load_rust_files(base_path: &str, max_files: usize) -> (Vec<f64>, usize) {
    let mut data = Vec::new();
    let mut file_count = 0;
    load_files_recursive(Path::new(base_path), &mut data, &mut file_count, max_files);
    (data, file_count)
}

fn load_files_recursive(path: &Path, data: &mut Vec<f64>, count: &mut usize, max: usize) {
    if *count >= max { return; }
    
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if *count >= max { break; }
            
            let path = entry.path();
            
            if path.is_dir() {
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if name == "target" || name == "node_modules" || name == ".git" { continue; }
                load_files_recursive(&path, data, count, max);
            } else if path.extension().map_or(false, |e| e == "rs") {
                if let Ok(mut file) = fs::File::open(&path) {
                    let mut content = String::new();
                    if file.read_to_string(&mut content).is_ok() {
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
