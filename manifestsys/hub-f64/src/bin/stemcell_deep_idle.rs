//! StemCell 深度空跑测试
//! 
//! 持续空跑直到系统完全稳定，观察最终形态

use stem_cell::StemCellComplete;
use std::time::Instant;

fn main() {
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("   StemCell 深度空跑测试 - 观察整理完成后的最终形态");
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!();
    
    // ═══ 阶段 1: 训练 ═══
    println!("【阶段 1】训练系统...\n");
    
    let mut stemcell = StemCellComplete::new();
    
    // 更丰富的训练数据
    let training_data = vec![
        // 数字
        "0", "1", "2", "3", "4", "5", "6", "7", "8", "9",
        "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten",
        // 颜色
        "red", "green", "blue", "yellow", "orange", "purple", "black", "white", "pink", "gray",
        // 动物
        "cat", "dog", "bird", "fish", "lion", "tiger", "elephant", "mouse", "rabbit", "snake",
        // 动词
        "run", "walk", "jump", "swim", "fly", "sleep", "eat", "drink", "think", "speak",
        // 形容词
        "big", "small", "fast", "slow", "hot", "cold", "good", "bad", "new", "old",
        // 短语
        "the cat runs", "a dog walks", "bird flies high", "fish swims fast",
        "red cat", "blue fish", "green bird", "yellow sun",
        "big elephant", "small mouse", "fast rabbit", "slow turtle",
        // 代码片段
        "fn main", "let x", "if true", "for i in", "while loop", "struct Cell",
        "impl Trait", "pub fn", "use std", "mod test",
    ];
    
    for text in &training_data {
        for ch in text.chars() {
            stemcell.receive_data(ch as u8 as f64);
        }
        stemcell.receive_data(0.0);
    }
    
    // 初步稳定
    for _ in 0..20 {
        stemcell.evolve(0.0);
    }
    
    println!("训练完成！");
    println!("  细胞数: {}", stemcell.cells.len());
    println!("  维度:   {:.2}", stemcell.space.current_dimension);
    
    // ═══ 阶段 2: 深度空跑 ═══
    println!("\n【阶段 2】深度空跑 - 直到完全稳定\n");
    
    let initial = capture_state(&stemcell);
    print_state("初始状态", &initial);
    
    println!("\n开始深度空跑...\n");
    println!("{:>8} | {:>6} | {:>8} | {:>8} | {:>10} | {:>12}",
        "tick", "cells", "conn", "dim", "Δconn/100", "状态");
    println!("{}", "-".repeat(75));
    
    let start_time = Instant::now();
    let mut prev_connections = initial.total_connections;
    let mut stable_count = 0;
    let mut total_ticks = 0u64;
    let max_ticks = 50000u64;  // 最多 5 万 tick
    
    let mut checkpoints: Vec<(u64, SystemState)> = vec![(0, initial)];
    
    loop {
        // 每次跑 100 tick
        for _ in 0..100 {
            stemcell.evolve(0.0);
            total_ticks += 1;
        }
        
        let state = capture_state(&stemcell);
        let delta_conn = state.total_connections as i64 - prev_connections as i64;
        
        // 判断稳定性
        let status = if delta_conn.abs() == 0 {
            stable_count += 1;
            if stable_count >= 10 {
                "完全稳定 ✓"
            } else {
                "趋于稳定..."
            }
        } else {
            stable_count = 0;
            if delta_conn > 0 { "连接增长 ↑" } else { "连接减少 ↓" }
        };
        
        // 输出
        if total_ticks <= 1000 || total_ticks % 1000 == 0 || stable_count >= 10 {
            println!("{:>8} | {:>6} | {:>8} | {:>8.2} | {:>+10} | {:>12}",
                total_ticks, state.cell_count, state.total_connections,
                state.dimension, delta_conn, status);
        }
        
        // 记录检查点
        if total_ticks % 5000 == 0 {
            checkpoints.push((total_ticks, state.clone()));
        }
        
        prev_connections = state.total_connections;
        
        // 终止条件
        if stable_count >= 10 {
            println!("\n系统已完全稳定！（连续 {} 个周期无变化）", stable_count);
            checkpoints.push((total_ticks, state));
            break;
        }
        
        if total_ticks >= max_ticks {
            println!("\n达到最大 tick 数 ({})", max_ticks);
            checkpoints.push((total_ticks, state));
            break;
        }
    }
    
    let elapsed = start_time.elapsed();
    println!("\n空跑耗时: {:.2}s", elapsed.as_secs_f64());
    
    // ═══ 阶段 3: 分析最终形态 ═══
    println!("\n【阶段 3】最终形态分析\n");
    
    let final_state = capture_state(&stemcell);
    print_state("最终状态", &final_state);
    
    // 检查点对比
    println!("\n演化轨迹:");
    println!("{:>8} | {:>6} | {:>8} | {:>8} | {:>10}",
        "tick", "cells", "conn", "dim", "conn/cell");
    println!("{}", "-".repeat(55));
    
    for (tick, state) in &checkpoints {
        println!("{:>8} | {:>6} | {:>8} | {:>8.2} | {:>10.2}",
            tick, state.cell_count, state.total_connections,
            state.dimension, state.avg_connections_per_cell);
    }
    
    // ═══ 阶段 4: 连接网络深度分析 ═══
    println!("\n【阶段 4】连接网络深度分析\n");
    
    analyze_connection_network(&stemcell);
    
    // ═══ 阶段 5: 细胞分布分析 ═══
    println!("\n【阶段 5】细胞分布分析\n");
    
    analyze_cell_distribution(&stemcell);
    
    // ═══ 结论 ═══
    println!("\n═══════════════════════════════════════════════════════════════════════════");
    println!("   最终形态总结");
    println!("═══════════════════════════════════════════════════════════════════════════\n");
    
    let initial_state = &checkpoints[0].1;
    
    println!("从训练后到完全稳定的变化:");
    println!("  tick 数:        {} → {}", 0, total_ticks);
    println!("  连接数:         {} → {} ({:+})", 
        initial_state.total_connections, 
        final_state.total_connections,
        final_state.total_connections as i64 - initial_state.total_connections as i64);
    println!("  连接密度:       {:.2} → {:.2}",
        initial_state.avg_connections_per_cell,
        final_state.avg_connections_per_cell);
    println!("  连接增长率:     {:.2}%",
        (final_state.total_connections as f64 / initial_state.total_connections as f64 - 1.0) * 100.0);
    
    println!("\n系统在无输入情况下的\"整理\"行为:");
    if total_ticks < 1000 {
        println!("  快速收敛 - 系统很快达到平衡");
    } else if total_ticks < 10000 {
        println!("  中速收敛 - 系统需要一定时间整理");
    } else {
        println!("  慢速收敛 - 系统内部结构复杂，需要长时间整理");
    }
}

