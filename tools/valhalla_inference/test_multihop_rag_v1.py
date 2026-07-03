#!/usr/bin/env python3
"""MultiHop-RAG — COLM 2024 multi-hop news QA.

Protocol: multihop-rag-v1 (v2 when --v2)

Usage:
  python3 tools/valhalla_inference/test_multihop_rag_v1.py --fast
  python3 tools/valhalla_inference/test_multihop_rag_v1.py --v2
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

from benchmark_runner import make_base, run_open_item  # noqa: E402
from common import load_bundled, score_short_answer, split_context_to_corpus, verdict_open_qa  # noqa: E402

PROTOCOL = "multihop-rag-v1"
REPORTS = REPO / "reports/valhalla_inference"
BUNDLE = "multihop_rag_slice25.json"


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--limit", type=int, default=0)
    ap.add_argument("--fast", action="store_true")
    ap.add_argument("--v2", action="store_true", help="Rust multihop_v2 structure composition")
    ap.add_argument("--max-corpus-lines", type=int, default=32)
    args = ap.parse_args()

    if args.v2:
        cmd = [
            str(REPO / "target/release/valhalla_base"),
            "demo",
            "multihop",
            "--repo",
            str(REPO),
        ]
        if args.fast:
            cmd.append("--fast")
        if args.limit > 0:
            cmd.extend(["--limit", str(args.limit)])
        subprocess.run(cmd, check=True)
        return

    doc = load_bundled(BUNDLE)
    items = [i for i in doc["items"] if i.get("corpus")]
    if args.limit > 0 or args.fast:
        items = items[: args.limit or 5]

    print(f"=== {PROTOCOL} n={len(items)} ===\n")
    rows = []
    correct = 0
    for item in items:
        base = make_base(fast=args.fast)
        corpus = item.get("corpus") or []
        if len(corpus) > args.max_corpus_lines:
            corpus = corpus[: args.max_corpus_lines]
        ans = run_open_item(base, item_id=item["id"], question=item["question"], corpus=corpus)
        ok = score_short_answer(ans, item["answer"])
        correct += int(ok)
        rows.append({"id": item["id"], "correct": ok, "answer": ans[:200]})
        print(f"  {'✓' if ok else '✗'} {item['id']}")

    n = len(items)
    acc = correct / max(n, 1)
    report = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "protocol": PROTOCOL,
        "benchmark": "MultiHop-RAG",
        "bundle": BUNDLE,
        "n": n,
        "correct": correct,
        "accuracy": round(acc, 4),
        "verdict": verdict_open_qa(acc),
        "rows": rows,
    }
    REPORTS.mkdir(parents=True, exist_ok=True)
    out = REPORTS / "multihop_rag_v1_test.json"
    out.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"\n{correct}/{n} ({acc:.1%}) → {out}")


if __name__ == "__main__":
    main()
