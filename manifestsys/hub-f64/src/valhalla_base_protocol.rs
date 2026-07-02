//! Unified programmatic entry/exit for protocol `valhalla-base-v1`.
//!
//! Rust callers: `execute_request` / `execute_request_json`  
//! CLI: `valhalla_base` binary (stdin/stdout JSON)  
//! C/C++: subprocess JSON — see `include/valhalla_base.h` for constants and field names.

use serde::{Deserialize, Serialize};

use crate::valhalla_base::{
    run_valhalla_base, ValhallaBaseConfig, ValhallaBaseDecode, ValhallaBaseRequest,
    ValhallaBaseResponse, ValhallaFateIngressRouting, ValhallaFollowUpDecode,
    ValhallaMcqDecode, VALHALLA_BASE_PROTOCOL,
};

/// Production defaults (matches Python `ValhallaBase` wrapper).
pub fn production_config() -> ValhallaBaseConfig {
    ValhallaBaseConfig {
        decode: ValhallaBaseDecode::Hybrid,
        follow_up_decode: ValhallaFollowUpDecode::NativeFollowUpAware,
        fate_ingress_routing: ValhallaFateIngressRouting::QuadCycle,
        fate_qa_feedback: true,
        fate_routing_free: false,
        proactive_enabled: true,
        ..ValhallaBaseConfig::default()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ValhallaBaseErrorKind {
    Parse,
    Validate,
    Runtime,
    Io,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValhallaBaseErrorBody {
    pub protocol: String,
    pub ok: bool,
    pub error: String,
    pub error_kind: ValhallaBaseErrorKind,
}

impl ValhallaBaseErrorBody {
    pub fn new(kind: ValhallaBaseErrorKind, error: impl Into<String>) -> Self {
        Self {
            protocol: VALHALLA_BASE_PROTOCOL.into(),
            ok: false,
            error: error.into(),
            error_kind: kind,
        }
    }
}

/// Parse JSON request string into typed request.
pub fn parse_request_json(input: &str) -> Result<ValhallaBaseRequest, ValhallaBaseErrorBody> {
    serde_json::from_str(input.trim()).map_err(|e| {
        ValhallaBaseErrorBody::new(
            ValhallaBaseErrorKind::Parse,
            format!("parse request: {e}"),
        )
    })
}

/// Validate minimal request shape before incubation/decode.
pub fn validate_request(req: &ValhallaBaseRequest) -> Result<(), ValhallaBaseErrorBody> {
    let has_context = !req.context.is_empty()
        || !req.context_items.is_empty()
        || !req.append_lines.is_empty()
        || !req.append_items.is_empty()
        || req.session.is_some();
    if !has_context {
        return Err(ValhallaBaseErrorBody::new(
            ValhallaBaseErrorKind::Validate,
            "context must contain lines/items, append payload, or session",
        ));
    }
    if req.questions.is_empty() && !req.proactive_idle {
        return Err(ValhallaBaseErrorBody::new(
            ValhallaBaseErrorKind::Validate,
            "questions must be non-empty (or set proactive_idle=true)",
        ));
    }
    Ok(())
}

/// Primary in-process entry — runs incubation + decode pipeline.
pub fn execute_request(
    req: &ValhallaBaseRequest,
) -> Result<ValhallaBaseResponse, ValhallaBaseErrorBody> {
    validate_request(req)?;
    run_valhalla_base(req).map_err(|e| ValhallaBaseErrorBody::new(ValhallaBaseErrorKind::Runtime, e))
}

/// JSON in → JSON out (success response or error envelope on failure).
pub fn execute_request_json(input: &str, pretty: bool) -> Result<String, ValhallaBaseErrorBody> {
    let req = parse_request_json(input)?;
    let resp = execute_request(&req)?;
    serialize_response(&resp, pretty)
}

pub fn serialize_response(
    resp: &ValhallaBaseResponse,
    pretty: bool,
) -> Result<String, ValhallaBaseErrorBody> {
    if pretty {
        serde_json::to_string_pretty(resp)
    } else {
        serde_json::to_string(resp)
    }
    .map_err(|e| ValhallaBaseErrorBody::new(ValhallaBaseErrorKind::Io, format!("serialize: {e}")))
}

pub fn serialize_error(err: &ValhallaBaseErrorBody, pretty: bool) -> String {
    if pretty {
        serde_json::to_string_pretty(err).unwrap_or_else(|_| err.error.clone())
    } else {
        serde_json::to_string(err).unwrap_or_else(|_| err.error.clone())
    }
}

const EXAMPLES: &[(&str, &str)] = &[
    ("open", "examples/valhalla_base/open_qa.json"),
    ("session", "examples/valhalla_base/session_append.json"),
    ("mcq", "examples/valhalla_base/mcq_hybrid.json"),
    ("proactive", "examples/valhalla_base/proactive_idle.json"),
    ("multimodal", "examples/valhalla_base/multimodal_context.json"),
    ("production", "examples/valhalla_base/production_stack.json"),
];

/// Load bundled example request by name (`open`, `session`, `mcq`, `proactive`, `multimodal`, `production`).
pub fn example_request(name: &str) -> Result<ValhallaBaseRequest, ValhallaBaseErrorBody> {
    let rel = EXAMPLES
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, p)| *p)
        .ok_or_else(|| {
            let names: Vec<&str> = EXAMPLES.iter().map(|(n, _)| *n).collect();
            ValhallaBaseErrorBody::new(
                ValhallaBaseErrorKind::Validate,
                format!("unknown example {name:?}; choose one of: {}", names.join(", ")),
            )
        })?;
    let path = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), rel);
    let raw = std::fs::read_to_string(&path).map_err(|e| {
        ValhallaBaseErrorBody::new(
            ValhallaBaseErrorKind::Io,
            format!("read example {path}: {e}"),
        )
    })?;
    parse_request_json(&raw)
}

pub fn example_names() -> Vec<&'static str> {
    EXAMPLES.iter().map(|(n, _)| *n).collect()
}

/// Validate JSON file/string without running inference (parse + validate only).
pub fn validate_request_json(input: &str) -> Result<ValhallaBaseRequest, ValhallaBaseErrorBody> {
    let req = parse_request_json(input)?;
    validate_request(&req)?;
    Ok(req)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_config_is_hybrid() {
        let cfg = production_config();
        assert_eq!(cfg.decode, ValhallaBaseDecode::Hybrid);
        assert_eq!(cfg.follow_up_decode, ValhallaFollowUpDecode::NativeFollowUpAware);
        assert_eq!(cfg.fate_ingress_routing, ValhallaFateIngressRouting::QuadCycle);
        assert_eq!(cfg.mcq_decode, ValhallaMcqDecode::LmPatch);
    }

    #[test]
    fn example_open_parses() {
        let req = example_request("open").expect("open example");
        assert!(!req.questions.is_empty());
    }
}
