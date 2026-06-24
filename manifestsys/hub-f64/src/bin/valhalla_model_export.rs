//! Export Valhalla structure + pre-bake manifest (Hub / Tile / StemCell / Triad).
//!
//! Structure-only export (no weight changes):
//!   valhalla_model_export --body triad --corpus-file corpus.txt --grinder --manifest-out out/valhalla_manifest.json
//!
//! Legacy stdin feed:
//!   valhalla_model_export --body hub --mode normalized_f64 < corpus.txt

use std::fs;
use std::io::{self, BufRead};
use std::path::PathBuf;

use hub_f64::model_export::{
    build_prebake_manifest, export_body_grinder, write_manifest, IncubationIngress,
    MANIFEST_VERSION,
};
use hub_f64::patch_export::ExportDoc;
use hub_f64::{BodyTarget, EncodingMode, IngressBudget, IngressPayload, TriadSession};

fn usage() -> ! {
    eprintln!(
        "Usage: valhalla_model_export [options]\n\
         Bodies: hub | tile | stemcell | triad\n\
         --body triad              incubation body (default triad)\n\
         --corpus-file PATH        corpus lines (else stdin / smoke)\n\
         --grinder                 grinder ingress (hybrid-aligned, default for --corpus-file)\n\
         --mode normalized_f64     legacy text ingress (default without --grinder)\n\
         --cycles N                optimize cycles (default 1)\n\
         --manifest-out PATH       write valhalla-bake-1.0 manifest (structure only)\n\
         --base-model-id ID        recorded in manifest (default unknown)\n\
         --bake-mode legacy|full|total  recorded for Python bake step\n\
         --strength F              recorded bake strength (default 0.08)\n\
         --experiment              experiment ingress budget"
    );
    std::process::exit(2);
}

fn read_corpus(path: Option<&PathBuf>) -> Vec<String> {
    if let Some(p) = path {
        let raw = fs::read_to_string(p).unwrap_or_else(|e| {
            eprintln!("read corpus {}: {e}", p.display());
            std::process::exit(1);
        });
        return raw
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(str::to_string)
            .collect();
    }
    let stdin = io::stdin();
    let mut lines = Vec::new();
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
    lines
}

fn export_normalized(body: BodyTarget, cycles: usize, lines: &[String], experiment: bool) -> ExportDoc {
    let budget = if experiment {
        IngressBudget::experiment()
    } else {
        IngressBudget::from_env()
    };
    let mut session = TriadSession::with_body(budget, body);
    for text in lines {
        session.feed(
            IngressPayload::Text {
                content: text.clone(),
            },
            EncodingMode::NormalizedF64,
        );
    }
    let report = session.finalize_with_cycles("normalized_f64", cycles);
    ExportDoc {
        body: format!("{body:?}").to_ascii_lowercase(),
        encoding: "NormalizedF64".into(),
        report,
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let body_str = args
        .iter()
        .position(|a| a == "--body")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("triad");
    let body = BodyTarget::parse(body_str);

    let cycles = args
        .iter()
        .position(|a| a == "--cycles")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    let corpus_file = args
        .iter()
        .position(|a| a == "--corpus-file")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from);

    let use_grinder = args.iter().any(|a| a == "--grinder") || corpus_file.is_some();
    let experiment = args.iter().any(|a| a == "--experiment");

    let manifest_out = args
        .iter()
        .position(|a| a == "--manifest-out")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from);

    let base_model_id = args
        .iter()
        .position(|a| a == "--base-model-id")
        .and_then(|i| args.get(i + 1))
        .cloned()
        .unwrap_or_else(|| "unknown".into());

    let bake_mode = args
        .iter()
        .position(|a| a == "--bake-mode")
        .and_then(|i| args.get(i + 1))
        .cloned()
        .unwrap_or_else(|| "legacy".into());

    let strength = args
        .iter()
        .position(|a| a == "--strength")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.08);

    if args.iter().any(|a| a == "--help" || a == "-h") {
        usage();
    }

    let lines = read_corpus(corpus_file.as_ref());
    let body_label = format!("{body:?}").to_ascii_lowercase();

    eprintln!(
        "valhalla_model_export body={body_label} lines={} grinder={use_grinder} cycles={cycles}",
        lines.len()
    );

    let doc = if use_grinder {
        export_body_grinder(&body_label, cycles, &lines)
    } else {
        export_normalized(body, cycles, &lines, experiment)
    };

    if let Some(out) = &manifest_out {
        let ingress = if use_grinder {
            IncubationIngress::Grinder
        } else {
            IncubationIngress::NormalizedF64
        };
        let manifest = build_prebake_manifest(
            doc.clone(),
            &base_model_id,
            &bake_mode,
            strength,
            ingress,
            lines.len(),
        );
        let dir = if out.extension().is_some() {
            out.parent().unwrap_or(std::path::Path::new(".")).to_path_buf()
        } else {
            out.clone()
        };
        write_manifest(&manifest, &dir).unwrap_or_else(|e| {
            eprintln!("write manifest: {e}");
            std::process::exit(1);
        });
        eprintln!(
            "wrote manifest {} ({MANIFEST_VERSION})",
            dir.join("valhalla_manifest.json").display()
        );
    }

    println!("{}", serde_json::to_string_pretty(&doc).expect("json"));
}
