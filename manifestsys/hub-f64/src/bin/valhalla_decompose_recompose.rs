//! The same material, two paths: decomposed then composed into a new model, or fused into the same model.
//! Watch whether what comes back is the body's own words. Higher scores or more blocks do not count as counter-evidence.

use hub_f64::free_multimodal_fate::mix_ports_weighted;
use hub_f64::grinder::grind_text_hash;
use hub_f64::model_fate::{run_model_fate_pipeline, ModelDecomposePlan, ModelDecomposeShard, MODEL_DECOMPOSE_PROTOCOL};
use hub_f64::{BodyTarget, IngressBudget, TriadSession};

fn shard(label: &str, text: &str) -> ModelDecomposeShard {
    ModelDecomposeShard {
        label: label.to_string(),
        body: "triad".to_string(),
        phase: "decompose".to_string(),
        ports: grind_text_hash(text, 32),
    }
}

fn plan(name: &str, texts: &[(&str, &str)]) -> ModelDecomposePlan {
    let shards: Vec<_> = texts.iter().map(|(label, text)| shard(label, text)).collect();
    let refs: Vec<_> = shards.iter().map(|item| &item.ports).collect();
    let weights = vec![1.0 / refs.len() as f64; refs.len()];
    let whole = if refs.len() == 1 {
        refs[0].clone()
    } else {
        mix_ports_weighted(&refs, &weights)
    };
    ModelDecomposePlan {
        protocol: MODEL_DECOMPOSE_PROTOCOL.to_string(),
        base_model: name.to_string(),
        layer_count: shards.len(),
        whole_ports: whole,
        shards,
    }
}

fn cosine(left: &[f64], right: &[f64]) -> f64 {
    let n = left.len().min(right.len());
    if n == 0 {
        return 0.0;
    }
    let mut dot = 0.0;
    let mut ll = 0.0;
    let mut rr = 0.0;
    for i in 0..n {
        dot += left[i] * right[i];
        ll += left[i] * left[i];
        rr += right[i] * right[i];
    }
    if ll == 0.0 || rr == 0.0 {
        0.0
    } else {
        dot / (ll.sqrt() * rr.sqrt())
    }
}

fn run_arm(name: &str, texts: &[(&str, &str)]) -> (usize, usize, f64, f64, Vec<f64>, String) {
    let mut session = TriadSession::with_body(IngressBudget::incubation(), BodyTarget::Triad);
    let outcome = run_model_fate_pipeline(&mut session, &plan(name, texts), name, 1);
    let report = &outcome.report;
    (
        report.tile_completed,
        report.stem_cells,
        outcome.decompose_score,
        outcome.model_closure_score,
        report.model_patch.patch_vector.clone(),
        outcome.stopped_reason,
    )
}

fn main() {
    let poem = "錐刀既無棄，轉化日已盈";
    let sum = "6 * 8 = <<6*8=48>>48 orange crayons";
    let (new_tiles, new_cells, new_score, new_closure, new_patch, new_stop) = run_arm("new-model", &[("poem", poem)]);
    let (fuse_tiles, fuse_cells, fuse_score, fuse_closure, fuse_patch, fuse_stop) =
        run_arm("fused-model", &[("poem", poem), ("formula", sum)]);
    let alike = cosine(&new_patch, &fuse_patch);
    println!(
        "{}",
        serde_json::json!({
            "comparison": "decomposed into a new model vs fused into one model",
            "materials": [poem, sum],
            "new_model": {
                "tiles": new_tiles,
                "stem_cells": new_cells,
                "decompose_score": new_score,
                "closure_score": new_closure,
                "patch_dim": new_patch.len(),
                "stop": new_stop,
            },
            "fused_model": {
                "tiles": fuse_tiles,
                "stem_cells": fuse_cells,
                "decompose_score": fuse_score,
                "closure_score": fuse_closure,
                "patch_dim": fuse_patch.len(),
                "stop": fuse_stop,
            },
            "patch_cosine_two_paths": alike,
            "body_returned": false,
            "counter_evidence_cultivation": false,
        })
    );
}
