//! Tier B: Native Valhalla Q&A — no external Transformer forward pass.
//!
//! Ingress → TriadSession cycles → structure-weighted memory retrieval → answer.

use serde::{Deserialize, Serialize};

use crate::{
    cortex::TransformType, encode_payload, hash_embed, mcq_fate,
    pipeline, BodyFrontPorts, BodyTarget, DlFrontConfig, EncodingMode, IngressBudget,
    IngressPayload, ModalityInput, TriadFeedReport, TriadSession, DL_EMBED_DIM,
};
use crate::signal_ingress::TriadIncubationSnapshot;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeQaRequest {
    pub body: String,
    pub cycles: usize,
    pub corpus: Vec<String>,
    pub question: String,
    pub score_type: String,
    /// `char` (default) | `dl_front` | `grinder`
    #[serde(default)]
    pub ingress_mode: Option<String>,
    /// Pre-ground ports from `tools/transformer_grinder` (one shard per corpus line).
    #[serde(default)]
    pub corpus_ports: Option<Vec<BodyFrontPorts>>,
    /// Pre-ground ports for the question line.
    #[serde(default)]
    pub question_ports: Option<BodyFrontPorts>,
    /// Incubation cycles after corpus ingress (before question). Falls back to `cycles`.
    #[serde(default)]
    pub corpus_cycles: Option<usize>,
    /// Structure optimize cycles after question ingress. Falls back to `cycles`.
    #[serde(default)]
    pub qa_cycles: Option<usize>,
    /// Feed every corpus line into Tile/Stem (not tail-only cap).
    #[serde(default)]
    pub ingest_all_corpus: Option<bool>,
}

/// One-shot incubation: full corpus → Tile/Stem, then many questions without re-ingesting corpus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeQaBatchRequest {
    pub body: String,
    pub corpus: Vec<String>,
    pub corpus_ports: Vec<BodyFrontPorts>,
    pub questions: Vec<NativeQaQuestionSpec>,
    #[serde(default = "default_incubation_cycles")]
    pub incubation_cycles: usize,
    #[serde(default = "default_qa_cycles")]
    pub qa_cycles: usize,
    #[serde(default)]
    pub ingress_mode: Option<String>,
    /// When true, run MCQ Fate pipeline for mcq items (overrides env when set in request).
    #[serde(default)]
    pub mcq_fate: bool,
    /// v4: LM decompose → Valhalla → recompose decode path.
    #[serde(default)]
    pub pipeline_v4: bool,
    /// Multi-hop structure composition (hop split + yes/no / entity synthesis).
    #[serde(default)]
    pub multihop_v2: bool,
    /// Multi-hop Fate hop-wise adaptation (E7 corpus affinity discovery).
    #[serde(default)]
    pub multihop_fate: bool,
    /// Pre-built incubation snapshot (skip corpus incubation; use for context+core fusion).
    #[serde(default)]
    pub incubation_snapshot: Option<TriadIncubationSnapshot>,
}

fn default_incubation_cycles() -> usize {
    10
}

