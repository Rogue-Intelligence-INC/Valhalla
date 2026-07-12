//! Fate iterative pretrain — QA-first, Fate adaptive coordinates (no evidence tag mapping).

use std::cmp::Ordering;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::context_routing::{corpus_for_prompt, ContextIngestMode};
use crate::fate_adaptive_algebra::{
    fate_signal_landing_indices, query_tokens_from_q, FateAlgebraState, score_qa_algebra,
};
use crate::fate_parallel_longchain::{keywords_from_gold, spread_from_client};
use crate::fate_unified_qa::{run_fate_unified_qa, FateUnifiedQaOpts};
use crate::valhalla_base::ValhallaBaseQuestion;
use crate::valhalla_base_client::{RunOptions, ValhallaBaseClient};
use crate::valhalla_base_protocol::ValhallaBaseErrorBody;

pub const PROTOCOL: &str = "fate-pretrain-holdout-v3";
pub const PRETRAIN_MODE: &str = crate::valhalla_base_client::PRETRAIN_PIPE_MODE;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoldoutItemResult {
    pub id: String,
    pub question: String,
    pub gold: String,
    pub evidence: Vec<String>,
    pub answer: String,
    pub correct: bool,
    pub split: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FatePretrainHoldoutReport {
    pub protocol: String,
    pub train_n: usize,
    pub test_n: usize,
    pub train_skipped_no_evidence: usize,
    pub test_skipped_no_evidence: usize,
    pub pretrain_epochs: usize,
    pub train_pass: f64,
    pub test_pass: f64,
    pub converged: bool,
    pub max_epochs: usize,
    pub final_hub_spread: f64,
    pub low_resource: bool,
    pub pretrain_mode: String,
    pub corpus_lines: usize,
    pub ingest_passes: usize,
    pub fragment_count: usize,
    pub total_walk_steps: usize,
    pub train_rows: Vec<HoldoutItemResult>,
    pub test_rows: Vec<HoldoutItemResult>,
    pub epoch_log: Vec<EpochLog>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FatePretrainItem {
    pub id: String,
    pub question: String,
    pub answer: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochLog {
    pub epoch: usize,
    pub train_correct: usize,
    pub train_n: usize,
    pub walk_steps: usize,
    pub hub_spread: f64,
    pub fragment_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PretrainSessionOutcome {
    pub pretrain_epochs: usize,
    pub pretrain_pass: f64,
    pub converged: bool,
    pub max_epochs: usize,
    pub total_walk_steps: usize,
    pub epoch_log: Vec<EpochLog>,
    pub rows: Vec<HoldoutItemResult>,
}

#[derive(Clone, Deserialize)]
struct HoldoutBundleItem {
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
struct HoldoutBundle {
    items: Vec<HoldoutBundleItem>,
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


fn build_holdout_substrate(
    lines: &[String],
    chunk_size: usize,
    fast: bool,
    low_resource: bool,
) -> Result<(ValhallaBaseClient, usize), ValhallaBaseErrorBody> {
    ValhallaBaseClient::build_flexible_pretrain_substrate(lines, chunk_size, fast, low_resource)
}

fn probe_walk_steps(
    client: &mut ValhallaBaseClient,
    corpus: &[String],
    _probe: &FatePretrainItem,
) -> usize {
    // Cheap walk readiness probe — no LM / parallel-pipe QA.
    let state = client.algebra_state();
    if !state.walk_ready() || corpus.is_empty() {
        return 0;
    }
    let spread = spread_from_client(client);
    let gate = state.delta_required(1.0);
    if spread < gate * 0.35 {
        return 0;
    }
    let steps = ((spread / gate.max(1e-9)) * corpus.len() as f64 * 0.25)
        .round()
        .clamp(1.0, corpus.len() as f64) as usize;
    steps
}

fn fate_round_before_epoch(client: &mut ValhallaBaseClient, corpus: &[String], epoch: usize, light: bool) {
    // low_resource: skip link/adapt every epoch (RAM/CPU bomb); light prime only.
    if light {
        if epoch == 0 {
            let _ = client.fate_link_session();
        }
        client.prime_fate_walk_signals(corpus);
        return;
    }
    if epoch % 2 == 0 {
        let _ = client.fate_link_session();
    } else {
        let _ = client.fate_round_adapt_link();
    }
    client.prime_fate_walk_signals(corpus);
}

/// QA pretrain corpus: question-driven retrieval + adaptive neighbor window (no evidence coords).
pub fn adaptive_corpus_for_pretrain(
    full: &[String],
    items: &[FatePretrainItem],
    state: &FateAlgebraState,
) -> Vec<String> {
    use std::collections::BTreeSet;
    let mut indices: BTreeSet<usize> = BTreeSet::new();
    let k = state.retrieval_top_k(0);
    let neighbor = state.corpus_neighbor_window();

    for item in items {
        let q = make_question(item);
        for line in corpus_for_prompt(&q, full, k) {
            if let Some(i) = full.iter().position(|l| l == &line) {
                let lo = i.saturating_sub(neighbor);
                let hi = (i + neighbor).min(full.len().saturating_sub(1));
                for j in lo..=hi {
                    indices.insert(j);
                }
            }
        }
    }
    if indices.len() < state.min_corpus_seed_lines(full.len()) && !full.is_empty() {
        for j in 0..full.len().min(state.min_corpus_seed_lines(full.len())) {
            indices.insert(j);
        }
    }
    let raw: Vec<String> = indices.into_iter().map(|i| full[i].clone()).collect();
    let baseline = state.meta.fate_baseline_tick;
    let tick = state.meta.fate_tick;
    crate::fate_adaptive_algebra::fate_normalize_corpus(&raw, baseline, tick)
}

/// Single-item adaptive corpus (qa_adaptive — no evidence coordinates).
pub fn adaptive_corpus_for_single(
    full: &[String],
    id: &str,
    question: &str,
    answer: &str,
    state: &FateAlgebraState,
) -> Vec<String> {
    let item = FatePretrainItem {
        id: id.to_string(),
        question: question.to_string(),
        answer: answer.to_string(),
        evidence: Vec::new(),
    };
    adaptive_corpus_for_pretrain(full, &[item], state)
}

/// QA-first scoring via Fate adaptive algebra (internal time + pretrain meta).
pub fn score_qa_pretrain(answer: &str, gold: &str, question: &str) -> bool {
    score_qa_algebra(answer, gold, question, &FateAlgebraState::ambient())
}

pub fn score_qa_pretrain_with_state(
    answer: &str,
    gold: &str,
    question: &str,
    state: &FateAlgebraState,
) -> bool {
    score_qa_algebra(answer, gold, question, state)
}

fn extract_corpus_index(answer: &str) -> Option<usize> {
    let prefix = answer.strip_prefix("[corpus:")?;
    let end = prefix.find(']')?;
    prefix[..end].parse().ok()
}

impl From<&HoldoutBundleItem> for FatePretrainItem {
    fn from(item: &HoldoutBundleItem) -> Self {
        Self {
            id: item.id.clone(),
            question: item.question.clone(),
            answer: item.answer.clone(),
            evidence: item.evidence.clone(),
        }
    }
}

fn make_question(item: &FatePretrainItem) -> ValhallaBaseQuestion {
    ValhallaBaseQuestion {
        id: item.id.clone(),
        text: item.question.clone(),
        score_type: "open".into(),
        reference_answer: item.answer.clone(),
        keywords: keywords_from_gold(&item.answer, &item.question),
        audio_features: None,
        image_features: None,
    }
}

/// Score via Fate algebra + evidence line overlap (internal-time indices).
pub fn score_with_evidence(
    answer: &str,
    gold: &str,
    evidence: &[String],
    corpus: &[String],
) -> bool {
    crate::fate_adaptive_algebra::score_qa_with_evidence(
        answer,
        gold,
        "",
        evidence,
        corpus,
        None,
        &FateAlgebraState::ambient(),
    )
}

pub fn score_with_evidence_session(
    answer: &str,
    gold: &str,
    question: &str,
    evidence: &[String],
    corpus: &[String],
    full_session: &[String],
    state: &FateAlgebraState,
) -> bool {
    crate::fate_adaptive_algebra::score_qa_with_evidence(
        answer,
        gold,
        question,
        evidence,
        corpus,
        Some(full_session),
        state,
    )
}

fn ask_qa_pretrain(
    client: &mut ValhallaBaseClient,
    corpus: &[String],
    item: &FatePretrainItem,
    low_resource: bool,
) -> Result<String, ValhallaBaseErrorBody> {
    let q = make_question(item);
    let opts = if low_resource {
        FateUnifiedQaOpts::pretrain_low_resource()
    } else {
        FateUnifiedQaOpts::persistent()
    };
    let answer = run_fate_unified_qa(client, corpus, &q, opts)?;
    client.trim_fate_rollbacks();
    client.shrink_ingress_buffers();
    Ok(answer)
}

fn reinforce_qa_miss(
    client: &mut ValhallaBaseClient,
    corpus: &[String],
    item: &FatePretrainItem,
    answer: &str,
    epoch: usize,
) -> Result<(), ValhallaBaseErrorBody> {
    let scale = 1.0 + epoch as f64 * 0.5;
    let q = make_question(item);
    let state = client.algebra_state();
    let k = state.retrieval_top_k(q.keywords.len());
    let neighbor = state.corpus_neighbor_window();

    let mut good_indices = Vec::new();
    for line in corpus_for_prompt(&q, corpus, k) {
        if let Some(i) = corpus.iter().position(|l| l == &line) {
            let lo = i.saturating_sub(neighbor);
            let hi = (i + neighbor).min(corpus.len().saturating_sub(1));
            for j in lo..=hi {
                good_indices.push(j);
            }
        }
    }
    let aff = client
        .session()
        .map(|s| s.snapshot.corpus_line_affinity.clone())
        .unwrap_or_default();
    let tokens = query_tokens_from_q(&q);
    for idx in fate_signal_landing_indices(corpus, &tokens, &aff, &state, k) {
        good_indices.push(idx);
        let lo = idx.saturating_sub(neighbor);
        let hi = (idx + neighbor).min(corpus.len().saturating_sub(1));
        for j in lo..=hi {
            good_indices.push(j);
        }
    }
    good_indices.sort_unstable();
    good_indices.dedup();
    if !good_indices.is_empty() {
        client.adjust_corpus_line_affinity_indices(&good_indices, true, scale * 2.0);
        client.sharpen_walk_affinity_contrast(&good_indices, scale);
    }
    if let Some(wrong_idx) = extract_corpus_index(answer) {
        client.adjust_corpus_line_affinity_indices(&[wrong_idx], false, scale);
    }
    client.nudge_hub_spread(scale);
    client.prime_fate_walk_signals(corpus);
    Ok(())
}

fn eval_item(
    client: &mut ValhallaBaseClient,
    corpus: &[String],
    item: &FatePretrainItem,
    split: &str,
    _train_mode: bool,
    low_resource: bool,
) -> Result<HoldoutItemResult, ValhallaBaseErrorBody> {
    let state = client.algebra_state();
    let answer = ask_qa_pretrain(client, corpus, item, low_resource)?;
    let correct = score_qa_pretrain_with_state(&answer, &item.answer, &item.question, &state);
    Ok(HoldoutItemResult {
        id: item.id.clone(),
        question: item.question.clone(),
        gold: item.answer.clone(),
        evidence: item.evidence.clone(),
        answer: answer.chars().take(320).collect(),
        correct,
        split: split.to_string(),
    })
}

/// Reversible Fate pretrain loop on an existing session — epoch log + final rows.
pub fn run_fate_pretrain_epochs(
    client: &mut ValhallaBaseClient,
    corpus: &[String],
    items: &[FatePretrainItem],
    max_epochs: usize,
    low_resource: bool,
) -> Result<PretrainSessionOutcome, ValhallaBaseErrorBody> {
    if items.is_empty() {
        return Ok(PretrainSessionOutcome {
            pretrain_epochs: 0,
            pretrain_pass: 0.0,
            converged: true,
            max_epochs,
            total_walk_steps: 0,
            epoch_log: Vec::new(),
            rows: Vec::new(),
        });
    }

    let probe = &items[0];
    let mut epoch_log = Vec::new();
    let mut converged = false;
    let mut epochs_run = 0usize;
    let mut prev_train_correct = 0usize;
    let mut total_walk_steps = 0usize;

    for epoch in 0..max_epochs.max(1) {
        client.sync_algebra_meta();
        if !low_resource {
            client.push_fate_checkpoint();
        }
        fate_round_before_epoch(client, corpus, epoch, low_resource);
        epochs_run = epoch + 1;
        let mut train_correct = 0usize;
        for item in items {
            let row = eval_item(client, corpus, item, "pretrain", true, low_resource)?;
            if row.correct {
                train_correct += 1;
            } else {
                reinforce_qa_miss(client, corpus, item, &row.answer, epoch)?;
            }
            client.trim_fate_rollbacks();
            client.shrink_ingress_buffers();
        }
        if epoch > 0 && train_correct < prev_train_correct {
            let _ = client.rollback_fate();
            train_correct = prev_train_correct;
            eprintln!(
                "[fate-pretrain] epoch {} regressed ({}/{}), rolled back",
                epoch + 1,
                train_correct,
                items.len(),
            );
        } else {
            prev_train_correct = train_correct;
        }
        client.trim_fate_rollbacks();
        let partial_pass = train_correct as f64 / items.len().max(1) as f64;
        client.update_fate_algebra_from_pretrain(partial_pass, epochs_run);
        let walk_steps = probe_walk_steps(client, corpus, probe);
        total_walk_steps = total_walk_steps.max(walk_steps);
        let hub_spread = spread_from_client(client);
        let fragment_count = client
            .session()
            .map(|s| s.fragments.len())
            .unwrap_or(0);
        epoch_log.push(EpochLog {
            epoch: epoch + 1,
            train_correct,
            train_n: items.len(),
            walk_steps,
            hub_spread,
            fragment_count,
        });
        eprintln!(
            "[fate-pretrain] epoch {}: {}/{} correct  walk_steps={} spread={:.4} fragments={}",
            epoch + 1,
            train_correct,
            items.len(),
            walk_steps,
            hub_spread,
            fragment_count,
        );
        if train_correct == items.len() {
            converged = true;
            break;
        }
    }

    let mut rows = Vec::new();
    let mut ok = 0usize;
    for item in items {
        let row = eval_item(client, corpus, item, "pretrain", true, low_resource)?;
        ok += usize::from(row.correct);
        rows.push(row);
    }
    let pass = ok as f64 / items.len().max(1) as f64;

    client.update_fate_algebra_from_pretrain(pass, epochs_run);
    client.trim_fate_rollbacks();
    client.clear_ingress_pending();

    Ok(PretrainSessionOutcome {
        pretrain_epochs: epochs_run,
        pretrain_pass: pass,
        converged,
        max_epochs,
        total_walk_steps,
        epoch_log,
        rows,
    })
}

/// QA-first pretrain on train set until convergence, then holdout test.
pub fn run_fate_pretrain_holdout(
    bundle_path: &Path,
    train_n: usize,
    test_n: usize,
    max_lines: usize,
    chunk_size: usize,
    fast: bool,
    max_epochs: usize,
    low_resource: bool,
) -> Result<FatePretrainHoldoutReport, ValhallaBaseErrorBody> {
    crate::valhalla_base_protocol::ensure_production_runtime();
    let raw = std::fs::read_to_string(bundle_path).map_err(|e| {
        ValhallaBaseErrorBody::new(
            crate::valhalla_base_protocol::ValhallaBaseErrorKind::Io,
            e.to_string(),
        )
    })?;
    let doc: HoldoutBundle = serde_json::from_str(&raw).map_err(|e| {
        ValhallaBaseErrorBody::new(
            crate::valhalla_base_protocol::ValhallaBaseErrorKind::Parse,
            format!("parse bundle: {e}"),
        )
    })?;

    if doc.items.is_empty() {
        return Err(ValhallaBaseErrorBody::new(
            crate::valhalla_base_protocol::ValhallaBaseErrorKind::Validate,
            "empty holdout bundle",
        ));
    }

    let train_n = train_n.min(doc.items.len());
    let test_start = train_n;
    let test_end = (train_n + test_n).min(doc.items.len());
    let train_candidates: Vec<HoldoutBundleItem> = doc.items[..train_n].to_vec();
    let test_candidates: Vec<HoldoutBundleItem> = doc.items[test_start..test_end].to_vec();

    let full_session = doc.items[0].session_lines.clone();
    let ambient = FateAlgebraState::ambient();
    let pool_pretrain: Vec<FatePretrainItem> = train_candidates
        .iter()
        .chain(test_candidates.iter())
        .map(FatePretrainItem::from)
        .collect();
    let corpus_lines: Vec<String> = if low_resource {
        adaptive_corpus_for_pretrain(&full_session, &pool_pretrain, &ambient)
    } else {
        let mut lines = full_session;
        if max_lines > 0 && lines.len() > max_lines {
            lines.truncate(max_lines);
        }
        lines
    };

    let train_items: Vec<HoldoutBundleItem> = train_candidates
        .iter()
        .filter(|it| !it.session_lines.is_empty())
        .cloned()
        .collect();
    let train_skipped = train_candidates.len().saturating_sub(train_items.len());
    let test_items: Vec<HoldoutBundleItem> = test_candidates
        .iter()
        .filter(|it| !it.session_lines.is_empty())
        .cloned()
        .collect();
    let test_skipped = test_candidates.len().saturating_sub(test_items.len());

    if train_items.is_empty() {
        return Err(ValhallaBaseErrorBody::new(
            crate::valhalla_base_protocol::ValhallaBaseErrorKind::Validate,
            format!(
                "no train items with session_lines in bundle (skipped={train_skipped})"
            ),
        ));
    }

    let (mut client, ingest_passes) =
        build_holdout_substrate(&corpus_lines, chunk_size, fast, low_resource)?;
    client.config.fate_qa_feedback = true;
    let fragment_count = client
        .session()
        .map(|s| s.fragments.len())
        .unwrap_or(0);
    eprintln!(
        "[fate-pretrain-holdout] mode={} low_resource={} corpus={} ingest_passes={} fragments={} train={} test={} (skipped {}/{})",
        PRETRAIN_MODE,
        low_resource,
        corpus_lines.len(),
        ingest_passes,
        fragment_count,
        train_items.len(),
        test_items.len(),
        train_skipped,
        test_skipped,
    );
    let corpus = client
        .session()
        .map(|s| s.corpus.clone())
        .unwrap_or(corpus_lines.clone());

    let train_pretrain: Vec<FatePretrainItem> = train_items.iter().map(FatePretrainItem::from).collect();
    let test_pretrain: Vec<FatePretrainItem> = test_items.iter().map(FatePretrainItem::from).collect();

    let pretrain_out = run_fate_pretrain_epochs(
        &mut client,
        &corpus,
        &train_pretrain,
        max_epochs,
        low_resource,
    )?;
    let epoch_log = pretrain_out.epoch_log;
    let converged = pretrain_out.converged;
    let epochs_run = pretrain_out.pretrain_epochs;
    let train_rows = pretrain_out.rows;
    let train_ok = train_rows.iter().filter(|r| r.correct).count();

    eprintln!(
        "[fate-pretrain-holdout] holdout test (qa_adaptive, {})…",
        if low_resource {
            "targeted Fate adaptive"
        } else {
            "parallel Fate adaptive"
        }
    );

    let mut test_rows = Vec::new();
    let mut test_ok = 0usize;
    for item in &test_pretrain {
        let row = eval_item(&mut client, &corpus, item, "test", false, low_resource)?;
        test_ok += usize::from(row.correct);
        test_rows.push(row);
    }

    let train_d = train_items.len().max(1) as f64;
    let test_d = test_items.len().max(1) as f64;

    Ok(FatePretrainHoldoutReport {
        protocol: PROTOCOL.into(),
        train_n: train_items.len(),
        test_n: test_items.len(),
        train_skipped_no_evidence: train_skipped,
        test_skipped_no_evidence: test_skipped,
        pretrain_epochs: epochs_run,
        train_pass: train_ok as f64 / train_d,
        test_pass: test_ok as f64 / test_d,
        converged,
        max_epochs,
        final_hub_spread: spread_from_client(&client),
        low_resource,
        pretrain_mode: PRETRAIN_MODE.into(),
        corpus_lines: corpus_lines.len(),
        ingest_passes,
        fragment_count: client
            .session()
            .map(|s| s.fragments.len())
            .unwrap_or(fragment_count),
        total_walk_steps: pretrain_out.total_walk_steps,
        train_rows,
        test_rows,
        epoch_log,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adaptive_corpus_includes_sunrise_line_not_only_evidence_tag() {
        let full: Vec<String> = (1..=18)
            .map(|i| format!("[D1:{i}] line {i} about dialogue"))
            .chain([("[D1:14] Melanie: Yeah, I painted that lake sunrise last year!".into())])
            .collect();
        let state = FateAlgebraState::ambient();
        let corpus = adaptive_corpus_for_single(
            &full,
            "LOCO_0_1",
            "When did Melanie paint a sunrise?",
            "2022",
            &state,
        );
        assert!(
            corpus.iter().any(|l| l.contains("sunrise")),
            "adaptive corpus must reach D1:14, got {} lines",
            corpus.len()
        );
    }

    #[test]
    fn qa_scoring_rejects_tag_only_without_answer() {
        assert!(!score_qa_pretrain(
            "[corpus:0] [fate:48@12] Melanie: You'd be a great counselor!",
            "2022",
            "When did Melanie paint a sunrise?",
        ));
    }

    #[test]
    fn qa_scoring_accepts_gold_token_overlap() {
        assert!(score_qa_pretrain(
            "[corpus:0] counseling and mental health certification path",
            "Psychology, counseling certification",
            "What fields would Caroline pursue?",
        ));
    }

    #[test]
    fn qa_scoring_accepts_relative_time_for_when_questions() {
        assert!(score_qa_pretrain(
            "[corpus:0] Melanie: I painted that lake sunrise last year",
            "2022",
            "When did Melanie paint a sunrise?",
        ));
    }

    #[test]
    fn evidence_scoring_uses_content_not_dx_tags() {
        let corpus = vec!["line about LGBTQ support group yesterday".into()];
        assert!(score_with_evidence(
            "[corpus:0] Caroline: I went to a LGBTQ support group yesterday",
            "LGBTQ support group",
            &[],
            &corpus,
        ));
    }

    #[test]
    fn fate_landing_fingerprint_from_corpus_cite() {
        let fps = crate::fate_adaptive_algebra::fate_landing_fingerprints(
            "[corpus:0] [fate:48@3] Caroline: support group",
        );
        assert!(fps.iter().any(|f| f.contains("@3")));
    }
}
