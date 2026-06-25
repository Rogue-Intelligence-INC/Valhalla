#!/usr/bin/env python3
"""Compare universal core: hash-encoded vs real TTS + SD-turbo features.

Usage:
  python3 tools/valhalla_inference/run_real_modality_core_compare.py --limit 12
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import tempfile
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
FAIR = REPO / "tools/fair_benchmark"
ENCODE = REPO / "tools/valhalla_dl_front/encode.py"
GEN_IMG = REPO / "tools/valhalla_dl_front/generate_image.py"
CORE_BIN = REPO / "target/release/valhalla_universal_core"
REPORTS = REPO / "reports/valhalla_inference"

sys.path.insert(0, str(HERE))
sys.path.insert(0, str(FAIR))

from run_fair_benchmark import load_suite, select_prompts  # noqa: E402
from run_scale_benchmark import build_sample, run_universal_core_compare  # noqa: E402
from run_strict_incubation_experiment import strict_user_text  # noqa: E402


def ensure_bin() -> None:
    if CORE_BIN.is_file():
        return
    subprocess.run(
        ["cargo", "build", "-p", "hub-f64", "--release", "--bin", "valhalla_universal_core"],
        check=True,
        cwd=REPO,
        env={**os.environ, "RUSTFLAGS": "-L /opt/cuda/lib64"},
    )


def tts(text: str, wav: Path) -> None:
    subprocess.run(["espeak", "-w", str(wav), text[:400]], check=True, capture_output=True)


def encode_wav(wav: Path) -> list[float]:
    proc = subprocess.run(
        [sys.executable, str(ENCODE)],
        input=json.dumps({"modality": "audio_path", "path": str(wav)}),
        capture_output=True,
        text=True,
        check=True,
    )
    return json.loads(proc.stdout)["embedding"]


def gen_image(prompt: str, sample_id: str, img_dir: Path, model_dir: Path) -> list[float]:
    proc = subprocess.run(
        [sys.executable, str(GEN_IMG)],
        input=json.dumps(
            {"prompt": prompt[:300], "id": sample_id, "out_dir": str(img_dir), "model_dir": str(model_dir)}
        ),
        capture_output=True,
        text=True,
        check=True,
    )
    for line in reversed(proc.stdout.splitlines()):
        line = line.strip()
        if line.startswith("{"):
            return json.loads(line)["image_features"]
    return json.loads(proc.stdout)["image_features"]


def build_real_samples(prompts: list[dict], img_dir: Path, model_dir: Path) -> list[dict]:
    samples: list[dict] = []
    img_dir.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory() as td:
        tmp = Path(td)
        for i, p in enumerate(prompts):
            sid = p["id"]
            text = strict_user_text(p)
            print(f"  real [{i+1}/{len(prompts)}] {sid}", file=sys.stderr)
            wav = tmp / f"{sid}.wav"
            tts(text, wav)
            samples.append(
                {
                    "id": sid,
                    "text": text,
                    "audio_features": encode_wav(wav),
                    "image_features": gen_image(text, sid, img_dir, model_dir),
                }
            )
    return samples


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--phase", default="test")
    ap.add_argument("--limit", type=int, default=12)
    ap.add_argument("--image-model-dir", type=Path, default=REPO / "models/image_gen/sd-turbo")
    args = ap.parse_args()

    ensure_bin()
    suite = load_suite()
    prompts = [p for p in select_prompts(suite, args.phase) if p.get("score_type") in ("open", "mcq")]
    prompts = prompts[: args.limit]

    hash_samples = [build_sample(p, None) for p in prompts]
    img_dir = REPORTS / "real_modality_images" / args.phase
    real_samples = build_real_samples(prompts, img_dir, args.image_model_dir)

    real_cache = REPORTS / f"real_modality_samples_{args.phase}_{args.limit}.json"
    real_cache.write_text(
        json.dumps({"samples": real_samples}, indent=2, ensure_ascii=False), encoding="utf-8"
    )

    print(f"\n=== Hash-encoded universal core (n={len(hash_samples)}) ===")
    hash_core = run_universal_core_compare(hash_samples, prompts, 6)
    print(f"  non-iso wins {hash_core['non_isolated_wins']}/{hash_core['task_count']}")
    print(f"  avg uni iso={hash_core['avg_universal_score_isolated']:.3f} non={hash_core['avg_universal_score_non_isolated']:.3f}")

    print(f"\n=== Real TTS/SD universal core (n={len(real_samples)}) ===")
    real_core = run_universal_core_compare(real_samples, prompts, 6)
    print(f"  non-iso wins {real_core['non_isolated_wins']}/{real_core['task_count']}")
    print(f"  avg uni iso={real_core['avg_universal_score_isolated']:.3f} non={real_core['avg_universal_score_non_isolated']:.3f}")

    report = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "protocol": "real-modality-core-compare-v1",
        "phase": args.phase,
        "task_count": len(prompts),
        "hash_encoded": hash_core,
        "real_tts_sd": real_core,
        "delta_non_iso_uni": (
            real_core["avg_universal_score_non_isolated"]
            - hash_core["avg_universal_score_non_isolated"]
        ),
        "verdict": (
            "REAL_MODALITY_CORE_OK"
            if real_core["non_isolated_wins"] >= len(prompts) * 0.9
            else "PARTIAL"
        ),
    }

    out = REPORTS / f"real_modality_core_compare_{args.phase}.json"
    out.write_text(json.dumps(report, indent=2, ensure_ascii=False), encoding="utf-8")
    print(f"\nΔ non-iso universal: {report['delta_non_iso_uni']:+.3f}")
    print(f"Verdict: {report['verdict']}")
    print(f"→ {out}")


if __name__ == "__main__":
    main()
