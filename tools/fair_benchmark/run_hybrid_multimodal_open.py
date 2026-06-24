#!/usr/bin/env python3
"""Generate TTS audio features for open prompts and run hybrid with multimodal v4.

Usage:
  python tools/fair_benchmark/run_hybrid_multimodal_open.py --phase test
  python tools/fair_benchmark/run_hybrid_multimodal_open.py --phase smoke --skip-run
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
FAIR = Path(__file__).resolve().parent
ENCODE = REPO / "tools/valhalla_dl_front/encode.py"
HYBRID = REPO / "target/release/valhalla_hybrid"
OUT_DIR = REPO / "reports/tts_tile_stem_fusion"

sys.path.insert(0, str(FAIR))
from run_fair_benchmark import load_suite, select_prompts  # noqa: E402
from run_strict_incubation_experiment import strict_user_text  # noqa: E402


def tts(text: str, wav: Path) -> None:
    subprocess.run(["espeak", "-w", str(wav), text], check=True, capture_output=True)


def encode_wav(wav: Path) -> list[float]:
    proc = subprocess.run(
        [sys.executable, str(ENCODE)],
        input=json.dumps({"modality": "audio_path", "path": str(wav)}),
        capture_output=True,
        text=True,
        check=True,
    )
    return json.loads(proc.stdout)["embedding"]


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--phase", default="test")
    parser.add_argument("--model", default="checkpoints/valhalla_patched/Qwen2.5-0.5B-Instruct_valhalla_s0.08")
    parser.add_argument("--skip-run", action="store_true")
    args = parser.parse_args()

    suite = load_suite()
    open_prompts = [p for p in select_prompts(suite, args.phase) if p.get("score_type") == "open"]
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    audio_json = OUT_DIR / f"open_audio_{args.phase}.json"

    audio_map: dict[str, list[float]] = {}
    with tempfile.TemporaryDirectory() as td:
        tmp = Path(td)
        for p in open_prompts:
            text = strict_user_text(p)
            wav = tmp / f"{p['id']}.wav"
            tts(text, wav)
            audio_map[p["id"]] = encode_wav(wav)
            print(f"  {p['id']}")

    audio_json.write_text(json.dumps(audio_map), encoding="utf-8")
    print(f"Wrote {audio_json} ({len(audio_map)} open prompts)")

    if args.skip_run:
        return

    if not HYBRID.is_file():
        print("Build: cargo build -p hub-f64 --release --bin valhalla_hybrid")
        sys.exit(1)

    out = OUT_DIR / f"hybrid_multimodal_open_{args.phase}.json"
    env = {
        **dict(**__import__("os").environ),
        "RUSTFLAGS": "-L /opt/cuda/lib64",
        "VALHALLA_MULTIMODAL_FUSION": "1",
        "VALHALLA_OPEN_PATCH_BIAS_SCALE": "0.15",
    }
    cmd = [
        str(HYBRID),
        "--phase",
        args.phase,
        "--run-patch",
        "--model",
        args.model,
        "--merge-mode",
        "deploy",
        "--multimodal-audio-json",
        str(audio_json),
        "--out",
        str(out),
    ]
    print("Running:", " ".join(cmd))
    subprocess.run(cmd, check=True, env=env)
    print(f"Done: {out}")


if __name__ == "__main__":
    main()