fn default_qa_cycles() -> usize {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeQaQuestionSpec {
    pub question: String,
    pub score_type: String,
    #[serde(default)]
    pub question_ports: Option<BodyFrontPorts>,
    #[serde(default)]
    pub id: Option<String>,
    /// TTS / audio mel features — fused after question_ports when `VALHALLA_MULTIMODAL_FUSION=1`.
    #[serde(default)]
    pub audio_features: Option<Vec<f64>>,
    /// Prior user questions in session — boosts follow-up-aware corpus ranking.
    #[serde(default)]
    pub prior_questions: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeQaBatchResponse {
    pub mode: String,
    pub corpus_lines: usize,
    pub corpus_lines_ingested: usize,
    pub incubation_cycles: usize,
    pub qa_cycles: usize,
    pub incubation_tile_count: usize,
    pub incubation_stem_cells: usize,
    pub incubation_tile_connections: usize,
    pub incubation_stem_organ_clusters: usize,
    pub incubation_tile_avg_activation: f64,
    pub results: Vec<NativeQaBatchItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeQaBatchItem {
    #[serde(default)]
    pub id: String,
    pub answer: String,
    pub score_type: String,
    pub decode_path: String,
    pub tile_count: usize,
    pub stem_cells: usize,
    pub tile_connections: usize,
    pub tile_avg_activation: f64,
    pub stem_organ_clusters: usize,
    pub correct: bool,
    #[serde(default)]
    pub mcq_fate_cycles: usize,
    #[serde(default)]
    pub mcq_fate_stopped: String,
    #[serde(default)]
    pub mcq_decompose_score: f64,
    #[serde(default)]
    pub mcq_aggregate_score: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fog_evidence: Option<FogEvidence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fog_decision: Option<FogDecision>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retrieval_corpus_idx: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeQaResponse {
    pub answer: String,
    pub body: String,
    pub cycles_executed: usize,
    pub score_type: String,
    pub memory_items: usize,
    pub tile_count: usize,
    pub stem_cells: usize,
    pub tile_dim: f64,
    pub stem_dim: f64,
    pub top_matches: Vec<NativeMatch>,
    pub report_mode: String,
    pub decode_path: String,
    #[serde(default)]
    pub ingress_mode: String,
    #[serde(default)]
    pub dl_encoder: String,
    #[serde(default)]
    pub tile_connections: usize,
    #[serde(default)]
    pub tile_avg_activation: f64,
    #[serde(default)]
    pub stem_organ_clusters: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeMatch {
    pub text: String,
    pub score: f64,
    pub source: String,
}

/// Native retrieval scores for Fate Output Gate (FOG).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FogEvidence {
    pub top_score: f64,
    pub second_score: f64,
    pub margin: f64,
    pub keyword_count: usize,
}

/// Algebraic FOG decision (E7-derived rationals, no fitted floats).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FogDecision {
    pub decision: String,
    pub structure_ready: bool,
    pub evidence_ready: bool,
    pub tau: f64,
    pub delta_required: f64,
    pub reason: String,
}

/// E7 parameters (algebraic integers): r=7, δ=5, dim=133 → base (dim+δ)=138.
const FOG_R: usize = crate::e7_algebra::R;
const FOG_DELTA: usize = crate::e7_algebra::DELTA;
const FOG_DIM: usize = crate::e7_algebra::DIM;

/// τ(k) = (r + δ) / (dim + δ + k)
pub fn fog_tau_algebraic(keyword_count: usize) -> f64 {
    crate::e7_algebra::tau(keyword_count)
}

/// Δ ≥ s₁ · δ / (dim + r)  — adjoint over rank+adjoint scale
pub fn fog_delta_required_algebraic(top_score: f64) -> f64 {
    crate::e7_algebra::delta_required(top_score)
}

pub fn fog_evidence_from_matches(top: &[NativeMatch], question: &str) -> FogEvidence {
    let keyword_count = question_keywords(question).len();
    let top_score = top.first().map(|m| m.score).unwrap_or(0.0);
    let second_score = top.get(1).map(|m| m.score).unwrap_or(0.0);
    FogEvidence {
        top_score,
        second_score,
        margin: top_score - second_score,
        keyword_count,
    }
}

pub fn fog_decide_algebraic(
    evidence: &FogEvidence,
    incubation_cycles: usize,
    min_structure_cycles: usize,
    native_reject: bool,
) -> FogDecision {
    let tau = fog_tau_algebraic(evidence.keyword_count);
    let delta_req = fog_delta_required_algebraic(evidence.top_score);
    let structure_ready = incubation_cycles >= min_structure_cycles;

    if !structure_ready {
        return FogDecision {
            decision: "defer".into(),
            structure_ready: false,
            evidence_ready: false,
            tau,
            delta_required: delta_req,
            reason: format!(
                "incubation_cycles={incubation_cycles}<{min_structure_cycles}"
            ),
        };
    }
    if native_reject {
        return FogDecision {
            decision: "abstain".into(),
            structure_ready: true,
            evidence_ready: false,
            tau,
            delta_required: delta_req,
            reason: "native_reject".into(),
        };
    }
    if evidence.top_score < tau {
        return FogDecision {
            decision: "abstain".into(),
            structure_ready: true,
            evidence_ready: false,
            tau,
            delta_required: delta_req,
            reason: format!(
                "s1={:.4}<tau(k)={:.4} [12/(138+{}))",
                evidence.top_score,
                tau,
                evidence.keyword_count
            ),
        };
    }
    if evidence.margin < delta_req {
        return FogDecision {
            decision: "abstain".into(),
            structure_ready: true,
            evidence_ready: false,
            tau,
            delta_required: delta_req,
            reason: format!(
                "margin={:.4}<s1*delta/(dim+r)={:.4}",
                evidence.margin, delta_req
            ),
        };
    }
    FogDecision {
        decision: "emit".into(),
        structure_ready: true,
        evidence_ready: true,
        tau,
        delta_required: delta_req,
        reason: "algebraic_ready".into(),
    }
}

#[derive(Debug, Clone)]
struct MemoryEntry {
    text: String,
    signals: Vec<f64>,
    source: String,
    corpus_idx: Option<usize>,
}

const HASH_SEED_TEXT: u64 = 0x544558545F464E54;

fn env_f64(key: &str, default: f64) -> f64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn env_usize(key: &str, default: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn min_retrieval_score() -> f64 {
    env_f64("VALHALLA_NATIVE_MIN_SCORE", 0.12)
}

fn min_open_keyword_overlap() -> usize {
    env_usize("VALHALLA_NATIVE_OPEN_MIN_KEYWORDS", 1)
}

fn question_keywords(q: &str) -> Vec<String> {
    let low = q.to_lowercase();
    low.split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 3 || w.chars().all(|c| c.is_ascii_digit()))
        .map(|w| w.to_string())
        .collect()
}

/// True when `keyword` appears in `text` but only inside a negated phrase ("not the capital").
fn keyword_negated_in_text(keyword: &str, text: &str) -> bool {
    let low = text.to_lowercase();
    let kw = keyword.to_lowercase();
    if !low.contains(&kw) {
        return false;
    }
    for pat in ["not the ", "not a ", "isn't the ", "is not the ", "isn't a ", "is not a "] {
        if low.contains(&format!("{pat}{kw}")) {
            return true;
        }
    }
    false
}

fn keyword_overlap_count(question: &str, text: &str) -> usize {
    question_keywords(question)
        .iter()
        .filter(|k| {
            let low = text.to_lowercase();
            low.contains(k.as_str()) && !keyword_negated_in_text(k, text)
        })
        .count()
}

fn question_history_cap() -> usize {
    env_usize("VALHALLA_QUESTION_HISTORY_MAX", 12)
}

fn trim_question_history(history: &[String]) -> Vec<String> {
    let cap = question_history_cap();
    if history.len() <= cap {
        return history.to_vec();
    }
    history[history.len() - cap..].to_vec()
}

fn memory_recency_weight() -> f64 {
    env_f64("VALHALLA_MEMORY_RECENCY_WEIGHT", 0.008)
}

/// Session-local IDF tie-break (corpus-only statistics; no domain lists or penalties).
fn session_idf_term_weight() -> f64 {
    env_f64("VALHALLA_SESSION_IDF_TERM_WEIGHT", 0.08)
}

/// Session-local document frequency → IDF (corpus lines only; no external index).
fn corpus_keyword_idf(keyword: &str, memories: &[MemoryEntry]) -> f64 {
    let corpus: Vec<&MemoryEntry> = memories
        .iter()
        .filter(|m| m.corpus_idx.is_some())
        .collect();
    if corpus.is_empty() {
        return 1.0;
    }
    let kw = keyword.to_lowercase();
    let df = corpus
        .iter()
        .filter(|m| m.text.to_lowercase().contains(kw.as_str()))
        .count();
    ((corpus.len() as f64 + 1.0) / (df as f64 + 1.0)).ln().max(0.05)
}

/// Gentle positive tie-break: rare session terms that appear in a memory line.
/// Does not penalize partial overlap — low confidence is handled by FOG abstain, not forced rerank.
fn session_idf_term_boost(question: &str, memory_text: &str, memories: &[MemoryEntry]) -> f64 {
    let w = session_idf_term_weight();
    if w <= 0.0 {
        return 0.0;
    }
    let ml = memory_text.to_lowercase();
    question_keywords(question)
        .iter()
        .filter(|k| ml.contains(k.as_str()) && !keyword_negated_in_text(k, memory_text))
        .map(|k| corpus_keyword_idf(k, memories) * w)
        .sum()
}

/// Scale grinder + patch + Tile/Stem terms in open retrieval (env tunable).
pub fn open_structure_weight() -> f64 {
    env_f64("VALHALLA_OPEN_STRUCTURE_WEIGHT", 1.2)
}

fn corpus_line_affinity_boost(
    corpus_idx: Option<usize>,
    affinities: &[f64],
    question_sig: &[f64],
    line_sig: &[f64],
    keyword_count: usize,
) -> f64 {
    let w = match std::env::var("VALHALLA_CORPUS_LINE_AFFINITY_WEIGHT") {
        Ok(v) if v.eq_ignore_ascii_case("algebra") || v.eq_ignore_ascii_case("tau") => {
            crate::e7_algebra::corpus_rerank_weight(keyword_count)
        }
        Ok(v) => v.parse().unwrap_or(0.0),
        Err(_) => 0.0,
    };
    let aff = corpus_idx
        .and_then(|i| affinities.get(i))
        .copied()
        .unwrap_or(0.0);
    if aff <= 0.0 {
        return 0.0;
    }
    let align = dot_cosine(question_sig, line_sig).max(0.0);
    aff * align * w
}

/// When true, open answers are withheld if FOG decides abstain (no forced hallucination).
pub fn open_fog_enforce() -> bool {
    std::env::var("VALHALLA_OPEN_FOG_ENFORCE")
        .map(|v| {
            let v = v.trim().to_ascii_lowercase();
            !(v == "0" || v == "false" || v == "off")
        })
        .unwrap_or(true)
}

pub fn apply_open_fog_gate(
    answer: String,
    decode_path: String,
    fog: Option<&FogDecision>,
) -> (String, String) {
    if !open_fog_enforce() {
        return (answer, decode_path);
    }
    if answer.starts_with("No confident match") {
        return (answer, decode_path);
    }
    // Corpus/session citation from native retrieval — structural match, not hallucination.
    if answer.starts_with('[') && answer.contains("] ") {
        return (answer, decode_path);
    }
    if let Some(f) = fog {
        if f.decision == "abstain" {
            return (
                "No confident match in incubated memory.".to_string(),
                "open_fog_abstain".into(),
            );
        }
    }
    (answer, decode_path)
}

/// Boost/penalize corpus line when current question follows prior turns in session.
fn follow_up_retrieval_boost(
    question: &str,
    prior_questions: &[String],
    memory_text: &str,
    corpus_idx: Option<usize>,
) -> f64 {
    if prior_questions.is_empty() {
        return 0.0;
    }
    let cur_overlap = keyword_overlap_count(question, memory_text) as f64;
    let prior_overlap = prior_questions
        .iter()
        .map(|pq| keyword_overlap_count(pq, memory_text) as f64)
        .sum::<f64>()
        / prior_questions.len().max(1) as f64;

    let mut boost = cur_overlap * 0.18;
    if prior_overlap > cur_overlap + 0.5 {
        boost -= (prior_overlap - cur_overlap) * 0.22;
    }

    let prior_kw: Vec<String> = prior_questions
        .iter()
        .flat_map(|pq| question_keywords(pq))
        .collect();
    let unique_hits = question_keywords(question)
        .iter()
        .filter(|k| !prior_kw.contains(k))
        .filter(|k| memory_text.to_lowercase().contains(k.as_str()))
        .count();
    boost += unique_hits as f64 * 0.15;

    if let Some(idx) = corpus_idx {
        boost += (idx as f64) * memory_recency_weight();
    }
    boost
}

fn option_body(opt_line: &str) -> String {
    let t = opt_line.trim();
    if let Some((_, rest)) = t.split_once(')') {
        return rest.trim().to_lowercase();
    }
    if let Some((_, rest)) = t.split_once('.') {
        return rest.trim().to_lowercase();
    }
    t.to_lowercase()
}

fn best_span(question: &str, memory_text: &str) -> Option<String> {
    let parts = memory_sentence_parts(memory_text);
    let mut best: Option<(usize, String)> = None;
    for part in parts {
        let overlap = keyword_overlap_count(question, &part);
        if overlap == 0 {
            continue;
        }
        if best.as_ref().map(|(o, _)| overlap > *o).unwrap_or(true) {
            best = Some((overlap, part));
        }
    }
    best.map(|(_, s)| s)
}

/// Split on sentence boundaries; do not break decimal numbers (e.g. 9.8).
fn memory_sentence_parts(text: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut start = 0usize;
    let bytes = text.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        let c = bytes[i] as char;
        if c == '.' || c == '。' || c == ';' || c == '\n' {
            if c == '.' && i > 0 && i + 1 < bytes.len() {
                let prev = bytes[i - 1];
                let next = bytes[i + 1];
                if prev.is_ascii_digit() && next.is_ascii_digit() {
                    i += 1;
                    continue;
                }
            }
            let chunk = text[start..i].trim();
            if chunk.len() > 4 {
                parts.push(chunk.to_string());
            }
            start = i + 1;
        }
        i += 1;
    }
    let tail = text[start..].trim();
    if tail.len() > 4 {
        parts.push(tail.to_string());
    }
    parts
}

fn decode_signal(
    text: &str,
    score_type: &str,
    ingress_mode: &str,
    budget: &IngressBudget,
    dl: bool,
    front: &DlFrontConfig,
) -> Vec<f64> {
    if ingress_mode == "grinder" {
        return hash_embed(text, DL_EMBED_DIM, HASH_SEED_TEXT);
    }
    memory_signal(text, score_type, budget, dl, front).0
}

fn mcq_corpus_option_boost(opt_line: &str, corpus_mem: &[&MemoryEntry]) -> f64 {
    let body = option_body(opt_line);
    if body.len() < 3 {
        return 0.0;
    }
    let per_hit = env_f64("VALHALLA_MCQ_CORPUS_BOOST_HIT", 0.12);
    let cap = env_f64("VALHALLA_MCQ_CORPUS_BOOST_CAP", 0.85);
    let min_word = env_usize("VALHALLA_MCQ_CORPUS_BOOST_MIN_WORD", 5);
    let mut boost = 0.0;
    for word in body.split_whitespace() {
        let w: String = word
            .trim_matches(|c: char| !c.is_alphanumeric())
            .to_lowercase();
        if w.len() < min_word {
            continue;
        }
        for m in corpus_mem {
            if m.text.to_lowercase().contains(&w) {
                boost += per_hit;
                break;
            }
        }
        if boost >= cap {
            break;
        }
    }
    boost.min(cap)
}

fn mcq_top_corpus_mem<'a>(
    corpus_mem: &[&'a MemoryEntry],
    query_sig: &[f64],
    k: usize,
) -> Vec<&'a MemoryEntry> {
    let mut scored: Vec<_> = corpus_mem
        .iter()
        .map(|m| (*m, dot_cosine(query_sig, &m.signals)))
        .collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.into_iter().take(k).map(|(m, _)| m).collect()
}

/// Reward options uniquely supported by top corpus rows (vs other options).
fn mcq_unique_corpus_lift(
    opt_line: &str,
    all_opts: &[(char, String)],
    top_mem: &[&MemoryEntry],
) -> f64 {
    let body = option_body(opt_line);
    let per = env_f64("VALHALLA_MCQ_UNIQUE_LIFT", 0.14);
    let cap = env_f64("VALHALLA_MCQ_UNIQUE_LIFT_CAP", 0.55);
    let mut lift = 0.0;
    for m in top_mem {
        let mine = keyword_overlap_count(&body, &m.text);
        if mine == 0 {
            continue;
        }
        let max_other = all_opts
            .iter()
            .filter(|(_, line)| line.as_str() != opt_line)
            .map(|(_, line)| keyword_overlap_count(&option_body(line), &m.text))
            .max()
            .unwrap_or(0);
        if mine > max_other {
            lift += (mine - max_other) as f64 * per;
        }
    }
    lift.min(cap)
}

fn mcq_stem(question: &str) -> String {
    mcq_fate::parse_mcq_parts(question).stem.to_lowercase()
}

fn mcq_entailment_enabled() -> bool {
    std::env::var("VALHALLA_MCQ_ENTAILMENT")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// Line-level: does this corpus row support this specific option (keywords + numbers)?
fn mcq_line_entails_option(
    question: &str,
    opt_line: &str,
    evidence: &MemoryEntry,
    opt_sig: &[f64],
) -> f64 {
    let body = option_body(opt_line);
    let stem = mcq_stem(question);
    let ev_low = evidence.text.to_lowercase();
    let mut s = 0.0;
    let mut opt_word_hits = 0usize;

    for word in body.split_whitespace() {
        let w: String = word
            .trim_matches(|c: char| !c.is_alphanumeric())
            .to_lowercase();
        if w.len() >= 3 && ev_low.contains(&w) {
            opt_word_hits += 1;
            s += 0.52;
        }
    }
    if opt_word_hits == 0 {
        s *= 0.25;
    }

    let opt_nums = numbers_in_text(&body);
    if !opt_nums.is_empty() {
        let ev_nums = numbers_in_text(&evidence.text);
        let matched = opt_nums
            .iter()
            .filter(|n| ev_nums.iter().any(|e| (e - **n).abs() < 1e-3))
            .count();
        s += (matched as f64 / opt_nums.len() as f64) * 1.35;
    }

    s += keyword_overlap_count(&body, &evidence.text) as f64 * 0.88;
    s += keyword_overlap_count(&stem, &evidence.text) as f64 * 0.24;
    s += keyword_overlap_count(question, &evidence.text) as f64 * 0.18;
    s += dot_cosine(opt_sig, &evidence.signals) * 0.28;
    s
}

fn rank_mcq_evidence_lines(
    question: &str,
    query_sig: &[f64],
    patch: &[f64],
    memories: &[MemoryEntry],
    tile_sigs: &[Vec<f64>],
    tile_energies: &[f64],
    tile_activations: &[f64],
    stem_sigs: &[Vec<f64>],
    stem_cluster_sigs: &[Vec<f64>],
    stem_cluster_energies: &[f64],
    hub_prefs: &[f64],
) -> Vec<(usize, f64)> {
    let min_ev = min_retrieval_score() * 0.4;
    let scored = score_memories(
        memories,
        query_sig,
        patch,
        tile_sigs,
        tile_energies,
        tile_activations,
        stem_sigs,
        stem_cluster_sigs,
        stem_cluster_energies,
        hub_prefs,
        &["question"],
        1.0,
    );
    scored
        .into_iter()
        .filter(|(i, s)| {
            let m = &memories[*i];
            m.source != "question"
                && *s >= min_ev
                && keyword_overlap_count(question, &m.text) >= 1
        })
        .take(env_usize("VALHALLA_MCQ_ENTAILMENT_TOPK", 6))
        .collect()
}

fn dot_cosine(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len().min(b.len());
    if n == 0 {
        return 0.0;
    }
    let mut dot = 0.0;
    let mut na = 0.0;
    let mut nb = 0.0;
    for i in 0..n {
        dot += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }
    let denom = na.sqrt() * nb.sqrt();
    if denom < 1e-12 {
        0.0
    } else {
        dot / denom
    }
}

fn encode_text(text: &str, budget: &IngressBudget) -> Vec<f64> {
    encode_payload(
        &IngressPayload::Text {
            content: text.to_string(),
        },
        EncodingMode::NormalizedF64,
        budget,
    )
}

fn encode_digits(text: &str, budget: &IngressBudget) -> Vec<f64> {
    let nums = extract_numbers(text);
    if nums.is_empty() {
        return encode_text(text, budget);
    }
    let payload: Vec<f64> = nums.iter().map(|n| (*n / 1000.0).clamp(-1.0, 1.0)).collect();
    encode_payload(
        &IngressPayload::F64 { values: payload },
        EncodingMode::NormalizedF64,
        budget,
    )
}

fn extract_numbers(text: &str) -> Vec<f64> {
    let mut expanded = text.to_string();
    let word_map = [
        ("zero", "0"),
        ("one", "1"),
        ("two", "2"),
        ("three", "3"),
        ("four", "4"),
        ("five", "5"),
        ("six", "6"),
        ("seven", "7"),
        ("eight", "8"),
        ("nine", "9"),
        ("ten", "10"),
        ("twelve", "12"),
        ("fifteen", "15"),
        ("sixteen", "16"),
        ("twenty", "20"),
        ("twenty-four", "24"),
        ("forty-two", "42"),
    ];
    let low = expanded.to_lowercase();
    for (w, d) in word_map {
        if low.contains(w) {
            expanded = expanded.replace(w, d);
            expanded = expanded.replace(&w.to_uppercase(), d);
        }
    }

    let mut nums = Vec::new();
    let mut cur = String::new();
    let mut seen_dot = false;
    for ch in expanded.chars() {
        if ch.is_ascii_digit() {
            cur.push(ch);
        } else if ch == '.' && !seen_dot && !cur.is_empty() {
            cur.push(ch);
            seen_dot = true;
        } else if !cur.is_empty() {
            if let Ok(v) = cur.parse::<f64>() {
                nums.push(v);
            }
            cur.clear();
            seen_dot = false;
        }
    }
    if !cur.is_empty() {
        if let Ok(v) = cur.parse::<f64>() {
            nums.push(v);
        }
    }
    nums
}

fn numbers_in_text(text: &str) -> Vec<f64> {
    extract_numbers(text)
}

fn parse_mcq_options(question: &str) -> Vec<(char, String)> {
    mcq_fate::parse_mcq_options(question)
}

fn fate_tile_weight(hub_prefs: &[f64], idx: usize) -> f64 {
    hub_prefs.get(idx % hub_prefs.len().max(1)).copied().unwrap_or(1.0).max(0.0)
}

fn score_structure_alignment(
    sig: &[f64],
    tile_sigs: &[Vec<f64>],
    tile_energies: &[f64],
    tile_activations: &[f64],
    hub_prefs: &[f64],
    stem_sigs: &[Vec<f64>],
    stem_cluster_sigs: &[Vec<f64>],
    stem_cluster_energies: &[f64],
) -> f64 {
    let mut s = 0.0;
    for (ti, ts) in tile_sigs.iter().enumerate() {
        let act = tile_activations.get(ti).copied().unwrap_or(0.0);
        let e = tile_energies.get(ti).copied().unwrap_or(1.0);
        s += 0.15
            * fate_tile_weight(hub_prefs, ti)
            * dot_cosine(sig, ts)
            * e.max(0.0)
            * (1.0 + act.abs());
    }
    let stem_w = if hub_prefs.len() >= 2 {
        (hub_prefs[1] + hub_prefs[3]) / 2.0
    } else {
        1.0
    };
    for ss in stem_sigs {
        s += 0.08 * stem_w * dot_cosine(sig, ss);
    }
    for (cs, ce) in stem_cluster_sigs.iter().zip(stem_cluster_energies.iter()) {
        s += 0.12 * stem_w * dot_cosine(sig, cs) * ce.max(0.01);
    }
    s
}

fn score_memories(
    memories: &[MemoryEntry],
    query_sig: &[f64],
    patch: &[f64],
    tile_sigs: &[Vec<f64>],
    tile_energies: &[f64],
    tile_activations: &[f64],
    stem_sigs: &[Vec<f64>],
    stem_cluster_sigs: &[Vec<f64>],
    stem_cluster_energies: &[f64],
    hub_prefs: &[f64],
    exclude_sources: &[&str],
    structure_weight: f64,
) -> Vec<(usize, f64)> {
    let sw = structure_weight.max(0.0);
    let mut scored: Vec<(usize, f64)> = memories
        .iter()
        .enumerate()
        .filter(|(_, m)| !exclude_sources.contains(&m.source.as_str()))
        .map(|(i, m)| {
            let mut s = dot_cosine(&m.signals, query_sig) * sw;
            s += 0.35 * dot_cosine(&m.signals, patch) * sw;
            s += score_structure_alignment(
                &m.signals,
                tile_sigs,
                tile_energies,
                tile_activations,
                hub_prefs,
                stem_sigs,
                stem_cluster_sigs,
                stem_cluster_energies,
            ) * sw;
            (i, s)
        })
        .collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored
}

fn keyword_domain_boost(question: &str, memory_text: &str) -> f64 {
    let q = question.to_lowercase();
    let m = memory_text.to_lowercase();
    let mut boost = 0.0;
    let math_keys = ["multiply", "divided", "square root", "integer", "total", "cost", "clips", "eggs"];
    let hist_keys = ["war", "century", "empire", "revolution", "王朝", "战争"];
    let sci_keys = ["unit", "newton", "joule", "photosynthesis", "dna"];
    if math_keys.iter().any(|k| q.contains(k)) && math_keys.iter().any(|k| m.contains(k)) {
        boost += 0.15;
    }
    if hist_keys.iter().any(|k| q.contains(k)) && hist_keys.iter().any(|k| m.contains(k)) {
        boost += 0.12;
    }
    if sci_keys.iter().any(|k| q.contains(k)) && sci_keys.iter().any(|k| m.contains(k)) {
        boost += 0.12;
    }
    boost
}

fn cortex_numeric_candidates(nums: &[f64]) -> Vec<f64> {
    if nums.is_empty() {
        return vec![];
    }
    let mut out = nums.to_vec();
    let diff = TransformType::Difference.apply(nums);
    out.extend(diff);
    let ratio = TransformType::Ratio.apply(nums);
    out.extend(ratio);
    let sum: f64 = nums.iter().sum();
    out.push(sum);
    if nums.len() >= 2 {
        out.push(nums[0] + nums[1]);
        out.push(nums[0] - nums[1]);
        out.push(nums[0] * nums[1]);
        if nums[1].abs() > 1e-9 {
            out.push(nums[0] / nums[1]);
        }
    }
    out.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    out.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
    out
}

fn try_direct_math(question: &str) -> Option<f64> {
    let q = question.to_lowercase();
    let nums = extract_numbers(question);

    if q.contains("multiplied by") && nums.len() >= 2 {
        return Some(nums[0] * nums[1]);
    }
    if q.contains("divided by") && nums.len() >= 2 && nums[1].abs() > 1e-9 {
        return Some(nums[0] / nums[1]);
    }
    if q.contains("square root of") && !nums.is_empty() {
        let v = nums[0];
        if v >= 0.0 {
            return Some(v.sqrt());
        }
    }
    if q.contains('x') && q.contains('=') && nums.len() >= 2 {
        if q.contains('+') {
            return Some(nums[1] - nums[0]);
        }
        if q.contains('-') {
            return Some(nums[1] + nums[0]);
        }
    }

    None
}

fn try_word_problem(question: &str) -> Option<f64> {
    let q = question.to_lowercase();
    let nums = extract_numbers(question);
    if nums.is_empty() {
        return None;
    }

    if q.contains("half as many") && q.contains("altogether") && !nums.is_empty() {
        let a = nums[0];
        return Some(a + a / 2.0);
    }

    if q.contains("per day") && q.contains("breakfast") && nums.len() >= 3 {
        let daily = nums[0];
        let eaten = nums.get(1).copied().unwrap_or(0.0);
        let baked = nums.get(2).copied().unwrap_or(0.0);
        let price = nums.get(3).copied().unwrap_or(1.0);
        return Some((daily - eaten - baked).max(0.0) * price);
    }

    if q.contains("each") && (q.contains("total cost") || q.contains("total")) && nums.len() >= 4 {
        return Some(nums[0] * nums[1] + nums[2] * nums[3]);
    }

    if q.contains("gives") && q.contains("buys") && nums.len() >= 3 {
        return Some(nums[0] - nums[1] + nums[2]);
    }

    if q.contains("altogether") && nums.len() >= 2 {
        return Some(nums.iter().sum());
    }

    None
}

fn format_num(v: f64) -> String {
    if (v - v.round()).abs() < 1e-6 {
        format!("{}", v.round() as i64)
    } else {
        format!("{:.2}", v)
    }
}

fn decode_numeric(
    question: &str,
    query_sig: &[f64],
    patch: &[f64],
    memories: &[MemoryEntry],
    tile_sigs: &[Vec<f64>],
    tile_energies: &[f64],
    tile_activations: &[f64],
    stem_sigs: &[Vec<f64>],
    stem_cluster_sigs: &[Vec<f64>],
    stem_cluster_energies: &[f64],
    hub_prefs: &[f64],
) -> (String, Vec<NativeMatch>, String) {
    let nums = extract_numbers(question);
    let mut candidates: Vec<(f64, f64, String, String)> = Vec::new(); // score, value, text, path

    if let Some(v) = try_direct_math(question) {
        candidates.push((100.0, v, format_num(v), "direct_math".into()));
    }
    if let Some(v) = try_word_problem(question) {
        candidates.push((95.0, v, format_num(v), "word_problem".into()));
    }
    for v in cortex_numeric_candidates(&nums) {
        candidates.push((40.0, v, format_num(v), "cortex".into()));
    }

    let scored_mem = score_memories(
        memories,
        query_sig,
        patch,
        tile_sigs,
        tile_energies,
        tile_activations,
        stem_sigs,
        stem_cluster_sigs,
        stem_cluster_energies,
        hub_prefs,
        &["question"],
        1.0,
    );

    for (i, s) in scored_mem.iter().take(8) {
        let m = &memories[*i];
        for v in numbers_in_text(&m.text) {
            let mem_score = *s * 10.0;
            candidates.push((mem_score, v, format_num(v), format!("memory:{}", m.source)));
        }
        let kb = keyword_domain_boost(question, &m.text);
        if kb > 0.0 {
            candidates.push((kb * 10.0, 0.0, m.text.clone(), "keyword".into()));
        }
    }

    candidates.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    let mut matches: Vec<NativeMatch> = candidates
        .iter()
        .take(5)
        .map(|(score, _, text, src)| NativeMatch {
            text: text.clone(),
            score: *score,
            source: src.clone(),
        })
        .collect();

    if let Some((_, _, ans, path)) = candidates.first() {
        return (ans.clone(), matches, path.clone());
    }

    (
        "No native match in incubated memory.".to_string(),
        matches,
        "none".into(),
    )
}

fn decode_mcq(
    question: &str,
    query_sig: &[f64],
    patch: &[f64],
    memories: &[MemoryEntry],
    hub_prefs: &[f64],
    tile_sigs: &[Vec<f64>],
    tile_energies: &[f64],
    tile_activations: &[f64],
    stem_sigs: &[Vec<f64>],
    stem_cluster_sigs: &[Vec<f64>],
    stem_cluster_energies: &[f64],
    budget: &IngressBudget,
    ingress_mode: &str,
    dl: bool,
    front: &DlFrontConfig,
) -> (String, Vec<NativeMatch>, String) {
    let opts = parse_mcq_options(question);
    let corpus_mem: Vec<_> = memories
        .iter()
        .filter(|m| m.source != "question")
        .collect();

    if opts.is_empty() {
        let scored = score_memories(
            memories,
            query_sig,
            patch,
            tile_sigs,
            tile_energies,
            tile_activations,
            stem_sigs,
            stem_cluster_sigs,
            stem_cluster_energies,
            hub_prefs,
            &["question"],
            1.0,
        );
        let best = scored.first().map(|(i, s)| NativeMatch {
            text: memories[*i].text.clone(),
            score: *s,
            source: memories[*i].source.clone(),
        });
        return (
            best.as_ref()
                .map(|m| m.text.clone())
                .unwrap_or_else(|| "ANSWER: A".to_string()),
            best.into_iter().collect(),
            "mcq_fallback".into(),
        );
    }

    let use_entailment = mcq_entailment_enabled();
    let evidence_lines = if use_entailment {
        rank_mcq_evidence_lines(
            question,
            query_sig,
            patch,
            memories,
            tile_sigs,
            tile_energies,
            tile_activations,
            stem_sigs,
            stem_cluster_sigs,
            stem_cluster_energies,
            hub_prefs,
        )
    } else {
        Vec::new()
    };
    let forward_w = env_f64("VALHALLA_MCQ_ENTAILMENT_FORWARD_WEIGHT", 0.48);
    let entail_w = env_f64("VALHALLA_MCQ_ENTAILMENT_WEIGHT", 0.58);
    let top_corpus = mcq_top_corpus_mem(&corpus_mem, query_sig, env_usize("VALHALLA_MCQ_TOP_CORPUS", 6));
    let mut used_entailment = false;

    let mut best_letter = opts.first().map(|(c, _)| *c).unwrap_or('A');
    let mut best_score = f64::NEG_INFINITY;
    let mut matches = Vec::new();
    let mut raw_scores = Vec::with_capacity(opts.len());

    for (letter, opt_line) in &opts {
        let opt_sig = decode_signal(opt_line, "mcq", ingress_mode, budget, dl, front);
        let mut forward = dot_cosine(&opt_sig, query_sig) + 0.5 * dot_cosine(&opt_sig, patch);
        forward += score_structure_alignment(
            &opt_sig,
            tile_sigs,
            tile_energies,
            tile_activations,
            hub_prefs,
            stem_sigs,
            stem_cluster_sigs,
            stem_cluster_energies,
        );
        for m in &corpus_mem {
            forward += 0.2 * dot_cosine(&opt_sig, &m.signals);
            forward += keyword_domain_boost(question, &m.text) * 0.2;
        }
        if !use_entailment {
            forward += mcq_corpus_option_boost(opt_line, &corpus_mem);
        } else {
            forward += mcq_corpus_option_boost(opt_line, &corpus_mem) * 0.1;
        }
        forward += mcq_unique_corpus_lift(opt_line, &opts, &top_corpus);
        forward += keyword_overlap_count(question, opt_line) as f64 * 0.08;
        if !hub_prefs.is_empty() {
            forward += 0.1 * hub_prefs.iter().take(4).sum::<f64>() / hub_prefs.len().min(4) as f64;
        }

        let mut entail = 0.0;
        if !evidence_lines.is_empty() {
            for (rank, (idx, mem_s)) in evidence_lines.iter().enumerate() {
                let w = 1.0 / (1.0 + rank as f64);
                entail += w
                    * mcq_line_entails_option(question, opt_line, &memories[*idx], &opt_sig)
                    * mem_s.max(0.05);
            }
            if entail > 0.08 {
                used_entailment = true;
            }
        }

        let s = if use_entailment && !evidence_lines.is_empty() {
            forward * forward_w + entail * entail_w
        } else {
            forward
        };

        raw_scores.push(s);

        matches.push(NativeMatch {
            text: format!(
                "{} (score={:.4}{})",
                letter,
                s,
                if use_entailment && entail > 0.08 {
                    format!(", ent={:.3}", entail)
                } else {
                    String::new()
                }
            ),
            score: s,
            source: if use_entailment && entail > 0.08 {
                "mcq_entailment".into()
            } else {
                "mcq_option".into()
            },
        });
    }

    let mean = if raw_scores.is_empty() {
        0.0
    } else {
        raw_scores.iter().sum::<f64>() / raw_scores.len() as f64
    };
    for (idx, (letter, _)) in opts.iter().enumerate() {
        let rel = raw_scores.get(idx).copied().unwrap_or(0.0) - mean;
        let hub = if !hub_prefs.is_empty() {
            hub_prefs.get(idx).copied().unwrap_or(0.25) * 0.02
        } else {
            0.0
        };
        let v = rel + hub;
        if v > best_score {
            best_score = v;
            best_letter = *letter;
        }
    }

    let mut decode_path = if use_entailment && used_entailment {
        "mcq_entailment".to_string()
    } else {
        "mcq_option".to_string()
    };

    if crate::mcq_pairwise::mcq_pairwise_enabled() && opts.len() >= 2 {
        let stem = mcq_stem(question);
        let priors: Vec<(char, f64)> = opts
            .iter()
            .enumerate()
            .map(|(idx, (l, _))| (*l, raw_scores.get(idx).copied().unwrap_or(0.0) - mean))
            .collect();
        let pw = crate::mcq_pairwise::pairwise_option_scores(&stem, &opts, &priors);
        if crate::mcq_pairwise::pairwise_spread(&pw) >= 0.008 {
            if let Some(pl) = crate::mcq_pairwise::pick_pairwise_letter(&pw) {
                best_letter = pl;
                decode_path = "mcq_pairwise".into();
            }
        }
    }

    matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    (format!("ANSWER: {}", best_letter), matches, decode_path)
}

fn decode_open(
    memories: &[MemoryEntry],
    query_sig: &[f64],
    patch: &[f64],
    tile_sigs: &[Vec<f64>],
    tile_energies: &[f64],
    tile_activations: &[f64],
    stem_sigs: &[Vec<f64>],
    stem_cluster_sigs: &[Vec<f64>],
    stem_cluster_energies: &[f64],
    hub_prefs: &[f64],
    question: &str,
    prior_questions: &[String],
    corpus_line_affinity: &[f64],
    multihop_v2: bool,
) -> (String, Vec<NativeMatch>, String, Option<usize>) {
    let min_score = min_retrieval_score();
    let min_kw = min_open_keyword_overlap();
    let sw = open_structure_weight();

    let scored = score_memories(
        memories,
        query_sig,
        patch,
        tile_sigs,
        tile_energies,
        tile_activations,
        stem_sigs,
        stem_cluster_sigs,
        stem_cluster_energies,
        hub_prefs,
        &["question"],
        sw,
    );

    let q_low = question.to_lowercase();
    let kw_count = question_keywords(question).len();
    let mut candidates: Vec<(usize, f64, usize)> = Vec::new();
    for (i, s) in &scored {
        let m = &memories[*i];
        if m.text.to_lowercase() == q_low {
            continue;
        }
        let kw = keyword_overlap_count(question, &m.text);
        if kw < min_kw {
            continue;
        }
        let mut score = *s;
        score += corpus_line_affinity_boost(
            m.corpus_idx,
            corpus_line_affinity,
            query_sig,
            &m.signals,
            kw_count,
        );
        score += session_idf_term_boost(question, &m.text, memories);
        score += follow_up_retrieval_boost(question, prior_questions, &m.text, m.corpus_idx);
        candidates.push((*i, score, kw));
    }

    candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let top: Vec<NativeMatch> = candidates
        .iter()
        .take(5)
        .map(|(i, s, _kw)| {
            let m = &memories[*i];
            let cite = m
                .corpus_idx
                .map(|idx| format!("corpus:{}", idx))
                .unwrap_or_else(|| m.source.clone());
            NativeMatch {
                text: format!("[{}] {}", cite, m.text),
                score: *s,
                source: cite,
            }
        })
        .collect();

    if let Some((idx, score, _)) = candidates.first() {
        if *score < min_score {
            return (
                "No confident match in incubated memory.".to_string(),
                top,
                "open_reject_low_score".into(),
                None,
            );
        }
        if multihop_v2 {
            let mh: Vec<crate::multihop_compose::MultihopCandidate> = candidates
                .iter()
                .map(|(i, sc, kw)| {
                    let m = &memories[*i];
                    crate::multihop_compose::MultihopCandidate {
                        text: m.text.clone(),
                        corpus_idx: m.corpus_idx,
                        source: m.source.clone(),
                        score: *sc,
                        kw: *kw,
                    }
                })
                .collect();
            if let Some((ans, path)) = crate::multihop_compose::compose_multihop_open(question, &mh) {
                if path.starts_with("multihop_v2_yesno") || path.starts_with("multihop_v2_entity") {
                    let picked_idx = mh
                        .iter()
                        .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal))
                        .and_then(|c| c.corpus_idx);
                    return (ans, top, path, picked_idx);
                }
            }
        }
        let m = &memories[*idx];
        let picked_idx = m.corpus_idx;
        let cite = m
            .corpus_idx
            .map(|i| format!("corpus:{}", i))
            .unwrap_or_else(|| m.source.clone());
        let decode_path = if prior_questions.is_empty() {
            "open_retrieval"
        } else {
            "open_retrieval_follow_up_aware"
        };
        if let Some(span) = best_span(question, &m.text) {
            return (
                format!("[{}] {}", cite, span),
                top,
                format!("{decode_path}_span"),
                picked_idx,
            );
        }
        return (
            format!("[{}] {}", cite, m.text),
            top,
            decode_path.into(),
            picked_idx,
        );
    }

    (
        "No confident match in incubated memory.".to_string(),
        top,
        "open_reject_no_keywords".into(),
        None,
    )
}

