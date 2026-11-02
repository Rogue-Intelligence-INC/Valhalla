//! Whole-body monitor: sits outside the body, read-only observation
//! of Tile/Stem structure, the eight quad stores, and global Fate.
//!
//! Reads the body's current state only; names no blocks, reads no
//! preferences, changes nothing in the body, takes part in no
//! judgement (Fate and the monitor are both observation layers).
//! - Merge counts come from cycle results (Tile has no cumulative field, the monitor adds per cycle; Stem keeps its own total_merges)
//! - Spontaneous edges: new connections reported by propagate itself, accumulated; the monitor sets no threshold to guess
//! - Stem links: connections as written by the field, counted as-is (strength is the field's own value by density/distance/field strength)
//! - Connected blocks: undirected connected components, sizes only, unnamed
//! - Quads: the eight stores read per store (entries/bytes/concatenated
//!   fingerprint); quad movement is seen externally from the diff of
//!   adjacent snapshots (the monitor reads no internal body counters)

use std::collections::{HashMap, HashSet};

use fate_f64::TripleFateCombined;
use stem_cell::stemcell_complete::CellCycleResult;
use stem_cell::StemCellComplete;
use valhalla_tile_quattro::tile_complete::TileCycleResult;
use valhalla_tile_quattro::TileQuattroComplete;

/// Fixed-order names of the eight quad stores (Tile quads, Stem quads).
pub const QUAD_NAMES: [&str; 8] = [
    "TileBlackHole", "TileMeshBrain", "TileMultiNova", "TileBaseForce",
    "StemBlackHole", "StemMeshBrain", "StemMultiNova", "StemBaseForce",
];

/// One quad store as seen externally by the monitor: entries, total bytes, concatenated fingerprint.
#[derive(Debug, Clone, Copy, Default)]
pub struct QuadView {
    pub entries: usize,
    pub bytes: usize,
    pub hash: u64,
}

