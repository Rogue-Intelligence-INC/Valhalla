//! 🌌 Valhalla 循环收敛训练
//!
//! 核心理念: 以 Fate 收敛为一个周期，进行多周期训练
//! 每个周期结束时记录 Fate 状态，观察跨周期的演化

use hub_f64::{ValhallaHub, HubRequest, RequestType, Priority};
use std::fs::{self, File};
use std::io::{self, Write, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use std::thread;
use std::env;
use rand::seq::SliceRandom;
use rand::thread_rng;

/// Checkpoint 数据结构
#[derive(Debug)]
struct Checkpoint {
    cycle: usize,
    total_signals: u64,
    fate_tick: u64,
}

/// 保存 checkpoint
fn save_checkpoint(path: &Path, cycle: usize, total_signals: u64, hub: &ValhallaHub) {
    let prefs = hub.get_fate_preferences();
    let status = hub.get_status();
    let json = format!(
        r#"{{
  "cycle": {},
  "total_signals": {},
  "fate_tick": {},
  "preferences": [{:.6}, {:.6}, {:.6}, {:.6}],
  "timestamp": "{:?}"
}}"#,
        cycle, total_signals, status.fate_signal_tick,
        prefs.affinities[0], prefs.affinities[1], prefs.affinities[2], prefs.affinities[3],
        std::time::SystemTime::now()
    );
    let _ = fs::write(path, json);
}

/// 加载 checkpoint
fn load_checkpoint(path: &Path) -> Option<Checkpoint> {
    let content = fs::read_to_string(path).ok()?;
    // 简单解析
    let cycle = content.find("\"cycle\":")?;
    let cycle_start = cycle + 8;
    let cycle_end = content[cycle_start..].find(',')?;
    let cycle_num: usize = content[cycle_start..cycle_start + cycle_end].trim().parse().ok()?;
    
    let signals = content.find("\"total_signals\":")?;
    let signals_start = signals + 16;
    let signals_end = content[signals_start..].find(',')?;
    let total_signals: u64 = content[signals_start..signals_start + signals_end].trim().parse().ok()?;
    
    let tick = content.find("\"fate_tick\":")?;
    let tick_start = tick + 12;
    let tick_end = content[tick_start..].find(',')?;
    let fate_tick: u64 = content[tick_start..tick_start + tick_end].trim().parse().ok()?;
    
    Some(Checkpoint { cycle: cycle_num, total_signals, fate_tick })
}

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

/// 模态类型
#[derive(Debug, Clone, Copy, PartialEq)]
enum Modality {
    Language,
    Code,
    Math,
    Dialogue,
    Narrative,
    Image,  // 新增: 图片描述
}

/// 统一数据项
struct UnifiedItem {
    content: String,
    modality: Modality,
    features: Vec<f64>,
}

impl UnifiedItem {
    fn extract_features(content: &str, modality: Modality) -> Vec<f64> {
        let len = content.len() as f64;
        let chars: Vec<char> = content.chars().collect();
        let char_count = chars.len() as f64;
        
        let alpha_ratio = chars.iter().filter(|c| c.is_alphabetic()).count() as f64 / char_count.max(1.0);
        let digit_ratio = chars.iter().filter(|c| c.is_numeric()).count() as f64 / char_count.max(1.0);
        let space_ratio = chars.iter().filter(|c| c.is_whitespace()).count() as f64 / char_count.max(1.0);
        let punct_ratio = chars.iter().filter(|c| c.is_ascii_punctuation()).count() as f64 / char_count.max(1.0);
        let bracket_ratio = chars.iter().filter(|c| "()[]{}".contains(**c)).count() as f64 / char_count.max(1.0);
        let operator_ratio = chars.iter().filter(|c| "+-*/=<>".contains(**c)).count() as f64 / char_count.max(1.0);
        
        let line_count = content.lines().count() as f64;
        let avg_line_len = if line_count > 0.0 { len / line_count } else { len };
        
        let unique_chars = {
            let mut seen = std::collections::HashSet::new();
            for c in chars.iter() { seen.insert(*c); }
            seen.len() as f64
        };
        let entropy_estimate = if char_count > 0.0 { unique_chars / char_count.sqrt() } else { 0.0 };
        
        let modality_bias = match modality {
            Modality::Language => 0.1,
            Modality::Code => 0.3,
            Modality::Math => 0.5,
            Modality::Dialogue => 0.2,
            Modality::Narrative => 0.15,
            Modality::Image => 0.4,  // 图片描述: 视觉+语言混合
        };
        
        vec![
            (len / 1000.0).min(1.0),
            alpha_ratio,
            digit_ratio,
            space_ratio,
            punct_ratio,
            bracket_ratio * 10.0,
            operator_ratio * 10.0,
            (avg_line_len / 100.0).min(1.0),
            entropy_estimate.min(1.0),
            modality_bias,
        ]
    }
    
