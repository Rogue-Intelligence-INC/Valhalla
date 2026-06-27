#!/usr/bin/env python3
"""ValhallaBase training potential benchmark — can structure+feedback replace fine-tuning?

Protocol: valhalla-base-training-potential-v2

Tests whether ValhallaBase acts as a **trainable AI substrate** without gradient
updates on LM weights:

  B1 Corpus scale curve     — data efficiency (10%→100% targeted context)
  B2 Learning paradigm      — isolated vs persistent vs curriculum ± QA feedback
  B3 Incubation depth       — cycles 1/2/4/8 vs eval accuracy
  B4 Fate transfer eval     — curriculum snapshot + fresh targeted retrieval (v2)
  B5 RAG baseline gap       — ValhallaBase vs traditional_lm RAG (optional)

Primary eval slice: fair-1.2 **open_retrieval** (corpus-grounded open QA).

Metrics:
  TPI-v2 ∈ [0,1] — isolated ceiling, data efficiency @10%, transfer lift,
  feedback transfer lift (positive components only).

Usage:
  python3 tools/valhalla_inference/test_valhalla_base_training_potential.py
  python3 tools/valhalla_inference/test_valhalla_base_training_potential.py --protocol v2
  python3 tools/valhalla_inference/test_valhalla_base_training_potential.py --eval-limit 20
  python3 tools/valhalla_inference/test_valhalla_base_training_potential.py --skip-cycles
"""

from __future__ import annotations

import argparse
import copy
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

from metrics import score_answer  # noqa: E402
from run_fair_benchmark import load_suite, select_prompts  # noqa: E402
from run_potential_analysis import classify_slices  # noqa: E402
from run_scale_benchmark import corpus_for_prompt  # noqa: E402
from run_strict_incubation_experiment import strict_user_text  # noqa: E402
from test_context_flexibility_goal import EXTENDED_TURNS, SEEDS  # noqa: E402
from valhalla_base import ValhallaBase  # noqa: E402

PROTOCOL_V1 = "valhalla-base-training-potential-v1"
PROTOCOL_V2 = "valhalla-base-training-potential-v2"
REPORTS = REPO / "reports/valhalla_inference"

CORPUS_FRAC_TIERS = (0.10, 0.25, 0.50, 1.00)
CYCLE_TIERS = (1, 2, 4, 8)
THREAD_IDS = ("australia", "chemistry", "physics")


def make_base(*, feedback: bool, cycles: int = 2) -> ValhallaBase:
    return ValhallaBase(
        decode="hybrid",
        follow_up_decode="native_follow_up_aware",
        fate_ingress_routing="quad_cycle",
        fate_qa_feedback=feedback,
        incubation_cycles=cycles,
        max_new=128,
    )


def question_payload(prompt: dict) -> dict:
    return {
        "id": prompt["id"],
        "text": strict_user_text(prompt),
        "score_type": prompt.get("score_type", "open"),
        "reference_answer": prompt.get("reference_answer", ""),
        "keywords": prompt.get("keywords") or [],
    }


def eval_batch(base: ValhallaBase, prompts: list[dict], *, append_only: bool) -> dict:
    if not prompts:
        return {"n": 0, "correct": 0, "accuracy": 0.0, "rows": []}
    qs = [question_payload(p) for p in prompts]
    doc = base.run(qs, append_only=append_only and base.session.session_state is not None)
    rows: list[dict] = []
    correct = 0
    for p, row in zip(prompts, doc.get("rows") or []):
        sc = score_answer(row.get("answer", ""), p)
        correct += int(sc["correct"])
        rows.append(
            {
                "id": p["id"],
                "correct": sc["correct"],
                "answer": (row.get("answer") or "")[:160],
            }
        )
    n = len(prompts)
    tel = doc.get("fate_telemetry") or {}
    return {
        "n": n,
        "correct": correct,
        "accuracy": correct / max(n, 1),
        "hub_prefs_spread": tel.get("hub_prefs_spread"),
        "context_lines": doc.get("context_lines"),
        "context_fingerprint": doc.get("context_fingerprint"),
        "rows": rows,
    }


def corpus_tier_lines(prompt: dict, corpus: list[str], frac: float, k_full: int = 8) -> list[str]:
    good = corpus_for_prompt(prompt, corpus, k=k_full)
    if frac >= 1.0:
        return good
    n = max(1, int(math.ceil(len(good) * frac)))
    return good[:n]


