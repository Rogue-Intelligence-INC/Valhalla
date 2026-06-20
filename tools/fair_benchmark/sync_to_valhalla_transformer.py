#!/usr/bin/env python3
"""Copy latest fair benchmark results to valhalla-transformer."""

from __future__ import annotations

import json
import shutil
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
FAIR = Path(__file__).resolve().parent
OUT_DIR = REPO / "reports/fair_benchmark"
VT = Path("/home/jouly/AI/valhalla-transformer")


def main() -> None:
    if not VT.exists():
        print(f"Missing {VT}", file=sys.stderr)
        sys.exit(1)

    runs = sorted(OUT_DIR.glob("fair_benchmark_*.json"))
    if not runs:
        print("No fair benchmark runs — run run_fair_benchmark.py first", file=sys.stderr)
        sys.exit(1)

    latest = runs[-1]
    payload = json.loads(latest.read_text(encoding="utf-8"))
    tag = payload["meta"]["timestamp"]
    phase = payload["meta"]["phase"]

    exp = VT / "experiments"
    exp.mkdir(parents=True, exist_ok=True)

    shutil.copy2(latest, exp / f"fair_benchmark_{phase}_{tag}.json")
    (exp / "fair_benchmark_latest.json").write_text(
        json.dumps(payload, indent=2, ensure_ascii=False), encoding="utf-8"
    )

    spec = FAIR / "BENCHMARK_SPEC.md"
    if spec.exists():
        shutil.copy2(spec, VT / "zh" / "10_FAIR_BENCHMARK_SPEC.md")
        shutil.copy2(spec, VT / "en" / "10_FAIR_BENCHMARK_SPEC.md")

    print(f"Synced {latest.name} -> valhalla-transformer/experiments/", file=sys.stderr)


if __name__ == "__main__":
    main()
