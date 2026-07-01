#!/usr/bin/env python3
"""LoCoMo long conversational memory — Snap ACL 2024.

Protocol: locomo-v1
Session built via append_context over full dialogue (300+ turns avg in full set).

Usage:
  python3 tools/valhalla_inference/test_locomo_v1.py --fast
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
from common import load_bundled, score_short_answer, verdict_open_qa  # noqa: E402

PROTOCOL = "locomo-v1"
REPORTS = REPO / "reports/valhalla_inference"
BUNDLE = "locomo_qa_slice30.json"


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--limit", type=int, default=0)
    ap.add_argument("--fast", action="store_true")
    ap.add_argument("--max-lines", type=int, default=64, help="cap session lines per conversation")
    args = ap.parse_args()

    doc = load_bundled(BUNDLE)
    items = doc["items"]
    if args.limit > 0 or args.fast:
        items = items[: args.limit or 5]

    print(f"=== {PROTOCOL} n={len(items)} ===\n")
    rows = []
    correct = 0
    for item in items:
        base = make_base(fast=args.fast)
        lines = list(item.get("session_lines") or [])[: args.max_lines]
        ans = run_open_session_append(
            base, item_id=item["id"], question=item["question"], lines=lines
        )
        ok = score_short_answer(ans, str(item["answer"]))
        correct += int(ok)
        rows.append(
            {
                "id": item["id"],
                "correct": ok,
                "answer": ans[:200],
                "gold": item["answer"],
                "session_lines_used": len(lines),
            }
        )
        print(f"  {'✓' if ok else '✗'} {item['id']} lines={len(lines)}")

    n = len(items)
    acc = correct / max(n, 1)
    report = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "protocol": PROTOCOL,
        "benchmark": "LoCoMo",
        "citation": doc.get("citation"),
        "bundle": BUNDLE,
        "n": n,
        "correct": correct,
        "accuracy": round(acc, 4),
        "max_lines": args.max_lines,
        "verdict": verdict_open_qa(acc, strong=0.50, viable=0.25),
        "rows": rows,
    }
    REPORTS.mkdir(parents=True, exist_ok=True)
    out = REPORTS / "locomo_v1_test.json"
    out.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"\n{correct}/{n} ({acc:.1%}) → {out}")


if __name__ == "__main__":
    main()
