#!/usr/bin/env python3
"""Ablation arm runners — Valhalla / LM backends with progress · pause · error stop."""

from __future__ import annotations

import json
import time
from dataclasses import dataclass
from typing import Any, Callable

from benchmark_cli import ProgressBar, pause_requested  # noqa: E402
from benchmark_env import apply_resource_budget, production_valhalla_base_kwargs  # noqa: E402
from pareto_backends import (  # noqa: E402
    RunMetrics,
    run_full_context_lm,
    run_vanilla_rag,
)
from valhalla_base import ValhallaBase  # noqa: E402

ARM_ORDER = [
    "full_context_lm",
    "vanilla_rag",
    "lm_patch",
    "native",
    "hybrid_full",
    "hybrid_no_fate",
    "hybrid_tile",
]

LM_ARMS = frozenset({"full_context_lm", "vanilla_rag"})


@dataclass
class ArmSpec:
    name: str
    label: str
    kind: str  # lm | valhalla


ARM_SPECS: dict[str, ArmSpec] = {
    "full_context_lm": ArmSpec("full_context_lm", "LM full-context", "lm"),
    "vanilla_rag": ArmSpec("vanilla_rag", "Vanilla RAG", "lm"),
    "lm_patch": ArmSpec("lm_patch", "Valhalla lm_patch", "valhalla"),
    "native": ArmSpec("native", "Valhalla native", "valhalla"),
    "hybrid_full": ArmSpec("hybrid_full", "Valhalla hybrid (full)", "valhalla"),
    "hybrid_no_fate": ArmSpec("hybrid_no_fate", "Hybrid − Fate", "valhalla"),
    "hybrid_tile": ArmSpec("hybrid_tile", "Hybrid − Triad (tile)", "valhalla"),
}


class ArmPaused(Exception):
    """Raised when VALHALLA_PAUSE_FILE / --pause-file appears mid-arm."""

    def __init__(self, metrics: RunMetrics, completed: int) -> None:
        self.metrics = metrics
        self.completed = completed
        super().__init__(f"paused after {completed} items")


class ArmError(Exception):
    """Fatal per-item error (triggers suite stop-on-error)."""

    def __init__(self, msg: str, metrics: RunMetrics | None = None) -> None:
        self.metrics = metrics
        super().__init__(msg)


def valhalla_kwargs_for_arm(arm: str, *, backbone: str) -> dict[str, Any]:
    kw = production_valhalla_base_kwargs(lm_model=backbone, max_new=128)
    if arm == "lm_patch":
        kw["decode"] = "lm_patch"
    elif arm == "native":
        kw["decode"] = "native"
    elif arm == "hybrid_full":
        kw["decode"] = "hybrid"
        kw["open_fate"] = True
        kw["multihop_fate"] = True
        kw["multihop_v2"] = True
        kw["patch_body"] = "triad"
    elif arm == "hybrid_no_fate":
        kw["decode"] = "hybrid"
        kw["open_fate"] = False
        kw["multihop_fate"] = False
        kw["multihop_v2"] = True
        kw["patch_body"] = "triad"
    elif arm == "hybrid_tile":
        kw["decode"] = "hybrid"
        kw["open_fate"] = True
        kw["multihop_fate"] = True
        kw["multihop_v2"] = True
        kw["patch_body"] = "tile"
    else:
        raise KeyError(f"not a valhalla arm: {arm}")
    return kw


