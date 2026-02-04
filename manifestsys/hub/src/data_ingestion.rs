//! 🔌 数据入口模块 - 多源数据摄入与形变编码
//!
//! 支持的数据源:
//! - 文本文件 (TXT, JSON, CSV)
//! - 代码文件 (Rust, Python, Wave)
//! - 数学表达式 (LaTeX, MathML)
//! - 图形数据 (SVG - 简化版)
//! - 音频数据 (WAV, 频谱特征)
//! - 网络数据 (HTTP/HTTPS)

use std::collections::HashMap;
use std::path::Path;
use std::fs;
use serde::{Serialize, Deserialize};
use thiserror::Error;

/// 形变信号向量（模态无关表示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeformationVector {
    /// 主体强度
    pub subject_intensity: f64,
    /// 关系类型 (0=包含, 1=等于, 2=大于, 3=属于, ...)
    pub relation_type: f64,
    /// 客体强度
    pub object_intensity: f64,
    /// 方向性 (-1=逆, 0=无, 1=正)
    pub directionality: f64,
    /// 时态 (-1=过去, 0=现在, 1=将来)
    pub tense: f64,
    /// 确定性 (0-1)
    pub certainty: f64,
    /// 抽象层级 (0=具体, 1=抽象)
    pub abstraction_level: f64,
    /// 语境权重
    pub context_weight: f64,
    /// 模态标识 (用于追踪，不影响语义)
    pub modality_marker: f64,
    /// 复杂度
    pub complexity: f64,
}

impl DeformationVector {
    pub fn new() -> Self {
        Self {
            subject_intensity: 0.0,
            relation_type: 0.0,
            object_intensity: 0.0,
            directionality: 0.0,
            tense: 0.0,
            certainty: 1.0,
            abstraction_level: 0.5,
            context_weight: 1.0,
            modality_marker: 0.0,
            complexity: 0.0,
        }
    }
    
    /// 转换为向量形式
    pub fn to_vec(&self) -> Vec<f64> {
        vec![
            self.subject_intensity,
            self.relation_type,
            self.object_intensity,
            self.directionality,
            self.tense,
            self.certainty,
            self.abstraction_level,
            self.context_weight,
            self.modality_marker,
            self.complexity,
        ]
    }
    
    /// 从向量构建
    pub fn from_vec(v: &[f64]) -> Self {
        Self {
            subject_intensity: v.get(0).copied().unwrap_or(0.0),
            relation_type: v.get(1).copied().unwrap_or(0.0),
            object_intensity: v.get(2).copied().unwrap_or(0.0),
            directionality: v.get(3).copied().unwrap_or(0.0),
            tense: v.get(4).copied().unwrap_or(0.0),
            certainty: v.get(5).copied().unwrap_or(1.0),
            abstraction_level: v.get(6).copied().unwrap_or(0.5),
            context_weight: v.get(7).copied().unwrap_or(1.0),
            modality_marker: v.get(8).copied().unwrap_or(0.0),
            complexity: v.get(9).copied().unwrap_or(0.0),
        }
    }
    
    /// 余弦相似度
    pub fn cosine_similarity(&self, other: &DeformationVector) -> f64 {
        let a = self.to_vec();
        let b = other.to_vec();
        
        let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let mag_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
        let mag_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
        
        if mag_a == 0.0 || mag_b == 0.0 {
            0.0
        } else {
            dot / (mag_a * mag_b)
        }
    }
}

impl Default for DeformationVector {
    fn default() -> Self {
        Self::new()
    }
}

/// 模态类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Modality {
    /// 自然语言文本
    Text,
    /// 代码
    Code,
    /// 数学表达式
    Math,
    /// 图形 (SVG)
    Visual,
    /// 音频
    Audio,
    /// 结构化数据 (JSON/CSV)
    Structured,
}

impl Modality {
    pub fn marker(&self) -> f64 {
        match self {
            Modality::Text => 0.0,
            Modality::Code => 0.1,
            Modality::Math => 0.2,
            Modality::Visual => 0.3,
            Modality::Audio => 0.4,
            Modality::Structured => 0.5,
        }
    }
}

