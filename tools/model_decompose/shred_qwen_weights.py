#!/usr/bin/env python3
"""Decompose Qwen decoder weights → Valhalla Hub / Tile / Stem ports (model-level, not text shards).

Each layer quad (early→late) becomes a MODEL_QUAD_* shard with body mapping:
  Q0 → Hub, Q1 → Tile, Q2 → Stem, Q3 → Hub (late layers / aggregate)

Usage:
  python tools/model_decompose/shred_qwen_weights.py \\
    --model Qwen/Qwen2.5-0.5B-Instruct \\
    --out reports/model_decompose/qwen_shards.json
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO / "tools" / "transformer_grinder"))

from grind import (  # noqa: E402
    BODY_PORT_SEED_HUB,
    BODY_PORT_SEED_STEM,
    BODY_PORT_SEED_TILE,
    EMBED_DIM,
    get_port_signal_cap,
    layer_quads,
    ports_from_quads,
    project_port,
    _mix_vectors,
    _normalize,
    _pad_dim,
)

NUM_QUADS = 4
BODY_FOR_QUAD = ("hub", "tile", "stem", "hub")


def layer_weight_signature(layer) -> list[float]:
    """Hidden-d signature from decoder layer norms + attn row mean."""
    import torch

    parts: list[torch.Tensor] = []
    for attr in ("input_layernorm", "post_attention_layernorm"):
        ln = getattr(layer, attr, None)
        if ln is not None and hasattr(ln, "weight"):
            w = ln.weight.detach().float()
            parts.append(w if w.dim() == 1 else w.mean(dim=0))
    attn = getattr(layer, "self_attn", None)
    if attn is not None:
        for name in ("q_proj", "k_proj", "v_proj", "o_proj"):
            lin = getattr(attn, name, None)
            if lin is not None and hasattr(lin, "weight"):
                w = lin.weight.detach().float()
                parts.append(w.mean(dim=0))
    if not parts:
        return [0.0] * EMBED_DIM
    acc = parts[0].clone()
    for p in parts[1:]:
        n = min(acc.numel(), p.numel())
        acc[:n] = acc[:n] + p[:n]
    acc = acc / len(parts)
    vec = acc.tolist()
    return _normalize(_pad_dim(vec, EMBED_DIM))


def project_hidden_to_embed(vec: list[float], dim: int = EMBED_DIM) -> list[float]:
    if len(vec) <= dim:
        return _normalize(_pad_dim(vec, dim))
    step = len(vec) / dim
    out = [vec[int(i * step)] for i in range(dim)]
    return _normalize(out)


def quad_ports(quad_means: list[list[float]], quad_idx: int, encoder: str) -> dict:
    """Single quad → body-specific port (primary body gets quad embedding)."""
    cap = get_port_signal_cap()
    emb = _pad_dim(quad_means[quad_idx], EMBED_DIM)
    body = BODY_FOR_QUAD[quad_idx]
    if body == "hub":
        mix = _mix_vectors(quad_means[0], quad_means[1], EMBED_DIM) if quad_idx == 0 else emb
        port = project_port(mix, BODY_PORT_SEED_HUB, cap)
    elif body == "tile":
        mix = _mix_vectors(quad_means[1], quad_means[2], EMBED_DIM)
        port = project_port(mix, BODY_PORT_SEED_TILE, cap)
    else:
        mix = _mix_vectors(quad_means[2], quad_means[3], EMBED_DIM)
        port = project_port(mix, BODY_PORT_SEED_STEM, cap)
    flat = _normalize([sum(_pad_dim(q, EMBED_DIM)[i] for q in quad_means) / NUM_QUADS for i in range(EMBED_DIM)])
    return {
        "hub": project_port(_mix_vectors(quad_means[0], quad_means[1], EMBED_DIM), BODY_PORT_SEED_HUB, cap),
        "tile": project_port(_mix_vectors(quad_means[1], quad_means[2], EMBED_DIM), BODY_PORT_SEED_TILE, cap),
        "stem": project_port(_mix_vectors(quad_means[2], quad_means[3], EMBED_DIM), BODY_PORT_SEED_STEM, cap),
        "embedding": flat,
        "embed_dim": EMBED_DIM,
        "encoder": f"{encoder}/quad_{quad_idx}",
        "modality_label": "model_weight",
        "_primary_body": body,
        "_primary_port_len": len(port),
    }


def _decoder_layers(model) -> list:
    """Qwen2 AutoModel vs AutoModelForCausalLM layer access."""
    if hasattr(model, "model") and hasattr(model.model, "layers"):
        return list(model.model.layers)
    if hasattr(model, "layers"):
        return list(model.layers)
    raise AttributeError("cannot find decoder layers on model")


def build_plan(model_name: str) -> dict:
    from transformers import AutoModel

    model = AutoModel.from_pretrained(model_name, trust_remote_code=True)
    model.eval()
    layers = _decoder_layers(model)
    layer_sigs = [layer_weight_signature(layer) for layer in layers]
    quad_means = layer_quads(layer_sigs)
    whole = ports_from_quads(quad_means, encoder=f"model/{model_name.replace('/', '_')}")

    shards: list[dict] = []
    for qi in range(NUM_QUADS):
        ports = quad_ports(quad_means, qi, f"model/{model_name.replace('/', '_')}")
        body = BODY_FOR_QUAD[qi]
        phase = "aggregate" if qi == NUM_QUADS - 1 else "decompose"
        shards.append(
            {
                "label": f"MODEL_QUAD_{qi}",
                "body": body,
                "phase": phase,
                "layer_range": [
                    qi * max(1, len(layers) // NUM_QUADS),
                    min((qi + 1) * max(1, len(layers) // NUM_QUADS), len(layers)),
                ],
                "ports": {k: v for k, v in ports.items() if not k.startswith("_")},
            }
        )

    shards.append(
        {
            "label": "MODEL_FATE_AGGREGATE",
            "body": "hub",
            "phase": "aggregate",
            "ports": whole,
        }
    )
    shards.append(
        {
            "label": "MODEL_RECOMPOSE",
            "body": "triad",
            "phase": "recompose",
            "ports": whole,
        }
    )

    return {
        "protocol": "valhalla-model-decompose-v1",
        "base_model": model_name,
        "layer_count": len(layers),
        "hidden_size": int(getattr(model.config, "hidden_size", EMBED_DIM)),
        "quad_dims": [len(q) for q in quad_means],
        "whole_ports": whole,
        "shards": shards,
    }


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--model", default="Qwen/Qwen2.5-0.5B-Instruct")
    ap.add_argument("--out", type=Path, required=True)
    args = ap.parse_args()

    plan = build_plan(args.model)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(plan, indent=2), encoding="utf-8")
    print(f"model decompose: {args.model} layers={plan['layer_count']} shards={len(plan['shards'])}", file=sys.stderr)
    print(f"→ {args.out}", file=sys.stderr)


if __name__ == "__main__":
    main()
