#!/usr/bin/env python3
"""Validate VPB memory suite: counts, evidence ids, abstention has no leaks."""

from __future__ import annotations

import json
import sys
from pathlib import Path

FAIR = Path(__file__).resolve().parent


def main() -> int:
    suite = json.loads((FAIR / "vpb_memory_suite.json").read_text(encoding="utf-8"))
    tiers = json.loads((FAIR / "vpb_corpus_tiers.json").read_text(encoding="utf-8"))
    idx = tiers["line_index_M"]
    errors = []

    counts = suite["meta"]["memory_task_counts"]
    expected = {
        "single_hop": 20,
        "multi_hop": 15,
        "temporal": 10,
        "knowledge_update": 10,
        "abstention": 15,
        "adversarial": 10,
    }
    for k, v in expected.items():
        if counts.get(k) != v:
            errors.append(f"task count {k}: {counts.get(k)} != {v}")

    for p in suite["prompts"]:
        for eid in p.get("evidence_line_ids") or []:
            if eid not in idx:
                errors.append(f"{p['id']}: unknown evidence {eid}")
        if p.get("abstention_expected") and p.get("evidence_line_ids"):
            errors.append(f"{p['id']}: abstention must not have evidence")

    if errors:
        for e in errors:
            print(f"FAIL: {e}", file=sys.stderr)
        return 1
    print(f"OK: {suite['meta']['prompt_count']} prompts · tiers {tiers['meta']['tiers']}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
