#!/usr/bin/env python3
"""Robust fetch helpers for benchmark bundle downloads.

Strategy:
  1. Disk cache under tools/standard_benchmarks/cache/ (reuse on retry / --offline)
  2. curl first for large HTTPS (GitHub raw) — more stable than urllib here
  3. urllib with retries as fallback
  4. GitHub mirror chain (raw → ghproxy → jsdelivr)
  5. HF datasets via HF_ENDPOINT (default hf-mirror.com when unset)
  6. Wikipedia: disk cache + curl + MediaWiki API fallback
"""

from __future__ import annotations

import hashlib
import json
import os
import re
import shutil
import subprocess
import time
import urllib.error
import urllib.parse
import urllib.request
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parent
CACHE = ROOT / "cache"
WIKI_CACHE = CACHE / "wiki"

USER_AGENT = "ValhallaBenchmark/1.0 (Valhalla; benchmark bundle downloader)"
DEFAULT_HF_ENDPOINTS = (
    "https://hf-mirror.com",
    "https://huggingface.co",
)

# GitHub raw mirrors (repo = owner/name, path = file path within repo)
GITHUB_MIRROR_TEMPLATES = (
    "https://raw.githubusercontent.com/{repo}/{branch}/{path}",
    "https://ghproxy.net/https://raw.githubusercontent.com/{repo}/{branch}/{path}",
    "https://cdn.jsdelivr.net/gh/{repo}@{branch}/{path}",
)


def ensure_hf_endpoint(preferred: str | None = None) -> str:
    """Set HF_ENDPOINT if unset; return active endpoint."""
    if preferred:
        os.environ["HF_ENDPOINT"] = preferred.rstrip("/")
        return os.environ["HF_ENDPOINT"]
    existing = os.environ.get("HF_ENDPOINT", "").strip()
    if existing:
        return existing.rstrip("/")
    os.environ["HF_ENDPOINT"] = DEFAULT_HF_ENDPOINTS[0]
    return os.environ["HF_ENDPOINT"]


def _cache_path(key: str, suffix: str = ".bin") -> Path:
    h = hashlib.sha256(key.encode()).hexdigest()[:16]
    safe = re.sub(r"[^a-zA-Z0-9._-]+", "_", key)[-80:]
    CACHE.mkdir(parents=True, exist_ok=True)
    return CACHE / f"{safe}.{h}{suffix}"


def curl_fetch(
    url: str,
    *,
    timeout_sec: int = 120,
    retries: int = 3,
    out_path: Path | None = None,
) -> bytes:
    """Download via curl (-L, retries, connect timeout). Raises on failure."""
    dest = out_path or _cache_path(url)
    if dest.is_file() and dest.stat().st_size > 0:
        return dest.read_bytes()

    last_err = ""
    for attempt in range(retries):
        cmd = [
            "curl",
            "-fsSL",
            "--connect-timeout",
            "30",
            "--max-time",
            str(timeout_sec),
            "-H",
            f"User-Agent: {USER_AGENT}",
            "-o",
            str(dest),
            url,
        ]
        try:
            subprocess.run(cmd, check=True, capture_output=True, text=True)
            data = dest.read_bytes()
            if data:
                return data
            last_err = "empty response"
        except subprocess.CalledProcessError as exc:
            last_err = (exc.stderr or exc.stdout or str(exc))[:200]
        if attempt + 1 < retries:
            time.sleep(2**attempt)
    raise RuntimeError(f"curl failed for {url}: {last_err}")


def urllib_fetch(
    url: str,
    *,
    timeout: float = 60.0,
    retries: int = 3,
) -> bytes:
    last_err = ""
    for attempt in range(retries):
        try:
            req = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
            with urllib.request.urlopen(req, timeout=timeout) as resp:
                return resp.read()
        except Exception as exc:  # noqa: BLE001
            last_err = str(exc)
            if attempt + 1 < retries:
                time.sleep(2**attempt)
    raise RuntimeError(f"urllib failed for {url}: {last_err}")


def fetch_bytes(
    url: str,
    *,
    timeout_sec: int = 120,
    retries: int = 3,
    prefer_curl: bool = True,
    cache_key: str | None = None,
) -> bytes:
    """Fetch URL with cache + curl/urllib fallback."""
    key = cache_key or url
    cached = _cache_path(key)
    if cached.is_file() and cached.stat().st_size > 0:
        return cached.read_bytes()

    if prefer_curl and shutil.which("curl"):
        try:
            data = curl_fetch(url, timeout_sec=timeout_sec, retries=retries, out_path=cached)
            return data
        except RuntimeError:
            pass

    data = urllib_fetch(url, timeout=float(timeout_sec), retries=retries)
    cached.write_bytes(data)
    return data


def fetch_json(url: str, **kwargs: Any) -> Any:
    return json.loads(fetch_bytes(url, **kwargs).decode("utf-8"))


def github_raw_urls(repo: str, path: str, *, branch: str = "main") -> list[str]:
    """Candidate URLs for a GitHub raw file."""
    return [
        tpl.format(repo=repo, path=path, branch=branch)
        for tpl in GITHUB_MIRROR_TEMPLATES
    ]


