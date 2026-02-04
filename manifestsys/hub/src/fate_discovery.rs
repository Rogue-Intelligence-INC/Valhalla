//! 🌌 Fate 自主发现模块 - 无预设语义分类
//!
//! 核心原则:
//! - 零预设：不预定义任何语义类别
//! - 原始特征：只提取物理/统计特征，不做语义解释
//! - Fate 学习：让 Fate 通过信号积累自主发现模式
//! - 自然涌现：语义结构从数据中自然涌现

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use fate::{FateSystem, DeformationSignal, Rational, SystemType};

/// 原始数据项 - 无语义标签
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawDataItem {
    pub id: String,
    /// 原始内容
    pub content: String,
    /// 原始特征向量 (纯统计/物理特征)
    pub raw_features: RawFeatures,
    /// Fate 分配的类别 (运行时发现)
    pub fate_cluster: Option<u32>,
    /// Fate 分配的置信度
    pub fate_confidence: Option<f64>,
}

/// 原始特征 - 纯统计特征，无语义解释
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawFeatures {
    // ═══ 长度特征 ═══
    pub byte_length: usize,
    pub char_count: usize,
    pub line_count: usize,
    pub word_count: usize,
    
    // ═══ 字符分布 ═══
    pub ascii_ratio: f64,
    pub digit_ratio: f64,
    pub alpha_ratio: f64,
    pub space_ratio: f64,
    pub punct_ratio: f64,
    pub cjk_ratio: f64,  // 中日韩字符
    
    // ═══ 结构特征 ═══
    pub bracket_depth: usize,  // 括号嵌套深度
    pub has_equals: bool,
    pub has_arrows: bool,      // -> => 
    pub has_operators: bool,   // + - * / > <
    pub newline_density: f64,  // 换行密度
    
    // ═══ 熵特征 ═══
    pub char_entropy: f64,     // 字符熵
    pub bigram_entropy: f64,   // 二元组熵
    
    // ═══ 新增：更细粒度的符号特征 ═══
    pub curly_brace_count: usize,    // { }
    pub square_bracket_count: usize, // [ ]
    pub paren_count: usize,          // ( )
    pub semicolon_count: usize,      // ;
    pub colon_count: usize,          // :
    pub dot_count: usize,            // .
    pub comma_count: usize,          // ,
    pub quote_count: usize,          // " '
    pub math_symbol_count: usize,    // ∫ ∑ ∀ ∃ → ∈ ⊆ 等
    pub greek_count: usize,          // α β γ δ 等
    pub subscript_count: usize,      // ₀ ₁ ₂ 等
    pub superscript_count: usize,    // ² ³ ⁿ 等
    pub uppercase_ratio: f64,        // 大写字母比例
    pub camelcase_score: f64,        // 驼峰命名得分
}

