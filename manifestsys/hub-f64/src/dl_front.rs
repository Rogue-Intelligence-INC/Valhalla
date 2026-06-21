//! Deep-learning modality front-end — sits **before** Hub / Tile / StemCell ingress.
//!
//! Raw text / image / audio / precomputed embeddings are encoded once, then projected
//! into three body-specific f64 signal streams (hub, tile, stem ports).
//!
//! DL runs in Python sidecar (`tools/valhalla_dl_front/encode.py`) when available;
//! deterministic hash embedding is the offline fallback (same family as ingress experiment).

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::io::Write;

use crate::IngressBudget;

pub const DL_EMBED_DIM: usize = 384;
pub const BODY_PORT_SEED_HUB: u64 = 0x4855425f464E54; // "HUB_FNT"
pub const BODY_PORT_SEED_TILE: u64 = 0x54494C455F464E54;
pub const BODY_PORT_SEED_STEM: u64 = 0x5354454D5F464E54;

/// Modality accepted at the DL front (encoder scope only — no LM generate).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "modality", rename_all = "snake_case")]
pub enum ModalityInput {
    Text { content: String },
    ImagePath { path: String },
    ImageFeatures { values: Vec<f64> },
    AudioFeatures { values: Vec<f64> },
    Embedding { values: Vec<f64> },
}

/// Three pre-body ports: one signal stream per Valhalla body subsystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyFrontPorts {
    pub hub: Vec<f64>,
    pub tile: Vec<f64>,
    pub stem: Vec<f64>,
    /// Unified embedding before per-body projection (for retrieval / QA).
    pub embedding: Vec<f64>,
    pub embed_dim: usize,
    pub encoder: String,
    pub modality_label: String,
}

#[derive(Debug, Clone)]
pub struct DlFrontConfig {
    pub embed_dim: usize,
    pub signals_per_port: usize,
    /// Python `encode.py` — set via `VALHALLA_DL_FRONT_SIDECAR` or default repo path.
    pub sidecar_script: Option<PathBuf>,
    pub use_sidecar: bool,
}

impl Default for DlFrontConfig {
    fn default() -> Self {
        let sidecar = std::env::var("VALHALLA_DL_FRONT_SIDECAR")
            .ok()
            .map(PathBuf::from)
            .or_else(default_sidecar_path);
        let use_sidecar = std::env::var("VALHALLA_DL_FRONT")
            .map(|v| {
                let v = v.trim().to_ascii_lowercase();
                v == "1" || v == "true" || v == "on"
            })
            .unwrap_or(false);
        Self {
            embed_dim: DL_EMBED_DIM,
            signals_per_port: IngressBudget::experiment().max_signals_per_item,
            sidecar_script: sidecar,
            use_sidecar,
        }
    }
}

