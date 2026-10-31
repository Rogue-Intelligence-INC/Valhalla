//! 孵化因果证明 v1（多 seed）：
//! C 培养 / A 空白 / F 完整状态恢复 / G 结构消融 / H 拓扑重排 / I 存储隔离。
//! 答题只走 Tile/Stem 自发网络（不读存储字节、不拼链），这样结构消融才测得到东西。
//! G = C − A（培养增益），L = C − G组（结构因果贡献）。不声称智能，只测因果。

use hub_f64::monitor::BodyMonitor;
use hub_f64::{BaseForceCoordinator, GravityField, MeshBrainSystem, MultinovaProcessor, TripleFateCombined};
use stem_cell::stemcell_complete::Cell;
use stem_cell::StemCellComplete;
use valhalla_tile_quattro::tile_complete::CompletedTile;
use valhalla_tile_quattro::TileQuattroComplete;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 33
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
}

fn symbol(i: usize) -> String {
    let a = (b'A' + (i / 26) as u8) as char;
    let b = (b'A' + (i % 26) as u8) as char;
    format!("{a}{b}")
}

fn shuffled_pool(rng: &mut Rng, pool: &[String], take: usize) -> Vec<String> {
    let mut v: Vec<String> = pool.to_vec();
    for i in (1..v.len()).rev() {
        let j = rng.below(i + 1);
        v.swap(i, j);
    }
    v.truncate(take);
    v
}

fn pairs_shuffled(chain: &[String], rng: &mut Rng) -> Vec<Vec<u8>> {
    let mut pairs: Vec<Vec<u8>> = (0..chain.len() - 1)
        .map(|i| format!("{}>{}", chain[i], chain[i + 1]).into_bytes())
        .collect();
    for i in (1..pairs.len()).rev() {
        let j = rng.below(i + 1);
        pairs.swap(i, j);
    }
    pairs
}

struct Body {
    tile: TileQuattroComplete,
    stem: StemCellComplete,
    monitor: BodyMonitor,
    global: TripleFateCombined,
}