def run_valhalla_arm(
    arm: str,
    items: list[dict],
    *,
    backbone: str,
    score_fn: Callable[[str, dict], bool],
    label: str = "",
    verbose: bool = True,
    pause_file: str | None = None,
    stop_on_error: bool = True,
) -> RunMetrics:
    apply_resource_budget()
    m = RunMetrics()
    kw = valhalla_kwargs_for_arm(arm, backbone=backbone)
    base = ValhallaBase(**kw)
    bar = ProgressBar(label or arm, len(items), enabled=verbose)

    for i, item in enumerate(items):
        if pause_requested(pause_file):
            bar.finish("PAUSED")
            raise ArmPaused(m, i)
        corpus = [ln for ln in (item.get("corpus") or item.get("session_lines") or []) if ln.strip()]
        if not corpus and item.get("context"):
            from common import split_context_to_corpus  # noqa: WPS433

            corpus = split_context_to_corpus(str(item["context"]), max_lines=0)
        q = str(item.get("question") or item.get("prompt") or item.get("query") or "")
        ref = (
            item.get("reference_answer")
            or item.get("answer")
            or item.get("gold")
            or item.get("reference_output")
        )
        score_type = item.get("score_type") or ("mcq" if item.get("choice_A") else "open")

        t0 = time.monotonic()
        try:
            base.clear_context()
            base.set_context(corpus)
            wire_reload = len(json.dumps({"context": corpus}).encode("utf-8"))
            payload: dict[str, Any] = {
                "id": str(item.get("id", "q")),
                "text": q,
                "score_type": score_type,
                "reference_answer": str(ref) if ref is not None else "",
                "keywords": item.get("keywords") or [],
            }
            doc = base.run([payload])
            ans = str((doc.get("rows") or [{}])[0].get("answer") or "")
            dt = (time.monotonic() - t0) * 1000.0
            ok = score_fn(ans, item)
            m.n += 1
            m.correct += int(ok)
            tel = doc.get("telemetry") or {}
            m.tokens_prompt_cum += int(tel.get("lm_prefill_tokens") or len(q.split()) * 4)
            m.tokens_decode_cum += int(tel.get("lm_decode_tokens") or 64)
            m.latency_ms_cum += dt
            m.session_reload_bytes_cum += wire_reload
            m.rows.append(
                {
                    "id": item.get("id"),
                    "correct": ok,
                    "decode_path": (doc.get("rows") or [{}])[0].get("decode_path"),
                }
            )
            if arm == "hybrid_full":
                try:
                    pdoc = base.proactive_idle()
                    if (pdoc.get("proactive") or {}).get("emitted"):
                        m.proactive_emitted += 1
                except Exception:
                    pass
        except ArmPaused:
            raise
        except Exception as exc:  # noqa: BLE001
            if stop_on_error:
                bar.finish("ERROR")
                raise ArmError(f"{arm} item={item.get('id')}: {exc}", m) from exc
            m.n += 1
            m.rows.append({"id": item.get("id"), "correct": False, "error": str(exc)[:200]})
        bar.inc()
        bar.tick_item()
    bar.finish()
    return m


def run_lm_arm(
    arm: str,
    items: list[dict],
    *,
    backbone: str,
    score_fn: Callable[[str, dict], bool],
    label: str = "",
    verbose: bool = True,
    pause_file: str | None = None,
    stop_on_error: bool = True,
) -> RunMetrics:
    """LM arms with pause checks between items (load once via backend)."""
    # Full backends don't expose per-item pause; wrap with chunked runs if pause mid-way.
    if pause_requested(pause_file):
        raise ArmPaused(RunMetrics(), 0)

    bar = ProgressBar(label or arm, len(items), enabled=verbose)
    # Run in small chunks so pause/error can interrupt.
    chunk = 5
    m = RunMetrics()
    for start in range(0, len(items), chunk):
        if pause_requested(pause_file):
            bar.finish("PAUSED")
            raise ArmPaused(m, start)
        slice_items = items[start : start + chunk]
        try:
            fn = run_full_context_lm if arm == "full_context_lm" else run_vanilla_rag
            part = fn(slice_items, backbone=backbone, score_fn=score_fn)
        except Exception as exc:  # noqa: BLE001
            if stop_on_error:
                bar.finish("ERROR")
                raise ArmError(f"{arm} chunk@{start}: {exc}", m) from exc
            part = RunMetrics()
            part.skipped = True
            part.skip_reason = str(exc)[:200]
        if part.skipped and stop_on_error:
            bar.finish("ERROR")
            raise ArmError(f"{arm}: {part.skip_reason}", m)
        m.n += part.n
        m.correct += part.correct
        m.tokens_prompt_cum += part.tokens_prompt_cum
        m.tokens_decode_cum += part.tokens_decode_cum
        m.latency_ms_cum += part.latency_ms_cum
        m.session_reload_bytes_cum += part.session_reload_bytes_cum
        m.rows.extend(part.rows)
        for _ in slice_items:
            bar.inc()
            bar.tick_item()
    bar.finish()
    return m


def run_arm(
    arm: str,
    items: list[dict],
    *,
    backbone: str,
    score_fn: Callable[[str, dict], bool],
    label: str = "",
    verbose: bool = True,
    pause_file: str | None = None,
    stop_on_error: bool = True,
) -> RunMetrics:
    if arm in LM_ARMS:
        return run_lm_arm(
            arm,
            items,
            backbone=backbone,
            score_fn=score_fn,
            label=label,
            verbose=verbose,
            pause_file=pause_file,
            stop_on_error=stop_on_error,
        )
    if arm not in ARM_SPECS:
        m = RunMetrics()
        m.skipped = True
        m.skip_reason = f"unknown arm: {arm}"
        return m
    return run_valhalla_arm(
        arm,
        items,
        backbone=backbone,
        score_fn=score_fn,
        label=label,
        verbose=verbose,
        pause_file=pause_file,
        stop_on_error=stop_on_error,
    )
