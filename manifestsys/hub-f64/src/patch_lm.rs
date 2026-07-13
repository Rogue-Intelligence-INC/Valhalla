//! Tier A patch LM — Qwen2 via candle + legacy Valhalla weight surgery.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use candle_core::{safetensors::load as load_safetensors, DType, Device, Tensor};
use candle_nn::{Activation, VarBuilder};
use candle_transformers::generation::LogitsProcessor;
use candle_transformers::models::qwen2::{Config, ModelForCausalLM};
use tokenizers::Tokenizer;

use crate::fair_score::{format_prompt_text, score_answer, FairPrompt};
use crate::grinder::{default_port_cap, grind_text_hash};
use crate::hybrid::PatchTrackRow;
use crate::patch_export::{self, PATCH_ANCHOR};
use crate::pipeline::{self, pick_mcq_fate_letter, pick_mcq_affinity_letter};
use crate::pipeline::PipelineConfig;
use crate::mcq_fate::McqFateOutcome;
use crate::signal_ingress::{TriadIncubationSnapshot, ValhallaModelPatch};

pub const DEFAULT_STRENGTH: f64 = 0.08;
pub const DEFAULT_MAX_NEW: usize = 192;

const MCQ_LETTERS: [char; 4] = ['A', 'B', 'C', 'D'];

/// How MCQ answers are chosen inside the patched LM (no external memory decode).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum McqDecodeMode {
    /// Generate + Valhalla affinity/fate vote (legacy hybrid patch path).
    #[default]
    GenerateResolve,
    /// Teacher-forced logprob on `ANSWER: X` — GPU prefill + suffix decode (4 fwd/option).
    LogprobBatch,
}

#[derive(Debug, Clone)]
pub struct PatchLmConfig {
    pub model_dir: PathBuf,
    pub patch_body: String,
    pub strength: f64,
    pub max_new: usize,
    pub export_cycles: usize,
    pub use_cpu: bool,
    pub pipeline_v4: bool,
    pub mcq_decode: McqDecodeMode,
    /// When `None`, auto: false if `valhalla_manifest.json` exists in model_dir.
    pub runtime_patch: Option<bool>,
}

impl PatchLmConfig {
    pub fn effective_runtime_patch(&self, model_dir: &Path) -> bool {
        self.runtime_patch.unwrap_or_else(|| {
            !crate::model_export::ValhallaBakedManifest::is_baked_checkpoint(model_dir)
        })
    }
}

impl Default for PatchLmConfig {
    fn default() -> Self {
        Self {
            model_dir: default_model_dir(),
            patch_body: "triad".into(),
            strength: DEFAULT_STRENGTH,
            max_new: DEFAULT_MAX_NEW,
            export_cycles: 1,
            use_cpu: true,
            pipeline_v4: true,
            mcq_decode: McqDecodeMode::GenerateResolve,
            runtime_patch: None,
        }
    }
}

pub fn default_model_dir() -> PathBuf {
    if let Ok(p) = std::env::var("VALHALLA_QWEN_MODEL_DIR") {
        return PathBuf::from(p);
    }
    PathBuf::from(
        "/home/jouly/.cache/huggingface/hub/models--Qwen--Qwen2.5-0.5B-Instruct/snapshots/7ae557604adf67be50417f59c2c2f167def9a775",
    )
}

pub fn resolve_model_dir(model: &str) -> PathBuf {
    let p = PathBuf::from(model);
    if p.exists() && snapshot_has_weights(&p) {
        return p;
    }
    if model.contains('/') {
        let slug = model.replace('/', "--");
        let base =
            PathBuf::from("/home/jouly/.cache/huggingface/hub").join(format!("models--{slug}"));
        if base.exists() {
            if let Ok(entries) = std::fs::read_dir(base.join("snapshots")) {
                let mut snaps: Vec<PathBuf> = entries
                    .flatten()
                    .map(|e| e.path())
                    .filter(|snap| snapshot_has_weights(snap))
                    .collect();
                snaps.sort();
                if let Some(snap) = snaps.pop() {
                    return snap;
                }
            }
        }
    }
    default_model_dir()
}

fn snapshot_has_weights(snap: &Path) -> bool {
    snap.join("model.safetensors").is_file()
        || snap.join("model.safetensors.index.json").is_file()
}

fn load_safetensors_from_dir(
    model_dir: &Path,
    device: &Device,
    dtype: DType,
) -> Result<HashMap<String, Tensor>, String> {
    let single = model_dir.join("model.safetensors");
    if single.is_file() {
        let len = std::fs::metadata(&single).map(|m| m.len()).unwrap_or(0);
        if len > 1024 {
            let loaded = load_safetensors(&single, device).map_err(|e| e.to_string())?;
            return tensors_to_dtype(loaded, dtype);
        }
    }
    let index_path = model_dir.join("model.safetensors.index.json");
    if !index_path.is_file() {
        return Err(format!(
            "no model.safetensors or model.safetensors.index.json under {}",
            model_dir.display()
        ));
    }
    let raw = std::fs::read_to_string(&index_path).map_err(|e| e.to_string())?;
    let v: serde_json::Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let weight_map = v
        .get("weight_map")
        .and_then(|w| w.as_object())
        .ok_or("missing weight_map in model.safetensors.index.json")?;
    let mut shard_names = HashSet::new();
    for name in weight_map.values() {
        if let Some(s) = name.as_str() {
            shard_names.insert(s.to_string());
        }
    }
    let mut tensors = HashMap::new();
    for shard in shard_names {
        let path = model_dir.join(&shard);
        if !path.is_file() {
            return Err(format!("missing weight shard: {}", path.display()));
        }
        let loaded = load_safetensors(&path, device).map_err(|e| e.to_string())?;
        for (k, t) in loaded {
            tensors.insert(k, t.to_dtype(dtype).map_err(|e| e.to_string())?);
        }
    }
    Ok(tensors)
}

fn env_bool(key: &str, default: bool) -> bool {
    std::env::var(key)
        .ok()
        .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(default)
}

/// CUDA inference dtype. Default F32 (candle matmul-safe on 4090).
/// Set `VALHALLA_LM_F16=1` for half weights; `VALHALLA_LM_BF16=1` if your candle build supports it.
fn lm_inference_dtype(use_cpu: bool) -> DType {
    if use_cpu {
        return DType::F32;
    }
    if env_bool("VALHALLA_LM_BF16", false) {
        DType::BF16
    } else if env_bool("VALHALLA_LM_F16", false) {
        DType::F16
    } else {
        DType::F32
    }
}