#[derive(Clone)]
struct SystemState {
    cell_count: usize,
    total_connections: usize,
    avg_connections_per_cell: f64,
    dimension: f64,
    avg_energy: f64,
    energy_variance: f64,
    max_connections: usize,
    min_connections: usize,
}

fn capture_state(stemcell: &StemCellComplete) -> SystemState {
    let cells = &stemcell.cells;
    
    if cells.is_empty() {
        return SystemState {
            cell_count: 0,
            total_connections: 0,
            avg_connections_per_cell: 0.0,
            dimension: stemcell.space.current_dimension,
            avg_energy: 0.0,
            energy_variance: 0.0,
            max_connections: 0,
            min_connections: 0,
        };
    }
    
    let connection_counts: Vec<usize> = cells.iter().map(|c| c.connections.len()).collect();
    let total_connections: usize = connection_counts.iter().sum();
    let avg_connections = total_connections as f64 / cells.len() as f64;
    let max_conn = *connection_counts.iter().max().unwrap_or(&0);
    let min_conn = *connection_counts.iter().min().unwrap_or(&0);
    
    let energies: Vec<f64> = cells.iter().map(|c| c.energy).collect();
    let avg_energy = energies.iter().sum::<f64>() / energies.len() as f64;
    let energy_variance = energies.iter()
        .map(|e| (e - avg_energy).powi(2))
        .sum::<f64>() / energies.len() as f64;
    
    SystemState {
        cell_count: cells.len(),
        total_connections,
        avg_connections_per_cell: avg_connections,
        dimension: stemcell.space.current_dimension,
        avg_energy,
        energy_variance,
        max_connections: max_conn,
        min_connections: min_conn,
    }
}

fn print_state(label: &str, state: &SystemState) {
    println!("{}:", label);
    println!("  细胞数:           {}", state.cell_count);
    println!("  总连接数:         {}", state.total_connections);
    println!("  平均连接/细胞:    {:.2}", state.avg_connections_per_cell);
    println!("  连接范围:         {} ~ {}", state.min_connections, state.max_connections);
    println!("  维度:             {:.2}", state.dimension);
    println!("  平均能量:         {:.6}", state.avg_energy);
    println!("  能量方差:         {:.6}", state.energy_variance);
}

