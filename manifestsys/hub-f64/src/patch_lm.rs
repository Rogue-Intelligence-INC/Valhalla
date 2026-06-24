//! Tier A patch LM — Qwen2 via candle + legacy Valhalla weight surgery.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

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
use crate::signal_ingress::ValhallaModelPatch;

pub const DEFAULT_STRENGTH: f64 = 0.08;
pub const DEFAULT_MAX_NEW: usize = 192;

#[derive(Debug, Clone)]
pub struct PatchLmConfig {
    pub model_dir: PathBuf,
    pub patch_body: String,
    pub strength: f64,
    pub max_new: usize,
    pub export_cycles: usize,
    pub use_cpu: bool,
    pub pipeline_v4: bool,
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
    if p.exists() {
        return p;
    }
    if model.contains('/') {
        let slug = model.replace('/', "--");
        let base =
            PathBuf::from("/home/jouly/.cache/huggingface/hub").join(format!("models--{slug}"));
        if base.exists() {
            if let Ok(entries) = std::fs::read_dir(base.join("snapshots")) {
                for ent in entries.flatten() {
                    let snap = ent.path();
                    if snap.join("model.safetensors").exists() {
                        return snap;
                    }
                }
            }
        }
    }
    default_model_dir()
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

impl BaseWeights {
    fn load(model_dir: &Path, use_cpu: bool) -> Result<Self, String> {
        let device = if use_cpu {
            Device::Cpu
        } else if candle_core::utils::cuda_is_available() {
            Device::new_cuda(0).map_err(|e| e.to_string())?
        } else {
            Device::Cpu
        };
        let dtype = DType::F32;
        let weights = model_dir.join("model.safetensors");
        if !weights.exists() {
            return Err(format!("missing weights: {}", weights.display()));
        }
        let loaded = load_safetensors(&weights, &device).map_err(|e| e.to_string())?;
        let mut tensors = HashMap::new();
        for (k, t) in loaded {
            tensors.insert(k, t.to_dtype(dtype).map_err(|e| e.to_string())?);
        }
        let cfg = load_hf_config(model_dir)?;
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
        let mut rows = emb.to_vec2::<f32>().map_err(|e| e.to_string())?;
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
        *emb = Tensor::from_vec(flat, (vocab, hidden), emb.device())
            .map_err(|e| e.to_string())?;
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

    let mut rows = emb.to_vec2::<f32>().map_err(|e| e.to_string())?;
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
    *emb = Tensor::from_vec(flat, (vocab, hidden), emb.device()).map_err(|e| e.to_string())?;
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

    for index in 0..max_new {
        let context_size = if index > 0 { 1 } else { tokens.len() };
        let start_pos = tokens.len().saturating_sub(context_size);
        let ctxt = &tokens[start_pos..];
        let input = Tensor::new(ctxt, device)
            .map_err(|e| e.to_string())?
            .unsqueeze(0)
            .map_err(|e| e.to_string())?;
        let logits = model.forward(&input, start_pos).map_err(|e| e.to_string())?;
        let logits = logits
            .squeeze(0)
            .map_err(|e| e.to_string())?
            .squeeze(0)
            .map_err(|e| e.to_string())?;
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

    let base = BaseWeights::load(&cfg.model_dir, cfg.use_cpu)?;
    let tokenizer = Tokenizer::from_file(cfg.model_dir.join("tokenizer.json"))
        .map_err(|e| e.to_string())?;
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
pub fn run_patch_track_v4(
    prompts: &[FairPrompt],
    corpus: &[String],
    cfg: &PatchLmConfig,
    audio_by_id: Option<&HashMap<String, Vec<f64>>>,
) -> Result<Vec<PatchTrackRow>, String> {
    let snap = patch_export::build_grinder_incubation_snapshot(
        &cfg.patch_body,
        corpus,
        cfg.export_cycles,
    );
    let base = BaseWeights::load(&cfg.model_dir, cfg.use_cpu)?;
    let tokenizer = Tokenizer::from_file(cfg.model_dir.join("tokenizer.json"))
        .map_err(|e| e.to_string())?;
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

    for (i, p) in prompts.iter().enumerate() {
        eprintln!("patch_v4 [{}/{}] {}", i + 1, prompts.len(), p.id);
        let user_text = format_prompt_text(p);
        let q_ports = grind_text_hash(&user_text, cap);
        let audio = audio_by_id.and_then(|m| m.get(&p.id).map(|v| v.as_slice()));
        let outcome = pipeline::run_question_pipeline(
            &snap,
            &user_text,
            &p.score_type,
            &q_ports,
            &pipe_cfg,
            &front,
            "patch_pipeline_v4",
            audio,
        );

        let mut token_ids = tokenizer
            .encode(PATCH_ANCHOR, false)
            .map_err(|e| e.to_string())?
            .get_ids()
            .to_vec();
        if let Ok(enc) = tokenizer.encode(user_text.as_str(), false) {
            token_ids.extend(enc.get_ids().iter().copied());
        }

        let mut tensors = base.clone_tensors()?;
        let mp = &outcome.recompose.model_patch;
        let effective_strength = if p.score_type == "open" {
            let boost = 1.0
                + outcome.recompose.open_structure_score * crate::multimodal::open_patch_bias_scale();
            cfg.strength * boost
        } else {
            cfg.strength
        };
        apply_legacy_patch_tensors(
            &mut tensors,
            mp,
            effective_strength,
            &token_ids,
            hidden,
            n_layers,
        )?;

        if p.score_type == "mcq" {
            let fate_scores = outcome
                .mcq_fate
                .as_ref()
                .map(|o| o.option_scores.as_slice())
                .unwrap_or(&[]);
            if !outcome.recompose.mcq_option_affinity.is_empty() || fate_scores.len() >= 2 {
                apply_mcq_patch_bias(
                    &mut tensors,
                    &tokenizer,
                    &outcome.recompose.mcq_option_affinity,
                    fate_scores,
                    cfg.strength,
                    hidden,
                )?;
            }
        }

        let mut model = base.build_model(tensors)?;
        let decode_tag = if p.score_type == "open" && audio.is_some() {
            format!("patch_pipeline_v4_{}_multimodal", p.score_type)
        } else {
            format!("patch_pipeline_v4_{}", p.score_type)
        };

        let (assistant, patch_subtag, patch_conf) = if p.score_type == "mcq" {
            let raw_prompt = format_chat_prompt(&user_text);
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
    }

    Ok(rows)
}
