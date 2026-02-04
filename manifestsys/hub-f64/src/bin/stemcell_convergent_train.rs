//! StemCell 收敛迭代训练
//!
//! 10次收敛迭代训练:
//! - 每次保留上一次的状态
//! - 加上全语料再次输入
//! - 观察收敛行为和细胞演化

use std::path::Path;
use std::time::Instant;
use std::collections::HashMap;

use stem_cell::{StemCellComplete, CompleteCell as Cell};

const EXTERNAL_CORPUS_BASE: &str = "/run/media/jouly/My Book/valhalla_training_data";

fn main() {
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("       StemCell 收敛迭代训练 (10次全语料累积)");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    // 加载全部语料
    println!("📦 加载语料...");
    let start = Instant::now();
    
    let mut all_corpus: Vec<(String, Vec<u8>)> = Vec::new();
    
    // 1. LibriSpeech 转录
    let libri = load_librispeech_transcripts();
    println!("   LibriSpeech: {} 文件, {:.1} MB", libri.len(), 
        libri.iter().map(|(_, d)| d.len()).sum::<usize>() as f64 / 1024.0 / 1024.0);
    all_corpus.extend(libri);
    
    // 2. COCO 标注
    let coco = load_coco_captions();
    println!("   COCO: {} 文件, {:.1} MB", coco.len(),
        coco.iter().map(|(_, d)| d.len()).sum::<usize>() as f64 / 1024.0 / 1024.0);
    all_corpus.extend(coco);
    
    // 3. Valhalla 源码
    let valhalla = load_rust_corpus(Path::new("/home/jouly/AI/Valhalla"));
    println!("   Valhalla: {} 文件, {:.1} MB", valhalla.len(),
        valhalla.iter().map(|(_, d)| d.len()).sum::<usize>() as f64 / 1024.0 / 1024.0);
    all_corpus.extend(valhalla);
    
    // 4. Cargo Registry
    let cargo = load_cargo_registry(3000);
    println!("   Cargo: {} 文件, {:.1} MB", cargo.len(),
        cargo.iter().map(|(_, d)| d.len()).sum::<usize>() as f64 / 1024.0 / 1024.0);
    all_corpus.extend(cargo);
    
    let total_bytes: usize = all_corpus.iter().map(|(_, d)| d.len()).sum();
    println!();
    println!("📊 语料总计: {} 文件, {:.1} MB", all_corpus.len(), total_bytes as f64 / 1024.0 / 1024.0);
    println!("   加载耗时: {:.2}s", start.elapsed().as_secs_f64());
    println!();
    
    // 初始化 StemCell
    let mut stemcell = StemCellComplete::new();
    println!("🧬 初始化 StemCell: {} 细胞", stemcell.cells.len());
    println!();
    
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("                    开始收敛迭代训练");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    let mut iteration_stats: Vec<IterationStats> = Vec::new();
    
    for iteration in 1..=10 {
        println!("┌─────────────────────────────────────────────────────────────────────────────┐");
        println!("│  迭代 {}/10                                                                 │", iteration);
        println!("└─────────────────────────────────────────────────────────────────────────────┘");
        
        let iter_start = Instant::now();
        let initial_cells = stemcell.cells.len();
        let initial_fate_tick = stemcell.triple_fate.global_tick;
        
        let mut new_cells_total = 0usize;
        let mut converged_count = 0usize;
        let mut total_signals = 0u64;
        let mut discoveries = 0usize;
        
        // 训练直到收敛
        let mut sub_round = 0;
        let mut last_cell_count = initial_cells;
        let mut stable_rounds = 0;
        
        loop {
            sub_round += 1;
            let round_start = Instant::now();
            let mut round_new_cells = 0;
            let mut round_converged = 0;
            
            // 输入全部语料
            for (_, bytes) in &all_corpus {
                for &b in bytes.iter() {
                    let result = stemcell.receive_data(b as f64);
                    round_new_cells += result.new_cells;
                    if result.container_converged {
                        round_converged += 1;
                    }
                    total_signals += 1;
                    discoveries += result.discoveries.len();
                }
            }
            
            new_cells_total += round_new_cells;
            converged_count += round_converged;
            
            let current_cells = stemcell.cells.len();
            let round_time = round_start.elapsed().as_secs_f64();
            let ops_per_sec = total_bytes as f64 / round_time / 1_000_000.0;
            
            println!("   轮 {}: +{} 细胞 → {} 总计 | {} 收敛 | {:.1}M ops/s",
                sub_round, round_new_cells, current_cells, round_converged, ops_per_sec);
            
            // 检查收敛：连续3轮细胞数变化 < 1%
            let change_rate = (current_cells as f64 - last_cell_count as f64).abs() / last_cell_count as f64;
            if change_rate < 0.01 {
                stable_rounds += 1;
            } else {
                stable_rounds = 0;
            }
            
            if stable_rounds >= 3 || sub_round >= 10 {
                println!("   └─ 收敛! (稳定轮数: {}, 变化率: {:.2}%)", stable_rounds, change_rate * 100.0);
                break;
            }
            
            last_cell_count = current_cells;
        }
        
        let iter_time = iter_start.elapsed().as_secs_f64();
        let final_cells = stemcell.cells.len();
        let final_fate_tick = stemcell.triple_fate.global_tick;
        
        // 分析细胞特征
        let cell_analysis = analyze_cells(&stemcell.cells);
        
        let stats = IterationStats {
            iteration,
            sub_rounds: sub_round,
            initial_cells,
            final_cells,
            new_cells: new_cells_total,
            converged_count,
            discoveries,
            total_signals,
            fate_tick_delta: final_fate_tick - initial_fate_tick,
            elapsed_secs: iter_time,
            cell_analysis,
        };
        
        println!();
        println!("   📊 迭代 {} 统计:", iteration);
        println!("      细胞: {} → {} (+{})", stats.initial_cells, stats.final_cells, stats.new_cells);
        println!("      收敛: {} | 发现: {}", stats.converged_count, stats.discoveries);
        println!("      Fate Tick: +{}", stats.fate_tick_delta);
        println!("      耗时: {:.2}s ({} 子轮)", stats.elapsed_secs, stats.sub_rounds);
        println!();
        println!("   🔬 细胞分化:");
        for (name, count) in &stats.cell_analysis.functional_groups {
            let pct = *count as f64 / stats.final_cells as f64 * 100.0;
            println!("      {}: {} ({:.1}%)", name, count, pct);
        }
        println!("      能量均值: {:.4} ± {:.4}", stats.cell_analysis.energy_mean, stats.cell_analysis.energy_std);
        println!("      信号均值: {:.0} ± {:.0}", stats.cell_analysis.signal_mean, stats.cell_analysis.signal_std);
        println!();
        
        iteration_stats.push(stats);
        
        // 保存中间状态
        if iteration % 3 == 0 {
            let path = format!("/tmp/stemcell_convergent_iter{}.stemcell", iteration);
            if let Err(e) = stemcell.save_slice(Path::new(&path)) {
                println!("   ⚠️  保存失败: {}", e);
            } else {
                println!("   💾 已保存: {}", path);
            }
        }
        
        // 存储迭代摘要到 Fate
        let summary = format!(
            "迭代{}: {}→{}细胞, {}收敛, {}发现, {:.1}s",
            iteration, initial_cells, final_cells, converged_count, discoveries, iter_time
        );
        stemcell.triple_fate.store_string(&format!("迭代{}: {}", iteration, summary));
    }
    
    // 最终分析
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("                    训练完成 - 最终分析");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    // 演化曲线
    println!("📈 细胞演化曲线:");
    println!("   迭代 │ 细胞数    │ 新增     │ 收敛     │ 耗时");
    println!("   ─────┼───────────┼──────────┼──────────┼────────");
    for s in &iteration_stats {
        println!("   {:>4} │ {:>9} │ {:>8} │ {:>8} │ {:.2}s",
            s.iteration, s.final_cells, s.new_cells, s.converged_count, s.elapsed_secs);
    }
    println!();
    
    // 功能分化演变
    println!("🧬 功能分化演变:");
    let func_names: Vec<&str> = vec!["核心处理", "信号调节", "活跃处理", "休眠储备", "高变异", "普通"];
    print!("   迭代 │");
    for name in &func_names {
        print!(" {:>8} │", name);
    }
    println!();
    print!("   ─────┼");
    for _ in &func_names {
        print!("──────────┼");
    }
    println!();
    
    for s in &iteration_stats {
        print!("   {:>4} │", s.iteration);
        for name in &func_names {
            let count = s.cell_analysis.functional_groups.get(*name).copied().unwrap_or(0);
            let pct = count as f64 / s.final_cells as f64 * 100.0;
            print!(" {:>5.1}% │", pct);
        }
        println!();
    }
    println!();
    
    // 最终状态
    let final_analysis = analyze_cells(&stemcell.cells);
    println!("🏁 最终状态:");
    println!("   总细胞: {}", stemcell.cells.len());
    println!("   Fate Tick: {}", stemcell.triple_fate.global_tick);
    println!("   能量分布: {:.4} ± {:.4}", final_analysis.energy_mean, final_analysis.energy_std);
    println!("   信号分布: {:.0} ± {:.0}", final_analysis.signal_mean, final_analysis.signal_std);
    println!("   方差分布: {:.2} ± {:.2}", final_analysis.variance_mean, final_analysis.variance_std);
    println!();
    
    // 器官分化得分
    let diversity_score = calculate_diversity(&stemcell.cells);
    let organ_score = calculate_organ_score(&final_analysis.functional_groups, stemcell.cells.len());
    
    println!("🎯 分化指数:");
    println!("   细胞多样性: {:.2}/10", diversity_score);
    println!("   类器官分化: {:.2}/10", organ_score);
    println!();
    
    // 保存最终状态
    let final_path = "/tmp/stemcell_convergent_final.stemcell";
    if let Err(e) = stemcell.save_slice(Path::new(final_path)) {
        println!("❌ 保存失败: {}", e);
    } else {
        println!("💾 最终状态已保存: {}", final_path);
    }
    
    println!();
    println!("═══════════════════════════════════════════════════════════════════════════════");
}

