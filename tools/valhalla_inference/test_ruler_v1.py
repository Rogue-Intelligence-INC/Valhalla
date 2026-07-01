#!/usr/bin/env python3
"""RULER-style synthetic long-context retrieval (single + multi needle).

Protocol: ruler-v1

Usage:
  python3 tools/valhalla_inference/test_ruler_v1.py --fast
"""

from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(REPO / "tools/standard_benchmarks"))

from benchmark_runner import make_base, run_open_session_append  # noqa: E402
from common import load_bundled, verdict_needle  # noqa: E402

PROTOCOL = "ruler-v1"
REPORTS = REPO / "reports/valhalla_inference"
BUNDLE = "ruler_v1_slice20.json"


def recall_ok(answer: str, hints: list[str]) -> bool:
    low = answer.lower()
    return any(h.lower() in low for h in hints if h)


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--limit", type=int, default=0)
    ap.add_argument("--fast", action="store_true")
    args = ap.parse_args()

    doc = load_bundled(BUNDLE)
    items = doc["items"]
    if args.limit > 0 or args.fast:
        items = items[: args.limit or 4]

    print(f"=== {PROTOCOL} n={len(items)} ===\n")
    rows = []
    correct = 0
    for item in items:
        base = make_base(fast=args.fast)
        lines = item.get("session_lines") or []
        ans = run_open_session_append(
            base, item_id=item["id"], question=item["question"], lines=lines
        )
        hints = item.get("hints") or []
        ok = recall_ok(ans, hints)
        correct += int(ok)
        rows.append(
            {
                "id": item["id"],
                "task": item.get("task"),
                "depth": item.get("depth"),
                "correct": ok,
                "answer": ans[:120],
            }
        )
        print(f"  {'✓' if ok else '✗'} {item['id']} depth={item.get('depth')} {item.get('task')}")

    n = len(items)
    recall = correct / max(n, 1)
    report = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "protocol": PROTOCOL,
        "benchmark": "RULER-style synthetic",
        "bundle": BUNDLE,
        "n": n,
        "correct": correct,
        "recall": round(recall, 4),
        "verdict": verdict_needle(recall),
        "rows": rows,
    }
    REPORTS.mkdir(parents=True, exist_ok=True)
    out = REPORTS / "ruler_v1_test.json"
    out.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"\n{correct}/{n} ({recall:.1%}) → {out}")


if __name__ == "__main__":
    main()
