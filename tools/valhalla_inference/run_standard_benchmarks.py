#!/usr/bin/env python3
"""Run all third-party standard benchmarks (RAGTruth, Needle, LongBench-v2).

Usage:
  python3 tools/valhalla_inference/run_standard_benchmarks.py
  python3 tools/valhalla_inference/run_standard_benchmarks.py --fast
"""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
TESTS = [
    "test_needle_session_v1.py",
    "test_ragtruth_v1.py",
    "test_ragtruth_full_v1.py",
    "test_longbench_v2_v1.py",
    "test_frames_v1.py",
    "test_multihop_rag_v1.py",
    "test_locomo_v1.py",
    "test_ruler_v1.py",
    "test_lara_v1.py",
]


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--fast", action="store_true")
    ap.add_argument("--limit", type=int, default=0, help="limit for RAGTruth/LongBench")
    args = ap.parse_args()

    flags = ["--fast"] if args.fast else []
    if args.limit > 0:
        flags += ["--limit", str(args.limit)]

    failed = 0
    for name in TESTS:
        cmd = [sys.executable, str(HERE / name), *flags]
        print(f"\n{'=' * 60}\n$ {' '.join(cmd)}\n")
        rc = subprocess.call(cmd)
        if rc != 0:
            failed += 1

    print(f"\n{'=' * 60}")
    if failed:
        sys.exit(f"{failed}/{len(TESTS)} standard benchmarks failed")
    print(f"All {len(TESTS)} standard benchmarks completed.")


if __name__ == "__main__":
    main()
