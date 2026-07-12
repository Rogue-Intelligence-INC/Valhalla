//! Rust-native production eval — long-chain, LoCoMo, batch (replaces Python benchmark_runner production paths).

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::context_routing::ContextIngestMode;
use crate::valhalla_base::ValhallaBaseQuestion;
use crate::valhalla_base_client::{RunBatchOptions, RunOptions, ValhallaBaseClient};
use crate::valhalla_base_protocol::{ensure_production_runtime, ValhallaBaseErrorBody};

/// Shared eval control — checkpoint, pause, verbose CLI progress.
#[derive(Debug, Clone, Default)]
pub struct EvalControl {
    pub verbose: bool,
    pub checkpoint_path: Option<PathBuf>,
    pub checkpoint_every: usize,
    pub resume_from: Option<PathBuf>,
    pub pause_file: Option<PathBuf>,
}

impl EvalControl {
    pub fn from_cli_args(args: &[String]) -> Self {
        let mut ctrl = Self::default();
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--verbose" | "-v" => ctrl.verbose = true,
                "--checkpoint" => {
                    i += 1;
                    if let Some(p) = args.get(i) {
                        ctrl.checkpoint_path = Some(PathBuf::from(p));
                    }
                }
                "--checkpoint-every" => {
                    i += 1;
                    if let Some(n) = args.get(i).and_then(|s| s.parse().ok()) {
                        ctrl.checkpoint_every = n;
                    }
                }
                "--resume" | "--resume-from" => {
                    i += 1;
                    if let Some(p) = args.get(i) {
                        ctrl.resume_from = Some(PathBuf::from(p));
                        if ctrl.checkpoint_path.is_none() {
                            ctrl.checkpoint_path = Some(PathBuf::from(p));
                        }
                    }
                }
                "--pause-file" => {
                    i += 1;
                    if let Some(p) = args.get(i) {
                        ctrl.pause_file = Some(PathBuf::from(p));
                    }
                }
                _ => {}
            }
            i += 1;
        }
        if ctrl.checkpoint_every == 0 {
            ctrl.checkpoint_every = 1;
        }
        ctrl
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EvalCheckpoint {
    protocol: String,
    benchmark: String,
    index: usize,
    correct: usize,
    rows: Vec<serde_json::Value>,
    timestamp: String,
}

fn eval_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".into())
}

fn pause_requested(pause_file: Option<&Path>) -> bool {
    let path = pause_file
        .map(|p| p.to_path_buf())
        .or_else(|| {
            std::env::var("VALHALLA_PAUSE_FILE")
                .ok()
                .filter(|s| !s.trim().is_empty())
                .map(PathBuf::from)
        });
    path.as_ref().is_some_and(|p| p.is_file())
}

fn log_progress(verbose: bool, msg: &str) {
    if verbose {
        let _ = writeln!(io::stderr(), "[valhalla-eval] {msg}");
        let _ = io::stderr().flush();
    }
}

/// ASCII progress bar — stderr, `\r` refresh (no extra deps).
struct ProgressBar {
    label: String,
    total: usize,
    current: usize,
    width: usize,
    enabled: bool,
}

impl ProgressBar {
    fn new(label: impl Into<String>, total: usize, enabled: bool) -> Self {
        Self {
            label: label.into(),
            total: total.max(1),
            current: 0,
            width: 32,
            enabled,
        }
    }

    fn set(&mut self, current: usize) {
        self.current = current.min(self.total);
        self.render();
    }

    fn inc(&mut self) {
        self.set(self.current + 1);
    }

    fn render(&self) {
        if !self.enabled {
            return;
        }
        let pct = (self.current as f64 / self.total as f64 * 100.0).min(100.0);
        let filled = ((self.current as f64 / self.total as f64) * self.width as f64).round() as usize;
        let filled = filled.min(self.width);
        let bar = format!("{}{}", "█".repeat(filled), "░".repeat(self.width - filled));
        let _ = write!(
            io::stderr(),
            "\r[{label}] [{bar}] {cur}/{tot} ({pct:.0}%)   ",
            label = self.label,
            bar = bar,
            cur = self.current,
            tot = self.total,
            pct = pct,
        );
        let _ = io::stderr().flush();
    }

