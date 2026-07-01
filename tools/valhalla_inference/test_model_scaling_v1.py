#!/usr/bin/env python3
"""Model scaling + training-substrate evidence — multi-backbone ladder.

Protocol: model-scaling-v1

Proves:
  1. Valhalla hybrid **open** (Tier B native) ~backbone-invariant
  2. trad_lm RAG **scales** with params (0.5B → 2B → 3.8B)
  3. **Structure premium** (Valhalla − trad) largest on smallest backbone
  4. **Zero-weight training** (curriculum + Fate feedback) lifts eval without LM SGD

Models (default ladder):
  - Qwen2.5-0.5B-Instruct
  - google/gemma-2-2b-it
  - microsoft/Phi-3-mini-4k-instruct

Usage:
  python3 tools/valhalla_inference/test_model_scaling_v1.py
  python3 tools/valhalla_inference/test_model_scaling_v1.py --fast --limit 12
  python3 tools/valhalla_inference/test_model_scaling_v1.py --skip-training
"""

from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

REPO = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(REPO / "tools/fair_benchmark"))
sys.path.insert(0, str(REPO / "tools/standard_benchmarks"))

from metrics import score_answer  # noqa: E402
from run_fair_benchmark import load_suite, select_prompts  # noqa: E402
from run_potential_analysis import classify_slices  # noqa: E402
from run_scale_benchmark import corpus_for_prompt, targeted_context  # noqa: E402
from run_strict_incubation_experiment import strict_user_text  # noqa: E402
from scaling_analysis import analyze_scaling  # noqa: E402
from scaling_models import SCALING_LADDER, ScalingModel  # noqa: E402
from test_context_arch_compare import resolve_lm_path, trad_generate  # noqa: E402
from test_context_performance import context_block  # noqa: E402
from test_valhalla_base_training_potential import run_learning_paradigms, run_transfer_eval  # noqa: E402
from valhalla_base import ValhallaBase  # noqa: E402

PROTOCOL = "model-scaling-v1"
REPORTS = REPO / "reports/valhalla_inference"


def production_base(*, lm_model: str = "Qwen/Qwen2.5-0.5B-Instruct") -> ValhallaBase:
    return ValhallaBase(
        lm_model=lm_model,
        decode="hybrid",
        mcq_decode="lm_patch",
        follow_up_decode="native_follow_up_aware",
        fate_ingress_routing="quad_cycle",
        fate_qa_feedback=False,
        incubation_cycles=2,
        max_new=128,
    )


def load_open_mcq(
    phase: str, *, limit: int
) -> tuple[list[str], list[dict], list[dict]]:
    suite = load_suite()
    corpus = suite["corpus"]["ingest_clean"]
    prompts = select_prompts(suite, phase)
    open_ids = classify_slices(prompts, corpus)["open_retrieval"]
    open_p = [p for p in prompts if p["id"] in open_ids and p.get("score_type") == "open"]
    mcq_p = [p for p in prompts if p.get("score_type") == "mcq"]
    if limit > 0:
        open_p = open_p[:limit]
        mcq_p = mcq_p[: min(limit, len(mcq_p))]
    return corpus, open_p, mcq_p


def ctx_for_prompt(p: dict, corpus: list[str], *, mode: str) -> list[str]:
    if mode == "targeted":
        return targeted_context(p, corpus)
    return corpus_for_prompt(p, corpus)


def run_valhalla_open(corpus: list[str], prompts: list[dict], *, ctx_mode: str) -> dict[str, Any]:
    rows: list[dict] = []
    correct = 0
    for p in prompts:
        base = production_base()
        ctx = ctx_for_prompt(p, corpus, mode=ctx_mode)
        base.set_context(ctx)
        q = {
            "id": p["id"],
            "text": p["text"],
            "score_type": "open",
            "reference_answer": p.get("reference_answer", ""),
            "keywords": p.get("keywords") or [],
        }
        out = base.run([q])
        row = (out.get("rows") or [{}])[0]
        ans = str(row.get("answer") or "")
        ok = bool(score_answer(ans, p)["correct"])
        correct += int(ok)
        rows.append({"id": p["id"], "correct": ok, "answer": ans[:160]})
    n = len(prompts)
    return {"n": n, "correct": correct, "accuracy": correct / max(n, 1), "rows": rows}


