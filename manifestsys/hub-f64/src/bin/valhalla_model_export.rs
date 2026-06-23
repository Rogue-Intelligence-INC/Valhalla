//! Export Valhalla structure as JSON for direct LLM patching.
//!
//!   echo "hello" | valhalla_model_export --body hub
//!   valhalla_model_export --body tile --mode normalized_f64 < corpus.txt

use std::io::{self, BufRead};

use hub_f64::{BodyTarget, EncodingMode, IngressBudget, IngressPayload, TriadSession};
use serde::Serialize;

#[derive(Serialize)]
struct ExportDoc {
    body: String,
    encoding: String,
    report: hub_f64::TriadFeedReport,
}

fn main() {
    let args: std::env::Args = std::env::args();
    let args: Vec<String> = args.collect();

    let mode = args
        .iter()
        .position(|a| a == "--mode")
        .and_then(|i| args.get(i + 1))
        .map(|s| EncodingMode::parse(s))
        .unwrap_or(EncodingMode::NormalizedF64);

    let body = args
        .iter()
        .position(|a| a == "--body")
        .and_then(|i| args.get(i + 1))
        .map(|s| BodyTarget::parse(s))
        .unwrap_or(BodyTarget::Triad);

    let optimize_cycles = args
        .iter()
        .position(|a| a == "--cycles")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(1);

    let budget = if args.iter().any(|a| a == "--experiment") {
        IngressBudget::experiment()
    } else {
        IngressBudget::from_env()
    };

    let stdin = io::stdin();
    let mut lines: Vec<String> = Vec::new();
    for line in stdin.lock().lines() {
        let line = line.unwrap_or_default();
        let t = line.trim();
        if !t.is_empty() && !t.starts_with('#') {
            lines.push(t.to_string());
        }
    }

    if lines.is_empty() {
        for s in hub_f64::SMOKE_CORPUS {
            lines.push((*s).to_string());
        }
    }

    let mut session = TriadSession::with_body(budget, body);
    for text in &lines {
        session.feed(
            IngressPayload::Text {
                content: text.clone(),
            },
            mode,
        );
    }

    let mode_label = format!("{:?}", mode);
    let report = session.finalize_with_cycles(&mode_label, optimize_cycles);

    let doc = ExportDoc {
        body: format!("{:?}", body).to_ascii_lowercase(),
        encoding: mode_label,
        report,
    };

    println!(
        "{}",
        serde_json::to_string_pretty(&doc).expect("json serialize")
    );
}
