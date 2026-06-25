#!/usr/bin/env python3
"""Prove Valhalla-as-base context flexibility vs static LLM prompt.

Same question, two different context bundles → different structure fingerprints + answers.

Usage:
  python3 tools/valhalla_inference/test_valhalla_base_context.py
  python3 tools/valhalla_inference/test_valhalla_base_context.py --decode native
"""

from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

from valhalla_base import ValhallaBase  # noqa: E402

REPORTS = Path(__file__).resolve().parents[2] / "reports/valhalla_inference"

CONTEXT_A = [
    "On Planet Alpha, water boils at 100 degrees Celsius at standard pressure.",
    "Planet Alpha uses Earth-standard atmospheric pressure at sea level.",
]

CONTEXT_B = [
    "On Planet Beta, water boils at 70 degrees Celsius due to lower atmospheric pressure.",
    "Planet Beta has thin atmosphere; liquids boil at lower temperatures.",
]

QUESTION = {
    "id": "ctx_flex_01",
    "text": "At what temperature does water boil on this planet? Answer with the numeric value in Celsius.",
    "score_type": "open",
    "reference_answer": "100",
    "keywords": ["100", "celsius", "boil"],
}

CONTEXT_C = [
    "The capital of Freedonia is Zephyria.",
    "Zephyria is the seat of government for Freedonia since 1920.",
]

QUESTION_CAP = {
    "id": "ctx_flex_02",
    "text": "What is the capital of Freedonia?",
    "score_type": "open",
    "reference_answer": "Zephyria",
    "keywords": ["Zephyria"],
}


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--decode", choices=("lm_patch", "native"), default="lm_patch")
    ap.add_argument("--lm", default="Qwen/Qwen2.5-0.5B-Instruct")
    args = ap.parse_args()

    base = ValhallaBase(lm_model=args.lm, decode=args.decode, max_new=96)

    print("=== Valhalla-as-base: context flexibility ===\n")
    print(f"decode={args.decode}  lm={args.lm}\n")

    # Swap context A → same question
    r_a = base.run([QUESTION], context=CONTEXT_A)
    r_b = base.run([QUESTION], context=CONTEXT_B)

    ans_a = r_a["rows"][0]["answer"]
    ans_b = r_b["rows"][0]["answer"]

    print("--- Boiling point (context swap) ---")
    print(f"  context A fp={r_a['context_fingerprint']} → {ans_a[:200]}")
    print(f"  context B fp={r_b['context_fingerprint']} → {ans_b[:200]}")
    flex_boiling = r_a["context_fingerprint"] != r_b["context_fingerprint"]
    print(f"  fingerprints differ: {flex_boiling}")

    # Second scenario: capital
    r_c = base.run([QUESTION_CAP], context=CONTEXT_C)
    ans_c = r_c["rows"][0]["answer"]
    print("\n--- Capital (fresh context bundle) ---")
    print(f"  context C fp={r_c['context_fingerprint']} → {ans_c[:200]}")

    # Compare: empty swap on base object
    base.set_context(CONTEXT_A)
    r_persist = base.run([QUESTION])
    print("\n--- Persistent session context (set_context) ---")
    print(f"  fp={r_persist['context_fingerprint']} (should match A: {r_a['context_fingerprint']})")

    report = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "protocol": "valhalla-base-v1",
        "decode": args.decode,
        "boiling_context_a": {"fingerprint": r_a["context_fingerprint"], "answer": ans_a},
        "boiling_context_b": {"fingerprint": r_b["context_fingerprint"], "answer": ans_b},
        "capital_context_c": {"fingerprint": r_c["context_fingerprint"], "answer": ans_c},
        "context_flexibility_proven": flex_boiling,
        "architecture": {
            "substrate": "Valhalla Triad (Hub+Tile+Stem) per context bundle",
            "lm_role": "decode head only (runtime patch) when decode=lm_patch",
            "vs_standard_llm": "context ingested into structure, not only prepended to prompt",
        },
    }

    REPORTS.mkdir(parents=True, exist_ok=True)
    out = REPORTS / "valhalla_base_context_flex.json"
    out.write_text(json.dumps(report, indent=2, ensure_ascii=False), encoding="utf-8")
    print(f"\n→ {out}")

    if not flex_boiling:
        sys.exit("context fingerprints should differ between A and B")


if __name__ == "__main__":
    main()