fn ingress_mode_label(req: &NativeQaRequest) -> String {
    if let Some(mode) = req.ingress_mode.as_deref() {
        return mode.to_ascii_lowercase();
    }
    if std::env::var("VALHALLA_DL_FRONT")
        .map(|v| {
            let v = v.trim().to_ascii_lowercase();
            v == "1" || v == "true" || v == "on"
        })
        .unwrap_or(false)
    {
        return "dl_front".into();
    }
    "char".into()
}

fn use_dl_front(req: &NativeQaRequest) -> bool {
    ingress_mode_label(req) == "dl_front"
}

fn memory_signal(text: &str, score_type: &str, budget: &IngressBudget, dl: bool, front: &DlFrontConfig) -> (Vec<f64>, String) {
    if score_type == "numeric" && dl {
        return (encode_digits(text, budget), "char_digits".into());
    }
    if dl {
        let ports = crate::encode_modality(
            &ModalityInput::Text {
                content: text.to_string(),
            },
            front,
        );
        return (ports.embedding, ports.encoder);
    }
    (encode_text(text, budget), "char".into())
}

/// Run native Tier-B QA: feed corpus + question, optimize structure, decode without HF.
pub fn run_native_qa(req: &NativeQaRequest) -> NativeQaResponse {
    let body = BodyTarget::parse(&req.body);
    let budget = IngressBudget::experiment();
    let mode = ingress_mode_label(req);
    let dl = mode == "dl_front";
    let grinder = mode == "grinder";
    let mut front = DlFrontConfig::default();
    if dl || grinder {
        // QA path: lighter port than training smoke (512) — large corpus × 512 times out.
        front.signals_per_port = std::env::var("VALHALLA_DL_FRONT_SIGNALS")
            .ok()
            .or_else(|| std::env::var("VALHALLA_GRINDER_SIGNALS_PER_PORT").ok())
            .and_then(|v| v.parse().ok())
            .unwrap_or(128);
    }
    let mut session = TriadSession::with_body(budget.clone(), body);
    let mut memories: Vec<MemoryEntry> = Vec::new();
    let mut dl_encoder = String::from("char");

    if grinder {
        let shards = req.corpus_ports.as_ref().filter(|s| !s.is_empty());
        if shards.is_none() {
            panic!("ingress_mode=grinder requires corpus_ports");
        }
        let shards = shards.unwrap();
        let max_ingest = std::env::var("VALHALLA_NATIVE_MAX_CORPUS_LINES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(32);
        let ingest_all = req.ingest_all_corpus.unwrap_or(false)
            || std::env::var("VALHALLA_INGEST_ALL_CORPUS")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false);
        let ingest_skip = if ingest_all {
            0
        } else {
            shards.len().saturating_sub(max_ingest)
        };
        for (i, ports) in shards.iter().enumerate() {
            let t = req
                .corpus
                .get(i)
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("grinder_corpus_{}", i));
            dl_encoder = ports.encoder.clone();
            memories.push(MemoryEntry {
                text: t.to_string(),
                signals: ports.embedding.clone(),
                source: "corpus".into(),
                corpus_idx: Some(i),
            });
            if i >= ingest_skip {
                session.emit_body_ports(ports);
            }
        }
    } else {
        for (ci, line) in req.corpus.iter().enumerate() {
            let t = line.trim();
            if t.is_empty() {
                continue;
            }
            let (sig, enc) = memory_signal(t, "open", &budget, dl, &front);
            if dl {
                dl_encoder = enc;
            }
            memories.push(MemoryEntry {
                text: t.to_string(),
                signals: sig,
                source: "corpus".into(),
                corpus_idx: Some(ci),
            });
            if dl {
                session.feed_modality(
                    ModalityInput::Text {
                        content: t.to_string(),
                    },
                    &front,
                );
            } else {
                session.feed(
                    IngressPayload::Text {
                        content: t.to_string(),
                    },
                    EncodingMode::NormalizedF64,
                );
            }
        }
    }

    let cycles = if req.cycles == 0 { 1 } else { req.cycles };
    let corpus_cycles = req.corpus_cycles.unwrap_or(cycles);
    if corpus_cycles > 0 {
        session.run_optimize_cycles(corpus_cycles);
    }

    if grinder {
        let ports = req
            .question_ports
            .as_ref()
            .expect("ingress_mode=grinder requires question_ports");
        dl_encoder = ports.encoder.clone();
        memories.push(MemoryEntry {
            text: req.question.clone(),
            signals: ports.embedding.clone(),
            source: "question".into(),
            corpus_idx: None,
        });
        session.emit_body_ports(ports);
    } else {
        let (q_sig, q_enc) = memory_signal(&req.question, &req.score_type, &budget, dl, &front);
        if dl {
            dl_encoder = q_enc;
        }
        memories.push(MemoryEntry {
            text: req.question.clone(),
            signals: q_sig.clone(),
            source: "question".into(),
            corpus_idx: None,
        });
        if dl {
            session.feed_modality(
                ModalityInput::Text {
                    content: req.question.clone(),
                },
                &front,
            );
        } else {
            session.feed(
                IngressPayload::Text {
                    content: req.question.clone(),
                },
                EncodingMode::NormalizedF64,
            );
        }
    }

    let qa_cycles = req.qa_cycles.unwrap_or(cycles);
    let mode_label = if grinder {
        "native_qa_grinder"
    } else if dl {
        "native_qa_dl_front"
    } else {
        "native_qa"
    };
    let report: TriadFeedReport = session.finalize_with_cycles(mode_label, qa_cycles);

    let q_sig = memories
        .iter()
        .find(|m| m.source == "question")
        .map(|m| m.signals.clone())
        .unwrap_or_else(|| encode_text(&req.question, &budget));

    let patch = &report.model_patch.patch_vector;
    let hub_prefs = &report.hub_prefs;
    let tile_sigs: Vec<Vec<f64>> = report
        .model_patch
        .tile_signatures
        .iter()
        .take(16)
        .cloned()
        .collect();
    let tile_energies: Vec<f64> = (0..tile_sigs.len()).map(|i| 1.0 / (1 + i) as f64).collect();
    let tile_activations: Vec<f64> = report
        .model_patch
        .tile_activations
        .iter()
        .take(16)
        .copied()
        .collect();
    let stem_sigs: Vec<Vec<f64>> = report.model_patch.stem_signatures.iter().take(16).cloned().collect();
    let stem_cluster_sigs: Vec<Vec<f64>> = report
        .model_patch
        .stem_cluster_signatures
        .iter()
        .take(8)
        .cloned()
        .collect();
    let stem_cluster_energies: Vec<f64> = report
        .model_patch
        .stem_cluster_energies
        .iter()
        .take(8)
        .copied()
        .collect();

    let (answer, top_matches, decode_path, _retrieval_idx) = match req.score_type.as_str() {
        "mcq" => {
            let (a, t, p) = decode_mcq(
                &req.question,
                &q_sig,
                patch,
                &memories,
                hub_prefs,
                &tile_sigs,
                &tile_energies,
                &tile_activations,
                &stem_sigs,
                &stem_cluster_sigs,
                &stem_cluster_energies,
                &budget,
                &mode,
                dl,
                &front,
            );
            (a, t, p, None)
        }
        "numeric" => {
            let (a, t, p) = decode_numeric(
                &req.question,
                &q_sig,
                patch,
                &memories,
                &tile_sigs,
                &tile_energies,
                &tile_activations,
                &stem_sigs,
                &stem_cluster_sigs,
                &stem_cluster_energies,
                hub_prefs,
            );
            (a, t, p, None)
        }
        _ => decode_open(
            &memories,
            &q_sig,
            patch,
            &tile_sigs,
            &tile_energies,
            &tile_activations,
            &stem_sigs,
            &stem_cluster_sigs,
            &stem_cluster_energies,
            hub_prefs,
            &req.question,
            &[],
            &[],
            false,
        ),
    };

    NativeQaResponse {
        answer,
        body: req.body.clone(),
        cycles_executed: report.optimize_cycles,
        score_type: req.score_type.clone(),
        memory_items: memories.len(),
        tile_count: report.model_patch.tile_count,
        stem_cells: report.model_patch.stem_cell_count,
        tile_dim: report.tile_dim,
        stem_dim: report.stem_dim,

        top_matches,
        report_mode: report.mode,
        decode_path,
        ingress_mode: mode,
        dl_encoder,
        tile_connections: report.model_patch.tile_connection_count,
        tile_avg_activation: report.model_patch.tile_avg_activation,
        stem_organ_clusters: report.model_patch.stem_organ_clusters.len(),
    }
}

