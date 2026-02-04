//! 🚀 Valhalla f64 完整训练
//!
//! - 70% 计算机资源
//! - 完整语料 (词汇/语法/句子/对话/小说/剧本/问答)
//! - Valhalla 内部循环5次（处理完重新进去）
//! - Fate 自主演化

use hub_f64::{
    ValhallaHub, HubConfig, HubRequest, RequestType, Priority,
};
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::time::{Duration, Instant};
use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use rand::seq::SliceRandom;

/// 训练配置
struct TrainingConfig {
    cpu_limit: f64,
    internal_cycles: usize,  // Valhalla 内部循环次数
    corpus_root: String,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            cpu_limit: 0.7,
            internal_cycles: 5,  // 内部循环5次
            corpus_root: "/run/media/jouly/My Book/valhalla_corpus".to_string(),
        }
    }
}

/// 语料项
#[derive(Clone, Debug)]
struct CorpusItem {
    content: String,
    source: String,      // 来源类型
    item_type: ItemType,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum ItemType {
    Vocabulary,   // 词汇
    Grammar,      // 语法
    Sentence,     // 句子
    Dialogue,     // 对话
    Novel,        // 小说
    Script,       // 剧本
    QA,           // 问答
    Article,      // 文章
}

/// 加载所有语料
fn load_all_corpus(root: &str) -> Vec<CorpusItem> {
    let mut items = Vec::new();
    let root = Path::new(root);
    
    // === 1. 加载语言语料 (词汇/语法/句子) ===
    println!("  📚 加载语言语料...");
    let lang_dir = root.join("languages");
    if lang_dir.exists() {
        let languages = ["chinese", "english", "japanese", "korean", "french", 
                         "german", "spanish", "russian", "arabic", "portuguese"];
        
        for lang in languages {
            let lang_path = lang_dir.join(lang);
            
            // 词汇
            let vocab_dir = lang_path.join("vocabulary");
            if vocab_dir.exists() {
                for entry in fs::read_dir(&vocab_dir).into_iter().flatten().flatten() {
                    if entry.path().extension().map_or(false, |e| e == "txt") {
                        if let Ok(file) = File::open(entry.path()) {
                            for line in BufReader::new(file).lines().take(10000).flatten() {
                                let word = line.split_whitespace().next().unwrap_or("").trim();
                                if word.len() > 1 && !word.starts_with('#') {
                                    items.push(CorpusItem {
                                        content: word.to_string(),
                                        source: format!("vocab_{}", lang),
                                        item_type: ItemType::Vocabulary,
                                    });
                                }
                            }
                        }
                    }
                }
            }
            
            // 句子
            let sent_dir = lang_path.join("sentences");
            if sent_dir.exists() {
                for entry in fs::read_dir(&sent_dir).into_iter().flatten().flatten() {
                    let path = entry.path();
                    let is_txt = path.extension().map_or(false, |e| e == "txt" || e == "tsv");
                    if is_txt {
                        if let Ok(file) = File::open(&path) {
                            for line in BufReader::new(file).lines().take(20000).flatten() {
                                let parts: Vec<&str> = line.split('\t').collect();
                                let sentence = if parts.len() >= 3 {
                                    parts[2].trim().to_string()
                                } else {
                                    line.trim().to_string()
                                };
                                if sentence.len() > 5 && sentence.len() < 500 {
                                    items.push(CorpusItem {
                                        content: sentence,
                                        source: format!("sent_{}", lang),
                                        item_type: ItemType::Sentence,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    let lang_count = items.len();
    println!("    语言语料: {} 项", lang_count);
    
    // === 2. 加载内容语料 ===
    println!("  📖 加载内容语料...");
    let content_dir = root.join("content");
    
    // 对话
    let dialogue_dir = content_dir.join("dialogues");
    if dialogue_dir.exists() {
        for entry in fs::read_dir(&dialogue_dir).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    // 简单提取对话文本
                    for line in content.lines().take(50000) {
                        let line = line.trim();
                        if line.len() > 10 && line.len() < 1000 {
                            // 提取引号内的对话
                            if let Some(start) = line.find('"') {
                                if let Some(end) = line[start+1..].find('"') {
                                    let text = &line[start+1..start+1+end];
                                    if text.len() > 5 {
                                        items.push(CorpusItem {
                                            content: text.to_string(),
                                            source: "dialogue".to_string(),
                                            item_type: ItemType::Dialogue,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    let dialogue_count = items.len() - lang_count;
    println!("    对话: {} 项", dialogue_count);
    
    // 小说
    let novel_dir = content_dir.join("novels");
    if novel_dir.exists() {
        for entry in fs::read_dir(&novel_dir).into_iter().flatten().flatten() {
            if entry.path().extension().map_or(false, |e| e == "txt") {
                if let Ok(file) = File::open(entry.path()) {
                    let mut paragraph = String::new();
                    for line in BufReader::new(file).lines().take(50000).flatten() {
                        let line = line.trim();
                        if line.is_empty() {
                            if paragraph.len() > 50 && paragraph.len() < 2000 {
                                items.push(CorpusItem {
                                    content: paragraph.clone(),
                                    source: "novel".to_string(),
                                    item_type: ItemType::Novel,
                                });
                            }
                            paragraph.clear();
                        } else {
                            if !paragraph.is_empty() {
                                paragraph.push(' ');
                            }
                            paragraph.push_str(line);
                        }
                    }
                }
            }
        }
    }
    let novel_count = items.len() - lang_count - dialogue_count;
    println!("    小说: {} 项", novel_count);
    
    // 剧本
    let script_dir = content_dir.join("scripts");
    let movie_lines = script_dir.join("cornell movie-dialogs corpus/movie_lines.txt");
    if movie_lines.exists() {
        if let Ok(file) = File::open(&movie_lines) {
            for line in BufReader::new(file).lines().take(50000).flatten() {
                let parts: Vec<&str> = line.split(" +++$+++ ").collect();
                if parts.len() >= 5 {
                    let text = parts[4].trim();
                    if text.len() > 5 && text.len() < 500 {
                        items.push(CorpusItem {
                            content: text.to_string(),
                            source: "script".to_string(),
                            item_type: ItemType::Script,
                        });
                    }
                }
            }
        }
    }
    let script_count = items.len() - lang_count - dialogue_count - novel_count;
    println!("    剧本: {} 项", script_count);
    
    // 问答 (SQuAD)
    let qa_file = content_dir.join("qa/squad_train.json");
    if qa_file.exists() {
        if let Ok(content) = fs::read_to_string(&qa_file) {
            // 简单提取问答
            for line in content.lines() {
                if line.contains("\"question\"") || line.contains("\"text\"") {
                    if let Some(start) = line.find(": \"") {
                        if let Some(end) = line[start+3..].find('"') {
                            let text = &line[start+3..start+3+end];
                            if text.len() > 5 && text.len() < 500 {
                                items.push(CorpusItem {
                                    content: text.to_string(),
                                    source: "qa".to_string(),
                                    item_type: ItemType::QA,
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    let qa_count = items.len() - lang_count - dialogue_count - novel_count - script_count;
    println!("    问答: {} 项", qa_count);
    
    // 新闻
    let news_file = content_dir.join("articles/ag_news_train.csv");
    if news_file.exists() {
        if let Ok(file) = File::open(&news_file) {
            for line in BufReader::new(file).lines().skip(1).take(30000).flatten() {
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() >= 3 {
                    let text = parts[2..].join(",").trim_matches('"').to_string();
                    if text.len() > 20 && text.len() < 1000 {
                        items.push(CorpusItem {
                            content: text,
                            source: "news".to_string(),
                            item_type: ItemType::Article,
                        });
                    }
                }
            }
        }
    }
    let news_count = items.len() - lang_count - dialogue_count - novel_count - script_count - qa_count;
    println!("    新闻: {} 项", news_count);
    
    items
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║     🚀 Valhalla f64 完整训练                                   ║");
    println!("║     内部循环5次 | 70% 资源 | Fate 自主演化                     ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    
    let config = TrainingConfig::default();
    let running = Arc::new(AtomicBool::new(true));
    
    // Ctrl+C 处理
    let running_clone = running.clone();
    ctrlc::set_handler(move || {
        println!("\n⚠️  收到停止信号...");
        running_clone.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");
    
    // 初始化 Hub
    println!("🔧 初始化 Valhalla Hub (f64, 70% 资源)...");
    let hub_config = HubConfig {
        resource_percent: 70,
        batch_size: 1024,
        ..Default::default()
    };
    let mut hub = ValhallaHub::with_config(hub_config);
    
    // 加载所有语料
    println!("\n📦 加载完整语料库...");
    let mut corpus = load_all_corpus(&config.corpus_root);
    
    println!("\n   ✅ 总计: {} 项", corpus.len());
    
    if corpus.is_empty() {
        println!("❌ 没有找到语料数据！");
        return;
    }
    
    // 打乱语料
    println!("   🔀 打乱语料顺序...");
    let mut rng = rand::thread_rng();
    corpus.shuffle(&mut rng);
    
    let total_items = corpus.len();
    let sleep_duration = Duration::from_micros((30.0 / config.cpu_limit) as u64);
    let total_start = Instant::now();
    
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  🔄 开始 Valhalla 内部循环训练 (共 {} 次)", config.internal_cycles);
    println!("  📊 每次处理 {} 项语料", total_items);
    println!("═══════════════════════════════════════════════════════════════\n");
    
    // ========================================
    // Valhalla 内部循环 - 处理完重新进去
    // ========================================
    for cycle in 1..=config.internal_cycles {
        if !running.load(Ordering::Relaxed) {
            break;
        }
        
        let cycle_start = Instant::now();
        println!("┌─────────────────────────────────────────────────────────────┐");
        println!("│  🔄 内部循环 {}/{}                                          │", cycle, config.internal_cycles);
        println!("└─────────────────────────────────────────────────────────────┘");
        
        let mut processed = 0usize;
        let mut type_counts = [0usize; 8];
        let report_interval = total_items / 10;
        
        // 处理所有语料（完整遍历一次）
        for (idx, item) in corpus.iter().enumerate() {
            if !running.load(Ordering::Relaxed) {
                break;
            }
            
            // 安全截取内容
            let content_preview: String = item.content.chars().take(100).collect();
            let intensity = (item.content.len() as f64 / 200.0).min(1.0).max(0.1);
            
            let request_type = match item.item_type {
                ItemType::Vocabulary => RequestType::Learning,
                ItemType::Grammar => RequestType::Reasoning,
                ItemType::Sentence => RequestType::DataProcessing,
                ItemType::Dialogue => RequestType::Adaptation,
                ItemType::Novel => RequestType::DataProcessing,
                ItemType::Script => RequestType::Adaptation,
                ItemType::QA => RequestType::Reasoning,
                ItemType::Article => RequestType::Learning,
            };
            
            let request = HubRequest {
                request_id: format!("c{}_i{}", cycle, idx),
                request_type,
                intensity,
                data: Some(serde_json::json!({
                    "content": content_preview,
                    "source": &item.source,
                    "type": format!("{:?}", item.item_type)
                })),
                priority: Priority::Normal,
            };
            
            hub.process(request);
            processed += 1;
            type_counts[item.item_type as usize] += 1;
            
            // 进度报告
            if processed % report_interval == 0 {
                let progress = (processed as f64 / total_items as f64) * 100.0;
                let prefs = hub.get_fate_preferences();
                print!("\r  进度: {:>5.1}% | BH:{:.1}% MB:{:.1}% MN:{:.1}% BF:{:.1}%    ",
                    progress,
                    prefs.affinities.get(0).unwrap_or(&0.0) * 100.0,
                    prefs.affinities.get(1).unwrap_or(&0.0) * 100.0,
                    prefs.affinities.get(2).unwrap_or(&0.0) * 100.0,
                    prefs.affinities.get(3).unwrap_or(&0.0) * 100.0,
                );
                std::io::Write::flush(&mut std::io::stdout()).ok();
            }
            
            thread::sleep(sleep_duration);
        }
        
        // 循环结束报告
        let cycle_elapsed = cycle_start.elapsed();
        let prefs = hub.get_fate_preferences();
        let stats = hub.get_statistics();
        
        println!("\n");
        println!("╔════════════════════════════════════════════════════════════════╗");
        println!("║  ✅ 循环 {}/{} 完成                                            ║", cycle, config.internal_cycles);
        println!("╠════════════════════════════════════════════════════════════════╣");
        println!("║  ⏱️  用时: {}分 {}秒                                           ║", 
                 cycle_elapsed.as_secs() / 60, cycle_elapsed.as_secs() % 60);
        println!("║  📊 处理: {} 项                                               ║", processed);
        println!("╠════════════════════════════════════════════════════════════════╣");
        println!("║  📚 词汇: {:>8} | 📖 语法: {:>8}                          ║", 
                 type_counts[0], type_counts[1]);
        println!("║  📝 句子: {:>8} | 💬 对话: {:>8}                          ║", 
                 type_counts[2], type_counts[3]);
        println!("║  📕 小说: {:>8} | 🎬 剧本: {:>8}                          ║", 
                 type_counts[4], type_counts[5]);
        println!("║  ❓ 问答: {:>8} | 📰 新闻: {:>8}                          ║", 
                 type_counts[6], type_counts[7]);
        println!("╠════════════════════════════════════════════════════════════════╣");
        println!("║  🎯 Fate 偏好:                                                 ║");
        println!("║     BlackHole:  {:>8.4}%                                     ║", prefs.affinities.get(0).unwrap_or(&0.0) * 100.0);
        println!("║     MeshBrain:  {:>8.4}%                                     ║", prefs.affinities.get(1).unwrap_or(&0.0) * 100.0);
        println!("║     MultiNova:  {:>8.4}%                                     ║", prefs.affinities.get(2).unwrap_or(&0.0) * 100.0);
        println!("║     BaseForce:  {:>8.4}%                                     ║", prefs.affinities.get(3).unwrap_or(&0.0) * 100.0);
        println!("╠════════════════════════════════════════════════════════════════╣");
        println!("║  ⚡ Fate tick: {}                                            ║", hub.fate_system().signal_tick);
        println!("║  📬 总请求: {}                                               ║", stats.total_requests);
        println!("╚════════════════════════════════════════════════════════════════╝\n");
    }
    
    // 最终报告
    let total_elapsed = total_start.elapsed();
    let final_prefs = hub.get_fate_preferences();
    let final_stats = hub.get_statistics();
    
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║                    🏁 全部训练完成                              ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!("\n⏱️  总用时: {}分 {}秒", total_elapsed.as_secs() / 60, total_elapsed.as_secs() % 60);
    println!("📊 总信号: {}", final_stats.total_requests);
    println!("🔄 内部循环: {} 次", config.internal_cycles);
    println!("📦 语料总量: {} 项", total_items);
    println!("\n🎯 最终 Fate 偏好:");
    println!("   BlackHole:  {:.4}%", final_prefs.affinities.get(0).unwrap_or(&0.0) * 100.0);
    println!("   MeshBrain:  {:.4}%", final_prefs.affinities.get(1).unwrap_or(&0.0) * 100.0);
    println!("   MultiNova:  {:.4}%", final_prefs.affinities.get(2).unwrap_or(&0.0) * 100.0);
    println!("   BaseForce:  {:.4}%", final_prefs.affinities.get(3).unwrap_or(&0.0) * 100.0);
    println!();
}
