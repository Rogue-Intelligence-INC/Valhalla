//! Smoke: DL front-end → Hub / Tile / StemCell three ports (legacy char path unchanged).

use hub_f64::{
    BodyTarget, DlFrontConfig, IngressBudget, ModalityInput, TriadSession, SMOKE_CORPUS,
};

fn main() {
    let budget = IngressBudget::experiment();
    let front = DlFrontConfig {
        use_sidecar: std::env::var("VALHALLA_DL_FRONT")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false),
        ..Default::default()
    };

    println!("=== Valhalla DL Front → Hub / Tile / StemCell ===");
    println!(
        "sidecar={} script={:?}",
        front.use_sidecar,
        front.sidecar_script
    );

    // Legacy path (unchanged)
    let mut legacy = TriadSession::with_body(budget.clone(), BodyTarget::Triad);
    for line in SMOKE_CORPUS {
        legacy.feed(
            hub_f64::IngressPayload::Text {
                content: (*line).to_string(),
            },
            hub_f64::EncodingMode::NormalizedF64,
        );
    }
    let legacy_r = legacy.finalize("legacy_char");

    // DL front path
    let mut dl = TriadSession::with_body(budget.clone(), BodyTarget::Triad);
    for line in SMOKE_CORPUS {
        dl.feed_modality(
            ModalityInput::Text {
                content: (*line).to_string(),
            },
            &front,
        );
    }
    let dl_r = dl.finalize("dl_front");

    println!("\n--- legacy char ingress ---");
    print_report(&legacy_r);
    println!("\n--- DL front ingress ---");
    print_report(&dl_r);

    let ok = legacy_r.stats.items_processed > 0
        && dl_r.stats.items_processed > 0
        && dl_r.hub_signals > 0
        && dl_r.tile_completed > 0
        && dl_r.stem_cells > 0;

    println!("\nOVERALL: {}", if ok { "PASS" } else { "FAIL" });
    if !ok {
        std::process::exit(1);
    }
}

fn print_report(r: &hub_f64::TriadFeedReport) {
    println!("  mode={} items={}", r.mode, r.stats.items_processed);
    println!(
        "  Hub: signals={} routed={:?}",
        r.hub_signals, r.hub_routed
    );
    println!(
        "  Tile: completed={} ops={} dim={:.2}",
        r.tile_completed, r.tile_ops, r.tile_dim
    );
    println!(
        "  StemCell: cells={} conn={} dim={:.2}",
        r.stem_cells, r.stem_connections, r.stem_dim
    );
}
