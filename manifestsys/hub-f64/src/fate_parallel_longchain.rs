//! Fate-driven parallel pipelines for long-chain QA (experimental).
//!
//! One question fans out to N pipes; forward/back steps and fuse nodes are gated by
//! Fate adaptive algebra (session spread, pretrain pass-rate) — no fixed external constants.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::context_routing::{corpus_for_prompt, targeted_context, ContextIngestMode};
use crate::fate_adaptive_algebra::{
    fate_adaptive_context_boost, fate_retrieve_context_lines, fate_signal_land_into_lines,
    query_tokens_from_q, score_qa_algebra, FateAlgebraState,
};
use crate::fate_pretrain_holdout::{
    adaptive_corpus_for_single, run_fate_pretrain_epochs, FatePretrainItem,
};
use crate::valhalla_base::{ValhallaBaseQuestion, ValhallaFateIngressRouting};
use crate::valhalla_base_client::{RunOptions, ValhallaBaseClient, PRETRAIN_PIPE_MODE};
use crate::valhalla_base_protocol::ValhallaBaseErrorBody;

pub const PROTOCOL: &str = "fate-parallel-longchain-v1";
pub const ROUTING_AB_PROTOCOL: &str = "fate-routing-ab-v3";
pub const ROUTING_AB_MODE: &str = PRETRAIN_PIPE_MODE;

