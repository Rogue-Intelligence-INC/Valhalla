#!/usr/bin/env python3
"""LoCoMo — Rust production path (dynamic density fragments + long-chain ingest).

Usage:
  python3 tools/valhalla_inference/test_locomo_v1.py --rust
  python3 tools/valhalla_inference/test_locomo_v1.py --rust --fast --limit 3
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(REPO / "tools/standard_benchmarks"))

from benchmark_cli import add_benchmark_args  # noqa: E402
from benchmark_env import apply_resource_budget, release_resources  # noqa: E402
from benchmark_runner import make_base, run_locomo_rust, run_open_session_append  # noqa: E402
from common import load_bundled, score_short_answer, verdict_open_qa  # noqa: E402

PROTOCOL = "locomo-v1"
REPORTS = REPO / "reports/valhalla_inference"
BUNDLE = "locomo_qa_slice30.json"


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--limit", type=int, default=0)
    ap.add_argument("--fast", action="store_true")
    ap.add_argument("--rust", action="store_true", help="Rust ValhallaBaseClient + dynamic fragments (default)")
    ap.add_argument("--legacy", action="store_true", help="legacy Python subprocess path")
    ap.add_argument("--production", action="store_true", help="alias for --rust")
    ap.add_argument("--max-lines", type=int, default=64)
    ap.add_argument("--chunk-size", type=int, default=0, help="0 = algebra-derived from ingress budget")
    add_benchmark_args(ap)
    args = ap.parse_args()
    apply_resource_budget()

    use_rust = args.rust or args.production or not args.legacy

    if use_rust:
        os.environ.setdefault("VALHALLA_FATE_PARALLEL_LCHAIN", "1")
        limit = args.limit or (3 if args.fast else 0)
        doc = run_locomo_rust(
            fast=args.fast,
            limit=limit,
            max_lines=args.max_lines,
            chunk_size=args.chunk_size,
            verbose=args.verbose,
            checkpoint=args.checkpoint,
            checkpoint_every=args.checkpoint_every,
            resume=args.resume,
            pause_file=args.pause_file,
        )
        report = {
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "protocol": doc.get("protocol", "locomo-v1-rust"),
            "benchmark": "LoCoMo",
            "bundle": BUNDLE,
            "n": doc.get("n"),
            "correct": doc.get("correct"),
            "accuracy": doc.get("accuracy"),
            "dynamic_fragments": doc.get("dynamic_fragments"),
            "fragment_mode": doc.get("fragment_mode"),
            "max_lines": args.max_lines,
            "chunk_size": args.chunk_size,
            "rows": doc.get("rows"),
            "verdict": verdict_open_qa(float(doc.get("accuracy") or 0)),
            "runtime": "rust",
        }
        REPORTS.mkdir(parents=True, exist_ok=True)
        out = REPORTS / "locomo_v1_test.json"
        out.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
        print(f"\n{report['correct']}/{report['n']} ({float(report['accuracy']):.1%}) → {out}")
        release_resources()
        return

    doc = load_bundled(BUNDLE)
    items = doc["items"]
    if args.limit > 0 or args.fast:
        items = items[: args.limit or 5]

    print(f"=== {PROTOCOL} legacy n={len(items)} ===\n")
    rows = []
    correct = 0
    for item in items:
        base = make_base(fast=args.fast, legacy=True)
        lines = list(item.get("session_lines") or [])[: args.max_lines]
        ans = run_open_session_append(
            base, item_id=item["id"], question=item["question"], lines=lines, production=False
        )
        ok = score_short_answer(ans, str(item["answer"]))
        correct += int(ok)
        rows.append({"id": item["id"], "correct": ok, "answer": ans[:200]})
        print(f"  {'✓' if ok else '✗'} {item['id']} lines={len(lines)}")

    n = len(items)
    acc = correct / max(n, 1)
    report = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "protocol": PROTOCOL,
        "runtime": "python-legacy",
        "n": n,
        "correct": correct,
        "accuracy": round(acc, 4),
        "rows": rows,
    }
    REPORTS.mkdir(parents=True, exist_ok=True)
    out = REPORTS / "locomo_v1_test.json"
    out.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"\n{correct}/{n} ({acc:.1%}) → {out}")


if __name__ == "__main__":
    main()