impl Body {
    fn new() -> Self {
        Self {
            tile: TileQuattroComplete::new(),
            stem: StemCellComplete::new(),
            monitor: BodyMonitor::new(),
            global: TripleFateCombined::new(),
        }
    }
    fn feed(&mut self, item: &[u8]) {
        self.tile.quad_keep(item);
        self.stem.quad_keep(item);
        let mean = item.iter().map(|b| *b as f64).sum::<f64>() / item.len().max(1) as f64;
        for b in item {
            self.global.receive_passive(*b as f64);
            self.stem.ingest(*b as f64);
        }
        self.stem.promote_from_fate();
        self.tile.ingest(mean);
        self.tile.promote_from_fate();
        self.tile.follow_global_fate(&self.global);
        self.stem.follow_global_fate(&self.global);
    }
    fn feed_all(&mut self, items: &[Vec<u8>]) {
        for it in items {
            self.feed(it);
        }
    }
    fn feed_tracked(&mut self, item: &[u8]) -> (Vec<String>, Vec<String>) {
        let tiles_before: std::collections::HashSet<String> =
            self.tile.completed_tiles.iter().map(|t| t.id.clone()).collect();
        let cells_before: std::collections::HashSet<String> =
            self.stem.cells.iter().map(|c| c.cell_id.clone()).collect();
        self.feed(item);
        let tiles: Vec<String> = self.tile.completed_tiles.iter()
            .map(|t| t.id.clone()).filter(|id| !tiles_before.contains(id)).collect();
        let cells: Vec<String> = self.stem.cells.iter()
            .map(|c| c.cell_id.clone()).filter(|id| !cells_before.contains(id)).collect();
        (tiles, cells)
    }
    fn cycle(&mut self) {
        let t = self.tile.process_tiles_cycle();
        let s = self.stem.process_cells_cycle();
        self.stem.settle_field();
        let p = self.tile.propagate(1.0);
        self.monitor.observe_propagate(p.new_connections);
        self.monitor.observe_cycle(&t, &s);
    }
    fn save_quads(&self, dir: &std::path::Path) -> std::io::Result<()> {
        std::fs::create_dir_all(dir)?;
        self.tile.blackhole.save_store(&dir.join("tile.blackhole.store"))?;
        self.tile.meshbrain.save_store(&dir.join("tile.meshbrain.store"))?;
        self.tile.multinova.save_store(&dir.join("tile.multinova.store"))?;
        self.tile.baseforce.save_store(&dir.join("tile.baseforce.store"))?;
        self.stem.blackhole.save_store(&dir.join("stem.blackhole.store"))?;
        self.stem.meshbrain.save_store(&dir.join("stem.meshbrain.store"))?;
        self.stem.multinova.save_store(&dir.join("stem.multinova.store"))?;
        self.stem.baseforce.save_store(&dir.join("stem.baseforce.store"))?;
        Ok(())
    }
    fn load_quads(&mut self, dir: &std::path::Path) -> std::io::Result<()> {
        self.tile.blackhole = GravityField::load_store(&dir.join("tile.blackhole.store"))?;
        self.tile.meshbrain = MeshBrainSystem::load_store(&dir.join("tile.meshbrain.store"))?;
        self.tile.multinova = MultinovaProcessor::load_store(&dir.join("tile.multinova.store"))?;
        self.tile.baseforce = BaseForceCoordinator::load_store(&dir.join("tile.baseforce.store"))?;
        self.stem.blackhole = GravityField::load_store(&dir.join("stem.blackhole.store"))?;
        self.stem.meshbrain = MeshBrainSystem::load_store(&dir.join("stem.meshbrain.store"))?;
        self.stem.multinova = MultinovaProcessor::load_store(&dir.join("stem.multinova.store"))?;
        self.stem.baseforce = BaseForceCoordinator::load_store(&dir.join("stem.baseforce.store"))?;
        Ok(())
    }
    fn clear_quads(&mut self) {
        self.tile.blackhole.stored_clear();
        self.tile.meshbrain.stored_clear();
        self.tile.multinova.stored_clear();
        self.tile.baseforce.stored_clear();
        self.stem.blackhole.stored_clear();
        self.stem.meshbrain.stored_clear();
        self.stem.multinova.stored_clear();
        self.stem.baseforce.stored_clear();
    }
    /// 完整状态：身体切片（Fate+计数）+ 容器本体 JSON + 四元存储 + 全局 Fate
    fn save_full(&self, dir: &std::path::Path) -> std::io::Result<()> {
        std::fs::create_dir_all(dir)?;
        self.tile.save_slice(&dir.join("tile.slice"))?;
        self.stem.save_slice(&dir.join("stem.slice"))?;
        std::fs::write(dir.join("tiles.json"), serde_json::to_string(&self.tile.completed_tiles).unwrap())?;
        std::fs::write(dir.join("cells.json"), serde_json::to_string(&self.stem.cells).unwrap())?;
        self.global.save_slice(&dir.join("global.fate"))?;
        self.save_quads(dir)
    }
    fn load_full(dir: &std::path::Path) -> std::io::Result<Self> {
        let mut body = Self {
            tile: TileQuattroComplete::load_slice(&dir.join("tile.slice"))?,
            stem: StemCellComplete::load_slice(&dir.join("stem.slice"))?,
            monitor: BodyMonitor::new(),
            global: TripleFateCombined::load_slice(&dir.join("global.fate"))?,
        };
        body.tile.completed_tiles =
            serde_json::from_str(&std::fs::read_to_string(dir.join("tiles.json"))?).unwrap_or_default();
        body.stem.cells =
            serde_json::from_str(&std::fs::read_to_string(dir.join("cells.json"))?).unwrap_or_default();
        body.load_quads(dir)?;
        Ok(body)
    }
    /// G：拆掉全部结构连接，存储原样保留
    fn ablate_structure(&mut self) {
        for t in self.tile.completed_tiles.iter_mut() {
            t.connections.clear();
            t.weights.clear();
        }
        for c in self.stem.cells.iter_mut() {
            c.connections.clear();
        }
    }
    /// H：保持每个节点的连接数（与 Stem 强度）不变，只把目标随机重排
    fn shuffle_structure(&mut self, rng: &mut Rng) {
        let tile_ids: Vec<String> = self.tile.completed_tiles.iter().map(|t| t.id.clone()).collect();
        for t in self.tile.completed_tiles.iter_mut() {
            let d = t.connections.len();
            let mut targets: Vec<String> = Vec::new();
            let mut guard = 0;
            while targets.len() < d && guard < d * 20 + 10 {
                guard += 1;
                let cand = tile_ids[rng.below(tile_ids.len())].clone();
                if cand != t.id && !targets.contains(&cand) {
                    targets.push(cand);
                }
            }
            t.connections = targets;
            t.weights.resize(d, 0.0);
        }
        let cell_ids: Vec<String> = self.stem.cells.iter().map(|c| c.cell_id.clone()).collect();
        for c in self.stem.cells.iter_mut() {
            let strengths: Vec<f64> = c.connections.iter().map(|(_, s)| *s).collect();
            let mut targets: Vec<String> = Vec::new();
            let mut guard = 0;
            while targets.len() < strengths.len() && guard < strengths.len() * 20 + 10 {
                guard += 1;
                let cand = cell_ids[rng.below(cell_ids.len())].clone();
                if cand != c.cell_id && !targets.contains(&cand) {
                    targets.push(cand);
                }
            }
            c.connections = targets.into_iter().zip(strengths).collect();
        }
    }
}

