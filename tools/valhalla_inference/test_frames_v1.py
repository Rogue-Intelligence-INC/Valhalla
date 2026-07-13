#!/usr/bin/env python3
"""FRAMES multi-hop RAG — Rust production path (Wikipedia fetch + dynamic fragments).

Protocol: frames-v1
Baseline (paper): naive 40.8% · oracle 72.9% · multi-step 66%

Usage:
  python3 tools/valhalla_inference/test_frames_v1.py --rust --fast
  python3 tools/valhalla_inference/test_frames_v1.py --legacy
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(REPO / "tools/standard_benchmarks"))

from benchmark_env import apply_resource_budget, release_resources  # noqa: E402
from benchmark_cli import add_benchmark_args  # noqa: E402
from benchmark_runner import make_base, run_frames_rust, run_open_item  # noqa: E402
from common import (
    frames_bundle_preflight,
    load_bundled,
    score_short_answer,
    split_context_to_corpus,
    verdict_open_qa,
)  # noqa: E402

PROTOCOL = "frames-v1"
REPORTS = REPO / "reports/valhalla_inference"
BUNDLE = "frames_slice25.json"


def wiki_reachable_quick(*, timeout_sec: float = 4.0) -> bool:
    import urllib.error
    import urllib.request

    try:
        urllib.request.urlopen(
            "https://en.wikipedia.org/w/api.php?action=query&format=json&titles=Main_Page",
            timeout=timeout_sec,
        )
        return True
    except (urllib.error.URLError, TimeoutError, OSError):
        return False


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--limit", type=int, default=0)
    ap.add_argument("--fast", action="store_true")
    ap.add_argument("--rust", action="store_true", help="Rust ValhallaBaseClient + wiki fetch (default)")
    ap.add_argument("--legacy", action="store_true", help="legacy Python subprocess path")
    ap.add_argument("--production", action="store_true", help="alias for --rust")
    ap.add_argument("--max-corpus-lines", type=int, default=0, help="0 = unlimited (algebra full ingest)")
    ap.add_argument("--offline-wiki", action="store_true")
    ap.add_argument(
        "--skip-preflight",
        action="store_true",
        help="run even when bundle has no Wikipedia extracts (score not citeable)",
    )
    add_benchmark_args(ap)
    args = ap.parse_args()
    apply_resource_budget()

    preflight = frames_bundle_preflight(BUNDLE)
    # Prefer already-enriched bundle (parasail/HF dump). Only hit network if not viable.
    if not preflight["viable"] and not args.offline_wiki and not wiki_reachable_quick():
        print("Wikipedia unreachable and bundle not enriched — trying prefetch …", flush=True)
        prefetch = subprocess.run(
            ["bash", str(REPO / "scripts/prefetch_frames_wiki.sh")],
            cwd=REPO,
            capture_output=True,
            text=True,
        )
        preflight = frames_bundle_preflight(BUNDLE)
        if prefetch.returncode != 0 and not preflight["viable"]:
            print("Prefetch failed — enabling --offline-wiki (Tier C until wiki cache populated)", flush=True)
            args.offline_wiki = True
    elif preflight["viable"] and not wiki_reachable_quick():
        print("FRAMES bundle already enriched (Tier A) — skipping live Wikipedia check", flush=True)
    if not args.skip_preflight and not preflight["viable"] and not args.offline_wiki:
        print(
            f"FRAMES preflight FAIL: {preflight['items_with_real_corpus']}/{preflight['n']} "
            f"items have wiki corpus (meta={preflight['wiki_items_with_extract_meta']}).",
            file=sys.stderr,
        )
        print(f"  Fix: {preflight['fix']}", file=sys.stderr)
        print("  Or: --skip-preflight (dev only — Tier C, do not cite)", file=sys.stderr)
        sys.exit(2)

    use_rust = args.rust or args.production or not args.legacy

    if use_rust:
        limit = args.limit or (5 if args.fast else 0)
        doc = run_frames_rust(
            fast=args.fast,
            limit=limit,
            max_corpus_lines=args.max_corpus_lines,
            offline_wiki=args.offline_wiki,
            verbose=args.verbose,
            checkpoint=args.checkpoint,
            checkpoint_every=args.checkpoint_every,
            resume=args.resume,
            pause_file=args.pause_file,
        )
        report = {
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "protocol": doc.get("protocol", "frames-v1-rust"),
            "benchmark": "FRAMES",
            "citation": doc.get("citation"),
            "bundle": BUNDLE,
            "n": doc.get("n"),
            "correct": doc.get("correct"),
            "accuracy": doc.get("accuracy"),
            "dynamic_fragments": doc.get("dynamic_fragments"),
            "fragment_mode": doc.get("fragment_mode"),
            "fragment_only": doc.get("fragment_only"),
            "wiki_items_fetched": doc.get("wiki_items_fetched"),
            "baseline_no_rag": doc.get("baseline_no_rag"),
            "baseline_oracle": doc.get("baseline_oracle"),
            "verdict": verdict_open_qa(float(doc.get("accuracy") or 0), strong=0.55, viable=0.35),
            "rows": doc.get("rows") or [],
        }
    else:
        doc = load_bundled(BUNDLE)
        items = doc["items"]
        if args.limit > 0 or args.fast:
            items = items[: args.limit or 5]

        print(f"=== {PROTOCOL} n={len(items)} source={doc['source']} (legacy) ===\n")
        rows: list[dict] = []
        correct = 0
        for item in items:
            base = make_base(fast=args.fast)
            corpus = item.get("corpus") or []
            if args.max_corpus_lines > 0 and len(corpus) > args.max_corpus_lines:
                corpus = split_context_to_corpus("\n\n".join(corpus), max_lines=args.max_corpus_lines)
            ans = run_open_item(base, item_id=item["id"], question=item["question"], corpus=corpus)
            ok = score_short_answer(ans, item["answer"])
            correct += int(ok)
            rows.append({"id": item["id"], "correct": ok, "answer": ans[:200], "gold": item["answer"]})
            print(f"  {'✓' if ok else '✗'} {item['id']}")

        n = len(items)
        acc = correct / max(n, 1)
        report = {
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "protocol": PROTOCOL,
            "benchmark": "FRAMES",
            "citation": doc.get("citation"),
            "bundle": BUNDLE,
            "n": n,
            "correct": correct,
            "accuracy": round(acc, 4),
            "baseline_no_rag": doc.get("baseline_no_rag"),
            "baseline_oracle": doc.get("baseline_oracle"),
            "verdict": verdict_open_qa(acc, strong=0.55, viable=0.35),
            "rows": rows,
        }

    REPORTS.mkdir(parents=True, exist_ok=True)
    out = REPORTS / "frames_v1_test.json"
    out.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"\n{report['correct']}/{report['n']} ({float(report['accuracy']):.1%}) → {out}")
    release_resources()


if __name__ == "__main__":
    main()