fn patchable_tensor_keys(n_layers: usize) -> Vec<String> {
    let mut keys = vec!["model.embed_tokens.weight".to_string()];
    for li in 0..n_layers {
        keys.push(format!("model.layers.{li}.input_layernorm.weight"));
    }
    keys
}

/// Reset only Valhalla patch targets (layernorm scales + embed rows) from base weights.
fn reset_patchable_tensors(
    base: &BaseWeights,
    scratch: &mut HashMap<String, Tensor>,
    n_layers: usize,
) -> Result<(), String> {
    for key in patchable_tensor_keys(n_layers) {
        if let Some(src) = base.tensors.get(&key) {
            scratch.insert(key, src.copy().map_err(|e| e.to_string())?);
        }
    }
    Ok(())
}

/// Mutable tensor workspace — one full clone per model, per-question patchable reset only.
struct PatchTensorScratch {
    tensors: HashMap<String, Tensor>,
}

impl PatchTensorScratch {
    fn new(base: &BaseWeights) -> Result<Self, String> {
        Ok(Self {
            tensors: base.clone_tensors()?,
        })
    }

    fn reset_and_patch(
        &mut self,
        base: &BaseWeights,
        n_layers: usize,
        patch: impl FnOnce(&mut HashMap<String, Tensor>) -> Result<(), String>,
    ) -> Result<ModelForCausalLM, String> {
        reset_patchable_tensors(base, &mut self.tensors, n_layers)?;
        patch(&mut self.tensors)?;
        base.build_model(self.tensors.clone())
    }
}
fn tensors_to_dtype(
    loaded: HashMap<String, Tensor>,
    dtype: DType,
) -> Result<HashMap<String, Tensor>, String> {
    loaded
        .into_iter()
        .map(|(k, t)| t.to_dtype(dtype).map(|c| (k, c)).map_err(|e| e.to_string()))
        .collect()
}

pub fn project_patch(patch_vec: &[f64], hidden: usize) -> Vec<f32> {
    let mut out = vec![0.0f32; hidden];
    if patch_vec.is_empty() {
        return out;
    }
    let n = patch_vec.len().min(hidden);
    for (i, &v) in patch_vec.iter().take(n).enumerate() {
        out[i] = v as f32;
    }
    for i in n..hidden {
        out[i] = patch_vec[i % patch_vec.len()] as f32;
    }
    let norm: f32 = out.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-9);
    out.iter_mut().for_each(|x| *x /= norm);
    out
}

fn load_hf_config(path: &Path) -> Result<Config, String> {
    let raw = std::fs::read_to_string(path.join("config.json")).map_err(|e| e.to_string())?;
    let v: serde_json::Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let rope_theta = v
        .pointer("/rope_parameters/rope_theta")
        .or_else(|| v.get("rope_theta"))
        .and_then(|x| x.as_f64())
        .unwrap_or(1_000_000.0);
    Ok(Config {
        vocab_size: v["vocab_size"].as_u64().unwrap_or(151936) as usize,
        hidden_size: v["hidden_size"].as_u64().unwrap_or(896) as usize,
        intermediate_size: v["intermediate_size"].as_u64().unwrap_or(4864) as usize,
        num_hidden_layers: v["num_hidden_layers"].as_u64().unwrap_or(24) as usize,
        num_attention_heads: v["num_attention_heads"].as_u64().unwrap_or(14) as usize,
        num_key_value_heads: v["num_key_value_heads"].as_u64().unwrap_or(2) as usize,
        max_position_embeddings: v["max_position_embeddings"].as_u64().unwrap_or(32768) as usize,
        sliding_window: v
            .get("sliding_window")
            .and_then(|x| x.as_u64())
            .unwrap_or(32768) as usize,
        max_window_layers: v
            .get("max_window_layers")
            .and_then(|x| x.as_u64())
            .unwrap_or(21) as usize,
        tie_word_embeddings: v
            .get("tie_word_embeddings")
            .and_then(|x| x.as_bool())
            .unwrap_or(true),
        rope_theta,
        rms_norm_eps: v["rms_norm_eps"].as_f64().unwrap_or(1e-6),
        use_sliding_window: v
            .get("use_sliding_window")
            .and_then(|x| x.as_bool())
            .unwrap_or(false),
        hidden_act: Activation::Silu,
    })
}

struct BaseWeights {
    cfg: Config,
    tensors: HashMap<String, Tensor>,
    device: Device,
    dtype: DType,
}

#[derive(Clone, Hash, PartialEq, Eq)]
struct LmCacheKey {
    model_dir: PathBuf,
    use_cpu: bool,
    force_f32: bool,
}

static LM_WEIGHT_CACHE: OnceLock<Mutex<HashMap<LmCacheKey, Arc<BaseWeights>>>> = OnceLock::new();
static TOKENIZER_CACHE: OnceLock<Mutex<HashMap<PathBuf, Arc<Tokenizer>>>> = OnceLock::new();

fn lm_cache_max_entries() -> usize {
    std::env::var("VALHALLA_LM_CACHE_MAX")
        .ok()
        .and_then(|v| v.parse().ok())
        .filter(|&n| n > 0)
        .unwrap_or(1)
}

/// Drop cached LM weights/tokenizers (HTTP `/v1/release` or between backbone switches).
pub fn clear_lm_caches() {
    if let Some(cache) = LM_WEIGHT_CACHE.get() {
        if let Ok(mut guard) = cache.lock() {
            guard.clear();
            guard.shrink_to_fit();
        }
    }
    if let Some(cache) = TOKENIZER_CACHE.get() {
        if let Ok(mut guard) = cache.lock() {
            guard.clear();
            guard.shrink_to_fit();
        }
    }
}

fn release_gpu_workspace(device: &Device) {
    if device.is_cuda() {
        let _ = device.synchronize();
    }
}

fn load_base_weights(model_dir: &Path, use_cpu: bool) -> Result<Arc<BaseWeights>, String> {
    // Serialize CUDA weight load across rayon workers (VRAM / driver safety).
    crate::elastic_resources::with_cuda_infer_lock(|| {
        let key = LmCacheKey {
            model_dir: model_dir.to_path_buf(),
            use_cpu,
            force_f32: !env_bool("VALHALLA_LM_F16", false) && !env_bool("VALHALLA_LM_BF16", false),
        };
        let cache = LM_WEIGHT_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
        let mut guard = cache.lock().map_err(|e| e.to_string())?;
        if let Some(hit) = guard.get(&key) {
            return Ok(Arc::clone(hit));
        }
        let max = lm_cache_max_entries();
        if max <= 1 {
            guard.clear();
        } else {
            while guard.len() >= max {
                if let Some(old) = guard.keys().next().cloned() {
                    guard.remove(&old);
                } else {
                    break;
                }
            }
        }
        eprintln!(
            "patch_lm: loading weights {} (cpu={} dtype={:?})",
            model_dir.display(),
            use_cpu,
            lm_inference_dtype(use_cpu)
        );
        let base = Arc::new(BaseWeights::load(model_dir, use_cpu)?);
        guard.insert(key, Arc::clone(&base));
        Ok(base)
    })
}

