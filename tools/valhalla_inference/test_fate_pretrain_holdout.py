#!/usr/bin/env python3
"""Fate pretrain until train set converges, then holdout test.

  # resource-controlled (minimal evidence corpus, no parallel pipes on test)
  python3 tools/valhalla_inference/test_fate_pretrain_holdout.py --low-resource

  # full benchmark (heavier)
  python3 tools/valhalla_inference/test_fate_pretrain_holdout.py --train 5 --test 5 --max-lines 280 --fast
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


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--train", type=int, default=None, help="train split size (first N items)")
    ap.add_argument("--test", type=int, default=None, help="holdout test size (next N items)")
    ap.add_argument("--max-lines", type=int, default=None, help="truncate session; ignored with --low-resource")
    ap.add_argument("--chunk-size", type=int, default=64)
    ap.add_argument("--max-epochs", type=int, default=None)
    ap.add_argument("--fast", action="store_true")
    ap.add_argument(
        "--low-resource",
        action="store_true",
        help="minimal evidence corpus, single-chunk ingest, targeted holdout (low CPU/GPU)",
    )
    ap.add_argument("--verbose", action="store_true")
    ap.add_argument(
        "--enhanced",
        action="store_true",
        help="more train/holdout items and epochs (8 train, 5 test, 8 epochs)",
    )
    args = ap.parse_args()

    if args.low_resource:
        if args.enhanced:
            train = args.train if args.train is not None else 8
            test = args.test if args.test is not None else 5
            max_epochs = args.max_epochs if args.max_epochs is not None else 8
        else:
            train = args.train if args.train is not None else 3
            test = args.test if args.test is not None else 3
            max_epochs = args.max_epochs if args.max_epochs is not None else 4
        max_lines = args.max_lines if args.max_lines is not None else 0
        fast = True
    elif args.enhanced:
        train = args.train if args.train is not None else 8
        test = args.test if args.test is not None else 5
        max_lines = args.max_lines if args.max_lines is not None else 128
        max_epochs = args.max_epochs if args.max_epochs is not None else 8
        fast = True
    else:
        train = args.train if args.train is not None else 5
        test = args.test if args.test is not None else 5
        max_lines = args.max_lines if args.max_lines is not None else 280
        max_epochs = args.max_epochs if args.max_epochs is not None else 24
        fast = args.fast

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
        str(BIN), "demo", "fate-pretrain-holdout",
        "--train", str(train),
        "--test", str(test),
        "--max-lines", str(max_lines),
        "--chunk-size", str(args.chunk_size),
        "--max-epochs", str(max_epochs),
    ]
    if fast:
        cmd.append("--fast")
    if args.low_resource:
        cmd.append("--low-resource")

    print(" ".join(cmd), flush=True)
    env = {**os.environ, "RUSTFLAGS": "-L /opt/cuda/lib64"}
    env.setdefault("VALHALLA_INGRESS_RESOURCE_PERCENT", "70")
    if args.low_resource:
        env.setdefault("VALHALLA_INGRESS_MAX_ITEMS", "128")
    proc = subprocess.run(cmd, cwd=REPO, capture_output=True, text=True, env=env)
    if args.verbose or args.low_resource:
        if proc.stdout:
            print(proc.stdout)
        if proc.stderr:
            print(proc.stderr, file=sys.stderr)
    if proc.returncode != 0:
        print(proc.stderr or proc.stdout, file=sys.stderr)
        return proc.returncode

    out = REPORTS / "fate_pretrain_holdout.json"
    doc = json.loads(out.read_text(encoding="utf-8"))
    conv = "yes" if doc.get("converged") else "no"
    mode = doc.get("pretrain_mode") or ("low-resource" if doc.get("low_resource") else "full")
    print(
        f"[{mode}] corpus={doc.get('corpus_lines', '?')} lines  "
        f"ingest_passes={doc.get('ingest_passes', '?')}  fragments={doc.get('fragment_count', '?')}  "
        f"epochs {doc['pretrain_epochs']}/{doc['max_epochs']}  converged={conv}  "
        f"train {doc['train_pass']:.1%} ({doc['train_n']} items, skipped {doc.get('train_skipped_no_evidence', 0)})  "
        f"test {doc['test_pass']:.1%} ({doc['test_n']} items, skipped {doc.get('test_skipped_no_evidence', 0)})  "
        f"hub_spread {doc.get('final_hub_spread', 0):.4f}  walk_steps {doc.get('total_walk_steps', 0)}"
    )
    for ep in doc.get("epoch_log") or []:
        print(
            f"  epoch {ep['epoch']}: train {ep['train_correct']}/{ep['train_n']}  "
            f"walk={ep.get('walk_steps', 0)} spread={ep.get('hub_spread', 0):.4f} "
            f"fragments={ep.get('fragment_count', 0)}"
        )
    print("--- train ---")
    for row in doc.get("train_rows") or []:
        mark = "✓" if row.get("correct") else "✗"
        print(f"  {mark} {row.get('id')}  ev={row.get('evidence')}  ans={str(row.get('answer', ''))[:80]}")
    print("--- test (holdout) ---")
    for row in doc.get("test_rows") or []:
        mark = "✓" if row.get("correct") else "✗"
        print(f"  {mark} {row.get('id')}  ev={row.get('evidence')}  ans={str(row.get('answer', ''))[:80]}")
    print(f"→ {out}")
    if not doc.get("converged") or doc.get("train_pass", 0) < 1.0 or doc.get("test_pass", 0) < (2.0 / 3.0):
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
