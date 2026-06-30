#!/usr/bin/env python3
"""Model ladder for scaling / backbone universality experiments."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any


@dataclass(frozen=True)
class ScalingModel:
    id: str
    hf_id: str
    params_b: float
    family: str
    lm_patch_rust: bool  # Candle Qwen2 path in valhalla_base


SCALING_LADDER: list[ScalingModel] = [
    ScalingModel(
        id="qwen_0.5b",
        hf_id="Qwen/Qwen2.5-0.5B-Instruct",
        params_b=0.5,
        family="qwen2",
        lm_patch_rust=True,
    ),
    ScalingModel(
        id="qwen_3b",
        hf_id="Qwen/Qwen2.5-3B-Instruct",
        params_b=3.0,
        family="qwen2",
        lm_patch_rust=True,
    ),
    ScalingModel(
        id="qwen_7b",
        hf_id="Qwen/Qwen2.5-7B-Instruct",
        params_b=7.0,
        family="qwen2",
        lm_patch_rust=True,
    ),
    ScalingModel(
        id="gemma_2b",
        hf_id="google/gemma-2-2b-it",
        params_b=2.0,
        family="gemma2",
        lm_patch_rust=False,
    ),
    ScalingModel(
        id="phi_3.8b",
        hf_id="microsoft/Phi-3-mini-4k-instruct",
        params_b=3.8,
        family="phi3",
        lm_patch_rust=False,
    ),
]

# Qwen models usable with Rust Candle lm_patch (Qwen2 architecture)
QWEN_LM_PATCH_LADDER: list[ScalingModel] = [m for m in SCALING_LADDER if m.lm_patch_rust]


def ladder_by_id(model_id: str) -> ScalingModel | None:
    for m in SCALING_LADDER:
        if m.id == model_id:
            return m
    return None


def ladder_dicts() -> list[dict[str, Any]]:
    return [
        {
            "id": m.id,
            "hf_id": m.hf_id,
            "params_b": m.params_b,
            "family": m.family,
            "lm_patch_rust": m.lm_patch_rust,
        }
        for m in SCALING_LADDER
    ]
