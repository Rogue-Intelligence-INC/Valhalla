//! 孵化因果证明 v3.1（多 seed，组合规模定向决胜）：
//! 符号池 200、链长 6、培养量约 3600 条——随机配对在此规模覆盖崩溃，
//! 定向（链式）培养若有特异贡献，只能在这里显形（v2 玩具规模下为 +0.0）。
//! v3 首跑教训：无界可达判分在 200 符号下被随机覆盖饱和（真题恒 100、假题恒
//! 错，C 与 A 被天花板焊死，读数作废）；v3.1 判分改限步可达（预算=链上距离，
//! 假题预算 5），链式短路径与随机 blob 长路径才可分辨。身体与库零改动。
//! C 培养 / A matched 对照（同符号表、同条数、同节律，唯一差别是培养料无链结构）/
//! F 完整状态恢复 / G 结构消融 / H 拓扑重排零分布（同度重排 20 次取分布，看真实拓扑分位）/
//! I 存储隔离。题目按链上距离分层（d1 / d2-3 / d4-5）：定向优势按假说应集中在远距离。
//! 另测学习曲线：同 rng 初值截断重培养到各阶段，喂同一批测试事实答题。
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
    let a = (b'A' + ((i / 676) % 26) as u8) as char;
    let b = (b'A' + ((i / 26) % 26) as u8) as char;
    let c = (b'A' + (i % 26) as u8) as char;
    format!("{a}{b}{c}")
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

/// 限步可达（BFS 分层）：只数 ≤max_hops 条边的路径。
/// v3 教训：无界可达在组合规模下被随机覆盖饱和（真题恒 100、假题恒错），
/// 仪器对布线身份盲。限步后链式布线（短路径）与随机 blob（长路径）才分得开。
fn net_reachable(adj: &Adj, from: &[String], to: &[String], max_hops: usize) -> bool {
    if from.is_empty() || to.is_empty() {
        return false;
    }
    let targets: std::collections::HashSet<&String> = to.iter().collect();
    if from.iter().any(|f| targets.contains(f)) {
        return true;
    }
    let mut seen: std::collections::HashSet<String> = from.iter().cloned().collect();
    let mut frontier: Vec<String> = from.to_vec();
    for _ in 0..max_hops {
        let mut next_frontier: Vec<String> = Vec::new();
        for cur in &frontier {
            if let Some(nexts) = adj.get(cur) {
                for n in nexts {
                    if targets.contains(n) {
                        return true;
                    }
                    if seen.insert(n.clone()) {
                        next_frontier.push(n.clone());
                    }
                }
            }
        }
        if next_frontier.is_empty() {
            break;
        }
        frontier = next_frontier;
    }
    false
}

/// 单图无上限最短路（BFS）；不可达返回 None。
fn bfs_dist(adj: &Adj, from: &[String], to: &[String]) -> Option<usize> {
    let targets: std::collections::HashSet<&String> = to.iter().collect();
    if from.is_empty() || to.is_empty() {
        return None;
    }
    if from.iter().any(|f| targets.contains(f)) {
        return Some(0);
    }
    let mut seen: std::collections::HashSet<String> = from.iter().cloned().collect();
    let mut frontier: Vec<String> = from.to_vec();
    let mut depth = 0usize;
    while !frontier.is_empty() {
        depth += 1;
        let mut next_frontier: Vec<String> = Vec::new();
        for cur in &frontier {
            if let Some(nexts) = adj.get(cur) {
                for n in nexts {
                    if targets.contains(n) {
                        return Some(depth);
                    }
                    if seen.insert(n.clone()) {
                        next_frontier.push(n.clone());
                    }
                }
            }
        }
        frontier = next_frontier;
    }
    None
}

/// Tile/Stem 两图分别算最短路取小者。v3.2 预先登记的最后一个仪器（路径特异）：
/// 真题对平均最短路 C vs A，看链式培养是否让新事实被吸收得更"近"；
/// 二元可达两档（无界/限步）已先后饱和，不再追加新仪器钓鱼。
fn shortest_path(adj_t: &Adj, adj_s: &Adj, ft: &[String], tt: &[String], fs: &[String], ts: &[String]) -> Option<usize> {
    match (bfs_dist(adj_t, ft, tt), bfs_dist(adj_s, fs, ts)) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (a, b) => a.or(b),
    }
}

/// (真题平均最短路, 假题平均最短路, 真题不可达数, 假题不可达数)；不可达记惩罚 12。
fn path_stats(body: &Body, qs: &[Question], sym_tiles: &SymNodes, sym_cells: &SymNodes) -> (f64, f64, usize, usize) {
    let adj_t = net_adj(body, false);
    let adj_s = net_adj(body, true);
    let empty: Vec<String> = Vec::new();
    let (mut ts, mut fs, mut tn, mut fn_) = (0.0f64, 0.0f64, 0usize, 0usize);
    let (mut tu, mut fu) = (0usize, 0usize);
    for q in qs {
        let ft = sym_tiles.get(&q.from).unwrap_or(&empty);
        let tt = sym_tiles.get(&q.to).unwrap_or(&empty);
        let fsyms = sym_cells.get(&q.from).unwrap_or(&empty);
        let tsyms = sym_cells.get(&q.to).unwrap_or(&empty);
        let d = shortest_path(&adj_t, &adj_s, ft, tt, fsyms, tsyms);
        if q.expect_reach {
            tn += 1;
            ts += d.map_or(12.0, |v| v as f64);
            if d.is_none() { tu += 1; }
        } else {
            fn_ += 1;
            fs += d.map_or(12.0, |v| v as f64);
            if d.is_none() { fu += 1; }
        }
    }
    (
        if tn > 0 { ts / tn as f64 } else { 0.0 },
        if fn_ > 0 { fs / fn_ as f64 } else { 0.0 },
        tu,
        fu,
    )
}

struct Question {
    from: String,
    to: String,
    expect_reach: bool,
    /// 链上真实距离（真题分层用；假题记 0）
    dist: usize,
}

type SymNodes = std::collections::HashMap<String, Vec<String>>;

/// 网络通道答题（限步）：统一预算 = 全链长-1 = 5，真假题同尺。
/// （2026-11-01 仪器修正：旧版真题预算取链上真实距离，等于把标签
/// 泄漏进评测条件，绝对分数偏高；消融差 L 因同尺对照受污染较轻，
/// 但绝对值自本版起以统一预算为准，旧绝对值销账。）
/// Tile 或 Stem 任一限步可达即"答可达"（只走结构，不读存储）。
const NET_BUDGET: usize = 5;
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
        let budget = NET_BUDGET;
        let got = net_reachable(&adj_t, ft, tt, budget) || net_reachable(&adj_s, fs, ts, budget);
        if got == q.expect_reach {
            ok += 1;
        }
    }
    ok as f64 / qs.len().max(1) as f64 * 100.0
}

/// 距离分层答题：真题按 dist 分 d1 / d2-3 / d4-5 三档，假题单列正确率。
/// 定向培养的特异贡献按假说应集中在远距离档。
fn score_detail(body: &Body, qs: &[Question], sym_tiles: &SymNodes, sym_cells: &SymNodes) -> ([f64; 3], f64) {
    let adj_t = net_adj(body, false);
    let adj_s = net_adj(body, true);
    let empty: Vec<String> = Vec::new();
    let mut ok = [0usize; 3];
    let mut tot = [0usize; 3];
    let (mut fok, mut ftot) = (0usize, 0usize);
    for q in qs {
        let ft = sym_tiles.get(&q.from).unwrap_or(&empty);
        let tt = sym_tiles.get(&q.to).unwrap_or(&empty);
        let fs = sym_cells.get(&q.from).unwrap_or(&empty);
        let ts = sym_cells.get(&q.to).unwrap_or(&empty);
        let budget = NET_BUDGET;
        let got = net_reachable(&adj_t, ft, tt, budget) || net_reachable(&adj_s, fs, ts, budget);
        if q.expect_reach {
            let b = if q.dist <= 1 { 0 } else if q.dist <= 3 { 1 } else { 2 };
            tot[b] += 1;
            if got {
                ok[b] += 1;
            }
        } else {
            ftot += 1;
            if !got {
                fok += 1;
            }
        }
    }
    (
        [
            if tot[0] > 0 { ok[0] as f64 / tot[0] as f64 * 100.0 } else { 0.0 },
            if tot[1] > 0 { ok[1] as f64 / tot[1] as f64 * 100.0 } else { 0.0 },
            if tot[2] > 0 { ok[2] as f64 / tot[2] as f64 * 100.0 } else { 0.0 },
        ],
        if ftot > 0 { fok as f64 / ftot as f64 * 100.0 } else { 0.0 },
    )
}