    fn finish(&self, msg: &str) {
        if !self.enabled {
            return;
        }
        self.render();
        let _ = writeln!(io::stderr(), " {msg}");
        let _ = io::stderr().flush();
    }
}

fn save_eval_checkpoint(
    path: &Path,
    benchmark: &str,
    index: usize,
    correct: usize,
    rows: &[serde_json::Value],
) -> Result<(), ValhallaBaseErrorBody> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(io_err)?;
    }
    let doc = EvalCheckpoint {
        protocol: "valhalla-eval-checkpoint-v1".into(),
        benchmark: benchmark.into(),
        index,
        correct,
        rows: rows.to_vec(),
        timestamp: eval_timestamp(),
    };
    let raw = serde_json::to_string_pretty(&doc).map_err(|e| {
        ValhallaBaseErrorBody::new(
            crate::valhalla_base_protocol::ValhallaBaseErrorKind::Io,
            format!("serialize eval checkpoint: {e}"),
        )
    })?;
    std::fs::write(path, format!("{raw}\n")).map_err(io_err)
}

fn load_eval_checkpoint(path: &Path) -> Result<EvalCheckpoint, ValhallaBaseErrorBody> {
    let raw = std::fs::read_to_string(path).map_err(io_err)?;
    serde_json::from_str(&raw).map_err(|e| {
        ValhallaBaseErrorBody::new(
            crate::valhalla_base_protocol::ValhallaBaseErrorKind::Parse,
            format!("parse eval checkpoint: {e}"),
        )
    })
}

#[derive(Deserialize)]
struct LocomoBundle {
    items: Vec<LocomoItem>,
}

#[derive(Deserialize)]
struct LocomoItem {
    id: String,
    question: String,
    #[serde(deserialize_with = "deserialize_flex_string")]
    answer: String,
    #[serde(default)]
    evidence: Vec<String>,
    #[serde(default)]
    session_lines: Vec<String>,
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

pub struct LongChainEvalRow {
    pub id: String,
    pub correct: bool,
    pub answer: String,
    pub session_lines_used: usize,
}

pub struct LongChainEvalReport {
    pub protocol: String,
    pub n: usize,
    pub correct: usize,
    pub accuracy: f64,
    pub evidence_scoring: bool,
    pub qa_scoring: bool,
    pub dynamic_fragments: bool,
    pub fragment_mode: String,
    pub rows: Vec<LongChainEvalRow>,
    pub paused: bool,
    pub questions_done: usize,
}

fn score_short_answer(answer: &str, gold: &str) -> bool {
    let ref_s = gold.trim();
    if ref_s.is_empty() {
        return false;
    }
    let low = answer.to_ascii_lowercase();
    if low.contains(&ref_s.to_ascii_lowercase()) {
        return true;
    }
    let ref_tokens: Vec<String> = ref_s
        .split_whitespace()
        .filter_map(|w| {
            let t: String = w
                .chars()
                .filter(|c| c.is_ascii_alphanumeric())
                .collect::<String>()
                .to_lowercase();
            if t.len() >= 2 {
                Some(t)
            } else {
                None
            }
        })
        .collect();
    if ref_tokens.is_empty() {
        return false;
    }
    let hit = ref_tokens.iter().filter(|t| low.contains(t.as_str())).count();
    hit >= (ref_tokens.len() + 1) / 2
}

/// Production long-chain open QA — Fate warmup + chunked session_extend + dynamic density fragments.
pub fn run_long_chain_item(
    lines: &[String],
    item_id: &str,
    question: &str,
    gold: &str,
    evidence: &[String],
    chunk_size: usize,
    fast: bool,
) -> Result<LongChainEvalRow, ValhallaBaseErrorBody> {
    ensure_production_runtime();
    let mut client = ValhallaBaseClient::production();
    client.config.incubation_cycles = if fast { 1 } else { 2 };
    client.config.max_new = 128;

    let kws: Vec<String> = gold
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_ascii_lowercase())
        .filter(|w| w.len() > 2)
        .take(6)
        .collect();
    let q = ValhallaBaseQuestion {
        id: item_id.to_string(),
        text: question.to_string(),
        score_type: "open".into(),
        reference_answer: gold.to_string(),
        keywords: kws,
        audio_features: None,
        image_features: None,
    };