def run_corpus_scale(
    eval_prompts: list[dict], corpus: list[str], tiers: tuple[float, ...]
) -> dict:
    out: dict[str, Any] = {"tiers": {}, "points": []}
    for frac in tiers:
        rows_acc: list[float] = []
        detail: list[dict] = []
        for p in eval_prompts:
            base = make_base(feedback=False)
            ctx = corpus_tier_lines(p, corpus, frac)
            base.clear_context()
            base.set_context(ctx)
            doc = base.run([question_payload(p)])
            row = (doc.get("rows") or [{}])[0]
            sc = score_answer(row.get("answer", ""), p)
            rows_acc.append(float(sc["correct"]))
            detail.append({"id": p["id"], "correct": sc["correct"], "context_lines": len(ctx)})
        acc = sum(rows_acc) / max(len(rows_acc), 1)
        out["tiers"][str(frac)] = {"fraction": frac, "accuracy": acc, "detail": detail}
        out["points"].append({"fraction": frac, "accuracy": acc})
    if len(out["points"]) >= 2:
        lo, hi = out["points"][0], out["points"][-1]
        out["scale_lift_pp"] = (hi["accuracy"] - lo["accuracy"]) * 100.0
    else:
        out["scale_lift_pp"] = 0.0
    return out


def run_curriculum(base: ValhallaBase, *, feedback: bool) -> dict:
    """30-round teaching loop; mutates base session."""
    base.fate_qa_feedback = feedback
    rounds: list[dict] = []
    hits = 0
    n = 0
    for tid in THREAD_IDS:
        for i, (append, question, hint, _forbidden) in enumerate(EXTENDED_TURNS[tid]):
            base.append_context(append)
            doc = base.run(
                [{
                    "id": f"train_{tid}_{i}",
                    "text": question,
                    "score_type": "open",
                    "reference_answer": hint,
                    "keywords": [hint],
                }],
                append_only=base.session.session_state is not None,
            )
            row = (doc.get("rows") or [{}])[0]
            ok = bool(row.get("correct"))
            hits += int(ok)
            n += 1
            tel = doc.get("fate_telemetry") or {}
            rounds.append(
                {
                    "thread": tid,
                    "round": i,
                    "train_correct": ok,
                    "hub_prefs_spread": tel.get("hub_prefs_spread"),
                }
            )
    return {
        "rounds": n,
        "train_accuracy": hits / max(n, 1),
        "final_spread": rounds[-1]["hub_prefs_spread"] if rounds else 0.0,
        "round_details": rounds,
    }


def run_learning_paradigms(eval_prompts: list[dict], corpus: list[str]) -> dict:
    full_corpus = list(corpus)

    # Arm 1 — isolated: fresh ingest per eval question
    iso_correct = 0
    for p in eval_prompts:
        base = make_base(feedback=False)
        ctx = corpus_for_prompt(p, full_corpus)
        base.set_context(ctx)
        doc = base.run([question_payload(p)])
        sc = score_answer((doc.get("rows") or [{}])[0].get("answer", ""), p)
        iso_correct += int(sc["correct"])
    isolated = {
        "arm": "isolated_baseline",
        "description": "Fresh session + targeted context per eval question",
        "n": len(eval_prompts),
        "correct": iso_correct,
        "accuracy": iso_correct / max(len(eval_prompts), 1),
    }

    # Arm 2 — persistent static: one full corpus ingest, no curriculum
    base_p = make_base(feedback=False)
    base_p.set_context(full_corpus)
    persistent = eval_batch(base_p, eval_prompts, append_only=True)
    persistent["arm"] = "persistent_static"
    persistent["description"] = "Single full-corpus ingest, batch eval (no curriculum)"

    # Arm 3 — curriculum without feedback
    base_nf = make_base(feedback=False)
    base_nf.set_context(full_corpus)
    curr_nf = run_curriculum(base_nf, feedback=False)
    eval_nf = eval_batch(base_nf, eval_prompts, append_only=True)
    eval_nf["arm"] = "curriculum_no_feedback"
    eval_nf["description"] = "30-round QA curriculum, fate_qa_feedback=false, then eval"
    eval_nf["curriculum"] = curr_nf

    # Arm 4 — curriculum with feedback (training loop)
    base_fb = make_base(feedback=True)
    base_fb.set_context(full_corpus)
    curr_fb = run_curriculum(base_fb, feedback=True)
    eval_fb = eval_batch(base_fb, eval_prompts, append_only=True)
    eval_fb["arm"] = "curriculum_with_feedback"
    eval_fb["description"] = "30-round QA curriculum, fate_qa_feedback=true, then eval"
    eval_fb["curriculum"] = curr_fb

    persistent_gain_pp = (persistent["accuracy"] - isolated["accuracy"]) * 100.0
    feedback_lift_pp = (eval_fb["accuracy"] - eval_nf["accuracy"]) * 100.0
    curriculum_gain_pp = (eval_fb["accuracy"] - persistent["accuracy"]) * 100.0

    return {
        "isolated_baseline": isolated,
        "persistent_static": persistent,
        "curriculum_no_feedback": eval_nf,
        "curriculum_with_feedback": eval_fb,
        "lifts_pp": {
            "persistent_gain_pp": persistent_gain_pp,
            "feedback_lift_pp": feedback_lift_pp,
            "curriculum_gain_pp": curriculum_gain_pp,
        },
    }