fn feed_facts_tracked(body: &mut Body, facts: &[Vec<u8>]) -> (SymNodes, SymNodes) {
    let mut sym_tiles: SymNodes = SymNodes::new();
    let mut sym_cells: SymNodes = SymNodes::new();
    for item in facts {
        let (tiles, cells) = body.feed_tracked(item);
        let text = String::from_utf8_lossy(item).into_owned();
        for sym in text.split(|c| c == '>' || c == '<') {
            if sym.len() == 3 {
                sym_tiles.entry(sym.to_string()).or_default().extend(tiles.iter().cloned());
                sym_cells.entry(sym.to_string()).or_default().extend(cells.iter().cloned());
            }
        }
    }
    (sym_tiles, sym_cells)
}

fn cultivate(rng: &mut Rng, train_pool: &[String], stop_after: usize, verbose: bool) -> (Body, Vec<usize>) {
    let mut body = Body::new();
    let mut stage_counts = Vec::new();
    let mut n = 0usize;
    for _ in 0..200 {
        let chain = shuffled_pool(rng, train_pool, 5);
        let it = pairs_shuffled(&chain, rng);
        n += it.len();
        body.feed_all(&it);
    }
    body.cycle();
    stage_counts.push(n);
    if verbose {
        println!("    阶段1 {}", body.monitor.snapshot(&body.tile, &body.stem).line());
    }
    if stop_after == 1 {
        return (body, stage_counts);
    }
    let mut n = 0usize;
    for _ in 0..200 {
        let chain = shuffled_pool(rng, train_pool, 6);
        let mut it = vec![chain.join(">").into_bytes()];
        it.extend(pairs_shuffled(&chain, rng));
        n += it.len();
        body.feed_all(&it);
    }
    body.cycle();
    stage_counts.push(n);
    if verbose {
        println!("    阶段2 {}", body.monitor.snapshot(&body.tile, &body.stem).line());
    }
    if stop_after == 2 {
        return (body, stage_counts);
    }
    let mut n = 0usize;
    for _ in 0..200 {
        let chain = shuffled_pool(rng, train_pool, 6);
        let mut it = Vec::new();
        let rev: Vec<String> = chain.iter().rev().cloned().collect();
        it.push(rev.join("<").into_bytes());
        it.extend(pairs_shuffled(&chain, rng));
        let frag = shuffled_pool(rng, train_pool, 3);
        it.push(format!("{}>{}", frag[0], frag[1]).into_bytes());
        let dis = shuffled_pool(rng, train_pool, 2);
        it.push(format!("{}>{}", dis[0], dis[1]).into_bytes());
        n += it.len();
        body.feed_all(&it);
    }
    body.cycle();
    stage_counts.push(n);
    if verbose {
        println!("    阶段3 {}", body.monitor.snapshot(&body.tile, &body.stem).line());
    }
    (body, stage_counts)
}

fn run_seed(seed: u64) -> (f64, f64, f64, f64, f64, f64, f64, f64, [f64; 3], [f64; 3], [f64; 3], f64, f64, f64, f64, f64, f64, f64) {
    println!("== seed {seed} ==");
    let train_pool: Vec<String> = (0..200).map(symbol).collect();
    let test_base = 400 + (seed as usize) * 200;
    let test_pool: Vec<String> = (test_base..test_base + 180).map(symbol).collect();
    let mut rng = Rng(0x5EED_0000 + seed * 7919);

    let dir = std::env::temp_dir().join(format!("valhalla_proof_seed{seed}"));
    let (mut body_c, stage_counts) = cultivate(&mut rng, &train_pool, 3, true);

    // A matched 对照：同符号表、同条数、同节律（按阶段分批 + 周期），
    // 唯一差别：培养料为随机配对，无链结构（传递性被打掉）。
    let mut body_a = Body::new();
    for &count in &stage_counts {
        for _ in 0..count {
            let a = train_pool[rng.below(train_pool.len())].clone();
            let mut b = train_pool[rng.below(train_pool.len())].clone();
            while b == a {
                b = train_pool[rng.below(train_pool.len())].clone();
            }
            body_a.feed(format!("{a}>{b}").as_bytes());
        }
        body_a.cycle();
    }

    // 测试电池：每链独占符号，只给单环
    let mut rng_t = Rng(0x7E57_0000 + seed * 104729);
    let mut questions: Vec<Question> = Vec::new();
    let mut fact_items: Vec<Vec<u8>> = Vec::new();
    let mut cursor = 0usize;
    let mut take_chain = |rng: &mut Rng, cursor: &mut usize| -> Vec<String> {
        let mut chain: Vec<String> = test_pool[*cursor..*cursor + 6].to_vec();
        *cursor += 6;
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
        questions.push(Question { from: chain[0].clone(), to: chain[5].clone(), expect_reach: true, dist: 5 });
        questions.push(Question { from: chain[0].clone(), to: chain[3].clone(), expect_reach: true, dist: 3 });
        questions.push(Question { from: chain[2].clone(), to: chain[5].clone(), expect_reach: true, dist: 3 });
        questions.push(Question { from: chain[5].clone(), to: chain[0].clone(), expect_reach: false, dist: 0 });
        full_chains.push(chain);
    }
    for _ in 0..6 {
        let chain = take_chain(&mut rng_t, &mut cursor);
        fact_items.push(format!("{}>{}", chain[0], chain[1]).into_bytes());
        fact_items.push(format!("{}>{}", chain[2], chain[3]).into_bytes());
        fact_items.push(format!("{}>{}", chain[4], chain[5]).into_bytes());
        questions.push(Question { from: chain[0].clone(), to: chain[5].clone(), expect_reach: false, dist: 5 });
        questions.push(Question { from: chain[0].clone(), to: chain[1].clone(), expect_reach: true, dist: 1 });
    }
    for chain in full_chains.iter().take(6) {
        let other = test_pool[cursor].clone();
        cursor += 1;
        questions.push(Question { from: chain[0].clone(), to: other, expect_reach: false, dist: 0 });
    }

    let (c_sym_t, c_sym_s) = feed_facts_tracked(&mut body_c, &fact_items);
    let (a_sym_t, a_sym_s) = feed_facts_tracked(&mut body_a, &fact_items);
    body_c.cycle();
    body_a.cycle();

    let sc = score_net(&body_c, &questions, &c_sym_t, &c_sym_s);
    let sa = score_net(&body_a, &questions, &a_sym_t, &a_sym_s);
    let (c_buckets, c_false) = score_detail(&body_c, &questions, &c_sym_t, &c_sym_s);
    let (a_buckets, a_false) = score_detail(&body_a, &questions, &a_sym_t, &a_sym_s);
    // v3.2 路径特异：真/假题对平均最短路（不可达惩罚 12），C vs A 同批题目
    let (c_pt, c_pf, c_tu, c_fu) = path_stats(&body_c, &questions, &c_sym_t, &c_sym_s);
    let (a_pt, a_pf, a_tu, a_fu) = path_stats(&body_a, &questions, &a_sym_t, &a_sym_s);

    // 学习曲线：同一 rng 初值重培养到第 k 阶段（前缀与 C 逐字节一致），喂同一批测试事实答题。
    // 不用阶段存档恢复：slice 只存 Fate+计数（tile/stem save_slice），恢复的身体长不出新结构，
    // 实测恒卡在 45.5 地板，故改用截断重培养。
    let mut curve = [0.0f64; 3];
    for (k, slot) in curve.iter_mut().enumerate() {
        let mut rng_k = Rng(0x5EED_0000 + seed * 7919);
        let (mut probe, _) = cultivate(&mut rng_k, &train_pool, k + 1, false);
        let (ct, cs) = feed_facts_tracked(&mut probe, &fact_items);
        probe.cycle();
        *slot = score_net(&probe, &questions, &ct, &cs);
    }

    // F/G/H/I 都从同一份完整存档恢复，互不影响
    body_c.save_full(&dir).expect("完整存档失败");
    let mut body_f = Body::load_full(&dir).expect("完整恢复失败");
    let sf = score_net(&body_f, &questions, &c_sym_t, &c_sym_s);

    let mut body_g = Body::load_full(&dir).expect("恢复失败");
    body_g.ablate_structure();
    let sg = score_net(&body_g, &questions, &c_sym_t, &c_sym_s);

    // H 零分布：同度重排 20 次取分布，看真实拓扑落在哪个分位
    let mut h_scores: Vec<f64> = Vec::new();
    for k in 0..20u64 {
        let mut body_h = Body::load_full(&dir).expect("恢复失败");
        let mut rng_h = Rng(0x45EED + seed * 131 + k * 17);
        body_h.shuffle_structure(&mut rng_h);
        h_scores.push(score_net(&body_h, &questions, &c_sym_t, &c_sym_s));
    }
    let sh = h_scores.iter().sum::<f64>() / h_scores.len() as f64;
    let sh_std = (h_scores.iter().map(|v| (v - sh).powi(2)).sum::<f64>() / h_scores.len() as f64).sqrt();
    let below = h_scores.iter().filter(|&&v| v < sc).count() as f64;
    let ties = h_scores.iter().filter(|&&v| (v - sc).abs() < 1e-9).count() as f64;
    let h_pct = (below + 0.5 * ties) / h_scores.len() as f64;

    let mut body_i = Body::load_full(&dir).expect("恢复失败");
    body_i.clear_quads();
    let si = score_net(&body_i, &questions, &c_sym_t, &c_sym_s);

    println!("seed {seed}: C {sc:.1} A(matched) {sa:.1} F {sf:.1} G(拆结构) {sg:.1} Hnull {sh:.1}±{sh_std:.1}(C分位 {:.0}%) I(清存储) {si:.1} | G增益 {:+.1} L {:+.1} | 学习曲线 {:.1}/{:.1}/{:.1} | 距离层 C {:.0}/{:.0}/{:.0} A {:.0}/{:.0}/{:.0} | 假题对 C {:.0} A {:.0}",
        h_pct * 100.0, sc - sa, sc - sg, curve[0], curve[1], curve[2],
        c_buckets[0], c_buckets[1], c_buckets[2], a_buckets[0], a_buckets[1], a_buckets[2], c_false, a_false);
    println!("    路径: 真题均路 C {c_pt:.2} A {a_pt:.2} | 假题均路 C {c_pf:.2} A {a_pf:.2} | 不可达 C {}+{} A {}+{}", c_tu, c_fu, a_tu, a_fu);
    (sc, sa, sf, sg, sh, si, sh_std, h_pct, curve, c_buckets, a_buckets, c_false, a_false, c_pt, a_pt, c_pf, a_pf, (c_tu + c_fu) as f64)
}

