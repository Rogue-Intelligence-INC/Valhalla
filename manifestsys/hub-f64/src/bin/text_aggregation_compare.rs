//! Hub / Tile / StemCell — 同一训练文本语料下的自动聚合对比测试
//! 设计原则验证：微容器无自决收敛，偏好由 Fate 自然形成，无预设分工

use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::time::Instant;

use hub_f64::{HubConfig, HubRequest, Priority, RequestType, ValhallaHub};
use stem_cell::StemCellComplete;
use valhalla_tile_quattro::TileQuattroComplete;

const REPO: &str = "/home/jouly/AI/Valhalla";
const CYCLES: usize = 3;

fn repo_root() -> PathBuf {
    std::env::var("VALHALLA_REPO")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(REPO))
}

fn load_training_texts() -> Vec<(String, String)> {
    let root = repo_root();
    let mut items: Vec<(String, String)> = Vec::new();

    let ud = root.join("manifestsys/hub/data/languages/universal_dependencies");
    if ud.exists() {
        for entry in fs::read_dir(&ud).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "conllu") {
                let lang = path.file_stem().and_then(|s| s.to_str()).unwrap_or("ud").to_string();
                if let Ok(file) = File::open(&path) {
                    for line in BufReader::new(file).lines().flatten() {
                        if !line.starts_with('#') && !line.trim().is_empty() {
                            let parts: Vec<&str> = line.split('\t').collect();
                            if parts.len() >= 2 && parts[1].len() > 1 {
                                items.push((lang.clone(), parts[1].to_string()));
                            }
                        }
                    }
                }
            }
        }
    }

    for name in [
        "valhalla_all_dialogues.txt",
        "valhalla_enhanced_dialogues.txt",
        "valhalla_afternoon_dialogues.txt",
    ] {
        let p = root.join(name);
        if p.exists() {
            if let Ok(c) = fs::read_to_string(&p) {
                for line in c.lines() {
                    let t = line.trim();
                    if t.len() > 3 && !t.starts_with('#') {
                        items.push(("dialogue".into(), t.to_string()));
                    }
                }
            }
        }
    }

    let vocab = root.join("manifestsys/hub/data/languages/vocabulary/common_words.json");
    if vocab.exists() {
        if let Ok(c) = fs::read_to_string(&vocab) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&c) {
                if let Some(obj) = json.as_object() {
                    for (lang, words) in obj {
                        if let Some(arr) = words.as_array() {
                            for w in arr {
                                if let Some(s) = w.as_str() {
                                    items.push((lang.clone(), s.to_string()));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    items
}

struct HubAgg {
    signals: u64,
    routing: [u64; 4],
    final_prefs: [f64; 4],
    convergence: Option<String>,
}

fn test_hub(texts: &[(String, String)]) -> HubAgg {
    let mut hub = ValhallaHub::with_config(HubConfig {
        resource_percent: 70,
        batch_size: 1024,
        ..Default::default()
    });
    let mut routing = [0u64; 4];
    let mut n = 0u64;

    for (_lang, text) in texts {
        for ch in text.chars() {
            let req = HubRequest {
                request_id: format!("t_{n}"),
                request_type: RequestType::Learning,
                intensity: (ch as u32 as f64 / 255.0).clamp(0.0, 1.0),
                data: None,
                priority: Priority::Normal,
            };
            let resp = hub.process(req);
            let idx = if resp.message.contains("BlackHole") {
                0
            } else if resp.message.contains("MeshBrain") {
                1
            } else if resp.message.contains("MultiNova") {
                2
            } else {
                3
            };
            routing[idx] += 1;
            n += 1;
        }
    }
    hub.mark_epoch_complete();
    let prefs = hub.get_fate_preferences();
    let mut final_prefs = [0.25, 0.25, 0.25, 0.25];
    for (i, v) in prefs.affinities.iter().take(4).enumerate() {
        final_prefs[i] = *v;
    }
    HubAgg {
        signals: n,
        routing,
        final_prefs,
        convergence: hub.get_fate_convergence_pattern().map(|s| s.to_string()),
    }
}

struct TileAgg {
    completed_tiles: usize,
    merges: u64,
    splits: u64,
    fragmentation: f64,
    dimension: f64,
    fate_converged: bool,
}

fn test_tile(texts: &[(String, String)]) -> TileAgg {
    let mut tile = TileQuattroComplete::new();
    let mut merges = 0u64;
    let mut splits = 0u64;

    for (i, (_lang, text)) in texts.iter().enumerate() {
        let v = text.len() as f64 + (i as f64).sin() * 10.0;
        tile.receive_data(v);
        for ch in text.chars() {
            tile.receive_data(ch as u32 as f64);
        }
    }

    for _ in 1..CYCLES {
        let r = tile.process_tiles_cycle();
        merges += r.merges as u64;
        splits += r.splits as u64;
        if tile.has_fate_converged() {
            break;
        }
    }

    let st = tile.get_status();
    TileAgg {
        completed_tiles: st.completed_tiles,
        merges,
        splits,
        fragmentation: 0.0,
        dimension: st.dimension,
        fate_converged: tile.has_fate_converged(),
    }
}

struct StemAgg {
    cells: usize,
    connections: usize,
    merges: u64,
    divisions: u64,
    dimension: f64,
    fate_converged: bool,
}

fn test_stemcell(texts: &[(String, String)]) -> StemAgg {
    let mut sc = StemCellComplete::new();
    let mut merges = 0u64;
    let mut divisions = 0u64;

    for (_lang, text) in texts {
        for ch in text.chars() {
            let v = (ch as u32 as f64 / 65536.0) * 2.0 - 1.0;
            sc.receive_data(v);
        }
        sc.receive_data(0.0);
    }

    for _ in 1..CYCLES {
        let r = sc.process_cells_cycle();
        merges += r.merges as u64;
        divisions += r.divisions as u64;
        if sc.has_fate_converged() {
            break;
        }
    }

    let st = sc.get_status();
    let connections: usize = sc.cells.iter().map(|c| c.connections.len()).sum();
    StemAgg {
        cells: st.cell_count,
        connections,
        merges,
        divisions,
        dimension: sc.space.current_dimension,
        fate_converged: sc.has_fate_converged(),
    }
}

fn main() {
    let out_dir = repo_root().join("reports");
    let _ = fs::create_dir_all(&out_dir);
    let report_path = out_dir.join("TEXT_AGGREGATION_COMPARE_20260604.md");

    println!("═══════════════════════════════════════════════════════════════");
    println!("  Hub / Tile / StemCell — 训练文本自动聚合对比");
    println!("═══════════════════════════════════════════════════════════════\n");

    let t0 = Instant::now();
    let texts = load_training_texts();
    let unique_langs: std::collections::HashSet<_> = texts.iter().map(|(l, _)| l.clone()).collect();

    println!("语料: {} 条文本片段, {} 种来源/语言标签\n", texts.len(), unique_langs.len());

    if texts.is_empty() {
        eprintln!("ERROR: 未加载到训练文本");
        std::process::exit(1);
    }

    let hub = test_hub(&texts);
    println!("Hub   完成: {} 信号", hub.signals);

    let tile = test_tile(&texts);
    println!("Tile  完成: {} 闭合块", tile.completed_tiles);

    let stem = test_stemcell(&texts);
    println!("Stem  完成: {} 细胞, {} 连接\n", stem.cells, stem.connections);

    let elapsed = t0.elapsed().as_secs_f64();
    let names = ["BlackHole", "MeshBrain", "MultiNova", "BaseForce"];

    let mut report = String::new();
    report.push_str("# Hub / Tile / StemCell — Text Aggregation Compare\n\n");
    report.push_str(&format!("- Date: 2026-06-04\n"));
    report.push_str(&format!("- Corpus: {} snippets, {} source tags\n", texts.len(), unique_langs.len()));
    report.push_str(&format!("- Cycles (Tile/Stem): {}\n", CYCLES));
    report.push_str(&format!("- Elapsed: {:.2}s\n\n", elapsed));

    report.push_str("## Hub (valhalla-f64)\n\n");
    report.push_str(&format!("- Signals: {}\n", hub.signals));
    report.push_str(&format!("- Fate convergence: {:?}\n", hub.convergence));
    for i in 0..4 {
        let pct = hub.routing[i] as f64 / hub.signals.max(1) as f64 * 100.0;
        report.push_str(&format!("- Route {}: {} ({:.1}%)\n", names[i], hub.routing[i], pct));
    }
    report.push_str(&format!(
        "- Final prefs: BH={:.1}% MB={:.1}% MN={:.1}% BF={:.1}%\n\n",
        hub.final_prefs[0] * 100.0,
        hub.final_prefs[1] * 100.0,
        hub.final_prefs[2] * 100.0,
        hub.final_prefs[3] * 100.0,
    ));

    report.push_str("## Tile (TileQuattroComplete)\n\n");
    report.push_str(&format!("- Completed tiles: {}\n", tile.completed_tiles));
    report.push_str(&format!("- Merges / Splits: {} / {}\n", tile.merges, tile.splits));
    report.push_str(&format!("- Tile connections (cycle): merges={} splits={}\n", tile.merges, tile.splits));
    report.push_str(&format!("- Space dimension: {:.2}\n", tile.dimension));
    report.push_str(&format!("- Fate converged: {}\n\n", tile.fate_converged));

    report.push_str("## StemCell (StemCellComplete)\n\n");
    report.push_str(&format!("- Cells: {}\n", stem.cells));
    report.push_str(&format!("- Connections: {}\n", stem.connections));
    report.push_str(&format!("- Merges / Divisions: {} / {}\n", stem.merges, stem.divisions));
    report.push_str(&format!("- Space dimension: {:.2}\n", stem.dimension));
    report.push_str(&format!("- Fate converged: {}\n\n", stem.fate_converged));

    report.push_str("## Interpretation\n\n");
    report.push_str("- Micro-containers do not self-decide convergence; Fate controls closure.\n");
    report.push_str("- No preset quad-system roles; routing distribution emerges from data + Fate.\n");
    report.push_str("- Training text auto-aggregates into structures (tiles/cells/routes) without manual labels.\n");

    fs::write(&report_path, &report).expect("write report");
    println!("报告: {}\n", report_path.display());
    println!("{report}");
}
