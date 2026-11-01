//! 证据探针（仅测试侧工具，库零改动）：四种模式由 VH_PROBE 选择。
//!   org        : VH_LOAD_DIR 解冻一具身体，打印自发网络组织统计
//!   order      : 同批料 A(文本)->B(代码) 与 B->A 两序训练，比终局方程（顺序留痕）
//!   domain     : 纯文本 vs 纯代码各训 24MB，比方程（选料定向对照版）
//!   assimilate : VH_LOAD_DIR 解冻身体 vs 空身体，同喂留出料（第 5 份分片，
//!                曲线训练从未见过），比同化期出生/合并/死亡/分裂
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
        other => println!("未知 VH_PROBE={other}（org|order|domain|assimilate|cycle|fateqa）"),
    }
}
