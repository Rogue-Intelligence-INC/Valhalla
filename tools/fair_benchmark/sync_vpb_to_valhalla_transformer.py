#!/usr/bin/env python3
"""Copy latest VPB results to valhalla-transformer."""

from __future__ import annotations

import json
import shutil
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
VPB = REPO / "reports/vpb"
VT = Path("/home/jouly/AI/valhalla-transformer")


def latest(pattern: str) -> Path | None:
    hits = sorted(VPB.glob(pattern))
    return hits[-1] if hits else None


def main() -> None:
    if not VT.exists():
        sys.exit(f"Missing {VT}")

    exp = VT / "experiments"
    exp.mkdir(parents=True, exist_ok=True)

    for pattern, dest_name in [
        ("vpb_matrix_baseline_*.json", None),
        ("vpb_scale_test_*.json", None),
        ("vpb_forecast_*.json", None),
    ]:
        p = latest(pattern)
        if not p:
            continue
        dest = exp / p.name
        shutil.copy2(p, dest)
        print(f"Synced {p.name}", file=sys.stderr)
        if "forecast" in p.name:
            (exp / "vpb_forecast_latest.json").write_text(
                p.read_text(encoding="utf-8"), encoding="utf-8"
            )
        if "matrix" in p.name:
            (exp / "vpb_matrix_latest.json").write_text(
                p.read_text(encoding="utf-8"), encoding="utf-8"
            )

    for doc in (
        REPO / "reports/VALHALLA_VPB_PHASE_BC_COMPLETE_20260620.md",
        REPO / "tools/fair_benchmark/VALHALLA_POTENTIAL_BENCHMARK_V1.md",
    ):
        if doc.exists():
            for lang in ("zh", "en"):
                target = VT / lang / "12_VPB_POTENTIAL_BENCHMARK.md"
                if lang == "en" and doc.name == "VALHALLA_VPB_PHASE_BC_COMPLETE_20260620.md":
                    shutil.copy2(doc, target)
                elif lang == "zh" and doc.name == "VALHALLA_VPB_PHASE_BC_COMPLETE_20260620.md":
                    shutil.copy2(doc, target)


if __name__ == "__main__":
    main()
