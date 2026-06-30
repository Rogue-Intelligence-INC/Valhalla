#!/usr/bin/env python3
"""MCQ hybrid lm_patch ladder — Qwen 0.5B / 3B / 7B (Rust Candle, sharded OK).

Protocol: mcq-ladder-v1

Runs ValhallaBase hybrid + lm_patch logprob on fair-1.2 MCQ slice for each
Qwen2 model in the scaling ladder. Produces log-linear MCQ scaling curve.

Usage:
  python3 tools/valhalla_inference/test_mcq_ladder_v1.py
  python3 tools/valhalla_inference/test_mcq_ladder_v1.py --limit 10 --fast
  python3 tools/valhalla_inference/test_mcq_ladder_v1.py --model qwen_3b
"""

from __future__ import annotations

import argparse
import json
import math
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

REPO = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(REPO / "tools/fair_benchmark"))
sys.path.insert(0, str(REPO / "tools/standard_benchmarks"))

from run_fair_benchmark import load_suite, select_prompts  # noqa: E402
from run_scale_benchmark import corpus_for_prompt  # noqa: E402
from run_strict_incubation_experiment import strict_user_text  # noqa: E402
from scaling_analysis import linear_fit, log10_params  # noqa: E402
from scaling_models import QWEN_LM_PATCH_LADDER, ScalingModel, ladder_by_id  # noqa: E402
from test_context_arch_compare import resolve_lm_path  # noqa: E402
from test_model_scaling_v1 import ctx_for_prompt, production_base  # noqa: E402
from test_context_performance import context_block  # noqa: E402
from metrics import score_answer  # noqa: E402

PROTOCOL = "mcq-ladder-v1"
REPORTS = REPO / "reports/valhalla_inference"


def run_valhalla_mcq_batch(
    corpus: list[str], prompts: list[dict], *, lm_model: str, ctx_mode: str
) -> dict[str, Any]:
    """One ValhallaBase + one LM load per model (important for 3B/7B)."""
    rows: list[dict] = []
    correct = 0
    base = production_base(lm_model=lm_model)
    for p in prompts:
        ctx = ctx_for_prompt(p, corpus, mode=ctx_mode)
        base.set_context(ctx)
        q = {
            "id": p["id"],
            "text": strict_user_text(p),
            "score_type": "mcq",
            "reference_answer": p.get("reference_answer", ""),
            "keywords": p.get("keywords") or [],
        }
        out = base.run([q])
        row = (out.get("rows") or [{}])[0]
        ans = str(row.get("answer") or "")
        ok = bool(score_answer(ans, p)["correct"])
        correct += int(ok)
        rows.append({"id": p["id"], "correct": ok, "answer": ans[:120]})
    n = len(prompts)
    return {
        "n": n,
        "correct": correct,
        "accuracy": correct / max(n, 1),
        "rows": rows,
        "lm_model": lm_model,
    }


