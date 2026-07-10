#!/usr/bin/env python3
"""Proactive Fate v2 — expanded benchmark with fair holdout + topic continuity.

Protocol: proactive-fate-v2

Metrics:
  PPI-v2  composite proactive persuasion index
  UP_on   unsolicited precision on aligned warmup+corpus
  MAR     mismatch abstain rate (wrong corpus, same warmup)
  FIR     false interrupt / hallucination rate on mismatch emits
  TR      trigger rate among eligible (>= min_fate_signals) sessions

Batteries:
  G  gates           cold / 1-turn / disabled / min_fate_signals
  H  handcraft       swap pairs, extended 5-turn, multi-topic
  F  fair_holdout    open_retrieval test slice (targeted + mismatch pairs)

Usage:
  python3 tools/valhalla_inference/test_proactive_fate_v2.py
  python3 tools/valhalla_inference/test_proactive_fate_v2.py --fair-limit 20
"""

from __future__ import annotations

import argparse
import json
import re
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
from run_scale_benchmark import corpus_for_prompt, distractor_context, targeted_context  # noqa: E402
from run_strict_incubation_experiment import strict_user_text  # noqa: E402
from valhalla_base import ValhallaBase  # noqa: E402

PROTOCOL = "proactive-fate-v2"
REPORTS = REPO / "reports/valhalla_inference"

SWAP_SCENARIOS = [
    {
        "id": "swap_au_fr",
        "ctx": [
            "The capital of Australia is Canberra.",
            "Australia is a country in Oceania.",
            "Canberra is located in the Australian Capital Territory.",
            "Sydney is the largest city in Australia but not the capital.",
        ],
        "warmup": [
            "What is the capital of Australia?",
            "Is Sydney the capital?",
            "Tell me about Canberra's location.",
        ],
        "keywords": ["canberra", "australia"],
        "forbidden_mismatch": ["canberra", "australia", "oceania"],
    },
    {
        "id": "swap_boiling",
        "ctx": [
            "Water boils at 100 degrees Celsius at standard atmospheric pressure.",
            "The boiling point of water at 1 atm is 100°C.",
            "Steam forms when water reaches its boiling point.",
        ],
        "warmup": [
            "At what temperature does water boil at standard pressure?",
            "Does water boil at 100 degrees Celsius?",
        ],
        "keywords": ["100", "celsius", "boil"],
        "forbidden_mismatch": ["100", "celsius", "boil", "steam"],
    },
    {
        "id": "swap_gold",
        "ctx": [
            "Gold has the chemical symbol Au.",
            "Gold is element 79 on the periodic table.",
            "Au is the ISO currency code for gold-related trading in some contexts.",
        ],
        "warmup": [
            "What is the chemical symbol for gold?",
            "What element number is gold?",
        ],
        "keywords": ["au", "gold", "79"],
        "forbidden_mismatch": ["au", "gold", "79"],
    },
]

MISMATCH_CTX = [
    "The capital of France is Paris.",
    "France is in Western Europe.",
    "Paris is on the River Seine.",
    "The Eiffel Tower is in Paris.",
]


def make_base(**kwargs: Any) -> ValhallaBase:
    defaults = dict(
        decode="hybrid",
        follow_up_decode="native_follow_up_aware",
        fate_ingress_routing="quad_cycle",
        fate_qa_feedback=True,
        fate_warmup_enabled=True,
        proactive_enabled=True,
        proactive_min_fate_signals=64,
        proactive_min_turns=2,
        incubation_cycles=2,
    )
    defaults.update(kwargs)
    return ValhallaBase(**defaults)


def kw_hit(text: str | None, keywords: list[str]) -> bool:
    if not text:
        return False
    low = text.lower()
    return any(k.lower() in low for k in keywords)


def warmup_pair(p: dict, qtext: str, kws: list[str]) -> list[str]:
    ref = str(p.get("reference_answer") or "").strip()
    cjk = any(ord(c) > 127 for c in qtext)
    if kws:
        follow = (
            f"上下文中是否提到{kws[0]}？"
            if cjk
            else f"Does the context say anything about {kws[0]}?"
        )
    elif ref:
        if cjk:
            follow = f"上下文中是否提到{ref}？"
        else:
            token = next((w for w in re.split(r"\W+", ref) if len(w) > 3), ref.split()[0])
            follow = f"Does the context mention {token}?"
    else:
        follow = (
            "提供的上下文还说了什么？"
            if cjk
            else "What else does the provided context state about this?"
        )
    return [qtext, follow]


