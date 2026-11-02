//! Valhalla full flow (consolidated, 2026-11-01): one programme, ingress-body-egress.
//!
//! Ingress (Fate auto-segmentation, original design): all material
//! (multilingual text / code / image / audio bytes / symbolic facts)
//! enters byte-wise through Stem/Tile receive_data - containers only
//! collect; Fate judges convergence with its own cluster equation and
//! closes segments on a hit; segment length is decided by the equation
//! and the material's statistics, the feeder presets no granularity.
//! Storage discipline unchanged: raw bytes are fully accounted in the
//! quad stores on both sides (natural material in 8KB stream blocks,
//! facts per item); birth segmentation is decoupled from storage grain.
//! Body: each round = one batch of material + several body cycles
//! (tile/stem cycles, global Fate tick, field settling, propagate),
//! on the trainer's rhythm.
//! Egress (test phase; channels scored separately, never mixed):
//!   A native response: held-out material enters byte-wise; record how
//!      Fate segments it and where newborns land in field space - the
//!      response form is unconstrained, only recorded.
//!   B structural QA (harness reads body state): chains fed in training,
//!      withheld transitive questions at test; storage channel = edges
//!      drawn from quad bytes with bounded reachability, network channel
//!      = bounded reachability over self-formed Tile/Stem connections
//!      (symbol-to-unit mapping from segment-level birth accounting
//!      while facts were fed).
//!   C external Fate QA (the react interface fixed in bbea60a is the QA
//!      interface): while feeding facts, the same bytes are double-written
//!      to global Fate's data_store (the mainline feed previously wrote
//!      only the quads, data_store stayed empty, react had nothing to
//!      send - this wiring was the missing step). At test, call react
//!      once per hop and parse the edges in the originals it sends;
//!      react is &self read-only: the stimulus does not enter internal
//!      Fate, does not tick, does not change the equation (the discipline
//!      is guaranteed by the signature; questions do not enter the body).
//! Checkpoints: measured at rounds 10 / 20 (same body keeps training),
//! reading whether ten more rounds of mixed material preserve or wash
//! the fact structure. Not included: vent-style extraction egress (D11
//! not established).
//!
//! Env: VH_ROUNDS (total rounds, default 20), VH_CHUNK_KB (KB per
//! material per round, default 128), VH_CYCLES (cycles per round,
//! default 2), VH_SEED. VH_MATS=ext extended materials (+MIDI /
//! binary / poetry / science, default off); VH_HOLDOUT_KB held-out
//! tail KB (default 128). See docs/valhalla/FULL_FLOW_RUNBOOK.md.

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};

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

fn env_usize(key: &str, default: usize) -> usize {
    std::env::var(key).ok().and_then(|v| v.parse().ok()).filter(|&v| v > 0).unwrap_or(default)
}

