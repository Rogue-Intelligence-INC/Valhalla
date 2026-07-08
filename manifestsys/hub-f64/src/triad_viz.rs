//! Tile / StemCell visualization builders for dashboard and third-party tools.

use std::collections::HashMap;

use valhalla_tile_quattro::TileQuattroComplete;

use crate::signal_ingress::{CycleSnapshot, StemOrganCluster, TriadFeedReport, ValhallaModelPatch};
use crate::triad_viz_types::signature_hue;

pub use crate::triad_viz_types::{
    CycleVizPoint, StemOrganViz, TileNeuronViz, TileSynapseViz, TriadVizSnapshot,
};
const PATCH_PREVIEW: usize = 32;
const MAX_TILES: usize = 64;
const WEIGHT_PREVIEW: usize = 8;
const SIG_PREVIEW: usize = 8;

pub fn build_tile_neurons(tile: &TileQuattroComplete) -> Vec<TileNeuronViz> {
    tile.completed_tiles
        .iter()
        .take(MAX_TILES)
        .enumerate()
        .map(|(i, t)| TileNeuronViz {
            index: i,
            id: t.id.clone(),
            activation: t.activation,
            bias: t.bias,
            weights: t.weights.iter().take(WEIGHT_PREVIEW).copied().collect(),
            hue: signature_hue(&t.data_signature),
            connection_count: t.connections.len(),
        })
        .collect()
}

pub fn build_tile_synapses(tile: &TileQuattroComplete) -> Vec<TileSynapseViz> {
    let id_to_idx: HashMap<String, usize> = tile
        .completed_tiles
        .iter()
        .enumerate()
        .map(|(i, t)| (t.id.clone(), i))
        .collect();
    let mut edges = Vec::new();
    for (i, t) in tile.completed_tiles.iter().enumerate().take(MAX_TILES) {
        for (wi, conn_id) in t.connections.iter().enumerate() {
            if let Some(&j) = id_to_idx.get(conn_id) {
                if i != j {
                    edges.push(TileSynapseViz {
                        from: i,
                        to: j,
                        weight: t.weights.get(wi).copied().unwrap_or(0.5),
                    });
                }
            }
        }
    }
    edges
}

pub fn build_stem_organs(
    clusters: &[StemOrganCluster],
    cluster_sigs: &[Vec<f64>],
    _cluster_energies: &[f64],
) -> Vec<StemOrganViz> {
    clusters
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let sig = cluster_sigs.get(i).cloned().unwrap_or_default();
            StemOrganViz {
                cluster_id: c.cluster_id,
                cell_count: c.cell_count,
                avg_energy: c.avg_energy,
                internal_edges: c.internal_edges,
                hue: signature_hue(&sig),
                signature_preview: sig.iter().take(SIG_PREVIEW).copied().collect(),
                member_preview: c.cell_ids.iter().take(6).cloned().collect(),
            }
        })
        .collect()
}

pub fn build_viz_from_patch(patch: &ValhallaModelPatch) -> TriadVizSnapshot {
    let tile_neurons = if !patch.tile_neurons.is_empty() {
        patch.tile_neurons.clone()
    } else {
        patch
            .tile_signatures
            .iter()
            .take(MAX_TILES)
            .enumerate()
            .map(|(i, sig)| TileNeuronViz {
                index: i,
                id: format!("tile_{i}"),
                activation: patch.tile_activations.get(i).copied().unwrap_or(0.0),
                bias: 0.0,
                weights: sig.iter().take(WEIGHT_PREVIEW).copied().collect(),
                hue: signature_hue(sig),
                connection_count: sig.get(3).copied().unwrap_or(0.0) as usize,
            })
            .collect()
    };

    TriadVizSnapshot {
        tile_neurons,
        tile_synapses: patch.tile_synapses.clone(),
        tile_connection_count: patch.tile_connection_count,
        tile_avg_activation: patch.tile_avg_activation,
        stem_organs: build_stem_organs(
            &patch.stem_organ_clusters,
            &patch.stem_cluster_signatures,
            &patch.stem_cluster_energies,
        ),
        cycle_trace: Vec::new(),
        patch_vector_preview: patch.patch_vector.iter().take(PATCH_PREVIEW).copied().collect(),
        quad_layer_scales: patch.quad_layer_scales,
    }
}

pub fn build_viz_from_feed(feed: &TriadFeedReport) -> TriadVizSnapshot {
    let mut viz = build_viz_from_patch(&feed.model_patch);
    viz.cycle_trace = feed.cycle_trace.iter().map(cycle_viz).collect();
    viz
}

fn cycle_viz(c: &CycleSnapshot) -> CycleVizPoint {
    CycleVizPoint {
        cycle: c.cycle,
        tile_completed: c.tile_completed,
        tile_merges: c.tile_merges,
        tile_splits: c.tile_splits,
        stem_cells: c.stem_cells,
        stem_connections: c.stem_connections,
    }
}
