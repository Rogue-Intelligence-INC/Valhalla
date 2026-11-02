//! Industry-style test battery (all self-generated analogues, not official dataset scores):
//! T1 rule induction (facts withheld: no facts fed at test; truth = transitive closure of the training edge set)
//! T2 SCAN-style length generalisation (short-chain vs long-chain training, tested on long-chain multi-hop)
//! T3 CLUTRR-style family-tree multi-hop (stratified by hops)
//! T4 CL-bench-style continual learning (after pool2, retest pool1 for forgetting)
//! Answer channels: the storage graph (dumb reachability over read-only quad bytes) and the self-formed Tile/Stem networks, scored separately.
//! MiniGrid Memory / Bongard-LOGO / ARC-AGI need action or grid-vision channels the current architecture lacks; not fake-run here.

use hub_f64::monitor::BodyMonitor;
use hub_f64::TripleFateCombined;
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
        // The old Fate arrangement: one multi-layer tick per cycle - base judges the mode, meta observes,
        // emergent rules push back; death counts are captured by global Fate together with body-state signals.
        // Previously this only receive_passive'd inside feed; the machine never actually turned.
        let state_signal = vec![
            t.tile_count as f64,
            s.cell_count as f64,
            (t.merges + s.merges) as f64,
            (t.splits + s.divisions) as f64,
            (t.deaths + s.deaths) as f64,
        ];
        let _ = self.global.tick(Some(state_signal));
        self.stem.settle_field();
        let p = self.tile.propagate(1.0);
        self.monitor.observe_propagate(p.new_connections);
        self.monitor.observe_cycle(&t, &s);
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
}

type Adj = std::collections::HashMap<String, Vec<String>>;
type SymNodes = std::collections::HashMap<String, Vec<String>>;

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

/// Symbol-graph BFS distance (test-side truth computation; not used in answering)
fn sym_dist(edges: &[(String, String)], from: &str, to: &str) -> Option<usize> {
    if from == to {
        return None;
    }
    let mut dist: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    dist.insert(from.to_string(), 0);
    let mut queue = std::collections::VecDeque::from([from.to_string()]);
    while let Some(cur) = queue.pop_front() {
        let d = dist[&cur];
        for (a, b) in edges {
            if a == &cur && !dist.contains_key(b) {
                if b == to {
                    return Some(d + 1);
                }
                dist.insert(b.clone(), d + 1);
                queue.push_back(b.clone());
            }
        }
    }
    None
}

#[derive(Clone)]
struct Question {
    from: String,
    to: String,
    expect_reach: bool,
    cat: &'static str,
}

/// Channels scored separately; returns (storage %, network %)
fn score_both(body: &Body, qs: &[Question], sym_t: &SymNodes, sym_s: &SymNodes) -> (f64, f64) {
    let edges = parse_relations(&body.stored_bytes());
    let adj_t = net_adj(body, false);
    let adj_s = net_adj(body, true);
    let empty: Vec<String> = Vec::new();
    let (mut ok_g, mut ok_n) = (0usize, 0usize);
    for q in qs {
        if sym_dist(&edges, &q.from, &q.to).is_some() == q.expect_reach {
            ok_g += 1;
        }
        let got = net_reachable(&adj_t, sym_t.get(&q.from).unwrap_or(&empty), sym_t.get(&q.to).unwrap_or(&empty))
            || net_reachable(&adj_s, sym_s.get(&q.from).unwrap_or(&empty), sym_s.get(&q.to).unwrap_or(&empty));
        if got == q.expect_reach {
            ok_n += 1;
        }
    }
    let n = qs.len().max(1) as f64;
    (ok_g as f64 / n * 100.0, ok_n as f64 / n * 100.0)
}

