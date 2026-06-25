//! Same-task universal core experiment — text / TTS / image each complete the task,
//! then measure shared core and fusion universal core formation.

use hub_f64::dl_front::{encode_modality, BodyFrontPorts, DlFrontConfig, ModalityInput};
use hub_f64::free_multimodal_fate::{
    run_same_task_universal_core, SameTaskUniversalCoreOutcome, TriModalSample,
    UNIVERSAL_CORE_PROTOCOL,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct ExperimentRequest {
    samples: Vec<TriModalSample>,
}

#[derive(Debug, Serialize)]
struct ExperimentSummary {
    protocol: String,
    sample_count: usize,
    common_core_rate: f64,
    universal_core_formation_rate: f64,
    avg_shared_patch: f64,
    avg_fusion_vs_centroid: f64,
    avg_core_tightening: f64,
    avg_universal_core_score: f64,
    verdict: String,
}

#[derive(Debug, Serialize)]
struct ExperimentResponse {
    samples: Vec<SameTaskUniversalCoreOutcome>,
    summary: ExperimentSummary,
}

fn ports_from_text(text: &str, front: &DlFrontConfig) -> BodyFrontPorts {
    encode_modality(
        &ModalityInput::Text {
            content: text.to_string(),
        },
        front,
    )
}

fn ports_from_audio(vals: &[f64], front: &DlFrontConfig) -> BodyFrontPorts {
    encode_modality(
        &ModalityInput::AudioFeatures {
            values: vals.to_vec(),
        },
        front,
    )
}

fn ports_from_image(vals: &[f64], front: &DlFrontConfig) -> BodyFrontPorts {
    encode_modality(
        &ModalityInput::ImageFeatures {
            values: vals.to_vec(),
        },
        front,
    )
}

fn main() {
    let raw = std::io::read_to_string(std::io::stdin()).unwrap_or_default();
    let req: ExperimentRequest = serde_json::from_str(&raw).unwrap_or_else(|e| {
        eprintln!("invalid JSON stdin: {e}");
        std::process::exit(1);
    });

    if req.samples.is_empty() {
        eprintln!("need samples");
        std::process::exit(1);
    }

    let front = DlFrontConfig::default();
    let mut outcomes = Vec::new();
    let mut common_core_n = 0usize;
    let mut universal_n = 0usize;
    let mut shared_patches = Vec::new();
    let mut fusion_centroids = Vec::new();
    let mut tightenings = Vec::new();
    let mut scores = Vec::new();

    for sample in &req.samples {
        let llm = ports_from_text(&sample.text, &front);
        let tts = ports_from_audio(&sample.audio_features, &front);
        let img = ports_from_image(&sample.image_features, &front);

        eprintln!(
            "  [{}] isolate text/tts/image → fuse",
            sample.id,
        );
        let out = run_same_task_universal_core(&sample.id, &llm, &tts, &img);
        let m = &out.metrics;

        eprintln!(
            "    shared_patch={:.3} fusion↔centroid={:.3} tighten={:.3} core={}/uni={}",
            m.shared_patch_pairwise_avg,
            m.fusion_vs_centroid_cosine,
            m.core_tightening,
            m.shares_common_core,
            m.universal_core_formed,
        );

        if m.shares_common_core {
            common_core_n += 1;
        }
        if m.universal_core_formed {
            universal_n += 1;
        }
        shared_patches.push(m.shared_patch_pairwise_avg);
        fusion_centroids.push(m.fusion_vs_centroid_cosine);
        tightenings.push(m.core_tightening);
        scores.push(m.universal_core_score);

        outcomes.push(out);
    }

    let n = req.samples.len() as f64;
    let common_core_rate = common_core_n as f64 / n;
    let universal_rate = universal_n as f64 / n;
    let avg_shared = shared_patches.iter().sum::<f64>() / n;
    let avg_fusion_c = fusion_centroids.iter().sum::<f64>() / n;
    let avg_tight = tightenings.iter().sum::<f64>() / n;
    let avg_score = scores.iter().sum::<f64>() / n;

    let verdict = if common_core_rate >= 0.50 && universal_rate >= 0.35 {
        "UNIVERSAL_CORE_CONFIRMED: same-task modalities share Fate patch core; fusion consolidates universal core"
    } else if common_core_rate >= 0.25 || avg_shared >= 0.35 {
        "PARTIAL_CORE: shared patch signal present; fusion may need more rounds"
    } else {
        "NOT_YET: modalities diverge under isolation — no reliable universal core"
    };

    let resp = ExperimentResponse {
        samples: outcomes,
        summary: ExperimentSummary {
            protocol: UNIVERSAL_CORE_PROTOCOL.to_string(),
            sample_count: req.samples.len(),
            common_core_rate,
            universal_core_formation_rate: universal_rate,
            avg_shared_patch: avg_shared,
            avg_fusion_vs_centroid: avg_fusion_c,
            avg_core_tightening: avg_tight,
            avg_universal_core_score: avg_score,
            verdict: verdict.to_string(),
        },
    };

    println!("{}", serde_json::to_string_pretty(&resp).unwrap_or_default());
    if universal_rate < 0.25 && common_core_rate < 0.20 {
        std::process::exit(2);
    }
}
