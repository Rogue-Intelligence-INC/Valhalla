//! Hub 完整循环处理测试
//!
//! 验证 StemCell、Tile、Chamber 三大系统的协同演化

use std::path::Path;
use std::time::Instant;

use hub_f64::ValhallaHub;

fn load_corpus() -> Vec<Vec<u8>> {
    let mut corpus = Vec::new();
    
    // 英文 - LibriSpeech
    let libri_path = Path::new("/run/media/jouly/My Book/valhalla_training_data/audio/LibriSpeech/train-clean-100");
    if libri_path.exists() {
        let mut count = 0;
        fn visit_dir(path: &Path, corpus: &mut Vec<Vec<u8>>, count: &mut usize, max: usize) {
            if *count >= max { return; }
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    if *count >= max { return; }
                    let p = entry.path();
                    if p.is_dir() {
                        visit_dir(&p, corpus, count, max);
                    } else if p.extension().map(|x| x == "txt").unwrap_or(false) {
                        if let Ok(content) = std::fs::read_to_string(&p) {
                            corpus.push(content.into_bytes());
                            *count += 1;
                        }
                    }
                }
            }
        }
        visit_dir(libri_path, &mut corpus, &mut count, 100);
        println!("   [英文] {} 条", count);
    }
    
    // 中文 - 维基百科 XML
    let wiki_path = Path::new("/run/media/jouly/My Book/valhalla_training_data/text/zhwiki-latest.xml");
    if wiki_path.exists() {
        let mut count = 0;
        if let Ok(content) = std::fs::read(wiki_path) {
            for chunk in content.chunks(50000) {
                corpus.push(chunk.to_vec());
                count += 1;
                if count >= 200 { break; }
            }
        }
        println!("   [中文] {} 块", count);
    }
    
    // 图像 - COCO
    let coco_path = Path::new("/run/media/jouly/My Book/valhalla_training_data/images/train2017");
    if coco_path.exists() {
        let mut count = 0;
        for entry in std::fs::read_dir(coco_path).into_iter().flatten().flatten() {
            if count >= 100 { break; }
            if let Ok(data) = std::fs::read(&entry.path()) {
                corpus.push(data);
                count += 1;
            }
        }
        println!("   [图像] {} 张", count);
    }
    
    corpus
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════════════════════╗");
    println!("║            🌐 Hub 完整循环处理测试                                          ║");
    println!("║     StemCell ↔ Tile ↔ Chamber 协同演化                                    ║");
    println!("╚════════════════════════════════════════════════════════════════════════════╝");
    println!();
    
    // 加载语料
    println!("📚 加载语料...");
    let corpus = load_corpus();
    let total_bytes: usize = corpus.iter().map(|d| d.len()).sum();
    println!("   总计: {} 条, {:.2} MB", corpus.len(), total_bytes as f64 / 1024.0 / 1024.0);
    println!();
    
    // 创建 Hub
    let mut hub = ValhallaHub::new();
    
    // 打印初始状态
    let overview = hub.get_overview();
    println!("═══ 初始状态 ═══");
    println!("   StemCell: {} 细胞", overview.stemcell_cells);
    println!("   Tile:     {} 瓷砖", overview.tile_count);
    println!("   Chamber:  {} 容器", overview.chamber_count);
    println!("   Fate:     {} 存储", overview.fate_stored);
    println!();
    
    // 注入数据
    println!("💉 注入数据...");
    let inject_start = Instant::now();
    for data in &corpus {
        hub.inject_data(data);
    }
    println!("   注入耗时: {:?}", inject_start.elapsed());
    println!();
    
    // 循环处理
    let rounds = 10;
    println!("🔄 开始循环处理 ({} 轮)...", rounds);
    println!();
    
    let total_start = Instant::now();
    
    for round in 1..=rounds {
        let result = hub.cycle();
        
        println!("   第{:02}轮 ({:?}):", round, result.elapsed);
        println!("      ┌─ Manifest 层 ─┐");
        println!("      │ StemCell: {:4} 细胞, 能量 {:7.2}", result.stemcell_cells, result.stemcell_energy);
        println!("      │ Tile:     {:4} 瓷砖, 能量 {:7.2}", result.tile_count, result.tile_energy);
        println!("      │ Chamber:  {:4} 容器, 能量 {:7.2}", result.chamber_count, result.chamber_energy);
        println!("      ├─ Quad 元系统 ─┤");
        println!("      │ BlackHole:  {:7.4}", result.blackhole_signal);
        println!("      │ MeshBrain:  {:7.4}", result.meshbrain_signal);
        println!("      │ MultiNova:  {:7.4}", result.multinova_signal);
        println!("      │ BaseForce:  {:7.4}", result.baseforce_signal);
        println!("      └─ Fate: {} 存储, {} 自主创建", result.fate_stored, result.fate_spontaneous);
        println!();
    }
    
    // 最终状态
    let final_overview = hub.get_overview();
    println!("═══ 最终状态 ═══");
    println!("   总耗时:    {:?}", total_start.elapsed());
    println!("   循环次数:  {}", final_overview.cycle_count);
    println!();
    println!("   StemCell: {} 细胞", final_overview.stemcell_cells);
    println!("   Tile:     {} 瓷砖", final_overview.tile_count);
    println!("   Chamber:  {} 容器", final_overview.chamber_count);
    println!("   Fate:     {} 存储, {} 自主创建", final_overview.fate_stored, final_overview.fate_spontaneous);
    println!();
    
    // 健康度
    println!("═══ 系统健康度 ═══");
    for (name, health) in &final_overview.health {
        let bar = "█".repeat((health * 20.0) as usize);
        let empty = "░".repeat(20 - (health * 20.0) as usize);
        println!("   {:10}: [{}{}] {:.1}%", name, bar, empty, health * 100.0);
    }
    println!();
    
    // 方程状态
    println!("═══ Fate 方程状态 ═══");
    println!("   {:.4}x² + {:.4}x + {:.4}",
        hub.triple_fate.cluster_equation.coef_a,
        hub.triple_fate.cluster_equation.coef_b,
        hub.triple_fate.cluster_equation.coef_c);
    println!();
    
    // 各系统监控统计
    println!("═══ 各系统监控统计 ═══");
    let sc_summary = hub.stemcell.monitor.summary();
    let tile_summary = hub.tile.monitor.summary();
    let chamber_summary = hub.chamber.monitor.summary();
    
    println!("   StemCell:  {} 片段, {} 事件, 平均能量 {:.2}",
        sc_summary.total_fragments, sc_summary.total_events, sc_summary.avg_energy);
    println!("   Tile:      {} 片段, {} 事件, 平均能量 {:.2}",
        tile_summary.total_fragments, tile_summary.total_events, tile_summary.avg_energy);
    println!("   Chamber:   {} 片段, {} 事件, 平均能量 {:.2}",
        chamber_summary.total_fragments, chamber_summary.total_events, chamber_summary.avg_energy);
    println!();
    
    // Hub 级别高能片段捕捉
    let fragments = hub.get_captured_fragments();
    println!("═══ Hub 高能片段捕捉 ═══");
    println!("   总捕捉: {} 个", fragments.len());
    
    let summary = hub.get_monitor_summary();
    if !summary.fragment_stats.is_empty() {
        println!("   按来源:");
        for (source, count) in &summary.fragment_stats {
            println!("      {}: {} 个", source, count);
        }
    }
    
    // 显示最近几个高能片段
    if !fragments.is_empty() {
        println!("   最近捕捉:");
        for frag in fragments.iter().rev().take(5) {
            println!("      tick={}, {:?}, {:.2} 能量, {:?}",
                frag.tick, frag.source, frag.energy, frag.signal_type);
        }
    }
    println!();
    
    // 监控事件
    println!("═══ 互相监控事件 ═══");
    println!("   总事件: {} 个", summary.total_events);
    
    if !summary.event_counts.is_empty() {
        println!("   按类型:");
        for (event_type, count) in &summary.event_counts {
            println!("      {}: {} 次", event_type, count);
        }
    }
    
    // 显示最近几个监控事件
    let recent_events = hub.get_monitor_events(5);
    if !recent_events.is_empty() {
        println!("   最近事件:");
        for event in recent_events {
            println!("      tick={}, {:?}→{:?}: {}",
                event.tick, event.monitor, event.target, event.detail);
        }
    }
    
    println!();
    println!("╔════════════════════════════════════════════════════════════════════════════╗");
    println!("║                     ✅ Hub 循环测试完成                                    ║");
    println!("╚════════════════════════════════════════════════════════════════════════════╝");
}