def strip_session_for_transfer(session_state: dict[str, Any]) -> dict[str, Any]:
    """Keep trained Fate snapshot; drop polluted corpus/history for fair retrieval."""
    sess = copy.deepcopy(session_state)
    sess["corpus"] = []
    sess["fingerprint"] = ""
    sess["lines_ingested"] = 0
    sess["question_history"] = []
    return sess


def eval_one_transfer(
    trained_session: dict[str, Any],
    prompt: dict,
    corpus: list[str],
    *,
    feedback: bool,
) -> dict:
    """Extend trained snapshot with per-question targeted lines only."""
    base = make_base(feedback=feedback)
    base.session.session_state = strip_session_for_transfer(trained_session)
    targeted = corpus_for_prompt(prompt, corpus)
    for line in targeted:
        base.append_context(line)
    doc = base.run([question_payload(prompt)], append_only=True)
    row = (doc.get("rows") or [{}])[0]
    sc = score_answer(row.get("answer", ""), prompt)
    tel = doc.get("fate_telemetry") or {}
    return {
        "id": prompt["id"],
        "correct": sc["correct"],
        "answer": (row.get("answer") or "")[:160],
        "context_lines": len(targeted),
        "hub_prefs_spread": tel.get("hub_prefs_spread"),
    }


def run_transfer_eval(eval_prompts: list[dict], corpus: list[str]) -> dict:
    """B4: after curriculum, eval with Fate snapshot + fresh targeted retrieval."""
    base_fb = make_base(feedback=True)
    base_fb.set_context(list(corpus))
    curr_fb = run_curriculum(base_fb, feedback=True)
    trained_session = base_fb.session.session_state
    if not trained_session:
        return {"error": "no session_state after curriculum", "skipped": True}

    in_session = eval_batch(base_fb, eval_prompts, append_only=True)
    in_session["arm"] = "curriculum_in_session_polluted"
    in_session["description"] = "v1-style eval on polluted session (80+ lines)"

    rows_nf: list[dict] = []
    correct_nf = 0
    for p in eval_prompts:
        row = eval_one_transfer(trained_session, p, corpus, feedback=False)
        correct_nf += int(row["correct"])
        rows_nf.append(row)
    transfer_nf = {
        "arm": "transfer_fresh_retrieval_no_feedback",
        "description": "Trained snapshot + per-q targeted append; corpus cleared",
        "n": len(eval_prompts),
        "correct": correct_nf,
        "accuracy": correct_nf / max(len(eval_prompts), 1),
        "curriculum": curr_fb,
        "rows": rows_nf,
    }

    transfer_lift_pp = (transfer_nf["accuracy"] - in_session["accuracy"]) * 100.0
    vs_isolated_pp = (transfer_nf["accuracy"] - transfer_nf.get("isolated", 0)) * 100.0

    isolated_acc = 0.0
    for p in eval_prompts:
        base = make_base(feedback=False)
        base.set_context(corpus_for_prompt(p, corpus))
        doc = base.run([question_payload(p)])
        sc = score_answer((doc.get("rows") or [{}])[0].get("answer", ""), p)
        isolated_acc += float(sc["correct"])
    isolated_acc /= max(len(eval_prompts), 1)
    vs_isolated_pp = (transfer_nf["accuracy"] - isolated_acc) * 100.0

    return {
        "curriculum_train_accuracy": curr_fb["train_accuracy"],
        "final_hub_prefs_spread": curr_fb["final_spread"],
        "curriculum_in_session_polluted": in_session,
        "transfer_fresh_retrieval": transfer_nf,
        "isolated_baseline_accuracy": isolated_acc,
        "lifts_pp": {
            "transfer_vs_polluted_session_pp": transfer_lift_pp,
            "transfer_vs_isolated_pp": vs_isolated_pp,
        },
    }


