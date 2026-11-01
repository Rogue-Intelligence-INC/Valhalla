//! 证据探针（仅测试侧工具，库零改动）：四种模式由 VH_PROBE 选择。
//!   org        : VH_LOAD_DIR 解冻一具身体，打印自发网络组织统计
//!   order      : 同批料 A(文本)->B(代码) 与 B->A 两序训练，比终局方程（顺序留痕）
//!   domain     : 纯文本 vs 纯代码各训 24MB，比方程（选料定向对照版）
//!   assimilate : VH_LOAD_DIR 解冻身体 vs 空身体，同喂留出料（第 5 份分片，
//!                曲线训练从未见过），比同化期出生/合并/死亡/分裂
//!   vent       : VH_LOAD_DIR 解冻身体，刺激只经 tile.propagate 扰动场，连跑
//!                若干拍后把回应最强的单位送出（出口通道 v1，不查找不匹配）
//!   vent2      : 双生对照（静默 vs 受刺激同拍相减），测刺激特异响应——
//!                对外 Fate 将来抓取的对象；先验内部能否正确响应
//!   vent3      : 两套规则版——Tile 走 propagate/激活织物、Stem 走
//!                ingest+promote+settle_field/位置场，各层各尺；刺激取
//!                语料原流不手选，扰动程度以实测响应（对静默涨落）报告
//!   vent4      : 混沌对照——M1 / M1翻一字节 / M1乱序，逐拍追 Stem 场
//!                位移分叉增长，判别"结构性听觉"与"蝴蝶效应"
//! Body/feed/cycle 与训练器逐行一致的整体喂法；喂法不存在绕过存储的路径。

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use hub_f64::monitor::BodyMonitor;
use hub_f64::{BaseForceCoordinator, GravityField, MeshBrainSystem, MultinovaProcessor, TripleFateCombined};
use stem_cell::stemcell_complete::CellCycleResult;
use stem_cell::StemCellComplete;
use valhalla_tile_quattro::tile_complete::TileCycleResult;
use valhalla_tile_quattro::TileQuattroComplete;

fn env_path(key: &str) -> Option<PathBuf> {
    std::env::var(key).ok().filter(|v| !v.trim().is_empty()).map(PathBuf::from)
}
fn env_usize(key: &str, default: usize) -> usize {
    std::env::var(key).ok().and_then(|v| v.parse().ok()).filter(|&v| v > 0).unwrap_or(default)
}

struct Body {
    tile: TileQuattroComplete,
    stem: StemCellComplete,
    monitor: BodyMonitor,
    global: TripleFateCombined,
}

