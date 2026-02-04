//! 🧱 TileQuattro + Hub Fate 训练
//!
//! 周期 = Fate 收敛一次
//! 全部语料导入，每次迭代训练上次的精炼
//! GPU 加速
//!
//! 四元系统整合 (神经网络模式):
//! - TileQuattro 内部使用四元系统产生神经网络行为
//! - 每个 Tile 作为神经元，有激活值、权重、连接
//! - BlackHole: 权重存储 | MeshBrain: 神经连接 | MultiNova: 并行传播 | BaseForce: 学习协调

use valhalla_tile_quattro::{TileQuattroMemoryManager, AllocationHint};
use hub_f64::{ValhallaHub, HubRequest, RequestType, Priority};
use std::fs::{self, File};
use std::io::{self, Write, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use std::thread;
use rand::seq::SliceRandom;
use rand::thread_rng;

fn find_conllu_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(find_conllu_files(&path));
            } else if path.extension().map_or(false, |e| e == "conllu") {
                files.push(path);
            }
        }
    }
    files
}

fn load_all_corpus() -> Vec<String> {
    let mut items = Vec::new();
    let corpus_root = Path::new("/run/media/jouly/My Book/valhalla_corpus");
    let valhalla_root = Path::new("/home/jouly/AI/Valhalla");
    
    println!("📦 加载全部训练语料...\n");
    
    print!("  🗣️  语言...");
    io::stdout().flush().unwrap();
    let lang_path = corpus_root.join("languages");
    let languages = ["english", "chinese", "japanese", "korean", "french", 
                     "german", "spanish", "russian", "arabic", "portuguese"];
    
    for lang in &languages {
        let lang_dir = lang_path.join(lang);
        
        let sent_path = lang_dir.join("sentences");
        if sent_path.exists() {
            for entry in fs::read_dir(&sent_path).into_iter().flatten().flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "txt" || e == "tsv") {
                    if let Ok(file) = File::open(&path) {
                        for line in BufReader::new(file).lines().flatten() {
                            let parts: Vec<&str> = line.split('\t').collect();
                            let sentence = if parts.len() >= 3 { parts[2].trim() } else { line.trim() };
                            if sentence.len() > 5 && sentence.len() < 500 {
                                items.push(sentence.to_string());
                            }
                        }
                    }
                }
            }
        }
        
        let vocab_path = lang_dir.join("vocabulary");
        if vocab_path.exists() {
            for entry in fs::read_dir(&vocab_path).into_iter().flatten().flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "txt") {
                    if let Ok(file) = File::open(&path) {
                        for line in BufReader::new(file).lines().flatten() {
                            let word = line.split_whitespace().next().unwrap_or("").trim();
                            if word.len() > 1 && !word.starts_with('#') {
                                items.push(word.to_string());
                            }
                        }
                    }
                }
            }
        }
        
        let grammar_path = lang_dir.join("grammar");
        if grammar_path.exists() {
            for path in find_conllu_files(&grammar_path) {
                if let Ok(file) = File::open(&path) {
                    let mut current_text = String::new();
                    for line in BufReader::new(file).lines().flatten() {
                        if line.starts_with("# text = ") {
                            current_text = line[9..].to_string();
                        } else if line.is_empty() && !current_text.is_empty() {
                            if current_text.len() > 5 && current_text.len() < 500 {
                                items.push(current_text.clone());
                            }
                            current_text.clear();
                        }
                    }
                }
            }
        }
    }
    println!(" {} 条", items.len());
    
    print!("  💻 代码...");
    io::stdout().flush().unwrap();
    let code_start = items.len();
    let code_files = ["code/valhalla_rust.txt", "code/valhalla_wave.txt", "code/valhalla_cosmic.txt"];
    for code_file in &code_files {
        let path = corpus_root.join(code_file);
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                for chunk in content.split("---FILE_SEPARATOR---") {
                    for block in chunk.split("\n\n").filter(|s| s.len() > 20 && s.len() < 2000) {
                        items.push(block.to_string());
                    }
                }
            }
        }
    }
    let src_dirs = ["manifestsys/hub-f64/src", "systems/metasystems/fate-f64/src", "valhalla-rational/src"];
    for src_dir in &src_dirs {
        let src_path = valhalla_root.join(src_dir);
        if src_path.exists() {
            for entry in fs::read_dir(&src_path).into_iter().flatten().flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "rs") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        for block in content.split("\n\n").filter(|s| s.len() > 30 && s.len() < 1500) {
                            items.push(block.to_string());
                        }
                    }
                }
            }
        }
    }
    println!(" {} 条", items.len() - code_start);
    
    print!("  💬 对话...");
    io::stdout().flush().unwrap();
    let dialogue_start = items.len();
    let cornell_path = corpus_root.join("dialogues/cornell movie-dialogs corpus/movie_lines.txt");
    if cornell_path.exists() {
        if let Ok(file) = File::open(&cornell_path) {
            for line in BufReader::new(file).lines().flatten() {
                let parts: Vec<&str> = line.split(" +++$+++ ").collect();
                if parts.len() >= 5 {
                    let content = parts[4].trim();
                    if content.len() > 5 && content.len() < 500 {
                        items.push(content.to_string());
                    }
                }
            }
        }
    }
    let convai_path = corpus_root.join("dialogues/convai/train_both_original.txt");
    if convai_path.exists() {
        if let Ok(file) = File::open(&convai_path) {
            for line in BufReader::new(file).lines().flatten() {
                let content = line.trim();
                if content.len() > 10 && content.len() < 500 && !content.starts_with("your persona:") {
                    items.push(content.to_string());
                }
            }
        }
    }
    println!(" {} 条", items.len() - dialogue_start);
    
    print!("  📖 叙事...");
    io::stdout().flush().unwrap();
    let narrative_start = items.len();
    let gutenberg_path = corpus_root.join("novels/gutenberg");
    if gutenberg_path.exists() {
        for entry in fs::read_dir(&gutenberg_path).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "txt") {
                if let Ok(file) = File::open(&path) {
                    let mut paragraph = String::new();
                    for line in BufReader::new(file).lines().flatten() {
                        if line.trim().is_empty() && !paragraph.is_empty() {
                            if paragraph.len() > 50 && paragraph.len() < 1000 {
                                items.push(paragraph.clone());
                            }
                            paragraph.clear();
                        } else {
                            paragraph.push_str(&line);
                            paragraph.push(' ');
                        }
                    }
                }
            }
        }
    }
    println!(" {} 条", items.len() - narrative_start);
    
    print!("  🖼️  图片描述...");
    io::stdout().flush().unwrap();
    let image_start = items.len();
    let coco_path = corpus_root.join("images/coco/annotations/captions_train2017.json");
    if coco_path.exists() {
        if let Ok(content) = fs::read_to_string(&coco_path) {
            for line in content.lines() {
                if let Some(start) = line.find("\"caption\":") {
                    let rest = &line[start + 11..];
                    if let Some(end) = rest.find('"') {
                        let caption = &rest[..end];
                        if caption.len() > 10 && caption.len() < 500 {
                            items.push(caption.to_string());
                        }
                    }
                }
            }
        }
    }
    println!(" {} 条", items.len() - image_start);
    
    println!("\n📊 总计: {} 条训练数据\n", items.len());
    items
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║     🧱 TileQuattro + Hub Fate 训练                             ║");
    println!("║     周期 = Fate 收敛 | 累积演化 | GPU 加速 | 70% 资源           ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    
    let mut corpus = load_all_corpus();
    if corpus.is_empty() {
        println!("❌ 无数据！");
        return;
    }
    
    let num_cpus = num_cpus::get();
    let active_cpus = (num_cpus as f64 * 0.7).ceil() as usize;
    rayon::ThreadPoolBuilder::new().num_threads(active_cpus).build_global().ok();
    
    let mut tile = TileQuattroMemoryManager::new();
    let mut hub = ValhallaHub::new();
    
    let stats = tile.get_stats();
    let quad_status = tile.get_quad_status();
    
    println!("⚙️  配置:");
    println!("   CPU: {} / {} 核心 (70%)", active_cpus, num_cpus);
    println!("   GPU: 启用");
    println!("   TileQuattro: 总容量 {}MB", tile.config.total_size());
    println!("   四元系统: {} (已激活)", if stats.quad_system_active { "✅" } else { "❌" });
    println!("   神经网络: Dense={} Recurrent={} Attention={} Embedding={}",
             stats.network_stats.dense_count,
             stats.network_stats.recurrent_count,
             stats.network_stats.attention_count,
             stats.network_stats.embedding_count);
    println!("   四元状态: BH引力={:.3} MB智能={:.3} MN辐射={:.3} BF协调={:.3}",
             quad_status.blackhole_gravity,
             quad_status.meshbrain_intelligence,
             quad_status.multinova_radiation,
             quad_status.baseforce_coordination);
    println!("   语料: {} 条\n", corpus.len());
    
    let start_time = Instant::now();
    let target_cycles = 10;
    let mut completed_cycles = 0u32;
    let mut total_signals = 0u64;
    let mut cycle_history: Vec<(u32, u64, f64, [f64; 4])> = Vec::new();
    
    let report_interval = 15;
    let cpu_limit = 0.7;
    let sleep_duration = Duration::from_micros(((1.0 / cpu_limit - 1.0) * 100.0) as u64);
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔄 开始训练 (周期 1-{})", target_cycles);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    while completed_cycles < target_cycles {
        let cycle_start = Instant::now();
        let mut cycle_signals = 0u64;
        let mut last_report = Instant::now();
        let mut round = 0usize;
        
        println!("═══════════════════════════════════════════════════════════════════");
        println!("📍 周期 {}/{} 开始", completed_cycles + 1, target_cycles);
        println!("═══════════════════════════════════════════════════════════════════\n");
        
        loop {
            round += 1;
            corpus.shuffle(&mut thread_rng());
            
            for (i, content) in corpus.iter().enumerate() {
                // TileQuattro 内存操作 (神经网络构建)
                let size = (content.len() as f64 / 10.0).max(1.0).min(128.0);
                if let Some(block_id) = tile.allocate(size, AllocationHint::Auto) {
                    // 随机释放部分块以模拟内存周转
                    if tile.total_allocations % 100 == 0 {
                        tile.deallocate(&block_id);
                    }
                }
                
                // 神经网络前向传播 (四元系统驱动)
                let signal_intensity = content.len() as f64 / 100.0;
                let prop_result = tile.propagate(signal_intensity);
                
                // 定期整理
                if tile.total_allocations % 1000 == 0 {
                    tile.collapse();
                }
                
                let stats = tile.get_stats();
                
                // 发送信号给 Hub
                let request = HubRequest {
                    request_id: format!("tile_c{}r{}i{}", completed_cycles, round, i),
                    request_type: RequestType::Learning,
                    intensity: stats.total_usage,
                    data: Some(serde_json::json!({
                        "content": content,
                        "usage": stats.total_usage,
                        "fragmentation": stats.total_fragmentation,
                        "dimension": stats.space_dimension,
                    })),
                    priority: Priority::Normal,
                };
                
                let _ = hub.process(request);
                
                cycle_signals += 1;
                total_signals += 1;
                
                if sleep_duration.as_micros() > 0 {
                    thread::sleep(sleep_duration);
                }
                
                if last_report.elapsed().as_secs() >= report_interval as u64 {
                    hub.record_fate_snapshot();
                    
                    let prefs = hub.get_fate_preferences();
                    let pattern = hub.get_fate_convergence_pattern().unwrap_or("探索中");
                    let fate_converged = hub.has_fate_converged();
                    let elapsed = cycle_start.elapsed().as_secs_f64();
                    let rate = cycle_signals as f64 / elapsed;
                    
                    // 获取 TileQuattro 四元系统状态
                    let quad_status = tile.get_quad_status();
                    
                    println!("┌─────────────────────────────────────────────────────────────┐");
                    println!("│ 周期 {} | 轮次 {} | 信号 {} | 速率 {:.0}/s",
                             completed_cycles + 1, round, cycle_signals, rate);
                    println!("├─────────────────────────────────────────────────────────────┤");
                    println!("│ 🧱 TileQuattro: 分配:{} 释放:{} 使用:{:.1}% 碎片:{:.1}%",
                             stats.total_allocations, stats.total_deallocations,
                             stats.total_usage * 100.0, stats.total_fragmentation * 100.0);
                    println!("│ 🧠 神经网络: 神经元={} 连接={} 平均激活={:.4}",
                             stats.network_stats.total_neurons,
                             stats.network_stats.total_connections,
                             stats.network_stats.avg_activation);
                    println!("│    类型: Dense={} Recurrent={} Attention={} Embed={}",
                             stats.network_stats.dense_count,
                             stats.network_stats.recurrent_count,
                             stats.network_stats.attention_count,
                             stats.network_stats.embedding_count);
                    println!("├─────────────────────────────────────────────────────────────┤");
                    println!("│ ⚫ 四元系统 (TileQuattro内部):");
                    println!("│    BlackHole引力: {:.4} | MeshBrain智能: {:.4}",
                             quad_status.blackhole_gravity, quad_status.meshbrain_intelligence);
                    println!("│    MultiNova辐射: {:.4} | BaseForce协调: {:.4}",
                             quad_status.multinova_radiation, quad_status.baseforce_coordination);
                    println!("│ 🔄 前向传播: 激活={:.4} 引力={:.4} 协调={:.4}",
                             prop_result.avg_activation, prop_result.gravity_effect, prop_result.coordination);
                    println!("├─────────────────────────────────────────────────────────────┤");
                    println!("│ 🎯 Hub Fate: BH={:.2}% MB={:.2}% MN={:.2}% BF={:.2}%",
                             prefs.affinities[0] * 100.0,
                             prefs.affinities[1] * 100.0,
                             prefs.affinities[2] * 100.0,
                             prefs.affinities[3] * 100.0);
                    println!("│ 📐 模式: {} | Fate 收敛: {}", pattern, if fate_converged { "是" } else { "否" });
                    println!("│ 📝 {}", &content.chars().take(50).collect::<String>());
                    println!("└─────────────────────────────────────────────────────────────┘\n");
                    
                    last_report = Instant::now();
                    
                    if fate_converged {
                        println!("🎉 Fate 判断: 周期 {} 收敛！\n", completed_cycles + 1);
                        break;
                    }
                }
            }
            
            if hub.has_fate_converged() {
                break;
            }
            
            if round > 30 {
                println!("⚠️ 周期 {} 达到最大轮次\n", completed_cycles + 1);
                break;
            }
        }
        
        completed_cycles += 1;
        
        let prefs = hub.get_fate_preferences();
        let cycle_time = cycle_start.elapsed().as_secs_f64();
        
        cycle_history.push((
            completed_cycles,
            cycle_signals,
            cycle_time,
            [prefs.affinities[0], prefs.affinities[1], prefs.affinities[2], prefs.affinities[3]],
        ));
        
        let final_stats = tile.get_stats();
        let final_quad = tile.get_quad_status();
        
        println!("┌─────────────────────────────────────────────────────────────┐");
        println!("│ 📊 周期 {} 完成报告", completed_cycles);
        println!("├─────────────────────────────────────────────────────────────┤");
        println!("│ 信号: {} | 时间: {:.1}s | 速率: {:.0}/s",
                 cycle_signals, cycle_time, cycle_signals as f64 / cycle_time);
        println!("├─────────────────────────────────────────────────────────────┤");
        println!("│ 🧱 TileQuattro 最终状态:");
        println!("│   分配总数: {} | 释放总数: {} | 活跃块: {}",
                 final_stats.total_allocations, final_stats.total_deallocations, final_stats.active_blocks);
        println!("│   使用率: {:.1}% | 碎片化: {:.1}% | Space维度: {:.2}",
                 final_stats.total_usage * 100.0, final_stats.total_fragmentation * 100.0, final_stats.space_dimension);
        println!("├─────────────────────────────────────────────────────────────┤");
        println!("│ 🧠 神经网络状态:");
        println!("│   神经元总数: {} | 连接总数: {}",
                 final_stats.network_stats.total_neurons, final_stats.network_stats.total_connections);
        println!("│   平均激活: {:.4} | 权重质量: {:.4}",
                 final_stats.network_stats.avg_activation, final_stats.network_stats.total_weight_mass);
        println!("│   类型分布:");
        println!("│     Dense: {} | Recurrent: {} | Attention: {} | Embedding: {}",
                 final_stats.network_stats.dense_count, final_stats.network_stats.recurrent_count,
                 final_stats.network_stats.attention_count, final_stats.network_stats.embedding_count);
        println!("├─────────────────────────────────────────────────────────────┤");
        println!("│ ⚫ 四元系统状态 (TileQuattro 内部):");
        println!("│   BlackHole 引力: {:.4}", final_quad.blackhole_gravity);
        println!("│   MeshBrain 智能: {:.4}", final_quad.meshbrain_intelligence);
        println!("│   MultiNova 辐射: {:.4}", final_quad.multinova_radiation);
        println!("│   BaseForce 协调: {:.4}", final_quad.baseforce_coordination);
        println!("├─────────────────────────────────────────────────────────────┤");
        println!("│ 🎯 Hub Fate 偏好:");
        println!("│   BlackHole:  {:.4} ({:.2}%)", prefs.affinities[0], prefs.affinities[0] * 100.0);
        println!("│   MeshBrain:  {:.4} ({:.2}%)", prefs.affinities[1], prefs.affinities[1] * 100.0);
        println!("│   MultiNova:  {:.4} ({:.2}%)", prefs.affinities[2], prefs.affinities[2] * 100.0);
        println!("│   BaseForce:  {:.4} ({:.2}%)", prefs.affinities[3], prefs.affinities[3] * 100.0);
        println!("└─────────────────────────────────────────────────────────────┘\n");
    }
    
    let total_time = start_time.elapsed();
    
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║                 📊 TileQuattro + Fate 训练完成                  ║");
    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("║ ⏱️  总时间: {:.2}s ({:.1}分钟)", total_time.as_secs_f64(), total_time.as_secs_f64() / 60.0);
    println!("║ 🔢 总信号: {}", total_signals);
    println!("║ 🔄 完成周期: {}", completed_cycles);
    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("║ 📈 跨周期偏好演化:");
    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("║ 周期 │ BlackHole │ MeshBrain │ MultiNova │ BaseForce │ 信号数  ║");
    println!("╠════════════════════════════════════════════════════════════════╣");
    
    for (cycle, signals, _time, prefs) in &cycle_history {
        println!("║  {}   │  {:.2}%   │  {:.2}%   │  {:.2}%   │  {:.2}%   │ {:>7} ║",
                 cycle,
                 prefs[0] * 100.0, prefs[1] * 100.0,
                 prefs[2] * 100.0, prefs[3] * 100.0,
                 signals);
    }
    
    println!("╚════════════════════════════════════════════════════════════════╝");
    
    if cycle_history.len() >= 2 {
        let first = &cycle_history[0].3;
        let last = &cycle_history[cycle_history.len() - 1].3;
        
        println!("\n💡 跨周期变化分析:");
        println!("   BlackHole: {:.2}% → {:.2}% (Δ{:+.2}%)",
                 first[0] * 100.0, last[0] * 100.0, (last[0] - first[0]) * 100.0);
        println!("   MeshBrain: {:.2}% → {:.2}% (Δ{:+.2}%)",
                 first[1] * 100.0, last[1] * 100.0, (last[1] - first[1]) * 100.0);
        println!("   MultiNova: {:.2}% → {:.2}% (Δ{:+.2}%)",
                 first[2] * 100.0, last[2] * 100.0, (last[2] - first[2]) * 100.0);
        println!("   BaseForce: {:.2}% → {:.2}% (Δ{:+.2}%)",
                 first[3] * 100.0, last[3] * 100.0, (last[3] - first[3]) * 100.0);
    }
    
    let final_stats = tile.get_stats();
    let final_quad = tile.get_quad_status();
    
    println!("\n🧱 TileQuattro 最终状态:");
    println!("   总分配: {}", final_stats.total_allocations);
    println!("   总释放: {}", final_stats.total_deallocations);
    println!("   活跃块: {}", final_stats.active_blocks);
    println!("   Space 维度: {:.2}", final_stats.space_dimension);
    
    println!("\n🧠 神经网络最终状态:");
    println!("   神经元总数: {}", final_stats.network_stats.total_neurons);
    println!("   连接总数: {}", final_stats.network_stats.total_connections);
    println!("   平均激活: {:.4}", final_stats.network_stats.avg_activation);
    println!("   权重质量: {:.4}", final_stats.network_stats.total_weight_mass);
    println!("   类型分布: Dense={} Recurrent={} Attention={} Embedding={}",
             final_stats.network_stats.dense_count,
             final_stats.network_stats.recurrent_count,
             final_stats.network_stats.attention_count,
             final_stats.network_stats.embedding_count);
    
    println!("\n⚫ 四元系统最终状态:");
    println!("   BlackHole 引力: {:.4}", final_quad.blackhole_gravity);
    println!("   MeshBrain 智能: {:.4}", final_quad.meshbrain_intelligence);
    println!("   MultiNova 辐射: {:.4}", final_quad.multinova_radiation);
    println!("   BaseForce 协调: {:.4}", final_quad.baseforce_coordination);
    
    println!("\n🏁 训练完成！");
}