def run_rag_baseline_gap(eval_prompts: list[dict], corpus: list[str]) -> dict:
    """B5: ValhallaBase isolated vs lm_patch vs traditional_lm RAG."""
    from test_context_performance import context_block  # noqa: E402

    iso_correct = 0
    iso_rows: list[dict] = []
    for p in eval_prompts:
        base = make_base(feedback=False)
        ctx = corpus_for_prompt(p, corpus)
        base.set_context(ctx)
        doc = base.run([question_payload(p)])
        ans = (doc.get("rows") or [{}])[0].get("answer", "")
        sc = score_answer(ans, p)
        iso_correct += int(sc["correct"])
        iso_rows.append({"id": p["id"], "correct": sc["correct"], "answer": ans[:160]})
    valhalla_isolated = {
        "arm": "valhalla_isolated_hybrid",
        "n": len(eval_prompts),
        "correct": iso_correct,
        "accuracy": iso_correct / max(len(eval_prompts), 1),
        "rows": iso_rows,
    }

    patch_correct = 0
    patch_rows: list[dict] = []
    for p in eval_prompts:
        base = ValhallaBase(
            decode="lm_patch",
            follow_up_decode="inherit",
            fate_ingress_routing="quad_cycle",
            fate_qa_feedback=False,
            max_new=128,
        )
        ctx = corpus_for_prompt(p, corpus)
        base.set_context(ctx)
        doc = base.run([question_payload(p)])
        ans = (doc.get("rows") or [{}])[0].get("answer", "")
        sc = score_answer(ans, p)
        patch_correct += int(sc["correct"])
        patch_rows.append({"id": p["id"], "correct": sc["correct"], "answer": ans[:160]})
    valhalla_lm_patch = {
        "arm": "valhalla_lm_patch_rag",
        "n": len(eval_prompts),
        "correct": patch_correct,
        "accuracy": patch_correct / max(len(eval_prompts), 1),
        "rows": patch_rows,
    }

    trad: dict[str, Any] = {"arm": "traditional_lm_rag", "skipped": True, "error": None}
    try:
        from test_context_arch_compare import resolve_lm_path  # noqa: E402
        from loader import ValhallaLLM  # noqa: E402

        lm = ValhallaLLM(resolve_lm_path("Qwen/Qwen2.5-0.5B-Instruct"))
        trad_correct = 0
        trad_rows: list[dict] = []
        for p in eval_prompts:
            ctx = corpus_for_prompt(p, corpus)
            qtext = strict_user_text(p)
            ans = lm.generate([context_block(ctx) + qtext], max_tokens=128)[0].text.strip()
            sc = score_answer(ans, p)
            trad_correct += int(sc["correct"])
            trad_rows.append({"id": p["id"], "correct": sc["correct"], "answer": ans[:160]})
        trad = {
            "arm": "traditional_lm_rag",
            "n": len(eval_prompts),
            "correct": trad_correct,
            "accuracy": trad_correct / max(len(eval_prompts), 1),
            "rows": trad_rows,
            "skipped": False,
        }
    except Exception as exc:  # noqa: BLE001
        trad["error"] = str(exc)

    structure_gain_pp = (valhalla_isolated["accuracy"] - valhalla_lm_patch["accuracy"]) * 100.0
    vs_trad_pp = None
    if not trad.get("skipped"):
        vs_trad_pp = (valhalla_isolated["accuracy"] - trad["accuracy"]) * 100.0

    return {
        "valhalla_isolated_hybrid": valhalla_isolated,
        "valhalla_lm_patch_rag": valhalla_lm_patch,
        "traditional_lm_rag": trad,
        "lifts_pp": {
            "structure_over_lm_patch_pp": structure_gain_pp,
            "valhalla_over_traditional_lm_pp": vs_trad_pp,
        },
    }


