//! 训练时监控：Tile 组合、Stem 组合、自发连接、连通块。
//!
//! 只读身体当下的状态，不给块起名字，不看四路偏好。
//! - 组合数来自周期结果（Tile 没有累计字段，由监控逐周期加上；Stem 另有 total_merges）
//! - 自发边：由 propagate 自己报的新连接数逐次累计，监控不另设阈值猜
//! - Stem 连接：场写出的连接原样计数（强度为场按密度/距离/场强选出的原值）
//! - 连通块：无向图连通分量，只报大小，不命名

use std::collections::{HashMap, HashSet};

use stem_cell::stemcell_complete::CellCycleResult;
use stem_cell::StemCellComplete;
use valhalla_tile_quattro::tile_complete::TileCycleResult;
use valhalla_tile_quattro::TileQuattroComplete;

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
            "monitor tiles {} cells {} tile_merges {} stem_merges {} tile_edges {} synapses_born {} stem_links {} tile_block {} stem_block {}",
            self.tiles,
            self.cells,
            self.tile_merges,
            self.stem_merges,
            self.tile_edges,
            self.tile_synapses_born,
            self.stem_links,
            self.largest_tile_block(),
            self.largest_stem_block(),
        )
    }
}

/// 训练时挂在周期旁边的监控。自己不改身体。
#[derive(Debug, Default)]
pub struct BodyMonitor {
    tile_merges: usize,
    synapses_born: usize,
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
