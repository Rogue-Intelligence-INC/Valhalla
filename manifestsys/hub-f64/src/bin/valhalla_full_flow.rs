//! Valhalla 全流程（巩固版，2026-11-01）：入口—体内—出口一条程式。
//!
//! 入口（Fate 自动分割，原版设计）：一切料（多语文本/代码/图片/声音
//! 字节/符号事实）都逐字节走 Stem/Tile 的 receive_data——容器只收集，
//! Fate 用自身簇方程判收敛、命中收口成段；段长由方程与料的统计形态
//! 决定，喂方不预设粒度。存储纪律不变：原始字节全量入双侧四元记账
//! （自然料按 8KB 流块、事实按条），出生切分与存储粒度脱钩。
//! 体内：每轮 = 喂一批料 + 若干身体周期（tile/stem 周期、全局 Fate
//! tick、场沉降、propagate），与训练器同节律。
//! 出口（测试期，各通道分开记分，不混）：
//!   A 原生回应：留出料逐字节入场，看 Fate 如何切分、 newborn 落在
//!      场空间何处——回应形态不设限，只记录。
//!   B 结构问答（harness 读身体状态）：训练期喂过的符号链，测试期
//!      问未喂过的传递问题；存储通道=四元字节里抽边做限步可达，
//!      网络通道=Tile/Stem 自发连接上做限步可达（符号→单位映射
//!      用喂事实时的段级出生记账）。
//!   C 对外 Fate 问答（今晨 bbea60a 定稿的 react 接口才是问答
//!      接口）：喂事实时同字节双写全球 Fate 的 data_store（主线
//!      喂法此前只写四元、data_store 恒空，react 无话可送——缺的
//!      是这步接线）。测试期每跳调用一次 react、解析它送出的原文
//!      中的边；react 为 &self 只读：刺激不入内部 Fate、不 tick、
//!      不改方程（纪律由签名保证，问题不入体）。
//! 检查点：第 10 / 20 轮各测一次（同体继续训），读"多训十轮混料
//! 对事实结构是保是洗"。未纳入：vent 式抓取出口（D11 未立）。
//!
//! 环境变量：VH_ROUNDS（总轮数，默认 20）、VH_CHUNK_KB（每轮每料
//! KB，默认 128）、VH_CYCLES（每轮周期数，默认 2）、VH_SEED。
//! 运行方法见 docs/valhalla/FULL_FLOW_RUNBOOK.md。

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
    /// Fate 分割入口：存储记账（流块）+ 逐字节收口出生。
    /// 返回（stem 新生段数，tile 新生段数）。
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
            "{tag} tiles {} cells {} stem_links {} 存储 {} 方程 {}",
            self.tile.completed_tiles.len(),
            self.stem.cells.len(),
            links,
            self.stored_total(),
            self.global.cluster_equation.equation_str()
        );
    }
}

/// 对外 Fate 一跳：以符号为刺激调 react，解析它送出的原文中的边，
/// 返回边两端符号。react 无命中即空——沉默如实呈现为无邻居。
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

