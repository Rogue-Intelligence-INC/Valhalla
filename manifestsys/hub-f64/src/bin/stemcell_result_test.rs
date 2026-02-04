//! StemCell 训练结果测试
//! 
//! 测试训练后的细胞网络响应能力

use std::time::Instant;
use std::collections::HashMap;

use stem_cell::stemcell_complete::{StemCellComplete, StemCellConfig, CellSpecialization};
use fate_f64::OrganDetector;

fn main() {
    eprintln!("╔═══════════════════════════════════════════════════════════════════╗");
    eprintln!("║           StemCell 训练结果测试                                   ║");
    eprintln!("╚═══════════════════════════════════════════════════════════════════╝");
    eprintln!();

    // 使用训练时相同的配置
    let config = StemCellConfig {
        initial_cell_count: 100,
        mutation_rate: 0.10,
        container_capacity: 100,
    };
    
    let mut stemcell = StemCellComplete::with_config(config);
    let mut organ_detector = OrganDetector::new();
    
    // 模拟训练：快速注入一些数据
    eprintln!("📥 模拟训练数据注入 (10000 条)...");
    let start = Instant::now();
    for i in 0..10000 {
        let data = (i as f64 * 0.01).sin() * 0.5 + (i as f64 * 0.03).cos() * 0.3;
        stemcell.receive_data(data);
    }
    eprintln!("   ✓ 完成 ({:.3}s)", start.elapsed().as_secs_f64());
    
    // 演化 5 代
    eprintln!();
    eprintln!("🧬 执行 5 代演化...");
    for gen in 1..=5 {
        let result = stemcell.process_cells_cycle();
        eprintln!("   第{}代: 细胞={} 分裂={} 死亡={} 合并={} 能量={:.4}",
            gen, result.cell_count, result.divisions, result.deaths, result.merges, result.avg_energy);
    }
    
    // 分析特化分布
    let (spec_dist, counts) = analyze_specialization(&stemcell);
    let _ = organ_detector.record(counts, 1);
    
    eprintln!();
    eprintln!("📊 细胞特化分布:");
    for (spec, count, pct) in &spec_dist {
        let bar_len = (*pct / 5.0) as usize;
        let bar = "█".repeat(bar_len.max(1));
        eprintln!("   {:12}: {:4} ({:5.1}%) {}", spec, count, pct, bar);
    }
    
    // 测试响应
    eprintln!();
    eprintln!("🧪 测试细胞网络响应:");
    
    // 正弦波测试
    let before_cells = stemcell.cells.len();
    let before_energy: f64 = stemcell.cells.iter().map(|c| c.energy).sum();
    for i in 0..100 {
        stemcell.receive_data((i as f64 * 0.1).sin());
    }
    let after_cells = stemcell.cells.len();
    let after_energy: f64 = stemcell.cells.iter().map(|c| c.energy).sum();
    eprintln!("   正弦波: 细胞变化={:+} 能量变化={:+.4}", 
        after_cells as i64 - before_cells as i64, after_energy - before_energy);
    
    // 随机噪声测试
    let before_cells = stemcell.cells.len();
    let before_energy: f64 = stemcell.cells.iter().map(|c| c.energy).sum();
    for i in 0..100 {
        let noise = ((i * 12345 + 67890) % 1000) as f64 / 1000.0 - 0.5;
        stemcell.receive_data(noise);
    }
    let after_cells = stemcell.cells.len();
    let after_energy: f64 = stemcell.cells.iter().map(|c| c.energy).sum();
    eprintln!("   随机噪声: 细胞变化={:+} 能量变化={:+.4}", 
        after_cells as i64 - before_cells as i64, after_energy - before_energy);
    
    // 常量测试
    let before_cells = stemcell.cells.len();
    let before_energy: f64 = stemcell.cells.iter().map(|c| c.energy).sum();
    for _ in 0..100 {
        stemcell.receive_data(0.5);
    }
    let after_cells = stemcell.cells.len();
    let after_energy: f64 = stemcell.cells.iter().map(|c| c.energy).sum();
    eprintln!("   常量: 细胞变化={:+} 能量变化={:+.4}", 
        after_cells as i64 - before_cells as i64, after_energy - before_energy);
    
    // 器官状态
    let organ_status = organ_detector.formed;
    eprintln!();
    eprintln!("🫀 器官形成状态:");
    eprintln!("   感知器官: {}", if organ_status[0] { "✅ 形成" } else { "⏳ 未稳定" });
    eprintln!("   处理器官: {}", if organ_status[1] { "✅ 形成" } else { "⏳ 未稳定" });
    eprintln!("   记忆器官: {}", if organ_status[2] { "✅ 形成" } else { "⏳ 未稳定" });
    eprintln!("   协调器官: {}", if organ_status[3] { "✅ 形成" } else { "⏳ 未稳定" });
    
    // 总结
    let total_cells = stemcell.cells.len();
    let status = stemcell.get_status();
    
    eprintln!();
    eprintln!("═══════════════════════════════════════════════════════════════════");
    eprintln!("📋 测试总结:");
    eprintln!("   总细胞: {}", total_cells);
    eprintln!("   总分裂: {}", status.total_divisions);
    eprintln!("   总死亡: {}", status.total_deaths);
    eprintln!("   总合并: {}", status.total_merges);
    eprintln!("   容器关闭: {}", status.total_containers_closed);
    eprintln!("   Fate tick: {}", status.fate_tick);
    eprintln!("═══════════════════════════════════════════════════════════════════");
}

fn analyze_specialization(stemcell: &StemCellComplete) -> (Vec<(String, usize, f64)>, [u64; 4]) {
    let mut counts_map: HashMap<String, usize> = HashMap::new();
    let total = stemcell.cells.len();
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
