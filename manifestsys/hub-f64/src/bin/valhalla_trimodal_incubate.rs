//! Persistent tri-modal free fusion (LLM + TTS + image-gen) → export for bake (v3).

use std::fs;
use std::path::PathBuf;

use hub_f64::dl_front::{encode_modality, BodyFrontPorts, DlFrontConfig, ModalityInput};
use hub_f64::free_multimodal_fate::{
    run_persistent_corpus_fusion_tri, TriModalSample, FREE_MULTIMODAL_FUSION_V3,
};
use hub_f64::model_export::{build_prebake_manifest, write_manifest, IncubationIngress, MANIFEST_VERSION};
use hub_f64::patch_export::ExportDoc;

#[derive(serde::Deserialize)]
struct IncubateRequest {
    samples: Vec<TriModalSample>,
    #[serde(default)]
    base_model_id: String,
    #[serde(default = "default_strength")]
    strength: f64,
    #[serde(default = "default_bake_mode")]
    bake_mode: String,
}

fn default_strength() -> f64 {
    0.10
}

fn default_bake_mode() -> String {
    "full".into()
}

fn ports_from_text(text: &str, front: &DlFrontConfig) -> BodyFrontPorts {
    encode_modality(
        &ModalityInput::Text {
            content: text.to_string(),
        },
        front,
    )
}

fn ports_from_audio(vals: &[f64], front: &DlFrontConfig) -> BodyFrontPorts {
    encode_modality(
        &ModalityInput::AudioFeatures {
            values: vals.to_vec(),
        },
        front,
    )
}

fn ports_from_image(vals: &[f64], front: &DlFrontConfig) -> BodyFrontPorts {
    encode_modality(
        &ModalityInput::ImageFeatures {
            values: vals.to_vec(),
        },
        front,
    )
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let manifest_out = args
        .iter()
        .position(|a| a == "--manifest-out")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from);

    let raw = std::io::read_to_string(std::io::stdin()).unwrap_or_default();
    let req: IncubateRequest = serde_json::from_str(&raw).unwrap_or_else(|e| {
        eprintln!("invalid JSON stdin: {e}");
        std::process::exit(1);
    });

    if req.samples.is_empty() {
        eprintln!("need samples");
        std::process::exit(1);
    }

    let base_id = if req.base_model_id.is_empty() {
        "Qwen/Qwen2.5-0.5B-Instruct".to_string()
    } else {
        req.base_model_id
    };

    let front = DlFrontConfig::default();
    let triples: Vec<(BodyFrontPorts, BodyFrontPorts, BodyFrontPorts)> = req
        .samples
        .iter()
        .map(|s| {
            (
                ports_from_text(&s.text, &front),
                ports_from_audio(&s.audio_features, &front),
                ports_from_image(&s.image_features, &front),
            )
        })
        .collect();

    eprintln!(
        "valhalla_trimodal_incubate v3 samples={} deep={} rounds/sample={}",
        triples.len(),
        hub_f64::free_multimodal_fate::free_multimodal_deep_enabled(),
        hub_f64::free_multimodal_fate::free_multimodal_rounds_per_sample(),
    );

    let outcome = run_persistent_corpus_fusion_tri(&triples, "persistent_trimodal_fusion_v3");
    let mp = &outcome.report.model_patch;
    eprintln!(
        "  fused: tiles={} stem={} signals={} aggregate={:.3} weights={:?}",
        outcome.report.tile_completed,
        outcome.report.stem_cells,
        outcome.report.stats.signals_emitted,
        outcome.aggregate_score,
        outcome.mix_weights,
    );

    let export_doc = ExportDoc {
        body: "triad".into(),
        encoding: "PersistentTriModalFusionV3".into(),
        report: outcome.report.clone(),
    };

    if let Some(ref out_dir) = manifest_out {
        let mut manifest = build_prebake_manifest(
            export_doc.clone(),
            &base_id,
            &req.bake_mode,
            req.strength,
            IncubationIngress::ModelDecompose,
            req.samples.len(),
        );
        manifest.product = "valhalla-trimodal-fusion-v3".into();
        manifest.free_multimodal = Some(serde_json::json!({
            "protocol": FREE_MULTIMODAL_FUSION_V3,
            "tri_modal": true,
            "samples_fed": outcome.samples_fed,
            "mix_weights": outcome.mix_weights,
            "aggregate_score": outcome.aggregate_score,
            "tile_count": mp.tile_count,
            "stem_cell_count": mp.stem_cell_count,
        }));
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
        eprintln!(
            "manifest {MANIFEST_VERSION} → {}/valhalla_manifest.json",
            out_dir.display()
        );
    }

    let payload = serde_json::json!({
        "export_doc": export_doc,
        "persistent_fusion": outcome,
    });
    println!("{}", serde_json::to_string_pretty(&payload).unwrap());
}
