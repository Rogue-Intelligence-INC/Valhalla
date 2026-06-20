#!/usr/bin/env python3
"""Tier A patch track — fresh reload per eval question (fair protocol)."""

from __future__ import annotations

import gc
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
BRIDGE = REPO / "tools/valhalla_model_bridge"
TF_FRONT = REPO / "tools/tier_a_tf_front"

sys.path.insert(0, str(BRIDGE))
sys.path.insert(0, str(TF_FRONT))

from metrics import score_answer, summarize_rows  # noqa: E402
from pipeline import apply_cumulative_patch, valhalla_export  # noqa: E402
from run_incubation_benchmark import DEFAULT_MODEL, DEFAULT_STRENGTH, generate_full, load_model  # noqa: E402
from run_strict_incubation_experiment import strict_user_text  # noqa: E402


def eval_patch_arm_fresh(
    arm_id: str,
    body: str,
    corpus: list[str],
    prompts: list[dict],
    *,
    model_name: str = DEFAULT_MODEL,
    strength: float = DEFAULT_STRENGTH,
    patch_mode: str = "legacy",
    max_new: int = 192,
    cycles: int = 1,
    export_doc: dict | None = None,
) -> dict:
    """
    Fair Tier A: export patch from ingest corpus only; fresh HF load per question.
    """
    if export_doc is None:
        patch_doc = valhalla_export(body, cycles, corpus)
    else:
        patch_doc = export_doc

    rows = []
    for p in prompts:
        model, tokenizer = load_model(model_name)
        apply_cumulative_patch(
            model,
            patch_doc,
            strength,
            tokenizer,
            body=body,
            corpus_lines=corpus,
            patch_mode=patch_mode,
        )
        user_text = strict_user_text(p)
        gen = generate_full(model, tokenizer, user_text, max_new)
        sc = score_answer(gen["assistant"], p, strict_mcq=True)
        rows.append(
            {
                "id": p["id"],
                "domain": p["domain"],
                "score_type": p.get("score_type"),
                "correct": sc["correct"],
                "score": sc,
                "assistant": gen["assistant"],
                "decode_path": f"patch_{patch_mode}_{body}",
            }
        )
        del model, tokenizer
        gc.collect()
        try:
            import torch

            if torch.cuda.is_available():
                torch.cuda.empty_cache()
        except ImportError:
            pass

    return {
        "arm": arm_id,
        "track": "patch_tier_a",
        "body": body,
        "patch_mode": patch_mode,
        "strength": strength,
        "fresh_reload_per_question": True,
        "summary": summarize_rows(rows),
        "rows": rows,
    }


def eval_transformer_fresh(
    prompts: list[dict],
    *,
    model_name: str = DEFAULT_MODEL,
    max_new: int = 192,
    arm_id: str = "transformer_zero_shot",
) -> dict:
    """Unpatched baseline — fresh load per question for parity with patch track."""
    rows = []
    for p in prompts:
        model, tokenizer = load_model(model_name)
        user_text = strict_user_text(p)
        gen = generate_full(model, tokenizer, user_text, max_new)
        sc = score_answer(gen["assistant"], p, strict_mcq=True)
        rows.append(
            {
                "id": p["id"],
                "domain": p["domain"],
                "score_type": p.get("score_type"),
                "correct": sc["correct"],
                "score": sc,
                "assistant": gen["assistant"],
                "decode_path": "transformer_generate",
            }
        )
        del model, tokenizer
        gc.collect()
        try:
            import torch

            if torch.cuda.is_available():
                torch.cuda.empty_cache()
        except ImportError:
            pass

    return {
        "arm": arm_id,
        "track": "generate_baseline",
        "model": model_name,
        "fresh_reload_per_question": True,
        "summary": summarize_rows(rows),
        "rows": rows,
    }