#[derive(Default, Clone, Copy)]
struct Cyc {
    t_merges: u64,
    t_splits: u64,
    t_deaths: u64,
    s_merges: u64,
    s_div: u64,
    s_deaths: u64,
}
impl Cyc {
    fn add(&mut self, t: &TileCycleResult, s: &CellCycleResult) {
        self.t_merges += t.merges as u64;
        self.t_splits += t.splits as u64;
        self.t_deaths += t.deaths as u64;
        self.s_merges += s.merges as u64;
        self.s_div += s.divisions as u64;
        self.s_deaths += s.deaths as u64;
    }
    fn line(&self) -> String {
        format!(
            "tile合并 {} 分裂 {} 死亡 {} | stem合并 {} 分裂 {} 死亡 {}",
            self.t_merges, self.t_splits, self.t_deaths, self.s_merges, self.s_div, self.s_deaths
        )
    }
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
    fn cycle(&mut self) -> (TileCycleResult, CellCycleResult) {
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
        (t, s)
    }
    /// 喂一整股字节流：8KB 切片、每 256 片一中周期（与训练器同节律）。
    fn feed_stream(&mut self, bytes: &[u8], acc: &mut Cyc) {
        let mut n = 0usize;
        for chunk in bytes.chunks(8 * 1024) {
            self.feed(chunk);
            n += 1;
            if n % 256 == 0 {
                let (t, s) = self.cycle();
                acc.add(&t, &s);
            }
        }
    }
    /// 收口：跑周期直到合并归零（上限 20），与训练器收紧同规。
    fn settle(&mut self, acc: &mut Cyc) {
        for _ in 0..20 {
            let (t, s) = self.cycle();
            let (tm, sm) = (t.merges, s.merges);
            acc.add(&t, &s);
            if tm == 0 && sm == 0 {
                break;
            }
        }
    }
    fn load_full(dir: &Path) -> std::io::Result<Self> {
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
        body.tile.blackhole = GravityField::load_store(&dir.join("tile.blackhole.store"))?;
        body.tile.meshbrain = MeshBrainSystem::load_store(&dir.join("tile.meshbrain.store"))?;
        body.tile.multinova = MultinovaProcessor::load_store(&dir.join("tile.multinova.store"))?;
        body.tile.baseforce = BaseForceCoordinator::load_store(&dir.join("tile.baseforce.store"))?;
        body.stem.blackhole = GravityField::load_store(&dir.join("stem.blackhole.store"))?;
        body.stem.meshbrain = MeshBrainSystem::load_store(&dir.join("stem.meshbrain.store"))?;
        body.stem.multinova = MultinovaProcessor::load_store(&dir.join("stem.multinova.store"))?;
        body.stem.baseforce = BaseForceCoordinator::load_store(&dir.join("stem.baseforce.store"))?;
        Ok(body)
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
    /// 自发网络组织统计：度分布 / 枢纽集中度 / 最大弱连通块 / 强度离散。
    fn org_report(&self, tag: &str) {
        // Tile 图
        let n = self.tile.completed_tiles.len();
        let mut deg: Vec<usize> = self.tile.completed_tiles.iter().map(|t| t.connections.len()).collect();
        let edges: usize = deg.iter().sum();
        deg.sort_unstable_by(|a, b| b.cmp(a));
        let top1 = deg.iter().take((n / 100).max(1)).sum::<usize>();
        // 弱连通块（并查集）
        let idx: HashMap<&str, usize> = self
            .tile
            .completed_tiles
            .iter()
            .enumerate()
            .map(|(i, t)| (t.id.as_str(), i))
            .collect();
        let mut parent: Vec<usize> = (0..n).collect();
        fn find(p: &mut [usize], mut x: usize) -> usize {
            while p[x] != x {
                p[x] = p[p[x]];
                x = p[x];
            }
            x
        }
        for t in &self.tile.completed_tiles {
            let a = idx[t.id.as_str()];
            for c in &t.connections {
                if let Some(&b) = idx.get(c.as_str()) {
                    let (ra, rb) = (find(&mut parent, a), find(&mut parent, b));
                    if ra != rb {
                        parent[ra] = rb;
                    }
                }
            }
        }
        let mut blocks: HashMap<usize, usize> = HashMap::new();
        for i in 0..n {
            let r = find(&mut parent, i);
            *blocks.entry(r).or_default() += 1;
        }
        let largest = blocks.values().max().copied().unwrap_or(0);
        let iso = deg.iter().filter(|&&d| d == 0).count();
        println!(
            "{tag} TILE n {n} 边 {edges} 均度 {:.2} 最大度 {} 孤立 {}({:.1}%) 最大块 {}({:.1}%) 块数 {} Top1%枢纽占边 {:.1}%",
            if n > 0 { edges as f64 / n as f64 } else { 0.0 },
            deg.first().copied().unwrap_or(0),
            iso,
            if n > 0 { iso as f64 / n as f64 * 100.0 } else { 0.0 },
            largest,
            if n > 0 { largest as f64 / n as f64 * 100.0 } else { 0.0 },
            blocks.len(),
            if edges > 0 { top1 as f64 / edges as f64 * 100.0 } else { 0.0 }
        );
        // Stem 图
        let m = self.stem.cells.len();
        let mut cdeg: Vec<usize> = self.stem.cells.iter().map(|c| c.connections.len()).collect();
        let links: usize = cdeg.iter().sum();
        cdeg.sort_unstable_by(|a, b| b.cmp(a));
        let ctop1 = cdeg.iter().take((m / 100).max(1)).sum::<usize>();
        let strengths: Vec<f64> = self.stem.cells.iter().flat_map(|c| c.connections.iter().map(|(_, s)| *s)).collect();
        let mean = if links > 0 { strengths.iter().sum::<f64>() / links as f64 } else { 0.0 };
        let var = if links > 0 {
            strengths.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / links as f64
        } else {
            0.0
        };
        let ciso = cdeg.iter().filter(|&&d| d == 0).count();
        println!(
            "{tag} STEM n {m} 边 {links} 均度 {:.2} 最大度 {} 孤立 {}({:.1}%) Top1%枢纽占边 {:.1}% 强度均 {:.3} CV {:.2} 最大 {:.1}",
            if m > 0 { links as f64 / m as f64 } else { 0.0 },
            cdeg.first().copied().unwrap_or(0),
            ciso,
            if m > 0 { ciso as f64 / m as f64 * 100.0 } else { 0.0 },
            if links > 0 { ctop1 as f64 / links as f64 * 100.0 } else { 0.0 },
            mean,
            if mean != 0.0 { var.sqrt() / mean } else { 0.0 },
            strengths.iter().fold(0.0f64, |a, &b| a.max(b))
        );
    }
}

// ---------- 语料 ----------
fn roots() -> Vec<PathBuf> {
    let base = Path::new(r"C:\Ai\rogue-products\valhalla-self-incubate");
    vec![base.join("corpus"), base.join("corpus_new"), base.join("corpus_diverse")]
}
fn keep(path: &Path) -> bool {
    if path.components().any(|c| c.as_os_str() == "Annotations") {
        return false;
    }
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("txt" | "json" | "jsonl" | "csv" | "tsv" | "rs" | "md" | "py" | "go" | "js" | "java" | "c" | "h" | "cpp" | "ts")
    )
}
fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, out);
        } else if keep(&path) {
            out.push(path);
        }
    }
}
fn all_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    for root in roots() {
        collect(&root, &mut files);
    }
    files
}
/// 从文件表顺序读出一股字节流（确定性序：路径排序后依次截取）。
fn read_stream(files: &[PathBuf], budget: usize) -> Vec<u8> {
    let mut sorted: Vec<PathBuf> = files.to_vec();
    sorted.sort();
    let mut out = Vec::with_capacity(budget);
    for p in &sorted {
        if out.len() >= budget {
            break;
        }
        if let Ok(f) = File::open(p) {
            let remain = (budget - out.len()) as u64;
            let _ = f.take(remain).read_to_end(&mut out);
        }
    }
    out
}
fn is_code(p: &Path) -> bool {
    matches!(
        p.extension().and_then(|e| e.to_str()),
        Some("rs" | "py" | "go" | "js" | "java" | "c" | "h" | "cpp" | "ts")
    )
}
fn is_text(p: &Path) -> bool {
    matches!(p.extension().and_then(|e| e.to_str()), Some("txt" | "md"))
}
/// 与训练器完全相同的 5 份贪心分片；留出料取第 5 份（曲线训练只喂过第 1 份）。
fn heldout_files(files: &[PathBuf]) -> Vec<PathBuf> {
    let mut sized: Vec<(u64, PathBuf)> = files
        .iter()
        .map(|p| (std::fs::metadata(p).map(|m| m.len()).unwrap_or(0), p.clone()))
        .collect();
    sized.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    let mut shards: Vec<Vec<PathBuf>> = vec![Vec::new(); 5];
    let mut shard_bytes = vec![0u64; 5];
    for (size, path) in sized {
        let lightest = (0..5).min_by_key(|&i| shard_bytes[i]).unwrap_or(0);
        shards[lightest].push(path);
        shard_bytes[lightest] += size;
    }
    shards[4].clone()
}

