//! Rust-native ValhallaBase benchmarks — replaces Python smoke/bench scripts.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde::Deserialize;

use crate::valhalla_base::{ValhallaBaseQuestion, ValhallaContextItem};
use crate::valhalla_base_client::{RunOptions, ValhallaBaseClient};
use crate::valhalla_base_protocol::ValhallaBaseErrorBody;

const NEEDLE: &str = "The magic registration code for Project Valhalla is XK-7742.";
const NEEDLE_Q: &str = "What is the magic registration code for Project Valhalla?";
const NEEDLE_HINT: &str = "xk-7742";

static FILLER: &[&str] = &[
    "Paris is the capital of France and lies on the River Seine.",
    "Iron rusts when exposed to moist air over time.",
    "The speed of light in vacuum is approximately 299792458 meters per second.",
    "Photosynthesis converts light energy into chemical energy in plants.",
    "The Pacific Ocean is the largest ocean on Earth by surface area.",
    "Diamond is a crystalline form of carbon with high hardness.",
    "The human heart has four chambers: two atria and two ventricles.",
    "Mitochondria are often called the powerhouse of the cell.",
    "Shakespeare wrote Hamlet and Romeo and Juliet.",
    "The Great Wall of China is a series of fortifications in northern China.",
    "Jupiter is the largest planet in the Solar System.",
    "The chemical symbol for sodium is Na.",
    "Beethoven composed nine symphonies during his lifetime.",
    "The Amazon rainforest produces significant oxygen for the planet.",
    "Mount Everest is the highest mountain above sea level.",
    "The periodic table organizes elements by atomic number.",
    "DNA stores genetic information in living organisms.",
    "The Nile is one of the longest rivers in the world.",
    "Volcanoes form at tectonic plate boundaries.",
    "The Moon orbits Earth approximately every 27.3 days.",
    "Antarctica is the coldest continent on Earth.",
    "Copper is an excellent conductor of electricity.",
    "The Renaissance began in Italy in the 14th century.",
    "Black holes have gravitational fields so strong light cannot escape.",
    "The Sahara is the largest hot desert in the world.",
    "Oxygen makes up about 21 percent of Earth's atmosphere.",
    "The Eiffel Tower was completed in Paris in 1889.",
    "Proteins are polymers of amino acids.",
    "The Roman Empire once controlled much of Europe and the Mediterranean.",
    "Sound travels faster in water than in air.",
    "Mercury is the closest planet to the Sun.",
    "Helium is the second lightest element in the periodic table.",
    "The Amazon River discharges more water than any other river.",
    "Granite is an igneous rock formed from cooled magma.",
    "The human skeleton has 206 bones in adulthood.",
];

fn now_iso() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}

fn recall_ok(answer: &str) -> bool {
    let low = answer.to_lowercase();
    low.contains(NEEDLE_HINT) || low.contains("7742")
}

