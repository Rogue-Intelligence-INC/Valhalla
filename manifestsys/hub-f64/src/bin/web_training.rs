//! 🌐 Valhalla Web 可视化训练监控
//!
//! 提供实时网页界面:
//! - 数据聚合可视化
//! - 训练周期进度
//! - Fate 系统偏好图表
//! - WebSocket 实时推送

use hub_f64::{ValhallaHub, HubConfig, HubRequest, RequestType, Priority};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use std::thread;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use rand::{seq::SliceRandom, Rng};
use bzip2::read::BzDecoder;
use serde::{Serialize, Deserialize};
use axum::{
    Router,
    routing::get,
    response::{Html, IntoResponse},
    extract::{State, ws::{WebSocket, WebSocketUpgrade, Message}},
    http::StatusCode,
};
use tower_http::cors::CorsLayer;
use futures::{SinkExt, StreamExt};

// ═══════════════════════════════════════════════════════════════════
//  数据结构
// ═══════════════════════════════════════════════════════════════════

#[derive(Clone)]
struct AppState {
    training_state: Arc<Mutex<TrainingStateShared>>,
    running: Arc<AtomicBool>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct TrainingStateShared {
    epoch: u64,
    total_processed: u64,
    epoch_progress: f64,
    rate: f64,
    elapsed_secs: u64,
    stable_count: u32,
    drift: f64,
    fate_preferences: Vec<f64>,
    data_stats: DataStatsJson,
    loaded_stats: DataStatsJson,
    last_save_time: String,
    is_running: bool,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct DataStatsJson {
    vocab: usize,
    sentence: usize,
    dialogue: usize,
    novel: usize,
    image: usize,
    audio: usize,
    wiki: usize,
}

#[derive(Clone)]
enum DataItem {
    Vocabulary { word: String },
    Sentence { text: String },
    Dialogue { text: String },
    Novel { text: String },
    Image { features: Vec<f64> },
    Audio { features: Vec<f64> },
    WikiText { text: String },
}

struct TrainingConfig {
    cpu_percent: u8,
    old_corpus_root: PathBuf,
    new_data_root: PathBuf,
    checkpoint_path: PathBuf,
    convergence_threshold: f64,
    convergence_stable_count: u32,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            cpu_percent: 70,
            old_corpus_root: PathBuf::from("/run/media/jouly/My Book/valhalla_corpus"),
            new_data_root: PathBuf::from("/run/media/jouly/My Book/valhalla_training_data"),
            checkpoint_path: PathBuf::from("/run/media/jouly/My Book/valhalla_training.valhalla"),
            convergence_threshold: 0.0001,
            convergence_stable_count: 100,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct CheckpointState {
    epoch: u64,
    total_processed: u64,
    stable_count: u32,
    fate_preferences: Vec<f64>,
    last_save_time: String,
}

// ═══════════════════════════════════════════════════════════════════
//  数据加载
// ═══════════════════════════════════════════════════════════════════

fn load_old_corpus(root: &Path) -> (Vec<DataItem>, DataStatsJson) {
    let mut items = Vec::new();
    let mut stats = DataStatsJson::default();
    
    // 语言语料
    let lang_dir = root.join("languages");
    if lang_dir.exists() {
        let languages = ["chinese", "english", "japanese", "korean", "french", 
                         "german", "spanish", "russian", "arabic", "portuguese"];
        
        for lang in languages {
            let lang_path = lang_dir.join(lang);
            
            let vocab_dir = lang_path.join("vocabulary");
            if vocab_dir.exists() {
                for entry in fs::read_dir(&vocab_dir).into_iter().flatten().flatten() {
                    if entry.path().extension().map_or(false, |e| e == "txt") {
                        if let Ok(file) = File::open(entry.path()) {
                            for line in BufReader::new(file).lines().take(5000).flatten() {
                                let word = line.split_whitespace().next().unwrap_or("").trim();
                                if word.len() > 1 && !word.starts_with('#') {
                                    items.push(DataItem::Vocabulary { word: word.to_string() });
                                    stats.vocab += 1;
                                }
                            }
                        }
                    }
                }
            }
            
            let sent_dir = lang_path.join("sentences");
            if sent_dir.exists() {
                for entry in fs::read_dir(&sent_dir).into_iter().flatten().flatten() {
                    if entry.path().extension().map_or(false, |e| e == "txt" || e == "tsv") {
                        if let Ok(file) = File::open(entry.path()) {
                            for line in BufReader::new(file).lines().take(10000).flatten() {
                                let parts: Vec<&str> = line.split('\t').collect();
                                let sentence = if parts.len() >= 3 {
                                    parts[2].trim().to_string()
                                } else {
                                    line.trim().to_string()
                                };
                                if sentence.len() > 5 && sentence.len() < 500 {
                                    items.push(DataItem::Sentence { text: sentence });
                                    stats.sentence += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // 对话
    let dialogue_dir = root.join("dialogues");
    if dialogue_dir.exists() {
        let daily = dialogue_dir.join("daily_dialog_train.json");
        if daily.exists() {
            if let Ok(content) = fs::read_to_string(&daily) {
                for line in content.lines().take(30000) {
                    if let Some(start) = line.find('"') {
                        if let Some(end) = line[start+1..].find('"') {
                            let text = &line[start+1..start+1+end];
                            if text.len() > 5 && text.len() < 500 {
                                items.push(DataItem::Dialogue { text: text.to_string() });
                                stats.dialogue += 1;
                            }
                        }
                    }
                }
            }
        }
        
        let cornell = dialogue_dir.join("cornell movie-dialogs corpus/movie_lines.txt");
        if cornell.exists() {
            if let Ok(file) = File::open(&cornell) {
                for line in BufReader::new(file).lines().take(30000).flatten() {
                    let parts: Vec<&str> = line.split(" +++$+++ ").collect();
                    if parts.len() >= 5 {
                        let text = parts[4].trim();
                        if text.len() > 5 && text.len() < 500 {
                            items.push(DataItem::Dialogue { text: text.to_string() });
                            stats.dialogue += 1;
                        }
                    }
                }
            }
        }
    }
    
    // 小说
    let novel_dir = root.join("novels");
    if novel_dir.exists() {
        for entry in fs::read_dir(&novel_dir).into_iter().flatten().flatten() {
            if entry.path().is_dir() {
                for sub in fs::read_dir(entry.path()).into_iter().flatten().flatten() {
                    if sub.path().extension().map_or(false, |e| e == "txt") {
                        if let Ok(file) = File::open(sub.path()) {
                            let mut para = String::new();
                            for line in BufReader::new(file).lines().take(10000).flatten() {
                                let line = line.trim();
                                if line.is_empty() {
                                    if para.len() > 50 && para.len() < 2000 {
                                        items.push(DataItem::Novel { text: para.clone() });
                                        stats.novel += 1;
                                    }
                                    para.clear();
                                } else {
                                    if !para.is_empty() { para.push(' '); }
                                    para.push_str(line);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    (items, stats)
}

fn load_new_data(root: &Path) -> (Vec<DataItem>, DataStatsJson) {
    let mut items = Vec::new();
    let mut stats = DataStatsJson::default();
    
    // COCO 图片
    let images_dir = root.join("images/train2017");
    if images_dir.exists() {
        if let Ok(entries) = fs::read_dir(&images_dir) {
            for entry in entries.flatten().take(30000) {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "jpg" || e == "jpeg") {
                    if let Ok(meta) = fs::metadata(&path) {
                        let size = meta.len() as f64;
                        let name = path.file_stem().unwrap_or_default().to_string_lossy();
                        let hash: u64 = name.bytes().fold(0u64, |a, b| a.wrapping_mul(31).wrapping_add(b as u64));
                        
                        let mut features = vec![0.0; 32];
                        features[0] = (size / 1_000_000.0).min(10.0) / 10.0;
                        for i in 1..32 {
                            features[i] = ((hash.wrapping_mul(i as u64 + 1) % 1000) as f64) / 1000.0;
                        }
                        
                        items.push(DataItem::Image { features });
                        stats.image += 1;
                    }
                }
            }
        }
    }
    
    // LibriSpeech 音频
    let audio_dir = root.join("audio/LibriSpeech");
    if audio_dir.exists() {
        fn scan_flac(dir: &Path, items: &mut Vec<DataItem>, stats: &mut DataStatsJson, limit: usize) {
            if stats.audio >= limit { return; }
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    if stats.audio >= limit { break; }
                    let path = entry.path();
                    if path.is_dir() {
                        scan_flac(&path, items, stats, limit);
                    } else if path.extension().map_or(false, |e| e == "flac") {
                        if let Ok(meta) = fs::metadata(&path) {
                            let size = meta.len() as f64;
                            let name = path.to_string_lossy();
                            let hash: u64 = name.bytes().fold(0u64, |a, b| a.wrapping_mul(37).wrapping_add(b as u64));
                            
                            let mut features = vec![0.0; 32];
                            features[0] = (size / 500_000.0).min(10.0) / 10.0;
                            for i in 1..32 {
                                features[i] = ((hash.wrapping_mul(i as u64 + 7) % 1000) as f64) / 1000.0;
                            }
                            
                            items.push(DataItem::Audio { features });
                            stats.audio += 1;
                        }
                    }
                }
            }
        }
        scan_flac(&audio_dir, &mut items, &mut stats, 15000);
    }
    
    // Wikipedia
    let wiki_file = root.join("text/zhwiki-latest.xml.bz2");
    if wiki_file.exists() {
        if let Ok(file) = File::open(&wiki_file) {
            let decoder = BzDecoder::new(file);
            let reader = BufReader::new(decoder);
            
            let mut current_text = String::new();
            let mut in_text = false;
            
            for line in reader.lines().take(300000).flatten() {
                if line.contains("<text") {
                    in_text = true;
                    current_text.clear();
                } else if line.contains("</text>") {
                    in_text = false;
                    for para in current_text.split("\n\n") {
                        let para = para.trim();
                        if para.len() > 50 && para.len() < 2000 
                           && !para.starts_with("{{") && !para.starts_with("[[") 
                           && !para.contains("[[Category:") {
                            items.push(DataItem::WikiText { text: para.to_string() });
                            stats.wiki += 1;
                            if stats.wiki >= 50000 { break; }
                        }
                    }
                } else if in_text {
                    current_text.push_str(&line);
                    current_text.push('\n');
                }
                if stats.wiki >= 50000 { break; }
            }
        }
    }
    
    (items, stats)
}

fn save_checkpoint(path: &Path, state: &CheckpointState) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(state)?;
    let mut file = File::create(path)?;
    file.write_all(b"VALHALLA_CHECKPOINT_V1\n")?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

fn load_checkpoint(path: &Path) -> Option<CheckpointState> {
    if !path.exists() { return None; }
    let content = fs::read_to_string(path).ok()?;
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() || !lines[0].starts_with("VALHALLA_CHECKPOINT") { return None; }
    let json = lines[1..].join("\n");
    serde_json::from_str(&json).ok()
}

// ═══════════════════════════════════════════════════════════════════
//  Web 界面 HTML
// ═══════════════════════════════════════════════════════════════════

const HTML_PAGE: &str = r##"<!DOCTYPE html>
<html lang="zh">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>🚀 Valhalla 训练监控</title>
    <link href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;600;700&family=Noto+Sans+SC:wght@400;500;700&display=swap" rel="stylesheet">
    <style>
        :root {
            --bg-primary: #0a0e14;
            --bg-secondary: #141a23;
            --bg-tertiary: #1a2332;
            --accent-cyan: #00d4ff;
            --accent-magenta: #ff00aa;
            --accent-gold: #ffd700;
            --accent-green: #00ff88;
            --accent-red: #ff4444;
            --accent-purple: #aa66ff;
            --accent-orange: #ff8844;
            --text-primary: #e8eaed;
            --text-secondary: #9aa0a6;
            --border-color: #2a3444;
            --glow-cyan: 0 0 20px rgba(0, 212, 255, 0.3);
            --glow-magenta: 0 0 20px rgba(255, 0, 170, 0.3);
        }
        
        * { margin: 0; padding: 0; box-sizing: border-box; }
        
        body {
            font-family: 'Noto Sans SC', 'JetBrains Mono', monospace;
            background: var(--bg-primary);
            color: var(--text-primary);
            min-height: 100vh;
            overflow-x: hidden;
        }
        
        /* 动态背景 */
        .bg-grid {
            position: fixed;
            inset: 0;
            background: 
                linear-gradient(90deg, rgba(0,212,255,0.03) 1px, transparent 1px),
                linear-gradient(rgba(0,212,255,0.03) 1px, transparent 1px);
            background-size: 50px 50px;
            pointer-events: none;
            z-index: 0;
        }
        
        .bg-glow {
            position: fixed;
            width: 600px;
            height: 600px;
            border-radius: 50%;
            filter: blur(150px);
            opacity: 0.15;
            pointer-events: none;
            z-index: 0;
        }
        
        .bg-glow-1 { background: var(--accent-cyan); top: -200px; left: -200px; }
        .bg-glow-2 { background: var(--accent-magenta); bottom: -200px; right: -200px; }
        
        .container {
            position: relative;
            z-index: 1;
            max-width: 1400px;
            margin: 0 auto;
            padding: 20px;
        }
        
        /* 头部 */
        header {
            text-align: center;
            padding: 30px 0;
            border-bottom: 1px solid var(--border-color);
            margin-bottom: 30px;
        }
        
        header h1 {
            font-size: 2.5rem;
            font-weight: 700;
            background: linear-gradient(135deg, var(--accent-cyan), var(--accent-magenta));
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            margin-bottom: 10px;
            text-shadow: var(--glow-cyan);
        }
        
        header .subtitle {
            color: var(--text-secondary);
            font-size: 1rem;
        }
        
        .status-badge {
            display: inline-flex;
            align-items: center;
            gap: 8px;
            padding: 8px 16px;
            background: var(--bg-tertiary);
            border-radius: 20px;
            margin-top: 15px;
            border: 1px solid var(--border-color);
        }
        
        .status-dot {
            width: 10px;
            height: 10px;
            border-radius: 50%;
            animation: pulse 2s ease-in-out infinite;
        }
        
        .status-dot.running { background: var(--accent-green); }
        .status-dot.stopped { background: var(--accent-red); }
        
        @keyframes pulse {
            0%, 100% { opacity: 1; transform: scale(1); }
            50% { opacity: 0.5; transform: scale(1.2); }
        }
        
        /* 网格布局 */
        .dashboard {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
            gap: 20px;
        }
        
        /* 卡片 */
        .card {
            background: var(--bg-secondary);
            border: 1px solid var(--border-color);
            border-radius: 16px;
            overflow: hidden;
            transition: all 0.3s ease;
        }
        
        .card:hover {
            border-color: var(--accent-cyan);
            box-shadow: var(--glow-cyan);
        }
        
        .card-header {
            padding: 20px;
            border-bottom: 1px solid var(--border-color);
            display: flex;
            align-items: center;
            gap: 12px;
        }
        
        .card-header .icon {
            font-size: 1.5rem;
        }
        
        .card-header h2 {
            font-size: 1.1rem;
            font-weight: 600;
        }
        
        .card-body {
            padding: 20px;
        }
        
        /* 进度条 */
        .progress-bar {
            height: 24px;
            background: var(--bg-tertiary);
            border-radius: 12px;
            overflow: hidden;
            position: relative;
            margin: 10px 0;
        }
        
        .progress-fill {
            height: 100%;
            border-radius: 12px;
            transition: width 0.5s ease;
            position: relative;
            overflow: hidden;
        }
        
        .progress-fill::after {
            content: '';
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: linear-gradient(90deg, transparent, rgba(255,255,255,0.2), transparent);
            animation: shimmer 2s infinite;
        }
        
        @keyframes shimmer {
            0% { transform: translateX(-100%); }
            100% { transform: translateX(100%); }
        }
        
        .progress-fill.cyan { background: linear-gradient(90deg, var(--accent-cyan), #0088aa); }
        .progress-fill.magenta { background: linear-gradient(90deg, var(--accent-magenta), #aa0077); }
        .progress-fill.gold { background: linear-gradient(90deg, var(--accent-gold), #cc9900); }
        .progress-fill.green { background: linear-gradient(90deg, var(--accent-green), #00aa55); }
        
        .progress-label {
            display: flex;
            justify-content: space-between;
            font-size: 0.9rem;
            color: var(--text-secondary);
        }
        
        .progress-value {
            font-family: 'JetBrains Mono', monospace;
            font-weight: 600;
            color: var(--text-primary);
        }
        
        /* 数据条形图 */
        .data-bar {
            display: flex;
            align-items: center;
            margin: 12px 0;
            gap: 12px;
        }
        
        .data-bar .label {
            width: 60px;
            font-size: 0.85rem;
            color: var(--text-secondary);
        }
        
        .data-bar .bar-container {
            flex: 1;
            height: 20px;
            background: var(--bg-tertiary);
            border-radius: 10px;
            overflow: hidden;
        }
        
        .data-bar .bar-fill {
            height: 100%;
            border-radius: 10px;
            transition: width 0.5s ease;
        }
        
        .data-bar .count {
            width: 80px;
            text-align: right;
            font-family: 'JetBrains Mono', monospace;
            font-size: 0.85rem;
        }
        
        .data-bar .pct {
            width: 50px;
            text-align: right;
            font-family: 'JetBrains Mono', monospace;
            font-size: 0.85rem;
            color: var(--accent-cyan);
        }
        
        /* FATE 雷达图区域 */
        .fate-grid {
            display: grid;
            grid-template-columns: repeat(2, 1fr);
            gap: 15px;
        }
        
        .fate-item {
            background: var(--bg-tertiary);
            padding: 15px;
            border-radius: 12px;
            text-align: center;
            transition: all 0.3s ease;
        }
        
        .fate-item:hover {
            transform: translateY(-2px);
        }
        
        .fate-item .name {
            font-size: 0.9rem;
            margin-bottom: 8px;
            font-weight: 500;
        }
        
        .fate-item .value {
            font-size: 1.8rem;
            font-weight: 700;
            font-family: 'JetBrains Mono', monospace;
        }
        
        .fate-item .indicator {
            font-size: 1rem;
            margin-left: 5px;
        }
        
        .fate-item.blackhole { color: var(--accent-red); }
        .fate-item.meshbrain { color: var(--accent-cyan); }
        .fate-item.multinova { color: var(--accent-gold); }
        .fate-item.baseforce { color: var(--accent-green); }
        
        /* 统计数值 */
        .stats-row {
            display: flex;
            justify-content: space-around;
            flex-wrap: wrap;
            gap: 20px;
            margin-top: 15px;
        }
        
        .stat-item {
            text-align: center;
        }
        
        .stat-item .value {
            font-size: 1.5rem;
            font-weight: 700;
            font-family: 'JetBrains Mono', monospace;
            color: var(--accent-cyan);
        }
        
        .stat-item .label {
            font-size: 0.8rem;
            color: var(--text-secondary);
            margin-top: 5px;
        }
        
        /* 收敛进度 */
        .convergence-section {
            margin-top: 20px;
            padding-top: 20px;
            border-top: 1px solid var(--border-color);
        }
        
        .convergence-info {
            display: flex;
            justify-content: space-between;
            margin-top: 10px;
            font-size: 0.9rem;
        }
        
        .drift-value {
            color: var(--accent-gold);
        }
        
        .drift-value.low { color: var(--accent-green); }
        .drift-value.high { color: var(--accent-red); }
        
        /* 全宽卡片 */
        .card.full-width {
            grid-column: 1 / -1;
        }
        
        /* 底部状态 */
        footer {
            margin-top: 30px;
            padding: 20px;
            text-align: center;
            color: var(--text-secondary);
            font-size: 0.85rem;
            border-top: 1px solid var(--border-color);
        }
        
        footer .checkpoint {
            font-family: 'JetBrains Mono', monospace;
            color: var(--accent-cyan);
        }
        
        /* 响应式 */
        @media (max-width: 900px) {
            .dashboard { grid-template-columns: 1fr; }
            header h1 { font-size: 1.8rem; }
        }
    </style>
</head>
<body>
    <div class="bg-grid"></div>
    <div class="bg-glow bg-glow-1"></div>
    <div class="bg-glow bg-glow-2"></div>
    
    <div class="container">
        <header>
            <h1>🚀 VALHALLA f64 训练监控</h1>
            <p class="subtitle">70% CPU | .valhalla 断点续训 | 实时监控</p>
            <div class="status-badge">
                <span class="status-dot" id="statusDot"></span>
                <span id="statusText">连接中...</span>
            </div>
        </header>
        
        <div class="dashboard">
            <!-- 训练周期 -->
            <div class="card">
                <div class="card-header">
                    <span class="icon">🔄</span>
                    <h2>训练周期</h2>
                </div>
                <div class="card-body">
                    <div class="progress-label">
                        <span>Epoch <strong id="epochNum">0</strong></span>
                        <span class="progress-value" id="epochProgress">0.0%</span>
                    </div>
                    <div class="progress-bar">
                        <div class="progress-fill cyan" id="epochBar" style="width: 0%"></div>
                    </div>
                    <div class="stats-row">
                        <div class="stat-item">
                            <div class="value" id="totalProcessed">0</div>
                            <div class="label">已处理</div>
                        </div>
                        <div class="stat-item">
                            <div class="value" id="rate">0</div>
                            <div class="label">速度/秒</div>
                        </div>
                        <div class="stat-item">
                            <div class="value" id="elapsed">0:00:00</div>
                            <div class="label">运行时间</div>
                        </div>
                    </div>
                </div>
            </div>
            
            <!-- FATE 系统 -->
            <div class="card">
                <div class="card-header">
                    <span class="icon">🌌</span>
                    <h2>FATE 系统偏好</h2>
                </div>
                <div class="card-body">
                    <div class="fate-grid">
                        <div class="fate-item blackhole">
                            <div class="name">⚫ BlackHole</div>
                            <div class="value"><span id="fateBlackhole">25.00</span>%<span class="indicator" id="indBH">●</span></div>
                        </div>
                        <div class="fate-item meshbrain">
                            <div class="name">🧠 MeshBrain</div>
                            <div class="value"><span id="fateMeshbrain">25.00</span>%<span class="indicator" id="indMB">●</span></div>
                        </div>
                        <div class="fate-item multinova">
                            <div class="name">✨ MultiNova</div>
                            <div class="value"><span id="fateMultinova">25.00</span>%<span class="indicator" id="indMN">●</span></div>
                        </div>
                        <div class="fate-item baseforce">
                            <div class="name">⚡ BaseForce</div>
                            <div class="value"><span id="fateBaseforce">25.00</span>%<span class="indicator" id="indBF">●</span></div>
                        </div>
                    </div>
                    <div class="convergence-section">
                        <div class="progress-label">
                            <span>收敛进度</span>
                            <span class="progress-value"><span id="stableCount">0</span>/30</span>
                        </div>
                        <div class="progress-bar">
                            <div class="progress-fill green" id="convergenceBar" style="width: 0%"></div>
                        </div>
                        <div class="convergence-info">
                            <span>偏好漂移: <span class="drift-value" id="driftValue">0.0000%</span></span>
                        </div>
                    </div>
                </div>
            </div>
            
            <!-- 数据聚合 -->
            <div class="card full-width">
                <div class="card-header">
                    <span class="icon">📊</span>
                    <h2>数据聚合形式</h2>
                </div>
                <div class="card-body">
                    <div class="data-bar">
                        <span class="label">词汇</span>
                        <div class="bar-container"><div class="bar-fill" id="barVocab" style="width: 0%; background: linear-gradient(90deg, #ffd700, #cc9900)"></div></div>
                        <span class="count" id="countVocab">0</span>
                        <span class="pct" id="pctVocab">0%</span>
                    </div>
                    <div class="data-bar">
                        <span class="label">句子</span>
                        <div class="bar-container"><div class="bar-fill" id="barSentence" style="width: 0%; background: linear-gradient(90deg, #00ff88, #00aa55)"></div></div>
                        <span class="count" id="countSentence">0</span>
                        <span class="pct" id="pctSentence">0%</span>
                    </div>
                    <div class="data-bar">
                        <span class="label">对话</span>
                        <div class="bar-container"><div class="bar-fill" id="barDialogue" style="width: 0%; background: linear-gradient(90deg, #00d4ff, #0088aa)"></div></div>
                        <span class="count" id="countDialogue">0</span>
                        <span class="pct" id="pctDialogue">0%</span>
                    </div>
                    <div class="data-bar">
                        <span class="label">小说</span>
                        <div class="bar-container"><div class="bar-fill" id="barNovel" style="width: 0%; background: linear-gradient(90deg, #ff00aa, #aa0077)"></div></div>
                        <span class="count" id="countNovel">0</span>
                        <span class="pct" id="pctNovel">0%</span>
                    </div>
                    <div class="data-bar">
                        <span class="label">图片</span>
                        <div class="bar-container"><div class="bar-fill" id="barImage" style="width: 0%; background: linear-gradient(90deg, #ff8844, #cc5500)"></div></div>
                        <span class="count" id="countImage">0</span>
                        <span class="pct" id="pctImage">0%</span>
                    </div>
                    <div class="data-bar">
                        <span class="label">音频</span>
                        <div class="bar-container"><div class="bar-fill" id="barAudio" style="width: 0%; background: linear-gradient(90deg, #ff66cc, #cc3399)"></div></div>
                        <span class="count" id="countAudio">0</span>
                        <span class="pct" id="pctAudio">0%</span>
                    </div>
                    <div class="data-bar">
                        <span class="label">Wiki</span>
                        <div class="bar-container"><div class="bar-fill" id="barWiki" style="width: 0%; background: linear-gradient(90deg, #88ff44, #55cc00)"></div></div>
                        <span class="count" id="countWiki">0</span>
                        <span class="pct" id="pctWiki">0%</span>
                    </div>
                </div>
            </div>
        </div>
        
        <footer>
            <p>💾 Checkpoint: <span class="checkpoint" id="checkpointPath">/run/media/jouly/My Book/valhalla_training.valhalla</span></p>
            <p>⏱ 上次保存: <span id="lastSave">-</span></p>
        </footer>
    </div>
    
    <script>
        const ws = new WebSocket(`ws://${location.host}/ws`);
        
        ws.onopen = () => {
            document.getElementById('statusDot').classList.add('running');
            document.getElementById('statusDot').classList.remove('stopped');
            document.getElementById('statusText').textContent = '训练中...';
        };
        
        ws.onclose = () => {
            document.getElementById('statusDot').classList.add('stopped');
            document.getElementById('statusDot').classList.remove('running');
            document.getElementById('statusText').textContent = '已断开';
        };
        
        ws.onmessage = (event) => {
            const data = JSON.parse(event.data);
            updateUI(data);
        };
        
        function formatNumber(n) {
            if (n >= 1000000) return (n / 1000000).toFixed(2) + 'M';
            if (n >= 1000) return (n / 1000).toFixed(1) + 'K';
            return n.toString();
        }
        
        function formatTime(secs) {
            const h = Math.floor(secs / 3600);
            const m = Math.floor((secs % 3600) / 60);
            const s = secs % 60;
            return `${h}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
        }
        
        function getIndicator(val) {
            if (val > 26) return '▲';
            if (val < 24) return '▼';
            return '●';
        }
        
        function updateUI(data) {
            // 状态
            if (data.is_running) {
                document.getElementById('statusDot').classList.add('running');
                document.getElementById('statusDot').classList.remove('stopped');
                document.getElementById('statusText').textContent = '训练中...';
            } else {
                document.getElementById('statusDot').classList.add('stopped');
                document.getElementById('statusDot').classList.remove('running');
                document.getElementById('statusText').textContent = '已停止';
            }
            
            // 训练周期
            document.getElementById('epochNum').textContent = data.epoch;
            document.getElementById('epochProgress').textContent = data.epoch_progress.toFixed(1) + '%';
            document.getElementById('epochBar').style.width = data.epoch_progress + '%';
            document.getElementById('totalProcessed').textContent = formatNumber(data.total_processed);
            document.getElementById('rate').textContent = formatNumber(Math.round(data.rate));
            document.getElementById('elapsed').textContent = formatTime(data.elapsed_secs);
            
            // FATE
            const prefs = data.fate_preferences || [0.25, 0.25, 0.25, 0.25];
            const bh = prefs[0] * 100, mb = prefs[1] * 100, mn = prefs[2] * 100, bf = prefs[3] * 100;
            document.getElementById('fateBlackhole').textContent = bh.toFixed(2);
            document.getElementById('fateMeshbrain').textContent = mb.toFixed(2);
            document.getElementById('fateMultinova').textContent = mn.toFixed(2);
            document.getElementById('fateBaseforce').textContent = bf.toFixed(2);
            document.getElementById('indBH').textContent = getIndicator(bh);
            document.getElementById('indMB').textContent = getIndicator(mb);
            document.getElementById('indMN').textContent = getIndicator(mn);
            document.getElementById('indBF').textContent = getIndicator(bf);
            
            // 收敛
            document.getElementById('stableCount').textContent = data.stable_count;
            document.getElementById('convergenceBar').style.width = (data.stable_count / 30 * 100) + '%';
            const driftEl = document.getElementById('driftValue');
            driftEl.textContent = (data.drift * 100).toFixed(4) + '%';
            driftEl.className = 'drift-value' + (data.drift < 0.001 ? ' low' : data.drift > 0.005 ? ' high' : '');
            
            // 数据聚合
            const stats = data.data_stats || {};
            const loaded = data.loaded_stats || {};
            const maxLoaded = Math.max(loaded.vocab||1, loaded.sentence||1, loaded.dialogue||1, 
                                        loaded.novel||1, loaded.image||1, loaded.audio||1, loaded.wiki||1);
            
            function updateBar(type, current, total) {
                const barWidth = (total / maxLoaded) * 100;
                document.getElementById('bar' + type).style.width = barWidth + '%';
                document.getElementById('count' + type).textContent = formatNumber(total);
                const pct = total > 0 ? (current / total * 100).toFixed(1) : '0';
                document.getElementById('pct' + type).textContent = pct + '%';
            }
            
            updateBar('Vocab', stats.vocab||0, loaded.vocab||0);
            updateBar('Sentence', stats.sentence||0, loaded.sentence||0);
            updateBar('Dialogue', stats.dialogue||0, loaded.dialogue||0);
            updateBar('Novel', stats.novel||0, loaded.novel||0);
            updateBar('Image', stats.image||0, loaded.image||0);
            updateBar('Audio', stats.audio||0, loaded.audio||0);
            updateBar('Wiki', stats.wiki||0, loaded.wiki||0);
            
            // 底部
            if (data.last_save_time) {
                document.getElementById('lastSave').textContent = data.last_save_time;
            }
        }
    </script>
</body>
</html>
"##;

// ═══════════════════════════════════════════════════════════════════
//  Web 路由
// ═══════════════════════════════════════════════════════════════════

async fn index_handler() -> Html<&'static str> {
    Html(HTML_PAGE)
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    loop {
        // 发送当前状态
        let training_state = state.training_state.lock().unwrap().clone();
        let json = serde_json::to_string(&training_state).unwrap_or_default();
        
        if socket.send(Message::Text(json)).await.is_err() {
            break;
        }
        
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        if !state.running.load(Ordering::Relaxed) {
            break;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
//  训练线程
// ═══════════════════════════════════════════════════════════════════

fn training_thread(
    state: Arc<Mutex<TrainingStateShared>>,
    running: Arc<AtomicBool>,
) {
    let config = TrainingConfig::default();
    
    // 加载checkpoint
    let mut checkpoint = load_checkpoint(&config.checkpoint_path).unwrap_or_default();
    
    // 重置stable_count以继续训练
    checkpoint.stable_count = 0;
    
    // 系统配置
    let num_cpus = num_cpus::get();
    let target_threads = (num_cpus as f64 * config.cpu_percent as f64 / 100.0) as usize;
    rayon::ThreadPoolBuilder::new()
        .num_threads(target_threads)
        .build_global()
        .ok();
    
    // 初始化 Hub
    let hub_config = HubConfig {
        resource_percent: config.cpu_percent,
        batch_size: 512,
        ..Default::default()
    };
    let mut hub = ValhallaHub::with_config(hub_config);
    
    // 加载数据
    println!("📦 加载数据...");
    let (mut old_data, old_stats) = load_old_corpus(&config.old_corpus_root);
    let (mut new_data, new_stats) = load_new_data(&config.new_data_root);
    
    let loaded_stats = DataStatsJson {
        vocab: old_stats.vocab,
        sentence: old_stats.sentence,
        dialogue: old_stats.dialogue,
        novel: old_stats.novel,
        image: new_stats.image,
        audio: new_stats.audio,
        wiki: new_stats.wiki,
    };
    
    // 更新状态
    {
        let mut s = state.lock().unwrap();
        s.loaded_stats = loaded_stats.clone();
        s.epoch = checkpoint.epoch;
        s.total_processed = checkpoint.total_processed;
        s.stable_count = checkpoint.stable_count;
        s.fate_preferences = if checkpoint.fate_preferences.is_empty() {
            vec![0.25, 0.25, 0.25, 0.25]
        } else {
            checkpoint.fate_preferences.clone()
        };
        s.is_running = true;
    }
    
    // 合并数据
    let mut all_data = Vec::with_capacity(old_data.len() + new_data.len());
    all_data.append(&mut old_data);
    all_data.append(&mut new_data);
    
    if all_data.is_empty() {
        println!("❌ 没有找到训练数据！");
        return;
    }
    
    println!("✅ 数据加载完成: {} 项", all_data.len());
    
    let mut rng = rand::thread_rng();
    let total_start = Instant::now();
    let mut last_save = Instant::now();
    let data_len = all_data.len();
    
    let mut prev_prefs: Option<Vec<f64>> = Some(checkpoint.fate_preferences.clone());
    
    // =========================================================================
    // Valhalla 累积演化训练
    // 周期1: 处理原始数据 → 产生Fate状态
    // 周期2+: 基于上一轮的Fate输出继续演化（不重复处理原始数据）
    // =========================================================================
    
    // 周期1: 首次处理所有原始数据
    if checkpoint.epoch == 0 {
        checkpoint.epoch = 1;
        println!("🌀 周期 1: 处理原始数据 ({} 项)...", data_len);
        
        all_data.shuffle(&mut rng);
        let mut current_stats = DataStatsJson::default();
        
        for (idx, item) in all_data.iter().enumerate() {
            if !running.load(Ordering::Relaxed) { break; }
            
            let (request_type, intensity) = match item {
                DataItem::Vocabulary { word } => {
                    current_stats.vocab += 1;
                    (RequestType::Learning, (word.len() as f64 / 20.0).min(1.0).max(0.1))
                },
                DataItem::Sentence { text } => {
                    current_stats.sentence += 1;
                    (RequestType::DataProcessing, (text.len() as f64 / 200.0).min(1.0).max(0.1))
                },
                DataItem::Dialogue { text } => {
                    current_stats.dialogue += 1;
                    (RequestType::Adaptation, (text.len() as f64 / 150.0).min(1.0).max(0.1))
                },
                DataItem::Novel { text } => {
                    current_stats.novel += 1;
                    (RequestType::DataProcessing, (text.len() as f64 / 500.0).min(1.0).max(0.1))
                },
                DataItem::Image { features } => {
                    current_stats.image += 1;
                    (RequestType::DataProcessing, features[0].max(0.1))
                },
                DataItem::Audio { features } => {
                    current_stats.audio += 1;
                    (RequestType::Learning, features[0].max(0.1))
                },
                DataItem::WikiText { text } => {
                    current_stats.wiki += 1;
                    (RequestType::Reasoning, (text.len() as f64 / 500.0).min(1.0).max(0.1))
                },
            };
            
            let request = HubRequest {
                request_id: format!("c1_i{}", idx),
                request_type,
                intensity,
                data: None,
                priority: Priority::Normal,
            };
            
            hub.process(request);
            checkpoint.total_processed += 1;
            
            // 更新UI (每100项)
            if idx % 100 == 0 {
                let prefs = hub.get_fate_preferences();
                let elapsed = total_start.elapsed();
                let rate = checkpoint.total_processed as f64 / elapsed.as_secs_f64().max(0.001);
                let progress = (idx as f64 / data_len as f64) * 100.0;
                
                {
                    let mut s = state.lock().unwrap();
                    s.epoch = 1;
                    s.total_processed = checkpoint.total_processed;
                    s.epoch_progress = progress;
                    s.rate = rate;
                    s.elapsed_secs = elapsed.as_secs();
                    s.fate_preferences = prefs.affinities.clone();
                    s.data_stats = current_stats.clone();
                    s.is_running = true;
                }
                
                checkpoint.fate_preferences = prefs.affinities.clone();
            }
            
            thread::sleep(Duration::from_micros(10));
        }
        
        println!("✅ 周期 1 完成，Fate状态已建立");
    }
    
    // 周期2+: 基于上一轮的Fate输出继续演化
    while running.load(Ordering::SeqCst) {
        checkpoint.epoch += 1;
        let cycle = checkpoint.epoch;
        
        println!("🔄 周期 {}: 基于上一轮Fate状态演化...", cycle);
        
        // 获取上一轮的Fate状态作为本轮输入
        let prev_fate = hub.get_fate_preferences();
        let fate_signal_tick = hub.fate_system().get_logical_time();
        
        // 基于Fate偏好生成演化信号（不重复处理原始数据）
        // 信号数量 = 上一轮累积的信号数 / 10（逐步收敛）
        let evolution_signals = (fate_signal_tick / 10).max(1000).min(50000) as usize;
        
        for i in 0..evolution_signals {
            if !running.load(Ordering::Relaxed) { break; }
            
            // 根据当前Fate偏好决定信号类型和强度
            let (request_type, intensity) = {
                let r: f64 = rng.gen();
                let prefs = &prev_fate.affinities;
                
                // 选择当前偏好最高的系统类型
                let cumsum: Vec<f64> = prefs.iter()
                    .scan(0.0, |acc, &x| { *acc += x; Some(*acc) })
                    .collect();
                
                let selected = cumsum.iter().position(|&c| r < c).unwrap_or(0);
                
                match selected {
                    0 => (RequestType::DataProcessing, prefs[0]),   // BlackHole
                    1 => (RequestType::Learning, prefs[1]),         // MeshBrain
                    2 => (RequestType::Reasoning, prefs[2]),        // MultiNova
                    _ => (RequestType::Adaptation, prefs[3]),       // BaseForce
                }
            };
            
            let request = HubRequest {
                request_id: format!("c{}_e{}", cycle, i),
                request_type,
                intensity: intensity.max(0.1),
                data: None,
                priority: Priority::Normal,
            };
            
            hub.process(request);
            checkpoint.total_processed += 1;
            
            // 更新UI (每100信号)
            if i % 100 == 0 {
                let prefs = hub.get_fate_preferences();
                let elapsed = total_start.elapsed();
                let rate = checkpoint.total_processed as f64 / elapsed.as_secs_f64().max(0.001);
                let progress = (i as f64 / evolution_signals as f64) * 100.0;
                
                // 计算漂移（收敛检测）
                let mut drift = 0.0f64;
                if let Some(ref prev) = prev_prefs {
                    drift = prefs.affinities.iter()
                        .zip(prev.iter())
                        .map(|(a, b)| (a - b).abs())
                        .sum();
                    
                    if drift < config.convergence_threshold {
                        checkpoint.stable_count += 1;
                        if checkpoint.stable_count >= config.convergence_stable_count {
                            println!("🎯 收敛检测: Fate偏好稳定，停止演化");
                            running.store(false, Ordering::SeqCst);
                        }
                    } else {
                        checkpoint.stable_count = 0;
                    }
                }
                prev_prefs = Some(prefs.affinities.clone());
                checkpoint.fate_preferences = prefs.affinities.clone();
                
                {
                    let mut s = state.lock().unwrap();
                    s.epoch = cycle;
                    s.total_processed = checkpoint.total_processed;
                    s.epoch_progress = progress;
                    s.rate = rate;
                    s.elapsed_secs = elapsed.as_secs();
                    s.stable_count = checkpoint.stable_count;
                    s.drift = drift;
                    s.fate_preferences = prefs.affinities.clone();
                    s.is_running = running.load(Ordering::Relaxed);
                }
            }
            
            // 自动保存 (每5分钟)
            if last_save.elapsed() >= Duration::from_secs(300) {
                checkpoint.last_save_time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                if save_checkpoint(&config.checkpoint_path, &checkpoint).is_ok() {
                    let mut s = state.lock().unwrap();
                    s.last_save_time = checkpoint.last_save_time.clone();
                }
                last_save = Instant::now();
            }
            
            thread::sleep(Duration::from_micros(5));
        }
        
        println!("✅ 周期 {} 完成", cycle);
    }
    
    // 最终保存
    checkpoint.last_save_time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let final_prefs = hub.get_fate_preferences();
    checkpoint.fate_preferences = final_prefs.affinities.clone();
    let _ = save_checkpoint(&config.checkpoint_path, &checkpoint);
    
    {
        let mut s = state.lock().unwrap();
        s.is_running = false;
        s.last_save_time = checkpoint.last_save_time;
    }
    
    println!("\n✅ 训练已保存并停止");
}

// ═══════════════════════════════════════════════════════════════════
//  主函数
// ═══════════════════════════════════════════════════════════════════

#[tokio::main]
async fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║     🌐 Valhalla Web 可视化训练                                 ║");
    println!("║     http://localhost:8765                                      ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    
    let running = Arc::new(AtomicBool::new(true));
    let training_state = Arc::new(Mutex::new(TrainingStateShared::default()));
    
    let app_state = AppState {
        training_state: training_state.clone(),
        running: running.clone(),
    };
    
    // Ctrl+C 处理
    let running_clone = running.clone();
    ctrlc::set_handler(move || {
        println!("\n⚠️  正在停止...");
        running_clone.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");
    
    // 启动训练线程
    let training_state_clone = training_state.clone();
    let running_clone = running.clone();
    let training_handle = thread::spawn(move || {
        training_thread(training_state_clone, running_clone);
    });
    
    // 启动 Web 服务器
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/ws", get(ws_handler))
        .layer(CorsLayer::permissive())
        .with_state(app_state);
    
    println!("🚀 Web 服务器启动: http://localhost:8765");
    println!("   在浏览器中打开查看训练进度");
    println!("   按 Ctrl+C 停止训练\n");
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8765").await.unwrap();
    
    // 使用tokio::select同时运行服务器和监控训练状态
    let running_for_server = running.clone();
    tokio::select! {
        _ = axum::serve(listener, app) => {
            println!("服务器已关闭");
        }
        _ = async {
            loop {
                tokio::time::sleep(Duration::from_millis(500)).await;
                if !running_for_server.load(Ordering::Relaxed) {
                    break;
                }
            }
        } => {
            println!("\n训练已停止，正在关闭服务器...");
        }
    }
    
    // 等待训练线程结束
    let _ = training_handle.join();
    
    println!("\n🏁 服务已停止");
}