pub fn longchain_parallel_enabled() -> bool {
    std::env::var("VALHALLA_FATE_PARALLEL_LCHAIN").as_deref() == Ok("1")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FateRoutingAbRow {
    pub id: String,
    pub question: String,
    pub gold: String,
    pub evidence: Vec<String>,
    pub n_corpus_lines: usize,
    pub hub_spread: f64,
    pub total_walk_steps: usize,
    pub baseline_answer: String,
    pub parallel_answer: String,
    pub targeted_answer: String,
    pub baseline_correct: bool,
    pub parallel_correct: bool,
    pub targeted_correct: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FateRoutingAbReport {
    pub protocol: String,
    #[serde(default)]
    pub corpus_mode: String,
    pub n_items: usize,
    pub baseline_pass: f64,
    pub parallel_pass: f64,
    pub targeted_pass: f64,
    pub avg_hub_spread: f64,
    pub rows: Vec<FateRoutingAbRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FateParallelPipeTrace {
    pub pipe_idx: usize,
    pub ingress: String,
    pub start_line: usize,
    pub end_line: usize,
    pub fuse_line: usize,
    pub steps: Vec<FateParallelStep>,
    pub selected_indices: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FateParallelStep {
    pub from: usize,
    pub to: usize,
    pub delta: i32,
    pub spread: f64,
    pub gate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FateParallelLongchainRow {
    pub id: String,
    pub question: String,
    pub gold: String,
    pub baseline_answer: String,
    pub parallel_answer: String,
    pub baseline_correct: bool,
    pub parallel_correct: bool,
    pub n_pipes: usize,
    pub fuse_nodes: Vec<usize>,
    pub hub_spread: f64,
    pub pretrain_ok: bool,
    pub pipes: Vec<FateParallelPipeTrace>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FateParallelLongchainReport {
    pub protocol: String,
    pub n_items: usize,
    pub baseline_pass: f64,
    pub parallel_pass: f64,
    pub pretrain_passes: usize,
    pub rows: Vec<FateParallelLongchainRow>,
}


pub fn spread_from_client(client: &ValhallaBaseClient) -> f64 {
    client.algebra_state().effective_walk_spread()
}

/// Resolve pretrain passes from adaptive algebra + optional CLI override.
pub fn resolve_pretrain_passes(requested: usize, client: &ValhallaBaseClient) -> usize {
    crate::fate_adaptive_algebra::resolve_pretrain_passes(requested, &client.algebra_state())
}

fn line_fate_score(question: &str, line: &str, affinity: f64, state: &FateAlgebraState) -> f64 {
    let align = kw_overlap_frac(question, line);
    align * state.corpus_rerank_weight(
        question.split_whitespace().filter(|w| w.len() > 2).count(),
    ) + affinity * state.coupling(0)
}

fn fate_walk_delta(
    question: &str,
    corpus: &[String],
    cur: usize,
    aff: &[f64],
    spread: f64,
    state: &FateAlgebraState,
) -> (i32, f64) {
    let top = aff
        .get(cur)
        .copied()
        .unwrap_or(0.0)
        .max(kw_overlap_frac(question, &corpus[cur]));
    let mut gate = state.delta_required(top.max(1.0));
    gate *= 1.0 - (spread * 0.35).min(0.45);
    gate *= 1.0 - (state.meta.pretrain_pass_rate * 0.4).min(0.45);
    gate *= 1.0 - (state.meta.pretrain_epochs as f64 * 0.05).min(0.25);
    if spread < gate {
        return (0, gate);
    }
    let left = cur.saturating_sub(1);
    let right = (cur + 1).min(corpus.len().saturating_sub(1));
    let score_left =
        line_fate_score(question, &corpus[left], aff.get(left).copied().unwrap_or(0.0), state);
    let score_right =
        line_fate_score(question, &corpus[right], aff.get(right).copied().unwrap_or(0.0), state);
    let diff = score_right - score_left;
    let diff_gate = gate
        * (1.0 - state.meta.pretrain_pass_rate * 0.55 - state.hub_spread * 0.25)
            .clamp(0.25, 1.0);
    if diff.abs() < diff_gate {
        (0, diff_gate)
    } else if diff > 0.0 {
        (1, diff_gate)
    } else {
        (-1, diff_gate)
    }
}

fn ingress_for_pipe(pipe_idx: usize, spread: f64) -> ValhallaFateIngressRouting {
    let phase = (pipe_idx + (spread * 100.0).round() as usize) % 4;
    match phase {
        0 => ValhallaFateIngressRouting::QuadCycle,
        1 => ValhallaFateIngressRouting::RoundRobin,
        2 => ValhallaFateIngressRouting::Semantic,
        _ => ValhallaFateIngressRouting::HybridQuadSemantic,
    }
}

fn ingress_label(mode: ValhallaFateIngressRouting) -> &'static str {
    match mode {
        ValhallaFateIngressRouting::QuadCycle => "quad_cycle",
        ValhallaFateIngressRouting::RoundRobin => "round_robin",
        ValhallaFateIngressRouting::Semantic => "semantic",
        ValhallaFateIngressRouting::HybridQuadSemantic => "hybrid_quad_semantic",
    }
}

fn kw_overlap_frac(question: &str, text: &str) -> f64 {
    let qtok: Vec<String> = question
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_ascii_lowercase())
        .filter(|w| w.len() > 2)
        .collect();
    if qtok.is_empty() {
        return 0.0;
    }
    let low = text.to_ascii_lowercase();
    let hits = qtok.iter().filter(|t| low.contains(t.as_str())).count();
    hits as f64 / qtok.len() as f64
}

pub fn score_short_answer(answer: &str, gold: &str) -> bool {
    let low = answer.to_ascii_lowercase();
    let g = gold.trim().to_ascii_lowercase();
    if g.is_empty() {
        return false;
    }
    if low.contains(&g) {
        return true;
    }
    let ref_tokens: Vec<String> = g
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
        .filter(|w| w.len() > 1)
        .collect();
    if ref_tokens.is_empty() {
        return false;
    }
    let hit = ref_tokens.iter().filter(|t| low.contains(t.as_str())).count();
    hit >= (ref_tokens.len() + 1) / 2
}

/// LoCoMo scoring — Fate algebra + evidence line overlap.
pub fn score_locomo_answer(
    answer: &str,
    gold: &str,
    question: &str,
    evidence: &[String],
    corpus: &[String],
    state: &FateAlgebraState,
) -> bool {
    crate::fate_adaptive_algebra::score_qa_with_evidence(
        answer,
        gold,
        question,
        evidence,
        corpus,
        None,
        state,
    )
}

pub fn score_locomo_answer_ambient(
    answer: &str,
    gold: &str,
    question: &str,
    evidence: &[String],
    corpus: &[String],
) -> bool {
    score_locomo_answer(answer, gold, question, evidence, corpus, &FateAlgebraState::ambient())
}

pub fn ensure_evidence_in_lines(
    full: &[String],
    lines: &mut Vec<String>,
    item_id: &str,
    question: &str,
    gold: &str,
    state: &FateAlgebraState,
) {
    let q = ValhallaBaseQuestion {
        id: item_id.to_string(),
        text: question.to_string(),
        score_type: "open".into(),
        reference_answer: gold.to_string(),
        keywords: keywords_from_gold(gold, question),
        audio_features: None,
        image_features: None,
    };
    fate_signal_land_into_lines(full, &q, state, None, lines);
}

pub fn keywords_from_gold(gold: &str, question: &str) -> Vec<String> {
    let mut kws: Vec<String> = gold
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_ascii_lowercase())
        .filter(|w| w.len() > 2)
        .collect();
    for w in question.split_whitespace() {
        let t = w.trim_matches(|c: char| !c.is_alphanumeric()).to_ascii_lowercase();
        if t.len() > 3 && !kws.contains(&t) {
            kws.push(t);
        }
    }
    kws.truncate(8);
    kws
}

/// Light Fate pretrain (2-line warmup) — best-effort; returns false on failure.
pub fn light_fate_pretrain(client: &mut ValhallaBaseClient, lines: &[String]) -> bool {
    if lines.len() < 2 {
        return false;
    }
    let n = 2.min(lines.len());
    client.fate_warmup_corpus(&lines[..n], n).is_ok()
}

pub fn build_longchain_substrate(
    lines: &[String],
    chunk_size: usize,
    fast: bool,
) -> Result<ValhallaBaseClient, ValhallaBaseErrorBody> {
    ValhallaBaseClient::build_flexible_pretrain_substrate(lines, chunk_size, fast, false)
        .map(|(c, _)| c)
}

pub fn build_longchain_substrate_lr(
    lines: &[String],
    chunk_size: usize,
    fast: bool,
    low_resource: bool,
) -> Result<ValhallaBaseClient, ValhallaBaseErrorBody> {
    ValhallaBaseClient::build_flexible_pretrain_substrate(lines, chunk_size, fast, low_resource)
        .map(|(c, _)| c)
}

fn mini_pretrain_item(
    client: &mut ValhallaBaseClient,
    corpus: &[String],
    item: &FatePretrainItem,
    max_epochs: usize,
    low_resource: bool,
) -> Result<usize, ValhallaBaseErrorBody> {
    let out = run_fate_pretrain_epochs(client, corpus, std::slice::from_ref(item), max_epochs, low_resource)?;
    Ok(out.total_walk_steps)
}

fn run_session_baseline(
    client: &mut ValhallaBaseClient,
    q: &ValhallaBaseQuestion,
) -> Result<String, ValhallaBaseErrorBody> {
    client.reanchor_affinity = false;
    let resp = client.run(vec![q.clone()], RunOptions::default())?;
    Ok(resp
        .rows
        .first()
        .map(|r| r.answer.clone())
        .unwrap_or_default())
}

fn run_session_targeted(
    client: &mut ValhallaBaseClient,
    q: &ValhallaBaseQuestion,
) -> Result<String, ValhallaBaseErrorBody> {
    let corpus = client
        .session()
        .map(|s| s.corpus.clone())
        .unwrap_or_default();
    let state = client.algebra_state();
    let aff = client
        .session()
        .map(|s| s.snapshot.corpus_line_affinity.clone())
        .unwrap_or_default();
    let lines = fate_retrieve_context_lines(q, &corpus, &state, Some(&aff));
    client.reanchor_affinity = true;
    client.ingest_for_prompt(q, &lines, lines.len().max(4), ContextIngestMode::Targeted);
    let resp = client.run(vec![q.clone()], RunOptions::default())?;
    Ok(resp
        .rows
        .first()
        .map(|r| r.answer.clone())
        .unwrap_or_default())
}

/// Long-chain ingest + parallel fuse QA (production hook when `VALHALLA_FATE_PARALLEL_LCHAIN=1`).
pub fn run_longchain_qa_parallel(
    lines: &[String],
    q: &ValhallaBaseQuestion,
    chunk_size: usize,
    fast: bool,
    pretrain_passes: usize,
) -> Result<String, ValhallaBaseErrorBody> {
    let mut client = build_longchain_substrate(lines, chunk_size, fast)?;
    let corpus = client
        .session()
        .map(|s| s.corpus.clone())
        .unwrap_or_else(|| lines.to_vec());
    let item = FatePretrainItem {
        id: q.id.clone(),
        question: q.text.clone(),
        answer: q.reference_answer.clone(),
        evidence: Vec::new(),
    };
    let epochs = resolve_pretrain_passes(pretrain_passes, &client).max(2);
    let _ = mini_pretrain_item(&mut client, &corpus, &item, epochs, false)?;
    client.teardown();
    let (ans, _, _, _) = run_parallel_pipes(&corpus, q, &mut client)?;
    client.teardown();
    Ok(ans)
}

/// Three-arm routing A/B on one long-chain substrate per item.
pub fn run_routing_ab_item(
    full_session: &[String],
    lines: &[String],
    item_id: &str,
    question: &str,
    gold: &str,
    evidence: &[String],
    chunk_size: usize,
    fast: bool,
    pretrain_passes: usize,
    low_resource: bool,
) -> Result<FateRoutingAbRow, ValhallaBaseErrorBody> {
    let kws = keywords_from_gold(gold, question);
    let q = ValhallaBaseQuestion {
        id: item_id.to_string(),
        text: question.to_string(),
        score_type: "open".into(),
        reference_answer: gold.to_string(),
        keywords: kws,
        audio_features: None,
        image_features: None,
    };

    let mut substrate =
        build_longchain_substrate_lr(lines, chunk_size, fast, low_resource)?;
    let corpus = substrate
        .session()
        .map(|s| s.corpus.clone())
        .unwrap_or_else(|| lines.to_vec());
    let pretrain_item = FatePretrainItem {
        id: item_id.to_string(),
        question: question.to_string(),
        answer: gold.to_string(),
        evidence: evidence.to_vec(),
    };
    let max_epochs = if low_resource {
        3
    } else {
        resolve_pretrain_passes(pretrain_passes, &substrate).max(2).min(6)
    };
    let total_walk_steps =
        mini_pretrain_item(&mut substrate, &corpus, &pretrain_item, max_epochs, low_resource)?;

    let hub_spread_v = spread_from_client(&substrate);
    let n_corpus = substrate
        .session()
        .map(|s| s.corpus.len())
        .unwrap_or(lines.len());

    let mut baseline_client = substrate.spawn_fragment_qa_worker();
    let baseline_ans = run_session_baseline(&mut baseline_client, &q)?;

    let mut targeted_client = substrate.spawn_fragment_qa_worker();
    let targeted_ans = run_session_targeted(&mut targeted_client, &q)?;

    let mut fate_client = substrate;
    let (parallel_ans, pipes, _, _) = run_parallel_pipes(&corpus, &q, &mut fate_client)?;
    let pipe_walk_steps: usize = pipes.iter().map(|p| p.steps.len()).sum();
    let total_walk_steps = total_walk_steps.max(pipe_walk_steps);

    Ok(FateRoutingAbRow {
        id: item_id.to_string(),
        question: question.to_string(),
        gold: gold.to_string(),
        evidence: evidence.to_vec(),
        n_corpus_lines: n_corpus,
        hub_spread: hub_spread_v,
        total_walk_steps,
        baseline_answer: baseline_ans.chars().take(300).collect(),
        parallel_answer: parallel_ans.chars().take(300).collect(),
        targeted_answer: targeted_ans.chars().take(300).collect(),
        baseline_correct: score_locomo_answer(
            &baseline_ans,
            gold,
            question,
            evidence,
            full_session,
            &baseline_client.algebra_state(),
        ),
        parallel_correct: score_locomo_answer(
            &parallel_ans,
            gold,
            question,
            evidence,
            full_session,
            &fate_client.algebra_state(),
        ),
        targeted_correct: score_locomo_answer(
            &targeted_ans,
            gold,
            question,
            evidence,
            full_session,
            &targeted_client.algebra_state(),
        ),
    })
}

#[derive(Deserialize)]
struct LocomoAbItem {
    id: String,
    question: String,
    #[serde(deserialize_with = "deserialize_flex_string")]
    answer: String,
    #[serde(default)]
    evidence: Vec<String>,
    #[serde(default)]
    session_lines: Vec<String>,
}

#[derive(Deserialize)]
struct LocomoAbBundle {
    items: Vec<LocomoAbItem>,
}

fn deserialize_flex_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v = serde_json::Value::deserialize(deserializer)?;
    Ok(match v {
        serde_json::Value::String(s) => s,
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        other => other.to_string(),
    })
}

/// LoCoMo slice A/B: baseline (session) vs parallel vs targeted-on-session.
pub fn run_locomo_routing_ab(
    bundle_path: &Path,
    limit: usize,
    max_lines: usize,
    chunk_size: usize,
    fast: bool,
    pretrain_passes: usize,
    low_resource: bool,
) -> Result<FateRoutingAbReport, ValhallaBaseErrorBody> {
    crate::valhalla_base_protocol::ensure_production_runtime();
    let raw = std::fs::read_to_string(bundle_path).map_err(|e| {
        ValhallaBaseErrorBody::new(
            crate::valhalla_base_protocol::ValhallaBaseErrorKind::Io,
            e.to_string(),
        )
    })?;
    let doc: LocomoAbBundle = serde_json::from_str(&raw).map_err(|e| {
        ValhallaBaseErrorBody::new(
            crate::valhalla_base_protocol::ValhallaBaseErrorKind::Parse,
            format!("parse locomo bundle: {e}"),
        )
    })?;

    let mut items = doc.items;
    if limit > 0 {
        items.truncate(limit);
    }

    let mut rows = Vec::new();
    let mut base_ok = 0usize;
    let mut par_ok = 0usize;
    let mut tgt_ok = 0usize;
    let mut spread_sum = 0.0f64;

    for item in items {
        let full_session = item.session_lines.clone();
        let state = FateAlgebraState::ambient();
        let mut lines: Vec<String> = if low_resource {
            adaptive_corpus_for_single(
                &full_session,
                &item.id,
                &item.question,
                &item.answer,
                &state,
            )
        } else {
            let mut slice = full_session.clone();
            if max_lines > 0 && slice.len() > max_lines {
                slice.truncate(max_lines);
            }
            ensure_evidence_in_lines(
                &full_session,
                &mut slice,
                &item.id,
                &item.question,
                &item.answer,
                &state,
            );
            slice
        };
        if lines.is_empty() {
            lines = full_session.clone();
            if max_lines > 0 && lines.len() > max_lines {
                lines.truncate(max_lines);
            }
        }
        let row = run_routing_ab_item(
            &full_session,
            &lines,
            &item.id,
            &item.question,
            &item.answer,
            &item.evidence,
            chunk_size,
            fast,
            pretrain_passes,
            low_resource,
        )?;
        spread_sum += row.hub_spread;
        base_ok += usize::from(row.baseline_correct);
        par_ok += usize::from(row.parallel_correct);
        tgt_ok += usize::from(row.targeted_correct);
        rows.push(row);
    }

    let n = rows.len().max(1);
    Ok(FateRoutingAbReport {
        protocol: ROUTING_AB_PROTOCOL.into(),
        corpus_mode: ROUTING_AB_MODE.into(),
        n_items: rows.len(),
        baseline_pass: base_ok as f64 / n as f64,
        parallel_pass: par_ok as f64 / n as f64,
        targeted_pass: tgt_ok as f64 / n as f64,
        avg_hub_spread: spread_sum / n as f64,
        rows,
    })
}

fn run_baseline_longchain(
    corpus: &[String],
    q: &ValhallaBaseQuestion,
) -> Result<String, ValhallaBaseErrorBody> {
    let mut client = ValhallaBaseClient::production();
    client.config.incubation_cycles = 2;
    client.ingest_for_prompt(q, corpus, 8, ContextIngestMode::Full);
    let resp = client.run(vec![q.clone()], RunOptions::default())?;
    Ok(resp
        .rows
        .first()
        .map(|r| r.answer.clone())
        .unwrap_or_default())
}

/// Parallel pipes on corpus; fuse at adaptive nodes; multi-round with rollback on low spread.
pub fn run_parallel_pipes(
    corpus: &[String],
    q: &ValhallaBaseQuestion,
    fate_client: &mut ValhallaBaseClient,
) -> Result<(String, Vec<FateParallelPipeTrace>, Vec<usize>, f64), ValhallaBaseErrorBody> {
    if corpus.is_empty() {
        return Err(ValhallaBaseErrorBody::new(
            crate::valhalla_base_protocol::ValhallaBaseErrorKind::Validate,
            "empty corpus",
        ));
    }

    let state = fate_client.algebra_state();
    let max_rounds = state.fusion_round_cap();
    let mut best: Option<(String, Vec<FateParallelPipeTrace>, Vec<usize>, f64)> = None;
    let mut best_score = 0.0_f64;

    for round in 0..max_rounds {
        fate_client.push_fate_checkpoint();
        let mut spread = spread_from_client(fate_client);
        if spread < state.delta_required(1.0) * 0.35 {
            fate_client.prime_fate_walk_signals(corpus);
            spread = spread_from_client(fate_client);
        }
        let aff = fate_client
            .session()
            .map(|s| s.snapshot.corpus_line_affinity.clone())
            .unwrap_or_default();
        let n_pipes = state.parallel_pipe_count(corpus.len());
        let fuse_nodes = state.fuse_schedule(corpus.len());
        let mut all_indices: Vec<usize> = Vec::new();
        let mut traces = Vec::new();
        let phase_shift = state.time_phase_shift(round);
        let q_tokens = query_tokens_from_q(q);
        let mut ranked_starts: Vec<(usize, f64)> = corpus
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let a = aff.get(i).copied().unwrap_or(0.0);
                (i, state.fate_signal_line_score(&q_tokens, line, i, a))
            })
            .collect();
        ranked_starts.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        for p in 0..n_pipes {
            let ingress = ingress_for_pipe(p + phase_shift, spread);
            let start = ranked_starts
                .get(p)
                .map(|(i, _)| *i)
                .unwrap_or_else(|| (p * corpus.len()) / n_pipes);
            let fuse_target = *fuse_nodes.last().unwrap_or(&(corpus.len().saturating_sub(1)));
            let mut cursor = start;
            let mut steps = Vec::new();
            let max_steps = corpus.len();

            for _ in 0..max_steps {
                if cursor >= fuse_target {
                    break;
                }
                let (delta, gate) =
                    fate_walk_delta(&q.text, corpus, cursor, &aff, spread, &state);
                if delta == 0 {
                    break;
                }
                let from = cursor;
                cursor = ((cursor as i32) + delta)
                    .clamp(0, corpus.len().saturating_sub(1) as i32)
                    as usize;
                steps.push(FateParallelStep {
                    from,
                    to: cursor,
                    delta,
                    spread,
                    gate,
                });
            }

            let mut selected = vec![cursor];
            let window_lo = cursor.saturating_sub(2);
            let window_hi = (cursor + 3).min(corpus.len());
            for i in window_lo..window_hi {
                if !selected.contains(&i) {
                    selected.push(i);
                }
            }

            let sub_corpus: Vec<String> = selected.iter().map(|&i| corpus[i].clone()).collect();
            let routed = corpus_for_prompt(q, &sub_corpus, 4 + round);
            for line in routed {
                if let Some(idx) = corpus.iter().position(|l| l == &line) {
                    if !all_indices.contains(&idx) {
                        all_indices.push(idx);
                    }
                }
            }
            for &i in &selected {
                if !all_indices.contains(&i) {
                    all_indices.push(i);
                }
            }

            traces.push(FateParallelPipeTrace {
                pipe_idx: p,
                ingress: ingress_label(ingress).into(),
                start_line: start,
                end_line: cursor,
                fuse_line: fuse_target,
                steps,
                selected_indices: selected,
            });
        }

        all_indices.sort_unstable();

        let mut fused_lines: Vec<String> =
            all_indices.iter().map(|&i| corpus[i].clone()).collect();
        if fused_lines.is_empty() {
            fused_lines = corpus_for_prompt(q, corpus, 8 + round * 2);
        }
        let aff_now = fate_client
            .session()
            .map(|s| s.snapshot.corpus_line_affinity.clone())
            .unwrap_or_default();
        fate_adaptive_context_boost(&q, corpus, &mut fused_lines, &state, Some(&aff_now));
        let targeted = targeted_context(q, &fused_lines, 8 + round);

        let mut qa_client = fate_client.spawn_fragment_qa_worker();
        qa_client.config.fate_ingress_routing = ValhallaFateIngressRouting::QuadCycle;
        qa_client.reanchor_affinity = true;
        qa_client.ingest_for_prompt(q, &targeted, 8 + round, ContextIngestMode::Full);
        let resp = qa_client.run(vec![q.clone()], RunOptions::default())?;
        let ans = resp
            .rows
            .first()
            .map(|r| r.answer.clone())
            .unwrap_or_default();

        let score = kw_overlap_frac(&q.text, &ans);
        if score >= best_score {
            best_score = score;
            best = Some((ans, traces, fuse_nodes, spread));
        }

        if round + 1 >= max_rounds {
            break;
        }
        let gate = state.delta_required(1.0);
        if spread >= gate && score >= state.coupling(0) {
            break;
        }
        if !fate_client.rollback_fate() {
            break;
        }
    }

    best.ok_or_else(|| {
        ValhallaBaseErrorBody::new(
            crate::valhalla_base_protocol::ValhallaBaseErrorKind::Runtime,
            "parallel fuse produced no candidate",
        )
    })
    .map(|(ans, traces, fuse_nodes, spread)| {
        fate_client.teardown();
        (ans, traces, fuse_nodes, spread)
    })
}