fn corpus_ingest_skip(len: usize, ingest_all: bool) -> usize {
    if ingest_all {
        return 0;
    }
    let max_ingest = env_usize("VALHALLA_NATIVE_MAX_CORPUS_LINES", 32);
    len.saturating_sub(max_ingest)
}

fn corpus_memories_from_ports(corpus: &[String], shards: &[BodyFrontPorts]) -> Vec<MemoryEntry> {
    let mut memories = Vec::new();
    for (i, ports) in shards.iter().enumerate() {
        let t = corpus
            .get(i)
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("grinder_corpus_{}", i));
        memories.push(MemoryEntry {
            text: t,
            signals: ports.embedding.clone(),
            source: "corpus".into(),
            corpus_idx: Some(i),
        });
    }
    memories
}

fn feed_grinder_corpus(
    session: &mut TriadSession,
    corpus: &[String],
    shards: &[BodyFrontPorts],
    ingest_all: bool,
) -> (Vec<MemoryEntry>, String) {
    let ingest_skip = corpus_ingest_skip(shards.len(), ingest_all);
    let mut memories = Vec::new();
    let mut dl_encoder = String::from("grinder");
    for (i, ports) in shards.iter().enumerate() {
        let t = corpus
            .get(i)
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("grinder_corpus_{}", i));
        dl_encoder = ports.encoder.clone();
        memories.push(MemoryEntry {
            text: t,
            signals: ports.embedding.clone(),
            source: "corpus".into(),
            corpus_idx: Some(i),
        });
        if i >= ingest_skip {
            session.emit_body_ports(ports);
        }
    }
    (memories, dl_encoder)
}

