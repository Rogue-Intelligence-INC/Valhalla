#!/usr/bin/env python3
"""FRAMES multi-hop RAG — Google NAACL 2025.

Protocol: frames-v1
Baseline (paper): naive 40.8% · oracle 72.9% · multi-step 66%

Usage:
  python3 tools/valhalla_inference/test_frames_v1.py --fast
"""

from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

REPO = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(REPO / "tools/standard_benchmarks"))

from benchmark_runner import make_base, run_open_item  # noqa: E402
from common import load_bundled, score_short_answer, split_context_to_corpus, verdict_open_qa  # noqa: E402

PROTOCOL = "frames-v1"
REPORTS = REPO / "reports/valhalla_inference"
BUNDLE = "frames_slice25.json"


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--limit", type=int, default=0)
    ap.add_argument("--fast", action="store_true")
    ap.add_argument("--max-corpus-lines", type=int, default=32)
    args = ap.parse_args()

    doc = load_bundled(BUNDLE)
    items = doc["items"]
    if args.limit > 0 or args.fast:
        items = items[: args.limit or 5]

    print(f"=== {PROTOCOL} n={len(items)} source={doc['source']} ===\n")
    rows: list[dict] = []
    correct = 0
    for item in items:
        base = make_base(fast=args.fast)
        corpus = item.get("corpus") or []
        if len(corpus) > args.max_corpus_lines:
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
    print(f"\n{correct}/{n} ({acc:.1%}) → {out}")


if __name__ == "__main__":
    main()