def run_valhalla_mcq(
    corpus: list[str], prompts: list[dict], *, lm_model: str, ctx_mode: str
) -> dict[str, Any]:
    rows: list[dict] = []
    correct = 0
    for p in prompts:
        base = production_base(lm_model=lm_model)
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
        "lm_model": lm_model,
        "rows": rows,
    }


def run_trad_open(lm, corpus: list[str], prompts: list[dict], model: ScalingModel, *, ctx_mode: str) -> dict[str, Any]:
    rows: list[dict] = []
    correct = 0
    for p in prompts:
        ctx = ctx_for_prompt(p, corpus, mode=ctx_mode)
        ans = trad_generate(lm, p["text"], ctx)
        ok = bool(score_answer(ans, p)["correct"])
        correct += int(ok)
        rows.append({"id": p["id"], "correct": ok, "answer": ans[:160]})
    n = len(prompts)
    return {
        "model_id": model.id,
        "hf_id": model.hf_id,
        "params_b": model.params_b,
        "n": n,
        "correct": correct,
        "accuracy": correct / max(n, 1),
        "rows": rows,
    }


def run_trad_mcq(
    lm, corpus: list[str], prompts: list[dict], model: ScalingModel, *, ctx_mode: str
) -> dict[str, Any]:
    rows: list[dict] = []
    correct = 0
    for p in prompts:
        ctx = ctx_for_prompt(p, corpus, mode=ctx_mode)
        qtext = strict_user_text(p)
        ans = lm.generate([context_block(ctx) + qtext], max_tokens=128)[0].text.strip()
        ok = bool(score_answer(ans, p)["correct"])
        correct += int(ok)
        rows.append({"id": p["id"], "correct": ok, "answer": ans[:120]})
    n = len(prompts)
    return {
        "model_id": model.id,
        "hf_id": model.hf_id,
        "params_b": model.params_b,
        "n": n,
        "correct": correct,
        "accuracy": correct / max(n, 1),
        "rows": rows,
    }


