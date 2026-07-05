#!/usr/bin/env python3
"""LaRA-style RAG vs long-context — structure RAG vs trad LC prompt.

Protocol: lara-v1
Arms: valhalla_hybrid (rag_corpus) vs trad_lm (long_context in prompt).

Usage:
  .venv-llm/bin/python3 tools/valhalla_inference/test_lara_v1.py --fast --with-trad
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
sys.path.insert(0, str(REPO / "tools/fair_benchmark"))
sys.path.insert(0, str(REPO / "tools/standard_benchmarks"))

from benchmark_runner import make_base, run_open_item  # noqa: E402
from common import load_bundled, score_short_answer, verdict_rag_vs_lc  # noqa: E402
from test_context_arch_compare import resolve_lm_path  # noqa: E402
from test_context_performance import context_block  # noqa: E402

PROTOCOL = "lara-v1"
REPORTS = REPO / "reports/valhalla_inference"
BUNDLE = "lara_rag_vs_lc_slice20.json"


def run_trad_lc(item: dict, *, lm_model: str) -> str:
    from loader import ValhallaLLM  # noqa: E402

    lm = ValhallaLLM(resolve_lm_path(lm_model), engine="transformers")
    ctx = "\n".join(item.get("long_context_lines") or [])
    prompt = context_block([ctx]) + item["question"]
    return lm.generate([prompt], max_tokens=128)[0].text.strip()


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--limit", type=int, default=0)
    ap.add_argument("--fast", action="store_true")
    ap.add_argument("--with-trad", action="store_true")
    from benchmark_env import default_lm_model  # noqa: E402

    ap.add_argument("--lm-model", default=None)
    ap.add_argument("--legacy-v2", action="store_true", help="enable legacy rule multihop_v2 ablation")
    ap.add_argument("--no-open-fate", action="store_true", help="disable global open Fate")
    args = ap.parse_args()
    lm_model = args.lm_model or default_lm_model()

    if args.legacy_v2:
        os.environ.setdefault("VALHALLA_MULTIHOP_V2", "1")
    if args.no_open_fate:
        os.environ["VALHALLA_OPEN_FATE"] = "0"

    doc = load_bundled(BUNDLE)
    items = doc["items"]
    if args.limit > 0 or args.fast:
        items = items[: args.limit or 5]

    print(f"=== {PROTOCOL} n={len(items)} ===\n")
    rag_rows, lc_rows = [], []
    rag_ok = lc_ok = 0
    lm = None
    if args.with_trad:
        try:
            from loader import ValhallaLLM  # noqa: E402

            lm = ValhallaLLM(resolve_lm_path(lm_model), engine="transformers")
        except Exception as exc:  # noqa: BLE001
            print(f"trad SKIP: {exc}\n")
            lm = None

    for item in items:
        base = make_base(fast=args.fast, lm_model=lm_model)
        corpus = item.get("rag_corpus") or []
        ans_rag = run_open_item(base, item_id=item["id"], question=item["question"], corpus=corpus)
        ok_rag = score_short_answer(ans_rag, item["answer"])
        rag_ok += int(ok_rag)
        rag_rows.append({"id": item["id"], "correct": ok_rag, "answer": ans_rag[:120]})

        ok_lc = False
        ans_lc = ""
        if lm is not None:
            ctx = "\n".join(item.get("long_context_lines") or [])
            prompt = context_block([ctx]) + item["question"]
            ans_lc = lm.generate([prompt], max_tokens=128)[0].text.strip()
            ok_lc = score_short_answer(ans_lc, item["answer"])
            lc_ok += int(ok_lc)
            lc_rows.append({"id": item["id"], "correct": ok_lc, "answer": ans_lc[:120]})

        print(f"  rag={'✓' if ok_rag else '✗'} lc={'✓' if ok_lc else '-'} {item['id']}")

    n = len(items)
    rag_acc = rag_ok / max(n, 1)
    lc_acc = lc_ok / max(n, 1) if lc_rows else None
    report = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "protocol": PROTOCOL,
        "benchmark": "LaRA-style RAG vs LC",
        "bundle": BUNDLE,
        "n": n,
        "valhalla_rag": {"correct": rag_ok, "accuracy": round(rag_acc, 4), "rows": rag_rows},
        "trad_long_context": (
            {"correct": lc_ok, "accuracy": round(lc_acc, 4), "rows": lc_rows} if lc_rows else None
        ),
        "verdict": verdict_rag_vs_lc(rag_acc, lc_acc or 0.0) if lc_rows else "RAG_ONLY",
    }
    REPORTS.mkdir(parents=True, exist_ok=True)
    out = REPORTS / "lara_v1_test.json"
    out.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"\nRAG {rag_ok}/{n} ({rag_acc:.1%})" + (f" · LC {lc_ok}/{n} ({lc_acc:.1%})" if lc_rows else ""))
    print(f"→ {out}")


if __name__ == "__main__":
    main()
