//! Offline manifold pretrain → fragment library JSON (`valhalla-fragment-library-v1`).

use std::env;
use std::path::PathBuf;

use hub_f64::fate_orchestrator::{self, FateOrchestratorConfig, FRAGMENT_LIBRARY_VERSION};
use hub_f64::signal_ingress::BodyTarget;

fn main() {
    let args: Vec<String> = env::args().collect();
    let out = args
        .iter()
        .position(|a| a == "--out")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("reports/manifold_pretrain_library.json"));

    let corpus: Vec<String> = if let Some(i) = args.iter().position(|a| a == "--corpus") {
        args.get(i + 1)
            .map(|p| std::fs::read_to_string(p).unwrap_or_default())
            .unwrap_or_default()
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect()
    } else {
        vec![
            "Mercury is the closest planet to the Sun.".into(),
            "Venus is the second planet from the Sun.".into(),
            "Earth is the third planet from the Sun.".into(),
            "Mars is the fourth planet from the Sun.".into(),
        ]
    };

    let cfg = FateOrchestratorConfig {
        min_cycles: 3,
        soft_max_cycles: 24,
        hard_max_cycles: 48,
        fate_adapt_unlock: true,
    };

    println!("=== manifold pretrain ({FRAGMENT_LIBRARY_VERSION}) ===");
    println!("corpus lines: {}", corpus.len());

    let library = fate_orchestrator::pretrain_corpus_to_library(BodyTarget::Triad, &corpus, &cfg);

    if let Some(parent) = out.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    library.save_json(&out).expect("save library");

    println!("fragments: {}", library.fragments.len());
    println!(
        "adapt: {} cycles, stopped={}",
        library.pretrain_stats.adapt_cycles, library.pretrain_stats.stopped_reason
    );
    println!("aggregate_score: {:.4}", library.pretrain_stats.aggregate_score);
    println!("written: {}", out.display());

    let loaded = hub_f64::ManifoldFragmentLibrary::load_json(&out).expect("reload");
    let session = loaded.session_from_library(
        hub_f64::signal_ingress::IngressBudget::experiment(),
        BodyTarget::Triad,
    );
    let snap = session.manifold_snapshot();
    assert_eq!(snap.manifold_protocol.as_deref(), Some(fate_orchestrator::MANIFOLD_PROTOCOL));
    println!("reload + session: OK");
    println!("=== PASS ===");
}
