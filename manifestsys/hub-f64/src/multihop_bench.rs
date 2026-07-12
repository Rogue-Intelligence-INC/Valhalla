//! MultiHop / LaRA benchmark runners — bundle I/O, parallel orchestration, JSON reports.
//!
//! Inference API: [`crate::open_qa_interface`]; ingress/egress: [`crate::fate_io`].

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;

use crate::elastic_resources::ElasticResourcePlan;
use crate::fate_io::{ensure_unified_runtime, fate_score_short_answer};
use crate::fate_orchestrator::{apply_eval_corpus_cap, dynamic_corpus_fragments_enabled};
use crate::open_qa_interface::{OpenQaFlags, OpenQaRequest, run_open_qa_scored};
use crate::valhalla_base_protocol::{ValhallaBaseErrorBody, ValhallaBaseErrorKind};

fn now_iso() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}

pub fn verdict_open_qa(acc: f64) -> &'static str {
    if acc >= 0.65 {
        "OPEN_QA_STRONG"
    } else if acc >= 0.40 {
        "OPEN_QA_VIABLE"
    } else {
        "OPEN_QA_DEVELOPING"
    }
}

fn verdict_rag_vs_lc(rag_acc: f64, _lc_acc: f64) -> &'static str {
    if rag_acc >= 0.65 {
        "STRONG"
    } else if rag_acc >= 0.4 {
        "COMPETITIVE"
    } else {
        "DEVELOPING"
    }
}

fn elastic_json(plan: &ElasticResourcePlan) -> serde_json::Value {
    serde_json::json!({
        "parallel_workers": plan.parallel_workers,
        "cuda_ready": plan.cuda_ready,
        "use_cpu": plan.use_cpu,
        "resource_percent": plan.budget.resource_percent,
        "max_items": plan.budget.max_items,
        "lm_cache_max": plan.lm_cache_max,
    })
}

#[derive(Deserialize)]
struct MultihopBundle {
    items: Vec<MultihopItem>,
}

#[derive(Deserialize, Clone)]
struct MultihopItem {
    id: String,
    question: String,
    answer: String,
    #[serde(default)]
    corpus: Vec<String>,
}

#[derive(Clone, Copy)]
struct MultihopNativeFlags {
    multihop_v2: bool,
    multihop_fate: bool,
}

impl From<MultihopNativeFlags> for OpenQaFlags {
    fn from(f: MultihopNativeFlags) -> Self {
        OpenQaFlags {
            multihop_v2: f.multihop_v2,
            open_fate: f.multihop_fate,
        }
    }
}

fn multihop_corpus(item: &MultihopItem, max_corpus_lines: usize) -> Vec<String> {
    apply_eval_corpus_cap(item.corpus.clone(), max_corpus_lines)
}

fn run_multihop_native_batch(
    item: &MultihopItem,
    corpus: &[String],
    fast: bool,
    flags: MultihopNativeFlags,
) -> crate::native_qa::NativeQaBatchItem {
    let mut req = OpenQaRequest::new(item.id.clone(), item.question.clone(), corpus.to_vec())
        .with_flags(flags.into());
    if fast {
        req = req.fast();
    }
    run_open_qa_scored(&req, &item.answer).row
}

fn multihop_result_row(
    item: &MultihopItem,
    row: &crate::native_qa::NativeQaBatchItem,
) -> (bool, serde_json::Value) {
    let ans = row.answer.as_str();
    let ok = fate_score_short_answer(ans, &item.answer);
    (
        ok,
        serde_json::json!({
            "id": item.id,
            "correct": ok,
            "answer": ans.chars().take(200).collect::<String>(),
            "decode_path": row.decode_path,
            "query_fragments": row.query_fragments,
            "query_fragment_scores": row.query_fragment_scores,
        }),
    )
}

fn run_multihop_items_parallel(
    items: &[MultihopItem],
    fast: bool,
    max_corpus_lines: usize,
    flags: MultihopNativeFlags,
) -> (usize, Vec<serde_json::Value>, ElasticResourcePlan) {
    let plan = ElasticResourcePlan::for_workload(items.len());
    let results = crate::elastic_resources::par_map_items(items, plan.parallel_workers, |item| {
        let corpus = multihop_corpus(item, max_corpus_lines);
        let row = run_multihop_native_batch(item, &corpus, fast, flags);
        multihop_result_row(item, &row)
    });
    let mut correct = 0usize;
    let mut rows = Vec::with_capacity(results.len());
    for (ok, json_row) in results {
        if ok {
            correct += 1;
        }
        rows.push(json_row);
    }
    (correct, rows, plan)
}

