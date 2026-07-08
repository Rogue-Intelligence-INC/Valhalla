//! Valhalla Triad dashboard — Tile / StemCell visualization (standalone server).

use std::net::SocketAddr;

use axum::Router;
use hub_f64::triad_dashboard;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    let port: u16 = std::env::var("VALHALLA_DASHBOARD_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8765);

    println!("Valhalla triad dashboard: running smoke aggregation...");
    let state = triad_dashboard::new_dashboard_state();
    let report = state.report.lock().unwrap();
    println!(
        "  Hub signals={} Tile={} StemCell={} tiles_viz={} → {}",
        report.hub_signals,
        report.tile_completed,
        report.stem_cells,
        report.viz.tile_neurons.len(),
        if report.overall_pass { "PASS" } else { "FAIL" }
    );
    drop(report);

    let app = Router::new()
        .merge(triad_dashboard::dashboard_router(state))
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("Open http://127.0.0.1:{port}/");
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}
