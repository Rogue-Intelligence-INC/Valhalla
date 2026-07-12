//! Fair benchmark scoring — port of `tools/fair_benchmark/metrics.py`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FairPrompt {
    pub id: String,
    #[serde(default)]
    pub domain: String,
    pub text: String,
    pub reference_answer: String,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default = "default_score_type")]
    pub score_type: String,
}

fn default_score_type() -> String {
    "open".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreResult {
    pub score_type: String,
    pub correct: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extracted: Option<String>,
    pub reference: String,
}

pub fn format_prompt_text(p: &FairPrompt) -> String {
    let mut t = p.text.clone();
    match p.score_type.as_str() {
        "mcq" => {
            t.push_str("\n\nThink briefly, then end with exactly one line: ANSWER: X (X must be A, B, C, or D).");
        }
        "numeric" => {
            t.push_str("\n\nEnd your response with the final numeric answer alone on the last line.");
        }
        _ => {}
    }
    t
}

pub fn extract_mcq_letter(text: &str) -> Option<char> {
    let upper = text.to_uppercase();
    for pat in ["ANSWER:", "ANSWER ", "答案:", "答案："] {
        if let Some(pos) = upper.find(pat) {
            let rest = &upper[pos + pat.len()..];
            for c in rest.chars() {
                if matches!(c, 'A' | 'B' | 'C' | 'D') {
                    return Some(c);
                }
            }
        }
    }
    if let Some(pos) = upper.find("CORRECT ANSWER") {
        let rest = &upper[pos..];
        for c in rest.chars() {
            if matches!(c, 'A' | 'B' | 'C' | 'D') {
                return Some(c);
            }
        }
    }
    for line in text.trim().lines().rev() {
        let t = line.trim();
        if t.len() == 1 {
            let c = t.chars().next().unwrap().to_ascii_uppercase();
            if matches!(c, 'A' | 'B' | 'C' | 'D') {
                return Some(c);
            }
        }
    }
    None
}

pub fn extract_number(text: &str) -> Option<String> {
    let cleaned = text.replace(',', "");
    let mut last: Option<String> = None;
    let mut cur = String::new();
    let mut in_num = false;
    for c in cleaned.chars() {
        if c.is_ascii_digit() || (c == '.' && in_num) || (c == '-' && !in_num && cur.is_empty()) {
            cur.push(c);
            in_num = true;
        } else if in_num {
            last = Some(cur.clone());
            cur.clear();
            in_num = false;
        }
    }
    if in_num && !cur.is_empty() {
        last = Some(cur);
    }
    last
}

pub fn score_answer(text: &str, prompt: &FairPrompt) -> ScoreResult {
    let ref_ans = prompt.reference_answer.trim();
    let st = prompt.score_type.as_str();
    let low = text.to_lowercase();

    match st {
        "mcq" => {
            let letter = extract_mcq_letter(text);
            let ref_letter = ref_ans.chars().next().unwrap_or(' ').to_ascii_uppercase();
            let hit = letter == Some(ref_letter);
            ScoreResult {
                score_type: "mcq".into(),
                correct: hit,
                extracted: letter.map(|c| c.to_string()),
                reference: ref_letter.to_string(),
            }
        }
        "numeric" => {
            let num = extract_number(text);
            let hit = num.as_deref() == Some(ref_ans)
                || num.as_ref().map(|n| ref_ans.contains(n)).unwrap_or(false)
                || low.contains(&ref_ans.to_lowercase());
            ScoreResult {
                score_type: "numeric".into(),
                correct: hit,
                extracted: num,
                reference: ref_ans.to_string(),
            }
        }
        _ => {
            // Empty reference + no keywords: unscored open (Fate uses structural on_topic).
            if ref_ans.is_empty() && prompt.keywords.iter().all(|k| k.trim().is_empty()) {
                return ScoreResult {
                    score_type: "open".into(),
                    correct: false,
                    extracted: None,
                    reference: String::new(),
                };
            }
            let ref_low = ref_ans.to_lowercase();
            let mut hit = !ref_low.is_empty() && low.contains(&ref_low);
            if !hit {
                let kw_ok = prompt
                    .keywords
                    .iter()
                    .filter(|k| !k.is_empty() && low.contains(&k.to_lowercase()))
                    .count();
                let need = (prompt.keywords.len() / 2).max(1);
                hit = kw_ok >= need;
            }
            if !hit && !ref_ans.is_empty() {
                hit = ref_ans
                    .split_whitespace()
                    .filter(|w| w.len() > 2)
                    .any(|w| low.contains(&w.to_lowercase()));
            }
            ScoreResult {
                score_type: "open".into(),
                correct: hit,
                extracted: None,
                reference: ref_ans.to_string(),
            }
        }
    }
}