// ============================================================================
// 数据结构
// ============================================================================

struct IterationStats {
    iteration: u32,
    sub_rounds: u32,
    initial_cells: usize,
    final_cells: usize,
    new_cells: usize,
    converged_count: usize,
    discoveries: usize,
    total_signals: u64,
    fate_tick_delta: u64,
    elapsed_secs: f64,
    cell_analysis: CellAnalysis,
}

struct CellAnalysis {
    functional_groups: HashMap<&'static str, usize>,
    energy_mean: f64,
    energy_std: f64,
    signal_mean: f64,
    signal_std: f64,
    variance_mean: f64,
    variance_std: f64,
}

// ============================================================================
// 分析函数
// ============================================================================

fn analyze_cells(cells: &[Cell]) -> CellAnalysis {
    if cells.is_empty() {
        return CellAnalysis {
            functional_groups: HashMap::new(),
            energy_mean: 0.0,
            energy_std: 0.0,
            signal_mean: 0.0,
            signal_std: 0.0,
            variance_mean: 0.0,
            variance_std: 0.0,
        };
    }
    
    // 统计
    let energies: Vec<f64> = cells.iter().map(|c| c.energy).collect();
    let signals: Vec<f64> = cells.iter().map(|c| c.signal_count as f64).collect();
    let variances: Vec<f64> = cells.iter().map(|c| c.variance).collect();
    
    let (energy_mean, energy_std) = mean_std(&energies);
    let (signal_mean, signal_std) = mean_std(&signals);
    let (variance_mean, variance_std) = mean_std(&variances);
    
    // 功能分类
    let mut groups: HashMap<&'static str, usize> = HashMap::new();
    for c in cells {
        let category = classify_cell(c);
        *groups.entry(category).or_insert(0) += 1;
    }
    
    CellAnalysis {
        functional_groups: groups,
        energy_mean,
        energy_std,
        signal_mean,
        signal_std,
        variance_mean,
        variance_std,
    }
}

