//! StemCell 自然周期分析
//!
//! 1. 加载训练后的 Fate 切片
//! 2. 运行自然周期（无新输入）
//! 3. 分析细胞分化、器官化、连接拓扑
//! 4. 为互动设计提供数据基础

use std::path::Path;
use std::collections::HashMap;

use stem_cell::StemCellComplete;
use fate_f64::TripleFateCombined;

fn main() {
    println!("╔════════════════════════════════════════════════════════════════════════════╗");
    println!("║          🧬 StemCell 自然周期分析                                           ║");
    println!("║          加载训练状态 → 自然演化 → 分析智能形态                              ║");
    println!("╚════════════════════════════════════════════════════════════════════════════╝");
    println!();

    // ========== 1. 加载训练后的 Fate 切片 ==========
    println!("📥 加载训练后的 Fate 切片...");
    let slice_path = Path::new("/tmp/unified_stemcell.fate");
    
    let fate = match TripleFateCombined::load_slice(slice_path) {
        Ok(f) => {
            println!("   ✓ 加载成功");
            println!("   - Fate tick: {}", f.global_tick);
            println!("   - 存储条目: {}", f.stored_count());
            println!("   - 外部导入: {}", f.list_external().len());
            println!("   - 内部创建: {}", f.list_internal().len());
            f
        }
        Err(e) => {
            println!("   ✗ 加载失败: {}", e);
            println!("   使用新建的 StemCell");
            TripleFateCombined::new()
        }
    };
    println!();

    // 创建 StemCell 并注入 Fate 状态
    let mut stemcell = StemCellComplete::new();
    stemcell.triple_fate = fate;
    
    // 显示初始状态
    println!("🧬 初始状态（训练后）:");
    print_stemcell_analysis(&stemcell);
    println!();

    // ========== 2. 自然周期演化（无新数据输入）==========
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("                    自然周期演化（空跑）");
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!();
    
    let total_cycles = 1000;
    let checkpoint_interval = 200;
    
    let mut history: Vec<CycleSnapshot> = Vec::new();
    
    for cycle in 1..=total_cycles {
        // 空输入触发演化
        stemcell.receive_data(0.0);
        stemcell.evolve(0.5);
        
        if cycle % checkpoint_interval == 0 || cycle == 1 {
            let snapshot = take_snapshot(&stemcell, cycle);
            println!("   周期 {}: {} 细胞, {} 连接, 能量方差 {:.4}",
                cycle, snapshot.cell_count, snapshot.total_connections, snapshot.energy_variance);
            history.push(snapshot);
        }
    }
    println!();

    // ========== 3. 深度分析 ==========
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("                    深度分析");
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!();

    // 3.1 细胞分化分析
    println!("📊 细胞分化分析:");
    let differentiation = analyze_differentiation(&stemcell);
    println!("   - 高能量细胞 (>1.5): {} ({:.1}%)", 
        differentiation.high_energy_count,
        differentiation.high_energy_count as f64 / stemcell.cells.len() as f64 * 100.0);
    println!("   - 中能量细胞 (0.5-1.5): {} ({:.1}%)",
        differentiation.medium_energy_count,
        differentiation.medium_energy_count as f64 / stemcell.cells.len() as f64 * 100.0);
    println!("   - 低能量细胞 (<0.5): {} ({:.1}%)",
        differentiation.low_energy_count,
        differentiation.low_energy_count as f64 / stemcell.cells.len() as f64 * 100.0);
    println!();

    // 3.2 连接拓扑分析
    println!("🔗 连接拓扑分析:");
    let topology = analyze_topology(&stemcell);
    println!("   - 总连接数: {}", topology.total_connections);
    println!("   - 平均连接数: {:.2}", topology.avg_connections);
    println!("   - 最大连接数: {} (细胞 {})", topology.max_connections, topology.hub_cell_id);
    println!("   - 孤立细胞: {} ({:.1}%)",
        topology.isolated_count,
        topology.isolated_count as f64 / stemcell.cells.len() as f64 * 100.0);
    println!("   - 核心细胞 (>10连接): {} ({:.1}%)",
        topology.core_count,
        topology.core_count as f64 / stemcell.cells.len() as f64 * 100.0);
    println!();

    // 3.3 器官化分析（聚类）
    println!("🫀 器官化分析（功能聚类）:");
    let organs = analyze_organ_formation(&stemcell);
    println!("   - 检测到 {} 个潜在器官（高密度连接区域）", organs.len());
    for (i, organ) in organs.iter().enumerate().take(5) {
        println!("   - 器官 {}: {} 细胞, 内部连接密度 {:.2}",
            i + 1, organ.cell_count, organ.internal_density);
    }
    println!();

    // 3.4 Fate 存储内容分析
    println!("📦 Fate 存储内容分析:");
    analyze_fate_content(&stemcell.triple_fate);
    println!();

    // 3.5 演化稳定性分析
    println!("📈 演化稳定性分析:");
    if history.len() >= 2 {
        let first = &history[0];
        let last = history.last().unwrap();
        let cell_change = (last.cell_count as i64 - first.cell_count as i64).abs();
        let conn_change = (last.total_connections as i64 - first.total_connections as i64).abs();
        
        println!("   - 细胞数变化: {} → {} (Δ{})",
            first.cell_count, last.cell_count, cell_change);
        println!("   - 连接数变化: {} → {} (Δ{})",
            first.total_connections, last.total_connections, conn_change);
        println!("   - 能量方差变化: {:.4} → {:.4}",
            first.energy_variance, last.energy_variance);
        
        let stability = if cell_change < 100 && conn_change < 500 {
            "高度稳定"
        } else if cell_change < 500 && conn_change < 2000 {
            "相对稳定"
        } else {
            "活跃演化中"
        };
        println!("   - 系统状态: {}", stability);
    }
    println!();

    // ========== 4. 智能形态分析 ==========
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("                    智能形态分析");
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!();
    
    analyze_intelligence_form(&stemcell);

    // ========== 5. 保存分析后状态 ==========
    let analyzed_path = Path::new("/tmp/stemcell_analyzed.fate");
    match stemcell.triple_fate.save_slice(analyzed_path) {
        Ok(()) => println!("\n💾 分析后状态已保存: {}", analyzed_path.display()),
        Err(e) => println!("\n❌ 保存失败: {}", e),
    }

    println!();
    println!("╔════════════════════════════════════════════════════════════════════════════╗");
    println!("║  ✅ 分析完成                                                                ║");
    println!("╚════════════════════════════════════════════════════════════════════════════╝");
}

