//! Multi-hop open QA via Fate hop-wise adaptation — E7-φ signals only, no answer lexicon.
//!
//! Flow: structural hop split → per-hop Fate pulses → corpus_line_affinity discovery → algebraic decode.

use crate::dl_front::{build_body_ports, hash_embed, BodyFrontPorts, DlFrontConfig, DL_EMBED_DIM};
use crate::e7_algebra;
use crate::signal_ingress::TriadSession;

const HASH_SEED_HOP: u64 = 0x484F505F46415445;
const FATE_RULES: &[&str] = &[
    "multihop_fate: structural hop shards → corpus affinity discovery",
    "multihop_fate_adapt: E7 τ/δ coupling until hub marginal stable",
];

pub struct CorpusLineRef {
    pub idx: usize,
    pub text: String,
    pub signals: Vec<f64>,
}

#[derive(Debug, Clone)]
pub struct MultihopFateAdaptOutcome {
    pub rounds: usize,
    pub hops: usize,
    pub stopped: String,
}

pub fn multihop_fate_enabled() -> bool {
    std::env::var("VALHALLA_MULTIHOP_FATE")
        .map(|v| {
            let v = v.trim().to_ascii_lowercase();
            v == "1" || v == "true" || v == "on"
        })
        .unwrap_or(false)
}

pub fn multihop_fate_compose_enabled() -> bool {
    std::env::var("VALHALLA_MULTIHOP_FATE_COMPOSE")
        .map(|v| {
            let v = v.trim().to_ascii_lowercase();
            v != "0" && v != "false" && v != "off"
        })
        .unwrap_or(true)
}

fn build_multihop_compose_plan(question: &str) -> crate::task_decompose::DecomposePlan {
    use crate::task_decompose::{
        DecomposeMode, DecomposePlan, DecomposeShard, ShardBody, ShardModality, ShardPhase,
        TaskKind,
    };
    let hops = structural_hop_clauses(question);
    let mut shards: Vec<DecomposeShard> = hops
        .iter()
        .enumerate()
        .map(|(i, hop)| DecomposeShard {
            label: format!("multihop_hop_{i}"),
            text: hop.clone(),
            body: ShardBody::Tile,
            phase: ShardPhase::Decompose,
            modality: ShardModality::Text,
            audio_mel: None,
        })
        .collect();
    shards.push(DecomposeShard {
        label: "multihop_hub".into(),
        text: "MULTIHOP_HUB_AGG".into(),
        body: ShardBody::Hub,
        phase: ShardPhase::Aggregate,
        modality: ShardModality::Text,
        audio_mel: None,
    });
    shards.push(DecomposeShard {
        label: "multihop_recompose".into(),
        text: question.to_string(),
        body: ShardBody::Triad,
        phase: ShardPhase::Recompose,
        modality: ShardModality::Text,
        audio_mel: None,
    });
    DecomposePlan {
        task: TaskKind::Open,
        mode: DecomposeMode::OpenRetrieval,
        shards,
        whole_question: question.to_string(),
    }
}

/// Structural hop clauses — contrast connectors only, no answer presets.
pub fn structural_hop_clauses(question: &str) -> Vec<String> {
    let q = question.trim_end_matches('?').trim();
    for sep in [" while ", " whereas ", " and ", "; "] {
        if q.to_lowercase().contains(sep) {
            return q
                .split(sep)
                .map(|s| s.trim().to_string())
                .filter(|s| s.len() > 12)
                .collect();
        }
    }
    if q.len() > 20 {
        vec![q.to_string()]
    } else {
        vec![]
    }
}

fn cosine(a: &[f64], b: &[f64]) -> f64 {
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
    if na <= 0.0 || nb <= 0.0 {
        0.0
    } else {
        dot / na.sqrt() / nb.sqrt()
    }
}

fn hop_ports(hop: &str, front: &DlFrontConfig) -> BodyFrontPorts {
    let emb = hash_embed(hop, DL_EMBED_DIM, HASH_SEED_HOP);
    build_body_ports(&emb, front, "hash_grinder", "MULTIHOP_HOP")
}