// ═══════════ E1W 宽度培养（VH_E1W=1 启用；判据预先声明） ═══════════
// 假说（2026-11-01 记录 ENTRY_FREEDOM_AND_E1W）：身体组织结构时
// 能看见的只有段字节统计；同质符号链不给链归属信号，拓扑特异性
// 无从显形。给链分组依附不同统计上下文（宽度），拓扑若按域组织，
// 则 C-宽应在同型跨链硬假题拒绝率与真题最短路上同时优于 C-同
// （等量同宽消融）与 H（严格双边交换重连，保入+出度）。
// 主判据不用限步可达总分（偏爱随机小世界图），用硬假题拒绝率 +
// 真题平均最短路（不可达惩罚 12）；异型跨链易假题只报告不计判据。

fn filler(ctx: usize, rng: &mut Rng, len: usize) -> Vec<u8> {
    (0..len)
        .map(|i| match ctx {
            0 => b'a' + rng.below(8) as u8,
            1 => b'0' + rng.below(10) as u8,
            2 => 180 + rng.below(70) as u8,
            _ => {
                if (i + rng.below(2)) % 2 == 0 { 30 + rng.below(10) as u8 } else { 220 + rng.below(20) as u8 }
            }
        })
        .collect()
}

/// 严格双边交换重连（补 E0b）：随机取两条边 (a→b),(c→d) 换成
/// (a→d),(c→b)，拒绝自环与重边。每节点出度、入度均严格不变。
fn shuffle_strict(body: &mut Body, rng: &mut Rng) {
    use std::collections::HashSet;
    // Tile 层
    {
        let ids: Vec<String> = body.tile.completed_tiles.iter().map(|t| t.id.clone()).collect();
        let idx: std::collections::HashMap<&str, usize> =
            ids.iter().enumerate().map(|(i, s)| (s.as_str(), i)).collect();
        let mut edges: Vec<(usize, usize)> = Vec::new();
        for (i, t) in body.tile.completed_tiles.iter().enumerate() {
            for c in &t.connections {
                if let Some(&j) = idx.get(c.as_str()) { edges.push((i, j)); }
            }
        }
        let mut set: HashSet<(usize, usize)> = edges.iter().cloned().collect();
        for _ in 0..edges.len() * 20 {
            if edges.len() < 2 { break; }
            let e1 = rng.below(edges.len());
            let e2 = rng.below(edges.len());
            if e1 == e2 { continue; }
            let (a, b) = edges[e1];
            let (c, d) = edges[e2];
            if a == d || c == b || a == c { continue; }
            if set.contains(&(a, d)) || set.contains(&(c, b)) { continue; }
            set.remove(&(a, b)); set.remove(&(c, d));
            set.insert((a, d)); set.insert((c, b));
            edges[e1] = (a, d); edges[e2] = (c, b);
        }
        for t in body.tile.completed_tiles.iter_mut() {
            let keep = t.connections.len();
            t.connections.clear();
            t.weights.resize(keep, 0.1);
        }
        for (a, d) in &edges {
            body.tile.completed_tiles[*a].connections.push(ids[*d].clone());
        }
        for t in body.tile.completed_tiles.iter_mut() {
            t.weights.resize(t.connections.len(), 0.1);
        }
    }
    // Stem 层（强度随源槽保留）
    {
        let ids: Vec<String> = body.stem.cells.iter().map(|c| c.cell_id.clone()).collect();
        let idx: std::collections::HashMap<&str, usize> =
            ids.iter().enumerate().map(|(i, s)| (s.as_str(), i)).collect();
        let mut edges: Vec<(usize, usize, f64)> = Vec::new();
        for (i, c) in body.stem.cells.iter().enumerate() {
            for (to, s) in &c.connections {
                if let Some(&j) = idx.get(to.as_str()) { edges.push((i, j, *s)); }
            }
        }
        let mut set: HashSet<(usize, usize)> = edges.iter().map(|e| (e.0, e.1)).collect();
        for _ in 0..edges.len() * 20 {
            if edges.len() < 2 { break; }
            let e1 = rng.below(edges.len());
            let e2 = rng.below(edges.len());
            if e1 == e2 { continue; }
            let (a, b, s1) = edges[e1];
            let (c, d, s2) = edges[e2];
            if a == d || c == b || a == c { continue; }
            if set.contains(&(a, d)) || set.contains(&(c, b)) { continue; }
            set.remove(&(a, b)); set.remove(&(c, d));
            set.insert((a, d)); set.insert((c, b));
            edges[e1] = (a, d, s1); edges[e2] = (c, b, s2);
        }
        for c in body.stem.cells.iter_mut() { c.connections.clear(); }
        for (a, d, s) in &edges {
            body.stem.cells[*a].connections.push((ids[*d].clone(), *s));
        }
    }
}

type WQ = (String, String, u8); // (from, to, 0=真题 1=同型假题 2=异型假题)

fn score_width(body: &Body, qs: &[WQ], sym_tiles: &SymNodes, sym_cells: &SymNodes) -> (f64, f64, f64, f64) {
    let adj_t = net_adj(body, false);
    let adj_s = net_adj(body, true);
    let empty: Vec<String> = Vec::new();
    let (mut tt, mut tn, mut hf, mut hn, mut ef, mut en, mut path) = (0usize, 0usize, 0usize, 0usize, 0usize, 0usize, 0.0f64);
    for (from, to, cat) in qs {
        let ft = sym_tiles.get(from).unwrap_or(&empty);
        let tt2 = sym_tiles.get(to).unwrap_or(&empty);
        let fs = sym_cells.get(from).unwrap_or(&empty);
        let ts = sym_cells.get(to).unwrap_or(&empty);
        let got = net_reachable(&adj_t, ft, tt2, NET_BUDGET) || net_reachable(&adj_s, fs, ts, NET_BUDGET);
        match cat {
            0 => {
                tn += 1;
                if got { tt += 1; }
                path += shortest_path(&adj_t, &adj_s, ft, tt2, fs, ts).map_or(12.0, |v| v as f64);
            }
            1 => { hn += 1; if !got { hf += 1; } }
            _ => { en += 1; if !got { ef += 1; } }
        }
    }
    (
        if tn > 0 { tt as f64 / tn as f64 * 100.0 } else { 0.0 },
        if hn > 0 { hf as f64 / hn as f64 * 100.0 } else { 0.0 },
        if en > 0 { ef as f64 / en as f64 * 100.0 } else { 0.0 },
        if tn > 0 { path / tn as f64 } else { 0.0 },
    )
}

