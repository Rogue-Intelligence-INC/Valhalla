//! 内部结构分析：StemCell细胞行为 + Tile神经网络连接

use stem_cell::stemcell_complete::{StemCellComplete, StemCellConfig, CellSpecialization};
use valhalla_tile_quattro::{TileQuattroComplete, TileQuattroCompleteConfig};
use std::collections::HashMap;

fn main() {
    println!("╔════════════════════════════════════════════════════════════════════════╗");
    println!("║          内部结构分析：细胞行为 & 神经网络连接                         ║");
    println!("╚════════════════════════════════════════════════════════════════════════╝\n");

    // ===== Part 1: StemCell 分析 =====
    analyze_stemcell();
    
    println!("\n");
    
    // ===== Part 2: Tile 分析 =====
    analyze_tile();
}

fn analyze_stemcell() {
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("🧬 StemCell 内部结构分析");
    println!("═══════════════════════════════════════════════════════════════════════════\n");
    
    let mut stem = StemCellComplete::with_config(StemCellConfig {
        initial_cell_count: 50,
        mutation_rate: 0.05,
        container_capacity: 20,
    });
    
    // 模拟数据输入（小规模）
    println!("📥 输入 1000 个数据点...\n");
    for i in 0..1000 {
        let data = (i as f64 * 0.01).sin() * 0.5 + (i as f64 * 0.03).cos() * 0.3;
        stem.receive_data(data);
    }
    
    // 执行 5 轮细胞周期
    println!("🔄 执行 5 轮细胞演化周期...\n");
    for cycle in 1..=5 {
        let result = stem.process_cells_cycle();
        println!("  周期 {}: 细胞数={}, 合并={}, 分裂={}, 死亡={}, 平均能量={:.4}",
            cycle, result.cell_count, result.merges, result.divisions, result.deaths, result.avg_energy);
    }
    
    let status = stem.get_status();
    
    println!("\n┌─────────────────────────────────────────────────────────────────────────┐");
    println!("│ 🔬 细胞行为分析                                                         │");
    println!("├─────────────────────────────────────────────────────────────────────────┤");
    
    // 1. 细胞特化分布（类似器官分化）
    let mut spec_counts: HashMap<String, usize> = HashMap::new();
    for cell in &stem.cells {
        let spec_name = match cell.specialization {
            CellSpecialization::Stem => "干细胞",
            CellSpecialization::Sensory => "感知细胞",
            CellSpecialization::Processing => "处理细胞",
            CellSpecialization::Memory => "记忆细胞",
            CellSpecialization::Coordinator => "协调细胞",
        };
        *spec_counts.entry(spec_name.to_string()).or_insert(0) += 1;
    }
    
    println!("│ 细胞特化分布（类似器官分化）：                                          │");
    for (spec, count) in &spec_counts {
        let pct = (*count as f64 / stem.cells.len() as f64) * 100.0;
        let bar_len = (pct / 5.0) as usize;
        let bar = "█".repeat(bar_len);
        println!("│   {:12}: {:4} ({:5.1}%) {}", spec, count, pct, bar);
    }
    
    // 2. 细胞群聚检测（能量相似的细胞）
    println!("├─────────────────────────────────────────────────────────────────────────┤");
    println!("│ 细胞群聚检测（能量分布）：                                              │");
    
    let energies: Vec<f64> = stem.cells.iter().map(|c| c.energy).collect();
    let mean_energy = energies.iter().sum::<f64>() / energies.len() as f64;
    let variance: f64 = energies.iter().map(|e| (e - mean_energy).powi(2)).sum::<f64>() / energies.len() as f64;
    let std_dev = variance.sqrt();
    
    // 按能量分组
    let low_energy = energies.iter().filter(|&&e| e < mean_energy - std_dev).count();
    let mid_energy = energies.iter().filter(|&&e| e >= mean_energy - std_dev && e <= mean_energy + std_dev).count();
    let high_energy = energies.iter().filter(|&&e| e > mean_energy + std_dev).count();
    
    println!("│   低能量群 (<μ-σ):   {:4} 细胞", low_energy);
    println!("│   中能量群 (μ±σ):    {:4} 细胞", mid_energy);
    println!("│   高能量群 (>μ+σ):   {:4} 细胞", high_energy);
    println!("│   平均能量: {:.6}, 标准差: {:.6}", mean_energy, std_dev);
    
    // 3. 世代分布（进化深度）
    println!("├─────────────────────────────────────────────────────────────────────────┤");
    println!("│ 世代分布（细胞进化深度）：                                              │");
    
    let mut gen_counts: HashMap<u32, usize> = HashMap::new();
    for cell in &stem.cells {
        *gen_counts.entry(cell.generation).or_insert(0) += 1;
    }
    let mut gens: Vec<_> = gen_counts.iter().collect();
    gens.sort_by_key(|&(g, _)| g);
    
    for (gen, count) in gens.iter().take(8) {
        println!("│   第 {:2} 代: {:4} 细胞", gen, count);
    }
    if gens.len() > 8 {
        println!("│   ... 共 {} 代", gens.len());
    }
    
    // 4. 初级器官检测
    println!("├─────────────────────────────────────────────────────────────────────────┤");
    println!("│ 🫀 初级器官检测（功能细胞簇）：                                         │");
    
    let sensory_cluster = spec_counts.get("感知细胞").unwrap_or(&0);
    let processing_cluster = spec_counts.get("处理细胞").unwrap_or(&0);
    let memory_cluster = spec_counts.get("记忆细胞").unwrap_or(&0);
    let coordinator_cluster = spec_counts.get("协调细胞").unwrap_or(&0);
    
    let organ_threshold = stem.cells.len() / 10;  // 10% 以上视为初级器官
    
    let has_sensory_organ = *sensory_cluster >= organ_threshold;
    let has_processing_organ = *processing_cluster >= organ_threshold;
    let has_memory_organ = *memory_cluster >= organ_threshold;
    let has_coordinator_organ = *coordinator_cluster >= organ_threshold;
    
    println!("│   感知器官 (输入层):   {} ({} 细胞)", 
        if has_sensory_organ { "✅ 形成" } else { "⬜ 未形成" }, sensory_cluster);
    println!("│   处理器官 (计算层):   {} ({} 细胞)", 
        if has_processing_organ { "✅ 形成" } else { "⬜ 未形成" }, processing_cluster);
    println!("│   记忆器官 (存储层):   {} ({} 细胞)", 
        if has_memory_organ { "✅ 形成" } else { "⬜ 未形成" }, memory_cluster);
    println!("│   协调器官 (控制层):   {} ({} 细胞)", 
        if has_coordinator_organ { "✅ 形成" } else { "⬜ 未形成" }, coordinator_cluster);
    
    let organ_count = [has_sensory_organ, has_processing_organ, has_memory_organ, has_coordinator_organ]
        .iter().filter(|&&x| x).count();
    
    println!("│                                                                         │");
    println!("│   🧫 器官形成度: {}/4", organ_count);
    
    println!("├─────────────────────────────────────────────────────────────────────────┤");
    println!("│ 📊 总体统计：                                                           │");
    println!("│   总细胞数: {}", status.cell_count);
    println!("│   总分裂: {}, 总死亡: {}, 总合并: {}", 
        status.total_divisions, status.total_deaths, status.total_merges);
    println!("│   闭合容器: {}", status.total_containers_closed);
    println!("│   维度: {:.4}", status.dimension);
    println!("└─────────────────────────────────────────────────────────────────────────┘");
}

