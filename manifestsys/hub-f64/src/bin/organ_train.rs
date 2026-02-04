//! Organ Train - 器官形成训练
//! 
//! 目标：让 StemCell 形成功能器官
//! 模式：数据摄入 + 细胞演化周期
//! 
//! 与 complete_train 的区别：
//! - 每周期后执行 process_cells_cycle() 进行细胞演化
//! - 细胞可以分裂、死亡、合并、特化
//! - 观察器官形成过程

use std::time::Instant;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::collections::HashMap;

use stem_cell::stemcell_complete::{StemCellComplete, StemCellConfig, CellSpecialization};

fn main() {
    eprintln!("╔═══════════════════════════════════════════════════════════════════╗");
    eprintln!("║     StemCell 器官形成训练 (数据摄入 + 细胞演化)                   ║");
    eprintln!("╚═══════════════════════════════════════════════════════════════════╝");
    eprintln!();

    // 加载语料
    eprintln!("📂 加载语料...");
    let corpus = load_corpus();
    let corpus_size = corpus.len();
    eprintln!("   总量: {} 条", corpus_size);
    eprintln!();

    if corpus.is_empty() {
        eprintln!("❌ 未找到语料！");
        return;
    }

    // 初始化 - 使用较高突变率加速器官形成
    let config = StemCellConfig {
        initial_cell_count: 100,
        mutation_rate: 0.10,  // 10% 突变率（比默认 1% 高）
        container_capacity: 100,
    };
    
    eprintln!("🔧 初始化 StemCellComplete");
    eprintln!("   突变率: {}%", config.mutation_rate * 100.0);
    eprintln!("   初始细胞: {}", config.initial_cell_count);
    let mut stemcell = StemCellComplete::with_config(config);
    eprintln!();

    let total_start = Instant::now();
    let data_cycles = 10;
    let evolution_cycles_per_data = 5;  // 每轮数据后演化 5 代

    for cycle in 1..=data_cycles {
        let cycle_start = Instant::now();
        eprintln!("┌───────────────────────────────────────────────────────────────┐");
        eprintln!("│ 周期 {}/{}: 数据摄入                                          │", cycle, data_cycles);
        eprintln!("└───────────────────────────────────────────────────────────────┘");

        // 数据摄入
        let report_interval = corpus_size / 5;
        for (i, content) in corpus.iter().enumerate() {
            let v = content.len() as f64 + (i as f64).sin() * 10.0;
            stemcell.receive_data(v);
            
            if i > 0 && i % report_interval == 0 {
                let pct = (i as f64 / corpus_size as f64 * 100.0) as u32;
                let elapsed = cycle_start.elapsed().as_secs_f64();
                let rate = i as f64 / elapsed;
                let sc = stemcell.get_status();
                eprintln!("  {}% | {:.0}/s | 细胞:{} 闭合:{}", pct, rate, sc.cell_count, sc.total_containers_closed);
            }
        }

        let data_time = cycle_start.elapsed().as_secs_f64();
        eprintln!("  ✓ 数据摄入完成 {:.2}s", data_time);
        eprintln!();

        // 细胞演化周期
        eprintln!("  🧬 细胞演化 ({} 代)...", evolution_cycles_per_data);
        let evo_start = Instant::now();
        
        for evo in 1..=evolution_cycles_per_data {
            let result = stemcell.process_cells_cycle();
            eprintln!("    第{}代: 细胞={} 分裂={} 死亡={} 合并={} 能量={:.4}",
                evo, result.cell_count, result.divisions, result.deaths, result.merges, result.avg_energy);
        }
        
        let evo_time = evo_start.elapsed().as_secs_f64();
        eprintln!("  ✓ 演化完成 {:.2}s", evo_time);
        eprintln!();

        // 分析特化分布
        let spec_dist = analyze_specialization(&stemcell);
        eprintln!("  📊 特化分布:");
        for (spec, count, pct) in &spec_dist {
            let bar_len = (*pct / 5.0) as usize;
            let bar = "█".repeat(bar_len);
            eprintln!("     {:12}: {:6} ({:5.1}%) {}", spec, count, pct, bar);
        }

        // 检测器官形成
        let organs = detect_organs(&spec_dist, stemcell.get_status().cell_count);
        eprintln!("  🫀 器官检测: {}/4 形成", organs.iter().filter(|&&x| x).count());
        if organs[0] { eprintln!("     ✅ 感知器官"); }
        if organs[1] { eprintln!("     ✅ 处理器官"); }
        if organs[2] { eprintln!("     ✅ 记忆器官"); }
        if organs[3] { eprintln!("     ✅ 协调器官"); }

        let sc = stemcell.get_status();
        eprintln!();
        eprintln!("  周期{} 总结: 细胞={} 维度={:.2} Fate收敛={}",
            cycle, sc.cell_count, sc.dimension, stemcell.has_fate_converged());
        eprintln!("             总分裂={} 总死亡={} 总合并={}",
            sc.total_divisions, sc.total_deaths, sc.total_merges);
        eprintln!();

        // 如果器官全部形成，提前结束
        if organs.iter().all(|&x| x) {
            eprintln!("🎉 所有器官已形成！提前结束训练。");
            break;
        }
    }

    let total = total_start.elapsed();
    eprintln!("╔═══════════════════════════════════════════════════════════════════╗");
    eprintln!("║ 完成! 总耗时: {:.1}分钟 ({:.0}秒)                                 ║", 
        total.as_secs_f64()/60.0, total.as_secs_f64());
    eprintln!("╚═══════════════════════════════════════════════════════════════════╝");
    eprintln!();
    
    // 最终报告
    let sc = stemcell.get_status();
    let spec_dist = analyze_specialization(&stemcell);
    let organs = detect_organs(&spec_dist, sc.cell_count);
    
    eprintln!("═══════════════════════════════════════════════════════════════════");
    eprintln!("📋 最终报告");
    eprintln!("═══════════════════════════════════════════════════════════════════");
    eprintln!("细胞总数: {}", sc.cell_count);
    eprintln!("总分裂: {} | 总死亡: {} | 总合并: {}", sc.total_divisions, sc.total_deaths, sc.total_merges);
    eprintln!("维度: {:.2}", sc.dimension);
    eprintln!();
    eprintln!("特化分布:");
    for (spec, count, pct) in &spec_dist {
        eprintln!("  {:12}: {:6} ({:5.1}%)", spec, count, pct);
    }
    eprintln!();
    eprintln!("器官形成: {}/4", organs.iter().filter(|&&x| x).count());
    eprintln!("  感知器官: {}", if organs[0] { "✅" } else { "❌" });
    eprintln!("  处理器官: {}", if organs[1] { "✅" } else { "❌" });
    eprintln!("  记忆器官: {}", if organs[2] { "✅" } else { "❌" });
    eprintln!("  协调器官: {}", if organs[3] { "✅" } else { "❌" });
    eprintln!("═══════════════════════════════════════════════════════════════════");
}

