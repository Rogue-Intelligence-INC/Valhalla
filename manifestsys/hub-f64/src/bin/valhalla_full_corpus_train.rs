//! 全量语料 × 5 轮换料：按昨天 valhalla_relation_incubate2 的训练路子跑。
//!
//! 换料：全量语料按大小贪心切成 5 份互不重叠的分片（大文件优先塞进当前
//! 最小的分片，各份字节量接近、料型混合），第 r 轮只喂第 r 份没见过的料，
//! 身体跨轮不重置。五轮合起来才是全量一遍，不再把同一批料重喂五遍。
//!
//! 高效路子（按 Valhalla 自己的特性，不加外来机制）：
//! - 切片改小到 256KB：单元数涨约 8 倍，签名不再被 2MB 里几千条句子平均掉。
//! - 周期内嵌：每喂 CYCLE_EVERY_SLICES 片就跑一次身体自己的周期
//!   （合并按签名边际递减删冗余、能量并入幸存者），让冗余在训练中途就被
//!   压掉，而不是喂完才收拾；细胞数被压住，GPU 场 gram(n²) 也跟着便宜。
//! - 新料 corpus_diverse：法/德/西/中文学、科学、戏剧、诗歌 + Python/Go/JS
//!   代码，字节分布与原语料（78% 是 caption JSON + 句子表）不同。
//!
//! Body/feed/cycle 与电池（valhalla_industry_battery）完全一致的整体喂法：
//! 每一片先 tile.quad_keep + stem.quad_keep 进四个四元存储（八份），
//! 再逐字节 global.receive_passive + stem.ingest，Tile 每片喂均值，
//! promote_from_fate 收口，两体 follow_global_fate；
//! 轮末 process_tiles_cycle + process_cells_cycle + settle_field(GPU) + propagate，
//! 监控用同一个 BodyMonitor，并打印 iter_line / metric_line。
//!
//! 不存在绕过存储的喂法：凡进 Valhalla 的数据都过四元存储。
//! 规模由 VH_MAX_MB 控制（八份存储 + Fate 内部状态随喂入量增长，
//! 这是整体喂法的真实代价，不再以关掉存储换速度）。

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Instant;

use hub_f64::monitor::BodyMonitor;
use hub_f64::TripleFateCombined;
use stem_cell::stemcell_complete::CellCycleResult;
use stem_cell::StemCellComplete;
use valhalla_tile_quattro::tile_complete::TileCycleResult;
use valhalla_tile_quattro::TileQuattroComplete;

const ROUNDS: usize = 5;
const REPORT: u64 = 256 * 1024 * 1024;

/// 环境变量读正整数（碎片化训练参数，只在训练器层面，不动库）。
fn env_usize(key: &str, default: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .filter(|&v| v > 0)
        .unwrap_or(default)
}

/// 一片 = 一次喂入的碎片：Stem 收口一个细胞，Tile 收口一个瓦片。
/// 多碎片、小量多餐：默认 8KB 一片（VH_SLICE_KB 可调；原为 256KB）。
fn slice_bytes() -> usize {
    env_usize("VH_SLICE_KB", 8) * 1024
}

/// 每喂多少数据跑一次中周期（合并当场删冗余）：默认 2MB
/// （VH_CYCLE_MB 可调；原为 256MB 一次，周期太稀）。
fn cycle_every_slices(slice: usize) -> usize {
    (env_usize("VH_CYCLE_MB", 2) * 1024 * 1024 / slice.max(1)).max(1)
}

/// 实际训练轮数（VH_ROUNDS 可调，默认全 5 轮不重叠分片）。
fn train_rounds() -> usize {
    env_usize("VH_ROUNDS", ROUNDS).min(ROUNDS)
}

/// 总喂入上限（字节）：训练到一定规模就停（VH_MAX_MB 可调，默认不限）。
/// 碎片训练下种群随喂入持续生长，场步 O(n²)，无上限会跑到小时级。
fn max_fed_bytes() -> u64 {
    std::env::var("VH_MAX_MB")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|&v| v > 0)
        .map(|mb| mb * 1024 * 1024)
        .unwrap_or(u64::MAX)
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

    /// 与电池 Body::feed 完全相同的整体喂法：每片先经 tile/stem 的
    /// quad_keep 进四个四元存储，再走信号通道。不存在绕过存储的喂法。
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
        // 老 Fate 搭法：每周期驱动一次多层 tick —— base 判模式、meta 观察 body 状态、
        // 涌现规则回推。之前的训练器只喂 receive_passive，这台机器从没转过。
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
}

fn roots() -> Vec<PathBuf> {
    let base = Path::new(r"C:\Ai\rogue-products\valhalla-self-incubate");
    vec![
        base.join("corpus"),
        base.join("corpus_new"),
        base.join("corpus_diverse"),
    ]
}

fn keep(path: &Path) -> bool {
    if path.components().any(|c| c.as_os_str() == "Annotations") {
        return false;
    }
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some(
            "txt" | "json" | "jsonl" | "csv" | "tsv" | "rs" | "md" | "py" | "go" | "js"
                | "java" | "c" | "h" | "cpp" | "ts"
        )
    )
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return; };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, out);
        } else if keep(&path) {
            out.push(path);
        }
    }
}