    fn new(content: String, modality: Modality) -> Self {
        let features = Self::extract_features(&content, modality);
        Self { content, modality, features }
    }
    
    fn intensity(&self) -> f64 {
        let sum: f64 = self.features.iter().sum();
        (sum / self.features.len() as f64).min(1.0).max(0.1)
    }
}

/// 加载扩展语料库 (包含新增对话和小说数据)
fn load_extended_corpus() -> Vec<UnifiedItem> {
    let mut items = Vec::new();
    let corpus_root = Path::new("/run/media/jouly/My Book/valhalla_corpus");
    let valhalla_root = Path::new("/home/jouly/AI/Valhalla");
    
    println!("📦 加载扩展语料库 (增强版)...\n");
    
    // === 1. 语言模态 (完整加载: 句子 + 词汇 + 语法) ===
    print!("  🗣️  语言模态 (句子/词汇/语法)...");
    io::stdout().flush().unwrap();
    let lang_path = corpus_root.join("languages");
    let languages = ["english", "chinese", "japanese", "korean", "french", 
                     "german", "spanish", "russian", "arabic", "portuguese"];
    
    let mut sent_count = 0usize;
    let mut vocab_count = 0usize;
    let mut grammar_count = 0usize;
    
    for lang in &languages {
        let lang_dir = lang_path.join(lang);
        
        // 1a. 加载句子 (无限流!)
        let sent_path = lang_dir.join("sentences");
        if sent_path.exists() {
            for entry in fs::read_dir(&sent_path).into_iter().flatten().flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "txt" || e == "tsv") {
                    if let Ok(file) = File::open(&path) {
                        let reader = BufReader::new(file);
                        for line in reader.lines().flatten() {  // 无限流!
                            let parts: Vec<&str> = line.split('\t').collect();
                            let sentence = if parts.len() >= 3 {
                                parts[2].trim().to_string()
                            } else {
                                line.trim().to_string()
                            };
                            if sentence.len() > 5 && sentence.len() < 500 {
                                items.push(UnifiedItem::new(sentence, Modality::Language));
                                sent_count += 1;
                            }
                        }
                    }
                }
            }
        }
        
        // 1b. 加载词汇 (无限流!)
        let vocab_path = lang_dir.join("vocabulary");
        if vocab_path.exists() {
            for entry in fs::read_dir(&vocab_path).into_iter().flatten().flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "txt") {
                    if let Ok(file) = File::open(&path) {
                        let reader = BufReader::new(file);
                        for line in reader.lines().flatten() {  // 无限流!
                            let word = line.split_whitespace().next().unwrap_or("").trim();
                            if word.len() > 1 && !word.starts_with('#') {
                                items.push(UnifiedItem::new(word.to_string(), Modality::Language));
                                vocab_count += 1;
                            }
                        }
                    }
                }
            }
        }
        
        // 1c. 加载语法 (递归搜索子目录中的 .conllu 文件, 无限流!)
        let grammar_path = lang_dir.join("grammar");
        if grammar_path.exists() {
            let conllu_files = find_conllu_files(&grammar_path);
            
            for path in conllu_files {
                if let Ok(file) = File::open(&path) {
                    let reader = BufReader::new(file);
                    let mut current_text = String::new();
                    
                    for line in reader.lines().flatten() {
                        if line.starts_with("# text = ") {
                            current_text = line[9..].to_string();
                        } else if line.is_empty() && !current_text.is_empty() {
                            if current_text.len() > 5 && current_text.len() < 500 {
                                items.push(UnifiedItem::new(current_text.clone(), Modality::Language));
                                grammar_count += 1;
                            }
                            current_text.clear();
                        }
                    }
                }
            }
        }
    }
    println!(" {} 条 (句子:{}, 词汇:{}, 语法:{})", 
        sent_count + vocab_count + grammar_count, sent_count, vocab_count, grammar_count);
    let _lang_count = items.len();
    
    // === 2. 代码模态 (加载 Valhalla 项目代码) ===
    print!("  💻 代码模态...");
    io::stdout().flush().unwrap();
    let code_start = items.len();
    
    // Valhalla Rust 代码
    let rust_path = corpus_root.join("code/valhalla_rust.txt");
    if rust_path.exists() {
        if let Ok(content) = fs::read_to_string(&rust_path) {
            for chunk in content.split("---FILE_SEPARATOR---") {
                for block in chunk.split("\n\n").filter(|s| s.len() > 20 && s.len() < 2000) {
                    items.push(UnifiedItem::new(block.to_string(), Modality::Code));
                }
            }
        }
    }
    
    // Valhalla Wave 代码
    let wave_path = corpus_root.join("code/valhalla_wave.txt");
    if wave_path.exists() {
        if let Ok(content) = fs::read_to_string(&wave_path) {
            for chunk in content.split("---FILE_SEPARATOR---") {
                for block in chunk.split("\n\n").filter(|s| s.len() > 20 && s.len() < 2000) {
                    items.push(UnifiedItem::new(block.to_string(), Modality::Code));
                }
            }
        }
    }
    
    // Valhalla Cosmic 代码
    let cosmic_path = corpus_root.join("code/valhalla_cosmic.txt");
    if cosmic_path.exists() {
        if let Ok(content) = fs::read_to_string(&cosmic_path) {
            for chunk in content.split("---FILE_SEPARATOR---") {
                for block in chunk.split("\n\n").filter(|s| s.len() > 20 && s.len() < 2000) {
                    items.push(UnifiedItem::new(block.to_string(), Modality::Code));
                }
            }
        }
    }
    
    // 直接从源文件加载
    let src_dirs = [
        "manifestsys/hub-f64/src",
        "manifestsys/hub/src", 
        "systems/metasystems/fate-f64/src",
        "systems/metasystems/fate/src",
        "valhalla-rational/src",
    ];
    for src_dir in &src_dirs {
        let src_path = valhalla_root.join(src_dir);
        if src_path.exists() {
            for entry in fs::read_dir(&src_path).into_iter().flatten().flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "rs") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        for block in content.split("\n\n").filter(|s| s.len() > 30 && s.len() < 1500) {
                            items.push(UnifiedItem::new(block.to_string(), Modality::Code));
                        }
                    }
                }
            }
        }
    }
    println!(" {} 条", items.len() - code_start);
    
    // === 3. 数学模态 ===
    print!("  🔢 数学模态...");
    io::stdout().flush().unwrap();
    let math_start = items.len();
    
    // 基础公式
    let formulas_path = corpus_root.join("math/formulas/basic_formulas.txt");
    if formulas_path.exists() {
        if let Ok(file) = File::open(&formulas_path) {
            let reader = BufReader::new(file);
            for line in reader.lines().flatten() {
                if line.len() > 3 {
                    items.push(UnifiedItem::new(line, Modality::Math));
                }
            }
        }
    }
    
    // 从 Valhalla 文档提取数学内容
    let docs_path = valhalla_root.join("docs");
    if docs_path.exists() {
        for entry in fs::read_dir(&docs_path).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "md") {
                if let Ok(content) = fs::read_to_string(&path) {
                    for line in content.lines() {
                        if line.contains("formula") || line.contains("equation") || 
                           (line.contains("=") && line.contains("^")) ||
                           line.contains("∫") || line.contains("∑") {
                            items.push(UnifiedItem::new(line.to_string(), Modality::Math));
                        }
                    }
                }
            }
        }
    }
    println!(" {} 条", items.len() - math_start);
    
    // === 4. 对话模态 (大幅扩展) ===
    print!("  💬 对话模态 (电影/角色/情感/多语言)...");
    io::stdout().flush().unwrap();
    let dialogue_start = items.len();
    
    // 4a. Cornell Movie Dialogs (304k行, 无限流!)
    let cornell_path = corpus_root.join("dialogues/cornell movie-dialogs corpus/movie_lines.txt");
    if cornell_path.exists() {
        if let Ok(file) = File::open(&cornell_path) {
            let reader = BufReader::new(file);
            for line in reader.lines().flatten() {
                // 格式: L123 +++$+++ u456 +++$+++ m789 +++$+++ CHARACTER +++$+++ 对话内容
                let parts: Vec<&str> = line.split(" +++$+++ ").collect();
                if parts.len() >= 5 {
                    let content = parts[4].trim();
                    if content.len() > 5 && content.len() < 500 {
                        items.push(UnifiedItem::new(content.to_string(), Modality::Dialogue));
                    }
                }
            }
        }
    }
    
    // 4b. ConvAI2 (292k行, 无限流!)
    let convai_path = corpus_root.join("dialogues/convai/train_both_original.txt");
    if convai_path.exists() {
        if let Ok(file) = File::open(&convai_path) {
            let reader = BufReader::new(file);
            for line in reader.lines().flatten() {
                let content = line.trim();
                if content.len() > 10 && content.len() < 500 && !content.starts_with("your persona:") {
                    items.push(UnifiedItem::new(content.to_string(), Modality::Dialogue));
                }
            }
        }
    }
    
    // 4c. Empathetic Dialogues (84k行, 无限流!)
    let empathetic_path = corpus_root.join("dialogues/empatheticdialogues/train.csv");
    if empathetic_path.exists() {
        if let Ok(file) = File::open(&empathetic_path) {
            let reader = BufReader::new(file);
            for line in reader.lines().skip(1).flatten() {
                // CSV 格式，取最后几列
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() >= 3 {
                    let content = parts[parts.len() - 1].trim_matches('"').trim();
                    if content.len() > 5 && content.len() < 500 {
                        items.push(UnifiedItem::new(content.to_string(), Modality::Dialogue));
                    }
                }
            }
        }
    }
    
    // 4d. 日语对话 (21k行, 无限流!)
    let ja_path = corpus_root.join("dialogues/japanese/ja_dialog.jsonl");
    if ja_path.exists() {
        if let Ok(file) = File::open(&ja_path) {
            let reader = BufReader::new(file);
            for line in reader.lines().flatten() {
                // 提取 "text" 字段
                if let Some(start) = line.find("\"text\":") {
                    let rest = &line[start + 8..];
                    if let Some(end) = rest.find('"') {
                        let text = &rest[..end];
                        if text.len() > 5 && text.len() < 500 {
                            items.push(UnifiedItem::new(text.to_string(), Modality::Dialogue));
                        }
                    }
                }
            }
        }
    }
    
    // 4e. 原有对话目录
    let dialogue_dirs = [
        "content/dialogues",
        "content/dialogues/empatheticdialogues",
        "content/dialogues/extra",
    ];
    
    for dir in &dialogue_dirs {
        let dialogue_path = corpus_root.join(dir);
        if dialogue_path.exists() {
            for entry in fs::read_dir(&dialogue_path).into_iter().flatten().flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "csv" || e == "txt" || e == "json") {
                    if let Ok(file) = File::open(&path) {
                        let reader = BufReader::new(file);
                        for line in reader.lines().flatten() {  // 无限流!
                            if line.len() > 10 && line.len() < 500 && !line.starts_with('{') {
                                let content = if line.contains(',') {
                                    line.split(',').last().unwrap_or(&line).trim_matches('"').to_string()
                                } else {
                                    line.clone()
                                };
                                if content.len() > 5 {
                                    items.push(UnifiedItem::new(content, Modality::Dialogue));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    println!(" {} 条", items.len() - dialogue_start);
    
    // === 5. 叙事模态 (大幅扩展) ===
    print!("  📖 叙事模态 (小说/文学/古典)...");
    io::stdout().flush().unwrap();
    let narrative_start = items.len();
    
    // 5a. Gutenberg 英文小说 (20本, 37MB, 无限流!)
    let gutenberg_path = corpus_root.join("novels/gutenberg");
    if gutenberg_path.exists() {
        for entry in fs::read_dir(&gutenberg_path).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "txt") {
                if let Ok(file) = File::open(&path) {
                    let reader = BufReader::new(file);
                    let mut paragraph = String::new();
                    for line in reader.lines().flatten() {  // 无限流!
                        if line.trim().is_empty() && !paragraph.is_empty() {
                            if paragraph.len() > 50 && paragraph.len() < 1000 {
                                items.push(UnifiedItem::new(paragraph.clone(), Modality::Narrative));
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
    
    // 5b. 中文小说 (红楼梦、三国演义英译 + 鲁迅 + 聊斋, 无限流!)
    let chinese_novel_path = corpus_root.join("novels/chinese");
    if chinese_novel_path.exists() {
        for entry in fs::read_dir(&chinese_novel_path).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "txt") {
                if let Ok(file) = File::open(&path) {
                    let reader = BufReader::new(file);
                    let mut paragraph = String::new();
                    for line in reader.lines().flatten() {  // 无限流!
                        if line.trim().is_empty() && !paragraph.is_empty() {
                            if paragraph.len() > 30 && paragraph.len() < 800 {
                                items.push(UnifiedItem::new(paragraph.clone(), Modality::Narrative));
                            }
                            paragraph.clear();
                        } else if !line.starts_with("<!DOCTYPE") && !line.starts_with("<html") {
                            paragraph.push_str(&line);
                            paragraph.push(' ');
                        }
                    }
                }
            }
        }
    }
    
    // 5c. 中文古典文学 (论语、诗经、幽梦影 JSON)
    let poetry_path = corpus_root.join("novels/chinese/poetry");
    if poetry_path.exists() {
        for entry in fs::read_dir(&poetry_path).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    // 提取 "paragraphs" 或 "content" 字段
                    for line in content.lines() {
                        let trimmed = line.trim();
                        if trimmed.starts_with('"') && trimmed.len() > 10 {
                            // 提取引号内的中文内容
                            let clean = trimmed.trim_matches(|c| c == '"' || c == ',' || c == '[' || c == ']');
                            if clean.len() > 5 && clean.len() < 500 && !clean.contains(':') {
                                items.push(UnifiedItem::new(clean.to_string(), Modality::Narrative));
                            }
                        }
                    }
                }
            }
        }
    }
    
    // 5d. 原有小说目录 (无限流!)
    let novels_path = corpus_root.join("content/novels");
    if novels_path.exists() {
        for entry in fs::read_dir(&novels_path).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "txt") {
                if let Ok(file) = File::open(&path) {
                    let reader = BufReader::new(file);
                    let mut paragraph = String::new();
                    for line in reader.lines().flatten() {  // 无限流!
                        if line.trim().is_empty() && !paragraph.is_empty() {
                            if paragraph.len() > 50 && paragraph.len() < 1000 {
                                items.push(UnifiedItem::new(paragraph.clone(), Modality::Narrative));
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
    
    // === 6. 图片模态 (图片描述 Captions) ===
    print!("  🖼️  图片模态 (COCO/Flickr30k/TextCaps/CC)...");
    io::stdout().flush().unwrap();
    let image_start = items.len();
    
    // 6a. COCO Captions (训练集)
    let coco_path = corpus_root.join("images/coco/annotations/captions_train2017.json");
    if coco_path.exists() {
        if let Ok(content) = fs::read_to_string(&coco_path) {
            // 提取 "caption" 字段
            for line in content.lines() {
                if let Some(start) = line.find("\"caption\":") {
                    let rest = &line[start + 11..];
                    if let Some(end) = rest.find('"') {
                        let caption = &rest[..end];
                        if caption.len() > 10 && caption.len() < 500 {
                            items.push(UnifiedItem::new(caption.to_string(), Modality::Image));
                        }
                    }
                }
            }
        }
    }
    
    // 6b. Flickr30k Sentences
    let flickr_path = corpus_root.join("images/flickr30k/Sentences");
    if flickr_path.exists() {
        for entry in fs::read_dir(&flickr_path).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "txt") {
                if let Ok(file) = File::open(&path) {
                    let reader = BufReader::new(file);
                    for line in reader.lines().flatten() {
                        let content = line.trim();
                        if content.len() > 10 && content.len() < 500 && !content.starts_with('#') {
                            items.push(UnifiedItem::new(content.to_string(), Modality::Image));
                        }
                    }
                }
            }
        }
    }
    
    // 6c. Conceptual Captions (TSV: caption \t url)
    let cc_path = corpus_root.join("images/conceptual_captions/GCC-1.1.0-Validation.tsv");
    if cc_path.exists() {
        if let Ok(file) = File::open(&cc_path) {
            let reader = BufReader::new(file);
            for line in reader.lines().flatten() {
                let parts: Vec<&str> = line.split('\t').collect();
                if !parts.is_empty() {
                    let caption = parts[0].trim();
                    if caption.len() > 10 && caption.len() < 500 {
                        items.push(UnifiedItem::new(caption.to_string(), Modality::Image));
                    }
                }
            }
        }
    }
    
    // 6d. TextCaps (JSON 格式)
    let textcaps_path = corpus_root.join("images/textcaps/TextCaps_0.1_train.json");
    if textcaps_path.exists() {
        if let Ok(content) = fs::read_to_string(&textcaps_path) {
            for line in content.lines() {
                if let Some(start) = line.find("\"caption\":") {
                    let rest = &line[start + 11..];
                    if let Some(end) = rest.find('"') {
                        let caption = &rest[..end];
                        if caption.len() > 10 && caption.len() < 500 {
                            items.push(UnifiedItem::new(caption.to_string(), Modality::Image));
                        }
                    }
                }
            }
        }
    }
    println!(" {} 条", items.len() - image_start);
    
    // 统计各模态
    let lang_count = items.iter().filter(|x| x.modality == Modality::Language).count();
    let code_count = items.iter().filter(|x| x.modality == Modality::Code).count();
    let math_count = items.iter().filter(|x| x.modality == Modality::Math).count();
    let dialogue_count = items.iter().filter(|x| x.modality == Modality::Dialogue).count();
    let narrative_count = items.iter().filter(|x| x.modality == Modality::Narrative).count();
    let image_count = items.iter().filter(|x| x.modality == Modality::Image).count();
    
    println!("\n📊 模态分布:");
    println!("   语言: {} ({:.1}%)", lang_count, lang_count as f64 / items.len() as f64 * 100.0);
    println!("   代码: {} ({:.1}%)", code_count, code_count as f64 / items.len() as f64 * 100.0);
    println!("   数学: {} ({:.1}%)", math_count, math_count as f64 / items.len() as f64 * 100.0);
    println!("   对话: {} ({:.1}%)", dialogue_count, dialogue_count as f64 / items.len() as f64 * 100.0);
    println!("   叙事: {} ({:.1}%)", narrative_count, narrative_count as f64 / items.len() as f64 * 100.0);
    println!("   图片: {} ({:.1}%)", image_count, image_count as f64 / items.len() as f64 * 100.0);
    
    println!("\n📊 总计: {} 条多模态数据 (无限流版本！)", items.len());
    items
}

/// 周期记录
#[derive(Debug)]
struct CycleRecord {
    cycle_num: usize,
    signals_processed: u64,
    duration_secs: f64,
    final_prefs: [f64; 4],
    convergence_drift: f64,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let continue_mode = args.iter().any(|a| a == "--continue" || a == "-c");
    let checkpoint_path = Path::new("/home/jouly/AI/Valhalla/manifestsys/hub-f64/data/checkpoint.json");
    
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║     🌌 Valhalla Fate 自主训练                                  ║");
    println!("║     Fate 代数形式边际效应收敛 | 10 轮 | 70% 资源               ║");
    if continue_mode {
        println!("║     📍 继续训练模式                                            ║");
    }
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    
    let max_cycles = 10;  // 10 轮 Fate 收敛
    let convergence_window = 30;  // 用于外部漂移检测（备用）
    let convergence_threshold = 0.001;  // 备用阈值
    let cpu_limit = 0.7;  // 70% CPU
    let report_interval = 15;  // 更频繁的报告
    
    // 确保 checkpoint 目录存在
    let _ = fs::create_dir_all(checkpoint_path.parent().unwrap());
    
    // 加载语料
    let mut corpus = load_extended_corpus();
    if corpus.is_empty() {
        println!("❌ 无数据！");
        return;
    }
    
    // 尝试加载 checkpoint
    let mut start_cycle = 1usize;
    let mut total_signals = 0u64;
    
    if continue_mode {
        if let Some(cp) = load_checkpoint(checkpoint_path) {
            println!("📂 加载 checkpoint: 周期 {}, 信号 {}, fate_tick {}", 
                     cp.cycle, cp.total_signals, cp.fate_tick);
            start_cycle = cp.cycle + 1;  // 从下一个周期开始
            total_signals = cp.total_signals;
            println!("🔄 从周期 {} 继续训练\n", start_cycle);
        } else {
            println!("⚠️ 无法加载 checkpoint，从头开始\n");
        }
    }
    
    let mut hub = ValhallaHub::new();
    
    // 如果是继续训练，先预热 Hub
    if continue_mode && total_signals > 0 {
        println!("🔥 预热 Hub (快速重放 {} 信号)...", total_signals.min(100000));
        let warmup_count = total_signals.min(100000) as usize;
        corpus.shuffle(&mut thread_rng());
        for (i, item) in corpus.iter().take(warmup_count).enumerate() {
            let request = HubRequest {
                request_id: format!("warmup_{}", i),
                request_type: RequestType::Learning,
                intensity: item.intensity(),
                data: Some(serde_json::json!({ "features": item.features })),
                priority: Priority::Normal,
            };
            let _ = hub.process(request);
        }
        println!("✅ 预热完成\n");
    }
    
    let mut cycle_records: Vec<CycleRecord> = Vec::new();
    let global_start = Instant::now();
    
    let sleep_duration = Duration::from_micros(((1.0 / cpu_limit - 1.0) * 100.0) as u64);
    
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║  🔄 开始循环收敛训练 (周期 {}-{})                              ║", start_cycle, max_cycles);
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    
    for cycle in start_cycle..=max_cycles {
        println!("═══════════════════════════════════════════════════════════════════");
        println!("📍 周期 {}/{} 开始", cycle, max_cycles);
        println!("═══════════════════════════════════════════════════════════════════\n");
        
        let cycle_start = Instant::now();
        let mut cycle_signals = 0u64;
        let mut stable_count = 0usize;
        let mut drift_history: Vec<f64> = Vec::new();
        let mut last_prefs = hub.get_fate_preferences();
        let mut last_report = Instant::now();
        let mut round = 0usize;
        
        // 在每个周期内循环数据直到收敛
        loop {
            round += 1;
            corpus.shuffle(&mut thread_rng());
            
            for (i, item) in corpus.iter().enumerate() {
                cycle_signals += 1;
                total_signals += 1;
                
                let request = HubRequest {
                    request_id: format!("c{}r{}i{}", cycle, round, i),
                    request_type: RequestType::Learning,
                    intensity: item.intensity(),
                    data: Some(serde_json::json!({ "features": item.features })),
                    priority: Priority::Normal,
                };
                
                let _response = hub.process(request);
                
                if sleep_duration.as_micros() > 0 {
                    thread::sleep(sleep_duration);
                }
                
                // 定期报告
                if last_report.elapsed().as_secs() >= report_interval as u64 {
                    // 记录快照用于收敛检测
                    hub.record_fate_snapshot();
                    
                    let prefs = hub.get_fate_preferences();
                    let drift: f64 = prefs.affinities.iter()
                        .zip(last_prefs.affinities.iter())
                        .map(|(a, b)| (a - b).abs())
                        .sum::<f64>() / 4.0;
                    
                    drift_history.push(drift);
                    if drift_history.len() > convergence_window {
                        drift_history.remove(0);
                    }
                    
                    // Fate 自主判断收敛（代数形式边际效应）
                    let fate_converged = hub.has_fate_converged();
                    let fate_pattern = hub.get_fate_convergence_pattern().unwrap_or("探索中");
                    
                    // 备用：外部漂移检测
                    if drift < convergence_threshold {
                        stable_count += 1;
                    } else {
                        stable_count = 0;
                    }
                    
                    let elapsed = cycle_start.elapsed().as_secs_f64();
                    let rate = cycle_signals as f64 / elapsed;
                    
                    println!("┌─────────────────────────────────────────────────────────────┐");
                    println!("│ 周期 {} | 轮次 {} | 信号 {} | 速率 {:.0}/s", cycle, round, cycle_signals, rate);
                    println!("│ 🎯 偏好: BH={:.2}% MB={:.2}% MN={:.2}% BF={:.2}%",
                             prefs.affinities[0] * 100.0,
                             prefs.affinities[1] * 100.0,
                             prefs.affinities[2] * 100.0,
                             prefs.affinities[3] * 100.0);
                    println!("│ 📐 漂移: {:.5} | 模式: {} | Fate收敛: {}", 
                             drift, fate_pattern, if fate_converged { "是" } else { "否" });
                    println!("└─────────────────────────────────────────────────────────────┘\n");
                    
                    last_prefs = prefs;
                    last_report = Instant::now();
                    
                    // 优先使用 Fate 自主判断，否则用外部稳定计数
                    if fate_converged {
                        println!("🎉 Fate 判断: 周期 {} 收敛！(边际效应递减)\n", cycle);
                        break;
                    } else if stable_count >= convergence_window {
                        println!("🎉 周期 {} 收敛！(外部漂移稳定)\n", cycle);
                        break;
                    }
                }
            }
            
            // 检查是否已收敛（可能在循环内已 break）
            if hub.has_fate_converged() || stable_count >= convergence_window {
                break;
            }
            
            // 防止无限循环
            if round > 30 {
                println!("⚠️ 周期 {} 达到最大轮次，强制结束\n", cycle);
                break;
            }
        }
        
        // 记录周期结果
        let final_prefs = hub.get_fate_preferences();
        let cycle_duration = cycle_start.elapsed().as_secs_f64();
        let final_drift = drift_history.last().copied().unwrap_or(0.0);
        
        let record = CycleRecord {
            cycle_num: cycle,
            signals_processed: cycle_signals,
            duration_secs: cycle_duration,
            final_prefs: [
                final_prefs.affinities[0],
                final_prefs.affinities[1],
                final_prefs.affinities[2],
                final_prefs.affinities[3],
            ],
            convergence_drift: final_drift,
        };
        
        println!("┌─────────────────────────────────────────────────────────────┐");
        println!("│ 📊 周期 {} 完成报告                                         │", cycle);
        println!("├─────────────────────────────────────────────────────────────┤");
        println!("│ 信号: {} | 时间: {:.1}s | 速率: {:.0}/s",
                 record.signals_processed, record.duration_secs,
                 record.signals_processed as f64 / record.duration_secs);
        println!("│ 最终偏好:");
        println!("│   BlackHole:  {:.4} ({:.2}%)", record.final_prefs[0], record.final_prefs[0] * 100.0);
        println!("│   MeshBrain:  {:.4} ({:.2}%)", record.final_prefs[1], record.final_prefs[1] * 100.0);
        println!("│   MultiNova:  {:.4} ({:.2}%)", record.final_prefs[2], record.final_prefs[2] * 100.0);
        println!("│   BaseForce:  {:.4} ({:.2}%)", record.final_prefs[3], record.final_prefs[3] * 100.0);
        println!("│ 收敛漂移: {:.6}", record.convergence_drift);
        println!("└─────────────────────────────────────────────────────────────┘\n");
        
        cycle_records.push(record);
        
        // 保存 checkpoint
        save_checkpoint(checkpoint_path, cycle, total_signals, &hub);
        println!("💾 Checkpoint 已保存 (周期 {})\n", cycle);
    }
    
    // 最终总结
    let global_elapsed = global_start.elapsed().as_secs_f64();
    
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║                 📊 循环收敛训练完成                             ║");
    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("║ ⏱️  总时间: {:.2}s ({:.1}分钟)", global_elapsed, global_elapsed / 60.0);
    println!("║ 🔢 总信号: {}", total_signals);
    println!("║ 🔄 完成周期: {}", cycle_records.len());
    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("║ 📈 跨周期偏好演化:");
    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("║ 周期 │ BlackHole │ MeshBrain │ MultiNova │ BaseForce │ 信号数  ║");
    println!("╠════════════════════════════════════════════════════════════════╣");
    
    for r in &cycle_records {
        println!("║  {}   │  {:.2}%   │  {:.2}%   │  {:.2}%   │  {:.2}%   │ {:>7} ║",
                 r.cycle_num,
                 r.final_prefs[0] * 100.0,
                 r.final_prefs[1] * 100.0,
                 r.final_prefs[2] * 100.0,
                 r.final_prefs[3] * 100.0,
                 r.signals_processed);
    }
    
    println!("╚════════════════════════════════════════════════════════════════╝");
    
    // 分析跨周期变化
    if cycle_records.len() >= 2 {
        let first = &cycle_records[0];
        let last = &cycle_records[cycle_records.len() - 1];
        
        println!("\n💡 跨周期变化分析:");
        println!("   BlackHole: {:.2}% → {:.2}% (Δ{:+.2}%)",
                 first.final_prefs[0] * 100.0, last.final_prefs[0] * 100.0,
                 (last.final_prefs[0] - first.final_prefs[0]) * 100.0);
        println!("   MeshBrain: {:.2}% → {:.2}% (Δ{:+.2}%)",
                 first.final_prefs[1] * 100.0, last.final_prefs[1] * 100.0,
                 (last.final_prefs[1] - first.final_prefs[1]) * 100.0);
        println!("   MultiNova: {:.2}% → {:.2}% (Δ{:+.2}%)",
                 first.final_prefs[2] * 100.0, last.final_prefs[2] * 100.0,
                 (last.final_prefs[2] - first.final_prefs[2]) * 100.0);
        println!("   BaseForce: {:.2}% → {:.2}% (Δ{:+.2}%)",
                 first.final_prefs[3] * 100.0, last.final_prefs[3] * 100.0,
                 (last.final_prefs[3] - first.final_prefs[3]) * 100.0);
    }
    
    println!("\n🏁 训练完成！可以进行互动测试。\n");
}