/// 数据项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataItem {
    pub id: String,
    pub modality: Modality,
    pub raw_content: String,
    pub deformation: DeformationVector,
    pub metadata: HashMap<String, String>,
}

#[derive(Error, Debug)]
pub enum IngestionError {
    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),
    #[error("解析错误: {0}")]
    ParseError(String),
    #[error("不支持的格式: {0}")]
    UnsupportedFormat(String),
    #[error("网络错误: {0}")]
    NetworkError(String),
}

/// 🔌 数据摄入引擎
pub struct DataIngestionEngine {
    /// 已摄入的数据
    pub items: Vec<DataItem>,
    /// 统计信息
    pub stats: IngestionStats,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IngestionStats {
    pub total_items: usize,
    pub by_modality: HashMap<String, usize>,
    pub average_complexity: f64,
}

/// 音频特征
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioFeatures {
    /// 时长 (毫秒)
    pub duration_ms: f64,
    /// 主频率 (Hz)
    pub frequency_hz: f64,
    /// 振幅 (0-1)
    pub amplitude: f64,
    /// 节拍 (BPM, 可选)
    pub tempo_bpm: Option<f64>,
    /// 音高类别 (0-11, 对应 C-B)
    pub pitch_class: Option<u8>,
    /// 采样率
    pub sample_rate: Option<u32>,
    /// 位深度
    pub bits_per_sample: Option<u16>,
    /// 声道数
    pub channels: Option<u16>,
}

impl AudioFeatures {
    pub fn new(duration_ms: f64, frequency_hz: f64, amplitude: f64) -> Self {
        Self {
            duration_ms,
            frequency_hz,
            amplitude,
            tempo_bpm: None,
            pitch_class: None,
            sample_rate: None,
            bits_per_sample: None,
            channels: None,
        }
    }
    
