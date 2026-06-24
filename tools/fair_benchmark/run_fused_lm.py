#!/usr/bin/env python3
"""Run fused Valhalla LM (single forward, no hybrid router) on fair benchmark."""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
BIN = REPO / "target/release/valhalla_fused_lm"
OUT_DIR = REPO / "reports/fused_valhalla_lm"
DEFAULT_MODEL = REPO / "checkpoints/valhalla_patched/Qwen2.5-0.5B-Instruct_valhalla_s0.08"


def main() -> None:
    ap = argparse.ArgumentParser(description="Fused Valhalla LM fair benchmark")
    ap.add_argument("--phase", choices=["smoke", "val", "test"], default="test")
    ap.add_argument("--model", type=Path, default=DEFAULT_MODEL)
    ap.add_argument("--patch-body", default="triad")
    ap.add_argument("--strength", type=float, default=0.08)
    ap.add_argument("--out", type=Path, default=None)
    args = ap.parse_args()

    if not BIN.exists():
        sys.exit(
            f"Missing {BIN}\n"
            "Build: RUSTFLAGS='-L /opt/cuda/lib64' cargo build -p hub-f64 --release --bin valhalla_fused_lm"
        )

    OUT_DIR.mkdir(parents=True, exist_ok=True)
    out = args.out or OUT_DIR / f"fused_{args.phase}.json"

    cmd = [
        str(BIN),
        "--phase",
        args.phase,
        "--model",
        str(args.model),
        "--patch-body",
        args.patch_body,
        "--strength",
        str(args.strength),
        "--out",
        str(out),
    ]
    print("Running:", " ".join(cmd), file=sys.stderr)
    subprocess.run(cmd, check=True, cwd=REPO)
    print(f"Done: {out}", file=sys.stderr)


if __name__ == "__main__":
    main()
