//! Same-task universal core — isolated vs non-isolated layout.

use hub_f64::dl_front::{encode_modality, BodyFrontPorts, DlFrontConfig, ModalityInput};
use hub_f64::free_multimodal_fate::{
    run_same_task_universal_core, run_same_task_universal_core_compare,
    run_same_task_universal_core_non_isolated, SameTaskUniversalCoreNonIsolatedOutcome,
    SameTaskUniversalCoreOutcome, TriModalSample, UniversalCoreCompareOutcome, UniversalCoreLayout,
    UNIVERSAL_CORE_NON_ISOLATED, UNIVERSAL_CORE_PROTOCOL,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct ExperimentRequest {
    samples: Vec<TriModalSample>,
    #[serde(default)]
    layout: UniversalCoreLayout,
}

#[derive(Debug, Serialize)]
struct LayoutSummary {
    protocol: String,
    layout: String,
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
struct CompareSummary {
    sample_count: usize,
    avg_shared_patch_isolated: f64,
    avg_shared_patch_non_isolated: f64,
    avg_fusion_centroid_isolated: f64,
    avg_fusion_centroid_non_isolated: f64,
    avg_universal_score_isolated: f64,
    avg_universal_score_non_isolated: f64,
    non_isolated_wins: usize,
    isolated_wins: usize,
    verdict: String,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum ExperimentResponse {
    Isolated {
        samples: Vec<SameTaskUniversalCoreOutcome>,
        summary: LayoutSummary,
    },
    NonIsolated {
        samples: Vec<SameTaskUniversalCoreNonIsolatedOutcome>,
        summary: LayoutSummary,
    },
    Compare {
        samples: Vec<UniversalCoreCompareOutcome>,
        summary: CompareSummary,
    },
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

fn summarize_isolated(outcomes: &[SameTaskUniversalCoreOutcome]) -> LayoutSummary {
    let n = outcomes.len() as f64;
    let common_core_n = outcomes
        .iter()
        .filter(|o| o.metrics.shares_common_core)
        .count();
    let universal_n = outcomes
        .iter()
        .filter(|o| o.metrics.universal_core_formed)
        .count();
    let avg_shared = outcomes
        .iter()
        .map(|o| o.metrics.shared_patch_pairwise_avg)
        .sum::<f64>()
        / n;
    let avg_fusion_c = outcomes
        .iter()
        .map(|o| o.metrics.fusion_vs_centroid_cosine)
        .sum::<f64>()
        / n;
    let avg_tight = outcomes
        .iter()
        .map(|o| o.metrics.core_tightening)
        .sum::<f64>()
        / n;
    let avg_score = outcomes
        .iter()
        .map(|o| o.metrics.universal_core_score)
        .sum::<f64>()
        / n;
    let common_core_rate = common_core_n as f64 / n;
    let universal_rate = universal_n as f64 / n;
    let verdict = if common_core_rate >= 0.50 && universal_rate >= 0.35 {
        "UNIVERSAL_CORE_CONFIRMED (isolated): separate modality runs share Fate patch core"
    } else if common_core_rate >= 0.25 || avg_shared >= 0.35 {
        "PARTIAL_CORE (isolated)"
    } else {
        "NOT_YET (isolated)"
    };
    LayoutSummary {
        protocol: UNIVERSAL_CORE_PROTOCOL.to_string(),
        layout: "isolated".into(),
        sample_count: outcomes.len(),
        common_core_rate,
        universal_core_formation_rate: universal_rate,
        avg_shared_patch: avg_shared,
        avg_fusion_vs_centroid: avg_fusion_c,
        avg_core_tightening: avg_tight,
        avg_universal_core_score: avg_score,
        verdict: verdict.to_string(),
    }
}

fn summarize_non_isolated(outcomes: &[SameTaskUniversalCoreNonIsolatedOutcome]) -> LayoutSummary {
    let n = outcomes.len() as f64;
    let common_core_n = outcomes
        .iter()
        .filter(|o| o.metrics.shares_common_core)
        .count();
    let universal_n = outcomes
        .iter()
        .filter(|o| o.metrics.universal_core_formed)
        .count();
    let avg_shared = outcomes
        .iter()
        .map(|o| o.metrics.shared_patch_pairwise_avg)
        .sum::<f64>()
        / n;
    let avg_fusion_c = outcomes
        .iter()
        .map(|o| o.metrics.fusion_vs_centroid_cosine)
        .sum::<f64>()
        / n;
    let avg_tight = outcomes
        .iter()
        .map(|o| o.metrics.core_tightening)
        .sum::<f64>()
        / n;
    let avg_score = outcomes
        .iter()
        .map(|o| o.metrics.universal_core_score)
        .sum::<f64>()
        / n;
    let common_core_rate = common_core_n as f64 / n;
    let universal_rate = universal_n as f64 / n;
    let verdict = if common_core_rate >= 0.50 && universal_rate >= 0.35 {
        "UNIVERSAL_CORE_CONFIRMED (non-isolated): concurrent session forms shared core"
    } else if common_core_rate >= 0.25 || avg_shared >= 0.35 {
        "PARTIAL_CORE (non-isolated)"
    } else {
        "NOT_YET (non-isolated)"
    };
    LayoutSummary {
        protocol: UNIVERSAL_CORE_NON_ISOLATED.to_string(),
        layout: "non_isolated".into(),
        sample_count: outcomes.len(),
        common_core_rate,
        universal_core_formation_rate: universal_rate,
        avg_shared_patch: avg_shared,
        avg_fusion_vs_centroid: avg_fusion_c,
        avg_core_tightening: avg_tight,
        avg_universal_core_score: avg_score,
        verdict: verdict.to_string(),
    }
}

fn summarize_compare(outcomes: &[UniversalCoreCompareOutcome]) -> CompareSummary {
    let n = outcomes.len() as f64;
    let avg_iso_shared = outcomes
        .iter()
        .map(|o| o.isolated.metrics.shared_patch_pairwise_avg)
        .sum::<f64>()
        / n;
    let avg_non_shared = outcomes
        .iter()
        .map(|o| o.non_isolated.metrics.shared_patch_pairwise_avg)
        .sum::<f64>()
        / n;
    let avg_iso_fc = outcomes
        .iter()
        .map(|o| o.isolated.metrics.fusion_vs_centroid_cosine)
        .sum::<f64>()
        / n;
    let avg_non_fc = outcomes
        .iter()
        .map(|o| o.non_isolated.metrics.fusion_vs_centroid_cosine)
        .sum::<f64>()
        / n;
    let avg_iso_score = outcomes
        .iter()
        .map(|o| o.isolated.metrics.universal_core_score)
        .sum::<f64>()
        / n;
    let avg_non_score = outcomes
        .iter()
        .map(|o| o.non_isolated.metrics.universal_core_score)
        .sum::<f64>()
        / n;
    let non_wins = outcomes.iter().filter(|o| o.non_isolated_better).count();
    let iso_wins = outcomes.len() - non_wins;
    let verdict = if avg_non_score > avg_iso_score + 0.02 {
        "NON_ISOLATED_BETTER: single-session concurrent fusion tightens universal core more"
    } else if avg_iso_score > avg_non_score + 0.02 {
        "ISOLATED_BETTER: modality separation before fusion yields clearer shared core"
    } else {
        "EQUIVALENT: isolated and non-isolated reach similar universal core quality"
    };
    CompareSummary {
        sample_count: outcomes.len(),
        avg_shared_patch_isolated: avg_iso_shared,
        avg_shared_patch_non_isolated: avg_non_shared,
        avg_fusion_centroid_isolated: avg_iso_fc,
        avg_fusion_centroid_non_isolated: avg_non_fc,
        avg_universal_score_isolated: avg_iso_score,
        avg_universal_score_non_isolated: avg_non_score,
        non_isolated_wins: non_wins,
        isolated_wins: iso_wins,
        verdict: verdict.to_string(),
    }
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

    let resp = match req.layout {
        UniversalCoreLayout::Isolated => {
            let mut outcomes = Vec::new();
            for sample in &req.samples {
                let llm = ports_from_text(&sample.text, &front);
                let tts = ports_from_audio(&sample.audio_features, &front);
                let img = ports_from_image(&sample.image_features, &front);
                eprintln!("  [{}] isolated text/tts/image → fuse", sample.id);
                let out = run_same_task_universal_core(&sample.id, &llm, &tts, &img);
                eprintln!(
                    "    shared={:.3} fusion↔c={:.3} uni={}",
                    out.metrics.shared_patch_pairwise_avg,
                    out.metrics.fusion_vs_centroid_cosine,
                    out.metrics.universal_core_formed
                );
                outcomes.push(out);
            }
            ExperimentResponse::Isolated {
                summary: summarize_isolated(&outcomes),
                samples: outcomes,
            }
        }
        UniversalCoreLayout::NonIsolated => {
            let mut outcomes = Vec::new();
            for sample in &req.samples {
                let llm = ports_from_text(&sample.text, &front);
                let tts = ports_from_audio(&sample.audio_features, &front);
                let img = ports_from_image(&sample.image_features, &front);
                eprintln!("  [{}] non-isolated concurrent session", sample.id);
                let out =
                    run_same_task_universal_core_non_isolated(&sample.id, &llm, &tts, &img);
                eprintln!(
                    "    shared={:.3} fusion↔c={:.3} uni={}",
                    out.metrics.shared_patch_pairwise_avg,
                    out.metrics.fusion_vs_centroid_cosine,
                    out.metrics.universal_core_formed
                );
                outcomes.push(out);
            }
            ExperimentResponse::NonIsolated {
                summary: summarize_non_isolated(&outcomes),
                samples: outcomes,
            }
        }
        UniversalCoreLayout::Compare => {
            let mut outcomes = Vec::new();
            for sample in &req.samples {
                let llm = ports_from_text(&sample.text, &front);
                let tts = ports_from_audio(&sample.audio_features, &front);
                let img = ports_from_image(&sample.image_features, &front);
                eprintln!("  [{}] compare isolated vs non-isolated", sample.id);
                let out = run_same_task_universal_core_compare(&sample.id, &llm, &tts, &img);
                eprintln!(
                    "    iso shared={:.3} non-iso shared={:.3} Δscore={:+.3}",
                    out.isolated.metrics.shared_patch_pairwise_avg,
                    out.non_isolated.metrics.shared_patch_pairwise_avg,
                    out.universal_score_delta
                );
                outcomes.push(out);
            }
            ExperimentResponse::Compare {
                summary: summarize_compare(&outcomes),
                samples: outcomes,
            }
        }
    };

    println!("{}", serde_json::to_string_pretty(&resp).unwrap_or_default());
}
