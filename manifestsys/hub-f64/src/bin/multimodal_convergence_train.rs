//! 🚀 Valhalla f64 多模态收敛训练
//!
//! 使用已下载的训练数据:
//! - COCO 2017 图片 (117GB, 118k+ 图片)
//! - LibriSpeech 音频 (30GB, 29k+ 音频)
//! - Wikipedia 中文 (3.1GB)
//!
//! 70% 系统资源 | 自动收敛停机

use hub_f64::{
    ValhallaHub, HubConfig, HubRequest, RequestType, Priority,
};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use std::thread;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use rand::seq::SliceRandom;
use rand::Rng;
use bzip2::read::BzDecoder;

/// 训练配置
struct TrainingConfig {
    cpu_percent: u8,
    data_root: PathBuf,
    convergence_threshold: f64,
    convergence_stable_count: u32,
    checkpoint_interval: u64,
    max_iterations: u64,  // 0 = 无限
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            cpu_percent: 70,
            data_root: PathBuf::from("/run/media/jouly/My Book/valhalla_training_data"),
            convergence_threshold: 0.001,  // 偏好变化小于此值视为稳定
            convergence_stable_count: 20,   // 连续稳定次数达到此值则收敛
            checkpoint_interval: 10000,
            max_iterations: 0,  // 无限运行直到收敛
        }
    }
}

/// 多模态数据项
#[derive(Clone)]
enum DataItem {
    Image { path: PathBuf, features: Vec<f64> },
    Audio { path: PathBuf, features: Vec<f64> },
    Text { content: String, source: String },
}

/// 从COCO图片提取特征（简化版 - 基于文件名和大小）
fn extract_image_features(path: &Path) -> Vec<f64> {
    let mut features = vec![0.0; 64];
    
    if let Ok(metadata) = fs::metadata(path) {
        let size = metadata.len() as f64;
        // 基于文件大小生成特征
        features[0] = (size / 1_000_000.0).min(10.0) / 10.0;  // 归一化大小
        
        // 从文件名提取ID作为伪随机种子
        if let Some(name) = path.file_stem() {
            let name_str = name.to_string_lossy();
            let hash: u64 = name_str.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
            
            for i in 1..64 {
                features[i] = ((hash.wrapping_mul(i as u64 + 1) % 1000) as f64) / 1000.0;
            }
        }
    }
    
    features
}

/// 从LibriSpeech音频提取特征（简化版）
fn extract_audio_features(path: &Path) -> Vec<f64> {
    let mut features = vec![0.0; 64];
    
    if let Ok(metadata) = fs::metadata(path) {
        let size = metadata.len() as f64;
        features[0] = (size / 500_000.0).min(10.0) / 10.0;
        
        // 从路径提取说话人ID等信息
        let path_str = path.to_string_lossy();
        let hash: u64 = path_str.bytes().fold(0u64, |acc, b| acc.wrapping_mul(37).wrapping_add(b as u64));
        
        for i in 1..64 {
            features[i] = ((hash.wrapping_mul(i as u64 + 7) % 1000) as f64) / 1000.0;
        }
    }
    
    features
}

