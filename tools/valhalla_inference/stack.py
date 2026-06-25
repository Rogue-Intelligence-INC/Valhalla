"""Valhalla module stack ↔ Transformer bridge (registry + manifest)."""

from __future__ import annotations

import json
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

REPO = Path(__file__).resolve().parents[2]

# L2 bodies → what gets baked into Qwen weights
MODULE_STACK: list[dict[str, str]] = [
    {
        "id": "hub",
        "rust": "manifestsys/hub-f64/src/lib.rs (ValhallaHub)",
        "role": "Fate quad routing BH/MB/MN/BF",
        "transformer_touch": "hub_prefs → quad_layer_scales → LayerNorm group scaling (4×6 layers)",
    },
    {
        "id": "tile",
        "rust": "manifestsys/tile-quattro/",
        "role": "Block merge/split signatures",
        "transformer_touch": "tile_signatures → patch_vector[256] → hidden_size embed delta",
    },
    {
        "id": "stemcell",
        "rust": "manifestsys/stem-cell/",
        "role": "Cell graph energy layers",
        "transformer_touch": "stem_signatures → fused into patch_vector",
    },
    {
        "id": "triad",
        "rust": "manifestsys/hub-f64/src/signal_ingress.rs (TriadSession)",
        "role": "Hub+Tile+Stem parallel incubation",
        "transformer_touch": "build_model_patch() → export_doc → bake or runtime patch_lm",
    },
    {
        "id": "dl_front",
        "rust": "manifestsys/hub-f64/src/dl_front.rs",
        "python": "tools/valhalla_dl_front/encode.py",
        "role": "384-d modality → hub/tile/stem ports",
        "transformer_touch": "Pre-LM ingress (text/audio/image features); not generate path",
    },
    {
        "id": "patch_lm",
        "rust": "manifestsys/hub-f64/src/patch_lm.rs",
        "role": "Candle Qwen2 load + weight surgery",
        "transformer_touch": "safetensors load; baked=offline surgery, runtime=per-question patch",
    },
    {
        "id": "fused_lm",
        "rust": "manifestsys/hub-f64/src/fused_lm.rs",
        "role": "Single-track LM product (no hybrid router)",
        "transformer_touch": "MCQ logprob batch + numeric/open generate on baked weights",
    },
]

LOAD_MODES = {
    "baked_hf": "valhalla_manifest.json present; weights contain Valhalla structure (runtime_patch=false)",
    "runtime_patch": "Raw base LM + per-question Triad patch (Python apply_valhalla_patch / Rust patch_lm)",
    "vllm_compat": "Pre-baked HF dir served as plain Qwen2; vLLM ignores manifest sidecar",
}


@dataclass
class ValhallaModelInfo:
    path: Path
    base_model_id: str
    product: str | None
    body: str | None
    load_mode: str
    runtime_patch: bool
    tile_count: int | None = None
    stem_cell_count: int | None = None
    strength: float | None = None
    modules: list[dict[str, str]] = field(default_factory=lambda: list(MODULE_STACK))
    extra: dict[str, Any] = field(default_factory=dict)

    def to_dict(self) -> dict[str, Any]:
        return {
            "path": str(self.path),
            "base_model_id": self.base_model_id,
            "product": self.product,
            "body": self.body,
            "load_mode": self.load_mode,
            "runtime_patch": self.runtime_patch,
            "tile_count": self.tile_count,
            "stem_cell_count": self.stem_cell_count,
            "strength": self.strength,
            "module_stack": self.modules,
            "extra": self.extra,
        }


def inspect_model(model_dir: Path | str) -> ValhallaModelInfo:
    p = Path(model_dir).resolve()
    manifest_path = p / "valhalla_manifest.json"
    if manifest_path.is_file():
        m = json.loads(manifest_path.read_text(encoding="utf-8"))
        inc = m.get("incubation") or {}
        bake = m.get("bake") or {}
        runtime = bool(m.get("runtime_patch", False))
        return ValhallaModelInfo(
            path=p,
            base_model_id=bake.get("base_model_id") or "unknown",
            product=m.get("product"),
            body=inc.get("body"),
            load_mode="runtime_patch" if runtime else "baked_hf",
            runtime_patch=runtime,
            tile_count=inc.get("tile_count"),
            stem_cell_count=inc.get("stem_cell_count"),
            strength=bake.get("strength"),
            extra={
                "manifest_version": m.get("manifest_version"),
                "bake_mode": bake.get("bake_mode"),
                "weight_delta_norm": bake.get("weight_delta_norm"),
                "ingress": inc.get("ingress"),
            },
        )
    return ValhallaModelInfo(
        path=p,
        base_model_id=str(p.name),
        product=None,
        body=None,
        load_mode="vllm_compat",
        runtime_patch=True,
        extra={"note": "No valhalla_manifest.json — treat as raw HF / base Qwen"},
    )


def stack_report(model_dir: Path | str) -> str:
    info = inspect_model(model_dir)
    lines = [
        "Valhalla ↔ Transformer stack",
        f"  model: {info.path}",
        f"  base:  {info.base_model_id}",
        f"  mode:  {info.load_mode}",
        f"  body:  {info.body}  tiles={info.tile_count}  stem={info.stem_cell_count}",
        "",
        "Modules → Transformer:",
    ]
    for mod in MODULE_STACK:
        touch = mod.get("transformer_touch", "")
        lines.append(f"  [{mod['id']}] {mod['role']}")
        lines.append(f"         → {touch}")
    return "\n".join(lines)
