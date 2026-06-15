//! Yaksha Evolve: Fate-controlled experimental modes
//!
//! This module provides experimental, Fate-controlled variants of Yaksha
//! encryption. It is designed for evolution and testing, without changing
//! the stable public APIs (`neural`, `parametric`, `aead`).
//!
//! High-level flow:
//! - Use `YakshaKdf` (DynamicKeyGenerator) as Fate-like signal engine
//! - Derive a compact `FateYakshaControl` describing:
//!   - internal mode (neural / parametric / mixed)
//!   - cascade pattern
//!   - "breath" count (how many internal steps)
//!   - strength levels for geometry / neural / extra PRG layer
//! - Apply base Yaksha transforms, then apply Fate-controlled cascade

use crate::aead::{FateQuality, YakshaKdf};
use crate::frac_container::FracDimContainer;
use crate::ocean_pipeline::{decrypt_with_ocean, encrypt_with_ocean};
use crate::{neural, parametric};

/// Experimental high-level mode selection controlled by Fate
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FateMode {
    Neural,
    Parametric,
    NeuralThenParametric,
    ParametricThenNeural,
}

/// Experimental cascade pattern, controlling how many times and in which
/// direction the internal Feistel-like cascade is applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CascadePattern {
    /// Use the existing forward Feistel cascade only
    ForwardFeistel,
    /// Alternate forward / "reverse" uses of the cascade for extra mixing
    Bidirectional,
    /// Perform multiple forward rounds; total rounds depend on breaths
    MultiRound(u8),
}

/// Fate → Yaksha control signal, derived from `YakshaKdf`.
/// This struct is intentionally compact so that it can be logged and later
/// used by Fate / S-wind to learn preferences over parameter regions.
///
/// See [EVOLVE_SPEC.md](https://github.com/Rogue-Intelligence-INC/Valhalla/blob/main/valhalla-crypto/yaksha-cipher/EVOLVE_SPEC.md) for normative encoding (§6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FateYakshaControl {
    /// Which internal transform pipeline to use
    pub mode: FateMode,
    /// How to apply the cascade (direction/round pattern)
    pub cascade_pattern: CascadePattern,
    /// Discrete "breath" count (number of internal steps), in [1, 16]
    pub breaths: u8,
    /// Geometry / curvature related strength (0–255)
    pub geom_level: u8,
    /// Neural / fluctuation related strength (0–255)
    pub neural_level: u8,
    /// Extra pseudo-random layer strength (0–255)
    pub prg_level: u8,
    /// Simple label for Fate-side classification (e.g. by domain)
    pub label: u8,
}

impl Default for FateYakshaControl {
    fn default() -> Self {
        Self {
            mode: FateMode::Neural,
            cascade_pattern: CascadePattern::ForwardFeistel,
            breaths: 1,
            geom_level: 0,
            neural_level: 0,
            prg_level: 0,
            label: 0,
        }
    }
}

impl FateYakshaControl {
    /// Encode control to 16-byte normative form (EVOLVE_SPEC §6).
    /// Used for storing control with ciphertext so decryption can be done later.
    pub fn to_bytes(&self) -> [u8; 16] {
        let (pattern_byte, extra_byte) = match self.cascade_pattern {
            CascadePattern::ForwardFeistel => (0, 0),
            CascadePattern::Bidirectional => (1, 0),
            CascadePattern::MultiRound(extra) => (2, extra),
        };
        let mode_byte = match self.mode {
            FateMode::Neural => 0,
            FateMode::Parametric => 1,
            FateMode::NeuralThenParametric => 2,
            FateMode::ParametricThenNeural => 3,
        };
        [
            mode_byte,
            pattern_byte,
            extra_byte,
            self.breaths,
            self.geom_level,
            self.neural_level,
            self.prg_level,
            self.label,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ]
    }