type Adj = std::collections::HashMap<String, Vec<String>>;

fn net_adj(body: &Body, stem: bool) -> Adj {
    let mut adj: Adj = Adj::new();
    if stem {
        for c in &body.stem.cells {
            for (to, _) in &c.connections {
                adj.entry(c.cell_id.clone()).or_default().push(to.clone());
            }
        }
    } else {
        for t in &body.tile.completed_tiles {
            for to in &t.connections {
                adj.entry(t.id.clone()).or_default().push(to.clone());
            }
        }
    }
    adj
}

fn net_reachable(adj: &Adj, from: &[String], to: &[String]) -> bool {
    if from.is_empty() || to.is_empty() {
        return false;
    }
    let targets: std::collections::HashSet<&String> = to.iter().collect();
    if from.iter().any(|f| targets.contains(f)) {
        return true;
    }
    let mut seen: std::collections::HashSet<String> = from.iter().cloned().collect();
    let mut stack: Vec<String> = from.to_vec();
    while let Some(cur) = stack.pop() {
        if let Some(nexts) = adj.get(&cur) {
            for n in nexts {
                if targets.contains(n) {
                    return true;
                }
                if seen.insert(n.clone()) {
                    stack.push(n.clone());
                }
            }
        }
    }
    false
}

struct Question {
    from: String,
    to: String,
    expect_reach: bool,
}

type SymNodes = std::collections::HashMap<String, Vec<String>>;

/// 网络通道答题：Tile 或 Stem 任一可达即答（只走结构，不读存储）
fn score_net(body: &Body, qs: &[Question], sym_tiles: &SymNodes, sym_cells: &SymNodes) -> f64 {
    let adj_t = net_adj(body, false);
    let adj_s = net_adj(body, true);
    let empty: Vec<String> = Vec::new();
    let mut ok = 0usize;
    for q in qs {
        let ft = sym_tiles.get(&q.from).unwrap_or(&empty);
        let tt = sym_tiles.get(&q.to).unwrap_or(&empty);
        let fs = sym_cells.get(&q.from).unwrap_or(&empty);
        let ts = sym_cells.get(&q.to).unwrap_or(&empty);
        let got = net_reachable(&adj_t, ft, tt) || net_reachable(&adj_s, fs, ts);
        if got == q.expect_reach {
            ok += 1;
        }
    }
    ok as f64 / qs.len().max(1) as f64 * 100.0
}

fn feed_facts_tracked(body: &mut Body, facts: &[Vec<u8>]) -> (SymNodes, SymNodes) {
    let mut sym_tiles: SymNodes = SymNodes::new();
    let mut sym_cells: SymNodes = SymNodes::new();
    for item in facts {
        let (tiles, cells) = body.feed_tracked(item);
        let text = String::from_utf8_lossy(item).into_owned();
        for sym in text.split(|c| c == '>' || c == '<') {
            if sym.len() == 2 {
                sym_tiles.entry(sym.to_string()).or_default().extend(tiles.iter().cloned());
                sym_cells.entry(sym.to_string()).or_default().extend(cells.iter().cloned());
            }
        }
    }
    (sym_tiles, sym_cells)
}