def fetch_github_json(repo: str, path: str, *, branch: str = "main", offline: bool = False) -> Any:
    """Download JSON from GitHub with mirror chain + cache."""
    cache_file = CACHE / "github" / repo.replace("/", "__") / path.replace("/", "__")
    cache_file.parent.mkdir(parents=True, exist_ok=True)
    if cache_file.is_file():
        return json.loads(cache_file.read_text(encoding="utf-8"))

    if offline:
        raise FileNotFoundError(f"offline and no cache: {repo}/{path}")

    errors: list[str] = []
    for url in github_raw_urls(repo, path, branch=branch):
        try:
            data = fetch_bytes(url, timeout_sec=300, cache_key=f"github:{repo}:{path}")
            cache_file.write_bytes(data)
            return json.loads(data.decode("utf-8"))
        except Exception as exc:  # noqa: BLE001
            errors.append(f"{url}: {exc}")

    raise RuntimeError(f"GitHub fetch failed for {repo}/{path}\n" + "\n".join(errors[:4]))


def load_hf_dataset(repo: str, *, config: str | None = None, split: str = "train", offline: bool = False):
    """Load HuggingFace dataset with endpoint + offline fallback."""
    from datasets import load_dataset

    endpoint = ensure_hf_endpoint()
    kwargs: dict[str, Any] = {"split": split}
    if config is not None:
        kwargs["name"] = config

    if offline:
        kwargs["download_mode"] = "reuse_dataset_if_exists"

    last_err = ""
    for ep in ([endpoint] if endpoint else []) + list(DEFAULT_HF_ENDPOINTS):
        try:
            os.environ["HF_ENDPOINT"] = ep
            if config is None:
                return load_dataset(repo, split=split, download_mode=kwargs.get("download_mode"))
            return load_dataset(repo, config, split=split, download_mode=kwargs.get("download_mode"))
        except Exception as exc:  # noqa: BLE001
            last_err = str(exc)
            continue
    raise RuntimeError(f"HF load failed for {repo}: {last_err}")


def _wiki_title_from_url(url: str) -> str:
    """Title segment only — strip query, fragment, trailing slash."""
    path = url.split("?", 1)[0].split("#", 1)[0].rstrip("/")
    return path.rsplit("/", 1)[-1]


def _wiki_cache_file(title: str) -> Path:
    WIKI_CACHE.mkdir(parents=True, exist_ok=True)
    safe = urllib.parse.quote(title, safe="")
    return WIKI_CACHE / f"{safe}.json"


def wiki_summary_fetch(title: str, *, timeout_sec: int = 12, offline: bool = False) -> str | None:
    """Fetch Wikipedia extract; returns None if unavailable."""
    cache = _wiki_cache_file(title)
    if cache.is_file():
        try:
            v = json.loads(cache.read_text(encoding="utf-8"))
            if v.get("extract"):
                return str(v["extract"])[:1200]
        except Exception:
            pass

    if offline:
        return None

    enc = urllib.parse.quote(title, safe="")
    urls = [
        f"https://en.wikipedia.org/api/rest_v1/page/summary/{enc}",
        (
            "https://en.wikipedia.org/w/api.php?"
            + urllib.parse.urlencode(
                {
                    "action": "query",
                    "prop": "extracts",
                    "exintro": "1",
                    "explaintext": "1",
                    "format": "json",
                    "titles": title.replace("_", " "),
                }
            )
        ),
    ]

    for url in urls:
        try:
            if shutil.which("curl"):
                raw = curl_fetch(url, timeout_sec=timeout_sec, retries=2).decode("utf-8", errors="replace")
            else:
                raw = urllib_fetch(url, timeout=float(timeout_sec), retries=2).decode("utf-8", errors="replace")
            data = json.loads(raw)
            extract = ""
            if "extract" in data:
                extract = str(data["extract"]).strip()
            elif "query" in data:
                pages = data.get("query", {}).get("pages", {})
                for page in pages.values():
                    extract = str(page.get("extract") or "").strip()
                    if extract:
                        break
            if extract:
                cache.write_text(json.dumps({"title": title, "extract": extract}, ensure_ascii=False), encoding="utf-8")
                return extract[:1200]
        except Exception:
            continue
    return None


def wiki_summaries_parallel(
    urls: list[str],
    *,
    max_workers: int = 4,
    timeout_per: int = 20,
    offline: bool = False,
) -> list[str]:
    titles = [_wiki_title_from_url(u) for u in urls if u]
    corpus: list[str] = []

    def one(title: str) -> str | None:
        return wiki_summary_fetch(title, timeout_sec=timeout_per, offline=offline)

    with ThreadPoolExecutor(max_workers=max_workers) as ex:
        futs = [ex.submit(one, t) for t in titles]
        for fut in as_completed(futs, timeout=max(120, timeout_per * len(titles) * 2)):
            try:
                s = fut.result()
                if s:
                    corpus.append(s)
            except Exception:
                pass
    return corpus