fn multihop_candidates_from_memories(
    question: &str,
    memories: &[MemoryEntry],
) -> Vec<crate::multihop_compose::MultihopCandidate> {
    let q_low = question.to_lowercase();
    let min_kw = min_open_keyword_overlap();
    let mut out: Vec<crate::multihop_compose::MultihopCandidate> = memories
        .iter()
        .filter(|m| m.corpus_idx.is_some())
        .filter(|m| m.text.to_lowercase() != q_low)
        .filter_map(|m| {
            let kw = keyword_overlap_count(question, &m.text);
            if kw < min_kw {
                return None;
            }
            let score = kw as f64 * 0.12 + m.signals.iter().take(8).sum::<f64>() * 1e-4;
            Some(crate::multihop_compose::MultihopCandidate {
                text: m.text.clone(),
                corpus_idx: m.corpus_idx,
                source: m.source.clone(),
                score,
                kw,
            })
        })
        .collect();
    out.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    out
}

fn apply_multihop_post_compose(
    question: &str,
    memories: &[MemoryEntry],
    answer: &str,
) -> Option<(String, String)> {
    let candidates = multihop_candidates_from_memories(question, memories);
    let composed = crate::multihop_compose::compose_multihop_short_answer(question, &candidates)?;
    let ans_low = answer.to_lowercase().trim().to_string();
    let mh_low = composed.0.to_lowercase();
    if composed.1.contains("yesno") {
        return Some(composed);
    }
    if composed.1.contains("entity") {
        if ans_low == mh_low {
            return None;
        }
        return Some(composed);
    }
    None
}