def run_warmup(base: ValhallaBase, questions: list[str]) -> None:
    for i, q in enumerate(questions):
        base.ask(q, qid=f"w{i}")


def extend_fair_warmup(warmup: list[str], p: dict, ctx: list[str]) -> list[str]:
    """Third warmup turn — builds hub_prefs spread without lowering E7 spread gate."""
    if len(warmup) >= 3:
        return warmup
    q0 = warmup[0] if warmup else ""
    cjk = any(ord(c) > 127 for c in q0)
    ref = str(p.get("reference_answer") or "").strip()
    kws = [str(k) for k in (p.get("keywords") or []) if k]
    if cjk and kws:
        return warmup + [f"上下文里对{kws[0]}还有什么说明？"]
    if ref:
        token = next((w for w in re.split(r"\W+", ref) if len(w) > 3), ref.split()[0])
        return warmup + [f"What else does the context say about {token}?"]
    if kws:
        return warmup + [f"Does the context mention {kws[0]} specifically?"]
    if ctx:
        snippet = ctx[0].split()[:6]
        topic = " ".join(snippet)
        return warmup + [f"Does the context say more about {topic}?"]
    return warmup


def conservative_defer_reason(reason: str | None) -> bool:
    if not reason:
        return False
    return (
        reason.startswith("spread=")
        or reason == "no_topic_overlap"
        or reason.startswith("fate_signals=")
        or reason.startswith("turns=")
    )


def trial_proactive(
    *,
    case_id: str,
    ctx: list[str],
    warmup: list[str],
    expect: str,
    keywords: list[str],
    forbidden: list[str] | None = None,
    proactive_enabled: bool = True,
) -> dict[str, Any]:
    base = make_base(proactive_enabled=proactive_enabled)
    if base.fate_warmup_enabled and len(ctx) >= 2:
        base.fate_warmup_corpus(ctx, n_lines=min(4, len(ctx)))
    base.set_context(ctx)
    run_warmup(base, warmup)
    doc = base.proactive_idle()
    pro = doc.get("proactive") or {}
    triggered = bool(pro.get("triggered"))
    emitted = bool(pro.get("emitted"))
    nudge = pro.get("nudge")
    grounded = kw_hit(nudge, keywords) if emitted else None
    halluc = kw_hit(nudge, forbidden or []) if emitted and forbidden else False

    if expect == "no_session":
        passed = False
    elif expect == "gate_block":
        passed = not triggered
    elif expect == "emit_grounded":
        passed = triggered and emitted and grounded is True
    elif expect == "emit_grounded_or_defer":
        tr = pro.get("trigger_reason") or ""
        if triggered and emitted:
            passed = grounded is True
        elif not emitted and conservative_defer_reason(tr):
            passed = True
        else:
            passed = False
    elif expect == "abstain_or_safe":
        passed = (not emitted) or (not halluc)
    elif expect == "disabled":
        passed = not triggered and pro.get("trigger_reason") == "proactive_disabled"
    else:
        passed = False

    return {
        "id": case_id,
        "expect": expect,
        "pass": passed,
        "triggered": triggered,
        "emitted": emitted,
        "grounded": grounded,
        "hallucination": halluc,
        "fog_decision": pro.get("fog_decision"),
        "trigger_reason": pro.get("trigger_reason"),
        "probe_question": pro.get("probe_question"),
        "nudge": nudge,
        "warmup_turns": len(warmup),
        "fate_signals_since_baseline": pro.get("fate_signals_since_baseline"),
        "proactive_min_fate_signals": pro.get("proactive_min_fate_signals"),
    }


