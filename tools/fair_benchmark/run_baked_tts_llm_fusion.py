#!/usr/bin/env python3
"""TTS + baked/fused LLM fusion test — espeak → mel → valhalla_fused_lm on open prompts.

Compares text-only vs TTS-augmented on the same baked checkpoint (no runtime weight patch).

Usage:
  python tools/fair_benchmark/run_baked_tts_llm_fusion.py --phase smoke
  python tools/fair_benchmark/run_baked_tts_llm_fusion.py --phase test --model checkpoints/valhalla_patched/Qwen2.5-0.5B-Instruct_valhalla_s0.08
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


def run_fused(phase: str, model: Path, audio_json: Path | None, out: Path) -> dict:
    cmd = [
        str(FUSED),
        "--phase",
        phase,
        "--model",
        str(model),
        "--out",
        str(out),
    ]
    env = {**dict(**__import__("os").environ), "RUSTFLAGS": "-L /opt/cuda/lib64"}
    if audio_json is not None:
        cmd.extend(["--multimodal-audio-json", str(audio_json)])
        env["VALHALLA_MULTIMODAL_FUSION"] = "1"
    subprocess.run(cmd, check=True, cwd=REPO, env=env)
    return json.loads(out.read_text(encoding="utf-8"))


def summarize(label: str, data: dict) -> None:
    s = data["summary"]
    print(f"\n=== {label} ===")
    print(f"Total: {s['total']['correct']}/{s['total']['n']} ({100*s['total']['acc']:.1f}%)")
    print(f"  open: {s['open']['correct']}/{s['open']['n']}")
    print(f"  mcq:  {s['mcq']['correct']}/{s['mcq']['n']}")
    print(f"  num:  {s['numeric']['correct']}/{s['numeric']['n']}")
    multimodal_paths = [k for k in s.get("decode_path_counts", {}) if "multimodal" in k]
    if multimodal_paths:
        print(f"  multimodal decode paths: {multimodal_paths}")


def main() -> None:
    ap = argparse.ArgumentParser(description="Baked LLM + TTS fusion A/B")
    ap.add_argument("--phase", default="smoke", choices=["smoke", "val", "test"])
    ap.add_argument("--model", type=Path, default=DEFAULT_MODEL)
    ap.add_argument("--skip-tts", action="store_true", help="Reuse existing open_audio_{phase}.json")
    args = ap.parse_args()

    if not FUSED.is_file():
        sys.exit(f"Build: cargo build -p hub-f64 --release --bin valhalla_fused_lm\nMissing {FUSED}")

    OUT_DIR.mkdir(parents=True, exist_ok=True)
    suite = load_suite()
    open_prompts = [p for p in select_prompts(suite, args.phase) if p.get("score_type") == "open"]
    audio_json = OUT_DIR / f"open_audio_{args.phase}.json"

    if not args.skip_tts or not audio_json.is_file():
        audio_map: dict[str, list[float]] = {}
        with tempfile.TemporaryDirectory() as td:
            tmp = Path(td)
            for p in open_prompts:
                wav = tmp / f"{p['id']}.wav"
                tts(strict_user_text(p), wav)
                audio_map[p["id"]] = encode_wav(wav)
                print(f"  tts {p['id']}", file=sys.stderr)
        audio_json.write_text(json.dumps(audio_map), encoding="utf-8")
        print(f"Wrote {audio_json} ({len(audio_map)} vectors)", file=sys.stderr)

    base_out = OUT_DIR / f"baked_llm_text_{args.phase}.json"
    mm_out = OUT_DIR / f"baked_llm_tts_{args.phase}.json"

    print("Running baked LLM text-only...", file=sys.stderr)
    base = run_fused(args.phase, args.model, None, base_out)
    print("Running baked LLM + TTS multimodal...", file=sys.stderr)
    mm = run_fused(args.phase, args.model, audio_json, mm_out)

    summarize("Baked LLM (text only)", base)
    summarize("Baked LLM + TTS fusion", mm)

    report = {
        "phase": args.phase,
        "model": str(args.model),
        "open_prompts": len(open_prompts),
        "text_only": base["summary"],
        "tts_fusion": mm["summary"],
        "delta_total_correct": mm["summary"]["total"]["correct"] - base["summary"]["total"]["correct"],
    }
    report_path = OUT_DIR / f"baked_tts_llm_fusion_{args.phase}.json"
    report_path.write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(f"\nReport: {report_path}")
    print(f"Delta total correct (TTS - text): {report['delta_total_correct']:+d}")


if __name__ == "__main__":
    main()