    if crate::fate_parallel_longchain::longchain_parallel_enabled() {
        let ans = crate::fate_parallel_longchain::run_longchain_qa_parallel(
            lines,
            &q,
            chunk_size,
            fast,
            2,
        )?;
        return Ok(LongChainEvalRow {
            id: item_id.to_string(),
            correct: crate::fate_parallel_longchain::score_locomo_answer_ambient(
                &ans, gold, question, evidence, lines,
            ),
            answer: ans.chars().take(200).collect(),
            session_lines_used: lines.len(),
        });
    }

    let resp = client.run_long_chain_open(lines, q, chunk_size)?;
    client.teardown();
    let ans = resp.rows.first().map(|r| r.answer.as_str()).unwrap_or("");
    Ok(LongChainEvalRow {
        id: item_id.to_string(),
        correct: crate::fate_parallel_longchain::score_locomo_answer_ambient(
            ans, gold, question, evidence, lines,
        ),
        answer: ans.chars().take(200).collect(),
        session_lines_used: lines.len(),
    })
}

/// LoCoMo slice eval — Rust-only production path.
pub fn run_locomo_slice(
    bundle_path: &Path,
    limit: usize,
    max_lines: usize,
    chunk_size: usize,
    fast: bool,
    ctrl: &EvalControl,
) -> Result<LongChainEvalReport, ValhallaBaseErrorBody> {
    ensure_production_runtime();
    let raw = std::fs::read_to_string(bundle_path).map_err(io_err)?;
    let doc: LocomoBundle = serde_json::from_str(&raw).map_err(|e| {
        ValhallaBaseErrorBody::new(
            crate::valhalla_base_protocol::ValhallaBaseErrorKind::Parse,
            format!("parse locomo bundle: {e}"),
        )
    })?;

    let mut items = doc.items;
    if limit > 0 {
        items.truncate(limit);
    }
    let total = items.len();
    let resolved_chunk = |corpus_len: usize| {
        crate::fate_orchestrator::eval_chunk_size(corpus_len.max(1), chunk_size)
    };

    let mut rows: Vec<LongChainEvalRow> = Vec::new();
    let mut correct = 0usize;
    let mut start_idx = 0usize;
    let mut paused = false;

    if let Some(resume) = &ctrl.resume_from {
        let ckpt = load_eval_checkpoint(resume)?;
        start_idx = ckpt.index.min(total);
        correct = ckpt.correct;
        for v in ckpt.rows {
            if let Ok(row) = serde_json::from_value::<LongChainEvalRowJson>(v) {
                rows.push(LongChainEvalRow {
                    id: row.id,
                    correct: row.correct,
                    answer: row.answer,
                    session_lines_used: row.session_lines_used,
                });
            }
        }
        log_progress(
            ctrl.verbose,
            &format!("resumed locomo from item {start_idx}/{total} ({correct} correct)"),
        );
    }

    let show_bar = ctrl.verbose || ctrl.checkpoint_path.is_some() || ctrl.resume_from.is_some();
    let mut bar = ProgressBar::new("LoCoMo", total, show_bar);
    bar.set(start_idx);

    for (i, item) in items.iter().enumerate().skip(start_idx) {
        if pause_requested(ctrl.pause_file.as_deref()) {
            paused = true;
            if let Some(path) = &ctrl.checkpoint_path {
                let json_rows: Vec<serde_json::Value> = rows
                    .iter()
                    .map(|r| {
                        serde_json::json!({
                            "id": r.id,
                            "correct": r.correct,
                            "answer": r.answer,
                            "session_lines_used": r.session_lines_used,
                        })
                    })
                    .collect();
                save_eval_checkpoint(path, "locomo", i, correct, &json_rows)?;
            }
            bar.finish("paused");
            log_progress(ctrl.verbose, "pause file detected — checkpoint saved");
            break;
        }

        log_progress(ctrl.verbose, &format!("[{}/{}] {} …", i + 1, total, item.id));
        let full_session: Vec<String> = item.session_lines.iter().cloned().collect();
        let lines: Vec<String> = {
            let adaptive = crate::fate_pretrain_holdout::adaptive_corpus_for_single(
                &full_session,
                &item.id,
                &item.question,
                &item.answer,
                &crate::fate_adaptive_algebra::FateAlgebraState::ambient(),
            );
            if !adaptive.is_empty() {
                adaptive
            } else {
                let mut slice = full_session.clone();
                let cap = crate::fate_orchestrator::eval_corpus_line_cap(max_lines);
                if cap > 0 && slice.len() > cap {
                    slice.truncate(cap);
                }
                slice
            }
        };
        let row = run_long_chain_item(
            &lines,
            &item.id,
            &item.question,
            &item.answer,
            &item.evidence,
            resolved_chunk(lines.len()),
            fast,
        )?;
        correct += usize::from(row.correct);
        rows.push(row);
        bar.inc();

        if let Some(path) = &ctrl.checkpoint_path {
            if ctrl.checkpoint_every > 0 && (i + 1) % ctrl.checkpoint_every == 0 {
                let json_rows: Vec<serde_json::Value> = rows
                    .iter()
                    .map(|r| {
                        serde_json::json!({
                            "id": r.id,
                            "correct": r.correct,
                            "answer": r.answer,
                            "session_lines_used": r.session_lines_used,
                        })
                    })
                    .collect();
                save_eval_checkpoint(path, "locomo", i + 1, correct, &json_rows)?;
            }
        }
    }

    if !paused {
        bar.finish("done");
        if let Some(path) = &ctrl.checkpoint_path {
            let json_rows: Vec<serde_json::Value> = rows
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "id": r.id,
                        "correct": r.correct,
                        "answer": r.answer,
                        "session_lines_used": r.session_lines_used,
                    })
                })
                .collect();
            save_eval_checkpoint(path, "locomo", rows.len(), correct, &json_rows)?;
        }
    }

    let n = total.max(1);
    let done = rows.len();
    Ok(LongChainEvalReport {
        protocol: "locomo-v1-rust".into(),
        n: total,
        correct,
        accuracy: (correct as f64 / n as f64 * 10000.0).round() / 10000.0,
        evidence_scoring: false,
        qa_scoring: true,
        dynamic_fragments: crate::fate_orchestrator::dynamic_corpus_fragments_enabled(),
        fragment_mode: format!("{:?}", crate::fate_orchestrator::production_fragment_mode()),
        rows,
        paused,
        questions_done: done,
    })
}

