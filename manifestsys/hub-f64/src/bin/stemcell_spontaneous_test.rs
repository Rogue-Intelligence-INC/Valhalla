//! StemCell 自发行为测试
//! 
//! 输入空符号触发 tick，观察系统是否有自发行为
//! - 没有新数据输入时，系统内部状态是否变化？
//! - 连接是否重组？能量是否流动？维度是否波动？

use stem_cell::StemCellComplete;

fn main() {
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("   StemCell 自发行为测试 - 空 tick 观察");
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!();
    
    // ═══ 阶段 1: 快速训练，建立基础结构 ═══
    println!("【阶段 1】训练系统，建立基础结构...\n");
    
    let mut stemcell = StemCellComplete::new();
    
    // 输入一些训练数据
    let training_data = vec![
        // 概念组 1: 数字
        "0", "1", "2", "3", "4", "5", "6", "7", "8", "9",
        "zero", "one", "two", "three", "four", "five",
        // 概念组 2: 颜色
        "red", "green", "blue", "yellow", "orange", "purple",
        // 概念组 3: 动物
        "cat", "dog", "bird", "fish", "lion", "tiger",
        // 概念组 4: 动词
        "run", "walk", "jump", "swim", "fly", "sleep",
        // 混合
        "the cat runs", "dog walks", "bird flies",
        "red cat", "blue fish", "green bird",
    ];
    
    for text in &training_data {
        for ch in text.chars() {
            stemcell.receive_data(ch as u8 as f64);
        }
        stemcell.receive_data(0.0);  // 分隔符
    }
    
    // 演化几轮稳定
    for _ in 0..10 {
        stemcell.evolve(0.0);
    }
    
    println!("训练完成！");
    println!("  细胞数: {}", stemcell.cells.len());
    println!("  维度:   {:.2}", stemcell.space.current_dimension);
    
    // ═══ 阶段 2: 记录初始状态 ═══
    println!("\n【阶段 2】记录初始状态...\n");
    
    let initial_state = capture_state(&stemcell);
    print_state("初始状态", &initial_state);
    
    // ═══ 阶段 3: 空 tick 测试 ═══
    println!("\n【阶段 3】空 tick 测试（无数据输入，只 evolve）\n");
    println!("观察系统在没有外部输入时的自发行为...\n");
    println!("{:>6} | {:>6} | {:>8} | {:>8} | {:>8} | {:>8} | {:>8} | {:>8}",
        "tick", "cells", "conn", "dim", "avg_E", "E_var", "Δconn", "ΔE");
    println!("{}", "-".repeat(90));
    
    let mut prev_state = initial_state.clone();
    let mut history: Vec<SystemState> = vec![initial_state];
    
    // 100 轮空 tick
    for tick in 1..=100 {
        // 只 evolve，不输入数据
        stemcell.evolve(0.0);
        
        let state = capture_state(&stemcell);
        
        // 计算变化量
        let delta_conn = state.total_connections as i64 - prev_state.total_connections as i64;
        let delta_energy = state.avg_energy - prev_state.avg_energy;
        
        // 每 10 轮输出
        if tick % 10 == 0 || tick <= 5 {
            println!("{:>6} | {:>6} | {:>8} | {:>8.2} | {:>8.4} | {:>8.6} | {:>+8} | {:>+8.6}",
                tick,
                state.cell_count,
                state.total_connections,
                state.dimension,
                state.avg_energy,
                state.energy_variance,
                delta_conn,
                delta_energy,
            );
        }
        
        history.push(state.clone());
        prev_state = state;
    }
    
    // ═══ 阶段 4: 分析自发行为 ═══
    println!("\n【阶段 4】自发行为分析\n");
    
    analyze_spontaneous_behavior(&history);
    
    // ═══ 阶段 5: 深度空跑测试（更长周期）═══
    println!("\n【阶段 5】深度空跑测试（500 tick）\n");
    
    let mut activity_count = 0;
    let mut dimension_changes = 0;
    let mut connection_changes = 0;
    let mut energy_flows = 0;
    
    let start_state = capture_state(&stemcell);
    let mut prev = start_state.clone();
    
    for _ in 0..500 {
        stemcell.evolve(0.0);
        
        let curr = capture_state(&stemcell);
        
        // 检测变化
        if curr.cell_count != prev.cell_count {
            activity_count += 1;
        }
        if (curr.dimension - prev.dimension).abs() > 0.01 {
            dimension_changes += 1;
        }
        if curr.total_connections != prev.total_connections {
            connection_changes += 1;
        }
        if (curr.avg_energy - prev.avg_energy).abs() > 0.0001 {
            energy_flows += 1;
        }
        
        prev = curr;
    }
    
    let end_state = capture_state(&stemcell);
    
    println!("深度空跑结果 (500 tick 无输入):");
    println!("  细胞数变化:     {} → {}", start_state.cell_count, end_state.cell_count);
    println!("  维度变化:       {:.2} → {:.2}", start_state.dimension, end_state.dimension);
    println!("  连接数变化:     {} → {}", start_state.total_connections, end_state.total_connections);
    println!("  平均能量变化:   {:.6} → {:.6}", start_state.avg_energy, end_state.avg_energy);
    println!();
    println!("自发活动统计:");
    println!("  细胞数变化次数:   {}/500 ({:.1}%)", activity_count, activity_count as f64 / 5.0);
    println!("  维度波动次数:     {}/500 ({:.1}%)", dimension_changes, dimension_changes as f64 / 5.0);
    println!("  连接重组次数:     {}/500 ({:.1}%)", connection_changes, connection_changes as f64 / 5.0);
    println!("  能量流动次数:     {}/500 ({:.1}%)", energy_flows, energy_flows as f64 / 5.0);
    
    // ═══ 结论 ═══
    println!("\n═══════════════════════════════════════════════════════════════════════════");
    println!("   结论");
    println!("═══════════════════════════════════════════════════════════════════════════\n");
    
    let total_activity = activity_count + dimension_changes + connection_changes + energy_flows;
    let activity_ratio = total_activity as f64 / 2000.0;  // 4 种活动 × 500 tick
    
    if activity_ratio > 0.5 {
        println!("【高活力】系统表现出显著的自发行为！");
        println!("  即使没有外部输入，系统内部仍在持续变化");
        println!("  这表明系统可能具有某种形式的\"内在活动\"");
    } else if activity_ratio > 0.1 {
        println!("【中等活力】系统有一定的自发行为");
        println!("  部分内部结构在持续调整");
        println!("  类似\"休眠但有呼吸\"的状态");
    } else if activity_ratio > 0.01 {
        println!("【低活力】系统有微弱的自发行为");
        println!("  大部分时间静止，偶尔有微小波动");
    } else {
        println!("【静止】系统在无输入时几乎完全静止");
        println!("  需要外部刺激才能产生变化");
    }
    
    println!("\n活力指数: {:.2}%", activity_ratio * 100.0);
}