fn classify_cell(cell: &Cell) -> &'static str {
    let energy = cell.energy;
    let signals = cell.signal_count;
    let variance = cell.variance;
    
    if energy > 0.8 && signals > 1000 {
        "核心处理"
    } else if energy > 0.6 && variance > 100.0 {
        "信号调节"
    } else if signals > 500 && variance < 10.0 {
        "信号接收"
    } else if energy < 0.3 && signals < 100 {
        "休眠储备"
    } else if variance > 50.0 {
        "高变异"
    } else if energy > 0.5 {
        "活跃处理"
    } else {
        "普通"
    }
}

fn mean_std(values: &[f64]) -> (f64, f64) {
    if values.is_empty() {
        return (0.0, 0.0);
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
    (mean, variance.sqrt())
}

fn calculate_diversity(cells: &[Cell]) -> f64 {
    if cells.is_empty() {
        return 0.0;
    }
    
    let energies: Vec<f64> = cells.iter().map(|c| c.energy).collect();
    let signals: Vec<f64> = cells.iter().map(|c| c.signal_count as f64).collect();
    let variances: Vec<f64> = cells.iter().map(|c| c.variance).collect();
    
    let (e_mean, e_std) = mean_std(&energies);
    let (s_mean, s_std) = mean_std(&signals);
    let (v_mean, v_std) = mean_std(&variances);
    
    let e_cv = if e_mean > 0.001 { e_std / e_mean } else { 0.0 };
    let s_cv = if s_mean > 0.001 { s_std / s_mean } else { 0.0 };
    let v_cv = if v_mean > 0.001 { v_std / v_mean } else { 0.0 };
    
    ((e_cv + s_cv + v_cv) / 3.0 * 10.0).min(10.0)
}

fn calculate_organ_score(groups: &HashMap<&str, usize>, total: usize) -> f64 {
    if groups.is_empty() || total == 0 {
        return 0.0;
    }
    
    let num_groups = groups.len() as f64;
    let mut entropy = 0.0;
    for (_, &count) in groups.iter() {
        let p = count as f64 / total as f64;
        if p > 0.0 {
            entropy -= p * p.ln();
        }
    }
    
    let max_entropy = num_groups.ln();
    let normalized_entropy = if max_entropy > 0.0 { entropy / max_entropy } else { 0.0 };
    let group_score = (num_groups / 7.0).min(1.0);
    
    ((normalized_entropy * 0.6 + group_score * 0.4) * 10.0).min(10.0)
}

// ============================================================================
// 语料加载
// ============================================================================

fn load_librispeech_transcripts() -> Vec<(String, Vec<u8>)> {
    let base = Path::new(EXTERNAL_CORPUS_BASE).join("audio/LibriSpeech");
    let mut corpus = Vec::new();
    load_files_recursive(&base, &mut corpus, "trans.txt", 10);
    corpus
}

fn load_coco_captions() -> Vec<(String, Vec<u8>)> {
    let base = Path::new(EXTERNAL_CORPUS_BASE).join("images/annotations");
    let mut corpus = Vec::new();
    
    for name in &["captions_train2017.json", "captions_val2017.json"] {
        let path = base.join(name);
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let captions = extract_captions_from_json(&content);
                corpus.push((path.display().to_string(), captions.into_bytes()));
            }
        }
    }
    corpus
}