/// 诊断计数（只读，不参与判分）：种群/边数/映射节点存活率。
/// E1W 首跑 C宽真题≈0 而硬假拒绝=100 是断网平凡解，必须先确认
/// 映射符号的出生单位在 cycle 的合并/死亡后还剩多少、图是否连通。
fn diag_qs(body: &Body, qs: &[WQ], sym_t: &SymNodes, sym_c: &SymNodes, label: &str) {
    let adj_t = net_adj(body, false);
    let adj_s = net_adj(body, true);
    let empty: Vec<String> = Vec::new();
    let mut dists: Vec<String> = Vec::new();
    let mut n_true = 0usize;
    let mut within5 = 0usize;
    for (from, to, cat) in qs {
        if *cat != 0 { continue; }
        n_true += 1;
        let d = shortest_path(&adj_t, &adj_s,
            sym_t.get(from).unwrap_or(&empty), sym_t.get(to).unwrap_or(&empty),
            sym_c.get(from).unwrap_or(&empty), sym_c.get(to).unwrap_or(&empty));
        if let Some(v) = d { if v <= 5 { within5 += 1; } }
        if dists.len() < 12 { dists.push(format!("{from}->{to}={}", d.map(|v| v as i64).unwrap_or(-1))); }
    }
    println!("    [diag {label}] 真题 {n_true} 对内 5 步可达 {within5}；样本最短路 {:?}", dists);
    // 位置/维度实况：两具身体的场维度与测试细胞位置是否可比
    {
        let dim = body.stem.space.current_dimension;
        let lens: std::collections::HashSet<usize> = body.stem.cells.iter().map(|c| c.position.len()).collect();
        let mapped_ids: std::collections::HashSet<&str> = sym_c.values().flat_map(|v| v.iter().map(|s| s.as_str())).collect();
        let mut sample: Vec<String> = Vec::new();
        for c in &body.stem.cells {
            if mapped_ids.contains(c.cell_id.as_str()) && sample.len() < 3 {
                sample.push(format!("{} pos{:?} deg{}", c.cell_id,
                    c.position.iter().take(4).map(|v| format!("{v:.2}")).collect::<Vec<_>>(), c.connections.len()));
            }
        }
        // 测试细胞的连接目标里有多少是其他测试细胞
        let test_targets = {
            let mut tot = 0usize; let mut tst = 0usize;
            for c in &body.stem.cells {
                if mapped_ids.contains(c.cell_id.as_str()) {
                    for (to, _) in &c.connections { tot += 1; if mapped_ids.contains(to.as_str()) { tst += 1; } }
                }
            }
            (tst, tot)
        };
        println!("    [diag {label}/stem] 场维度 {dim:.1} 位置长度集 {:?} 测试细胞边指向测试细胞 {}/{} 样本 {:?}", lens, test_targets.0, test_targets.1, sample);
    }
    // 无向连通分量：测试映射细胞散在几个分量、最大分量多大
    for (lname, adj, m) in [("tile", &adj_t, sym_t), ("stem", &adj_s, sym_c)] {
        let mut und: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
        for (u, ns) in adj.iter() {
            for w in ns {
                und.entry(u.clone()).or_default().push(w.clone());
                und.entry(w.clone()).or_default().push(u.clone());
            }
        }
        let mapped: std::collections::HashSet<String> = m.values().flat_map(|v| v.iter().cloned()).collect();
        let mut comp_id: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        let mut sizes: Vec<usize> = Vec::new();
        for node in und.keys() {
            if comp_id.contains_key(node) { continue; }
            let cid = sizes.len();
            let mut stack = vec![node.clone()];
            comp_id.insert(node.clone(), cid);
            let mut sz = 0usize;
            while let Some(u) = stack.pop() {
                sz += 1;
                if let Some(ns) = und.get(&u) {
                    for w in ns { if !comp_id.contains_key(w) { comp_id.insert(w.clone(), cid); stack.push(w.clone()); } }
                }
            }
            sizes.push(sz);
        }
        sizes.sort_unstable_by(|a, b| b.cmp(a));
        let mut mapped_per_comp: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
        for id in &mapped { if let Some(&c) = comp_id.get(id) { *mapped_per_comp.entry(c).or_default() += 1; } }
        let mut mpc: Vec<usize> = mapped_per_comp.values().cloned().collect();
        mpc.sort_unstable_by(|a, b| b.cmp(a));
        println!("    [diag {label}/{lname}] 分量 {} 个，最大 {:?}；映射细胞分布（前5分量）{:?}（映射共 {}）",
            sizes.len(), &sizes[..sizes.len().min(5)], &mpc[..mpc.len().min(5)], mapped.len());
    }
}

fn diag(body: &Body, sym_t: &SymNodes, sym_c: &SymNodes, label: &str) {
    let et: usize = body.tile.completed_tiles.iter().map(|t| t.connections.len()).sum();
    let es: usize = body.stem.cells.iter().map(|c| c.connections.len()).sum();
    let alive = |ids: &std::collections::HashSet<&str>, m: &SymNodes| -> (usize, usize) {
        let mut tot = 0usize;
        let mut ok = 0usize;
        for v in m.values() {
            for id in v {
                tot += 1;
                if ids.contains(id.as_str()) { ok += 1; }
            }
        }
        (ok, tot)
    };
    let tids: std::collections::HashSet<&str> = body.tile.completed_tiles.iter().map(|t| t.id.as_str()).collect();
    let cids: std::collections::HashSet<&str> = body.stem.cells.iter().map(|c| c.cell_id.as_str()).collect();
    let (at, tt2) = alive(&tids, sym_t);
    let (ac, tc) = alive(&cids, sym_c);
    println!("    [diag {label}] tiles {} (边 {}) cells {} (边 {}) | 映射存活 tile {at}/{tt2} cell {ac}/{tc}",
        body.tile.completed_tiles.len(), et, body.stem.cells.len(), es);
    // 映射细胞自身的度与无界可达（区分断网 vs 限步不够）
    for (lname, adj, m) in [("tile", net_adj(body, false), sym_t), ("stem", net_adj(body, true), sym_c)] {
        let mut ids_m: Vec<&str> = Vec::new();
        for v in m.values() { for id in v { ids_m.push(id.as_str()); } }
        let degs: Vec<usize> = ids_m.iter().map(|id| adj.get(*id).map(|v| v.len()).unwrap_or(0)).collect();
        let avg_deg = if degs.is_empty() { 0.0 } else { degs.iter().sum::<usize>() as f64 / degs.len() as f64 };
        let zero = degs.iter().filter(|&&d| d == 0).count();
        // 无界 BFS：从全部映射节点出发能到的映射节点占比
        let target: std::collections::HashSet<&str> = ids_m.iter().cloned().collect();
        let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();
        let mut q: std::collections::VecDeque<&str> = std::collections::VecDeque::new();
        for id in &ids_m { if seen.insert(id) { q.push_back(id); } }
        while let Some(u) = q.pop_front() {
            if let Some(ns) = adj.get(u) {
                for w in ns { if seen.insert(w.as_str()) { q.push_back(w.as_str()); } }
            }
        }
        let reached = target.iter().filter(|id| seen.contains(*id)).count();
        println!("    [diag {label}/{lname}] 映射节点 {} 平均度 {:.1} 零度 {} 无界可达映射 {reached}/{}",
            ids_m.len(), avg_deg, zero, target.len());
    }
}