fn decode_from_report(
    question: &str,
    score_type: &str,
    memories: &[MemoryEntry],
    report: &TriadFeedReport,
    budget: &IngressBudget,
    mode: &str,
    dl: bool,
    front: &DlFrontConfig,
    prior_questions: &[String],
    corpus_line_affinity: &[f64],
    multihop_v2: bool,
) -> (String, Vec<NativeMatch>, String, Option<usize>) {
    let q_sig = memories
        .iter()
        .find(|m| m.source == "question")
        .map(|m| m.signals.clone())
        .unwrap_or_else(|| encode_text(question, budget));

    let patch = &report.model_patch.patch_vector;
    let hub_prefs = &report.hub_prefs;
    let tile_sigs: Vec<Vec<f64>> = report
        .model_patch
        .tile_signatures
        .iter()
        .take(16)
        .cloned()
        .collect();
    let tile_energies: Vec<f64> = (0..tile_sigs.len()).map(|i| 1.0 / (1 + i) as f64).collect();
    let tile_activations: Vec<f64> = report
        .model_patch
        .tile_activations
        .iter()
        .take(16)
        .copied()
        .collect();
    let stem_sigs: Vec<Vec<f64>> = report.model_patch.stem_signatures.iter().take(16).cloned().collect();
    let stem_cluster_sigs: Vec<Vec<f64>> = report
        .model_patch
        .stem_cluster_signatures
        .iter()
        .take(8)
        .cloned()
        .collect();
    let stem_cluster_energies: Vec<f64> = report
        .model_patch
        .stem_cluster_energies
        .iter()
        .take(8)
        .copied()
        .collect();

    match score_type {
        "mcq" => {
            let (a, t, p) = decode_mcq(
                question,
                &q_sig,
                patch,
                memories,
                hub_prefs,
                &tile_sigs,
                &tile_energies,
                &tile_activations,
                &stem_sigs,
                &stem_cluster_sigs,
                &stem_cluster_energies,
                budget,
                mode,
                dl,
                front,
            );
            (a, t, p, None)
        }
        "numeric" => {
            let (a, t, p) = decode_numeric(
                question,
                &q_sig,
                patch,
                memories,
                &tile_sigs,
                &tile_energies,
                &tile_activations,
                &stem_sigs,
                &stem_cluster_sigs,
                &stem_cluster_energies,
                hub_prefs,
            );
            (a, t, p, None)
        }
        _ => decode_open(
            memories,
            &q_sig,
            patch,
            &tile_sigs,
            &tile_energies,
            &tile_activations,
            &stem_sigs,
            &stem_cluster_sigs,
            &stem_cluster_energies,
            hub_prefs,
            question,
            prior_questions,
            corpus_line_affinity,
            multihop_v2,
        ),
    }
}

