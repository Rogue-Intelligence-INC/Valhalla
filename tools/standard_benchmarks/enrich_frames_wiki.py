#!/usr/bin/env python3
"""Enrich FRAMES bundle with Wikipedia fulltext — citeable Tier A path.

Sources (in order):
  1. HuggingFace ``parasail-ai/frames-benchmark-wikipedia`` (prefetched Oct 2024)
  2. Live Wikipedia REST/API via ``wiki_summaries_parallel`` (network)
  3. Leave Reference: URL placeholders (Tier C — not citeable)

Usage:
  python3 tools/standard_benchmarks/enrich_frames_wiki.py
  python3 tools/standard_benchmarks/enrich_frames_wiki.py --limit 25 --prefer live
  python3 tools/standard_benchmarks/enrich_frames_wiki.py --offline   # parasail/cache only
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path
from typing import Any
from urllib.parse import unquote

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
sys.path.insert(0, str(HERE))

from common import frames_bundle_preflight, load_bundled  # noqa: E402
BUNDLED = HERE / "bundled"
DEFAULT_BUNDLE = "frames_slice25.json"


def _title_from_url(url: str) -> str:
    u = unquote(str(url).strip())
    if "/wiki/" in u:
        return u.split("/wiki/", 1)[1].split("#", 1)[0].replace(" ", "_")
    return u.replace(" ", "_")


def _norm_link(url: str) -> str:
    u = unquote(str(url).strip()).split("#", 1)[0].rstrip("/")
    return u.lower()


def load_parasail_index(*, offline: bool = False) -> dict[str, str]:
    """Map normalized wiki URL / title → article text."""
    from download_utils import load_hf_dataset  # noqa: WPS433

    ds = load_hf_dataset(
        "parasail-ai/frames-benchmark-wikipedia",
        split="train",
        offline=offline,
    )
    by_link: dict[str, str] = {}
    by_title: dict[str, str] = {}
    for row in ds:
        text = str(row.get("text") or "").strip()
        if len(text) < 40:
            continue
        link = str(row.get("link") or "")
        title = str(row.get("title") or "").replace(" ", "_")
        if link:
            by_link[_norm_link(link)] = text
        if title:
            by_title[title.lower()] = text
        for ol in row.get("original_link") or []:
            if ol:
                by_link[_norm_link(str(ol))] = text
    print(f"  parasail index: {len(by_link)} links · {len(by_title)} titles", flush=True)
    return {**{f"title:{k}": v for k, v in by_title.items()}, **by_link}


def lookup_parasail(index: dict[str, str], url: str) -> str | None:
    key = _norm_link(url)
    if key in index:
        return index[key]
    title = _title_from_url(url).lower()
    return index.get(f"title:{title}")


def enrich_item(
    item: dict[str, Any],
    *,
    parasail: dict[str, str] | None,
    prefer: str,
    offline: bool,
    max_chars_per_page: int,
) -> dict[str, Any]:
    from download_utils import wiki_summary_fetch  # noqa: WPS433

    links = [str(u) for u in (item.get("wiki_links") or []) if u][:8]
    corpus: list[str] = []
    sources: list[str] = []

    def add_page(url: str, text: str, src: str) -> None:
        title = _title_from_url(url)
        body = text.strip()
        if max_chars_per_page > 0 and len(body) > max_chars_per_page:
            body = body[:max_chars_per_page] + "…"
        corpus.append(f"[{title}] {body}")
        sources.append(src)

    order = ("parasail", "live") if prefer == "parasail" else ("live", "parasail")
    for url in links:
        got = False
        for src in order:
            if src == "parasail" and parasail:
                text = lookup_parasail(parasail, url)
                if text:
                    add_page(url, text, "parasail")
                    got = True
                    break
            if src == "live" and not offline:
                text = wiki_summary_fetch(_title_from_url(url), offline=False)
                if text:
                    add_page(url, text, "wikipedia_live")
                    got = True
                    break
        if not got:
            corpus.append(f"Reference: {url}")
            sources.append("placeholder")

    # If every line is still a Reference placeholder, mark not fetched
    real = [c for c in corpus if not str(c).startswith("Reference:")]
    out = dict(item)
    if real:
        out["corpus"] = real if len(real) >= max(1, len(links) // 2) else corpus
        out["wiki_fetched"] = True
    else:
        out["corpus"] = corpus
        out["wiki_fetched"] = False
    out["wiki_source"] = ",".join(sorted(set(sources)))
    return out


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--bundle", default=DEFAULT_BUNDLE)
    ap.add_argument("--limit", type=int, default=0)
    ap.add_argument("--prefer", choices=("parasail", "live"), default="parasail")
    ap.add_argument("--offline", action="store_true")
    ap.add_argument("--max-chars-per-page", type=int, default=4000)
    ap.add_argument("--no-parasail", action="store_true")
    args = ap.parse_args()

    os.environ.setdefault("HF_ENDPOINT", os.environ.get("HF_ENDPOINT", "https://hf-mirror.com"))

    doc = load_bundled(args.bundle)
    items = list(doc.get("items") or [])
    if args.limit > 0:
        items = items[: args.limit]

    parasail: dict[str, str] | None = None
    if not args.no_parasail:
        print("[1] load parasail-ai/frames-benchmark-wikipedia …", flush=True)
        try:
            parasail = load_parasail_index(offline=args.offline)
        except Exception as exc:  # noqa: BLE001
            print(f"  WARN parasail unavailable: {exc}", flush=True)
            parasail = None

    print(f"[2] enrich {len(items)} items prefer={args.prefer} …", flush=True)
    enriched = []
    hits = 0
    for i, it in enumerate(items):
        row = enrich_item(
            it,
            parasail=parasail,
            prefer=args.prefer,
            offline=args.offline,
            max_chars_per_page=args.max_chars_per_page,
        )
        if row.get("wiki_fetched"):
            hits += 1
        enriched.append(row)
        if (i + 1) % 5 == 0:
            print(f"  {i + 1}/{len(items)} wiki_ok={hits}", flush=True)

    # keep remaining items if limit was used on enrich-only head
    if args.limit > 0 and len(doc.get("items") or []) > args.limit:
        enriched.extend(doc["items"][args.limit :])

    out_doc = dict(doc)
    out_doc["items"] = enriched
    out_doc["n"] = len(enriched)
    out_doc["wiki_items_with_extract"] = sum(1 for it in enriched if it.get("wiki_fetched"))
    out_doc["wiki_enrichment"] = {
        "prefer": args.prefer,
        "parasail": parasail is not None,
        "offline": args.offline,
        "source": "parasail-ai/frames-benchmark-wikipedia + optional live Wikipedia",
    }
    out_doc["citation"] = doc.get("citation") or "NAACL 2025 FRAMES — multi-hop RAG"
    path = BUNDLED / args.bundle
    path.write_text(json.dumps(out_doc, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"→ {path}  wiki_items_with_extract={out_doc['wiki_items_with_extract']}/{out_doc['n']}", flush=True)

    pre = frames_bundle_preflight(args.bundle)
    print(json.dumps(pre, indent=2), flush=True)
    if not pre["viable"]:
        print("FAIL: still Tier C — try without --offline or check HF mirror", flush=True)
        return 1
    print("OK: FRAMES bundle citeable (Tier A preflight)", flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
