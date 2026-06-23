//! Tile + StemCell multimodal fusion smoke — text vs TTS audio features vs fused triad.
//!
//! Reads JSON from stdin:
//! ```json
//! {
//!   "optimize_cycles": 2,
//!   "samples": [{"id":"s1","text":"hello","audio_features":[0.1,...]}]
//! }
//! ```
//! Writes JSON comparison report to stdout.

use hub_f64::{BodyTarget, DlFrontConfig, IngressBudget, ModalityInput, TriadSession};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct FusionRequest {
    #[serde(default = "default_cycles")]
    optimize_cycles: usize,
    samples: Vec<FusionSample>,
}

#[derive(Debug, Deserialize)]
struct FusionSample {
    id: String,
    text: String,
    audio_features: Vec<f64>,
}

fn default_cycles() -> usize {
    2
}

#[derive(Debug, Serialize)]
struct ModeReport {
    mode: String,
    tile_completed: usize,
    tile_ops: u64,
    tile_dim: f64,
    stem_cells: usize,
    stem_connections: usize,
    stem_dim: f64,
    patch_norm: f64,
    tile_sig_count: usize,
    stem_cluster_count: usize,
}

#[derive(Debug, Serialize)]
struct SampleFusionReport {
    id: String,
    text: String,
    modes: Vec<ModeReport>,
    text_audio_patch_cosine: f64,
    fusion_vs_text_cosine: f64,
    fusion_vs_audio_cosine: f64,
    tile_text_audio_sig_cosine: f64,
    stem_text_audio_sig_cosine: f64,
}

#[derive(Debug, Serialize)]
struct FusionResponse {
    optimize_cycles: usize,
    sidecar: bool,
    samples: Vec<SampleFusionReport>,
    summary: FusionSummary,
}

#[derive(Debug, Serialize)]
struct FusionSummary {
    avg_fusion_vs_text_cosine: f64,
    avg_text_audio_patch_cosine: f64,
    fusion_grows_structure: bool,
}

fn cosine(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len().min(b.len());
    if n == 0 {
        return 0.0;
    }
    let mut dot = 0.0;
    let mut na = 0.0;
    let mut nb = 0.0;
    for i in 0..n {
        dot += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }
    let denom = na.sqrt() * nb.sqrt();
    if denom < 1e-12 {
        0.0
    } else {
        dot / denom
    }
}

fn avg_sig_cosine(a: &[Vec<f64>], b: &[Vec<f64>]) -> f64 {
    let n = a.len().min(b.len());
    if n == 0 {
        return 0.0;
    }
    (0..n).map(|i| cosine(&a[i], &b[i])).sum::<f64>() / n as f64
}

fn patch_norm(v: &[f64]) -> f64 {
    v.iter().map(|x| x * x).sum::<f64>().sqrt()
}

fn run_mode(
    label: &str,
    budget: &IngressBudget,
    front: &DlFrontConfig,
    feeds: &[ModalityInput],
    cycles: usize,
) -> ModeReport {
    let mut session = TriadSession::with_body(budget.clone(), BodyTarget::Triad);
    for input in feeds {
        session.feed_modality(input.clone(), front);
    }
    let report = session.finalize_with_cycles(label, cycles);
    let patch = &report.model_patch;
    ModeReport {
        mode: label.to_string(),
        tile_completed: report.tile_completed,
        tile_ops: report.tile_ops,
        tile_dim: report.tile_dim,
        stem_cells: report.stem_cells,
        stem_connections: report.stem_connections,
        stem_dim: report.stem_dim,
        patch_norm: patch_norm(&patch.patch_vector),
        tile_sig_count: patch.tile_signatures.len(),
        stem_cluster_count: patch.stem_cluster_signatures.len(),
    }
}