fn extract_captions_from_json(json: &str) -> String {
    let mut result = String::new();
    for line in json.lines() {
        if let Some(start) = line.find("\"caption\"") {
            if let Some(colon) = line[start..].find(':') {
                let after_colon = &line[start + colon + 1..];
                if let Some(quote_start) = after_colon.find('"') {
                    let rest = &after_colon[quote_start + 1..];
                    if let Some(quote_end) = rest.find('"') {
                        result.push_str(&rest[..quote_end]);
                        result.push('\n');
                    }
                }
            }
        }
    }
    result
}

fn load_rust_corpus(base: &Path) -> Vec<(String, Vec<u8>)> {
    let mut corpus = Vec::new();
    load_files_recursive(base, &mut corpus, "rs", 10);
    corpus
}

fn load_cargo_registry(limit: usize) -> Vec<(String, Vec<u8>)> {
    let base = Path::new("/home/jouly/.cargo/registry/src");
    let mut corpus = Vec::new();
    load_files_recursive(base, &mut corpus, "rs", 5);
    if corpus.len() > limit {
        corpus.truncate(limit);
    }
    corpus
}

fn load_files_recursive(path: &Path, corpus: &mut Vec<(String, Vec<u8>)>, ext: &str, max_depth: usize) {
    if max_depth == 0 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(path) else { return };
    
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        
        if name.starts_with('.') || name == "target" || name == "node_modules" {
            continue;
        }
        
        if path.is_dir() {
            load_files_recursive(&path, corpus, ext, max_depth - 1);
        } else if path.extension().and_then(|e| e.to_str()).map(|e| e == ext || name.ends_with(ext)).unwrap_or(false) {
            if let Ok(content) = std::fs::read(&path) {
                corpus.push((path.display().to_string(), content));
            }
        }
    }
}
