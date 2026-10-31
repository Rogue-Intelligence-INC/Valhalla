//! Fate 全局决定探针：同一喂法喂两个身体。
//! X 组：全局 Fate 逐字节走字，两体 follow_global_fate 跟随（现行接线）。
//! Y 组：全局 Fate 从不走字、从不跟随（Fate 冻结在初态）。
//! 5 轮后比较结构：哪些数字因 Fate 而不同，哪些完全一样。
//! 一样的那些，就是 Fate 没有参与决定的部分。不声称智能，只查决策回流。

use hub_f64::monitor::BodyMonitor;
use hub_f64::TripleFateCombined;
use stem_cell::StemCellComplete;
use valhalla_tile_quattro::TileQuattroComplete;

struct Body {
    tile: TileQuattroComplete,
    stem: StemCellComplete,
    monitor: BodyMonitor,
    global: TripleFateCombined,
    fate_live: bool,
}

impl Body {
    fn new(fate_live: bool) -> Self {
        Self {
            tile: TileQuattroComplete::new(),
            stem: StemCellComplete::new(),
            monitor: BodyMonitor::new(),
            global: TripleFateCombined::new(),
            fate_live,
        }
    }
    fn feed(&mut self, item: &[u8]) {
        self.tile.quad_keep(item);
        self.stem.quad_keep(item);
        let mean = item.iter().map(|b| *b as f64).sum::<f64>() / item.len().max(1) as f64;
        for b in item {
            if self.fate_live {
                self.global.receive_passive(*b as f64);
            }
            self.stem.ingest(*b as f64);
        }
        self.stem.promote_from_fate();
        self.tile.ingest(mean);
        self.tile.promote_from_fate();
        if self.fate_live {
            self.tile.follow_global_fate(&self.global);
            self.stem.follow_global_fate(&self.global);
        }
    }
    fn cycle(&mut self) {
        let t = self.tile.process_tiles_cycle();
        let s = self.stem.process_cells_cycle();
        self.stem.settle_field();
        let p = self.tile.propagate(1.0);
        self.monitor.observe_propagate(p.new_connections);
        self.monitor.observe_cycle(&t, &s);
    }
    fn fate_line(&self) -> String {
        let g = &self.global;
        format!(
            "global[eff {:.4} sig {} disc {} base_rules {} meta_rules {} eq {}]",
            g.base.state.efficiency,
            g.base.state.signal_count,
            g.base.state.discovery_count,
            g.base.rules.len(),
            g.meta.emergent_rules.len(),
            g.cluster_equation.equation_str(),
        )
    }
}

fn main() {
    // 与关系培养相同的事实流：链 3→4 的单环 + 整链，符号池 30
    let pool: Vec<String> = (0..30).map(|i| {
        let a = (b'A' + (i / 26) as u8) as char;
        let b = (b'A' + (i % 26) as u8) as char;
        format!("{a}{b}")
    }).collect();
    let mut items: Vec<Vec<u8>> = Vec::new();
    let mut seed = 0x5EED_2026u64;
    let mut next = move || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        seed >> 33
    };
    for _ in 0..60 {
        let len = 3 + (next() % 2) as usize;
        let mut chain: Vec<String> = Vec::new();
        while chain.len() < len {
            let s = pool[(next() % 30) as usize].clone();
            if !chain.contains(&s) {
                chain.push(s);
            }
        }
        items.push(chain.join(">").into_bytes());
        for i in 0..chain.len() - 1 {
            items.push(format!("{}>{}", chain[i], chain[i + 1]).into_bytes());
        }
    }

    let mut x = Body::new(true);
    let mut y = Body::new(false);
    println!("== Fate 全局决定探针（5 轮，同一批 {} 条事实） ==", items.len());
    for round in 1..=5 {
        for it in &items {
            x.feed(it);
            y.feed(it);
        }
        x.cycle();
        y.cycle();
        let sx = x.monitor.snapshot(&x.tile, &x.stem);
        let sy = y.monitor.snapshot(&y.tile, &y.stem);
        println!("round {round} X {}", sx.line());
        println!("round {round} Y {}", sy.line());
        let diff = |a: usize, b: usize| if a == b { "同" } else { "异" };
        println!(
            "   对比 tiles {} cells {} tile_edges {} stem_links {} stem_block {}",
            diff(sx.tiles, sy.tiles),
            diff(sx.cells, sy.cells),
            diff(sx.tile_edges, sy.tile_edges),
            diff(sx.stem_links, sy.stem_links),
            diff(sx.largest_stem_block(), sy.largest_stem_block()),
        );
    }
    println!("\nX {}", x.fate_line());
    println!("Y {}", y.fate_line());
    println!("\n判读：X/Y 一样的结构量，Fate 全局状态没有参与决定；不一样的，才是 Fate 真正定到的地方。");
}