def run_incubation_ablation(eval_prompts: list[dict], corpus: list[str], cycles: tuple[int, ...]) -> dict:
    tiers: dict[str, dict] = {}
    for c in cycles:
        accs: list[float] = []
        for p in eval_prompts:
            base = make_base(feedback=False, cycles=c)
            base.set_context(corpus_for_prompt(p, corpus))
            doc = base.run([question_payload(p)])
            sc = score_answer((doc.get("rows") or [{}])[0].get("answer", ""), p)
            accs.append(float(sc["correct"]))
        tiers[str(c)] = {
            "incubation_cycles": c,
            "accuracy": sum(accs) / max(len(accs), 1),
        }
    vals = [tiers[str(c)]["accuracy"] for c in cycles]
    tiers["delta_c1_to_max"] = {
        "from": cycles[0],
        "to": cycles[-1],
        "lift_pp": (vals[-1] - vals[0]) * 100.0,
    }
    return tiers


def norm_pp(x: float, cap: float = 25.0) -> float:
    return max(0.0, min(1.0, x / cap))


def compute_tpi_v2(scale: dict, learning: dict, transfer: dict | None, rag: dict | None) -> dict:
    """TPI-v2: reward demonstrated capabilities, not polluted-session paradigms."""
    iso = float(learning["isolated_baseline"]["accuracy"])
    eff_10 = 0.0
    for pt in scale.get("points") or []:
        if abs(pt.get("fraction", 0) - 0.10) < 0.01:
            eff_10 = float(pt["accuracy"])
            break

    transfer_lift = 0.0
    transfer_vs_iso = 0.0
    if transfer and not transfer.get("skipped"):
        lifts = transfer.get("lifts_pp") or {}
        transfer_lift = max(0.0, float(lifts.get("transfer_vs_polluted_session_pp") or 0))
        transfer_vs_iso = float(lifts.get("transfer_vs_isolated_pp") or 0)

    structure_gain = 0.0
    if rag:
        structure_gain = max(0.0, float((rag.get("lifts_pp") or {}).get("structure_over_lm_patch_pp") or 0))

    components = {
        "isolated_ceiling": iso,
        "data_efficiency_10pct": eff_10,
        "transfer_vs_polluted": min(1.0, transfer_lift / 40.0),
        "transfer_vs_isolated_norm": max(0.0, min(1.0, (transfer_vs_iso + 5.0) / 10.0)),
        "structure_over_lm_patch": min(1.0, structure_gain / 15.0),
    }
    weights = {
        "isolated_ceiling": 0.30,
        "data_efficiency_10pct": 0.25,
        "transfer_vs_polluted": 0.20,
        "transfer_vs_isolated_norm": 0.10,
        "structure_over_lm_patch": 0.15,
    }
    tpi = sum(weights[k] * components[k] for k in weights)
    return {
        "tpi": round(tpi, 4),
        "components": {k: round(v, 4) for k, v in components.items()},
        "weights": weights,
    }


def verdict_v2(tpi: float, learning: dict, transfer: dict | None, rag: dict | None) -> list[str]:
    tags: list[str] = []
    iso = learning["isolated_baseline"]["accuracy"]
    if tpi >= 0.75:
        tags.append("STRONG_INFERENCE_TIME_LEARNING")
    elif tpi >= 0.55:
        tags.append("MODERATE_INFERENCE_TIME_LEARNING")
    else:
        tags.append("INFERENCE_TIME_LEARNING_DEVELOPING")
    if iso >= 0.90:
        tags.append("RAG_CEILING_HIGH")
    if transfer and not transfer.get("skipped"):
        lift = float((transfer.get("lifts_pp") or {}).get("transfer_vs_polluted_session_pp") or 0)
        if lift > 5.0:
            tags.append("FATE_TRANSFER_RESCUES_SESSION")
        elif lift < 1.0:
            tags.append("FATE_TRANSFER_NEUTRAL")
    if rag:
        sg = float((rag.get("lifts_pp") or {}).get("structure_over_lm_patch_pp") or 0)
        if sg > 2.0:
            tags.append("STRUCTURE_BEATS_LM_PATCH")
    trad = (rag or {}).get("traditional_lm_rag") or {}
    if not trad.get("skipped") and iso > trad.get("accuracy", 0):
        tags.append("VALHALLA_BEATS_TRAD_LM_RAG")
    if learning["lifts_pp"]["feedback_lift_pp"] > 1.0:
        tags.append("QA_FEEDBACK_LEARNS")
    return tags or ["NEEDS_MORE_DATA"]