    /// Decode control from 16-byte normative form (EVOLVE_SPEC §6).
    /// Returns None if bytes are invalid (e.g. mode > 3, breaths 0).
    pub fn from_bytes(bytes: &[u8; 16]) -> Option<Self> {
        let mode = match bytes[0] {
            0 => FateMode::Neural,
            1 => FateMode::Parametric,
            2 => FateMode::NeuralThenParametric,
            3 => FateMode::ParametricThenNeural,
            _ => return None,
        };
        let cascade_pattern = match bytes[1] {
            0 => CascadePattern::ForwardFeistel,
            1 => CascadePattern::Bidirectional,
            2 => CascadePattern::MultiRound(bytes[2]),
            _ => return None,
        };
        let breaths = bytes[3];
        if breaths == 0 || breaths > 16 {
            return None;
        }
        Some(Self {
            mode,
            cascade_pattern,
            breaths,
            geom_level: bytes[4],
            neural_level: bytes[5],
            prg_level: bytes[6],
            label: bytes[7],
        })
    }

    /// Derive a control signal from `YakshaKdf`.
    ///
    /// This uses a derived key as a compact random source, but keeps the
    /// mapping simple and deterministic so that Fate can later learn which
    /// regions of the control space lead to better statistical behaviour.
    pub fn from_fate(kdf: &mut YakshaKdf) -> Self {
        // 16 bytes of control material
        let control_key = kdf.derive_key(b"yaksha_control");
        let domain_idx = kdf.domain() as u8;

        // Mode selection from low 2 bits
        let mode = match control_key[0] % 4 {
            0 => FateMode::Neural,
            1 => FateMode::Parametric,
            2 => FateMode::NeuralThenParametric,
            _ => FateMode::ParametricThenNeural,
        };

        // Cascade pattern from next byte
        let cascade_pattern = match control_key[1] % 3 {
            0 => CascadePattern::ForwardFeistel,
            1 => CascadePattern::Bidirectional,
            _ => CascadePattern::MultiRound(2 + (control_key[2] % 5)),
        };

        // Breath count: at least 1, at most 16
        let breaths = 1 + (control_key[3] & 0x0F);

        Self {
            mode,
            cascade_pattern,
            breaths,
            geom_level: control_key[4],
            neural_level: control_key[5],
            prg_level: control_key[6],
            label: domain_idx,
        }
    }
}

/// 粗粒度数据类型提示（由调用方给出）
#[derive(Debug, Clone, Copy)]
pub enum DataTypeHint {
    Text,
    Binary,
    LowEntropy,
    HighEntropy,
}

/// 使用场景提示（由调用方给出）
#[derive(Debug, Clone, Copy)]
pub enum Scenario {
    Storage,
    Channel,
    HighRandomness,
    Experimental,
}

/// Fate 规则模板：可修改的“最低要求 + 偏好”
#[derive(Debug, Clone)]
pub struct FateRuleTemplate {
    pub scenario: Scenario,
    pub data_type: DataTypeHint,
    pub min_breaths: u8,
    pub prefer_pattern: Option<CascadePattern>,
    pub forbid_bidirectional: bool,
    pub min_quality_score: f64,
}

impl FateRuleTemplate {
    pub fn default_for(scenario: Scenario, data_type: DataTypeHint) -> Self {
        match (scenario, data_type) {
            (Scenario::HighRandomness, _) => Self {
                scenario,
                data_type,
                min_breaths: 6,
                prefer_pattern: Some(CascadePattern::MultiRound(4)),
                forbid_bidirectional: false,
                min_quality_score: 0.6,
            },
            (Scenario::Storage, DataTypeHint::LowEntropy) => Self {
                scenario,
                data_type,
                min_breaths: 4,
                prefer_pattern: Some(CascadePattern::MultiRound(2)),
                forbid_bidirectional: false,
                min_quality_score: 0.5,
            },
            (Scenario::Channel, _) => Self {
                scenario,
                data_type,
                min_breaths: 3,
                prefer_pattern: Some(CascadePattern::ForwardFeistel),
                forbid_bidirectional: false,
                min_quality_score: 0.4,
            },
            _ => Self {
                scenario,
                data_type,
                min_breaths: 2,
                prefer_pattern: None,
                forbid_bidirectional: false,
                min_quality_score: 0.0,
            },
        }
    }
}

