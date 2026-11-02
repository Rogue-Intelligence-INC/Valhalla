//! 整体监控：坐在身体外部，只读观察 Tile/Stem 结构、四元
//! 八份存储与全局 Fate。
//!
//! 只读身体当下的状态，不给块起名字，不看四路偏好，自己不
//! 改身体、不参与任何判定（Fate 与监控都是观察层）。
//! - 组合数来自周期结果（Tile 没有累计字段，由监控逐周期加上；Stem 另有 total_merges）
//! - 自发边：由 propagate 自己报的新连接数逐次累计，监控不另设阈值猜
//! - Stem 连接：场写出的连接原样计数（强度为场按密度/距离/场强选出的原值）
//! - 连通块：无向图连通分量，只报大小，不命名
//! - 四元：八份存储逐份读条目数/字节数/拼接指纹；四元的动静
//!   由相邻两次快照的差分从外部看出（监控不读身体内部计数）

use std::collections::{HashMap, HashSet};

use fate_f64::TripleFateCombined;
use stem_cell::stemcell_complete::CellCycleResult;
use stem_cell::StemCellComplete;
use valhalla_tile_quattro::tile_complete::TileCycleResult;
use valhalla_tile_quattro::TileQuattroComplete;

/// 八份四元存储的固定顺序名（Tile 四元、Stem 四元）。
pub const QUAD_NAMES: [&str; 8] = [
    "Tile黑洞", "Tile膜脑", "Tile新星", "Tile底力",
    "Stem黑洞", "Stem膜脑", "Stem新星", "Stem底力",
];

/// 监控从外部看到的一份四元存储：条目数、总字节、拼接指纹。
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

/// 一次身体快照。训练中每个周期取一次。
#[derive(Debug, Clone, Default)]
pub struct BodySnapshot {
    pub tiles: usize,
    pub cells: usize,
    /// 监控累计的 Tile 合并数
    pub tile_merges: usize,
    /// Stem 内部累计的合并数
    pub stem_merges: u64,
    /// Tile 无向边数
    pub tile_edges: usize,
    /// propagate 累计报出的自发新连接数
    pub tile_synapses_born: usize,
    /// Stem 场写出的连接条数（原样强度）
    pub stem_links: usize,
    /// Tile 连通块大小，从大到小，只留 >= 2 的
    pub tile_blocks: Vec<usize>,
    /// Stem 连通块大小，从大到小，只留 >= 2 的
    pub stem_blocks: Vec<usize>,
    /// 八份四元存储的外部视图（顺序见 QUAD_NAMES）
    pub quad: [QuadView; 8],
    /// 八份存储条目总数
    pub stored_entries: usize,
    /// 八份存储总字节
    pub stored_bytes: usize,
    /// 同层四份是否已分化（四元按各自规则维护后为 true）
    pub quad_diverged: bool,
    /// 相对上一次 watch 的每份条目数变化（外部差分；首次为 0）
    pub quad_delta: [i64; 8],
    /// 全局 Fate tick（仅 watch_with_fate 填写）
    pub fate_tick: u64,
    /// 全局 Fate 存储条数（仅 watch_with_fate 填写）
    pub fate_stored: usize,
    /// 全局 Fate 簇方程（仅 watch_with_fate 填写）
    pub fate_equation: String,
}

impl BodySnapshot {
    pub fn largest_tile_block(&self) -> usize {
        self.tile_blocks.first().copied().unwrap_or(0)
    }

    pub fn largest_stem_block(&self) -> usize {
        self.stem_blocks.first().copied().unwrap_or(0)
    }

    /// 训练日志用的单行形式
    pub fn line(&self) -> String {
        format!(
            "monitor tiles {} cells {} tile_merges {} stem_merges {} tile_edges {} synapses_born {} stem_links {} tile_block {} stem_block {} stored {}条/{}B quad_diverged {}",
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

    /// 四元八份的单行形式：每份 条目/字节/指纹 + 本次差分
    pub fn quad_line(&self) -> String {
        let mut out = String::from("quad");
        for (i, q) in self.quad.iter().enumerate() {
            out.push_str(&format!(
                " {} {}条/{}B#{:08x}({:+})",
                QUAD_NAMES[i], q.entries, q.bytes, (q.hash & 0xffff_ffff) as u32, self.quad_delta[i]
            ));
        }
        out
    }

    /// 全局 Fate 的单行形式（watch_with_fate 后才有内容）
    pub fn fate_line(&self) -> String {
        format!("fate tick {} stored {} eq {}", self.fate_tick, self.fate_stored, self.fate_equation)
    }
}

/// 训练时挂在周期旁边的整体监控。坐在外部，自己不改身体。
#[derive(Debug, Default)]
pub struct BodyMonitor {
    tile_merges: usize,
    synapses_born: usize,
    /// 上一次 watch 时八份的条目数，供外部差分
    prev_quad_entries: Option<[usize; 8]>,
}

impl BodyMonitor {
    pub fn new() -> Self {
        Self::default()
    }

    /// 每个周期调用一次，把这一周期的合并数记下来
    pub fn observe_cycle(&mut self, tile: &TileCycleResult, _stem: &CellCycleResult) {
        self.tile_merges += tile.merges;
    }

    /// propagate 每报一次新连接就累计（监控不另设阈值猜哪条是新生的）
    pub fn observe_propagate(&mut self, new_connections: usize) {
        self.synapses_born += new_connections;
    }

    /// 整体观察一次：取快照并记下八份条目数，下次 watch 时
    /// 差分即四元在外部留下的动静。监控只读，不进身体内部。
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

    /// 同 watch，并把全局 Fate 的 tick/存储/方程一并读进来
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

        // Tile 边：connections 是对端 id。无向去重
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

        // Stem 边：connections 是 (cell_id, strength)，场选出的原样计数，只略过零强度
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

        // 四元八份：逐份外部视图（监控只读 stored 本身）
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

/// 无向图连通分量大小，从大到小，只留 >= 2 的
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
        // 周期一起，黑洞吞掉自家静默条：外部差分与分化同时可见
        assert!(second.quad_diverged);
        assert_eq!(second.quad_delta[0], -1);
        assert!(second.stored_entries < first.stored_entries);
    }
}
