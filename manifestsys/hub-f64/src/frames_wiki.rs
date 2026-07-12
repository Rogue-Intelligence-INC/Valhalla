//! Wikipedia extract fetch for FRAMES eval — mirrors `tools/standard_benchmarks/download_utils.py`.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

fn wiki_cache_dir(repo_root: &Path) -> PathBuf {
    repo_root
        .join("tools/standard_benchmarks/cache/wiki")
}

fn wiki_title_from_url(url: &str) -> String {
    let path = url
        .split('?')
        .next()
        .unwrap_or(url)
        .split('#')
        .next()
        .unwrap_or(url)
        .trim_end_matches('/');
    path.rsplit('/').next().unwrap_or(path).to_string()
}

fn pct_encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

fn wiki_cache_file(repo_root: &Path, title: &str) -> PathBuf {
    let safe = pct_encode(title);
    wiki_cache_dir(repo_root).join(format!("{safe}.json"))
}

fn read_cached_extract(repo_root: &Path, title: &str) -> Option<String> {
    let cache = wiki_cache_file(repo_root, title);
    let raw = std::fs::read_to_string(cache).ok()?;
    let v: Value = serde_json::from_str(&raw).ok()?;
    let extract = v.get("extract")?.as_str()?.trim();
    if extract.is_empty() {
        return None;
    }
    Some(truncate_extract(extract))
}

fn truncate_extract(s: &str) -> String {
    s.chars().take(1200).collect()
}

fn curl_fetch(url: &str, timeout_sec: u64) -> Option<String> {
    let ua = std::env::var("VALHALLA_WIKI_USER_AGENT").unwrap_or_else(|_| {
        "ValhallaBenchmark/1.0 (benchmark bundle; +https://github.com/Rogue-Intelligence-INC)"
            .into()
    });
    let out = Command::new("curl")
        .args([
            "-fsSL",
            "--connect-timeout",
            "15",
            "--max-time",
            &timeout_sec.to_string(),
            "-H",
            &format!("User-Agent: {ua}"),
            "-H",
            "Accept: application/json",
            url,
        ])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8(out.stdout).ok()
}

fn python_wiki_fetch(repo_root: &Path, title: &str, offline: bool) -> Option<String> {
    let script = repo_root.join("tools/standard_benchmarks/wiki_fetch_one.py");
    if !script.is_file() {
        return None;
    }
    let py = if repo_root.join(".venv-llm/bin/python3").is_file() {
        repo_root.join(".venv-llm/bin/python3")
    } else {
        PathBuf::from("python3")
    };
    let mut cmd = Command::new(py);
    cmd.arg(&script).arg(title).arg("--timeout").arg("25");
    if offline {
        cmd.arg("--offline");
    }
    let out = cmd.output().ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8(out.stdout).ok()?;
    let t = text.trim();
    if t.is_empty() {
        None
    } else {
        Some(truncate_extract(t))
    }
}

fn strip_html_tags(s: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.trim().to_string()
}

fn dbpedia_lookup_fetch(title: &str) -> Option<String> {
    let name = title.replace('_', " ");
    let url = format!(
        "https://lookup.dbpedia.org/api/search?query={}&format=json",
        pct_encode(&name)
    );
    let raw = curl_fetch(&url, 20)?;
    let data: Value = serde_json::from_str(&raw).ok()?;
    let docs = data.get("docs")?.as_array()?;
    let doc = docs.first()?;
    let comment = doc.get("comment");
    let mut text = String::new();
    if let Some(arr) = comment.and_then(|v| v.as_array()) {
        for v in arr {
            if let Some(s) = v.as_str() {
                text.push_str(s);
            }
        }
    } else if let Some(s) = comment.and_then(|v| v.as_str()) {
        text = s.to_string();
    }
    text = strip_html_tags(&text);
    if text.is_empty() {
        if let Some(arr) = doc.get("label").and_then(|v| v.as_array()) {
            if let Some(s) = arr.first().and_then(|v| v.as_str()) {
                text = strip_html_tags(s);
            }
        }
    }
    if text.is_empty() {
        return None;
    }
    Some(truncate_extract(&text))
}