/// Fate 对一次控制决策的完整观察结果
#[derive(Debug, Clone)]
pub struct FateDecision {
    pub scenario: Scenario,
    pub data_type: DataTypeHint,
    pub control_before_rules: FateYakshaControl,
    pub control_after_rules: FateYakshaControl,
    pub quality: FateQuality,
    pub quality_score: f64,
    pub template: FateRuleTemplate,
    pub notes: Vec<String>,
}

/// 只做决策和记录，不立即加密。调用方可以查看 FateDecision 后选择是否接受控制。
pub fn fate_decide_for(
    _plaintext: &[u8],
    master_key: &[u8],
    nonce: &[u8],
    scenario: Scenario,
    data_type: DataTypeHint,
) -> FateDecision {
    let mut kdf = YakshaKdf::new(master_key);
    let _ = kdf.derive_key(nonce);
    let (quality, score) = kdf.assess_quality();

    let template = FateRuleTemplate::default_for(scenario, data_type);

    // Fate 原始控制
    let mut control = FateYakshaControl::from_fate(&mut kdf);
    let original = control;

    let mut notes = Vec::new();

    // 应用模板约束：最小呼吸步数
    if control.breaths < template.min_breaths {
        notes.push(format!(
            "breaths {} < min_breaths {}, bumping",
            control.breaths, template.min_breaths
        ));
        control.breaths = template.min_breaths;
    }

    // 模式偏好：级联模式
    if let Some(pref) = template.prefer_pattern {
        if control.cascade_pattern != pref {
            notes.push(format!(
                "pattern {:?} -> {:?} (template preference)",
                control.cascade_pattern, pref
            ));
            control.cascade_pattern = pref;
        }
    }

    // 禁止 Bidirectional
    if template.forbid_bidirectional {
        if let CascadePattern::Bidirectional = control.cascade_pattern {
            notes.push(
                "Bidirectional forbidden for this scenario, switching to ForwardFeistel".into(),
            );
            control.cascade_pattern = CascadePattern::ForwardFeistel;
        }
    }

    // 质量过低时轻微加强扩散
    if score < template.min_quality_score {
        notes.push(format!(
            "quality score {:.2} < min_quality_score {:.2}, bump breaths by +1",
            score, template.min_quality_score
        ));
        control.breaths = control.breaths.saturating_add(1).min(16);
    }

    FateDecision {
        scenario,
        data_type,
        control_before_rules: original,
        control_after_rules: control,
        quality,
        quality_score: score,
        template,
        notes,
    }
}

/// Fate-controlled experimental encrypt: use Fate to choose internal mode,
/// and built-in **ocean** disturbance (index-keyed predictive XOR from
/// `(master_key, nonce, phase, i)` only — self-inverse, no sequential mask buffer).
/// Depends only on (master_key, nonce) and does not perform any quality judgement.
pub fn fate_neural_encrypt(
    plaintext: &[u8],
    master_key: &[u8],
    nonce: &[u8],
) -> (Vec<u8>, FateYakshaControl) {
    if plaintext.is_empty() || master_key.is_empty() {
        return (plaintext.to_vec(), FateYakshaControl::default());
    }

    // 1. Build Fate-like key generator and absorb nonce as session signal
    let mut kdf = YakshaKdf::new(master_key);
    let _session_key = kdf.derive_key(nonce);

    // 2. Derive a single control signal for this operation
    let control = FateYakshaControl::from_fate(&mut kdf);

    let container = FracDimContainer::standard();
    let mut buf = encrypt_with_ocean(
        plaintext,
        master_key,
        nonce,
        container,
        |p| match control.mode {
            FateMode::Neural => neural::encrypt_body(p, master_key),
            FateMode::Parametric => parametric::encrypt_entropy_body(p, master_key),
            FateMode::NeuralThenParametric => {
                let tmp = neural::encrypt_body(p, master_key);
                parametric::encrypt_entropy_body(&tmp, master_key)
            }
            FateMode::ParametricThenNeural => {
                let tmp = parametric::encrypt_entropy_body(p, master_key);
                neural::encrypt_body(&tmp, master_key)
            }
        },
    );

    apply_fate_cascade(&mut buf, master_key, &control);

    (buf, control)
}

