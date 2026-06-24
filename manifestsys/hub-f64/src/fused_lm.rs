//! Fused Valhalla LM — single forward-pass product (no hybrid router, no memory track).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::fair_score::{score_answer, FairPrompt, ScoreResult};
use crate::fair_suite::classify_slices;
use crate::hybrid::PatchTrackRow;
use crate::patch_lm::{run_patch_track_v4, McqDecodeMode, PatchLmConfig};

pub const FUSED_PRODUCT_VERSION: &str = "fused-1.0-valhalla-lm";
pub const FAIR_BENCHMARK_VERSION: &str = "fair-1.2";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusedRow {
    pub id: String,
    pub score_type: String,
    pub correct: bool,
    pub score: ScoreResult,
    pub answer: String,
    pub decode_path: String,
    pub fused_product_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TypeSummary {
    pub n: usize,
    pub correct: usize,
    #[serde(default)]
    pub acc: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FusedSummary {
    pub total: TypeSummary,
    pub open: TypeSummary,
    pub mcq: TypeSummary,
    pub numeric: TypeSummary,
    pub other: TypeSummary,
    pub decode_path_counts: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusedEvalOutput {
    pub fused_product_version: String,
    pub fair_benchmark_version: String,
    pub protocol: String,
    pub summary: FusedSummary,
    pub fused_rows: Vec<FusedRow>,
}

fn answer_from_patch(row: &PatchTrackRow) -> String {
    row.assistant
        .clone()
        .or_else(|| row.native_answer.clone())
        .unwrap_or_default()
}

fn summarize_rows(rows: &[FusedRow]) -> FusedSummary {
    let mut out = FusedSummary::default();
    out.total.n = rows.len();
    out.total.correct = rows.iter().filter(|r| r.correct).count();
    out.total.acc = if out.total.n > 0 {
        out.total.correct as f64 / out.total.n as f64
    } else {
        0.0
    };

    for r in rows {
        *out.decode_path_counts.entry(r.decode_path.clone()).or_insert(0) += 1;
        let bucket = match r.score_type.as_str() {
            "open" | "open_generation" => &mut out.open,
            "mcq" => &mut out.mcq,
            "numeric" => &mut out.numeric,
            _ => &mut out.other,
        };
        bucket.n += 1;
        if r.correct {
            bucket.correct += 1;
        }
    }

    for bucket in [&mut out.open, &mut out.mcq, &mut out.numeric, &mut out.other] {
        bucket.acc = if bucket.n > 0 {
            bucket.correct as f64 / bucket.n as f64
        } else {
            0.0
        };
    }
    out
}

pub fn patch_rows_to_fused(rows: &[PatchTrackRow], prompts: &[FairPrompt]) -> Vec<FusedRow> {
    let id_to_prompt: HashMap<_, _> = prompts.iter().map(|p| (p.id.as_str(), p)).collect();
    rows.iter()
        .filter_map(|pr| {
            id_to_prompt.get(pr.id.as_str()).map(|p| {
                let answer = answer_from_patch(pr);
                let sc = score_answer(&answer, p);
                let decode_path = pr
                    .decode_path
                    .clone()
                    .unwrap_or_else(|| "fused_unknown".into());
                FusedRow {
                    id: p.id.clone(),
                    score_type: p.score_type.clone(),
                    correct: sc.correct,
                    score: sc,
                    answer,
                    decode_path,
                    fused_product_version: FUSED_PRODUCT_VERSION.to_string(),
                }
            })
        })
        .collect()
}

/// Run fused eval: pipeline v4 patch + LM only; MCQ via logprob batch (no memory router).
#[cfg(feature = "patch-lm")]
pub fn run_fused_eval(
    prompts: &[FairPrompt],
    corpus: &[String],
    cfg: &PatchLmConfig,
) -> Result<FusedEvalOutput, String> {
    let mut fused_cfg = cfg.clone();
    fused_cfg.pipeline_v4 = true;
    fused_cfg.mcq_decode = McqDecodeMode::LogprobBatch;

    eprintln!(
        "fused_lm: {} prompts · body={} · strength={} · mcq=logprob_batch · no hybrid router",
        prompts.len(),
        fused_cfg.patch_body,
        fused_cfg.strength,
    );

    let patch_rows = run_patch_track_v4(prompts, corpus, &fused_cfg, None)?;
    let fused_rows = patch_rows_to_fused(&patch_rows, prompts);
    let _slices = classify_slices(prompts, corpus);

    Ok(FusedEvalOutput {
        fused_product_version: FUSED_PRODUCT_VERSION.to_string(),
        fair_benchmark_version: FAIR_BENCHMARK_VERSION.to_string(),
        protocol: "single_lm_forward_v4_logprob_mcq".into(),
        summary: summarize_rows(&fused_rows),
        fused_rows,
    })
}