fn finalize_patch(
    budget: &IngressBudget,
    front: &DlFrontConfig,
    feeds: &[ModalityInput],
    cycles: usize,
) -> hub_f64::TriadFeedReport {
    let mut session = TriadSession::with_body(budget.clone(), BodyTarget::Triad);
    for input in feeds {
        session.feed_modality(input.clone(), front);
    }
    session.finalize_with_cycles("patch", cycles)
}

fn main() {
    let raw = std::io::read_to_string(std::io::stdin()).unwrap_or_default();
    let req: FusionRequest = serde_json::from_str(&raw).unwrap_or_else(|e| {
        eprintln!("invalid JSON stdin: {e}");
        std::process::exit(1);
    });

    let budget = IngressBudget::experiment();
    let front = DlFrontConfig::default();
    let cycles = req.optimize_cycles.max(1);

    let mut sample_reports = Vec::new();
    let mut fusion_text_cosines = Vec::new();
    let mut text_audio_cosines = Vec::new();
    let mut fusion_structure_growth = 0usize;

    for sample in &req.samples {
        let text_input = ModalityInput::Text {
            content: sample.text.clone(),
        };
        let audio_input = ModalityInput::AudioFeatures {
            values: sample.audio_features.clone(),
        };
        let fusion_feeds = vec![text_input.clone(), audio_input.clone()];

        let text_only = run_mode("text_only", &budget, &front, &[text_input.clone()], cycles);
        let audio_only = run_mode("audio_only", &budget, &front, &[audio_input.clone()], cycles);
        let fusion = run_mode("text+audio_fusion", &budget, &front, &fusion_feeds, cycles);

        let text_patch = finalize_patch(&budget, &front, &[text_input], cycles);
        let audio_patch = finalize_patch(&budget, &front, &[audio_input], cycles);
        let fusion_patch = finalize_patch(&budget, &front, &fusion_feeds, cycles);

        let text_audio_patch_cosine = cosine(
            &text_patch.model_patch.patch_vector,
            &audio_patch.model_patch.patch_vector,
        );
        let fusion_vs_text = cosine(
            &fusion_patch.model_patch.patch_vector,
            &text_patch.model_patch.patch_vector,
        );
        let fusion_vs_audio = cosine(
            &fusion_patch.model_patch.patch_vector,
            &audio_patch.model_patch.patch_vector,
        );
        let tile_sig_cos = avg_sig_cosine(
            &text_patch.model_patch.tile_signatures,
            &audio_patch.model_patch.tile_signatures,
        );
        let stem_sig_cos = avg_sig_cosine(
            &text_patch.model_patch.stem_cluster_signatures,
            &audio_patch.model_patch.stem_cluster_signatures,
        );

        if fusion.tile_completed >= text_only.tile_completed.max(audio_only.tile_completed)
            && fusion.stem_cells >= text_only.stem_cells.max(audio_only.stem_cells)
        {
            fusion_structure_growth += 1;
        }

        fusion_text_cosines.push(fusion_vs_text);
        text_audio_cosines.push(text_audio_patch_cosine);

        sample_reports.push(SampleFusionReport {
            id: sample.id.clone(),
            text: sample.text.clone(),
            modes: vec![text_only, audio_only, fusion],
            text_audio_patch_cosine: text_audio_patch_cosine,
            fusion_vs_text_cosine: fusion_vs_text,
            fusion_vs_audio_cosine: fusion_vs_audio,
            tile_text_audio_sig_cosine: tile_sig_cos,
            stem_text_audio_sig_cosine: stem_sig_cos,
        });
    }

    let n = sample_reports.len().max(1) as f64;
    let resp = FusionResponse {
        optimize_cycles: cycles,
        sidecar: front.use_sidecar,
        samples: sample_reports,
        summary: FusionSummary {
            avg_fusion_vs_text_cosine: fusion_text_cosines.iter().sum::<f64>() / n,
            avg_text_audio_patch_cosine: text_audio_cosines.iter().sum::<f64>() / n,
            fusion_grows_structure: fusion_structure_growth > 0,
        },
    };

    println!("{}", serde_json::to_string_pretty(&resp).unwrap_or_default());
}
