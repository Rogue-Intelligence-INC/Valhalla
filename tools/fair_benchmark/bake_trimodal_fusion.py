#!/usr/bin/env python3
"""Bake tri-modal v3 fusion → HF model under models/hf/."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
INCUBATE_BIN = REPO / "target/release/valhalla_trimodal_incubate"
BRIDGE = REPO / "tools/valhalla_model_bridge"
FAIR = REPO / "tools/fair_benchmark"

sys.path.insert(0, str(FAIR))
from bake_free_multimodal_fusion import ensure_bin as ensure_v2_bin, strong_env, tts, encode_wav  # noqa: E402
from run_fair_benchmark import load_suite, select_prompts  # noqa: E402
from run_strict_incubation_experiment import strict_user_text  # noqa: E402

GEN_IMG = REPO / "tools/valhalla_dl_front/generate_image.py"


def ensure_trimodal_bin() -> None:
    if INCUBATE_BIN.is_file():
        return
    subprocess.run(
        ["cargo", "build", "-p", "hub-f64", "--release", "--bin", "valhalla_trimodal_incubate"],
        cwd=REPO,
        check=True,
        env={**os.environ, "RUSTFLAGS": "-L /opt/cuda/lib64"},
    )


def gen_image(prompt: str, sample_id: str, img_dir: Path, model_dir: Path) -> list[float]:
    proc = subprocess.run(
        [sys.executable, str(GEN_IMG)],
        input=json.dumps({"prompt": prompt, "id": sample_id, "out_dir": str(img_dir), "model_dir": str(model_dir)}),
        capture_output=True,
        text=True,
        check=True,
    )
    if proc.stderr:
        print(proc.stderr, file=sys.stderr, end="")
    for line in reversed(proc.stdout.splitlines()):
        line = line.strip()
        if line.startswith("{"):
            return json.loads(line)["image_features"]
    return json.loads(proc.stdout)["image_features"]


def load_samples(samples_json: Path | None, phase: str, limit: int) -> list[dict]:
    if samples_json and samples_json.is_file():
        doc = json.loads(samples_json.read_text(encoding="utf-8"))
        if "samples" in doc and doc["samples"]:
            first = doc["samples"][0]
            if "image_features" in first and "audio_features" in first:
                return doc["samples"]
            if "text" in first and "audio_features" in first:
                return doc["samples"]
    suite = load_suite()
    prompts = [p for p in select_prompts(suite, phase) if p.get("score_type") in ("open", "mcq")]
    if limit > 0:
        prompts = prompts[:limit]
    return [{"id": p["id"], "text": strict_user_text(p)} for p in prompts]


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--phase", default="smoke")
    ap.add_argument("--limit", type=int, default=0)
    ap.add_argument("--samples-json", type=Path, default=None)
    ap.add_argument("--base-model", default="Qwen/Qwen2.5-0.5B-Instruct")
    ap.add_argument("--strength", type=float, default=0.10)
    ap.add_argument("--strong", action="store_true")
    ap.add_argument("--image-model-dir", type=Path, default=REPO / "models/image_gen/sd-turbo")
    ap.add_argument("--out", type=Path, default=None)
    ap.add_argument("--skip-hf-verify", action="store_true")
    args = ap.parse_args()

    tag = args.base_model.split("/")[-1]
    out_dir = args.out or REPO / "models/hf" / f"{tag}-Valhalla-TriModal-Fusion-v3-{args.phase}-s{args.strength:.2f}"

    ensure_trimodal_bin()
    raw_samples = load_samples(args.samples_json, args.phase, args.limit)

    img_dir = REPO / "reports/natural_multimodal_potential/trimodal_images" / args.phase
    samples: list[dict] = []
    with tempfile.TemporaryDirectory() as td:
        tmp = Path(td)
        for i, s in enumerate(raw_samples):
            text = s.get("text") or s.get("prompt", "")
            sid = s["id"]
            if s.get("audio_features") and s.get("image_features"):
                samples.append(s)
                continue
            print(f"  encode [{i+1}/{len(raw_samples)}] {sid}", file=sys.stderr)
            wav = tmp / f"{sid}.wav"
            tts(text, wav)
            samples.append(
                {
                    "id": sid,
                    "text": text,
                    "audio_features": s.get("audio_features") or encode_wav(wav),
                    "image_features": s.get("image_features") or gen_image(text, sid, img_dir, args.image_model_dir),
                }
            )

    staging = out_dir.parent / f".{out_dir.name}_staging"
    staging.mkdir(parents=True, exist_ok=True)

    env = {**os.environ, "RUSTFLAGS": "-L /opt/cuda/lib64"}
    if args.strong:
        env.update(strong_env())
        env["VALHALLA_FREE_MULTIMODAL_MIX_LLM"] = "0.40"
        env["VALHALLA_FREE_MULTIMODAL_MIX_TTS"] = "0.30"
        env["VALHALLA_FREE_MULTIMODAL_MIX_IMG"] = "0.30"

    payload = {
        "samples": samples,
        "base_model_id": args.base_model,
        "strength": args.strength,
        "bake_mode": "full",
    }
    print(f"=== Tri-modal incubate v3: {len(samples)} samples ===", file=sys.stderr)
    proc = subprocess.run(
        [str(INCUBATE_BIN), "--manifest-out", str(staging)],
        input=json.dumps(payload),
        capture_output=True,
        text=True,
        check=True,
        env=env,
    )
    if proc.stderr:
        print(proc.stderr, file=sys.stderr, end="")
    fusion = json.loads(proc.stdout)

    sys.path.insert(0, str(BRIDGE))
    from bake_valhalla_model import bake_weights, finalize_manifest  # noqa: E402
    from finalize_hf_repo import finalize_hf_repo  # noqa: E402

    export_doc = fusion["export_doc"]
    import shutil

    out_dir.mkdir(parents=True, exist_ok=True)
    staging_manifest = staging / "valhalla_manifest.json"
    if staging_manifest.is_file():
        shutil.copy(staging_manifest, out_dir / "valhalla_manifest.json")

    metrics = bake_weights(
        args.base_model,
        export_doc,
        body="triad",
        bake_mode="full",
        strength=args.strength,
        corpus=[f"trimodal_fusion_v3 sample={s['id']}" for s in samples[:8]],
        out_dir=out_dir,
        compose_adapter=True,
    )
    finalize_manifest(
        out_dir,
        metrics,
        base_model_id=args.base_model,
        bake_mode="full",
        strength=args.strength,
        corpus_lines=len(samples),
    )
    manifest_path = out_dir / "valhalla_manifest.json"
    if manifest_path.is_file():
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        manifest["product"] = "valhalla-trimodal-fusion-v3"
        manifest["trimodal"] = {
            "phase": args.phase,
            "sample_count": len(samples),
            "persistent_fusion": fusion.get("persistent_fusion"),
        }
        manifest_path.write_text(json.dumps(manifest, indent=2, ensure_ascii=False), encoding="utf-8")

    finalize_hf_repo(out_dir, repo_id=None, verify=not args.skip_hf_verify)
    if staging.is_dir():
        shutil.rmtree(staging, ignore_errors=True)

    report = REPO / "reports/natural_multimodal_potential" / f"trimodal_bake_{args.phase}.json"
    report.write_text(
        json.dumps({"out_dir": str(out_dir), "bake_metrics": metrics, "fusion": fusion.get("persistent_fusion")}, indent=2),
        encoding="utf-8",
    )
    print(f"\nHF tri-modal model → {out_dir}", file=sys.stderr)
    print(f"  report → {report}", file=sys.stderr)


if __name__ == "__main__":
    main()