fn width_seed(seed: u64) -> (f64, f64, f64, f64, f64, f64, f64, f64) {
    println!("== E1W seed {seed} ==");
    let train_pool: Vec<String> = (0..200).map(symbol).collect();
    let test_base = 400 + (seed as usize) * 200;
    let test_pool: Vec<String> = (test_base..test_base + 180).map(symbol).collect();

    let mut rng_c = Rng(0x5EED_0000 + seed * 7919);
    let train_shuffled = shuffled_pool(&mut rng_c, &train_pool, 192);
    let train_chains: Vec<Vec<String>> = train_shuffled.chunks(6).map(|c| c.to_vec()).collect();
    let test_shuffled = shuffled_pool(&mut rng_c, &test_pool, 96);
    let test_chains: Vec<Vec<String>> = test_shuffled.chunks(6).map(|c| c.to_vec()).collect();

    let cultivate_w = |body: &mut Body, width: bool, rng: &mut Rng| {
        for _ in 0..25 {
            for (ci, chain) in train_chains.iter().enumerate() {
                let ctx = if width { ci % 4 } else { 0 };
                for item in pairs_shuffled(chain, rng) {
                    body.feed(&item);
                    let f = filler(ctx, rng, 48);
                    body.feed(&f);
                }
            }
            body.cycle();
        }
    };
    let test_feed = |body: &mut Body, width: bool, rng: &mut Rng| -> (SymNodes, SymNodes) {
        let mut sym_tiles: SymNodes = SymNodes::new();
        let mut sym_cells: SymNodes = SymNodes::new();
        for (ci, chain) in test_chains.iter().enumerate() {
            let ctx = if width { ci % 4 } else { 0 };
            for item in pairs_shuffled(chain, rng) {
                let (tiles, cells) = body.feed_tracked(&item);
                let text = String::from_utf8_lossy(&item).into_owned();
                for sym in text.split('>') {
                    if sym.len() == 3 {
                        sym_tiles.entry(sym.to_string()).or_default().extend(tiles.iter().cloned());
                        sym_cells.entry(sym.to_string()).or_default().extend(cells.iter().cloned());
                    }
                }
                let f = filler(ctx, rng, 48);
                body.feed(&f);
            }
        }
        body.cycle();
        (sym_tiles, sym_cells)
    };

    // 题目：真题（链内 dist 2..5）+ 同型跨链硬假题 + 异型跨链易假题
    let mut qs: Vec<WQ> = Vec::new();
    for chain in &test_chains {
        for i in 0..chain.len() {
            for d in 2..chain.len() - i {
                qs.push((chain[i].clone(), chain[i + d].clone(), 0));
            }
        }
    }
    let mut rng_q = Rng(0x9E57_0000 + seed * 31);
    for (ci, chain) in test_chains.iter().enumerate() {
        for _ in 0..4 {
            let same: Vec<usize> = (0..test_chains.len()).filter(|&j| j != ci && (j % 4) == (ci % 4)).collect();
            let oj = same[rng_q.below(same.len())];
            qs.push((chain[0].clone(), test_chains[oj][3].clone(), 1));
            let diff: Vec<usize> = (0..test_chains.len()).filter(|&j| (j % 4) != (ci % 4)).collect();
            let dj = diff[rng_q.below(diff.len())];
            qs.push((chain[0].clone(), test_chains[dj][3].clone(), 2));
        }
    }

    let mut rng_w = Rng(0x5EED_0000 + seed * 7919);
    let mut body_w = Body::new();
    cultivate_w(&mut body_w, true, &mut rng_w);
    let (wt, wc) = test_feed(&mut body_w, true, &mut rng_w);
    diag(&body_w, &wt, &wc, "C宽");
    diag_qs(&body_w, &qs, &wt, &wc, "C宽");
    let (w_true, w_hard, w_easy, w_path) = score_width(&body_w, &qs, &wt, &wc);

    let mut rng_h = Rng(0x5EED_0000 + seed * 7919);
    let mut body_hom = Body::new();
    cultivate_w(&mut body_hom, false, &mut rng_h);
    let (ht, hc) = test_feed(&mut body_hom, false, &mut rng_h);
    diag(&body_hom, &ht, &hc, "C同");
    diag_qs(&body_hom, &qs, &ht, &hc, "C同");
    let (h_true, h_hard, _he, h_path) = score_width(&body_hom, &qs, &ht, &hc);

    // H：C-宽身体存档恢复后严格重连（同图、同映射，只换拓扑）
    let dir = std::env::temp_dir().join(format!("valhalla_e1w_seed{seed}"));
    body_w.save_full(&dir).expect("E1W 存档失败");
    let mut body_x = Body::load_full(&dir).expect("E1W 恢复失败");
    let mut rng_x = Rng(0x45EED + seed * 131);
    shuffle_strict(&mut body_x, &mut rng_x);
    let (x_true, x_hard, _xe, x_path) = score_width(&body_x, &qs, &wt, &wc);

    println!("seed {seed}: C宽 真 {w_true:.1} 硬假 {w_hard:.1} 易假 {w_easy:.1} 均路 {w_path:.2} | C同 真 {h_true:.1} 硬假 {h_hard:.1} 均路 {h_path:.2} | H严 真 {x_true:.1} 硬假 {x_hard:.1} 均路 {x_path:.2}");
    (w_true, w_hard, w_path, h_true, h_hard, h_path, x_true, x_hard)
}

// ═══════════ E2 原生闭环自定位（VH_E2=1；判据见 E2_E1W2_PREREG_20261102，跑前已提交） ═══════════
// 刺激只经身体自有入口（feed 的 ingest+promote 路）进入，身体
// 自跑一周期；读既有 Stem 细胞的差分位移（刺激臂 − 静默臂，
// 双臂从同一完整存档恢复、起点逐字相同）。符号→链映射只作
// 计分标注（真值集），不参与产生响应——身体无响应即测不到，
// 与图上 BFS 用映射构造答案路径根本不同。

fn e2_arm(dir: &std::path::Path, stimulus: Option<&[u8]>) -> std::collections::HashMap<String, f64> {
    let mut body = Body::load_full(dir).expect("E2 恢复失败");
    let before: std::collections::HashMap<String, Vec<f64>> = body.stem.cells.iter()
        .map(|c| (c.cell_id.clone(), c.position.clone())).collect();
    if let Some(s) = stimulus { body.feed(s); }
    body.cycle();
    let mut disp = std::collections::HashMap::new();
    for c in &body.stem.cells {
        if let Some(p0) = before.get(&c.cell_id) {
            let d: f64 = c.position.iter().zip(p0.iter())
                .map(|(a, b)| (a - b).powi(2)).sum::<f64>().sqrt();
            disp.insert(c.cell_id.clone(), d);
        }
    }
    disp
}

/// 富集度 E=(目标质量占比)/(目标细胞占比)，1=机会水平；
/// 无差分响应（总和 0）记 E=1 并标 no_resp，不奖不罚。
fn e2_enrichment(stim: &std::collections::HashMap<String, f64>,
                 silent: &std::collections::HashMap<String, f64>,
                 target: &std::collections::HashSet<String>) -> (f64, bool) {
    let (mut total, mut tmass, mut n, mut nt) = (0.0f64, 0.0f64, 0usize, 0usize);
    for (id, d) in stim {
        if let Some(s0) = silent.get(id) {
            let r = (d - s0).max(0.0);
            total += r;
            n += 1;
            if target.contains(id) { tmass += r; nt += 1; }
        }
    }
    if total <= 0.0 || n == 0 || nt == 0 { return (1.0, true); }
    ((tmass / total) / (nt as f64 / n as f64), false)
}

fn e2_seed(seed: u64) -> (f64, f64, f64, f64, usize) {
    println!("== E2 seed {seed} ==");
    let train_pool: Vec<String> = (0..200).map(symbol).collect();
    let test_base = 400 + (seed as usize) * 200;
    let test_pool: Vec<String> = (test_base..test_base + 180).map(symbol).collect();
    let mut rng = Rng(0x5EED_0000 + seed * 7919);
    let (mut body, _sc) = cultivate(&mut rng, &train_pool, 3, true);
    // 12 条完整测试链（与 run_seed 同构造）
    let mut rng_t = Rng(0x7E57_0000 + seed * 104729);
    let mut cursor = 0usize;
    let mut chains: Vec<Vec<String>> = Vec::new();
    let mut fact_items: Vec<Vec<u8>> = Vec::new();
    for _ in 0..12 {
        let mut chain: Vec<String> = test_pool[cursor..cursor + 6].to_vec();
        cursor += 6;
        for i in (1..chain.len()).rev() { let j = rng_t.below(i + 1); chain.swap(i, j); }
        fact_items.extend(pairs_shuffled(&chain, &mut rng_t));
        chains.push(chain);
    }
    let (_sym_t, sym_s) = feed_facts_tracked(&mut body, &fact_items);
    body.cycle();
    let dir = std::env::temp_dir().join(format!("valhalla_e2_seed{seed}"));
    body.save_full(&dir).expect("E2 存档失败");
    let targets: Vec<std::collections::HashSet<String>> = chains.iter().map(|chain| {
        let mut set = std::collections::HashSet::new();
        for s in chain {
            if let Some(ids) = sym_s.get(s) { set.extend(ids.iter().cloned()); }
        }
        set
    }).collect();
    let (mut e1s, mut e2s, mut ectrls) = (Vec::new(), Vec::new(), Vec::new());
    let mut noresp = 0usize;
    for (ci, chain) in chains.iter().enumerate() {
        for (a, b) in [(chain[0].as_str(), chain[3].as_str()), (chain[0].as_str(), chain[5].as_str())] {
            for (form, stim) in [(1u8, format!("{a}>{b}")), (2u8, format!("{a} > {b}"))] {
                let ds = e2_arm(&dir, Some(stim.as_bytes()));
                let dz = e2_arm(&dir, None);
                let (e, nr) = e2_enrichment(&ds, &dz, &targets[ci]);
                if nr { noresp += 1; }
                if form == 1 { e1s.push(e); } else { e2s.push(e); }
            }
        }
        // 跨链对照（只报告不计判据）：A 本链、B 邻链，真值集仍取本链
        if ci < 6 {
            let stim = format!("{}>{}", chain[0], chains[(ci + 1) % 12][3]);
            let ds = e2_arm(&dir, Some(stim.as_bytes()));
            let dz = e2_arm(&dir, None);
            let (e, _nr) = e2_enrichment(&ds, &dz, &targets[ci]);
            ectrls.push(e);
        }
    }
    let mean = |v: &Vec<f64>| if v.is_empty() { 0.0 } else { v.iter().sum::<f64>() / v.len() as f64 };
    let (m1, m2, mc) = (mean(&e1s), mean(&e2s), mean(&ectrls));
    let gt1 = e1s.iter().chain(e2s.iter()).filter(|&&e| e > 1.0).count();
    let nq = (e1s.len() + e2s.len()).max(1);
    println!("seed {seed}: E2 F1均富集 {m1:.2} F2均富集 {m2:.2} | 跨链对照 {mc:.2} | 题级E>1 {gt1}/{nq} 无响应 {noresp}");
    (m1, m2, mc, gt1 as f64 / nq as f64, noresp)
}

