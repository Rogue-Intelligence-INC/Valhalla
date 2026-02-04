//! 🌌 Valhalla 语料压缩器
//!
//! 将原始语料压缩为 Valhalla 内部表示 (direction vectors)
//! 真正的压缩 - 不存文本，只存向量

use std::fs::{self, File};
use std::io::{self, Write, BufRead, BufReader, BufWriter};
use std::path::Path;

/// 压缩后的数据项 - 只有向量，没有原始文本
#[derive(Debug, Clone)]
struct CompressedItem {
    /// 模态标记 (1 byte: 0=lang, 1=dialogue, 2=image, 3=narrative)
    modality: u8,
    /// 语言标记 (1 byte: 0=en, 1=zh, 2=ja, 3=ko, 4=fr, 5=de, 6=es, 7=ru, 8=ar, 9=pt)
    language: u8,
    /// 压缩后的 direction 向量 (4维 f32 = 16 bytes)
    direction: [f32; 4],
    /// 强度 (f32 = 4 bytes)
    intensity: f32,
    /// 质量分 (f32 = 4 bytes)
    quality: f32,
}

// 每条数据 = 1 + 1 + 16 + 4 + 4 = 26 bytes
// 对比原始: 平均 100 字符 = 100 bytes
// 压缩率: ~74%

impl CompressedItem {
    fn from_text(content: &str, modality: u8, language: u8) -> Self {
        let chars: Vec<char> = content.chars().collect();
        let len = content.len() as f32;
        let char_count = chars.len() as f32;
        
        // 计算 direction 向量 (4维压缩表示)
        let alpha_ratio = chars.iter().filter(|c| c.is_alphabetic()).count() as f32 / char_count.max(1.0);
        let digit_ratio = chars.iter().filter(|c| c.is_numeric()).count() as f32 / char_count.max(1.0);
        let space_ratio = chars.iter().filter(|c| c.is_whitespace()).count() as f32 / char_count.max(1.0);
        let punct_ratio = chars.iter().filter(|c| c.is_ascii_punctuation()).count() as f32 / char_count.max(1.0);
        
        // 计算熵 (内容复杂度)
        let unique_chars = {
            let mut seen = std::collections::HashSet::new();
            for c in chars.iter() { seen.insert(*c); }
            seen.len() as f32
        };
        let entropy = if char_count > 0.0 { unique_chars / char_count.sqrt() } else { 0.0 };
        
        // direction 向量: 压缩内容特征为 4 维
        let direction = [
            alpha_ratio - digit_ratio,  // 文本性 vs 数字性
            space_ratio - punct_ratio,  // 流畅性 vs 结构性
            (len / 500.0).min(1.0),     // 长度归一化
            entropy.min(1.0),           // 复杂度
        ];
        
        // intensity: 基于长度和模态
        let modality_weight = match modality {
            0 => 0.3,  // language
            1 => 0.5,  // dialogue (对话权重高)
            2 => 0.4,  // image
            3 => 0.35, // narrative
            _ => 0.25,
        };
        let intensity = ((len / 100.0).min(1.0) * 0.5 + modality_weight * 0.5) as f32;
        
        // quality: 基于熵和长度
        let quality = ((entropy * 0.5 + (len / 200.0).min(0.5)) as f32).min(1.0);
        
        Self {
            modality,
            language,
            direction,
            intensity,
            quality,
        }
    }
    
    /// 序列化为 26 字节
    fn to_bytes(&self) -> [u8; 26] {
        let mut bytes = [0u8; 26];
        bytes[0] = self.modality;
        bytes[1] = self.language;
        
        // direction (4 x f32 = 16 bytes)
        for (i, &v) in self.direction.iter().enumerate() {
            let vb = v.to_le_bytes();
            bytes[2 + i*4..6 + i*4].copy_from_slice(&vb);
        }
        
        // intensity (f32 = 4 bytes)
        let ib = self.intensity.to_le_bytes();
        bytes[18..22].copy_from_slice(&ib);
        
        // quality (f32 = 4 bytes)
        let qb = self.quality.to_le_bytes();
        bytes[22..26].copy_from_slice(&qb);
        
        bytes
    }
    
