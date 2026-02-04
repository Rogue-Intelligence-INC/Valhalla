//! StemCell 高维物理场智能测试
//!
//! 测试:
//! - 高维位置演化
//! - 能量场吸引/排斥
//! - 自然连接涌现
//! - 关联学习能力

use stem_cell::StemCellComplete;

fn main() {
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("       StemCell 高维物理场智能测试");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    let mut stemcell = StemCellComplete::new();
    
    println!("📊 初始状态:");
    println!("   细胞数: {}", stemcell.cells.len());
    println!("   Space 维度: {:.2}", stemcell.space.current_dimension);
    println!();
    
    // 检查初始位置
    if !stemcell.cells.is_empty() {
        let first_cell = &stemcell.cells[0];
        println!("   首个细胞位置维度: {}", first_cell.position.len());
        println!("   首个细胞连接数: {}", first_cell.connections.len());
    }
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 阶段 1: 输入相似数据，观察聚集
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("  阶段 1: 输入相似数据 - 观察能量吸引和聚集");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    // 输入 "cat" 多次
    println!("   📝 输入 'cat' 10 次:");
    for _ in 0..10 {
        for byte in "cat".bytes() {
            stemcell.receive_data(byte as f64);
        }
    }
    
    // evolve 以触发物理场更新
    for _ in 0..5 {
        stemcell.evolve(0.5);
    }
    
    let cells_after_cat = stemcell.cells.len();
    println!("      细胞数: {}", cells_after_cat);
    println!("      维度: {:.2}", stemcell.space.current_dimension);
    
    // 输入 "meow" 多次 (与 cat 关联)
    println!("   📝 输入 'meow' 10 次:");
    for _ in 0..10 {
        for byte in "meow".bytes() {
            stemcell.receive_data(byte as f64);
        }
    }
    
    for _ in 0..5 {
        stemcell.evolve(0.5);
    }
    
    let cells_after_meow = stemcell.cells.len();
    println!("      细胞数: {}", cells_after_meow);
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 阶段 2: 检查连接形成
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("  阶段 2: 检查自然连接涌现");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    // 统计连接
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
    
    println!("   📡 连接统计:");
    println!("      有连接的细胞: {} / {}", cells_with_connections, stemcell.cells.len());
    println!("      总连接数: {}", total_connections);
    println!("      最大连接数: {}", max_connections);
    
    if cells_with_connections > 0 {
        println!("      平均连接数: {:.2}", total_connections as f64 / cells_with_connections as f64);
    }
    println!();
    
    // 显示一些具体连接
    println!("   🔗 部分连接示例:");
    for (i, cell) in stemcell.cells.iter().enumerate().take(5) {
        if !cell.connections.is_empty() {
            let connections: Vec<String> = cell.connections.iter()
                .take(3)
                .map(|(id, strength)| format!("{}({:.2})", id, strength))
                .collect();
            println!("      {}: -> [{}]", cell.cell_id, connections.join(", "));
        }
    }
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 阶段 3: 输入不相关数据，观察排斥
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("  阶段 3: 输入不相关数据 - 观察能量排斥");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    // 输入完全不同的数据
    println!("   📝 输入 '12345' 10 次:");
    for _ in 0..10 {
        for byte in "12345".bytes() {
            stemcell.receive_data(byte as f64);
        }
    }
    
    for _ in 0..10 {
        stemcell.evolve(0.8);
    }
    
    println!("      细胞数: {}", stemcell.cells.len());
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 阶段 4: 分析位置分布
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("  阶段 4: 高维位置分布分析");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    // 位置维度
    let pos_dims: Vec<usize> = stemcell.cells.iter()
        .map(|c| c.position.len())
        .collect();
    let min_dim = *pos_dims.iter().min().unwrap_or(&0);
    let max_dim = *pos_dims.iter().max().unwrap_or(&0);
    
    println!("   📐 位置维度: {} ~ {}", min_dim, max_dim);
    println!("   📐 Space 当前维度: {:.2}", stemcell.space.current_dimension);
    println!();
    
    // 计算位置分布范围
    if !stemcell.cells.is_empty() && min_dim > 0 {
        let common_dim = min_dim.min(5);  // 取前5维分析
        
        println!("   📊 前 {} 维位置分布:", common_dim);
        for d in 0..common_dim {
            let values: Vec<f64> = stemcell.cells.iter()
                .filter(|c| c.position.len() > d)
                .map(|c| c.position[d])
                .collect();
            
            if !values.is_empty() {
                let min_val = values.iter().cloned().fold(f64::INFINITY, f64::min);
                let max_val = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let mean = values.iter().sum::<f64>() / values.len() as f64;
                
                println!("      维度 {}: [{:.3}, {:.3}] 均值={:.3}", d, min_val, max_val, mean);
            }
        }
    }
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 阶段 5: 关联测试
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("  阶段 5: 基于物理连接的关联测试");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    // 建立关联对
    let associations = vec![
        ("happy", "joy"),
        ("sad", "tears"),
        ("big", "huge"),
    ];
    
    println!("   📚 建立关联:");
    for (a, b) in &associations {
        // 交替输入 a 和 b 以建立时间关联
        for _ in 0..20 {
            for byte in a.bytes() {
                stemcell.receive_data(byte as f64);
            }
            for byte in b.bytes() {
                stemcell.receive_data(byte as f64);
            }
        }
        stemcell.evolve(0.7);
        println!("      {} <-> {}: 完成", a, b);
    }
    println!();
    
    // 多次 evolve 让物理场稳定
    println!("   ⏳ 物理场稳定化 (20 次 evolve)...");
    for _ in 0..20 {
        stemcell.evolve(0.6);
    }
    println!();
    
    // 再次检查连接
    let mut total_connections = 0usize;
    let mut cells_with_connections = 0usize;
    
    for cell in &stemcell.cells {
        let conn_count = cell.connections.len();
        total_connections += conn_count;
        if conn_count > 0 {
            cells_with_connections += 1;
        }
    }
    
    println!("   📡 最终连接统计:");
    println!("      有连接的细胞: {} / {}", cells_with_connections, stemcell.cells.len());
    println!("      总连接数: {}", total_connections);
    
    let connectivity_ratio = cells_with_connections as f64 / stemcell.cells.len() as f64;
    let avg_connections = if cells_with_connections > 0 {
        total_connections as f64 / cells_with_connections as f64
    } else {
        0.0
    };
    
    println!("      连通率: {:.1}%", connectivity_ratio * 100.0);
    println!("      平均连接: {:.2}", avg_connections);
    println!();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 综合评估
    // ═══════════════════════════════════════════════════════════════════════════
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("                    综合评估");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    // 计算各项得分
    let connectivity_score = (connectivity_ratio * 10.0).min(10.0);
    let connection_density_score = (avg_connections / 5.0 * 10.0).min(10.0);
    let dimension_adaptivity = ((stemcell.space.current_dimension - 4.0) / 30.0 * 10.0).min(10.0).max(0.0);
    
    let total_score = (connectivity_score + connection_density_score + dimension_adaptivity) / 3.0;
    
    println!("   ┌────────────────────────────┬────────┐");
    println!("   │ 指标                       │  得分  │");
    println!("   ├────────────────────────────┼────────┤");
    println!("   │ 连通率                     │ {:>5.2} │", connectivity_score);
    println!("   │ 连接密度                   │ {:>5.2} │", connection_density_score);
    println!("   │ 维度自适应                 │ {:>5.2} │", dimension_adaptivity);
    println!("   ├────────────────────────────┼────────┤");
    println!("   │ 物理场智能总分             │ {:>5.2} │", total_score);
    println!("   └────────────────────────────┴────────┘");
    println!();
    
    println!("   🏁 最终状态:");
    println!("      细胞总数: {}", stemcell.cells.len());
    println!("      Space 维度: {:.2}", stemcell.space.current_dimension);
    println!("      Fate Tick: {}", stemcell.triple_fate.global_tick);
    println!();
    println!("═══════════════════════════════════════════════════════════════════════════════");
}
