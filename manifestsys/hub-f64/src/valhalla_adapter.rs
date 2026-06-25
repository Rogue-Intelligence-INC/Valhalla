//! Valhalla compose adapter — bake Stem (text) + Tile (audio) + Hub (Fate) into LM weights.
//!
//! Built once at export/bake; inference reads `compose_adapter` from manifest (no per-prompt surgery).

use crate::multimodal_compose::{COMPOSE_MCQ_PROTOCOL, COMPOSE_PROTOCOL};
use crate::signal_ingress::{ValhallaModelPatch, PATCH_VECTOR_DIM};

pub const ADAPTER_VERSION: &str = "valhalla-adapter-compose-v1";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComposeAdapterSpec {
    pub adapter_version: String,
    pub open_protocol: String,
    pub mcq_protocol: String,
    /// Text / MCQ stem path (Stem body).
    pub stem_direction: Vec<f64>,
    /// TTS mel band path (Tile body).
    pub tile_direction: Vec<f64>,
    /// Hub Fate gate (4 quad affinities).
    pub hub_gate: Vec<f64>,
    /// Fused direction for lm_head + late-layer LN (hidden-sized after projection).
    pub lm_head_compose: Vec<f64>,
    pub cross_modal_prior: f64,
}

fn mean_sig(sigs: &[Vec<f64>]) -> Vec<f64> {
    if sigs.is_empty() {
        return Vec::new();
    }
    let dim = sigs.iter().map(|s| s.len()).max().unwrap_or(0);
    if dim == 0 {
        return Vec::new();
    }
    let mut acc = vec![0.0; dim];
    let mut n = 0usize;
    for s in sigs {
        if s.is_empty() {
            continue;
        }
        for (i, &v) in s.iter().enumerate().take(dim) {
            acc[i] += v;
        }
        n += 1;
    }
    if n == 0 {
        return acc;
    }
    for a in &mut acc {
        *a /= n as f64;
    }
    normalize_vec(&mut acc);
    acc
}

fn normalize_vec(v: &mut [f64]) {
    let n: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt();
    if n > 1e-12 {
        for x in v.iter_mut() {
            *x /= n;
        }
    }
}

fn project_patch(vec: &[f64], hidden: usize) -> Vec<f64> {
    if vec.is_empty() {
        return vec![0.0; hidden.min(PATCH_VECTOR_DIM).max(1)];
    }
    let mut out = vec![0.0; hidden];
    for (i, slot) in out.iter_mut().enumerate() {
        *slot = vec[i % vec.len()];
    }
    normalize_vec(&mut out);
    out
}

fn fuse_directions(stem: &[f64], tile: &[f64], patch: &[f64], hidden: usize) -> Vec<f64> {
    let mut out = vec![0.0; hidden];
    for i in 0..hidden {
        let s = stem.get(i % stem.len()).copied().unwrap_or(0.0);
        let t = tile.get(i % tile.len()).copied().unwrap_or(0.0);
        let p = patch.get(i % patch.len()).copied().unwrap_or(0.0);
        out[i] = s * 0.38 + t * 0.38 + p * 0.24;
    }
    normalize_vec(&mut out);
    out
}

/// Build compose adapter spec from incubated structure (pre-bake / manifest).
pub fn build_compose_adapter(mp: &ValhallaModelPatch, hidden: usize) -> ComposeAdapterSpec {
    let stem_direction = if mp.stem_signatures.is_empty() {
        mean_sig(&mp.stem_cluster_signatures)
    } else {
        mean_sig(&mp.stem_signatures)
    };
    let tile_direction = mean_sig(&mp.tile_signatures);
    let hub_gate = if mp.fate_affinities.len() >= 4 {
        mp.fate_affinities.clone()
    } else {
        vec![0.25, 0.25, 0.25, 0.25]
    };
    let patch_proj = project_patch(&mp.patch_vector, hidden);
    let stem_h = if stem_direction.is_empty() {
        patch_proj.clone()
    } else {
        project_patch(&stem_direction, hidden)
    };
    let tile_h = if tile_direction.is_empty() {
        patch_proj.clone()
    } else {
        project_patch(&tile_direction, hidden)
    };
    let lm_head_compose = fuse_directions(&stem_h, &tile_h, &patch_proj, hidden);
    let hub_mean = hub_gate.iter().sum::<f64>() / hub_gate.len().max(1) as f64;
    let cross_modal_prior = (0.5 + (hub_mean - 0.25) * 0.6).clamp(0.05, 0.95);

    ComposeAdapterSpec {
        adapter_version: ADAPTER_VERSION.to_string(),
        open_protocol: COMPOSE_PROTOCOL.to_string(),
        mcq_protocol: COMPOSE_MCQ_PROTOCOL.to_string(),
        stem_direction: stem_h,
        tile_direction: tile_h,
        hub_gate,
        lm_head_compose,
        cross_modal_prior,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compose_adapter_has_hidden_direction() {
        let mp = ValhallaModelPatch {
            body_source: "triad".into(),
            fate_affinities: vec![0.3, 0.2, 0.35, 0.15],
            hub_routed: [1, 2, 3, 4],
            quad_layer_scales: [1.0, 1.02, 0.98, 1.01],
            tile_signatures: vec![vec![1.0, 0.0, 0.5]],
            stem_signatures: vec![vec![0.0, 1.0, 0.5]],
            patch_vector: (0..PATCH_VECTOR_DIM).map(|i| i as f64 * 0.01).collect(),
            signals_emitted: 100,
            tile_count: 5,
            stem_cell_count: 8,
            tile_activations: vec![],
            tile_connection_count: 0,
            tile_avg_activation: 0.0,
            stem_organ_clusters: vec![],
            stem_cluster_signatures: vec![],
            stem_cluster_energies: vec![],
            ingress_encoder: String::new(),
        };
        let spec = build_compose_adapter(&mp, 896);
        assert_eq!(spec.lm_head_compose.len(), 896);
        assert_eq!(spec.hub_gate.len(), 4);
        assert!(spec.cross_modal_prior > 0.05);
    }
}
