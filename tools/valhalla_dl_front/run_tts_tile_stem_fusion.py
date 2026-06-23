#!/usr/bin/env python3
"""Download TTS (Piper ONNX), synthesize speech, extract audio features, run Tile+Stem fusion.

Usage:
  python tools/valhalla_dl_front/run_tts_tile_stem_fusion.py
  python tools/valhalla_dl_front/run_tts_tile_stem_fusion.py --tts edge

Requires (optional): edge-tts, piper-tts, onnxruntime, scipy, numpy
Rust binary: cargo build -p hub-f64 --release --bin valhalla_tile_stem_tts_fusion
"""

from __future__ import annotations

import argparse
import asyncio
import json
import subprocess
import sys
import urllib.request
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
MODEL_DIR = REPO / "tools/valhalla_dl_front/models/piper"
OUT_DIR = REPO / "reports/tts_tile_stem_fusion"
ENCODE_PY = REPO / "tools/valhalla_dl_front/encode.py"
FUSION_BIN = REPO / "target/release/valhalla_tile_stem_tts_fusion"

PIPER_VOICE = "en_US-lessac-medium"
PIPER_ONNX = MODEL_DIR / f"{PIPER_VOICE}.onnx"
PIPER_JSON = MODEL_DIR / f"{PIPER_VOICE}.onnx.json"
PIPER_BASE = (
    "https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/lessac/medium"
)

SAMPLES = [
    ("s1", "Love is eternal"),
    ("s2", "The container holds many items"),
    ("s3", "What is the capital of France"),
    ("s4", "Water boils at one hundred degrees Celsius"),
    ("s5", "A quick brown fox jumps over the lazy dog"),
]


def download_file(url: str, dest: Path) -> bool:
    dest.parent.mkdir(parents=True, exist_ok=True)
    if dest.is_file() and dest.stat().st_size > 1000:
        print(f"  skip (exists): {dest.name}")
        return True
    print(f"  download: {dest.name}")
    try:
        urllib.request.urlretrieve(url, dest)
        return dest.is_file()
    except Exception as e:
        print(f"  download failed ({dest.name}): {e}")
        return False


def ensure_piper_model() -> bool:
    print("=== Piper TTS model ===")
    ok1 = download_file(f"{PIPER_BASE}/{PIPER_ONNX.name}", PIPER_ONNX)
    ok2 = download_file(f"{PIPER_BASE}/{PIPER_JSON.name}", PIPER_JSON)
    return ok1 and ok2


def synthesize_piper(text: str, wav_path: Path) -> bool:
    try:
        from piper import PiperVoice

        voice = PiperVoice.load(str(PIPER_ONNX), config_path=str(PIPER_JSON))
        with wav_path.open("wb") as f:
            voice.synthesize(text, f)
        return wav_path.is_file()
    except Exception as e:
        print(f"  piper failed: {e}")
        return False


async def synthesize_edge(text: str, wav_path: Path) -> bool:
    try:
        import edge_tts

        mp3 = wav_path.with_suffix(".mp3")
        communicate = edge_tts.Communicate(text, "en-US-AriaNeural")
        await communicate.save(str(mp3))
        if not mp3.is_file():
            return False
        subprocess.run(
            ["ffmpeg", "-y", "-i", str(mp3), "-ar", "22050", "-ac", "1", str(wav_path)],
            check=True,
            capture_output=True,
        )
        mp3.unlink(missing_ok=True)
        return wav_path.is_file()
    except Exception as e:
        print(f"  edge-tts failed: {e}")
        return False


def synthesize_espeak(text: str, wav_path: Path) -> bool:
    try:
        subprocess.run(
            ["espeak", "-w", str(wav_path), text],
            check=True,
            capture_output=True,
        )
        return wav_path.is_file()
    except Exception as e:
        print(f"  espeak failed: {e}")
        return False


def encode_wav(path: Path) -> tuple[list[float], str]:
    proc = subprocess.run(
        [sys.executable, str(ENCODE_PY)],
        input=json.dumps({"modality": "audio_path", "path": str(path)}),
        capture_output=True,
        text=True,
        check=True,
    )
    out = json.loads(proc.stdout)
    return out["embedding"], out.get("encoder", "unknown")


def build_fusion_payload(samples: list[dict], cycles: int) -> dict:
    return {"optimize_cycles": cycles, "samples": samples}


def run_fusion(payload: dict) -> dict:
    if not FUSION_BIN.is_file():
        raise FileNotFoundError(
            f"Build first: cargo build -p hub-f64 --release --bin valhalla_tile_stem_tts_fusion\n  missing {FUSION_BIN}"
        )
    proc = subprocess.run(
        [str(FUSION_BIN)],
        input=json.dumps(payload),
        capture_output=True,
        text=True,
        check=True,
        env={**dict(**__import__("os").environ), "RUSTFLAGS": "-L /opt/cuda/lib64"},
    )
    return json.loads(proc.stdout)


def main() -> None:
    parser = argparse.ArgumentParser(description="TTS → Tile/Stem multimodal fusion test")
    parser.add_argument(
        "--tts",
        choices=["piper", "edge", "espeak", "auto"],
        default="auto",
        help="TTS backend (auto: piper → edge → espeak)",
    )
    parser.add_argument("--cycles", type=int, default=2)
    parser.add_argument("--skip-download", action="store_true")
    args = parser.parse_args()

    OUT_DIR.mkdir(parents=True, exist_ok=True)
    piper_ready = PIPER_ONNX.is_file() and PIPER_JSON.is_file()
    if not args.skip_download and args.tts in ("piper", "auto") and not piper_ready:
        piper_ready = ensure_piper_model()

    print("\n=== Synthesize + encode ===")
    fusion_samples: list[dict] = []
    tts_used: list[str] = []

    for sid, text in SAMPLES:
        wav = OUT_DIR / f"{sid}.wav"
        ok = False
        backend = args.tts

        if backend == "auto":
            for candidate in ("piper", "edge", "espeak"):
                if candidate == "piper" and not piper_ready:
                    continue
                if candidate == "piper":
                    ok = synthesize_piper(text, wav)
                elif candidate == "edge":
                    ok = asyncio.run(synthesize_edge(text, wav))
                else:
                    ok = synthesize_espeak(text, wav)
                if ok:
                    backend = candidate
                    break
        elif backend == "piper":
            ok = synthesize_piper(text, wav)
        elif backend == "edge":
            ok = asyncio.run(synthesize_edge(text, wav))
        else:
            ok = synthesize_espeak(text, wav)

        if not ok:
            print(f"  FAIL {sid}: no TTS output for {text!r}")
            continue

        tts_used.append(backend)
        emb, encoder = encode_wav(wav)
        print(f"  {sid} [{backend}] wav={wav.name} encoder={encoder} dim={len(emb)}")
        fusion_samples.append({"id": sid, "text": text, "audio_features": emb})

    if not fusion_samples:
        print("No samples synthesized — install piper-tts, edge-tts, or use espeak.")
        sys.exit(1)

    print(f"\n=== Tile + Stem fusion (n={len(fusion_samples)}, cycles={args.cycles}) ===")
    payload = build_fusion_payload(fusion_samples, args.cycles)
    report = run_fusion(payload)
    report["tts_backend"] = max(set(tts_used), key=tts_used.count) if tts_used else "none"
    report["wav_dir"] = str(OUT_DIR)

    out_json = OUT_DIR / "fusion_report.json"
    out_json.write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(json.dumps(report, indent=2))
    print(f"\nWrote {out_json}")


if __name__ == "__main__":
    main()