// ============================================================================
// 数据结构
// ============================================================================

struct CycleSnapshot {
    cycle: usize,
    cell_count: usize,
    total_connections: usize,
    energy_variance: f64,
}

struct DifferentiationAnalysis {
    high_energy_count: usize,
    medium_energy_count: usize,
    low_energy_count: usize,
}

struct TopologyAnalysis {
    total_connections: usize,
    avg_connections: f64,
    max_connections: usize,
    hub_cell_id: String,
    isolated_count: usize,
    core_count: usize,
}

struct OrganInfo {
    cell_count: usize,
    internal_density: f64,
}

// ============================================================================
// 分析函数
// ============================================================================

fn print_stemcell_analysis(stemcell: &StemCellComplete) {
    let status = stemcell.get_status();
    println!("   ├─ 细胞数: {}", status.cell_count);
    println!("   ├─ 世代: {}", status.generation);
    println!("   ├─ 总分裂: {}", status.total_divisions);
    println!("   ├─ 总死亡: {}", status.total_deaths);
    println!("   ├─ 总合并: {}", status.total_merges);
    println!("   ├─ Fate tick: {}", status.fate_tick);
    println!("   ├─ Fate 存储: {}", stemcell.triple_fate.stored_count());
    println!("   └─ 方程: {}", stemcell.triple_fate.get_equation());
}