fn parse_wiki_json(raw: &str) -> Option<String> {
    let data: Value = serde_json::from_str(raw).ok()?;
    if let Some(extract) = data.get("extract").and_then(|v| v.as_str()) {
        let t = extract.trim();
        if !t.is_empty() {
            return Some(truncate_extract(t));
        }
    }
    if let Some(pages) = data
        .get("query")
        .and_then(|q| q.get("pages"))
    {
        if let Some(arr) = pages.as_array() {
            for page in arr {
                if let Some(extract) = page.get("extract").and_then(|v| v.as_str()) {
                    let t = extract.trim();
                    if !t.is_empty() {
                        return Some(truncate_extract(t));
                    }
                }
            }
        } else if let Some(obj) = pages.as_object() {
            for page in obj.values() {
                if let Some(extract) = page.get("extract").and_then(|v| v.as_str()) {
                    let t = extract.trim();
                    if !t.is_empty() {
                        return Some(truncate_extract(t));
                    }
                }
            }
        }
    }
    None
}

fn write_cache(repo_root: &Path, title: &str, extract: &str) {
    let dir = wiki_cache_dir(repo_root);
    let _ = std::fs::create_dir_all(&dir);
    let cache = wiki_cache_file(repo_root, title);
    let doc = serde_json::json!({ "title": title, "extract": extract });
    let _ = std::fs::write(cache, serde_json::to_string(&doc).unwrap_or_default());
}

/// Fetch one Wikipedia summary (cache-first, then REST + action API via curl).
pub fn wiki_summary_fetch(
    repo_root: &Path,
    title: &str,
    offline: bool,
) -> Option<String> {
    if let Some(cached) = read_cached_extract(repo_root, title) {
        return Some(cached);
    }
    if offline {
        return None;
    }
    let enc = pct_encode(title);
    let title_sp = title.replace('_', " ");
    let urls = [
        format!("https://en.wikipedia.org/api/rest_v1/page/summary/{enc}"),
        format!("https://en.m.wikipedia.org/api/rest_v1/page/summary/{enc}"),
        format!(
            "https://en.wikipedia.org/w/api.php?action=query&prop=extracts&exintro=1&explaintext=1&format=json&formatversion=2&titles={}",
            pct_encode(&title_sp)
        ),
        format!(
            "https://en.wikipedia.org/w/api.php?action=query&prop=extracts&exintro=1&explaintext=1&format=json&titles={}",
            pct_encode(&title_sp)
        ),
    ];
    for url in &urls {
        if let Some(raw) = curl_fetch(url, 20) {
            if let Some(extract) = parse_wiki_json(&raw) {
                write_cache(repo_root, title, &extract);
                return Some(extract);
            }
        }
    }
    if let Some(extract) = python_wiki_fetch(repo_root, title, offline) {
        return Some(extract);
    }
    if let Some(extract) = dbpedia_lookup_fetch(title) {
        write_cache(repo_root, title, &extract);
        return Some(extract);
    }
    None
}

pub fn wiki_summaries_for_links(
    repo_root: &Path,
    links: &[String],
    offline: bool,
) -> Vec<String> {
    let mut corpus = Vec::new();
    for url in links.iter().take(8) {
        if url.len() < 8 || url.starts_with('[') {
            continue;
        }
        let title = wiki_title_from_url(url);
        if let Some(extract) = wiki_summary_fetch(repo_root, &title, offline) {
            corpus.push(extract);
        }
    }
    corpus
}

pub fn corpus_needs_wiki_fetch(corpus: &[String]) -> bool {
    corpus.is_empty()
        || corpus
            .first()
            .is_some_and(|s| s.starts_with("Reference:") && s.len() < 64)
}

pub fn resolve_frames_corpus(
    repo_root: &Path,
    wiki_links: &[String],
    corpus: &[String],
    offline: bool,
) -> (Vec<String>, bool) {
    if !corpus_needs_wiki_fetch(corpus) {
        return (corpus.to_vec(), false);
    }
    let fetched = wiki_summaries_for_links(repo_root, wiki_links, offline);
    if !fetched.is_empty() {
        return (fetched, true);
    }
    let fallback: Vec<String> = wiki_links
        .iter()
        .filter(|u| u.len() > 8 && !u.starts_with('['))
        .take(8)
        .map(|u| format!("Reference: {u}"))
        .collect();
    (fallback, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn broken_corpus_detected() {
        assert!(corpus_needs_wiki_fetch(&["Reference: h".into()]));
        assert!(!corpus_needs_wiki_fetch(&["Long paragraph about history.".into()]));
    }
}