struct Body {
    tile: TileQuattroComplete,
    stem: StemCellComplete,
    #[allow(dead_code)]
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
    /// Fate segmentation ingress: storage accounting (stream blocks) + byte-wise closed births.
    /// Returns (new stem segments, new tile segments).
    fn feed_fateseg(&mut self, bytes: &[u8]) -> (usize, usize) {
        for chunk in bytes.chunks(8 * 1024) {
            self.tile.quad_keep(chunk);
            self.stem.quad_keep(chunk);
        }
        let c0 = self.stem.cells.len();
        let t0 = self.tile.completed_tiles.len();
        for &byte in bytes {
            let v = byte as f64;
            self.global.receive_passive(v);
            self.stem.receive_data(v);
            self.tile.receive_data(v);
        }
        (self.stem.cells.len() - c0, self.tile.completed_tiles.len() - t0)
    }
    fn cycle(&mut self) {
        let t = self.tile.process_tiles_cycle();
        let s = self.stem.process_cells_cycle();
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
    fn stored_total(&self) -> usize {
        [
            &self.tile.blackhole.stored, &self.tile.meshbrain.stored,
            &self.tile.multinova.stored, &self.tile.baseforce.stored,
            &self.stem.blackhole.stored, &self.stem.meshbrain.stored,
            &self.stem.multinova.stored, &self.stem.baseforce.stored,
        ]
        .iter()
        .map(|s| s.len())
        .sum()
    }
    fn stored_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        for store in [
            &self.tile.blackhole.stored, &self.tile.meshbrain.stored,
            &self.tile.multinova.stored, &self.tile.baseforce.stored,
            &self.stem.blackhole.stored, &self.stem.meshbrain.stored,
            &self.stem.multinova.stored, &self.stem.baseforce.stored,
        ] {
            for item in store.iter() {
                out.extend_from_slice(item);
                out.push(b'\n');
            }
        }
        out
    }
    fn fingerprint(&self, tag: &str) {
        let links: usize = self.stem.cells.iter().map(|c| c.connections.len()).sum();
        println!(
            "{tag} tiles {} cells {} stem_links {} stored {} equation {}",
            self.tile.completed_tiles.len(),
            self.stem.cells.len(),
            links,
            self.stored_total(),
            self.global.cluster_equation.equation_str()
        );
    }
}

/// One external Fate hop: call react with a symbol stimulus, parse the edges in the originals it sends,
/// return the symbols at both ends. No react hit means empty - silence honestly shown as no neighbours.
fn react_neighbors(global: &TripleFateCombined, sym: &str) -> Vec<String> {
    let out = global.react(sym.as_bytes());
    let mut ns = Vec::new();
    for w in out.windows(7) {
        if w[3] == b'>' && w[..3].iter().all(|c| c.is_ascii_uppercase()) && w[4..].iter().all(|c| c.is_ascii_uppercase()) {
            ns.push(String::from_utf8_lossy(&w[..3]).into_owned());
            ns.push(String::from_utf8_lossy(&w[4..]).into_owned());
        }
    }
    ns
}

/// Bounded reachability (undirected, budget = steps)
fn reachable(adj: &HashMap<String, Vec<String>>, from: &[String], to: &HashSet<String>, budget: usize) -> bool {
    let mut seen: HashSet<String> = HashSet::new();
    let mut q: VecDeque<(String, usize)> = VecDeque::new();
    for f in from {
        if to.contains(f) {
            return true;
        }
        if seen.insert(f.clone()) {
            q.push_back((f.clone(), 0));
        }
    }
    while let Some((u, d)) = q.pop_front() {
        if d >= budget {
            continue;
        }
        if let Some(ns) = adj.get(&u) {
            for n in ns {
                if to.contains(n) {
                    return true;
                }
                if seen.insert(n.clone()) {
                    q.push_back((n.clone(), d + 1));
                }
            }
        }
    }
    false
}

fn symbol(i: usize) -> String {
    let a = (b'A' + ((i / 676) % 26) as u8) as char;
    let b = (b'A' + ((i / 26) % 26) as u8) as char;
    let c = (b'A' + (i % 26) as u8) as char;
    format!("{a}{b}{c}")
}

fn read_file(p: &Path) -> Vec<u8> {
    std::fs::read(p).unwrap_or_default()
}

fn main() {
    // Large body cycles involve deep recursion / big frames; run main logic on a big-stack thread, as in probes.
    let h = std::thread::Builder::new()
        .stack_size(512 * 1024 * 1024)
        .spawn(run)
        .expect("spawn big-stack thread");
    let _ = h.join();
}

