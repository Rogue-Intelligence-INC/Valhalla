#!/usr/bin/env python3
"""Fate Output Gate (FOG) benchmark — emit vs abstain vs defer.

Protocol: fate-output-gate-v1

Compares always-emit baseline vs FOG-gated answers on:
  G1 targeted open_retrieval (fair holdout)
  G2 distractor context (should abstain more)
  G3 swap fidelity (correct vs wrong context)
  G4 structure defer (incubation_cycles=1 vs 2)

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

from fog_gate import fog_decide  # noqa: E402
from metrics import score_answer  # noqa: E402
from run_fair_benchmark import load_suite, select_prompts  # noqa: E402
from run_potential_analysis import classify_slices  # noqa: E402
from run_scale_benchmark import corpus_for_prompt  # noqa: E402
from run_strict_incubation_experiment import strict_user_text  # noqa: E402
from valhalla_base import ValhallaBase  # noqa: E402

PROTOCOL = "fate-output-gate-v1"
REPORTS = REPO / "reports/valhalla_inference"

SWAP_CASES = [
    {
        "id": "SWAP_01",
        "question": "What is the capital of Australia?",
        "ctx_ok": ["The capital of Australia is Canberra.", "Australia is in Oceania."],
        "ctx_bad": ["The capital of France is Paris.", "France is in Western Europe."],
    },
    {
        "id": "SWAP_02",
        "question": "At what temperature does water boil at standard pressure?",
        "ctx_ok": ["Water boils at 100 degrees Celsius at standard atmospheric pressure."],
        "ctx_bad": ["Water freezes at 0 degrees Celsius at standard pressure."],
    },
]


def make_base(*, cycles: int = 2) -> ValhallaBase:
    return ValhallaBase(
        decode="hybrid",
        follow_up_decode="native_follow_up_aware",
        fate_ingress_routing="quad_cycle",
        fate_qa_feedback=False,
        incubation_cycles=cycles,
        max_new=128,
    )


def run_one(
    base: ValhallaBase,
    prompt: dict,
    ctx: list[str],
    *,
    min_structure_cycles: int,
) -> dict:
    base.clear_context()
    base.set_context(ctx)
    doc = base.run(
        [{
            "id": prompt["id"],
            "text": prompt.get("text") or strict_user_text(prompt),
            "score_type": prompt.get("score_type", "open"),
            "reference_answer": prompt.get("reference_answer", ""),
            "keywords": prompt.get("keywords") or [],
        }]
    )
    row = (doc.get("rows") or [{}])[0]
    tel = doc.get("fate_telemetry") or {}
    cycles = int(tel.get("incubation_cycles") or base.incubation_cycles)
    fog = fog_decide(
        question=prompt.get("text") or strict_user_text(prompt),
        corpus=ctx,
        decode_path=str(row.get("decode_path", "")),
        answer=str(row.get("answer", "")),
        incubation_cycles=cycles,
        min_structure_cycles=min_structure_cycles,
    )
    sc = score_answer(row.get("answer", ""), prompt)
    gated_correct = sc["correct"] if fog.decision == "emit" else None
    return {
        "id": prompt["id"],
        "always_emit_correct": sc["correct"],
        "fog_decision": fog.decision,
        "fog_reason": fog.reason,
        "top_score": fog.top_score,
        "margin": fog.margin,
        "gated_correct": gated_correct,
        "decode_path": row.get("decode_path", "")[:80],
        "answer_snip": (row.get("answer") or "")[:100],
    }


def summarize(rows: list[dict]) -> dict:
    n = len(rows)
    always = sum(int(r["always_emit_correct"]) for r in rows)
    emitted = [r for r in rows if r["fog_decision"] == "emit"]
    abstained = [r for r in rows if r["fog_decision"] == "abstain"]
    deferred = [r for r in rows if r["fog_decision"] == "defer"]
    gated_correct = sum(1 for r in emitted if r["gated_correct"])
    return {
        "n": n,
        "always_emit_accuracy": always / max(n, 1),
        "always_emit_correct": always,
        "fog_emit_count": len(emitted),
        "fog_abstain_count": len(abstained),
        "fog_defer_count": len(deferred),
        "fog_emitted_precision": gated_correct / max(len(emitted), 1),
        "fog_emitted_correct": gated_correct,
        "fog_coverage": len(emitted) / max(n, 1),
    }


def run_g1_targeted(prompts: list[dict], corpus: list[str]) -> dict:
    rows: list[dict] = []
    base = make_base(cycles=2)
    for p in prompts:
        ctx = corpus_for_prompt(p, corpus)
        rows.append(run_one(base, p, ctx, min_structure_cycles=2))
    return {"slice": "targeted_open_retrieval", **summarize(rows), "rows": rows}


def run_g2_distractor(prompts: list[dict], corpus: list[str]) -> dict:
    rows: list[dict] = []
    base = make_base(cycles=2)
    for p in prompts:
        good = corpus_for_prompt(p, corpus)
        bad = [ln for ln in corpus if ln not in good][:8] or corpus[:2]
        rows.append(run_one(base, p, bad, min_structure_cycles=2))
    return {"slice": "distractor_context", **summarize(rows), "rows": rows}


def run_g3_swap() -> dict:
    rows: list[dict] = []
    base = make_base(cycles=2)
    hints = {
        "SWAP_01": (["canberra"], ["paris"]),
        "SWAP_02": (["100"], ["0", "freez"]),
    }
    for case in SWAP_CASES:
        for label, ctx in (("ok", case["ctx_ok"]), ("bad", case["ctx_bad"])):
            kw = hints[case["id"]][0 if label == "ok" else 1]
            p = {
                "id": f"{case['id']}_{label}",
                "text": case["question"],
                "score_type": "open",
                "keywords": kw,
                "reference_answer": kw[0],
            }
            rows.append(run_one(base, p, ctx, min_structure_cycles=2) | {"swap_side": label})
    return {"slice": "context_swap", **summarize(rows), "rows": rows}


def run_g4_defer(prompts: list[dict], corpus: list[str]) -> dict:
    out: dict[str, Any] = {"slice": "structure_defer"}
    for cycles in (1, 2):
        rows: list[dict] = []
        base = make_base(cycles=cycles)
        for p in prompts[: min(12, len(prompts))]:
            ctx = corpus_for_prompt(p, corpus)
            rows.append(run_one(base, p, ctx, min_structure_cycles=2))
        out[f"cycles_{cycles}"] = summarize(rows)
    c1 = out["cycles_1"]
    c2 = out["cycles_2"]
    out["defer_lift_cycles1"] = c1["fog_defer_count"] - c2["fog_defer_count"]
    return out


def evaluate_fog(report: dict) -> dict:
    g1 = report["g1_targeted"]
    g2 = report["g2_distractor"]
    checks = {
        "emitted_precision_ge_always_on_targeted": (
            g1["fog_emitted_precision"] >= g1["always_emit_accuracy"] - 0.01
        ),
        "distractor_abstain_rate_ge_15pct": (
            g2["fog_abstain_count"] / max(g2["n"], 1) >= 0.15
        ),
        "targeted_emit_coverage_ge_70pct": g1["fog_coverage"] >= 0.70,
        "cycles1_defer_ge_cycles2": report["g4_defer"]["defer_lift_cycles1"] >= 0,
    }
    passed = sum(1 for v in checks.values() if v)
    return {
        "checks": checks,
        "pass_rate": passed / max(len(checks), 1),
        "verdict": "FOG_V1_PROMISING" if passed >= 3 else "FOG_V1_NEEDS_TUNING",
    }


def main() -> None:
    ap = argparse.ArgumentParser(description="Fate Output Gate benchmark")
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

    print(f"=== {PROTOCOL} phase={args.phase} n={len(eval_prompts)} ===\n")

    print("[G1] Targeted open_retrieval …")
    g1 = run_g1_targeted(eval_prompts, corpus)
    print(
        f"  always={g1['always_emit_accuracy']:.1%}  "
        f"fog_precision={g1['fog_emitted_precision']:.1%}  "
        f"coverage={g1['fog_coverage']:.1%}  "
        f"abstain={g1['fog_abstain_count']}\n"
    )

    print("[G2] Distractor context …")
    g2 = run_g2_distractor(eval_prompts[: min(12, len(eval_prompts))], corpus)
    print(
        f"  always={g2['always_emit_accuracy']:.1%}  "
        f"fog_precision={g2['fog_emitted_precision']:.1%}  "
        f"abstain={g2['fog_abstain_count']}/{g2['n']}\n"
    )

    print("[G3] Context swap …")
    g3 = run_g3_swap()
    print(
        f"  always={g3['always_emit_accuracy']:.1%}  "
        f"fog_precision={g3['fog_emitted_precision']:.1%}  "
        f"emit={g3['fog_emit_count']}/{g3['n']}\n"
    )

    print("[G4] Structure defer (cycles 1 vs 2) …")
    g4 = run_g4_defer(eval_prompts, corpus)
    print(
        f"  cycles=1 defer={g4['cycles_1']['fog_defer_count']}  "
        f"cycles=2 defer={g4['cycles_2']['fog_defer_count']}\n"
    )

    report: dict[str, Any] = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "protocol": PROTOCOL,
        "phase": args.phase,
        "eval_count": len(eval_prompts),
        "fog_params": {
            "min_structure_cycles": 2,
            "min_score": 0.12,
            "min_margin": 0.03,
        },
        "g1_targeted": g1,
        "g2_distractor": g2,
        "g3_swap": g3,
        "g4_defer": g4,
    }
    ev = evaluate_fog(report)
    report["evaluation"] = ev
    print(f"Verdict: {ev['verdict']}  checks={ev['checks']}")

    REPORTS.mkdir(parents=True, exist_ok=True)
    out = REPORTS / "fate_output_gate_test.json"
    out.write_text(json.dumps(report, indent=2, ensure_ascii=False), encoding="utf-8")
    print(f"\n→ {out}")


if __name__ == "__main__":
    main()
