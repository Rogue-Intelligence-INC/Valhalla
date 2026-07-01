#!/usr/bin/env python3
"""RAGTruth QA faithfulness — third-party RAG hallucination benchmark.

Protocol: ragtruth-v1

Dataset: wandb/RAGTruth-processed (ACL 2024 RAGTruth), bundled QA test slice.
Valhalla ingests context as session corpus, answers query, scored for:
  - context_support (answer tokens grounded in context)
  - reference_overlap (vs expert faithful reference_output)
  - faithful (combined gate)

Usage:
  python3 tools/valhalla_inference/test_ragtruth_v1.py
  python3 tools/valhalla_inference/test_ragtruth_v1.py --limit 10 --fast
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

from common import (  # noqa: E402
    load_bundled,
    score_ragtruth_item,
    split_context_to_corpus,
    verdict_ragtruth,
)
from valhalla_base import ValhallaBase  # noqa: E402

PROTOCOL = "ragtruth-v1"
REPORTS = REPO / "reports/valhalla_inference"
BUNDLE = "ragtruth_qa_test40.json"


def make_base(*, fast: bool) -> ValhallaBase:
    return ValhallaBase(
        decode="hybrid",
        follow_up_decode="native_follow_up_aware",
        fate_ingress_routing="quad_cycle",
        fate_qa_feedback=False,
        incubation_cycles=1 if fast else 2,
        max_new=128,
    )


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

    print(f"=== {PROTOCOL} n={len(items)} source={doc['source']} ===\n")

    rows: list[dict[str, Any]] = []
    faithful_n = 0
    abstain_n = 0
    label_faithful_n = 0
    label_halluc_n = 0

    for item in items:
        base = make_base(fast=args.fast)
        corpus = split_context_to_corpus(item["context"], max_lines=args.max_corpus_lines)
        base.set_context(corpus)
        out = base.run(
            [
                {
                    "id": str(item["id"]),
                    "text": item["query"],
                    "score_type": "open",
                }
            ]
        )
        ans = str((out.get("rows") or [{}])[0].get("answer") or "")
        sc = score_ragtruth_item(ans, item)
        faithful_n += int(sc["faithful"])
        abstain_n += int(sc["abstain"])
        if item.get("has_hallucination"):
            label_halluc_n += 1
        else:
            label_faithful_n += 1
        rows.append(
            {
                "id": item["id"],
                "query": item["query"][:100],
                "has_hallucination_label": item.get("has_hallucination"),
                "answer": ans[:200],
                **sc,
            }
        )
        mark = "✓" if sc["faithful"] else ("-" if sc["abstain"] else "✗")
        print(f"  {mark} {item['id']:6s} support={sc['context_support']:.2f} overlap={sc['reference_overlap']:.2f}")

    n = len(items)
    faithful_rate = faithful_n / max(n, 1)
    abstain_rate = abstain_n / max(n, 1)
    report = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "protocol": PROTOCOL,
        "benchmark": "RAGTruth (wandb/RAGTruth-processed)",
        "citation": "ACL 2024 — RAGTruth hallucination corpus",
        "bundle": BUNDLE,
        "n": n,
        "faithful": faithful_n,
        "faithful_rate": round(faithful_rate, 4),
        "abstain": abstain_n,
        "abstain_rate": round(abstain_rate, 4),
        "label_faithful_refs": label_faithful_n,
        "label_halluc_refs": label_halluc_n,
        "rows": rows,
        "verdict": verdict_ragtruth(faithful_rate, abstain_rate),
    }

    REPORTS.mkdir(parents=True, exist_ok=True)
    out_path = REPORTS / "ragtruth_v1_test.json"
    out_path.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(
        f"\nFaithful: {faithful_n}/{n} ({faithful_rate:.1%})  "
        f"abstain={abstain_n}  verdict={report['verdict']}"
    )
    print(f"→ {out_path}")


if __name__ == "__main__":
    main()
