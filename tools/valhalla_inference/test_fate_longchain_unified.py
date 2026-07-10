#!/usr/bin/env python3
"""Unified long-chain + long-term memory benchmark (theme drift after interleave).

Phases on ONE session (no reset):
  0) fate pretrain     — configurable items × epochs (reversible epoch log)
  A) anchor baseline   — D1 theme anchors
  B) interleave        — D2/D3 long-chain questions
  C) anchor reprobe    — same anchors → theme/tag drift
  D) holdout           — fresh probes never asked in A/B

  python3 tools/valhalla_inference/test_fate_longchain_unified.py --low-resource
  python3 tools/valhalla_inference/test_fate_longchain_unified.py --low-resource --enhanced
"""
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
REPORTS = REPO / "reports" / "valhalla_inference"
BIN = REPO / "target" / "release" / "valhalla_base"
SPEC = REPO / "tools/standard_benchmarks/bundled/fate_longchain_unified_locomo.json"


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--low-resource", action="store_true")
    ap.add_argument("--parallel", action="store_true", help="use parallel pipes (heavier)")
    ap.add_argument("--max-lines", type=int, default=0)
    ap.add_argument("--chunk-size", type=int, default=64)
    ap.add_argument("--pretrain-passes", type=int, default=None)
    ap.add_argument("--max-epochs", type=int, default=None)
    ap.add_argument(
        "--enhanced",
        action="store_true",
        help="stronger pretrain: 8 items × 8 epochs, Fate adaptive multi-pass warmup",
    )
    ap.add_argument("--verbose", action="store_true")
    ap.add_argument(
        "--smoke",
        action="store_true",
        help="ultra-light smoke: 24 corpus lines, chunk 24, ingress cap 48",
    )
    args = ap.parse_args()

    if args.smoke:
        args.low_resource = True
        if args.max_lines == 0:
            args.max_lines = 24
        if args.chunk_size == 64:
            args.chunk_size = 24

    if args.enhanced:
        pretrain_passes = args.pretrain_passes if args.pretrain_passes is not None else 0
        max_epochs = args.max_epochs if args.max_epochs is not None else 8
    else:
        pretrain_passes = args.pretrain_passes if args.pretrain_passes is not None else 0
        max_epochs = args.max_epochs if args.max_epochs is not None else 0

    if not BIN.is_file():
        subprocess.check_call(
            [
                "cargo", "build", "-p", "hub-f64", "--release", "--bin", "valhalla_base",
                "--features", "patch-lm",
            ],
            cwd=REPO,
            env={**os.environ, "RUSTFLAGS": "-L /opt/cuda/lib64"},
        )

    cmd = [
        str(BIN), "demo", "fate-longchain-unified",
        "--max-lines", str(args.max_lines),
        "--chunk-size", str(args.chunk_size),
        "--pretrain-passes", str(pretrain_passes),
        "--max-epochs", str(max_epochs),
        "--spec", str(SPEC.relative_to(REPO)),
    ]
    if args.low_resource:
        cmd.append("--low-resource")
        cmd.append("--fast")
    if args.parallel:
        cmd.append("--parallel")

    print(" ".join(cmd), flush=True)
    env = {**os.environ, "RUSTFLAGS": "-L /opt/cuda/lib64"}
    env.setdefault("VALHALLA_INGRESS_RESOURCE_PERCENT", "70")
    if args.low_resource or args.smoke:
        env.setdefault("VALHALLA_INGRESS_MAX_ITEMS", "48" if args.smoke else "128")
    proc = subprocess.run(cmd, cwd=REPO, capture_output=True, text=True, env=env)
    if args.verbose or args.low_resource or args.enhanced:
        if proc.stdout:
            print(proc.stdout)
        if proc.stderr:
            print(proc.stderr, file=sys.stderr)
    if proc.returncode != 0:
        print(proc.stderr or proc.stdout, file=sys.stderr)
        return proc.returncode

    out = REPORTS / "fate_longchain_unified.json"
    doc = json.loads(out.read_text(encoding="utf-8"))
    print(
        f"protocol {doc['protocol']}  routing={doc['routing']}  corpus={doc['corpus_lines']} lines\n"
        f"  pretrain {doc.get('pretrain_pass', 0):.1%} "
        f"({doc.get('pretrain_epochs', 0)}/{doc.get('pretrain_max_epochs', 0)} epochs, "
        f"passes={doc.get('pretrain_passes', 0)}, conv={doc.get('pretrain_converged')})\n"
        f"  anchor baseline {doc['anchor_baseline_pass']:.1%}  reprobe {doc['anchor_reprobe_pass']:.1%}\n"
        f"  theme_drift {doc['theme_drift_rate']:.1%}  retention {doc['anchor_retention_rate']:.1%}\n"
        f"  pollution {doc['context_pollution_rate']:.1%}  interleave {doc['interleave_pass']:.1%}\n"
        f"  holdout {doc['holdout_pass']:.1%}  composite {doc['composite_score']:.3f}"
    )
    for ep in doc.get("pretrain_epoch_log") or []:
        print(f"  pretrain epoch {ep['epoch']}: {ep['train_correct']}/{ep['train_n']}")
    print("--- anchor drift ---")
    for row in doc.get("anchor_drift_rows") or []:
        drift = "DRIFT" if row.get("theme_drift") else "ok"
        ret = "retained" if row.get("retained") else "lost"
        print(
            f"  {row.get('id')}  {drift}  {ret}  "
            f"base={row.get('baseline_tags')} → reprobe={row.get('reprobe_tags')}"
        )
    print(f"→ {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