impl RawFeatures {
    /// 从原始内容提取特征 - 纯统计，无语义判断
    pub fn extract(content: &str) -> Self {
        let bytes = content.as_bytes();
        let chars: Vec<char> = content.chars().collect();
        let char_count = chars.len();
        
        // 字符分类计数
        let mut ascii_count = 0;
        let mut digit_count = 0;
        let mut alpha_count = 0;
        let mut space_count = 0;
        let mut punct_count = 0;
        let mut cjk_count = 0;
        
        for c in &chars {
            if c.is_ascii() { ascii_count += 1; }
            if c.is_ascii_digit() { digit_count += 1; }
            if c.is_alphabetic() { alpha_count += 1; }
            if c.is_whitespace() { space_count += 1; }
            if c.is_ascii_punctuation() { punct_count += 1; }
            if is_cjk(*c) { cjk_count += 1; }
        }
        
        let char_count_f = char_count.max(1) as f64;
        
        // 括号深度
        let mut max_depth: usize = 0;
        let mut current_depth: usize = 0;
        for c in &chars {
            match c {
                '(' | '[' | '{' | '<' => {
                    current_depth += 1;
                    max_depth = max_depth.max(current_depth);
                }
                ')' | ']' | '}' | '>' => {
                    current_depth = current_depth.saturating_sub(1);
                }
                _ => {}
            }
        }
        
        // 熵计算
        let char_entropy = calculate_entropy(&chars);
        let bigram_entropy = calculate_bigram_entropy(content);
        
        let line_count = content.lines().count().max(1);
        let word_count = content.split_whitespace().count();
        
        // 新增符号计数
        let curly_brace_count = chars.iter().filter(|c| **c == '{' || **c == '}').count();
        let square_bracket_count = chars.iter().filter(|c| **c == '[' || **c == ']').count();
        let paren_count = chars.iter().filter(|c| **c == '(' || **c == ')').count();
        let semicolon_count = chars.iter().filter(|c| **c == ';').count();
        let colon_count = chars.iter().filter(|c| **c == ':').count();
        let dot_count = chars.iter().filter(|c| **c == '.').count();
        let comma_count = chars.iter().filter(|c| **c == ',').count();
        let quote_count = chars.iter().filter(|c| **c == '"' || **c == '\'').count();
        
        // 数学符号
        let math_symbol_count = chars.iter().filter(|c| is_math_symbol(**c)).count();
        let greek_count = chars.iter().filter(|c| is_greek(**c)).count();
        let subscript_count = chars.iter().filter(|c| is_subscript(**c)).count();
        let superscript_count = chars.iter().filter(|c| is_superscript(**c)).count();
        
        // 大写比例和驼峰得分
        let uppercase_count = chars.iter().filter(|c| c.is_uppercase()).count();
        let uppercase_ratio = uppercase_count as f64 / char_count_f;
        let camelcase_score = calculate_camelcase_score(content);
        
        Self {
            byte_length: bytes.len(),
            char_count,
            line_count,
            word_count,
            ascii_ratio: ascii_count as f64 / char_count_f,
            digit_ratio: digit_count as f64 / char_count_f,
            alpha_ratio: alpha_count as f64 / char_count_f,
            space_ratio: space_count as f64 / char_count_f,
            punct_ratio: punct_count as f64 / char_count_f,
            cjk_ratio: cjk_count as f64 / char_count_f,
            bracket_depth: max_depth,
            has_equals: content.contains('='),
            has_arrows: content.contains("->") || content.contains("=>"),
            has_operators: chars.iter().any(|c| matches!(c, '+' | '-' | '*' | '/' | '>' | '<')),
            newline_density: (line_count as f64 - 1.0) / char_count_f,
            char_entropy,
            bigram_entropy,
            curly_brace_count,
            square_bracket_count,
            paren_count,
            semicolon_count,
            colon_count,
            dot_count,
            comma_count,
            quote_count,
            math_symbol_count,
            greek_count,
            subscript_count,
            superscript_count,
            uppercase_ratio,
            camelcase_score,
        }
    }
    
    /// 转换为 Fate 信号向量 (使用 Rational 近似)
    pub fn to_signal_vector(&self) -> Vec<Rational> {
        vec![
            f64_to_rational(self.byte_length as f64 / 1000.0),
            f64_to_rational(self.ascii_ratio),
            f64_to_rational(self.digit_ratio),
            f64_to_rational(self.alpha_ratio),
            f64_to_rational(self.cjk_ratio),
            f64_to_rational(self.punct_ratio),
            f64_to_rational(self.bracket_depth as f64 / 10.0),
            if self.has_equals { Rational::one() } else { Rational::zero() },
            if self.has_arrows { Rational::one() } else { Rational::zero() },
            if self.has_operators { Rational::one() } else { Rational::zero() },
            f64_to_rational(self.char_entropy / 8.0),
            f64_to_rational(self.bigram_entropy / 16.0),
            // 新增特征
            f64_to_rational(self.curly_brace_count as f64 / 10.0),
            f64_to_rational(self.paren_count as f64 / 10.0),
            f64_to_rational(self.semicolon_count as f64 / 5.0),
            f64_to_rational(self.math_symbol_count as f64 / 5.0),
            f64_to_rational(self.uppercase_ratio),
            f64_to_rational(self.camelcase_score),
        ]
    }
    
