//! 皮层 = 穷尽 + 排除（借鉴 s-wind 结构，不写物理）
//!
//! **概念**：
//! - **皮层 (Cortex)**：本模块实现的「穷尽探索 + 排除」循环，即思考层。输入来自脑体，输出为结构发现与排除集。
//! - **脑体 (Body)**：Valhalla（Hub、StemCell、Tile、Chamber）—— 产生并消费数据，由皮层观察与约束。
//!
//! **流程（与 s-wind 对齐）**：
//! 1. 从脑体取得 f64 信号
//! 2. Space 变换：Difference、Ratio、CumulativeSum 等
//! 3. 对变换后信号做简单结构识别（常数、线性、递推等）
//! 4. 排除：误差大或不稳定的假设丢弃
//! 5. 重复直至 closure（无新发现或达最大轮数）

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// 脑体来源（Valhalla 各系统）
// ---------------------------------------------------------------------------

/// 脑体单元类型：信号来自哪一个 Valhalla 系统
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BodyKind {
    Hub,
    StemCell,
    Tile,
    Chamber,
}

impl BodyKind {
    pub fn as_storage_key(&self) -> &'static str {
        match self {
            BodyKind::Hub => "hub",
            BodyKind::StemCell => "stemcell",
            BodyKind::Tile => "tile",
            BodyKind::Chamber => "chamber",
        }
    }
}

/// 来自脑体的一路信号（供皮层消费）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodySignal {
    pub source: BodyKind,
    pub channel_id: String,
    pub sequence: Vec<f64>,
}

// ---------------------------------------------------------------------------
// Space 变换（f64，与 s-wind TransformType 对应）
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransformType {
    /// Δ: 一阶差分
    Difference,
    /// Δ²: 二阶差分
    SecondDifference,
    /// 连续项比值
    Ratio,
    /// 累加
    CumulativeSum,
    /// 绝对值
    AbsoluteValue,
    /// 滑动平均，窗口长度
    MovingAverage(usize),
}

impl TransformType {
    /// 对 f64 序列做变换，返回新序列（可能变短）
    pub fn apply(&self, signal: &[f64]) -> Vec<f64> {
        match self {
            TransformType::Difference => {
                if signal.len() < 2 {
                    return vec![];
                }
                signal.windows(2).map(|w| w[1] - w[0]).collect()
            }
            TransformType::SecondDifference => {
                let first: Vec<f64> = signal.windows(2).map(|w| w[1] - w[0]).collect();
                if first.len() < 2 {
                    return vec![];
                }
                first.windows(2).map(|w| w[1] - w[0]).collect()
            }
            TransformType::Ratio => {
                if signal.len() < 2 {
                    return vec![];
                }
                signal
                    .windows(2)
                    .filter_map(|w| {
                        if w[0].abs() < 1e-20 {
                            None
                        } else {
                            Some(w[1] / w[0])
                        }
                    })
                    .collect()
            }
            TransformType::CumulativeSum => {
                let mut out = Vec::with_capacity(signal.len());
                let mut s = 0.0;
                for &x in signal {
                    s += x;
                    out.push(s);
                }
                out
            }
            TransformType::AbsoluteValue => signal.iter().map(|x| x.abs()).collect(),
            TransformType::MovingAverage(w) => {
                let w = (*w).max(1);
                if signal.len() < w {
                    return signal.to_vec();
                }
                signal
                    .windows(w)
                    .map(|wnd| wnd.iter().sum::<f64>() / w as f64)
                    .collect()
            }
        }
    }
}

/// 所有可枚举的变换（用于穷尽）
pub fn all_transforms_for_exhaustion() -> Vec<TransformType> {
    vec![
        TransformType::Difference,
        TransformType::SecondDifference,
        TransformType::Ratio,
        TransformType::CumulativeSum,
        TransformType::AbsoluteValue,
        TransformType::MovingAverage(2),
        TransformType::MovingAverage(3),
    ]
}

// ---------------------------------------------------------------------------
// 简单结构识别（用于建模 + 排除）
// ---------------------------------------------------------------------------

/// 识别出的简单结构类型（仅结构，无物理名）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StructureKind {
    /// 近似常数
    Constant(f64),
    /// 近似线性 y ≈ a*x + b
    Linear { a: f64, b: f64 },
    /// 未识别或排除
    Unknown,
}

/// 对一段 f64 做简单结构识别；返回结构类型与均方误差（用于排除）
pub fn fit_simple_structure(signal: &[f64]) -> (StructureKind, f64) {
    if signal.is_empty() {
        return (StructureKind::Unknown, f64::MAX);
    }
    let n = signal.len() as f64;
    let mean: f64 = signal.iter().sum::<f64>() / n;
    let var: f64 = signal.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
    let std = var.sqrt();
    if std < 1e-12 || var < 1e-20 {
        return (StructureKind::Constant(mean), 0.0);
    }
    let xs: Vec<f64> = (0..signal.len()).map(|i| i as f64).collect();
    let (a, b) = linear_fit(&xs, signal);
    let mse: f64 = xs
        .iter()
        .zip(signal.iter())
        .map(|(x, y)| (a * x + b - y).powi(2))
        .sum::<f64>()
        / n;
    (StructureKind::Linear { a, b }, mse)
}

