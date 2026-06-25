#!/usr/bin/env python3
"""Fate weight ladder: 100 rounds high-constraint + 100 rounds gradual release.

Phase 1 (rounds 0–99):   Fate fixed weights, routing locked, low optimize cycles
Phase 2 (rounds 100–199): Relax → evaluated weights, routing free, more optimize cycles

Tracks hub_prefs spread / routing / answer quality across 200 rounds in one session.

Usage:
  python3 tools/valhalla_inference/test_fate_weight_ladder_200.py
  python3 tools/valhalla_inference/test_fate_weight_ladder_200.py --phase1 50 --phase2 50  # smoke
"""

from __future__ import annotations

import argparse
import json
import math
import sys
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

from test_context_flexibility_goal import EXTENDED_TURNS, SEEDS  # noqa: E402
from test_context_constraint_ladder import TOPIC_CONTEXT, topic_ok_strict  # noqa: E402
from valhalla_base import ValhallaBase  # noqa: E402

REPORTS = REPO / "reports/valhalla_inference"


def constraint_for_round(round_idx: int, phase1: int, phase2: int) -> float:
    """1.0 = fully constrained, 0.0 = unconstrained."""
    total = phase1 + phase2
    if round_idx < phase1:
        # Phase 1: 1.0 → 0.55 (still mostly constrained)
        t = round_idx / max(phase1 - 1, 1)
        return 1.0 - 0.45 * t
    j = round_idx - phase1
    t = j / max(phase2 - 1, 1)
    return 0.55 * (1.0 - t)


def fate_params(constraint: float, ingress_routing: str) -> dict:
    """Map constraint [0,1] to Fate / incubation knobs."""
    c = max(0.0, min(1.0, constraint))
    if c > 0.66:
        weight_mode = "fixed"
    elif c > 0.33:
        weight_mode = "evaluated" if c < 0.55 else "fixed"
    else:
        weight_mode = "evaluated"
    if c >= 0.5:
        ingress = ingress_routing
    else:
        ingress = "round_robin"
    return {
        "fate_weight_mode": weight_mode,
        "fate_routing_free": c < 0.5,
        "fate_ingress_routing": ingress,
        "incubation_cycles": 1 + int((1.0 - c) * 14),  # 1 .. 15
    }


def quad_balance(hub_routed: list[int]) -> dict:
    """How evenly the 4 quads share Hub ingress."""
    names = ["BlackHole", "MeshBrain", "MultiNova", "BaseForce"]
    total = sum(hub_routed) or 1
    shares = [r / total for r in hub_routed]
    active = sum(1 for s in shares if s >= 0.05)
    max_s = max(shares)
    min_s = min(shares)
    return {
        "hub_routed": dict(zip(names, hub_routed)),
        "shares": {n: round(s, 4) for n, s in zip(names, shares)},
        "active_quads": active,
        "share_spread": round(max_s - min_s, 4),
        "balance_score": round(1.0 - (max_s - min_s), 4),  # 1.0 = perfectly even
    }


def append_for_round(thread_id: str, aligned: str, constraint: float, r: int) -> list[str]:
    if constraint > 0.55:
        return [aligned]
    if constraint > 0.25:
        ctx = TOPIC_CONTEXT[thread_id]
        return [ctx[r % len(ctx)], aligned]
    ctx = TOPIC_CONTEXT[thread_id]
    return [ctx[r % len(ctx)]]


def prefs_entropy(prefs: list[float]) -> float:
    if not prefs:
        return 0.0
    s = sum(prefs)
    if s <= 0:
        return 0.0
    ent = 0.0
    for p in prefs:
        q = p / s
        if q > 1e-12:
            ent -= q * math.log(q)
    return ent


