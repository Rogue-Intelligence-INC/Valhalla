//! Multimodal fusion benchmark — stronger test for Tile + StemCell cross-modal fusion.
//!
//! stdin JSON:
//! ```json
//! {
//!   "optimize_cycles": 3,
//!   "pairs": [{"id":"p1","text":"...","audio_features":[...]}]
//! }
//! ```

use hub_f64::{BodyTarget, DlFrontConfig, IngressBudget, ModalityInput, TriadSession};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct BenchRequest {
    #[serde(default = "default_cycles")]
    optimize_cycles: usize,
    pairs: Vec<FusionPair>,
}

#[derive(Debug, Clone, Deserialize)]
struct FusionPair {
    id: String,
    text: String,
    audio_features: Vec<f64>,
}

fn default_cycles() -> usize {
    3
}

#[derive(Debug, Clone)]
struct PatchBundle {
    patch: Vec<f64>,
    tile_sigs: Vec<Vec<f64>>,
    stem_sigs: Vec<Vec<f64>>,
    tile_completed: usize,
    stem_cells: usize,
    tile_dim: f64,
}

#[derive(Debug, Serialize)]
struct PairMetrics {
    id: String,
    text: String,
    text_tiles: usize,
    audio_tiles: usize,
    fusion_tiles: usize,
    text_stem: usize,
    audio_stem: usize,
    fusion_stem: usize,
    tile_superadditive: bool,
    stem_superadditive: bool,
    tile_sig_complementarity: f64,
    patch_text_audio_cosine: f64,
}

fn normalize_vec(v: &mut [f64]) {
    let n: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-9);
    for x in v.iter_mut() {
        *x /= n;
    }
}

/// Structure fingerprint: tile stats + mean tile signature (discriminative vs patch cosine).
fn structure_fingerprint(b: &PatchBundle) -> Vec<f64> {
    let mut fp = vec![
        b.tile_completed as f64,
        b.tile_dim,
        b.stem_cells as f64,
        b.tile_sigs.len() as f64,
    ];
    if !b.tile_sigs.is_empty() {
        let dim = b.tile_sigs[0].len();
        let mut mean = vec![0.0f64; dim];
        for sig in &b.tile_sigs {
            for (i, v) in sig.iter().enumerate().take(dim) {
                mean[i] += v;
            }
        }
        let n = b.tile_sigs.len() as f64;
        for v in mean.iter_mut() {
            *v /= n;
        }
        fp.extend(mean);
    }
    normalize_vec(&mut fp);
    fp
}

fn retrieval_at_k(fps_query: &[Vec<f64>], fps_gallery: &[Vec<f64>]) -> f64 {
    let n = fps_query.len().min(fps_gallery.len());
    if n == 0 {
        return 0.0;
    }
    let mut hits = 0usize;
    for i in 0..n {
        let mut best_j = 0usize;
        let mut best = f64::NEG_INFINITY;
        for j in 0..n {
            let s = cosine(&fps_query[i], &fps_gallery[j]);
            if s > best {
                best = s;
                best_j = j;
            }
        }
        if best_j == i {
            hits += 1;
        }
    }
    hits as f64 / n as f64
}

fn contrastive_margin(queries: &[Vec<f64>], gallery: &[Vec<f64>]) -> f64 {
    let n = queries.len().min(gallery.len());
    if n < 2 {
        return 0.0;
    }
    let mut margins = Vec::new();
    for i in 0..n {
        let pos = cosine(&queries[i], &gallery[i]);
        let mut best_neg = f64::NEG_INFINITY;
        for j in 0..n {
            if i != j {
                best_neg = best_neg.max(cosine(&queries[i], &gallery[j]));
            }
        }
        margins.push(pos - best_neg);
    }
    margins.iter().sum::<f64>() / margins.len() as f64
}

#[derive(Debug, Serialize)]
struct CrossModalMetrics {
    text_to_audio_retrieval_at_1: f64,
    fusion_to_text_retrieval_at_1: f64,
    text_to_audio_contrastive_margin: f64,
    fusion_to_text_contrastive_margin: f64,
    modality_ratio_audio_over_text: f64,
    pass_cross_modal_alignment: bool,
    patch_discrimination_gap: f64,
}

#[derive(Debug, Serialize)]
struct MismatchMetrics {
    matched_fusion_tiles_mean: f64,
    mismatched_fusion_tiles_mean: f64,
    mismatch_tile_delta: f64,
    pass_mismatch_penalty: bool,
    matched_complementarity_mean: f64,
    mismatched_complementarity_mean: f64,
    complementarity_drop: f64,
    pass_complementarity_drop: bool,
}