fn analyze_connection_network(stemcell: &StemCellComplete) {
    let cells = &stemcell.cells;
    if cells.is_empty() {
        println!("无细胞");
        return;
    }
    
    // 连接分布
    let mut conn_histogram: Vec<usize> = vec![0; 21];  // 0-20
    for cell in cells {
        let bucket = cell.connections.len().min(20);
        conn_histogram[bucket] += 1;
    }
    
    println!("连接数分布 (每个细胞的连接数):");
    for (i, &count) in conn_histogram.iter().enumerate() {
        if count > 0 {
            let bar_len = (count * 40 / cells.len()).max(1);
            let bar: String = "█".repeat(bar_len);
            println!("  {:>2} 连接: {:>4} 个细胞 ({:>5.1}%) {}", 
                i, count, count as f64 / cells.len() as f64 * 100.0, bar);
        }
    }
    
    // 连接强度分布
    let mut all_strengths: Vec<f64> = Vec::new();
    for cell in cells {
        for (_, strength) in &cell.connections {
            all_strengths.push(*strength);
        }
    }
    
    if !all_strengths.is_empty() {
        all_strengths.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let min_str = all_strengths.first().unwrap();
        let max_str = all_strengths.last().unwrap();
        let median_str = all_strengths[all_strengths.len() / 2];
        let avg_str = all_strengths.iter().sum::<f64>() / all_strengths.len() as f64;
        
        println!("\n连接强度统计:");
        println!("  最小: {:.4}", min_str);
        println!("  最大: {:.4}", max_str);
        println!("  中位: {:.4}", median_str);
        println!("  平均: {:.4}", avg_str);
    }
    
    // 检查是否形成了"核心"节点
    let mut high_conn_cells: Vec<(&str, usize)> = cells.iter()
        .map(|c| (c.cell_id.as_str(), c.connections.len()))
        .collect();
    high_conn_cells.sort_by(|a, b| b.1.cmp(&a.1));
    
    println!("\n连接数最多的细胞 (核心节点):");
    for (id, conn) in high_conn_cells.iter().take(5) {
        println!("  {}: {} 连接", id, conn);
    }
}

fn analyze_cell_distribution(stemcell: &StemCellComplete) {
    let cells = &stemcell.cells;
    if cells.is_empty() {
        println!("无细胞");
        return;
    }
    
    // 能量分布
    let mut energy_buckets = vec![0usize; 10];  // 0.0-0.1, 0.1-0.2, ...
    for cell in cells {
        let bucket = ((cell.energy * 10.0) as usize).min(9);
        energy_buckets[bucket] += 1;
    }
    
    println!("能量分布:");
    for (i, &count) in energy_buckets.iter().enumerate() {
        if count > 0 {
            let bar_len = (count * 30 / cells.len()).max(1);
            let bar: String = "█".repeat(bar_len);
            println!("  {:.1}-{:.1}: {:>4} ({:>5.1}%) {}", 
                i as f64 / 10.0, (i + 1) as f64 / 10.0,
                count, count as f64 / cells.len() as f64 * 100.0, bar);
        }
    }
    
    // 场强分布
    let field_strengths: Vec<f64> = cells.iter().map(|c| c.field_strength).collect();
    let avg_field = field_strengths.iter().sum::<f64>() / field_strengths.len() as f64;
    let max_field = field_strengths.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let min_field = field_strengths.iter().cloned().fold(f64::INFINITY, f64::min);
    
    println!("\n场强统计:");
    println!("  最小: {:.4}", min_field);
    println!("  最大: {:.4}", max_field);
    println!("  平均: {:.4}", avg_field);
    
    // 位置分布（检查是否聚集）
    if !cells.is_empty() && !cells[0].position.is_empty() {
        let dim = cells[0].position.len();
        println!("\n空间位置统计 ({}维):", dim);
        
        // 计算每个维度的分布
        for d in 0..dim.min(5) {  // 只显示前5个维度
            let positions: Vec<f64> = cells.iter()
                .filter_map(|c| c.position.get(d).copied())
                .collect();
            
            if !positions.is_empty() {
                let min_pos = positions.iter().cloned().fold(f64::INFINITY, f64::min);
                let max_pos = positions.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let avg_pos = positions.iter().sum::<f64>() / positions.len() as f64;
                let spread = max_pos - min_pos;
                
                println!("  维度 {}: 范围 [{:.2}, {:.2}], 跨度 {:.2}, 中心 {:.2}",
                    d, min_pos, max_pos, spread, avg_pos);
            }
        }
    }
    
    // 检查细胞"年龄"分布
    let ages: Vec<u64> = cells.iter().map(|c| c.age).collect();
    let max_age = *ages.iter().max().unwrap_or(&0);
    let min_age = *ages.iter().min().unwrap_or(&0);
    let avg_age = ages.iter().sum::<u64>() as f64 / ages.len() as f64;
    
    println!("\n细胞年龄统计:");
    println!("  最小: {}", min_age);
    println!("  最大: {}", max_age);
    println!("  平均: {:.1}", avg_age);
}
