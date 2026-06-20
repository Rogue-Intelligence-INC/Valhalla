#!/usr/bin/env python3
"""Copy latest tile/stemcell cycle experiment to valhalla-transformer."""

from __future__ import annotations

import json
import shutil
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
OUT_DIR = REPO / "reports/tile_stemcell_cycles"
VT = Path("/home/jouly/AI/valhalla-transformer")


def main() -> None:
    if not VT.exists():
        print(f"Missing {VT}", file=sys.stderr)
        sys.exit(1)

    for phase in ("test", "smoke"):
        runs = sorted(OUT_DIR.glob(f"tile_stemcell_cycles_{phase}_*.json"))
        if not runs:
            continue
        latest = runs[-1]
        payload = json.loads(latest.read_text(encoding="utf-8"))
        tag = payload["meta"]["timestamp"]

        exp = VT / "experiments"
        exp.mkdir(parents=True, exist_ok=True)
        dest = exp / f"tile_stemcell_cycles_{phase}_{tag}.json"
        shutil.copy2(latest, dest)
        if phase == "test":
            (exp / "tile_stemcell_cycles_latest.json").write_text(
                json.dumps(payload, indent=2, ensure_ascii=False), encoding="utf-8"
            )
        print(f"Synced {latest.name} -> {dest}", file=sys.stderr)

    report = REPO / "reports/VALHALLA_TILE_STEMCELL_MULTI_CYCLE_EXPERIMENT_20260620.md"
    if report.exists():
        for lang in ("zh", "en"):
            doc = VT / lang / "11_TILE_STEMCELL_CYCLE_EXPERIMENT.md"
            if doc.exists():
                print(f"Doc already at {doc}", file=sys.stderr)


if __name__ == "__main__":
    main()