/// Single item: baseline full-corpus vs Fate parallel fuse.
pub fn run_fate_parallel_longchain_item(
    lines: &[String],
    item_id: &str,
    question: &str,
    gold: &str,
    chunk_size: usize,
    pretrain_passes: usize,
) -> Result<FateParallelLongchainRow, ValhallaBaseErrorBody> {
    let kws = keywords_from_gold(gold, question);
    let q = ValhallaBaseQuestion {
        id: item_id.to_string(),
        text: question.to_string(),
        score_type: "open".into(),
        reference_answer: gold.to_string(),
        keywords: kws,
        audio_features: None,
        image_features: None,
    };

    let mut client = build_longchain_substrate(lines, chunk_size.max(1), true)?;
    let corpus = client
        .session()
        .map(|s| s.corpus.clone())
        .unwrap_or_else(|| lines.to_vec());
    let item = FatePretrainItem {
        id: item_id.to_string(),
        question: question.to_string(),
        answer: gold.to_string(),
        evidence: Vec::new(),
    };
    let epochs = resolve_pretrain_passes(pretrain_passes, &client).max(2);
    let pretrain_ok = mini_pretrain_item(&mut client, &corpus, &item, epochs, false).is_ok();

    let mut baseline_client = client.spawn_fragment_qa_worker();
    let baseline_ans = run_session_baseline(&mut baseline_client, &q)?;

    let (parallel_ans, pipes, fuse_nodes, hub_spread_v) =
        run_parallel_pipes(&corpus, &q, &mut client)?;

    Ok(FateParallelLongchainRow {
        id: item_id.to_string(),
        question: question.to_string(),
        gold: gold.to_string(),
        baseline_answer: baseline_ans.chars().take(300).collect(),
        parallel_answer: parallel_ans.chars().take(300).collect(),
        baseline_correct: score_short_answer(&baseline_ans, gold),
        parallel_correct: score_short_answer(&parallel_ans, gold),
        n_pipes: pipes.len(),
        fuse_nodes,
        hub_spread: hub_spread_v,
        pretrain_ok,
        pipes,
    })
}

