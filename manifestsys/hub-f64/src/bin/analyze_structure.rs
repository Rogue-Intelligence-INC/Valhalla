//! 分析 StemCell 和 Tile 的细胞结构、器官化行为

use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::Path;

use stem_cell::StemCellComplete;
use valhalla_tile_quattro::TileQuattroComplete;
use chamber::ChamberComplete;

fn main() {
    eprintln!("╔═══════════════════════════════════════════════════════════════════╗");
    eprintln!("║     细胞结构 / 器官化 / 行为 分析                                  ║");
    eprintln!("╚═══════════════════════════════════════════════════════════════════╝\n");

    // 加载语料
    eprintln!("📂 加载语料样本...");
    let corpus = load_sample_corpus(10000);
    eprintln!("   样本: {} 条\n", corpus.len());

    // 初始化系统
    let mut stemcell = StemCellComplete::new();
    let mut tile = TileQuattroComplete::new();
    let mut chamber = ChamberComplete::new();

    eprintln!("🔬 数据流入并观察结构形成...\n");

    // 数据流入
    for (i, content) in corpus.iter().enumerate() {
        let v = content.len() as f64 + (i as f64).sin() * 10.0;
        
        stemcell.receive_data(v);
        tile.receive_data(v);
        chamber.receive_data(v);

        // 每 2000 条检查一次
        if i > 0 && i % 2000 == 0 {
            analyze_snapshot(i, &stemcell, &tile, &chamber);
        }
    }

    // 最终分析
    eprintln!("\n╔═══════════════════════════════════════════════════════════════════╗");
    eprintln!("║                     最终结构分析                                   ║");
    eprintln!("╚═══════════════════════════════════════════════════════════════════╝\n");

    analyze_final(&stemcell, &tile, &chamber);
}

fn analyze_snapshot(step: usize, sc: &StemCellComplete, tile: &TileQuattroComplete, ch: &ChamberComplete) {
    let sc_status = sc.get_status();
    let tile_status = tile.get_status();
    let ch_status = ch.get_status();

    eprintln!("─── 步骤 {} ───", step);
    eprintln!("  StemCell: 细胞={} 容器闭合={} 维度={:.3}", 
              sc_status.cell_count, sc_status.total_containers_closed, sc_status.dimension);
    eprintln!("  Tile:     完成={} 活跃={} 维度={:.3}", 
              tile_status.completed_tiles, tile_status.active_tiles, tile_status.dimension);
    eprintln!("  Chamber:  完成={} 扩缩={} 维度={:.3}\n", 
              ch_status.completed_containers, ch_status.scale_events, ch_status.dimension);
}