fn quad_view(stored: &[Vec<u8>]) -> QuadView {
    let mut hash: u64 = 0xcbf29ce484222325;
    let mut bytes = 0usize;
    for e in stored {
        bytes += e.len();
        for b in e {
            hash ^= *b as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
    }
    QuadView { entries: stored.len(), bytes, hash }
}

/// One body snapshot. Taken once per cycle during training.
#[derive(Debug, Clone, Default)]
pub struct BodySnapshot {
    pub tiles: usize,
    pub cells: usize,
    /// Tile merges accumulated by the monitor
    pub tile_merges: usize,
    /// Merges accumulated inside Stem
    pub stem_merges: u64,
    /// Tile undirected edge count
    pub tile_edges: usize,
    /// Spontaneous new connections reported by propagate, accumulated
    pub tile_synapses_born: usize,
    /// Connection count written by the Stem field (strength as-is)
    pub stem_links: usize,
    /// Tile block sizes, descending, only >= 2 kept
    pub tile_blocks: Vec<usize>,
    /// Stem block sizes, descending, only >= 2 kept
    pub stem_blocks: Vec<usize>,
    /// External view of the eight quad stores (order: QUAD_NAMES)
    pub quad: [QuadView; 8],
    /// Total entries across the eight stores
    pub stored_entries: usize,
    /// Total bytes across the eight stores
    pub stored_bytes: usize,
    /// Whether the four same-layer stores have diverged (true once the quads maintain by their own rules)
    pub quad_diverged: bool,
    /// Per-store entry change since the previous watch (external diff; 0 on first)
    pub quad_delta: [i64; 8],
    /// Global Fate tick (filled only by watch_with_fate)
    pub fate_tick: u64,
    /// Global Fate stored count (filled only by watch_with_fate)
    pub fate_stored: usize,
    /// Global Fate cluster equation (filled only by watch_with_fate)
    pub fate_equation: String,
}

impl BodySnapshot {
    pub fn largest_tile_block(&self) -> usize {
        self.tile_blocks.first().copied().unwrap_or(0)
    }

    pub fn largest_stem_block(&self) -> usize {
        self.stem_blocks.first().copied().unwrap_or(0)
    }

    /// One-line form for training logs
    pub fn line(&self) -> String {
        format!(
            "monitor tiles {} cells {} tile_merges {} stem_merges {} tile_edges {} synapses_born {} stem_links {} tile_block {} stem_block {} stored {}/{}B quad_diverged {}",
            self.tiles,
            self.cells,
            self.tile_merges,
            self.stem_merges,
            self.tile_edges,
            self.tile_synapses_born,
            self.stem_links,
            self.largest_tile_block(),
            self.largest_stem_block(),
            self.stored_entries,
            self.stored_bytes,
            self.quad_diverged,
        )
    }

    /// One-line form of the eight quad stores: per store entries/bytes/fingerprint + this diff
    pub fn quad_line(&self) -> String {
        let mut out = String::from("quad");
        for (i, q) in self.quad.iter().enumerate() {
            out.push_str(&format!(
                " {} {}/{}B#{:08x}({:+})",
                QUAD_NAMES[i], q.entries, q.bytes, (q.hash & 0xffff_ffff) as u32, self.quad_delta[i]
            ));
        }
        out
    }

    /// One-line form of global Fate (has content only after watch_with_fate)
    pub fn fate_line(&self) -> String {
        format!("fate tick {} stored {} eq {}", self.fate_tick, self.fate_stored, self.fate_equation)
    }
}

/// Whole-body monitor hung beside the cycles during training. Sits outside; never changes the body.
#[derive(Debug, Default)]
pub struct BodyMonitor {
    tile_merges: usize,
    synapses_born: usize,
    /// Entry counts of the eight stores at the previous watch, for external diffing
    prev_quad_entries: Option<[usize; 8]>,
}

impl BodyMonitor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Called once per cycle; records this cycle's merge count
    pub fn observe_cycle(&mut self, tile: &TileCycleResult, _stem: &CellCycleResult) {
        self.tile_merges += tile.merges;
    }

    /// Accumulates each new connection propagate reports (the monitor sets no threshold to guess which are newborn)
    pub fn observe_propagate(&mut self, new_connections: usize) {
        self.synapses_born += new_connections;
    }

    /// Observe the whole once: snapshot and record the eight stores'
    /// entry counts; at the next watch, the diff is the quads'
    /// externally visible movement. Read-only; never enters the body.
    pub fn watch(&mut self, tile: &TileQuattroComplete, stem: &StemCellComplete) -> BodySnapshot {
        let mut snap = self.snapshot(tile, stem);
        if let Some(prev) = self.prev_quad_entries {
            for i in 0..8 {
                snap.quad_delta[i] = snap.quad[i].entries as i64 - prev[i] as i64;
            }
        }
        self.prev_quad_entries = Some(snap.quad.map(|q| q.entries));
        snap
    }

    /// Like watch, and also reads in global Fate's tick/storage/equation
    pub fn watch_with_fate(
        &mut self,
        tile: &TileQuattroComplete,
        stem: &StemCellComplete,
        global: &TripleFateCombined,
    ) -> BodySnapshot {
        let mut snap = self.watch(tile, stem);
        snap.fate_tick = global.global_tick;
        snap.fate_stored = global.stored_count();
        snap.fate_equation = global.cluster_equation.equation_str();
        snap
    }

    pub fn snapshot(&self, tile: &TileQuattroComplete, stem: &StemCellComplete) -> BodySnapshot {
        let mut snap = BodySnapshot {
            tiles: tile.completed_tiles.len(),
            cells: stem.cells.len(),
            tile_merges: self.tile_merges,
            stem_merges: stem.total_merges,
            tile_synapses_born: self.synapses_born,
            ..Default::default()
        };

        // Tile edges: connections are peer ids. Undirected dedup
        let tile_index: HashMap<&str, usize> = tile
            .completed_tiles
            .iter()
            .enumerate()
            .map(|(i, t)| (t.id.as_str(), i))
            .collect();
        let mut tile_pairs: HashSet<(usize, usize)> = HashSet::new();
        let mut tile_adj: Vec<Vec<usize>> = vec![Vec::new(); tile.completed_tiles.len()];
        for (i, t) in tile.completed_tiles.iter().enumerate() {
            for peer in t.connections.iter() {
                let Some(&j) = tile_index.get(peer.as_str()) else {
                    continue;
                };
                if i == j {
                    continue;
                }
                let pair = if i < j { (i, j) } else { (j, i) };
                if tile_pairs.insert(pair) {
                    tile_adj[i].push(j);
                    tile_adj[j].push(i);
                }
            }
        }
        snap.tile_edges = tile_pairs.len();
        snap.tile_blocks = component_sizes(&tile_adj);

        // Stem edges: connections are (cell_id, strength), counted as the field chose them, skipping zero strength only
        let stem_index: HashMap<&str, usize> = stem
            .cells
            .iter()
            .enumerate()
            .map(|(i, c)| (c.cell_id.as_str(), i))
            .collect();
        let mut stem_adj: Vec<Vec<usize>> = vec![Vec::new(); stem.cells.len()];
        for (i, cell) in stem.cells.iter().enumerate() {
            for (peer, strength) in &cell.connections {
                if *strength <= 0.0 {
                    continue;
                }
                snap.stem_links += 1;
                if let Some(&j) = stem_index.get(peer.as_str()) {
                    if i != j {
                        stem_adj[i].push(j);
                        stem_adj[j].push(i);
                    }
                }
            }
        }
        snap.stem_blocks = component_sizes(&stem_adj);

        // The eight quad stores: per-store external view (the monitor reads only `stored` itself)
        snap.quad = [
            quad_view(&tile.blackhole.stored),
            quad_view(&tile.meshbrain.stored),
            quad_view(&tile.multinova.stored),
            quad_view(&tile.baseforce.stored),
            quad_view(&stem.blackhole.stored),
            quad_view(&stem.meshbrain.stored),
            quad_view(&stem.multinova.stored),
            quad_view(&stem.baseforce.stored),
        ];
        snap.stored_entries = snap.quad.iter().map(|q| q.entries).sum();
        snap.stored_bytes = snap.quad.iter().map(|q| q.bytes).sum();
        snap.quad_diverged = !(tile.blackhole.stored == tile.meshbrain.stored
            && tile.blackhole.stored == tile.multinova.stored
            && tile.blackhole.stored == tile.baseforce.stored)
            || !(stem.blackhole.stored == stem.meshbrain.stored
                && stem.blackhole.stored == stem.multinova.stored
                && stem.blackhole.stored == stem.baseforce.stored);

        snap
    }
}