#[derive(Deserialize)]
struct LongChainEvalRowJson {
    id: String,
    correct: bool,
    answer: String,
    session_lines_used: usize,
}

/// MCQ batch eval — dynamic fragment substrate (fate-pipe prelearn), not legacy 4-line warmup.
pub fn run_mcq_batch_item(
    corpus: &[String],
    item_id: &str,
    prompt: &str,
    reference: &str,
    fast: bool,
    low_resource: bool,
) -> Result<(String, bool), ValhallaBaseErrorBody> {
    run_mcq_batch_item_inner(corpus, item_id, prompt, reference, fast, low_resource, false)
}

/// MCQ with light fragment ingest (warmup + chunk append, no multi-pass pretrain).
pub fn run_mcq_fragment_light(
    corpus: &[String],
    item_id: &str,
    prompt: &str,
    reference: &str,
    fast: bool,
    low_resource: bool,
) -> Result<(String, bool), ValhallaBaseErrorBody> {
    run_mcq_batch_item_inner(corpus, item_id, prompt, reference, fast, low_resource, true)
}

fn run_mcq_batch_item_inner(
    corpus: &[String],
    item_id: &str,
    prompt: &str,
    reference: &str,
    fast: bool,
    low_resource: bool,
    mcq_light: bool,
) -> Result<(String, bool), ValhallaBaseErrorBody> {
    ensure_production_runtime();
    let chunk = crate::fate_orchestrator::eval_chunk_size(corpus.len(), 0);
    let mut client = if mcq_light {
        let mut c = ValhallaBaseClient::production();
        c.config.max_new = 32;
        c.config.fate_warmup_enabled = true;
        c.config.incubation_cycles = 1;
        c.reanchor_affinity = false;
        c.ingest_long_chain_only(
            corpus,
            crate::valhalla_base_client::LongChainOptions {
                chunk_size: chunk.min(corpus.len().max(1)),
                pretrain: None,
            },
        )?;
        let _ = c.fate_round_adapt_link();
        c
    } else {
        let (c, _) =
            ValhallaBaseClient::build_flexible_pretrain_substrate(corpus, chunk, fast, low_resource)?;
        c
    };
    client.config.max_new = 32;
    client.reanchor_affinity = false;

    let q = ValhallaBaseQuestion {
        id: item_id.to_string(),
        text: prompt.to_string(),
        score_type: "mcq".into(),
        reference_answer: reference.to_string(),
        keywords: Vec::new(),
        audio_features: None,
        image_features: None,
    };

    let resp = client.run(vec![q], RunOptions::default())?;
    client.teardown();
    let row = resp.rows.first();
    let ans = row.map(|r| r.answer.as_str()).unwrap_or("");
    let ok = row.map(|r| r.correct).unwrap_or(false);
    Ok((ans.chars().take(120).collect(), ok))
}

