//! 🧬 StemCell + Hub Fate 训练
//!
//! 周期 = Fate 收敛一次
//! 全部语料导入，每次迭代训练上次的精炼
//! GPU 加速
//!
//! 四元系统整合:
//! - StemCell 内部使用四元系统产生器官行为
//! - Cell 状态由 BlackHole/MeshBrain/MultiNova/BaseForce 驱动
//! - Hub 的四元系统处理信号路由

use stem_cell::{StemCell, TissueLayer};
use hub_f64::{ValhallaHub, HubRequest, RequestType, Priority};
use std::fs::{self, File};
use std::io::{self, Write, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use std::thread;
use rand::seq::SliceRandom;
use rand::thread_rng;

/// 递归查找所有 .conllu 文件
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

/// 加载全部训练语料
fn load_all_corpus() -> Vec<String> {
    let mut items = Vec::new();
    let corpus_root = Path::new("/run/media/jouly/My Book/valhalla_corpus");
    let valhalla_root = Path::new("/home/jouly/AI/Valhalla");
    
    println!("📦 加载全部训练语料...\n");
    
    // 1. 语言 (句子/词汇/语法)
    print!("  🗣️  语言...");
    io::stdout().flush().unwrap();
    let lang_path = corpus_root.join("languages");
    let languages = ["english", "chinese", "japanese", "korean", "french", 
                     "german", "spanish", "russian", "arabic", "portuguese"];
    
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
                            let sentence = if parts.len() >= 3 { parts[2].trim() } else { line.trim() };
                            if sentence.len() > 5 && sentence.len() < 500 {
                                items.push(sentence.to_string());
                            }
                        }
                    }
                }
            }
        }
        
        // 词汇
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
        
        // 语法
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
    
    // 2. 代码
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
    
    // 3. 对话
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
    
    // 4. 叙事
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
    
    // 5. 图片描述
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
    println!("║     🧬 StemCell + Hub Fate 训练                                ║");
    println!("║     周期 = Fate 收敛 | 累积演化 | GPU 加速 | 70% 资源           ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    
    // 加载语料
    let mut corpus = load_all_corpus();
    if corpus.is_empty() {
        println!("❌ 无数据！");
        return;
    }
    
    // CPU 70%
    let num_cpus = num_cpus::get();
    let active_cpus = (num_cpus as f64 * 0.7).ceil() as usize;
    rayon::ThreadPoolBuilder::new().num_threads(active_cpus).build_global().ok();
    
    // 初始化 Hub (已默认启用 GPU 加速)
    let mut hub = ValhallaHub::new();
    
    // 初始化 StemCell
    let mut stemcell = StemCell::new();
    
    let status = stemcell.get_status();
    let quad_status = stemcell.get_quad_status();
    
    println!("⚙️  配置:");
    println!("   CPU: {} / {} 核心 (70%)", active_cpus, num_cpus);
    println!("   GPU: 启用");
    println!("   StemCell: {} 细胞", status.cell_count);
    println!("   四元系统: {} (已激活)", if status.quad_system_active { "✅" } else { "❌" });
    println!("   细胞统计: 数量={} 连接={} 信号={:.3}",
             status.tissue_stats.cell_count,
             status.tissue_stats.total_connections,
             status.tissue_stats.avg_signal_strength);
    println!("   四元状态: BH密度={:.3} MB连接={:.3} MN活动={:.3} BF协调={:.3}",
             quad_status.blackhole_density,
             quad_status.meshbrain_connectivity,
             quad_status.multinova_activity,
             quad_status.baseforce_coordination);
    println!("   语料: {} 条\n", corpus.len());
    
    let start_time = Instant::now();
    let target_cycles = 10;
    let mut completed_cycles = 0u32;
    let mut total_signals = 0u64;
    let mut cycle_history: Vec<(u32, u64, f64, [f64; 4])> = Vec::new();
    
    let report_interval = 15; // 每15秒报告
    let cpu_limit = 0.7;
    let sleep_duration = Duration::from_micros(((1.0 / cpu_limit - 1.0) * 100.0) as u64);
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔄 开始训练 (周期 1-{})", target_cycles);
    println!("   一个周期 = 处理完全部 {} 条语料一次", corpus.len());
    println!("   收敛检查只在周期结束后进行");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    for cycle in 1..=target_cycles {
        let cycle_start = Instant::now();
        let mut cycle_signals = 0u64;
        let mut last_report = Instant::now();
        
        println!("═══════════════════════════════════════════════════════════════════");
        println!("📍 周期 {}/{} 开始 - 处理全部 {} 条语料", cycle, target_cycles, corpus.len());
        println!("═══════════════════════════════════════════════════════════════════\n");
        
        // 打乱语料顺序
        corpus.shuffle(&mut thread_rng());
        
        // 处理全部语料，不中断
        for (i, content) in corpus.iter().enumerate() {
            // StemCell 演化
            let prefs = hub.get_fate_preferences();
            let env_signal = prefs.affinities.iter().sum::<f64>() / 4.0;
            let evo_result = stemcell.evolve(env_signal);
            
            // 发送信号给 Hub
            let request = HubRequest {
                request_id: format!("sc_c{}i{}", cycle, i),
                request_type: RequestType::Learning,
                intensity: evo_result.average_energy,
                data: Some(serde_json::json!({
                    "content": content,
                    "cells": evo_result.cell_count,
                    "energy": evo_result.average_energy,
                })),
                priority: Priority::Normal,
            };
            
            let _ = hub.process(request);
            
            cycle_signals += 1;
            total_signals += 1;
            
            if sleep_duration.as_micros() > 0 {
                thread::sleep(sleep_duration);
            }
            
            // 命令行监控 (每15秒) - 只显示进度，不检查收敛
            if last_report.elapsed().as_secs() >= report_interval as u64 {
                let prefs = hub.get_fate_preferences();
                let elapsed = cycle_start.elapsed().as_secs_f64();
                let rate = cycle_signals as f64 / elapsed;
                let progress = (i + 1) as f64 / corpus.len() as f64 * 100.0;
                
                // 获取 StemCell 四元系统状态
                let sc_status = stemcell.get_status();
                let sc_quad = stemcell.get_quad_status();
                
                println!("┌─────────────────────────────────────────────────────────────┐");
                println!("│ 周期 {}/{} | 进度 {:.1}% ({}/{}) | 速率 {:.0}/s",
                         cycle, target_cycles, progress, i + 1, corpus.len(), rate);
                println!("├─────────────────────────────────────────────────────────────┤");
                println!("│ 🧬 StemCell: {} 细胞 | 分裂 {} | 死亡 {}",
                         evo_result.cell_count, sc_status.total_divisions, sc_status.total_deaths);
                println!("│ 🔗 连接: {} | 信号强度: {:.3} | 记忆密度: {:.3}",
                         sc_status.tissue_stats.total_connections,
                         sc_status.tissue_stats.avg_signal_strength,
                         sc_status.tissue_stats.avg_memory_density);
                println!("├─────────────────────────────────────────────────────────────┤");
                println!("│ ⚫ 四元系统 (StemCell内部):");
                println!("│    BlackHole密度: {:.4} | MeshBrain连接: {:.4}",
                         sc_quad.blackhole_density, sc_quad.meshbrain_connectivity);
                println!("│    MultiNova活动: {:.4} | BaseForce协调: {:.4}",
                         sc_quad.multinova_activity, sc_quad.baseforce_coordination);
                println!("├─────────────────────────────────────────────────────────────┤");
                println!("│ 🎯 Hub Fate: BH={:.2}% MB={:.2}% MN={:.2}% BF={:.2}%",
                         prefs.affinities[0] * 100.0,
                         prefs.affinities[1] * 100.0,
                         prefs.affinities[2] * 100.0,
                         prefs.affinities[3] * 100.0);
                println!("│ 📝 {}", &content.chars().take(50).collect::<String>());
                println!("└─────────────────────────────────────────────────────────────┘\n");
                
                last_report = Instant::now();
            }
        }
        
        // 周期结束，标记 epoch 完成（允许检查收敛）
        hub.mark_epoch_complete();
        completed_cycles = cycle;
        
        // 检查是否收敛
        let fate_converged = hub.has_fate_converged();
        if fate_converged {
            println!("🎉 Fate 判断: 周期 {} 收敛！(数据处理完毕 + 边际效应递减)\n", cycle);
        }
        
        let prefs = hub.get_fate_preferences();
        let cycle_time = cycle_start.elapsed().as_secs_f64();
        
        cycle_history.push((
            completed_cycles,
            cycle_signals,
            cycle_time,
            [prefs.affinities[0], prefs.affinities[1], prefs.affinities[2], prefs.affinities[3]],
        ));
        
        let final_status = stemcell.get_status();
        let final_quad = stemcell.get_quad_status();
        
        println!("┌─────────────────────────────────────────────────────────────┐");
        println!("│ 📊 周期 {} 完成报告", completed_cycles);
        println!("├─────────────────────────────────────────────────────────────┤");
        println!("│ 信号: {} | 时间: {:.1}s | 速率: {:.0}/s",
                 cycle_signals, cycle_time, cycle_signals as f64 / cycle_time);
        println!("├─────────────────────────────────────────────────────────────┤");
        println!("│ 🧬 StemCell 最终状态:");
        println!("│   细胞数: {} | 分裂总数: {} | 死亡总数: {}",
                 final_status.cell_count, final_status.total_divisions, final_status.total_deaths);
        println!("│   四元系统统计:");
        println!("│     连接: {} | 信号强度: {:.4} | 记忆密度: {:.4}",
                 final_status.tissue_stats.total_connections,
                 final_status.tissue_stats.avg_signal_strength,
                 final_status.tissue_stats.avg_memory_density);
        println!("│     活动级别: {:.4} | 协调度: {:.4}",
                 final_status.tissue_stats.avg_activity_level,
                 final_status.tissue_stats.avg_coordination);
        println!("├─────────────────────────────────────────────────────────────┤");
        println!("│ ⚫ 四元系统状态 (StemCell 内部):");
        println!("│   BlackHole 密度: {:.4}", final_quad.blackhole_density);
        println!("│   MeshBrain 连接: {:.4}", final_quad.meshbrain_connectivity);
        println!("│   MultiNova 活动: {:.4}", final_quad.multinova_activity);
        println!("│   BaseForce 协调: {:.4}", final_quad.baseforce_coordination);
        println!("├─────────────────────────────────────────────────────────────┤");
        println!("│ 🎯 Hub Fate 偏好:");
        println!("│   BlackHole:  {:.4} ({:.2}%)", prefs.affinities[0], prefs.affinities[0] * 100.0);
        println!("│   MeshBrain:  {:.4} ({:.2}%)", prefs.affinities[1], prefs.affinities[1] * 100.0);
        println!("│   MultiNova:  {:.4} ({:.2}%)", prefs.affinities[2], prefs.affinities[2] * 100.0);
        println!("│   BaseForce:  {:.4} ({:.2}%)", prefs.affinities[3], prefs.affinities[3] * 100.0);
        println!("└─────────────────────────────────────────────────────────────┘\n");
    }
    
    // 最终报告
    let total_time = start_time.elapsed();
    
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║                 📊 训练完成                                     ║");
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
    
    // 偏好变化分析
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
    
    println!("\n🏁 训练完成！");
}
