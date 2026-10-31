//! In-process ValhallaBase session client — production eval + long-chain ingest.
//!
//! Replaces Python `tools/valhalla_inference/valhalla_base.py` for embedded Rust callers.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::context_routing::{resolve_context_lines, ContextIngestMode};
use crate::valhalla_base::{
    ValhallaBaseConfig, ValhallaBaseQuestion, ValhallaBaseRequest, ValhallaBaseResponse,
    ValhallaBaseRow, ValhallaBaseSessionState, ValhallaContextItem,
};
use crate::valhalla_base_protocol::{execute_request, production_config, ValhallaBaseErrorBody};

/// Shared Fate pretrain / routing / holdout pipeline mode.
pub const PRETRAIN_PIPE_MODE: &str = "qa_adaptive_fragment";

fn ingress_resource_percent() -> u8 {
    std::env::var("VALHALLA_INGRESS_RESOURCE_PERCENT")
        .ok()
        .and_then(|v| v.parse::<u8>().ok())
        .unwrap_or(70)
        .clamp(1, 100)
}

#[derive(Debug, Clone, Default)]
pub struct RunOptions {
    pub context: Option<Vec<String>>,
    pub context_items: Option<Vec<ValhallaContextItem>>,
    pub append_only: bool,
}

#[derive(Debug, Clone, Default)]
pub struct RunBatchOptions {
    pub ingest_mode: ContextIngestMode,
    pub context_k: usize,
    pub warmup_corpus_lines: usize,
    pub dialogue_warmup: Vec<String>,
    pub checkpoint_path: Option<PathBuf>,
    pub checkpoint_every: usize,
    pub resume_from: Option<PathBuf>,
    pub pause_file: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValhallaBaseCheckpoint {
    pub protocol: String,
    pub index: usize,
    pub questions_done: usize,
    pub rows: Vec<ValhallaBaseRow>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_state: Option<ValhallaBaseSessionState>,
    pub warmup_done: bool,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
pub struct RunBatchOutcome {
    pub last: Option<ValhallaBaseResponse>,
    pub rows: Vec<ValhallaBaseRow>,
    pub paused: bool,
    pub questions_done: usize,
}

/// Stateful ValhallaBase client — mirrors Python `ValhallaBase`.
#[derive(Debug, Clone)]
pub struct ValhallaBaseClient {
    pub config: ValhallaBaseConfig,
    pub context_ingest: ContextIngestMode,
    pub reanchor_affinity: bool,
    session: Option<ValhallaBaseSessionState>,
    context_items: Vec<ValhallaContextItem>,
    pending_append: Vec<ValhallaContextItem>,
    use_append_only: bool,
}

impl Default for ValhallaBaseClient {
    fn default() -> Self {
        Self::production()
    }
}

#[derive(Debug, Clone)]
pub struct FatePretrainOptions {
    /// Number of Fate pretrain passes before long-chain extend.
    pub passes: usize,
    /// Domain knowledge injected before each pass (controls Fate priors).
    pub prior_knowledge: Vec<String>,
    /// Warmup cycles per pass (`fate_warmup_cycles`).
    pub warmup_cycles: usize,
    /// During pretrain: RoundRobin + Evaluated (explore); restore QuadCycle for QA.
    pub explore_during_pretrain: bool,
}

impl Default for FatePretrainOptions {
    fn default() -> Self {
        Self {
            passes: crate::fate_orchestrator::default_pretrain_passes(),
            prior_knowledge: Vec::new(),
            warmup_cycles: 4,
            explore_during_pretrain: true,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct LongChainOptions {
    pub chunk_size: usize,
    pub pretrain: Option<FatePretrainOptions>,
}

impl ValhallaBaseClient {
    pub fn new(config: ValhallaBaseConfig) -> Self {
        Self {
            config,
            context_ingest: ContextIngestMode::Targeted,
            reanchor_affinity: true,
            session: None,
            context_items: Vec::new(),
            pending_append: Vec::new(),
            use_append_only: false,
        }
    }

    pub fn production() -> Self {
        crate::valhalla_base_protocol::ensure_production_runtime();
        Self::new(production_config())
    }

    /// Unified solution client: RelAddr ingress + Fate adapt + multihop v2 compose.
    pub fn unified() -> Self {
        crate::fate_io::ensure_unified_runtime();
        Self::new(crate::fate_io::unified_solution_config())
    }

    /// Enable unified solution flags on an existing client (programmable).
    pub fn enable_unified_solution(&mut self) -> &mut Self {
        crate::fate_io::apply_unified_solution(&mut self.config);
        self
    }

    /// Generic Fate ingress: RelAddr retrieve → ingest into session/context.
    pub fn fate_ingress(
        &mut self,
        q: &ValhallaBaseQuestion,
        corpus: &[String],
        evidence: Option<&[String]>,
    ) -> crate::fate_io::FateIngressBundle {
        let state = crate::fate_adaptive_algebra::FateAlgebraState::ambient();
        let aff = self
            .session
            .as_ref()
            .map(|s| s.snapshot.corpus_line_affinity.as_slice());
        let full = self.session.as_ref().map(|s| s.corpus.as_slice());
        let bundle = crate::fate_io::fate_ingress(q, corpus, &state, aff, evidence, full);
        self.ingest_retrieved_lines(&bundle.lines);
        bundle
    }

    /// Ask one open question on the unified stack (`use_ingress` = RelAddr retrieve first).
    pub fn ask_unified(
        &mut self,
        text: &str,
        corpus: &[String],
        use_ingress: bool,
    ) -> Result<String, ValhallaBaseErrorBody> {
        self.enable_unified_solution();
        let q = ValhallaBaseQuestion {
            id: "ask_unified".into(),
            text: text.to_string(),
            score_type: "open".into(),
            reference_answer: String::new(),
            keywords: Vec::new(),
            audio_features: None,
            image_features: None,
        };
        if use_ingress {
            let _ = self.fate_ingress(&q, corpus, None);
        } else if self.session.is_none() && self.context_items.is_empty() {
            self.set_context(corpus.iter().map(|s| s.as_str()));
        }
        let resp = self.run(vec![q], RunOptions::default())?;
        Ok(resp
            .rows
            .first()
            .map(|r| r.answer.clone())
            .unwrap_or_default())
    }

    pub fn session(&self) -> Option<&ValhallaBaseSessionState> {
        self.session.as_ref()
    }

    pub fn set_context(&mut self, lines: impl IntoIterator<Item = impl AsRef<str>>) {
        self.session = None;
        self.context_items = lines
            .into_iter()
            .map(|s| s.as_ref().trim().to_string())
            .filter(|s| !s.is_empty())
            .map(|t| ValhallaContextItem {
                text: Some(t),
                ..Default::default()
            })
            .collect();
        self.pending_append.clear();
        self.use_append_only = false;
    }

    pub fn reanchor_session_corpus(&mut self) {
        let Some(sess) = self.session.as_mut() else {
            return;
        };
        sess.corpus.clear();
        sess.fingerprint.clear();
        self.context_items.clear();
        self.pending_append.clear();
    }

    pub fn ingest_for_prompt(
        &mut self,
        q: &ValhallaBaseQuestion,
        corpus: &[String],
        k: usize,
        mode: ContextIngestMode,
    ) -> Vec<String> {
        let lines = resolve_context_lines(q, corpus, mode, k);
        if self.reanchor_affinity && self.session.is_some() {
            self.reanchor_session_corpus();
            for ln in &lines {
                self.append_context(ln);
            }
            self.use_append_only = true;
        } else {
            self.set_context(lines.iter().map(|s| s.as_str()));
        }
        lines
    }

    /// Inject lines from [`fate_retrieve_context_lines`] without re-running retrieval.
    /// Append-only path uses `pending_append` only (does not grow `context_items`).
    pub fn ingest_retrieved_lines(&mut self, lines: &[String]) {
        if lines.is_empty() {
            return;
        }
        if self.session.is_some() {
            if self.reanchor_affinity {
                self.reanchor_session_corpus();
            }
            for ln in lines {
                let line = ln.trim();
                if line.is_empty() {
                    continue;
                }
                self.pending_append.push(ValhallaContextItem {
                    text: Some(line.to_string()),
                    ..Default::default()
                });
            }
            self.use_append_only = true;
        } else {
            self.set_context(lines.iter().map(|s| s.as_str()));
        }
    }

    pub fn append_context(&mut self, line: impl AsRef<str>) {
        let line = line.as_ref().trim();
        if line.is_empty() {
            return;
        }
        let item = ValhallaContextItem {
            text: Some(line.to_string()),
            ..Default::default()
        };
        // Session path: pending only — corpus lives in session; avoid duplicate context_items RAM.
        if self.session.is_none() {
            self.context_items.push(item.clone());
        }
        self.pending_append.push(item);
    }

    pub fn append_multimodal(
        &mut self,
        text: Option<&str>,
        audio_features: Option<Vec<f64>>,
        image_features: Option<Vec<f64>>,
    ) {
        let mut item = ValhallaContextItem::default();
        if let Some(t) = text.map(str::trim).filter(|s| !s.is_empty()) {
            item.text = Some(t.to_string());
        }
        if audio_features.as_ref().is_some_and(|v| !v.is_empty()) {
            item.audio_features = audio_features;
        }
        if image_features.as_ref().is_some_and(|v| !v.is_empty()) {
            item.image_features = image_features;
        }
        if item.text.is_some() || item.audio_features.is_some() || item.image_features.is_some() {
            if self.session.is_none() {
                self.context_items.push(item.clone());
            }
            self.pending_append.push(item);
        }
    }

    pub fn clear_context(&mut self) {
        self.session = None;
        self.context_items.clear();
        self.pending_append.clear();
        self.use_append_only = false;
    }

    pub fn fate_warmup_corpus(
        &mut self,
        corpus: &[String],
        n_lines: usize,
    ) -> Result<ValhallaBaseResponse, ValhallaBaseErrorBody> {
        let mut lines: Vec<String> = corpus
            .iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if lines.is_empty() {
            return Err(ValhallaBaseErrorBody::new(
                crate::valhalla_base_protocol::ValhallaBaseErrorKind::Validate,
                "fate_warmup_corpus: corpus is empty",
            ));
        }
        if lines.len() == 1 {
            lines.push(lines[0].clone());
        } else {
            lines.truncate(n_lines.max(2));
        }
        let req = ValhallaBaseRequest {
            context: lines,
            questions: Vec::new(),
            fate_warmup_only: true,
            session: self.session.clone(),
            config: self.config.clone(),
            ..Default::default()
        };
        self.execute_request(req)
    }

    pub fn session_extend(&mut self) -> Result<ValhallaBaseResponse, ValhallaBaseErrorBody> {
        if self.session.is_none() {
            return Err(ValhallaBaseErrorBody::new(
                crate::valhalla_base_protocol::ValhallaBaseErrorKind::Validate,
                "session_extend requires an active session",
            ));
        }
        if self.pending_append.is_empty() {
            return Err(ValhallaBaseErrorBody::new(
                crate::valhalla_base_protocol::ValhallaBaseErrorKind::Validate,
                "session_extend requires pending append lines",
            ));
        }
        let pending = std::mem::take(&mut self.pending_append);
        let append_lines: Vec<String> = pending.iter().filter_map(|it| it.text.clone()).collect();
        let req = ValhallaBaseRequest {
            questions: Vec::new(),
            session_extend_only: true,
            session: self.session.clone(),
            append_items: pending,
            append_lines,
            config: self.config.clone(),
            ..Default::default()
        };
        self.execute_request(req)
    }

    pub fn fate_link_session(&mut self) -> Result<ValhallaBaseResponse, ValhallaBaseErrorBody> {
        if self.session.is_none() {
            return Err(ValhallaBaseErrorBody::new(
                crate::valhalla_base_protocol::ValhallaBaseErrorKind::Validate,
                "fate_link_session requires an active session",
            ));
        }
        let req = ValhallaBaseRequest {
            questions: Vec::new(),
            fate_link_only: true,
            session: self.session.clone(),
            config: self.config.clone(),
            ..Default::default()
        };
        self.execute_request(req)
    }

    pub fn fate_pretrain_adapt(&mut self) -> Result<ValhallaBaseResponse, ValhallaBaseErrorBody> {
        if self.session.is_none() {
            return Err(ValhallaBaseErrorBody::new(
                crate::valhalla_base_protocol::ValhallaBaseErrorKind::Validate,
                "fate_pretrain_adapt requires an active session",
            ));
        }
        let req = ValhallaBaseRequest {
            questions: Vec::new(),
            fate_pretrain_adapt_only: true,
            session: self.session.clone(),
            config: self.config.clone(),
            ..Default::default()
        };
        self.execute_request(req)
    }

    /// Fate adaptive algebra view from current session (ambient if none).
    pub fn algebra_state(&self) -> crate::fate_adaptive_algebra::FateAlgebraState {
        self.session
            .as_ref()
            .map(crate::fate_adaptive_algebra::FateAlgebraState::from_session)
            .unwrap_or_else(crate::fate_adaptive_algebra::FateAlgebraState::ambient)
    }

    pub fn sync_algebra_meta(&mut self) {
        if let Some(sess) = self.session.as_mut() {
            crate::fate_adaptive_algebra::sync_algebra_meta(sess);
        }
    }

    pub fn push_fate_checkpoint(&mut self) {
        if let Some(sess) = self.session.as_mut() {
            crate::fate_adaptive_algebra::push_rollback_checkpoint(sess);
        }
    }

    pub fn rollback_fate(&mut self) -> bool {
        self.session
            .as_mut()
            .map(crate::fate_adaptive_algebra::rollback_session)
            .unwrap_or(false)
    }

    /// Drop Fate rollback checkpoints (releases snapshot clones retained for rollback).
    pub fn trim_fate_rollbacks(&mut self) {
        if let Some(sess) = self.session.as_mut() {
            sess.fate_rollback_stack.clear();
            sess.fate_rollback_stack.shrink_to_fit();
        }
    }

    /// Clear pending append / ingest buffers without dropping session.
    pub fn clear_ingress_pending(&mut self) {
        self.pending_append.clear();
        self.use_append_only = false;
    }

    /// Drop duplicate client-side context copies when session already holds corpus.
    pub fn shrink_ingress_buffers(&mut self) {
        self.context_items.clear();
        self.context_items.shrink_to_fit();
        self.clear_ingress_pending();
    }

    /// Truncate session corpus back to `keep_len` after ephemeral QA appends.
    /// Keeps affinity / fragments / snapshot; drops question_history + ingress buffers.
    pub fn shrink_session_corpus_to(&mut self, keep_len: usize) {
        if let Some(sess) = self.session.as_mut() {
            if sess.corpus.len() > keep_len {
                sess.corpus.truncate(keep_len);
                sess.corpus.shrink_to_fit();
                sess.lines_ingested = keep_len.min(sess.lines_ingested);
            }
            sess.question_history.clear();
        }
        self.shrink_ingress_buffers();
        self.trim_fate_rollbacks();
    }

    pub fn session_corpus_len(&self) -> usize {
        self.session.as_ref().map(|s| s.corpus.len()).unwrap_or(0)
    }

    /// Release per-item eval memory (rollback stack + question history + ingress pending).
    pub fn teardown(&mut self) {
        self.trim_fate_rollbacks();
        self.clear_ingress_pending();
        if let Some(sess) = self.session.as_mut() {
            sess.question_history.clear();
            sess.question_history.shrink_to_fit();
        }
    }

    /// Lightweight isolated QA worker: shares corpus/fragments, drops rollback/history blobs.
    pub fn spawn_fragment_qa_worker(&self) -> Self {
        let mut w = Self::production();
        w.config = self.config.clone();
        w.config.fate_qa_feedback = false;
        w.reanchor_affinity = false;
        w.context_ingest = self.context_ingest;
        let Some(sess) = self.session.as_ref() else {
            return w;
        };
        if crate::fate_orchestrator::fragment_only_session_enabled() && !sess.fragments.is_empty() {
            use crate::fate_orchestrator::{
                export_session_after_ingest, session_from_stored_fragments,
            };
            use crate::signal_ingress::{BodyTarget, IngressBudget};
            let budget = IngressBudget::experiment();
            let target = BodyTarget::parse(&w.config.patch_body);
            let triad = session_from_stored_fragments(
                &sess.fragments,
                &sess.snapshot.corpus_line_affinity,
                budget,
                target,
            );
            let snap = export_session_after_ingest(&triad);
            w.session = Some(ValhallaBaseSessionState {
                corpus: sess.corpus.clone(),
                fingerprint: sess.fingerprint.clone(),
                lines_ingested: sess.lines_ingested,
                fragments: crate::fate_orchestrator::stored_fragments(&snap),
                snapshot: snap,
                fate_algebra_meta: sess.fate_algebra_meta.clone(),
                fate_rollback_stack: Vec::new(),
                question_history: Vec::new(),
                fate_baseline_tick: sess.fate_baseline_tick,
                address_book: sess.address_book.clone(),
                corpus_modality_mask: sess.corpus_modality_mask.clone(),
            });
        } else {
            let mut slim = sess.clone();
            slim.fate_rollback_stack.clear();
            slim.question_history.clear();
            w.session = Some(slim);
        }
        w
    }

    pub fn update_fate_algebra_from_pretrain(&mut self, pass_rate: f64, epochs: usize) {
        if let Some(sess) = self.session.as_mut() {
            crate::fate_adaptive_algebra::update_meta_from_pretrain(sess, pass_rate, epochs);
        }
    }

    /// Supervised corpus-line affinity nudge by corpus index (QA pretrain — no dialogue tags).
    pub fn adjust_corpus_line_affinity_indices(
        &mut self,
        indices: &[usize],
        positive: bool,
        scale: f64,
    ) {
        let state = self.algebra_state();
        let Some(sess) = self.session.as_mut() else {
            return;
        };
        let align = 0.85_f64;
        let unit = if positive {
            state.affinity_pos_delta(align)
        } else {
            state.affinity_neg_delta(align)
        };
        let delta = if positive { unit * scale } else { -unit * scale };
        for &idx in indices {
            while sess.snapshot.corpus_line_affinity.len() <= idx {
                sess.snapshot.corpus_line_affinity.push(0.0);
            }
            sess.snapshot.corpus_line_affinity[idx] =
                (sess.snapshot.corpus_line_affinity[idx] + delta).clamp(0.0, 1.0);
        }
    }

    /// Supervised corpus-line affinity nudge (pretrain holdout / retrieval feedback).
    pub fn adjust_corpus_line_affinity(
        &mut self,
        corpus: &[String],
        line_tags: &[String],
        positive: bool,
        scale: f64,
    ) {
        let state = self.algebra_state();
        let Some(sess) = self.session.as_mut() else {
            return;
        };
        let align = 0.85_f64;
        let unit = if positive {
            state.affinity_pos_delta(align)
        } else {
            state.affinity_neg_delta(align)
        };
        let delta = if positive { unit * scale } else { -unit * scale };
        for tag in line_tags {
            let needle = format!("[{tag}]");
            if let Some(idx) = corpus.iter().position(|l| l.contains(&needle)) {
                while sess.snapshot.corpus_line_affinity.len() <= idx {
                    sess.snapshot.corpus_line_affinity.push(0.0);
                }
                sess.snapshot.corpus_line_affinity[idx] =
                    (sess.snapshot.corpus_line_affinity[idx] + delta).clamp(0.0, 1.0);
            }
        }
    }

    /// Sharpen affinity contrast around reinforced indices (enables natural walk steps).
    pub fn sharpen_walk_affinity_contrast(&mut self, peak_indices: &[usize], scale: f64) {
        let state = self.algebra_state();
        let Some(sess) = self.session.as_mut() else {
            return;
        };
        let aff = &mut sess.snapshot.corpus_line_affinity;
        if aff.is_empty() {
            return;
        }
        let damp = state.neg_coupling() * 0.35 * scale;
        for (i, v) in aff.iter_mut().enumerate() {
            if peak_indices.contains(&i) {
                *v = (*v + state.affinity_pos_delta(0.7) * scale * 0.5).clamp(0.0, 1.0);
            } else {
                *v = (*v - damp).clamp(0.0, 1.0);
            }
        }
        self.sync_algebra_meta();
    }

    /// Fate adapt + fragment link — one flexible pretrain round.
    pub fn fate_round_adapt_link(&mut self) -> Result<(), ValhallaBaseErrorBody> {
        let _ = self.fate_pretrain_adapt()?;
        let _ = self.fate_link_session()?;
        self.sync_algebra_meta();
        Ok(())
    }

    /// Differentiate hub prefs so parallel Fate walk can exceed Δ_req gate.
    pub fn nudge_hub_spread(&mut self, scale: f64) {
        let state = self.algebra_state();
        let Some(sess) = self.session.as_mut() else {
            return;
        };
        let prefs = &mut sess.snapshot.hub_prefs;
        if prefs.len() < 4 {
            return;
        }
        // Relative 4-quad frame — keep prefs in a bounded simplex neighborhood.
        for i in 1..4 {
            prefs[i] = (prefs[i] + state.coupling(i) * 0.04 * scale).clamp(0.05, 2.5);
        }
        prefs[0] = prefs[0].clamp(0.05, 2.5);
        let sum: f64 = prefs.iter().take(4).sum::<f64>().max(1e-9);
        for p in prefs.iter_mut().take(4) {
            *p = (*p / sum) * 4.0 * 0.25; // renorm toward mean 0.25 scale
            *p = p.clamp(0.05, 1.5);
        }
    }

    /// Seed hub spread + corpus affinity from Valhalla `fate_signal_tick` (enables parallel walk).
    pub fn prime_fate_walk_signals(&mut self, corpus: &[String]) {
        self.sync_algebra_meta();
        let state = self.algebra_state();
        let scale = 1.0 + (state.signals_since_baseline() as f64 * 0.01).min(0.5);
        self.nudge_hub_spread(scale);
        let Some(sess) = self.session.as_mut() else {
            return;
        };
        let n = corpus.len().max(sess.corpus.len());
        while sess.snapshot.corpus_line_affinity.len() < n {
            sess.snapshot.corpus_line_affinity.push(0.0);
        }
        // Never rewrite sess.corpus here — repeated fate_normalize was a memory bomb when
        // callers primed every QA / epoch. Affinity nudge only.
        let tick = sess.snapshot.fate_signal_tick;
        let unit = state.coupling(0) * 0.15;
        for (i, line) in corpus.iter().enumerate() {
            let phase = ((tick / 3 + i as u64) % 11) as f64 / 11.0;
            let declarative = if line.contains('?') { 0.0 } else { state.coupling(0) * 0.08 };
            let dialogue = if line.starts_with('[') && line.contains("fate:") {
                state.coupling(0) * 0.05
            } else {
                0.0
            };
            let delta = unit * phase + declarative + dialogue;
            if let Some(slot) = sess.snapshot.corpus_line_affinity.get_mut(i) {
                *slot = (*slot + delta).clamp(0.0, 1.0);
            }
        }
    }

    fn corpus_has_line(&self, line: &str) -> bool {
        let key = line.split_whitespace().collect::<Vec<_>>().join(" ").to_ascii_lowercase();
        self.session
            .as_ref()
            .is_some_and(|s| {
                s.corpus.iter().any(|l| {
                    l.split_whitespace().collect::<Vec<_>>().join(" ").to_ascii_lowercase() == key
                })
            })
    }

    /// Multi-pass Fate pretrain: prior knowledge + rotating seed windows + link each pass.
    pub fn run_fate_pretrain_passes(
        &mut self,
        lines: &[String],
        opts: &FatePretrainOptions,
    ) -> Result<Vec<crate::valhalla_base::FateWarmupSummary>, ValhallaBaseErrorBody> {
        let saved_routing = self.config.fate_ingress_routing;
        let saved_mode = self.config.fate_weight_mode;
        let saved_free = self.config.fate_routing_free;
        if opts.explore_during_pretrain {
            self.config.fate_ingress_routing = crate::valhalla_base::ValhallaFateIngressRouting::RoundRobin;
            self.config.fate_weight_mode = crate::valhalla_base::ValhallaFateWeightMode::Evaluated;
            self.config.fate_routing_free = true;
        }
        self.config.fate_warmup_cycles = opts.warmup_cycles.clamp(1, 64);

        let passes = opts.passes.max(1);
        let seed_n = crate::fate_orchestrator::production_prelearn_count(lines.len()).max(2);
        let mut summaries = Vec::new();

        for pass in 0..passes {
            let start = pass * seed_n / passes;
            let end = ((pass + 1) * seed_n / passes).max(start + 1).min(seed_n);
            let mut batch = opts.prior_knowledge.clone();
            batch.extend(lines.iter().take(seed_n).skip(start).take(end - start).cloned());

            if self.session.is_none() {
                let n = batch.len().min(4).max(2);
                let resp = self.fate_warmup_corpus(&batch, n)?;
                if let Some(w) = resp.fate_warmup {
                    summaries.push(w);
                }
            } else {
                for line in &batch {
                    if !self.corpus_has_line(line) {
                        self.append_context(line);
                    }
                }
                if !self.pending_append.is_empty() {
                    let _ = self.session_extend()?;
                }
                let resp = self.fate_pretrain_adapt()?;
                if let Some(w) = resp.fate_warmup {
                    summaries.push(w);
                }
            }
            let _ = self.fate_link_session()?;
        }

        self.config.fate_ingress_routing = saved_routing;
        self.config.fate_weight_mode = saved_mode;
        self.config.fate_routing_free = saved_free;
        Ok(summaries)
    }

    /// Long-chain: multi-pass Fate pretrain → chunked extend → link finalize → QA.
    pub fn run_long_chain_open(
        &mut self,
        lines: &[String],
        question: ValhallaBaseQuestion,
        chunk_size: usize,
    ) -> Result<ValhallaBaseResponse, ValhallaBaseErrorBody> {
        self.run_long_chain_open_with_options(
            lines,
            question,
            LongChainOptions {
                chunk_size,
                pretrain: Some(FatePretrainOptions::default()),
            },
        )
    }

    pub fn run_long_chain_open_with_options(
        &mut self,
        lines: &[String],
        question: ValhallaBaseQuestion,
        opts: LongChainOptions,
    ) -> Result<ValhallaBaseResponse, ValhallaBaseErrorBody> {
        self.ingest_long_chain_only(lines, opts)?;
        self.run(vec![question], RunOptions::default())
    }

    /// Long-chain ingest + Fate pretrain/link — no final QA (parallel-pipe substrate).
    pub fn ingest_long_chain_only(
        &mut self,
        lines: &[String],
        opts: LongChainOptions,
    ) -> Result<(), ValhallaBaseErrorBody> {
        if lines.is_empty() {
            self.clear_context();
            return Ok(());
        }
        if !self.config.fate_warmup_enabled || lines.len() < 2 {
            self.set_context(lines.iter().map(|s| s.as_str()));
            return Ok(());
        }

        let chunk_size = opts.chunk_size.max(1);
        if let Some(ref pretrain) = opts.pretrain {
            self.run_fate_pretrain_passes(lines, pretrain)?;
        } else {
            let seed_n = crate::fate_orchestrator::production_prelearn_count(lines.len()).max(2);
            self.fate_warmup_corpus(lines, seed_n)?;
        }

        let seed_n = crate::fate_orchestrator::production_prelearn_count(lines.len()).max(2);
        let rest = &lines[seed_n..];
        let mut offset = 0usize;
        while offset < rest.len() {
            let end = (offset + chunk_size).min(rest.len());
            for line in &rest[offset..end] {
                self.append_context(line);
            }
            self.session_extend()?;
            if crate::fate_orchestrator::fate_link_fragments_enabled()
                && crate::fate_orchestrator::fate_link_each_extend_enabled()
            {
                let _ = self.fate_link_session()?;
            }
            offset = end;
        }
        if crate::fate_orchestrator::fate_link_fragments_enabled() {
            let _ = self.fate_pretrain_adapt()?;
            let _ = self.fate_link_session()?;
        }
        self.shrink_ingress_buffers();
        Ok(())
    }

    /// Unified Fate pretrain ingest: dynamic fragmentation + multi-pass pretrain + link.
    pub fn build_flexible_pretrain_substrate(
        lines: &[String],
        chunk_size: usize,
        fast: bool,
        low_resource: bool,
    ) -> Result<(Self, usize), ValhallaBaseErrorBody> {
        let mut client = Self::production();
        let rp = ingress_resource_percent();
        client.config.incubation_cycles = 1;
        eprintln!("[fate-pipe] resource_percent={rp}% dynamic_fragments=on mode={PRETRAIN_PIPE_MODE}");
        client.config.fate_warmup_enabled = true;
        if low_resource {
            client.config.max_new = 32;
            client.config.fate_warmup_cycles = 1;
        } else {
            client.config.max_new = 96;
        }
        let ambient = crate::fate_adaptive_algebra::FateAlgebraState::ambient();
        let ingest_passes = if low_resource {
            2
        } else {
            ambient.default_ingest_pretrain_passes().max(2)
        };
        let chunk = if low_resource {
            chunk_size.max(4).min(16).min(lines.len().max(1))
        } else if fast {
            chunk_size.max(8).min(64).min(lines.len().max(1))
        } else {
            chunk_size.max(4).min(lines.len().max(1))
        };
        let pretrain = FatePretrainOptions {
            passes: ingest_passes,
            explore_during_pretrain: !low_resource,
            warmup_cycles: if low_resource { 1 } else { ingest_passes.max(1) },
            prior_knowledge: Vec::new(),
        };
        client.ingest_long_chain_only(
            lines,
            LongChainOptions {
                chunk_size: chunk,
                pretrain: Some(pretrain),
            },
        )?;
        let _ = client.fate_round_adapt_link();
        client.prime_fate_walk_signals(lines);
        eprintln!(
            "[fate-pipe] ingest done spread={:.4} fragments={}",
            client.algebra_state().effective_walk_spread(),
            client
                .session()
                .map(|s| s.fragments.len())
                .unwrap_or(0),
        );
        Ok((client, ingest_passes))
    }

    pub fn dialogue_warmup(&mut self, turns: &[String]) -> Result<(), ValhallaBaseErrorBody> {
        for (i, q) in turns.iter().enumerate() {
            let _ = self.ask(q, "open", &format!("dw{i}"))?;
        }
        Ok(())
    }

    pub fn save_checkpoint(
        &self,
        path: &Path,
        index: usize,
        rows: &[ValhallaBaseRow],
        warmup_done: bool,
    ) -> Result<(), ValhallaBaseErrorBody> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(io_err)?;
        }
        let doc = ValhallaBaseCheckpoint {
            protocol: "valhalla-base-checkpoint-v1".into(),
            index,
            questions_done: index,
            rows: rows.to_vec(),
            session_state: self.session.clone(),
            warmup_done,
            timestamp: checkpoint_timestamp(),
        };
        let raw = serde_json::to_string_pretty(&doc).map_err(|e| {
            ValhallaBaseErrorBody::new(
                crate::valhalla_base_protocol::ValhallaBaseErrorKind::Io,
                format!("serialize checkpoint: {e}"),
            )
        })?;
        std::fs::write(path, format!("{raw}\n")).map_err(io_err)
    }

    pub fn load_checkpoint(&mut self, path: &Path) -> Result<ValhallaBaseCheckpoint, ValhallaBaseErrorBody> {
        let raw = std::fs::read_to_string(path).map_err(io_err)?;
        let doc: ValhallaBaseCheckpoint = serde_json::from_str(&raw).map_err(|e| {
            ValhallaBaseErrorBody::new(
                crate::valhalla_base_protocol::ValhallaBaseErrorKind::Parse,
                format!("parse checkpoint: {e}"),
            )
        })?;
        if let Some(sess) = &doc.session_state {
            self.session = Some(sess.clone());
        }
        Ok(doc)
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

    pub fn run_batch(
        &mut self,
        questions: Vec<ValhallaBaseQuestion>,
        corpus: &[String],
        opts: RunBatchOptions,
    ) -> Result<RunBatchOutcome, ValhallaBaseErrorBody> {
        let mut all_rows = Vec::new();
        let mut start_idx = 0usize;
        let mut warmup_done = false;
        let mut checkpoint_path = opts.checkpoint_path.clone();

        if let Some(resume) = &opts.resume_from {
            let ckpt = self.load_checkpoint(resume)?;
            all_rows = ckpt.rows;
            start_idx = ckpt.index;
            warmup_done = ckpt.warmup_done;
            if checkpoint_path.is_none() {
                checkpoint_path = Some(resume.clone());
            }
        }

        if !warmup_done
            && self.config.fate_warmup_enabled
            && opts.warmup_corpus_lines > 0
            && !corpus.is_empty()
        {
            self.fate_warmup_corpus(corpus, opts.warmup_corpus_lines)?;
            warmup_done = true;
        }

        if !opts.dialogue_warmup.is_empty() {
            self.dialogue_warmup(&opts.dialogue_warmup)?;
        }

        let ingest_mode = if opts.ingest_mode == ContextIngestMode::Targeted {
            self.context_ingest
        } else {
            opts.ingest_mode
        };
        let k = if opts.context_k == 0 { 8 } else { opts.context_k };

        let mut last: Option<ValhallaBaseResponse> = None;
        let mut paused = false;

        for i in start_idx..questions.len() {
            if Self::pause_requested(opts.pause_file.as_deref()) {
                paused = true;
                if let Some(path) = &checkpoint_path {
                    self.save_checkpoint(path, i, &all_rows, warmup_done)?;
                }
                break;
            }
            let q = questions[i].clone();
            self.ingest_for_prompt(&q, corpus, k, ingest_mode);
            let append = self.use_append_only;
            self.use_append_only = false;
            let resp = self.run(
                vec![q],
                RunOptions {
                    append_only: append,
                    ..Default::default()
                },
            )?;
            all_rows.extend(resp.rows.clone());
            last = Some(resp);
            if opts.checkpoint_every > 0
                && (i + 1) % opts.checkpoint_every == 0
            {
                if let Some(path) = &checkpoint_path {
                    self.save_checkpoint(path, i + 1, &all_rows, warmup_done)?;
                }
            }
        }

        if !paused {
            if let Some(path) = &checkpoint_path {
                if all_rows.len() >= questions.len() && !questions.is_empty() {
                    self.save_checkpoint(path, questions.len(), &all_rows, warmup_done)?;
                }
            }
        }

        Ok(RunBatchOutcome {
            questions_done: all_rows.len(),
            rows: all_rows,
            paused,
            last,
        })
    }

    pub fn run(
        &mut self,
        questions: Vec<ValhallaBaseQuestion>,
        mut opts: RunOptions,
    ) -> Result<ValhallaBaseResponse, ValhallaBaseErrorBody> {
        if self.use_append_only && !opts.append_only {
            opts.append_only = true;
            self.use_append_only = false;
        }
        let req = self.build_request(questions, false, false, false, opts);
        self.execute_request(req)
    }

    pub fn ask(
        &mut self,
        text: impl Into<String>,
        score_type: &str,
        id: &str,
    ) -> Result<String, ValhallaBaseErrorBody> {
        let q = ValhallaBaseQuestion {
            id: id.to_string(),
            text: text.into(),
            score_type: score_type.to_string(),
            reference_answer: String::new(),
            keywords: Vec::new(),
            audio_features: None,
            image_features: None,
        };
        let resp = self.run(vec![q], RunOptions::default())?;
        Ok(resp
            .rows
            .first()
            .map(|r| r.answer.clone())
            .unwrap_or_default())
    }

    pub fn proactive_idle(&mut self) -> Result<ValhallaBaseResponse, ValhallaBaseErrorBody> {
        if self.session.is_none() {
            return Err(ValhallaBaseErrorBody::new(
                crate::valhalla_base_protocol::ValhallaBaseErrorKind::Validate,
                "proactive_idle requires an active session (warmup dialogue first)",
            ));
        }
        let req = self.build_request(
            Vec::new(),
            true,
            false,
            false,
            RunOptions::default(),
        );
        self.execute_request(req)
    }

    fn execute_request(
        &mut self,
        req: ValhallaBaseRequest,
    ) -> Result<ValhallaBaseResponse, ValhallaBaseErrorBody> {
        let resp = execute_request(&req)?;
        self.apply_response(&resp);
        Ok(resp)
    }

    fn apply_response(&mut self, resp: &ValhallaBaseResponse) {
        self.session = Some(resp.session.clone());
        self.clear_ingress_pending();
        self.sync_algebra_meta();
    }

    fn build_request(
        &self,
        questions: Vec<ValhallaBaseQuestion>,
        proactive_idle: bool,
        fate_warmup_only: bool,
        session_extend_only: bool,
        opts: RunOptions,
    ) -> ValhallaBaseRequest {
        let mut req = ValhallaBaseRequest {
            questions,
            proactive_idle,
            fate_warmup_only,
            session_extend_only,
            config: self.config.clone(),
            ..Default::default()
        };

        if opts.append_only && self.session.is_some() {
            req.session = self.session.clone();
            let pending = opts
                .context_items
                .clone()
                .unwrap_or_else(|| self.pending_append.clone());
            req.append_items = pending.clone();
            req.append_lines = pending.iter().filter_map(|it| it.text.clone()).collect();
        } else {
            let has_fresh = opts.context.is_some() || opts.context_items.is_some();
            let continuing = self.session.is_some() && !has_fresh;
            if continuing {
                req.session = self.session.clone();
            } else if let Some(lines) = opts.context {
                req.context = lines;
            } else {
                let items = opts
                    .context_items
                    .clone()
                    .unwrap_or_else(|| self.context_items.clone());
                req.context = items.iter().filter_map(|it| it.text.clone()).collect();
                if !items.is_empty() {
                    req.context_items = items;
                }
            }
            if self.session.is_some() && !has_fresh && req.session.is_none() {
                req.session = self.session.clone();
            }
        }
        req
    }
}

fn checkpoint_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".into())
}

fn io_err(e: std::io::Error) -> ValhallaBaseErrorBody {
    ValhallaBaseErrorBody::new(
        crate::valhalla_base_protocol::ValhallaBaseErrorKind::Io,
        e.to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::valhalla_base::ValhallaBaseDecode;

    #[test]
    fn production_client_uses_hybrid() {
        let c = ValhallaBaseClient::production();
        assert_eq!(c.config.decode, ValhallaBaseDecode::Hybrid);
        assert!(c.config.fate_warmup_enabled);
        assert!(c.reanchor_affinity);
    }

    #[test]
    fn fate_warmup_establishes_session() {
        let mut c = ValhallaBaseClient::production();
        c.config.incubation_cycles = 1;
        let corpus = vec![
            "The capital of France is Paris.".into(),
            "Paris is on the Seine.".into(),
            "France is in Europe.".into(),
        ];
        let resp = c.fate_warmup_corpus(&corpus, 3).expect("warmup");
        assert!(resp.fate_warmup.is_some());
        assert!(c.session().is_some());
    }

    #[test]
    fn session_extend_grows_corpus() {
        let mut c = ValhallaBaseClient::production();
        c.config.incubation_cycles = 1;
        c.fate_warmup_corpus(
            &["line zero".into(), "line one".into()],
            2,
        )
        .expect("warmup");
        c.append_context("line two");
        c.append_context("line three");
        let resp = c.session_extend().expect("extend");
        assert_eq!(resp.session.lines_ingested, 4);
    }

    #[test]
    fn long_chain_open_recalls_needle() {
        let mut lines: Vec<String> = (0..40)
            .map(|i| format!("Filler dialogue turn {i}"))
            .collect();
        lines.insert(12, "The secret code is ALPHA-7749.".into());
        let mut c = ValhallaBaseClient::production();
        c.config.incubation_cycles = 1;
        c.config.max_new = 64;
        let resp = c
            .run_long_chain_open(
                &lines,
                ValhallaBaseQuestion {
                    id: "lc".into(),
                    text: "What is the secret code?".into(),
                    score_type: "open".into(),
                    reference_answer: "ALPHA-7749".into(),
                    keywords: vec!["7749".into()],
                    audio_features: None,
                    image_features: None,
                },
                16,
            )
            .expect("long chain");
        let ans = resp.rows.first().map(|r| r.answer.as_str()).unwrap_or("");
        let low = ans.to_ascii_lowercase();
        assert!(
            low.contains("alpha") || low.contains("7749"),
            "expected code in {ans:?}"
        );
    }
}