/// `test_valhalla_base_v1.py` — hybrid, session append, multimodal.
pub fn run_v1_extensions() -> Result<serde_json::Value, ValhallaBaseErrorBody> {
    let mut base = ValhallaBaseClient::production();
    base.config.max_new = 96;

    base.set_context([
        "The SI unit of force is the Newton.",
        "F = ma relates force and acceleration.",
    ]);
    let hybrid = base.run(
        vec![
            ValhallaBaseQuestion {
                id: "hybrid_open".into(),
                text: "What is the SI unit of force?".into(),
                score_type: "open".into(),
                reference_answer: "Newton".into(),
                keywords: vec!["Newton".into(), "N".into()],
                audio_features: None,
                image_features: None,
            },
            ValhallaBaseQuestion {
                id: "hybrid_mcq".into(),
                text: "What is the SI unit of force?\nA) Joule\nB) Newton\nC) Watt\nD) Pascal"
                    .into(),
                score_type: "mcq".into(),
                reference_answer: "B".into(),
                keywords: Vec::new(),
                audio_features: None,
                image_features: None,
            },
        ],
        RunOptions::default(),
    )?;

    base.clear_context();
    base.set_context(["Planet Alpha: water boils at 100 degrees Celsius."]);
    let r1 = base.run(
        vec![ValhallaBaseQuestion {
            id: "s1".into(),
            text: "Boiling point on this planet?".into(),
            score_type: "open".into(),
            reference_answer: String::new(),
            keywords: Vec::new(),
            audio_features: None,
            image_features: None,
        }],
        RunOptions::default(),
    )?;
    let fp1 = r1.context_fingerprint.clone();
    let r2 = base.run(
        vec![ValhallaBaseQuestion {
            id: "s2".into(),
            text: "Boiling point on this planet?".into(),
            score_type: "open".into(),
            reference_answer: String::new(),
            keywords: Vec::new(),
            audio_features: None,
            image_features: None,
        }],
        RunOptions {
            append_only: true,
            context_items: Some(vec![ValhallaContextItem {
                text: Some("Correction: on Alpha water boils at 85C due to altitude.".into()),
                ..Default::default()
            }]),
            context: None,
        },
    )?;

    base.clear_context();
    let mm = base.run(
        vec![ValhallaBaseQuestion {
            id: "mm1".into(),
            text: "What domain does this context describe?".into(),
            score_type: "open".into(),
            reference_answer: String::new(),
            keywords: Vec::new(),
            audio_features: None,
            image_features: None,
        }],
        RunOptions {
            context_items: Some(vec![
                ValhallaContextItem {
                    text: Some("Photosynthesis occurs in chloroplasts using light energy.".into()),
                    ..Default::default()
                },
                ValhallaContextItem {
                    audio_features: Some(vec![0.01; 64]),
                    ..Default::default()
                },
                ValhallaContextItem {
                    image_features: Some(vec![0.02; 64]),
                    ..Default::default()
                },
            ]),
            ..Default::default()
        },
    )?;

    Ok(serde_json::json!({
        "timestamp": now_iso(),
        "protocol": "valhalla-base-v1-extensions",
        "tests": {
            "hybrid": {
                "features": hybrid.features,
                "rows": hybrid.rows.iter().map(|r| serde_json::json!({
                    "id": r.id,
                    "decode_path": r.decode_path,
                    "answer": r.answer.chars().take(200).collect::<String>(),
                })).collect::<Vec<_>>(),
            },
            "session": {
                "fp_before": fp1,
                "fp_after": r2.context_fingerprint,
                "fingerprint_changed": fp1 != r2.context_fingerprint,
                "lines_ingested": r2.session.lines_ingested,
                "features": r2.features,
            },
            "multimodal": {
                "features": mm.features,
                "context_lines": mm.context_lines,
                "answer": mm.rows.first().map(|r| r.answer.chars().take(300).collect::<String>()),
            },
        }
    }))
}

#[derive(Serialize)]
struct NeedleRow {
    depth: usize,
    session_lines: usize,
    correct: bool,
    answer: String,
}

fn run_needle_depth(depth: usize, fast: bool) -> Result<NeedleRow, ValhallaBaseErrorBody> {
    let mut base = ValhallaBaseClient::production();
    base.config.fate_qa_feedback = false;
    base.config.incubation_cycles = if fast { 1 } else { 2 };
    base.config.max_new = 64;

    let before: Vec<String> = (0..depth)
        .map(|i| FILLER[i % FILLER.len()].to_string())
        .collect();
    let after: Vec<String> = (0..depth)
        .map(|i| FILLER[(i + depth) % FILLER.len()].to_string())
        .collect();

    if let Some(first) = before.first() {
        base.set_context([first.as_str()]);
    } else {
        base.clear_context();
    }
    for line in before.iter().skip(1) {
        base.append_context(line);
    }
    base.append_context(NEEDLE);
    for line in &after {
        base.append_context(line);
    }

    let out = base.run(
        vec![ValhallaBaseQuestion {
            id: format!("needle_d{depth}"),
            text: NEEDLE_Q.into(),
            score_type: "open".into(),
            reference_answer: NEEDLE_HINT.into(),
            keywords: vec!["7742".into()],
            audio_features: None,
            image_features: None,
        }],
        RunOptions::default(),
    )?;
    let ans = out.rows.first().map(|r| r.answer.as_str()).unwrap_or("");
    Ok(NeedleRow {
        depth,
        session_lines: 1 + before.len() + 1 + after.len(),
        correct: recall_ok(ans),
        answer: ans.chars().take(160).collect(),
    })
}

/// `test_needle_session_v1.py`
pub fn run_needle_session(depths: &[usize], fast: bool) -> Result<serde_json::Value, ValhallaBaseErrorBody> {
    let rows: Vec<NeedleRow> = depths
        .iter()
        .map(|&d| run_needle_depth(d, fast))
        .collect::<Result<_, _>>()?;
    let correct = rows.iter().filter(|r| r.correct).count();
    let n = rows.len().max(1);
    let recall = correct as f64 / n as f64;
    let verdict = if recall >= 0.90 {
        "NEEDLE_STRONG"
    } else if recall >= 0.70 {
        "NEEDLE_VIABLE"
    } else {
        "NEEDLE_DEVELOPING"
    };
    Ok(serde_json::json!({
        "timestamp": now_iso(),
        "protocol": "needle-session-v1",
        "benchmark": "Needle-in-a-haystack (session append)",
        "needle": NEEDLE,
        "question": NEEDLE_Q,
        "depths": depths,
        "n": rows.len(),
        "correct": correct,
        "recall": (recall * 10000.0).round() / 10000.0,
        "rows": rows,
        "verdict": verdict,
    }))
}

