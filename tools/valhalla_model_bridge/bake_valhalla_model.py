#!/usr/bin/env python3
"""
Valhalla model bake — incubate Hub/Tile/StemCell/Triad → export structure → write NEW checkpoint.

The base HuggingFace model is never modified in-place. We copy weights to `--out`, apply
Valhalla structure once, and write `valhalla_manifest.json` for hybrid / fused / native QA.

Flow:
  1. Rust `valhalla_model_export --grinder --body {hub|tile|stemcell|triad}`
  2. Load base LM from HuggingFace (read-only cache)
  3. Apply Valhalla bake (legacy | full | total) on the copy
  4. Save new checkpoint + manifest

Usage:
  python bake_valhalla_model.py --body triad --phase test --strength 0.08 \\
    --out checkpoints/valhalla_baked/triad_fair_test_s0.08

  python bake_valhalla_model.py --body tile --corpus my_corpus.txt --bake-mode full
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
BRIDGE = Path(__file__).resolve().parent
EXPORT_BIN = REPO / "target/release/valhalla_model_export"
FAIR_SUITE = REPO / "tools/fair_benchmark/fair_benchmark_suite.json"
PATCH_ANCHOR = (
    "Valhalla incubation anchor: structure from Hub Tile StemCell Fate converges into model weights."
)

BODIES = ("hub", "tile", "stemcell", "triad")
BAKE_MODES = ("legacy", "full", "total")


def ensure_export_bin() -> None:
    if EXPORT_BIN.exists():
        return
    subprocess.run(
        [
            "cargo",
            "build",
            "-p",
            "hub-f64",
            "--release",
            "--bin",
            "valhalla_model_export",
        ],
        cwd=REPO,
        check=True,
        env={**os.environ, "RUSTFLAGS": "-L /opt/cuda/lib64"},
    )


def load_corpus(*, corpus_file: Path | None, phase: str | None) -> list[str]:
    if corpus_file is not None:
        return [
            ln.strip()
            for ln in corpus_file.read_text(encoding="utf-8").splitlines()
            if ln.strip() and not ln.strip().startswith("#")
        ]
    if phase is None:
        raise ValueError("need --corpus or --phase")
    suite = json.loads(FAIR_SUITE.read_text(encoding="utf-8"))
    phase_cfg = suite["phases"][phase]
    key = phase_cfg["corpus"]
    return [ln.strip() for ln in suite["corpus"][key] if ln.strip()]


def run_structure_export(
    body: str,
    corpus: list[str],
    *,
    cycles: int,
    out_dir: Path,
    base_model_id: str,
    bake_mode: str,
    strength: float,
) -> dict:
    ensure_export_bin()
    out_dir.mkdir(parents=True, exist_ok=True)
    proc = subprocess.run(
        [
            str(EXPORT_BIN),
            "--body",
            body,
            "--grinder",
            "--cycles",
            str(cycles),
            "--manifest-out",
            str(out_dir),
            "--base-model-id",
            base_model_id,
            "--bake-mode",
            bake_mode,
            "--strength",
            str(strength),
        ],
        input="\n".join(corpus) + "\n",
        capture_output=True,
        text=True,
        check=True,
        env={**os.environ, "RUSTFLAGS": "-L /opt/cuda/lib64"},
    )
    export_doc = json.loads(proc.stdout)
    (out_dir / "valhalla_export.json").write_text(
        json.dumps(export_doc, indent=2, ensure_ascii=False),
        encoding="utf-8",
    )
    return export_doc


def bake_weights(
    base_model: str,
    export_doc: dict,
    *,
    body: str,
    bake_mode: str,
    strength: float,
    corpus: list[str],
    out_dir: Path,
    compose_adapter: bool = False,
) -> dict:
    sys.path.insert(0, str(BRIDGE))
    sys.path.insert(0, str(REPO / "tools/tier_a_tf_front"))

    from apply_and_compare import apply_valhalla_patch, project_patch  # noqa: E402

    os.environ.setdefault("HF_ENDPOINT", "https://hf-mirror.com")
    import torch
    from transformers import AutoModelForCausalLM, AutoTokenizer

    if out_dir.exists() and any(out_dir.iterdir()):
        # Never overwrite an existing bake dir silently.
        for child in list(out_dir.iterdir()):
            if child.name in ("valhalla_export.json", "valhalla_manifest.json"):
                continue
            if child.is_dir():
                shutil.rmtree(child)
            else:
                child.unlink()

    tokenizer = AutoTokenizer.from_pretrained(base_model, trust_remote_code=True)
    dtype = torch.float16 if torch.cuda.is_available() else torch.float32
    model = AutoModelForCausalLM.from_pretrained(
        base_model,
        torch_dtype=dtype,
        device_map="auto" if torch.cuda.is_available() else None,
        trust_remote_code=True,
    )
    model.eval()

    if bake_mode == "total":
        from patch_total import apply_valhalla_patch_total  # noqa: E402

        metrics = apply_valhalla_patch_total(
            model,
            export_doc,
            strength,
            tokenizer,
            body=body,
            corpus_lines=corpus,
            anchor=PATCH_ANCHOR,
        )
    elif bake_mode == "full":
        from patch_full import apply_valhalla_patch_full  # noqa: E402

        metrics = apply_valhalla_patch_full(
            model,
            export_doc,
            strength,
            tokenizer,
            body=body,
            corpus_lines=corpus,
            anchor=PATCH_ANCHOR,
        )
    else:
        # legacy: anchor + corpus token embed perturbation
        anchor_prompt = PATCH_ANCHOR
        metrics = apply_valhalla_patch(model, export_doc, strength, anchor_prompt, tokenizer)
        mp = export_doc["report"]["model_patch"]
        vec = project_patch(mp["patch_vector"], model.config.hidden_size)
        emb = model.model.embed_tokens.weight
        for line in corpus:
            for tid in tokenizer.encode(line, add_special_tokens=False):
                emb.data[tid] += strength * vec.to(emb.device)

    if compose_adapter:
        manifest_path = out_dir / "valhalla_manifest.json"
        if manifest_path.is_file():
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            spec = manifest.get("compose_adapter")
            if spec:
                from patch_compose_adapter import apply_compose_adapter  # noqa: E402

                apply_compose_adapter(model, spec, strength, tokenizer, metrics)
                print("  compose_adapter: lm_head + late LN patched", file=sys.stderr)

    out_dir.mkdir(parents=True, exist_ok=True)
    model.save_pretrained(out_dir)
    tokenizer.save_pretrained(out_dir)

    return {
        "weight_delta_norm": float(metrics.get("weight_delta_norm", 0.0)),
        "bake_mode": bake_mode,
        "strength": strength,
    }


def finalize_manifest(
    out_dir: Path,
    bake_metrics: dict,
    *,
    base_model_id: str = "Qwen/Qwen2.5-0.5B-Instruct",
    bake_mode: str = "legacy",
    strength: float = 0.08,
    corpus_lines: int = 0,
) -> None:
    manifest_path = out_dir / "valhalla_manifest.json"
    if manifest_path.exists():
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    else:
        export_path = out_dir / "valhalla_export.json"
        if not export_path.is_file():
            raise FileNotFoundError(f"missing {manifest_path} and {export_path}")
        export_doc = json.loads(export_path.read_text(encoding="utf-8"))
        report = export_doc["report"]
        mp = report["model_patch"]
        manifest = {
            "manifest_version": "valhalla-bake-1.0",
            "product": "valhalla-baked-lm",
            "incubation": {
                "body": export_doc.get("body", "triad"),
                "ingress": export_doc.get("encoding", "grinder").lower(),
                "cycles": report.get("optimize_cycles", 1),
                "corpus_lines": corpus_lines,
                "hub_routed": report.get("hub_routed", [0, 0, 0, 0]),
                "hub_prefs": report.get("hub_prefs", []),
                "tile_count": mp.get("tile_count", 0),
                "stem_cell_count": mp.get("stem_cell_count", 0),
                "signals_emitted": report.get("stats", {}).get("signals_emitted", 0),
                "patch_fingerprint": "recovered_from_export",
            },
            "bake": {
                "base_model_id": base_model_id,
                "bake_mode": bake_mode,
                "strength": strength,
                "weight_delta_norm": None,
                "baked_at": datetime.now(timezone.utc).isoformat(),
            },
            "export_doc": export_doc,
            "runtime_patch": False,
        }

    manifest["bake"]["weight_delta_norm"] = bake_metrics["weight_delta_norm"]
    manifest["bake"]["baked_at"] = datetime.now(timezone.utc).isoformat()
    manifest["runtime_patch"] = False
    manifest_path.write_text(json.dumps(manifest, indent=2, ensure_ascii=False), encoding="utf-8")


def main() -> None:
    ap = argparse.ArgumentParser(description="Bake Valhalla structure into a new LM checkpoint")
    ap.add_argument("--body", choices=BODIES, default="triad", help="Incubation body (Hub/Tile/Stem/Triad)")
    ap.add_argument("--base-model", default="Qwen/Qwen2.5-0.5B-Instruct")
    ap.add_argument("--corpus", type=Path, default=None, help="Corpus text file (one line per item)")
    ap.add_argument("--phase", choices=["smoke", "val", "test"], default="test", help="Fair benchmark corpus tier")
    ap.add_argument("--cycles", type=int, default=1)
    ap.add_argument("--strength", type=float, default=0.08)
    ap.add_argument("--bake-mode", choices=BAKE_MODES, default="legacy")
    ap.add_argument(
        "--compose-adapter",
        action="store_true",
        help="Apply Valhalla compose adapter (Stem+Tile+Hub) to lm_head after bake",
    )
    ap.add_argument("--out", type=Path, required=True, help="Output HF model directory (new model)")
    ap.add_argument("--repo-id", default=None, help="HF repo id for README model card")
    ap.add_argument("--push-hub", action="store_true", help="Upload to Hugging Face Hub after bake")
    ap.add_argument("--public-hub", action="store_true", help="Public HF repo when --push-hub")
    ap.add_argument("--skip-hf-verify", action="store_true")
    args = ap.parse_args()

    if args.bake_mode == "total" and args.body not in ("tile", "stemcell"):
        sys.exit("--bake-mode total only supports tile or stemcell")

    corpus = load_corpus(corpus_file=args.corpus, phase=args.phase if args.corpus is None else None)
    tag = args.base_model.split("/")[-1]
    out_dir = args.out
    if not args.out.name:
        out_dir = args.out / f"{tag}_valhalla_{args.body}_s{args.strength}"

    print(
        f"=== Valhalla bake: body={args.body} corpus={len(corpus)} lines "
        f"mode={args.bake_mode} strength={args.strength} ===",
        file=sys.stderr,
    )

    export_doc = run_structure_export(
        args.body,
        corpus,
        cycles=args.cycles,
        out_dir=out_dir,
        base_model_id=args.base_model,
        bake_mode=args.bake_mode,
        strength=args.strength,
    )
    mp = export_doc["report"]["model_patch"]
    print(
        f"  structure: tiles={mp['tile_count']} stem={mp['stem_cell_count']} "
        f"signals={export_doc['report']['stats']['signals_emitted']}",
        file=sys.stderr,
    )

    print("=== Baking weights into new checkpoint (base model untouched) ===", file=sys.stderr)
    bake_metrics = bake_weights(
        args.base_model,
        export_doc,
        body=args.body,
        bake_mode=args.bake_mode,
        strength=args.strength,
        corpus=corpus,
        out_dir=out_dir,
        compose_adapter=args.compose_adapter or args.bake_mode in ("full", "total"),
    )
    finalize_manifest(
        out_dir,
        bake_metrics,
        base_model_id=args.base_model,
        bake_mode=args.bake_mode,
        strength=args.strength,
        corpus_lines=len(corpus),
    )

    from finalize_hf_repo import finalize_hf_repo  # noqa: E402

    hf_result = finalize_hf_repo(
        out_dir,
        repo_id=args.repo_id,
        verify=not args.skip_hf_verify,
        push=args.push_hub,
        private=not args.public_hub,
    )

    print(f"\nHF-ready model → {out_dir}", file=sys.stderr)
    print(f"  valhalla_manifest.json  (runtime_patch=false)", file=sys.stderr)
    print(f"  weight_delta_norm={bake_metrics['weight_delta_norm']:.6f}", file=sys.stderr)
    if hf_result.get("hub_url"):
        print(f"  hub → {hf_result['hub_url']}", file=sys.stderr)
    print("\nLoad with transformers:", file=sys.stderr)
    rid = args.repo_id or out_dir
    print(f"  AutoModelForCausalLM.from_pretrained('{rid}', trust_remote_code=True)", file=sys.stderr)
    print("\nUse with hybrid / fused (no runtime patch):", file=sys.stderr)
    print(f"  valhalla_fused_lm --model {out_dir} --phase test", file=sys.stderr)
    print(f"  valhalla_hybrid --model {out_dir} --phase test --run-patch", file=sys.stderr)


if __name__ == "__main__":
    main()
