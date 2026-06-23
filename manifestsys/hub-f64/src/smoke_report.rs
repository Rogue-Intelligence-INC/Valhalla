//! Shared smoke metrics for Hub / Tile / StemCell — used by CLI smoke and dashboard.

use serde::{Deserialize, Serialize};

use crate::signal_ingress::{EncodingMode, IngressBudget, IngressPayload, TriadSession};
use crate::{HubConfig, HubRequest, Priority, RequestType, ValhallaHub};
use stem_cell::StemCellComplete;
use valhalla_tile_quattro::TileQuattroComplete;

pub const SMOKE_CORPUS: &[&str] = &[
    "hello world",
    "你好世界",
    "こんにちは",
    "the cat runs fast",
    "红色 蓝色 绿色",
    "machine learning text aggregation",
    "neural structure emerges naturally",
    "data flows without preset roles",
    "Fate decides when to converge",
    "micro container no self preference",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmokeReport {
    pub hub_signals: u64,
    pub hub_routed: [u64; 4],
    pub hub_prefs: Vec<f64>,
    pub hub_ok: bool,
    pub tile_completed: usize,
    pub tile_ops: u64,
    pub tile_dim: f64,
    pub tile_ok: bool,
    pub stem_cells: usize,
    pub stem_connections: usize,
    pub stem_dim: f64,
    pub stem_ok: bool,
    pub overall_pass: bool,
    /// Encoding used for this run (`char` legacy path or ingress mode name).
    pub encoding_mode: String,
    pub ingress_items: usize,
    pub budget_exhausted: bool,
}

impl SmokeReport {
    /// Legacy char-per-scalar path (backward compatible).
    pub fn run() -> Self {
        Self::run_with_mode(EncodingMode::Char)
    }

    /// Run smoke through universal ingress with optional budget.
    pub fn run_with_mode(mode: EncodingMode) -> Self {
        Self::run_with_budget(mode, IngressBudget::default())
    }

    pub fn run_with_budget(mode: EncodingMode, budget: IngressBudget) -> Self {
        let mut session = TriadSession::new(budget);
        for text in SMOKE_CORPUS {
            session.feed(
                IngressPayload::Text {
                    content: text.to_string(),
                },
                mode,
            );
        }
        let mode_label = format!("{:?}", mode);
        let r = session.finalize(&mode_label);
        let hub_ok = r.hub_signals > 0 && r.hub_routed.iter().sum::<u64>() == r.hub_signals;
        let tile_ok = r.tile_completed > 0 || r.tile_ops > 0;
        let stem_ok = r.stem_cells > 0;

        SmokeReport {
            hub_signals: r.hub_signals,
            hub_routed: r.hub_routed,
            hub_prefs: r.hub_prefs,
            hub_ok,
            tile_completed: r.tile_completed,
            tile_ops: r.tile_ops,
            tile_dim: r.tile_dim,
            tile_ok,
            stem_cells: r.stem_cells,
            stem_connections: r.stem_connections,
            stem_dim: r.stem_dim,
            stem_ok,
            overall_pass: hub_ok && tile_ok && stem_ok,
            encoding_mode: r.mode,
            ingress_items: r.stats.items_processed,
            budget_exhausted: r.stats.budget_exhausted,
        }
    }

    /// Original per-system char loop (kept for regression diff).
    pub fn run_legacy_char() -> Self {
        let mut ok = true;

        let mut hub = ValhallaHub::with_config(HubConfig::default());
        let mut routed = [0u64; 4];
        let mut signals = 0u64;
        for text in SMOKE_CORPUS {
            for ch in text.chars() {
                let resp = hub.process(HubRequest {
                    request_id: format!("s{signals}"),
                    request_type: RequestType::Learning,
                    intensity: (ch as u32 as f64 / 255.0).clamp(0.0, 1.0),
                    data: None,
                    priority: Priority::Normal,
                });
                let idx = if resp.message.contains("BlackHole") {
                    0
                } else if resp.message.contains("MeshBrain") {
                    1
                } else if resp.message.contains("MultiNova") {
                    2
                } else {
                    3
                };
                routed[idx] += 1;
                signals += 1;
            }
        }
        hub.mark_epoch_complete();
        let prefs = hub.get_fate_preferences();
        let hub_ok = signals > 0 && routed.iter().sum::<u64>() == signals;
        ok &= hub_ok;

        let mut tile = TileQuattroComplete::new();
        for (i, text) in SMOKE_CORPUS.iter().enumerate() {
            tile.receive_data(text.len() as f64 + i as f64);
            for ch in text.chars() {
                tile.receive_data(ch as u32 as f64);
            }
        }
        let _ = tile.process_tiles_cycle();
        let tstatus = tile.get_status();
        let tile_ok = tstatus.completed_tiles > 0 || tstatus.total_operations > 0;
        ok &= tile_ok;

        let mut stem = StemCellComplete::new();
        for text in SMOKE_CORPUS {
            for ch in text.chars() {
                stem.receive_data((ch as u32 as f64 / 65536.0) * 2.0 - 1.0);
            }
            stem.receive_data(0.0);
        }
        let _ = stem.process_cells_cycle();
        let sstatus = stem.get_status();
        let connections: usize = stem.cells.iter().map(|c| c.connections.len()).sum();
        let stem_ok = sstatus.cell_count > 0;
        ok &= stem_ok;

        SmokeReport {
            hub_signals: signals,
            hub_routed: routed,
            hub_prefs: prefs.affinities.clone(),
            hub_ok,
            tile_completed: tstatus.completed_tiles,
            tile_ops: tstatus.total_operations,
            tile_dim: tstatus.dimension,
            tile_ok,
            stem_cells: sstatus.cell_count,
            stem_connections: connections,
            stem_dim: stem.space.current_dimension,
            stem_ok,
            overall_pass: ok,
            encoding_mode: "legacy_char".into(),
            ingress_items: SMOKE_CORPUS.len(),
            budget_exhausted: false,
        }
    }
}
