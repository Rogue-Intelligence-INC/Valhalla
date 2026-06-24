#!/usr/bin/env python3
"""Test Valhalla compose path: TTS mel bands + LM text shards → Fate → LM recompose.

Non-traditional: no sidecar concat; shards fed per Tile/Stem/Hub body.

Usage:
  python tools/fair_benchmark/run_multimodal_compose_test.py --phase smoke
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
    open_prompts = [p for p in select_prompts(suite, args.phase) if p.get("score_type") == "open"]
    if not open_prompts:
        print(f"No open prompts in phase={args.phase}; use --phase test", file=sys.stderr)
        sys.exit(1)

    OUT_DIR.mkdir(parents=True, exist_ok=True)
    audio_map: dict[str, list[float]] = {}
    with tempfile.TemporaryDirectory() as td:
        tmp = Path(td)
        for p in open_prompts:
            wav = tmp / f"{p['id']}.wav"
            tts(strict_user_text(p), wav)
            audio_map[p["id"]] = encode_wav(wav)

    audio_json = OUT_DIR / f"compose_audio_{args.phase}.json"
    audio_json.write_text(json.dumps(audio_map), encoding="utf-8")

    out = OUT_DIR / f"multimodal_compose_{args.phase}.json"
    env = {
        **dict(**__import__("os").environ),
        "RUSTFLAGS": "-L /opt/cuda/lib64",
        "VALHALLA_MULTIMODAL_FUSION": "1",
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
    print("Running compose:", " ".join(cmd), file=sys.stderr)
    subprocess.run(cmd, check=True, cwd=REPO, env=env)

    data = json.loads(out.read_text(encoding="utf-8"))
    compose_paths = [
        r["decode_path"]
        for r in data["fused_rows"]
        if r.get("score_type") == "open" and "compose" in r.get("decode_path", "")
    ]
    open_n = data["summary"]["open"]["n"]
    print(f"\nOpen: {data['summary']['open']['correct']}/{open_n}")
    print(f"Compose decode paths: {len(compose_paths)}/{open_n}")
    print(f"Protocol: {data.get('protocol')}")
    if len(compose_paths) < open_n:
        sys.exit(f"FAIL: expected compose paths on all open rows, got {len(compose_paths)}")
    print("PASS: Valhalla compose shards wired for TTS+LM")


if __name__ == "__main__":
    main()