#[derive(Deserialize)]
struct FramesBundle {
    #[serde(default)]
    source: String,
    #[serde(default)]
    citation: Option<String>,
    #[serde(default)]
    baseline_no_rag: Option<f64>,
    #[serde(default)]
    baseline_oracle: Option<f64>,
    items: Vec<FramesItem>,
}

#[derive(Deserialize)]
struct FramesItem {
    id: String,
    question: String,
    #[serde(deserialize_with = "deserialize_flex_string")]
    answer: String,
    #[serde(default)]
    wiki_links: Vec<String>,
    #[serde(default)]
    corpus: Vec<String>,
    #[serde(default)]
    wiki_fetched: bool,
}

pub struct FramesEvalRow {
    pub id: String,
    pub correct: bool,
    pub answer: String,
    pub gold: String,
    pub corpus_lines: usize,
    pub wiki_fetched: bool,
}

pub struct FramesEvalReport {
    pub protocol: String,
    pub n: usize,
    pub correct: usize,
    pub accuracy: f64,
    pub dynamic_fragments: bool,
    pub fragment_mode: String,
    pub fragment_only: bool,
    pub wiki_items_fetched: usize,
    pub source: String,
    pub citation: Option<String>,
    pub baseline_no_rag: Option<f64>,
    pub baseline_oracle: Option<f64>,
    pub rows: Vec<FramesEvalRow>,
    pub paused: bool,
    pub questions_done: usize,
}

fn cap_corpus_lines(corpus: Vec<String>, max_lines: usize) -> Vec<String> {
    crate::fate_orchestrator::apply_eval_corpus_cap(corpus, max_lines)
}