/// Encrypt with an explicit control (for conformance tests and when control was stored).
/// Same pipeline as `fate_neural_encrypt` but skips KDF; decryption uses `fate_neural_decrypt(..., control)`.
pub fn fate_neural_encrypt_with_control(
    plaintext: &[u8],
    master_key: &[u8],
    control: &FateYakshaControl,
) -> Vec<u8> {
    if plaintext.is_empty() || master_key.is_empty() {
        return plaintext.to_vec();
    }
    let session = crate::ocean_pipeline::primitive_session_anchor(master_key);
    let container = FracDimContainer::standard();
    let mut buf = encrypt_with_ocean(
        plaintext,
        master_key,
        &session,
        container,
        |p| match control.mode {
            FateMode::Neural => neural::encrypt_body(p, master_key),
            FateMode::Parametric => parametric::encrypt_entropy_body(p, master_key),
            FateMode::NeuralThenParametric => {
                let tmp = neural::encrypt_body(p, master_key);
                parametric::encrypt_entropy_body(&tmp, master_key)
            }
            FateMode::ParametricThenNeural => {
                let tmp = parametric::encrypt_entropy_body(p, master_key);
                neural::encrypt_body(&tmp, master_key)
            }
        },
    );
    apply_fate_cascade(&mut buf, master_key, control);
    buf
}

/// Fate-controlled experimental decrypt: inverse of `fate_neural_encrypt`.
pub fn fate_neural_decrypt(
    ciphertext: &[u8],
    master_key: &[u8],
    nonce: &[u8],
    control: &FateYakshaControl,
) -> Vec<u8> {
    if ciphertext.is_empty() || master_key.is_empty() {
        return ciphertext.to_vec();
    }

    // Inverse order:
    // 1) undo Fate-controlled cascade
    let mut buf = ciphertext.to_vec();
    apply_fate_cascade_inverse(&mut buf, master_key, control);

    let container = FracDimContainer::standard();
    decrypt_with_ocean(
        &buf,
        master_key,
        nonce,
        container,
        |c| match control.mode {
            FateMode::Neural => neural::decrypt_body(c, master_key),
            FateMode::Parametric => parametric::decrypt_entropy_body(c, master_key),
            FateMode::NeuralThenParametric => {
                let tmp = parametric::decrypt_entropy_body(c, master_key);
                neural::decrypt_body(&tmp, master_key)
            }
            FateMode::ParametricThenNeural => {
                let tmp = neural::decrypt_body(c, master_key);
                parametric::decrypt_entropy_body(&tmp, master_key)
            }
        },
    )
}

// --------------------------------------------------------------------------
// Internal helpers: Fate-controlled cascade
// --------------------------------------------------------------------------

fn apply_fate_cascade(buf: &mut [u8], key: &[u8], control: &FateYakshaControl) {
    use crate::cascade;

    let breaths = control.breaths.max(1);

    match control.cascade_pattern {
        CascadePattern::ForwardFeistel => {
            for _ in 0..breaths {
                cascade::cascade_encrypt(buf, key);
            }
        }
        CascadePattern::Bidirectional => {
            for _ in 0..breaths {
                cascade::cascade_encrypt(buf, key);
                cascade::cascade_decrypt(buf, key);
            }
        }
        CascadePattern::MultiRound(extra) => {
            let total = breaths.saturating_add(extra).max(2);
            for _ in 0..total {
                cascade::cascade_encrypt(buf, key);
            }
        }
    }
}

fn apply_fate_cascade_inverse(buf: &mut [u8], key: &[u8], control: &FateYakshaControl) {
    use crate::cascade;

    let breaths = control.breaths.max(1);

    match control.cascade_pattern {
        CascadePattern::ForwardFeistel => {
            for _ in 0..breaths {
                cascade::cascade_decrypt(buf, key);
            }
        }
        CascadePattern::Bidirectional => {
            for _ in 0..breaths {
                // Inverse of (encrypt → decrypt) is (encrypt → decrypt) again
                cascade::cascade_encrypt(buf, key);
                cascade::cascade_decrypt(buf, key);
            }
        }
        CascadePattern::MultiRound(extra) => {
            let total = breaths.saturating_add(extra).max(2);
            for _ in 0..total {
                cascade::cascade_decrypt(buf, key);
            }
        }
    }
}

