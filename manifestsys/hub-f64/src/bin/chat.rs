//! 🌌 Valhalla f64 Interactive Chat
//!
//! 让系统自主响应，基于多语言训练语料

use hub_f64::{
    ValhallaHub, HubRequest, RequestType, Priority,
};
use std::io::{self, Write, BufRead, BufReader};
use std::fs::{self, File};
use std::path::Path;

/// 语料库 - 系统的"记忆"
struct Corpus {
    sentences: Vec<(String, String)>,  // (内容, 语言)
    vocabulary: Vec<(String, String)>, // (词汇, 语言)
}

impl Corpus {
    fn load() -> Self {
        let base = Path::new("/run/media/jouly/My Book/valhalla_corpus/languages");
        let mut sentences = Vec::new();
        let mut vocabulary = Vec::new();
        
        let languages = ["chinese", "english", "japanese", "korean", "french", 
                         "german", "spanish", "russian", "arabic", "portuguese"];
        
        for lang in languages {
            let lang_dir = base.join(lang);
            
            // 加载词汇
            let vocab_dir = lang_dir.join("vocabulary");
            if vocab_dir.exists() {
                if let Ok(entries) = fs::read_dir(&vocab_dir) {
                    for entry in entries.flatten() {
                        if let Ok(file) = File::open(entry.path()) {
                            let reader = BufReader::new(file);
                            for line in reader.lines().take(5000) {
                                if let Ok(content) = line {
                                    let content = content.trim().to_string();
                                    if !content.is_empty() && content.len() > 1 && !content.starts_with('#') {
                                        vocabulary.push((content, lang.to_string()));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // 加载句子
            let sent_dir = lang_dir.join("sentences");
            if sent_dir.exists() {
                if let Ok(entries) = fs::read_dir(&sent_dir) {
                    for entry in entries.flatten() {
                        if let Ok(file) = File::open(entry.path()) {
                            let reader = BufReader::new(file);
                            for line in reader.lines().take(10000) {
                                if let Ok(content) = line {
                                    // Tatoeba 格式: ID\tLang\tSentence
                                    let parts: Vec<&str> = content.split('\t').collect();
                                    let sentence = if parts.len() >= 3 {
                                        parts[2].to_string()
                                    } else {
                                        content.trim().to_string()
                                    };
                                    
                                    if !sentence.is_empty() && sentence.len() > 5 {
                                        sentences.push((sentence, lang.to_string()));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Self { sentences, vocabulary }
    }
    
    /// 基于 Fate 偏好和输入，让系统自主选择响应
    fn respond(&self, input: &str, prefs: &[f64], signal_tick: u64) -> String {
        let seed = signal_tick as usize;
        
        // Fate 偏好决定响应风格
        let max_idx = prefs.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0);
        
        let input_len = input.len();
        let has_question = input.contains('?') || input.contains('？');
        
        // 检测输入语言（简单启发式）
        let is_chinese = input.chars().any(|c| c >= '\u{4e00}' && c <= '\u{9fff}');
        let is_japanese = input.chars().any(|c| (c >= '\u{3040}' && c <= '\u{309f}') || (c >= '\u{30a0}' && c <= '\u{30ff}'));
        
        // 选择响应来源
        let response = match max_idx {
            0 => {
                // BlackHole - 倾向深度/完整句子
                if !self.sentences.is_empty() {
                    let filtered: Vec<_> = if is_chinese {
                        self.sentences.iter().filter(|(_, l)| l == "chinese").collect()
                    } else if is_japanese {
                        self.sentences.iter().filter(|(_, l)| l == "japanese").collect()
                    } else {
                        self.sentences.iter().filter(|(_, l)| l == "english").collect()
                    };
                    
                    if !filtered.is_empty() {
                        let idx = (seed * 7 + input_len) % filtered.len();
                        filtered[idx].0.clone()
                    } else {
                        let idx = (seed * 7 + input_len) % self.sentences.len();
                        self.sentences[idx].0.clone()
                    }
                } else {
                    format!("[BlackHole 吸收信号 tick={}]", signal_tick)
                }
            }
            1 => {
                // MeshBrain - 倾向思考/问答
                if has_question && !self.sentences.is_empty() {
                    let idx = (seed * 11 + input_len * 2) % self.sentences.len();
                    self.sentences[idx].0.clone()
                } else if !self.vocabulary.is_empty() {
                    // 返回相关词汇
                    let idx = (seed * 3 + input_len) % self.vocabulary.len();
                    format!("💭 {}", self.vocabulary[idx].0)
                } else {
                    format!("[MeshBrain 处理中 tick={}]", signal_tick)
                }
            }
            2 => {
                // MultiNova - 倾向能量/变化
                if !self.sentences.is_empty() {
                    let idx = (seed * 13 + input_len * 3) % self.sentences.len();
                    self.sentences[idx].0.clone()
                } else {
                    format!("[MultiNova 演化中 tick={}]", signal_tick)
                }
            }
            _ => {
                // BaseForce - 倾向基础/词汇
                if !self.vocabulary.is_empty() {
                    let idx = (seed * 17 + input_len) % self.vocabulary.len();
                    self.vocabulary[idx].0.clone()
                } else if !self.sentences.is_empty() {
                    let idx = (seed * 5 + input_len) % self.sentences.len();
                    self.sentences[idx].0.clone()
                } else {
                    format!("[BaseForce 协调中 tick={}]", signal_tick)
                }
            }
        };
        
        response
    }
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║     🌌 Valhalla 多语言对话系统                                 ║");
    println!("║     基于训练语料的自主响应                                     ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();
    
    print!("加载多语言语料库...");
    io::stdout().flush().unwrap();
    let corpus = Corpus::load();
    println!(" ✅");
    println!("   句子: {} | 词汇: {}", corpus.sentences.len(), corpus.vocabulary.len());
    println!();
    println!("输入任何内容与 Valhalla 对话 (quit 退出)\n");
    
    let mut hub = ValhallaHub::new();
    let stdin = io::stdin();
    let mut counter = 0u64;
    
    loop {
        print!("你> ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        if stdin.lock().read_line(&mut input).is_err() {
            break;
        }
        
        let input = input.trim();
        if input.is_empty() {
            continue;
        }
        
        if input.to_lowercase() == "quit" || input.to_lowercase() == "exit" {
            println!("再见！👋");
            break;
        }
        
        counter += 1;
        let intensity = (input.len() as f64 / 50.0).min(1.0).max(0.1);
        
        let request = HubRequest {
            request_id: format!("chat_{}", counter),
            request_type: classify_input(input),
            intensity,
            data: Some(serde_json::json!({ "text": input })),
            priority: Priority::Normal,
        };
        
        let _response = hub.process(request);
        
        let prefs = hub.get_fate_preferences();
        let status = hub.get_status();
        
        let reply = corpus.respond(input, &prefs.affinities, status.fate_signal_tick);
        
        println!("\nValhalla> {}\n", reply);
    }
}

fn classify_input(input: &str) -> RequestType {
    let lower = input.to_lowercase();
    
    if lower.contains('?') || lower.contains('？') {
        RequestType::Reasoning
    } else if lower.contains("代码") || lower.contains("code") {
        RequestType::DataProcessing
    } else if lower.contains("学") || lower.contains("learn") {
        RequestType::Learning
    } else {
        RequestType::SystemCoordination
    }
}