fn analyze_specialization(stemcell: &StemCellComplete) -> Vec<(String, usize, f64)> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    let total = stemcell.cells.len();
    
    for cell in &stemcell.cells {
        let name = match cell.specialization {
            CellSpecialization::Stem => "干细胞",
            CellSpecialization::Sensory => "感知细胞",
            CellSpecialization::Processing => "处理细胞",
            CellSpecialization::Memory => "记忆细胞",
            CellSpecialization::Coordinator => "协调细胞",
        };
        *counts.entry(name.to_string()).or_insert(0) += 1;
    }
    
    let mut result: Vec<_> = counts.into_iter()
        .map(|(name, count)| {
            let pct = if total > 0 { count as f64 / total as f64 * 100.0 } else { 0.0 };
            (name, count, pct)
        })
        .collect();
    result.sort_by(|a, b| b.1.cmp(&a.1));
    result
}

fn detect_organs(spec_dist: &[(String, usize, f64)], total_cells: usize) -> [bool; 4] {
    let threshold = 10.0;  // 10% 视为器官形成
    let mut organs = [false; 4];
    
    for (name, _, pct) in spec_dist {
        if *pct >= threshold {
            match name.as_str() {
                "感知细胞" => organs[0] = true,
                "处理细胞" => organs[1] = true,
                "记忆细胞" => organs[2] = true,
                "协调细胞" => organs[3] = true,
                _ => {}
            }
        }
    }
    organs
}

fn load_corpus() -> Vec<String> {
    let mut items = Vec::new();
    let corpus_root = Path::new("/run/media/jouly/My Book/valhalla_corpus");
    let valhalla_root = Path::new("/home/jouly/AI/Valhalla");

    // 语言
    let lang_path = corpus_root.join("languages");
    for lang in ["english", "chinese", "japanese", "korean", "french", "german", "spanish"] {
        let sent_path = lang_path.join(lang).join("sentences");
        if sent_path.exists() {
            for entry in fs::read_dir(&sent_path).into_iter().flatten().flatten() {
                if let Ok(file) = File::open(entry.path()) {
                    for line in BufReader::new(file).lines().take(100000).flatten() {
                        if !line.is_empty() {
                            items.push(line);
                        }
                    }
                }
            }
        }
    }

    // 对话
    let dialogue_files = [
        valhalla_root.join("valhalla_all_dialogues.txt"),
        corpus_root.join("dialogues/all_dialogues.txt"),
    ];
    for path in dialogue_files {
        if path.exists() {
            if let Ok(file) = File::open(&path) {
                for line in BufReader::new(file).lines().flatten() {
                    if !line.is_empty() {
                        items.push(line);
                    }
                }
            }
        }
    }

    items
}