fn load_tokenizer(model_dir: &Path) -> Result<Arc<Tokenizer>, String> {
    let key = model_dir.to_path_buf();
    let cache = TOKENIZER_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = cache.lock().map_err(|e| e.to_string())?;
    if let Some(hit) = guard.get(&key) {
        return Ok(Arc::clone(hit));
    }
    let tok = Arc::new(
        Tokenizer::from_file(model_dir.join("tokenizer.json")).map_err(|e| e.to_string())?,
    );
    guard.insert(key, Arc::clone(&tok));
    Ok(tok)
}

impl BaseWeights {
    fn load(model_dir: &Path, use_cpu: bool) -> Result<Self, String> {
        crate::elastic_resources::ElasticResourcePlan::ensure_cuda_runtime_env();
        let device = if use_cpu {
            Device::Cpu
        } else if candle_core::utils::cuda_is_available() {
            let idx = crate::elastic_resources::cuda_device_index();
            Device::new_cuda(idx).map_err(|e| {
                format!("CUDA device {idx} unavailable ({e}); set VALHALLA_USE_CPU=1 or VALHALLA_CUDA_DEVICE")
            })?
        } else {
            Device::Cpu
        };
        let dtype = lm_inference_dtype(use_cpu || !device.is_cuda());
        let tensors = load_safetensors_from_dir(model_dir, &device, dtype)?;
        let cfg = load_hf_config(model_dir)?;
        if device.is_cuda() {
            eprintln!(
                "patch_lm: CUDA device={} dtype={:?} model={}",
                crate::elastic_resources::cuda_device_index(),
                dtype,
                model_dir.display()
            );
        }
        Ok(Self {
            cfg,
            tensors,
            device,
            dtype,
        })
    }

    fn clone_tensors(&self) -> Result<HashMap<String, Tensor>, String> {
        self.tensors
            .iter()
            .map(|(k, t)| t.copy().map(|c| (k.clone(), c)).map_err(|e| e.to_string()))
            .collect()
    }

    fn build_model(&self, tensors: HashMap<String, Tensor>) -> Result<ModelForCausalLM, String> {
        let vb = VarBuilder::from_tensors(tensors, self.dtype, &self.device);
        ModelForCausalLM::new(&self.cfg, vb).map_err(|e| e.to_string())
    }
}

fn apply_legacy_patch_tensors(
    tensors: &mut HashMap<String, Tensor>,
    mp: &ValhallaModelPatch,
    strength: f64,
    token_ids: &[u32],
    hidden: usize,
    n_layers: usize,
) -> Result<(), String> {
    let scales = mp.quad_layer_scales;
    let patch_proj = project_patch(&mp.patch_vector, hidden);
    let group_size = (n_layers / 4).max(1);

    for li in 0..n_layers {
        let key = format!("model.layers.{li}.input_layernorm.weight");
        let qi = (li / group_size).min(3);
        let scale = 1.0 + strength * (scales[qi] - 1.0);
        if let Some(t) = tensors.get_mut(&key) {
            *t = t
                .affine(scale, 0.0)
                .map_err(|e| e.to_string())?;
        }
    }

    let emb_key = "model.embed_tokens.weight".to_string();
    if let Some(emb) = tensors.get_mut(&emb_key) {
        let emb_dtype = emb.dtype();
        let mut rows = emb
            .to_dtype(DType::F32)
            .map_err(|e| e.to_string())?
            .to_vec2::<f32>()
            .map_err(|e| e.to_string())?;
        for &tid in token_ids {
            let idx = tid as usize;
            if idx >= rows.len() {
                continue;
            }
            for (j, d) in patch_proj.iter().enumerate() {
                if j < rows[idx].len() {
                    rows[idx][j] += strength as f32 * d;
                }
            }
        }
        let flat: Vec<f32> = rows.into_iter().flatten().collect();
        let vocab = emb.dim(0).map_err(|e| e.to_string())?;
        let patched = Tensor::from_vec(flat, (vocab, hidden), emb.device())
            .map_err(|e| e.to_string())?
            .to_dtype(emb_dtype)
            .map_err(|e| e.to_string())?;
        *emb = patched;
    }
    Ok(())
}

