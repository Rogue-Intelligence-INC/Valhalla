//! Triad dashboard routes — Tile / StemCell visualization (shared by serve + valhalla_dashboard).

use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    extract::State,
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use serde::Serialize;

use crate::SmokeReport;

pub const DASHBOARD_HTML: &str = include_str!("../assets/triad_dashboard.html");

#[derive(Clone)]
pub struct TriadDashboardState {
    pub report: Arc<Mutex<SmokeReport>>,
    pub last_run_unix: Arc<Mutex<u64>>,
}

#[derive(Serialize)]
pub struct TriadDashboardPayload {
    pub report: SmokeReport,
    pub last_run_unix: u64,
    pub quad_labels: [&'static str; 4],
    pub build_version: &'static str,
    pub git_sha: &'static str,
}

pub fn new_dashboard_state() -> TriadDashboardState {
    let report = SmokeReport::run();
    let now = unix_now();
    TriadDashboardState {
        report: Arc::new(Mutex::new(report)),
        last_run_unix: Arc::new(Mutex::new(now)),
    }
}

pub fn dashboard_router(state: TriadDashboardState) -> Router {
    Router::new()
        .route("/", get(dashboard_index))
        .route("/api/state", get(dashboard_state))
        .route("/api/rerun", post(dashboard_rerun))
        .with_state(state)
}

async fn dashboard_index() -> Html<&'static str> {
    Html(DASHBOARD_HTML)
}

async fn dashboard_state(State(st): State<TriadDashboardState>) -> Json<TriadDashboardPayload> {
    Json(build_payload(&st))
}

async fn dashboard_rerun(State(st): State<TriadDashboardState>) -> Json<TriadDashboardPayload> {
    let report = SmokeReport::run();
    *st.report.lock().unwrap() = report;
    *st.last_run_unix.lock().unwrap() = unix_now();
    Json(build_payload(&st))
}

fn build_payload(st: &TriadDashboardState) -> TriadDashboardPayload {
    TriadDashboardPayload {
        report: st.report.lock().unwrap().clone(),
        last_run_unix: *st.last_run_unix.lock().unwrap(),
        quad_labels: ["BlackHole", "MeshBrain", "MultiNova", "BaseForce"],
        build_version: env!("CARGO_PKG_VERSION"),
        git_sha: option_env!("VALHALLA_GIT_SHA").unwrap_or("dev"),
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
