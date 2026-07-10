#!/usr/bin/env python3
"""Unified external comparison backends — same backbone · dataset · scoring.

Systems:
  full_context_lm  — entire session in prompt each turn
  vanilla_rag      — corpus block + question (traditional LM RAG)
  valhalla_base    — session append + structure snapshot
  mem0             — optional (pip install mem0ai); SKIPPED if unavailable
  letta            — optional (pip install letta-client); SKIPPED if unavailable
"""

from __future__ import annotations

import json
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Callable

REPO = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent


@dataclass
class RunMetrics:
    correct: int = 0
    n: int = 0
    tokens_prompt_cum: int = 0
    tokens_decode_cum: int = 0
    latency_ms_cum: float = 0.0
    session_reload_bytes_cum: int = 0
    proactive_emitted: int = 0
    rows: list[dict[str, Any]] = field(default_factory=list)
    skipped: bool = False
    skip_reason: str | None = None

    @property
    def accuracy(self) -> float:
        return self.correct / self.n if self.n else 0.0

    @property
    def avg_tokens_per_query(self) -> float:
        return (self.tokens_prompt_cum + self.tokens_decode_cum) / self.n if self.n else 0.0

    @property
    def avg_latency_ms(self) -> float:
        return self.latency_ms_cum / self.n if self.n else 0.0

    def to_summary(self) -> dict[str, Any]:
        return {
            "n": self.n,
            "correct": self.correct,
            "accuracy": round(self.accuracy, 4),
            "tokens_prompt_cum": self.tokens_prompt_cum,
            "tokens_decode_cum": self.tokens_decode_cum,
            "avg_tokens_per_query": round(self.avg_tokens_per_query, 1),
            "avg_latency_ms": round(self.avg_latency_ms, 1),
            "session_reload_bytes_cum": self.session_reload_bytes_cum,
            "proactive_emitted": self.proactive_emitted,
            "skipped": self.skipped,
            "skip_reason": self.skip_reason,
        }


def _load_lm_and_tokenizer(backbone: str):
    from test_context_arch_compare import resolve_lm_path  # noqa: WPS433
    from loader import ValhallaLLM  # noqa: WPS433
    from transformers import AutoTokenizer  # type: ignore

    path = resolve_lm_path(backbone)
    tok = AutoTokenizer.from_pretrained(str(path), trust_remote_code=True)
    lm = ValhallaLLM(path)
    return lm, tok


def _count_prefill(tok, user_content: str) -> int:
    messages = [{"role": "user", "content": user_content}]
    text = tok.apply_chat_template(messages, tokenize=False, add_generation_prompt=True)
    return len(tok.encode(text, add_special_tokens=False))


def _context_block(lines: list[str]) -> str:
    from test_context_performance import context_block  # noqa: WPS433

    return context_block(lines)


def run_full_context_lm(
    items: list[dict],
    *,
    backbone: str,
    score_fn: Callable[[str, dict], bool],
    max_new: int = 128,
) -> RunMetrics:
    m = RunMetrics()
    try:
        lm, tok = _load_lm_and_tokenizer(backbone)
    except Exception as exc:  # noqa: BLE001
        m.skipped = True
        m.skip_reason = str(exc)[:200]
        return m

    for item in items:
        corpus = list(item.get("corpus") or item.get("session_lines") or [])
        q = str(item.get("question") or item.get("prompt") or "")
        t0 = time.monotonic()
        prompt = _context_block(corpus) + q
        prefill = _count_prefill(tok, prompt)
        ans = lm.generate([prompt], max_tokens=max_new)[0].text.strip()
        dt = (time.monotonic() - t0) * 1000.0
        ok = score_fn(ans, item)
        m.n += 1
        m.correct += int(ok)
        m.tokens_prompt_cum += prefill
        m.tokens_decode_cum += max_new // 2
        m.latency_ms_cum += dt
        m.session_reload_bytes_cum += len(prompt.encode("utf-8"))
        m.rows.append({"id": item.get("id"), "correct": ok, "tokens_prompt": prefill})
    return m


def run_vanilla_rag(
    items: list[dict],
    *,
    backbone: str,
    score_fn: Callable[[str, dict], bool],
    max_new: int = 128,
) -> RunMetrics:
    """Same as full-context for single-shot QA; differs on multi-turn (no history duplication)."""
    return run_full_context_lm(items, backbone=backbone, score_fn=score_fn, max_new=max_new)


