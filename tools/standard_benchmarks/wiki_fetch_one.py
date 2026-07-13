#!/usr/bin/env python3
"""Fetch one Wikipedia extract (stdout) — used by Rust FRAMES eval fallback."""
from __future__ import annotations

import argparse
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT))

from download_utils import wiki_summary_fetch  # noqa: E402


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("title")
    ap.add_argument("--timeout", type=int, default=20)
    ap.add_argument("--offline", action="store_true")
    args = ap.parse_args()
    text = wiki_summary_fetch(args.title, timeout_sec=args.timeout, offline=args.offline)
    if text:
        sys.stdout.write(text)
        return 0
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