def compute_tpi(scale: dict, learning: dict) -> dict:
    lifts = learning["lifts_pp"]
    scale_lift = float(scale.get("scale_lift_pp") or 0.0)
    components = {
        "persistent_gain": norm_pp(lifts["persistent_gain_pp"]),
        "feedback_lift": norm_pp(lifts["feedback_lift_pp"]),
        "curriculum_gain": norm_pp(lifts["curriculum_gain_pp"]),
        "corpus_scale": norm_pp(scale_lift),
    }
    weights = {
        "persistent_gain": 0.25,
        "feedback_lift": 0.25,
        "curriculum_gain": 0.25,
        "corpus_scale": 0.25,
    }
    tpi = sum(weights[k] * components[k] for k in components)
    return {
        "tpi": round(tpi, 4),
        "components": {k: round(v, 4) for k, v in components.items()},
        "weights": weights,
    }


def verdict(tpi: float, learning: dict) -> list[str]:
    tags: list[str] = []
    lifts = learning["lifts_pp"]
    if tpi >= 0.65:
        tags.append("STRONG_TRAINING_SUBSTRATE")
    elif tpi >= 0.40:
        tags.append("MODERATE_TRAINING_SUBSTRATE")
    else:
        tags.append("TRAINING_SUBSTRATE_DEVELOPING")
    if lifts["persistent_gain_pp"] > 2.0:
        tags.append("PERSISTENT_INGEST_WINS")
    if lifts["feedback_lift_pp"] > 1.0:
        tags.append("QA_FEEDBACK_LEARNS")
    if learning["curriculum_with_feedback"]["accuracy"] >= 0.85:
        tags.append("POST_CURRICULUM_EVAL_STRONG")
    return tags or ["NEEDS_MORE_DATA"]