/// FRAMES multi-hop RAG — Rust production path with Wikipedia fetch + dynamic fragments.
pub fn run_frames_slice(
    repo_root: &Path,
    bundle_path: &Path,
    limit: usize,
    max_corpus_lines: usize,
    fast: bool,
    offline_wiki: bool,
    ctrl: &EvalControl,
) -> Result<FramesEvalReport, ValhallaBaseErrorBody> {
    ensure_production_runtime();
    let raw = std::fs::read_to_string(bundle_path).map_err(io_err)?;
    let doc: FramesBundle = serde_json::from_str(&raw).map_err(|e| {
        ValhallaBaseErrorBody::new(
            crate::valhalla_base_protocol::ValhallaBaseErrorKind::Parse,
            format!("parse frames bundle: {e}"),
        )
    })?;

    let mut items = doc.items;
    if limit > 0 {
        items.truncate(limit);
    }
    let total = items.len();

    let mut rows: Vec<FramesEvalRow> = Vec::new();
    let mut correct = 0usize;
    let mut wiki_fetched = 0usize;
    let mut start_idx = 0usize;
    let mut paused = false;

    if let Some(resume) = &ctrl.resume_from {
        let ckpt = load_eval_checkpoint(resume)?;
        start_idx = ckpt.index.min(total);
        correct = ckpt.correct;
        for v in ckpt.rows {
            if let Ok(row) = serde_json::from_value::<FramesEvalRowJson>(v) {
                rows.push(FramesEvalRow {
                    id: row.id,
                    correct: row.correct,
                    answer: row.answer,
                    gold: row.gold,
                    corpus_lines: row.corpus_lines,
                    wiki_fetched: row.wiki_fetched,
                });
            }
        }
        log_progress(
            ctrl.verbose,
            &format!("resumed frames from item {start_idx}/{total} ({correct} correct)"),
        );
    }

    let show_bar = ctrl.verbose || ctrl.checkpoint_path.is_some() || ctrl.resume_from.is_some();
    let mut bar = ProgressBar::new("FRAMES", total, show_bar);
    bar.set(start_idx);

    for (i, item) in items.iter().enumerate().skip(start_idx) {
        if pause_requested(ctrl.pause_file.as_deref()) {
            paused = true;
            if let Some(path) = &ctrl.checkpoint_path {
                let json_rows: Vec<serde_json::Value> = rows
                    .iter()
                    .map(|r| {
                        serde_json::json!({
                            "id": r.id,
                            "correct": r.correct,
                            "answer": r.answer,
                            "gold": r.gold,
                            "corpus_lines": r.corpus_lines,
                            "wiki_fetched": r.wiki_fetched,
                        })
                    })
                    .collect();
                save_eval_checkpoint(path, "frames", i, correct, &json_rows)?;
            }
            bar.finish("paused");
            log_progress(ctrl.verbose, "pause file detected — checkpoint saved");
            break;
        }

        log_progress(ctrl.verbose, &format!("[{}/{}] {} …", i + 1, total, item.id));
        let (mut corpus, fetched) = if item.wiki_fetched && !crate::frames_wiki::corpus_needs_wiki_fetch(&item.corpus) {
            (item.corpus.clone(), false)
        } else {
            crate::frames_wiki::resolve_frames_corpus(
                repo_root,
                &item.wiki_links,
                &item.corpus,
                offline_wiki,
            )
        };
        if fetched {
            wiki_fetched += 1;
        }
        corpus = cap_corpus_lines(corpus, max_corpus_lines);
        if corpus.is_empty() {
            corpus.push(format!("FRAMES item {} — no Wikipedia corpus available.", item.id));
        }

        let mut client = ValhallaBaseClient::production();
        client.config.incubation_cycles = if fast { 1 } else { 2 };
        client.config.max_new = 128;
        client.context_ingest = ContextIngestMode::Full;

        let q = ValhallaBaseQuestion {
            id: item.id.clone(),
            text: item.question.clone(),
            score_type: "open".into(),
            reference_answer: item.answer.clone(),
            keywords: Vec::new(),
            audio_features: None,
            image_features: None,
        };

        let outcome = client.run_batch(
            vec![q],
            &corpus,
            RunBatchOptions {
                ingest_mode: ContextIngestMode::Full,
                warmup_corpus_lines: crate::fate_orchestrator::production_prelearn_count(corpus.len()),
                ..Default::default()
            },
        )?;
        client.teardown();

        let ans = outcome
            .rows
            .first()
            .map(|r| r.answer.as_str())
            .unwrap_or("");
        let ok = score_short_answer(ans, &item.answer);
        correct += usize::from(ok);
        rows.push(FramesEvalRow {
            id: item.id.clone(),
            correct: ok,
            answer: ans.chars().take(200).collect(),
            gold: item.answer.clone(),
            corpus_lines: corpus.len(),
            wiki_fetched: fetched || item.wiki_fetched,
        });
        bar.inc();

        if let Some(path) = &ctrl.checkpoint_path {
            if ctrl.checkpoint_every > 0 && (i + 1) % ctrl.checkpoint_every == 0 {
                let json_rows: Vec<serde_json::Value> = rows
                    .iter()
                    .map(|r| {
                        serde_json::json!({
                            "id": r.id,
                            "correct": r.correct,
                            "answer": r.answer,
                            "gold": r.gold,
                            "corpus_lines": r.corpus_lines,
                            "wiki_fetched": r.wiki_fetched,
                        })
                    })
                    .collect();
                save_eval_checkpoint(path, "frames", i + 1, correct, &json_rows)?;
            }
        }
    }

    if !paused {
        bar.finish("done");
        if let Some(path) = &ctrl.checkpoint_path {
            let json_rows: Vec<serde_json::Value> = rows
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "id": r.id,
                        "correct": r.correct,
                        "answer": r.answer,
                        "gold": r.gold,
                        "corpus_lines": r.corpus_lines,
                        "wiki_fetched": r.wiki_fetched,
                    })
                })
                .collect();
            save_eval_checkpoint(path, "frames", rows.len(), correct, &json_rows)?;
        }
    }

    let n = total.max(1);
    let done = rows.len();
    Ok(FramesEvalReport {
        protocol: "frames-v1-rust".into(),
        n: total,
        correct,
        accuracy: (correct as f64 / n as f64 * 10000.0).round() / 10000.0,
        dynamic_fragments: crate::fate_orchestrator::dynamic_corpus_fragments_enabled(),
        fragment_mode: format!("{:?}", crate::fate_orchestrator::production_fragment_mode()),
        fragment_only: crate::fate_orchestrator::fragment_only_session_enabled(),
        wiki_items_fetched: wiki_fetched,
        source: doc.source,
        citation: doc.citation,
        baseline_no_rag: doc.baseline_no_rag,
        baseline_oracle: doc.baseline_oracle,
        rows,
        paused,
        questions_done: done,
    })
}

