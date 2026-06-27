#!/usr/bin/env python3
"""QA → Fate feedback loop test: 20 rounds gradual release + 20 rounds free.

Phase 1 (0–19): constraint 1.0 → ~0.05, fate_qa_feedback on, quad_cycle ingress
Phase 2 (20–39): fully free (routing free, evaluated, min constraint)

Usage:
  python3 tools/valhalla_inference/test_fate_qa_feedback_40.py
"""

from __future__ import annotations

import json
import sys
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

from test_context_flexibility_goal import EXTENDED_TURNS, SEEDS  # noqa: E402
from test_context_constraint_ladder import TOPIC_CONTEXT, topic_ok_strict  # noqa: E402
from test_fate_weight_ladder_200 import (  # noqa: E402
    append_for_round,
    fate_params,
    quad_balance,
)
from valhalla_base import ValhallaBase  # noqa: E402

REPORTS = REPO / "reports/valhalla_inference"


def constraint_gradual(round_idx: int, phase1: int) -> float:
    """Phase1: 1.0 → 0.05 linear."""
    if round_idx >= phase1:
        return 0.0
    t = round_idx / max(phase1 - 1, 1)
    return 1.0 - 0.95 * t


def run_thread(thread_id: str, phase1: int, phase2: int) -> dict:
    templates = EXTENDED_TURNS[thread_id]
    total = phase1 + phase2
    base = ValhallaBase(
        decode="hybrid",
        follow_up_decode="native_follow_up_aware",
        fate_ingress_routing="quad_cycle",
        fate_qa_feedback=True,
    )
    base.set_context(SEEDS[thread_id])

    turns: list[dict] = []
    phase_stats = {
        "gradual": {"hits": 0, "n": 0, "spread_sum": 0.0},
        "free": {"hits": 0, "n": 0, "spread_sum": 0.0},
    }

    for r in range(total):
        phase_key = "gradual" if r < phase1 else "free"
        c = constraint_gradual(r, phase1)
        fp = fate_params(c, "quad_cycle")
        if phase_key == "free":
            fp = {
                "fate_weight_mode": "evaluated",
                "fate_routing_free": True,
                "fate_ingress_routing": "round_robin",
                "incubation_cycles": 15,
            }
        base.fate_weight_mode = fp["fate_weight_mode"]
        base.fate_routing_free = fp["fate_routing_free"]
        base.fate_ingress_routing = fp["fate_ingress_routing"]
        base.incubation_cycles = fp["incubation_cycles"]

        aligned, question, hint, forbidden = templates[r % len(templates)]
        for line in append_for_round(thread_id, aligned, c if phase_key == "gradual" else 0.0, r):
            base.append_context(line)

        doc = base.run(
            [{
                "id": f"qa_fb_{thread_id}_{r}",
                "text": question,
                "score_type": "open",
                "reference_answer": hint,
                "keywords": [hint],
            }],
            append_only=base.session.session_state is not None,
        )
        tel = doc.get("fate_telemetry") or {}
        prefs = tel.get("hub_prefs") or []
        spread = float(tel.get("hub_prefs_spread") or 0.0)
        routed = tel.get("hub_routed") or [0, 0, 0, 0]
        ans = doc["rows"][0]["answer"] if doc.get("rows") else ""
        row_correct = doc["rows"][0].get("correct") if doc.get("rows") else False
        ok = topic_ok_strict(ans, hint, forbidden)

        phase_stats[phase_key]["hits"] += int(ok)
        phase_stats[phase_key]["n"] += 1
        phase_stats[phase_key]["spread_sum"] += spread

        turns.append({
            "round": r,
            "phase": phase_key,
            "constraint": round(c, 4),
            "hub_prefs": [round(x, 5) for x in prefs],
            "hub_prefs_spread": round(spread, 5),
            "hub_routed": routed,
            "quad_balance": quad_balance(routed),
            "on_topic": ok,
            "row_correct": row_correct,
            "fate_qa_feedback": True,
        })
        if r % 5 == 0 or r == total - 1:
            qb = quad_balance(routed)
            print(
                f"  r{r:2d} [{phase_key:7s}] c={c:.2f} spread={spread:.3f} "
                f"quads={qb['active_quads']}/4 ok={ok} fb_correct={row_correct}",
                flush=True,
            )

    for pk in phase_stats:
        n = phase_stats[pk]["n"]
        phase_stats[pk]["rate"] = phase_stats[pk]["hits"] / max(n, 1)
        phase_stats[pk]["avg_spread"] = phase_stats[pk]["spread_sum"] / max(n, 1)
        del phase_stats[pk]["spread_sum"]

    hits = sum(t["on_topic"] for t in turns)
    return {
        "thread_id": thread_id,
        "total_rounds": total,
        "on_topic_hits": hits,
        "on_topic_rate": hits / total,
        "phase_stats": phase_stats,
        "final_spread": turns[-1]["hub_prefs_spread"] if turns else 0.0,
        "final_prefs": turns[-1]["hub_prefs"] if turns else [],
        "turns": turns,
    }


def main() -> None:
    phase1, phase2 = 20, 20
    print(f"=== Fate QA feedback {phase1} gradual + {phase2} free ===\n")

    results = []
    for tid in ["physics", "chemistry", "australia"]:
        print(f"\n--- {tid} ---")
        results.append(run_thread(tid, phase1, phase2))

    agg = {"gradual": {"hits": 0, "n": 0, "spread": 0.0}, "free": {"hits": 0, "n": 0, "spread": 0.0}}
    for r in results:
        for pk in agg:
            st = r["phase_stats"][pk]
            agg[pk]["hits"] += st["hits"]
            agg[pk]["n"] += st["n"]
            agg[pk]["spread"] += st["avg_spread"] * st["n"]

    summary = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "protocol": "fate-qa-feedback-v1",
        "phase1_gradual_rounds": phase1,
        "phase2_free_rounds": phase2,
        "fate_qa_feedback": True,
        "fate_ingress_routing": "quad_cycle",
        "threads": results,
        "aggregate": {
            pk: {
                "hits": agg[pk]["hits"],
                "n": agg[pk]["n"],
                "rate": agg[pk]["hits"] / max(agg[pk]["n"], 1),
                "avg_hub_prefs_spread": agg[pk]["spread"] / max(agg[pk]["n"], 1),
            }
            for pk in agg
        },
    }
    g, f = summary["aggregate"]["gradual"], summary["aggregate"]["free"]
    print("\n=== Aggregate ===")
    print(f"  Gradual (20): {g['hits']}/{g['n']} ({100*g['rate']:.1f}%) spread={g['avg_hub_prefs_spread']:.4f}")
    print(f"  Free (20):    {f['hits']}/{f['n']} ({100*f['rate']:.1f}%) spread={f['avg_hub_prefs_spread']:.4f}")
    print(f"  Δ accuracy:   {100*(f['rate']-g['rate']):+.1f}pp")
    print(f"  Δ spread:     {f['avg_hub_prefs_spread']-g['avg_hub_prefs_spread']:+.4f}")

    REPORTS.mkdir(parents=True, exist_ok=True)
    out = REPORTS / "fate_qa_feedback_40.json"
    out.write_text(json.dumps(summary, indent=2, ensure_ascii=False), encoding="utf-8")
    print(f"\n→ {out}")


if __name__ == "__main__":
    main()