fn run() {
    let rounds = env_usize("VH_ROUNDS", 20);
    let chunk = env_usize("VH_CHUNK_KB", 128) * 1024;
    let cycles = env_usize("VH_CYCLES", 2);
    let seed: u64 = std::env::var("VH_SEED").ok().and_then(|v| v.parse().ok()).unwrap_or(7);
    let mut rng = Rng(seed ^ 0x5EED);

    // -- Material sources (four natural kinds) --
    let div = Path::new(r"C:\Ai\rogue-products\valhalla-self-incubate\corpus_diverse");
    let mut text = Vec::new();
    for f in ["zh_honglou_24264.txt", "fr_montecristo_17989.txt", "de_faust_2229.txt"] {
        text.extend_from_slice(&read_file(&div.join(f)));
    }
    let mut code = Vec::new();
    for f in ["manifestsys/hub-f64/src/bin/valhalla_probes.rs", "manifestsys/stem-cell/src/stemcell_complete.rs"] {
        code.extend_from_slice(&read_file(&Path::new(r"C:\Ai\rogue-products\Valhalla").join(f)));
    }
    let mut img = Vec::new();
    if let Ok(rd) = std::fs::read_dir(r"C:\Windows\Web\Wallpaper\Windows") {
        let mut ps: Vec<PathBuf> = rd.flatten().map(|e| e.path()).filter(|p| p.extension().and_then(|x| x.to_str()) == Some("jpg")).collect();
        ps.sort();
        for p in ps {
            img.extend_from_slice(&read_file(&p));
        }
    }
    let mut aud = Vec::new();
    for f in ["Alarm01.wav", "Alarm02.wav", "Alarm03.wav"] {
        aud.extend_from_slice(&read_file(&Path::new(r"C:\Windows\Media").join(f)));
    }
    let mut mats: Vec<(&str, &Vec<u8>)> = vec![("multilingual text", &text), ("code", &code), ("image", &img), ("audio", &aud)];
    // Extended materials (VH_MATS=ext, default off, default programme
    // unchanged): four more statistically distinct kinds - MIDI music,
    // binary programs, poetry, scientific text - showing Fate
    // segmentation and births differentiating by material. MIDI is
    // small (86KB); round-robin feeding re-exposes it, accounted as-is
    // (real behaviour on a small corpus).
    let mut midi = Vec::new();
    let mut bin = Vec::new();
    let mut poetry = Vec::new();
    let mut science = Vec::new();
    if std::env::var("VH_MATS").map(|v| v == "ext").unwrap_or(false) {
        for f in ["flourish.mid", "onestop.mid", "town.mid"] {
            midi.extend_from_slice(&read_file(&Path::new(r"C:\Windows\Media").join(f)));
        }
        bin.extend_from_slice(&read_file(Path::new(r"C:\Windows\System32\notepad.exe")));
        bin.extend_from_slice(&read_file(Path::new(r"C:\Windows\System32\kernel32.dll")));
        poetry.extend_from_slice(&read_file(&div.join("poetry_leaves_1322.txt")));
        science.extend_from_slice(&read_file(&div.join("science_relativity_30155.txt")));
        mats.push(("MIDI music", &midi));
        mats.push(("binary", &bin));
        mats.push(("poetry", &poetry));
        mats.push(("science", &science));
    }
    println!("Materials: {}", mats.iter().map(|(n, b)| format!("{n} {}KB", b.len() / 1024)).collect::<Vec<_>>().join(" / "));
    // Held-out tail (response test only, training rounds never touch it):
    // last 128KB of each material (VH_HOLDOUT_KB adjustable; small
    // extended corpora need it lowered to yield usable segments)
    let holdout = env_usize("VH_HOLDOUT_KB", 128) * 1024;

    // -- Symbolic facts: 200 chains x 6 symbols --
    // Scale calibration, measured: at 1KB of fact stream Fate births
    // nothing (natural segments run hundreds to thousands of bytes);
    // total facts must reach segment-length scale before units are
    // measurable. Hub contamination from cross-chain segments is left
    // for the false-pair rate to expose, not prettified in advance.
    let n_chains = 200usize;
    let chain_len = 6usize;
    let chains: Vec<Vec<String>> = (0..n_chains)
        .map(|c| (0..chain_len).map(|k| symbol(c * chain_len + k)).collect())
        .collect();
    let mut fact_pairs: Vec<(String, String)> = Vec::new();
    for ch in &chains {
        for w in ch.windows(2) {
            fact_pairs.push((w[0].clone(), w[1].clone()));
        }
    }
    // Shuffle fact order (fixed seed)
    for i in (1..fact_pairs.len()).rev() {
        let j = rng.below(i + 1);
        fact_pairs.swap(i, j);
    }
    // Test questions: transitive (dist 2..5, withheld) + false (cross-chain)
    let mut questions: Vec<(String, String, usize, bool)> = Vec::new(); // (a, b, dist, is_true)
    for ch in &chains {
        for i in 0..chain_len {
            for d in 2..chain_len - i {
                questions.push((ch[i].clone(), ch[i + d].clone(), d, true));
            }
        }
    }
    let mut qi = 0usize;
    while questions.iter().filter(|q| !q.3).count() < 48 && qi < 10000 {
        qi += 1;
        let a = symbol(rng.below(n_chains * chain_len));
        let b = symbol(rng.below(n_chains * chain_len));
        let ca = chains.iter().position(|ch| ch.contains(&a));
        let cb = chains.iter().position(|ch| ch.contains(&b));
        if ca != cb && !questions.iter().any(|q| q.0 == a && q.1 == b) {
            questions.push((a, b, 5, false));
        }
    }

    let mut body = Body::new();
    // Symbol-to-unit birth accounting (tail segments while feeding facts)
    let mut sym_tiles: HashMap<String, HashSet<String>> = HashMap::new();
    let mut sym_cells: HashMap<String, HashSet<String>> = HashMap::new();

    let test_at = |body: &Body,
                   sym_tiles: &HashMap<String, HashSet<String>>,
                   sym_cells: &HashMap<String, HashSet<String>>,
                   tag: &str| {
        // Storage channel: draw AAA>BBB edges from the bytes
        let bytes = body.stored_bytes();
        let mut edges: HashSet<(String, String)> = HashSet::new();
        for w in bytes.windows(7) {
            if w[3] == b'>' && w[..3].iter().all(|c| c.is_ascii_uppercase()) && w[4..].iter().all(|c| c.is_ascii_uppercase()) {
                edges.insert((
                    String::from_utf8_lossy(&w[..3]).into_owned(),
                    String::from_utf8_lossy(&w[4..]).into_owned(),
                ));
            }
        }
        let mut sadj: HashMap<String, Vec<String>> = HashMap::new();
        for (a, b) in &edges {
            sadj.entry(a.clone()).or_default().push(b.clone());
            sadj.entry(b.clone()).or_default().push(a.clone());
        }
        // Network channel: self-formed connection graph
        let mut tadj: HashMap<String, Vec<String>> = HashMap::new();
        for t in &body.tile.completed_tiles {
            for c in &t.connections {
                tadj.entry(t.id.clone()).or_default().push(c.clone());
                tadj.entry(c.clone()).or_default().push(t.id.clone());
            }
        }
        let mut cadj: HashMap<String, Vec<String>> = HashMap::new();
        for c in &body.stem.cells {
            for (n, _) in &c.connections {
                cadj.entry(c.cell_id.clone()).or_default().push(n.clone());
                cadj.entry(n.clone()).or_default().push(c.cell_id.clone());
            }
        }
        let score = |pick: &dyn Fn(&String, &String, usize) -> bool| -> (f64, f64) {
            let (mut tt, mut tn, mut ft, mut fn_) = (0usize, 0usize, 0usize, 0usize);
            for (a, b, d, is_true) in &questions {
                let hit = pick(a, b, *d);
                if *is_true {
                    tn += 1;
                    if hit { tt += 1; }
                } else {
                    fn_ += 1;
                    if !hit { ft += 1; }
                }
            }
            (
                if tn > 0 { tt as f64 / tn as f64 * 100.0 } else { 0.0 },
                if fn_ > 0 { ft as f64 / fn_ as f64 * 100.0 } else { 0.0 },
            )
        };
        // Unified budget 5 (full chain length - 1), same scale for true and
        // false questions - 2026-11-01 instrument correction: the old
        // version budgeted by true distance d, which leaked the label;
        // old absolute values are written off.
        let (s_true, s_false) = score(&|a, b, _d| {
            let to: HashSet<String> = [b.clone()].into_iter().collect();
            reachable(&sadj, std::slice::from_ref(a), &to, 5)
        });
        let net = |map: &HashMap<String, HashSet<String>>, adj: &HashMap<String, Vec<String>>| {
            score(&|a, b, _d| {
                let (Some(fa), Some(tb)) = (map.get(a), map.get(b)) else { return false };
                let from: Vec<String> = fa.iter().cloned().collect();
                reachable(adj, &from, tb, 5)
            })
        };
        let (t_true, t_false) = net(sym_tiles, &tadj);
        let (c_true, c_false) = net(sym_cells, &cadj);
        println!("[{tag}] Structural QA true/false pairs: storage {:.1}/{:.1} | Tile network {:.1}/{:.1} | Stem network {:.1}/{:.1} (questions {}, symbol map tile {} symbols stem {} symbols)",
            s_true, s_false, t_true, t_false, c_true, c_false, questions.len(), sym_tiles.len(), sym_cells.len());
    };

    // Egress C: external Fate QA. One react per hop (read-only,
    // deterministic; equation at rest during testing), same-symbol
    // results cached in-process. First-hop silence (react sends
    // nothing for chain-head symbols) is listed separately - that is
    // the interface having nothing to say, not a wrong answer.
    let react_at = |body: &Body, tag: &str| {
        let mut cache: HashMap<String, Vec<String>> = HashMap::new();
        let (mut tt, mut tn, mut ft, mut fn_) = (0usize, 0usize, 0usize, 0usize);
        let mut first_silent = 0usize;
        for (a, b, d, is_true) in &questions {
            let mut seen: HashSet<String> = HashSet::new();
            let mut frontier: Vec<String> = vec![a.clone()];
            seen.insert(a.clone());
            let mut hit = a == b;
            for _hop in 0..*d {
                if hit {
                    break;
                }
                let mut next: Vec<String> = Vec::new();
                for node in &frontier {
                    let ns = cache.entry(node.clone()).or_insert_with(|| react_neighbors(&body.global, node));
                    for n in ns {
                        if n == b {
                            hit = true;
                        }
                        if seen.insert(n.clone()) {
                            next.push(n.clone());
                        }
                    }
                }
                frontier = next;
                if frontier.is_empty() {
                    break;
                }
            }
            if *is_true {
                tn += 1;
                if hit {
                    tt += 1;
                }
                let first = cache.entry(a.clone()).or_insert_with(|| react_neighbors(&body.global, a));
                if first.is_empty() {
                    first_silent += 1;
                }
            } else {
                fn_ += 1;
                if !hit {
                    ft += 1;
                }
            }
        }
        let pct = |x: usize, n: usize| if n > 0 { x as f64 / n as f64 * 100.0 } else { 0.0 };
        println!("[{tag}] External Fate QA (react) true/false pairs: {:.1}/{:.1} (true first-hop silence {}, react-covered symbols {}, data_store {} entries)",
            pct(tt, tn), pct(ft, fn_), first_silent, cache.len(), body.global.stored_count());
    };

    let respond_at = |body: &mut Body, mats: &Vec<(&str, &Vec<u8>)>, holdout: usize, tag: &str| {
        for (name, bytes) in mats {
            if bytes.len() <= holdout {
                continue;
            }
            let tail = &bytes[bytes.len() - holdout..];
            let clen = body.stem.cells.len();
            let (sc, st) = body.feed_fateseg(tail);
            let segs: Vec<u64> = body.stem.cells[clen..].iter().map(|c| c.signal_count).collect();
            let mean = if segs.is_empty() { 0.0 } else { segs.iter().sum::<u64>() as f64 / segs.len() as f64 };
            println!("[{tag}·{name}] held-out {}KB -> Fate cut stem {} segments (mean {:.0}) tile {} segments", holdout / 1024, sc, mean, st);
            let _ = clen;
        }
    };

    // -- Training rounds --
    // Curriculum order: round 1 feeds all facts first (containers fresh, closure driven only by fact-byte
    // statistics; measured: feeding after natural material gives Stem zero closures in two batches, nothing measurable),
    // later rounds are pure natural material - round-10/20 QA reads whether the fact structure is washed out.
    for r in 1..=rounds {
        // Facts (round 1 only, fed whole before natural material): chains written as a text stream entering
        // byte-wise; storage still accounts per pair (AAA>BBB entries kept in the quads for the storage
        // channel to draw edges from). Birth accounting is segment-level: Fate does not close inside a single fact pair;
        // symbols queue as unborn, and whichever segment closes at birth registers all queued symbols
        // to the newborn unit (the segment content is exactly those chains).
        if r == 1 {
            let mut pend_cells: Vec<String> = Vec::new();
            let mut pend_tiles: Vec<String> = Vec::new();
            for ch in &chains {
                for w in ch.windows(2) {
                    let item = format!("{}>{}", w[0], w[1]).into_bytes();
                    body.tile.quad_keep(&item);
                    body.stem.quad_keep(&item);
                    // Wiring (the step bbea60a's react interface was missing): the
                    // same bytes are double-written to global Fate's
                    // data_store so the QA interface has originals to
                    // send. Quad accounting and Fate storage each keep
                    // one copy.
                    body.global.store_data(&item);
                }
                let line = format!("{}\n", ch.join(">"));
                let clen = body.stem.cells.len();
                let tlen = body.tile.completed_tiles.len();
                for &byte in line.as_bytes() {
                    let v = byte as f64;
                    body.global.receive_passive(v);
                    body.stem.receive_data(v);
                    body.tile.receive_data(v);
                }
                for s in ch {
                    pend_cells.push(s.clone());
                    pend_tiles.push(s.clone());
                }
                if body.stem.cells.len() > clen {
                    for c in &body.stem.cells[clen..] {
                        for s in &pend_cells {
                            sym_cells.entry(s.clone()).or_default().insert(c.cell_id.clone());
                        }
                    }
                    pend_cells.clear();
                }
                if body.tile.completed_tiles.len() > tlen {
                    for t in &body.tile.completed_tiles[tlen..] {
                        for s in &pend_tiles {
                            sym_tiles.entry(s.clone()).or_default().insert(t.id.clone());
                        }
                    }
                    pend_tiles.clear();
                }
            }
            println!("    Fact stream (round 1, fed whole first): symbol map tile {} symbols stem {} symbols (unclosed residue stem {} tile {})",
                sym_tiles.len(), sym_cells.len(), pend_cells.len(), pend_tiles.len());
        }
        let mut seg_line = Vec::new();
        for (name, bytes) in &mats {
            let usable = bytes.len().saturating_sub(holdout);
            if usable == 0 {
                continue;
            }
            let start = (r - 1) * chunk % usable;
            let end = (start + chunk).min(usable);
            let (sc, st) = body.feed_fateseg(&bytes[start..end]);
            seg_line.push(format!("{name} {sc}/{st}"));
        }
        for _ in 0..cycles {
            body.cycle();
        }
        body.fingerprint(&format!("round {r}"));
        println!("    new segments this round (stem/tile): {}", seg_line.join(" · "));
        if r == 10 || r == rounds {
            test_at(&body, &sym_tiles, &sym_cells, &format!("round{r}"));
            react_at(&body, &format!("round{r}"));
            if r == rounds {
                respond_at(&mut body, &mats, holdout, "respond");
            }
        }
    }
    body.fingerprint("final");
    println!("Full flow done: {} rounds x (Fate segmentation ingress + {} cycles), both egress channels measured.", rounds, cycles);
}