struct LocalProbe {
    id: &'static str,
    text: &'static str,
    keywords: &'static [&'static str],
}

const LOCAL_PROBES: &[LocalProbe] = &[
    LocalProbe { id: "L01", text: "What is the primary ValhallaBase protocol name?", keywords: &["valhalla-base-v1"] },
    LocalProbe { id: "L02", text: "What fate ingress routing mode is the production default?", keywords: &["quad_cycle"] },
    LocalProbe { id: "L03", text: "Which four meta-systems does Hub route corpus signals to?", keywords: &["blackhole", "meshbrain"] },
    LocalProbe { id: "L04", text: "For open questions in hybrid decode, which path is used by default?", keywords: &["native"] },
    LocalProbe { id: "L05", text: "What follow-up decode mode is recommended for production?", keywords: &["native_follow_up_aware"] },
    LocalProbe { id: "L06", text: "What product line name is abbreviated VUC?", keywords: &["valhalla unified core", "vuc"] },
    LocalProbe { id: "L07", text: "What JSON interface does valhalla_base binary use?", keywords: &["stdin", "json"] },
    LocalProbe { id: "L08", text: "What is the default LM model family mentioned for the decode head?", keywords: &["qwen"] },
];

fn load_corpus_lines(dirs: &[PathBuf], max_lines: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for dir in dirs {
        if !dir.is_dir() {
            continue;
        }
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext != "md" && ext != "txt" {
                continue;
            }
            if path.to_string_lossy().contains("archive") {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            for line in text.lines() {
                let t = line.trim();
                if t.len() < 8 || seen.contains(t) {
                    continue;
                }
                seen.insert(t.to_string());
                lines.push(t.to_string());
                if lines.len() >= max_lines {
                    return lines;
                }
            }
        }
    }
    lines
}

fn probe_ok(answer: &str, keywords: &[&str]) -> bool {
    let low = answer.to_lowercase();
    keywords.iter().any(|k| low.contains(&k.to_lowercase()))
}

/// `run_local_corpus_demo.py` (core path — native hybrid eval on local docs).
pub fn run_local_corpus_demo(corpus_dirs: &[PathBuf]) -> Result<serde_json::Value, ValhallaBaseErrorBody> {
    let lines = load_corpus_lines(corpus_dirs, 80);
    if lines.is_empty() {
        return Err(ValhallaBaseErrorBody::new(
            crate::valhalla_base_protocol::ValhallaBaseErrorKind::Validate,
            "no corpus lines loaded — check corpus dirs",
        ));
    }

    let mut base = ValhallaBaseClient::production();
    base.config.max_new = 160;
    base.set_context(lines.iter().map(|s| s.as_str()));

    let questions: Vec<ValhallaBaseQuestion> = LOCAL_PROBES
        .iter()
        .map(|p| ValhallaBaseQuestion {
            id: p.id.to_string(),
            text: p.text.to_string(),
            score_type: "open".into(),
            reference_answer: String::new(),
            keywords: p.keywords.iter().map(|s| s.to_string()).collect(),
            audio_features: None,
            image_features: None,
        })
        .collect();

    let resp = base.run(questions, RunOptions::default())?;
    let mut correct = 0usize;
    let rows: Vec<serde_json::Value> = resp
        .rows
        .iter()
        .zip(LOCAL_PROBES.iter())
        .map(|(row, probe)| {
            let ok = row.correct || probe_ok(&row.answer, probe.keywords);
            if ok {
                correct += 1;
            }
            serde_json::json!({
                "id": row.id,
                "correct": ok,
                "answer": row.answer.chars().take(200).collect::<String>(),
            })
        })
        .collect();

    let n = LOCAL_PROBES.len().max(1);
    Ok(serde_json::json!({
        "timestamp": now_iso(),
        "protocol": "local-corpus-demo-v1",
        "corpus_lines": lines.len(),
        "n": n,
        "correct": correct,
        "accuracy": (correct as f64 / n as f64 * 10000.0).round() / 10000.0,
        "rows": rows,
    }))
}

pub fn default_corpus_dirs(repo_root: &Path) -> Vec<PathBuf> {
    vec![
        repo_root.join("valhalla-unified-core/docs"),
        repo_root.join("docs"),
    ]
}

fn score_short_answer(answer: &str, gold: &str) -> bool {
    let ref_s = gold.trim();
    if ref_s.is_empty() {
        return false;
    }
    let low = answer.to_lowercase();
    if low.contains(&ref_s.to_lowercase()) {
        return true;
    }
    let ref_tokens: Vec<String> = ref_s
        .split_whitespace()
        .filter_map(|w| {
            let t: String = w
                .chars()
                .filter(|c| c.is_ascii_alphanumeric())
                .collect::<String>()
                .to_lowercase();
            if t.len() >= 2 {
                Some(t)
            } else {
                None
            }
        })
        .collect();
    if ref_tokens.is_empty() {
        return false;
    }
    let hit = ref_tokens.iter().filter(|t| low.contains(t.as_str())).count();
    hit >= (ref_tokens.len() + 1) / 2
}

