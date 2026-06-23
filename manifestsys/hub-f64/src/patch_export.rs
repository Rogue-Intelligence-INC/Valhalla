//! In-process Valhalla structure export for Tier A patch (no subprocess).

use crate::grinder::{default_port_cap, grind_text_hash};
use crate::signal_ingress::{
    BodyTarget, EncodingMode, IngressBudget, IngressPayload, TriadFeedReport, TriadIncubationSnapshot,
    TriadSession,
};

pub const PATCH_ANCHOR: &str =
    "Valhalla incubation anchor: structure from Hub Tile StemCell Fate converges into model weights.";

#[derive(Debug, Clone, serde::Serialize)]
pub struct ExportDoc {
    pub body: String,
    pub encoding: String,
    pub report: TriadFeedReport,
}

pub fn export_patch_from_corpus(body: &str, cycles: usize, corpus: &[String]) -> ExportDoc {
    let target = BodyTarget::parse(body);
    let budget = IngressBudget::experiment();
    let mut session = TriadSession::with_body(budget, target);
    for text in corpus {
        session.feed(
            IngressPayload::Text {
                content: text.clone(),
            },
            EncodingMode::NormalizedF64,
        );
    }
    let report = session.finalize_with_cycles("normalized_f64", cycles);
    ExportDoc {
        body: body.to_ascii_lowercase(),
        encoding: "NormalizedF64".into(),
        report,
    }
}

/// Grinder ingress incubation snapshot (matches native_qa batch Phase-1).
pub fn build_grinder_incubation_snapshot(
    body: &str,
    corpus: &[String],
    incubation_cycles: usize,
) -> TriadIncubationSnapshot {
    let target = BodyTarget::parse(body);
    let budget = IngressBudget::experiment();
    let mut session = TriadSession::with_body(budget, target);
    let cap = default_port_cap();
    for line in corpus {
        let ports = grind_text_hash(line, cap);
        session.emit_body_ports(&ports);
    }
    session.run_optimize_cycles(incubation_cycles);
    session.incubation_snapshot()
}