def run_trad_mcq(lm, corpus: list[str], prompts: list[dict], model: ScalingModel) -> dict[str, Any]:
    rows: list[dict] = []
    correct = 0
    for p in prompts:
        ctx = corpus_for_prompt(p, corpus)
        qtext = strict_user_text(p)
        ans = lm.generate([context_block(ctx) + qtext], max_tokens=128)[0].text.strip()
        ok = bool(score_answer(ans, p)["correct"])
        correct += int(ok)
        rows.append({"id": p["id"], "correct": ok, "answer": ans[:100]})
    n = len(prompts)
    return {
        "model_id": model.id,
        "hf_id": model.hf_id,
        "params_b": model.params_b,
        "arm": "trad_lm_mcq",
        "n": n,
        "correct": correct,
        "accuracy": correct / max(n, 1),
        "rows": rows,
    }


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--phase", default="test")
    ap.add_argument("--limit", type=int, default=0)
    ap.add_argument("--fast", action="store_true", help="limit=10")
    ap.add_argument("--with-trad", action="store_true", help="also run trad_lm per model")
    ap.add_argument("--model", action="append", dest="model_ids")
    ap.add_argument("--skip-7b", action="store_true", default=True)
    args = ap.parse_args()

    limit = 10 if args.fast else args.limit
    models = [m for m in QWEN_LM_PATCH_LADDER if not args.model_ids or m.id in args.model_ids]
    if args.skip_7b:
        models = [m for m in models if m.id != "qwen_7b"]

    suite = load_suite()
    corpus = suite["corpus"][suite["phases"][args.phase]["corpus"]]
    prompts = [p for p in select_prompts(suite, args.phase) if p.get("score_type") == "mcq"]
    if limit > 0:
        prompts = prompts[:limit]

    print(f"=== {PROTOCOL} n={len(prompts)} models={[m.id for m in models]} ===\n")

    arms: dict[str, Any] = {}
    xs: list[float] = []
    ys_hybrid: list[float] = []
    ys_trad: list[float] = []

    for model in models:
        print(f"[hybrid lm_patch] {model.hf_id} ({model.params_b}B) …")
        try:
            resolve_lm_path(model.hf_id)  # fail fast if weights missing
            arm = run_valhalla_mcq_batch(corpus, prompts, lm_model=model.hf_id, ctx_mode="corpus_for_prompt")
            arm["model_id"] = model.id
            arm["params_b"] = model.params_b
            arm["arm"] = "valhalla_hybrid_lm_patch"
            arms[f"hybrid/{model.id}"] = arm
            xs.append(log10_params(model.params_b))
            ys_hybrid.append(arm["accuracy"])
            print(f"    {arm['correct']}/{arm['n']} ({arm['accuracy']:.1%})\n")
        except Exception as exc:  # noqa: BLE001
            arms[f"hybrid/{model.id}"] = {"skipped": True, "error": str(exc), "model_id": model.id}
            print(f"    SKIP: {exc}\n")

        if args.with_trad:
            from loader import ValhallaLLM  # noqa: E402

            print(f"[trad mcq] {model.hf_id} …")
            try:
                lm = ValhallaLLM(resolve_lm_path(model.hf_id), engine="transformers")
                tarm = run_trad_mcq(lm, corpus, prompts, model)
                arms[f"trad/{model.id}"] = tarm
                ys_trad.append(tarm["accuracy"])
                print(f"    {tarm['correct']}/{tarm['n']} ({tarm['accuracy']:.1%})\n")
            except Exception as exc:  # noqa: BLE001
                arms[f"trad/{model.id}"] = {"skipped": True, "error": str(exc)}
                print(f"    SKIP: {exc}\n")

    hybrid_fit = linear_fit(xs, ys_hybrid) if len(xs) >= 2 else {}
    trad_fit = linear_fit(xs, ys_trad) if len(ys_trad) >= 2 else {}

    report: dict[str, Any] = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "protocol": PROTOCOL,
        "phase": args.phase,
        "mcq_n": len(prompts),
        "models": [{"id": m.id, "hf_id": m.hf_id, "params_b": m.params_b} for m in models],
        "arms": arms,
        "hybrid_lm_patch_scaling": {
            "x": "log10(params_b)",
            **{k: round(v, 4) for k, v in hybrid_fit.items()},
            "points": [
                {"model_id": m.id, "params_b": m.params_b, "accuracy": arms.get(f"hybrid/{m.id}", {}).get("accuracy")}
                for m in models
            ],
        },
        "trad_lm_scaling": {
            "x": "log10(params_b)",
            **{k: round(v, 4) for k, v in trad_fit.items()},
        }
        if trad_fit
        else None,
        "notes": [
            "Rust lm_patch supports sharded safetensors (Qwen 3B/7B index.json).",
            "Gemma/Phi require bake to Qwen2 or separate architecture — not in this ladder.",
        ],
    }

    if hybrid_fit.get("slope", 0) > 0.03:
        report["verdict"] = "MCQ_LADDER_SCALES"
    elif len(ys_hybrid) >= 2 and ys_hybrid[-1] > ys_hybrid[0]:
        report["verdict"] = "MCQ_LADDER_IMPROVING"
    else:
        report["verdict"] = "MCQ_LADDER_FLAT"

    if hybrid_fit.get("slope") is not None:
        proj = []
        for pb in (7.0,):
            x = math.log10(pb)
            acc = min(1.0, hybrid_fit["intercept"] + hybrid_fit["slope"] * x)
            proj.append({"params_b": pb, "model_id": "qwen_7b", "accuracy_proj": round(acc, 4)})
        report["hybrid_lm_patch_scaling"]["projections"] = proj

    REPORTS.mkdir(parents=True, exist_ok=True)
    out = REPORTS / "mcq_ladder_v1_test.json"
    out.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"Verdict: {report['verdict']}")
    if hybrid_fit:
        print(f"Hybrid slope (log10 params): {hybrid_fit.get('slope', 0):.4f}  R²={hybrid_fit.get('r2', 0):.3f}")
    print(f"→ {out}")


if __name__ == "__main__":
    main()
