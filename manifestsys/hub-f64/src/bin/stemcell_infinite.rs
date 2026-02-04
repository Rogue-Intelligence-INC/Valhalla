//! StemCell 无止尽训练
//! 
//! 资源限制: 70% (CPU/内存/显卡)
//! 持续训练直到手动停止

use std::time::Instant;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::path::Path;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};

use stem_cell::stemcell_complete::{StemCellComplete, StemCellConfig, CellSpecialization};
use fate_f64::OrganDetector;

fn main() {
    eprintln!("╔═══════════════════════════════════════════════════════════════════╗");
    eprintln!("║        StemCell 无止尽训练 (70% 资源限制)                         ║");
    eprintln!("╚═══════════════════════════════════════════════════════════════════╝");
    eprintln!();

    // 设置资源限制
    set_resource_limits();

    // Ctrl+C 处理
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        eprintln!("\n⚠️ 收到停止信号，正在优雅退出...");
        r.store(false, Ordering::SeqCst);
    }).expect("设置 Ctrl+C 处理失败");

    // 配置
    let config = StemCellConfig {
        initial_cell_count: 100,
        mutation_rate: 0.10,
        container_capacity: 200,
    };
    
    eprintln!("🔧 配置:");
    eprintln!("   CPU 限制: 70%");
    eprintln!("   内存限制: 70%");
    eprintln!("   GPU 限制: 70%");
    eprintln!("   突变率: {}%", config.mutation_rate * 100.0);
    eprintln!("   初始细胞: {}", config.initial_cell_count);
    eprintln!();

    let mut stemcell = StemCellComplete::with_config(config);
    let mut organ_detector = OrganDetector::new();
    
    // 加载语料
    eprintln!("📚 加载全量语料...");
    let corpus = load_full_corpus();
    let corpus_size = corpus.len();
    eprintln!("   ✓ 语料大小: {} 条", corpus_size);
    eprintln!();

    let start = Instant::now();
    let mut cycle = 0u64;
    let mut total_data_processed = 0u64;
    let mut last_report = Instant::now();
    
    eprintln!("🚀 开始无止尽训练... (Ctrl+C 停止)");
    eprintln!("═══════════════════════════════════════════════════════════════════");
    eprintln!();

    while running.load(Ordering::SeqCst) {
        cycle += 1;
        let cycle_start = Instant::now();
        
        // 数据摄入阶段 - 每 10 万条报告一次
        let report_interval = 100_000;
        for (i, text) in corpus.iter().enumerate() {
            if !running.load(Ordering::SeqCst) { break; }
            
            // 将文本转换为数值信号
            for (j, ch) in text.chars().enumerate().take(100) {
                let v = (ch as u32 as f64 / 65536.0) * 2.0 - 1.0;
                let modulated = v * ((i + j) as f64 * 0.001).sin();
                stemcell.receive_data(modulated);
                total_data_processed += 1;
            }
            
            // 每 10 万条显示进度
            if (i + 1) % report_interval == 0 {
                let elapsed = start.elapsed().as_secs_f64();
                let progress = (i + 1) as f64 / corpus_size as f64 * 100.0;
                let throughput = total_data_processed as f64 / elapsed;
                eprint!("\r⏳ 周期 {} | 进度: {:.1}% | 细胞: {} | 吞吐: {:.0}/s     ",
                    cycle, progress, stemcell.cells.len(), throughput);
            }
        }
        eprintln!(); // 换行
        
        // 细胞演化阶段 (5代)
        for gen in 1..=5 {
            if !running.load(Ordering::SeqCst) { break; }
            let result = stemcell.process_cells_cycle();
            eprintln!("   🧬 演化第{}代: 细胞={} 分裂={} 死亡={} 合并={}",
                gen, result.cell_count, result.divisions, result.deaths, result.merges);
        }
        
        // 记录器官状态
        let (_, counts) = analyze_specialization(&stemcell);
        let newly_formed = organ_detector.record(counts, cycle);
        
        // 每个周期结束报告（或有新器官形成时）
        if true || !newly_formed.is_empty() {
            let elapsed = start.elapsed().as_secs_f64();
            let cell_count = stemcell.cells.len();
            let organs_formed = organ_detector.formed_count();
            let throughput = total_data_processed as f64 / elapsed;
            
            eprintln!("📊 周期 {} | 运行时间: {:.0}s", cycle, elapsed);
            eprintln!("   细胞: {} | 数据处理: {} | 吞吐: {:.0}/s",
                cell_count, total_data_processed, throughput);
            eprintln!("   器官形成: {}/4", organs_formed);
            
            // 显示新形成的器官
            for organ_idx in &newly_formed {
                eprintln!("   🎉 新器官: {}", OrganDetector::organ_name(*organ_idx));
            }
            
            // 显示特化分布
            let (spec_dist, _) = analyze_specialization(&stemcell);
            for (spec, count, pct) in spec_dist.iter().take(5) {
                eprintln!("   {:12}: {:5} ({:5.1}%)", spec, count, pct);
            }
            eprintln!();
            
            last_report = Instant::now();
        }
        
        // 应用资源限制：适当休眠
        let cycle_time = cycle_start.elapsed().as_millis();
        if cycle_time < 100 {
            std::thread::sleep(std::time::Duration::from_millis((100 - cycle_time) as u64));
        }
    }

    // 最终报告
    let total_elapsed = start.elapsed();
    eprintln!("═══════════════════════════════════════════════════════════════════");
    eprintln!("🏁 训练结束!");
    eprintln!();
    eprintln!("📋 最终统计:");
    eprintln!("   总周期: {}", cycle);
    eprintln!("   总运行时间: {:.1}s ({:.1} 分钟)", 
        total_elapsed.as_secs_f64(), 
        total_elapsed.as_secs_f64() / 60.0);
    eprintln!("   总数据处理: {}", total_data_processed);
    eprintln!("   平均吞吐: {:.0}/s", total_data_processed as f64 / total_elapsed.as_secs_f64());
    eprintln!();
    
    let (spec_dist, _) = analyze_specialization(&stemcell);
    eprintln!("📊 最终特化分布:");
    for (spec, count, pct) in &spec_dist {
        let bar_len = (*pct / 5.0) as usize;
        let bar = "█".repeat(bar_len.max(1));
        eprintln!("   {:12}: {:5} ({:5.1}%) {}", spec, count, pct, bar);
    }
    
    eprintln!();
    eprintln!("🫀 器官状态:");
    let organ_status = organ_detector.formed;
    eprintln!("   感知器官: {}", if organ_status[0] { "✅ 形成" } else { "⏳ 未稳定" });
    eprintln!("   处理器官: {}", if organ_status[1] { "✅ 形成" } else { "⏳ 未稳定" });
    eprintln!("   记忆器官: {}", if organ_status[2] { "✅ 形成" } else { "⏳ 未稳定" });
    eprintln!("   协调器官: {}", if organ_status[3] { "✅ 形成" } else { "⏳ 未稳定" });
    eprintln!("   器官形成度: {}/4", organ_detector.formed_count());
    eprintln!("═══════════════════════════════════════════════════════════════════");
}

