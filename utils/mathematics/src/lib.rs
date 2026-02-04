//! Valhalla Mathematics - 被动数学库
//!
//! 提供公式查询、未知模式检测等数学能力

use serde::{Deserialize, Serialize};
use log::info;
use std::collections::HashMap;

/// 被动数学库
#[derive(Debug, Clone)]
pub struct PassiveMathematicalLibrary {
    pub library_id: String,
    pub formulas: HashMap<String, MathematicalFormula>,
    pub unknown_patterns: Vec<UnknownPattern>,
}

/// 数学公式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MathematicalFormula {
    pub formula_name: String,
    pub formula_expression: String,
    pub domain: String,
    pub confidence: f64,
}

/// 未知模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnknownPattern {
    pub pattern_id: String,
    pub description: String,
    pub detected_at: u64,
    pub unknown_degree: f64,
}

/// 查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LookupResult {
    pub found: bool,
    pub applicable_formulas: Vec<MathematicalFormula>,
    pub confidence: f64,
}

impl PassiveMathematicalLibrary {
    pub fn new() -> Self {
        info!("🔧 初始化被动数学库");
        let mut formulas = HashMap::new();
        
        // 预置一些基础公式
        formulas.insert("weight".to_string(), MathematicalFormula {
            formula_name: "weight".to_string(),
            formula_expression: "weight = intensity × quality_score".to_string(),
            domain: "fate_system".to_string(),
            confidence: 1.0,
        });
        
        formulas.insert("affinity".to_string(), MathematicalFormula {
            formula_name: "affinity".to_string(),
            formula_expression: "affinity[i] = Σ(direction[i] × weight) / Σ(weight)".to_string(),
            domain: "fate_system".to_string(),
            confidence: 1.0,
        });

        Self {
            library_id: uuid::Uuid::new_v4().to_string(),
            formulas,
            unknown_patterns: Vec::new(),
        }
    }

    /// 查询公式
    pub fn lookup(&self, query: &str) -> LookupResult {
        let mut applicable = Vec::new();
        let mut max_confidence = 0.0;

        for (name, formula) in &self.formulas {
            if name.contains(query) || formula.domain.contains(query) {
                applicable.push(formula.clone());
                if formula.confidence > max_confidence {
                    max_confidence = formula.confidence;
                }
            }
        }

        LookupResult {
            found: !applicable.is_empty(),
            applicable_formulas: applicable,
            confidence: max_confidence,
        }
    }

    /// 检测未知模式
    pub fn detect_unknown_pattern(&mut self, data: &[f64], threshold: f64) -> Option<UnknownPattern> {
        let variance = self.calculate_variance(data);
        if variance > threshold {
            let pattern = UnknownPattern {
                pattern_id: uuid::Uuid::new_v4().to_string(),
                description: format!("高方差模式: variance={:.4}", variance),
                detected_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                unknown_degree: (variance / threshold).min(1.0),
            };
            self.unknown_patterns.push(pattern.clone());
            Some(pattern)
        } else {
            None
        }
    }

    fn calculate_variance(&self, data: &[f64]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / data.len() as f64
    }
}

impl Default for PassiveMathematicalLibrary {
    fn default() -> Self {
        Self::new()
    }
}