fn cultivate(rng: &mut Rng, train_pool: &[String]) -> (Body, usize) {
    let mut body = Body::new();
    let mut items = 0usize;
    for _ in 0..40 {
        let chain = shuffled_pool(rng, train_pool, 3);
        let it = pairs_shuffled(&chain, rng);
        items += it.len();
        body.feed_all(&it);
    }
    body.cycle();
    println!("    阶段1 {}", body.monitor.snapshot(&body.tile, &body.stem).line());
    for _ in 0..40 {
        let chain = shuffled_pool(rng, train_pool, 4);
        let mut it = vec![chain.join(">").into_bytes()];
        it.extend(pairs_shuffled(&chain, rng));
        items += it.len();
        body.feed_all(&it);
    }
    body.cycle();
    println!("    阶段2 {}", body.monitor.snapshot(&body.tile, &body.stem).line());
    for _ in 0..40 {
        let chain = shuffled_pool(rng, train_pool, 4);
        let mut it = Vec::new();
        let rev: Vec<String> = chain.iter().rev().cloned().collect();
        it.push(rev.join("<").into_bytes());
        it.extend(pairs_shuffled(&chain, rng));
        let frag = shuffled_pool(rng, train_pool, 3);
        it.push(format!("{}>{}", frag[0], frag[1]).into_bytes());
        let dis = shuffled_pool(rng, train_pool, 2);
        it.push(format!("{}>{}", dis[0], dis[1]).into_bytes());
        items += it.len();
        body.feed_all(&it);
    }
    body.cycle();
    println!("    阶段3 {}", body.monitor.snapshot(&body.tile, &body.stem).line());
    (body, items)
}

fn run_seed(seed: u64) -> (f64, f64, f64, f64, f64, f64) {
    println!("== seed {seed} ==");
    let train_pool: Vec<String> = (0..30).map(symbol).collect();
    let test_base = 100 + (seed as usize) * 80;
    let test_pool: Vec<String> = (test_base..test_base + 80).map(symbol).collect();
    let mut rng = Rng(0x5EED_0000 + seed * 7919);

    let (mut body_c, items_c) = cultivate(&mut rng, &train_pool);

    // A 空白组：等条数无关字节
    let mut body_a = Body::new();
    let noise = ["lorem", "ipsum", "dolor", "amet", "velit", "sed", "tempor", "magna", "aliqua", "enim"];
    for _ in 0..items_c {
        let w = noise[rng.below(noise.len())];
        let w2 = noise[rng.below(noise.len())];
        body_a.feed(format!("{w} {w2}").as_bytes());
    }
    body_a.cycle();

    // 测试电池：每链独占符号，只给单环
    let mut rng_t = Rng(0x7E57_0000 + seed * 104729);
    let mut questions: Vec<Question> = Vec::new();
    let mut fact_items: Vec<Vec<u8>> = Vec::new();
    let mut cursor = 0usize;
    let mut take_chain = |rng: &mut Rng, cursor: &mut usize| -> Vec<String> {
        let mut chain: Vec<String> = test_pool[*cursor..*cursor + 4].to_vec();
        *cursor += 4;
        for i in (1..chain.len()).rev() {
            let j = rng.below(i + 1);
            chain.swap(i, j);
        }
        chain
    };
    let mut full_chains: Vec<Vec<String>> = Vec::new();
    for _ in 0..12 {
        let chain = take_chain(&mut rng_t, &mut cursor);
        fact_items.extend(pairs_shuffled(&chain, &mut rng_t));
        questions.push(Question { from: chain[0].clone(), to: chain[3].clone(), expect_reach: true });
        questions.push(Question { from: chain[0].clone(), to: chain[2].clone(), expect_reach: true });
        questions.push(Question { from: chain[1].clone(), to: chain[3].clone(), expect_reach: true });
        questions.push(Question { from: chain[3].clone(), to: chain[0].clone(), expect_reach: false });
        full_chains.push(chain);
    }
    for _ in 0..6 {
        let chain = take_chain(&mut rng_t, &mut cursor);
        fact_items.push(format!("{}>{}", chain[0], chain[1]).into_bytes());
        fact_items.push(format!("{}>{}", chain[2], chain[3]).into_bytes());
        questions.push(Question { from: chain[0].clone(), to: chain[3].clone(), expect_reach: false });
        questions.push(Question { from: chain[0].clone(), to: chain[1].clone(), expect_reach: true });
    }
    for chain in full_chains.iter().take(6) {
        let other = test_pool[cursor].clone();
        cursor += 1;
        questions.push(Question { from: chain[0].clone(), to: other, expect_reach: false });
    }

    let (c_sym_t, c_sym_s) = feed_facts_tracked(&mut body_c, &fact_items);
    let (a_sym_t, a_sym_s) = feed_facts_tracked(&mut body_a, &fact_items);
    body_c.cycle();
    body_a.cycle();

    let sc = score_net(&body_c, &questions, &c_sym_t, &c_sym_s);
    let sa = score_net(&body_a, &questions, &a_sym_t, &a_sym_s);

    // F/G/H/I 都从同一份完整存档恢复，互不影响
    let dir = std::env::temp_dir().join(format!("valhalla_proof_seed{seed}"));
    body_c.save_full(&dir).expect("完整存档失败");
    let mut body_f = Body::load_full(&dir).expect("完整恢复失败");
    let sf = score_net(&body_f, &questions, &c_sym_t, &c_sym_s);

    let mut body_g = Body::load_full(&dir).expect("恢复失败");
    body_g.ablate_structure();
    let sg = score_net(&body_g, &questions, &c_sym_t, &c_sym_s);

    let mut body_h = Body::load_full(&dir).expect("恢复失败");
    let mut rng_h = Rng(0x45EED + seed);
    body_h.shuffle_structure(&mut rng_h);
    let sh = score_net(&body_h, &questions, &c_sym_t, &c_sym_s);

    let mut body_i = Body::load_full(&dir).expect("恢复失败");
    body_i.clear_quads();
    let si = score_net(&body_i, &questions, &c_sym_t, &c_sym_s);

    println!("seed {seed}: C {sc:.1} A {sa:.1} F {sf:.1} G(拆结构) {sg:.1} H(重排) {sh:.1} I(清存储) {si:.1} | G增益 {:+.1} L {:+.1}",
        sc - sa, sc - sg);
    (sc, sa, sf, sg, sh, si)
}

