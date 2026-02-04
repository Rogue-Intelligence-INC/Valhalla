//! Fate 自主训练 - 代数形式收敛检测
//!
//! 训练直到 Fate 自己判断收敛 (边际效应递减)
//! 不使用硬编码阈值

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::thread;
use std::path::Path;
use std::fs;

use rand::prelude::*;
use hub_f64::{
    ValhallaHub, HubConfig, HubRequest, RequestType, Priority,
};

/// 数据项
enum DataItem {
    Text { content: String },
    Vocab { word: String },
    Image { features: Vec<f64> },
    Audio { features: Vec<f64> },
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║     🌌 Valhalla Fate 自主训练                                  ║");
    println!("║     代数形式收敛检测 | 70% 资源                                ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    
    ctrlc::set_handler(move || {
        println!("\n⚠️  收到停止信号...");
        running_clone.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    // 初始化 Hub
    println!("🔧 初始化 Valhalla Hub...");
    let hub_config = HubConfig {
        resource_percent: 70,
        batch_size: 256,
        ..Default::default()
    };
    let mut hub = ValhallaHub::with_config(hub_config);
    println!("   ✅ Hub 就绪\n");

    // 加载数据
    println!("📦 加载数据...");
    let data = load_all_data();
    println!("   ✅ 总计: {} 项\n", data.len());

    if data.is_empty() {
        println!("❌ 没有数据！");
        return;
    }

    let mut rng = rand::thread_rng();
    let mut data_shuffled: Vec<_> = data.iter().collect();
    
    let total_start = Instant::now();
    let mut cycle = 0u64;
    let mut total_signals = 0u64;
    let mut last_print = Instant::now();
    let snapshot_interval = 1000; // 每处理 1000 个信号记录一次快照

    println!("═══════════════════════════════════════════════════════════════");
    println!("  🏃 开始训练 - Fate 自主判断收敛");
    println!("═══════════════════════════════════════════════════════════════\n");

    while running.load(Ordering::SeqCst) {
        cycle += 1;
        data_shuffled.shuffle(&mut rng);
        
        println!("┌─────────────────────────────────────────────────────────────┐");
        println!("│  📖 Cycle {} 开始                                           │", cycle);
        println!("└─────────────────────────────────────────────────────────────┘");

        let cycle_start = Instant::now();
        
        for (idx, item) in data_shuffled.iter().enumerate() {
            if !running.load(Ordering::Relaxed) {
                break;
            }

            let (request_type, intensity) = match item {
                DataItem::Text { content } => (
                    RequestType::Reasoning,
                    (content.len() as f64 / 500.0).min(1.0).max(0.1),
                ),
                DataItem::Vocab { word: _ } => (
                    RequestType::Learning,
                    0.3,
                ),
                DataItem::Image { features } => (
                    RequestType::DataProcessing,
                    features.get(0).copied().unwrap_or(0.5).min(1.0).max(0.1),
                ),
                DataItem::Audio { features } => (
                    RequestType::Adaptation,
                    features.get(0).copied().unwrap_or(0.5).min(1.0).max(0.1),
                ),
            };

            let request = HubRequest {
                request_id: format!("c{}_i{}", cycle, idx),
                request_type,
                intensity,
                data: None,
                priority: Priority::Normal,
            };

            hub.process(request);
            total_signals += 1;

            // 定期记录快照
            if total_signals % snapshot_interval as u64 == 0 {
                hub.record_fate_snapshot();
            }

            // 定期打印进度
            if last_print.elapsed() >= Duration::from_secs(2) {
                let prefs = hub.get_fate_preferences();
                let elapsed = total_start.elapsed();
                let rate = total_signals as f64 / elapsed.as_secs_f64();
                let progress = (idx as f64 / data_shuffled.len() as f64) * 100.0;
                
                let pattern = hub.get_fate_convergence_pattern().unwrap_or("探索中");
                
                print!("\r  Cycle {} | {:.1}% | {}/s | BH:{:.1}% MB:{:.1}% MN:{:.1}% BF:{:.1}% | 模式: {}     ",
                    cycle,
                    progress,
                    rate as u64,
                    prefs.affinities.get(0).unwrap_or(&0.0) * 100.0,
                    prefs.affinities.get(1).unwrap_or(&0.0) * 100.0,
                    prefs.affinities.get(2).unwrap_or(&0.0) * 100.0,
                    prefs.affinities.get(3).unwrap_or(&0.0) * 100.0,
                    pattern,
                );
                std::io::Write::flush(&mut std::io::stdout()).ok();
                last_print = Instant::now();
            }

            // 检查 Fate 是否自己判断收敛
            if total_signals % snapshot_interval as u64 == 0 && hub.has_fate_converged() {
                println!("\n\n🎉 Fate 判断: 收敛 (边际效应递减)");
                running.store(false, Ordering::SeqCst);
                break;
            }

            thread::sleep(Duration::from_micros(10));
        }

        let cycle_elapsed = cycle_start.elapsed();
        println!("\n   ✅ Cycle {} 完成 | 用时: {}m {}s",
            cycle,
            cycle_elapsed.as_secs() / 60,
            cycle_elapsed.as_secs() % 60
        );

        if !running.load(Ordering::Relaxed) {
            break;
        }
    }

    // 最终报告
    let total_elapsed = total_start.elapsed();
    let final_prefs = hub.get_fate_preferences();
    let final_pattern = hub.get_fate_convergence_pattern().unwrap_or("未知");

    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║                    🏁 训练结束                                  ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!("\n⏱️  总用时: {}m {}s", total_elapsed.as_secs() / 60, total_elapsed.as_secs() % 60);
    println!("📊 总信号: {}", total_signals);
    println!("🔄 完成周期: {}", cycle);
    println!("📈 最终模式: {}", final_pattern);
    println!("\n🎯 最终 Fate 偏好:");
    println!("   BlackHole:  {:.4}%", final_prefs.affinities.get(0).unwrap_or(&0.0) * 100.0);
    println!("   MeshBrain:  {:.4}%", final_prefs.affinities.get(1).unwrap_or(&0.0) * 100.0);
    println!("   MultiNova:  {:.4}%", final_prefs.affinities.get(2).unwrap_or(&0.0) * 100.0);
    println!("   BaseForce:  {:.4}%", final_prefs.affinities.get(3).unwrap_or(&0.0) * 100.0);
    println!();
}

fn load_all_data() -> Vec<DataItem> {
    let mut items = Vec::new();
    
    // 加载旧语料
    let corpus_root = Path::new("/run/media/jouly/My Book/valhalla_corpus");
    if corpus_root.exists() {
        // 词汇
        if let Ok(content) = fs::read_to_string(corpus_root.join("vocabulary/chinese_words.txt")) {
            for line in content.lines().take(10000) {
                if !line.trim().is_empty() {
                    items.push(DataItem::Vocab { word: line.trim().to_string() });
                }
            }
        }
        
        // 句子
        if let Ok(content) = fs::read_to_string(corpus_root.join("sentences/chinese_sentences.txt")) {
            for line in content.lines().take(5000) {
                if !line.trim().is_empty() {
                    items.push(DataItem::Text { content: line.trim().to_string() });
                }
            }
        }
        
        // 对话
        for file in ["dialogues/chinese_dialogues.txt", "dialogues/english_dialogues.txt"] {
            if let Ok(content) = fs::read_to_string(corpus_root.join(file)) {
                for line in content.lines().take(2000) {
                    if !line.trim().is_empty() {
                        items.push(DataItem::Text { content: line.trim().to_string() });
                    }
                }
            }
        }
    }
    
    // 加载新数据
    let data_root = Path::new("/run/media/jouly/My Book/valhalla_training_data");
    if data_root.exists() {
        // 图片 (使用文件名作为特征)
        let images_dir = data_root.join("images/train2017");
        if images_dir.exists() {
            if let Ok(entries) = fs::read_dir(&images_dir) {
                for (i, entry) in entries.take(5000).enumerate() {
                    if entry.is_ok() {
                        let hash = i as f64 / 5000.0;
                        items.push(DataItem::Image { 
                            features: vec![hash, (hash * 2.0) % 1.0, (hash * 3.0) % 1.0, (hash * 5.0) % 1.0]
                        });
                    }
                }
            }
        }
        
        // 音频 (使用文件名作为特征)  
        let audio_dirs = ["audio/LibriSpeech/train-clean-100", "audio/LibriSpeech/train-clean-360"];
        for audio_dir in &audio_dirs {
            let dir = data_root.join(audio_dir);
            if dir.exists() {
                visit_dir_audio(&dir, &mut items, 2000);
            }
        }
        
        // Wiki 文本
        let wiki_dir = data_root.join("text/wiki_zh");
        if wiki_dir.exists() {
            if let Ok(entries) = fs::read_dir(&wiki_dir) {
                for entry in entries.take(100) {
                    if let Ok(entry) = entry {
                        if let Ok(content) = fs::read_to_string(entry.path()) {
                            for line in content.lines().take(100) {
                                if line.len() > 20 {
                                    items.push(DataItem::Text { content: line.to_string() });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    println!("   词汇: {}", items.iter().filter(|i| matches!(i, DataItem::Vocab { .. })).count());
    println!("   文本: {}", items.iter().filter(|i| matches!(i, DataItem::Text { .. })).count());
    println!("   图片: {}", items.iter().filter(|i| matches!(i, DataItem::Image { .. })).count());
    println!("   音频: {}", items.iter().filter(|i| matches!(i, DataItem::Audio { .. })).count());
    
    items
}

fn visit_dir_audio(dir: &Path, items: &mut Vec<DataItem>, limit: usize) {
    let count = items.iter().filter(|i| matches!(i, DataItem::Audio { .. })).count();
    if count >= limit {
        return;
    }
    
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    visit_dir_audio(&path, items, limit);
                } else if path.extension().map(|e| e == "flac").unwrap_or(false) {
                    let hash = items.len() as f64 / 10000.0;
                    items.push(DataItem::Audio {
                        features: vec![hash, (hash * 2.0) % 1.0, (hash * 3.0) % 1.0, (hash * 7.0) % 1.0]
                    });
                    
                    if items.iter().filter(|i| matches!(i, DataItem::Audio { .. })).count() >= limit {
                        return;
                    }
                }
            }
        }
    }
}