    /// 计算与另一个特征的欧氏距离
    pub fn distance(&self, other: &RawFeatures) -> f64 {
        let v1 = self.to_f64_vec();
        let v2 = other.to_f64_vec();
        
        v1.iter().zip(v2.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            .sqrt()
    }
    
    fn to_f64_vec(&self) -> Vec<f64> {
        vec![
            self.byte_length as f64 / 1000.0,
            self.ascii_ratio,
            self.digit_ratio,
            self.alpha_ratio,
            self.cjk_ratio,
            self.punct_ratio,
            self.bracket_depth as f64 / 10.0,
            if self.has_equals { 1.0 } else { 0.0 },
            if self.has_arrows { 1.0 } else { 0.0 },
            if self.has_operators { 1.0 } else { 0.0 },
            self.char_entropy / 8.0,
            self.bigram_entropy / 16.0,
            // 新增特征 - 关键区分维度
            self.curly_brace_count as f64 / 10.0,
            self.paren_count as f64 / 10.0,
            self.semicolon_count as f64 / 5.0,
            self.math_symbol_count as f64 / 5.0,
            self.uppercase_ratio,
            self.camelcase_score,
        ]
    }
}

/// 将 f64 转换为 Rational (使用整数近似)
fn f64_to_rational(value: f64) -> Rational {
    // 使用 1000 作为分母精度
    let scaled = (value * 1000.0).round() as i128;
    Rational::new(scaled, 1000)
}

/// 将 Rational 转换为 f64
fn rational_to_f64(r: &Rational) -> f64 {
    r.numerator as f64 / r.denominator as f64
}

/// 检测 CJK 字符
fn is_cjk(c: char) -> bool {
    let code = c as u32;
    (0x4E00..=0x9FFF).contains(&code) ||  // CJK 基本
    (0x3400..=0x4DBF).contains(&code) ||  // CJK 扩展 A
    (0x3000..=0x303F).contains(&code) ||  // CJK 标点
    (0x3040..=0x309F).contains(&code) ||  // 平假名
    (0x30A0..=0x30FF).contains(&code)     // 片假名
}

/// 检测数学符号
fn is_math_symbol(c: char) -> bool {
    matches!(c, '∫' | '∑' | '∏' | '∀' | '∃' | '∈' | '∉' | '⊆' | '⊂' | '⊇' | '⊃' |
               '∪' | '∩' | '→' | '←' | '↔' | '⇒' | '⇔' | '∧' | '∨' | '¬' |
               '≤' | '≥' | '≠' | '≈' | '≡' | '∞' | '∂' | '∇' | '√' | '∝' |
               '±' | '×' | '÷' | '·' | '°')
}

/// 检测希腊字母
fn is_greek(c: char) -> bool {
    let code = c as u32;
    (0x0370..=0x03FF).contains(&code) // Greek and Coptic
}

/// 检测下标字符
fn is_subscript(c: char) -> bool {
    matches!(c, '₀' | '₁' | '₂' | '₃' | '₄' | '₅' | '₆' | '₇' | '₈' | '₉' |
               'ₐ' | 'ₑ' | 'ₕ' | 'ᵢ' | 'ⱼ' | 'ₖ' | 'ₗ' | 'ₘ' | 'ₙ' | 'ₒ' | 'ₚ' | 'ᵣ' | 'ₛ' | 'ₜ' | 'ᵤ' | 'ᵥ' | 'ₓ')
}

/// 检测上标字符
fn is_superscript(c: char) -> bool {
    matches!(c, '⁰' | '¹' | '²' | '³' | '⁴' | '⁵' | '⁶' | '⁷' | '⁸' | '⁹' |
               'ⁿ' | 'ˣ' | 'ʸ')
}

/// 计算驼峰命名得分
fn calculate_camelcase_score(text: &str) -> f64 {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() < 2 {
        return 0.0;
    }
    
    let mut transitions = 0;
    for window in chars.windows(2) {
        // 小写后跟大写 = 驼峰
        if window[0].is_lowercase() && window[1].is_uppercase() {
            transitions += 1;
        }
    }
    
    (transitions as f64 / chars.len() as f64).min(1.0)
}

/// 计算字符熵
fn calculate_entropy(chars: &[char]) -> f64 {
    if chars.is_empty() { return 0.0; }
    
    let mut freq: HashMap<char, usize> = HashMap::new();
    for c in chars {
        *freq.entry(*c).or_insert(0) += 1;
    }
    
    let total = chars.len() as f64;
    freq.values()
        .map(|&count| {
            let p = count as f64 / total;
            if p > 0.0 { -p * p.log2() } else { 0.0 }
        })
        .sum()
}

/// 计算二元组熵
fn calculate_bigram_entropy(text: &str) -> f64 {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() < 2 { return 0.0; }
    
    let mut freq: HashMap<(char, char), usize> = HashMap::new();
    for window in chars.windows(2) {
        *freq.entry((window[0], window[1])).or_insert(0) += 1;
    }
    
    let total = (chars.len() - 1) as f64;
    freq.values()
        .map(|&count| {
            let p = count as f64 / total;
            if p > 0.0 { -p * p.log2() } else { 0.0 }
        })
        .sum()
}

/// 🌌 Fate 自主发现引擎
pub struct FateDiscoveryEngine {
    /// Fate 系统
    fate: FateSystem,
    /// 已处理的数据项
    items: Vec<RawDataItem>,
    /// Fate 发现的聚类中心
    cluster_centers: Vec<RawFeatures>,
    /// 信号计数
    signal_count: u64,
}

impl FateDiscoveryEngine {
    pub fn new() -> Self {
        let resource_limits = fate::ResourceLimits {
            max_memory_mb: 8192,
            max_cpu_percent: 50,
            max_buffer_size: 10000,
            processing_interval_ms: 10,
        };
        
        Self {
            fate: FateSystem::new("fate_discovery".to_string(), resource_limits),
            items: Vec::new(),
            cluster_centers: Vec::new(),
            signal_count: 0,
        }
    }
    