fn load_multihop_bundle(
    bundle_path: &Path,
    limit: usize,
) -> Result<Vec<MultihopItem>, ValhallaBaseErrorBody> {
    let raw = std::fs::read_to_string(bundle_path).map_err(|e| {
        ValhallaBaseErrorBody::new(
            ValhallaBaseErrorKind::Io,
            format!("read {}: {e}", bundle_path.display()),
        )
    })?;
    let doc: MultihopBundle = serde_json::from_str(&raw).map_err(|e| {
        ValhallaBaseErrorBody::new(
            ValhallaBaseErrorKind::Parse,
            format!("parse multihop bundle: {e}"),
        )
    })?;
    let mut items: Vec<_> = doc
        .items
        .into_iter()
        .filter(|i| !i.corpus.is_empty())
        .collect();
    if limit > 0 {
        items.truncate(limit);
    }
    Ok(items)
}

/// MultiHop-RAG slice25 — structure composition decode (multihop_v2).
pub fn run_multihop_rag(
    bundle_path: &Path,
    limit: usize,
    fast: bool,
    max_corpus_lines: usize,
) -> Result<serde_json::Value, ValhallaBaseErrorBody> {
    ElasticResourcePlan::ensure_cuda_runtime_env();
    let items = load_multihop_bundle(bundle_path, limit)?;
    let (correct, rows, plan) = run_multihop_items_parallel(
        &items,
        fast,
        max_corpus_lines,
        MultihopNativeFlags {
            multihop_v2: true,
            multihop_fate: false,
        },
    );
    let n = items.len().max(1);
    let acc = correct as f64 / n as f64;
    Ok(serde_json::json!({
        "timestamp": now_iso(),
        "protocol": "multihop-rag-v2",
        "benchmark": "MultiHop-RAG",
        "bundle": bundle_path.file_name().and_then(|s| s.to_str()).unwrap_or(""),
        "multihop_v2": true,
        "native_qa_batch": true,
        "dynamic_corpus_fragments": dynamic_corpus_fragments_enabled(),
        "elastic": elastic_json(&plan),
        "n": items.len(),
        "correct": correct,
        "accuracy": (acc * 10000.0).round() / 10000.0,
        "verdict": verdict_open_qa(acc),
        "rows": rows,
    }))
}

/// MultiHop-RAG — Fate hop-wise adaptation (E7-φ, no answer lexicon).
pub fn run_multihop_fate_rag(
    bundle_path: &Path,
    limit: usize,
    fast: bool,
    max_corpus_lines: usize,
) -> Result<serde_json::Value, ValhallaBaseErrorBody> {
    ElasticResourcePlan::ensure_cuda_runtime_env();
    std::env::set_var("VALHALLA_CORPUS_LINE_AFFINITY_WEIGHT", "algebra");
    let items = load_multihop_bundle(bundle_path, limit)?;
    let (correct, rows, plan) = run_multihop_items_parallel(
        &items,
        fast,
        max_corpus_lines,
        MultihopNativeFlags {
            multihop_v2: false,
            multihop_fate: true,
        },
    );
    let n = items.len().max(1);
    let acc = correct as f64 / n as f64;
    Ok(serde_json::json!({
        "timestamp": now_iso(),
        "protocol": "multihop-rag-fate",
        "benchmark": "MultiHop-RAG",
        "bundle": bundle_path.file_name().and_then(|s| s.to_str()).unwrap_or(""),
        "multihop_fate": true,
        "native_qa_batch": true,
        "dynamic_corpus_fragments": dynamic_corpus_fragments_enabled(),
        "elastic": elastic_json(&plan),
        "n": items.len(),
        "correct": correct,
        "accuracy": (acc * 10000.0).round() / 10000.0,
        "verdict": verdict_open_qa(acc),
        "rows": rows,
    }))
}