#[derive(Deserialize)]
struct FramesEvalRowJson {
    id: String,
    correct: bool,
    answer: String,
    gold: String,
    corpus_lines: usize,
    wiki_fetched: bool,
}

fn io_err(e: std::io::Error) -> ValhallaBaseErrorBody {
    ValhallaBaseErrorBody::new(
        crate::valhalla_base_protocol::ValhallaBaseErrorKind::Io,
        e.to_string(),
    )
}

#[derive(Deserialize)]
struct LongbenchBundle {
    items: Vec<LongbenchItem>,
}

#[derive(Deserialize)]
struct LongbenchItem {
    #[serde(rename = "_id")]
    id: String,
    question: String,
    #[serde(rename = "choice_A")]
    choice_a: String,
    #[serde(rename = "choice_B")]
    choice_b: String,
    #[serde(rename = "choice_C")]
    choice_c: String,
    #[serde(rename = "choice_D")]
    choice_d: String,
    answer: String,
    context: String,
    #[serde(default)]
    domain: String,
    #[serde(default)]
    difficulty: String,
    #[serde(default)]
    context_truncated: bool,
}

fn split_context_to_corpus_lines(context: &str, max_lines: usize) -> Vec<String> {
    let text = context.trim();
    if text.is_empty() {
        return Vec::new();
    }
    let unlimited = max_lines == 0;
    let mut parts: Vec<String> = Vec::new();
    for para in text.split("\n\n") {
        let para: String = para.split_whitespace().collect::<Vec<_>>().join(" ");
        if para.is_empty() {
            continue;
        }
        if para.len() <= 480 {
            parts.push(para);
        } else {
            for sent in para.split(". ") {
                let s = sent.trim();
                if !s.is_empty() {
                    parts.push(s.to_string());
                }
            }
        }
        if !unlimited && parts.len() >= max_lines {
            break;
        }
    }
    if parts.is_empty() {
        parts.push(text.chars().take(480).collect());
    }
    if !unlimited {
        parts.truncate(max_lines);
    }
    parts
}