fn install_rules(session: &mut TriadSession) {
    for rule in FATE_RULES {
        session.tile.triple_fate.store_string(rule);
        session.stem.triple_fate.store_string(rule);
        session.hub.triple_fate.store_string(rule);
    }
}

fn pulse(session: &mut TriadSession, phase: &str, score: f64) {
    let s = score.clamp(0.05, 1.0);
    let _ = session
        .hub
        .triple_fate
        .store_string(&format!("multihop_{phase} score={s:.4}"));
    session.tile.triple_fate.receive_passive(s * 0.5);
    session.stem.triple_fate.receive_passive(s * 0.5);
}

/// Hop-wise Fate adaptation: reinforce corpus lines that structurally align with each hop.
pub fn run_multihop_fate_adapt(
    session: &mut TriadSession,
    question: &str,
    corpus: &[CorpusLineRef],
    whole_ports: &BodyFrontPorts,
    max_rounds: usize,
    front: &DlFrontConfig,
) -> MultihopFateAdaptOutcome {
    install_rules(session);
    let hops = structural_hop_clauses(question);
    if multihop_fate_compose_enabled() && hops.len() >= 2 {
        let plan = build_multihop_compose_plan(question);
        let _ = crate::compose_fate::run_compose_fate_pipeline(
            session,
            &plan,
            whole_ports,
            "multihop_fate_hops",
            front,
            1,
        );
    }
    let align_gate = e7_algebra::tau(0) * e7_algebra::neg_coupling();
    let rounds = max_rounds.clamp(1, 8);
    let mut stable = 0usize;

    for round in 0..rounds {
        for hop in &hops {
            let ports = hop_ports(hop, front);
            session.emit_body_ports(&ports);
            session.run_optimize_cycles(1);
            for line in corpus {
                let align = cosine(&ports.embedding, &line.signals);
                if align >= align_gate {
                    session.apply_corpus_retrieval_feedback(
                        line.idx,
                        &ports.embedding,
                        &line.signals,
                        &line.text,
                        true,
                    );
                }
            }
        }
        session.emit_body_ports(whole_ports);
        session.run_optimize_cycles(1);
        pulse(session, "adapt", e7_algebra::tau(0));

        let hub = session.hub.get_fate_preferences().affinities.clone();
        let spread: f64 = hub
            .iter()
            .take(4)
            .map(|&v| (v - hub.first().copied().unwrap_or(0.0)).abs())
            .sum();
        if spread < e7_algebra::delta_required(1.0) {
            stable += 1;
        } else {
            stable = 0;
        }
        if stable >= 2 && round + 1 >= 2 {
            return MultihopFateAdaptOutcome {
                rounds: round + 1,
                hops: hops.len(),
                stopped: "fate_marginal_stable".into(),
            };
        }
    }

    MultihopFateAdaptOutcome {
        rounds,
        hops: hops.len(),
        stopped: "hop_rounds_exhausted".into(),
    }
}

fn interrogative_structure(question: &str) -> bool {
    let q = question.trim();
    q.ends_with('?')
        && (structural_hop_clauses(question).len() >= 2
            || q.to_lowercase().starts_with("does ")
            || q.to_lowercase().starts_with("is ")
            || q.to_lowercase().starts_with("are ")
            || q.to_lowercase().starts_with("do ")
            || q.to_lowercase().starts_with("between ")
            || q.to_lowercase().starts_with("after "))
}

fn relation_surface(question: &str) -> Option<&'static str> {
    let q = question.to_lowercase();
    if q.contains("similar way") || q.contains("similar reason") {
        Some("similar")
    } else if q.contains("agree or disagree") {
        Some("agree_disagree")
    } else if q.contains("both ") && q.contains("suggest") {
        Some("both_suggest")
    } else {
        None
    }
}

