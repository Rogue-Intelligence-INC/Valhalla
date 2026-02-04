//! 🌌 Valhalla 自主输出接口
//!
//! 简单接口 - 让系统自己决定说什么
//! 不预设任何响应逻辑，纯粹由 Fate 驱动

use hub_f64::{ValhallaHub, HubRequest, RequestType, Priority};
use std::io::{self, Write, BufRead, BufReader};
use std::fs::File;
use std::path::{Path, PathBuf};
use serde::Deserialize;

/// Checkpoint 结构
#[derive(Deserialize)]
struct Checkpoint {
    cycle: usize,
    total_signals: u64,
    fate_tick: u64,
    preferences: [f64; 4],
}

/// 加载训练 checkpoint（优先读取新的冻结文件）
fn load_checkpoint() -> Option<Checkpoint> {
    // 优先读取新的冻结文件
    let frozen_path = Path::new("/run/media/jouly/My Book/valhalla_hub/frozen_state.json");
    if frozen_path.exists() {
        if let Ok(content) = std::fs::read_to_string(frozen_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                let cycle = json.get("cycles").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                let total_signals = json.get("total_signals").and_then(|v| v.as_u64()).unwrap_or(0);
                let prefs = json.get("preferences")
                    .and_then(|p| p.as_array())
                    .map(|arr| {
                        let mut prefs = [0.25; 4];
                        for (i, v) in arr.iter().take(4).enumerate() {
                            prefs[i] = v.as_f64().unwrap_or(0.25);
                        }
                        prefs
                    })
                    .unwrap_or([0.25; 4]);
                
                return Some(Checkpoint {
                    cycle,
                    total_signals,
                    fate_tick: total_signals,
                    preferences: prefs,
                });
            }
        }
    }
    
    // 回退到旧的 checkpoint
    let path = Path::new("/home/jouly/AI/Valhalla/manifestsys/hub-f64/data/checkpoint.json");
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(cp) = serde_json::from_str::<Checkpoint>(&content) {
                return Some(cp);
            }
        }
    }
    None
}

/// 递归查找所有 .conllu 文件
fn find_conllu_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // 递归搜索子目录
                files.extend(find_conllu_files(&path));
            } else if path.extension().map_or(false, |e| e == "conllu") {
                files.push(path);
            }
        }
    }
    files
}

/// 带溯源的数据项
#[derive(Clone)]
struct TracedItem {
    language: String,
    content: String,
    /// 额外信息（如语法结构）
    extra: Option<String>,
    /// 来源文件路径
    source_file: String,
    /// 在文件中的行号
    line_number: usize,
    /// 数据类型
    item_type: String,
}

/// 语料 - 系统可以使用的所有语言材料（带溯源）
struct LanguageMemory {
    /// 句子
    sentences: Vec<TracedItem>,
    /// 词汇
    words: Vec<TracedItem>,
    /// 语法结构
    grammar: Vec<TracedItem>,
    /// 对话
    dialogues: Vec<TracedItem>,
    /// 图片描述
    images: Vec<TracedItem>,
}