fn main() {
    let mode = std::env::var("VH_PROBE").unwrap_or_else(|_| "org".to_string());
    match mode.as_str() {
        "org" => {
            let dir = env_path("VH_LOAD_DIR").expect("org 模式需要 VH_LOAD_DIR");
            let t = std::time::Instant::now();
            match Body::load_full(&dir) {
                Ok(b) => {
                    println!("解冻 {} 用时 {:.0}s", dir.display(), t.elapsed().as_secs_f64());
                    b.fingerprint("ORG");
                    b.org_report("ORG");
                }
                Err(e) => println!("解冻失败 {e}"),
            }
        }
        "order" | "domain" => {
            let files = all_files();
            let mb = env_usize("VH_PROBE_MB", 12) * 1024 * 1024;
            let text = read_stream(&files.iter().filter(|p| is_text(p)).cloned().collect::<Vec<_>>(), mb);
            let code = read_stream(&files.iter().filter(|p| is_code(p)).cloned().collect::<Vec<_>>(), mb);
            println!("料: 文本 {} MB / 代码 {} MB", text.len() / 1_000_000, code.len() / 1_000_000);
            if mode == "order" {
                let mut ab = Body::new();
                let mut acc = Cyc::default();
                ab.feed_stream(&text, &mut acc);
                ab.feed_stream(&code, &mut acc);
                ab.settle(&mut acc);
                ab.fingerprint("顺序 文本->代码");
                println!("  {}", acc.line());
                let mut ba = Body::new();
                let mut acc = Cyc::default();
                ba.feed_stream(&code, &mut acc);
                ba.feed_stream(&text, &mut acc);
                ba.settle(&mut acc);
                ba.fingerprint("顺序 代码->文本");
                println!("  {}", acc.line());
            } else {
                let mut t = Body::new();
                let mut acc = Cyc::default();
                t.feed_stream(&text, &mut acc);
                t.settle(&mut acc);
                t.fingerprint("纯文本");
                println!("  {}", acc.line());
                let mut c = Body::new();
                let mut acc = Cyc::default();
                c.feed_stream(&code, &mut acc);
                c.settle(&mut acc);
                c.fingerprint("纯代码");
                println!("  {}", acc.line());
            }
        }
        "assimilate" => {
            let dir = env_path("VH_LOAD_DIR").expect("assimilate 需要 VH_LOAD_DIR");
            let mb = env_usize("VH_PROBE_MB", 12) * 1024 * 1024;
            let files = all_files();
            let held = heldout_files(&files);
            let bytes = read_stream(&held, mb);
            println!("留出料 {} MB（第 5 份分片 {} 个文件）", bytes.len() / 1_000_000, held.len());
            let run = |mut body: Body, tag: &str| {
                body.fingerprint(&format!("{tag} 同化前"));
                let tiles0: HashSet<String> = body.tile.completed_tiles.iter().map(|t| t.id.clone()).collect();
                let cells0: HashSet<String> = body.stem.cells.iter().map(|c| c.cell_id.clone()).collect();
                let mut seen_t = tiles0.clone();
                let mut seen_c = cells0.clone();
                let mut born_t = 0usize;
                let mut born_c = 0usize;
                let mut acc = Cyc::default();
                let mut n = 0usize;
                for chunk in bytes.chunks(8 * 1024) {
                    body.feed(chunk);
                    n += 1;
                    for t in &body.tile.completed_tiles {
                        if seen_t.insert(t.id.clone()) {
                            born_t += 1;
                        }
                    }
                    for c in &body.stem.cells {
                        if seen_c.insert(c.cell_id.clone()) {
                            born_c += 1;
                        }
                    }
                    if n % 256 == 0 {
                        let (t, s) = body.cycle();
                        acc.add(&t, &s);
                    }
                }
                body.settle(&mut acc);
                println!("{tag} 同化中新出生 tiles {born_t} cells {born_c}（喂前 {}/{}）", tiles0.len(), cells0.len());
                println!("{tag} 同化周期 {}", acc.line());
                body.fingerprint(&format!("{tag} 同化后"));
            };
            match Body::load_full(&dir) {
                Ok(b) => run(b, "预组织(解冻)"),
                Err(e) => println!("解冻失败 {e}"),
            }
            run(Body::new(), "空身体");
        }
        "cycle" => {
            // 死亡机制实测：解冻身体不喂料连跑 N 周期，看死亡到底发不发生，
            // 并量"原始能量已低于死亡线、靠回流续命"的存量（尸体口径）。
            let dir = env_path("VH_LOAD_DIR").expect("cycle 需要 VH_LOAD_DIR");
            let rounds = env_usize("VH_PROBE_CYCLES", 20);
            match Body::load_full(&dir) {
                Ok(mut body) => {
                    let es: Vec<f64> = body.stem.cells.iter().map(|c| c.energy).collect();
                    let avg = if es.is_empty() { 0.0 } else { es.iter().sum::<f64>() / es.len() as f64 };
                    let below = es.iter().filter(|&&e| e < avg * 0.1 || e < 0.01).count();
                    let emin = es.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                    let tes: Vec<f64> = body.tile.completed_tiles.iter().map(|t| t.energy).collect();
                    let tavg = if tes.is_empty() { 0.0 } else { tes.iter().sum::<f64>() / tes.len() as f64 };
                    let tbelow = tes.iter().filter(|&&e| e < tavg * 0.1 || e < 0.01).count();
                    println!(
                        "能量存量 stem avg {avg:.4} min {emin:.4} 低于死亡线 {below}/{} ({:.2}%) | tile avg {tavg:.4} 低于线 {tbelow}/{}",
                        es.len(),
                        if es.is_empty() { 0.0 } else { below as f64 / es.len() as f64 * 100.0 },
                        tes.len()
                    );
                    body.fingerprint("空转前");
                    let mut acc = Cyc::default();
                    for _ in 0..rounds {
                        let (t, s) = body.cycle();
                        acc.add(&t, &s);
                    }
                    println!("不喂料空转 {rounds} 周期 {}", acc.line());
                    body.fingerprint("空转后");
                }
                Err(e) => println!("解冻失败 {e}"),
            }
        }
        "vent" => {
            // 出口通道 v1：刺激只经 tile.propagate 扰动场（库里既有的交互入口），
            // 身体自跑若干拍（不 global.tick、不 feed、不写存储），回应最强的
            // 单位把自己的状态送出。不查找、不匹配、不设质量门；先让它乱送。
            let dir = env_path("VH_LOAD_DIR").expect("vent 需要 VH_LOAD_DIR");
            let beats = env_usize("VH_VENT_BEATS", 6);
            let stim_a = std::env::var("VH_STIMULUS").unwrap_or_else(|_| "法国的首都是巴黎".to_string());
            let arms: Vec<(&str, Option<Vec<u8>>)> = vec![
                ("静默", None),
                ("刺激A", Some(stim_a.into_bytes())),
                ("刺激B", Some(b"fn main() { let x = 10 + 30; println!(\"{}\", x); }".to_vec())),
            ];
            let seal = |b: &Body| {
                (
                    b.global.global_tick,
                    b.global.cluster_equation.equation_str(),
                    b.stored_total(),
                    b.tile.triple_fate.global_tick,
                    b.stem.triple_fate.global_tick,
                )
            };
            let mut tile_tops: Vec<Vec<usize>> = Vec::new();
            for (tag, stim) in &arms {
                let mut body = match Body::load_full(&dir) {
                    Ok(b) => b,
                    Err(e) => {
                        println!("解冻失败 {e}");
                        return;
                    }
                };
                let seal0 = seal(&body);
                let (nt0, nc0) = (body.tile.completed_tiles.len(), body.stem.cells.len());
                let cbytes: usize = body.tile.completed_tiles.iter().map(|t| t.content.len()).sum::<usize>()
                    + body.stem.cells.iter().map(|c| c.content.len()).sum::<usize>();
                let act0: Vec<f64> = body.tile.completed_tiles.iter().map(|t| t.activation).collect();
                let eng0: Vec<f64> = body.stem.cells.iter().map(|c| c.energy).collect();
                let sig = stim.as_ref().map(|s| s.iter().map(|b| *b as f64).sum::<f64>() / s.len().max(1) as f64);
                let mut new_conns = 0usize;
                for _ in 0..beats {
                    if let Some(s) = sig {
                        new_conns += body.tile.propagate(s).new_connections;
                    }
                    let _ = body.tile.process_tiles_cycle();
                    let _ = body.stem.process_cells_cycle();
                    body.stem.settle_field();
                }
                let seal1 = seal(&body);
                let mut td: Vec<(usize, f64)> = body
                    .tile
                    .completed_tiles
                    .iter()
                    .enumerate()
                    .map(|(i, t)| (i, (t.activation - act0.get(i).copied().unwrap_or(0.0)).abs()))
                    .collect();
                td.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                let mut sd: Vec<(usize, f64)> = body
                    .stem
                    .cells
                    .iter()
                    .enumerate()
                    .map(|(i, c)| (i, (c.energy - eng0.get(i).copied().unwrap_or(0.0)).abs()))
                    .collect();
                sd.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                let tmoved = td.iter().filter(|x| x.1 > 0.01).count();
                let smoved = sd.iter().filter(|x| x.1 > 0.01).count();
                println!(
                    "[{tag}] 信号 {:?} 拍 {beats} 新连接 {new_conns} | 单位 {}->{} / {}->{} | content总字节 {cbytes} | 动>0.01 tile {tmoved} stem {smoved}",
                    sig.map(|s| format!("{s:.1}")),
                    nt0,
                    body.tile.completed_tiles.len(),
                    nc0,
                    body.stem.cells.len()
                );
                println!(
                    "  纪律 全局tick {}->{} 方程 {} 存储 {}->{} 内层tick tile {}->{} stem {}->{}",
                    seal0.0, seal1.0, seal0.1, seal0.2, seal1.2, seal0.3, seal1.3, seal0.4, seal1.4
                );
                println!("  出口 tile 回应最强:");
                for (i, d) in td.iter().take(8) {
                    let t = &body.tile.completed_tiles[*i];
                    println!(
                        "    {} Δact {:.3} act {:.3} 能量 {:.2} 度 {} sig {:?} content {} 字节",
                        t.id, d, t.activation, t.energy, t.connections.len(), t.data_signature, t.content.len()
                    );
                }
                println!("  出口 stem 回应最强:");
                for (i, d) in sd.iter().take(8) {
                    let c = &body.stem.cells[*i];
                    println!(
                        "    {} ΔE {:.3} E {:.3} 场 {:.3} 度 {} content {} 字节",
                        c.cell_id, d, c.energy, c.field_strength, c.connections.len(), c.content.len()
                    );
                }
                tile_tops.push(td.iter().take(12).map(|x| x.0).collect());
            }
            if tile_tops.len() == 3 {
                let ov = |a: &[usize], b: &[usize]| a.iter().filter(|x| b.contains(x)).count();
                println!(
                    "tile 前12回应重叠 静默∩A {} 静默∩B {} A∩B {}（共12）",
                    ov(&tile_tops[0], &tile_tops[1]),
                    ov(&tile_tops[0], &tile_tops[2]),
                    ov(&tile_tops[1], &tile_tops[2])
                );
            }
        }
        "vent2" => {
            // 出口 v2：双生对照。确定性已证（A10），同盘解冻的身体同拍运行，
            // 静默具作反事实基线，相减扣掉周期固定节目，剩下刺激特异响应——
            // 即对外 Fate 将来自然抓取的对象。本模式只量"内部能否正确响应"。
            let dir = env_path("VH_LOAD_DIR").expect("vent2 需要 VH_LOAD_DIR");
            let beats = env_usize("VH_VENT_BEATS", 6);
            let stim_a = std::env::var("VH_STIMULUS").unwrap_or_else(|_| "法国的首都是巴黎".to_string());
            type Snap = (HashMap<String, f64>, HashSet<(String, String)>, HashMap<String, f64>);
            let snap = |b: &Body| -> Snap {
                let mut act: HashMap<String, f64> = HashMap::new();
                let mut edges: HashSet<(String, String)> = HashSet::new();
                for t in &b.tile.completed_tiles {
                    act.entry(t.id.clone()).or_insert(t.activation);
                    for c in &t.connections {
                        edges.insert((t.id.clone(), c.clone()));
                    }
                }
                let mut eng: HashMap<String, f64> = HashMap::new();
                for c in &b.stem.cells {
                    eng.entry(c.cell_id.clone()).or_insert(c.energy);
                }
                (act, edges, eng)
            };
            fn twin_beat(b: &mut Body, sig: Option<f64>, beats: usize) -> usize {
                let mut nc = 0usize;
                for _ in 0..beats {
                    if let Some(s) = sig {
                        nc += b.tile.propagate(s).new_connections;
                    }
                    let _ = b.tile.process_tiles_cycle();
                    let _ = b.stem.process_cells_cycle();
                    b.stem.settle_field();
                }
                nc
            }
            let mean_sig = |s: &[u8]| s.iter().map(|b| *b as f64).sum::<f64>() / s.len().max(1) as f64;
            let load = |dir: &Path| -> Body {
                match Body::load_full(dir) {
                    Ok(b) => b,
                    Err(e) => {
                        println!("解冻失败 {e}");
                        std::process::exit(1);
                    }
                }
            };
            // 静默双生：验仪器（同盘两具身体同拍，差应为零）
            let mut s1 = load(&dir);
            let mut s2 = load(&dir);
            let _ = twin_beat(&mut s1, None, beats);
            let _ = twin_beat(&mut s2, None, beats);
            let (r_act, r_edges, r_eng) = snap(&s1);
            let (_, s2_edges, _) = snap(&s2);
            let sanity = {
                let (_, a, _) = snap(&s2);
                let _ = a;
                s2_edges.symmetric_difference(&r_edges).count()
            };
            println!("仪器自检 静默双生边集差 {sanity}（应为 0）");
            let diff_report = |tag: &str, b: &Body, nc: usize| -> Vec<String> {
                let (act, edges, eng) = snap(b);
                let mut td: Vec<(&String, f64, f64)> = act
                    .iter()
                    .filter_map(|(id, a)| r_act.get(id).map(|r| (id, *a - *r, *a)))
                    .collect();
                td.sort_by(|x, y| y.1.abs().partial_cmp(&x.1.abs()).unwrap_or(std::cmp::Ordering::Equal));
                let mut sd: Vec<(&String, f64, f64)> = eng
                    .iter()
                    .filter_map(|(id, e)| r_eng.get(id).map(|r| (id, *e - *r, *e)))
                    .collect();
                sd.sort_by(|x, y| y.1.abs().partial_cmp(&x.1.abs()).unwrap_or(std::cmp::Ordering::Equal));
                let only_arm = edges.difference(&r_edges).count();
                let only_ref = r_edges.difference(&edges).count();
                println!(
                    "[{tag}] 新连接 {nc} | 边 仅刺激臂 {only_arm} 仅静默 {only_ref} | tile差分Top:"
                );
                let by_id: HashMap<&str, &valhalla_tile_quattro::tile_complete::CompletedTile> =
                    b.tile.completed_tiles.iter().map(|t| (t.id.as_str(), t)).collect();
                for (id, d, a) in td.iter().take(8) {
                    let deg = by_id.get(id.as_str()).map(|t| t.connections.len()).unwrap_or(0);
                    println!("    {id} Δact {d:+.3} act {a:.3} 度 {deg}");
                }
                println!("  stem差分Top:");
                for (id, d, e) in sd.iter().take(5) {
                    println!("    {id} ΔE {d:+.3} E {e:.3}");
                }
                println!(
                    "  纪律 全局tick {} 方程 {} 存储 {}",
                    b.global.global_tick,
                    b.global.cluster_equation.equation_str(),
                    b.stored_total()
                );
                td.iter().take(12).map(|x| x.0.clone()).collect()
            };
            let mut a_body = load(&dir);
            let nc_a = twin_beat(&mut a_body, Some(mean_sig(stim_a.as_bytes())), beats);
            let top_a = diff_report("刺激A", &a_body, nc_a);
            let stim_b: &[u8] = b"fn main() { let x = 10 + 30; println!(\"{}\", x); }";
            let mut b_body = load(&dir);
            let nc_b = twin_beat(&mut b_body, Some(mean_sig(stim_b)), beats);
            let top_b = diff_report("刺激B", &b_body, nc_b);
            let ov = top_a.iter().filter(|x| top_b.contains(x)).count();
            println!("A/B tile 差分前12重叠 {ov}/12（低=分化，高=没分化）");
        }
        "vent3" => {
            // 出口 v3：两套规则。Tile 规则=propagate 激活织物（读 Δact/边）；
            // Stem 规则=ingest+promote+settle_field 位置场（读 Δpos/场强）。
            // 刺激取语料原流（不手选文本、不定标量），两层各以原生节律入场；
            // 扰动程度以实测响应幅度（对静默双生涨落之差）报告。
            // 不 quad_keep、不 global.tick，存储与全局方程不受扰。
            let dir = env_path("VH_LOAD_DIR").expect("vent3 需要 VH_LOAD_DIR");
            let beats = env_usize("VH_VENT_BEATS", 6);
            let kb = env_usize("VH_VENT_KB", 256);
            let files = all_files();
            let stream = read_stream(&files, 2 * kb * 1024);
            let m1: Vec<u8> = stream.iter().take(kb * 1024).cloned().collect();
            let m2: Vec<u8> = stream.iter().skip(kb * 1024).take(kb * 1024).cloned().collect();
            // 对照：M1 切片乱序（同批材料，破顺序）——确定性伪随机交织
            let mut m3: Vec<u8> = Vec::with_capacity(m1.len());
            {
                let chunks: Vec<&[u8]> = m1.chunks(8 * 1024).collect();
                let mut idx: Vec<usize> = (0..chunks.len()).collect();
                let mut s: u64 = 0x9E3779B97F4A7C15;
                for i in (1..idx.len()).rev() {
                    s ^= s << 13;
                    s ^= s >> 7;
                    s ^= s << 17;
                    idx.swap(i, (s % (i as u64 + 1)) as usize);
                }
                for i in idx {
                    m3.extend_from_slice(chunks[i]);
                }
            }
            println!("料 M1 {} KB / M2 {} KB / M3(M1乱序) {} KB", m1.len() / 1024, m2.len() / 1024, m3.len() / 1024);
            #[derive(Clone)]
            struct V3Snap {
                t_act: HashMap<String, f64>,
                t_edges: HashSet<(String, String)>,
                s_pos: HashMap<String, Vec<f64>>,
                s_field: HashMap<String, f64>,
                s_links: usize,
                tiles: usize,
                cells: usize,
            }
            let snap3 = |b: &Body| -> V3Snap {
                let mut t_act: HashMap<String, f64> = HashMap::new();
                let mut t_edges: HashSet<(String, String)> = HashSet::new();
                for t in &b.tile.completed_tiles {
                    t_act.entry(t.id.clone()).or_insert(t.activation);
                    for c in &t.connections {
                        t_edges.insert((t.id.clone(), c.clone()));
                    }
                }
                let mut s_pos: HashMap<String, Vec<f64>> = HashMap::new();
                let mut s_field: HashMap<String, f64> = HashMap::new();
                let mut s_links = 0usize;
                for c in &b.stem.cells {
                    s_pos.entry(c.cell_id.clone()).or_insert_with(|| c.position.clone());
                    s_field.entry(c.cell_id.clone()).or_insert(c.field_strength);
                    s_links += c.connections.len();
                }
                V3Snap {
                    t_act,
                    t_edges,
                    s_pos,
                    s_field,
                    s_links,
                    tiles: b.tile.completed_tiles.len(),
                    cells: b.stem.cells.len(),
                }
            };
            let load = |dir: &Path| -> Body {
                match Body::load_full(dir) {
                    Ok(b) => b,
                    Err(e) => {
                        println!("解冻失败 {e}");
                        std::process::exit(1);
                    }
                }
            };
            fn beat3(b: &mut Body, stream: Option<&[u8]>, beats: usize) {
                let slices: Vec<&[u8]> = stream.map(|s| s.chunks(8 * 1024).collect()).unwrap_or_default();
                let per = (slices.len() + beats - 1) / beats.max(1);
                for beat in 0..beats {
                    for sl in slices.iter().skip(beat * per).take(per) {
                        // Tile 规则：切片均值入 propagate
                        let mean = sl.iter().map(|x| *x as f64).sum::<f64>() / sl.len().max(1) as f64;
                        let _ = b.tile.propagate(mean);
                        // Stem 规则：逐值 ingest + 每片收口一次（喂入节律）
                        for x in *sl {
                            b.stem.ingest(*x as f64);
                        }
                        let _ = b.stem.promote_from_fate();
                    }
                    let _ = b.tile.process_tiles_cycle();
                    let _ = b.stem.process_cells_cycle();
                    b.stem.settle_field();
                }
            }
            // 静默双生（基线 + 自检）
            let mut r1 = load(&dir);
            let mut r2 = load(&dir);
            beat3(&mut r1, None, beats);
            beat3(&mut r2, None, beats);
            let rs = snap3(&r1);
            let r2s = snap3(&r2);
            let sanity_pos: f64 = rs
                .s_pos
                .iter()
                .filter_map(|(id, p)| r2s.s_pos.get(id).map(|q| p.iter().zip(q.iter()).map(|(a, b)| (a - b) * (a - b)).sum::<f64>().sqrt()))
                .sum();
            println!("仪器自检 静默双生 Σ‖Δpos‖ {sanity_pos:.4}（应为 0）");
            let report = |tag: &str, b: &Body, pre_tiles: usize, pre_cells: usize| -> (Vec<String>, Vec<String>) {
                let a = snap3(b);
                let mut td: Vec<(&String, f64)> = a
                    .t_act
                    .iter()
                    .filter_map(|(id, v)| rs.t_act.get(id).map(|r| (id, *v - *r)))
                    .collect();
                td.sort_by(|x, y| y.1.abs().partial_cmp(&x.1.abs()).unwrap_or(std::cmp::Ordering::Equal));
                let t_amp: f64 = td.iter().map(|x| x.1.abs()).sum();
                let t_amp_silent: f64 = r2s
                    .t_act
                    .iter()
                    .filter_map(|(id, v)| rs.t_act.get(id).map(|r| (*v - *r).abs()))
                    .sum();
                let mut pd: Vec<(&String, f64, f64)> = a
                    .s_pos
                    .iter()
                    .filter_map(|(id, p)| {
                        rs.s_pos.get(id).map(|q| {
                            let d = p.iter().zip(q.iter()).map(|(x, y)| (x - y) * (x - y)).sum::<f64>().sqrt();
                            (id, d, (a.s_field.get(id).copied().unwrap_or(0.0) - rs.s_field.get(id).copied().unwrap_or(0.0)).abs())
                        })
                    })
                    .collect();
                pd.sort_by(|x, y| y.1.partial_cmp(&x.1).unwrap_or(std::cmp::Ordering::Equal));
                let s_amp: f64 = pd.iter().map(|x| x.1).sum();
                let s_amp_silent: f64 = r2s
                    .s_pos
                    .iter()
                    .filter_map(|(id, p)| rs.s_pos.get(id).map(|q| p.iter().zip(q.iter()).map(|(x, y)| (x - y) * (x - y)).sum::<f64>().sqrt()))
                    .sum();
                let only_arm = a.t_edges.difference(&rs.t_edges).count();
                println!(
                    "[{tag}] 扰动程度 tile Σ|Δact| {t_amp:.1}（静默涨落 {t_amp_silent:.1}）stem Σ‖Δpos‖ {s_amp:.2}（静默 {s_amp_silent:.2}）| 边仅本臂 {only_arm} stem连接 {}->{} | 单位 {}->{} / {}->{}",
                    a.s_links, rs.s_links, pre_tiles, a.tiles, pre_cells, a.cells
                );
                println!("  Tile规则响应Top:");
                for (id, d) in td.iter().take(6) {
                    println!("    {id} Δact {d:+.3}");
                }
                println!("  Stem规则响应Top（位移/场）:");
                for (id, d, f) in pd.iter().take(6) {
                    println!("    {id} ‖Δpos‖ {d:.3} |Δ场| {f:.3}");
                }
                println!(
                    "  纪律 全局tick {} 方程 {} 存储 {}",
                    b.global.global_tick,
                    b.global.cluster_equation.equation_str(),
                    b.stored_total()
                );
                (
                    td.iter().take(12).map(|x| x.0.clone()).collect(),
                    pd.iter().take(12).map(|x| x.0.clone()).collect(),
                )
            };
            let (pt, pc) = (rs.tiles, rs.cells);
            let mut arm = |tag: &str, stream: &[u8]| -> (Vec<String>, Vec<String>) {
                let mut b = load(&dir);
                beat3(&mut b, Some(stream), beats);
                report(tag, &b, pt, pc)
            };
            let (t1, s1) = arm("M1", &m1);
            let (t2, s2) = arm("M2", &m2);
            let (t3, s3) = arm("M3乱序", &m3);
            let ov = |a: &[String], b: &[String]| a.iter().filter(|x| b.contains(x)).count();
            println!(
                "前12重叠 Tile M1∩M2 {} M1∩M3 {} | Stem M1∩M2 {} M1∩M3 {}（低=分化）",
                ov(&t1, &t2),
                ov(&t1, &t3),
                ov(&s1, &s2),
                ov(&s1, &s3)
            );
        }
        "vent4" => {
            // 混沌对照：Stem 规则逐拍追位移分叉。四具同盘身体锁步：静默、
            // M1、M1 只翻一字节、M1 切片乱序。每拍后量各臂对静默的
            // Σ‖Δpos‖。判别：翻一字节若与 M1 贴合而乱序分叉→结构性听觉；
            // 翻一字节也指数分叉→蝴蝶效应，分化不可归功于内容。
            let dir = env_path("VH_LOAD_DIR").expect("vent4 需要 VH_LOAD_DIR");
            let beats = env_usize("VH_VENT_BEATS", 6);
            let kb = env_usize("VH_VENT_KB", 256);
            let files = all_files();
            let stream = read_stream(&files, kb * 1024);
            let m1: Vec<u8> = stream;
            let mut flip = m1.clone();
            if flip.len() > 1001 {
                flip[1000] = flip[1000].wrapping_add(1);
            }
            let mut shuf: Vec<u8> = Vec::with_capacity(m1.len());
            {
                let chunks: Vec<&[u8]> = m1.chunks(8 * 1024).collect();
                let mut idx: Vec<usize> = (0..chunks.len()).collect();
                let mut s: u64 = 0x9E3779B97F4A7C15;
                for i in (1..idx.len()).rev() {
                    s ^= s << 13;
                    s ^= s >> 7;
                    s ^= s << 17;
                    idx.swap(i, (s % (i as u64 + 1)) as usize);
                }
                for i in idx {
                    shuf.extend_from_slice(chunks[i]);
                }
            }
            let load = |dir: &Path| -> Body {
                match Body::load_full(dir) {
                    Ok(b) => b,
                    Err(e) => {
                        println!("解冻失败 {e}");
                        std::process::exit(1);
                    }
                }
            };
            let pos_map = |b: &Body| -> HashMap<String, Vec<f64>> {
                let mut m: HashMap<String, Vec<f64>> = HashMap::new();
                for c in &b.stem.cells {
                    m.entry(c.cell_id.clone()).or_insert_with(|| c.position.clone());
                }
                m
            };
            let disp = |a: &HashMap<String, Vec<f64>>, r: &HashMap<String, Vec<f64>>| -> f64 {
                a.iter()
                    .filter_map(|(id, p)| r.get(id).map(|q| p.iter().zip(q.iter()).map(|(x, y)| (x - y) * (x - y)).sum::<f64>().sqrt()))
                    .sum()
            };
            let top_ids = |a: &HashMap<String, Vec<f64>>, r: &HashMap<String, Vec<f64>>| -> Vec<String> {
                let mut v: Vec<(&String, f64)> = a
                    .iter()
                    .filter_map(|(id, p)| r.get(id).map(|q| (id, p.iter().zip(q.iter()).map(|(x, y)| (x - y) * (x - y)).sum::<f64>().sqrt())))
                    .collect();
                v.sort_by(|x, y| y.1.partial_cmp(&x.1).unwrap_or(std::cmp::Ordering::Equal));
                v.iter().take(12).map(|x| x.0.clone()).collect()
            };
            let beat_once = |b: &mut Body, stream: Option<&[u8]>, beat: usize, beats: usize| {
                if let Some(s) = stream {
                    let slices: Vec<&[u8]> = s.chunks(8 * 1024).collect();
                    let per = (slices.len() + beats - 1) / beats.max(1);
                    for sl in slices.iter().skip(beat * per).take(per) {
                        for x in *sl {
                            b.stem.ingest(*x as f64);
                        }
                        let _ = b.stem.promote_from_fate();
                    }
                }
                let _ = b.stem.process_cells_cycle();
                b.stem.settle_field();
            };
            let mut silent = load(&dir);
            let mut b_m1 = load(&dir);
            let mut b_flip = load(&dir);
            let mut b_shuf = load(&dir);
            let mut tops: Vec<Vec<String>> = Vec::new();
            for beat in 0..beats {
                beat_once(&mut silent, None, beat, beats);
                beat_once(&mut b_m1, Some(&m1), beat, beats);
                beat_once(&mut b_flip, Some(&flip), beat, beats);
                beat_once(&mut b_shuf, Some(&shuf), beat, beats);
                let rp = pos_map(&silent);
                let d1 = disp(&pos_map(&b_m1), &rp);
                let df = disp(&pos_map(&b_flip), &rp);
                let ds = disp(&pos_map(&b_shuf), &rp);
                // 臂间分叉（对 M1）：翻一字节/乱序各自离 M1 多远
                let pm = pos_map(&b_m1);
                let xf = disp(&pos_map(&b_flip), &pm);
                let xs = disp(&pos_map(&b_shuf), &pm);
                println!(
                    "拍 {} 对静默 Σ‖Δpos‖ M1 {:.1} 翻字节 {:.1} 乱序 {:.1} | 对M1分叉 翻字节 {:.1} 乱序 {:.1}",
                    beat + 1,
                    d1,
                    df,
                    ds,
                    xf,
                    xs
                );
                if beat + 1 == beats {
                    tops.push(top_ids(&pos_map(&b_m1), &rp));
                    tops.push(top_ids(&pos_map(&b_flip), &rp));
                    tops.push(top_ids(&pos_map(&b_shuf), &rp));
                }
            }
            if tops.len() == 3 {
                let ov = |a: &[String], b: &[String]| a.iter().filter(|x| b.contains(x)).count();
                println!(
                    "末拍前12重叠 M1∩翻字节 {} M1∩乱序 {}（翻字节贴合且乱序低=结构性听觉）",
                    ov(&tops[0], &tops[1]),
                    ov(&tops[0], &tops[2])
                );
            }
            println!(
                "纪律 全局tick {} 方程 {} 存储 {}",
                b_m1.global.global_tick,
                b_m1.global.cluster_equation.equation_str(),
                b_m1.stored_total()
            );
        }
        "fateqa" => {
            // 对外 Fate 试问：store_data 入口与 react 出口的现状普查。
            // react(&self) 只读：刺激不入 receive_passive、不 tick、不改方程存储；
            // 对"签名近等（阈值内）或原文包含刺激"的条目按 tick 一次送出。
            let mut stem = StemCellComplete::new();
            let facts = [
                "法国的首都是巴黎",
                "英国的首都是伦敦",
                "德国的首都是柏林",
                "巴黎是法国的首都，人口约一千万",
            ];
            for f in &facts {
                stem.fate_store(f.as_bytes());
            }
            println!("存入后 fate_count {}", stem.fate_count());
            println!(
                "签名 法国的首都是巴黎 {:.2} | 法国首都 {:.2} | 日本的首都是 {:.2}（方程阈值 0.01）",
                stem.triple_fate.compute_signature("法国的首都是巴黎".as_bytes()),
                stem.triple_fate.compute_signature("法国首都".as_bytes()),
                stem.triple_fate.compute_signature("日本的首都是".as_bytes())
            );
            let ask = |q: &str, stem: &StemCellComplete| {
                let out = stem.triple_fate.react(q.as_bytes());
                println!("问「{q}」-> {} 字节 {:?}", out.len(), String::from_utf8_lossy(&out));
            };
            ask("法国的首都是", &stem); // 包含命中预期
            ask("法国首都", &stem); // 非子串：中间隔了「的」，只能靠签名
            ask("日本的首都是", &stem); // 未存事实
            // 对照：整体喂法喂过的容器（quad_keep+逐字节），react 是否有东西
            let mut fed = StemCellComplete::new();
            for f in &facts {
                fed.quad_keep(f.as_bytes());
                for b in f.as_bytes() {
                    fed.ingest(*b as f64);
                }
            }
            println!("整体喂法后 fate_count {}（对照）", fed.fate_count());
            let out = fed.triple_fate.react("法国的首都是".as_bytes());
            println!("对照问 -> {} 字节 {:?}", out.len(), String::from_utf8_lossy(&out));
        }
        other => println!("未知 VH_PROBE={other}（org|order|domain|assimilate|cycle|vent|vent2|vent3|vent4|fateqa）"),
    }
}
