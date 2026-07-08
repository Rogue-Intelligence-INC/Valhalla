//! Serializable viz types for Tile neural nets and StemCell organs (no ingress deps).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TileNeuronViz {
    pub index: usize,
    pub id: String,
    pub activation: f64,
    pub bias: f64,
    /// Internal synaptic weights (spontaneous formation).
    pub weights: Vec<f64>,
    /// 0..1 identity hue from data_signature.
    pub hue: f64,
    pub connection_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TileSynapseViz {
    pub from: usize,
    pub to: usize,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StemOrganViz {
    pub cluster_id: usize,
    pub cell_count: usize,
    pub avg_energy: f64,
    pub internal_edges: usize,
    pub hue: f64,
    pub signature_preview: Vec<f64>,
    pub member_preview: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CycleVizPoint {
    pub cycle: usize,
    pub tile_completed: usize,
    pub tile_merges: u64,
    pub tile_splits: u64,
    pub stem_cells: usize,
    pub stem_connections: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TriadVizSnapshot {
    pub tile_neurons: Vec<TileNeuronViz>,
    pub tile_synapses: Vec<TileSynapseViz>,
    pub tile_connection_count: usize,
    pub tile_avg_activation: f64,
    pub stem_organs: Vec<StemOrganViz>,
    pub cycle_trace: Vec<CycleVizPoint>,
    pub patch_vector_preview: Vec<f64>,
    pub quad_layer_scales: [f64; 4],
}

pub fn signature_hue(sig: &[f64]) -> f64 {
    if sig.is_empty() {
        return 0.55;
    }
    let a = sig.first().copied().unwrap_or(0.0).abs();
    let b = sig.get(1).copied().unwrap_or(0.0).abs();
    let t = (a * 0.6 + b * 0.4).fract();
    t.clamp(0.0, 1.0)
}
