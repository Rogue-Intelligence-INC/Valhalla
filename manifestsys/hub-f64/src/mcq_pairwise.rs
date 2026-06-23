//! Stem-conditioned pairwise option comparator for MCQ structure memory.
//!
//! Scores each option relative to siblings (not in isolation), addressing the
//! "four-way comparison" bottleneck. Enable with `VALHALLA_MCQ_PAIRWISE=1`.

use std::collections::HashMap;

pub fn mcq_pairwise_enabled() -> bool {
    std::env::var("VALHALLA_MCQ_PAIRWISE")
        .map(|v| {
            let v = v.trim().to_ascii_lowercase();
            v == "1" || v == "true" || v == "on"
        })
        .unwrap_or(false)
}

fn word_set(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
        .filter(|w| w.len() >= 3)
        .collect()
}

fn option_body(opt_line: &str) -> String {
    let t = opt_line.trim();
    if let Some((_, rest)) = t.split_once(')') {
        return rest.trim().to_lowercase();
    }
    if let Some((_, rest)) = t.split_once('.') {
        return rest.trim().to_lowercase();
    }
    t.to_lowercase()
}

fn keyword_overlap(a: &str, b: &str) -> usize {
    let bw: std::collections::HashSet<_> = word_set(b).into_iter().collect();
    word_set(a)
        .into_iter()
        .filter(|w| bw.contains(w))
        .count()
}

/// Relative margin: positive => `body_a` preferred over `body_b` given stem.
pub fn stem_pair_margin(stem: &str, body_a: &str, body_b: &str) -> f64 {
    let stem_words = word_set(stem);
    let aw = word_set(body_a);
    let bw = word_set(body_b);

    let mut margin = 0.0;

    for w in &aw {
        if stem_words.iter().any(|s| s == w || s.contains(w.as_str()) || w.contains(s.as_str())) {
            margin += 0.18;
        }
        if !bw.contains(w) {
            margin += 0.12;
        }
    }
    for w in &bw {
        if stem_words.iter().any(|s| s == w || s.contains(w.as_str()) || w.contains(s.as_str())) {
            margin -= 0.18;
        }
        if !aw.contains(w) {
            margin -= 0.12;
        }
    }

    margin += (keyword_overlap(stem, body_a) as f64 - keyword_overlap(stem, body_b) as f64) * 0.22;
    margin += (keyword_overlap(body_a, body_b) as f64).min(1.0) * -0.05;

    margin
}

/// Borda-style aggregate from all pairwise margins; demeaned for affinity use.
pub fn pairwise_option_scores(
    stem: &str,
    options: &[(char, String)],
    priors: &[(char, f64)],
) -> Vec<(char, f64)> {
    if options.len() < 2 {
        return priors.to_vec();
    }
    let prior_map: HashMap<char, f64> = priors.iter().copied().collect();
    let mut wins: HashMap<char, f64> = HashMap::new();
    for (letter, _) in options {
        wins.insert(*letter, prior_map.get(letter).copied().unwrap_or(0.0) * 0.35);
    }

    for i in 0..options.len() {
        for j in (i + 1)..options.len() {
            let (li, line_i) = &options[i];
            let (lj, line_j) = &options[j];
            let body_i = option_body(line_i);
            let body_j = option_body(line_j);
            let m = stem_pair_margin(stem, &body_i, &body_j);
            if m > 0.0 {
                *wins.get_mut(li).unwrap() += 1.0 + m;
            } else if m < 0.0 {
                *wins.get_mut(lj).unwrap() += 1.0 - m;
            } else {
                *wins.get_mut(li).unwrap() += 0.5;
                *wins.get_mut(lj).unwrap() += 0.5;
            }
        }
    }

    let mut raw: Vec<(char, f64)> = options
        .iter()
        .map(|(l, _)| (*l, *wins.get(l).unwrap_or(&0.0)))
        .collect();
    let mean = raw.iter().map(|(_, s)| *s).sum::<f64>() / raw.len() as f64;
    raw.into_iter().map(|(l, s)| (l, s - mean)).collect()
}

pub fn pairwise_spread(scores: &[(char, f64)]) -> f64 {
    if scores.len() < 2 {
        return 0.0;
    }
    let mut vals: Vec<f64> = scores.iter().map(|(_, s)| *s).collect();
    vals.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    vals[0] - vals[1]
}

pub fn pick_pairwise_letter(scores: &[(char, f64)]) -> Option<char> {
    scores
        .iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(c, _)| *c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stem_pair_margin_prefers_stem_aligned_option() {
        let stem = "glycogen is the storage form of glucose in animals";
        let a = "glycogen";
        let b = "cellulose";
        assert!(stem_pair_margin(stem, a, b) > 0.0);
    }

    #[test]
    fn pairwise_scores_demeaned() {
        let stem = "the capital city paris france europe";
        let opts = vec![
            ('A', "A) Paris".into()),
            ('B', "B) London".into()),
            ('C', "C) Berlin".into()),
            ('D', "D) Rome".into()),
        ];
        let priors = vec![('A', 0.0), ('B', 0.0), ('C', 0.0), ('D', 0.0)];
        let scores = pairwise_option_scores(stem, &opts, &priors);
        let sum: f64 = scores.iter().map(|(_, s)| *s).sum();
        assert!(sum.abs() < 1e-9);
        let top = pick_pairwise_letter(&scores).unwrap();
        assert_eq!(top, 'A');
    }
}