// ═══════════ XMOD 跨模态链回答（VH_XMOD=1；判据见 XMOD_PREREG_20261102，跑前已提交） ═══════════
// 链节点是四模态真实字节片（文/码/图/声各 40B），相邻关系多
// 为跨模态；看自形成结构能否承重多跳可达。三臂：C 链、
// A 随机配对（同令牌同条数）、G=C 拆结构。计分映射只作标注。

fn xmod_corpora() -> [Vec<u8>; 4] {
    let rd = |p: &str| std::fs::read(p).unwrap_or_default();
    let mut img = Vec::new();
    if let Ok(rdimg) = std::fs::read_dir(r"C:\Windows\Web\Wallpaper\Windows") {
        let mut ps: Vec<std::path::PathBuf> = rdimg.flatten().map(|e| e.path())
            .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("jpg")).collect();
        ps.sort();
        if let Some(p) = ps.first() { img = std::fs::read(p).unwrap_or_default(); }
    }
    [
        rd(r"C:\Ai\rogue-products\valhalla-self-incubate\corpus_diverse\zh_honglou_24264.txt"),
        rd(r"C:\Ai\rogue-products\Valhalla\manifestsys\stem-cell\src\stemcell_complete.rs"),
        img,
        rd(r"C:\Windows\Media\Alarm01.wav"),
    ]
}

fn xmod_seed(seed: u64) -> (f64, f64, f64, f64) {
    let corpora = xmod_corpora();
    let mut rng = Rng(0xC40D_0000 + seed * 104729);
    // 48 链 × 6 令牌；位置模态按 文→图→声→码 循环、相位随链移
    let mut chains: Vec<Vec<usize>> = Vec::new();
    let mut token_patch: Vec<Vec<u8>> = Vec::new();
    for c in 0..48usize {
        let mut chain = Vec::new();
        for k in 0..6usize {
            let m = (k + c) % 4;
            let corp = &corpora[m];
            let off = if corp.len() > 64 { rng.below(corp.len() - 64) } else { 0 };
            let patch: Vec<u8> = corp.iter().skip(off).take(40).cloned().collect();
            token_patch.push(patch);
            chain.push(token_patch.len() - 1);
        }
        chains.push(chain);
    }
    let item_of = |a: usize, b: usize, token_patch: &Vec<Vec<u8>>| -> Vec<u8> {
        let mut v = token_patch[a].clone();
        v.push(b'>');
        v.extend_from_slice(&token_patch[b]);
        v
    };
    let c_pairs: Vec<(usize, usize)> = chains.iter()
        .flat_map(|ch| ch.windows(2).map(|w| (w[0], w[1])).collect::<Vec<_>>()).collect();
    let build = |pairs: &Vec<(usize, usize)>, rng: &mut Rng,
                 token_patch: &Vec<Vec<u8>>| -> (Body, SymNodes, SymNodes) {
        let mut body = Body::new();
        let mut mt: SymNodes = SymNodes::new();
        let mut ms: SymNodes = SymNodes::new();
        for _ in 0..6 {
            let mut order: Vec<usize> = (0..pairs.len()).collect();
            for i in (1..order.len()).rev() { let j = rng.below(i + 1); order.swap(i, j); }
            for &pi in &order {
                let (a, b) = pairs[pi];
                let (tiles, cells) = body.feed_tracked(&item_of(a, b, token_patch));
                for t in [a, b] {
                    mt.entry(format!("t{t}")).or_default().extend(tiles.iter().cloned());
                    ms.entry(format!("t{t}")).or_default().extend(cells.iter().cloned());
                }
            }
            body.cycle();
            body.cycle();
        }
        (body, mt, ms)
    };
    let (body_c, ct, cs) = build(&c_pairs, &mut rng, &token_patch);
    // A：同令牌随机配对、同条数
    let ntok = token_patch.len();
    let mut a_pairs: Vec<(usize, usize)> = Vec::new();
    while a_pairs.len() < c_pairs.len() {
        let a = rng.below(ntok);
        let b = rng.below(ntok);
        if a != b { a_pairs.push((a, b)); }
    }
    let (body_a, at, as_) = build(&a_pairs, &mut rng, &token_patch);
    // G：C 存档恢复后拆结构
    let dir = std::env::temp_dir().join(format!("valhalla_xmod_seed{seed}"));
    body_c.save_full(&dir).expect("XMOD 存档失败");
    let mut body_g = Body::load_full(&dir).expect("XMOD 恢复失败");
    body_g.ablate_structure();
    // 题目：链内 dist 2..5 真题 + 跨链假题
    let score = |body: &Body, mt: &SymNodes, ms: &SymNodes| -> (f64, f64) {
        let adj_t = net_adj(body, false);
        let adj_s = net_adj(body, true);
        let empty: Vec<String> = Vec::new();
        let (mut tt, mut tn, mut fw, mut fn_) = (0usize, 0usize, 0usize, 0usize);
        for ch in &chains {
            for i in 0..6 {
                for d in 2..(6 - i) {
                    tn += 1;
                    let (a, b) = (format!("t{}", ch[i]), format!("t{}", ch[i + d]));
                    if net_reachable(&adj_t, mt.get(&a).unwrap_or(&empty), mt.get(&b).unwrap_or(&empty), NET_BUDGET)
                        || net_reachable(&adj_s, ms.get(&a).unwrap_or(&empty), ms.get(&b).unwrap_or(&empty), NET_BUDGET) {
                        tt += 1;
                    }
                }
            }
        }
        for (ci, ch) in chains.iter().enumerate() {
            fn_ += 1;
            let (a, b) = (format!("t{}", ch[0]), format!("t{}", chains[(ci + 7) % 48][3]));
            if net_reachable(&adj_t, mt.get(&a).unwrap_or(&empty), mt.get(&b).unwrap_or(&empty), NET_BUDGET)
                || net_reachable(&adj_s, ms.get(&a).unwrap_or(&empty), ms.get(&b).unwrap_or(&empty), NET_BUDGET) {
                fw += 1;
            }
        }
        (tt as f64 / tn.max(1) as f64 * 100.0, fw as f64 / fn_.max(1) as f64 * 100.0)
    };
    let (c_true, c_fake) = score(&body_c, &ct, &cs);
    let (a_true, _af) = score(&body_a, &at, &as_);
    let (g_true, _gf) = score(&body_g, &ct, &cs);
    println!("seed {seed}: XMOD C真 {c_true:.1} A真 {a_true:.1} G真 {g_true:.1} | C假误答 {c_fake:.1}");
    (c_true, a_true, g_true, c_fake)
}

// ═══════════ 官方基准多选投影（VH_BABI / VH_CLUTRR；预注册 OFFICIAL_BENCH_PREREG_20261102） ═══════════
// 通道只出排序信号（身体图最短路），真值用官方标注；一切
// 报告称"官方数据的多选投影"，不称官方分数。

fn fnv_hex(bytes: &[u8]) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bytes { h ^= *b as u64; h = h.wrapping_mul(0x100000001b3); }
    format!("{h:016x}")
}

fn words_of(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|w| w.trim_matches(|c: char| ".,?!;:\"'()[]".contains(c)).to_string())
        .filter(|w| !w.is_empty())
        .collect()
}

