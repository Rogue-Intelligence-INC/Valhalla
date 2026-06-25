//! Model decompose → Fate multi-cycle adapt → export structure for **new checkpoint bake**.
//!
//! Input: JSON from `tools/model_decompose/shred_qwen_weights.py`
//! Output: ExportDoc JSON (stdout) + optional pre-bake manifest directory
//!
//!   valhalla_model_fate --shards qwen_shards.json --manifest-out checkpoints/model_fate_staging

use std::fs;
use std::path::PathBuf;

use hub_f64::model_export::{build_model_fate_manifest, write_manifest, MANIFEST_VERSION};
use hub_f64::model_fate::{load_model_decompose_plan, run_model_fate_pipeline, MODEL_DECOMPOSE_PROTOCOL};
use hub_f64::patch_export::ExportDoc;
use hub_f64::{BodyTarget, IngressBudget, TriadSession};

fn usage() -> ! {
    eprintln!(
        "Usage: valhalla_model_fate --shards PATH [options]\n\
         --shards PATH           model decompose plan (shred_qwen_weights.py)\n\
         --manifest-out PATH     write valhalla_manifest.json (structure only, pre-bake)\n\
         --base-model-id ID      recorded in manifest (default from plan)\n\
         --bake-mode legacy|full|total  recorded for Python bake (default full)\n\
         --strength F            bake strength hint (default 0.08)\n\
         --cycles-per-shard N    optimize cycles per weight shard (default 2)\n\
         Env: VALHALLA_MODEL_FATE_MIN/MAX/HARD_MAX_CYCLES"
    );
    std::process::exit(2);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let shards_path = args
        .iter()
        .position(|a| a == "--shards")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from);
    let manifest_out = args
        .iter()
        .position(|a| a == "--manifest-out")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from);
    let base_model_id = args
        .iter()
        .position(|a| a == "--base-model-id")
        .and_then(|i| args.get(i + 1))
        .cloned();
    let bake_mode = args
        .iter()
        .position(|a| a == "--bake-mode")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("full");
    let strength = args
        .iter()
        .position(|a| a == "--strength")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.08);
    let cycles_per_shard = args
        .iter()
        .position(|a| a == "--cycles-per-shard")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(2);

    let shards_path = shards_path.unwrap_or_else(|| usage());
    let raw = fs::read_to_string(&shards_path).unwrap_or_else(|e| {
        eprintln!("read {}: {e}", shards_path.display());
        std::process::exit(1);
    });
    let plan = load_model_decompose_plan(&raw).unwrap_or_else(|e| {
        eprintln!("parse model decompose plan: {e}");
        std::process::exit(1);
    });
    if plan.protocol != MODEL_DECOMPOSE_PROTOCOL {
        eprintln!(
            "warn: expected protocol {MODEL_DECOMPOSE_PROTOCOL}, got {}",
            plan.protocol
        );
    }

    let base_id = base_model_id.unwrap_or_else(|| plan.base_model.clone());
    eprintln!(
        "valhalla_model_fate base={base_id} layers={} shards={}",
        plan.layer_count,
        plan.shards.len()
    );

    let budget = IngressBudget::incubation();
    let mut session = TriadSession::with_body(budget, BodyTarget::Triad);
    let outcome = run_model_fate_pipeline(
        &mut session,
        &plan,
        "model_decompose_fate",
        cycles_per_shard,
    );

    eprintln!(
        "model_fate: adapt_rounds={} stopped={} closure={:.3} tiles={} stem={}",
        outcome.adaptation_rounds,
        outcome.stopped_reason,
        outcome.model_closure_score,
        outcome.report.tile_completed,
        outcome.report.stem_cells,
    );

    let export_doc = ExportDoc {
        body: "triad".into(),
        encoding: "ModelDecomposeFate".into(),
        report: outcome.report.clone(),
    };

    if let Some(ref out_dir) = manifest_out {
        let manifest = build_model_fate_manifest(
            export_doc.clone(),
            &base_id,
            plan.layer_count,
            bake_mode,
            strength,
            &outcome,
            &plan.protocol,
        );
        write_manifest(&manifest, out_dir).unwrap_or_else(|e| {
            eprintln!("write manifest: {e}");
            std::process::exit(1);
        });
        fs::write(
            out_dir.join("valhalla_export.json"),
            serde_json::to_string_pretty(&export_doc).unwrap(),
        )
        .unwrap_or_else(|e| {
            eprintln!("write export: {e}");
            std::process::exit(1);
        });
        fs::write(
            out_dir.join("model_fate_outcome.json"),
            serde_json::to_string_pretty(&outcome).unwrap(),
        )
        .unwrap_or_else(|e| {
            eprintln!("write outcome: {e}");
            std::process::exit(1);
        });
        eprintln!(
            "manifest {MANIFEST_VERSION} → {}/valhalla_manifest.json (pre-bake)",
            out_dir.display()
        );
    }

    let payload = serde_json::json!({
        "export_doc": export_doc,
        "model_fate": outcome,
    });
    println!("{}", serde_json::to_string_pretty(&payload).unwrap());
}