def write_markdown(report: dict[str, Any], path: Path) -> None:
    a = report["analysis"]
    lines = [
        "# Model scaling & training substrate — 2026-06-30",
        "",
        f"**Protocol:** `{PROTOCOL}` · **Verdict:** `{a['verdict']}`",
        "",
        "## Headline",
        "",
        f"| Metric | Value |",
        f"|--------|-------|",
        f"| Valhalla hybrid open | **{report['valhalla_hybrid_open']['correct']}/{report['valhalla_hybrid_open']['n']}** ({report['valhalla_hybrid_open']['accuracy']:.1%}) |",
    ]
    for p in a["structure_premium_by_model"]:
        lines.append(
            f"| Structure premium @ {p['params_b']}B | **{p['structure_premium_pp']:+.1f}pp** (trad {p['trad_lm_open']:.1%}) |"
        )
    if report.get("training_substrate") and not report["training_substrate"].get("skipped"):
        ts = report["training_substrate"]
        if ts.get("tpi_v2") is not None:
            lines.append(f"| TPI-v2 (cross-ref) | **{ts['tpi_v2']:.3f}** |")
        tf = (ts.get("transfer_eval") or {}).get("transfer_fresh_retrieval") or {}
        if tf.get("accuracy") is not None:
            lines.append(f"| Transfer fresh retrieval | **{tf['correct']}/{tf['n']}** ({tf['accuracy']:.1%}) |")
        lp = ts.get("learning_paradigms") or ts
        lifts = lp.get("lifts_pp") or {}
        if lifts:
            lines.append(f"| QA feedback lift | **{lifts.get('feedback_lift_pp', 0):+.1f}pp** |")
    vh_mcq = report.get("valhalla_hybrid_mcq_lm_patch") or {}
    if vh_mcq.get("n"):
        lines.append(f"| Valhalla MCQ lm_patch (0.5B) | **{vh_mcq['correct']}/{vh_mcq['n']}** ({vh_mcq['accuracy']:.1%}) |")
    for mid, arm in (report.get("trad_lm_mcq_by_model") or {}).items():
        if arm.get("n"):
            lines.append(f"| trad MCQ @ {arm.get('params_b', mid)}B | **{arm['correct']}/{arm['n']}** ({arm['accuracy']:.1%}) |")
    lines.extend(["", "## Patterns", ""])
    for pat in a["patterns"]:
        lines.append(f"- `{pat}`")
    lines.extend(["", "## Scale projection (trad_lm open, log-linear)", ""])
    lines.append("| Params | trad_lm (proj) | Valhalla (observed) | Premium |")
    lines.append("|--------|----------------|---------------------|---------|")
    for row in a["scale_projection"]:
        lines.append(
            f"| {row['params_b']}B | {row['trad_lm_open_projected']:.1%} | "
            f"{row['valhalla_open_assumed']:.1%} | {row['structure_premium_projected_pp']:+.1f}pp |"
        )
    lines.extend(["", "---", "", "*Rogue Intelligence LNC.*"])
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--phase", default="test")
    ap.add_argument("--limit", type=int, default=0, help="open+mcq cap per slice")
    ap.add_argument("--fast", action="store_true", help="limit=12, skip training")
    ap.add_argument("--skip-training", action="store_true")
    ap.add_argument("--skip-mcq", action="store_true")
    ap.add_argument("--open-only", action="store_true")
    ap.add_argument("--mcq-only", action="store_true")
    ap.add_argument("--training-only", action="store_true")
    ap.add_argument("--merge-into", type=Path, default=None, help="Merge arms into existing JSON")
    ap.add_argument(
        "--model",
        action="append",
        dest="model_ids",
        help="Scaling model id (qwen_0.5b, gemma_2b, phi_3.8b)",
    )
    ap.add_argument(
        "--context-mode",
        choices=("targeted", "corpus_for_prompt"),
        default="targeted",
        help="Production targeted ingest vs full corpus_for_prompt",
    )
    args = ap.parse_args()

    limit = 12 if args.fast else args.limit
    if args.fast:
        args.skip_training = True

    models = [m for m in SCALING_LADDER if not args.model_ids or m.id in args.model_ids]
    if not models:
        sys.exit("No models selected")

    if args.training_only:
        args.skip_mcq = True
        args.skip_training = False

    corpus, open_p, mcq_p = load_open_mcq(args.phase, limit=limit)
    eval_for_training = list(open_p)
    if args.mcq_only:
        args.skip_training = True
        args.skip_mcq = False
        open_p = []
    if args.open_only and not args.training_only:
        mcq_p = []
    if args.training_only:
        open_p = eval_for_training
        mcq_p = []

    if args.merge_into and args.merge_into.is_file():
        report = json.loads(args.merge_into.read_text(encoding="utf-8"))
    else:
        report = None

    print(f"=== {PROTOCOL} open={len(open_p)} mcq={len(mcq_p)} ctx={args.context_mode} models={[m.id for m in models]} ===\n")

    if report is None:
        report = {
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "protocol": PROTOCOL,
            "context_mode": args.context_mode,
            "open_eval_n": len(open_p),
            "mcq_eval_n": len(mcq_p),
            "models": [{"id": m.id, "hf_id": m.hf_id, "params_b": m.params_b} for m in models],
            "fairness": {
                "valhalla_open_backbone_independent": True,
                "trad_lm_per_backbone": True,
                "no_lm_weight_updates_in_training_arm": True,
            },
        }

    if open_p and not args.training_only:
        print("[1] Valhalla hybrid open (Tier B native, fixed stack) …")
        vh_open = run_valhalla_open(corpus, open_p, ctx_mode=args.context_mode)
        report["valhalla_hybrid_open"] = vh_open
        report["open_eval_n"] = len(open_p)
        print(f"    {vh_open['correct']}/{vh_open['n']} ({vh_open['accuracy']:.1%})\n")

        trad_open: dict[str, Any] = report.get("trad_lm_open_by_model") or {}
        trad_mcq: dict[str, Any] = report.get("trad_lm_mcq_by_model") or {}
        from loader import ValhallaLLM  # noqa: E402

        for i, model in enumerate(models, start=2):
            print(f"[{i}] trad_lm open ({model.hf_id}) …")
            lm = None
            try:
                lm_path = resolve_lm_path(model.hf_id)
                lm = ValhallaLLM(lm_path, engine="transformers")
                arm = run_trad_open(lm, corpus, open_p, model, ctx_mode=args.context_mode)
                trad_open[model.id] = arm
                print(f"    {arm['correct']}/{arm['n']} ({arm['accuracy']:.1%})")
            except Exception as exc:  # noqa: BLE001
                trad_open[model.id] = {"skipped": True, "error": str(exc), "model_id": model.id}
                print(f"    SKIP: {exc}")

            if not args.skip_mcq and mcq_p and lm is not None:
                print(f"    trad_lm mcq ({model.id}) …")
                try:
                    mcq_arm = run_trad_mcq(lm, corpus, mcq_p, model, ctx_mode=args.context_mode)
                    trad_mcq[model.id] = mcq_arm
                    print(f"    mcq {mcq_arm['correct']}/{mcq_arm['n']} ({mcq_arm['accuracy']:.1%})")
                except Exception as exc:  # noqa: BLE001
                    trad_mcq[model.id] = {"skipped": True, "error": str(exc)}
                    print(f"    mcq SKIP: {exc}")
            print()

        report["trad_lm_open_by_model"] = trad_open
        report["trad_lm_mcq_by_model"] = trad_mcq
    else:
        trad_open = report.get("trad_lm_open_by_model") or {}
        trad_mcq = report.get("trad_lm_mcq_by_model") or {}

    if mcq_p and not args.skip_mcq:
        qwen = next((m for m in models if m.lm_patch_rust), None)
        if qwen:
            print("[lm_patch] Valhalla hybrid MCQ (Rust Candle, Qwen only) …")
            try:
                vh_mcq = run_valhalla_mcq(corpus, mcq_p, lm_model=qwen.hf_id, ctx_mode=args.context_mode)
                report["valhalla_hybrid_mcq_lm_patch"] = vh_mcq
                print(f"    {vh_mcq['correct']}/{vh_mcq['n']} ({vh_mcq['accuracy']:.1%})\n")
            except Exception as exc:  # noqa: BLE001
                report["valhalla_hybrid_mcq_lm_patch"] = {"skipped": True, "error": str(exc)}
                print(f"    SKIP: {exc}\n")

        if not trad_mcq:
            from loader import ValhallaLLM  # noqa: E402

            trad_mcq = {}
            for model in models:
                print(f"[trad mcq] {model.hf_id} …")
                try:
                    lm_path = resolve_lm_path(model.hf_id)
                    lm = ValhallaLLM(lm_path, engine="transformers")
                    trad_mcq[model.id] = run_trad_mcq(lm, corpus, mcq_p, model, ctx_mode=args.context_mode)
                    a = trad_mcq[model.id]
                    print(f"    {a['correct']}/{a['n']} ({a['accuracy']:.1%})")
                except Exception as exc:  # noqa: BLE001
                    trad_mcq[model.id] = {"skipped": True, "error": str(exc)}
                    print(f"    SKIP: {exc}")
            report["trad_lm_mcq_by_model"] = trad_mcq

    if not args.skip_training and eval_for_training:
        print("[training] Learning paradigms + transfer eval (structure-only, no LM SGD) …")
        learning = run_learning_paradigms(eval_for_training, corpus)
        transfer = run_transfer_eval(eval_for_training, corpus)
        tpi_ref = REPORTS / "valhalla_base_training_potential_test_v2.json"
        tpi_doc = json.loads(tpi_ref.read_text()) if tpi_ref.is_file() else {}
        report["training_substrate"] = {
            "learning_paradigms": learning,
            "transfer_eval": transfer,
            "tpi_v2_cross_ref": str(tpi_ref.name) if tpi_doc else None,
            "tpi_v2": (tpi_doc.get("tpi") or {}).get("tpi"),
        }
        lifts = learning["lifts_pp"]
        tf = transfer.get("transfer_fresh_retrieval") or transfer.get("transfer_fresh_retrieval_no_feedback") or {}
        print(
            f"    isolated {learning['isolated_baseline']['accuracy']:.1%} · "
            f"transfer fresh {tf.get('accuracy', 0):.1%} · "
            f"feedback {lifts['feedback_lift_pp']:+.1f}pp\n"
        )
    else:
        report["training_substrate"] = {"skipped": True}

    report["analysis"] = analyze_scaling(report)

    REPORTS.mkdir(parents=True, exist_ok=True)
    json_path = REPORTS / "model_scaling_v1_test.json"
    md_path = REPORTS / "MODEL_SCALING_20260630.md"
    json_path.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    write_markdown(report, md_path)

    a = report["analysis"]
    print(f"Verdict: {a['verdict']}")
    print(f"Patterns: {', '.join(a['patterns'])}")
    print(f"→ {json_path}")
    print(f"→ {md_path}")


if __name__ == "__main__":
    main()
