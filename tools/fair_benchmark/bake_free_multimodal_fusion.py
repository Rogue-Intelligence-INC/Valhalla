#!/usr/bin/env python3
"""Strong fusion bake — persistent TTS+LLM corpus fusion → new checkpoint (v2).

Unlike v1 bake (corpus grinder only), this aggregates **all phase samples**
through persistent free fusion before weight surgery.

Usage:
  python tools/fair_benchmark/bake_free_multimodal_fusion.py --phase test --strong
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
INCUBATE_BIN = REPO / "target/release/valhalla_free_multimodal_incubate"
BRIDGE = REPO / "tools/valhalla_model_bridge"

sys.path.insert(0, str(FAIR))
from run_fair_benchmark import load_suite, select_prompts  # noqa: E402
from run_strict_incubation_experiment import strict_user_text  # noqa: E402


def ensure_bin() -> None:
    if INCUBATE_BIN.is_file():
        return
    subprocess.run(
        [
            "cargo",
            "build",
            "-p",
            "hub-f64",
            "--release",
            "--bin",
            "valhalla_free_multimodal_incubate",
        ],
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


def strong_env() -> dict[str, str]:
    return {
        "VALHALLA_FREE_MULTIMODAL": "1",
        "VALHALLA_FREE_MULTIMODAL_DEEP": "1",
        "VALHALLA_FREE_MULTIMODAL_MIN_CYCLES": "12",
        "VALHALLA_FREE_MULTIMODAL_MAX_CYCLES": "96",
        "VALHALLA_FREE_MULTIMODAL_ROUNDS_PER_SAMPLE": "6",
        "VALHALLA_FREE_MULTIMODAL_CLOSURE_ROUNDS": "12",
        "VALHALLA_FREE_MULTIMODAL_PULSE_CYCLES": "3",
        "VALHALLA_FREE_MULTIMODAL_MIX_ALPHA": "0.45",
        "VALHALLA_MULTIMODAL_FUSION": "1",
    }


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--phase", default="test")
    ap.add_argument("--base-model", default="Qwen/Qwen2.5-0.5B-Instruct")
    ap.add_argument("--strength", type=float, default=0.10)
    ap.add_argument("--bake-mode", default="full", choices=("legacy", "full", "total"))
    ap.add_argument(
        "--out",
        type=Path,
        default=None,
        help="Output checkpoint dir (default checkpoints/valhalla_free_multimodal/{phase}_fused_v2_s{strength})",
    )
    ap.add_argument("--strong", action="store_true", help="Deep fusion env (default on)")
    args = ap.parse_args()

    tag = args.base_model.split("/")[-1]
    out_dir = args.out or (
        REPO / f"checkpoints/valhalla_free_multimodal/{args.phase}_fused_v2_s{args.strength:.2f}"
    )

    ensure_bin()
    suite = load_suite()
    prompts = [
        p
        for p in select_prompts(suite, args.phase)
        if p.get("score_type") in ("open", "mcq")
    ]
    if not prompts:
        sys.exit(f"No open/mcq in phase={args.phase}")

    samples: list[dict] = []
    with tempfile.TemporaryDirectory() as td:
        tmp = Path(td)
        for p in prompts:
            text = strict_user_text(p)
            wav = tmp / f"{p['id']}.wav"
            tts(text, wav)
            samples.append({"id": p["id"], "text": text, "audio_features": encode_wav(wav)})

    staging = out_dir.parent / f".{out_dir.name}_staging"
    staging.mkdir(parents=True, exist_ok=True)

    env = {**os.environ, "RUSTFLAGS": "-L /opt/cuda/lib64", **strong_env()}
    payload = {
        "samples": samples,
        "base_model_id": args.base_model,
        "strength": args.strength,
        "bake_mode": args.bake_mode,
    }
    print(
        f"=== Persistent fusion incubate: {len(samples)} samples → {staging} ===",
        file=sys.stderr,
    )
    proc = subprocess.run(
        [
            str(INCUBATE_BIN),
            "--manifest-out",
            str(staging),
        ],
        input=json.dumps(payload),
        capture_output=True,
        text=True,
        check=True,
        cwd=REPO,
        env=env,
    )
    if proc.stderr:
        print(proc.stderr, file=sys.stderr, end="")
    fusion_payload = json.loads(proc.stdout)

    sys.path.insert(0, str(BRIDGE))
    sys.path.insert(0, str(REPO / "tools/tier_a_tf_front"))
    from bake_valhalla_model import bake_weights, finalize_manifest  # noqa: E402

    export_doc = fusion_payload["export_doc"]
    pf = fusion_payload.get("persistent_fusion", {})
    print(
        f"  structure: tiles={export_doc['report']['tile_completed']} "
        f"stem={export_doc['report']['stem_cells']} "
        f"aggregate={pf.get('aggregate_score', 0):.3f}",
        file=sys.stderr,
    )

    import shutil

    out_dir.mkdir(parents=True, exist_ok=True)
    staging_manifest = staging / "valhalla_manifest.json"
    if staging_manifest.is_file():
        shutil.copy(staging_manifest, out_dir / "valhalla_manifest.json")

    print(f"=== Baking v2 fused weights → {out_dir} ===", file=sys.stderr)
    metrics = bake_weights(
        args.base_model,
        export_doc,
        body="triad",
        bake_mode=args.bake_mode,
        strength=args.strength,
        corpus=[f"free_multimodal_fusion_v2 sample={s['id']}" for s in samples[:8]],
        out_dir=out_dir,
        compose_adapter=True,
    )
    finalize_manifest(
        out_dir,
        metrics,
        base_model_id=args.base_model,
        bake_mode=args.bake_mode,
        strength=args.strength,
        corpus_lines=len(samples),
    )

    manifest_path = out_dir / "valhalla_manifest.json"
    if manifest_path.is_file():
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        manifest["product"] = "valhalla-free-multimodal-fusion-v2"
        manifest["free_multimodal"] = {
            **(manifest.get("free_multimodal") or {}),
            "phase": args.phase,
            "sample_count": len(samples),
            "strength": args.strength,
            "bake_mode": args.bake_mode,
        }
        manifest_path.write_text(json.dumps(manifest, indent=2, ensure_ascii=False), encoding="utf-8")

    report_path = REPO / "reports/natural_multimodal_potential" / f"fusion_v2_bake_{args.phase}.json"
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(
        json.dumps(
            {
                "out_dir": str(out_dir),
                "persistent_fusion": pf,
                "bake_metrics": metrics,
                "sample_count": len(samples),
            },
            indent=2,
        ),
        encoding="utf-8",
    )

    if staging.is_dir():
        shutil.rmtree(staging, ignore_errors=True)

    print(f"\nNew v2 fused model → {out_dir}", file=sys.stderr)
    print(f"  weight_delta_norm={metrics.get('weight_delta_norm', 0):.4f}", file=sys.stderr)
    print(f"  report → {report_path}", file=sys.stderr)
    print(f"\nBenchmark:", file=sys.stderr)
    print(f"  valhalla_fused_lm --model {out_dir} --phase test", file=sys.stderr)


if __name__ == "__main__":
    main()