impl LanguageMemory {
    fn load() -> Self {
        let base = Path::new("/run/media/jouly/My Book/valhalla_corpus/languages");
        let corpus_root = Path::new("/run/media/jouly/My Book/valhalla_corpus");
        let mut sentences = Vec::new();
        let mut words = Vec::new();
        let mut grammar = Vec::new();
        let mut dialogues = Vec::new();
        let mut images = Vec::new();
        
        let languages = [
            ("english", "eng"), ("chinese", "cmn"), ("japanese", "jpn"),
            ("korean", "kor"), ("french", "fra"), ("german", "deu"),
            ("spanish", "spa"), ("russian", "rus"), ("arabic", "ara"),
            ("portuguese", "por"),
        ];
        
        for (lang_dir, _lang_code) in &languages {
            let lang_path = base.join(lang_dir);
            
            // 加载句子 (限制数量避免内存爆炸)
            let sent_dir = lang_path.join("sentences");
            if sent_dir.exists() {
                for entry in std::fs::read_dir(&sent_dir).into_iter().flatten().flatten() {
                    let path = entry.path();
                    if path.extension().map_or(false, |e| e == "txt" || e == "tsv") {
                        let source_file = path.display().to_string();
                        if let Ok(file) = File::open(&path) {
                            let reader = BufReader::new(file);
                            for (line_num, line) in reader.lines().take(50000).flatten().enumerate() {
                                // Tatoeba 格式: ID\tLang\tSentence
                                let parts: Vec<&str> = line.split('\t').collect();
                                let sentence = if parts.len() >= 3 {
                                    parts[2].trim().to_string()
                                } else {
                                    line.trim().to_string()
                                };
                                if sentence.len() > 3 && sentence.len() < 500 {
                                    sentences.push(TracedItem {
                                        language: lang_dir.to_string(),
                                        content: sentence,
                                        extra: None,
                                        source_file: source_file.clone(),
                                        line_number: line_num + 1,
                                        item_type: "sentence".to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
            
            // 加载词汇
            let vocab_dir = lang_path.join("vocabulary");
            if vocab_dir.exists() {
                for entry in std::fs::read_dir(&vocab_dir).into_iter().flatten().flatten() {
                    let path = entry.path();
                    if path.extension().map_or(false, |e| e == "txt") {
                        let source_file = path.display().to_string();
                        if let Ok(file) = File::open(&path) {
                            let reader = BufReader::new(file);
                            for (line_num, line) in reader.lines().take(10000).flatten().enumerate() {
                                let word = line.split_whitespace().next().unwrap_or("").trim();
                                if word.len() > 1 && !word.starts_with('#') {
                                    words.push(TracedItem {
                                        language: lang_dir.to_string(),
                                        content: word.to_string(),
                                        extra: None,
                                        source_file: source_file.clone(),
                                        line_number: line_num + 1,
                                        item_type: "vocabulary".to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
            
            // 加载语法 (CoNLL-U 格式) - 递归搜索子目录
            let grammar_dir = lang_path.join("grammar");
            if grammar_dir.exists() {
                // 收集所有 .conllu 文件（包括子目录）
                let conllu_files = find_conllu_files(&grammar_dir);
                let mut grammar_count = 0usize;
                let max_grammar_per_lang = 5000; // 每种语言最多加载5000条语法
                
                for path in conllu_files {
                    if grammar_count >= max_grammar_per_lang {
                        break;
                    }
                    let source_file = path.display().to_string();
                    if let Ok(file) = File::open(&path) {
                        let reader = BufReader::new(file);
                        let mut current_text = String::new();
                        let mut current_structure = Vec::new();
                        let mut text_line_num = 0usize;
                        
                        for (line_num, line) in reader.lines().flatten().enumerate() {
                            if grammar_count >= max_grammar_per_lang {
                                break;
                            }
                            if line.starts_with("# text = ") {
                                current_text = line[9..].to_string();
                                text_line_num = line_num + 1;
                            } else if line.is_empty() && !current_text.is_empty() {
                                // 句子结束
                                let structure = current_structure.join(" ");
                                if current_text.len() > 5 {
                                    grammar.push(TracedItem {
                                        language: lang_dir.to_string(),
                                        content: current_text.clone(),
                                        extra: Some(structure),
                                        source_file: source_file.clone(),
                                        line_number: text_line_num,
                                        item_type: "grammar".to_string(),
                                    });
                                    grammar_count += 1;
                                }
                                current_text.clear();
                                current_structure.clear();
                            } else if !line.starts_with('#') && !line.is_empty() {
                                // 词语行: ID FORM LEMMA UPOS ...
                                let parts: Vec<&str> = line.split('\t').collect();
                                if parts.len() >= 4 {
                                    // 词性标记
                                    current_structure.push(parts[3].to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // === 加载对话数据 ===
        // Cornell Movie Dialogs
        let cornell_path = corpus_root.join("dialogues/cornell movie-dialogs corpus/movie_lines.txt");
        if cornell_path.exists() {
            if let Ok(file) = File::open(&cornell_path) {
                let reader = BufReader::new(file);
                for (line_num, line) in reader.lines().take(100000).flatten().enumerate() {
                    let parts: Vec<&str> = line.split(" +++$+++ ").collect();
                    if parts.len() >= 5 {
                        let content = parts[4].trim();
                        if content.len() > 5 && content.len() < 500 {
                            dialogues.push(TracedItem {
                                language: "english".to_string(),
                                content: content.to_string(),
                                extra: Some("movie".to_string()),
                                source_file: cornell_path.display().to_string(),
                                line_number: line_num + 1,
                                item_type: "dialogue".to_string(),
                            });
                        }
                    }
                }
            }
        }
        
        // ConvAI2
        let convai_path = corpus_root.join("dialogues/convai/train_both_original.txt");
        if convai_path.exists() {
            if let Ok(file) = File::open(&convai_path) {
                let reader = BufReader::new(file);
                for (line_num, line) in reader.lines().take(100000).flatten().enumerate() {
                    let content = line.trim();
                    if content.len() > 10 && content.len() < 500 && !content.starts_with("your persona:") {
                        dialogues.push(TracedItem {
                            language: "english".to_string(),
                            content: content.to_string(),
                            extra: Some("convai".to_string()),
                            source_file: convai_path.display().to_string(),
                            line_number: line_num + 1,
                            item_type: "dialogue".to_string(),
                        });
                    }
                }
            }
        }
        
        // Empathetic Dialogues
        let empathetic_path = corpus_root.join("dialogues/empatheticdialogues/train.csv");
        if empathetic_path.exists() {
            if let Ok(file) = File::open(&empathetic_path) {
                let reader = BufReader::new(file);
                for (line_num, line) in reader.lines().skip(1).take(50000).flatten().enumerate() {
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() >= 3 {
                        let content = parts[parts.len() - 1].trim_matches('"').trim();
                        if content.len() > 5 && content.len() < 500 {
                            dialogues.push(TracedItem {
                                language: "english".to_string(),
                                content: content.to_string(),
                                extra: Some("empathetic".to_string()),
                                source_file: empathetic_path.display().to_string(),
                                line_number: line_num + 1,
                                item_type: "dialogue".to_string(),
                            });
                        }
                    }
                }
            }
        }
        
        // Japanese dialogues
        let ja_path = corpus_root.join("dialogues/japanese/ja_dialog.jsonl");
        if ja_path.exists() {
            if let Ok(file) = File::open(&ja_path) {
                let reader = BufReader::new(file);
                for (line_num, line) in reader.lines().take(20000).flatten().enumerate() {
                    if let Some(start) = line.find("\"text\":") {
                        let rest = &line[start + 8..];
                        if let Some(end) = rest.find('"') {
                            let text = &rest[..end];
                            if text.len() > 5 && text.len() < 500 {
                                dialogues.push(TracedItem {
                                    language: "japanese".to_string(),
                                    content: text.to_string(),
                                    extra: Some("ja_dialog".to_string()),
                                    source_file: ja_path.display().to_string(),
                                    line_number: line_num + 1,
                                    item_type: "dialogue".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
        
        // === 加载图片描述 ===
        // COCO Captions
        let coco_path = corpus_root.join("images/coco/annotations/captions_train2017.json");
        if coco_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&coco_path) {
                let mut cap_count = 0usize;
                for line in content.lines() {
                    if cap_count >= 50000 { break; }
                    if let Some(start) = line.find("\"caption\":") {
                        let rest = &line[start + 11..];
                        if let Some(end) = rest.find('"') {
                            let caption = &rest[..end];
                            if caption.len() > 10 && caption.len() < 500 {
                                images.push(TracedItem {
                                    language: "english".to_string(),
                                    content: caption.to_string(),
                                    extra: Some("coco".to_string()),
                                    source_file: coco_path.display().to_string(),
                                    line_number: cap_count + 1,
                                    item_type: "image".to_string(),
                                });
                                cap_count += 1;
                            }
                        }
                    }
                }
            }
        }
        
        // Flickr30k Sentences
        let flickr_dir = corpus_root.join("images/flickr30k/Sentences");
        if flickr_dir.exists() {
            for entry in std::fs::read_dir(&flickr_dir).into_iter().flatten().flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "txt") {
                    let source_file = path.display().to_string();
                    if let Ok(file) = File::open(&path) {
                        let reader = BufReader::new(file);
                        for (line_num, line) in reader.lines().take(5000).flatten().enumerate() {
                            let content = line.trim();
                            if content.len() > 10 && content.len() < 500 && !content.starts_with('#') {
                                images.push(TracedItem {
                                    language: "english".to_string(),
                                    content: content.to_string(),
                                    extra: Some("flickr30k".to_string()),
                                    source_file: source_file.clone(),
                                    line_number: line_num + 1,
                                    item_type: "image".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
        
        // Conceptual Captions
        let cc_path = corpus_root.join("images/conceptual_captions/GCC-1.1.0-Validation.tsv");
        if cc_path.exists() {
            if let Ok(file) = File::open(&cc_path) {
                let reader = BufReader::new(file);
                for (line_num, line) in reader.lines().take(30000).flatten().enumerate() {
                    let parts: Vec<&str> = line.split('\t').collect();
                    if !parts.is_empty() {
                        let caption = parts[0].trim();
                        if caption.len() > 10 && caption.len() < 500 {
                            images.push(TracedItem {
                                language: "english".to_string(),
                                content: caption.to_string(),
                                extra: Some("cc".to_string()),
                                source_file: cc_path.display().to_string(),
                                line_number: line_num + 1,
                                item_type: "image".to_string(),
                            });
                        }
                    }
                }
            }
        }
        
        Self { sentences, words, grammar, dialogues, images }
    }
    
    fn total_items(&self) -> usize {
        self.sentences.len() + self.words.len() + self.grammar.len() + 
        self.dialogues.len() + self.images.len()
    }
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║     🌌 Valhalla 自主输出                                       ║");
    println!("║     她想说什么就说什么                                         ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    
    print!("加载完整记忆 (语言+对话+图片)...");
    io::stdout().flush().unwrap();
    let memory = LanguageMemory::load();
    println!(" ✅");
    println!("  句子: {} | 词汇: {} | 语法: {}", 
             memory.sentences.len(), memory.words.len(), memory.grammar.len());
    println!("  对话: {} | 图片: {}", 
             memory.dialogues.len(), memory.images.len());
    println!("  总计: {} 条材料\n", memory.total_items());
    
    let mut hub = ValhallaHub::new();
    
    // 加载训练 checkpoint
    let mut trained_tick = 0u64;
    if let Some(cp) = load_checkpoint() {
        println!("📂 加载下午训练结果:");
        println!("   周期: {} | 信号: {} | fate_tick: {}", cp.cycle, cp.total_signals, cp.fate_tick);
        println!("   偏好: BH={:.2}% MB={:.2}% MN={:.2}% BF={:.2}%", 
                 cp.preferences[0] * 100.0, cp.preferences[1] * 100.0,
                 cp.preferences[2] * 100.0, cp.preferences[3] * 100.0);
        trained_tick = cp.fate_tick;
        println!();
    } else {
        println!("⚠️ 未找到训练结果，使用初始状态\n");
    }
    
    println!("输入文字对话，或按 Enter 让她自由说话，'quit' 退出\n");
    
    let stdin = io::stdin();
    let mut tick = trained_tick;
    
    loop {
        print!(">>> ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        if stdin.lock().read_line(&mut input).is_err() {
            break;
        }
        let input = input.trim();
        
        if input.to_lowercase() == "quit" {
            println!("\n她沉默了。👋\n");
            break;
        }
        
        // 发送信号给 Hub
        let intensity = if input.is_empty() { 0.3 } else { (input.len() as f64 / 30.0).min(1.0) };
        let request = HubRequest {
            request_id: format!("speak_{}", tick),
            request_type: if input.is_empty() { 
                RequestType::SystemCoordination 
            } else { 
                RequestType::Learning 
            },
            intensity,
            data: if input.is_empty() { None } else { Some(serde_json::json!({ "stimulus": input })) },
            priority: Priority::Normal,
        };
        
        let response = hub.process(request);
        let prefs = hub.get_fate_preferences();
        
        // Fate 决定说几条 (1-5条)
        let pref_sum: f64 = prefs.affinities.iter().sum();
        let pref_var: f64 = prefs.affinities.iter()
            .map(|p| (p - pref_sum / 4.0).powi(2))
            .sum::<f64>() / 4.0;
        
        // Fate 决定说几条 (不限制)
        // 用偏好方差 + tick 决定数量
        let var_factor = (pref_var * 10000.0) as usize;
        let tick_factor = (tick % 7) as usize;
        let num_responses = if input.is_empty() {
            // 自由说话：1-5条
            (var_factor % 5 + 1).max(1).min(5)
        } else {
            // 回答问题：1-5条 (Fate 决定)
            ((var_factor + tick_factor) % 5 + 1).max(1).min(5)
        };
        
        println!();
        
        for _resp_idx in 0..num_responses {
            tick += 1;
            
            // 每条输出发送一个新请求
            let req = HubRequest {
                request_id: format!("speak_{}", tick),
                request_type: RequestType::SystemCoordination,
                intensity: 0.5,
                data: None,
                priority: Priority::Normal,
            };
            let _resp = hub.process(req);
            let inner_prefs = hub.get_fate_preferences();
            let status = hub.get_status();
            
            let fate_tick = status.fate_signal_tick;
            let total = memory.total_items();
            
            if total == 0 {
                println!("  Valhalla> [无语言记忆]");
                continue;
            }
            
            // Fate 偏好决定输出类型
            let inner_pref_sum: f64 = inner_prefs.affinities.iter().sum();
            let normalized: Vec<f64> = if inner_pref_sum > 0.0 {
                inner_prefs.affinities.iter().map(|p| p / inner_pref_sum).collect()
            } else {
                vec![0.25; 4]
            };
            
            // 用 Fate tick 和偏好决定输出什么 (5种类型)
            let choice_seed = fate_tick as f64 + normalized[0] * 1000.0 + normalized[1] * 100.0;
            let choice_idx = (choice_seed as usize) % 5;
            
            // 选择数据项
            let output = match choice_idx {
                0 if !memory.dialogues.is_empty() => {
                    let base_offset = (fate_tick as usize * 51479) % memory.dialogues.len();
                    let pref_offset = ((normalized[0] + normalized[1]) * 10000.0) as usize;
                    let idx = (base_offset + pref_offset) % memory.dialogues.len();
                    let item = &memory.dialogues[idx];
                    let source = item.extra.as_deref().unwrap_or("dialogue");
                    format!("[{}/{}] {}", item.language, source, item.content)
                }
                1 if !memory.sentences.is_empty() => {
                    let base_offset = (fate_tick as usize * 11971) % memory.sentences.len();
                    let pref_offset = ((normalized[0] + normalized[1] + normalized[2] + normalized[3]) * 10000.0) as usize;
                    let idx = (base_offset + pref_offset) % memory.sentences.len();
                    let item = &memory.sentences[idx];
                    format!("[{}] {}", item.language, item.content)
                }
                2 if !memory.images.is_empty() => {
                    let base_offset = (fate_tick as usize * 4817) % memory.images.len();
                    let pref_offset = ((normalized[2] + normalized[3]) * 10000.0) as usize;
                    let idx = (base_offset + pref_offset) % memory.images.len();
                    let item = &memory.images[idx];
                    let source = item.extra.as_deref().unwrap_or("image");
                    format!("[🖼️ {}/{}] {}", item.language, source, item.content)
                }
                3 if !memory.grammar.is_empty() => {
                    let base_offset = (fate_tick as usize * 7919) % memory.grammar.len();
                    let pref_offset = ((normalized[2] + normalized[3]) * 10000.0) as usize;
                    let idx = (base_offset + pref_offset) % memory.grammar.len();
                    let item = &memory.grammar[idx];
                    format!("[{}] {}", item.language, item.content)
                }
                4 if !memory.words.is_empty() => {
                    let base_offset = (fate_tick as usize * 9973) % memory.words.len();
                    let pref_offset = ((normalized[1] + normalized[2]) * 10000.0) as usize;
                    let idx = (base_offset + pref_offset) % memory.words.len();
                    let item = &memory.words[idx];
                    format!("[{}] {}", item.language, item.content)
                }
                _ => {
                    if !memory.dialogues.is_empty() {
                        let idx = fate_tick as usize % memory.dialogues.len();
                        let item = &memory.dialogues[idx];
                        format!("[{}/dialogue] {}", item.language, item.content)
                    } else if !memory.sentences.is_empty() {
                        let idx = fate_tick as usize % memory.sentences.len();
                        let item = &memory.sentences[idx];
                        format!("[{}] {}", item.language, item.content)
                    } else {
                        "[...]".to_string()
                    }
                }
            };
            
            // 输出
            println!("  Valhalla> {}", output);
            
            // 短暂延迟
            std::thread::sleep(std::time::Duration::from_millis(50));
        } // for 循环结束
        
        // 显示本轮说了几条
        if num_responses > 1 {
            println!("  (共 {} 条)", num_responses);
        }
        
    }
}