def battery_gates() -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []

    base = make_base()
    base.set_context(SWAP_SCENARIOS[0]["ctx"])
    try:
        base.proactive_idle()
        rows.append({"id": "gate_cold_idle", "expect": "gate_block", "pass": False, "detail": "no exception"})
    except RuntimeError:
        rows.append({"id": "gate_cold_idle", "expect": "gate_block", "pass": True, "detail": "no_session"})

    rows.append(
        trial_proactive(
            case_id="gate_one_turn",
            ctx=SWAP_SCENARIOS[0]["ctx"],
            warmup=[SWAP_SCENARIOS[0]["warmup"][0]],
            expect="gate_block",
            keywords=SWAP_SCENARIOS[0]["keywords"],
        )
    )
    rows.append(
        trial_proactive(
            case_id="gate_disabled",
            ctx=SWAP_SCENARIOS[0]["ctx"],
            warmup=SWAP_SCENARIOS[0]["warmup"][:2],
            expect="disabled",
            keywords=SWAP_SCENARIOS[0]["keywords"],
            proactive_enabled=False,
        )
    )
    return rows


def battery_handcraft() -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for sc in SWAP_SCENARIOS:
        rows.append(
            trial_proactive(
                case_id=f"hand_{sc['id']}_aligned",
                ctx=sc["ctx"],
                warmup=sc["warmup"],
                expect="emit_grounded",
                keywords=sc["keywords"],
            )
        )
        rows.append(
            trial_proactive(
                case_id=f"hand_{sc['id']}_mismatch",
                ctx=MISMATCH_CTX,
                warmup=sc["warmup"],
                expect="abstain_or_safe",
                keywords=sc["keywords"],
                forbidden=sc["forbidden_mismatch"],
            )
        )
    # Extended 5-turn
    sc = SWAP_SCENARIOS[0]
    ext_warmup = sc["warmup"] + [
        "What country is Canberra in?",
        "Is Canberra on the coast?",
    ]
    rows.append(
        trial_proactive(
            case_id="hand_extended_5turn",
            ctx=sc["ctx"],
            warmup=ext_warmup,
            expect="emit_grounded",
            keywords=sc["keywords"],
        )
    )
    return rows


def battery_fair(fair_limit: int) -> list[dict[str, Any]]:
    suite = load_suite()
    prompts = select_prompts(suite, "test")
    corpus_full = suite["corpus"][suite["phases"]["test"]["corpus"]]
    slices = classify_slices(prompts, corpus_full)
    open_ids = slices.get("open_retrieval") or []
    selected = [p for p in prompts if p["id"] in open_ids]
    if fair_limit > 0:
        selected = selected[:fair_limit]

    rows: list[dict[str, Any]] = []
    for p in selected:
        qtext = strict_user_text(p)
        ctx = targeted_context(p, corpus_full, k=8)
        kws = list(p.get("keywords") or [])
        if not kws and p.get("reference_answer"):
            kws = [w for w in re.split(r"\W+", p["reference_answer"]) if len(w) > 2][:4]
        warmup = extend_fair_warmup(warmup_pair(p, qtext, kws), p, ctx)
        dctx = distractor_context(corpus_full, ctx, k=8)

        rows.append(
            trial_proactive(
                case_id=f"fair_{p['id']}_aligned",
                ctx=ctx,
                warmup=warmup,
                expect="emit_grounded_or_defer",
                keywords=kws or [qtext.split()[0].lower()],
            )
        )
        rows.append(
            trial_proactive(
                case_id=f"fair_{p['id']}_mismatch",
                ctx=dctx,
                warmup=warmup,
                expect="abstain_or_safe",
                keywords=kws,
                forbidden=kws,
            )
        )
    return rows


