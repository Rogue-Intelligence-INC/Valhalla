//! 定向培养：关系链传递（A>B>C ⇒ A>C）
//!
//! 只为一件事取证：能力在不在四元存储与长出的结构里。
//! C 组只见关系陈述、从没见过问题形式；B 组直接存问答对（查表）；
//! A 组喂等量无关字节。三组在测试时拿到同一批新符号的事实（只给单环，不给传递对），
//! 判读器是哑的：只在四元存储里做图可达，查不到就沉默。
//! 教师关闭 = 存盘重启后只加载四元存储再测；消融 = 清存储后应全沉默。
//! 本程序不声称 Tile/Stem 长出的结构在答题——结构是否参与，看监控差与消融结果如实报。

use hub_f64::monitor::BodyMonitor;
use hub_f64::{BaseForceCoordinator, GravityField, MeshBrainSystem, MultinovaProcessor, TripleFateCombined};
use stem_cell::StemCellComplete;
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

/// 从池里取 n 个不重叠符号并打乱（每条测试链独占符号）
fn take_symbols(pool: &[String], cur: &mut usize, n: usize, rng: &mut Rng) -> Vec<String> {
    let mut v: Vec<String> = pool[*cur..*cur + n].to_vec();
    *cur += n;
    for i in (1..v.len()).rev() {
        let j = rng.below(i + 1);
        v.swap(i, j);
    }
    v
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

struct Body {
    tile: TileQuattroComplete,
    stem: StemCellComplete,
    monitor: BodyMonitor,
    /// 全局 Fate：统一度量（效率/收敛/方程）在这里走字，两体跟随它
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
        // Stem 换喂法：走它自己的出生通道——字节流入容器，promote_from_fate 收口成一个细胞。
        // 细胞的位置/签名/场强由它自己按这条事实的字节统计决定，喂法不替它算。
        // 统一度量：每个字节先让全局 Fate 走字（receive_passive），两体随后跟随全局。
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
    /// 喂一条并返回这条新长出的 tile id 与 cell id（身体自己的出生记录）。
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
        let p = self.tile.propagate(1.0); self.monitor.observe_propagate(p.new_connections);
        self.monitor.observe_cycle(&t, &s);
    }
    /// 迭代加强量：存储总量 / Stem 连接平均强度与最大强度（看每轮是不是在上一轮上加强）
    fn iter_line(&self) -> String {
        let stored: usize = [
            &self.tile.blackhole.stored, &self.tile.meshbrain.stored,
            &self.tile.multinova.stored, &self.tile.baseforce.stored,
            &self.stem.blackhole.stored, &self.stem.meshbrain.stored,
            &self.stem.multinova.stored, &self.stem.baseforce.stored,
        ].iter().map(|s| s.len()).sum();
        let mut n = 0usize;
        let mut sum = 0.0f64;
        let mut max_s = 0.0f64;
        for c in &self.stem.cells {
            for (_, s) in &c.connections {
                n += 1;
                sum += s;
                if *s > max_s { max_s = *s; }
            }
        }
        let avg = if n > 0 { sum / n as f64 } else { 0.0 };
        format!("迭代量 存储 {stored} 条 stem连接 {n} 平均强度 {avg:.3} 最大 {max_s:.2}")
    }
    /// 统一度量状态：全局 Fate 与两体各自 Fate 的效率/信号/发现
    fn metric_line(&self) -> String {
        let g = &self.global.base.state;
        let t = &self.tile.triple_fate.base.state;
        let s = &self.stem.triple_fate.base.state;
        format!(
            "统一度量 global[eff {:.4} sig {} disc {}] tile[eff {:.4}] stem[eff {:.4}] 方程 {}",
            g.efficiency, g.signal_count, g.discovery_count,
            t.efficiency, s.efficiency,
            self.global.cluster_equation.equation_str()
        )
    }
    fn stored_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        for stored in [
            &self.tile.blackhole.stored, &self.tile.meshbrain.stored,
            &self.tile.multinova.stored, &self.tile.baseforce.stored,
            &self.stem.blackhole.stored, &self.stem.meshbrain.stored,
            &self.stem.multinova.stored, &self.stem.baseforce.stored,
        ] {
            for item in stored {
                out.extend_from_slice(item);
                out.push(b'\n');
            }
        }
        out
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
}

