#!/usr/bin/env python3
"""Sync hybrid-2.0 unified product to valhalla-transformer."""

from __future__ import annotations

import json
import shutil
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
VT = Path("/home/jouly/AI/valhalla-transformer")
HYBRID_DIR = REPO / "reports/hybrid_valhalla_lm"
FAIR_DIR = REPO / "reports/fair_benchmark"


def latest(pattern: str, base: Path) -> Path | None:
    hits = sorted(base.glob(pattern))
    return hits[-1] if hits else None


def pick_hybrid_json() -> Path:
    for pattern in (
        "hybrid_v4_mcq_patch_strengthen_final.json",
        "hybrid_v4_*.json",
        "hybrid_unified_*.json",
    ):
        p = latest(pattern, HYBRID_DIR) if "*" in pattern else HYBRID_DIR / pattern
        if p and p.exists():
            return p
    sys.exit(
        "No hybrid JSON — run valhalla_hybrid or run_hybrid_valhalla_lm.py first"
    )


def write_hybrid_product_md(payload: dict, dest: Path) -> None:
    meta = payload.get("meta", {})
    summary = payload.get("summary", {}).get("hybrid", {})
    by_type = summary.get("by_score_type", {})
    total = summary.get("acc", 0.0)
    n = summary.get("n", 145)

    lines = [
        "# Hybrid Valhalla 2.0 — Unified Product Snapshot",
        "",
        f"**Phase**: {meta.get('phase', 'test')} · **Timestamp**: {meta.get('timestamp', 'unknown')}",
        f"**Engine**: {meta.get('engine', meta.get('product_version', 'hybrid-2.0'))}",
        "",
        "## Metrics (fair-1.2 test)",
        "",
        "| Metric | Value |",
        "|--------|-------|",
        f"| Total | **{100 * total:.1f}%** ({summary.get('correct', '?')}/{n}) |",
    ]
    for key, label in (("open", "Open"), ("mcq", "MCQ"), ("numeric", "Numeric")):
        row = by_type.get(key, {})
        if row:
            lines.append(
                f"| {label} | **{100 * row.get('acc', 0):.1f}%** ({row.get('correct', '?')}/{row.get('n', '?')}) |"
            )
    lines.extend(
        [
            "",
            "## Artifact",
            "",
            f"`{meta.get('out', 'hybrid_unified_latest.json')}`",
            "",
            "Reproduce: see `tools/fair_benchmark/BENCHMARK_SPEC.md` in Valhalla repo.",
        ]
    )
    dest.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> None:
    if not VT.exists():
        sys.exit(f"Missing {VT}")

    exp = VT / "experiments"
    exp.mkdir(parents=True, exist_ok=True)

    hybrid_json = pick_hybrid_json()
    payload = json.loads(hybrid_json.read_text(encoding="utf-8"))
    meta = payload.setdefault("meta", {})
    tag = meta.get("timestamp") or hybrid_json.stem
    phase = meta.get("phase", "test")
    meta["source_file"] = hybrid_json.name

    dest = exp / f"hybrid_unified_{phase}_{tag}.json"
    shutil.copy2(hybrid_json, dest)
    (exp / "hybrid_unified_latest.json").write_text(
        json.dumps(payload, indent=2, ensure_ascii=False), encoding="utf-8"
    )
    print(f"Synced {hybrid_json.name} -> experiments/", file=sys.stderr)

    day_summary = REPO / "docs/hybrid_2.0/DAY_SUMMARY_20260623.md"
    for lang in ("zh", "en"):
        product_md = VT / lang / "13_HYBRID_UNIFIED_PRODUCT.md"
        if day_summary.exists():
            shutil.copy2(day_summary, product_md)
        else:
            write_hybrid_product_md(payload, product_md)

    mcq_summary = (
        REPO / "docs/hybrid_2.0/DAY_SUMMARY_20260623.md"
        if day_summary.exists()
        else REPO / "reports/VALHALLA_MCQ_AND_FAIR_BENCHMARK_SUMMARY_20260622.md"
    )
    if mcq_summary.exists():
        shutil.copy2(mcq_summary, VT / "zh" / "14_MCQ_FAIR_BENCHMARK_SUMMARY.md")
        shutil.copy2(mcq_summary, VT / "en" / "14_MCQ_FAIR_BENCHMARK_SUMMARY.md")

    fair = FAIR_DIR / "fair_benchmark_test_20260620_0859.json"
    if fair.exists():
        shutil.copy2(fair, exp / fair.name)
        (exp / "fair_benchmark_latest.json").write_text(
            fair.read_text(encoding="utf-8"), encoding="utf-8"
        )

    spec = REPO / "tools/fair_benchmark/BENCHMARK_SPEC.md"
    if spec.exists():
        shutil.copy2(spec, VT / "zh" / "10_FAIR_BENCHMARK_SPEC.md")
        shutil.copy2(spec, VT / "en" / "10_FAIR_BENCHMARK_SPEC.md")

    mcq_fate = latest("MCQ_FATE_CYCLE_*.json", REPO / "reports/experiments")
    if mcq_fate:
        shutil.copy2(mcq_fate, exp / mcq_fate.name)


if __name__ == "__main__":
    main()