/// 加载数据集
fn load_datasets(config: &TrainingConfig) -> Vec<DataItem> {
    let mut items = Vec::new();
    let root = &config.data_root;
    
    println!("\n📦 加载多模态数据集...");
    
    // === 1. COCO 2017 图片 ===
    let images_dir = root.join("images/train2017");
    if images_dir.exists() {
        print!("  🖼️  加载 COCO 2017 图片...");
        std::io::Write::flush(&mut std::io::stdout()).ok();
        
        let mut image_count = 0;
        if let Ok(entries) = fs::read_dir(&images_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "jpg" || e == "jpeg" || e == "png") {
                    let features = extract_image_features(&path);
                    items.push(DataItem::Image { path, features });
                    image_count += 1;
                    
                    // 限制加载数量以节省内存（特征已提取）
                    if image_count >= 50000 {
                        break;
                    }
                }
            }
        }
        println!(" {} 张", image_count);
    }
    
    // === 2. LibriSpeech 音频 ===
    let audio_dir = root.join("audio/LibriSpeech");
    if audio_dir.exists() {
        print!("  🔊 加载 LibriSpeech 音频...");
        std::io::Write::flush(&mut std::io::stdout()).ok();
        
        let mut audio_count = 0;
        fn scan_audio_dir(dir: &Path, items: &mut Vec<DataItem>, count: &mut usize, limit: usize) {
            if *count >= limit { return; }
            
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    if *count >= limit { break; }
                    
                    let path = entry.path();
                    if path.is_dir() {
                        scan_audio_dir(&path, items, count, limit);
                    } else if path.extension().map_or(false, |e| e == "flac" || e == "wav") {
                        let features = extract_audio_features(&path);
                        items.push(DataItem::Audio { path, features });
                        *count += 1;
                    }
                }
            }
        }
        scan_audio_dir(&audio_dir, &mut items, &mut audio_count, 20000);
        println!(" {} 个", audio_count);
    }
    
    // === 3. Wikipedia 中文文本 ===
    let wiki_file = root.join("text/zhwiki-latest.xml.bz2");
    if wiki_file.exists() {
        print!("  📚 加载 Wikipedia 中文...");
        std::io::Write::flush(&mut std::io::stdout()).ok();
        
        let mut text_count = 0;
        if let Ok(file) = File::open(&wiki_file) {
            let decoder = BzDecoder::new(file);
            let reader = BufReader::new(decoder);
            
            let mut current_text = String::new();
            let mut in_text = false;
            
            for line in reader.lines().take(500000).flatten() {
                if line.contains("<text") {
                    in_text = true;
                    current_text.clear();
                } else if line.contains("</text>") {
                    in_text = false;
                    // 提取有意义的段落
                    for para in current_text.split("\n\n") {
                        let para = para.trim();
                        if para.len() > 50 && para.len() < 2000 && !para.starts_with("{{") && !para.starts_with("[[") {
                            items.push(DataItem::Text {
                                content: para.to_string(),
                                source: "wikipedia_zh".to_string(),
                            });
                            text_count += 1;
                            
                            if text_count >= 100000 {
                                break;
                            }
                        }
                    }
                } else if in_text {
                    current_text.push_str(&line);
                    current_text.push('\n');
                }
                
                if text_count >= 100000 {
                    break;
                }
            }
        }
        println!(" {} 段落", text_count);
    }
    
    println!("\n  ✅ 总计: {} 项多模态数据\n", items.len());
    
    items
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║     🚀 Valhalla f64 多模态收敛训练                             ║");
    println!("║     70% 资源 | 自动收敛停机 | COCO + LibriSpeech + Wikipedia   ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    
    let config = TrainingConfig::default();
    let running = Arc::new(AtomicBool::new(true));
    let iteration_counter = Arc::new(AtomicU64::new(0));
    
    // Ctrl+C 处理
    let running_clone = running.clone();
    ctrlc::set_handler(move || {
        println!("\n⚠️  收到停止信号，正在优雅退出...");
        running_clone.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");
    
    // 系统资源配置
    let num_cpus = num_cpus::get();
    let target_threads = (num_cpus as f64 * config.cpu_percent as f64 / 100.0) as usize;
    
    println!("📊 系统配置:");
    println!("   CPU 核心: {}", num_cpus);
    println!("   使用线程: {} ({}%)", target_threads, config.cpu_percent);
    println!("   收敛阈值: {}", config.convergence_threshold);
    println!("   稳定计数: {}", config.convergence_stable_count);
    
    // 配置 rayon 线程池
    rayon::ThreadPoolBuilder::new()
        .num_threads(target_threads)
        .build_global()
        .unwrap();
    
    // 初始化 Hub
    print!("\n🔧 初始化 Valhalla Hub (f64)...");
    let hub_config = HubConfig {
        resource_percent: config.cpu_percent,
        batch_size: 512,
        ..Default::default()
    };
    let mut hub = ValhallaHub::with_config(hub_config);
    println!(" ✅");
    
    // 加载数据
    let mut data = load_datasets(&config);
    
    if data.is_empty() {
        println!("❌ 没有找到训练数据！");
        println!("   请确认数据目录: {:?}", config.data_root);
        return;
    }
    
    // 打乱数据
    println!("🔀 打乱数据顺序...");
    let mut rng = rand::thread_rng();
    data.shuffle(&mut rng);
    
    // 训练状态
    let mut prev_preferences: Option<Vec<f64>> = None;
    let mut stable_count = 0u32;
    let mut total_processed = 0u64;
    
    let total_start = Instant::now();
    let mut last_report = Instant::now();
    let data_len = data.len();
    
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  🏃 开始训练 - 持续运行直到收敛");
    println!("═══════════════════════════════════════════════════════════════\n");
    
    let mut epoch = 0u64;
    
    // 主训练循环 - 无限运行直到收敛
    while running.load(Ordering::SeqCst) {
        epoch += 1;
        let epoch_start = Instant::now();
        
        println!("┌─────────────────────────────────────────────────────────────┐");
        println!("│  📖 Epoch {} 开始                                           │", epoch);
        println!("└─────────────────────────────────────────────────────────────┘");
        
        // 每个 epoch 重新打乱
        data.shuffle(&mut rng);
        
        let mut epoch_processed = 0usize;
        let mut image_count = 0usize;
        let mut audio_count = 0usize;
        let mut text_count = 0usize;
        
        for item in data.iter() {
            if !running.load(Ordering::Relaxed) {
                break;
            }
            
            let (request_type, intensity, data_json) = match item {
                DataItem::Image { path, features } => {
                    image_count += 1;
                    let intensity = features[0].max(0.1);
                    (
                        RequestType::DataProcessing,
                        intensity,
                        Some(serde_json::json!({
                            "type": "image",
                            "features": &features[..8],  // 只发送部分特征
                        }))
                    )
                },
                DataItem::Audio { path, features } => {
                    audio_count += 1;
                    let intensity = features[0].max(0.1);
                    (
                        RequestType::Learning,
                        intensity,
                        Some(serde_json::json!({
                            "type": "audio",
                            "features": &features[..8],
                        }))
                    )
                },
                DataItem::Text { content, source } => {
                    text_count += 1;
                    let intensity = (content.len() as f64 / 500.0).min(1.0).max(0.1);
                    let preview: String = content.chars().take(100).collect();
                    (
                        RequestType::Reasoning,
                        intensity,
                        Some(serde_json::json!({
                            "type": "text",
                            "preview": preview,
                            "source": source,
                        }))
                    )
                },
            };
            
            let request = HubRequest {
                request_id: format!("e{}_i{}", epoch, epoch_processed),
                request_type,
                intensity,
                data: data_json,
                priority: Priority::Normal,
            };
            
            hub.process(request);
            epoch_processed += 1;
            total_processed += 1;
            
            // 定期报告
            if last_report.elapsed() >= Duration::from_secs(10) {
                let prefs = hub.get_fate_preferences();
                let elapsed = total_start.elapsed();
                let rate = total_processed as f64 / elapsed.as_secs_f64();
                
                println!("\n📈 进度报告 @ epoch {} ({:.1}%)", 
                    epoch, (epoch_processed as f64 / data_len as f64) * 100.0);
                println!("   总处理: {} | 速率: {:.0}/s | 时间: {}m{}s",
                    total_processed, rate, elapsed.as_secs() / 60, elapsed.as_secs() % 60);
                println!("   🖼️ 图片: {} | 🔊 音频: {} | 📚 文本: {}",
                    image_count, audio_count, text_count);
                println!("\n   🎯 Fate 偏好:");
                println!("      BlackHole:  {:.4}%", prefs.affinities.get(0).unwrap_or(&0.0) * 100.0);
                println!("      MeshBrain:  {:.4}%", prefs.affinities.get(1).unwrap_or(&0.0) * 100.0);
                println!("      MultiNova:  {:.4}%", prefs.affinities.get(2).unwrap_or(&0.0) * 100.0);
                println!("      BaseForce:  {:.4}%", prefs.affinities.get(3).unwrap_or(&0.0) * 100.0);
                
                // 检查收敛
                if let Some(ref prev) = prev_preferences {
                    let diff: f64 = prefs.affinities.iter()
                        .zip(prev.iter())
                        .map(|(a, b)| (a - b).abs())
                        .sum();
                    
                    println!("   📐 偏好漂移: {:.6}", diff);
                    
                    if diff < config.convergence_threshold {
                        stable_count += 1;
                        println!("   ✅ 稳定计数: {}/{}", stable_count, config.convergence_stable_count);
                        
                        if stable_count >= config.convergence_stable_count {
                            println!("\n🎉 检测到收敛! 自动停机...");
                            running.store(false, Ordering::SeqCst);
                            break;
                        }
                    } else {
                        stable_count = 0;
                    }
                }
                prev_preferences = Some(prefs.affinities.clone());
                
                last_report = Instant::now();
            }
            
            // CPU 限制
            thread::sleep(Duration::from_micros(20));
        }
        
        // Epoch 结束报告
        let epoch_elapsed = epoch_start.elapsed();
        println!("\n╔════════════════════════════════════════════════════════════════╗");
        println!("║  ✅ Epoch {} 完成                                              ║", epoch);
        println!("║  ⏱️  用时: {}m {}s | 处理: {} 项                               ║",
            epoch_elapsed.as_secs() / 60, epoch_elapsed.as_secs() % 60, epoch_processed);
        println!("╚════════════════════════════════════════════════════════════════╝\n");
        
        // 检查最大迭代
        if config.max_iterations > 0 && total_processed >= config.max_iterations {
            println!("📊 达到最大迭代次数: {}", config.max_iterations);
            break;
        }
    }
    
    // 最终报告
    let total_elapsed = total_start.elapsed();
    let final_prefs = hub.get_fate_preferences();
    let stats = hub.get_statistics();
    
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║                    🏁 训练完成报告                              ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();
    println!("⏱️  总用时: {}h {}m {}s", 
        total_elapsed.as_secs() / 3600,
        (total_elapsed.as_secs() % 3600) / 60,
        total_elapsed.as_secs() % 60);
    println!("📊 总处理: {} 项", total_processed);
    println!("🔄 完成 Epoch: {}", epoch);
    println!("📈 平均速率: {:.0} 项/秒", total_processed as f64 / total_elapsed.as_secs_f64());
    println!();
    println!("🎯 最终 Fate 偏好:");
    println!("   BlackHole:  {:.6}%", final_prefs.affinities.get(0).unwrap_or(&0.0) * 100.0);
    println!("   MeshBrain:  {:.6}%", final_prefs.affinities.get(1).unwrap_or(&0.0) * 100.0);
    println!("   MultiNova:  {:.6}%", final_prefs.affinities.get(2).unwrap_or(&0.0) * 100.0);
    println!("   BaseForce:  {:.6}%", final_prefs.affinities.get(3).unwrap_or(&0.0) * 100.0);
    println!();
    
    if stable_count >= config.convergence_stable_count {
        println!("✅ 状态: 收敛完成");
    } else {
        println!("⚠️ 状态: 手动停止 (稳定计数: {})", stable_count);
    }
    
    println!();
    println!("📬 Hub 统计:");
    println!("   总请求: {}", stats.total_requests);
    println!("   成功路由: {}", stats.successful_routes);
    println!("   平均响应: {:.4}ms", stats.average_response_time_ms);
}
