#!/usr/bin/env python3
"""Tri-modal natural fusion experiment — LLM + TTS + image-gen (v3).

Usage:
  python run_trimodal_fusion_experiment.py --phase smoke
  python run_trimodal_fusion_experiment.py --phase smoke --bake --strong
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
TRIMODAL_BIN = REPO / "target/release/valhalla_trimodal"
INCUBATE_BIN = REPO / "target/release/valhalla_trimodal_incubate"
OUT_DIR = REPO / "reports/natural_multimodal_potential"

sys.path.insert(0, str(FAIR))
from run_fair_benchmark import load_suite, select_prompts  # noqa: E402
from run_strict_incubation_experiment import strict_user_text  # noqa: E402


def ensure_bins(*names: str) -> None:
    missing = [n for n in names if not (REPO / f"target/release/{n}").is_file()]
    if not missing:
        return
    for bin_name in missing:
        subprocess.run(
            ["cargo", "build", "-p", "hub-f64", "--release", "--bin", bin_name],
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
        input=json.dumps(
            {
                "prompt": prompt,
                "id": sample_id,
                "out_dir": str(img_dir),
                "model_dir": str(model_dir),
            }
        ),
        capture_output=True,
        text=True,
        check=True,
    )
    if proc.stderr:
        print(proc.stderr, file=sys.stderr, end="")
    # stdout may contain stray log lines; parse last JSON object line
    for line in reversed(proc.stdout.splitlines()):
        line = line.strip()
        if line.startswith("{"):
            return json.loads(line)["image_features"]
    return json.loads(proc.stdout)["image_features"]


def strong_env() -> dict[str, str]:
    return {
        "VALHALLA_FREE_MULTIMODAL": "1",
        "VALHALLA_FREE_MULTIMODAL_DEEP": "1",
        "VALHALLA_FREE_MULTIMODAL_MIN_CYCLES": "12",
        "VALHALLA_FREE_MULTIMODAL_MAX_CYCLES": "96",
        "VALHALLA_FREE_MULTIMODAL_ROUNDS_PER_SAMPLE": "6",
        "VALHALLA_FREE_MULTIMODAL_CLOSURE_ROUNDS": "12",
        "VALHALLA_FREE_MULTIMODAL_PULSE_CYCLES": "3",
        "VALHALLA_FREE_MULTIMODAL_MIX_LLM": "0.40",
        "VALHALLA_FREE_MULTIMODAL_MIX_TTS": "0.30",
        "VALHALLA_FREE_MULTIMODAL_MIX_IMG": "0.30",
        "VALHALLA_MULTIMODAL_FUSION": "1",
    }


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--phase", default="smoke")
    ap.add_argument("--limit", type=int, default=0, help="Max samples (0=all in phase)")
    ap.add_argument("--image-model-dir", type=Path, default=REPO / "models/image_gen/sd-turbo")
    ap.add_argument("--bake", action="store_true", help="Run v3 incubate + HF bake after potential")
    ap.add_argument("--strong", action="store_true")
    ap.add_argument("--base-model", default="Qwen/Qwen2.5-0.5B-Instruct")
    ap.add_argument("--strength", type=float, default=0.10)
    args = ap.parse_args()

    ensure_bins("valhalla_trimodal")
    if args.bake:
        ensure_bins("valhalla_trimodal_incubate")

    suite = load_suite()
    prompts = [
        p
        for p in select_prompts(suite, args.phase)
        if p.get("score_type") in ("open", "mcq")
    ]
    if args.limit > 0:
        prompts = prompts[: args.limit]
    if not prompts:
        sys.exit(f"No prompts for phase={args.phase}")

    env = {**os.environ, "RUSTFLAGS": "-L /opt/cuda/lib64"}
    if args.strong:
        env.update(strong_env())

    samples: list[dict] = []
    img_out = REPO / "reports/natural_multimodal_potential/trimodal_images" / args.phase
    img_out.mkdir(parents=True, exist_ok=True)

    with tempfile.TemporaryDirectory() as td:
        tmp = Path(td)
        for i, p in enumerate(prompts):
            text = strict_user_text(p)
            wav = tmp / f"{p['id']}.wav"
            print(f"[{i+1}/{len(prompts)}] {p['id']}: TTS + image-gen...", file=sys.stderr)
            tts(text, wav)
            audio = encode_wav(wav)
            image = gen_image(text, p["id"], img_out, args.image_model_dir)
            samples.append(
                {
                    "id": p["id"],
                    "text": text,
                    "audio_features": audio,
                    "image_features": image,
                }
            )

    print(f"=== Tri-modal potential ({len(samples)} samples) ===", file=sys.stderr)
    proc = subprocess.run(
        [str(TRIMODAL_BIN)],
        input=json.dumps({"samples": samples}),
        capture_output=True,
        text=True,
        env=env,
    )
    if proc.stderr:
        print(proc.stderr, file=sys.stderr, end="")

    OUT_DIR.mkdir(parents=True, exist_ok=True)
    out_path = OUT_DIR / f"trimodal_potential_{args.phase}.json"
    samples_path = OUT_DIR / f"trimodal_samples_{args.phase}.json"
    samples_path.write_text(json.dumps({"samples": samples}, indent=2), encoding="utf-8")
    if proc.stdout.strip():
        data = json.loads(proc.stdout)
        out_path.write_text(json.dumps(data, indent=2, ensure_ascii=False), encoding="utf-8")
    else:
        sys.exit(f"trimodal binary failed: {proc.returncode}\n{proc.stderr}")

    s = data["summary"]
    print(f"\n=== Tri-modal ({args.phase}) ===")
    print(f"Samples: {s['sample_count']}")
    print(f"Emergence rate: {s['emergence_rate']:.1%}")
    print(f"Avg natural score: {s['avg_natural_multimodal_score']:.3f}")
    print(f"Verdict: {s['verdict']}")
    print(f"→ {out_path}")

    if args.bake:
        bake_script = REPO / "tools/fair_benchmark/bake_trimodal_fusion.py"
        subprocess.run(
            [
                sys.executable,
                str(bake_script),
                "--phase",
                args.phase,
                "--strength",
                str(args.strength),
                "--samples-json",
                str(out_path),
            ]
            + (["--strong"] if args.strong else []),
            check=True,
            cwd=REPO,
            env=env,
        )

    if not s.get("potential_confirmed"):
        sys.exit(2)


if __name__ == "__main__":
    main()
