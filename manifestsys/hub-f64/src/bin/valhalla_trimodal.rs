//! Tri-modal potential experiment — LLM + TTS + image-gen, free body routing (v3).

use hub_f64::dl_front::{encode_modality, BodyFrontPorts, DlFrontConfig, ModalityInput};
use hub_f64::free_multimodal_fate::{
    run_free_trimodal_fate, FreeMultimodalOutcome, TriModalSample, FREE_MULTIMODAL_FUSION_V3,
};
use hub_f64::{BodyTarget, IngressBudget, TriadSession};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct ExperimentRequest {
    samples: Vec<TriModalSample>,
}

#[derive(Debug, Serialize)]
struct SampleReport {
    id: String,
    outcome: FreeMultimodalOutcome,
}

#[derive(Debug, Serialize)]
struct PotentialSummary {
    protocol: String,
    sample_count: usize,
    emergence_rate: f64,
    avg_natural_multimodal_score: f64,
    avg_adaptation_rounds: f64,
    tile_superadditive_rate: f64,
    potential_confirmed: bool,
    verdict: String,
}

#[derive(Debug, Serialize)]
struct ExperimentResponse {
    samples: Vec<SampleReport>,
    summary: PotentialSummary,
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

    let budget = IngressBudget::incubation();
    let front = DlFrontConfig::default();
    let n = req.samples.len();

    let mut reports = Vec::new();
    let mut emergence_count = 0usize;
    let mut natural_scores = Vec::new();
    let mut adapt_rounds = Vec::new();
    let mut tile_super = 0usize;

    for sample in &req.samples {
        let llm = ports_from_text(&sample.text, &front);
        let tts = ports_from_audio(&sample.audio_features, &front);
        let img = ports_from_image(&sample.image_features, &front);

        let mut session = TriadSession::with_body(budget.clone(), BodyTarget::Triad);
        let outcome = run_free_trimodal_fate(&mut session, &llm, &tts, &img, "trimodal_fate");

        let e = &outcome.emergence;
        if e.emergence_likely {
            emergence_count += 1;
        }
        if e.tile_superadditive {
            tile_super += 1;
        }
        natural_scores.push(e.natural_multimodal_score);
        adapt_rounds.push(outcome.adaptation_rounds as f64);

        eprintln!(
            "  [{}] rounds={} score={:.3} emerge={}",
            sample.id,
            outcome.adaptation_rounds,
            e.natural_multimodal_score,
            e.emergence_likely,
        );

        reports.push(SampleReport {
            id: sample.id.clone(),
            outcome,
        });
    }

    let nf = n as f64;
    let emergence_rate = emergence_count as f64 / nf;
    let avg_natural = natural_scores.iter().sum::<f64>() / nf;
    let avg_rounds = adapt_rounds.iter().sum::<f64>() / nf;
    let tile_rate = tile_super as f64 / nf;

    let potential_confirmed = emergence_rate >= 0.30 && avg_natural >= 0.28 && tile_rate >= 0.35;

    let verdict = if potential_confirmed {
        "TRIMODAL_POTENTIAL_CONFIRMED: LLM+TTS+image-gen free fusion shows emergent structure"
    } else if emergence_rate >= 0.12 || avg_natural >= 0.18 {
        "PARTIAL_SIGNAL: extend corpus / image model quality / fusion rounds"
    } else {
        "NOT_YET: no reliable tri-modal emergence under current settings"
    };

    let resp = ExperimentResponse {
        samples: reports,
        summary: PotentialSummary {
            protocol: FREE_MULTIMODAL_FUSION_V3.to_string(),
            sample_count: n,
            emergence_rate,
            avg_natural_multimodal_score: avg_natural,
            avg_adaptation_rounds: avg_rounds,
            tile_superadditive_rate: tile_rate,
            potential_confirmed,
            verdict: verdict.to_string(),
        },
    };

    println!("{}", serde_json::to_string_pretty(&resp).unwrap_or_default());
    if !potential_confirmed {
        std::process::exit(2);
    }
}
