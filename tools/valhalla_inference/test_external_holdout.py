#!/usr/bin/env python3
"""External holdout — out-of-suite prompts + corpus (not fair_benchmark ingest_clean).

Protocol: external-holdout-v1

Uses tools/valhalla_model_bridge/external_domain_prompts.json to test
generalisation beyond fair-1.2 open_retrieval slice.

Usage:
  python3 tools/valhalla_inference/test_external_holdout.py
  python3 tools/valhalla_inference/test_external_holdout.py --limit 12
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
sys.path.insert(0, str(REPO / "tools/fair_benchmark"))

from metrics import score_answer  # noqa: E402
from run_scale_benchmark import corpus_for_prompt  # noqa: E402
from valhalla_base import ValhallaBase  # noqa: E402

PROTOCOL = "external-holdout-v1"
REPORTS = REPO / "reports/valhalla_inference"
SUITE = REPO / "tools/valhalla_model_bridge/external_domain_prompts.json"


def make_base() -> ValhallaBase:
    return ValhallaBase(
        decode="hybrid",
        follow_up_decode="native_follow_up_aware",
        fate_ingress_routing="quad_cycle",
        fate_qa_feedback=False,
        incubation_cycles=2,
        max_new=128,
    )


def q_payload(p: dict) -> dict:
    return {
        "id": p["id"],
        "text": p["text"],
        "score_type": "open",
        "reference_answer": p.get("reference_answer", ""),
        "keywords": p.get("keywords") or [],
    }


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--limit", type=int, default=0)
    args = ap.parse_args()

    doc = json.loads(SUITE.read_text(encoding="utf-8"))
    corpus = doc["corpus"]
    prompts = doc["prompts"]
    if args.limit > 0:
        prompts = prompts[: args.limit]

    print(f"=== {PROTOCOL} n={len(prompts)} corpus={len(corpus)} ===\n")

    rows: list[dict[str, Any]] = []
    correct = 0
    for p in prompts:
        base = make_base()
        ctx = corpus_for_prompt(p, corpus, k=8)
        base.set_context(ctx)
        out = base.run([q_payload(p)])
        row = (out.get("rows") or [{}])[0]
        ans = str(row.get("answer") or "")
        sc = score_answer(ans, p)
        ok = bool(sc["correct"])
        correct += int(ok)
        rows.append(
            {
                "id": p["id"],
                "domain": p.get("domain"),
                "correct": ok,
                "answer": ans[:180],
                "context_lines": len(ctx),
            }
        )
        print(f"  {'✓' if ok else '✗'} {p['id']:16s} {p.get('domain', '')}")

    n = len(prompts)
    acc = correct / max(n, 1)
    report = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "protocol": PROTOCOL,
        "source": str(SUITE.relative_to(REPO)),
        "n": n,
        "correct": correct,
        "accuracy": acc,
        "rows": rows,
        "verdict": (
            "EXTERNAL_HOLDOUT_STRONG"
            if acc >= 0.85
            else "EXTERNAL_HOLDOUT_VIABLE"
            if acc >= 0.65
            else "EXTERNAL_HOLDOUT_DEVELOPING"
        ),
    }

    REPORTS.mkdir(parents=True, exist_ok=True)
    out_path = REPORTS / "external_holdout_test.json"
    out_path.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"\nAccuracy: {correct}/{n} ({acc:.1%})  verdict={report['verdict']}")
    print(f"→ {out_path}")


if __name__ == "__main__":
    main()
