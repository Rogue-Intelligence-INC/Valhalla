#!/usr/bin/env python3
"""Shared benchmark defaults — 3B backbone + CUDA GPU for ValhallaBase lm_patch."""

from __future__ import annotations

import os

DEFAULT_LM_MODEL = "Qwen/Qwen2.5-3B-Instruct"
DEFAULT_LM_MODEL_ID = "qwen_3b"
LEGACY_LM_MODEL = "Qwen/Qwen2.5-0.5B-Instruct"

CUDA_LIB = "/opt/cuda/targets/x86_64-linux/lib"
CUDA_RUSTFLAGS = "-L /opt/cuda/lib64"


def default_lm_model() -> str:
    return os.environ.get("VALHALLA_LM_MODEL", DEFAULT_LM_MODEL).strip()


def apply_gpu_env() -> dict[str, str]:
    """Set LIBRARY_PATH / RUSTFLAGS / CPU policy for Rust Candle CUDA."""
    lib = os.environ.get("LIBRARY_PATH", "")
    if CUDA_LIB not in lib:
        lib = f"{CUDA_LIB}:{lib}" if lib else CUDA_LIB
    env = {
        **os.environ,
        "LIBRARY_PATH": lib,
        "RUSTFLAGS": os.environ.get("RUSTFLAGS", CUDA_RUSTFLAGS),
        "VALHALLA_USE_CPU": os.environ.get("VALHALLA_USE_CPU", "0"),
        "VALHALLA_LM_MODEL": default_lm_model(),
    }
    os.environ.update(env)
    return env


def production_valhalla_base_kwargs(**overrides):
    """Kwargs aligned with benchmark production_base + 3B default."""
    from valhalla_base import ValhallaBase  # noqa: WPS433 — local import

    base = {
        "lm_model": default_lm_model(),
        "decode": "hybrid",
        "mcq_decode": "lm_patch",
        "follow_up_decode": "native_follow_up_aware",
        "fate_ingress_routing": "quad_cycle",
        "fate_qa_feedback": False,
        "incubation_cycles": 2,
        "max_new": 128,
    }
    base.update(overrides)
    return ValhallaBase(**base)