fn score_cat(body: &Body, qs: &[Question], sym_t: &SymNodes, sym_s: &SymNodes, cat: &str) -> (f64, f64, usize) {
    let sub: Vec<Question> = qs.iter().filter(|q| q.cat == cat).cloned().collect();
    let n = sub.len();
    let (g, net) = score_both(body, &sub, sym_t, sym_s);
    (g, net, n)
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

fn merge_sym(dst: &mut SymNodes, src: SymNodes) {
    for (k, v) in src {
        dst.entry(k).or_default().extend(v);
    }
}

/// Standard cultivation (three stages, same curriculum as the causal proof); returns the training edge set
fn cultivate_tracked(rng: &mut Rng, body: &mut Body, pool: &[String], sym_t: &mut SymNodes, sym_s: &mut SymNodes) -> Vec<(String, String)> {
    let mut edges: Vec<(String, String)> = Vec::new();
    let mut run = |body: &mut Body, items: &[Vec<u8>], sym_t: &mut SymNodes, sym_s: &mut SymNodes, edges: &mut Vec<(String, String)>| {
        for it in items {
            edges.extend(parse_relations(it));
        }
        let (t, s) = feed_facts_tracked(body, items);
        merge_sym(sym_t, t);
        merge_sym(sym_s, s);
        body.cycle();
    };
    let mut items: Vec<Vec<u8>> = Vec::new();
    for _ in 0..40 {
        let chain = shuffled_pool(rng, pool, 3);
        items.extend(pairs_shuffled(&chain, rng));
    }
    run(body, &items, sym_t, sym_s, &mut edges);
    let mut items: Vec<Vec<u8>> = Vec::new();
    for _ in 0..40 {
        let chain = shuffled_pool(rng, pool, 4);
        items.push(chain.join(">").into_bytes());
        items.extend(pairs_shuffled(&chain, rng));
    }
    run(body, &items, sym_t, sym_s, &mut edges);
    let mut items: Vec<Vec<u8>> = Vec::new();
    for _ in 0..40 {
        let chain = shuffled_pool(rng, pool, 4);
        let rev: Vec<String> = chain.iter().rev().cloned().collect();
        items.push(rev.join("<").into_bytes());
        items.extend(pairs_shuffled(&chain, rng));
        let frag = shuffled_pool(rng, pool, 3);
        items.push(format!("{}>{}", frag[0], frag[1]).into_bytes());
        let dis = shuffled_pool(rng, pool, 2);
        items.push(format!("{}>{}", dis[0], dis[1]).into_bytes());
    }
    run(body, &items, sym_t, sym_s, &mut edges);
    edges
}

fn pct(rows: &[f64]) -> (f64, f64) {
    if rows.is_empty() {
        return (0.0, 0.0);
    }
    let m = rows.iter().sum::<f64>() / rows.len() as f64;
    let s = (rows.iter().map(|r| (r - m).powi(2)).sum::<f64>() / rows.len() as f64).sqrt();
    (m, s)
}

// ---------------- T1 rule induction (facts withheld, disjoint chain families) ----------------
fn t1_seed(seed: u64) -> (f64, f64, f64, f64, f64, f64) {
    // 12 disjoint 5-symbol chains: closure exists only in-chain; cross-chain/reverse are true negatives
    let mut rng = Rng(0x1111_0000 + seed * 7919);
    let mut chains: Vec<Vec<String>> = Vec::new();
    for k in 0..12 {
        let mut chain: Vec<String> = (k * 5..k * 5 + 5).map(symbol).collect();
        for i in (1..chain.len()).rev() {
            let j = rng.below(i + 1);
            chain.swap(i, j);
        }
        chains.push(chain);
    }
    let mut body = Body::new();
    let (mut sym_t, mut sym_s) = (SymNodes::new(), SymNodes::new());
    let mut edges: Vec<(String, String)> = Vec::new();
    for _pass in 0..3 {
        let mut items: Vec<Vec<u8>> = Vec::new();
        for chain in &chains {
            items.push(chain.join(">").into_bytes());
            items.extend(pairs_shuffled(chain, &mut rng));
        }
        for i in (1..items.len()).rev() {
            let j = rng.below(i + 1);
            items.swap(i, j);
        }
        for it in &items {
            edges.extend(parse_relations(it));
        }
        let (t, s) = feed_facts_tracked(&mut body, &items);
        merge_sym(&mut sym_t, t);
        merge_sym(&mut sym_s, s);
        body.cycle();
    }

    // Candidate questions: in-chain ordered pairs (closure-labelled) + cross-chain pairs (true negatives), drawn by quota
    let mut cands: Vec<Question> = Vec::new();
    for chain in &chains {
        for a in chain {
            for b in chain {
                if a != b {
                    let cat = match sym_dist(&edges, a, b) {
                        Some(1) => "t1_d1",
                        Some(2) => "t1_d2",
                        Some(3) => "t1_d3",
                        Some(_) => "t1_d4p",
                        None => "t1_sil",
                    };
                    cands.push(Question { from: a.clone(), to: b.clone(), expect_reach: cat != "t1_sil", cat });
                }
            }
        }
    }
    let mut rng_q = Rng(0x1112_0000 + seed * 104729);
    for _ in 0..40 {
        let i = rng_q.below(chains.len());
        let mut j = rng_q.below(chains.len());
        if i == j {
            j = (j + 1) % chains.len();
        }
        cands.push(Question { from: chains[i][0].clone(), to: chains[j][2].clone(), expect_reach: false, cat: "t1_sil" });
    }
    for i in (1..cands.len()).rev() {
        let j = rng_q.below(i + 1);
        cands.swap(i, j);
    }
    let quota = |c: &str| -> usize { match c { "t1_d1" => 15, "t1_d2" => 20, "t1_d3" => 15, "t1_d4p" => 10, _ => 30 } };
    let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    let mut qs: Vec<Question> = Vec::new();
    for q in cands {
        if counts.get(q.cat).copied().unwrap_or(0) < quota(q.cat) {
            *counts.entry(q.cat).or_insert(0) += 1;
            qs.push(q);
        }
    }
    println!("    T1 questions: d1 {} d2 {} d3 {} d4+ {} silent {}", counts.get("t1_d1").unwrap_or(&0), counts.get("t1_d2").unwrap_or(&0), counts.get("t1_d3").unwrap_or(&0), counts.get("t1_d4p").unwrap_or(&0), counts.get("t1_sil").unwrap_or(&0));
    let (cg, cn) = score_both(&body, &qs, &sym_t, &sym_s);
    let (_, cn2, _) = score_cat(&body, &qs, &sym_t, &sym_s, "t1_d2");
    let (_, cn3, _) = score_cat(&body, &qs, &sym_t, &sym_s, "t1_d3");
    let (_, cns, _) = score_cat(&body, &qs, &sym_t, &sym_s, "t1_sil");

    // A: noise body, likewise no test facts fed (empty symbol nodes -> network fully silent)
    let mut body_a = Body::new();
    let noise = ["lorem", "ipsum", "dolor", "amet", "velit", "sed", "tempor", "magna"];
    for _ in 0..900 {
        body_a.feed(format!("{} {}", noise[rng.below(8)], noise[rng.below(8)]).as_bytes());
    }
    body_a.cycle();
    let (ag, an) = score_both(&body_a, &qs, &SymNodes::new(), &SymNodes::new());
    println!("seed {seed} T1 rule induction (withheld): C storage {cg:.1} network {cn:.1} (2-hop {cn2:.1} 3-hop {cn3:.1} silent {cns:.1}) | A storage {ag:.1} network {an:.1}");
    (cg, cn, cn2, cn3, cns, an)
}
// ---------------- T2 SCAN-style length generalisation ----------------
fn cultivate_len(rng: &mut Rng, body: &mut Body, pool: &[String], max_len: usize) {
    for stage in 0..3 {
        let mut items: Vec<Vec<u8>> = Vec::new();
        for _ in 0..40 {
            let len = 2 + rng.below(max_len - 1).min(max_len - 2 + stage.min(1));
            let chain = shuffled_pool(rng, pool, len.max(2));
            if stage > 0 {
                items.push(chain.join(">").into_bytes());
            }
            items.extend(pairs_shuffled(&chain, rng));
        }
        body.feed_all(&items);
        body.cycle();
    }
}

fn t2_seed(seed: u64) -> (f64, f64, f64, f64) {
    let pool: Vec<String> = (0..30).map(symbol).collect();
    let test_base = 300;
    let test_pool: Vec<String> = (test_base..test_base + 120).map(symbol).collect();
    let mut rng = Rng(0x2222_0000 + seed * 7919);
    let mut short = Body::new();
    cultivate_len(&mut rng, &mut short, &pool, 3);
    let mut long = Body::new();
    cultivate_len(&mut rng, &mut long, &pool, 6);

    // Test: 6 long chains; facts given at test time per protocol
    let mut rng_t = Rng(0x2223_0000 + seed * 104729);
    let mut qs: Vec<Question> = Vec::new();
    let mut facts: Vec<Vec<u8>> = Vec::new();
    let mut cursor = 0usize;
    for _ in 0..12 {
        let mut chain: Vec<String> = test_pool[cursor..cursor + 6].to_vec();
        cursor += 6;
        for i in (1..chain.len()).rev() {
            let j = rng_t.below(i + 1);
            chain.swap(i, j);
        }
        facts.extend(pairs_shuffled(&chain, &mut rng_t));
        qs.push(Question { from: chain[0].clone(), to: chain[5].clone(), expect_reach: true, cat: "t2_d5" });
        qs.push(Question { from: chain[0].clone(), to: chain[4].clone(), expect_reach: true, cat: "t2_d4" });
        qs.push(Question { from: chain[1].clone(), to: chain[5].clone(), expect_reach: true, cat: "t2_d4" });
        qs.push(Question { from: chain[0].clone(), to: chain[2].clone(), expect_reach: true, cat: "t2_d2" });
        qs.push(Question { from: chain[5].clone(), to: chain[0].clone(), expect_reach: false, cat: "t2_sil" });
    }
    let (st, ss) = feed_facts_tracked(&mut short, &facts);
    short.cycle();
    let (lt, ls) = feed_facts_tracked(&mut long, &facts);
    long.cycle();
    let (_, sn45, _) = {
        let sub: Vec<Question> = qs.iter().filter(|q| q.cat == "t2_d5" || q.cat == "t2_d4").cloned().collect();
        let (g, n) = score_both(&short, &sub, &st, &ss);
        (g, n, sub.len())
    };
    let (_, ln45, _) = {
        let sub: Vec<Question> = qs.iter().filter(|q| q.cat == "t2_d5" || q.cat == "t2_d4").cloned().collect();
        let (g, n) = score_both(&long, &sub, &lt, &ls);
        (g, n, sub.len())
    };
    let (sg, sn) = score_both(&short, &qs, &st, &ss);
    let (lg, ln) = score_both(&long, &qs, &lt, &ls);
    println!("seed {seed} T2 length generalisation: short-trained storage {sg:.1} network {sn:.1} (4-5 hop {sn45:.1}) | long-trained storage {lg:.1} network {ln:.1} (4-5 hop {ln45:.1})");
    (sn, ln, sn45, ln45)
}

// ---------------- T3 CLUTRR-style family-tree multi-hop ----------------
fn t3_seed(seed: u64) -> (f64, f64, f64, f64, f64) {
    let pool: Vec<String> = (0..30).map(symbol).collect();
    let mut rng = Rng(0x3333_0000 + seed * 7919);
    let mut body = Body::new();
    let (mut _t, mut _s) = (SymNodes::new(), SymNodes::new());
    cultivate_tracked(&mut rng, &mut body, &pool, &mut _t, &mut _s);

    // Family tree: per family 2 grandparents -> parents -> 3 children (edge = parent-of); symbols exclusive
    let fam_base = 500;
    let mut rng_t = Rng(0x3334_0000 + seed * 104729);
    let mut qs: Vec<Question> = Vec::new();
    let mut facts: Vec<Vec<u8>> = Vec::new();
    let mut heads: Vec<String> = Vec::new();
    for f in 0..10 {
        let b = fam_base + f * 7;
        let (g1, g2, p, c1, c2, c3) = (symbol(b), symbol(b + 1), symbol(b + 2), symbol(b + 3), symbol(b + 4), symbol(b + 5));
        let mut fam_facts = vec![
            format!("{g1}>{p}"), format!("{g2}>{p}"),
            format!("{p}>{c1}"), format!("{p}>{c2}"), format!("{p}>{c3}"),
        ];
        for i in (1..fam_facts.len()).rev() {
            let j = rng_t.below(i + 1);
            fam_facts.swap(i, j);
        }
        facts.extend(fam_facts.iter().map(|s| s.clone().into_bytes()));
        heads.push(g1.clone());
        qs.push(Question { from: p.clone(), to: c1.clone(), expect_reach: true, cat: "t3_d1" });
        qs.push(Question { from: g1.clone(), to: c1.clone(), expect_reach: true, cat: "t3_d2" });
        qs.push(Question { from: g2.clone(), to: c3.clone(), expect_reach: true, cat: "t3_d2" });
        qs.push(Question { from: c1.clone(), to: g1.clone(), expect_reach: false, cat: "t3_rev" });
        qs.push(Question { from: c1.clone(), to: c2.clone(), expect_reach: false, cat: "t3_sib" });
    }
    for i in 0..10 {
        qs.push(Question { from: heads[i].clone(), to: heads[(i + 1) % 10].clone(), expect_reach: false, cat: "t3_xfam" });
    }
    let (st, ss) = feed_facts_tracked(&mut body, &facts);
    body.cycle();
    let (g, n) = score_both(&body, &qs, &st, &ss);
    let (_, n2, _) = score_cat(&body, &qs, &st, &ss, "t3_d2");
    let (_, nrev, _) = score_cat(&body, &qs, &st, &ss, "t3_rev");
    let (_, nsib, _) = score_cat(&body, &qs, &st, &ss, "t3_sib");
    println!("seed {seed} T3 family multi-hop: storage {g:.1} network {n:.1} (2-hop {n2:.1} reverse silence {nrev:.1} sibling silence {nsib:.1})");
    (g, n, n2, nrev, nsib)
}

// ---------------- T4 CL-bench-style continual learning ----------------
fn battery(rng: &mut Rng, test_pool: &[String], cursor: &mut usize) -> (Vec<Question>, Vec<Vec<u8>>) {
    let mut qs: Vec<Question> = Vec::new();
    let mut facts: Vec<Vec<u8>> = Vec::new();
    for _ in 0..10 {
        let mut chain: Vec<String> = test_pool[*cursor..*cursor + 4].to_vec();
        *cursor += 4;
        for i in (1..chain.len()).rev() {
            let j = rng.below(i + 1);
            chain.swap(i, j);
        }
        facts.extend(pairs_shuffled(&chain, rng));
        qs.push(Question { from: chain[0].clone(), to: chain[3].clone(), expect_reach: true, cat: "t4_pos" });
        qs.push(Question { from: chain[0].clone(), to: chain[2].clone(), expect_reach: true, cat: "t4_pos" });
        qs.push(Question { from: chain[3].clone(), to: chain[0].clone(), expect_reach: false, cat: "t4_neg" });
    }
    (qs, facts)
}

fn t4_seed(seed: u64) -> (f64, f64, f64, f64, f64, f64) {
    let pool1: Vec<String> = (0..30).map(symbol).collect();
    let pool2: Vec<String> = (40..70).map(symbol).collect();
    let test1: Vec<String> = (100..160).map(symbol).collect();
    let test2: Vec<String> = (200..260).map(symbol).collect();
    let mut rng = Rng(0x4444_0000 + seed * 7919);
    let mut body = Body::new();
    let (mut sym_t, mut sym_s) = (SymNodes::new(), SymNodes::new());
    cultivate_tracked(&mut rng, &mut body, &pool1, &mut sym_t, &mut sym_s);

    let mut rng_t = Rng(0x4445_0000 + seed * 104729);
    let mut cur = 0usize;
    let (q1, f1) = battery(&mut rng_t, &test1, &mut cur);
    let (t1m, s1m) = feed_facts_tracked(&mut body, &f1);
    merge_sym(&mut sym_t, t1m);
    merge_sym(&mut sym_s, s1m);
    body.cycle();
    let (g_before, n_before) = score_both(&body, &q1, &sym_t, &sym_s);

    // Stage two: learn the disjoint pool2 (same scale)
    let (mut t2m, mut s2m) = (SymNodes::new(), SymNodes::new());
    cultivate_tracked(&mut rng, &mut body, &pool2, &mut t2m, &mut s2m);
    merge_sym(&mut sym_t, t2m);
    merge_sym(&mut sym_s, s2m);
    let mut cur2 = 0usize;
    let (q2, f2) = battery(&mut rng_t, &test2, &mut cur2);
    let (t2b, s2b) = feed_facts_tracked(&mut body, &f2);
    merge_sym(&mut sym_t, t2b);
    merge_sym(&mut sym_s, s2b);
    body.cycle();
    let (g2, n2) = score_both(&body, &q2, &sym_t, &sym_s);

    // Retest battery1: facts not re-fed (withheld); relies on storage and already-grown structure
    let (g_after, n_after) = score_both(&body, &q1, &sym_t, &sym_s);
    println!("seed {seed} T4 continual learning: task1 after learning storage {g_before:.1} network {n_before:.1} -> retest after task2 storage {g_after:.1} network {n_after:.1} | task2 storage {g2:.1} network {n2:.1}");
    (g_before, n_before, g_after, n_after, g2, n2)
}

fn main() {
    let seeds: Vec<u64> = std::env::var("VALHALLA_IND_SEEDS")
        .unwrap_or_else(|_| "1,2,3,4,5".to_string())
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    println!("Industry-style battery (self-generated analogues, not official scores); seeds = {seeds:?}");
    let (mut t1, mut t2, mut t3, mut t4) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for seed in &seeds {
        println!("== seed {seed} ==");
        t1.push(t1_seed(*seed));
        t2.push(t2_seed(*seed));
        t3.push(t3_seed(*seed));
        t4.push(t4_seed(*seed));
    }
    let col = |rows: &[Vec<f64>], i: usize| -> Vec<f64> { rows.iter().map(|r| r[i]).collect() };
    let t1v: Vec<Vec<f64>> = t1.iter().map(|r| vec![r.0, r.1, r.2, r.3, r.4, r.5]).collect();
    let t2v: Vec<Vec<f64>> = t2.iter().map(|r| vec![r.0, r.1, r.2, r.3]).collect();
    let t3v: Vec<Vec<f64>> = t3.iter().map(|r| vec![r.0, r.1, r.2, r.3, r.4]).collect();
    let t4v: Vec<Vec<f64>> = t4.iter().map(|r| vec![r.0, r.1, r.2, r.3, r.4, r.5]).collect();
    let show = |name: &str, rows: &[Vec<f64>], i: usize| {
        let (m, s) = pct(&col(rows, i));
        println!("  {name}: {m:.1} ± {s:.1}");
    };
    println!("\n== Industry battery summary ({} seeds, %) ==", seeds.len());
    println!("T1 rule induction (no facts fed at test):");
    show("C storage channel", &t1v, 0);
    show("C network channel", &t1v, 1);
    show("C network 2-hop", &t1v, 2);
    show("C network 3-hop", &t1v, 3);
    show("C network silence", &t1v, 4);
    show("A network channel", &t1v, 5);
    println!("T2 SCAN-style length generalisation (6 long chains tested):");
    show("short-trained(<=3) network", &t2v, 0);
    show("long-trained(<=6) network", &t2v, 1);
    show("short-trained 4-5 hop", &t2v, 2);
    show("long-trained 4-5 hop", &t2v, 3);
    println!("T3 CLUTRR-style family-tree multi-hop:");
    show("storage channel", &t3v, 0);
    show("network channel", &t3v, 1);
    show("network 2-hop (grandparents)", &t3v, 2);
    show("network reverse silence", &t3v, 3);
    show("network sibling silence", &t3v, 4);
    println!("T4 CL-bench-style continual learning:");
    show("task1 learned storage", &t4v, 0);
    show("task1 learned network", &t4v, 1);
    show("task1 after-task2 storage", &t4v, 2);
    show("task1 after-task2 network", &t4v, 3);
    show("task2 storage", &t4v, 4);
    show("task2 network", &t4v, 5);
    let (gb, _) = pct(&col(&t4v, 0));
    let (ga, _) = pct(&col(&t4v, 2));
    let (nb, _) = pct(&col(&t4v, 1));
    let (na, _) = pct(&col(&t4v, 3));
    println!("  Backward transfer BWT: storage {:+.1} pp, network {:+.1} pp (negative = forgetting)", ga - gb, na - nb);
}