/// MultiHop-RAG — Fate adapt + v2 compose (unified fatewithv2 solution).
pub fn run_multihop_hybrid_rag(
    bundle_path: &Path,
    limit: usize,
    fast: bool,
    max_corpus_lines: usize,
) -> Result<serde_json::Value, ValhallaBaseErrorBody> {
    ensure_unified_runtime();
    std::env::set_var("VALHALLA_CORPUS_LINE_AFFINITY_WEIGHT", "algebra");
    let items = load_multihop_bundle(bundle_path, limit)?;
    let (correct, rows, plan) = run_multihop_items_parallel(
        &items,
        fast,
        max_corpus_lines,
        MultihopNativeFlags {
            multihop_v2: true,
            multihop_fate: true,
        },
    );
    let n = items.len().max(1);
    let acc = correct as f64 / n as f64;
    Ok(serde_json::json!({
        "timestamp": now_iso(),
        "protocol": "multihop-rag-fatewithv2",
        "unified_solution": true,
        "multihop_fate": true,
        "multihop_v2": true,
        "native_qa_batch": true,
        "dynamic_corpus_fragments": dynamic_corpus_fragments_enabled(),
        "elastic": elastic_json(&plan),
        "n": items.len(),
        "correct": correct,
        "accuracy": (acc * 10000.0).round() / 10000.0,
        "verdict": verdict_open_qa(acc),
        "rows": rows,
    }))
}

#[derive(Deserialize)]
struct LaraItem {
    id: String,
    question: String,
    answer: String,
    #[serde(default)]
    rag_corpus: Vec<String>,
}

#[derive(Deserialize)]
struct LaraBundle {
    #[serde(default)]
    source: String,
    items: Vec<LaraItem>,
}

/// LaRA RAG arm — fatewithv2 compose on rag_corpus.
pub fn run_lara_fatewithv2(
    bundle_path: &Path,
    limit: usize,
    fast: bool,
) -> Result<serde_json::Value, ValhallaBaseErrorBody> {
    ensure_unified_runtime();
    std::env::set_var("VALHALLA_CORPUS_LINE_AFFINITY_WEIGHT", "algebra");
    let raw = std::fs::read_to_string(bundle_path).map_err(|e| {
        ValhallaBaseErrorBody::new(
            ValhallaBaseErrorKind::Io,
            format!("read {}: {e}", bundle_path.display()),
        )
    })?;
    let doc: LaraBundle = serde_json::from_str(&raw).map_err(|e| {
        ValhallaBaseErrorBody::new(
            ValhallaBaseErrorKind::Parse,
            format!("parse lara bundle: {e}"),
        )
    })?;
    let mut items: Vec<_> = doc
        .items
        .into_iter()
        .filter(|i| !i.rag_corpus.is_empty())
        .collect();
    if limit > 0 {
        items.truncate(limit);
    }
    let mut rows = Vec::new();
    let mut correct = 0usize;
    for item in &items {
        let corpus = item.rag_corpus.clone();
        let pseudo = MultihopItem {
            id: item.id.clone(),
            question: item.question.clone(),
            answer: item.answer.clone(),
            corpus: corpus.clone(),
        };
        let row = run_multihop_native_batch(
            &pseudo,
            &corpus,
            fast,
            MultihopNativeFlags {
                multihop_v2: true,
                multihop_fate: true,
            },
        );
        let (ok, json_row) = multihop_result_row(&pseudo, &row);
        if ok {
            correct += 1;
        }
        rows.push(json_row);
    }
    let n = items.len().max(1);
    let acc = correct as f64 / n as f64;
    Ok(serde_json::json!({
        "timestamp": now_iso(),
        "protocol": "lara-fatewithv2-v1",
        "benchmark": "LaRA-style RAG vs LC",
        "bundle": bundle_path.file_name().and_then(|s| s.to_str()).unwrap_or(""),
        "fatewithv2": true,
        "multihop_v2": true,
        "multihop_fate": true,
        "native_qa_batch": true,
        "dynamic_corpus_fragments": dynamic_corpus_fragments_enabled(),
        "n": items.len(),
        "correct": correct,
        "accuracy": (acc * 10000.0).round() / 10000.0,
        "verdict": verdict_rag_vs_lc(acc, 0.0),
        "rows": rows,
    }))
}