    /// 摄入原始数据 - 无任何语义预设
    pub fn ingest(&mut self, content: &str) -> RawDataItem {
        let features = RawFeatures::extract(content);
        
        // 根据特征选择初始系统 (轮询方式，无偏好)
        let source_system = match self.signal_count as usize % 4 {
            0 => SystemType::BlackHole,
            1 => SystemType::MeshBrain,
            2 => SystemType::MultiNova,
            _ => SystemType::BaseForce,
        };
        
        // 创建形变信号发送给 Fate
        let signal = DeformationSignal {
            source_system,
            signal_tick: self.fate.get_logical_time(),
            intensity: f64_to_rational(features.char_entropy / 8.0),
            direction: features.to_signal_vector(),
            quality_score: Rational::one(),
        };
        
        // 发送给 Fate
        self.fate.receive_signal(signal);
        self.signal_count += 1;
        
        let item = RawDataItem {
            id: format!("item_{}", self.items.len()),
            content: content.to_string(),
            raw_features: features,
            fate_cluster: None,
            fate_confidence: None,
        };
        
        self.items.push(item.clone());
        item
    }
    
    /// 让 Fate 处理并发现模式 (通过清理过期信号触发)
    pub fn fate_process(&mut self) {
        // 清理旧信号会触发 Fate 内部处理
        self.fate.cleanup_expired_signals(1000);
    }
    
    /// Fate 自主聚类 - 无预设类别数
    pub fn fate_discover_clusters(&mut self) -> Vec<FateCluster> {
        if self.items.is_empty() {
            return Vec::new();
        }
        
        // 获取 Fate 的系统偏好作为聚类指导
        let prefs = self.fate.calculate_system_preferences();
        
        // 使用 Fate 的亲和力和数据特征动态决定聚类数
        // 亲和力方差 + 数据多样性 → 聚类数
        let affinity_variance = calculate_affinity_variance(&prefs.affinities);
        
        // 计算数据多样性得分
        let diversity = self.calculate_data_diversity();
        
        // 动态 k: 基于方差和多样性
        let base_k = ((affinity_variance * 15.0) + (diversity * 8.0)).ceil() as usize;
        let dynamic_k = base_k.max(3).min(12); // 至少 3 个，最多 12 个
        
        println!("  📊 Fate 动态聚类数: {} (方差={:.4}, 多样性={:.4})", dynamic_k, affinity_variance, diversity);
        
        // 简单 k-means，但 k 由 Fate 决定
        let clusters = self.kmeans_cluster(dynamic_k);
        
        // 更新每个 item 的聚类归属
        for (i, item) in self.items.iter_mut().enumerate() {
            if let Some(cluster_id) = clusters.iter()
                .find(|c| c.member_indices.contains(&i))
                .map(|c| c.cluster_id)
            {
                item.fate_cluster = Some(cluster_id);
            }
        }
        
        clusters
    }
    