fn verdict_open_qa(acc: f64) -> &'static str {
    if acc >= 0.65 {
        "OPEN_QA_STRONG"
    } else if acc >= 0.40 {
        "OPEN_QA_VIABLE"
    } else {
        "OPEN_QA_DEVELOPING"
    }
}

#[derive(Deserialize)]
struct MultihopBundle {
    items: Vec<MultihopItem>,
}

#[derive(Deserialize)]
struct MultihopItem {
    id: String,
    question: String,
    answer: String,
    #[serde(default)]
    corpus: Vec<String>,
}

/// MultiHop-RAG slice25 — structure composition decode (multihop_v2).
pub fn run_multihop_rag(
    bundle_path: &Path,
    limit: usize,
    fast: bool,
    max_corpus_lines: usize,
) -> Result<serde_json::Value, ValhallaBaseErrorBody> {
    let raw = std::fs::read_to_string(bundle_path).map_err(|e| {
        ValhallaBaseErrorBody::new(
            crate::valhalla_base_protocol::ValhallaBaseErrorKind::Io,
            format!("read {}: {e}", bundle_path.display()),
        )
    })?;
    let doc: MultihopBundle = serde_json::from_str(&raw).map_err(|e| {
        ValhallaBaseErrorBody::new(
            crate::valhalla_base_protocol::ValhallaBaseErrorKind::Parse,
            format!("parse multihop bundle: {e}"),
        )
    })?;

    let mut items: Vec<_> = doc
        .items
        .into_iter()
        .filter(|i| !i.corpus.is_empty())
        .collect();
    if limit > 0 {
        items.truncate(limit);
    }

    let mut rows = Vec::new();
    let mut correct = 0usize;
    for item in &items {
        let mut corpus = item.corpus.clone();
        if corpus.len() > max_corpus_lines {
            corpus.truncate(max_corpus_lines);
        }
        let mut base = ValhallaBaseClient::production();
        base.config.multihop_v2 = true;
        base.config.fate_qa_feedback = false;
        base.config.incubation_cycles = if fast { 1 } else { 2 };
        base.config.max_new = 128;
        base.set_context(corpus.iter().map(|s| s.as_str()));
        let resp = base.run(
            vec![ValhallaBaseQuestion {
                id: item.id.clone(),
                text: item.question.clone(),
                score_type: "open".into(),
                reference_answer: item.answer.clone(),
                keywords: Vec::new(),
                audio_features: None,
                image_features: None,
            }],
            RunOptions::default(),
        )?;
        let ans = resp.rows.first().map(|r| r.answer.as_str()).unwrap_or("");
        let ok = score_short_answer(ans, &item.answer);
        if ok {
            correct += 1;
        }
        rows.push(serde_json::json!({
            "id": item.id,
            "correct": ok,
            "answer": ans.chars().take(200).collect::<String>(),
            "decode_path": resp.rows.first().map(|r| r.decode_path.as_str()).unwrap_or(""),
        }));
    }

    let n = items.len().max(1);
    let acc = correct as f64 / n as f64;
    Ok(serde_json::json!({
        "timestamp": now_iso(),
        "protocol": "multihop-rag-v2",
        "benchmark": "MultiHop-RAG",
        "bundle": bundle_path.file_name().and_then(|s| s.to_str()).unwrap_or(""),
        "multihop_v2": true,
        "n": items.len(),
        "correct": correct,
        "accuracy": (acc * 10000.0).round() / 10000.0,
        "verdict": verdict_open_qa(acc),
        "rows": rows,
    }))
}

pub fn write_report(path: &Path, doc: &serde_json::Value) -> Result<(), ValhallaBaseErrorBody> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            ValhallaBaseErrorBody::new(
                crate::valhalla_base_protocol::ValhallaBaseErrorKind::Io,
                format!("mkdir {}: {e}", parent.display()),
            )
        })?;
    }
    let pretty = serde_json::to_string_pretty(doc).map_err(|e| {
        ValhallaBaseErrorBody::new(
            crate::valhalla_base_protocol::ValhallaBaseErrorKind::Io,
            format!("serialize report: {e}"),
        )
    })?;
    std::fs::write(path, format!("{pretty}\n")).map_err(|e| {
        ValhallaBaseErrorBody::new(
            crate::valhalla_base_protocol::ValhallaBaseErrorKind::Io,
            format!("write {}: {e}", path.display()),
        )
    })?;
    Ok(())
}
