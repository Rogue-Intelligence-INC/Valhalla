#!/usr/bin/env python3
"""Backbone presets — extend PRESETS dict or use Backbone.from_hf() ad-hoc."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Literal

Kind = Literal["causal_lm", "encoder", "vision"]
Category = Literal["dense", "ssm", "rwkv", "moe", "encoder", "vision", "custom"]

CATEGORY_ORDER: list[Category] = ["dense", "ssm", "rwkv", "moe", "encoder", "vision"]


@dataclass(frozen=True)
class Backbone:
    """One LM backbone for ValhallaBase experiments."""

    hf_id: str
    kind: Kind = "causal_lm"
    id: str = ""
    category: Category = "custom"
    load_in_4bit: bool = False
    trust_remote_code: bool = True
    lm_patch_rust: bool = False
    paper: bool = False

    def __post_init__(self) -> None:
        if not self.id:
            object.__setattr__(self, "id", self.hf_id.split("/")[-1].lower().replace(".", "_"))

    @staticmethod
    def from_hf(
        hf_id: str,
        *,
        kind: Kind | None = None,
        category: Category = "custom",
        load_in_4bit: bool = False,
        **kw: Any,
    ) -> Backbone:
        k = kind or _guess_kind(hf_id)
        q4 = load_in_4bit or "4bit" in hf_id.lower()
        return Backbone(hf_id=hf_id, kind=k, category=category, load_in_4bit=q4, **kw)

    def valhalla_arms(self) -> tuple[str, ...]:
        if self.kind == "causal_lm":
            return ("native", "hybrid", "trad")
        if self.kind == "encoder":
            return ("native", "encoder")
        return ("native", "vision")


def _guess_kind(hf_id: str) -> Kind:
    low = hf_id.lower()
    if any(x in low for x in ("embed", "modernbert", "nomic-embed", "jina-embed", "bge-")):
        return "encoder"
    if any(x in low for x in ("-vl-", "vision", "paligemma", "llava")):
        return "vision"
    return "causal_lm"


def _b(**kw: Any) -> Backbone:
    return Backbone(**kw)


# fmt: off — add new models here (one line each)
PRESETS: dict[str, Backbone] = {
    # dense
    "qwen3_4b": _b(hf_id="Qwen/Qwen3-4B-Instruct-2507", category="dense", paper=True),
    "qwen_3b": _b(id="qwen_3b", hf_id="Qwen/Qwen2.5-3B-Instruct", category="dense", lm_patch_rust=True),
    "gemma3_4b": _b(hf_id="google/gemma-3-4b-it", category="dense", paper=True),
    "llama3_2_3b": _b(hf_id="meta-llama/Llama-3.2-3B-Instruct", category="dense"),
    # ssm
    "mamba2_2.7b": _b(hf_id="state-spaces/mamba2-2.7b", category="ssm", paper=True),
    "falcon_mamba_7b_4bit": _b(hf_id="tiiuae/falcon-mamba-7b-instruct-4bit", category="ssm", load_in_4bit=True),
    # rwkv
    "rwkv7_0.1b": _b(hf_id="fla-hub/rwkv7-0.1B-g1", category="rwkv", paper=True),
    # moe
    "qwen3_30b_a3b": _b(hf_id="Qwen/Qwen3-30B-A3B-Instruct-2507", category="moe", paper=True),
    "phi_tiny_moe": _b(hf_id="microsoft/Phi-tiny-MoE-instruct", category="moe"),
    "deepseek_v2_lite": _b(hf_id="deepseek-ai/DeepSeek-V2-Lite-Chat", category="moe"),
    # encoder
    "modernbert": _b(hf_id="answerdotai/ModernBERT-base", kind="encoder", category="encoder", paper=True),
    "jina_v3": _b(hf_id="jinaai/jina-embeddings-v3", kind="encoder", category="encoder"),
    "nomic_embed": _b(hf_id="nomic-ai/nomic-embed-text-v1", kind="encoder", category="encoder"),
    # vision
    "qwen2_5_vl_3b": _b(hf_id="Qwen/Qwen2.5-VL-3B-Instruct", kind="vision", category="vision", paper=True),
}
# fmt: on

# Legacy aliases
BackboneSpec = Backbone
ALL_SPECS = list(PRESETS.values())


def get(name: str) -> Backbone | None:
    return PRESETS.get(name)


def list_presets(category: Category | None = None) -> list[Backbone]:
    if category is None:
        return list(PRESETS.values())
    return [b for b in PRESETS.values() if b.category == category]


def paper_panel() -> list[Backbone]:
    seen: set[Category] = set()
    out: list[Backbone] = []
    for cat in CATEGORY_ORDER:
        if cat in seen:
            continue
        picks = [b for b in PRESETS.values() if b.category == cat and b.paper]
        if picks:
            out.append(picks[0])
            seen.add(cat)
    return out


def practical_panel() -> list[Backbone]:
    """Six-class panel using locally typical / lighter presets (cached weights)."""
    keys: list[tuple[Category, str]] = [
        ("dense", "qwen_3b"),
        ("ssm", "mamba2_2.7b"),
        ("rwkv", "rwkv7_0.1b"),
        ("moe", "phi_tiny_moe"),
        ("encoder", "modernbert"),
        ("vision", "qwen2_5_vl_3b"),
    ]
    return [PRESETS[k] for cat, k in keys if PRESETS.get(k)]


def resolve(
    *,
    hf: list[str] | None = None,
    preset: list[str] | None = None,
    category: Category | None = None,
    paper: bool = False,
    practical: bool = False,
) -> list[Backbone]:
    """Merge hf / preset / category / paper / practical selections (dedupe by hf_id)."""
    out: list[Backbone] = []
    seen: set[str] = set()

    def add(b: Backbone) -> None:
        key = b.hf_id.lower()
        if key not in seen:
            seen.add(key)
            out.append(b)

    if practical:
        for b in practical_panel():
            add(b)
    elif paper:
        for b in paper_panel():
            add(b)
    if category:
        for b in list_presets(category):
            add(b)
    if preset:
        for name in preset:
            add(PRESETS.get(name) or Backbone.from_hf(name))
    if hf:
        for h in hf:
            add(Backbone.from_hf(h))
    return out


# scaling_models compat
def spec_by_id(model_id: str) -> Backbone | None:
    return get(model_id)


def resolve_panel(**kw: Any) -> list[Backbone]:
    preset = kw.pop("model_ids", None)
    return resolve(preset=preset, **kw)
