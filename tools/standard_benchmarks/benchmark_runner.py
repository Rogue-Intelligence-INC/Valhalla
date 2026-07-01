#!/usr/bin/env python3
"""Shared ValhallaBase runner for third-party benchmark scripts."""

from __future__ import annotations

import sys
from pathlib import Path
from typing import Any

REPO = Path(__file__).resolve().parents[2]
INF = REPO / "tools/valhalla_inference"
if str(INF) not in sys.path:
    sys.path.insert(0, str(INF))

from valhalla_base import ValhallaBase  # noqa: E402


def make_base(*, fast: bool = False, lm_model: str = "Qwen/Qwen2.5-0.5B-Instruct") -> ValhallaBase:
    return ValhallaBase(
        lm_model=lm_model,
        decode="hybrid",
        mcq_decode="lm_patch",
        follow_up_decode="native_follow_up_aware",
        fate_ingress_routing="quad_cycle",
        fate_qa_feedback=False,
        incubation_cycles=1 if fast else 2,
        max_new=128,
    )


def run_open_item(
    base: ValhallaBase,
    *,
    item_id: str,
    question: str,
    corpus: list[str],
    append_only: bool = False,
) -> str:
    if append_only:
        for line in corpus:
            base.append_context(line)
    else:
        base.set_context(corpus)
    if not base.context_items and not base.session.corpus:
        return ""
    out = base.run([{"id": item_id, "text": question, "score_type": "open"}])
    return str((out.get("rows") or [{}])[0].get("answer") or "")


def run_open_session_append(
    base: ValhallaBase,
    *,
    item_id: str,
    question: str,
    lines: list[str],
) -> str:
    """Build corpus via session append (LoCoMo / long dialogue)."""
    if not lines:
        base.set_context([])
    else:
        base.set_context([lines[0]])
        for line in lines[1:]:
            base.append_context(line)
    out = base.run([{"id": item_id, "text": question, "score_type": "open"}])
    return str((out.get("rows") or [{}])[0].get("answer") or "")
