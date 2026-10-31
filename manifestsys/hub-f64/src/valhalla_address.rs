//! Ingress/egress adapter: absolute corpus lines stay here; Fate scores inside.
//!
//! The scheduler uses the existing relative-address scorer in `fate_adaptive_algebra`.
//! Corpus text is kept and citation tags are stripped before the container sees it.

use serde::{Deserialize, Serialize};

use crate::fate_adaptive_algebra::{
    line_query_score, query_tokens_from_q, strip_all_citations, FateAlgebraState, FateRelAddr,
};
use crate::fate_orchestrator::SnapshotFragment;
use crate::valhalla_base::ValhallaBaseQuestion;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AddressBook {
    pub lines: Vec<String>,
}

impl AddressBook {
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn scrubbed_corpus(&self) -> Vec<String> {
        self.lines
            .iter()
            .map(|line| strip_all_citations(line))
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct QuestionProbe {
    pub tokens: Vec<String>,
    pub addr: FateRelAddr,
}

pub fn convert_interface_corpus(lines: &[String], _state: &FateAlgebraState) -> AddressBook {
    AddressBook {
        lines: lines.to_vec(),
    }
}

pub fn extend_address_book(
    prev: &AddressBook,
    new_raw: &[String],
    _state: &FateAlgebraState,
) -> AddressBook {
    let mut lines = prev.lines.clone();
    lines.extend(new_raw.iter().cloned());
    AddressBook { lines }
}

pub fn convert_interface_question(
    q: &ValhallaBaseQuestion,
    state: &FateAlgebraState,
) -> QuestionProbe {
    let tokens = query_tokens_from_q(q);
    let addr = FateRelAddr::from_query(&tokens, state);
    QuestionProbe { tokens, addr }
}

pub fn fate_schedule_affinity(
    book: &AddressBook,
    probe: &QuestionProbe,
    _frags: &[SnapshotFragment],
    state: &FateAlgebraState,
) -> Vec<f64> {
    book.lines
        .iter()
        .enumerate()
        .map(|(i, line)| state.fate_signal_line_score(&probe.tokens, line, i, 0.0))
        .collect()
}

/// Down-weight schedule peaks that share no query tokens with the line.
pub fn content_gate_schedule(schedule: &[f64], lines: &[String], tokens: &[String]) -> Vec<f64> {
    schedule
        .iter()
        .zip(lines.iter())
        .map(|(score, line)| {
            let overlap = line_query_score(line, tokens);
            if overlap <= 0.0 {
                score * 0.15
            } else {
                *score
            }
        })
        .collect()
}

pub fn apply_fate_schedule_to_affinity(affinity: &mut Vec<f64>, schedule: &[f64]) {
    if affinity.len() < schedule.len() {
        affinity.resize(schedule.len(), 0.0);
    }
    for (slot, score) in affinity.iter_mut().zip(schedule.iter()) {
        *slot = score.clamp(0.0, 1.0);
    }
}
