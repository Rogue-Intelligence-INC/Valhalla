#!/usr/bin/env python3
"""
Hybrid Valhalla — unified product: all question types routed through one policy.

Tier B memory (best body per type) + Valhalla-patched LM, merged per prompt.

Product version: hybrid-2.0 (unified router, Tile+Fate MCQ, dynamic Fate cycles).
Fair benchmark suite: fair-1.2 · hub-f64: 1.1.0
"""

HYBRID_PRODUCT_VERSION = "hybrid-2.0"
FAIR_BENCHMARK_VERSION = "fair-1.2"

from __future__ import annotations

from typing import Any

from metrics import summarize_rows
from run_potential_analysis import classify_slices

# Memory source keys used by the unified runner.
MEMORY_TRIAD = "triad_persistent"
MEMORY_TILE_FATE = "tile_fate_persistent"


def hybrid_route(prompt: dict, slices: dict[str, set[str]]) -> str:
    """Return routing policy id for one prompt (all types go through Hybrid)."""
    st = prompt.get("score_type", "open")
    pid = prompt["id"]
    og = pid in slices["open_generation"]

    if st == "open" or og:
        return "patch_lm"
    if st == "mcq":
        return "memory_mcq_best"
    if st == "numeric":
        return "hybrid_max_all"
    if pid in slices["corpus_supported"]:
        return "memory_triad"
    return "memory_triad"


def patch_prompt_ids(prompts: list[dict], slices: dict[str, set[str]]) -> set[str]:
    """Prompts that need Valhalla-patched LM eval."""
    return {p["id"] for p in prompts if hybrid_route(p, slices) in ("patch_lm", "hybrid_max_all")}


def _row_map(rows: list[dict]) -> dict[str, dict]:
    return {r["id"]: r for r in rows}


def _pick_best_row(
    candidates: list[tuple[str, dict | None]],
    *,
    prefer: list[str] | None = None,
) -> tuple[str, dict, bool]:
    """Pick first correct by prefer order, else first available row."""
    prefer = prefer or [k for k, _ in candidates]
    by_key = {k: r for k, r in candidates if r is not None}
    for key in prefer:
        row = by_key.get(key)
        if row and row.get("correct"):
            return key, row, True
    for key in prefer:
        row = by_key.get(key)
        if row:
            return key, row, False
    for key, row in candidates:
        if row:
            return key, row, False
    key, row = candidates[0]
    return key, row or {}, False


def build_hybrid_rows(
    prompts: list[dict],
    memory_sources: dict[str, list[dict]],
    patch_rows: list[dict],
    corpus: list[str],
) -> tuple[list[dict], dict[str, set[str]]]:
    """
    Unified Hybrid product rows — every prompt resolved through hybrid_route().

    memory_sources: e.g. triad_persistent, tile_fate_persistent
    """
    slices = classify_slices(prompts, corpus)
    mem_maps = {k: _row_map(v) for k, v in memory_sources.items()}
    patch = _row_map(patch_rows)
    out: list[dict] = []

    triad = mem_maps.get(MEMORY_TRIAD, {})
    tile_fate = mem_maps.get(MEMORY_TILE_FATE, {})

    for p in prompts:
        pid = p["id"]
        choice = hybrid_route(p, slices)
        triad_row = triad.get(pid)
        tile_row = tile_fate.get(pid)
        prow = patch.get(pid)

        if choice == "patch_lm":
            correct = prow["correct"] if prow else False
            src = "patch_lm" if prow else "none"
            base = prow or triad_row or tile_row or {}
        elif choice == "memory_mcq_best":
            src_key, base, correct = _pick_best_row(
                [
                    (MEMORY_TILE_FATE, tile_row),
                    (MEMORY_TRIAD, triad_row),
                ],
                prefer=[MEMORY_TILE_FATE, MEMORY_TRIAD],
            )
            src = src_key
        elif choice == "hybrid_max_all":
            pool: list[tuple[str, dict | None]] = [
                (MEMORY_TILE_FATE, tile_row),
                (MEMORY_TRIAD, triad_row),
                ("patch_lm", prow),
            ]
            src_key, base, correct = _pick_best_row(
                pool,
                prefer=[MEMORY_TILE_FATE, MEMORY_TRIAD, "patch_lm"],
            )
            src = src_key
        else:
            base = triad_row or tile_row or {}
            correct = bool(base.get("correct"))
            src = MEMORY_TRIAD if triad_row else MEMORY_TILE_FATE

        out.append(
            {
                **{k: v for k, v in (base or {}).items() if k not in ("correct",)},
                "score_type": p.get("score_type"),
                "correct": correct,
                "hybrid_source": src,
                "hybrid_policy": choice,
                "answer": (base or {}).get("assistant") or (base or {}).get("native_answer"),
            }
        )

    return out, slices