    /// k-means 聚类 (k 由 Fate 决定)
    fn kmeans_cluster(&mut self, k: usize) -> Vec<FateCluster> {
        if self.items.len() < k {
            return Vec::new();
        }
        
        // 初始化聚类中心 (随机选择)
        let mut centers: Vec<RawFeatures> = self.items.iter()
            .take(k)
            .map(|item| item.raw_features.clone())
            .collect();
        
        let max_iterations = 50;
        let mut assignments: Vec<usize> = vec![0; self.items.len()];
        
        for _ in 0..max_iterations {
            // 分配阶段
            let mut changed = false;
            for (i, item) in self.items.iter().enumerate() {
                let closest = centers.iter()
                    .enumerate()
                    .min_by(|(_, a), (_, b)| {
                        item.raw_features.distance(a)
                            .partial_cmp(&item.raw_features.distance(b))
                            .unwrap()
                    })
                    .map(|(idx, _)| idx)
                    .unwrap_or(0);
                
                if assignments[i] != closest {
                    assignments[i] = closest;
                    changed = true;
                }
            }
            
            if !changed {
                break;
            }
            
            // 更新中心
            for (c_idx, center) in centers.iter_mut().enumerate() {
                let members: Vec<_> = self.items.iter()
                    .enumerate()
                    .filter(|(i, _)| assignments[*i] == c_idx)
                    .map(|(_, item)| &item.raw_features)
                    .collect();
                
                if !members.is_empty() {
                    *center = average_features(&members);
                }
            }
        }
        
        // 构建聚类结果
        self.cluster_centers = centers.clone();
        
        let mut clusters: Vec<FateCluster> = Vec::new();
        for c_idx in 0..k {
            let member_indices: Vec<usize> = assignments.iter()
                .enumerate()
                .filter(|(_, &a)| a == c_idx)
                .map(|(i, _)| i)
                .collect();
            
            if !member_indices.is_empty() {
                clusters.push(FateCluster {
                    cluster_id: c_idx as u32,
                    center: centers[c_idx].clone(),
                    member_indices,
                    discovered_pattern: None, // Fate 稍后填充
                });
            }
        }
        
        clusters
    }
    
    /// Fate 分析聚类特征 - 事后发现，非预设
    pub fn fate_analyze_clusters(&self, clusters: &mut [FateCluster]) {
        for cluster in clusters.iter_mut() {
            let center = &cluster.center;
            
            // Fate 观察特征，事后推断模式（不是预设）
            // 按特征显著性排序判断
            let pattern = if center.cjk_ratio > 0.5 {
                format!("CJK主导 (CJK={:.0}%)", center.cjk_ratio * 100.0)
            } else if center.math_symbol_count > 0 || center.greek_count > 0 {
                format!("数学符号 (∫∑={}, Greek={})", center.math_symbol_count, center.greek_count)
            } else if center.curly_brace_count > 2 && center.semicolon_count > 0 {
                format!("类C代码 ({{}}={}, ;={})", center.curly_brace_count, center.semicolon_count)
            } else if center.colon_count > 1 && center.has_arrows {
                format!("类Python/Rust (def/fn) (:={}, →={})", center.colon_count, if center.has_arrows { "✓" } else { "✗" })
            } else if center.bracket_depth > 2 {
                format!("深嵌套结构 (深度={})", center.bracket_depth)
            } else if center.paren_count > 3 {
                format!("括号密集 (()={})", center.paren_count)
            } else if center.quote_count > 2 {
                format!("字符串密集 (引号={})", center.quote_count)
            } else if center.has_operators && center.digit_ratio > 0.15 {
                format!("算术表达式 (数字={:.0}%)", center.digit_ratio * 100.0)
            } else if center.uppercase_ratio > 0.3 {
                format!("高大写比例 (大写={:.0}%)", center.uppercase_ratio * 100.0)
            } else if center.camelcase_score > 0.05 {
                format!("驼峰命名 (得分={:.2})", center.camelcase_score)
            } else if center.alpha_ratio > 0.7 {
                format!("自然语言 (字母={:.0}%)", center.alpha_ratio * 100.0)
            } else {
                format!("混合特征 (熵={:.2}, ASCII={:.0}%)", center.char_entropy, center.ascii_ratio * 100.0)
            };
            
            cluster.discovered_pattern = Some(pattern);
        }
    }
    
