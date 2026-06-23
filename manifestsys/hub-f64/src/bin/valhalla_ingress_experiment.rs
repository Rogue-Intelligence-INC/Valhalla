//! Universal ingress experiment — compare encoding modes under resource budget.
//!
//! Usage:
//!   valhalla_ingress_experiment
//!   valhalla_ingress_experiment --mode text_stats
//!   valhalla_ingress_experiment --json embeddings.json
//!
//! Env: VALHALLA_INGRESS_MAX_ITEMS, VALHALLA_INGRESS_MAX_SIGNALS, ...

use std::env;
use std::fs;
use std::path::PathBuf;

use hub_f64::{
    EncodingMode, IngressBudget, IngressPayload, SMOKE_CORPUS, SmokeReport, TriadSession,
};

fn main() {
    let args: Vec<String> = env::args().collect();
    let budget = if args.iter().any(|a| a == "--experiment") {
        IngressBudget::experiment()
    } else {
        IngressBudget::from_env()
    };

    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return;
    }

    // Single mode run
    if let Some(pos) = args.iter().position(|a| a == "--mode") {
        if let Some(mode_str) = args.get(pos + 1) {
            let mode = EncodingMode::parse(mode_str);
            let r = SmokeReport::run_with_budget(mode, budget.clone());
            print_smoke(&r);
            return;
        }
    }

    // JSON f64 batch (e.g. transformer embeddings)
    if let Some(pos) = args.iter().position(|a| a == "--json") {
        if let Some(path) = args.get(pos + 1) {
            run_json_experiment(path, budget);
            return;
        }
    }

    // Default: compare all modes on smoke corpus under budget
    println!("=== Valhalla Universal Ingress Experiment ===");
    println!(
        "Budget: max_items={} max_signals={} per_item={} stride={} resource={}%",
        budget.max_items,
        budget.max_signals_total,
        budget.max_signals_per_item,
        budget.sample_stride,
        budget.resource_percent
    );
    println!();

    let modes = [
        ("legacy_char", None),
        ("char", Some(EncodingMode::Char)),
        ("byte_sample", Some(EncodingMode::ByteSample)),
        ("text_stats", Some(EncodingMode::TextStats)),
        ("direct_f64", Some(EncodingMode::DirectF64)),
        ("normalized_f64", Some(EncodingMode::NormalizedF64)),
    ];

    for (label, mode) in modes {
        let r = match mode {
            None => SmokeReport::run_legacy_char(),
            Some(m) => SmokeReport::run_with_budget(m, budget.clone()),
        };
        println!("--- mode: {label} ---");
        print_smoke(&r);
        println!();
    }

    // Demo: feed synthetic embedding vectors (no external model required)
    println!("--- mode: synthetic_embedding (384-d) ---");
    let mut session = TriadSession::new(budget.clone());
    for (i, text) in SMOKE_CORPUS.iter().enumerate() {
        let emb = synthetic_embedding(text, 384, i as u64);
        session.feed(IngressPayload::F64 { values: emb }, EncodingMode::NormalizedF64);
    }
    let syn = session.finalize("synthetic_embedding");
    println!(
        "  items={} signals={} hub={} tile={}/dim={:.1} stem={}/conn={}/dim={:.1} budget_hit={}",
        syn.stats.items_processed,
        syn.stats.signals_emitted,
        syn.hub_signals,
        syn.tile_completed,
        syn.tile_dim,
        syn.stem_cells,
        syn.stem_connections,
        syn.stem_dim,
        syn.stats.budget_exhausted
    );
}

fn run_json_experiment(path: &str, budget: IngressBudget) {
    let data = fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Cannot read {path}: {e}");
        std::process::exit(1);
    });
    let items: Vec<serde_json::Value> = if data.trim_start().starts_with('[') {
        serde_json::from_str(&data).unwrap_or_else(|e| {
            eprintln!("JSON array parse error: {e}");
            std::process::exit(1);
        })
    } else {
        data.lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| serde_json::from_str(l).expect("JSONL line parse"))
            .collect()
    };

    let mut session = TriadSession::new(budget.clone());
    for item in items {
        let payload = if let Some(vals) = item.get("values").and_then(|v| v.as_array()) {
            let values: Vec<f64> = vals
                .iter()
                .filter_map(|x| x.as_f64())
                .collect();
            IngressPayload::F64 { values }
        } else {
            IngressPayload::Json { value: item }
        };
        session.feed(payload, EncodingMode::Auto);
    }
    let r = session.finalize("json_file");
    println!("JSON ingress from {}", path);
    println!(
        "  items={} signals={} hub={} tile={} stem={} budget_hit={}",
        r.stats.items_processed,
        r.stats.signals_emitted,
        r.hub_signals,
        r.tile_completed,
        r.stem_cells,
        r.stats.budget_exhausted
    );
}

fn synthetic_embedding(text: &str, dim: usize, seed: u64) -> Vec<f64> {
    let mut out = vec![0.0f64; dim];
    let mut h = seed.wrapping_mul(0x9E3779B97F4A7C15);
    for b in text.bytes() {
        h = h.wrapping_mul(6364136223846793005).wrapping_add(b as u64);
    }
    for i in 0..dim {
        h = h.wrapping_mul(6364136223846793005).wrapping_add(i as u64);
        let x = ((h >> 33) as f64 / u32::MAX as f64) * 2.0 - 1.0;
        out[i] = x;
    }
    let norm: f64 = out.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-9);
    out.iter_mut().for_each(|x| *x /= norm);
    out
}

fn print_smoke(r: &SmokeReport) {
    println!(
        "  encoding={} items={} budget_hit={}",
        r.encoding_mode, r.ingress_items, r.budget_exhausted
    );
    println!(
        "  Hub: signals={} routed={:?} OK={}",
        r.hub_signals, r.hub_routed, r.hub_ok
    );
    println!(
        "  Tile: completed={} ops={} dim={:.2} OK={}",
        r.tile_completed, r.tile_ops, r.tile_dim, r.tile_ok
    );
    println!(
        "  StemCell: cells={} conn={} dim={:.2} OK={}",
        r.stem_cells, r.stem_connections, r.stem_dim, r.stem_ok
    );
    println!("  OVERALL: {}", if r.overall_pass { "PASS" } else { "FAIL" });
}

fn print_help() {
    eprintln!(
        "valhalla_ingress_experiment [--experiment] [--mode MODE] [--json FILE]

Modes: char, byte_sample, text_stats, direct_f64, normalized_f64, auto

Env:
  VALHALLA_INGRESS_MAX_ITEMS
  VALHALLA_INGRESS_MAX_SIGNALS
  VALHALLA_INGRESS_MAX_SIGNALS_PER_ITEM
  VALHALLA_INGRESS_SAMPLE_STRIDE
  VALHALLA_INGRESS_RESOURCE_PERCENT
"
    );
}