/// 限步可达（无向，预算=步数）
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
    // 大身体周期含深递归/大帧，与 probes 同规：大栈线程跑主逻辑。
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

    // ── 料源（自然料四类）──
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
    let mats: Vec<(&str, &Vec<u8>)> = vec![("多语文本", &text), ("代码", &code), ("图片", &img), ("声音", &aud)];
    println!("料源: {}", mats.iter().map(|(n, b)| format!("{n} {}KB", b.len() / 1024)).collect::<Vec<_>>().join(" / "));
    // 留出尾（回应测试专用，训练轮不碰）：每料最后 128KB
    let holdout = 128 * 1024usize;

    // ── 符号事实：200 链 × 6 符号 ──
    // 规模定标实测依据：事实流 1KB 时 Fate 零出生（自然段长数百至
    // 数千字节）；事实总量须达段长量级以上才有单位可测。段跨链
    // 产生的枢纽污染由假题对率负责暴露，不预先粉饰。
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
    // 打乱事实顺序（固定种子）
    for i in (1..fact_pairs.len()).rev() {
        let j = rng.below(i + 1);
        fact_pairs.swap(i, j);
    }
    // 测试题：传递（dist 2..5，未喂）+ 假题（跨链）
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
    // 符号→单位出生记账（喂事实期间的尾段）
    let mut sym_tiles: HashMap<String, HashSet<String>> = HashMap::new();
    let mut sym_cells: HashMap<String, HashSet<String>> = HashMap::new();

    let test_at = |body: &Body,
                   sym_tiles: &HashMap<String, HashSet<String>>,
                   sym_cells: &HashMap<String, HashSet<String>>,
                   tag: &str| {
        // 存储通道：字节里抽 AAA>BBB 边
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
        // 网络通道：自发连接图
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
        let (s_true, s_false) = score(&|a, b, d| {
            let to: HashSet<String> = [b.clone()].into_iter().collect();
            reachable(&sadj, std::slice::from_ref(a), &to, d)
        });
        let net = |map: &HashMap<String, HashSet<String>>, adj: &HashMap<String, Vec<String>>| {
            score(&|a, b, d| {
                let (Some(fa), Some(tb)) = (map.get(a), map.get(b)) else { return false };
                let from: Vec<String> = fa.iter().cloned().collect();
                reachable(adj, &from, tb, d)
            })
        };
        let (t_true, t_false) = net(sym_tiles, &tadj);
        let (c_true, c_false) = net(sym_cells, &cadj);
        println!("[{tag}] 结构问答 真题/假题对: 存储 {:.1}/{:.1} | Tile网络 {:.1}/{:.1} | Stem网络 {:.1}/{:.1}（题量 {}，符号映射 tile {} 符号 stem {} 符号）",
            s_true, s_false, t_true, t_false, c_true, c_false, questions.len(), sym_tiles.len(), sym_cells.len());
    };

    // 出口 C：对外 Fate 问答。每跳一次 react（只读、确定；测试期
    // 方程静止），同符号结果进程内缓存。首跳沉默（react 对链首
    // 符号无任何送出）单列——那是接口无话，不是答错。
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
        println!("[{tag}] 对外Fate问答(react) 真题/假题对: {:.1}/{:.1}（真题首跳沉默 {}，react 覆盖符号 {}，data_store {} 条）",
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
            println!("[{tag}·{name}] 留出 128KB -> Fate 切 stem {} 段（均 {:.0}）tile {} 段", sc, mean, st);
            let _ = clen;
        }
    };

    // ── 训练轮 ──
    // 课程顺序：第 1 轮先行整批喂事实（容器新鲜，收口只由事实字节
    // 统计驱动；实测在自然料之后喂时 Stem 两批零收口、测不到），
    // 其后各轮纯自然料——第 10/20 轮问答读"事实结构被冲洗与否"。
    for r in 1..=rounds {
        // 事实（仅第 1 轮，先于自然料整批喂完）：链连写成文本流逐
        // 字节入场；存储仍按对记账（四元里留 AAA>BBB 条目供存储
        // 通道抽边）。出生记账段级化：Fate 不在单个事实对内收口，
        // 符号先入待出生队列，哪段收口出生就把队列中全部符号记到
        // 新生单位名下（段内容即这些链）。
        if r == 1 {
            let mut pend_cells: Vec<String> = Vec::new();
            let mut pend_tiles: Vec<String> = Vec::new();
            for ch in &chains {
                for w in ch.windows(2) {
                    let item = format!("{}>{}", w[0], w[1]).into_bytes();
                    body.tile.quad_keep(&item);
                    body.stem.quad_keep(&item);
                    // 接线（bbea60a 的 react 接口此前缺这一步）：同
                    // 字节双写全球 Fate 的 data_store，问答接口才
                    // 有原文可送。四元记账与 Fate 存储各记一份。
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
            println!("    事实流（第 1 轮先行整批）：符号映射 tile {} 符号 stem {} 符号（残留未收口 stem {} tile {}）",
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
        body.fingerprint(&format!("第 {r} 轮"));
        println!("    本轮新生段（stem/tile）: {}", seg_line.join(" · "));
        if r == 10 || r == rounds {
            test_at(&body, &sym_tiles, &sym_cells, &format!("第{r}轮"));
            react_at(&body, &format!("第{r}轮"));
            if r == rounds {
                respond_at(&mut body, &mats, holdout, "回应");
            }
        }
    }
    body.fingerprint("终局");
    println!("全流程毕：{} 轮 ×（Fate 分割入口 + {} 周期），出口双通道已测。", rounds, cycles);
}