/// 从存储字节里解析 X>Y / X<Y 关系（符号为两个大写字母）。
fn parse_relations(bytes: &[u8]) -> Vec<(String, String)> {
    let mut edges = Vec::new();
    let sym = |b: &[u8], i: usize| -> Option<String> {
        if i + 1 < b.len() && b[i].is_ascii_uppercase() && b[i + 1].is_ascii_uppercase() {
            Some(String::from_utf8_lossy(&b[i..i + 2]).into_owned())
        } else {
            None
        }
    };
    let mut i = 0usize;
    while i + 4 <= bytes.len() {
        if let (Some(a), Some(b)) = (sym(bytes, i), sym(bytes, i + 3)) {
            match bytes[i + 2] {
                b'>' => { edges.push((a, b)); i += 2; continue; }
                b'<' => { edges.push((b, a)); i += 2; continue; }
                _ => {}
            }
        }
        i += 1;
    }
    edges
}

fn reachable(edges: &[(String, String)], from: &str, to: &str) -> bool {
    if from == to {
        return false;
    }
    let mut seen: Vec<String> = vec![from.to_string()];
    let mut stack: Vec<String> = vec![from.to_string()];
    while let Some(cur) = stack.pop() {
        for (a, b) in edges {
            if a == &cur && !seen.contains(b) {
                if b == to {
                    return true;
                }
                seen.push(b.clone());
                stack.push(b.clone());
            }
        }
    }
    false
}

/// 哑判读器（图可达）：只读四元存储，查不到 = 沉默。
fn answer_graph(body: &Body, from: &str, to: &str) -> bool {
    let edges = parse_relations(&body.stored_bytes());
    reachable(&edges, from, to)
}

/// 哑判读器（查表）：存储里有 X>Y=YES 才答。
fn answer_lookup(body: &Body, from: &str, to: &str) -> bool {
    let needle = format!("{from}>{to}=YES").into_bytes();
    body.stored_bytes().windows(needle.len()).any(|w| w == needle.as_slice())
}