/// Undirected connected-component sizes, descending, only >= 2 kept
fn component_sizes(adj: &[Vec<usize>]) -> Vec<usize> {
    let mut seen = vec![false; adj.len()];
    let mut sizes = Vec::new();
    for start in 0..adj.len() {
        if seen[start] {
            continue;
        }
        seen[start] = true;
        let mut stack = vec![start];
        let mut size = 0usize;
        while let Some(node) = stack.pop() {
            size += 1;
            for &next in &adj[node] {
                if !seen[next] {
                    seen[next] = true;
                    stack.push(next);
                }
            }
        }
        if size >= 2 {
            sizes.push(size);
        }
    }
    sizes.sort_by(|a, b| b.cmp(a));
    sizes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn watch_sees_quad_diverge_from_outside() {
        let mut tile = TileQuattroComplete::new();
        let mut stem = StemCellComplete::new();
        let mut monitor = BodyMonitor::new();
        tile.quad_keep(&vec![0u8; 512]);
        stem.quad_keep(&vec![0u8; 512]);
        tile.quad_keep(b"plain text entry for the monitor test");
        stem.quad_keep(b"plain text entry for the monitor test");
        let first = monitor.watch(&tile, &stem);
        assert_eq!(first.stored_entries, 16);
        assert!(!first.quad_diverged);
        tile.process_tiles_cycle();
        stem.process_cells_cycle();
        let second = monitor.watch(&tile, &stem);
        // With the cycles, the black hole swallows its own silent entries: external diff and divergence visible together
        assert!(second.quad_diverged);
        assert_eq!(second.quad_delta[0], -1);
        assert!(second.stored_entries < first.stored_entries);
    }
}