    /// 计算数据多样性
    fn calculate_data_diversity(&self) -> f64 {
        if self.items.len() < 2 {
            return 0.0;
        }
        
        // 计算各特征的标准差
        let mut cjk_values: Vec<f64> = Vec::new();
        let mut brace_values: Vec<f64> = Vec::new();
        let mut math_values: Vec<f64> = Vec::new();
        let mut entropy_values: Vec<f64> = Vec::new();
        
        for item in &self.items {
            cjk_values.push(item.raw_features.cjk_ratio);
            brace_values.push(item.raw_features.curly_brace_count as f64);
            math_values.push(item.raw_features.math_symbol_count as f64);
            entropy_values.push(item.raw_features.char_entropy);
        }
        
        let cjk_std = std_dev(&cjk_values);
        let brace_std = std_dev(&brace_values) / 10.0; // 归一化
        let math_std = std_dev(&math_values) / 5.0;
        let entropy_std = std_dev(&entropy_values) / 4.0;
        
        // 综合多样性得分
        (cjk_std + brace_std + math_std + entropy_std) / 4.0
    }
    
    /// 获取统计信息
    pub fn get_stats(&self) -> DiscoveryStats {
        DiscoveryStats {
            total_items: self.items.len(),
            signal_count: self.signal_count,
            cluster_count: self.cluster_centers.len(),
            fate_logical_time: self.fate.get_logical_time(),
        }
    }
    
    /// 获取所有数据项
    pub fn get_items(&self) -> &[RawDataItem] {
        &self.items
    }
    
