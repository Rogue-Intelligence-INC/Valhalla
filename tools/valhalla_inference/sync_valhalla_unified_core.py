#!/usr/bin/env python3
"""Sync latest ValhallaBase reports + key docs into valhalla-unified-core mirror.

Usage:
  python3 tools/valhalla_inference/sync_valhalla_unified_core.py
  python3 tools/valhalla_inference/sync_valhalla_unified_core.py --dry-run
"""

from __future__ import annotations

import argparse
import shutil
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
SRC = REPO / "reports" / "valhalla_inference"
DST = REPO / "valhalla-unified-core" / "reports" / "valhalla_inference"

# JSON + MD reports to mirror (add new headline artifacts here).
REPORT_FILES = [
    "fate_longchain_unified.json",
    "fate_pretrain_holdout.json",
    "fate_parallel_longchain_test.json",
    "fate_routing_ab_locomo.json",
    "valhalla_independence_test.json",
    "valhalla_integrated_status.json",
    "industry_benchmarks_summary.json",
    "external_pareto_matrix.json",
    "locomo_v1_test.json",
    "locomo_v1_rust_test.json",
    "needle_session_v1_test.json",
    "ragtruth_v1_test.json",
    "proactive_fate_v2_test.json",
    "valhalla_base_training_potential_test_v2.json",
    "multihop_rag_v2_test.json",
    "multihop_rag_fate_test.json",
    "multihop_rag_hybrid_test.json",
    "longbench_v2_v1_test.json",
    "lara_v1_test.json",
    "external_holdout_test.json",
    "mcq_ladder_v1_test.json",
    "EXPERIMENT_SUMMARY_20260713.md",
    "BENCHMARK_SUMMARY_20260713.md",
    "EXPERIMENT_SUMMARY_20260711.md",
    "VALHALLA_INTEGRATED_STATUS.md",
    "EXTERNAL_PARETO_MATRIX.md",
    "VALHALLA_INDEPENDENCE_EXPERIMENT_20260710.md",
    "BENCHMARK_SUMMARY_20260708.md",
    "FATE_LONGCHAIN_UNIFIED.md",
    "FATE_PRETRAIN_HOLDOUT.md",
]

DOC_PATCHES: list[tuple[Path, dict[str, str]]] = [
    (
        REPO / "valhalla-unified-core" / "docs" / "EXPERIMENT_RECORD_INFERENCE_PARADIGM.md",
        {
            "> **Status:** Living document · **Updated:** 2026-06-28": (
                f"> **Status:** Living document · **Updated:** {datetime.now(timezone.utc).strftime('%Y-%m-%d')}"
            ),
            "> **Status:** Living document · **Updated:** 2026-07-11": (
                f"> **Status:** Living document · **Updated:** {datetime.now(timezone.utc).strftime('%Y-%m-%d')}"
            ),
        },
    ),
]


def sync_reports(*, dry_run: bool) -> list[str]:
    copied: list[str] = []
    DST.mkdir(parents=True, exist_ok=True)
    for name in REPORT_FILES:
        src = SRC / name
        if not src.is_file():
            continue
        dst = DST / name
        if dry_run:
            copied.append(f"would copy {name}")
            continue
        shutil.copy2(src, dst)
        copied.append(name)
    return copied


def patch_docs(*, dry_run: bool) -> list[str]:
    touched: list[str] = []
    for path, replacements in DOC_PATCHES:
        if not path.is_file():
            continue
        text = path.read_text(encoding="utf-8")
        new_text = text
        for old, new in replacements.items():
            new_text = new_text.replace(old, new)
        if new_text != text:
            if dry_run:
                touched.append(f"would patch {path.relative_to(REPO)}")
            else:
                path.write_text(new_text, encoding="utf-8")
                touched.append(str(path.relative_to(REPO)))
    return touched


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--dry-run", action="store_true")
    args = ap.parse_args()

    copied = sync_reports(dry_run=args.dry_run)
    patched = patch_docs(dry_run=args.dry_run)
    print(f"synced {len(copied)} report(s) → {DST.relative_to(REPO)}")
    for name in copied:
        print(f"  {name}")
    if patched:
        print("patched docs:")
        for p in patched:
            print(f"  {p}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