fn analyze_final(sc: &StemCellComplete, tile: &TileQuattroComplete, ch: &ChamberComplete) {
    let sc_status = sc.get_status();
    let tile_status = tile.get_status();
    let ch_status = ch.get_status();

    // === StemCell 分析 ===
    eprintln!("🧬 StemCellComplete 细胞结构分析");
    eprintln!("├─ 细胞数量: {}", sc_status.cell_count);
    eprintln!("├─ 容器闭合(局部收敛): {}", sc_status.total_containers_closed);
    eprintln!("├─ 分裂次数: {}", sc_status.total_divisions);
    eprintln!("├─ 死亡次数: {}", sc_status.total_deaths);
    eprintln!("├─ 维度: {:.4}", sc_status.dimension);
    eprintln!("├─ Fate tick: {}", sc_status.fate_tick);
    eprintln!("├─ Base Generation: {}", sc_status.base_generation);
    eprintln!("├─ Meta Generation: {}", sc_status.meta_generation);
    eprintln!("└─ Fate 收敛: {}\n", sc.has_fate_converged());

    // 细胞行为分析
    let cell_behavior = if sc_status.total_containers_closed > 0 {
        let ratio = sc_status.cell_count as f64 / sc_status.total_containers_closed as f64;
        if ratio > 1.5 {
            "细胞分裂活跃 (增殖行为)"
        } else if ratio < 0.5 {
            "细胞死亡较多 (凋亡行为)"
        } else {
            "细胞稳态 (平衡行为)"
        }
    } else {
        "数据不足"
    };
    eprintln!("   📊 细胞行为: {}", cell_behavior);

    // 组织化分析
    let tissue_level = if sc_status.total_containers_closed > 50 {
        "高度组织化 - 类似组织形成"
    } else if sc_status.total_containers_closed > 20 {
        "中度组织化 - 细胞簇形成"
    } else if sc_status.total_containers_closed > 5 {
        "低度组织化 - 散在细胞"
    } else {
        "未组织化"
    };
    eprintln!("   📊 组织化程度: {}\n", tissue_level);

    // === Tile 分析 ===
    eprintln!("🧠 TileQuattroComplete 神经结构分析");
    eprintln!("├─ 完成的 Tile: {}", tile_status.completed_tiles);
    eprintln!("├─ 活跃 Tile: {}", tile_status.active_tiles);
    eprintln!("├─ 总操作: {}", tile_status.total_operations);
    eprintln!("├─ 维度: {:.4}", tile_status.dimension);
    eprintln!("├─ Fate tick: {}", tile_status.fate_tick);
    eprintln!("└─ Fate 收敛: {}\n", tile.has_fate_converged());

    let quad_dist = tile.get_quad_distribution();
    eprintln!("   四元分布:");
    eprintln!("     BlackHole: {:.2}%", quad_dist[0] * 100.0);
    eprintln!("     MeshBrain: {:.2}%", quad_dist[1] * 100.0);
    eprintln!("     MultiNova: {:.2}%", quad_dist[2] * 100.0);
    eprintln!("     BaseForce: {:.2}%\n", quad_dist[3] * 100.0);

    // 神经网络化分析
    let neural_level = if tile_status.completed_tiles > 50 {
        "高度神经网络化 - 类似神经元集群"
    } else if tile_status.completed_tiles > 20 {
        "中度神经网络化 - 神经元连接形成"
    } else if tile_status.completed_tiles > 5 {
        "低度神经网络化 - 散在节点"
    } else {
        "未神经网络化"
    };
    eprintln!("   📊 神经网络化程度: {}\n", neural_level);

    // === Chamber 分析 ===
    eprintln!("📦 ChamberComplete 容器编排分析");
    eprintln!("├─ 完成容器: {}", ch_status.completed_containers);
    eprintln!("├─ 活跃容器: {}", ch_status.active_containers);
    eprintln!("├─ 总操作: {}", ch_status.total_operations);
    eprintln!("├─ 扩缩事件: {}", ch_status.scale_events);
    eprintln!("├─ 平均负载: {:.2}%", ch_status.average_load * 100.0);
    eprintln!("├─ 维度: {:.4}", ch_status.dimension);
    eprintln!("└─ Fate 收敛: {}\n", ch.has_fate_converged());

    // 器官化分析
    let organ_level = if ch_status.scale_events > 10 && ch_status.completed_containers > 30 {
        "高度器官化 - 动态协调的功能单元"
    } else if ch_status.scale_events > 3 || ch_status.completed_containers > 15 {
        "中度器官化 - 协调的容器群"
    } else if ch_status.completed_containers > 5 {
        "低度器官化 - 简单功能聚合"
    } else {
        "未器官化"
    };
    eprintln!("   📊 器官化程度: {}\n", organ_level);

    // === 综合结论 ===
    eprintln!("╔═══════════════════════════════════════════════════════════════════╗");
    eprintln!("║                     综合结论                                       ║");
    eprintln!("╚═══════════════════════════════════════════════════════════════════╝");
    eprintln!();
    eprintln!("  🧬 StemCell: {}", cell_behavior);
    eprintln!("     └─ {}", tissue_level);
    eprintln!();
    eprintln!("  🧠 Tile: {}", neural_level);
    eprintln!("     └─ 四元偏好: BH={:.1}% MB={:.1}% MN={:.1}% BF={:.1}%",
              quad_dist[0]*100.0, quad_dist[1]*100.0, quad_dist[2]*100.0, quad_dist[3]*100.0);
    eprintln!();
    eprintln!("  📦 Chamber: {}", organ_level);
    eprintln!("     └─ 扩缩响应: {} 次", ch_status.scale_events);
    eprintln!();

    // 整体评估
    let total_structures = sc_status.total_containers_closed as usize + tile_status.completed_tiles + ch_status.completed_containers;
    if total_structures > 100 {
        eprintln!("  ✅ 整体评估: 出现明显的结构化行为");
        eprintln!("     - 局部收敛形成数据簇 (细胞)");
        eprintln!("     - 多层 Fate 监控协调");
        eprintln!("     - 自动维度扩展响应复杂数据");
    } else if total_structures > 30 {
        eprintln!("  ⚠️ 整体评估: 初步结构化，需要更多数据");
    } else {
        eprintln!("  ℹ️ 整体评估: 结构化程度较低，建议更长训练");
    }
}

fn load_sample_corpus(limit: usize) -> Vec<String> {
    let mut items = Vec::new();
    let corpus_root = Path::new("/run/media/jouly/My Book/valhalla_corpus");
    let valhalla_root = Path::new("/home/jouly/AI/Valhalla");

    // 语言
    let sent_path = corpus_root.join("languages/english/sentences");
    if sent_path.exists() {
        for entry in fs::read_dir(&sent_path).into_iter().flatten().flatten().take(3) {
            let p = entry.path();
            if p.extension().map_or(false, |e| e == "txt" || e == "tsv") {
                if let Ok(f) = File::open(&p) {
                    for line in BufReader::new(f).lines().flatten().take(limit / 2) {
                        let parts: Vec<&str> = line.split('\t').collect();
                        let s = if parts.len() >= 3 { parts[2].trim() } else { line.trim() };
                        if s.len() > 5 && s.len() < 500 {
                            items.push(s.to_string());
                            if items.len() >= limit { return items; }
                        }
                    }
                }
            }
        }
    }

    // 对话
    let p = valhalla_root.join("valhalla_all_dialogues.txt");
    if p.exists() {
        if let Ok(c) = fs::read_to_string(&p) {
            for line in c.lines().take(limit / 2) {
                if line.trim().len() > 3 { 
                    items.push(line.trim().to_string());
                    if items.len() >= limit { return items; }
                }
            }
        }
    }

    items
}