def hybrid_summary(
    hybrid_rows: list[dict],
    memory_sources: dict[str, list[dict]],
    patch_rows: list[dict],
    slices: dict[str, set[str]],
) -> dict[str, Any]:
    patch_by = _row_map(patch_rows)
    triad_rows = memory_sources.get(MEMORY_TRIAD, [])
    tile_rows = memory_sources.get(MEMORY_TILE_FATE, [])

    open_ids = {p["id"] for p in hybrid_rows if p.get("score_type") == "open"}
    mcq_ids = {p["id"] for p in hybrid_rows if p.get("score_type") == "mcq"}
    num_ids = {p["id"] for p in hybrid_rows if p.get("score_type") == "numeric"}

    triad_by = _row_map(triad_rows)
    tile_by = _row_map(tile_rows)

    by_src: dict[str, int] = {}
    for r in hybrid_rows:
        s = r.get("hybrid_source", "?")
        by_src[s] = by_src.get(s, 0) + 1

    def subset_rows(ids: set[str], src_rows: list[dict]) -> list[dict]:
        m = _row_map(src_rows)
        return [m[i] for i in ids if i in m]

    return {
        "hybrid": summarize_rows(hybrid_rows),
        "memory_triad_persistent": summarize_rows(triad_rows),
        "memory_tile_fate_persistent": summarize_rows(tile_rows),
        "patch_lm_subset": summarize_rows(patch_rows),
        "open_only": {
            "triad_memory": summarize_rows(subset_rows(open_ids, triad_rows)),
            "patch_lm": summarize_rows([patch_by[i] for i in open_ids if i in patch_by]),
            "hybrid": summarize_rows([r for r in hybrid_rows if r["id"] in open_ids]),
        },
        "mcq_only": {
            "triad_memory": summarize_rows(subset_rows(mcq_ids, triad_rows)),
            "tile_fate_memory": summarize_rows(subset_rows(mcq_ids, tile_rows)),
            "hybrid": summarize_rows([r for r in hybrid_rows if r["id"] in mcq_ids]),
        },
        "numeric_only": {
            "triad_memory": summarize_rows(subset_rows(num_ids, triad_rows)),
            "tile_fate_memory": summarize_rows(subset_rows(num_ids, tile_rows)),
            "patch_lm": summarize_rows([patch_by[i] for i in num_ids if i in patch_by]),
            "hybrid": summarize_rows([r for r in hybrid_rows if r["id"] in num_ids]),
        },
        "open_retrieval_slice": summarize_rows(
            [r for r in hybrid_rows if r["id"] in slices["open_retrieval"]]
        ),
        "open_generation_slice": summarize_rows(
            [r for r in hybrid_rows if r["id"] in slices["open_generation"]]
        ),
        "hybrid_source_counts": by_src,
        "slices": {k: len(v) for k, v in slices.items()},
        # Legacy aliases
        "memory_persistent": summarize_rows(triad_rows),
    }


HYBRID_POLICY_DOC = """
Unified Hybrid Valhalla — all 145 questions through one product router:

| Type | Route |
|------|-------|
| open / open_generation | Valhalla patch + Qwen generate |
| mcq | best(Tile+Fate memory, Triad memory) |
| numeric | max(Tile+Fate, Triad, patch LM) |
| other | Triad memory |

Memory protocol: persistent batch + whole-question grinder ingress.
MCQ memory: Tile body + VALHALLA_MCQ_FATE_CYCLES=1 (decompose/aggregate).
""".strip()