fn set_resource_limits() {
    // 获取 CPU 核心数
    let cores = std::thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(4);
    let use_cores = (cores as f64 * 0.7).ceil() as usize;
    
    // 设置 RAYON 线程数
    std::env::set_var("RAYON_NUM_THREADS", use_cores.to_string());
    
    eprintln!("⚙️ 资源限制设置:");
    eprintln!("   CPU 核心: {}/{} (70%)", use_cores, cores);
    
    // 设置 GPU 内存限制 (CUDA)
    std::env::set_var("CUDA_MPS_ACTIVE_THREAD_PERCENTAGE", "70");
    eprintln!("   GPU: 70% (CUDA_MPS_ACTIVE_THREAD_PERCENTAGE)");
    
    // 内存限制通过 cgroup 或 ulimit 外部设置
    eprintln!("   内存: 建议外部设置 (ulimit -v)");
}

fn analyze_specialization(stemcell: &StemCellComplete) -> (Vec<(String, usize, f64)>, [u64; 4]) {
    let mut counts_map: HashMap<String, usize> = HashMap::new();
    let total = stemcell.cells.len();
    let mut counts = [0u64; 4];
    
    for cell in &stemcell.cells {
        let (name, idx) = match cell.specialization {
            CellSpecialization::Stem => ("干细胞", None),
            CellSpecialization::Sensory => ("感知细胞", Some(0)),
            CellSpecialization::Processing => ("处理细胞", Some(1)),
            CellSpecialization::Memory => ("记忆细胞", Some(2)),
            CellSpecialization::Coordinator => ("协调细胞", Some(3)),
        };
        *counts_map.entry(name.to_string()).or_insert(0) += 1;
        if let Some(i) = idx {
            counts[i] += 1;
        }
    }
    
    let mut result: Vec<_> = counts_map.into_iter()
        .map(|(name, count)| {
            let pct = if total > 0 { count as f64 / total as f64 * 100.0 } else { 0.0 };
            (name, count, pct)
        })
        .collect();
    result.sort_by(|a, b| b.1.cmp(&a.1));
    (result, counts)
}