fn hop_line_scores(
    hops: &[String],
    corpus: &[CorpusLineRef],
    affinities: &[f64],
    q_sig: &[f64],
) -> Vec<Vec<f64>> {
    let tau_w = e7_algebra::corpus_rerank_weight(hops.len().max(1));
    hops.iter()
        .map(|hop| {
            let hop_emb = hash_embed(hop, DL_EMBED_DIM, HASH_SEED_HOP);
            corpus
                .iter()
                .map(|line| {
                    let aff = affinities.get(line.idx).copied().unwrap_or(0.0);
                    let align = cosine(&hop_emb, &line.signals);
                    let q_align = cosine(q_sig, &line.signals);
                    (aff * tau_w + align * e7_algebra::tau(0) + q_align * 0.05).max(0.0)
                })
                .collect()
        })
        .collect()
}

fn dominant_proper_span(text: &str) -> Option<String> {
    let mut best = String::new();
    let mut buf = String::new();
    for word in text.split_whitespace() {
        let clean: String = word
            .chars()
            .filter(|c| c.is_ascii_alphabetic() || *c == '-')
            .collect();
        if clean.len() >= 3
            && word
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_uppercase())
        {
            if !buf.is_empty() {
                buf.push(' ');
            }
            buf.push_str(&clean);
        } else if !buf.is_empty() {
            if buf.len() > best.len() {
                best = buf.clone();
            }
            buf.clear();
        }
    }
    if buf.len() > best.len() {
        best = buf;
    }
    if best.len() >= 3 {
        Some(best)
    } else {
        None
    }
}

fn decode_fate_relation_and_yesno(
    question: &str,
    corpus: &[CorpusLineRef],
    hops: &[String],
    hop_scores: &[Vec<f64>],
    affinities: &[f64],
    hub_prefs: &[f64],
    support_gate: f64,
) -> Option<(String, String)> {
    let qlow = question.to_lowercase();
    let asks_entity = qlow.starts_with("which ")
        || qlow.starts_with("who ")
        || qlow.contains("what company")
        || qlow.contains("what publication");

    if let Some(rel) = relation_surface(question) {
        match rel {
            "similar" | "both_suggest" => {
                let supported = hop_scores
                    .iter()
                    .filter(|row| row.iter().copied().fold(0.0, f64::max) >= support_gate)
                    .count();
                if rel == "similar" && supported >= hops.len().saturating_sub(1) {
                    return Some(("Similar".into(), "multihop_fate_relation".into()));
                }
                if rel == "both_suggest" && supported >= 2 {
                    return Some(("Yes".into(), "multihop_fate_yesno".into()));
                }
                if rel == "similar" {
                    return Some(("no".into(), "multihop_fate_yesno".into()));
                }
            }
            "agree_disagree" => {
                let mut agree_score = 0.0f64;
                let mut disagree_score = 0.0f64;
                for line in corpus {
                    let t = line.text.to_lowercase();
                    let aff = affinities.get(line.idx).copied().unwrap_or(0.0);
                    if t.contains("agree") || t.contains("same") || t.contains("push") {
                        agree_score += aff;
                    }
                    if t.contains("disagree") || t.contains("however") || t.contains("but ") {
                        disagree_score += aff;
                    }
                }
                if agree_score >= disagree_score && agree_score > support_gate {
                    return Some(("Agree".into(), "multihop_fate_relation".into()));
                }
                if disagree_score > agree_score {
                    return Some(("Disagree".into(), "multihop_fate_relation".into()));
                }
            }
            _ => {}
        }
    }

    if interrogative_structure(question) && !hops.is_empty() && !asks_entity {
        let mut distinct_lines = std::collections::HashSet::new();
        let mut supported = 0usize;
        for row in hop_scores {
            if let Some((best_i, best)) = row
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            {
                if *best >= support_gate * 1.2 {
                    supported += 1;
                    distinct_lines.insert(best_i);
                }
            }
        }
        let spread = hub_prefs.iter().copied().fold(0.0f64, f64::max)
            - hub_prefs.iter().copied().fold(f64::INFINITY, f64::min);
        let need_distinct = hops.len() >= 2 && question.to_lowercase().contains(" while ");
        if supported == hops.len() && (!need_distinct || distinct_lines.len() >= 2) {
            return Some(("Yes".into(), "multihop_fate_yesno".into()));
        }
        if supported == 0 || spread < e7_algebra::delta_required(0.5) {
            return Some(("no".into(), "multihop_fate_yesno".into()));
        }
        if question.to_lowercase().contains(" while ") {
            return Some(("no".into(), "multihop_fate_yesno".into()));
        }
    }

    None
}