fn linear_fit(x: &[f64], y: &[f64]) -> (f64, f64) {
    let n = x.len() as f64;
    if n < 2.0 {
        return (0.0, y.first().copied().unwrap_or(0.0));
    }
    let sx: f64 = x.iter().sum();
    let sy: f64 = y.iter().sum();
    let sxx: f64 = x.iter().map(|v| v * v).sum();
    let sxy: f64 = x.iter().zip(y.iter()).map(|(a, b)| a * b).sum();
    let den = n * sxx - sx * sx;
    if den.abs() < 1e-20 {
        return (0.0, sy / n);
    }
    let a = (n * sxy - sx * sy) / den;
    let b = (sy - a * sx) / n;
    (a, b)
}

// ---------------------------------------------------------------------------
// 排除与 closure
// ---------------------------------------------------------------------------

/// 单次探索结果：某变换 + 某结构，若误差过大则视为排除
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryCandidate {
    pub transform: TransformType,
    pub structure: StructureKind,
    pub mse: f64,
    pub excluded: bool,
}

/// 排除条件：误差大于阈值或结构为 Unknown 且误差大
pub fn should_exclude(mse: f64, structure: &StructureKind, error_threshold: f64) -> bool {
    if mse > error_threshold {
        return true;
    }
    matches!(structure, StructureKind::Unknown) && mse > 0.0
}

// ---------------------------------------------------------------------------
// 皮层：穷尽 + 排除 引擎
// ---------------------------------------------------------------------------

/// 皮层状态：当前轮数、已发现结构、已排除集合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CortexState {
    pub round: u32,
    pub discovered: Vec<DiscoveryCandidate>,
    pub excluded_count: usize,
}

/// 皮层配置（穷尽与排除参数）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CortexConfig {
    pub max_rounds: u32,
    pub error_threshold: f64,
    pub min_signal_len: usize,
}

impl Default for CortexConfig {
    fn default() -> Self {
        Self {
            max_rounds: 20,
            error_threshold: 1e-6,
            min_signal_len: 3,
        }
    }
}

/// 皮层：对脑体信号做穷尽探索 + 排除，直至 closure
pub struct Cortex {
    pub config: CortexConfig,
    pub state: CortexState,
    /// 已出现过的 (transform, structure) 签名，用于判断是否 closure（无新发现）
    seen_signatures: HashSet<String>,
}

impl Cortex {
    pub fn new(config: CortexConfig) -> Self {
        Self {
            config,
            state: CortexState {
                round: 0,
                discovered: Vec::new(),
                excluded_count: 0,
            },
            seen_signatures: HashSet::new(),
        }
    }

    /// 对当前信号做一轮：枚举变换 → 拟合结构 → 排除
    pub fn step(&mut self, signal: &[f64]) -> bool {
        if signal.len() < self.config.min_signal_len {
            return true;
        }
        self.state.round += 1;
        let mut new_this_round = 0;
        for transform in all_transforms_for_exhaustion() {
            let transformed = transform.apply(signal);
            if transformed.len() < self.config.min_signal_len {
                continue;
            }
            let (structure, mse) = fit_simple_structure(&transformed);
            let excluded =
                should_exclude(mse, &structure, self.config.error_threshold);
            let sig = format!("{:?}_{:?}", transform, structure);
            if !self.seen_signatures.contains(&sig) {
                self.seen_signatures.insert(sig);
                new_this_round += 1;
            }
            self.state.discovered.push(DiscoveryCandidate {
                transform,
                structure: structure.clone(),
                mse,
                excluded,
            });
            if excluded {
                self.state.excluded_count += 1;
            }
        }
        new_this_round == 0
    }

    /// 穷尽循环：从脑体信号出发，反复变换→拟合→排除，直到 closure 或达到 max_rounds
    pub fn run_until_closure(&mut self, body_signal: &BodySignal) {
        let mut current: Vec<f64> = body_signal.sequence.clone();
        while self.state.round < self.config.max_rounds {
            if current.len() < self.config.min_signal_len {
                break;
            }
            let start_len = self.state.discovered.len();
            let no_new = self.step(&current);
            let this_round = &self.state.discovered[start_len..];
            let best = this_round
                .iter()
                .filter(|c| !c.excluded)
                .min_by(|a, b| a.mse.partial_cmp(&b.mse).unwrap_or(std::cmp::Ordering::Equal));
            if let Some(b) = best {
                current = b.transform.apply(&current);
            } else {
                break;
            }
            if no_new {
                break;
            }
        }
    }

    pub fn state(&self) -> &CortexState {
        &self.state
    }
}

// ---------------------------------------------------------------------------
// 兼容旧 API：CortexKind / CortexSummary 视为「脑体单元」标记
// ---------------------------------------------------------------------------

/// 脑体单元类型（与 BodyKind 同义，用于对外命名「Valhalla 各系统」）
pub type CortexKind = BodyKind;

/// 脑体单元在某一时刻的职责摘要（是否承担存储、是否受皮层控制）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CortexSummary {
    pub kind: BodyKind,
    pub is_storage_point: bool,
    pub is_controller: bool,
}

impl CortexSummary {
    pub fn hub(is_storage_point: bool, is_controller: bool) -> Self {
        Self {
            kind: BodyKind::Hub,
            is_storage_point,
            is_controller,
        }
    }
    pub fn stemcell(is_storage_point: bool, is_controller: bool) -> Self {
        Self {
            kind: BodyKind::StemCell,
            is_storage_point,
            is_controller,
        }
    }
    pub fn tile(is_storage_point: bool, is_controller: bool) -> Self {
        Self {
            kind: BodyKind::Tile,
            is_storage_point,
            is_controller,
        }
    }
    pub fn chamber(is_storage_point: bool, is_controller: bool) -> Self {
        Self {
            kind: BodyKind::Chamber,
            is_storage_point,
            is_controller,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CortexRole {
    Cortex,
}
