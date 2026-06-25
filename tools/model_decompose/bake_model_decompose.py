#!/usr/bin/env python3
"""Model decompose → Fate adapt → bake **new LM checkpoint** (not runtime patch).

Flow:
  1. shred_qwen_weights.py  — Qwen layers → Hub/Tile/Stem weight shards
  2. valhalla_model_fate    — Fate multi-cycle adapt → adapted structure
  3. bake_valhalla_model    — write new safetensors + valhalla_manifest.json

The base HuggingFace model is never modified in-place.

Usage:
  python tools/model_decompose/bake_model_decompose.py \\
    --base-model Qwen/Qwen2.5-0.5B-Instruct \\
    --out checkpoints/valhalla_model_fate/Qwen2.5-0.5B-Instruct_model_fate_s0.08

  # Then benchmark the **new model** directly:
  valhalla_fused_lm --model checkpoints/valhalla_model_fate/... --phase test
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
SHRED = REPO / "tools/model_decompose/shred_qwen_weights.py"
FATE_BIN = REPO / "target/release/valhalla_model_fate"
BRIDGE = REPO / "tools/valhalla_model_bridge"
PATCH_ANCHOR = (
    "Valhalla model decompose anchor: LM layer quads adapted by Fate into new weights."
)


def ensure_bins() -> None:
    if FATE_BIN.exists():
        return
    subprocess.run(
        [
            "cargo",
            "build",
            "-p",
            "hub-f64",
            "--release",
            "--bin",
            "valhalla_model_fate",
        ],
        cwd=REPO,
        check=True,
        env={**os.environ, "RUSTFLAGS": "-L /opt/cuda/lib64"},
    )


def shred_model(base_model: str, shards_path: Path) -> dict:
    shards_path.parent.mkdir(parents=True, exist_ok=True)
    subprocess.run(
        [sys.executable, str(SHRED), "--model", base_model, "--out", str(shards_path)],
        check=True,
        cwd=REPO,
    )
    return json.loads(shards_path.read_text(encoding="utf-8"))


def run_model_fate(
    shards_path: Path,
    staging_dir: Path,
    *,
    base_model: str,
    bake_mode: str,
    strength: float,
    cycles_per_shard: int,
) -> dict:
    ensure_bins()
    staging_dir.mkdir(parents=True, exist_ok=True)
    env = {
        **os.environ,
        "RUSTFLAGS": "-L /opt/cuda/lib64",
        "VALHALLA_MODEL_FATE_CYCLES": "1",
    }
    proc = subprocess.run(
        [
            str(FATE_BIN),
            "--shards",
            str(shards_path),
            "--manifest-out",
            str(staging_dir),
            "--base-model-id",
            base_model,
            "--bake-mode",
            bake_mode,
            "--strength",
            str(strength),
            "--cycles-per-shard",
            str(cycles_per_shard),
        ],
        capture_output=True,
        text=True,
        check=True,
        cwd=REPO,
        env=env,
    )
    if proc.stderr:
        print(proc.stderr, file=sys.stderr, end="")
    payload = json.loads(proc.stdout)
    return payload


def bake_new_model(
    base_model: str,
    export_doc: dict,
    *,
    bake_mode: str,
    strength: float,
    out_dir: Path,
    staging_dir: Path,
) -> dict:
    import shutil

    sys.path.insert(0, str(BRIDGE))
    sys.path.insert(0, str(REPO / "tools/tier_a_tf_front"))
    from bake_valhalla_model import bake_weights, finalize_manifest  # noqa: E402

    out_dir.mkdir(parents=True, exist_ok=True)
    staging_manifest = staging_dir / "valhalla_manifest.json"
    if staging_manifest.is_file():
        shutil.copy(staging_manifest, out_dir / "valhalla_manifest.json")

    metrics = bake_weights(
        base_model,
        export_doc,
        body="triad",
        bake_mode=bake_mode,
        strength=strength,
        corpus=[PATCH_ANCHOR],
        out_dir=out_dir,
        compose_adapter=bake_mode in ("full", "total"),
    )
    finalize_manifest(out_dir, metrics)
    return metrics


def merge_model_fate_manifest(out_dir: Path, fate_payload: dict, plan: dict) -> None:
    manifest_path = out_dir / "valhalla_manifest.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    outcome = fate_payload.get("model_fate") or {}
    manifest["product"] = "valhalla-model-fate-bake"
    manifest["runtime_patch"] = False
    manifest["model_fate"] = {
        "protocol": plan.get("protocol", "valhalla-model-decompose-v1"),
        "base_model": plan.get("base_model"),
        "layer_count": plan.get("layer_count"),
        "adaptation_rounds": outcome.get("adaptation_rounds"),
        "stopped_reason": outcome.get("stopped_reason"),
        "model_closure_score": outcome.get("model_closure_score"),
        "decompose_score": outcome.get("decompose_score"),
        "aggregate_score": outcome.get("aggregate_score"),
        "baked_at": datetime.now(timezone.utc).isoformat(),
    }
    manifest_path.write_text(json.dumps(manifest, indent=2, ensure_ascii=False), encoding="utf-8")


def main() -> None:
    ap = argparse.ArgumentParser(description="Decompose LM → Fate adapt → new checkpoint")
    ap.add_argument("--base-model", default="Qwen/Qwen2.5-0.5B-Instruct")
    ap.add_argument("--out", type=Path, required=True, help="Output **new model** directory")
    ap.add_argument("--strength", type=float, default=0.08)
    ap.add_argument("--bake-mode", choices=("legacy", "full", "total"), default="full")
    ap.add_argument("--cycles-per-shard", type=int, default=2)
    ap.add_argument(
        "--keep-staging",
        action="store_true",
        help="Keep intermediate shards.json + staging/ under out parent",
    )
    args = ap.parse_args()

    tag = args.base_model.split("/")[-1]
    out_dir = args.out
    if not args.out.name or args.out.suffix:
        out_dir = args.out / f"{tag}_model_fate_s{args.strength:.2f}"

    work = out_dir.parent / f".{out_dir.name}_work"
    shards_path = work / "model_shards.json"
    staging_dir = work / "staging"

    print(
        f"=== Model decompose bake: {args.base_model} → {out_dir} "
        f"mode={args.bake_mode} strength={args.strength} ===",
        file=sys.stderr,
    )

    plan = shred_model(args.base_model, shards_path)
    print(
        f"  shredded: layers={plan['layer_count']} shards={len(plan['shards'])}",
        file=sys.stderr,
    )

    fate_payload = run_model_fate(
        shards_path,
        staging_dir,
        base_model=args.base_model,
        bake_mode=args.bake_mode,
        strength=args.strength,
        cycles_per_shard=args.cycles_per_shard,
    )
    export_doc = fate_payload["export_doc"]
    mp = export_doc["report"]["model_patch"]
    outcome = fate_payload.get("model_fate", {})
    print(
        f"  model_fate: rounds={outcome.get('adaptation_rounds')} "
        f"stopped={outcome.get('stopped_reason')} "
        f"tiles={mp.get('tile_count')} stem={mp.get('stem_cell_count')}",
        file=sys.stderr,
    )

    print("=== Baking adapted structure into NEW checkpoint ===", file=sys.stderr)
    metrics = bake_new_model(
        args.base_model,
        export_doc,
        bake_mode=args.bake_mode,
        strength=args.strength,
        out_dir=out_dir,
        staging_dir=staging_dir,
    )
    merge_model_fate_manifest(out_dir, fate_payload, plan)

    # Copy audit artifacts into model dir
    audit_dir = out_dir / "valhalla_model_fate_audit"
    audit_dir.mkdir(exist_ok=True)
    for src, name in [
        (shards_path, "model_shards.json"),
        (staging_dir / "model_fate_outcome.json", "model_fate_outcome.json"),
    ]:
        if src.is_file():
            (audit_dir / name).write_text(src.read_text(encoding="utf-8"), encoding="utf-8")

    if not args.keep_staging:
        import shutil

        if work.is_dir():
            shutil.rmtree(work, ignore_errors=True)

    print(f"\nNew model saved → {out_dir}", file=sys.stderr)
    print(f"  config.json + model.safetensors + tokenizer", file=sys.stderr)
    print(f"  valhalla_manifest.json  runtime_patch=false  product=valhalla-model-fate-bake", file=sys.stderr)
    print(f"  weight_delta_norm={metrics.get('weight_delta_norm', 0.0):.6f}", file=sys.stderr)
    print("\nBenchmark the new model (no runtime patch):", file=sys.stderr)
    print(f"  valhalla_fused_lm --model {out_dir} --phase test", file=sys.stderr)


if __name__ == "__main__":
    main()
