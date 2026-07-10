#!/usr/bin/env python3
"""Redirect → valhalla_backbone.py --probe"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
sys.argv = [sys.argv[0], "--probe", *sys.argv[1:]]
from valhalla_backbone import main

if __name__ == "__main__":
    raise SystemExit(main())