fn take_snapshot(stemcell: &StemCellComplete, cycle: usize) -> CycleSnapshot {
    let total_connections: usize = stemcell.cells.iter()
        .map(|c| c.connections.len())
        .sum();
    
    let energies: Vec<f64> = stemcell.cells.iter().map(|c| c.energy).collect();
    let avg_energy = energies.iter().sum::<f64>() / energies.len().max(1) as f64;
    let energy_variance = energies.iter()
        .map(|e| (e - avg_energy).powi(2))
        .sum::<f64>() / energies.len().max(1) as f64;
    
    CycleSnapshot {
        cycle,
        cell_count: stemcell.cells.len(),
        total_connections,
        energy_variance,
    }
}

fn analyze_differentiation(stemcell: &StemCellComplete) -> DifferentiationAnalysis {
    let mut high = 0;
    let mut medium = 0;
    let mut low = 0;
    
    for cell in &stemcell.cells {
        if cell.energy > 1.5 {
            high += 1;
        } else if cell.energy >= 0.5 {
            medium += 1;
        } else {
            low += 1;
        }
    }
    
    DifferentiationAnalysis {
        high_energy_count: high,
        medium_energy_count: medium,
        low_energy_count: low,
    }
}

fn analyze_topology(stemcell: &StemCellComplete) -> TopologyAnalysis {
    let total_connections: usize = stemcell.cells.iter()
        .map(|c| c.connections.len())
        .sum();
    
    let avg_connections = total_connections as f64 / stemcell.cells.len().max(1) as f64;
    
    let mut max_connections = 0;
    let mut hub_cell_id = String::new();
    let mut isolated_count = 0;
    let mut core_count = 0;
    
    for cell in &stemcell.cells {
        let conn_count = cell.connections.len();
        if conn_count > max_connections {
            max_connections = conn_count;
            hub_cell_id = cell.cell_id.clone();
        }
        if conn_count == 0 {
            isolated_count += 1;
        }
        if conn_count > 10 {
            core_count += 1;
        }
    }
    
    TopologyAnalysis {
        total_connections,
        avg_connections,
        max_connections,
        hub_cell_id,
        isolated_count,
        core_count,
    }
}

fn analyze_organ_formation(stemcell: &StemCellComplete) -> Vec<OrganInfo> {
    // 简化的器官检测：找到高连接度的细胞群
    // 真正的器官检测需要更复杂的聚类算法
    
    let mut organs = Vec::new();
    
    // 按连接数分组
    let mut connection_groups: HashMap<usize, Vec<&str>> = HashMap::new();
    for cell in &stemcell.cells {
        let bucket = cell.connections.len() / 5; // 每5个连接一组
        connection_groups.entry(bucket).or_default().push(&cell.cell_id);
    }
    
    // 高连接度组作为"器官"
    for (bucket, cells) in connection_groups.iter() {
        if *bucket >= 2 && cells.len() >= 10 {
            organs.push(OrganInfo {
                cell_count: cells.len(),
                internal_density: *bucket as f64 * 0.2,
            });
        }
    }
    
    organs.sort_by(|a, b| b.cell_count.cmp(&a.cell_count));
    organs
}

fn analyze_fate_content(fate: &TripleFateCombined) {
    let stored = fate.list_stored();
    let total_count = stored.len();
    
    // 统计数据类型
    let mut rust_code = 0;
    let mut text = 0;
    let mut binary = 0;
    let mut total_bytes = 0usize;
    
    for id in stored.iter().take(1000) {
        if let Some(entry) = fate.load_data(id) {
            total_bytes += entry.raw.len();
            
            if let Ok(text_content) = String::from_utf8(entry.raw.clone()) {
                if text_content.contains("fn ") || text_content.contains("use ") 
                    || text_content.contains("pub ") || text_content.contains("struct ") {
                    rust_code += 1;
                } else {
                    text += 1;
                }
            } else {
                binary += 1;
            }
        }
    }
    
    println!("   - 总存储条目: {}", total_count);
    println!("   - 采样分析 (前1000条):");
    println!("     - Rust 代码: {} ({:.1}%)", rust_code, rust_code as f64 / 1000.0 * 100.0);
    println!("     - 文本: {} ({:.1}%)", text, text as f64 / 1000.0 * 100.0);
    println!("     - 二进制: {} ({:.1}%)", binary, binary as f64 / 1000.0 * 100.0);
    println!("   - 采样数据量: {:.2} MB", total_bytes as f64 / 1024.0 / 1024.0);
}