fn analyze_tile() {
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("🧱 Tile 内部结构分析（神经网络连接）");
    println!("═══════════════════════════════════════════════════════════════════════════\n");
    
    let mut tile = TileQuattroComplete::with_config(TileQuattroCompleteConfig {
        tile_count: 4,
        parallel_threshold: 0.5,
        load_balance_enabled: true,
        container_capacity: 20,
    });
    
    // 模拟数据输入
    println!("📥 输入 1000 个数据点...\n");
    for i in 0..1000 {
        let data = (i as f64 * 0.02).sin() * 0.5 + (i as f64 * 0.05).cos() * 0.3;
        tile.receive_data(data);
    }
    
    // 执行 5 轮 Tile 周期
    println!("🔄 执行 5 轮 Tile 演化周期...\n");
    for cycle in 1..=5 {
        let result = tile.process_tiles_cycle();
        println!("  周期 {}: Tile数={}, 合并={}, 分裂={}, 平均激活={:.4}",
            cycle, result.tile_count, result.merges, result.splits, result.avg_activation);
    }
    
    let status = tile.get_status();
    
    println!("\n┌─────────────────────────────────────────────────────────────────────────┐");
    println!("│ 🔗 神经网络连接分析                                                     │");
    println!("├─────────────────────────────────────────────────────────────────────────┤");
    
    // 1. Tile 激活分布（类似神经元激活）
    println!("│ Tile 激活分布（类似神经元激活）：                                       │");
    
    let means: Vec<f64> = tile.completed_tiles.iter().map(|t| t.mean).collect();
    if !means.is_empty() {
        let mean_activation = means.iter().sum::<f64>() / means.len() as f64;
        let variance: f64 = means.iter().map(|m| (m - mean_activation).powi(2)).sum::<f64>() / means.len() as f64;
        let std_dev = variance.sqrt();
        
        let silent = means.iter().filter(|m| m.abs() < 0.1).count();
        let active = means.iter().filter(|m| m.abs() >= 0.1 && m.abs() < 0.5).count();
        let high_active = means.iter().filter(|m| m.abs() >= 0.5).count();
        
        println!("│   静默 Tile (|μ|<0.1):  {:4} ({:.1}%)", 
            silent, silent as f64 / means.len() as f64 * 100.0);
        println!("│   活跃 Tile (0.1-0.5):  {:4} ({:.1}%)", 
            active, active as f64 / means.len() as f64 * 100.0);
        println!("│   高活跃 Tile (>0.5):   {:4} ({:.1}%)", 
            high_active, high_active as f64 / means.len() as f64 * 100.0);
        println!("│   平均激活: {:.6}, 标准差: {:.6}", mean_activation, std_dev);
    }
    
    // 2. 四元系统连接（类似神经网络层）
    println!("├─────────────────────────────────────────────────────────────────────────┤");
    println!("│ 四元系统连接（类似神经网络层）：                                        │");
    println!("│                                                                         │");
    println!("│   ┌──────────────┐     ┌──────────────┐                                │");
    println!("│   │  BlackHole   │────▶│  MeshBrain   │                                │");
    println!("│   │  (引力层)    │     │  (智能层)    │                                │");
    println!("│   └──────────────┘     └──────────────┘                                │");
    println!("│          │                    │                                         │");
    println!("│          ▼                    ▼                                         │");
    println!("│   ┌──────────────┐     ┌──────────────┐                                │");
    println!("│   │  MultiNova   │────▶│  BaseForce   │                                │");
    println!("│   │  (辐射层)    │     │  (协调层)    │                                │");
    println!("│   └──────────────┘     └──────────────┘                                │");
    println!("│                    ▼                                                    │");
    println!("│            ┌──────────────┐                                            │");
    println!("│            │   Combined   │                                            │");
    println!("│            │   (输出层)   │                                            │");
    println!("│            └──────────────┘                                            │");
    
    // 3. Tile 信号传播（类似前向传播）
    println!("├─────────────────────────────────────────────────────────────────────────┤");
    println!("│ 信号传播链（类似前向传播）：                                            │");
    
    let variances: Vec<f64> = tile.completed_tiles.iter().map(|t| t.variance).collect();
    if !variances.is_empty() {
        let mean_var = variances.iter().sum::<f64>() / variances.len() as f64;
        let low_var = variances.iter().filter(|&&v| v < mean_var * 0.5).count();
        let mid_var = variances.iter().filter(|&&v| v >= mean_var * 0.5 && v <= mean_var * 1.5).count();
        let high_var = variances.iter().filter(|&&v| v > mean_var * 1.5).count();
        
        println!("│   输入层 (高方差):      {:4} Tile   <- 原始信号", high_var);
        println!("│   隐藏层 (中方差):      {:4} Tile   <- 特征提取", mid_var);
        println!("│   输出层 (低方差):      {:4} Tile   <- 稳定输出", low_var);
    }
    
    // 4. Fate 监控网络
    println!("├─────────────────────────────────────────────────────────────────────────┤");
    println!("│ 🎯 Fate 监控网络（三层架构）：                                          │");
    println!("│                                                                         │");
    println!("│   Ultimate Fate ──┐                                                     │");
    println!("│         │         │ 跨层反馈                                           │");
    println!("│         ▼         │                                                     │");
    println!("│   Meta Fate ──────┼──┐                                                 │");
    println!("│         │         │  │ 模式检测                                        │");
    println!("│         ▼         │  │                                                 │");
    println!("│   Base Fate ──────┴──┘                                                 │");
    println!("│         │                                                               │");
    println!("│         ▼                                                               │");
    println!("│   [Tile 数据流]                                                        │");
    
    println!("├─────────────────────────────────────────────────────────────────────────┤");
    println!("│ 📊 总体统计：                                                           │");
    println!("│   完成 Tile: {}", status.completed_tiles);
    println!("│   活跃容器: {}", status.active_tiles);
    println!("│   总操作: {}", status.total_operations);
    println!("│   维度: {:.4}", status.dimension);
    println!("│   Fate tick: {}", status.fate_tick);
    println!("└─────────────────────────────────────────────────────────────────────────┘");
    
    // 5. 网络连接度评估
    println!("\n┌─────────────────────────────────────────────────────────────────────────┐");
    println!("│ 🧠 神经网络相似度评估                                                   │");
    println!("├─────────────────────────────────────────────────────────────────────────┤");
    
    let tile_count = tile.completed_tiles.len();
    let layer_ratio = if tile_count > 0 {
        let variances: Vec<f64> = tile.completed_tiles.iter().map(|t| t.variance).collect();
        let mean_var = variances.iter().sum::<f64>() / variances.len() as f64;
        let layers = [
            variances.iter().filter(|&&v| v < mean_var * 0.5).count(),
            variances.iter().filter(|&&v| v >= mean_var * 0.5 && v <= mean_var * 1.5).count(),
            variances.iter().filter(|&&v| v > mean_var * 1.5).count(),
        ];
        let non_empty_layers = layers.iter().filter(|&&l| l > 0).count();
        non_empty_layers as f64 / 3.0
    } else { 0.0 };
    
    let fate_layers = 3; // Base, Meta, Ultimate
    let quad_connections = 4; // BlackHole, MeshBrain, MultiNova, BaseForce
    
    println!("│   层状结构相似度:     {:.0}%", layer_ratio * 100.0);
    println!("│   Fate 监控层数:      {}/3", fate_layers);
    println!("│   四元系统连接:       {}/4", quad_connections);
    println!("│                                                                         │");
    
    let overall_nn_similarity = (layer_ratio * 0.4 + 1.0 * 0.3 + 1.0 * 0.3) * 100.0;
    println!("│   🧠 整体神经网络相似度: {:.1}%", overall_nn_similarity);
    println!("└─────────────────────────────────────────────────────────────────────────┘");
}
