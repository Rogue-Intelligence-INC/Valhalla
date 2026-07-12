#!/usr/bin/env python3
"""MultiHop-RAG — COLM 2024 multi-hop news QA.

Protocol: multihop-rag-v1 (v2 / fate / hybrid via Rust valhalla_base demo)

Usage:
  python3 tools/valhalla_inference/test_multihop_rag_v1.py --fast
  python3 tools/valhalla_inference/test_multihop_rag_v1.py --v2
  python3 tools/valhalla_inference/test_multihop_rag_v1.py --fate
  python3 tools/valhalla_inference/test_multihop_rag_v1.py --hybrid
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(REPO / "tools/standard_benchmarks"))

from benchmark_env import apply_gpu_env, apply_resource_budget, release_resources  # noqa: E402

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
    ap.add_argument("--fate", action="store_true", help="Rust multihop_fate hop-wise adaptation")
    ap.add_argument("--fatewithv2", action="store_true", help="Rust multihop_fate + multihop_v2 (fatewithv2)")
    ap.add_argument("--hybrid", action="store_true", help=argparse.SUPPRESS)  # legacy alias
    ap.add_argument("--max-corpus-lines", type=int, default=0, help="0 = unlimited (algebra full ingest)")
    args = ap.parse_args()
    apply_resource_budget()
    apply_gpu_env()

    rust_demo = None
    if args.fatewithv2 or args.hybrid:
        rust_demo = "multihop-hybrid"
    elif args.fate:
        rust_demo = "multihop-fate"
    elif args.v2:
        rust_demo = "multihop"

    if rust_demo:
        cmd = [
            str(REPO / "target/release/valhalla_base"),
            "demo",
            rust_demo,
            "--repo",
            str(REPO),
        ]
        if args.fast:
            cmd.append("--fast")
        if args.limit > 0:
            cmd.extend(["--limit", str(args.limit)])
        subprocess.run(cmd, check=True, env={**os.environ, "RUSTFLAGS": "-L /opt/cuda/lib64"})
        release_resources()
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
        if args.max_corpus_lines > 0 and len(corpus) > args.max_corpus_lines:
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
