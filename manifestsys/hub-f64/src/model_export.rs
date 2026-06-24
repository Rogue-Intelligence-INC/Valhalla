//! Valhalla baked-model export manifest — structure from Hub / Tile / StemCell / Triad
//! is incubated once and written into a **new** checkpoint (base model never modified).

use std::path::Path;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::patch_export::{self, ExportDoc};
use crate::signal_ingress::TriadFeedReport;

pub const MANIFEST_VERSION: &str = "valhalla-bake-1.0";

/// Supported incubation bodies (matches `BodyTarget` / hybrid memory bodies).
pub const BODIES: &[&str] = &["hub", "tile", "stemcell", "triad"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IncubationIngress {
    /// Grinder hash ports — same protocol as hybrid / native_qa batch.
    Grinder,
    /// Legacy normalized_f64 text feed (`valhalla_model_export` default).
    NormalizedF64,
}

impl Default for IncubationIngress {
    fn default() -> Self {
        Self::Grinder
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncubationProvenance {
    pub body: String,
    pub ingress: String,
    pub cycles: usize,
    pub corpus_lines: usize,
    pub hub_routed: [u64; 4],
    pub hub_prefs: Vec<f64>,
    pub tile_count: usize,
    pub stem_cell_count: usize,
    pub signals_emitted: u64,
    pub patch_fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BakeProvenance {
    pub base_model_id: String,
    pub bake_mode: String,
    pub strength: f64,
    pub weight_delta_norm: Option<f64>,
    pub baked_at: String,
}

/// Written to `{checkpoint_dir}/valhalla_manifest.json` after bake.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValhallaBakedManifest {
    pub manifest_version: String,
    pub product: String,
    pub incubation: IncubationProvenance,
    pub bake: BakeProvenance,
    pub export_doc: ExportDoc,
    /// Hybrid / fused / native QA can read this to skip runtime weight surgery.
    pub runtime_patch: bool,
}

impl ValhallaBakedManifest {
    pub fn is_baked_checkpoint(path: &Path) -> bool {
        path.join("valhalla_manifest.json").is_file()
    }
}

pub fn patch_fingerprint(report: &TriadFeedReport) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mp = &report.model_patch;
    let mut h = DefaultHasher::new();
    for s in mp.quad_layer_scales {
        s.to_bits().hash(&mut h);
    }
    mp.patch_vector.len().hash(&mut h);
    if let Some(&x) = mp.patch_vector.first() {
        x.to_bits().hash(&mut h);
    }
    mp.tile_count.hash(&mut h);
    mp.stem_cell_count.hash(&mut h);
    format!("{:016x}", h.finish())
}

pub fn provenance_from_report(
    body: &str,
    ingress: IncubationIngress,
    cycles: usize,
    corpus_lines: usize,
    report: &TriadFeedReport,
) -> IncubationProvenance {
    IncubationProvenance {
        body: body.to_ascii_lowercase(),
        ingress: format!("{ingress:?}").to_ascii_lowercase(),
        cycles,
        corpus_lines,
        hub_routed: report.hub_routed,
        hub_prefs: report.hub_prefs.clone(),
        tile_count: report.tile_completed,
        stem_cell_count: report.stem_cells,
        signals_emitted: report.stats.signals_emitted,
        patch_fingerprint: patch_fingerprint(report),
    }
}

/// Incubate corpus with grinder ingress (hybrid-aligned).
pub fn export_body_grinder(body: &str, cycles: usize, corpus: &[String]) -> ExportDoc {
    patch_export::export_patch_from_corpus(body, cycles, corpus)
}

/// Build a pre-bake manifest (structure JSON only; weights applied by `bake_valhalla_model.py`).
pub fn build_prebake_manifest(
    export_doc: ExportDoc,
    base_model_id: &str,
    bake_mode: &str,
    strength: f64,
    ingress: IncubationIngress,
    corpus_lines: usize,
) -> ValhallaBakedManifest {
    let body = export_doc.body.clone();
    let report = &export_doc.report;
    ValhallaBakedManifest {
        manifest_version: MANIFEST_VERSION.to_string(),
        product: "valhalla-baked-lm".into(),
        incubation: provenance_from_report(&body, ingress, report.optimize_cycles, corpus_lines, report),
        bake: BakeProvenance {
            base_model_id: base_model_id.to_string(),
            bake_mode: bake_mode.to_string(),
            strength,
            weight_delta_norm: None,
            baked_at: Utc::now().to_rfc3339(),
        },
        export_doc,
        runtime_patch: false,
    }
}

pub fn load_manifest(model_dir: &Path) -> Result<ValhallaBakedManifest, String> {
    let path = model_dir.join("valhalla_manifest.json");
    let raw = std::fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    serde_json::from_str(&raw).map_err(|e| format!("parse manifest: {e}"))
}

pub fn write_manifest(manifest: &ValhallaBakedManifest, model_dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(model_dir).map_err(|e| e.to_string())?;
    let path = model_dir.join("valhalla_manifest.json");
    let json = serde_json::to_string_pretty(manifest).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())
}
