//! 🏛️ Chamber + Hub Fate 训练
//!
//! 周期 = Fate 收敛一次
//! 全部语料导入，每次迭代训练上次的精炼
//! GPU 加速
//!
//! Chamber: 容器编排系统 - 模拟容器创建/销毁/调度

use hub_f64::{ValhallaHub, HubRequest, RequestType, Priority};
use std::fs::{self, File};
use std::io::{self, Write, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use std::thread;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};

/// Chamber 容器类型
#[derive(Debug, Clone, Copy, PartialEq)]
enum ContainerType {
    Compute,
    Storage,
    Network,
    Coordination,
}

/// Chamber 容器状态
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Container {
    id: String,
    container_type: String,
    resource_usage: f64,
    created_at: u64,
    active: bool,
}

/// Chamber 编排系统 (简化版)
#[derive(Debug)]
struct ChamberOrchestrator {
    containers: Vec<Container>,
    total_created: u64,
    total_destroyed: u64,
    tick: u64,
}

impl ChamberOrchestrator {
    fn new() -> Self {
        Self {
            containers: Vec::new(),
            total_created: 0,
            total_destroyed: 0,
            tick: 0,
        }
    }
    
    fn create_container(&mut self, container_type: ContainerType) -> String {
        self.tick += 1;
        self.total_created += 1;
        
        let type_str = match container_type {
            ContainerType::Compute => "compute",
            ContainerType::Storage => "storage",
            ContainerType::Network => "network",
            ContainerType::Coordination => "coordination",
        };
        
        let id = format!("{}_{}", type_str, self.total_created);
        
        self.containers.push(Container {
            id: id.clone(),
            container_type: type_str.to_string(),
            resource_usage: 0.0,
            created_at: self.tick,
            active: true,
        });
        
        id
    }
    
    fn destroy_container(&mut self, id: &str) -> bool {
        if let Some(pos) = self.containers.iter().position(|c| c.id == id && c.active) {
            self.containers[pos].active = false;
            self.total_destroyed += 1;
            true
        } else {
            false
        }
    }
    
    fn update_usage(&mut self, id: &str, usage: f64) {
        if let Some(container) = self.containers.iter_mut().find(|c| c.id == id && c.active) {
            container.resource_usage = usage;
        }
    }
    
    fn active_count(&self) -> usize {
        self.containers.iter().filter(|c| c.active).count()
    }
    
    fn total_usage(&self) -> f64 {
        let active: Vec<_> = self.containers.iter().filter(|c| c.active).collect();
        if active.is_empty() {
            0.0
        } else {
            active.iter().map(|c| c.resource_usage).sum::<f64>() / active.len() as f64
        }
    }
    
    fn collapse(&mut self) {
        self.containers.retain(|c| c.active);
    }
}

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
    println!("║     🏛️ Chamber + Hub Fate 训练                                 ║");
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
    
    let mut chamber = ChamberOrchestrator::new();
    let mut hub = ValhallaHub::new();
    
    println!("⚙️  配置:");
    println!("   CPU: {} / {} 核心 (70%)", active_cpus, num_cpus);
    println!("   GPU: 启用");
    println!("   Chamber: 容器编排系统");
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
                // Chamber 容器操作
                let container_type = match i % 4 {
                    0 => ContainerType::Compute,
                    1 => ContainerType::Storage,
                    2 => ContainerType::Network,
                    _ => ContainerType::Coordination,
                };
                
                let container_id = chamber.create_container(container_type);
                
                // 更新容器使用率
                let usage = (content.len() as f64 / 500.0).min(1.0);
                chamber.update_usage(&container_id, usage);
                
                // 随机销毁部分容器
                if chamber.total_created % 50 == 0 && chamber.active_count() > 10 {
                    let active: Vec<_> = chamber.containers.iter()
                        .filter(|c| c.active)
                        .map(|c| c.id.clone())
                        .collect();
                    if let Some(id) = active.first() {
                        chamber.destroy_container(id);
                    }
                }
                
                // 定期整理
                if chamber.total_created % 500 == 0 {
                    chamber.collapse();
                }
                
                // 发送信号给 Hub
                let request = HubRequest {
                    request_id: format!("chamber_c{}r{}i{}", completed_cycles, round, i),
                    request_type: RequestType::Learning,
                    intensity: chamber.total_usage(),
                    data: Some(serde_json::json!({
                        "content": content,
                        "containers": chamber.active_count(),
                        "usage": chamber.total_usage(),
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
                    
                    println!("┌─────────────────────────────────────────────────────────────┐");
                    println!("│ 周期 {} | 轮次 {} | 信号 {} | 速率 {:.0}/s",
                             completed_cycles + 1, round, cycle_signals, rate);
                    println!("│ 🏛️ 创建:{} 销毁:{} 活跃:{} 使用:{:.1}%",
                             chamber.total_created, chamber.total_destroyed,
                             chamber.active_count(), chamber.total_usage() * 100.0);
                    println!("│ 🎯 BH={:.2}% MB={:.2}% MN={:.2}% BF={:.2}%",
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
        
        println!("┌─────────────────────────────────────────────────────────────┐");
        println!("│ 📊 周期 {} 完成报告", completed_cycles);
        println!("├─────────────────────────────────────────────────────────────┤");
        println!("│ 信号: {} | 时间: {:.1}s | 速率: {:.0}/s",
                 cycle_signals, cycle_time, cycle_signals as f64 / cycle_time);
        println!("│ Chamber: 创建={} 销毁={} 活跃={}",
                 chamber.total_created, chamber.total_destroyed, chamber.active_count());
        println!("│ 最终偏好:");
        println!("│   BlackHole:  {:.4} ({:.2}%)", prefs.affinities[0], prefs.affinities[0] * 100.0);
        println!("│   MeshBrain:  {:.4} ({:.2}%)", prefs.affinities[1], prefs.affinities[1] * 100.0);
        println!("│   MultiNova:  {:.4} ({:.2}%)", prefs.affinities[2], prefs.affinities[2] * 100.0);
        println!("│   BaseForce:  {:.4} ({:.2}%)", prefs.affinities[3], prefs.affinities[3] * 100.0);
        println!("└─────────────────────────────────────────────────────────────┘\n");
    }
    
    let total_time = start_time.elapsed();
    
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║                 📊 Chamber + Fate 训练完成                      ║");
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
    
    println!("\n🏛️ Chamber 最终状态:");
    println!("   总创建: {}", chamber.total_created);
    println!("   总销毁: {}", chamber.total_destroyed);
    println!("   活跃容器: {}", chamber.active_count());
    println!("   平均使用率: {:.1}%", chamber.total_usage() * 100.0);
    
    println!("\n🏁 训练完成！");
}
