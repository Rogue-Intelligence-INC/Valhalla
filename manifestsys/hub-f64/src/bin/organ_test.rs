//! Organ Test - 器官形成测试
//! 
//! 使用 Fate 的 OrganDetector 进行代数化边际效应检测
//! 不使用硬编码阈值，自适应检测器官形成

use std::time::Instant;
use std::collections::HashMap;

use stem_cell::stemcell_complete::{StemCellComplete, StemCellConfig, CellSpecialization};
use fate_f64::OrganDetector;

fn main() {
    eprintln!("╔═══════════════════════════════════════════════════════════════════╗");
    eprintln!("║   器官形成测试 (Fate 代数化边际效应检测)                          ║");
    eprintln!("╚═══════════════════════════════════════════════════════════════════╝");
    eprintln!();

    // 测试配置：10% 突变率
    let config = StemCellConfig {
        initial_cell_count: 50,
        mutation_rate: 0.10,  // 10% 特化率
        container_capacity: 100,
    };
    
    eprintln!("🔧 配置:");
    eprintln!("   突变率: {}%", config.mutation_rate * 100.0);
    eprintln!("   初始细胞: {}", config.initial_cell_count);
    eprintln!("   器官检测: Fate OrganDetector (代数化边际效应)");
    eprintln!();

    let mut stemcell = StemCellComplete::with_config(config);
    let mut organ_detector = OrganDetector::new();
    
    let start = Instant::now();
    
    // 小规模测试：1000 数据
    eprintln!("📥 输入 1000 个数据点...");
    for i in 0..1000 {
        let data = (i as f64 * 0.01).sin() * 0.5 + (i as f64 * 0.03).cos() * 0.3;
        stemcell.receive_data(data);
    }
    
    let after_data = Instant::now();
    eprintln!("   ✓ 数据摄入完成 {:.3}s", (after_data - start).as_secs_f64());
    
    // 分析数据摄入后的特化分布
    let (spec_dist, counts) = analyze_specialization(&stemcell);
    eprintln!();
    eprintln!("📊 数据摄入后特化分布:");
    for (spec, count, pct) in &spec_dist {
        let bar_len = (*pct / 5.0) as usize;
        let bar = "█".repeat(bar_len.max(1));
        eprintln!("   {:12}: {:4} ({:5.1}%) {}", spec, count, pct, bar);
    }
    
    // 使用 Fate OrganDetector 检测器官
    let newly_formed = organ_detector.record(counts, 0);
    eprintln!();
    eprintln!("🎯 Fate OrganDetector (代数化边际效应):");
    eprintln!("   检测原理: 增长率边际递减 → 器官形成");
    eprintln!("   当前状态: 需要更多历史数据建立基线");
    
    // 演化 10 代，每代记录特化分布
    eprintln!();
    eprintln!("🧬 执行 10 代演化 (观察边际效应)...");
    for gen in 1..=10 {
        let result = stemcell.process_cells_cycle();
        let (_, counts) = analyze_specialization(&stemcell);
        let newly_formed = organ_detector.record(counts, gen as u64);
        
        eprintln!("   第{:2}代: 细胞={:4} 分裂={} 死亡={} 合并={} | 器官形成: {}",
            gen, result.cell_count, result.divisions, result.deaths, result.merges,
            organ_detector.formed_count());
        
        for organ_idx in &newly_formed {
            eprintln!("      🎉 新器官形成: {}", OrganDetector::organ_name(*organ_idx));
        }
    }
    
    // 演化后分析
    let (spec_dist_after, _) = analyze_specialization(&stemcell);
    eprintln!();
    eprintln!("📊 演化后特化分布:");
    for (spec, count, pct) in &spec_dist_after {
        let bar_len = (*pct / 5.0) as usize;
        let bar = "█".repeat(bar_len.max(1));
        eprintln!("   {:12}: {:4} ({:5.1}%) {}", spec, count, pct, bar);
    }
    
    // 最终器官检测 (Fate 决定)
    let organ_status = organ_detector.formed;
    eprintln!();
    eprintln!("🫀 最终器官状态 (Fate 代数化检测):");
    eprintln!("   感知器官: {} (边际递减检测)", if organ_status[0] { "✅ 形成" } else { "⏳ 未稳定" });
    eprintln!("   处理器官: {} (边际递减检测)", if organ_status[1] { "✅ 形成" } else { "⏳ 未稳定" });
    eprintln!("   记忆器官: {} (边际递减检测)", if organ_status[2] { "✅ 形成" } else { "⏳ 未稳定" });
    eprintln!("   协调器官: {} (边际递减检测)", if organ_status[3] { "✅ 形成" } else { "⏳ 未稳定" });
    eprintln!("   器官形成度: {}/4", organ_detector.formed_count());
    
    let total = start.elapsed();
    eprintln!();
    eprintln!("═══════════════════════════════════════════════════════════════════");
    eprintln!("✅ 测试完成! 总耗时: {:.3}s", total.as_secs_f64());
    eprintln!();
    
    // 验证结论
    let total_cells = stemcell.cells.len();
    let stem_count = spec_dist_after.iter().find(|(n, _, _)| n == "干细胞").map(|(_, c, _)| *c).unwrap_or(0);
    let specialized = total_cells - stem_count;
    let spec_rate = specialized as f64 / total_cells as f64 * 100.0;
    
    eprintln!("📋 总结:");
    eprintln!("   总细胞: {}", total_cells);
    eprintln!("   特化细胞: {} ({:.1}%)", specialized, spec_rate);
    eprintln!("   预期特化率: ~10% (mutation_rate)");
    eprintln!();
    eprintln!("💡 Fate OrganDetector 优势:");
    eprintln!("   - 无硬编码阈值 (如 10%)");
    eprintln!("   - 自适应检测增长率边际递减");
    eprintln!("   - 代数化判断器官成熟度");
    eprintln!("═══════════════════════════════════════════════════════════════════");
}

fn analyze_specialization(stemcell: &StemCellComplete) -> (Vec<(String, usize, f64)>, [u64; 4]) {
    let mut counts_map: HashMap<String, usize> = HashMap::new();
    let total = stemcell.cells.len();
    
    // 特化计数 [sensory, processing, memory, coordinator]
    let mut counts = [0u64; 4];
    
    for cell in &stemcell.cells {
        let (name, idx) = match cell.specialization {
            CellSpecialization::Stem => ("干细胞", None),
            CellSpecialization::Sensory => ("感知细胞", Some(0)),
            CellSpecialization::Processing => ("处理细胞", Some(1)),
            CellSpecialization::Memory => ("记忆细胞", Some(2)),
            CellSpecialization::Coordinator => ("协调细胞", Some(3)),
        };
        *counts_map.entry(name.to_string()).or_insert(0) += 1;
        if let Some(i) = idx {
            counts[i] += 1;
        }
    }
    
    let mut result: Vec<_> = counts_map.into_iter()
        .map(|(name, count)| {
            let pct = if total > 0 { count as f64 / total as f64 * 100.0 } else { 0.0 };
            (name, count, pct)
        })
        .collect();
    result.sort_by(|a, b| b.1.cmp(&a.1));
    (result, counts)
}
