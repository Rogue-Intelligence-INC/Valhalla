//! HTTP server for ValhallaBase — stateless JSON API (session in request body).

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

use crate::valhalla_base::{ValhallaBaseRequest, ValhallaBaseResponse, VALHALLA_BASE_PROTOCOL};
use crate::valhalla_base_protocol::{
    execute_request, serialize_error, ValhallaBaseErrorBody, ValhallaBaseErrorKind,
};

#[derive(Clone)]
pub struct HttpState {
    pub pretty: bool,
}

#[derive(Serialize)]
struct HealthResponse {
    protocol: &'static str,
    ok: bool,
    service: &'static str,
}

#[derive(Serialize)]
struct ProtocolInfo {
    protocol: &'static str,
    transport: &'static str,
    endpoints: Vec<&'static str>,
    header: &'static str,
}

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        protocol: VALHALLA_BASE_PROTOCOL,
        ok: true,
        service: "valhalla_base",
    })
}

pub async fn protocol_info() -> Json<ProtocolInfo> {
    Json(ProtocolInfo {
        protocol: VALHALLA_BASE_PROTOCOL,
        transport: "application/json",
        endpoints: vec!["POST /v1/infer", "GET /health", "GET /v1/protocol"],
        header: "manifestsys/hub-f64/include/valhalla_base.h",
    })
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

pub fn router(state: Arc<HttpState>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/protocol", get(protocol_info))
        .route("/v1/infer", post(infer))
        .layer(DefaultBodyLimit::max(32 * 1024 * 1024))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

pub async fn serve(addr: SocketAddr, pretty: bool) -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(HttpState { pretty });
    let app = router(state);
    eprintln!(
        "{VALHALLA_BASE_PROTOCOL}: HTTP listening on http://{addr}/health"
    );
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

/// Convenience for typed callers expecting raw response.
pub fn infer_sync(req: &ValhallaBaseRequest) -> Result<ValhallaBaseResponse, ValhallaBaseErrorBody> {
    execute_request(req)
}
