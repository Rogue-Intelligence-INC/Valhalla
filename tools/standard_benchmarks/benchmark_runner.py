#!/usr/bin/env python3
"""Shared ValhallaBase runner — production paths delegate to Rust `valhalla_base` binary.

Legacy Python ingest remains for `--legacy` / non-production only.
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path
from typing import Any

REPO = Path(__file__).resolve().parents[2]
INF = REPO / "tools/valhalla_inference"
VB_BIN = REPO / "target/release/valhalla_base"

if str(INF) not in sys.path:
    sys.path.insert(0, str(INF))

from valhalla_base import ValhallaBase  # noqa: E402


def _ensure_rust_bin() -> Path:
    if not VB_BIN.is_file():
        subprocess.run(
            ["cargo", "build", "-p", "hub-f64", "--release", "--bin", "valhalla_base"],
            check=True,
            cwd=REPO,
        )
    return VB_BIN


def _run_rust_demo(name: str, *extra: str) -> dict[str, Any]:
    bin_path = _ensure_rust_bin()
    env = {**os.environ, "RUSTFLAGS": os.environ.get("RUSTFLAGS", "-L /opt/cuda/lib64")}
    if name in ("locomo", "longchain-fate", "multihop-hybrid"):
        env.setdefault("VALHALLA_FATE_PARALLEL_LCHAIN", "1")
    proc = subprocess.run(
        [str(bin_path), "demo", name, *extra],
        capture_output=True,
        text=True,
        cwd=REPO,
        env=env,
    )
    if proc.stderr:
        print(proc.stderr, end="", flush=True)
    raw = proc.stdout.strip()
    if not raw:
        raise RuntimeError(proc.stderr or f"valhalla_base demo {name} empty stdout")
    return json.loads(raw)


def rust_demo_flags(
    *,
    fast: bool = False,
    limit: int = 0,
    verbose: bool = False,
    checkpoint: Path | str | None = None,
    checkpoint_every: int = 1,
    resume: Path | str | None = None,
    pause_file: Path | str | None = None,
) -> list[str]:
    extra: list[str] = []
    if fast:
        extra.append("--fast")
    if limit > 0:
        extra.extend(["--limit", str(limit)])
    if verbose:
        extra.append("--verbose")
    if checkpoint:
        extra.extend(["--checkpoint", str(checkpoint), "--checkpoint-every", str(checkpoint_every)])
    if resume:
        extra.extend(["--resume", str(resume)])
    if pause_file:
        extra.extend(["--pause-file", str(pause_file)])
    return extra


def make_base(
    *,
    fast: bool = False,
    lm_model: str | None = None,
    production: bool = False,
    legacy: bool = False,
) -> ValhallaBase:
    """Production stack: use Rust ValhallaBaseClient via binary (not Python subprocess loop)."""
    if production and not legacy:
        raise RuntimeError(
            "production=True: use Rust API — "
            "valhalla_base demo locomo|needle-prod or hub_f64::ValhallaBaseClient"
        )
    if production:
        from benchmark_env import default_lm_model, production_valhalla_base_kwargs  # noqa: WPS433

        kw: dict[str, Any] = production_valhalla_base_kwargs(
            incubation_cycles=1 if fast else 2,
            max_new=128,
        )
        if lm_model:
            kw["lm_model"] = lm_model
        else:
            kw.setdefault("lm_model", default_lm_model())
        return ValhallaBase(**kw)
    model = lm_model or "Qwen/Qwen2.5-0.5B-Instruct"
    return ValhallaBase(
        lm_model=model,
        decode="hybrid",
        mcq_decode="lm_patch",
        follow_up_decode="native_follow_up_aware",
        fate_ingress_routing="quad_cycle",
        fate_qa_feedback=False,
        incubation_cycles=1 if fast else 2,
        max_new=128,
    )


def run_open_session_append(
    base: ValhallaBase,
    *,
    item_id: str,
    question: str,
    lines: list[str],
    production: bool = False,
    chunk_size: int = 0,
    fast: bool = False,
) -> str:
    if production:
        raise RuntimeError("use: valhalla_base demo locomo (Rust long-chain path)")
    base.set_context([lines[0]] if lines else [])
    for line in lines[1:]:
        base.append_context(line)
    out = base.run([{"id": item_id, "text": question, "score_type": "open"}])
    return str((out.get("rows") or [{}])[0].get("answer") or "")


def run_longbench_rust(
    *,
    fast: bool = False,
    limit: int = 0,
    max_corpus_lines: int = 0,
    low_resource: bool = True,
    mcq_light: bool = True,
) -> dict[str, Any]:
    """Rust LongBench with fragmentwithfate prelearn (not legacy 4-line warmup)."""
    extra = rust_demo_flags(fast=fast, limit=limit)
    extra.extend(["--max-corpus-lines", str(max_corpus_lines)])
    if low_resource:
        extra.append("--low-resource")
    if not mcq_light:
        extra.append("--no-mcq-light")
    return _run_rust_demo("longbench", *extra)


def run_lara_fatewithv2_rust(
    *,
    fast: bool = False,
    limit: int = 0,
) -> dict[str, Any]:
    """LaRA RAG arm via fatewithv2 compose (multihop_v2 + multihop_fate)."""
    extra = rust_demo_flags(fast=fast, limit=limit)
    return _run_rust_demo("lara-fatewithv2", *extra)


def run_locomo_rust(
    *,
    fast: bool = False,
    limit: int = 0,
    max_lines: int = 128,
    chunk_size: int = 0,
    verbose: bool = False,
    checkpoint: Path | str | None = None,
    checkpoint_every: int = 1,
    resume: Path | str | None = None,
    pause_file: Path | str | None = None,
) -> dict[str, Any]:
    """Rust-only LoCoMo eval with dynamic density fragments."""
    extra = rust_demo_flags(
        fast=fast,
        limit=limit,
        verbose=verbose,
        checkpoint=checkpoint,
        checkpoint_every=checkpoint_every,
        resume=resume,
        pause_file=pause_file,
    )
    extra.extend(["--max-lines", str(max_lines), "--chunk-size", str(chunk_size)])
    return _run_rust_demo("locomo", *extra)


def run_frames_rust(
    *,
    fast: bool = False,
    limit: int = 0,
    max_corpus_lines: int = 0,
    offline_wiki: bool = False,
    verbose: bool = False,
    checkpoint: Path | str | None = None,
    checkpoint_every: int = 1,
    resume: Path | str | None = None,
    pause_file: Path | str | None = None,
) -> dict[str, Any]:
    """Rust-only FRAMES eval — Wikipedia fetch + dynamic fragments."""
    extra = rust_demo_flags(
        fast=fast,
        limit=limit,
        verbose=verbose,
        checkpoint=checkpoint,
        checkpoint_every=checkpoint_every,
        resume=resume,
        pause_file=pause_file,
    )
    if offline_wiki:
        extra.append("--offline-wiki")
    extra.extend(["--max-corpus-lines", str(max_corpus_lines)])
    return _run_rust_demo("frames", *extra)


def run_open_item(
    base: ValhallaBase,
    *,
    item_id: str,
    question: str,
    corpus: list[str],
    append_only: bool = False,
    production: bool = False,
) -> str:
    if production:
        raise RuntimeError("production open eval: use hub_f64::ValhallaBaseClient in Rust")
    if append_only:
        for line in corpus:
            base.append_context(line)
    else:
        base.set_context(corpus)
    out = base.run([{"id": item_id, "text": question, "score_type": "open"}])
    return str((out.get("rows") or [{}])[0].get("answer") or "")


def run_mcq_item(
    base: ValhallaBase,
    *,
    item_id: str,
    prompt: str,
    corpus: list[str],
    reference_answer: str,
    production: bool = False,
    ingest_mode: str = "full",
) -> dict[str, Any]:
    if production:
        raise RuntimeError("production MCQ: use hub_f64::run_mcq_batch_item or ValhallaBaseClient::run_batch")
    q = {
        "id": item_id,
        "text": prompt,
        "score_type": "mcq",
        "reference_answer": reference_answer,
    }
    base.set_context(corpus)
    out = base.run([q])
    row = (out.get("rows") or [{}])[0]
    return {"out": out, "row": row, "answer": str(row.get("answer") or "")}