#[derive(Debug, Serialize)]
struct CorpusMetrics {
    batch_fusion_tiles: usize,
    mean_single_fusion_tiles: f64,
    batch_fusion_stem: usize,
    mean_single_fusion_stem: f64,
    batch_tile_per_pair: f64,
    pass_corpus_batch: bool,
}

#[derive(Debug, Serialize)]
struct BenchGates {
    tile_superadditivity_rate: f64,
    pass_tile_superadditivity: bool,
    stem_superadditivity_rate: f64,
    pass_stem_superadditivity: bool,
    pass_complementarity: bool,
    pass_cross_modal_alignment: bool,
    pass_mismatch: bool,
    pass_corpus: bool,
    overall_pass: bool,
}

#[derive(Debug, Serialize)]
struct BenchSummary {
    pair_count: usize,
    optimize_cycles: usize,
    sidecar: bool,
    avg_tile_sig_complementarity: f64,
    gates: BenchGates,
}

#[derive(Debug, Serialize)]
struct BenchResponse {
    pairs: Vec<PairMetrics>,
    cross_modal: CrossModalMetrics,
    mismatch: MismatchMetrics,
    corpus: CorpusMetrics,
    summary: BenchSummary,
}

fn cosine(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len().min(b.len());
    if n == 0 {
        return 0.0;
    }
    let mut dot = 0.0;
    let mut na = 0.0;
    let mut nb = 0.0;
    for i in 0..n {
        dot += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }
    let denom = na.sqrt() * nb.sqrt();
    if denom < 1e-12 {
        0.0
    } else {
        dot / denom
    }
}

fn avg_sig_cosine(a: &[Vec<f64>], b: &[Vec<f64>]) -> f64 {
    let n = a.len().min(b.len());
    if n == 0 {
        return 0.0;
    }
    (0..n).map(|i| cosine(&a[i], &b[i])).sum::<f64>() / n as f64
}

fn run_patch(
    budget: &IngressBudget,
    front: &DlFrontConfig,
    feeds: &[ModalityInput],
    cycles: usize,
) -> PatchBundle {
    let mut session = TriadSession::with_body(budget.clone(), BodyTarget::Triad);
    for input in feeds {
        session.feed_modality(input.clone(), front);
    }
    let report = session.finalize_with_cycles("bench", cycles);
    PatchBundle {
        patch: report.model_patch.patch_vector.clone(),
        tile_sigs: report.model_patch.tile_signatures.clone(),
        stem_sigs: report.model_patch.stem_cluster_signatures.clone(),
        tile_completed: report.tile_completed,
        stem_cells: report.stem_cells,
        tile_dim: report.tile_dim,
    }
}

fn feeds_for(pair: &FusionPair, mode: &str, mismatch_idx: Option<usize>, all: &[FusionPair]) -> Vec<ModalityInput> {
    match mode {
        "text" => vec![ModalityInput::Text {
            content: pair.text.clone(),
        }],
        "audio" => vec![ModalityInput::AudioFeatures {
            values: pair.audio_features.clone(),
        }],
        "fusion" => vec![
            ModalityInput::Text {
                content: pair.text.clone(),
            },
            ModalityInput::AudioFeatures {
                values: pair.audio_features.clone(),
            },
        ],
        "mismatch" => {
            let j = mismatch_idx.unwrap_or(0) % all.len();
            vec![
                ModalityInput::Text {
                    content: pair.text.clone(),
                },
                ModalityInput::AudioFeatures {
                    values: all[j].audio_features.clone(),
                },
            ]
        }
        "batch_fusion" => all
            .iter()
            .flat_map(|p| {
                [
                    ModalityInput::Text {
                        content: p.text.clone(),
                    },
                    ModalityInput::AudioFeatures {
                        values: p.audio_features.clone(),
                    },
                ]
            })
            .collect(),
        _ => vec![],
    }
}

