#!/usr/bin/env python3
"""Attach valhalla_manifest.json to an existing baked checkpoint (legacy _patch.json)."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("checkpoint_dir", type=Path, help="Directory with model.safetensors")
    ap.add_argument("--patch-json", type=Path, default=None, help="Defaults to {dir}_patch.json sibling")
    ap.add_argument("--base-model-id", default="Qwen/Qwen2.5-0.5B-Instruct")
    ap.add_argument("--body", default="triad")
    ap.add_argument("--strength", type=float, default=0.08)
    ap.add_argument("--bake-mode", default="legacy")
    args = ap.parse_args()

    ckpt = args.checkpoint_dir
    patch_path = args.patch_json
    if patch_path is None:
        patch_path = ckpt.parent / f"{ckpt.name}_patch.json"
    if not patch_path.is_file():
        raise SystemExit(f"missing patch json: {patch_path}")

    export_doc = json.loads(patch_path.read_text(encoding="utf-8"))
    if "report" not in export_doc:
        export_doc = {"body": args.body, "encoding": export_doc.get("encoding", "NormalizedF64"), "report": export_doc}

    report = export_doc["report"]
    mp = report["model_patch"]
    manifest = {
        "manifest_version": "valhalla-bake-1.0",
        "product": "valhalla-baked-lm",
        "incubation": {
            "body": args.body,
            "ingress": "normalized_f64",
            "cycles": report.get("optimize_cycles", 1),
            "corpus_lines": report["stats"]["items_processed"],
            "hub_routed": report["hub_routed"],
            "hub_prefs": report["hub_prefs"],
            "tile_count": report.get("tile_completed", mp.get("tile_count", 0)),
            "stem_cell_count": report.get("stem_cells", mp.get("stem_cell_count", 0)),
            "signals_emitted": report["stats"]["signals_emitted"],
            "patch_fingerprint": "legacy_migrated",
        },
        "bake": {
            "base_model_id": args.base_model_id,
            "bake_mode": args.bake_mode,
            "strength": args.strength,
            "weight_delta_norm": None,
            "baked_at": datetime.now(timezone.utc).isoformat(),
        },
        "export_doc": export_doc if "body" in export_doc else {
            "body": args.body,
            "encoding": export_doc.get("encoding", "NormalizedF64"),
            "report": report,
        },
        "runtime_patch": False,
    }

    out = ckpt / "valhalla_manifest.json"
    out.write_text(json.dumps(manifest, indent=2, ensure_ascii=False), encoding="utf-8")
    (ckpt / "valhalla_export.json").write_text(json.dumps(manifest["export_doc"], indent=2), encoding="utf-8")
    print(f"Wrote {out} (runtime_patch=false)")


if __name__ == "__main__":
    main()
