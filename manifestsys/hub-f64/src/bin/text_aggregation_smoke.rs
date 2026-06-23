//! 轻量 smoke：Hub / Tile / StemCell 训练文本自动聚合 — 仅验证预期行为可完成

use hub_f64::SmokeReport;

fn main() {
    let r = SmokeReport::run();
    println!("=== Hub / Tile / StemCell text aggregation smoke ===\n");
    println!("encoding: {} (universal ingress)", r.encoding_mode);
    println!(
        "Hub: signals={} routed={:?} prefs_len={} OK={}",
        r.hub_signals,
        r.hub_routed,
        r.hub_prefs.len(),
        r.hub_ok
    );
    println!(
        "Tile: completed={} ops={} dim={:.2} OK={}",
        r.tile_completed, r.tile_ops, r.tile_dim, r.tile_ok
    );
    println!(
        "StemCell: cells={} connections={} dim={:.2} OK={}",
        r.stem_cells, r.stem_connections, r.stem_dim, r.stem_ok
    );
    println!("\nOVERALL: {}", if r.overall_pass { "PASS" } else { "FAIL" });
    if !r.overall_pass {
        std::process::exit(1);
    }
}
