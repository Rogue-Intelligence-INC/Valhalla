//! Load `fair_benchmark_suite.json`.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use serde::Deserialize;

use crate::fair_score::FairPrompt;

#[derive(Debug, Deserialize)]
pub struct SuiteFile {
    prompts: Vec<FairPrompt>,
    splits: HashMap<String, Vec<String>>,
    phases: HashMap<String, PhaseCfg>,
    corpus: HashMap<String, Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct PhaseCfg {
    split: String,
    corpus: String,
}

pub fn load_suite(path: &Path) -> Result<SuiteFile, String> {
    let raw = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

pub fn select_prompts(suite: &SuiteFile, phase: &str) -> Result<Vec<FairPrompt>, String> {
    let phase_cfg = suite
        .phases
        .get(phase)
        .ok_or_else(|| format!("unknown phase: {phase}"))?;
    let ids: HashSet<&str> = suite
        .splits
        .get(&phase_cfg.split)
        .ok_or_else(|| format!("unknown split: {}", phase_cfg.split))?
        .iter()
        .map(|s| s.as_str())
        .collect();
    Ok(suite
        .prompts
        .iter()
        .filter(|p| ids.contains(p.id.as_str()))
        .cloned()
        .collect())
}

pub fn select_corpus(suite: &SuiteFile, phase: &str) -> Result<Vec<String>, String> {
    let phase_cfg = suite
        .phases
        .get(phase)
        .ok_or_else(|| format!("unknown phase: {phase}"))?;
    suite
        .corpus
        .get(&phase_cfg.corpus)
        .cloned()
        .ok_or_else(|| format!("unknown corpus tier: {}", phase_cfg.corpus))
}

pub fn classify_slices(prompts: &[FairPrompt], corpus: &[String]) -> FairSlices {
    let mut slices = FairSlices::default();
    for p in prompts {
        let st = p.score_type.as_str();
        if st == "numeric" {
            slices.numeric_compute.insert(p.id.clone());
        }
        if st == "mcq" {
            slices.mcq_routing.insert(p.id.clone());
        }
        if corpus_supports(p, corpus) {
            slices.corpus_supported.insert(p.id.clone());
            if st == "open" {
                slices.open_retrieval.insert(p.id.clone());
            }
        } else if st == "open" {
            slices.open_generation.insert(p.id.clone());
        }
    }
    slices
}

fn corpus_supports(p: &FairPrompt, corpus: &[String]) -> bool {
    let ref_low = p.reference_answer.to_lowercase();
    let kws: Vec<String> = p.keywords.iter().map(|k| k.to_lowercase()).collect();
    for line in corpus {
        let low = line.to_lowercase();
        if ref_low.len() >= 2 && low.contains(&ref_low) {
            return true;
        }
        let hits = kws.iter().filter(|k| !k.is_empty() && low.contains(k.as_str())).count();
        if hits >= (kws.len() / 2).max(1) {
            return true;
        }
    }
    false
}

#[derive(Debug, Default, Clone)]
pub struct FairSlices {
    pub corpus_supported: HashSet<String>,
    pub numeric_compute: HashSet<String>,
    pub mcq_routing: HashSet<String>,
    pub open_retrieval: HashSet<String>,
    pub open_generation: HashSet<String>,
}

// Re-export SuiteFile internals via functions only
pub type LoadedSuite = SuiteFile;