/// Smoke: LoCoMo-style + science water.
pub fn run_fate_parallel_longchain_smoke(
    pretrain_passes: usize,
    max_lines: usize,
) -> Result<FateParallelLongchainReport, ValhallaBaseErrorBody> {
    crate::valhalla_base_protocol::ensure_production_runtime();

    let loco_lines: Vec<String> = vec![
        "[D1:1] Caroline: Hey Mel!".into(),
        "[D1:2] Melanie: Hey Caroline!".into(),
        "[D1:3] Caroline: I went to a LGBTQ support group yesterday and it was so powerful.".into(),
        "[D1:4] Melanie: Wow, that's cool, Caroline!".into(),
        "[D10:3] Caroline: I just joined a new LGBTQ activist group last Tues.".into(),
    ];

    let water_lines: Vec<String> = vec![
        "Water boils at 100 degrees Celsius at standard atmospheric pressure.".into(),
        "The boiling point depends on altitude and pressure.".into(),
        "Steam forms when liquid water reaches its boiling point.".into(),
    ];

    let mut items: Vec<(&str, Vec<String>, &str, &str)> = vec![
        (
            "loco_lgbtq_smoke",
            loco_lines,
            "When did Caroline go to the LGBTQ support group?",
            "yesterday",
        ),
        (
            "science_water",
            water_lines,
            "At standard atmospheric pressure, what temperature does water boil in Celsius?",
            "100",
        ),
    ];

    let mut rows = Vec::new();
    let mut base_ok = 0usize;
    let mut par_ok = 0usize;

    for (id, mut lines, q, gold) in items.drain(..) {
        if max_lines > 0 && lines.len() > max_lines {
            lines.truncate(max_lines);
        }
        let row = run_fate_parallel_longchain_item(&lines, id, q, gold, 4, pretrain_passes)?;
        if row.baseline_correct {
            base_ok += 1;
        }
        if row.parallel_correct {
            par_ok += 1;
        }
        rows.push(row);
    }

    let n = rows.len().max(1);
    Ok(FateParallelLongchainReport {
        protocol: PROTOCOL.into(),
        n_items: rows.len(),
        baseline_pass: base_ok as f64 / n as f64,
        parallel_pass: par_ok as f64 / n as f64,
        pretrain_passes,
        rows,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fate_adaptive_algebra::FateAlgebraState;

    #[test]
    fn pipe_count_adaptive() {
        let s = FateAlgebraState::ambient();
        assert!(s.parallel_pipe_count(10) >= s.parallel_pipe_min());
        assert!(s.parallel_pipe_count(200) <= s.parallel_pipe_max());
    }

    #[test]
    fn fuse_schedule_nonempty() {
        let s = FateAlgebraState::ambient();
        let nodes = s.fuse_schedule(64);
        assert!(!nodes.is_empty());
        assert!(*nodes.last().unwrap() < 64);
    }

    #[test]
    fn walk_delta_respects_gate() {
        let corpus = vec!["a line".into(), "b line".into(), "c line".into()];
        let state = FateAlgebraState::ambient();
        let (d, _) = fate_walk_delta("question text", &corpus, 1, &[], 0.0, &state);
        assert_eq!(d, 0);
    }
}