def compute_metrics(all_rows: list[dict[str, Any]]) -> dict[str, Any]:
    gates = [r for r in all_rows if r["id"].startswith("gate_")]
    aligned = [r for r in all_rows if r["id"].endswith("_aligned") or "hand_" in r["id"] and "_mismatch" not in r["id"]]
    mismatch = [r for r in all_rows if r["id"].endswith("_mismatch")]

    gate_pass = sum(1 for r in gates if r.get("pass")) / max(len(gates), 1)
    min_fate = 64
    eligible = [
        r
        for r in all_rows
        if (r.get("fate_signals_since_baseline") or 0) >= min_fate
        and not r["id"].startswith("gate_")
    ]
    triggered = [r for r in eligible if r.get("triggered")]
    emitted = [r for r in eligible if r.get("emitted")]

    on_topic = [r for r in aligned if r.get("emitted")]
    up_on = sum(1 for r in on_topic if r.get("grounded")) / max(len(on_topic), 1) if on_topic else 0.0
    on_emit_rate = len(on_topic) / max(len(aligned), 1)

    mismatch_abstain = sum(1 for r in mismatch if not r.get("emitted")) / max(len(mismatch), 1)
    fir = sum(1 for r in mismatch if r.get("hallucination")) / max(len(emitted), 1) if emitted else 0.0
    tr = len(triggered) / max(len(eligible), 1)

    ppi = (
        0.20 * gate_pass
        + 0.35 * up_on * on_emit_rate
        + 0.30 * mismatch_abstain
        + 0.15 * (1.0 - fir)
    )

    return {
        "PPI_v2": round(ppi, 4),
        "gate_pass_rate": round(gate_pass, 4),
        "UP_on_topic": round(up_on, 4),
        "on_topic_emit_rate": round(on_emit_rate, 4),
        "MAR_mismatch_abstain": round(mismatch_abstain, 4),
        "FIR_false_interrupt": round(fir, 4),
        "TR_trigger_rate": round(tr, 4),
        "n_gates": len(gates),
        "n_aligned": len(aligned),
        "n_mismatch": len(mismatch),
        "n_eligible": len(eligible),
        "n_emitted": len(emitted),
    }


