#!/usr/bin/env python3
"""Validate fair_benchmark_suite.json — fail CI if clean corpus leaks eval answers."""

from __future__ import annotations

import json
import sys
from pathlib import Path

FAIR = Path(__file__).resolve().parent
SUITE = FAIR / "fair_benchmark_suite.json"


def main() -> int:
    sys.path.insert(0, str(FAIR))
    from contamination import corpus_contamination_report

    if not SUITE.exists():
        print(f"Missing {SUITE} — run build_fair_suite.py first", file=sys.stderr)
        return 1

    suite = json.loads(SUITE.read_text(encoding="utf-8"))
    prompts = suite["prompts"]
    errors = []

    # 1. Clean corpus must have zero leakage
    clean = suite["corpus"]["ingest_clean"]
    audit = corpus_contamination_report(clean, prompts)
    if not audit["clean"]:
        errors.append(
            f"ingest_clean has {audit['contaminated_line_count']} contaminated lines "
            f"affecting {audit['contaminated_prompt_count']} prompts"
        )

    # 2. Split integrity
    all_ids = {p["id"] for p in prompts}
    split_ids: set[str] = set()
    for name, ids in suite["splits"].items():
        for pid in ids:
            if pid not in all_ids:
                errors.append(f"split {name} references unknown id {pid}")
            if pid in split_ids:
                errors.append(f"duplicate split assignment for {pid}")
            split_ids.add(pid)
    if split_ids != all_ids:
        missing = all_ids - split_ids
        extra = split_ids - all_ids
        if missing:
            errors.append(f"prompts not in any split: {sorted(missing)[:5]}...")
        if extra:
            errors.append(f"unknown ids in splits: {extra}")

    # 3. Smoke = 12 fixed
    if len(suite["splits"]["smoke"]) != 12:
        errors.append(f"smoke split must be 12, got {len(suite['splits']['smoke'])}")

    # 4. val + test must not overlap smoke for reporting
    val_test = set(suite["splits"]["val"]) | set(suite["splits"]["test"])
    smoke = set(suite["splits"]["smoke"])
    if val_test & smoke:
        errors.append("smoke overlaps val/test")

    # 5. Legacy corpus should show numeric leakage (sanity)
    legacy = suite["corpus"]["ingest_legacy_contaminated"]
    legacy_audit = corpus_contamination_report(legacy, prompts)
    if legacy_audit["contaminated_line_count"] == 0:
        errors.append("legacy corpus unexpectedly has no leaks — ablation arm useless")

    if errors:
        print("VALIDATE FAILED:", file=sys.stderr)
        for e in errors:
            print(f"  - {e}", file=sys.stderr)
        return 1

    print(
        f"OK fair_benchmark_suite: prompts={len(prompts)} "
        f"clean_corpus={len(clean)} lines "
        f"smoke/val/test={len(suite['splits']['smoke'])}/"
        f"{len(suite['splits']['val'])}/{len(suite['splits']['test'])} "
        f"legacy_leaks={legacy_audit['contaminated_line_count']} lines",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