fn main() {
    let mut files = Vec::new();
    for root in roots() {
        collect(&root, &mut files);
    }
    // 换料分片：按大小从大到小贪心装进当前最轻的分片，份间字节量接近。
    let mut sized: Vec<(u64, PathBuf)> = files
        .iter()
        .map(|p| (std::fs::metadata(p).map(|m| m.len()).unwrap_or(0), p.clone()))
        .collect();
    sized.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    let mut shards: Vec<Vec<PathBuf>> = vec![Vec::new(); ROUNDS];
    let mut shard_bytes = vec![0u64; ROUNDS];
    for (size, path) in sized {
        let lightest = (0..ROUNDS)
            .min_by_key(|&i| shard_bytes[i])
            .unwrap_or(0);
        shards[lightest].push(path);
        shard_bytes[lightest] += size;
    }
    for shard in &mut shards {
        shard.sort();
    }
    let total: u64 = shard_bytes.iter().sum();
    let slice = slice_bytes();
    let cycle_every = cycle_every_slices(slice);
    let rounds = train_rounds();
    println!(
        "== 换料模式 全量语料 {} 个文件 {:.2} GB 切 {} 份不重叠 | 昨天路子 Body/feed/cycle | 每片 {} KB 每 {} 片({} MB)一中周期 | 训练 {} 轮 | 场 GPU ==",
        files.len(),
        total as f64 / (1024.0 * 1024.0 * 1024.0),
        ROUNDS,
        slice / 1024,
        cycle_every,
        cycle_every * slice / (1024 * 1024),
        rounds
    );
    for (i, shard) in shards.iter().enumerate() {
        println!(
            "  第 {} 份料 {} 个文件 {:.2} GB（本身体没见过）",
            i + 1,
            shard.len(),
            shard_bytes[i] as f64 / 1e9
        );
    }
    println!("注: 每一片都经 tile/stem 的 quad_keep 进四元存储（八份），与电池同一条整体喂入口");

    let mut body = Body::new();
    let mut buf = vec![0u8; slice];
    let all_start = Instant::now();
    let max_fed = max_fed_bytes();
    let mut fed_total: u64 = 0;
    let mut stop = false;
    for round in 1..=rounds {
        let t0 = Instant::now();
        let mut fed: u64 = 0;
        let mut next = REPORT / 4;
        // 跨文件攒满一片再喂：flickr 那 3 万个小句文件不能一个文件收口一个细胞，
        // 否则细胞数会冲满，GPU gram(n²) 爆显存。
        let mut filled = 0usize;
        let mut since_cycle = 0usize;
        for path in &shards[round - 1] {
            let mut file = match File::open(path) {
                Ok(f) => f,
                Err(_) => continue,
            };
            loop {
                let n = match file.read(&mut buf[filled..]) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => n,
                };
                filled += n;
                fed += n as u64;
                fed_total += n as u64;
                if fed_total >= max_fed {
                    // 训练到预定规模：收掉当前残片后停喂，
                    // 仍走本轮轮末周期+收紧再收尾（终局状态在那里打印）。
                    if filled > 0 {
                        body.feed(&buf[..filled]);
                        filled = 0;
                    }
                    println!(
                        "== 已喂 {:.0} MB 达到预定规模（VH_MAX_MB），停止喂入 ==",
                        fed_total as f64 / 1e6
                    );
                    stop = true;
                    break;
                }
                if filled == slice {
                    body.feed(&buf);
                    filled = 0;
                    since_cycle += 1;
                    if since_cycle >= cycle_every {
                        let (t, s) = body.cycle();
                        since_cycle = 0;
                        println!(
                            "  round {round} 中周期 {} | tile合并 {} 分裂 {} stem合并 {} 分裂 {} 死亡 {}",
                            body.monitor.snapshot(&body.tile, &body.stem).line(),
                            t.merges,
                            t.splits,
                            s.merges,
                            s.divisions,
                            s.deaths
                        );
                    }
                }
                if fed >= next {
                    println!(
                        "  round {round} 新料已喂 {:.2} / {:.2} GB | 用时 {:.0}s",
                        fed as f64 / 1e9,
                        shard_bytes[round - 1] as f64 / 1e9,
                        t0.elapsed().as_secs_f64()
                    );
                    next += REPORT / 4;
                }
            }
            if stop {
                break;
            }
        }
        if filled > 0 {
            body.feed(&buf[..filled]);
        }
        let cyc = Instant::now();
        let (t, s) = body.cycle();
        println!(
            "round {round} 轮末周期 tile合并 {} 分裂 {} stem合并 {} 分裂 {} 死亡 {}",
            t.merges, t.splits, s.merges, s.divisions, s.deaths
        );
        // 收紧阶段：不再喂料，只反复跑身体自己的周期，让合并把冗余压干净，
        // 直到某一周期合并归零（或顶到上限）。Tile 和 Stem 一起收，看 Tile 压多少。
        for tighten in 1..=20 {
            let (t, s) = body.cycle();
            let snap = body.monitor.snapshot(&body.tile, &body.stem);
            println!(
                "round {round} 收紧 {tighten} tiles {} cells {} | tile合并 {} stem合并 {} | tile_block {} stem_block {}",
                snap.tiles, snap.cells, t.merges, s.merges, snap.largest_tile_block(), snap.largest_stem_block()
            );
            if t.merges == 0 && s.merges == 0 {
                break;
            }
        }
        println!("round {round} {}", body.monitor.snapshot(&body.tile, &body.stem).line());
        println!("round {round} {}", body.iter_line());
        println!(
            "round {round} {} | 场设备 {}",
            body.metric_line(),
            body.stem.field_device
        );
        println!(
            "round {round} 本轮 {:.2} GB | 喂 {:.1}s 周期 {:.1}s | 累计 {:.1}s",
            fed as f64 / 1e9,
            t0.elapsed().as_secs_f64() - cyc.elapsed().as_secs_f64(),
            cyc.elapsed().as_secs_f64(),
            all_start.elapsed().as_secs_f64()
        );
        if stop {
            break;
        }
    }
    println!("{rounds} 轮结束 | 总用时 {:.1}s", all_start.elapsed().as_secs_f64());
}