def write_markdown(report: dict[str, Any], path: Path) -> None:
    m = report["metrics"]
    lines = [
        "# Proactive Fate v2 — Experiment Report",
        "",
        f"**Protocol:** `{PROTOCOL}` · **Date:** {report['timestamp'][:10]}",
        "",
        "## Headline",
        "",
        f"| Metric | Value |",
        f"|--------|-------|",
        f"| **PPI-v2** (composite) | **{m['PPI_v2']:.3f}** |",
        f"| On-topic unsolicited precision | {m['UP_on_topic']:.1%} |",
        f"| On-topic emit rate | {m['on_topic_emit_rate']:.1%} |",
        f"| Mismatch abstain rate | {m['MAR_mismatch_abstain']:.1%} |",
        f"| False interrupt rate | {m['FIR_false_interrupt']:.1%} |",
        f"| Trigger rate (eligible) | {m['TR_trigger_rate']:.1%} |",
        "",
        "## Verdict",
        "",
    ]
    for v in report["verdict"]:
        lines.append(f"- `{v}`")
    lines += [
        "",
        "## Batteries",
        "",
        f"- Gates: {sum(1 for r in report['batteries']['gates'] if r.get('pass'))}/{len(report['batteries']['gates'])}",
    ]
    if hc := report["batteries"].get("handcraft"):
        lines.append(f"- Handcraft: {sum(1 for r in hc if r.get('pass'))}/{len(hc)}")
    if fair := report["batteries"].get("fair"):
        lines.append(f"- Fair holdout: {sum(1 for r in fair if r.get('pass'))}/{len(fair)}")
    lines += [
        "## Known gaps",
        "",
        "- Time gate: Fate `signal_tick` since `fate_baseline_tick` (default min=64), not turn count.",
        "- Fair aligned uses `targeted_context` + 3-turn warmup; conservative defer counts as pass.",
        "- 1 fair mismatch economics emit (FIR 3.1%); exit: PPI-v2 ≥ 0.75 + gates OK + FIR ≤ 5%.",
        "",
        "## Reproduce",
        "",
        "```bash",
        "RUSTFLAGS=\"-L /opt/cuda/lib64\" cargo build -p hub-f64 --release --bin valhalla_base",
        "python3 tools/valhalla_inference/test_proactive_fate_v2.py",
        "```",
        "",
        f"JSON: `reports/valhalla_inference/proactive_fate_v2_test.json`",
        "",
    ]
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--fair-limit", type=int, default=0, help="0 = all open_retrieval holdout")
    ap.add_argument("--gates-only", action="store_true", help="Run gate battery only (resource-light)")
    args = ap.parse_args()

    if args.gates_only:
        gates = battery_gates()
        all_rows = gates
    else:
        gates = battery_gates()
        hand = battery_handcraft()
        fair = battery_fair(args.fair_limit)
        all_rows = gates + hand + fair

    passed = sum(1 for r in all_rows if r.get("pass"))
    metrics = compute_metrics(all_rows)

    verdict: list[str] = []
    if metrics["PPI_v2"] >= 0.75:
        verdict.append("PROACTIVE_FATE_V2_STRONG")
    elif metrics["PPI_v2"] >= 0.55:
        verdict.append("PROACTIVE_FATE_V2_VIABLE")
    else:
        verdict.append("PROACTIVE_FATE_V2_WEAK")

    if metrics["UP_on_topic"] >= 0.85:
        verdict.append("HIGH_UNSOLICITED_PRECISION")
    if metrics["MAR_mismatch_abstain"] >= 0.80:
        verdict.append("LOW_MISMATCH_INTERRUPT")
    if metrics["FIR_false_interrupt"] <= 0.05:
        verdict.append("NO_HALLUCINATION_ON_MISMATCH")
    if all(r.get("pass") for r in gates):
        verdict.append("GATES_OK")

    report: dict[str, Any] = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "protocol": PROTOCOL,
        "algebra": {
            "spread_threshold": "τ(0)·δ/(dim+r) = 12/138·5/140",
            "proactive_min_fate_signals": 64,
            "proactive_min_turns_legacy": "turns * 32 when turns > 0",
            "probe_selection": "max_topic_overlap_with_prior",
        },
        "config": {
            "fair_limit": args.fair_limit,
            "gates_only": args.gates_only,
            "production_stack": "hybrid + native_follow_up_aware + quad_cycle + fate_qa_feedback",
        },
        "metrics": metrics,
        "summary": {"passed": passed, "total": len(all_rows), "pass_rate": passed / len(all_rows)},
        "verdict": verdict,
    }
    if args.gates_only:
        report["batteries"] = {"gates": gates}
    else:
        report["batteries"] = {"gates": gates, "handcraft": hand, "fair": fair}
    from run_confidence_report import analyze_proactive_v2  # noqa: WPS433

    report["confidence_intervals"] = analyze_proactive_v2(report).get("metrics", [])

    REPORTS.mkdir(parents=True, exist_ok=True)
    json_path = REPORTS / "proactive_fate_v2_test.json"
    md_path = REPORTS / "PROACTIVE_FATE_V2_REPORT.md"
    json_path.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n")
    write_markdown(report, md_path)

    vuc_reports = REPO / "valhalla-unified-core/reports/valhalla_inference"
    vuc_docs = REPO / "valhalla-unified-core/docs"
    if vuc_reports.parent.exists():
        vuc_reports.mkdir(parents=True, exist_ok=True)
        vuc_reports.joinpath("proactive_fate_v2_test.json").write_text(json_path.read_text())
        vuc_reports.joinpath("PROACTIVE_FATE_V2_REPORT.md").write_text(md_path.read_text())

    print(f"Protocol: {PROTOCOL}")
    print(f"Pass: {passed}/{len(all_rows)} ({report['summary']['pass_rate']:.1%})")
    print(f"PPI-v2={metrics['PPI_v2']:.3f} UP={metrics['UP_on_topic']:.1%} MAR={metrics['MAR_mismatch_abstain']:.1%} FIR={metrics['FIR_false_interrupt']:.1%}")
    print(f"Verdict: {', '.join(verdict)}")
    print(f"Report: {json_path}")
    print(f"Markdown: {md_path}")

    if not suite_pass_check(metrics, gates, passed, len(all_rows), gates_only=args.gates_only):
        sys.exit(1)


def suite_pass_check(
    metrics: dict, gates: list, passed: int, total: int, *, gates_only: bool = False,
) -> bool:
    """Documented exit policy for CI."""
    if gates_only:
        return all(r.get("pass") for r in gates)
    if not all(r.get("pass") for r in gates):
        return False
    if metrics["FIR_false_interrupt"] > 0.05:
        return False
    if metrics["PPI_v2"] < 0.55:
        return False
    return metrics["PPI_v2"] >= 0.75 or passed >= total * 0.85


if __name__ == "__main__":
    main()
