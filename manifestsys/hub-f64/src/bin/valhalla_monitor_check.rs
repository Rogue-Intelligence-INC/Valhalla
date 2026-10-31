//! 功能核查 + 训练时监控演示
//!
//! 不测智能。只确认四元、Stem、Tile 的功能在实际运行：
//! 信息转换、互相影响、局部循环、全局跟随。
//! 监控读数来自 hub_f64::monitor::BodyMonitor。

use hub_f64::monitor::BodyMonitor;
use hub_f64::{
    BaseForceCoordinator, GravityField, GravityFieldConfig, MeshBrainConfig, MeshBrainSystem,
    MultinovaConfig, MultinovaProcessor, TripleFateCombined,
};
use stem_cell::StemCellComplete;
use valhalla_tile_quattro::TileQuattroComplete;

fn main() {
    println!("== 四元：信息转换 ==");
    let mut blackhole = GravityField::new(GravityFieldConfig::default());
    let g_low = blackhole.keep(&[1, 1, 1, 1]);
    let g_high = blackhole.keep(&[127, 127, 127, 127]);
    println!("黑洞 gravity low {g_low:.6} high {g_high:.6} stored {}", blackhole.stored_count());
    assert!(g_low.is_finite() && g_high.is_finite() && (g_low - g_high).abs() > 1e-9);
    assert_eq!(blackhole.stored_count(), 2);
    assert!(blackhole.stored_modify(0, &[9, 9]));
    assert_eq!(blackhole.stored_remove(1), Some(vec![127, 127, 127, 127]));

    let mut meshbrain = MeshBrainSystem::new(MeshBrainConfig::default());
    let m1 = meshbrain.keep(b"hello");
    let m2 = meshbrain.keep(b"hello world, longer text");
    println!("膜脑 signal {m1:.6} -> {m2:.6} buffer {} stored {}", meshbrain.signal_buffer.len(), meshbrain.stored_count());
    assert_eq!(meshbrain.signal_buffer.len(), 2);

    let mut multinova = MultinovaProcessor::new(MultinovaConfig::default());
    let n1 = multinova.keep(&[10, 10, 10]);
    let n2 = multinova.keep(&[120, 120, 120]);
    println!("新星 radiation {n1:.4} -> {n2:.4} events {}", multinova.total_events);
    assert_eq!(multinova.total_events, 2);

    println!("\n== 四元：互相影响（BaseForce 协调三个元的输出）==");
    let mut baseforce = BaseForceCoordinator::new();
    let a = baseforce.coordinate(1.0, 2.0, 3.0).total_activity;
    let b = baseforce.coordinate(10.0, 2.0, 3.0).total_activity;
    println!("底力 activity (1,2,3)={a:.4} (10,2,3)={b:.4}");
    assert!((a - 2.0).abs() < 1e-9 && (b - 5.0).abs() < 1e-9);
    baseforce.keep(b"window", g_low, m1, n1);
    assert_eq!(baseforce.stored_count(), 1);

    println!("\n== Stem：存储、局部循环、全局跟随 ==");
    let mut stem = StemCellComplete::new();
    let id = stem.fate_store(b"fed text");
    assert_eq!(stem.fate_count(), 1);
    assert!(stem.fate_modify(&id, b"changed".to_vec()));
    assert_eq!(stem.fate_load(&id).map(|e| e.raw.as_slice()), Some(b"changed".as_slice()));
    assert!(stem.fate_remove(&id).is_some());
    assert_eq!(stem.fate_count(), 0);

    for i in 0..200 {
        stem.receive_data((i % 50) as f64 * 0.1);
    }
    let stem_cycle = stem.process_cells_cycle();
    println!(
        "Stem 局部循环 generation {} cells {} merges {} divisions {} deaths {}",
        stem_cycle.generation, stem_cycle.cell_count, stem_cycle.merges, stem_cycle.divisions, stem_cycle.deaths
    );
    stem.settle_field();
    println!("Stem 物理场 device {}", stem.field_device);

    let mut global = TripleFateCombined::new();
    let _ = global.tick(Some(vec![1.0, 2.0, 3.0]));
    stem.follow_global_fate(&global);
    println!("Stem 跟随全局 Fate tick {} -> {}", global.global_tick, stem.triple_fate.global_tick);
    assert_eq!(stem.triple_fate.global_tick, global.global_tick);

    println!("\n== Tile：出生、局部循环、传播 ==");
    let mut tile = TileQuattroComplete::new();
    let mut born = 0usize;
    for i in 0..600 {
        tile.ingest((i % 50) as f64 * 0.1 + 0.01);
        if tile.promote_from_fate() {
            born += 1;
        }
    }
    println!("Tile 出生 {born} 完成 {}", tile.completed_tiles.len());
    assert!(!tile.completed_tiles.is_empty());
    let tile_cycle = tile.process_tiles_cycle();
    println!(
        "Tile 局部循环 tiles {} merges {} splits {} new_connections {}",
        tile_cycle.tile_count, tile_cycle.merges, tile_cycle.splits, tile_cycle.new_connections
    );
    let prop1 = tile.propagate(1.0);
    let prop2 = tile.propagate(1.0);
    println!(
        "Tile 传播 propagated {} avg_activation {:.4} new_connections {} + {}",
        prop2.propagated_tiles, prop2.avg_activation, prop1.new_connections, prop2.new_connections
    );
    tile.follow_global_fate(&global);
    assert_eq!(tile.triple_fate.global_tick, global.global_tick);
    println!("Tile 跟随全局 Fate tick {}", tile.triple_fate.global_tick);

    println!("\n== 监控：小训练三轮，每轮读一次 ==");
    let mut monitor = BodyMonitor::new();
    let mut stem2 = StemCellComplete::new();
    let mut tile2 = TileQuattroComplete::new();
    for round in 1..=3 {
        for i in 0..300 {
            let v = ((round * 100 + i) % 50) as f64 * 0.1 + 0.01;
            stem2.receive_data(v);
            tile2.ingest(v);
            tile2.promote_from_fate();
        }
        let tc = tile2.process_tiles_cycle();
        let sc = stem2.process_cells_cycle();
        stem2.settle_field();
        tile2.propagate(1.0);
        monitor.observe_cycle(&tc, &sc);
        println!("round {round} {}", monitor.snapshot(&tile2, &stem2).line());
    }

    println!("\n核查完成：四元转换、BaseForce 协调、Stem/Tile 局部循环、全局跟随、监控读数都在运行。");
}