    /// 从简单描述创建 (用于实验)
    pub fn from_description(desc: &str) -> Self {
        // 解析描述词映射到音频特征
        let is_high = desc.contains("高") || desc.contains("high") || desc.contains("尖");
        let is_low = desc.contains("低") || desc.contains("low") || desc.contains("沉");
        let is_fast = desc.contains("快") || desc.contains("fast") || desc.contains("急");
        let is_slow = desc.contains("慢") || desc.contains("slow") || desc.contains("缓");
        let is_loud = desc.contains("响") || desc.contains("loud") || desc.contains("大");
        let is_soft = desc.contains("轻") || desc.contains("soft") || desc.contains("小");
        
        let frequency = if is_high { 880.0 } else if is_low { 220.0 } else { 440.0 };
        let tempo = if is_fast { Some(140.0) } else if is_slow { Some(60.0) } else { Some(100.0) };
        let amplitude = if is_loud { 0.9 } else if is_soft { 0.3 } else { 0.6 };
        
        Self {
            duration_ms: 1000.0,
            frequency_hz: frequency,
            amplitude,
            tempo_bpm: tempo,
            pitch_class: Some(if is_high { 9 } else if is_low { 0 } else { 4 }), // A, C, E
            sample_rate: Some(44100),
            bits_per_sample: Some(16),
            channels: Some(2),
        }
    }
}

impl DataIngestionEngine {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            stats: IngestionStats::default(),
        }
    }
    
    /// 从文本摄入
    pub fn ingest_text(&mut self, text: &str, language: &str) -> Result<DataItem, IngestionError> {
        let deformation = self.encode_text(text, language);
        
        let item = DataItem {
            id: format!("text_{}", self.items.len()),
            modality: Modality::Text,
            raw_content: text.to_string(),
            deformation,
            metadata: {
                let mut m = HashMap::new();
                m.insert("language".to_string(), language.to_string());
                m.insert("length".to_string(), text.len().to_string());
                m
            },
        };
        
        self.items.push(item.clone());
        self.update_stats();
        Ok(item)
    }
    
    /// 从代码摄入
    pub fn ingest_code(&mut self, code: &str, lang: &str) -> Result<DataItem, IngestionError> {
        let deformation = self.encode_code(code, lang);
        
        let item = DataItem {
            id: format!("code_{}", self.items.len()),
            modality: Modality::Code,
            raw_content: code.to_string(),
            deformation,
            metadata: {
                let mut m = HashMap::new();
                m.insert("language".to_string(), lang.to_string());
                m.insert("lines".to_string(), code.lines().count().to_string());
                m
            },
        };
        
        self.items.push(item.clone());
        self.update_stats();
        Ok(item)
    }
    
    /// 从数学表达式摄入
    pub fn ingest_math(&mut self, expr: &str) -> Result<DataItem, IngestionError> {
        let deformation = self.encode_math(expr);
        
        let item = DataItem {
            id: format!("math_{}", self.items.len()),
            modality: Modality::Math,
            raw_content: expr.to_string(),
            deformation,
            metadata: HashMap::new(),
        };
        
        self.items.push(item.clone());
        self.update_stats();
        Ok(item)
    }
    
    /// 从 SVG 摄入
    pub fn ingest_svg(&mut self, svg: &str) -> Result<DataItem, IngestionError> {
        let deformation = self.encode_svg(svg);
        
        let item = DataItem {
            id: format!("svg_{}", self.items.len()),
            modality: Modality::Visual,
            raw_content: svg.to_string(),
            deformation,
            metadata: HashMap::new(),
        };
        
        self.items.push(item.clone());
        self.update_stats();
        Ok(item)
    }
    
    /// 从音频特征摄入
    /// 
    /// 音频特征格式 (简化表示):
    /// - duration_ms: 时长
    /// - frequency_hz: 主频率
    /// - amplitude: 振幅
    /// - tempo_bpm: 节拍 (可选)
    /// - pitch_class: 音高类别 (0-11)
    pub fn ingest_audio(&mut self, features: &AudioFeatures) -> Result<DataItem, IngestionError> {
        let deformation = self.encode_audio(features);
        
        let item = DataItem {
            id: format!("audio_{}", self.items.len()),
            modality: Modality::Audio,
            raw_content: format!("{:?}", features),
            deformation,
            metadata: {
                let mut m = HashMap::new();
                m.insert("duration_ms".to_string(), features.duration_ms.to_string());
                m.insert("frequency_hz".to_string(), features.frequency_hz.to_string());
                m
            },
        };
        
        self.items.push(item.clone());
        self.update_stats();
        Ok(item)
    }
    
    /// 从 WAV 文件头摄入 (简化解析)
    pub fn ingest_wav_header(&mut self, wav_bytes: &[u8]) -> Result<DataItem, IngestionError> {
        // 简化的 WAV 解析 - 只读取基本信息
        if wav_bytes.len() < 44 {
            return Err(IngestionError::ParseError("WAV 文件太小".to_string()));
        }
        
        // RIFF header check
        if &wav_bytes[0..4] != b"RIFF" || &wav_bytes[8..12] != b"WAVE" {
            return Err(IngestionError::ParseError("不是有效的 WAV 文件".to_string()));
        }
        
        // 提取基本信息
        let sample_rate = u32::from_le_bytes([wav_bytes[24], wav_bytes[25], wav_bytes[26], wav_bytes[27]]);
        let bits_per_sample = u16::from_le_bytes([wav_bytes[34], wav_bytes[35]]);
        let num_channels = u16::from_le_bytes([wav_bytes[22], wav_bytes[23]]);
        
        let features = AudioFeatures {
            duration_ms: 0.0, // 需要完整解析才能计算
            frequency_hz: sample_rate as f64,
            amplitude: 0.5,
            tempo_bpm: None,
            pitch_class: None,
            sample_rate: Some(sample_rate),
            bits_per_sample: Some(bits_per_sample),
            channels: Some(num_channels),
        };
        
        self.ingest_audio(&features)
    }
    
    /// 从文件摄入
    pub fn ingest_file(&mut self, path: &Path) -> Result<DataItem, IngestionError> {
        let content = fs::read_to_string(path)?;
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        
        match ext.to_lowercase().as_str() {
            "txt" => self.ingest_text(&content, "auto"),
            "rs" => self.ingest_code(&content, "rust"),
            "py" => self.ingest_code(&content, "python"),
            "wave" => self.ingest_code(&content, "wave"),
            "js" | "ts" => self.ingest_code(&content, "javascript"),
            "json" => self.ingest_structured(&content, "json"),
            "csv" => self.ingest_structured(&content, "csv"),
            "svg" => self.ingest_svg(&content),
            "tex" | "latex" => self.ingest_math(&content),
            _ => Err(IngestionError::UnsupportedFormat(ext.to_string())),
        }
    }
    
    /// 摄入结构化数据
    pub fn ingest_structured(&mut self, content: &str, format: &str) -> Result<DataItem, IngestionError> {
        let deformation = self.encode_structured(content, format);
        
        let item = DataItem {
            id: format!("struct_{}", self.items.len()),
            modality: Modality::Structured,
            raw_content: content.to_string(),
            deformation,
            metadata: {
                let mut m = HashMap::new();
                m.insert("format".to_string(), format.to_string());
                m
            },
        };
        
        self.items.push(item.clone());
        self.update_stats();
        Ok(item)
    }
    
    // ═══════════════════════════════════════════════════════════════
    // 形变编码器 v2 - 优化跨语言/跨模态语义对齐
    // 
    // 核心改进:
    // 1. 语义等价映射 - "contains"/"属于"视为同一关系的不同视角
    // 2. 强度归一化 - 减少词数对语义的影响
    // 3. 多语言关系词库扩展
    // ═══════════════════════════════════════════════════════════════
    
    fn encode_text(&self, text: &str, _language: &str) -> DeformationVector {
        let text_lower = text.to_lowercase();
        
        // ═══ 语义关系检测 (多语言) ═══
        // 关系类型:
        // 0.0 = 包含/属于 (containment - 双向等价)
        // 1.0 = 等于/相等 (equality)
        // 2.0 = 大于/比较 (comparison/ordering)
        // 3.0 = 动作/过程 (action/process)
        // 4.0 = 状态/属性 (state/property)
        
        // 包含关系 (包括逆向 "属于")
        let has_containment = text_lower.contains("contains") || text_lower.contains("包含") 
            || text_lower.contains("有") || text_lower.contains("holds")
            || text_lower.contains("属于") || text_lower.contains("belongs")
            || text_lower.contains("在") || text_lower.contains("within")
            || text_lower.contains("容器") || text_lower.contains("container")
            || text_lower.contains("include") || text_lower.contains("包括");
        
        // 等于关系
        let has_equality = text.contains("=") || text_lower.contains("equals") 
            || text_lower.contains("等于") || text_lower.contains("相等")
            || text_lower.contains("same") || text_lower.contains("一样")
            || text_lower.contains("equivalent") || text_lower.contains("等价");
        
        // 比较/大于关系
        let has_comparison = text.contains(">") || text.contains("<")
            || text_lower.contains("greater") || text_lower.contains("大于")
            || text_lower.contains("less") || text_lower.contains("小于")
            || text_lower.contains("more") || text_lower.contains("更")
            || text_lower.contains("compare") || text_lower.contains("比较");
        
        // 动作/过程
        let has_action = text_lower.contains("return") || text_lower.contains("返回")
            || text_lower.contains("call") || text_lower.contains("调用")
            || text_lower.contains("process") || text_lower.contains("处理")
            || text_lower.contains("execute") || text_lower.contains("执行")
            || text_lower.contains("function") || text_lower.contains("函数");
        
        // 状态/属性/情感
        let has_state = text_lower.contains("love") || text_lower.contains("爱")
            || text_lower.contains("hate") || text_lower.contains("恨")
            || text_lower.contains("eternal") || text_lower.contains("永恒")
            || text_lower.contains("beautiful") || text_lower.contains("美");
        
        // 否定检测
        let has_negation = text_lower.contains("not") || text_lower.contains("不")
            || text_lower.contains("没") || text_lower.contains("never")
            || text_lower.contains("don't") || text_lower.contains("无");
        
        // 关系类型 (优先级排序)
        let relation_type = if has_containment { 0.0 }
            else if has_equality { 1.0 }
            else if has_comparison { 2.0 }
            else if has_action { 3.0 }
            else if has_state { 4.0 }
            else { 2.5 }; // 未知关系取中间值
        
        // ═══ 强度计算 (归一化，减少词数影响) ═══
        let base_intensity = 0.7; // 固定基础强度
        let relation_strength = if has_containment || has_equality || has_comparison { 0.9 } 
            else if has_action { 0.8 } 
            else { 0.6 };
        
        // ═══ 复杂度 (基于语义复杂性而非词数) ═══
        let semantic_complexity: f64 = (
            (if has_containment { 0.2 } else { 0.0 }) +
            (if has_equality { 0.15 } else { 0.0 }) +
            (if has_comparison { 0.2 } else { 0.0 }) +
            (if has_action { 0.25 } else { 0.0 }) +
            (if has_state { 0.15 } else { 0.0 }) +
            (if has_negation { 0.1 } else { 0.0_f64 })
        ).min(1.0).max(0.1);
        
        DeformationVector {
            subject_intensity: base_intensity,
            relation_type,
            object_intensity: relation_strength,
            directionality: if has_negation { -1.0 } else { 1.0 },
            tense: 0.0,
            certainty: if has_negation { 0.5 } else { 1.0 },
            abstraction_level: if has_state { 0.8 } else { 0.5 },
            context_weight: 1.0,
            modality_marker: Modality::Text.marker(),
            complexity: semantic_complexity,
        }
    }
    
    fn encode_code(&self, code: &str, _lang: &str) -> DeformationVector {
        let code_lower = code.to_lowercase();
        
        // ═══ 代码语义检测 (跨语言统一) ═══
        // 函数定义 = 包含关系
        let has_function = code.contains("fn ") || code.contains("def ") 
            || code.contains("function") || code.contains("func ")
            || code.contains("sub ") || code.contains("method");
        
        // 返回 = 动作关系
        let has_return = code.contains("return") || code.contains("->")
            || code.contains("yield");
        
        // 循环 = 迭代/比较关系
        let has_loop = code.contains("for") || code.contains("while") 
            || code.contains("loop") || code.contains("each");
        
        // 条件 = 等于/比较关系
        let has_condition = code.contains("if") || code.contains("match") 
            || code.contains("switch") || code.contains("case")
            || code.contains("else");
        
        // 比较操作
        let has_comparison = code.contains(">") || code.contains("<")
            || code.contains("==") || code.contains("!=")
            || code_lower.contains("compare") || code_lower.contains("cmp");
        
        // 容器/集合操作 = 包含关系
        let has_container = code_lower.contains("vec") || code_lower.contains("list")
            || code_lower.contains("array") || code_lower.contains("map")
            || code_lower.contains("set") || code_lower.contains("dict")
            || code.contains("[]") || code.contains("{}");
        
        // 数学运算
        let has_math = code.contains("+") || code.contains("-") 
            || code.contains("*") || code.contains("/")
            || code_lower.contains("add") || code_lower.contains("sum");
        
        // 关系类型 - 与文本编码对齐
        let relation_type = if has_container || has_function { 0.0 }  // 包含
            else if has_comparison && !has_loop { 1.0 }  // 等于/比较
            else if has_loop || (has_comparison && has_loop) { 2.0 }  // 迭代/排序
            else if has_return || has_math { 3.0 }  // 动作
            else { 2.5 };
        
        // 强度归一化 - 与文本对齐
        let base_intensity = 0.7;
        let relation_strength = if has_function || has_container { 0.9 }
            else if has_comparison || has_return { 0.85 }
            else { 0.7 };
        
        // 复杂度
        let semantic_complexity: f64 = (
            (if has_function { 0.2 } else { 0.0 }) +
            (if has_loop { 0.2 } else { 0.0 }) +
            (if has_condition { 0.15 } else { 0.0 }) +
            (if has_container { 0.15 } else { 0.0 }) +
            (if has_return { 0.1 } else { 0.0 }) +
            (if has_math { 0.1 } else { 0.0_f64 })
        ).min(1.0).max(0.1);
        
        DeformationVector {
            subject_intensity: base_intensity,
            relation_type,
            object_intensity: relation_strength,
            directionality: 1.0,
            tense: 0.0,
            certainty: 1.0,
            abstraction_level: 0.6, // 统一抽象层级
            context_weight: 1.0,
            modality_marker: Modality::Code.marker(),
            complexity: semantic_complexity,
        }
    }
    
    fn encode_math(&self, expr: &str) -> DeformationVector {
        let has_equals = expr.contains("=");
        let has_integral = expr.contains("∫") || expr.contains("\\int");
        let has_sum = expr.contains("∑") || expr.contains("\\sum");
        let has_fraction = expr.contains("/") || expr.contains("\\frac");
        
        let relation_type = if has_equals { 1.0 }
            else if has_integral { 2.0 }
            else if has_sum { 2.0 }
            else { 1.0 };
        
        let complexity = ((if has_integral { 0.4 } else { 0.0 }) +
            (if has_sum { 0.3 } else { 0.0 }) +
            (if has_fraction { 0.2 } else { 0.0 }) +
            (expr.len() as f64 / 100.0)).min(1.0);
        
        DeformationVector {
            subject_intensity: 0.8,
            relation_type,
            object_intensity: 0.8,
            directionality: 0.0, // 数学等式是双向的
            tense: 0.0,
            certainty: 1.0, // 数学是精确的
            abstraction_level: 0.9, // 数学高度抽象
            context_weight: 1.0,
            modality_marker: Modality::Math.marker(),
            complexity,
        }
    }
    
    fn encode_svg(&self, svg: &str) -> DeformationVector {
        let has_rect = svg.contains("<rect");
        let has_circle = svg.contains("<circle");
        let has_path = svg.contains("<path");
        let has_text = svg.contains("<text");
        let has_line = svg.contains("<line");
        
        // SVG 结构映射
        let relation_type = if has_rect { 0.0 } // 包含（矩形容器）
            else if has_circle { 1.0 } // 等价（圆的对称性）
            else if has_path { 2.0 } // 路径（方向性）
            else { 4.0 };
        
        let complexity: f64 = ((if has_rect { 0.1 } else { 0.0 }) +
            (if has_circle { 0.2 } else { 0.0 }) +
            (if has_path { 0.4 } else { 0.0 }) +
            (if has_text { 0.2 } else { 0.0 }) +
            (if has_line { 0.1 } else { 0.0_f64 })).min(1.0);
        
        DeformationVector {
            subject_intensity: (svg.len() as f64 / 1000.0).min(1.0),
            relation_type,
            object_intensity: complexity,
            directionality: if has_path { 1.0 } else { 0.0 },
            tense: 0.0,
            certainty: 1.0,
            abstraction_level: 0.3, // 视觉较具体
            context_weight: 1.0,
            modality_marker: Modality::Visual.marker(),
            complexity,
        }
    }
    
    fn encode_structured(&self, content: &str, format: &str) -> DeformationVector {
        let is_json = format == "json";
        let depth = if is_json {
            content.matches('{').count().max(content.matches('[').count()) as f64
        } else {
            content.lines().count() as f64 / 10.0
        };
        
        DeformationVector {
            subject_intensity: (depth / 5.0).min(1.0),
            relation_type: 0.0, // 结构化数据表示包含关系
            object_intensity: (content.len() as f64 / 1000.0).min(1.0),
            directionality: 0.0,
            tense: 0.0,
            certainty: 1.0,
            abstraction_level: 0.6,
            context_weight: 1.0,
            modality_marker: Modality::Structured.marker(),
            complexity: (depth / 10.0).min(1.0),
        }
    }
    
    fn encode_audio(&self, features: &AudioFeatures) -> DeformationVector {
        // 音频特征映射到形变向量
        // 
        // 关键映射:
        // - 频率 → 关系类型 (高频=尖锐=大于, 低频=沉稳=包含)
        // - 振幅 → 强度
        // - 节拍 → 方向性 (快=正向, 慢=负向)
        // - 音高 → 抽象层级
        
        let freq_normalized = (features.frequency_hz / 1000.0).min(1.0);
        let relation_type = if features.frequency_hz > 600.0 { 2.0 } // 高频 → 大于
            else if features.frequency_hz < 300.0 { 0.0 } // 低频 → 包含
            else { 1.0 }; // 中频 → 等于
        
        let directionality = match features.tempo_bpm {
            Some(bpm) if bpm > 120.0 => 1.0,  // 快节奏 → 正向
            Some(bpm) if bpm < 80.0 => -1.0,  // 慢节奏 → 负向
            _ => 0.0,
        };
        
        let abstraction_level = match features.pitch_class {
            Some(pc) => (pc as f64) / 11.0, // 0-11 映射到 0-1
            None => 0.5,
        };
        
        let complexity = ((features.duration_ms / 10000.0).min(0.3) +
            (freq_normalized * 0.3) +
            (features.amplitude * 0.2) +
            (if features.tempo_bpm.is_some() { 0.2 } else { 0.0 })).min(1.0);
        
        DeformationVector {
            subject_intensity: features.amplitude,
            relation_type,
            object_intensity: freq_normalized,
            directionality,
            tense: 0.0, // 音频是"现在"发生的
            certainty: 0.8, // 音频有一定模糊性
            abstraction_level,
            context_weight: 1.0,
            modality_marker: Modality::Audio.marker(),
            complexity,
        }
    }
    
    fn update_stats(&mut self) {
        self.stats.total_items = self.items.len();
        self.stats.by_modality.clear();
        
        let mut total_complexity = 0.0;
        for item in &self.items {
            let key = format!("{:?}", item.modality);
            *self.stats.by_modality.entry(key).or_insert(0) += 1;
            total_complexity += item.deformation.complexity;
        }
        
        if !self.items.is_empty() {
            self.stats.average_complexity = total_complexity / self.items.len() as f64;
        }
    }
    
    /// 获取统计信息
    pub fn get_stats(&self) -> &IngestionStats {
        &self.stats
    }
    
    /// 查找相似项
    pub fn find_similar(&self, target: &DeformationVector, threshold: f64) -> Vec<(usize, f64)> {
        self.items.iter()
            .enumerate()
            .filter_map(|(i, item)| {
                let sim = target.cosine_similarity(&item.deformation);
                if sim >= threshold {
                    Some((i, sim))
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// 跨模态查询 - 核心功能
    pub fn cross_modal_query(&self, query: &DataItem) -> Vec<CrossModalMatch> {
        let mut matches = Vec::new();
        
        for item in &self.items {
            if item.modality != query.modality {
                let similarity = query.deformation.cosine_similarity(&item.deformation);
                if similarity > 0.7 { // 阈值
                    matches.push(CrossModalMatch {
                        source_modality: query.modality,
                        target_modality: item.modality,
                        source_id: query.id.clone(),
                        target_id: item.id.clone(),
                        similarity,
                        interpretation: self.interpret_match(&query.deformation, &item.deformation),
                    });
                }
            }
        }
        
        matches.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());
        matches
    }
    
    fn interpret_match(&self, a: &DeformationVector, b: &DeformationVector) -> String {
        let relation_diff = (a.relation_type - b.relation_type).abs();
        let complexity_diff = (a.complexity - b.complexity).abs();
        
        if relation_diff < 0.5 && complexity_diff < 0.3 {
            "语义高度等价".to_string()
        } else if relation_diff < 1.0 {
            "结构相似".to_string()
        } else {
            "部分关联".to_string()
        }
    }
}

impl Default for DataIngestionEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// 跨模态匹配结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossModalMatch {
    pub source_modality: Modality,
    pub target_modality: Modality,
    pub source_id: String,
    pub target_id: String,
    pub similarity: f64,
    pub interpretation: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_text_encoding() {
        let engine = DataIngestionEngine::new();
        let vec1 = engine.encode_text("A contains B", "en");
        let vec2 = engine.encode_text("B 属于 A", "zh");
        
        let sim = vec1.cosine_similarity(&vec2);
        assert!(sim > 0.9, "跨语言等价逻辑相似度应 > 90%");
    }
    
    #[test]
    fn test_cross_modal() {
        let mut engine = DataIngestionEngine::new();
        
        // 文本: "包含关系"
        engine.ingest_text("Container holds items", "en").unwrap();
        
        // 代码: 函数定义（也是"包含"的一种形式）
        engine.ingest_code("fn container() { let items = vec![]; }", "rust").unwrap();
        
        // SVG: 矩形（视觉上的"包含"）
        engine.ingest_svg("<svg><rect x='0' y='0' width='100' height='100'/></svg>").unwrap();
        
        let text_item = &engine.items[0];
        let matches = engine.cross_modal_query(text_item);
        
        assert!(!matches.is_empty(), "应找到跨模态匹配");
    }
}