    /// 从 26 字节反序列化
    fn from_bytes(bytes: &[u8; 26]) -> Self {
        let modality = bytes[0];
        let language = bytes[1];
        
        let mut direction = [0.0f32; 4];
        for i in 0..4 {
            let mut vb = [0u8; 4];
            vb.copy_from_slice(&bytes[2 + i*4..6 + i*4]);
            direction[i] = f32::from_le_bytes(vb);
        }
        
        let mut ib = [0u8; 4];
        ib.copy_from_slice(&bytes[18..22]);
        let intensity = f32::from_le_bytes(ib);
        
        let mut qb = [0u8; 4];
        qb.copy_from_slice(&bytes[22..26]);
        let quality = f32::from_le_bytes(qb);
        
        Self { modality, language, direction, intensity, quality }
    }
}

fn language_to_id(lang: &str) -> u8 {
    match lang {
        "english" => 0,
        "chinese" => 1,
        "japanese" => 2,
        "korean" => 3,
        "french" => 4,
        "german" => 5,
        "spanish" => 6,
        "russian" => 7,
        "arabic" => 8,
        "portuguese" => 9,
        _ => 0,
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
    println!("║     🌌 Valhalla 语料压缩器                                     ║");
    println!("║     真正的压缩 - 文本 → direction 向量                         ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    
    let corpus_root = Path::new("/run/media/jouly/My Book/valhalla_corpus");
    let output_path = corpus_root.join("valhalla_compressed.bin");
    
    println!("📂 输入: {}", corpus_root.display());
    println!("📦 输出: {}\n", output_path.display());
    println!("📐 每条数据: 26 字节 (原始平均 ~100 字节, 压缩率 ~74%)\n");
    
    let output_file = File::create(&output_path)?;
    let mut writer = BufWriter::new(output_file);
    
    // 写入魔数和版本
    writer.write_all(b"VALHALLA_COMPRESSED_V1\0\0")?; // 24 bytes header
    
    let mut total_count = 0u64;
    let mut items_buffer: Vec<CompressedItem> = Vec::with_capacity(100000);
    
    // === 1. 语言模态 ===
    println!("🗣️  压缩语言模态...");
    let lang_path = corpus_root.join("languages");
    let languages = ["english", "chinese", "japanese", "korean", "french", 
                     "german", "spanish", "russian", "arabic", "portuguese"];
    
    for lang in &languages {
        let lang_id = language_to_id(lang);
        let lang_dir = lang_path.join(lang);
        let mut lang_count = 0u64;
        
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
                                parts[2].trim()
                            } else {
                                line.trim()
                            };
                            if sentence.len() > 5 && sentence.len() < 500 {
                                let item = CompressedItem::from_text(sentence, 0, lang_id);
                                writer.write_all(&item.to_bytes())?;
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
                                let item = CompressedItem::from_text(word, 0, lang_id);
                                writer.write_all(&item.to_bytes())?;
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
                                let item = CompressedItem::from_text(&current_text, 0, lang_id);
                                writer.write_all(&item.to_bytes())?;
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
    println!("\n💬 压缩对话模态...");
    
    // Cornell Movie Dialogs
    let cornell_path = corpus_root.join("dialogues/cornell movie-dialogs corpus/movie_lines.txt");
    if cornell_path.exists() {
        let mut count = 0u64;
        if let Ok(file) = File::open(&cornell_path) {
            let reader = BufReader::new(file);
            for line in reader.lines().flatten() {
                let parts: Vec<&str> = line.split(" +++$+++ ").collect();
                if parts.len() >= 5 {
                    let content = parts[4].trim();
                    if content.len() > 5 && content.len() < 500 {
                        let item = CompressedItem::from_text(content, 1, 0); // dialogue, english
                        writer.write_all(&item.to_bytes())?;
                        count += 1;
                    }
                }
            }
        }
        println!("   cornell : {} 条", count);
        total_count += count;
    }
    
    // ConvAI2
    let convai_path = corpus_root.join("dialogues/convai/train_both_original.txt");
    if convai_path.exists() {
        let mut count = 0u64;
        if let Ok(file) = File::open(&convai_path) {
            let reader = BufReader::new(file);
            for line in reader.lines().flatten() {
                let content = line.trim();
                if content.len() > 10 && content.len() < 500 && !content.starts_with("your persona:") {
                    let item = CompressedItem::from_text(content, 1, 0);
                    writer.write_all(&item.to_bytes())?;
                    count += 1;
                }
            }
        }
        println!("   convai : {} 条", count);
        total_count += count;
    }
    
    // === 3. 图片模态 ===
    println!("\n🖼️  压缩图片模态...");
    
    // Flickr30k
    let flickr_dir = corpus_root.join("images/flickr30k/Sentences");
    if flickr_dir.exists() {
        let mut count = 0u64;
        for entry in fs::read_dir(&flickr_dir).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "txt") {
                if let Ok(file) = File::open(&path) {
                    let reader = BufReader::new(file);
                    for line in reader.lines().flatten() {
                        let content = line.trim();
                        if content.len() > 10 && content.len() < 500 && !content.starts_with('#') {
                            let item = CompressedItem::from_text(content, 2, 0); // image, english
                            writer.write_all(&item.to_bytes())?;
                            count += 1;
                        }
                    }
                }
            }
        }
        println!("   flickr30k : {} 条", count);
        total_count += count;
    }
    
    // Conceptual Captions
    let cc_path = corpus_root.join("images/conceptual_captions/GCC-1.1.0-Validation.tsv");
    if cc_path.exists() {
        let mut count = 0u64;
        if let Ok(file) = File::open(&cc_path) {
            let reader = BufReader::new(file);
            for line in reader.lines().flatten() {
                let parts: Vec<&str> = line.split('\t').collect();
                if !parts.is_empty() {
                    let caption = parts[0].trim();
                    if caption.len() > 10 && caption.len() < 500 {
                        let item = CompressedItem::from_text(caption, 2, 0);
                        writer.write_all(&item.to_bytes())?;
                        count += 1;
                    }
                }
            }
        }
        println!("   conceptual_captions : {} 条", count);
        total_count += count;
    }
    
    // === 4. 叙事模态 ===
    println!("\n📖 压缩叙事模态...");
    
    let gutenberg_path = corpus_root.join("novels/gutenberg");
    if gutenberg_path.exists() {
        let mut count = 0u64;
        for entry in fs::read_dir(&gutenberg_path).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "txt") {
                if let Ok(file) = File::open(&path) {
                    let reader = BufReader::new(file);
                    let mut paragraph = String::new();
                    for line in reader.lines().flatten() {
                        if line.trim().is_empty() && !paragraph.is_empty() {
                            if paragraph.len() > 50 && paragraph.len() < 1000 {
                                let item = CompressedItem::from_text(&paragraph, 3, 0); // narrative, english
                                writer.write_all(&item.to_bytes())?;
                                count += 1;
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
        println!("   gutenberg : {} 条", count);
        total_count += count;
    }
    
    writer.flush()?;
    
    // 获取文件大小
    let file_size = fs::metadata(&output_path)?.len();
    let size_mb = file_size as f64 / 1024.0 / 1024.0;
    let raw_size_estimate = total_count as f64 * 100.0 / 1024.0 / 1024.0; // 假设原始平均 100 bytes
    let compression_ratio = (1.0 - size_mb / raw_size_estimate) * 100.0;
    
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║                    📦 压缩完成                                  ║");
    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("║ 总数据: {} 条", total_count);
    println!("║ 压缩后: {:.2} MB", size_mb);
    println!("║ 原始估计: {:.2} MB", raw_size_estimate);
    println!("║ 压缩率: {:.1}%", compression_ratio);
    println!("║ 每条: 26 bytes (modality + language + direction[4] + intensity + quality)");
    println!("║ 输出: {}", output_path.display());
    println!("╚════════════════════════════════════════════════════════════════╝");
    
    Ok(())
}