fn main() {
    let seeds: Vec<u64> = std::env::var("VALHALLA_PROOF_SEEDS")
        .unwrap_or_else(|_| "1,2,3,4,5".to_string())
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    let mut rows: Vec<(f64, f64, f64, f64, f64, f64)> = Vec::new();
    for seed in &seeds {
        rows.push(run_seed(*seed));
    }
    let n = rows.len() as f64;
    let mean = |f: &dyn Fn(&(f64, f64, f64, f64, f64, f64)) -> f64| rows.iter().map(f).sum::<f64>() / n;
    let std = |f: &dyn Fn(&(f64, f64, f64, f64, f64, f64)) -> f64| {
        let m = mean(f);
        (rows.iter().map(|r| (f(r) - m).powi(2)).sum::<f64>() / n).sqrt()
    };
    let mc = mean(&|r| r.0);
    let ma = mean(&|r| r.1);
    let mf = mean(&|r| r.2);
    let mg = mean(&|r| r.3);
    let mh = mean(&|r| r.4);
    let mi = mean(&|r| r.5);
    let gain = mean(&|r| r.0 - r.1);
    let gain_std = std(&|r| r.0 - r.1);
    let loss = mean(&|r| r.0 - r.3);
    let loss_std = std(&|r| r.0 - r.3);
    println!("\n== 因果证明汇总（{} seeds，网络通道，%） ==", rows.len());
    println!("C {mc:.1}±{:.1}  A {ma:.1}±{:.1}  F {mf:.1}±{:.1}  G拆结构 {mg:.1}±{:.1}  H重排 {mh:.1}±{:.1}  I清存储 {mi:.1}±{:.1}",
        std(&|r| r.0), std(&|r| r.1), std(&|r| r.2), std(&|r| r.3), std(&|r| r.4), std(&|r| r.5));
    println!("培养增益 G = C−A = {gain:+.1} ± {gain_std:.1} pp");
    println!("结构因果 L = C−G = {loss:+.1} ± {loss_std:.1} pp");
    println!("判读：F 应与 C 一致（完整状态可恢复）；G/H 掉分说明结构承担答题；I 不掉说明网络通道不靠存储字节。");
}