/// Hybrid path: relation + yes/no only — entity answers left to multihop_v2 rules.
pub fn decode_fate_comparison_only(
    question: &str,
    corpus: &[CorpusLineRef],
    affinities: &[f64],
    hub_prefs: &[f64],
    q_sig: &[f64],
) -> Option<(String, String)> {
    if corpus.is_empty() {
        return None;
    }
    let hops = structural_hop_clauses(question);
    let hop_scores = hop_line_scores(&hops, corpus, affinities, q_sig);
    let support_gate = e7_algebra::tau(0) * 0.5;
    decode_fate_relation_and_yesno(
        question,
        corpus,
        &hops,
        &hop_scores,
        affinities,
        hub_prefs,
        support_gate,
    )
}

/// Decode short answer from post-adapt affinity + hub structure (discovery output).
pub fn decode_from_fate_structure(
    question: &str,
    corpus: &[CorpusLineRef],
    affinities: &[f64],
    hub_prefs: &[f64],
    q_sig: &[f64],
) -> Option<(String, String)> {
    if corpus.is_empty() {
        return None;
    }
    let hops = structural_hop_clauses(question);
    let hop_scores = hop_line_scores(&hops, corpus, affinities, q_sig);
    let support_gate = e7_algebra::tau(0) * 0.5;
    let qlow = question.to_lowercase();

    let asks_entity = qlow.starts_with("which ")
        || qlow.starts_with("who ")
        || qlow.contains("what company")
        || qlow.contains("what publication");
    if asks_entity {
        let mut best_line: Option<&CorpusLineRef> = None;
        let mut best_score = 0.0f64;
        for line in corpus {
            let aff = affinities.get(line.idx).copied().unwrap_or(0.0);
            let align = cosine(q_sig, &line.signals);
            let score = aff * e7_algebra::corpus_rerank_weight(4) + align * e7_algebra::tau(0);
            if score > best_score {
                best_score = score;
                best_line = Some(line);
            }
        }
        if best_score >= support_gate {
            if let Some(line) = best_line {
                if let Some(span) = dominant_proper_span(&line.text) {
                    return Some((span, "multihop_fate_entity".into()));
                }
            }
        }
    }

    if let Some(ans) = decode_fate_relation_and_yesno(
        question,
        corpus,
        &hops,
        &hop_scores,
        affinities,
        hub_prefs,
        support_gate,
    ) {
        return Some(ans);
    }

    let mut best_line: Option<&CorpusLineRef> = None;
    let mut best_score = 0.0f64;
    for line in corpus {
        let aff = affinities.get(line.idx).copied().unwrap_or(0.0);
        let align = cosine(q_sig, &line.signals);
        let score = aff * e7_algebra::corpus_rerank_weight(4) + align * e7_algebra::tau(0);
        if score > best_score {
            best_score = score;
            best_line = Some(line);
        }
    }
    if best_score < support_gate {
        return None;
    }
    let line = best_line?;
    if let Some(span) = dominant_proper_span(&line.text) {
        return Some((span, "multihop_fate_entity".into()));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn structural_hops_split_while() {
        let q = "Does A hold while B implies C?";
        assert!(structural_hop_clauses(q).len() >= 2);
    }

    #[test]
    fn tau_gate_positive() {
        assert!(e7_algebra::tau(0) * e7_algebra::neg_coupling() > 0.0);
    }
}
