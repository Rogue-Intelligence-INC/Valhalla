#!/usr/bin/env python3
"""LongBench-v2 short MCQ — recognized long-context benchmark (truncated slice).

Protocol: longbench-v2-v1

Dataset: THUDM/LongBench-v2 (503 MCQ, human ~53.7%), bundled short-length slice.
Context truncated to <=12k chars, split into session corpus lines (max 32).
Hybrid decode routes MCQ to lm_patch logprob.

Usage:
  python3 tools/valhalla_inference/test_longbench_v2_v1.py
  python3 tools/valhalla_inference/test_longbench_v2_v1.py --limit 5 --fast
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

REPO = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(REPO / "tools/fair_benchmark"))
sys.path.insert(0, str(REPO / "tools/standard_benchmarks"))

from common import format_mcq_prompt, load_bundled, split_context_to_corpus, verdict_longbench  # noqa: E402
from metrics import score_answer  # noqa: E402
from valhalla_base import ValhallaBase  # noqa: E402

PROTOCOL = "longbench-v2-v1"
REPORTS = REPO / "reports/valhalla_inference"
BUNDLE = "longbench_v2_mcq_short20.json"


def make_base(*, fast: bool) -> ValhallaBase:
    return ValhallaBase(
        decode="hybrid",
        mcq_decode="lm_patch",
        follow_up_decode="native_follow_up_aware",
        fate_ingress_routing="quad_cycle",
        fate_qa_feedback=False,
        incubation_cycles=1 if fast else 2,
        max_new=32,
    )


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--limit", type=int, default=0)
    ap.add_argument("--fast", action="store_true")
    ap.add_argument("--max-corpus-lines", type=int, default=32)
    args = ap.parse_args()

    os.environ.setdefault("VALHALLA_NATIVE_MAX_CORPUS_LINES", str(args.max_corpus_lines))

    doc = load_bundled(BUNDLE)
    items = doc["items"]
    if args.limit > 0:
        items = items[: args.limit]

    print(f"=== {PROTOCOL} n={len(items)} source={doc['source']} ===\n")

    rows: list[dict[str, Any]] = []
    correct = 0
    for item in items:
        base = make_base(fast=args.fast)
        corpus = split_context_to_corpus(item["context"], max_lines=args.max_corpus_lines)
        base.set_context(corpus)
        prompt = format_mcq_prompt(item)
        out = base.run(
            [
                {
                    "id": str(item["_id"]),
                    "text": prompt,
                    "score_type": "mcq",
                    "reference_answer": item["answer"],
                }
            ]
        )
        ans = str((out.get("rows") or [{}])[0].get("answer") or "")
        sc = score_answer(ans, {"reference_answer": item["answer"], "score_type": "mcq"})
        ok = bool(sc["correct"])
        correct += int(ok)
        rows.append(
            {
                "id": item["_id"],
                "domain": item.get("domain"),
                "difficulty": item.get("difficulty"),
                "context_truncated": item.get("context_truncated", False),
                "correct": ok,
                "extracted": sc.get("extracted"),
                "reference": item["answer"],
                "answer": ans[:120],
            }
        )
        print(f"  {'✓' if ok else '✗'} {item['_id'][:12]} {item.get('domain','')[:20]}")

    n = len(items)
    acc = correct / max(n, 1)
    report = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "protocol": PROTOCOL,
        "benchmark": "LongBench-v2 (THUDM)",
        "citation": "LongBench v2 — 503 long-context MCQ, human baseline ~53.7%",
        "bundle": BUNDLE,
        "context_note": "short length slice, context truncated to <=12k chars, max 32 corpus lines",
        "human_baseline_short": 0.501,
        "n": n,
        "correct": correct,
        "accuracy": round(acc, 4),
        "rows": rows,
        "verdict": verdict_longbench(acc),
    }

    REPORTS.mkdir(parents=True, exist_ok=True)
    out_path = REPORTS / "longbench_v2_v1_test.json"
    out_path.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"\nAccuracy: {correct}/{n} ({acc:.1%})  verdict={report['verdict']}")
    print(f"→ {out_path}")


if __name__ == "__main__":
    main()