    /// 跨数据相似度查询 - 基于原始特征，非语义
    pub fn find_similar(&self, target: &RawDataItem, threshold: f64) -> Vec<(usize, f64)> {
        self.items.iter()
            .enumerate()
            .filter(|(i, _)| self.items[*i].id != target.id)
            .filter_map(|(i, item)| {
                let dist = target.raw_features.distance(&item.raw_features);
                let similarity = 1.0 / (1.0 + dist); // 距离转相似度
                if similarity >= threshold {
                    Some((i, similarity))
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Default for FateDiscoveryEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Fate 发现的聚类
#[derive(Debug, Clone)]
pub struct FateCluster {
    pub cluster_id: u32,
    pub center: RawFeatures,
    pub member_indices: Vec<usize>,
    /// Fate 事后发现的模式描述（非预设）
    pub discovered_pattern: Option<String>,
}

/// 发现统计
#[derive(Debug, Clone)]
pub struct DiscoveryStats {
    pub total_items: usize,
    pub signal_count: u64,
    pub cluster_count: usize,
    pub fate_logical_time: u64,
}

/// 计算标准差
fn std_dev(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values.iter()
        .map(|v| (v - mean).powi(2))
        .sum::<f64>() / values.len() as f64;
    variance.sqrt()
}

/// 计算亲和力方差
fn calculate_affinity_variance(affinities: &[Rational]) -> f64 {
    if affinities.is_empty() {
        return 0.0;
    }
    
    let values: Vec<f64> = affinities.iter()
        .map(|r| rational_to_f64(r))
        .collect();
    
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values.iter()
        .map(|v| (v - mean).powi(2))
        .sum::<f64>() / values.len() as f64;
    
    variance
}

/// 计算特征平均值
fn average_features(features: &[&RawFeatures]) -> RawFeatures {
    if features.is_empty() {
        return RawFeatures::extract("");
    }
    
    let n = features.len() as f64;
    
    RawFeatures {
        byte_length: (features.iter().map(|f| f.byte_length).sum::<usize>() as f64 / n) as usize,
        char_count: (features.iter().map(|f| f.char_count).sum::<usize>() as f64 / n) as usize,
        line_count: (features.iter().map(|f| f.line_count).sum::<usize>() as f64 / n) as usize,
        word_count: (features.iter().map(|f| f.word_count).sum::<usize>() as f64 / n) as usize,
        ascii_ratio: features.iter().map(|f| f.ascii_ratio).sum::<f64>() / n,
        digit_ratio: features.iter().map(|f| f.digit_ratio).sum::<f64>() / n,
        alpha_ratio: features.iter().map(|f| f.alpha_ratio).sum::<f64>() / n,
        space_ratio: features.iter().map(|f| f.space_ratio).sum::<f64>() / n,
        punct_ratio: features.iter().map(|f| f.punct_ratio).sum::<f64>() / n,
        cjk_ratio: features.iter().map(|f| f.cjk_ratio).sum::<f64>() / n,
        bracket_depth: (features.iter().map(|f| f.bracket_depth).sum::<usize>() as f64 / n) as usize,
        has_equals: features.iter().filter(|f| f.has_equals).count() > features.len() / 2,
        has_arrows: features.iter().filter(|f| f.has_arrows).count() > features.len() / 2,
        has_operators: features.iter().filter(|f| f.has_operators).count() > features.len() / 2,
        newline_density: features.iter().map(|f| f.newline_density).sum::<f64>() / n,
        char_entropy: features.iter().map(|f| f.char_entropy).sum::<f64>() / n,
        bigram_entropy: features.iter().map(|f| f.bigram_entropy).sum::<f64>() / n,
        // 新增字段
        curly_brace_count: (features.iter().map(|f| f.curly_brace_count).sum::<usize>() as f64 / n) as usize,
        square_bracket_count: (features.iter().map(|f| f.square_bracket_count).sum::<usize>() as f64 / n) as usize,
        paren_count: (features.iter().map(|f| f.paren_count).sum::<usize>() as f64 / n) as usize,
        semicolon_count: (features.iter().map(|f| f.semicolon_count).sum::<usize>() as f64 / n) as usize,
        colon_count: (features.iter().map(|f| f.colon_count).sum::<usize>() as f64 / n) as usize,
        dot_count: (features.iter().map(|f| f.dot_count).sum::<usize>() as f64 / n) as usize,
        comma_count: (features.iter().map(|f| f.comma_count).sum::<usize>() as f64 / n) as usize,
        quote_count: (features.iter().map(|f| f.quote_count).sum::<usize>() as f64 / n) as usize,
        math_symbol_count: (features.iter().map(|f| f.math_symbol_count).sum::<usize>() as f64 / n) as usize,
        greek_count: (features.iter().map(|f| f.greek_count).sum::<usize>() as f64 / n) as usize,
        subscript_count: (features.iter().map(|f| f.subscript_count).sum::<usize>() as f64 / n) as usize,
        superscript_count: (features.iter().map(|f| f.superscript_count).sum::<usize>() as f64 / n) as usize,
        uppercase_ratio: features.iter().map(|f| f.uppercase_ratio).sum::<f64>() / n,
        camelcase_score: features.iter().map(|f| f.camelcase_score).sum::<f64>() / n,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_raw_feature_extraction() {
        let features = RawFeatures::extract("Hello World 你好世界");
        assert!(features.cjk_ratio > 0.0);
        assert!(features.ascii_ratio > 0.0);
    }
    
    #[test]
    fn test_fate_discovery() {
        let mut engine = FateDiscoveryEngine::new();
        
        // 摄入不同类型的数据 - 无标签
        engine.ingest("Hello World");
        engine.ingest("你好世界");
        engine.ingest("fn main() { println!(\"test\"); }");
        engine.ingest("def main(): print('test')");
        engine.ingest("x = 1 + 2 * 3");
        
        // 让 Fate 处理
        engine.fate_process();
        
        // Fate 自主聚类
        let clusters = engine.fate_discover_clusters();
        
        assert!(!clusters.is_empty());
    }
}
