#!/usr/bin/env python3
"""Download small text-to-image model for Valhalla tri-modal fusion.

Default: stabilityai/sd-turbo (~1.5GB, 1-step generation).

Usage:
  python download_image_model.py
  python download_image_model.py --model segmind/Segmind-Vega
"""

from __future__ import annotations

import argparse
import os
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
DEFAULT_MODEL = "stabilityai/sd-turbo"
DEFAULT_OUT = REPO / "models" / "image_gen" / "sd-turbo"


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--model", default=DEFAULT_MODEL)
    ap.add_argument("--out", type=Path, default=DEFAULT_OUT)
    args = ap.parse_args()

    os.environ.setdefault("HF_ENDPOINT", "https://hf-mirror.com")

    try:
        from huggingface_hub import snapshot_download
    except ImportError:
        sys.exit("pip install huggingface_hub")

    args.out.mkdir(parents=True, exist_ok=True)
    print(f"Downloading {args.model} → {args.out}", file=sys.stderr)
    path = snapshot_download(
        args.model,
        local_dir=str(args.out),
        local_dir_use_symlinks=False,
    )
    print(json_path := args.out / "model_id.txt")
    (args.out / "model_id.txt").write_text(args.model + "\n", encoding="utf-8")
    print(f"Done: {path}", file=sys.stderr)


if __name__ == "__main__":
    main()