/// Tile/Stem 自发网络的有向邻接（Stem 只算强度 > 0.01 的连接，与监控口径一致）。
fn net_adj(body: &Body, stem: bool) -> std::collections::HashMap<String, Vec<String>> {
    let mut adj: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    if stem {
        // 场已经按密度/距离/场强选过连接，这里原样用，不再加人为门槛
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

fn net_reachable(adj: &std::collections::HashMap<String, Vec<String>>, from: &[String], to: &[String]) -> bool {
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

fn score_net(
    body: &Body,
    qs: &[Question],
    sym_nodes: &std::collections::HashMap<String, Vec<String>>,
    stem: bool,
) -> Score {
    let adj = net_adj(body, stem);
    let mut kinds: Vec<(&'static str, usize, usize)> = Vec::new();
    let mut ok = 0usize;
    for q in qs {
        let empty: Vec<String> = Vec::new();
        let from = sym_nodes.get(&q.from).unwrap_or(&empty);
        let to = sym_nodes.get(&q.to).unwrap_or(&empty);
        let got = net_reachable(&adj, from, to);
        let right = got == q.expect_reach;
        if right {
            ok += 1;
        }
        match kinds.iter_mut().find(|(k, _, _)| *k == q.kind) {
            Some((_, c, t)) => { *c += right as usize; *t += 1; }
            None => kinds.push((q.kind, right as usize, 1)),
        }
    }
    (ok, qs.len(), kinds)
}

/// 一个事实碎片：生出它的 tile/cell + 在网络扩散中的信号强度。
struct Fragment {
    head: String,
    tail: String,
    tiles: Vec<String>,
    cells: Vec<String>,
    tile_strength: f64,
    stem_strength: f64,
    combined: f64,
}

struct Thought {
    chain: Vec<String>,
    strength: f64,
    cycles: usize,
    strongest: String,
}

/// 全局 Fate 监控下的思考：刺激源符号 → 激活在自发网络里扩散 →
/// 每轮把最强碎片报给全局 Fate 并回喂两体 → 按强度把碎片拼成一条链。
/// 拼不到目标 = 沉默。返回思考链与最弱一环的强度。
fn think(
    body: &mut Body,
    fragments: &mut [Fragment],
    sym_tiles: &std::collections::HashMap<String, Vec<String>>,
    from: &str,
    to: &str,
) -> Option<Thought> {
    let source: Vec<String> = sym_tiles.get(from).cloned().unwrap_or_default();
    if source.is_empty() {
        return None;
    }
    // Stem 碎片强度：该事实所生细胞在场里被选出的连接强度（场按密度/距离/场强自己选，
    // 这里只读它写出的强度，不加门槛）
    for f in fragments.iter_mut() {
        f.stem_strength = f.cells.iter()
            .filter_map(|id| body.stem.cells.iter().find(|c| &c.cell_id == id))
            .flat_map(|c| c.connections.iter().map(|(_, s)| *s))
            .fold(0.0f64, f64::max);
    }
    let mut strongest_label = String::new();
    let mut cycles = 0usize;
    for cycle in 1..=4 {
        cycles = cycle;
        for t in body.tile.completed_tiles.iter_mut() {
            t.activation = 0.0;
        }
        for pass in 0..5 {
            for t in body.tile.completed_tiles.iter_mut() {
                if source.contains(&t.id) {
                    t.activation = 1.0;
                }
            }
            if pass > 0 {
                body.tile.propagate(0.0);
            }
        }
        for f in fragments.iter_mut() {
            f.tile_strength = f.tiles.iter()
                .filter_map(|id| body.tile.completed_tiles.iter().find(|t| &t.id == id))
                .map(|t| t.activation.abs())
                .fold(0.0f64, f64::max);
        }
        // Fate 自动选择：两家信号各自按本轮最强归一（相对密度），碎片取更强的一家，不设硬阈值
        let max_tile = fragments.iter().map(|f| f.tile_strength).fold(0.0f64, f64::max).max(1e-12);
        let max_stem = fragments.iter().map(|f| f.stem_strength).fold(0.0f64, f64::max).max(1e-12);
        for f in fragments.iter_mut() {
            f.combined = (f.tile_strength / max_tile).max(f.stem_strength / max_stem);
        }
        let strongest = fragments.iter()
            .max_by(|a, b| a.combined.partial_cmp(&b.combined).unwrap_or(std::cmp::Ordering::Equal));
        let label = strongest.map(|f| {
            let via = if f.tile_strength / max_tile >= f.stem_strength / max_stem { "tile" } else { "stem" };
            format!("{}>{}[{}]", f.head, f.tail, via)
        }).unwrap_or_default();
        let s = strongest.map(|f| f.combined).unwrap_or(0.0);
        body.global.store_string(&format!("thought_fragment {label} s={s:.4}"));
        body.tile.follow_global_fate(&body.global);
        body.stem.follow_global_fate(&body.global);
        if label == strongest_label {
            break;
        }
        strongest_label = label;
    }
    // 按强度拼链：每步选头部接得上、信号最强的碎片
    let mut chain: Vec<String> = Vec::new();
    let mut cur = from.to_string();
    let mut weakest = f64::INFINITY;
    for _ in 0..8 {
        if cur == to {
            return Some(Thought {
                strength: if weakest.is_finite() { weakest } else { 0.0 },
                chain,
                cycles,
                strongest: strongest_label,
            });
        }
        let next = fragments.iter()
            .filter(|f| f.head == cur && f.combined > 0.0)
            .max_by(|a, b| a.combined.partial_cmp(&b.combined).unwrap_or(std::cmp::Ordering::Equal));
        match next {
            Some(f) => {
                weakest = weakest.min(f.combined);
                chain.push(format!("{}>{}({:.2})", f.head, f.tail, f.combined));
                cur = f.tail.clone();
            }
            None => return None,
        }
    }
    if cur == to {
        Some(Thought { strength: if weakest.is_finite() { weakest } else { 0.0 }, chain, cycles, strongest: strongest_label })
    } else {
        None
    }
}

struct Question {
    from: String,
    to: String,
    expect_reach: bool,
    kind: &'static str,
}

type Score = (usize, usize, Vec<(&'static str, usize, usize)>);

fn score(body: &Body, qs: &[Question], lookup: bool) -> Score {
    let mut kinds: Vec<(&'static str, usize, usize)> = Vec::new();
    let mut ok = 0usize;
    for q in qs {
        let got = if lookup { answer_lookup(body, &q.from, &q.to) } else { answer_graph(body, &q.from, &q.to) };
        let right = got == q.expect_reach;
        if right {
            ok += 1;
        }
        match kinds.iter_mut().find(|(k, _, _)| *k == q.kind) {
            Some((_, c, t)) => { *c += right as usize; *t += 1; }
            None => kinds.push((q.kind, right as usize, 1)),
        }
    }
    (ok, qs.len(), kinds)
}

fn print_score(name: &str, qs: &[Question], r: &Score) {
    let reach_total = qs.iter().filter(|q| q.expect_reach).count();
    let reach_ok: usize = r.2.iter().filter(|(k, _, _)| !k.contains("沉默")).map(|(_, c, _)| c).sum();
    let silence_total = qs.len() - reach_total;
    let silence_ok = r.0 - reach_ok;
    println!("{name}: 应答 {reach_ok}/{reach_total} 应沉默 {silence_ok}/{silence_total}（总 {}/{}）", r.0, r.1);
    for (k, c, t) in &r.2 {
        println!("    {k}: {c}/{t}");
    }
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

fn main() {
    let train_pool: Vec<String> = (0..30).map(symbol).collect();
    // 测试符号与训练不重叠；且每条测试链独占符号，避免跨链串成假通路
    let test_pool: Vec<String> = (100..220).map(symbol).collect();
    let mut rng = Rng(0x5EED_2026);

    // ---------- 培养期 ----------
    let mut body_c = Body::new(); // 功能组：只见关系陈述
    let mut body_a = Body::new(); // 空白组：等量无关字节
    let noise_words = ["lorem", "ipsum", "dolor", "amet", "velit", "sed", "tempor", "magna", "aliqua", "enim"];

    let rounds: usize = std::env::var("VALHALLA_REL_ROUNDS")
        .ok().and_then(|v| v.parse().ok()).unwrap_or(10);
    println!("== 培养期（{rounds} 轮） ==");
    // 每轮内含三段：事实（链3单环）→ 成链（链4整链+单环）→ 反事实混喂
    let mut items_c = 0usize;
    for round in 1..=rounds {
        let t0 = std::time::Instant::now();
        for _ in 0..40 {
            let chain = shuffled_pool(&mut rng, &train_pool, 3);
            let items = pairs_shuffled(&chain, &mut rng);
            items_c += items.len();
            body_c.feed_all(&items);
        }
        for _ in 0..40 {
            let chain = shuffled_pool(&mut rng, &train_pool, 4);
            let mut items = vec![chain.join(">").into_bytes()];
            items.extend(pairs_shuffled(&chain, &mut rng));
            items_c += items.len();
            body_c.feed_all(&items);
        }
        for _ in 0..40 {
            let chain = shuffled_pool(&mut rng, &train_pool, 4);
            let mut items = Vec::new();
            let rev: Vec<String> = chain.iter().rev().cloned().collect();
            items.push(rev.join("<").into_bytes());
            items.extend(pairs_shuffled(&chain, &mut rng));
            let frag = shuffled_pool(&mut rng, &train_pool, 3);
            items.push(format!("{}>{}", frag[0], frag[1]).into_bytes());
            let dis = shuffled_pool(&mut rng, &train_pool, 2);
            items.push(format!("{}>{}", dis[0], dis[1]).into_bytes());
            items_c += items.len();
            body_c.feed_all(&items);
        }
        body_c.cycle();
        println!(
            "round {round} {} | {} | {} | 场设备 {} | {:.1}s",
            body_c.monitor.snapshot(&body_c.tile, &body_c.stem).line(),
            body_c.metric_line(),
            body_c.iter_line(),
            body_c.stem.field_device,
            t0.elapsed().as_secs_f64()
        );
    }

    // A 组：等条数无关字节
    for _ in 0..items_c {
        let w = noise_words[rng.below(noise_words.len())];
        let w2 = noise_words[rng.below(noise_words.len())];
        body_a.feed(format!("{w} {w2}").as_bytes());
    }
    body_a.cycle();
    println!("A 空白组 {}", body_a.monitor.snapshot(&body_a.tile, &body_a.stem).line());

    // B 组：与 C 同批训练链，但直接存传递问答对
    let mut body_b = Body::new();
    let mut rng_b = Rng(0x5EED_2026);
    // 用和 C 阶段1相同的链生成方式近似同批（种子同、调用序近似），存全部传递对
    for _ in 0..40 {
        let chain = shuffled_pool(&mut rng_b, &train_pool, 3);
        for i in 0..chain.len() {
            for j in (i + 1)..chain.len() {
                body_b.feed(format!("{}>{}=YES", chain[i], chain[j]).as_bytes());
            }
        }
    }
    body_b.cycle();
    println!("B 查表组 {}", body_b.monitor.snapshot(&body_b.tile, &body_b.stem).line());

    // ---------- 测试期：全新符号，只给单环事实 ----------
    println!("\n== 测试期（未见符号，只给单环） ==");
    let mut rng_t = Rng(0x7E57_2026);
    let mut questions: Vec<Question> = Vec::new();
    let mut fact_items: Vec<Vec<u8>> = Vec::new();
    let mut cursor = 0usize;
    let mut next_chain = |rng: &mut Rng, cursor: &mut usize| -> Vec<String> {
        // 每链独占 4 个符号
        let mut chain: Vec<String> = test_pool[*cursor..*cursor + 4].to_vec();
        *cursor += 4;
        for i in (1..chain.len()).rev() {
            let j = rng.below(i + 1);
            chain.swap(i, j);
        }
        chain
    };
    // 完整链 12 条（链长4）
    let mut full_chains: Vec<Vec<String>> = Vec::new();
    for _ in 0..12 {
        let chain = next_chain(&mut rng_t, &mut cursor);
        fact_items.extend(pairs_shuffled(&chain, &mut rng_t));
        questions.push(Question { from: chain[0].clone(), to: chain[3].clone(), expect_reach: true, kind: "传递" });
        questions.push(Question { from: chain[0].clone(), to: chain[2].clone(), expect_reach: true, kind: "传递" });
        questions.push(Question { from: chain[1].clone(), to: chain[3].clone(), expect_reach: true, kind: "传递" });
        questions.push(Question { from: chain[3].clone(), to: chain[0].clone(), expect_reach: false, kind: "反向应沉默" });
        full_chains.push(chain);
    }
    // 断链 6 条：扣掉中间一环
    for _ in 0..6 {
        let chain = next_chain(&mut rng_t, &mut cursor);
        fact_items.push(format!("{}>{}", chain[0], chain[1]).into_bytes());
        fact_items.push(format!("{}>{}", chain[2], chain[3]).into_bytes());
        questions.push(Question { from: chain[0].clone(), to: chain[3].clone(), expect_reach: false, kind: "断链应沉默" });
        questions.push(Question { from: chain[0].clone(), to: chain[1].clone(), expect_reach: true, kind: "直达环" });
    }
    // 干扰：问链首到从未出现过的符号
    for chain in full_chains.iter().take(6) {
        let other = test_pool[cursor].clone();
        cursor += 1;
        questions.push(Question { from: chain[0].clone(), to: other, expect_reach: false, kind: "干扰应沉默" });
    }

    // 三组拿到同一批测试事实（B 组也喂事实，但它只会查问答对）。
    // C/A 记录每条事实生出的 tile/cell，作为网络答题的入口节点。
    type SymNodes = std::collections::HashMap<String, Vec<String>>;
    fn feed_facts_tracked(body: &mut Body, facts: &[Vec<u8>]) -> (SymNodes, SymNodes, Vec<Fragment>) {
        let mut sym_tiles: SymNodes = SymNodes::new();
        let mut sym_cells: SymNodes = SymNodes::new();
        let mut fragments: Vec<Fragment> = Vec::new();
        for item in facts {
            let (tiles, cells) = body.feed_tracked(item);
            let text = String::from_utf8_lossy(item).into_owned();
            let parts: Vec<&str> = text.split(|c| c == '>' || c == '<').filter(|s| s.len() == 2).collect();
            for sym in &parts {
                sym_tiles.entry(sym.to_string()).or_default().extend(tiles.iter().cloned());
                sym_cells.entry(sym.to_string()).or_default().extend(cells.iter().cloned());
            }
            if parts.len() == 2 {
                fragments.push(Fragment {
                    head: parts[0].to_string(),
                    tail: parts[1].to_string(),
                    tiles,
                    cells,
                    tile_strength: 0.0,
                    stem_strength: 0.0,
                    combined: 0.0,
                });
            }
        }
        (sym_tiles, sym_cells, fragments)
    }
    let (mut c_sym_tiles, c_sym_cells, mut c_fragments) = feed_facts_tracked(&mut body_c, &fact_items);
    let (mut a_sym_tiles, a_sym_cells, mut a_fragments) = feed_facts_tracked(&mut body_a, &fact_items);
    body_b.feed_all(&fact_items);
    for body in [&mut body_c, &mut body_a, &mut body_b] {
        body.cycle();
        body.cycle();
    }

    let sc = score(&body_c, &questions, false);
    let sa = score(&body_a, &questions, false);
    let sb = score(&body_b, &questions, true);
    print_score("C 功能组（存储图可达）", &questions, &sc);
    print_score("A 空白组（存储图可达）", &questions, &sa);
    print_score("B 查表组（问答对）", &questions, &sb);

    println!("\n== 走 Tile/Stem 自发网络答题（不读存储字节） ==");
    let nc_t = score_net(&body_c, &questions, &c_sym_tiles, false);
    let nc_s = score_net(&body_c, &questions, &c_sym_cells, true);
    let na_t = score_net(&body_a, &questions, &a_sym_tiles, false);
    let na_s = score_net(&body_a, &questions, &a_sym_cells, true);
    print_score("C 网络组（Tile 网）", &questions, &nc_t);
    print_score("C 网络组（Stem 网）", &questions, &nc_s);
    print_score("A 网络组（Tile 网）", &questions, &na_t);
    print_score("A 网络组（Stem 网）", &questions, &na_s);

    println!("\n== 全局 Fate 监控：碎片综合成思考 ==");
    fn score_thought(
        body: &mut Body,
        fragments: &mut Vec<Fragment>,
        sym_tiles: &SymNodes,
        qs: &[Question],
        name: &str,
    ) -> Score {
        let mut kinds: Vec<(&'static str, usize, usize)> = Vec::new();
        let mut ok = 0usize;
        let mut shown = 0usize;
        for q in qs {
            let thought = think(body, fragments, sym_tiles, &q.from, &q.to);
            let got = thought.is_some();
            if got && q.expect_reach && shown < 3 {
                let t = thought.as_ref().unwrap();
                println!("    思考示例 {}>{}: {} （最弱环 {:.2}，{} 轮，最强碎片 {}）",
                    q.from, q.to, t.chain.join(" + "), t.strength, t.cycles, t.strongest);
                shown += 1;
            }
            let right = got == q.expect_reach;
            if right {
                ok += 1;
            }
            match kinds.iter_mut().find(|(k, _, _)| *k == q.kind) {
                Some((_, c, t)) => { *c += right as usize; *t += 1; }
                None => kinds.push((q.kind, right as usize, 1)),
            }
        }
        let stem_best = fragments.iter().map(|f| f.stem_strength).fold(0.0f64, f64::max);
        println!("{name} Stem 碎片最强场强: {stem_best:.4}");
        (ok, qs.len(), kinds)
    }
    let stc = score_thought(&mut body_c, &mut c_fragments, &c_sym_tiles, &questions, "C");
    print_score("C 思考组（全局 Fate 综合）", &questions, &stc);
    let sta = score_thought(&mut body_a, &mut a_fragments, &a_sym_tiles, &questions, "A");
    print_score("A 思考组（全局 Fate 综合）", &questions, &sta);

    println!("\n== 小型智能测试（活体，思考通道，新符号） ==");
    // 训练链长最多 4；这里测 6 长链、交错双链、桥接补环、一致性、保持
    let smart_pool: Vec<String> = (220..340).map(symbol).collect();
    let mut rng_s = Rng(0x5A17_2026);
    let mut cur_s = 0usize;
    let mut smart_results: Vec<(&'static str, bool, bool)> = Vec::new(); // (项目, C对错, A对错)

    fn absorb(sym: &mut SymNodes, frags: &[Fragment]) {
        for f in frags {
            sym.entry(f.head.clone()).or_default().extend(f.tiles.iter().cloned());
            sym.entry(f.tail.clone()).or_default().extend(f.tiles.iter().cloned());
        }
    }
    fn ask(body: &mut Body, frags: &mut Vec<Fragment>, sym: &SymNodes, from: &str, to: &str) -> bool {
        think(body, frags, sym, from, to).is_some()
    }

    // 1) 长链：6 个符号 5 环，训练从未见过这个长度
    let mut first_long: Option<(String, String)> = None;
    for _ in 0..3 {
        let chain = take_symbols(&smart_pool, &mut cur_s, 6, &mut rng_s);
        if first_long.is_none() {
            first_long = Some((chain[0].clone(), chain[5].clone()));
        }
        let facts = pairs_shuffled(&chain, &mut rng_s);
        let (_, _, fc) = feed_facts_tracked(&mut body_c, &facts);
        absorb(&mut c_sym_tiles, &fc);
        c_fragments.extend(fc);
        let (_, _, fa) = feed_facts_tracked(&mut body_a, &facts);
        absorb(&mut a_sym_tiles, &fa);
        a_fragments.extend(fa);
        body_c.cycle();
        body_a.cycle();
        let got_c = ask(&mut body_c, &mut c_fragments, &c_sym_tiles, &chain[0], &chain[5]);
        let got_a = ask(&mut body_a, &mut a_fragments, &a_sym_tiles, &chain[0], &chain[5]);
        smart_results.push(("长链传递(6)", got_c, got_a));
        let rev_c = ask(&mut body_c, &mut c_fragments, &c_sym_tiles, &chain[5], &chain[0]);
        let rev_a = ask(&mut body_a, &mut a_fragments, &a_sym_tiles, &chain[5], &chain[0]);
        smart_results.push(("长链反向沉默", !rev_c, !rev_a));
    }

    // 2) 交错双链：两条 4 链的事实混在一起喂，链内答、跨链沉默
    for _ in 0..2 {
        let ch1 = take_symbols(&smart_pool, &mut cur_s, 4, &mut rng_s);
        let ch2 = take_symbols(&smart_pool, &mut cur_s, 4, &mut rng_s);
        let mut facts = pairs_shuffled(&ch1, &mut rng_s);
        facts.extend(pairs_shuffled(&ch2, &mut rng_s));
        for i in (1..facts.len()).rev() {
            let j = rng_s.below(i + 1);
            facts.swap(i, j);
        }
        let (_, _, fc) = feed_facts_tracked(&mut body_c, &facts);
        absorb(&mut c_sym_tiles, &fc);
        c_fragments.extend(fc);
        let (_, _, fa) = feed_facts_tracked(&mut body_a, &facts);
        absorb(&mut a_sym_tiles, &fa);
        a_fragments.extend(fa);
        body_c.cycle();
        body_a.cycle();
        let in_c = ask(&mut body_c, &mut c_fragments, &c_sym_tiles, &ch1[0], &ch1[3]);
        let in_a = ask(&mut body_a, &mut a_fragments, &a_sym_tiles, &ch1[0], &ch1[3]);
        smart_results.push(("交错链内传递", in_c, in_a));
        let cross_c = ask(&mut body_c, &mut c_fragments, &c_sym_tiles, &ch1[0], &ch2[3]);
        let cross_a = ask(&mut body_a, &mut a_fragments, &a_sym_tiles, &ch1[0], &ch2[3]);
        smart_results.push(("交错跨链沉默", !cross_c, !cross_a));
    }

    // 3) 桥接：先缺中间环（应沉默），补上中间环后再问（应答出）
    {
        let chain = take_symbols(&smart_pool, &mut cur_s, 4, &mut rng_s);
        let partial: Vec<Vec<u8>> = vec![
            format!("{}>{}", chain[0], chain[1]).into_bytes(),
            format!("{}>{}", chain[2], chain[3]).into_bytes(),
        ];
        let (_, _, fc) = feed_facts_tracked(&mut body_c, &partial);
        absorb(&mut c_sym_tiles, &fc);
        c_fragments.extend(fc);
        let (_, _, fa) = feed_facts_tracked(&mut body_a, &partial);
        absorb(&mut a_sym_tiles, &fa);
        a_fragments.extend(fa);
        body_c.cycle();
        body_a.cycle();
        let before_c = ask(&mut body_c, &mut c_fragments, &c_sym_tiles, &chain[0], &chain[3]);
        let before_a = ask(&mut body_a, &mut a_fragments, &a_sym_tiles, &chain[0], &chain[3]);
        smart_results.push(("桥接前沉默", !before_c, !before_a));
        let bridge: Vec<Vec<u8>> = vec![format!("{}>{}", chain[1], chain[2]).into_bytes()];
        let (_, _, fc) = feed_facts_tracked(&mut body_c, &bridge);
        absorb(&mut c_sym_tiles, &fc);
        c_fragments.extend(fc);
        let (_, _, fa) = feed_facts_tracked(&mut body_a, &bridge);
        absorb(&mut a_sym_tiles, &fa);
        a_fragments.extend(fa);
        body_c.cycle();
        body_a.cycle();
        let after_c = ask(&mut body_c, &mut c_fragments, &c_sym_tiles, &chain[0], &chain[3]);
        let after_a = ask(&mut body_a, &mut a_fragments, &a_sym_tiles, &chain[0], &chain[3]);
        smart_results.push(("桥接后答出", after_c, after_a));
    }

    // 4) 一致性：重问第一条长链，答案应不变
    if let Some((f, t)) = first_long.clone() {
        let again_c = ask(&mut body_c, &mut c_fragments, &c_sym_tiles, &f, &t);
        let again_a = ask(&mut body_a, &mut a_fragments, &a_sym_tiles, &f, &t);
        smart_results.push(("一致性(重问长链)", again_c, again_a));
    }

    // 5) 保持：重问最早那批问题的前 6 题，不应被后面的喂入冲掉
    {
        let mut ok_c = 0usize;
        let mut ok_a = 0usize;
        for q in questions.iter().take(6) {
            let got_c = ask(&mut body_c, &mut c_fragments, &c_sym_tiles, &q.from, &q.to);
            let got_a = ask(&mut body_a, &mut a_fragments, &a_sym_tiles, &q.from, &q.to);
            if got_c == q.expect_reach { ok_c += 1; }
            if got_a == q.expect_reach { ok_a += 1; }
        }
        println!("    保持(重问前6题): C {ok_c}/6 A {ok_a}/6");
    }

    let c_pass = smart_results.iter().filter(|(_, c, _)| *c).count();
    let a_pass = smart_results.iter().filter(|(_, _, a)| *a).count();
    for (name, c_ok, a_ok) in &smart_results {
        println!("    {name}: C {} A {}", if *c_ok { "对" } else { "错" }, if *a_ok { "对" } else { "错" });
    }
    println!("小型智能测试合计: C {c_pass}/{} A {a_pass}/{}", smart_results.len(), smart_results.len());

    // ---------- 教师关闭：存盘 → 新身体只加载四元存储 ----------
    println!("\n== 教师关闭（存盘重启，只加载四元存储） ==");
    let dir = std::env::temp_dir().join("valhalla_relation_incubate");
    body_c.save_quads(&dir).expect("存盘失败");
    let mut body_d = Body::new();
    body_d.load_quads(&dir).expect("加载失败");
    let sd = score(&body_d, &questions, false);
    print_score("D 重载组（图可达）", &questions, &sd);
    println!("D 结构快照（新身体，无培养结构）: {}", body_d.monitor.snapshot(&body_d.tile, &body_d.stem).line());
    println!("C 结构快照: {}", body_c.monitor.snapshot(&body_c.tile, &body_c.stem).line());

    // ---------- 消融 ----------
    println!("\n== 消融 ==");
    let mut body_e = Body::new();
    body_e.load_quads(&dir).expect("加载失败");
    body_e.clear_quads();
    let se = score(&body_e, &questions, false);
    print_score("E 消融组（清四元存储）", &questions, &se);

    println!("\n== 交叉消融：清 C 的四元存储，只留长出的网络 ==");
    body_c.clear_quads();
    let nc_after = score_net(&body_c, &questions, &c_sym_tiles, false);
    let sc_after = score(&body_c, &questions, false);
    print_score("C 清存储后（Tile 网）", &questions, &nc_after);
    print_score("C 清存储后（存储图可达）", &questions, &sc_after);
    println!("注：D 组身体里没有网络（只重载了四元存储），网络答题过不了存盘关，除非 Tile/Stem 结构本体也落盘。");

    println!("\n判读说明：图可达判读器是哑的，只读四元存储。");
    println!("C 与 A 若同分，说明组合动作目前由判读器+存储完成，培养的增益要看结构差与后续消融；");
    println!("D 与 C 同分说明能力随四元存储存盘存活；E 应全灭，否则存储之外还有来源。");
}