/// 系统状态快照
#[derive(Clone)]
struct SystemState {
    cell_count: usize,
    total_connections: usize,
    avg_connections_per_cell: f64,
    dimension: f64,
    avg_energy: f64,
    energy_variance: f64,
    max_energy: f64,
    min_energy: f64,
    avg_field_strength: f64,
}

/// 捕获系统状态
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
            max_energy: 0.0,
            min_energy: 0.0,
            avg_field_strength: 0.0,
        };
    }
    
    let total_connections: usize = cells.iter().map(|c| c.connections.len()).sum();
    let avg_connections = total_connections as f64 / cells.len() as f64;
    
    let energies: Vec<f64> = cells.iter().map(|c| c.energy).collect();
    let avg_energy = energies.iter().sum::<f64>() / energies.len() as f64;
    let energy_variance = energies.iter()
        .map(|e| (e - avg_energy).powi(2))
        .sum::<f64>() / energies.len() as f64;
    
    let max_energy = energies.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let min_energy = energies.iter().cloned().fold(f64::INFINITY, f64::min);
    
    let avg_field = cells.iter().map(|c| c.field_strength).sum::<f64>() / cells.len() as f64;
    
    SystemState {
        cell_count: cells.len(),
        total_connections,
        avg_connections_per_cell: avg_connections,
        dimension: stemcell.space.current_dimension,
        avg_energy,
        energy_variance,
        max_energy,
        min_energy,
        avg_field_strength: avg_field,
    }
}

/// 打印状态
fn print_state(label: &str, state: &SystemState) {
    println!("{}:", label);
    println!("  细胞数:         {}", state.cell_count);
    println!("  总连接数:       {}", state.total_connections);
    println!("  平均连接/细胞:  {:.2}", state.avg_connections_per_cell);
    println!("  维度:           {:.2}", state.dimension);
    println!("  平均能量:       {:.6}", state.avg_energy);
    println!("  能量方差:       {:.6}", state.energy_variance);
    println!("  能量范围:       {:.4} ~ {:.4}", state.min_energy, state.max_energy);
    println!("  平均场强:       {:.6}", state.avg_field_strength);
}