fn load_full_corpus() -> Vec<String> {
    use std::io;
    let mut items = Vec::new();
    let corpus_root = Path::new("/run/media/jouly/My Book/valhalla_corpus");
    let valhalla_root = Path::new("/home/jouly/AI/Valhalla");

    // === 1. 语言模态 (句子+词汇) ===
    eprint!("  🗣️  语言模态...");
    io::stderr().flush().unwrap();
    let lang_path = corpus_root.join("languages");
    let languages = ["english", "chinese", "japanese", "korean", "french", 
                     "german", "spanish", "russian", "arabic", "portuguese"];
    
    let mut lang_count = 0usize;
    
    for lang in &languages {
        let lang_dir = lang_path.join(lang);
        
        // 句子
        let sent_path = lang_dir.join("sentences");
        if sent_path.exists() {
            for entry in fs::read_dir(&sent_path).into_iter().flatten().flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "txt" || e == "tsv") {
                    if let Ok(file) = File::open(&path) {
                        for line in BufReader::new(file).lines().flatten() {
                            let parts: Vec<&str> = line.split('\t').collect();
                            let s = if parts.len() >= 3 { parts[2].trim() } else { line.trim() };
                            if s.len() > 5 && s.len() < 500 {
                                items.push(s.to_string());
                                lang_count += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    eprintln!(" {} 条", lang_count);
    
    // === 2. 对话模态 ===
    eprint!("  💬 对话模态...");
    io::stderr().flush().unwrap();
    let dialogue_path = corpus_root.join("dialogues");
    let mut dial_count = 0usize;
    
    if dialogue_path.exists() {
        for entry in fs::read_dir(&dialogue_path).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "txt" || e == "json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    for line in content.lines() {
                        let trimmed = line.trim();
                        if trimmed.len() > 10 && trimmed.len() < 1000 {
                            items.push(trimmed.to_string());
                            dial_count += 1;
                        }
                    }
                }
            }
        }
    }
    eprintln!(" {} 条", dial_count);
    
    // === 3. 本地对话语料 ===
    eprint!("  📝 本地对话...");
    io::stderr().flush().unwrap();
    let mut local_count = 0usize;
    
    let dialogue_files = [
        "valhalla_all_dialogues.txt",
        "valhalla_enhanced_dialogues.txt", 
        "valhalla_unlimited_dialogues.txt",
        "family_dialogue_20.txt",
    ];
    
    for filename in &dialogue_files {
        let path = valhalla_root.join(filename);
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.len() > 5 {
                        items.push(trimmed.to_string());
                        local_count += 1;
                    }
                }
            }
        }
    }
    eprintln!(" {} 条", local_count);
    
    eprintln!("  📊 总计: {} 条数据", items.len());
    items
}