fn default_sidecar_path() -> Option<PathBuf> {
    let candidates = [
        PathBuf::from("tools/valhalla_dl_front/encode.py"),
        PathBuf::from("/home/jouly/AI/Valhalla/tools/valhalla_dl_front/encode.py"),
    ];
    for p in candidates {
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

/// Deterministic normalized embedding (offline / CI fallback).
pub fn hash_embed(text: &str, dim: usize, seed: u64) -> Vec<f64> {
    let mut out = vec![0.0f64; dim];
    let mut h = seed.wrapping_mul(0x9E3779B97F4A7C15);
    for b in text.bytes() {
        h = h.wrapping_mul(6364136223846793005).wrapping_add(b as u64);
    }
    for i in 0..dim {
        h = h.wrapping_mul(6364136223846793005).wrapping_add(i as u64);
        out[i] = ((h >> 33) as f64 / u32::MAX as f64) * 2.0 - 1.0;
    }
    normalize(&mut out);
    out
}

fn normalize(v: &mut [f64]) {
    let n: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-9);
    for x in v.iter_mut() {
        *x /= n;
    }
}

/// Project unified embedding → body-specific port (fixed orthogonal-style mix).
pub fn project_port(embedding: &[f64], body_seed: u64, cap: usize) -> Vec<f64> {
    if embedding.is_empty() || cap == 0 {
        return Vec::new();
    }
    let mut out = vec![0.0f64; cap];
    let mut h = body_seed;
    for (i, slot) in out.iter_mut().enumerate() {
        h = h.wrapping_mul(6364136223846793005).wrapping_add(i as u64);
        let j = (h as usize) % embedding.len();
        let w = ((h >> 17) as f64 / u32::MAX as f64) * 2.0 - 1.0;
        *slot = embedding[j] * w;
    }
    normalize(&mut out);
    out
}

pub fn build_body_ports(embedding: &[f64], cfg: &DlFrontConfig, encoder: &str, modality_label: &str) -> BodyFrontPorts {
    BodyFrontPorts {
        hub: project_port(embedding, BODY_PORT_SEED_HUB, cfg.signals_per_port),
        tile: project_port(embedding, BODY_PORT_SEED_TILE, cfg.signals_per_port),
        stem: project_port(embedding, BODY_PORT_SEED_STEM, cfg.signals_per_port),
        embedding: embedding.to_vec(),
        embed_dim: embedding.len(),
        encoder: encoder.to_string(),
        modality_label: modality_label.to_string(),
    }
}

fn sidecar_encode(input: &ModalityInput, script: &Path) -> Result<(Vec<f64>, String), String> {
    let json = serde_json::to_string(input).map_err(|e| e.to_string())?;
    let mut child = Command::new("python3")
        .arg(script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("spawn sidecar: {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(json.as_bytes())
            .map_err(|e| format!("sidecar stdin: {e}"))?;
    }

    let out = child
        .wait_with_output()
        .map_err(|e| format!("sidecar wait: {e}"))?;

    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(format!("sidecar failed: {err}"));
    }

    let resp: SidecarResponse = serde_json::from_slice(&out.stdout)
        .map_err(|e| format!("sidecar json: {e} | {}", String::from_utf8_lossy(&out.stdout)))?;

    Ok((resp.embedding, resp.encoder))
}

#[derive(Debug, Deserialize)]
struct SidecarResponse {
    embedding: Vec<f64>,
    encoder: String,
}

fn local_encode(input: &ModalityInput, cfg: &DlFrontConfig) -> (Vec<f64>, String) {
    match input {
        ModalityInput::Embedding { values } => (values.clone(), "precomputed".into()),
        ModalityInput::ImageFeatures { values } | ModalityInput::AudioFeatures { values } => {
            (pad_or_truncate(values, cfg.embed_dim), "feature_vector".into())
        }
        ModalityInput::ImagePath { path } => (
            hash_embed(path, cfg.embed_dim, 0x494D475F50415448),
            "hash_image_path".into(),
        ),
        ModalityInput::Text { content } => (
            hash_embed(content, cfg.embed_dim, 0x544558545F464E54),
            "hash_text".into(),
        ),
    }
}

fn pad_or_truncate(v: &[f64], dim: usize) -> Vec<f64> {
    if v.len() == dim {
        return v.to_vec();
    }
    if v.len() > dim {
        return v[..dim].to_vec();
    }
    let mut out = v.to_vec();
    out.resize(dim, 0.0);
    normalize(&mut out);
    out
}

fn modality_label(input: &ModalityInput) -> String {
    match input {
        ModalityInput::Text { .. } => "text".into(),
        ModalityInput::ImagePath { .. } => "image_path".into(),
        ModalityInput::ImageFeatures { .. } => "image_features".into(),
        ModalityInput::AudioFeatures { .. } => "audio_features".into(),
        ModalityInput::Embedding { .. } => "embedding".into(),
    }
}

/// Encode one modality item → three body front ports.
pub fn encode_modality(input: &ModalityInput, cfg: &DlFrontConfig) -> BodyFrontPorts {
    let label = modality_label(input);

    let (embedding, encoder) = if cfg.use_sidecar {
        if let Some(script) = cfg.sidecar_script.as_ref().filter(|p| p.is_file()) {
            match sidecar_encode(input, script) {
                Ok(pair) => pair,
                Err(e) => {
                    log::warn!("DL front sidecar fallback ({e})");
                    local_encode(input, cfg)
                }
            }
        } else {
            local_encode(input, cfg)
        }
    } else {
        local_encode(input, cfg)
    };

    build_body_ports(&embedding, cfg, &encoder, &label)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_ports_differ() {
        let cfg = DlFrontConfig {
            use_sidecar: false,
            ..Default::default()
        };
        let input = ModalityInput::Text {
            content: "Valhalla triad ingress".into(),
        };
        let ports = encode_modality(&input, &cfg);
        assert!(!ports.hub.is_empty());
        assert_ne!(ports.hub, ports.tile);
        assert_ne!(ports.tile, ports.stem);
    }
}
