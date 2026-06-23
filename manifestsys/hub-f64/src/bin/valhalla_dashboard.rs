//! Valhalla 可视化仪表盘 — smoke 聚合 + 架构图（轻量，无需语料/CUDA 训练）

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    Router,
    extract::State,
    response::{Html, Json},
    routing::{get, post},
};
use hub_f64::SmokeReport;
use serde::Serialize;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
struct AppState {
    report: Arc<Mutex<SmokeReport>>,
    last_run_unix: Arc<Mutex<u64>>,
}

#[derive(Serialize)]
struct DashboardPayload {
    report: SmokeReport,
    last_run_unix: u64,
    quad_labels: [&'static str; 4],
}

const HTML: &str = include_str!("valhalla_dashboard.html");

async fn index() -> Html<&'static str> {
    Html(HTML)
}

async fn api_state(State(st): State<AppState>) -> Json<DashboardPayload> {
    Json(build_payload(&st))
}

async fn api_rerun(State(st): State<AppState>) -> Json<DashboardPayload> {
    let report = SmokeReport::run();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    *st.report.lock().unwrap() = report;
    *st.last_run_unix.lock().unwrap() = now;
    Json(build_payload(&st))
}

fn build_payload(st: &AppState) -> DashboardPayload {
    DashboardPayload {
        report: st.report.lock().unwrap().clone(),
        last_run_unix: *st.last_run_unix.lock().unwrap(),
        quad_labels: ["BlackHole", "MeshBrain", "MultiNova", "BaseForce"],
    }
}

#[tokio::main]
async fn main() {
    let port: u16 = std::env::var("VALHALLA_DASHBOARD_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8765);

    println!("Valhalla dashboard: running smoke aggregation...");
    let report = SmokeReport::run();
    println!(
        "  Hub signals={} Tile={} StemCell={} → {}",
        report.hub_signals,
        report.tile_completed,
        report.stem_cells,
        if report.overall_pass { "PASS" } else { "FAIL" }
    );

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let state = AppState {
        report: Arc::new(Mutex::new(report)),
        last_run_unix: Arc::new(Mutex::new(now)),
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/api/state", get(api_state))
        .route("/api/rerun", post(api_rerun))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("Open http://127.0.0.1:{port}/");
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}
