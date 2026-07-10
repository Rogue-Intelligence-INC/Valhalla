#!/usr/bin/env python3
"""Redirect → valhalla_backbone.py --preset mamba2_2.7b falcon_mamba_7b_4bit"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
if "--preset" not in sys.argv and "--hf" not in sys.argv:
    sys.argv[1:1] = ["--preset", "mamba2_2.7b", "--preset", "falcon_mamba_7b_4bit"]
from valhalla_backbone import main

if __name__ == "__main__":
    raise SystemExit(main())
