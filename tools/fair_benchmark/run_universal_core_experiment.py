#!/usr/bin/env python3
"""Universal core experiment — same task completed via text / TTS / image separately.

Tests:
  1. Do three modality-isolated runs share a common Fate patch core?
  2. Does tri-modal fusion form a tighter universal core (fusion ↔ centroid)?

Usage:
  python run_universal_core_experiment.py --phase smoke --limit 4
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
FAIR = REPO / "tools/fair_benchmark"
ENCODE = REPO / "tools/valhalla_dl_front/encode.py"
GEN_IMG = REPO / "tools/valhalla_dl_front/generate_image.py"
CORE_BIN = REPO / "target/release/valhalla_universal_core"
OUT_DIR = REPO / "reports/natural_multimodal_potential"

sys.path.insert(0, str(FAIR))
from run_fair_benchmark import load_suite, select_prompts  # noqa: E402
from run_strict_incubation_experiment import strict_user_text  # noqa: E402


def ensure_bin() -> None:
    if CORE_BIN.is_file():
        return
    subprocess.run(
        ["cargo", "build", "-p", "hub-f64", "--release", "--bin", "valhalla_universal_core"],
        cwd=REPO,
        check=True,
        env={**os.environ, "RUSTFLAGS": "-L /opt/cuda/lib64"},
    )


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


def gen_image(prompt: str, sample_id: str, img_dir: Path, model_dir: Path) -> list[float]:
    proc = subprocess.run(
        [sys.executable, str(GEN_IMG)],
        input=json.dumps({"prompt": prompt, "id": sample_id, "out_dir": str(img_dir), "model_dir": str(model_dir)}),
        capture_output=True,
        text=True,
        check=True,
    )
    for line in reversed(proc.stdout.splitlines()):
        line = line.strip()
        if line.startswith("{"):
            return json.loads(line)["image_features"]
    return json.loads(proc.stdout)["image_features"]


def build_samples(prompts: list[dict], img_dir: Path, model_dir: Path) -> list[dict]:
    samples: list[dict] = []
    with tempfile.TemporaryDirectory() as td:
        tmp = Path(td)
        for i, p in enumerate(prompts):
            task_text = strict_user_text(p)
            sid = p["id"]
            print(
                f"[{i+1}/{len(prompts)}] {sid}: same task → text | TTS | image",
                file=sys.stderr,
            )
            wav = tmp / f"{sid}.wav"
            tts(task_text, wav)
            samples.append(
                {
                    "id": sid,
                    "text": task_text,
                    "audio_features": encode_wav(wav),
                    "image_features": gen_image(task_text, sid, img_dir, model_dir),
                }
            )
    return samples


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--phase", default="smoke")
    ap.add_argument("--limit", type=int, default=0)
    ap.add_argument("--image-model-dir", type=Path, default=REPO / "models/image_gen/sd-turbo")
    ap.add_argument("--isolation-rounds", type=int, default=6)
    args = ap.parse_args()

    ensure_bin()
    suite = load_suite()
    prompts = [p for p in select_prompts(suite, args.phase) if p.get("score_type") in ("open", "mcq")]
    if args.limit > 0:
        prompts = prompts[: args.limit]
    if not prompts:
        sys.exit(f"No prompts for phase={args.phase}")

    img_dir = OUT_DIR / "universal_core_images" / args.phase
    samples = build_samples(prompts, img_dir, args.image_model_dir)

    env = {
        **os.environ,
        "RUSTFLAGS": "-L /opt/cuda/lib64",
        "VALHALLA_FREE_MULTIMODAL": "1",
        "VALHALLA_UNIVERSAL_CORE_ISOLATION_ROUNDS": str(args.isolation_rounds),
        "VALHALLA_FREE_MULTIMODAL_MIN_CYCLES": "8",
    }

    print(f"\n=== Universal core test ({len(samples)} tasks) ===", file=sys.stderr)
    proc = subprocess.run(
        [str(CORE_BIN)],
        input=json.dumps({"samples": samples}),
        capture_output=True,
        text=True,
        env=env,
    )
    if proc.stderr:
        print(proc.stderr, file=sys.stderr, end="")

    OUT_DIR.mkdir(parents=True, exist_ok=True)
    summary_path = OUT_DIR / f"universal_core_{args.phase}_summary.json"
    full_path = OUT_DIR / f"universal_core_{args.phase}.json"

    if not proc.stdout.strip():
        sys.exit(f"binary failed rc={proc.returncode}\n{proc.stderr}")

    data = json.loads(proc.stdout)
    s = data["summary"]

    # Slim summary for git
    slim = {
        "protocol": s["protocol"],
        "phase": args.phase,
        "sample_count": s["sample_count"],
        "common_core_rate": s["common_core_rate"],
        "universal_core_formation_rate": s["universal_core_formation_rate"],
        "avg_shared_patch": s["avg_shared_patch"],
        "avg_fusion_vs_centroid": s["avg_fusion_vs_centroid"],
        "avg_core_tightening": s["avg_core_tightening"],
        "avg_universal_core_score": s["avg_universal_core_score"],
        "verdict": s["verdict"],
        "per_task": [
            {
                "id": o["task_id"],
                "shared_patch": o["metrics"]["shared_patch_pairwise_avg"],
                "fusion_vs_centroid": o["metrics"]["fusion_vs_centroid_cosine"],
                "shares_common_core": o["metrics"]["shares_common_core"],
                "universal_core_formed": o["metrics"]["universal_core_formed"],
            }
            for o in data["samples"]
        ],
    }
    summary_path.write_text(json.dumps(slim, indent=2, ensure_ascii=False), encoding="utf-8")
    full_path.write_text(json.dumps(data, indent=2, ensure_ascii=False), encoding="utf-8")

    print(f"\n=== Universal Core ({args.phase}) ===")
    print(f"Tasks: {s['sample_count']}")
    print(f"共用核心 rate: {s['common_core_rate']:.1%}  (avg patch align {s['avg_shared_patch']:.3f})")
    print(f"通用核心形成 rate: {s['universal_core_formation_rate']:.1%}  (fusion↔centroid {s['avg_fusion_vs_centroid']:.3f})")
    print(f"Core tightening: {s['avg_core_tightening']:+.3f}")
    print(f"Verdict: {s['verdict']}")
    print(f"→ {summary_path}")

    if proc.returncode != 0:
        sys.exit(proc.returncode)


if __name__ == "__main__":
    main()
