#!/usr/bin/env python3
"""Fate Output Gate benchmark — v1 fixed vs v2 E7 algebraic vs Rust native scores.

Protocol: fate-output-gate-v2

Algebra:
  τ(k) = (r+δ)/(dim+δ+k) = 12/(138+k)
  Δ_req(s₁) = s₁·δ/(2r) = s₁·5/14

Usage:
  python3 tools/valhalla_inference/test_fate_output_gate.py
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

from fog_gate import FOG_BASE, FOG_DELTA, FOG_DIM, FOG_R, fog_from_base_row  # noqa: E402
from metrics import score_answer  # noqa: E402
from run_fair_benchmark import load_suite, select_prompts  # noqa: E402
from run_potential_analysis import classify_slices  # noqa: E402
from run_scale_benchmark import corpus_for_prompt  # noqa: E402
from run_strict_incubation_experiment import strict_user_text  # noqa: E402
from valhalla_base import ValhallaBase  # noqa: E402

PROTOCOL = "fate-output-gate-v2"
REPORTS = REPO / "reports/valhalla_inference"

SWAP_CASES = [
    {
        "id": "SWAP_01",
        "question": "What is the capital of Australia?",
        "ctx_ok": ["The capital of Australia is Canberra.", "Australia is in Oceania."],
        "ctx_bad": ["The capital of France is Paris.", "France is in Western Europe."],
        "keywords_ok": ["canberra"],
        "keywords_bad": ["paris"],
    },
    {
        "id": "SWAP_02",
        "question": "At what temperature does water boil at standard pressure?",
        "ctx_ok": ["Water boils at 100 degrees Celsius at standard atmospheric pressure."],
        "ctx_bad": ["Water freezes at 0 degrees Celsius at standard pressure."],
        "keywords_ok": ["100"],
        "keywords_bad": ["0", "freez"],
    },
]

GATE_MODES = ("v1_fixed", "v2_algebraic", "v2_rust_native")


def make_base(*, cycles: int = 2) -> ValhallaBase:
    return ValhallaBase(
        decode="hybrid",
        follow_up_decode="native_follow_up_aware",
        fate_ingress_routing="quad_cycle",
        fate_qa_feedback=False,
        incubation_cycles=cycles,
        max_new=128,
    )


def run_probe(
    base: ValhallaBase,
    prompt: dict,
    ctx: list[str],
    *,
    gate_mode: str,
) -> dict:
    base.clear_context()
    base.set_context(ctx)
    doc = base.run([{
        "id": prompt["id"],
        "text": prompt.get("text") or strict_user_text(prompt),
        "score_type": prompt.get("score_type", "open"),
        "reference_answer": prompt.get("reference_answer", ""),
        "keywords": prompt.get("keywords") or [],
    }])
    row = (doc.get("rows") or [{}])[0]
    tel = doc.get("fate_telemetry") or {}
    mode_key = "native" if gate_mode == "v2_rust_native" else "algebraic"
    if gate_mode == "v1_fixed":
        fe = row.get("fog_evidence") or {}
        from fog_gate import fog_decide_v1_fixed
        fog = fog_decide_v1_fixed(
            top_score=float(fe.get("top_score") or 0),
            margin=float(fe.get("top_score", 0) or 0) - float(fe.get("second_score") or 0),
            keyword_count=int(fe.get("keyword_count") or 0),
            incubation_cycles=int(tel.get("incubation_cycles") or 2),
            decode_path=str(row.get("decode_path", "")),
            answer=str(row.get("answer", "")),
        )
    else:
        fog = fog_from_base_row(row, tel, mode=mode_key)

    sc = score_answer(row.get("answer", ""), prompt)
    return {
        "id": prompt["id"],
        "gate_mode": gate_mode,
        "always_emit_correct": sc["correct"],
        "fog_decision": fog.decision,
        "fog_reason": fog.reason,
        "top_score": fog.top_score,
        "margin": fog.margin,
        "tau": fog.tau,
        "delta_required": fog.delta_required,
        "formula": fog.formula,
        "gated_correct": sc["correct"] if fog.decision == "emit" else None,
        "native_fog": row.get("fog_decision"),
        "native_evidence": row.get("fog_evidence"),
    }


def summarize(rows: list[dict]) -> dict:
    n = len(rows)
    always = sum(int(r["always_emit_correct"]) for r in rows)
    emitted = [r for r in rows if r["fog_decision"] == "emit"]
    gated_correct = sum(1 for r in emitted if r["gated_correct"])
    return {
        "n": n,
        "always_emit_accuracy": always / max(n, 1),
        "fog_emit_count": len(emitted),
        "fog_abstain_count": sum(1 for r in rows if r["fog_decision"] == "abstain"),
        "fog_defer_count": sum(1 for r in rows if r["fog_decision"] == "defer"),
        "fog_emitted_precision": gated_correct / max(len(emitted), 1),
        "fog_coverage": len(emitted) / max(n, 1),
    }


def run_battery(
    prompts: list[dict],
    corpus: list[str],
    *,
    ctx_fn,
    gate_mode: str,
    cycles: int = 2,
) -> dict:
    base = make_base(cycles=cycles)
    rows = [run_probe(base, p, ctx_fn(p, corpus), gate_mode=gate_mode) for p in prompts]
    return {**summarize(rows), "rows": rows}


def evaluate(report: dict) -> dict:
    alg = report["modes"]["v2_algebraic"]["g1_targeted"]
    v1 = report["modes"]["v1_fixed"]["g1_targeted"]
    checks = {
        "algebraic_coverage_ge_v1": alg["fog_coverage"] >= v1["fog_coverage"],
        "algebraic_precision_ge_always_minus_2pp": (
            alg["fog_emitted_precision"] >= alg["always_emit_accuracy"] - 0.02
        ),
        "algebraic_targeted_coverage_ge_75pct": alg["fog_coverage"] >= 0.75,
        "distractor_abstain_ge_50pct": (
            report["modes"]["v2_algebraic"]["g2_distractor"]["fog_abstain_count"]
            / max(report["modes"]["v2_algebraic"]["g2_distractor"]["n"], 1)
            >= 0.50
        ),
    }
    passed = sum(1 for v in checks.values() if v)
    return {
        "checks": checks,
        "pass_rate": passed / max(len(checks), 1),
        "verdict": "FOG_V2_ALGEBRAIC_OK" if passed >= 3 else "FOG_V2_NEEDS_TUNING",
    }


def main() -> None:
    ap = argparse.ArgumentParser(description="FOG v2 algebraic benchmark")
    ap.add_argument("--phase", default="test")
    ap.add_argument("--limit", type=int, default=0)
    args = ap.parse_args()

    suite = load_suite()
    corpus = suite["corpus"]["ingest_clean"]
    prompts = select_prompts(suite, args.phase)
    slices = classify_slices(prompts, corpus)
    eval_prompts = [p for p in prompts if p["id"] in slices["open_retrieval"]]
    if args.limit > 0:
        eval_prompts = eval_prompts[: args.limit]

    print(f"=== {PROTOCOL} ===")
    print(f"  algebra: τ(k)=12/(138+k), Δ_req=s₁·{FOG_DELTA}/({FOG_DIM}+{FOG_R})")
    print(f"  n={len(eval_prompts)}\n")

    modes: dict[str, Any] = {}
    for gm in GATE_MODES:
        print(f"--- {gm} ---")
        g1 = run_battery(
            eval_prompts, corpus,
            ctx_fn=lambda p, c: corpus_for_prompt(p, c),
            gate_mode=gm,
        )
        g2 = run_battery(
            eval_prompts[: min(12, len(eval_prompts))],
            corpus,
            ctx_fn=lambda p, c: [ln for ln in c if ln not in corpus_for_prompt(p, c)][:8] or c[:2],
            gate_mode=gm,
        )
        swap_rows: list[dict] = []
        base = make_base()
        for case in SWAP_CASES:
            for side, ctx, kw in (
                ("ok", case["ctx_ok"], case["keywords_ok"]),
                ("bad", case["ctx_bad"], case["keywords_bad"]),
            ):
                p = {
                    "id": f"{case['id']}_{side}",
                    "text": case["question"],
                    "score_type": "open",
                    "keywords": kw,
                    "reference_answer": kw[0],
                }
                swap_rows.append(run_probe(base, p, ctx, gate_mode=gm))
        g3 = {**summarize(swap_rows), "rows": swap_rows}
        print(
            f"  G1 acc={g1['always_emit_accuracy']:.1%} "
            f"prec={g1['fog_emitted_precision']:.1%} cov={g1['fog_coverage']:.1%}\n"
        )
        modes[gm] = {"g1_targeted": g1, "g2_distractor": g2, "g3_swap": g3}

    report = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "protocol": PROTOCOL,
        "algebra": {
            "r": FOG_R,
            "delta": FOG_DELTA,
            "dim_plus_delta": FOG_BASE,
            "tau_formula": "τ(k) = (r+δ)/(dim+δ+k) = 12/(138+k)",
            "delta_formula": "Δ_req(s₁) = s₁·δ/(dim+r) = s₁·5/140 = s₁/28",
        },
        "modes": modes,
    }
    ev = evaluate(report)
    report["evaluation"] = ev
    print(f"Verdict: {ev['verdict']}  checks={ev['checks']}")

    REPORTS.mkdir(parents=True, exist_ok=True)
    out = REPORTS / "fate_output_gate_v2_test.json"
    out.write_text(json.dumps(report, indent=2, ensure_ascii=False), encoding="utf-8")
    print(f"\n→ {out}")


if __name__ == "__main__":
    main()
