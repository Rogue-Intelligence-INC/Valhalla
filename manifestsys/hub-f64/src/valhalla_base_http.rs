//! ValhallaBase HTTP server — JSON API + optional triad dashboard.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    extract::{DefaultBodyLimit, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use tower_http::cors::CorsLayer;

use crate::patch_lm;
use crate::triad_dashboard::{self, TriadDashboardState};
use crate::valhalla_base::{ValhallaBaseRequest, ValhallaBaseResponse, VALHALLA_BASE_PROTOCOL};
use crate::valhalla_base_protocol::{
    execute_request, serialize_error, ValhallaBaseErrorBody, ValhallaBaseErrorKind,
};

#[derive(Clone)]
pub struct HttpState {
    pub pretty: bool,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub protocol: &'static str,
    pub ok: bool,
    pub service: &'static str,
    pub version: &'static str,
    pub git_sha: &'static str,
    pub endpoints: Vec<&'static str>,
}

#[derive(Serialize)]
pub struct ProtocolInfo {
    pub protocol: &'static str,
    pub transport: &'static str,
    pub endpoints: Vec<&'static str>,
    pub header: &'static str,
    pub reproduction_doc: &'static str,
}

#[derive(Serialize)]
pub struct ReproManifest {
    pub protocol: &'static str,
    pub version: &'static str,
    pub git_sha: &'static str,
    pub smoke_command: &'static str,
    pub suite_command: &'static str,
    pub dashboard_url: &'static str,
    pub infer_url: &'static str,
    pub health_url: &'static str,
}

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        protocol: VALHALLA_BASE_PROTOCOL,
        ok: true,
        service: "valhalla_base",
        version: env!("CARGO_PKG_VERSION"),
        git_sha: option_env!("VALHALLA_GIT_SHA").unwrap_or("dev"),
        endpoints: vec![
            "GET /health",
            "GET /v1/protocol",
            "GET /v1/repro",
            "POST /v1/infer",
            "POST /v1/release",
            "GET /dashboard/",
        ],
    })
}

pub async fn protocol_info() -> Json<ProtocolInfo> {
    Json(ProtocolInfo {
        protocol: VALHALLA_BASE_PROTOCOL,
        transport: "application/json",
        endpoints: vec![
            "POST /v1/infer",
            "POST /v1/release",
            "GET /health",
            "GET /v1/protocol",
            "GET /v1/repro",
            "GET /dashboard/",
        ],
        header: "manifestsys/hub-f64/include/valhalla_base.h",
        reproduction_doc: "REPRODUCTION.md",
    })
}

pub async fn repro_manifest() -> Json<ReproManifest> {
    Json(ReproManifest {
        protocol: VALHALLA_BASE_PROTOCOL,
        version: env!("CARGO_PKG_VERSION"),
        git_sha: option_env!("VALHALLA_GIT_SHA").unwrap_or("dev"),
        smoke_command: "valhalla_base demo v1",
        suite_command: "python3 tools/valhalla_inference/run_suite_sequential.py --suite fate-core",
        dashboard_url: "/dashboard/",
        infer_url: "/v1/infer",
        health_url: "/health",
    })
}

pub async fn release_lm() -> Json<HealthResponse> {
    patch_lm::clear_lm_caches();
    health().await
}

pub async fn infer(
    State(st): State<Arc<HttpState>>,
    Json(req): Json<ValhallaBaseRequest>,
) -> Response {
    match execute_request(&req) {
        Ok(resp) => Json(resp).into_response(),
        Err(err) => api_error(err, st.pretty),
    }
}

fn api_error(err: ValhallaBaseErrorBody, pretty: bool) -> Response {
    let status = match err.error_kind {
        ValhallaBaseErrorKind::Parse | ValhallaBaseErrorKind::Validate => StatusCode::BAD_REQUEST,
        ValhallaBaseErrorKind::Runtime => StatusCode::UNPROCESSABLE_ENTITY,
        ValhallaBaseErrorKind::Io => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (
        status,
        [("content-type", "application/json")],
        serialize_error(&err, pretty),
    )
        .into_response()
}

pub fn api_router(state: Arc<HttpState>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/protocol", get(protocol_info))
        .route("/v1/repro", get(repro_manifest))
        .route("/v1/infer", post(infer))
        .route("/v1/release", post(release_lm))
        .layer(DefaultBodyLimit::max(32 * 1024 * 1024))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

pub fn router(state: Arc<HttpState>, dashboard: Option<TriadDashboardState>) -> Router {
    let mut app = api_router(state);
    if let Some(dash) = dashboard {
        app = app.nest("/dashboard", triad_dashboard::dashboard_router(dash));
    }
    app
}

pub async fn serve(
    addr: SocketAddr,
    pretty: bool,
    with_dashboard: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(HttpState { pretty });
    let dashboard = if with_dashboard {
        Some(triad_dashboard::new_dashboard_state())
    } else {
        None
    };
    let app = router(state, dashboard);
    eprintln!("{VALHALLA_BASE_PROTOCOL}: HTTP http://{addr}/health");
    if with_dashboard {
        eprintln!("  triad dashboard: http://{addr}/dashboard/");
    }
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

/// Convenience for typed callers expecting raw response.
pub fn infer_sync(req: &ValhallaBaseRequest) -> Result<ValhallaBaseResponse, ValhallaBaseErrorBody> {
    execute_request(req)
}