/// 分析自发行为
fn analyze_spontaneous_behavior(history: &[SystemState]) {
    if history.len() < 2 {
        println!("数据不足以分析");
        return;
    }
    
    // 计算各指标的变化
    let mut cell_changes = 0;
    let mut connection_changes = 0;
    let mut dimension_fluctuations = 0;
    let mut energy_movements = 0;
    
    let mut max_conn_delta = 0i64;
    let mut max_energy_delta = 0.0f64;
    let mut max_dim_delta = 0.0f64;
    
    for i in 1..history.len() {
        let prev = &history[i - 1];
        let curr = &history[i];
        
        // 细胞数变化
        if curr.cell_count != prev.cell_count {
            cell_changes += 1;
        }
        
        // 连接变化
        let conn_delta = (curr.total_connections as i64 - prev.total_connections as i64).abs();
        if conn_delta > 0 {
            connection_changes += 1;
            max_conn_delta = max_conn_delta.max(conn_delta);
        }
        
        // 维度波动
        let dim_delta = (curr.dimension - prev.dimension).abs();
        if dim_delta > 0.001 {
            dimension_fluctuations += 1;
            max_dim_delta = max_dim_delta.max(dim_delta);
        }
        
        // 能量流动
        let energy_delta = (curr.avg_energy - prev.avg_energy).abs();
        if energy_delta > 0.00001 {
            energy_movements += 1;
            max_energy_delta = max_energy_delta.max(energy_delta);
        }
    }
    
    let total_ticks = history.len() - 1;
    
    println!("100 tick 内的自发变化:");
    println!();
    println!("  细胞数变化:      {}/{} tick ({:.1}%)", 
        cell_changes, total_ticks, cell_changes as f64 / total_ticks as f64 * 100.0);
    println!("  连接重组:        {}/{} tick ({:.1}%)", 
        connection_changes, total_ticks, connection_changes as f64 / total_ticks as f64 * 100.0);
    println!("  维度波动:        {}/{} tick ({:.1}%)", 
        dimension_fluctuations, total_ticks, dimension_fluctuations as f64 / total_ticks as f64 * 100.0);
    println!("  能量流动:        {}/{} tick ({:.1}%)", 
        energy_movements, total_ticks, energy_movements as f64 / total_ticks as f64 * 100.0);
    println!();
    println!("最大单 tick 变化:");
    println!("  最大连接变化:    {}", max_conn_delta);
    println!("  最大维度变化:    {:.6}", max_dim_delta);
    println!("  最大能量变化:    {:.6}", max_energy_delta);
    
    // 判断是否有"呼吸"模式
    println!();
    println!("模式分析:");
    
    // 检查周期性
    let energies: Vec<f64> = history.iter().map(|s| s.avg_energy).collect();
    let connections: Vec<usize> = history.iter().map(|s| s.total_connections).collect();
    
    // 简单的周期检测：看是否有波动模式
    let energy_trend = detect_trend(&energies);
    let conn_trend = detect_trend_usize(&connections);
    
    println!("  能量趋势:        {}", energy_trend);
    println!("  连接趋势:        {}", conn_trend);
}

fn detect_trend(data: &[f64]) -> &'static str {
    if data.len() < 10 {
        return "数据不足";
    }
    
    let mut increasing = 0;
    let mut decreasing = 0;
    let mut stable = 0;
    
    for i in 1..data.len() {
        let diff = data[i] - data[i - 1];
        if diff.abs() < 0.00001 {
            stable += 1;
        } else if diff > 0.0 {
            increasing += 1;
        } else {
            decreasing += 1;
        }
    }
    
    let total = data.len() - 1;
    
    if stable as f64 / total as f64 > 0.8 {
        "静止"
    } else if (increasing as f64 / total as f64) > 0.6 {
        "持续上升 ↑"
    } else if (decreasing as f64 / total as f64) > 0.6 {
        "持续下降 ↓"
    } else if increasing > 0 && decreasing > 0 {
        "波动 〜"
    } else {
        "不确定"
    }
}

fn detect_trend_usize(data: &[usize]) -> &'static str {
    let float_data: Vec<f64> = data.iter().map(|&x| x as f64).collect();
    detect_trend(&float_data)
}