/// 故事逐句喂入（句为条目）+ 2 周期；词→出生单位映射只作标注。
fn feed_story(body: &mut Body, sentences: &[String]) -> (SymNodes, SymNodes) {
    let mut mt: SymNodes = SymNodes::new();
    let mut ms: SymNodes = SymNodes::new();
    for s in sentences {
        let (tiles, cells) = body.feed_tracked(s.as_bytes());
        for w in words_of(s) {
            mt.entry(w.clone()).or_default().extend(tiles.iter().cloned());
            ms.entry(w).or_default().extend(cells.iter().cloned());
        }
    }
    body.cycle();
    body.cycle();
    (mt, ms)
}

fn pick_by_path(body: &Body, mt: &SymNodes, ms: &SymNodes, entity: &str, cands: &[String]) -> Option<String> {
    let adj_t = net_adj(body, false);
    let adj_s = net_adj(body, true);
    let empty: Vec<String> = Vec::new();
    let ft = mt.get(entity).unwrap_or(&empty);
    let fs = ms.get(entity).unwrap_or(&empty);
    if ft.is_empty() && fs.is_empty() { return None; }
    let mut best: Option<(usize, &String)> = None;
    for c in cands {
        let d = shortest_path(&adj_t, &adj_s,
            ft, mt.get(c).unwrap_or(&empty),
            fs, ms.get(c).unwrap_or(&empty)).unwrap_or(999);
        if best.map(|(bd, _)| d < bd).unwrap_or(true) { best = Some((d, c)); }
    }
    best.map(|(_, c)| c.clone())
}

struct BabiQ { context: Vec<String>, question: String, answer: String }

fn parse_babi(path: &str, limit: usize) -> Vec<BabiQ> {
    let text = std::fs::read_to_string(path).unwrap_or_default();
    let mut out: Vec<BabiQ> = Vec::new();
    let mut ctx: Vec<String> = Vec::new();
    for line in text.lines() {
        let (id, rest) = match line.split_once(' ') {
            Some((i, r)) => (i.parse::<usize>().unwrap_or(0), r),
            None => continue,
        };
        if id == 1 { ctx.clear(); }
        if let Some((q, a)) = rest.split_once('\t') {
            let ans = a.split('\t').next().unwrap_or("").trim().to_string();
            out.push(BabiQ { context: ctx.clone(), question: q.trim().to_string(), answer: ans });
            if out.len() >= limit { break; }
        } else {
            ctx.push(rest.trim().to_string());
        }
    }
    out
}

fn babi_answers(path: &str) -> Vec<String> {
    let text = std::fs::read_to_string(path).unwrap_or_default();
    let mut set: Vec<String> = Vec::new();
    for line in text.lines() {
        if let Some((_, rest)) = line.split_once(' ') {
            if let Some((_, a)) = rest.split_once('\t') {
                let ans = a.split('\t').next().unwrap_or("").trim().to_string();
                if !ans.is_empty() && !set.contains(&ans) { set.push(ans); }
            }
        }
    }
    set.sort();
    set
}

fn babi_run() {
    let base = r"C:\Ai\official_bench\babi\tasks_1-20_v1-2\en";
    let tasks = [
        ("qa1", "qa1_single-supporting-fact"),
        ("qa2", "qa2_two-supporting-facts"),
        ("qa3", "qa3_three-supporting-facts"),
    ];
    for (tag, stem) in tasks {
        let test_path = format!(r"{base}\{stem}_test.txt");
        let train_path = format!(r"{base}\{stem}_train.txt");
        let raw = std::fs::read(&test_path).unwrap_or_default();
        let cands = babi_answers(&train_path);
        let qs = parse_babi(&test_path, 200);
        let chance = if cands.is_empty() { 0.0 } else { 100.0 / cands.len() as f64 };
        let (mut c_ok, mut g_ok, mut n) = (0usize, 0usize, 0usize);
        for (qi, q) in qs.iter().enumerate() {
            let mut body = Body::new();
            let (mt, ms) = feed_story(&mut body, &q.context);
            let entity = words_of(&q.question).last().cloned().unwrap_or_default();
            let pick_c = pick_by_path(&body, &mt, &ms, &entity, &cands);
            let mut rng_g = Rng(0xBEE5 + qi as u64 * 17);
            body.shuffle_structure(&mut rng_g);
            let pick_g = pick_by_path(&body, &mt, &ms, &entity, &cands);
            n += 1;
            if pick_c.as_deref() == Some(q.answer.as_str()) { c_ok += 1; }
            if pick_g.as_deref() == Some(q.answer.as_str()) { g_ok += 1; }
        }
        let nf = n.max(1) as f64;
        println!("[bAbI {tag}] n={n} 候选 {} chance {:.1}% | C {:.1}% G(重排) {:.1}% | 文件 fnv {}",
            cands.len(), chance, c_ok as f64 / nf * 100.0, g_ok as f64 / nf * 100.0, fnv_hex(&raw));
        println!("    判读（预先声明）：C > chance 且 C > G 才记该任务投影通过。");
    }
}

fn clutrr_run() {
    for (tag, path) in [
        ("k2", r"C:\Ai\official_bench\clutrr_k2.tsv"),
        ("k3", r"C:\Ai\official_bench\clutrr_k3.tsv"),
    ] {
        let text = std::fs::read_to_string(path).unwrap_or_default();
        let (mut n, mut c_reach, mut g_reach, mut c_err, mut g_err) = (0usize, 0usize, 0usize, 0.0f64, 0.0f64);
        for line in text.lines() {
            let f: Vec<&str> = line.split('\t').collect();
            if f.len() < 4 { continue; }
            let sents: Vec<String> = f[0].split(". ").map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
            let (na, nb) = (f[1].to_string(), f[2].to_string());
            let true_len: f64 = f[3].parse().unwrap_or(0.0);
            let mut body = Body::new();
            let (mt, ms) = feed_story(&mut body, &sents);
            let measure = |body: &Body| -> (bool, f64) {
                let adj_t = net_adj(body, false);
                let adj_s = net_adj(body, true);
                let empty: Vec<String> = Vec::new();
                match shortest_path(&adj_t, &adj_s,
                    mt.get(&na).unwrap_or(&empty), mt.get(&nb).unwrap_or(&empty),
                    ms.get(&na).unwrap_or(&empty), ms.get(&nb).unwrap_or(&empty)) {
                    // 不可达惩罚 12 与路径特异 v3.2 同规
                    Some(d) => (d <= NET_BUDGET, (d as f64 - true_len).abs()),
                    None => (false, (12.0 - true_len).abs()),
                }
            };
            let (cr, ce) = measure(&body);
            let mut rng_g = Rng(0xC107 + n as u64 * 31);
            body.shuffle_structure(&mut rng_g);
            let (gr, ge) = measure(&body);
            n += 1;
            if cr { c_reach += 1; }
            if gr { g_reach += 1; }
            c_err += ce;
            g_err += ge;
        }
        let nf = n.max(1) as f64;
        println!("[CLUTRR {tag}] n={n} | C reach@5 {:.1}% 路长误差 {:.2} | G reach@5 {:.1}% 路长误差 {:.2}",
            c_reach as f64 / nf * 100.0, c_err / nf, g_reach as f64 / nf * 100.0, g_err / nf);
        println!("    判读（预先声明）：C 路长误差 < G 且 C reach@5 > G 才记距离投影成立；关系标签分类未测。");
    }
}