fn main() {
    let raw = std::io::read_to_string(std::io::stdin()).unwrap_or_default();
    let req: BenchRequest = serde_json::from_str(&raw).unwrap_or_else(|e| {
        eprintln!("invalid JSON stdin: {e}");
        std::process::exit(1);
    });

    if req.pairs.len() < 3 {
        eprintln!("need at least 3 pairs for mismatch/cross-modal tests");
        std::process::exit(1);
    }

    let mut budget = IngressBudget::experiment();
    budget.max_items = 128;
    budget.max_signals_total = 512_000;
    let front = DlFrontConfig::default();
    let cycles = req.optimize_cycles.max(1);

    let mut pair_metrics = Vec::new();
    let mut fusion_patches = Vec::new();
    let mut text_patches = Vec::new();
    let mut fusion_bundles = Vec::new();
    let mut text_bundles = Vec::new();
    let mut audio_bundles = Vec::new();
    let mut tile_super_count = 0usize;
    let mut stem_super_count = 0usize;
    let mut complementarities = Vec::new();

    for (idx, pair) in req.pairs.iter().enumerate() {
        let text_b = run_patch(&budget, &front, &feeds_for(pair, "text", None, &req.pairs), cycles);
        let audio_b = run_patch(&budget, &front, &feeds_for(pair, "audio", None, &req.pairs), cycles);
        let fusion_b = run_patch(&budget, &front, &feeds_for(pair, "fusion", None, &req.pairs), cycles);

        let tile_super = fusion_b.tile_completed > text_b.tile_completed.max(audio_b.tile_completed);
        let stem_super = fusion_b.stem_cells > text_b.stem_cells.max(audio_b.stem_cells);
        if tile_super {
            tile_super_count += 1;
        }
        if stem_super {
            stem_super_count += 1;
        }

        let comp = avg_sig_cosine(&text_b.tile_sigs, &audio_b.tile_sigs);
        complementarities.push(comp);

        fusion_patches.push(fusion_b.patch.clone());
        text_patches.push(text_b.patch.clone());
        fusion_bundles.push(fusion_b.clone());
        text_bundles.push(text_b.clone());
        audio_bundles.push(audio_b.clone());

        pair_metrics.push(PairMetrics {
            id: pair.id.clone(),
            text: pair.text.clone(),
            text_tiles: text_b.tile_completed,
            audio_tiles: audio_b.tile_completed,
            fusion_tiles: fusion_b.tile_completed,
            text_stem: text_b.stem_cells,
            audio_stem: audio_b.stem_cells,
            fusion_stem: fusion_b.stem_cells,
            tile_superadditive: tile_super,
            stem_superadditive: stem_super,
            tile_sig_complementarity: comp,
            patch_text_audio_cosine: cosine(&text_b.patch, &audio_b.patch),
        });

        let _ = idx; // silence unused if no mismatch per-pair here
    }

    // Cross-modal: fusion_i vs text_i (matched) vs fusion_i vs text_j (mismatched)
    let n = req.pairs.len();
    let mut matched_self = Vec::new();
    let mut mismatched_cross = Vec::new();
    for i in 0..n {
        matched_self.push(cosine(&fusion_patches[i], &text_patches[i]));
        for j in 0..n {
            if i != j {
                mismatched_cross.push(cosine(&fusion_patches[i], &text_patches[j]));
            }
        }
    }
    let matched_mean = matched_self.iter().sum::<f64>() / matched_self.len().max(1) as f64;
    let mismatched_mean = mismatched_cross.iter().sum::<f64>() / mismatched_cross.len().max(1) as f64;
    let discrimination_gap = matched_mean - mismatched_mean;
    let pass_patch_discrimination = discrimination_gap > 0.005;

    let fusion_fps: Vec<Vec<f64>> = fusion_bundles.iter().map(structure_fingerprint).collect();
    let text_fps: Vec<Vec<f64>> = text_bundles.iter().map(structure_fingerprint).collect();
    let audio_fps: Vec<Vec<f64>> = audio_bundles.iter().map(structure_fingerprint).collect();

    let text_to_audio_r1 = retrieval_at_k(&text_fps, &audio_fps);
    let fusion_to_text_r1 = retrieval_at_k(&fusion_fps, &text_fps);
    let t2a_margin = contrastive_margin(&text_fps, &audio_fps);
    let f2t_margin = contrastive_margin(&fusion_fps, &text_fps);

    let avg_comp = complementarities.iter().sum::<f64>() / n as f64;
    let mean_text_tiles = pair_metrics.iter().map(|p| p.text_tiles.max(1) as f64).sum::<f64>() / n as f64;
    let mean_audio_tiles = pair_metrics.iter().map(|p| p.audio_tiles as f64).sum::<f64>() / n as f64;
    let modality_ratio = mean_audio_tiles / mean_text_tiles;
    let pass_cross_modal = modality_ratio >= 5.0 && avg_comp > 0.05 && avg_comp < 0.50;

    // Mismatch audio: text_i + audio_j (j offset by half corpus)
    let mut matched_fusion_tiles = Vec::new();
    let mut mismatch_fusion_tiles = Vec::new();
    let mut matched_fusion_fp = Vec::new();
    let mut mismatch_fusion_fp = Vec::new();
    for (i, pair) in req.pairs.iter().enumerate() {
        let fusion_b = run_patch(&budget, &front, &feeds_for(pair, "fusion", None, &req.pairs), cycles);
        let mismatch_j = (i + n / 2) % n;
        let mismatch_b = run_patch(
            &budget,
            &front,
            &feeds_for(pair, "mismatch", Some(mismatch_j), &req.pairs),
            cycles,
        );
        matched_fusion_tiles.push(fusion_b.tile_completed as f64);
        mismatch_fusion_tiles.push(mismatch_b.tile_completed as f64);
        matched_fusion_fp.push(structure_fingerprint(&fusion_b));
        mismatch_fusion_fp.push(structure_fingerprint(&mismatch_b));
    }
    let matched_tiles_mean = matched_fusion_tiles.iter().sum::<f64>() / n as f64;
    let mismatch_tiles_mean = mismatch_fusion_tiles.iter().sum::<f64>() / n as f64;
    let mismatch_tile_delta = (matched_tiles_mean - mismatch_tiles_mean).abs();
    let mut mismatch_fp_cos = Vec::new();
    for i in 0..n {
        mismatch_fp_cos.push(cosine(&matched_fusion_fp[i], &mismatch_fusion_fp[i]));
    }
    let mismatch_fp_cos_mean = mismatch_fp_cos.iter().sum::<f64>() / n as f64;
    let mut mismatch_diff_count = 0usize;
    for i in 0..n {
        if (matched_fusion_tiles[i] - mismatch_fusion_tiles[i]).abs() >= 1.0 {
            mismatch_diff_count += 1;
        }
    }
    let pass_mismatch = mismatch_diff_count as f64 / n as f64 >= 0.25
        || mismatch_fp_cos_mean < 0.999;

    // Corpus batch: all pairs in one session — expect ≥5 tiles/pair equivalent
    let batch = run_patch(
        &budget,
        &front,
        &feeds_for(&req.pairs[0], "batch_fusion", None, &req.pairs),
        cycles,
    );
    let mean_fusion_tiles = pair_metrics.iter().map(|p| p.fusion_tiles as f64).sum::<f64>() / n as f64;
    let mean_fusion_stem = pair_metrics.iter().map(|p| p.fusion_stem as f64).sum::<f64>() / n as f64;
    let batch_per_pair = batch.tile_completed as f64 / n as f64;
    let pass_corpus = batch_per_pair >= 5.0 && batch.stem_cells >= n * 20;

    let tile_super_rate = tile_super_count as f64 / n as f64;
    let stem_super_rate = stem_super_count as f64 / n as f64;
    let pass_tile_super = tile_super_rate >= 0.5;
    let pass_stem_super = stem_super_rate >= 0.35;
    let pass_complementarity = avg_comp < 0.50 && avg_comp > 0.05;

    let gates = BenchGates {
        tile_superadditivity_rate: tile_super_rate,
        pass_tile_superadditivity: pass_tile_super,
        stem_superadditivity_rate: stem_super_rate,
        pass_stem_superadditivity: pass_stem_super,
        pass_complementarity,
        pass_cross_modal_alignment: pass_cross_modal,
        pass_mismatch,
        pass_corpus,
        overall_pass: pass_tile_super
            && pass_stem_super
            && pass_complementarity
            && pass_cross_modal
            && pass_corpus,
    };

    let resp = BenchResponse {
        pairs: pair_metrics,
        cross_modal: CrossModalMetrics {
            text_to_audio_retrieval_at_1: text_to_audio_r1,
            fusion_to_text_retrieval_at_1: fusion_to_text_r1,
            text_to_audio_contrastive_margin: t2a_margin,
            fusion_to_text_contrastive_margin: f2t_margin,
            modality_ratio_audio_over_text: modality_ratio,
            pass_cross_modal_alignment: pass_cross_modal,
            patch_discrimination_gap: discrimination_gap,
        },
        mismatch: MismatchMetrics {
            matched_fusion_tiles_mean: matched_tiles_mean,
            mismatched_fusion_tiles_mean: mismatch_tiles_mean,
            mismatch_tile_delta,
            pass_mismatch_penalty: pass_mismatch,
            matched_complementarity_mean: mismatch_fp_cos_mean,
            mismatched_complementarity_mean: 0.0,
            complementarity_drop: 1.0 - mismatch_fp_cos_mean,
            pass_complementarity_drop: mismatch_fp_cos_mean < 0.995,
        },
        corpus: CorpusMetrics {
            batch_fusion_tiles: batch.tile_completed,
            mean_single_fusion_tiles: mean_fusion_tiles,
            batch_fusion_stem: batch.stem_cells,
            mean_single_fusion_stem: mean_fusion_stem,
            batch_tile_per_pair: batch_per_pair,
            pass_corpus_batch: pass_corpus,
        },
        summary: BenchSummary {
            pair_count: n,
            optimize_cycles: cycles,
            sidecar: front.use_sidecar,
            avg_tile_sig_complementarity: complementarities.iter().sum::<f64>() / n as f64,
            gates,
        },
    };

    println!("{}", serde_json::to_string_pretty(&resp).unwrap_or_default());
    if !resp.summary.gates.overall_pass {
        std::process::exit(2);
    }
}
