#!/usr/bin/env python3
"""RAGTruth QA — expanded 100-item faithfulness slice.

Protocol: ragtruth-full-v1

Usage:
  python3 tools/valhalla_inference/test_ragtruth_full_v1.py --fast
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

from benchmark_runner import make_base, run_open_item  # noqa: E402
from common import (  # noqa: E402
    load_bundled,
    score_ragtruth_item,
    split_context_to_corpus,
    verdict_ragtruth,
)

PROTOCOL = "ragtruth-full-v1"
REPORTS = REPO / "reports/valhalla_inference"
BUNDLE = "ragtruth_qa_test100.json"


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--limit", type=int, default=0)
    ap.add_argument("--fast", action="store_true")
    ap.add_argument("--max-corpus-lines", type=int, default=32)
    args = ap.parse_args()

    doc = load_bundled(BUNDLE)
    items = doc["items"]
    if args.limit > 0:
        items = items[: args.limit]
    elif args.fast:
        items = items[:10]

    print(f"=== {PROTOCOL} n={len(items)} ===\n")
    rows = []
    faithful_n = abstain_n = 0
    for item in items:
        base = make_base(fast=args.fast)
        corpus = split_context_to_corpus(item["context"], max_lines=args.max_corpus_lines)
        ans = run_open_item(base, item_id=str(item["id"]), question=item["query"], corpus=corpus)
        sc = score_ragtruth_item(ans, item)
        faithful_n += int(sc["faithful"])
        abstain_n += int(sc["abstain"])
        rows.append({"id": item["id"], "faithful": sc["faithful"], "abstain": sc["abstain"], "answer": ans[:120]})
        print(f"  {'✓' if sc['faithful'] else ('-' if sc['abstain'] else '✗')} {item['id']}")

    n = len(items)
    fr = faithful_n / max(n, 1)
    report = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "protocol": PROTOCOL,
        "benchmark": "RAGTruth expanded",
        "bundle": BUNDLE,
        "n": n,
        "faithful": faithful_n,
        "faithful_rate": round(fr, 4),
        "abstain": abstain_n,
        "verdict": verdict_ragtruth(fr, abstain_n / max(n, 1)),
        "rows": rows,
    }
    REPORTS.mkdir(parents=True, exist_ok=True)
    out = REPORTS / "ragtruth_full_v1_test.json"
    out.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"\n{faithful_n}/{n} faithful ({fr:.1%}) → {out}")


if __name__ == "__main__":
    main()
