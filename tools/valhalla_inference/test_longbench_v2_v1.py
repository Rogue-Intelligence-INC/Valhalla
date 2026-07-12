#!/usr/bin/env python3
"""LongBench-v2 short MCQ — recognized long-context benchmark (truncated slice).

Protocol: longbench-v2-v1 · production: longbench-v2-fragmentwithfate-v1

Dataset: THUDM/LongBench-v2 (503 MCQ, human ~53.7%), bundled short-length slice.
Context truncated to <=12k chars, split into session corpus lines (max 32).
Production uses dynamic fragment prelearn (fate-pipe), not legacy 4-line fate_warmup.
Default: unlimited corpus lines + mcq-light ingest (warmup only, no multi-pass pretrain).

Usage:
  python3 tools/valhalla_inference/test_longbench_v2_v1.py --fragment-warmup
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

from benchmark_env import apply_resource_budget, default_lm_model, release_resources  # noqa: E402
from benchmark_runner import make_base, run_longbench_rust  # noqa: E402
from common import format_mcq_prompt, load_bundled, split_context_to_corpus, verdict_longbench  # noqa: E402
from metrics import score_answer  # noqa: E402

PROTOCOL = "longbench-v2-v1"
REPORTS = REPO / "reports" / "valhalla_inference"
BUNDLE = "longbench_v2_mcq_short20.json"


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--limit", type=int, default=0)
    ap.add_argument("--fast", action="store_true")
    ap.add_argument(
        "--fragment-warmup",
        action="store_true",
        help="Rust fragmentwithfate prelearn (recommended; fixes legacy warmup)",
    )
    ap.add_argument(
        "--production",
        action="store_true",
        help="alias for --fragment-warmup",
    )
    ap.add_argument("--legacy", action="store_true", help="legacy Python subprocess path")
    ap.add_argument("--max-corpus-lines", type=int, default=0, help="0 = unlimited")
    ap.add_argument(
        "--heavy-pretrain",
        action="store_true",
        help="full fate-pipe pretrain (slower; ablation vs default mcq-light)",
    )
    args = ap.parse_args()

    os.environ.setdefault("VALHALLA_DYNAMIC_CORPUS_FRAGMENTS", "1")
    apply_resource_budget()

    use_rust = args.fragment_warmup or args.production or not args.legacy

    if use_rust:
        limit = args.limit or (5 if args.fast else 0)
        doc = run_longbench_rust(
            fast=args.fast,
            limit=limit,
            max_corpus_lines=args.max_corpus_lines,
            low_resource=not args.fast,
            mcq_light=not args.heavy_pretrain,
        )
        report = {
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "protocol": doc.get("protocol", "longbench-v2-fragmentwithfate-v1"),
            "benchmark": "LongBench-v2 (THUDM)",
            "citation": "LongBench v2 — 503 long-context MCQ, human baseline ~53.7%",
            "bundle": BUNDLE,
            "context_note": "short length slice, context truncated to <=12k chars",
            "warmup_mode": doc.get("warmup_mode", "fragmentwithfate"),
            "dynamic_fragments": doc.get("dynamic_fragments"),
            "fragment_mode": doc.get("fragment_mode"),
            "human_baseline_short": 0.501,
            "n": doc.get("n"),
            "correct": doc.get("correct"),
            "accuracy": doc.get("accuracy"),
            "rows": doc.get("rows"),
            "verdict": verdict_longbench(float(doc.get("accuracy") or 0)),
            "runtime": "rust",
        }
        REPORTS.mkdir(parents=True, exist_ok=True)
        out_path = REPORTS / "longbench_v2_v1_test.json"
        out_path.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
        print(
            f"\nAccuracy: {report['correct']}/{report['n']} "
            f"({float(report['accuracy']):.1%})  warmup={report['warmup_mode']}  "
            f"verdict={report['verdict']}"
        )
        print(f"→ {out_path}")
        release_resources()
        return 0

    doc = load_bundled(BUNDLE)
    items = doc["items"]
    if args.limit > 0:
        items = items[: args.limit]

    print(f"=== {PROTOCOL} legacy n={len(items)} ===\n")
    rows: list[dict[str, Any]] = []
    correct = 0
    base = make_base(fast=args.fast, lm_model=default_lm_model())
    for item in items:
        corpus = split_context_to_corpus(item["context"], max_lines=args.max_corpus_lines)
        prompt = format_mcq_prompt(item)
        base.set_context(corpus)
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
        print(f"  {'✓' if ok else '✗'} {item['_id'][:12]} {item.get('domain', '')[:20]}")

    release_resources()
    n = len(items)
    acc = correct / max(n, 1)
    report = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "protocol": PROTOCOL,
        "benchmark": "LongBench-v2 (THUDM)",
        "bundle": BUNDLE,
        "warmup_mode": "legacy_python",
        "n": n,
        "correct": correct,
        "accuracy": round(acc, 4),
        "rows": rows,
        "verdict": verdict_longbench(acc),
        "runtime": "python-legacy",
    }
    REPORTS.mkdir(parents=True, exist_ok=True)
    out_path = REPORTS / "longbench_v2_v1_test.json"
    out_path.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"\nAccuracy: {correct}/{n} ({acc:.1%})  verdict={report['verdict']}")
    print(f"→ {out_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
