//! StemCell 自主训练 - Fate 控制训练周期
//!
//! 特点:
//! - Training Guardian 监控训练状态
//! - 自动检测过度训练
//! - 自适应学习率
//! - 无预设训练轮数

use stem_cell::{StemCellComplete, TrainingPhase};

fn main() {
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("       StemCell 自主训练 - Fate 控制训练周期");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    let mut stemcell = StemCellComplete::new();
    
    println!("📊 初始状态:");
    println!("   细胞数: {}", stemcell.cells.len());
    println!("   训练阶段: {:?}", stemcell.training_phase());
    println!("   学习率: {:.3}", stemcell.current_learning_rate());
    println!();
    
    // 准备训练数据
    let training_patterns = vec![
        "hello world",
        "machine learning",
        "neural network",
        "deep learning",
        "artificial intelligence",
        "pattern recognition",
        "data science",
        "computer vision",
    ];
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 自主训练循环 - 由 Fate 决定何时停止
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("  开始自主训练 (Fate 控制)");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    let mut epoch = 0;
    let mut total_iterations = 0;
    let max_epochs = 20;  // 最大 epoch 数 (安全限制)
    
    while epoch < max_epochs {
        epoch += 1;
        stemcell.start_training_epoch();
        
        println!("┌─────────────────────────────────────────────────────────────────────────────┐");
        println!("│  Epoch {}/{}                                                                │", epoch, max_epochs);
        println!("└─────────────────────────────────────────────────────────────────────────────┘");
        
        // 检查是否应该继续
        if !stemcell.should_continue_training() {
            println!("   ⛔ Guardian 决定停止训练");
            println!("   状态: {}", stemcell.training_status());
            break;
        }
        
        let cells_before = stemcell.cells.len();
        let lr = stemcell.current_learning_rate();
        
        // 输入训练数据
        for pattern in &training_patterns {
            for _ in 0..10 {  // 每个模式重复 10 次
                for byte in pattern.bytes() {
                    stemcell.receive_data(byte as f64 * lr);
                }
            }
        }
        
        // 多次 evolve
        for i in 0..10 {
            let result = stemcell.evolve(0.5 + (i as f64 * 0.02));
            total_iterations += 1;
            
            // 每次 evolve 检查状态
            if !stemcell.should_continue_training() {
                println!("   ⚠️  Epoch {} 中途停止 (迭代 {})", epoch, i + 1);
                break;
            }
        }
        
        let cells_after = stemcell.cells.len();
        let growth = cells_after as i64 - cells_before as i64;
        
        // 显示状态
        println!("   📈 细胞: {} → {} ({:+})", cells_before, cells_after, growth);
        println!("   📊 学习率: {:.3}", stemcell.current_learning_rate());
        println!("   🎯 阶段: {:?}", stemcell.training_phase());
        println!("   📋 {}", stemcell.training_status());
        println!();
        
        // 根据阶段调整行为
        match stemcell.training_phase() {
            TrainingPhase::Overtraining => {
                println!("   🛑 检测到过度训练，停止!");
                break;
            }
            TrainingPhase::Saturated => {
                println!("   📦 系统饱和，需要新数据");
                // 可以在这里添加新数据
            }
            TrainingPhase::Resting => {
                println!("   😴 系统休息中...");
            }
            TrainingPhase::Consolidating => {
                println!("   🔄 巩固阶段，降低学习率");
            }
            TrainingPhase::Exploring => {
                // 正常探索
            }
        }
    }
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 最终结果
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("                    训练完成 - 最终状态");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    // 连接统计
    let mut total_connections = 0usize;
    let mut cells_with_connections = 0usize;
    
    for cell in &stemcell.cells {
        if !cell.connections.is_empty() {
            cells_with_connections += 1;
            total_connections += cell.connections.len();
        }
    }
    
    let connectivity = cells_with_connections as f64 / stemcell.cells.len() as f64 * 100.0;
    
    println!("   ┌────────────────────────────┬───────────────┐");
    println!("   │ 指标                       │ 值            │");
    println!("   ├────────────────────────────┼───────────────┤");
    println!("   │ 总 Epoch                   │ {:>13} │", epoch);
    println!("   │ 总迭代                     │ {:>13} │", total_iterations);
    println!("   │ 最终细胞数                 │ {:>13} │", stemcell.cells.len());
    println!("   │ 连通率                     │ {:>12.1}% │", connectivity);
    println!("   │ 总连接数                   │ {:>13} │", total_connections);
    println!("   │ Space 维度                 │ {:>13.2} │", stemcell.space.current_dimension);
    println!("   │ Fate Tick                  │ {:>13} │", stemcell.triple_fate.global_tick);
    println!("   └────────────────────────────┴───────────────┘");
    println!();
    
    // Guardian 详情
    println!("   🛡️ Training Guardian 状态:");
    println!("      最终阶段: {:?}", stemcell.training_phase());
    println!("      最终学习率: {:.4}", stemcell.current_learning_rate());
    println!("      状态摘要: {}", stemcell.training_status());
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 测试自主训练 API
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("  测试自主训练 API (autonomous_train)");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    // 重置训练状态
    stemcell.reset_training();
    println!("   ✅ 训练状态已重置");
    println!("   阶段: {:?}", stemcell.training_phase());
    println!();
    
    // 新数据
    let new_data: Vec<f64> = "quantum computing blockchain cryptocurrency"
        .bytes()
        .map(|b| b as f64)
        .collect();
    
    println!("   📝 使用新数据进行自主训练...");
    let (iterations, decision) = stemcell.autonomous_train(&new_data, 100);
    
    println!("   📊 结果:");
    println!("      迭代数: {}", iterations);
    println!("      最终决策: {:?}", decision);
    println!("      细胞数: {}", stemcell.cells.len());
    println!("      阶段: {:?}", stemcell.training_phase());
    println!();
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("   ✅ 自主训练演示完成");
    println!("   Fate 成功控制了训练周期，防止过度训练");
    println!("═══════════════════════════════════════════════════════════════════════════════");
}
