#!/usr/bin/env python3
"""Redirect → valhalla_backbone.py"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from valhalla_backbone import main

if __name__ == "__main__":
    raise SystemExit(main())