def main() -> None:
    ap = argparse.ArgumentParser(description="ValhallaBase training potential benchmark")
    ap.add_argument("--phase", default="test")
    ap.add_argument("--protocol", choices=("v1", "v2"), default="v2")
    ap.add_argument("--eval-limit", type=int, default=0, help="0 = all open_retrieval")
    ap.add_argument("--skip-scale", action="store_true")
    ap.add_argument("--skip-cycles", action="store_true")
    ap.add_argument("--skip-curriculum", action="store_true")
    ap.add_argument("--skip-transfer", action="store_true")
    ap.add_argument("--skip-rag", action="store_true")
    ap.add_argument("--only-rag", action="store_true", help="Run B5 only; merge into --report-in JSON")
    ap.add_argument(
        "--report-in",
        type=Path,
        default=None,
        help="Existing report JSON to patch (with --only-rag)",
    )
    args = ap.parse_args()

    if args.only_rag and args.report_in is None:
        args.report_in = REPORTS / "valhalla_base_training_potential_test_v2.json"

    protocol = PROTOCOL_V2 if args.protocol == "v2" else PROTOCOL_V1

    suite = load_suite()
    corpus = suite["corpus"]["ingest_clean"]
    prompts = select_prompts(suite, args.phase)
    slices = classify_slices(prompts, corpus)
    eval_prompts = [p for p in prompts if p["id"] in slices["open_retrieval"]]
    if args.eval_limit > 0:
        eval_prompts = eval_prompts[: args.eval_limit]

    if args.only_rag:
        args.skip_scale = True
        args.skip_curriculum = True
        args.skip_transfer = True
        args.skip_cycles = True
        src = args.report_in
        if src is None or not src.is_file():
            print(f"Missing --report-in: {src}", file=sys.stderr)
            sys.exit(1)
        report = json.loads(src.read_text(encoding="utf-8"))
        print(f"=== {protocol} B5-only patch → {src.name} ===")
        print(f"  eval: open_retrieval n={len(eval_prompts)}\n")
        print("[B5] RAG baseline gap (Valhalla vs lm_patch vs traditional_lm) …")
        rag = run_rag_baseline_gap(eval_prompts, corpus)
        report["rag_baseline_gap"] = rag
        report["timestamp"] = datetime.now(timezone.utc).isoformat()
        vh = rag["valhalla_isolated_hybrid"]
        lp = rag["valhalla_lm_patch_rag"]
        print(f"  valhalla_hybrid         {vh['correct']}/{vh['n']} ({vh['accuracy']:.1%})")
        print(f"  valhalla_lm_patch       {lp['correct']}/{lp['n']} ({lp['accuracy']:.1%})")
        trad = rag["traditional_lm_rag"]
        if trad.get("skipped"):
            print(f"  traditional_lm          skipped ({trad.get('error', 'unavailable')})")
        else:
            print(f"  traditional_lm          {trad['correct']}/{trad['n']} ({trad['accuracy']:.1%})")
        rl = rag["lifts_pp"]
        print(f"  structure over lm_patch: {rl['structure_over_lm_patch_pp']:+.1f}pp")
        if rl["valhalla_over_traditional_lm_pp"] is not None:
            print(f"  valhalla over trad_lm:   {rl['valhalla_over_traditional_lm_pp']:+.1f}pp")
        transfer = report.get("transfer_eval")
        if report.get("learning_paradigms") and not report["learning_paradigms"].get("skipped"):
            tpi_doc = compute_tpi_v2(
                report.get("corpus_scale") or {},
                report["learning_paradigms"],
                transfer,
                rag,
            )
            report["tpi"] = tpi_doc
            report["verdict"] = verdict_v2(
                tpi_doc["tpi"], report["learning_paradigms"], transfer, rag
            )
            print(f"\nTPI-v2 = {tpi_doc['tpi']:.3f}  components={tpi_doc['components']}")
            print(f"Verdict: {report['verdict']}")
        src.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
        print(f"\n→ {src}")
        return

    print(f"=== {protocol} phase={args.phase} ===")
    print(f"  eval: open_retrieval n={len(eval_prompts)}  corpus_lines={len(corpus)}\n")

    report: dict[str, Any] = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "protocol": protocol,
        "fair_benchmark_version": suite.get("version", "fair-1.2"),
        "phase": args.phase,
        "eval_slice": "open_retrieval",
        "eval_count": len(eval_prompts),
        "corpus_lines": len(corpus),
        "training_curriculum_rounds": 30,
        "fairness": {
            "no_lm_weight_updates": True,
            "eval_holdout": "fair open_retrieval (not in curriculum templates)",
            "production_stack": {
                "decode": "hybrid",
                "follow_up_decode": "native_follow_up_aware",
                "fate_ingress_routing": "quad_cycle",
            },
        },
    }

    if not args.skip_scale:
        print("[B1] Corpus scale curve …")
        scale = run_corpus_scale(eval_prompts, corpus, CORPUS_FRAC_TIERS)
        report["corpus_scale"] = scale
        for pt in scale["points"]:
            print(f"  frac={pt['fraction']:.0%}  acc={pt['accuracy']:.1%}")
        print(f"  scale_lift: {scale.get('scale_lift_pp', 0):+.1f}pp\n")

    if not args.skip_curriculum:
        print("[B2] Learning paradigms (30-round curriculum + eval) …")
        learning = run_learning_paradigms(eval_prompts, corpus)
        report["learning_paradigms"] = learning
        for key in (
            "isolated_baseline",
            "persistent_static",
            "curriculum_no_feedback",
            "curriculum_with_feedback",
        ):
            arm = learning[key]
            print(f"  {arm['arm']:28s} {arm['correct']}/{arm['n']} ({arm['accuracy']:.1%})")
        lifts = learning["lifts_pp"]
        print(
            f"  lifts: persistent {lifts['persistent_gain_pp']:+.1f}pp  "
            f"feedback {lifts['feedback_lift_pp']:+.1f}pp  "
            f"curriculum {lifts['curriculum_gain_pp']:+.1f}pp\n"
        )
    else:
        report["learning_paradigms"] = {"skipped": True}
        learning = {
            "lifts_pp": {"persistent_gain_pp": 0, "feedback_lift_pp": 0, "curriculum_gain_pp": 0}
        }
        scale = report.get("corpus_scale") or {"scale_lift_pp": 0}

    transfer: dict | None = None
    rag: dict | None = None
    if args.protocol == "v2" and not args.skip_transfer:
        print("[B4] Fate transfer eval (snapshot + fresh targeted retrieval) …")
        transfer = run_transfer_eval(eval_prompts, corpus)
        report["transfer_eval"] = transfer
        if transfer.get("skipped"):
            print(f"  skipped: {transfer.get('error')}\n")
        else:
            pol = transfer["curriculum_in_session_polluted"]
            tr = transfer["transfer_fresh_retrieval"]
            print(f"  in_session_polluted     {pol['correct']}/{pol['n']} ({pol['accuracy']:.1%})")
            print(f"  transfer_fresh          {tr['correct']}/{tr['n']} ({tr['accuracy']:.1%})")
            print(f"  isolated_baseline       {transfer['isolated_baseline_accuracy']:.1%}")
            lifts = transfer["lifts_pp"]
            print(
                f"  lifts: vs polluted {lifts['transfer_vs_polluted_session_pp']:+.1f}pp  "
                f"vs isolated {lifts['transfer_vs_isolated_pp']:+.1f}pp\n"
            )

    if args.protocol == "v2" and not args.skip_rag:
        print("[B5] RAG baseline gap (Valhalla vs lm_patch vs traditional_lm) …")
        rag = run_rag_baseline_gap(eval_prompts, corpus)
        report["rag_baseline_gap"] = rag
        vh = rag["valhalla_isolated_hybrid"]
        lp = rag["valhalla_lm_patch_rag"]
        print(f"  valhalla_hybrid         {vh['correct']}/{vh['n']} ({vh['accuracy']:.1%})")
        print(f"  valhalla_lm_patch       {lp['correct']}/{lp['n']} ({lp['accuracy']:.1%})")
        trad = rag["traditional_lm_rag"]
        if trad.get("skipped"):
            print(f"  traditional_lm          skipped ({trad.get('error', 'unavailable')})")
        else:
            print(f"  traditional_lm          {trad['correct']}/{trad['n']} ({trad['accuracy']:.1%})")
        rl = rag["lifts_pp"]
        print(f"  structure over lm_patch: {rl['structure_over_lm_patch_pp']:+.1f}pp")
        if rl["valhalla_over_traditional_lm_pp"] is not None:
            print(f"  valhalla over trad_lm:   {rl['valhalla_over_traditional_lm_pp']:+.1f}pp")
        print()

    if not args.skip_cycles:
        print("[B3] Incubation cycles ablation …")
        cycles = run_incubation_ablation(eval_prompts, corpus, CYCLE_TIERS)
        report["incubation_cycles"] = cycles
        for c in CYCLE_TIERS:
            print(f"  cycles={c}  acc={cycles[str(c)]['accuracy']:.1%}")
        print(
            f"  Δ c{CYCLE_TIERS[0]}→c{CYCLE_TIERS[-1]}: "
            f"{cycles['delta_c1_to_max']['lift_pp']:+.1f}pp\n"
        )

    if "corpus_scale" not in report:
        report["corpus_scale"] = {"scale_lift_pp": 0, "points": []}
    if not args.skip_curriculum:
        if args.protocol == "v2":
            tpi_doc = compute_tpi_v2(
                report["corpus_scale"],
                report["learning_paradigms"],
                transfer,
                rag,
            )
            report["tpi"] = tpi_doc
            report["verdict"] = verdict_v2(
                tpi_doc["tpi"], report["learning_paradigms"], transfer, rag
            )
            print(f"TPI-v2 = {tpi_doc['tpi']:.3f}  components={tpi_doc['components']}")
        else:
            tpi_doc = compute_tpi(report["corpus_scale"], report["learning_paradigms"])
            report["tpi"] = tpi_doc
            report["verdict"] = verdict(tpi_doc["tpi"], report["learning_paradigms"])
            print(f"TPI-v1 = {tpi_doc['tpi']:.3f}  components={tpi_doc['components']}")
        print(f"Verdict: {report['verdict']}")

    REPORTS.mkdir(parents=True, exist_ok=True)
    suffix = f"_{args.protocol}" if args.protocol == "v2" else ""
    out = REPORTS / f"valhalla_base_training_potential_{args.phase}{suffix}.json"
    out.write_text(json.dumps(report, indent=2, ensure_ascii=False), encoding="utf-8")
    print(f"\n→ {out}")


if __name__ == "__main__":
    main()
