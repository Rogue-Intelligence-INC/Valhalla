#!/usr/bin/env python3
"""Experiment: LLM+TTS decompose → Valhalla Fate multi-cycle adapt → structure recompose → LM.

Requires:
  VALHALLA_MULTIMODAL_FUSION=1
  VALHALLA_COMPOSE_FATE_CYCLES=1

Usage:
  python tools/fair_benchmark/run_compose_fate_experiment.py --phase smoke
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
FUSED = REPO / "target/release/valhalla_fused_lm"
OUT_DIR = REPO / "reports/tts_tile_stem_fusion"
DEFAULT_MODEL = REPO / "checkpoints/valhalla_patched/Qwen2.5-0.5B-Instruct_valhalla_s0.08"

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
    ap = argparse.ArgumentParser()
    ap.add_argument("--phase", default="smoke")
    ap.add_argument("--model", type=Path, default=DEFAULT_MODEL)
    args = ap.parse_args()

    if not FUSED.is_file():
        sys.exit(f"Build valhalla_fused_lm first: {FUSED}")

    suite = load_suite()
    prompts = [
        p
        for p in select_prompts(suite, args.phase)
        if p.get("score_type") in ("open", "mcq")
    ]
    if not prompts:
        sys.exit(f"No open/mcq prompts in phase={args.phase}")

    OUT_DIR.mkdir(parents=True, exist_ok=True)
    audio_map: dict[str, list[float]] = {}
    with tempfile.TemporaryDirectory() as td:
        tmp = Path(td)
        for p in prompts:
            wav = tmp / f"{p['id']}.wav"
            tts(strict_user_text(p), wav)
            audio_map[p["id"]] = encode_wav(wav)

    audio_json = OUT_DIR / f"compose_fate_audio_{args.phase}.json"
    audio_json.write_text(json.dumps(audio_map), encoding="utf-8")

    out = OUT_DIR / f"compose_fate_{args.phase}.json"
    env = {
        **dict(**__import__("os").environ),
        "RUSTFLAGS": "-L /opt/cuda/lib64",
        "VALHALLA_MULTIMODAL_FUSION": "1",
        "VALHALLA_COMPOSE_FATE_CYCLES": "1",
        "VALHALLA_COMPOSE_FATE_MIN_CYCLES": "3",
    }
    cmd = [
        str(FUSED),
        "--phase",
        args.phase,
        "--model",
        str(args.model),
        "--multimodal-audio-json",
        str(audio_json),
        "--out",
        str(out),
    ]
    print("Compose Fate experiment:", " ".join(cmd), file=sys.stderr)
    subprocess.run(cmd, check=True, cwd=REPO, env=env)

    data = json.loads(out.read_text(encoding="utf-8"))
    rows = data.get("fused_rows", [])
    fate_paths = [r for r in rows if "compose_fate" in r.get("decode_path", "")]
    print(f"\nTotal: {data['summary']['total']['correct']}/{data['summary']['total']['n']}")
    print(f"Open: {data['summary']['open']['correct']}/{data['summary']['open']['n']}")
    print(f"MCQ: {data['summary']['mcq']['correct']}/{data['summary']['mcq']['n']}")
    print(f"Compose Fate decode paths: {len(fate_paths)}/{len(prompts)}")
    if len(fate_paths) < len(prompts):
        sys.exit(f"FAIL: expected compose_fate paths on all multimodal rows")
    print("PASS: Fate multi-cycle adapt experiment wired")


if __name__ == "__main__":
    main()