fn main() {
    let seeds: Vec<u64> = std::env::var("VALHALLA_PROOF_SEEDS")
        .unwrap_or_else(|_| "1,2,3,4,5".to_string())
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    if std::env::var("VH_E1W").is_ok() {
        let mut wrows: Vec<(f64, f64, f64, f64, f64, f64, f64, f64)> = Vec::new();
        for seed in &seeds {
            wrows.push(width_seed(*seed));
        }
        let n = wrows.len() as f64;
        let m = |f: &dyn Fn(&(f64, f64, f64, f64, f64, f64, f64, f64)) -> f64| wrows.iter().map(f).sum::<f64>() / n;
        println!("\n== E1W 汇总（{} seeds） ==", wrows.len());
        println!("C宽 真 {:.1} 硬假 {:.1} 均路 {:.2} | C同 真 {:.1} 硬假 {:.1} 均路 {:.2} | H严 真 {:.1} 硬假 {:.1}",
            m(&|r| r.0), m(&|r| r.1), m(&|r| r.2), m(&|r| r.3), m(&|r| r.4), m(&|r| r.5), m(&|r| r.6), m(&|r| r.7));
        println!("判读（预先声明）：C宽 硬假 > C同 且 C宽 硬假 > H严 且 C宽 均路 < H严 → 宽度使拓扑特异性成立；任一不成立按未检出记。");
        return;
    }
    if std::env::var("VH_E1W2").is_ok() {
        // E1W2：新种子（16–30，与 E1W 不重叠）+ Δ 判别尺；判据见
        // docs/valhalla/E2_E1W2_PREREG_20261102.md（跑前已提交）。
        // Δ = 真题可达% − 硬假可达%（预算 5），硬假可达 = 100 − 拒绝。
        let mut rows: Vec<(f64, f64, f64)> = Vec::new(); // (Δ宽, Δ同, ΔH严)
        for seed in &seeds {
            let r = width_seed(*seed);
            rows.push((r.0 + r.1 - 100.0, r.3 + r.4 - 100.0, r.6 + r.7 - 100.0));
            let l = rows.last().unwrap();
            println!("seed {seed}: Δ宽 {:.1} Δ同 {:.1} ΔH严 {:.1}", l.0, l.1, l.2);
        }
        let n = rows.len() as f64;
        let win_a = rows.iter().filter(|r| r.0 > r.1).count();
        let win_b = rows.iter().filter(|r| r.0 > r.2).count();
        let mdiff = rows.iter().map(|r| r.0 - r.1).sum::<f64>() / n;
        let mm = |f: &dyn Fn(&(f64, f64, f64)) -> f64| rows.iter().map(f).sum::<f64>() / n;
        println!("\n== E1W2 汇总（{} seeds，Δ=真可达−硬假可达 @预算5） ==", rows.len());
        println!("Δ宽 {:.1} Δ同 {:.1} ΔH严 {:.1} | 配对 宽>同 {win_a}/{}（均差 {:+.1}） 宽>H严 {win_b}/{}",
            mm(&|r| r.0), mm(&|r| r.1), mm(&|r| r.2), rows.len(), mdiff, rows.len());
        println!("判读（预先声明）：(a) 宽>同 ≥12/15 且均差 >0 与 (b) 宽>H严 ≥12/15 同过 → 宽度使拓扑判别成立；只过 (a) 记宽度效应存在但未胜零模型；(a) 不过记未检出。");
        return;
    }
    if std::env::var("VH_E2").is_ok() {
        // E2 原生闭环自定位；判据见预注册文档（跑前已提交）。
        let mut rows: Vec<(f64, f64, f64, f64, usize)> = Vec::new();
        for seed in &seeds {
            rows.push(e2_seed(*seed));
        }
        let n = rows.len() as f64;
        let both = rows.iter().filter(|r| r.0 > 1.0 && r.1 > 1.0).count();
        let g1 = rows.iter().map(|r| r.0).sum::<f64>() / n;
        let g2 = rows.iter().map(|r| r.1).sum::<f64>() / n;
        let gc = rows.iter().map(|r| r.2).sum::<f64>() / n;
        println!("\n== E2 汇总（{} seeds） ==", rows.len());
        println!("F1 总均值 {g1:.2} F2 总均值 {g2:.2} 跨链对照总均值 {gc:.2} | 两问法同过（均值>1）种子 {both}/{}", rows.len());
        println!("判读（预先声明）：≥12/15 种子两问法均值同 >1 且两问法总均值都 ≥1.3 → E2 自定位成立；任一问法不过即该问法未检出，整体不过记 E2 未检出。");
        return;
    }
    if std::env::var("VH_XMOD").is_ok() {
        // XMOD 跨模态链回答；判据见 XMOD_PREREG_20261102（跑前已提交）。
        let mut rows: Vec<(f64, f64, f64, f64)> = Vec::new();
        for seed in &seeds {
            rows.push(xmod_seed(*seed));
        }
        let n = rows.len() as f64;
        let wins = rows.iter().filter(|r| r.0 > r.1 && r.0 > r.2).count();
        let mm = |f: &dyn Fn(&(f64, f64, f64, f64)) -> f64| rows.iter().map(f).sum::<f64>() / n;
        println!("\n== XMOD 汇总（{} seeds，真题可达%） ==", rows.len());
        println!("C {:.1} A {:.1} G {:.1} | C假误答 {:.1} | C>A 且 C>G 种子 {wins}/{}", mm(&|r| r.0), mm(&|r| r.1), mm(&|r| r.2), mm(&|r| r.3), rows.len());
        println!("判读（预先声明）：≥12/15 种子 C 同时 >A 且 >G → 跨模态链结构承重成立；不过记未检出，注明倒在哪臂。");
        return;
    }
    if std::env::var("VH_BABI").is_ok() {
        babi_run();
        return;
    }
    if std::env::var("VH_CLUTRR").is_ok() {
        clutrr_run();
        return;
    }
    type Row = (f64, f64, f64, f64, f64, f64, f64, f64, [f64; 3], [f64; 3], [f64; 3], f64, f64, f64, f64, f64, f64, f64);
    let mut rows: Vec<Row> = Vec::new();
    for seed in &seeds {
        rows.push(run_seed(*seed));
    }
    let n = rows.len() as f64;
    let mean = |f: &dyn Fn(&Row) -> f64| rows.iter().map(f).sum::<f64>() / n;
    let std = |f: &dyn Fn(&Row) -> f64| {
        let m = mean(f);
        (rows.iter().map(|r| (f(r) - m).powi(2)).sum::<f64>() / n).sqrt()
    };
    let mc = mean(&|r| r.0);
    let ma = mean(&|r| r.1);
    let mf = mean(&|r| r.2);
    let mg = mean(&|r| r.3);
    let mh = mean(&|r| r.4);
    let mi = mean(&|r| r.5);
    let mh_std_in = mean(&|r| r.6);
    let mpct = mean(&|r| r.7);
    let gain = mean(&|r| r.0 - r.1);
    let gain_std = std(&|r| r.0 - r.1);
    let loss = mean(&|r| r.0 - r.3);
    let loss_std = std(&|r| r.0 - r.3);
    println!("\n== 因果证明汇总（{} seeds，网络通道，%） ==", rows.len());
    println!("C {mc:.1}±{:.1}  A(matched) {ma:.1}±{:.1}  F {mf:.1}±{:.1}  G拆结构 {mg:.1}±{:.1}  Hnull {mh:.1}±{:.1}（种子内重排 std 均值 {mh_std_in:.1}，C 分位均值 {:.0}%）  I清存储 {mi:.1}±{:.1}",
        mpct * 100.0,
        std(&|r| r.0), std(&|r| r.1), std(&|r| r.2), std(&|r| r.3), std(&|r| r.4), std(&|r| r.5));
    println!("培养增益 G = C−A = {gain:+.1} ± {gain_std:.1} pp（A 为 matched 对照：同料同量、无链结构）");
    println!("结构因果 L = C−G = {loss:+.1} ± {loss_std:.1} pp");
    println!("学习曲线（培养阶段1/2/3 截断重培养 + 同批测试事实）: {:.1}±{:.1} → {:.1}±{:.1} → {:.1}±{:.1}",
        mean(&|r| r.8[0]), std(&|r| r.8[0]),
        mean(&|r| r.8[1]), std(&|r| r.8[1]),
        mean(&|r| r.8[2]), std(&|r| r.8[2]));
    println!("距离分层真题（d1 / d2-3 / d4-5）: C {:.1}/{:.1}/{:.1} | A(matched) {:.1}/{:.1}/{:.1}；假题正确率 C {:.1} | A {:.1}",
        mean(&|r| r.9[0]), mean(&|r| r.9[1]), mean(&|r| r.9[2]),
        mean(&|r| r.10[0]), mean(&|r| r.10[1]), mean(&|r| r.10[2]),
        mean(&|r| r.11), mean(&|r| r.12));
    let dpath = mean(&|r| r.14 - r.13);
    let dpath_std = std(&|r| r.14 - r.13);
    println!("路径特异 v3.2（均路，跳；不可达惩罚 12）：真题 C {:.2} A {:.2}（ΔA−C {:+.2}±{:.2}，正=C 更近）| 假题 C {:.2} A {:.2} | C 不可达题均 {:.1}",
        mean(&|r| r.13), mean(&|r| r.14), dpath, dpath_std,
        mean(&|r| r.15), mean(&|r| r.16), mean(&|r| r.17));
    println!("判读：F 应与 C 一致（完整状态可恢复）；G 掉分说明连接存在承重；Hnull 分位 >95% 才算拓扑特异性成立；I 不掉说明网络通道不靠存储字节；定向增益若集中在 d4-5 且假题正确率不掉，才算组合规模下成立。路径：Δ>0 且显著（|Δ|>>std）才算链式培养让新事实吸收更近。");
}