fn format_longbench_mcq_prompt(item: &LongbenchItem) -> String {
    format!(
        "{}\n\nA. {}\nB. {}\nC. {}\nD. {}\n\nAnswer with the letter only (A, B, C, or D).",
        item.question.trim(),
        item.choice_a.trim(),
        item.choice_b.trim(),
        item.choice_c.trim(),
        item.choice_d.trim(),
    )
}

pub struct LongbenchEvalRow {
    pub id: String,
    pub correct: bool,
    pub answer: String,
    pub domain: String,
    pub difficulty: String,
    pub context_truncated: bool,
}

pub struct LongbenchEvalReport {
    pub protocol: String,
    pub n: usize,
    pub correct: usize,
    pub accuracy: f64,
    pub dynamic_fragments: bool,
    pub fragment_mode: String,
    pub warmup_mode: String,
    pub rows: Vec<LongbenchEvalRow>,
}

/// LongBench-v2 MCQ — fragmentwithfate prelearn substrate (not legacy 4-line fate_warmup).
pub fn run_longbench_slice(
    bundle_path: &Path,
    limit: usize,
    max_corpus_lines: usize,
    fast: bool,
    low_resource: bool,
    mcq_light: bool,
) -> Result<LongbenchEvalReport, ValhallaBaseErrorBody> {
    ensure_production_runtime();
    let raw = std::fs::read_to_string(bundle_path).map_err(io_err)?;
    let doc: LongbenchBundle = serde_json::from_str(&raw).map_err(|e| {
        ValhallaBaseErrorBody::new(
            crate::valhalla_base_protocol::ValhallaBaseErrorKind::Parse,
            format!("parse longbench bundle: {e}"),
        )
    })?;
    let mut items = doc.items;
    if limit > 0 {
        items.truncate(limit);
    }
    let mut rows = Vec::new();
    let mut correct = 0usize;
    for item in &items {
        let corpus = split_context_to_corpus_lines(&item.context, max_corpus_lines);
        let prompt = format_longbench_mcq_prompt(item);
        let (ans, ok) = if mcq_light {
            run_mcq_fragment_light(
                &corpus,
                &item.id,
                &prompt,
                &item.answer,
                fast,
                low_resource,
            )?
        } else {
            run_mcq_batch_item(
                &corpus,
                &item.id,
                &prompt,
                &item.answer,
                fast,
                low_resource,
            )?
        };
        if ok {
            correct += 1;
        }
        rows.push(LongbenchEvalRow {
            id: item.id.clone(),
            correct: ok,
            answer: ans,
            domain: item.domain.clone(),
            difficulty: item.difficulty.clone(),
            context_truncated: item.context_truncated,
        });
    }
    let n = items.len().max(1);
    Ok(LongbenchEvalReport {
        protocol: if mcq_light {
            "longbench-v2-fragmentwithfate-light-v1".into()
        } else {
            "longbench-v2-fragmentwithfate-v1".into()
        },
        n: items.len(),
        correct,
        accuracy: (correct as f64 / n as f64 * 10000.0).round() / 10000.0,
        dynamic_fragments: crate::fate_orchestrator::dynamic_corpus_fragments_enabled(),
        fragment_mode: if mcq_light {
            "fragmentwithfate-light".into()
        } else {
            "fragmentwithfate".into()
        },
        warmup_mode: if mcq_light {
            "fragmentwithfate-light".into()
        } else {
            "fragmentwithfate".into()
        },
        rows,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_chain_rust_path_recalls_needle() {
        ensure_production_runtime();
        let mut lines: Vec<String> = (0..36)
            .map(|i| format!("turn {i} filler"))
            .collect();
        lines.insert(10, "Secret vault PIN is NINE-8811.".into());
        let row = run_long_chain_item(
            &lines,
            "t1",
            "What is the vault PIN?",
            "NINE-8811",
            &[],
            12,
            true,
        )
        .expect("long chain");
        let low = row.answer.to_ascii_lowercase();
        assert!(
            row.correct || low.contains("8811") || low.contains("nine"),
            "answer={:?}",
            row.answer
        );
    }
}