def run_thread(thread_id: str, phase1: int, phase2: int, log_every: int, ingress_routing: str) -> dict:
    templates = EXTENDED_TURNS[thread_id]
    total = phase1 + phase2
    base = ValhallaBase(decode="hybrid", follow_up_decode="native_follow_up_aware")
    base.set_context(SEEDS[thread_id])

    turns: list[dict] = []
    phase_stats = {
        "phase1": {"hits": 0, "n": 0, "spread_sum": 0.0},
        "phase2": {"hits": 0, "n": 0, "spread_sum": 0.0},
    }
    spread_series: list[dict] = []

    for r in range(total):
        c = constraint_for_round(r, phase1, phase2)
        fp = fate_params(c, ingress_routing)
        base.fate_weight_mode = fp["fate_weight_mode"]
        base.fate_routing_free = fp["fate_routing_free"]
        base.fate_ingress_routing = fp["fate_ingress_routing"]
        base.incubation_cycles = fp["incubation_cycles"]

        aligned, question, hint, forbidden = templates[r % len(templates)]
        for line in append_for_round(thread_id, aligned, c, r):
            base.append_context(line)

        doc = base.run(
            [{"id": f"fate_{thread_id}_{r}", "text": question, "score_type": "open"}],
            append_only=base.session.session_state is not None,
        )
        tel = doc.get("fate_telemetry") or {}
        prefs = tel.get("hub_prefs") or (doc.get("session") or {}).get("snapshot", {}).get("hub_prefs") or []
        spread = float(tel.get("hub_prefs_spread") or 0.0)
        routed = tel.get("hub_routed") or [0, 0, 0, 0]
        qbal = quad_balance(routed)
        ans = doc["rows"][0]["answer"] if doc.get("rows") else ""
        ok = topic_ok_strict(ans, hint, forbidden)

        phase_key = "phase1" if r < phase1 else "phase2"
        phase_stats[phase_key]["hits"] += int(ok)
        phase_stats[phase_key]["n"] += 1
        phase_stats[phase_key]["spread_sum"] += spread

        row = {
            "round": r,
            "phase": phase_key,
            "constraint": round(c, 4),
            "fate_weight_mode": fp["fate_weight_mode"],
            "fate_routing_free": fp["fate_routing_free"],
            "fate_ingress_routing": fp["fate_ingress_routing"],
            "incubation_cycles": fp["incubation_cycles"],
            "hub_prefs": [round(x, 5) for x in prefs],
            "hub_prefs_spread": round(spread, 5),
            "hub_prefs_entropy": round(prefs_entropy(prefs), 5),
            "hub_routed": routed,
            "quad_balance": qbal,
            "on_topic": ok,
            "context_lines": doc.get("context_lines"),
        }
        if r % log_every == 0 or r == total - 1:
            row["answer_sample"] = ans[:120]
        turns.append(row)
        if r % log_every == 0:
            print(
                f"  r{r:3d} [{phase_key}] c={c:.2f} mode={fp['fate_weight_mode']:9s} "
                f"ingress={fp['fate_ingress_routing']:10s} cyc={fp['incubation_cycles']:2d} "
                f"spread={spread:.3f} quads={qbal['active_quads']}/4 bal={qbal['balance_score']:.2f} ok={ok}",
                flush=True,
            )

    for pk in phase_stats:
        n = phase_stats[pk]["n"]
        phase_stats[pk]["rate"] = phase_stats[pk]["hits"] / max(n, 1)
        phase_stats[pk]["avg_spread"] = phase_stats[pk]["spread_sum"] / max(n, 1)
        del phase_stats[pk]["spread_sum"]

    hits = sum(t["on_topic"] for t in turns)
    final_routed = turns[-1]["hub_routed"] if turns else [0, 0, 0, 0]
    return {
        "thread_id": thread_id,
        "ingress_routing": ingress_routing,
        "total_rounds": total,
        "on_topic_hits": hits,
        "on_topic_rate": hits / total,
        "phase_stats": phase_stats,
        "final_hub_prefs": turns[-1]["hub_prefs"] if turns else [],
        "final_spread": turns[-1]["hub_prefs_spread"] if turns else 0.0,
        "final_quad_balance": quad_balance(final_routed),
        "turns": turns if total <= 80 else turns[:: max(1, total // 80)] + [turns[-1]],
    }


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--phase1", type=int, default=15)
    ap.add_argument("--phase2", type=int, default=15)
    ap.add_argument(
        "--ingress-routing",
        choices=["quad_cycle", "semantic", "round_robin", "hybrid_quad_semantic"],
        default="quad_cycle",
        help="Phase1 ingress mode while fate_routing_free=false",
    )
    ap.add_argument(
        "--compare-all",
        action="store_true",
        help="Run quad_cycle, semantic, hybrid_quad_semantic and print comparison table",
    )
    ap.add_argument("--threads", nargs="+", default=["physics", "chemistry", "australia"])
    ap.add_argument("--log-every", type=int, default=5)
    ap.add_argument("--out", type=str, default="")
    args = ap.parse_args()

    modes = ["quad_cycle", "semantic", "hybrid_quad_semantic"] if args.compare_all else [args.ingress_routing]
    all_summaries = []

    for mode in modes:
        print(
            f"\n{'='*60}\n"
            f"=== Fate weight ladder {args.phase1}+{args.phase2} ingress={mode} ===\n"
        )
        results = []
        for tid in args.threads:
            print(f"\n--- thread {tid} ---")
            results.append(
                run_thread(tid, args.phase1, args.phase2, args.log_every, mode)
            )

        agg_p1 = {"hits": 0, "n": 0, "spread": 0.0}
        agg_p2 = {"hits": 0, "n": 0, "spread": 0.0}
        for r in results:
            for pk, agg in [("phase1", agg_p1), ("phase2", agg_p2)]:
                st = r["phase_stats"][pk]
                agg["hits"] += st["hits"]
                agg["n"] += st["n"]
                agg["spread"] += st["avg_spread"] * st["n"]

        summary = {
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "protocol": "fate-weight-ladder-v1",
            "ingress_routing": mode,
            "phase1_rounds": args.phase1,
            "phase2_rounds": args.phase2,
            "total_rounds_per_thread": args.phase1 + args.phase2,
            "description": (
                f"Phase1 high constraint + {mode} → "
                "Phase2 evaluated weights + routing free"
            ),
            "threads": results,
            "aggregate": {
                "phase1": {
                    "hits": agg_p1["hits"],
                    "n": agg_p1["n"],
                    "rate": agg_p1["hits"] / max(agg_p1["n"], 1),
                    "avg_hub_prefs_spread": agg_p1["spread"] / max(agg_p1["n"], 1),
                },
                "phase2": {
                    "hits": agg_p2["hits"],
                    "n": agg_p2["n"],
                    "rate": agg_p2["hits"] / max(agg_p2["n"], 1),
                    "avg_hub_prefs_spread": agg_p2["spread"] / max(agg_p2["n"], 1),
                },
                "spread_delta_phase2_minus_phase1": (
                    agg_p2["spread"] / max(agg_p2["n"], 1) - agg_p1["spread"] / max(agg_p1["n"], 1)
                ),
                "accuracy_delta_phase2_minus_phase1": (
                    agg_p2["hits"] / max(agg_p2["n"], 1) - agg_p1["hits"] / max(agg_p1["n"], 1)
                ),
            },
        }
        all_summaries.append(summary)

        print("\n=== Aggregate ===")
        a = summary["aggregate"]
        print(
            f"  Phase1: {a['phase1']['hits']}/{a['phase1']['n']} "
            f"({100*a['phase1']['rate']:.1f}%) spread={a['phase1']['avg_hub_prefs_spread']:.4f}"
        )
        print(
            f"  Phase2: {a['phase2']['hits']}/{a['phase2']['n']} "
            f"({100*a['phase2']['rate']:.1f}%) spread={a['phase2']['avg_hub_prefs_spread']:.4f}"
        )
        print(f"  Δ spread (p2-p1): {a['spread_delta_phase2_minus_phase1']:+.4f}")
        print(f"  Δ accuracy (p2-p1): {100*a['accuracy_delta_phase2_minus_phase1']:+.1f}pp")
        for r in results:
            qb = r["final_quad_balance"]
            print(
                f"  {r['thread_id']}: quads={qb['active_quads']}/4 "
                f"balance={qb['balance_score']:.2f} spread={r['final_spread']:.3f} "
                f"shares={qb['shares']}"
            )

        REPORTS.mkdir(parents=True, exist_ok=True)
        out_name = args.out if args.out and len(modes) == 1 else f"fate_weight_ladder_{mode}_{args.phase1 + args.phase2}.json"
        out = REPORTS / out_name
        out.write_text(json.dumps(summary, indent=2, ensure_ascii=False), encoding="utf-8")
        print(f"\n→ {out}")

    if len(all_summaries) > 1:
        print(f"\n{'='*60}\n=== COMPARISON ===")
        print(f"{'mode':<22} {'P1%':>6} {'P2%':>6} {'P1 spread':>10} {'P2 spread':>10} {'avg bal':>8}")
        for s in all_summaries:
            a = s["aggregate"]
            avg_bal = sum(t["final_quad_balance"]["balance_score"] for t in s["threads"]) / len(s["threads"])
            print(
                f"{s['ingress_routing']:<22} "
                f"{100*a['phase1']['rate']:>5.1f}% "
                f"{100*a['phase2']['rate']:>5.1f}% "
                f"{a['phase1']['avg_hub_prefs_spread']:>10.4f} "
                f"{a['phase2']['avg_hub_prefs_spread']:>10.4f} "
                f"{avg_bal:>8.2f}"
            )
        cmp_out = REPORTS / f"fate_weight_ladder_compare_{args.phase1 + args.phase2}.json"
        cmp_out.write_text(json.dumps(all_summaries, indent=2, ensure_ascii=False), encoding="utf-8")
        print(f"\n→ {cmp_out}")


if __name__ == "__main__":
    main()
