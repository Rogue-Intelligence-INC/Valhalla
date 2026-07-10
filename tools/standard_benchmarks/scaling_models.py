#!/usr/bin/env python3
"""Model ladder — compat wrapper over backbone_registry."""

from __future__ import annotations

from dataclasses import dataclass
import re
from typing import Any

from backbone_registry import Backbone, PRESETS, list_presets, paper_panel, spec_by_id


@dataclass(frozen=True)
class ScalingModel:
    id: str
    hf_id: str
    params_b: float
    family: str
    lm_patch_rust: bool


def _params_b(hf_id: str) -> float:
    for pat in (r"(\d+(?:\.\d+)?)[bB]", r"[-_](\d+(?:\.\d+)?)b[-_]"):
        m = re.search(pat, hf_id, re.I)
        if m:
            return float(m.group(1))
    return 0.5


def _to_scaling(b: Backbone) -> ScalingModel:
    return ScalingModel(
        id=b.id,
        hf_id=b.hf_id,
        params_b=_params_b(b.hf_id),
        family=b.category,
        lm_patch_rust=b.lm_patch_rust,
    )


SCALING_LADDER = [_to_scaling(b) for b in PRESETS.values() if b.kind == "causal_lm"]
MAMBA_LADDER = [_to_scaling(b) for b in list_presets("ssm")]
RWKV7_LADDER = [_to_scaling(b) for b in list_presets("rwkv")]
MOE_LADDER = [_to_scaling(b) for b in list_presets("moe")]
ENCODER_LADDER = [_to_scaling(b) for b in list_presets("encoder")]
VISION_LADDER = [_to_scaling(b) for b in list_presets("vision")]
ALT_BACKBONE_LADDER = RWKV7_LADDER + MOE_LADDER + ENCODER_LADDER + VISION_LADDER
QWEN_LM_PATCH_LADDER = [m for m in SCALING_LADDER if m.lm_patch_rust]
PAPER_PANEL = [_to_scaling(b) for b in paper_panel()]


def ladder_by_id(model_id: str) -> ScalingModel | None:
    b = spec_by_id(model_id)
    if b and b.kind == "causal_lm":
        return _to_scaling(b)
    return next((m for m in QWEN_LM_PATCH_LADDER if m.id == model_id), None)


def ladder_dicts() -> list[dict[str, Any]]:
    return [
        {"id": m.id, "hf_id": m.hf_id, "params_b": m.params_b,
         "family": m.family, "lm_patch_rust": m.lm_patch_rust}
        for m in SCALING_LADDER
    ]