fn analyze_intelligence_form(stemcell: &StemCellComplete) {
    println!("🧠 智能形态特征:");
    println!();
    
    // 1. 自组织能力
    let total_connections: usize = stemcell.cells.iter()
        .map(|c| c.connections.len())
        .sum();
    let connection_density = total_connections as f64 / stemcell.cells.len().max(1) as f64;
    
    println!("   1. 自组织能力:");
    println!("      - 连接密度: {:.2} 连接/细胞", connection_density);
    if connection_density > 5.0 {
        println!("      - 评估: 高度自组织，形成了密集网络");
    } else if connection_density > 2.0 {
        println!("      - 评估: 中等自组织，有明显的连接模式");
    } else {
        println!("      - 评估: 低自组织，连接较稀疏");
    }
    println!();
    
    // 2. 分化程度
    let energies: Vec<f64> = stemcell.cells.iter().map(|c| c.energy).collect();
    let energy_variance = {
        let avg = energies.iter().sum::<f64>() / energies.len().max(1) as f64;
        energies.iter().map(|e| (e - avg).powi(2)).sum::<f64>() / energies.len().max(1) as f64
    };
    
    println!("   2. 细胞分化程度:");
    println!("      - 能量方差: {:.4}", energy_variance);
    if energy_variance > 0.5 {
        println!("      - 评估: 高分化，细胞呈现明显的功能差异");
    } else if energy_variance > 0.1 {
        println!("      - 评估: 中等分化，存在功能分工");
    } else {
        println!("      - 评估: 低分化，细胞较为同质");
    }
    println!();
    
    // 3. 信息存储能力
    let stored_count = stemcell.triple_fate.stored_count();
    println!("   3. 信息存储能力:");
    println!("      - Fate 存储条目: {}", stored_count);
    println!("      - 每细胞平均存储: {:.2} 条", stored_count as f64 / stemcell.cells.len().max(1) as f64);
    println!();
    
    // 4. 动态方程状态
    println!("   4. Fate 动态方程:");
    println!("      - {}", stemcell.triple_fate.get_equation());
    println!("      - 收敛状态: {}", if stemcell.triple_fate.is_passively_converged() { "已收敛" } else { "演化中" });
    println!();
    
    // 5. 核心-外围结构
    let isolated: usize = stemcell.cells.iter().filter(|c| c.connections.is_empty()).count();
    let core: usize = stemcell.cells.iter().filter(|c| c.connections.len() > 10).count();
    
    println!("   5. 核心-外围结构:");
    println!("      - 核心细胞 (>10连接): {} ({:.1}%)", core, core as f64 / stemcell.cells.len().max(1) as f64 * 100.0);
    println!("      - 孤立细胞 (0连接): {} ({:.1}%)", isolated, isolated as f64 / stemcell.cells.len().max(1) as f64 * 100.0);
    println!();
    
    // 6. 智能形态总结
    println!("   📋 智能形态总结:");
    println!("      ┌─────────────────────────────────────────────────────────────────┐");
    println!("      │ 当前系统呈现一种【分布式关联记忆】的智能形态：                    │");
    println!("      │                                                                 │");
    println!("      │ • 数据以 Fate 存储形式保留原始语义                               │");
    println!("      │ • 细胞网络形成关联索引，通过连接强度表达相似性                    │");
    println!("      │ • 核心细胞作为[枢纽]连接多个概念区域                             │");
    println!("      │ • 孤立细胞代表独特/罕见的数据模式                                │");
    println!("      │ • Fate 方程动态调整，反映系统对数据的[理解]                      │");
    println!("      │                                                                 │");
    println!("      │ 互动模式建议：基于【关联激活】而非【问答匹配】                   │");
    println!("      └─────────────────────────────────────────────────────────────────┘");
}
