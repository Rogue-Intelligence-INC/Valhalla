//! 🌌 Valhalla 语料预处理器
//!
//! 处理原始语料并导出为 Valhalla 格式
//! 处理一次，永久使用

use std::fs::{self, File};
use std::io::{self, Write, BufRead, BufReader, BufWriter};
use std::path::Path;

/// 处理后的数据项
#[derive(Debug)]
struct ProcessedItem {
    content: String,
    modality: String,
    language: String,
    source: String,
    features: [f64; 10],
}

impl ProcessedItem {
    fn to_line(&self) -> String {
        format!(
            "{}\t{}\t{}\t{}\t{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6}",
            self.modality,
            self.language,
            self.source,
            self.content.replace('\t', " ").replace('\n', " "),
            self.features[0], self.features[1], self.features[2], self.features[3], self.features[4],
            self.features[5], self.features[6], self.features[7], self.features[8], self.features[9],
        )
    }
    
    fn extract_features(content: &str, modality: &str) -> [f64; 10] {
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
            "language" => 0.1,
            "code" => 0.3,
            "math" => 0.5,
            "dialogue" => 0.2,
            "narrative" => 0.15,
            "image" => 0.4,
            _ => 0.25,
        };
        
        [
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
}

/// 递归查找所有 .conllu 文件
fn find_conllu_files(dir: &Path) -> Vec<std::path::PathBuf> {
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

fn main() -> io::Result<()> {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║     🌌 Valhalla 语料预处理器                                   ║");
    println!("║     处理一次，永久使用                                         ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    
    let corpus_root = Path::new("/run/media/jouly/My Book/valhalla_corpus");
    let output_path = corpus_root.join("valhalla_processed.tsv");
    
    println!("📂 输入: {}", corpus_root.display());
    println!("📦 输出: {}\n", output_path.display());
    
    let output_file = File::create(&output_path)?;
    let mut writer = BufWriter::new(output_file);
    
    // 写入头部
    writeln!(writer, "# Valhalla Processed Corpus")?;
    writeln!(writer, "# Format: modality\\tlanguage\\tsource\\tcontent\\tfeatures[10]")?;
    writeln!(writer, "# Generated: {:?}", std::time::SystemTime::now())?;
    writeln!(writer, "#")?;
    
    let mut total_count = 0usize;
    
    // === 1. 语言模态 ===
    println!("🗣️  处理语言模态...");
    let lang_path = corpus_root.join("languages");
    let languages = ["english", "chinese", "japanese", "korean", "french", 
                     "german", "spanish", "russian", "arabic", "portuguese"];
    
    for lang in &languages {
        let lang_dir = lang_path.join(lang);
        let mut lang_count = 0usize;
        
        // 句子
        let sent_path = lang_dir.join("sentences");
        if sent_path.exists() {
            for entry in fs::read_dir(&sent_path).into_iter().flatten().flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "txt" || e == "tsv") {
                    if let Ok(file) = File::open(&path) {
                        let reader = BufReader::new(file);
                        for line in reader.lines().flatten() {
                            let parts: Vec<&str> = line.split('\t').collect();
                            let sentence = if parts.len() >= 3 {
                                parts[2].trim().to_string()
                            } else {
                                line.trim().to_string()
                            };
                            if sentence.len() > 5 && sentence.len() < 500 {
                                let item = ProcessedItem {
                                    content: sentence,
                                    modality: "language".to_string(),
                                    language: lang.to_string(),
                                    source: "sentence".to_string(),
                                    features: ProcessedItem::extract_features(&line, "language"),
                                };
                                writeln!(writer, "{}", item.to_line())?;
                                lang_count += 1;
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
                        let reader = BufReader::new(file);
                        for line in reader.lines().flatten() {
                            let word = line.split_whitespace().next().unwrap_or("").trim();
                            if word.len() > 1 && !word.starts_with('#') {
                                let item = ProcessedItem {
                                    content: word.to_string(),
                                    modality: "language".to_string(),
                                    language: lang.to_string(),
                                    source: "vocabulary".to_string(),
                                    features: ProcessedItem::extract_features(word, "language"),
                                };
                                writeln!(writer, "{}", item.to_line())?;
                                lang_count += 1;
                            }
                        }
                    }
                }
            }
        }
        
        // 语法
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
                                let item = ProcessedItem {
                                    content: current_text.clone(),
                                    modality: "language".to_string(),
                                    language: lang.to_string(),
                                    source: "grammar".to_string(),
                                    features: ProcessedItem::extract_features(&current_text, "language"),
                                };
                                writeln!(writer, "{}", item.to_line())?;
                                lang_count += 1;
                            }
                            current_text.clear();
                        }
                    }
                }
            }
        }
        
        if lang_count > 0 {
            println!("   {} : {} 条", lang, lang_count);
        }
        total_count += lang_count;
    }
    
    // === 2. 对话模态 ===
    println!("\n💬 处理对话模态...");
    let mut dialogue_count = 0usize;
    
    // Cornell Movie Dialogs
    let cornell_path = corpus_root.join("dialogues/cornell movie-dialogs corpus/movie_lines.txt");
    if cornell_path.exists() {
        if let Ok(file) = File::open(&cornell_path) {
            let reader = BufReader::new(file);
            for line in reader.lines().flatten() {
                let parts: Vec<&str> = line.split(" +++$+++ ").collect();
                if parts.len() >= 5 {
                    let content = parts[4].trim();
                    if content.len() > 5 && content.len() < 500 {
                        let item = ProcessedItem {
                            content: content.to_string(),
                            modality: "dialogue".to_string(),
                            language: "english".to_string(),
                            source: "cornell_movie".to_string(),
                            features: ProcessedItem::extract_features(content, "dialogue"),
                        };
                        writeln!(writer, "{}", item.to_line())?;
                        dialogue_count += 1;
                    }
                }
            }
        }
    }
    println!("   cornell_movie : {} 条", dialogue_count);
    let cornell_count = dialogue_count;
    
    // ConvAI2
    let convai_path = corpus_root.join("dialogues/convai/train_both_original.txt");
    if convai_path.exists() {
        let mut convai_count = 0usize;
        if let Ok(file) = File::open(&convai_path) {
            let reader = BufReader::new(file);
            for line in reader.lines().flatten() {
                let content = line.trim();
                if content.len() > 10 && content.len() < 500 && !content.starts_with("your persona:") {
                    let item = ProcessedItem {
                        content: content.to_string(),
                        modality: "dialogue".to_string(),
                        language: "english".to_string(),
                        source: "convai".to_string(),
                        features: ProcessedItem::extract_features(content, "dialogue"),
                    };
                    writeln!(writer, "{}", item.to_line())?;
                    convai_count += 1;
                }
            }
        }
        println!("   convai : {} 条", convai_count);
        dialogue_count += convai_count;
    }
    
    // Empathetic Dialogues
    let empathetic_path = corpus_root.join("dialogues/empatheticdialogues/train.csv");
    if empathetic_path.exists() {
        let mut emp_count = 0usize;
        if let Ok(file) = File::open(&empathetic_path) {
            let reader = BufReader::new(file);
            for line in reader.lines().skip(1).flatten() {
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() >= 3 {
                    let content = parts[parts.len() - 1].trim_matches('"').trim();
                    if content.len() > 5 && content.len() < 500 {
                        let item = ProcessedItem {
                            content: content.to_string(),
                            modality: "dialogue".to_string(),
                            language: "english".to_string(),
                            source: "empathetic".to_string(),
                            features: ProcessedItem::extract_features(content, "dialogue"),
                        };
                        writeln!(writer, "{}", item.to_line())?;
                        emp_count += 1;
                    }
                }
            }
        }
        println!("   empathetic : {} 条", emp_count);
        dialogue_count += emp_count;
    }
    
    // Japanese dialogues
    let ja_path = corpus_root.join("dialogues/japanese/ja_dialog.jsonl");
    if ja_path.exists() {
        let mut ja_count = 0usize;
        if let Ok(file) = File::open(&ja_path) {
            let reader = BufReader::new(file);
            for line in reader.lines().flatten() {
                if let Some(start) = line.find("\"text\":") {
                    let rest = &line[start + 8..];
                    if let Some(end) = rest.find('"') {
                        let text = &rest[..end];
                        if text.len() > 5 && text.len() < 500 {
                            let item = ProcessedItem {
                                content: text.to_string(),
                                modality: "dialogue".to_string(),
                                language: "japanese".to_string(),
                                source: "ja_dialog".to_string(),
                                features: ProcessedItem::extract_features(text, "dialogue"),
                            };
                            writeln!(writer, "{}", item.to_line())?;
                            ja_count += 1;
                        }
                    }
                }
            }
        }
        println!("   ja_dialog : {} 条", ja_count);
        dialogue_count += ja_count;
    }
    
    total_count += dialogue_count;
    
    // === 3. 图片模态 ===
    println!("\n🖼️  处理图片模态...");
    let mut image_count = 0usize;
    
    // COCO Captions
    let coco_path = corpus_root.join("images/coco/annotations/captions_train2017.json");
    if coco_path.exists() {
        if let Ok(content) = fs::read_to_string(&coco_path) {
            let mut coco_count = 0usize;
            for line in content.lines() {
                if let Some(start) = line.find("\"caption\":") {
                    let rest = &line[start + 11..];
                    if let Some(end) = rest.find('"') {
                        let caption = &rest[..end];
                        if caption.len() > 10 && caption.len() < 500 {
                            let item = ProcessedItem {
                                content: caption.to_string(),
                                modality: "image".to_string(),
                                language: "english".to_string(),
                                source: "coco".to_string(),
                                features: ProcessedItem::extract_features(caption, "image"),
                            };
                            writeln!(writer, "{}", item.to_line())?;
                            coco_count += 1;
                        }
                    }
                }
            }
            println!("   coco : {} 条", coco_count);
            image_count += coco_count;
        }
    }
    
    // Flickr30k
    let flickr_dir = corpus_root.join("images/flickr30k/Sentences");
    if flickr_dir.exists() {
        let mut flickr_count = 0usize;
        for entry in fs::read_dir(&flickr_dir).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "txt") {
                if let Ok(file) = File::open(&path) {
                    let reader = BufReader::new(file);
                    for line in reader.lines().flatten() {
                        let content = line.trim();
                        if content.len() > 10 && content.len() < 500 && !content.starts_with('#') {
                            let item = ProcessedItem {
                                content: content.to_string(),
                                modality: "image".to_string(),
                                language: "english".to_string(),
                                source: "flickr30k".to_string(),
                                features: ProcessedItem::extract_features(content, "image"),
                            };
                            writeln!(writer, "{}", item.to_line())?;
                            flickr_count += 1;
                        }
                    }
                }
            }
        }
        println!("   flickr30k : {} 条", flickr_count);
        image_count += flickr_count;
    }
    
    // Conceptual Captions
    let cc_path = corpus_root.join("images/conceptual_captions/GCC-1.1.0-Validation.tsv");
    if cc_path.exists() {
        let mut cc_count = 0usize;
        if let Ok(file) = File::open(&cc_path) {
            let reader = BufReader::new(file);
            for line in reader.lines().flatten() {
                let parts: Vec<&str> = line.split('\t').collect();
                if !parts.is_empty() {
                    let caption = parts[0].trim();
                    if caption.len() > 10 && caption.len() < 500 {
                        let item = ProcessedItem {
                            content: caption.to_string(),
                            modality: "image".to_string(),
                            language: "english".to_string(),
                            source: "conceptual_captions".to_string(),
                            features: ProcessedItem::extract_features(caption, "image"),
                        };
                        writeln!(writer, "{}", item.to_line())?;
                        cc_count += 1;
                    }
                }
            }
        }
        println!("   conceptual_captions : {} 条", cc_count);
        image_count += cc_count;
    }
    
    total_count += image_count;
    
    // === 4. 叙事模态 ===
    println!("\n📖 处理叙事模态...");
    let mut narrative_count = 0usize;
    
    // Gutenberg
    let gutenberg_path = corpus_root.join("novels/gutenberg");
    if gutenberg_path.exists() {
        for entry in fs::read_dir(&gutenberg_path).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "txt") {
                if let Ok(file) = File::open(&path) {
                    let reader = BufReader::new(file);
                    let mut paragraph = String::new();
                    for line in reader.lines().flatten() {
                        if line.trim().is_empty() && !paragraph.is_empty() {
                            if paragraph.len() > 50 && paragraph.len() < 1000 {
                                let item = ProcessedItem {
                                    content: paragraph.clone(),
                                    modality: "narrative".to_string(),
                                    language: "english".to_string(),
                                    source: "gutenberg".to_string(),
                                    features: ProcessedItem::extract_features(&paragraph, "narrative"),
                                };
                                writeln!(writer, "{}", item.to_line())?;
                                narrative_count += 1;
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
    println!("   gutenberg : {} 条", narrative_count);
    
    // Chinese novels
    let chinese_novel_path = corpus_root.join("novels/chinese");
    if chinese_novel_path.exists() {
        let mut cn_count = 0usize;
        for entry in fs::read_dir(&chinese_novel_path).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "txt") {
                if let Ok(file) = File::open(&path) {
                    let reader = BufReader::new(file);
                    let mut paragraph = String::new();
                    for line in reader.lines().flatten() {
                        if line.trim().is_empty() && !paragraph.is_empty() {
                            if paragraph.len() > 30 && paragraph.len() < 800 {
                                let item = ProcessedItem {
                                    content: paragraph.clone(),
                                    modality: "narrative".to_string(),
                                    language: "chinese".to_string(),
                                    source: "chinese_novel".to_string(),
                                    features: ProcessedItem::extract_features(&paragraph, "narrative"),
                                };
                                writeln!(writer, "{}", item.to_line())?;
                                cn_count += 1;
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
        println!("   chinese_novel : {} 条", cn_count);
        narrative_count += cn_count;
    }
    
    total_count += narrative_count;
    
    writer.flush()?;
    
    // 获取文件大小
    let file_size = fs::metadata(&output_path)?.len();
    let size_mb = file_size as f64 / 1024.0 / 1024.0;
    
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║                    📦 预处理完成                                ║");
    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("║ 总数据: {} 条", total_count);
    println!("║ 文件大小: {:.2} MB", size_mb);
    println!("║ 输出路径: {}", output_path.display());
    println!("╚════════════════════════════════════════════════════════════════╝");
    
    Ok(())
}
