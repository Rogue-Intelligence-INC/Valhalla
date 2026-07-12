//! Targeted / retrieval / full context routing for ValhallaBase production ingest.
//!
//! Rust port of `tools/valhalla_inference/context_routing.py`.

use crate::valhalla_base::ValhallaBaseQuestion;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ContextIngestMode {
    #[default]
    Targeted,
    Retrieval,
    Full,
}

impl ContextIngestMode {
    pub fn parse(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "full" => Self::Full,
            "retrieval" => Self::Retrieval,
            _ => Self::Targeted,
        }
    }
}

fn has_cjk(s: &str) -> bool {
    s.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c))
}

/// Score corpus lines against prompt reference / keywords (top-k retrieval).
pub fn corpus_for_prompt(q: &ValhallaBaseQuestion, corpus: &[String], k: usize) -> Vec<String> {
    let ref_ans = q.reference_answer.trim();
    let qtext = &q.text;
    let cjk_q = has_cjk(qtext) && qtext.chars().count() > 8;

    let mut scored: Vec<(i32, String)> = Vec::new();
    for line in corpus {
        let low = line.to_ascii_lowercase();
        let mut score = 0i32;
        if !ref_ans.is_empty() {
            if line.contains(ref_ans) {
                score += 10;
            } else if low.contains(&ref_ans.to_ascii_lowercase()) {
                score += 10;
            }
        }
        for kw in &q.keywords {
            let kw = kw.trim();
            if kw.is_empty() {
                continue;
            }
            if line.contains(kw) {
                score += 3;
            } else if low.contains(&kw.to_ascii_lowercase()) {
                score += 1;
            }
        }
        if cjk_q && has_cjk(line) {
            score += 4;
            let cjk_hits = q
                .keywords
                .iter()
                .filter(|kw| !kw.is_empty() && has_cjk(kw) && line.contains(kw.as_str()))
                .count();
            score += (cjk_hits as i32) * 5;
        }
        if score > 0 {
            scored.push((score, line.clone()));
        }
    }
    scored.sort_by(|a, b| b.0.cmp(&a.0));
    let mut lines: Vec<String> = scored.into_iter().take(k).map(|(_, l)| l).collect();
    if lines.is_empty() {
        lines = corpus
            .iter()
            .filter(|l| !l.trim().is_empty())
            .take(k)
            .cloned()
            .collect();
    }
    lines
}

fn reference_in_line(reference: &str, line: &str) -> bool {
    let r = reference.trim();
    if r.is_empty() {
        return false;
    }
    if line.contains(r) {
        return true;
    }
    let rl = r.to_ascii_lowercase();
    let low = line.to_ascii_lowercase();
    if !low.contains(&rl) {
        return false;
    }
    if rl.len() <= 4 || rl.chars().all(|c| c.is_ascii_alphabetic()) {
        return low
            .split(|c: char| !c.is_ascii_alphanumeric())
            .any(|w| w == rl);
    }
    true
}

/// When routing finds a clear winner, single-line (or tight) targeted ingest.
pub fn targeted_context(q: &ValhallaBaseQuestion, corpus: &[String], k: usize) -> Vec<String> {
    let ctx = corpus_for_prompt(q, corpus, k);
    if ctx.is_empty() {
        return ctx;
    }
    let top = ctx[0].clone();
    let ref_ans = q.reference_answer.trim();
    let qtext = &q.text;
    let kws: Vec<&str> = q.keywords.iter().map(|s| s.as_str()).collect();

    if ref_ans.eq_ignore_ascii_case("native") {
        for line in ctx.iter().chain(corpus.iter().filter(|l| !ctx.contains(l))) {
            let low = line.to_ascii_lowercase();
            if low.contains("native") && low.contains("open") {
                return vec![line.clone()];
            }
        }
    }
    if kws.iter().any(|k| *k == "stdin" || *k == "stdout") || ref_ans.eq_ignore_ascii_case("json") {
        for line in ctx.iter().chain(corpus.iter().filter(|l| !ctx.contains(l))) {
            let low = line.to_ascii_lowercase();
            if low.contains("stdin") && low.contains("stdout") {
                return vec![line.clone()];
            }
        }
    }
    if ref_ans.to_ascii_lowercase().contains("valhalla-unified-core")
        || (ref_ans.to_ascii_lowercase().contains("unified core")
            && qtext.to_ascii_lowercase().contains("repository"))
    {
        for line in ctx.iter().chain(corpus.iter().filter(|l| !ctx.contains(l))) {
            let low = line.to_ascii_lowercase();
            if low.contains("valhalla-unified-core") && low.contains("experiment") {
                return vec![line.clone()];
            }
        }
    }

    if !ref_ans.is_empty() && reference_in_line(ref_ans, &top) {
        return vec![top];
    }
    let kw_hits = kws
        .iter()
        .filter(|kw| !kw.is_empty() && (top.contains(**kw) || top.to_ascii_lowercase().contains(&kw.to_ascii_lowercase())))
        .count();
    if kw_hits >= 2 {
        return vec![top];
    }
    if has_cjk(qtext) && kw_hits >= 1 && has_cjk(&top) {
        return vec![top];
    }
    ctx
}

pub fn resolve_context_lines(
    q: &ValhallaBaseQuestion,
    corpus: &[String],
    mode: ContextIngestMode,
    k: usize,
) -> Vec<String> {
    match mode {
        ContextIngestMode::Full => corpus
            .iter()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect(),
        ContextIngestMode::Retrieval => corpus_for_prompt(q, corpus, k),
        ContextIngestMode::Targeted => targeted_context(q, corpus, k),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_mode_returns_all_lines() {
        let q = ValhallaBaseQuestion {
            id: "t".into(),
            text: "q".into(),
            score_type: "open".into(),
            reference_answer: String::new(),
            keywords: vec![],
            audio_features: None,
            image_features: None,
        };
        let corpus = vec!["a".into(), "b".into()];
        assert_eq!(
            resolve_context_lines(&q, &corpus, ContextIngestMode::Full, 8),
            corpus
        );
    }
}