fn response_from_report(
    req_body: &str,
    score_type: &str,
    memories: &[MemoryEntry],
    report: &TriadFeedReport,
    mode: &str,
    dl_encoder: &str,
    answer: String,
    top_matches: Vec<NativeMatch>,
    decode_path: String,
) -> NativeQaResponse {
    NativeQaResponse {
        answer,
        body: req_body.to_string(),
        cycles_executed: report.optimize_cycles,
        score_type: score_type.to_string(),
        memory_items: memories.len(),
        tile_count: report.model_patch.tile_count,
        stem_cells: report.model_patch.stem_cell_count,
        tile_dim: report.tile_dim,
        stem_dim: report.stem_dim,
        top_matches,
        report_mode: report.mode.clone(),
        decode_path,
        ingress_mode: mode.to_string(),
        dl_encoder: dl_encoder.to_string(),
        tile_connections: report.model_patch.tile_connection_count,
        tile_avg_activation: report.model_patch.tile_avg_activation,
        stem_organ_clusters: report.model_patch.stem_organ_clusters.len(),
    }
}

/// Full training corpus → Tile/Stem incubation, then QA without re-feeding corpus.
pub fn run_native_qa_batch(req: &NativeQaBatchRequest) -> NativeQaBatchResponse {
    let body = BodyTarget::parse(&req.body);
    let budget = IngressBudget::incubation();
    let mode = req
        .ingress_mode
        .as_deref()
        .unwrap_or("grinder")
        .to_ascii_lowercase();
    let dl = mode == "dl_front";
    let mut front = DlFrontConfig::default();
    front.signals_per_port = std::env::var("VALHALLA_DL_FRONT_SIGNALS")
        .ok()
        .or_else(|| std::env::var("VALHALLA_GRINDER_SIGNALS_PER_PORT").ok())
        .and_then(|v| v.parse().ok())
        .unwrap_or(128);

    let (corpus_memories, dl_encoder, snap, incubation_report) =
        if let Some(pre) = req.incubation_snapshot.clone() {
            let memories = corpus_memories_from_ports(&req.corpus, &req.corpus_ports);
            let mut session = TriadSession::from_incubation_snapshot(budget.clone(), body, pre.clone());
            let report = session.build_feed_report("native_batch_prebuilt_snap", 0);
            (memories, "grinder".into(), pre, report)
        } else {
            let mut session = TriadSession::with_body(budget.clone(), body);
            let (corpus_memories, dl_encoder) =
                feed_grinder_corpus(&mut session, &req.corpus, &req.corpus_ports, true);
            session.run_optimize_cycles(req.incubation_cycles);
            let incubation_report = session.build_feed_report("native_batch_incubation", 0);
            let snap = session.incubation_snapshot();
            (corpus_memories, dl_encoder, snap, incubation_report)
        };

    let mode_label = if mode == "grinder" {
        "native_qa_batch_grinder"
    } else if dl {
        "native_qa_batch_dl_front"
    } else {
        "native_qa_batch"
    };

    let mut results = Vec::with_capacity(req.questions.len());
    let multihop_v2 = req.multihop_v2 || crate::multihop_compose::multihop_v2_enabled();
    let multihop_fate = req.multihop_fate || crate::multihop_fate::multihop_fate_enabled();
    let use_pipeline_v4 = req.pipeline_v4 || pipeline::pipeline_v4_enabled();
    let pipe_cfg = pipeline::PipelineConfig::default();

    for q in &req.questions {
        let prior = q.prior_questions.as_deref().unwrap_or(&[]);
        let mut qa_session =
            TriadSession::from_incubation_snapshot(budget.clone(), body, snap.clone());
        let mut memories = corpus_memories.clone();
        let question_ports = q
            .question_ports
            .as_ref()
            .expect("batch QA requires question_ports per item (grinder ingress)");
        memories.push(MemoryEntry {
            text: q.question.clone(),
            signals: question_ports.embedding.clone(),
            source: "question".into(),
            corpus_idx: None,
        });
        qa_session.emit_body_ports(question_ports);
        crate::multimodal::feed_audio(
            &mut qa_session,
            q.audio_features.as_deref(),
            &front,
        );

        let mut fate_line_aff = snap.corpus_line_affinity.clone();
        if multihop_fate && q.score_type == "open" {
            let corpus_refs: Vec<crate::multihop_fate::CorpusLineRef> = memories
                .iter()
                .filter_map(|m| {
                    m.corpus_idx.map(|idx| crate::multihop_fate::CorpusLineRef {
                        idx,
                        text: m.text.clone(),
                        signals: m.signals.clone(),
                    })
                })
                .collect();
            let _fate_adapt = crate::multihop_fate::run_multihop_fate_adapt(
                &mut qa_session,
                &q.question,
                &corpus_refs,
                question_ports,
                req.qa_cycles.max(2),
                &front,
            );
            fate_line_aff = qa_session.incubation_snapshot().corpus_line_affinity;
        }

        let use_mcq_fate = req.mcq_fate || mcq_fate::mcq_fate_cycles_enabled();

        let (report, mcq_fate_meta, pipeline_outcome) = if use_pipeline_v4 {
            let outcome = pipeline::run_question_pipeline(
                &snap,
                &q.question,
                &q.score_type,
                question_ports,
                &pipe_cfg,
                &front,
                mode_label,
                q.audio_features.as_deref(),
            );
            let report = outcome.report.clone();
            let meta = outcome.mcq_fate.as_ref().map(|o| {
                (
                    o.cycles_executed,
                    o.stopped_reason.clone(),
                    o.decompose_score,
                    o.aggregate_score,
                )
            });
            (report, meta, Some(outcome))
        } else if q.score_type == "mcq" && use_mcq_fate {
            let outcome = mcq_fate::run_mcq_fate_pipeline(
                &mut qa_session,
                &q.question,
                question_ports,
                mode_label,
                &front,
            );
            (
                outcome.report,
                Some((
                    outcome.cycles_executed,
                    outcome.stopped_reason,
                    outcome.decompose_score,
                    outcome.aggregate_score,
                )),
                None,
            )
        } else {
            (
                qa_session.build_feed_report(mode_label, req.qa_cycles),
                None,
                None,
            )
        };

        let line_aff = if multihop_fate && q.score_type == "open" {
            &fate_line_aff
        } else {
            &snap.corpus_line_affinity
        };
        let (mut answer, top, mut decode_path, retrieval_corpus_idx) =
            if let Some(ref po) = pipeline_outcome {
                if q.score_type == "mcq" {
                    let (dec_ans, _, dec_path, _) = decode_from_report(
                        &q.question,
                        &q.score_type,
                        &memories,
                        &report,
                        &budget,
                        &mode,
                        dl,
                        &front,
                        prior,
                        line_aff,
                        multihop_v2,
                    );
                    let (ans, path) = pipeline::merge_mcq_pipeline_decode(
                        &dec_ans,
                        &dec_path,
                        &po.recompose.mcq_option_affinity,
                    );
                    (ans, Vec::new(), path, None)
                } else {
                    let (a, t, _dp, ridx) = decode_from_report(
                        &q.question,
                        &q.score_type,
                        &memories,
                        &report,
                        &budget,
                        &mode,
                        dl,
                        &front,
                        prior,
                        line_aff,
                        multihop_v2,
                    );
                    (a, t, format!("pipeline_v4_{}", q.score_type), ridx)
                }
            } else {
                decode_from_report(
                    &q.question,
                    &q.score_type,
                    &memories,
                    &report,
                    &budget,
                    &mode,
                    dl,
                    &front,
                    prior,
                    line_aff,
                    multihop_v2,
                )
            };

        if mcq_fate_meta.is_some() && pipeline_outcome.is_none() {
            decode_path = "mcq_fate_cycle".to_string();
        } else if let Some(ref po) = pipeline_outcome {
            if q.score_type != "mcq" {
                decode_path = po.decode_hint.clone();
            } else if !decode_path.starts_with("pipeline_v4_") && !decode_path.starts_with("mcq_") {
                decode_path = po.decode_hint.clone();
            }
        }
        let fog_evidence = fog_evidence_from_matches(&top, &q.question);
        let native_reject = decode_path.contains("reject")
            || answer.starts_with("No confident match");
        let fog_decision = fog_decide_algebraic(
            &fog_evidence,
            req.incubation_cycles.max(1),
            2,
            native_reject,
        );

        if q.score_type == "open" && !multihop_v2 && !multihop_fate {
            (answer, decode_path) = apply_open_fog_gate(answer, decode_path, Some(&fog_decision));
        }

        if multihop_v2 && q.score_type == "open" {
            if let Some((mh_ans, mh_path)) =
                apply_multihop_post_compose(&q.question, &memories, &answer)
            {
                answer = mh_ans;
                decode_path = mh_path;
            }
        }
        if multihop_fate && q.score_type == "open" {
            let corpus_refs: Vec<crate::multihop_fate::CorpusLineRef> = memories
                .iter()
                .filter_map(|m| {
                    m.corpus_idx.map(|idx| crate::multihop_fate::CorpusLineRef {
                        idx,
                        text: m.text.clone(),
                        signals: m.signals.clone(),
                    })
                })
                .collect();
            let q_sig = question_ports.embedding.clone();
            let fate_decode = if multihop_v2 {
                crate::multihop_fate::decode_fate_comparison_only(
                    &q.question,
                    &corpus_refs,
                    &fate_line_aff,
                    &report.hub_prefs,
                    &q_sig,
                )
            } else {
                crate::multihop_fate::decode_from_fate_structure(
                    &q.question,
                    &corpus_refs,
                    &fate_line_aff,
                    &report.hub_prefs,
                    &q_sig,
                )
            };
            if let Some((mh_ans, mh_path)) = fate_decode {
                answer = mh_ans;
                decode_path = if multihop_v2 {
                    format!("{decode_path}+{mh_path}")
                } else {
                    mh_path
                };
            }
        }

        results.push(NativeQaBatchItem {
            id: q.id.clone().unwrap_or_default(),
            answer,
            score_type: q.score_type.clone(),
            decode_path,
            tile_count: report.model_patch.tile_count,
            stem_cells: report.model_patch.stem_cell_count,
            tile_connections: report.model_patch.tile_connection_count,
            tile_avg_activation: report.model_patch.tile_avg_activation,
            stem_organ_clusters: report.model_patch.stem_organ_clusters.len(),
            correct: false,
            mcq_fate_cycles: mcq_fate_meta.as_ref().map(|m| m.0).unwrap_or(0),
            mcq_fate_stopped: mcq_fate_meta
                .as_ref()
                .map(|m| m.1.clone())
                .unwrap_or_default(),
            mcq_decompose_score: mcq_fate_meta.as_ref().map(|m| m.2).unwrap_or(0.0),
            mcq_aggregate_score: mcq_fate_meta.as_ref().map(|m| m.3).unwrap_or(0.0),
            fog_evidence: Some(fog_evidence),
            fog_decision: Some(fog_decision),
            retrieval_corpus_idx,
        });
    }

    NativeQaBatchResponse {
        mode: mode_label.to_string(),
        corpus_lines: req.corpus.len(),
        corpus_lines_ingested: req.corpus_ports.len(),
        incubation_cycles: req.incubation_cycles,
        qa_cycles: req.qa_cycles,
        incubation_tile_count: incubation_report.model_patch.tile_count,
        incubation_stem_cells: incubation_report.model_patch.stem_cell_count,
        incubation_tile_connections: incubation_report.model_patch.tile_connection_count,
        incubation_stem_organ_clusters: incubation_report.model_patch.stem_organ_clusters.len(),
        incubation_tile_avg_activation: incubation_report.model_patch.tile_avg_activation,
        results,
    }
}
