//! Hybrid Valhalla product engine — Rust-native Tier B + merge + optional patch rows.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::fair_score::{format_prompt_text, score_answer, extract_mcq_letter, FairPrompt, ScoreResult};
use crate::fair_suite::{classify_slices, FairSlices};
use crate::grinder::{default_port_cap, grind_text_hash};
use crate::native_qa::{
    run_native_qa_batch, NativeQaBatchItem, NativeQaBatchRequest, NativeQaQuestionSpec,
};

pub const HYBRID_PRODUCT_VERSION: &str = "hybrid-4.0-rust-pipeline";
pub const FAIR_BENCHMARK_VERSION: &str = "fair-1.2";

pub const MEMORY_TRIAD: &str = "triad_persistent";
pub const MEMORY_TILE_FATE: &str = "tile_fate_persistent";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MergeMode {
    Deploy,
    Oracle,
}

#[derive(Debug, Clone)]
pub struct HybridConfig {
    pub merge_mode: MergeMode,
    pub incubation_cycles: usize,
    pub qa_cycles: usize,
    pub port_cap: usize,
    pub pipeline_v4: bool,
}

impl Default for HybridConfig {
    fn default() -> Self {
        Self {
            merge_mode: MergeMode::Deploy,
            incubation_cycles: 1,
            qa_cycles: 1,
            port_cap: default_port_cap(),
            pipeline_v4: crate::pipeline::pipeline_v4_enabled(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryTrackRow {
    pub id: String,
    pub score_type: String,
    pub correct: bool,
    pub score: ScoreResult,
    pub native_answer: String,
    pub decode_path: String,
    pub memory_body: String,
    pub session: String,
    #[serde(default)]
    pub mcq_fate_cycles: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchTrackRow {
    pub id: String,
    #[serde(default)]
    pub assistant: Option<String>,
    #[serde(default)]
    pub native_answer: Option<String>,
    #[serde(default)]
    pub decode_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridRow {
    pub id: String,
    pub score_type: String,
    pub correct: bool,
    pub score: ScoreResult,
    pub hybrid_source: String,
    pub hybrid_policy: String,
    pub hybrid_merge_mode: String,
    pub hybrid_product_version: String,
    pub answer: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decode_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hybrid_native_path_rank: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeSummary {
    pub correct: usize,
    pub total: usize,
    pub acc: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackSummary {
    pub n: usize,
    pub correct: usize,
    pub acc: f64,
    pub by_score_type: HashMap<String, TypeSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridEvalOutput {
    pub hybrid_product_version: String,
    pub fair_benchmark_version: String,
    pub merge_mode: String,
    pub hybrid_rows: Vec<HybridRow>,
    pub memory_triad: Vec<MemoryTrackRow>,
    pub memory_tile_fate: Vec<MemoryTrackRow>,
    pub patch_rows: Vec<PatchTrackRow>,
    pub summary: HybridSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSummary {
    pub hybrid: TrackSummary,
    pub memory_triad_persistent: TrackSummary,
    pub memory_tile_fate_persistent: TrackSummary,
    pub patch_lm_subset: TrackSummary,
    pub hybrid_source_counts: HashMap<String, usize>,
}

pub fn load_patch_rows(path: &std::path::Path) -> Result<Vec<PatchTrackRow>, String> {
    let raw = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let v: serde_json::Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    if let Some(arr) = v.get("patch_rows").and_then(|x| x.as_array()) {
        return serde_json::from_value(serde_json::Value::Array(arr.clone()))
            .map_err(|e| e.to_string());
    }
    if let Some(results) = v.get("results").and_then(|x| x.as_array()) {
        for arm in results {
            let arm_name = arm
                .get("arm")
                .or_else(|| arm.get("arm_id"))
                .and_then(|a| a.as_str())
                .unwrap_or("");
            if arm_name.contains("patch") {
                if let Some(rows) = arm.get("rows") {
                    return serde_json::from_value(rows.clone()).map_err(|e| e.to_string());
                }
            }
        }
    }
    if v.is_array() {
        return serde_json::from_value(v).map_err(|e| e.to_string());
    }
    Err("patch JSON: expected patch_rows array or fair benchmark patch arm".into())
}

fn hybrid_route(prompt: &FairPrompt, slices: &FairSlices, pipeline_v4: bool) -> &'static str {
    let st = prompt.score_type.as_str();
    if st == "open" || slices.open_generation.contains(&prompt.id) {
        return "patch_lm";
    }
    if st == "mcq" {
        if pipeline_v4 {
            return "memory_mcq_dual";
        }
        return "memory_mcq_best";
    }
    if st == "numeric" {
        return "numeric_confidence";
    }
    "memory_triad"
}

fn patch_prompt_ids(
    prompts: &[FairPrompt],
    slices: &FairSlices,
    pipeline_v4: bool,
) -> HashSet<String> {
    prompts
        .iter()
        .filter(|p| {
            matches!(
                hybrid_route(p, slices, pipeline_v4),
                "patch_lm" | "numeric_confidence" | "memory_mcq_dual"
            )
        })
        .map(|p| p.id.clone())
        .collect()
}

fn native_path_rank(decode_path: &str) -> i32 {
    match decode_path {
        "direct_math" => 100,
        "word_problem" => 95,
        "cortex" => 40,
        p if p.starts_with("patch_pipeline") => 98,
        p if p.starts_with("pipeline_v4") => 96,
        p if p.starts_with("patch") => 35,
        p if p.starts_with("memory") => 25,
        "none" => 0,
        _ => 15,
    }
}

fn batch_item_to_memory_row(item: &NativeQaBatchItem, body: &str, prompt: &FairPrompt) -> MemoryTrackRow {
    let sc = score_answer(&item.answer, prompt);
    MemoryTrackRow {
        id: item.id.clone(),
        score_type: item.score_type.clone(),
        correct: sc.correct,
        score: sc,
        native_answer: item.answer.clone(),
        decode_path: item.decode_path.clone(),
        memory_body: body.to_string(),
        session: "persistent".into(),
        mcq_fate_cycles: item.mcq_fate_cycles,
    }
}

pub fn run_memory_track(
    body: &str,
    corpus: &[String],
    prompts: &[FairPrompt],
    cfg: &HybridConfig,
    mcq_fate: bool,
) -> Result<Vec<MemoryTrackRow>, String> {
    let port_cap = cfg.port_cap;
    let corpus_ports: Vec<_> = corpus
        .iter()
        .map(|line| grind_text_hash(line, port_cap))
        .collect();
    let questions: Vec<NativeQaQuestionSpec> = prompts
        .iter()
        .map(|p| {
            let text = format_prompt_text(p);
            NativeQaQuestionSpec {
                question: text.clone(),
                score_type: p.score_type.clone(),
                question_ports: Some(grind_text_hash(&text, port_cap)),
                id: Some(p.id.clone()),
            }
        })
        .collect();

    let req = NativeQaBatchRequest {
        body: body.to_string(),
        corpus: corpus.to_vec(),
        corpus_ports,
        questions,
        incubation_cycles: cfg.incubation_cycles,
        qa_cycles: cfg.qa_cycles,
        ingress_mode: Some("grinder".into()),
        mcq_fate,
        pipeline_v4: cfg.pipeline_v4,
    };

    let resp = run_native_qa_batch(&req);
    let by_id: HashMap<_, _> = resp.results.iter().map(|r| (r.id.as_str(), r)).collect();
    prompts
        .iter()
        .map(|p| {
            let item = by_id
                .get(p.id.as_str())
                .ok_or_else(|| format!("missing batch result for {}", p.id))?;
            Ok(batch_item_to_memory_row(item, body, p))
        })
        .collect()
}

fn answer_from_patch(row: &PatchTrackRow) -> String {
    row.assistant
        .clone()
        .or_else(|| row.native_answer.clone())
        .unwrap_or_default()
}

fn pick_preferred<'a>(
    candidates: &[(&'static str, Option<&'a MemoryTrackRow>)],
    prefer: &[&'static str],
) -> (&'static str, Option<&'a MemoryTrackRow>) {
    let map: HashMap<&str, &MemoryTrackRow> = candidates
        .iter()
        .filter_map(|(k, v)| v.map(|r| (*k, r)))
        .collect();
    for key in prefer {
        if let Some(r) = map.get(key) {
            return (key, Some(*r));
        }
    }
    for (k, v) in candidates {
        if v.is_some() {
            return (k, *v);
        }
    }
    ("none", None)
}

fn pick_oracle<'a>(
    candidates: &[(&'static str, Option<&'a MemoryTrackRow>)],
    prefer: &[&'static str],
) -> (&'static str, Option<&'a MemoryTrackRow>) {
    let map: HashMap<&str, &MemoryTrackRow> = candidates
        .iter()
        .filter_map(|(k, v)| v.map(|r| (*k, r)))
        .collect();
    for key in prefer {
        if let Some(r) = map.get(key) {
            if r.correct {
                return (key, Some(*r));
            }
        }
    }
    pick_preferred(candidates, prefer)
}

fn pick_numeric_deploy<'a>(
    tile: Option<&'a MemoryTrackRow>,
    triad: Option<&'a MemoryTrackRow>,
    patch: Option<&'a PatchTrackRow>,
) -> (&'static str, String, Option<String>) {
    let natives: Vec<(&str, &MemoryTrackRow)> = [
        (MEMORY_TILE_FATE, tile),
        (MEMORY_TRIAD, triad),
    ]
    .into_iter()
    .filter_map(|(k, v)| v.map(|r| (k, r)))
    .collect();

    if !natives.is_empty() {
        let (key, row) = natives
            .iter()
            .max_by_key(|(k, r)| (native_path_rank(&r.decode_path), *k == MEMORY_TILE_FATE))
            .unwrap();
        if native_path_rank(&row.decode_path) >= 95 {
            return (
                *key,
                row.native_answer.clone(),
                Some(row.decode_path.clone()),
            );
        }
    }

    if let Some(p) = patch {
        return (
            "patch_lm",
            answer_from_patch(p),
            p.decode_path.clone(),
        );
    }

    if let Some((key, row)) = natives.first() {
        return (
            *key,
            row.native_answer.clone(),
            Some(row.decode_path.clone()),
        );
    }

    ("none", String::new(), None)
}

fn pick_numeric_oracle<'a>(
    prompt: &FairPrompt,
    tile: Option<&'a MemoryTrackRow>,
    triad: Option<&'a MemoryTrackRow>,
    patch: Option<&'a PatchTrackRow>,
) -> (&'static str, String, Option<String>) {
    let order: [(&str, Option<&MemoryTrackRow>, Option<&PatchTrackRow>); 3] = [
        (MEMORY_TILE_FATE, tile, None),
        (MEMORY_TRIAD, triad, None),
        ("patch_lm", None, patch),
    ];
    for (key, mem, pr) in order {
        if let Some(r) = mem {
            if r.correct {
                return (
                    key,
                    r.native_answer.clone(),
                    Some(r.decode_path.clone()),
                );
            }
        }
        if key == "patch_lm" {
            if let Some(p) = pr {
                let ans = answer_from_patch(p);
                if score_answer(&ans, prompt).correct {
                    return ("patch_lm", ans, p.decode_path.clone());
                }
            }
        }
    }
    pick_numeric_deploy(tile, triad, patch)
}

fn pick_mcq_deploy<'a>(
    tile: Option<&'a MemoryTrackRow>,
    triad: Option<&'a MemoryTrackRow>,
    patch: Option<&'a PatchTrackRow>,
) -> (&'static str, String, Option<String>) {
    let mem_cands = [(MEMORY_TILE_FATE, tile), (MEMORY_TRIAD, triad)];
    let prefer = [MEMORY_TILE_FATE, MEMORY_TRIAD];
    let (mem_k, mem_r) = pick_preferred(&mem_cands, &prefer);

    if let (Some(pr), Some(mr)) = (patch, mem_r) {
        let patch_ans = answer_from_patch(pr);
        let mem_ans = mr.native_answer.clone();
        let same = extract_mcq_letter(&patch_ans) == extract_mcq_letter(&mem_ans);
        if same {
            return (mem_k, mem_ans, Some(mr.decode_path.clone()));
        }
        if mr.decode_path.contains("mcq") || mr.decode_path.contains("pipeline") {
            return (mem_k, mem_ans, Some(mr.decode_path.clone()));
        }
        return ("patch_lm", patch_ans, pr.decode_path.clone());
    }

    if let Some(pr) = patch {
        return (
            "patch_lm",
            answer_from_patch(pr),
            pr.decode_path.clone(),
        );
    }

    (
        mem_k,
        mem_r
            .map(|x| x.native_answer.clone())
            .unwrap_or_default(),
        mem_r.map(|x| x.decode_path.clone()),
    )
}

fn pick_mcq_oracle<'a>(
    prompt: &FairPrompt,
    tile: Option<&'a MemoryTrackRow>,
    triad: Option<&'a MemoryTrackRow>,
    patch: Option<&'a PatchTrackRow>,
) -> (&'static str, String, Option<String>) {
    let order: [(&str, Option<&MemoryTrackRow>, Option<&PatchTrackRow>); 3] = [
        (MEMORY_TILE_FATE, tile, None),
        (MEMORY_TRIAD, triad, None),
        ("patch_lm", None, patch),
    ];
    for (key, mem, pr) in order {
        if let Some(r) = mem {
            if r.correct {
                return (
                    key,
                    r.native_answer.clone(),
                    Some(r.decode_path.clone()),
                );
            }
        }
        if key == "patch_lm" {
            if let Some(p) = pr {
                let ans = answer_from_patch(p);
                if score_answer(&ans, prompt).correct {
                    return ("patch_lm", ans, p.decode_path.clone());
                }
            }
        }
    }
    pick_mcq_deploy(tile, triad, patch)
}

fn summarize_typed(items: Vec<(&str, bool)>) -> TrackSummary {
    let n = items.len();
    let correct = items.iter().filter(|(_, c)| *c).count();
    let mut by: HashMap<String, (usize, usize)> = HashMap::new();
    for (st, ok) in items {
        let e = by.entry(st.to_string()).or_insert((0, 0));
        e.1 += 1;
        if ok {
            e.0 += 1;
        }
    }
    let by_score_type = by
        .into_iter()
        .map(|(k, (c, t))| {
            (
                k,
                TypeSummary {
                    correct: c,
                    total: t,
                    acc: if t > 0 { c as f64 / t as f64 } else { 0.0 },
                },
            )
        })
        .collect();
    TrackSummary {
        n,
        correct,
        acc: if n > 0 { correct as f64 / n as f64 } else { 0.0 },
        by_score_type,
    }
}

fn summarize_rows(rows: &[HybridRow]) -> TrackSummary {
    summarize_typed(
        rows.iter()
            .map(|r| (r.score_type.as_str(), r.correct))
            .collect(),
    )
}

fn summarize_memory(rows: &[MemoryTrackRow]) -> TrackSummary {
    summarize_typed(
        rows.iter()
            .map(|r| (r.score_type.as_str(), r.correct))
            .collect(),
    )
}

pub fn build_hybrid_rows(
    prompts: &[FairPrompt],
    corpus: &[String],
    triad: &[MemoryTrackRow],
    tile_fate: &[MemoryTrackRow],
    patch_rows: &[PatchTrackRow],
    cfg: &HybridConfig,
) -> (Vec<HybridRow>, FairSlices) {
    let slices = classify_slices(prompts, corpus);
    let triad_map: HashMap<_, _> = triad.iter().map(|r| (r.id.as_str(), r)).collect();
    let tile_map: HashMap<_, _> = tile_fate.iter().map(|r| (r.id.as_str(), r)).collect();
    let patch_map: HashMap<_, _> = patch_rows.iter().map(|r| (r.id.as_str(), r)).collect();

    let merge_label = match cfg.merge_mode {
        MergeMode::Deploy => "deploy",
        MergeMode::Oracle => "oracle",
    };

    let mut out = Vec::with_capacity(prompts.len());
    for p in prompts {
        let policy = hybrid_route(p, &slices, cfg.pipeline_v4);
        let triad_r = triad_map.get(p.id.as_str()).copied();
        let tile_r = tile_map.get(p.id.as_str()).copied();
        let patch_r = patch_map.get(p.id.as_str()).copied();

        let (src, answer, decode_path, path_rank) = match policy {
            "patch_lm" => {
                if let Some(pr) = patch_r {
                    (
                        "patch_lm",
                        answer_from_patch(pr),
                        pr.decode_path.clone(),
                        None,
                    )
                } else if let Some(t) = tile_r {
                    (
                        MEMORY_TILE_FATE,
                        t.native_answer.clone(),
                        Some(t.decode_path.clone()),
                        Some(native_path_rank(&t.decode_path)),
                    )
                } else if let Some(t) = triad_r {
                    (
                        MEMORY_TRIAD,
                        t.native_answer.clone(),
                        Some(t.decode_path.clone()),
                        Some(native_path_rank(&t.decode_path)),
                    )
                } else {
                    ("none", String::new(), None, None)
                }
            }
            "memory_mcq_best" => {
                let cands = [(MEMORY_TILE_FATE, tile_r), (MEMORY_TRIAD, triad_r)];
                let prefer = [MEMORY_TILE_FATE, MEMORY_TRIAD];
                let (k, r) = match cfg.merge_mode {
                    MergeMode::Deploy => pick_preferred(&cands, &prefer),
                    MergeMode::Oracle => pick_oracle(&cands, &prefer),
                };
                (
                    k,
                    r.map(|x| x.native_answer.clone()).unwrap_or_default(),
                    r.map(|x| x.decode_path.clone()),
                    None,
                )
            }
            "memory_mcq_dual" => {
                let (k, ans, dp) = match cfg.merge_mode {
                    MergeMode::Deploy => pick_mcq_deploy(tile_r, triad_r, patch_r),
                    MergeMode::Oracle => pick_mcq_oracle(p, tile_r, triad_r, patch_r),
                };
                (k, ans, dp, None)
            }
            "numeric_confidence" => {
                let (k, ans, dp) = match cfg.merge_mode {
                    MergeMode::Deploy => pick_numeric_deploy(tile_r, triad_r, patch_r),
                    MergeMode::Oracle => pick_numeric_oracle(p, tile_r, triad_r, patch_r),
                };
                let rank = dp.as_ref().map(|d| native_path_rank(d));
                (k, ans, dp, rank)
            }
            _ => {
                let cands = [(MEMORY_TRIAD, triad_r), (MEMORY_TILE_FATE, tile_r)];
                let prefer = [MEMORY_TRIAD, MEMORY_TILE_FATE];
                let (k, r) = match cfg.merge_mode {
                    MergeMode::Deploy => pick_preferred(&cands, &prefer),
                    MergeMode::Oracle => pick_oracle(&cands, &prefer),
                };
                (
                    k,
                    r.map(|x| x.native_answer.clone()).unwrap_or_default(),
                    r.map(|x| x.decode_path.clone()),
                    None,
                )
            }
        };

        let sc = score_answer(&answer, p);
        out.push(HybridRow {
            id: p.id.clone(),
            score_type: p.score_type.clone(),
            correct: sc.correct,
            score: sc,
            hybrid_source: src.to_string(),
            hybrid_policy: policy.to_string(),
            hybrid_merge_mode: merge_label.to_string(),
            hybrid_product_version: HYBRID_PRODUCT_VERSION.to_string(),
            answer,
            decode_path,
            hybrid_native_path_rank: path_rank,
        });
    }
    (out, slices)
}

pub fn run_hybrid_eval(
    prompts: &[FairPrompt],
    corpus: &[String],
    patch_rows: &[PatchTrackRow],
    cfg: &HybridConfig,
) -> Result<HybridEvalOutput, String> {
    let triad = run_memory_track("triad", corpus, prompts, cfg, false)?;
    let tile_fate = run_memory_track("tile", corpus, prompts, cfg, true)?;
    let slices = classify_slices(prompts, corpus);
    let (hybrid_rows, _) =
        build_hybrid_rows(prompts, corpus, &triad, &tile_fate, patch_rows, cfg);

    let patch_ids = patch_prompt_ids(prompts, &slices, cfg.pipeline_v4);
    let patch_subset: Vec<PatchTrackRow> = patch_rows
        .iter()
        .filter(|r| patch_ids.contains(&r.id))
        .cloned()
        .collect();

    let mut src_counts: HashMap<String, usize> = HashMap::new();
    for r in &hybrid_rows {
        *src_counts.entry(r.hybrid_source.clone()).or_insert(0) += 1;
    }

    let id_to_prompt: HashMap<_, _> = prompts.iter().map(|p| (p.id.as_str(), p)).collect();
    let patch_scored: Vec<HybridRow> = patch_subset
        .iter()
        .filter_map(|pr| {
            id_to_prompt.get(pr.id.as_str()).map(|p| {
                let ans = answer_from_patch(pr);
                let sc = score_answer(&ans, p);
                HybridRow {
                    id: p.id.clone(),
                    score_type: p.score_type.clone(),
                    correct: sc.correct,
                    score: sc,
                    hybrid_source: "patch_lm".into(),
                    hybrid_policy: "patch_lm".into(),
                    hybrid_merge_mode: "patch_only".into(),
                    hybrid_product_version: HYBRID_PRODUCT_VERSION.to_string(),
                    answer: ans,
                    decode_path: pr.decode_path.clone(),
                    hybrid_native_path_rank: None,
                }
            })
        })
        .collect();

    Ok(HybridEvalOutput {
        hybrid_product_version: HYBRID_PRODUCT_VERSION.to_string(),
        fair_benchmark_version: FAIR_BENCHMARK_VERSION.to_string(),
        merge_mode: match cfg.merge_mode {
            MergeMode::Deploy => "deploy".into(),
            MergeMode::Oracle => "oracle".into(),
        },
        summary: HybridSummary {
            hybrid: summarize_rows(&hybrid_rows),
            memory_triad_persistent: summarize_memory(&triad),
            memory_tile_fate_persistent: summarize_memory(&tile_fate),
            patch_lm_subset: summarize_rows(&patch_scored),
            hybrid_source_counts: src_counts,
        },
        hybrid_rows,
        memory_triad: triad,
        memory_tile_fate: tile_fate,
        patch_rows: patch_subset,
    })
}

/// End-to-end Rust hybrid: Tier B memory + Tier A patch LM + merge.
#[cfg(feature = "patch-lm")]
pub fn run_hybrid_eval_full(
    prompts: &[FairPrompt],
    corpus: &[String],
    cfg: &HybridConfig,
    patch_cfg: &crate::patch_lm::PatchLmConfig,
) -> Result<HybridEvalOutput, String> {
    let slices = classify_slices(prompts, corpus);
    let patch_ids = patch_prompt_ids(prompts, &slices, cfg.pipeline_v4);
    let patch_prompts: Vec<FairPrompt> = prompts
        .iter()
        .filter(|p| patch_ids.contains(&p.id))
        .cloned()
        .collect();
    eprintln!(
        "patch_lm: {} prompts on body={} strength={}",
        patch_prompts.len(),
        patch_cfg.patch_body,
        patch_cfg.strength
    );
    let patch_rows = crate::patch_lm::run_patch_track(&patch_prompts, corpus, patch_cfg, None)?;
    run_hybrid_eval(prompts, corpus, &patch_rows, cfg)
}