/// Boost embed rows for MCQ letters using affinity + Fate option energies.
fn apply_mcq_patch_bias(
    tensors: &mut HashMap<String, Tensor>,
    tokenizer: &Tokenizer,
    affinity: &[(char, f64)],
    option_scores: &[(char, f64)],
    strength: f64,
    hidden: usize,
) -> Result<(), String> {
    let emb_key = "model.embed_tokens.weight";
    let Some(emb) = tensors.get_mut(emb_key) else {
        return Ok(());
    };
    let scale = std::env::var("VALHALLA_MCQ_PATCH_BIAS_SCALE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.35)
        * strength;

    let mut letter_weight: HashMap<char, f64> = HashMap::new();
    if !affinity.is_empty() {
        let mean = affinity.iter().map(|(_, s)| *s).sum::<f64>() / affinity.len() as f64;
        for (letter, s) in affinity {
            *letter_weight.entry(*letter).or_default() += (s - mean).max(0.0);
        }
    }
    if option_scores.len() >= 2 {
        let mean =
            option_scores.iter().map(|(_, s)| *s).sum::<f64>() / option_scores.len() as f64;
        for (letter, s) in option_scores {
            *letter_weight.entry(*letter).or_default() += 0.85 * (s - mean).max(0.0);
        }
    }
    if letter_weight.is_empty() {
        return Ok(());
    }

    let mut rows = emb
        .to_dtype(DType::F32)
        .map_err(|e| e.to_string())?
        .to_vec2::<f32>()
        .map_err(|e| e.to_string())?;
    for (letter, w) in letter_weight {
        if w <= 0.0 {
            continue;
        }
        let patterns = [
            letter.to_string(),
            format!("{letter})"),
            format!("ANSWER: {letter}"),
        ];
        let mut token_ids = HashSet::new();
        for pat in patterns {
            if let Ok(enc) = tokenizer.encode(pat, false) {
                for &tid in enc.get_ids() {
                    token_ids.insert(tid);
                }
            }
        }
        let delta_mag = (w * scale) as f32;
        for tid in token_ids {
            let idx = tid as usize;
            if idx >= rows.len() {
                continue;
            }
            for j in 0..hidden.min(rows[idx].len()) {
                rows[idx][j] += delta_mag * ((j as f32 % 7.0) - 3.0) * 0.01;
            }
        }
    }

    let flat: Vec<f32> = rows.into_iter().flatten().collect();
    let vocab = emb.dim(0).map_err(|e| e.to_string())?;
    let emb_dtype = emb.dtype();
    *emb = Tensor::from_vec(flat, (vocab, hidden), emb.device())
        .map_err(|e| e.to_string())?
        .to_dtype(emb_dtype)
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Resolve patch MCQ answer: Valhalla latent signals + LM generate (no blind trust of wrong gen).
fn resolve_patch_mcq_answer(
    generated: &str,
    affinity: &[(char, f64)],
    mcq_fate: Option<&McqFateOutcome>,
) -> (String, String, f64) {
    use crate::fair_score::extract_mcq_letter;

    let gen_l = extract_mcq_letter(generated);
    let aff_l = pick_mcq_affinity_letter(affinity);
    let fate_l = mcq_fate.and_then(|o| pick_mcq_fate_letter(&o.option_scores));
    let aff_spread = pipeline::mcq_affinity_spread(affinity);
    let fate_spread = mcq_fate
        .map(|o| pipeline::mcq_fate_spread(&o.option_scores))
        .unwrap_or(0.0);

    let fate_min = pipeline::mcq_fate_min_spread();

    let mut votes: HashMap<char, f64> = HashMap::new();
    if let Some(c) = gen_l {
        *votes.entry(c).or_default() += 1.0;
    }
    if let Some(c) = aff_l {
        let w = if aff_spread >= pipeline::MCQ_AFFINITY_MIN_SPREAD {
            1.15
        } else {
            0.35
        };
        *votes.entry(c).or_default() += w;
    }
    if let Some(c) = fate_l {
        let w = if fate_spread >= fate_min {
            1.25
        } else {
            0.2
        };
        *votes.entry(c).or_default() += w;
    }

    let pick = votes
        .iter()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(c, _)| *c);
    let confidence = pick
        .and_then(|c| votes.get(&c).copied())
        .unwrap_or(0.0);

    if let Some(c) = pick {
        let tag = if gen_l == Some(c) && fate_l == Some(c) && fate_spread >= fate_min {
            "patch_mcq_gen_fate"
        } else if fate_l == Some(c) && fate_spread >= fate_min && gen_l != Some(c) {
            "patch_mcq_fate"
        } else if gen_l == Some(c) && aff_l == Some(c) && aff_spread >= pipeline::MCQ_AFFINITY_MIN_SPREAD {
            "patch_mcq_blend"
        } else if gen_l == Some(c) {
            "patch_mcq_gen_solo"
        } else if aff_l == Some(c) {
            if aff_spread >= pipeline::MCQ_AFFINITY_MIN_SPREAD {
                "patch_mcq_affinity"
            } else {
                "patch_mcq_affinity_weak"
            }
        } else {
            "patch_mcq_blend"
        };
        if gen_l == Some(c) && generated.to_uppercase().contains("ANSWER:") {
            return (generated.trim().to_string(), tag.into(), confidence);
        }
        return (format!("ANSWER: {c}"), tag.into(), confidence);
    }

    if gen_l.is_some() {
        return (
            generated.to_string(),
            "patch_mcq_gen_solo".into(),
            confidence,
        );
    }
    (
        generated.to_string(),
        "patch_mcq_gen_solo".into(),
        confidence,
    )
}

fn logprob_of_token(logits: &Tensor, token_id: u32) -> Result<f64, String> {
    let logits = logits
        .to_dtype(DType::F32)
        .map_err(|e| e.to_string())?;
    let logits_vec = logits.to_vec1::<f32>().map_err(|e| e.to_string())?;
    if logits_vec.is_empty() {
        return Ok(f64::NEG_INFINITY);
    }
    let max = logits_vec
        .iter()
        .cloned()
        .fold(f32::NEG_INFINITY, f32::max);
    let sum_exp: f32 = logits_vec.iter().map(|x| (x - max).exp()).sum();
    let log_sum = (sum_exp.max(1e-30)).ln() + max;
    let tok = token_id as usize;
    if tok >= logits_vec.len() {
        return Ok(f64::NEG_INFINITY);
    }
    Ok((logits_vec[tok] - log_sum) as f64)
}

fn token_input(device: &Device, tokens: &[u32]) -> Result<Tensor, String> {
    Tensor::new(tokens, device)
        .map_err(|e| e.to_string())?
        .unsqueeze(0)
        .map_err(|e| e.to_string())
}

fn squeeze_last_logits(logits: &Tensor) -> Result<Tensor, String> {
    logits
        .squeeze(0)
        .map_err(|e| e.to_string())?
        .squeeze(0)
        .map_err(|e| e.to_string())
}

/// Sum log P(token[i] | token[0..i]) for i in `score_from`..len.
/// GPU path: one prefill forward for the prefix, then one forward per suffix token.
fn sequence_logprob_from(
    model: &mut ModelForCausalLM,
    device: &Device,
    tokens: &[u32],
    score_from: usize,
) -> Result<f64, String> {
    if tokens.len() <= score_from || tokens.len() < 2 {
        return Ok(f64::NEG_INFINITY);
    }
    model.clear_kv_cache();
    let mut total = 0.0f64;

    let prefill_len = score_from.max(1);
    let prefill = &tokens[0..prefill_len];
    let input = token_input(device, prefill)?;
    let logits = model.forward(&input, 0).map_err(|e| e.to_string())?;
    let logits = squeeze_last_logits(&logits)?;

    if score_from >= 1 && tokens.len() > score_from {
        total += logprob_of_token(&logits, tokens[score_from])?;
    } else if score_from == 0 {
        total += logprob_of_token(&logits, tokens[1])?;
    }

    for i in (score_from + 1).max(2)..tokens.len() {
        let input = token_input(device, &tokens[i - 1..i])?;
        let logits = model
            .forward(&input, i - 1)
            .map_err(|e| e.to_string())?;
        let logits = squeeze_last_logits(&logits)?;
        total += logprob_of_token(&logits, tokens[i])?;
    }
    Ok(total)
}

/// Score `ANSWER: X` suffix for each MCQ letter; pick argmax logprob (single LM, no router).
pub fn decode_mcq_logprob_batch(
    model: &mut ModelForCausalLM,
    device: &Device,
    tokenizer: &Tokenizer,
    raw_prompt: &str,
) -> Result<(char, HashMap<char, f64>), String> {
    let prompt_tokens = tokenizer
        .encode(raw_prompt, true)
        .map_err(|e| e.to_string())?
        .get_ids()
        .to_vec();
    let score_from = prompt_tokens.len();

    let mut scores: HashMap<char, f64> = HashMap::new();
    let mut best_l = 'A';
    let mut best_lp = f64::NEG_INFINITY;

    for &letter in &MCQ_LETTERS {
        let suffix = format!("ANSWER: {letter}");
        let full = format!("{raw_prompt}{suffix}");
        let full_tokens = tokenizer
            .encode(full.as_str(), true)
            .map_err(|e| e.to_string())?
            .get_ids()
            .to_vec();
        if full_tokens.len() <= score_from {
            scores.insert(letter, f64::NEG_INFINITY);
            continue;
        }
        let lp = sequence_logprob_from(model, device, &full_tokens, score_from)?;
        scores.insert(letter, lp);
        if lp > best_lp {
            best_lp = lp;
            best_l = letter;
        }
    }

    Ok((best_l, scores))
}

/// MCQ compose: LM logprob + Valhalla shard affinity prior (no external memory).
pub fn decode_mcq_logprob_compose(
    model: &mut ModelForCausalLM,
    device: &Device,
    tokenizer: &Tokenizer,
    raw_prompt: &str,
    affinity: &[(char, f64)],
    cross_modal: f64,
) -> Result<(char, HashMap<char, f64>), String> {
    let (_, mut scores) = decode_mcq_logprob_batch(model, device, tokenizer, raw_prompt)?;
    if affinity.is_empty() {
        return Ok((
            scores
                .iter()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(c, _)| *c)
                .unwrap_or('A'),
            scores,
        ));
    }
    let bias_scale = std::env::var("VALHALLA_COMPOSE_MCQ_BIAS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.55);
    let gate = 0.5 + (cross_modal - 0.5) * 0.85;
    let mean = affinity.iter().map(|(_, s)| *s).sum::<f64>() / affinity.len() as f64;
    let mut best_l = 'A';
    let mut best_lp = f64::NEG_INFINITY;
    for letter in MCQ_LETTERS {
        let lp = scores.get(&letter).copied().unwrap_or(f64::NEG_INFINITY);
        let aff = affinity
            .iter()
            .find(|(c, _)| *c == letter)
            .map(|(_, s)| *s - mean)
            .unwrap_or(0.0);
        let biased = lp + aff * bias_scale * gate;
        scores.insert(letter, biased);
        if biased > best_lp {
            best_lp = biased;
            best_l = letter;
        }
    }
    Ok((best_l, scores))
}

/// Build LM from baked base + Triad `model_patch` after Stem/Tile/Hub shard recompose.
fn build_compose_recompose_model(
    base: &BaseWeights,
    scratch: &mut PatchTensorScratch,
    tokenizer: &Tokenizer,
    user_text: &str,
    recompose: &crate::task_decompose::RecomposePatch,
    cfg: &PatchLmConfig,
    score_type: &str,
) -> Result<ModelForCausalLM, String> {
    let hidden = base.cfg.hidden_size;
    let n_layers = base.cfg.num_hidden_layers;
    let mut token_ids = tokenizer
        .encode(PATCH_ANCHOR, false)
        .map_err(|e| e.to_string())?
        .get_ids()
        .to_vec();
    if let Ok(enc) = tokenizer.encode(user_text, false) {
        token_ids.extend(enc.get_ids().iter().copied());
    }
    let strength =
        crate::task_decompose::compose_patch_strength(cfg.strength, recompose, score_type);
    let mcq_affinity = recompose.mcq_option_affinity.clone();
    let score_type = score_type.to_string();
    scratch.reset_and_patch(base, n_layers, |tensors| {
        apply_legacy_patch_tensors(
            tensors,
            &recompose.model_patch,
            strength,
            &token_ids,
            hidden,
            n_layers,
        )?;
        if score_type == "mcq" && !mcq_affinity.is_empty() {
            apply_mcq_patch_bias(
                tensors,
                tokenizer,
                &mcq_affinity,
                &[],
                strength,
                hidden,
            )?;
        }
        Ok(())
    })
}

fn generate(
    model: &mut ModelForCausalLM,
    device: &Device,
    tokenizer: &Tokenizer,
    prompt: &str,
    max_new: usize,
    eos: u32,
    im_end: u32,
) -> Result<String, String> {
    let mut tokens = tokenizer
        .encode(prompt, true)
        .map_err(|e| e.to_string())?
        .get_ids()
        .to_vec();
    let mut logits_processor = LogitsProcessor::new(299792458, None, None);
    model.clear_kv_cache();
    let prompt_len = tokens.len();

    for gen_i in 0..max_new {
        let (input_slice, start_pos) = if gen_i == 0 && prompt_len > 0 {
            (&tokens[..], 0usize)
        } else {
            let pos = tokens.len().saturating_sub(1);
            (&tokens[pos..], pos)
        };
        let input = token_input(device, input_slice)?;
        let logits = model.forward(&input, start_pos).map_err(|e| e.to_string())?;
        let logits = squeeze_last_logits(&logits)?;
        let next = logits_processor.sample(&logits).map_err(|e| e.to_string())?;
        tokens.push(next);
        if next == eos || next == im_end {
            break;
        }
    }

    let decoded = tokenizer
        .decode(&tokens, true)
        .map_err(|e| e.to_string())?;
    Ok(extract_assistant(&decoded, prompt))
}

pub fn format_chat_prompt(user_text: &str) -> String {
    format!(
        "<|im_start|>system\nYou are Qwen, created by Alibaba Cloud. You are a helpful assistant.<|im_end|>\n<|im_start|>user\n{user_text}<|im_end|>\n<|im_start|>assistant\n"
    )
}

pub fn extract_assistant(full: &str, raw_prompt: &str) -> String {
    let mut text = full;
    if raw_prompt.is_empty() {
        return text.trim().to_string();
    }
    if let Some(rest) = text.strip_prefix(raw_prompt) {
        text = rest;
    }
    for marker in ["assistant\n", "assistant\r\n", "assistant"] {
        if let Some(pos) = text.find(marker) {
            text = &text[pos + marker.len()..];
            break;
        }
    }
    text.trim().to_string()
}

pub fn run_patch_track(
    prompts: &[FairPrompt],
    corpus: &[String],
    cfg: &PatchLmConfig,
    export_doc: Option<&patch_export::ExportDoc>,
    audio_by_id: Option<&HashMap<String, Vec<f64>>>,
) -> Result<Vec<PatchTrackRow>, String> {
    if cfg.pipeline_v4 || pipeline::pipeline_v4_enabled() {
        return run_patch_track_v4(prompts, corpus, cfg, audio_by_id);
    }
    let export = match export_doc {
        Some(d) => d.clone(),
        None => crate::patch_export::export_patch_from_corpus(
            &cfg.patch_body,
            cfg.export_cycles,
            corpus,
        ),
    };
    let mp = export.report.model_patch.clone();

    let base = load_base_weights(&cfg.model_dir, cfg.use_cpu)?;
    let tokenizer = load_tokenizer(&cfg.model_dir)?;
    let eos = tokenizer
        .token_to_id("<|endoftext|>")
        .or_else(|| tokenizer.token_to_id("<|im_end|>"))
        .unwrap_or(151645);
    let im_end = tokenizer.token_to_id("<|im_end|>").unwrap_or(151645);
    let anchor_ids = tokenizer
        .encode(PATCH_ANCHOR, false)
        .map_err(|e| e.to_string())?
        .get_ids()
        .to_vec();

    let decode_tag = format!("patch_legacy_{}", cfg.patch_body);
    let hidden = base.cfg.hidden_size;
    let n_layers = base.cfg.num_hidden_layers;
    let mut rows = Vec::with_capacity(prompts.len());

    for (i, p) in prompts.iter().enumerate() {
        eprintln!("patch_lm [{}/{}] {}", i + 1, prompts.len(), p.id);
        let mut tensors = base.clone_tensors()?;
        apply_legacy_patch_tensors(
            &mut tensors,
            &mp,
            cfg.strength,
            &anchor_ids,
            hidden,
            n_layers,
        )?;
        let mut model = base.build_model(tensors)?;
        let user_text = format_prompt_text(p);
        let raw_prompt = format_chat_prompt(&user_text);
        let assistant = generate(
            &mut model,
            &base.device,
            &tokenizer,
            &raw_prompt,
            cfg.max_new,
            eos,
            im_end,
        )?;
        let _sc = score_answer(&assistant, p);
        rows.push(PatchTrackRow {
            id: p.id.clone(),
            assistant: Some(assistant),
            native_answer: None,
            decode_path: Some(decode_tag.clone()),
            mcq_patch_subtag: None,
            mcq_patch_confidence: None,
            mcq_fate_spread: None,
            mcq_fate_letter: None,
            mcq_affinity_spread: None,
        });
    }

    Ok(rows)
}

/// v4: per-question pipeline recompose → patch tensors → LM generate.
/// When `model_dir` contains `valhalla_manifest.json`, loads pre-baked weights only (no runtime surgery).
pub fn run_patch_track_v4(
    prompts: &[FairPrompt],
    corpus: &[String],
    cfg: &PatchLmConfig,
    audio_by_id: Option<&HashMap<String, Vec<f64>>>,
) -> Result<Vec<PatchTrackRow>, String> {
    let runtime_patch = cfg.effective_runtime_patch(&cfg.model_dir);
    if !runtime_patch {
        return run_baked_track_v4(prompts, corpus, cfg, audio_by_id);
    }

    let snap = patch_export::build_grinder_incubation_snapshot(
        &cfg.patch_body,
        corpus,
        cfg.export_cycles,
    );
    run_patch_track_v4_with_snap(&snap, prompts, cfg, audio_by_id)
}

/// v4 with pre-built incubation snapshot (context + core fusion path).
pub fn run_patch_track_v4_with_snap(
    snap: &TriadIncubationSnapshot,
    prompts: &[FairPrompt],
    cfg: &PatchLmConfig,
    audio_by_id: Option<&HashMap<String, Vec<f64>>>,
) -> Result<Vec<PatchTrackRow>, String> {
    let base = load_base_weights(&cfg.model_dir, cfg.use_cpu)?;
    let tokenizer = load_tokenizer(&cfg.model_dir)?;
    let eos = tokenizer
        .token_to_id("<|endoftext|>")
        .or_else(|| tokenizer.token_to_id("<|im_end|>"))
        .unwrap_or(151645);
    let im_end = tokenizer.token_to_id("<|im_end|>").unwrap_or(151645);
    let hidden = base.cfg.hidden_size;
    let n_layers = base.cfg.num_hidden_layers;
    let cap = default_port_cap();
    let front = crate::dl_front::DlFrontConfig::default();
    let pipe_cfg = PipelineConfig::default();
    let mut rows = Vec::with_capacity(prompts.len());
    let mut patch_scratch = PatchTensorScratch::new(&base)?;

    for (i, p) in prompts.iter().enumerate() {
        eprintln!("patch_v4 [{}/{}] {}", i + 1, prompts.len(), p.id);
        let base_user_text = format_prompt_text(p);
        let q_ports = grind_text_hash(&base_user_text, cap);
        let audio = audio_by_id.and_then(|m| m.get(&p.id).map(|v| v.as_slice()));
        let outcome = pipeline::run_question_pipeline(
            &snap,
            &base_user_text,
            &p.score_type,
            &q_ports,
            &pipe_cfg,
            &front,
            "patch_pipeline_v4",
            audio,
        );
        let user_text = base_user_text.clone();

        let mut token_ids = tokenizer
            .encode(PATCH_ANCHOR, false)
            .map_err(|e| e.to_string())?
            .get_ids()
            .to_vec();
        if let Ok(enc) = tokenizer.encode(user_text.as_str(), false) {
            token_ids.extend(enc.get_ids().iter().copied());
        }

        let mp = outcome.recompose.model_patch.clone();
        let effective_strength = if !outcome.recompose.compose_protocol.is_empty() {
            crate::task_decompose::compose_patch_strength(
                cfg.strength,
                &outcome.recompose,
                &p.score_type,
            )
        } else if p.score_type == "open" {
            let boost = 1.0
                + outcome.recompose.open_structure_score * crate::multimodal::open_patch_bias_scale();
            cfg.strength * boost
        } else {
            cfg.strength
        };
        let fate_scores: Vec<(char, f64)> = outcome
            .mcq_fate
            .as_ref()
            .map(|o| o.option_scores.clone())
            .unwrap_or_default();
        let mcq_affinity = outcome.recompose.mcq_option_affinity.clone();
        let score_type = p.score_type.clone();

        let mut model = patch_scratch.reset_and_patch(&base, n_layers, |tensors| {
            apply_legacy_patch_tensors(
                tensors,
                &mp,
                effective_strength,
                &token_ids,
                hidden,
                n_layers,
            )?;
            if score_type == "mcq" {
                if !mcq_affinity.is_empty() || fate_scores.len() >= 2 {
                    apply_mcq_patch_bias(
                        tensors,
                        &tokenizer,
                        &mcq_affinity,
                        &fate_scores,
                        effective_strength,
                        hidden,
                    )?;
                }
            }
            Ok(())
        })?;
        let decode_tag = if let Some(ref cf) = outcome.compose_fate {
            crate::task_decompose::compose_fate_decode_tag(
                &p.score_type,
                cf.cross_modal_score,
                cf.adaptation_rounds,
            )
        } else if !outcome.recompose.compose_protocol.is_empty() {
            crate::task_decompose::compose_decode_tag(&p.score_type, outcome.recompose.cross_modal_score)
        } else if p.score_type == "open" && audio.is_some() {
            format!("patch_pipeline_v4_{}_multimodal", p.score_type)
        } else {
            format!("patch_pipeline_v4_{}", p.score_type)
        };

        let (assistant, patch_subtag, patch_conf) = if p.score_type == "mcq" {
            let raw_prompt = format_chat_prompt(&user_text);
            if cfg.mcq_decode == McqDecodeMode::LogprobBatch {
                let (letter, lps) = if !outcome.recompose.compose_protocol.is_empty()
                    && crate::multimodal_compose::is_structure_recompose_protocol(
                        &outcome.recompose.compose_protocol,
                    )
                {
                    decode_mcq_logprob_batch(
                        &mut model,
                        &base.device,
                        &tokenizer,
                        &raw_prompt,
                    )?
                } else if !outcome.recompose.compose_protocol.is_empty() {
                    decode_mcq_logprob_compose(
                        &mut model,
                        &base.device,
                        &tokenizer,
                        &raw_prompt,
                        &outcome.recompose.mcq_option_affinity,
                        outcome.recompose.cross_modal_score,
                    )?
                } else {
                    decode_mcq_logprob_batch(
                        &mut model,
                        &base.device,
                        &tokenizer,
                        &raw_prompt,
                    )?
                };
                let margin = {
                    let mut vals: Vec<f64> = lps.values().copied().collect();
                    vals.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
                    if vals.len() >= 2 {
                        vals[0] - vals[1]
                    } else {
                        0.0
                    }
                };
                let subtag = if !outcome.recompose.compose_protocol.is_empty()
                    && crate::multimodal_compose::is_structure_recompose_protocol(
                        &outcome.recompose.compose_protocol,
                    )
                {
                    "structure_recompose_logprob".into()
                } else if outcome.recompose.compose_protocol.is_empty() {
                    "fused_mcq_logprob".into()
                } else {
                    "fused_mcq_compose_logprob".into()
                };
                (
                    format!("ANSWER: {letter}"),
                    subtag,
                    margin,
                )
            } else {
                let generated = generate(
                    &mut model,
                    &base.device,
                    &tokenizer,
                    &raw_prompt,
                    cfg.max_new.min(96),
                    eos,
                    im_end,
                )?;
                resolve_patch_mcq_answer(
                    &generated,
                    &outcome.recompose.mcq_option_affinity,
                    outcome.mcq_fate.as_ref(),
                )
            }
        } else {
            let raw_prompt = format_chat_prompt(&user_text);
            let generated = generate(
                &mut model,
                &base.device,
                &tokenizer,
                &raw_prompt,
                cfg.max_new,
                eos,
                im_end,
            )?;
            (generated, "gen".into(), 0.0)
        };

        let (fate_spread, fate_letter, aff_spread) = if p.score_type == "mcq" {
            let fs = outcome
                .mcq_fate
                .as_ref()
                .map(|o| pipeline::mcq_fate_spread(&o.option_scores));
            let fl = outcome
                .mcq_fate
                .as_ref()
                .and_then(|o| pipeline::pick_mcq_fate_letter(&o.option_scores))
                .map(|c| c.to_string());
            let aff = pipeline::mcq_affinity_spread(&outcome.recompose.mcq_option_affinity);
            (fs, fl, Some(aff))
        } else {
            (None, None, None)
        };

        let _sc = score_answer(&assistant, p);
        rows.push(PatchTrackRow {
            id: p.id.clone(),
            assistant: Some(assistant),
            native_answer: None,
            decode_path: Some(format!("{decode_tag}/{patch_subtag}")),
            mcq_patch_subtag: if p.score_type == "mcq" {
                Some(patch_subtag)
            } else {
                None
            },
            mcq_patch_confidence: if p.score_type == "mcq" {
                Some(patch_conf)
            } else {
                None
            },
            mcq_fate_spread: fate_spread,
            mcq_fate_letter: fate_letter,
            mcq_affinity_spread: aff_spread,
        });
        drop(model);
        release_gpu_workspace(&base.device);
    }

    Ok(rows)
}

/// Inference on a pre-baked Valhalla checkpoint — no per-question weight patch.
fn run_baked_track_v4(
    prompts: &[FairPrompt],
    corpus: &[String],
    cfg: &PatchLmConfig,
    audio_by_id: Option<&HashMap<String, Vec<f64>>>,
) -> Result<Vec<PatchTrackRow>, String> {
    if let Ok(m) = crate::model_export::load_manifest(&cfg.model_dir) {
        eprintln!(
            "baked_lm: {} body={} tiles={} stem={} base={} (runtime_patch=false)",
            cfg.model_dir.display(),
            m.incubation.body,
            m.incubation.tile_count,
            m.incubation.stem_cell_count,
            m.bake.base_model_id,
        );
    } else {
        eprintln!(
            "baked_lm: {} (valhalla_manifest.json present, no runtime patch)",
            cfg.model_dir.display()
        );
    }

    let base = load_base_weights(&cfg.model_dir, cfg.use_cpu)?;
    let tokenizer = load_tokenizer(&cfg.model_dir)?;
    let eos = tokenizer
        .token_to_id("<|endoftext|>")
        .or_else(|| tokenizer.token_to_id("<|im_end|>"))
        .unwrap_or(151645);
    let im_end = tokenizer.token_to_id("<|im_end|>").unwrap_or(151645);

    let mut patch_scratch = PatchTensorScratch::new(&base)?;
    let mut default_model = base.build_model(patch_scratch.tensors.clone())?;
    let mut rows = Vec::with_capacity(prompts.len());
    let cap = default_port_cap();
    let front = crate::dl_front::DlFrontConfig::default();
    let pipe_cfg = PipelineConfig::default();
    let snap = patch_export::build_grinder_incubation_snapshot(
        &cfg.patch_body,
        corpus,
        cfg.export_cycles,
    );
    let multimodal = crate::multimodal::fusion_enabled();

    for (i, p) in prompts.iter().enumerate() {
        eprintln!("baked_lm [{}/{}] {}", i + 1, prompts.len(), p.id);
        let base_user_text = format_prompt_text(p);
        let audio = audio_by_id.and_then(|m| m.get(&p.id).map(|v| v.as_slice()));

        let compose_types = multimodal && audio.is_some() && (p.score_type == "open" || p.score_type == "mcq");
        let compose_outcome = if compose_types {
            let q_ports = grind_text_hash(&base_user_text, cap);
            Some(pipeline::run_question_pipeline(
                &snap,
                &base_user_text,
                &p.score_type,
                &q_ports,
                &pipe_cfg,
                &front,
                "baked_lm_compose",
                audio,
            ))
        } else {
            None
        };

        let structure_compose = compose_outcome.as_ref().map(|o| {
            o.compose_fate.is_some()
                || crate::multimodal_compose::is_structure_recompose_protocol(&o.recompose.compose_protocol)
        }).unwrap_or(false);

        let user_text = base_user_text.clone();
        let decode_tag = if compose_outcome.as_ref().and_then(|o| o.compose_fate.as_ref()).is_some() {
            let o = compose_outcome.as_ref().unwrap();
            let cf = o.compose_fate.as_ref().unwrap();
            crate::task_decompose::compose_fate_decode_tag(
                &p.score_type,
                cf.cross_modal_score,
                cf.adaptation_rounds,
            )
        } else if structure_compose {
            crate::task_decompose::compose_decode_tag(
                &p.score_type,
                compose_outcome.as_ref().unwrap().recompose.cross_modal_score,
            )
        } else if compose_outcome.as_ref().map(|o| !o.recompose.compose_protocol.is_empty()).unwrap_or(false) {
            format!("baked_lm_{}", p.score_type)
        } else {
            format!("baked_lm_{}", p.score_type)
        };
        let max_new = if structure_compose && p.score_type == "open" {
            let o = compose_outcome.as_ref().unwrap();
            cfg.max_new + (o.recompose.open_structure_score * 48.0).round() as usize
        } else {
            cfg.max_new
        };

        let (assistant, patch_subtag, patch_conf) = if p.score_type == "mcq" {
            let raw_prompt = format_chat_prompt(&user_text);
            if cfg.mcq_decode == McqDecodeMode::LogprobBatch {
                let (letter, lps) = if structure_compose {
                    let o = compose_outcome.as_ref().unwrap();
                    let mut model = build_compose_recompose_model(
                        &base,
                        &mut patch_scratch,
                        &tokenizer,
                        &user_text,
                        &o.recompose,
                        cfg,
                        &p.score_type,
                    )?;
                    let r = decode_mcq_logprob_batch(&mut model, &base.device, &tokenizer, &raw_prompt)?;
                    r
                } else if let Some(ref outcome) = compose_outcome {
                    if outcome.recompose.compose_protocol.is_empty() {
                        decode_mcq_logprob_batch(&mut default_model, &base.device, &tokenizer, &raw_prompt)?
                    } else {
                        decode_mcq_logprob_compose(
                            &mut default_model,
                            &base.device,
                            &tokenizer,
                            &raw_prompt,
                            &outcome.recompose.mcq_option_affinity,
                            outcome.recompose.cross_modal_score,
                        )?
                    }
                } else {
                    decode_mcq_logprob_batch(&mut default_model, &base.device, &tokenizer, &raw_prompt)?
                };
                let margin = {
                    let mut vals: Vec<f64> = lps.values().copied().collect();
                    vals.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
                    if vals.len() >= 2 {
                        vals[0] - vals[1]
                    } else {
                        0.0
                    }
                };
                let subtag = if structure_compose {
                    "structure_recompose_logprob".into()
                } else if compose_outcome
                    .as_ref()
                    .map(|o| !o.recompose.compose_protocol.is_empty())
                    .unwrap_or(false)
                {
                    "baked_mcq_compose_logprob".into()
                } else {
                    "baked_mcq_logprob".into()
                };
                (
                    format!("ANSWER: {letter}"),
                    subtag,
                    margin,
                )
            } else {
                let generated = if structure_compose {
                    let o = compose_outcome.as_ref().unwrap();
                    let mut model = build_compose_recompose_model(
                        &base,
                        &mut patch_scratch,
                        &tokenizer,
                        &user_text,
                        &o.recompose,
                        cfg,
                        &p.score_type,
                    )?;
                    generate(
                        &mut model,
                        &base.device,
                        &tokenizer,
                        &raw_prompt,
                        cfg.max_new.min(96),
                        eos,
                        im_end,
                    )?
                } else {
                    generate(
                        &mut default_model,
                        &base.device,
                        &tokenizer,
                        &raw_prompt,
                        cfg.max_new.min(96),
                        eos,
                        im_end,
                    )?
                };
                (generated, "baked_mcq_gen".into(), 0.0)
            }
        } else {
            let raw_prompt = format_chat_prompt(&user_text);
            let generated = if structure_compose {
                let o = compose_outcome.as_ref().unwrap();
                let mut model = build_compose_recompose_model(
                    &base,
                    &mut patch_scratch,
                    &tokenizer,
                    &user_text,
                    &o.recompose,
                    cfg,
                    &p.score_type,
                )?;
                generate(
                    &mut model,
                    &base.device,
                    &tokenizer,
                    &raw_prompt,
                    max_new,
                    eos,
                    im_end,
                )?
            } else {
                generate(
                    &mut default_model,
                    &base.device,
                    &tokenizer,
                    &raw_prompt,
                    max_new,
                    eos,
                    im_end,
                )?
            };
            (
                generated,
                if structure_compose {
                    "structure_recompose_gen".into()
                } else {
                    "gen".into()
                },
                0.0,
            )
        };

        let _sc = score_answer(&assistant, p);
        rows.push(PatchTrackRow {
            id: p.id.clone(),
            assistant: Some(assistant),
            native_answer: None,
            decode_path: Some(format!("{decode_tag}/{patch_subtag}")),
            mcq_patch_subtag: if p.score_type == "mcq" {
                Some(patch_subtag)
            } else {
                None
            },
            mcq_patch_confidence: if p.score_type == "mcq" {
                Some(patch_conf)
            } else {
                None
            },
            mcq_fate_spread: None,
            mcq_fate_letter: None,
            mcq_affinity_spread: None,
        });
    }

    Ok(rows)
}