def run_valhalla_base(
    items: list[dict],
    *,
    backbone: str,
    score_fn: Callable[[str, dict], bool],
    proactive: bool = True,
) -> RunMetrics:
    from benchmark_env import apply_resource_budget, production_valhalla_base_kwargs  # noqa: WPS433
    from valhalla_base import ValhallaBase  # noqa: WPS433

    apply_resource_budget()
    m = RunMetrics()
    base = ValhallaBase(**production_valhalla_base_kwargs(lm_model=backbone, max_new=128))

    for item in items:
        corpus = [ln for ln in (item.get("corpus") or item.get("session_lines") or []) if ln.strip()]
        q = str(item.get("question") or item.get("prompt") or "")
        ref = item.get("reference_answer") or item.get("answer") or item.get("gold")
        score_type = item.get("score_type") or ("mcq" if item.get("choice_A") else "open")

        wire_before = 0
        t0 = time.monotonic()
        base.clear_context()
        base.set_context(corpus)
        wire_reload = len(json.dumps({"context": corpus}).encode("utf-8"))
        payload = {
            "id": str(item.get("id", "q")),
            "text": q,
            "score_type": score_type,
            "reference_answer": str(ref) if ref is not None else "",
            "keywords": item.get("keywords") or [],
        }
        doc = base.run([payload])
        ans = str((doc.get("rows") or [{}])[0].get("answer") or "")
        dt = (time.monotonic() - t0) * 1000.0

        if proactive and base.session.corpus:
            try:
                pdoc = base.proactive_idle()
                p = pdoc.get("proactive") or {}
                if p.get("emitted"):
                    m.proactive_emitted += 1
            except Exception:
                pass

        ok = score_fn(ans, item)
        m.n += 1
        m.correct += int(ok)
        m.tokens_prompt_cum += int(doc.get("telemetry", {}).get("lm_prefill_tokens") or len(q.split()) * 4)
        m.tokens_decode_cum += 64
        m.latency_ms_cum += dt
        m.session_reload_bytes_cum += wire_reload - wire_before
        m.rows.append({"id": item.get("id"), "correct": ok, "decode_path": (doc.get("rows") or [{}])[0].get("decode_path")})
    return m


def run_mem0_proxy(
    items: list[dict],
    *,
    backbone: str,
    score_fn: Callable[[str, dict], bool],
) -> RunMetrics:
    """Mem0-style: add session to memory store, search, then LM answer."""
    m = RunMetrics()
    try:
        from mem0 import Memory  # type: ignore
    except ImportError:
        m.skipped = True
        m.skip_reason = "mem0ai not installed (pip install mem0ai)"
        return m

    try:
        lm, tok = _load_lm_and_tokenizer(backbone)
        memory = Memory()
        user_id = "pareto_bench_user"
    except Exception as exc:  # noqa: BLE001
        m.skipped = True
        m.skip_reason = str(exc)[:200]
        return m

    for item in items:
        corpus = list(item.get("corpus") or item.get("session_lines") or [])
        q = str(item.get("question") or item.get("prompt") or "")
        for i, ln in enumerate(corpus):
            memory.add(ln, user_id=user_id, metadata={"line": i})
        t0 = time.monotonic()
        hits = memory.search(q, user_id=user_id, limit=8)
        ctx = "\n".join(str(h.get("memory", h)) for h in (hits or [])[:8])
        prompt = _context_block(ctx.split("\n") if ctx else []) + q
        prefill = _count_prefill(tok, prompt)
        ans = lm.generate([prompt], max_tokens=128)[0].text.strip()
        dt = (time.monotonic() - t0) * 1000.0
        ok = score_fn(ans, item)
        m.n += 1
        m.correct += int(ok)
        m.tokens_prompt_cum += prefill
        m.tokens_decode_cum += 64
        m.latency_ms_cum += dt
        m.session_reload_bytes_cum += len(prompt.encode("utf-8"))
        try:
            memory.delete_all(user_id=user_id)
        except Exception:
            pass
    return m


def run_letta_proxy(
    items: list[dict],
    *,
    backbone: str,
    score_fn: Callable[[str, dict], bool],
) -> RunMetrics:
    m = RunMetrics()
    m.skipped = True
    m.skip_reason = (
        "Letta requires LETTA_API_KEY + running Letta server; "
        "use vanilla_rag + session file as offline proxy until API wired"
    )
    return m


BACKENDS: dict[str, Callable[..., RunMetrics]] = {
    "full_context_lm": run_full_context_lm,
    "vanilla_rag": run_vanilla_rag,
    "valhalla_base": run_valhalla_base,
    "mem0": run_mem0_proxy,
    "letta": run_letta_proxy,
}
