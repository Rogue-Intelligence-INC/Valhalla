//! Smoke test for valhalla-manifold-v1 / FateOrchestrator (no LM / no CUDA link at runtime).

use hub_f64::fate_orchestrator::{self, FateOrchestratorConfig, MANIFOLD_PROTOCOL};
use hub_f64::grinder::{default_port_cap, grind_text_hash};
use hub_f64::signal_ingress::{BodyTarget, IngressBudget, TriadSession};

fn main() {
    println!("=== valhalla-manifold-v1 smoke ===");

    let budget = IngressBudget::experiment();
    let mut session = TriadSession::with_body(budget.clone(), BodyTarget::Triad);
    let cap = default_port_cap();
    let corpus = [
        "Mercury is the closest planet to the Sun.",
        "Venus is the second planet from the Sun.",
        "Earth is the third planet from the Sun.",
    ];
    for line in &corpus {
        let ports = grind_text_hash(line, cap);
        session.emit_body_ports_with_line(&ports, Some(line));
    }

    let cfg = FateOrchestratorConfig {
        min_cycles: 3,
        soft_max_cycles: 16,
        hard_max_cycles: 32,
        fate_adapt_unlock: true,
    };
    let (outcome, snap) = fate_orchestrator::ingress_and_adapt(&mut session, 2, &cfg);

    println!("adapt cycles: {}", outcome.cycles_executed);
    println!("stopped: {}", outcome.stopped_reason);
    println!("aggregate_score: {:.4}", outcome.aggregate_score);
    println!("hub_pattern: {}", outcome.hub_pattern);
    println!(
        "manifold_protocol: {:?}",
        snap.manifold_protocol.as_deref()
    );
    println!("fragments: {}", snap.fragments.len());
    println!(
        "fate_history_len: {}",
        snap.fate_convergence
            .as_ref()
            .map(|f| f.preference_history.len())
            .unwrap_or(0)
    );
    println!("patch_dim: {}", snap.patch_vector.len());

    assert_eq!(snap.manifold_protocol.as_deref(), Some(MANIFOLD_PROTOCOL));
    assert!(!snap.fragments.is_empty());
    assert!(!snap.patch_vector.is_empty());

    let restored = TriadSession::from_incubation_snapshot(budget, BodyTarget::Triad, snap);
    let frag_b = fate_orchestrator::export_fragment(
        &restored,
        "restored",
        "roundtrip",
        fate_orchestrator::FragmentCloseMeta::default(),
    );
    let merged = fate_orchestrator::merge_fragments(
        &[frag_b.clone()],
        IngressBudget::experiment(),
        BodyTarget::Triad,
    );
    let merged_snap = merged.manifold_snapshot();
    assert_eq!(
        merged_snap.manifold_protocol.as_deref(),
        Some(MANIFOLD_PROTOCOL)
    );

    println!("roundtrip + merge: OK");
    println!("=== PASS ===");
}
