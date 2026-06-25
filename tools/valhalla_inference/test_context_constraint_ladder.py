#!/usr/bin/env python3
"""Constraint ladder test: high → medium → loose → none, more rounds per session.

Within each topic thread, constraint relaxes every `rounds_per_tier` turns:
  strict   — append aligns with answer; native decode
  medium   — topic-related append (no answer keywords); follow-up-aware hybrid
  loose    — aligned append + cross-topic distractor
  none     — distractor-only append; minimal retrieval support

Usage:
  python3 tools/valhalla_inference/test_context_constraint_ladder.py
  python3 tools/valhalla_inference/test_context_constraint_ladder.py --rounds-per-tier 6
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

from test_context_arch_compare import resolve_lm_path  # noqa: E402
from test_context_flexibility_goal import EXTENDED_TURNS, SEEDS  # noqa: E402
from test_context_performance import context_block  # noqa: E402
from valhalla_base import ValhallaBase  # noqa: E402

REPORTS = REPO / "reports/valhalla_inference"

# Cross-topic noise injected at loose / none tiers.
DISTRACTORS = {
    "france": [
        "The capital of France is Paris.",
        "The Eiffel Tower is located in Paris, France.",
        "French is the official language of France.",
    ],
    "water_misc": [
        "Coffee is typically served hot near 85 degrees Celsius.",
        "The chemical formula for table salt is NaCl.",
    ],
    "space": [
        "Mercury is the closest planet to the Sun.",
        "The Pacific Ocean is the largest ocean on Earth.",
    ],
    "biology": [
        "Photosynthesis occurs in chloroplasts using light energy.",
        "DNA stores genetic information in living cells.",
    ],
}

# Topic-adjacent lines without answer keywords (medium tier).
TOPIC_CONTEXT = {
    "australia": [
        "Australia is a large island nation in the southern hemisphere.",
        "Many unique species live on the Australian continent.",
        "Australian cities are spread along the coastline.",
        "The interior of Australia is mostly arid.",
        "Australia has six states and two major mainland territories.",
        "Indigenous cultures have lived in Australia for millennia.",
        "Australia participates in international sporting events.",
        "Regional climates vary across Australia.",
        "Transport between Australian cities often uses domestic flights.",
        "Australia maintains diplomatic relations worldwide.",
    ],
    "chemistry": [
        "Liquid water is essential for most biochemical reactions.",
        "Phase changes involve energy transfer.",
        "Solutions can alter boiling and freezing behavior.",
        "Molecular bonds hold atoms together in compounds.",
        "Laboratory glassware is used to heat liquids carefully.",
        "Pure substances have characteristic physical properties.",
        "Chemical reactions may absorb or release heat.",
        "Solvents dissolve solutes at varying rates.",
        "Pressure affects phase equilibrium.",
        "Water participates in many industrial processes.",
    ],
    "physics": [
        "Physical quantities are measured with standard SI units.",
        "Forces can change the motion of objects.",
        "Energy appears in multiple interchangeable forms.",
        "Vectors have both magnitude and direction.",
        "Friction converts kinetic energy to heat.",
        "Gravity acts on all masses near Earth.",
        "Experiments verify relationships between physical variables.",
        "Motion can be described relative to a reference frame.",
        "Work and energy share related units in mechanics.",
        "Collisions transfer momentum between objects.",
    ],
}


@dataclass
class ConstraintTier:
    id: str
    append_mode: str  # aligned | topic_only | aligned_plus_distractor | distractor_only
    decode: str
    follow_up_decode: str
    score_mode: str  # strict | relaxed


TIERS: list[ConstraintTier] = [
    ConstraintTier("strict", "aligned", "native", "inherit", "strict"),
    ConstraintTier("medium", "topic_only", "hybrid", "native_follow_up_aware", "strict"),
    ConstraintTier("loose", "aligned_plus_distractor", "hybrid", "native_follow_up_aware", "strict"),
    ConstraintTier("none", "distractor_only", "hybrid", "native_follow_up_aware", "relaxed"),
]


def norm(s: str) -> str:
    return re.sub(r"\s+", " ", s.lower().strip())


def topic_ok_strict(answer: str, hint: str, forbidden: list[str]) -> bool:
    low = norm(answer)
    if hint.lower() not in low:
        return False
    for f in forbidden:
        f = f.strip().lower()
        if f and re.search(rf"\b{re.escape(f)}\b", low):
            return False
    return True


def topic_ok_relaxed(answer: str, hint: str, forbidden: list[str]) -> bool:
    low = norm(answer)
    if hint.lower() in low:
        return True
    # Accept corpus citation when retrieval path used under no-constraint tier.
    if "[corpus:" in low and any(w in low for w in hint.lower().split() if len(w) >= 3):
        return True
    return not any(f.strip().lower() in low for f in forbidden if f.strip())


def tier_for_round(round_idx: int, rounds_per_tier: int) -> ConstraintTier:
    tier_idx = min(round_idx // rounds_per_tier, len(TIERS) - 1)
    return TIERS[tier_idx]


def distractor_lines(tier_id: str, round_idx: int) -> list[str]:
    pools = list(DISTRACTORS.values())
    a = pools[round_idx % len(pools)]
    b = pools[(round_idx + 1) % len(pools)]
    return [a[round_idx % len(a)], b[round_idx % len(b)]]


def build_appends(
    thread_id: str,
    aligned: str,
    tier: ConstraintTier,
    round_idx: int,
) -> list[str]:
    ctx = TOPIC_CONTEXT[thread_id]
    if tier.append_mode == "aligned":
        return [aligned]
    if tier.append_mode == "topic_only":
        return [ctx[round_idx % len(ctx)]]
    if tier.append_mode == "aligned_plus_distractor":
        return [aligned] + distractor_lines(tier.id, round_idx)
    # distractor_only
    return distractor_lines(tier.id, round_idx)


def make_base(tier: ConstraintTier) -> ValhallaBase:
    return ValhallaBase(decode=tier.decode, follow_up_decode=tier.follow_up_decode)  # type: ignore[arg-type]


def run_ladder_thread(
    thread_id: str,
    rounds_per_tier: int,
    arch: str,
    lm=None,
) -> dict:
    templates = EXTENDED_TURNS[thread_id]
    total_rounds = rounds_per_tier * len(TIERS)

    if arch == "traditional_lm":
        corpus = list(SEEDS[thread_id])
        turns: list[dict] = []
        tier_stats: dict[str, dict] = {t.id: {"hits": 0, "n": 0} for t in TIERS}

        for r in range(total_rounds):
            tier = tier_for_round(r, rounds_per_tier)
            aligned, question, hint, forbidden = templates[r % len(templates)]
            for line in build_appends(thread_id, aligned, tier, r):
                corpus.append(line)
            ans = lm.generate([context_block(corpus) + question], max_tokens=128)[0].text.strip()
            ok_fn = topic_ok_strict if tier.score_mode == "strict" else topic_ok_relaxed
            ok = ok_fn(ans, hint, forbidden)
            tier_stats[tier.id]["hits"] += int(ok)
            tier_stats[tier.id]["n"] += 1
            turns.append(
                {
                    "round": r,
                    "tier": tier.id,
                    "question": question,
                    "appends": build_appends(thread_id, aligned, tier, r),
                    "answer": ans[:200],
                    "on_topic": ok,
                    "context_lines": len(corpus),
                }
            )
        hits = sum(t["on_topic"] for t in turns)
        return {
            "thread_id": thread_id,
            "architecture": arch,
            "total_rounds": total_rounds,
            "on_topic_hits": hits,
            "on_topic_rate": hits / total_rounds,
            "by_tier": {
                tid: {
                    "hits": tier_stats[tid]["hits"],
                    "n": tier_stats[tid]["n"],
                    "rate": tier_stats[tid]["hits"] / max(tier_stats[tid]["n"], 1),
                }
                for tid in tier_stats
            },
            "turns": turns,
        }

    # ValhallaBase path — re-create session per thread; tier may change decode each round
    base = ValhallaBase()
    base.set_context(SEEDS[thread_id])
    turns = []
    tier_stats: dict[str, dict] = {t.id: {"hits": 0, "n": 0} for t in TIERS}
    hits = 0

    for r in range(total_rounds):
        tier = tier_for_round(r, rounds_per_tier)
        base.decode = tier.decode  # type: ignore[assignment]
        base.follow_up_decode = tier.follow_up_decode  # type: ignore[assignment]
        aligned, question, hint, forbidden = templates[r % len(templates)]
        append_lines = build_appends(thread_id, aligned, tier, r)
        for line in append_lines:
            base.append_context(line)
        doc = base.run(
            [{"id": f"{thread_id}_r{r}", "text": question, "score_type": "open"}],
            append_only=base.session.session_state is not None,
        )
        ans = doc["rows"][0]["answer"] if doc.get("rows") else ""
        path = doc["rows"][0].get("decode_path", "") if doc.get("rows") else ""
        ok_fn = topic_ok_strict if tier.score_mode == "strict" else topic_ok_relaxed
        ok = ok_fn(ans, hint, forbidden)
        hits += int(ok)
        tier_stats[tier.id]["hits"] += int(ok)
        tier_stats[tier.id]["n"] += 1
        corpus = (doc.get("session") or {}).get("corpus") or []
        turns.append(
            {
                "round": r,
                "tier": tier.id,
                "append_mode": tier.append_mode,
                "question": question,
                "appends": append_lines,
                "answer": ans[:200],
                "decode_path": path,
                "on_topic": ok,
                "context_lines": len(corpus),
            }
        )

    total = len(turns)
    return {
        "thread_id": thread_id,
        "architecture": arch,
        "total_rounds": total,
        "on_topic_hits": hits,
        "on_topic_rate": hits / max(total, 1),
        "by_tier": {
            tid: {
                "hits": tier_stats[tid]["hits"],
                "n": tier_stats[tid]["n"],
                "rate": tier_stats[tid]["hits"] / max(tier_stats[tid]["n"], 1),
            }
            for tid in tier_stats
        },
        "turns": turns,
    }


def aggregate_by_tier(threads: list[dict]) -> dict[str, dict]:
    out: dict[str, dict] = {t.id: {"hits": 0, "n": 0} for t in TIERS}
    for th in threads:
        for tid, st in th["by_tier"].items():
            out[tid]["hits"] += st["hits"]
            out[tid]["n"] += st["n"]
    for tid in out:
        n = out[tid]["n"]
        out[tid]["rate"] = out[tid]["hits"] / max(n, 1)
    return out


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--rounds-per-tier", type=int, default=5, help="Rounds at each constraint level")
    ap.add_argument("--lm", default="Qwen/Qwen2.5-0.5B-Instruct")
    ap.add_argument(
        "--arch",
        nargs="+",
        default=["follow_up_aware", "native_strict", "traditional_lm"],
        choices=["follow_up_aware", "native_strict", "traditional_lm"],
    )
    args = ap.parse_args()

    from loader import ValhallaLLM

    lm_path = resolve_lm_path(args.lm)
    lm = ValhallaLLM(lm_path, engine="transformers")

    total_rounds = args.rounds_per_tier * len(TIERS)
    print(f"=== Constraint ladder ({args.rounds_per_tier} rounds/tier × {len(TIERS)} tiers = {total_rounds} rounds/thread) ===\n")
    for t in TIERS:
        print(f"  {t.id:8s} append={t.append_mode:22s} decode={t.decode}+{t.follow_up_decode}")

    # Map arch names to ladder runner label (tier drives decode for aware path)
    arch_results: dict[str, dict] = {}

    for arch in args.arch:
        print(f"\n--- {arch} ---")
        threads = []
        for tid in EXTENDED_TURNS:
            if arch == "native_strict":
                # Force strict-tier decode for all rounds by overriding tier decode in runner
                # Use separate implementation: always native
                threads.append(run_ladder_thread_native_strict(tid, args.rounds_per_tier))
            elif arch == "follow_up_aware":
                threads.append(run_ladder_thread(tid, args.rounds_per_tier, "follow_up_aware"))
            else:
                threads.append(run_ladder_thread(tid, args.rounds_per_tier, "traditional_lm", lm=lm))

        agg_tier = aggregate_by_tier(threads)
        total_h = sum(t["on_topic_hits"] for t in threads)
        total_n = sum(t["total_rounds"] for t in threads)
        print(f"  overall {total_h}/{total_n} ({100*total_h/total_n:.1f}%)")
        for tier_id in [t.id for t in TIERS]:
            st = agg_tier[tier_id]
            print(f"    {tier_id:8s} {st['hits']}/{st['n']} ({100*st['rate']:.1f}%)")

        arch_results[arch] = {
            "overall": {"hits": total_h, "n": total_n, "rate": total_h / total_n},
            "by_tier": agg_tier,
            "threads": threads,
        }

    # Degradation: strict rate - none rate per architecture
    degradation = {}
    for arch, res in arch_results.items():
        s = res["by_tier"]["strict"]["rate"]
        n = res["by_tier"]["none"]["rate"]
        degradation[arch] = {"strict_to_none_delta": s - n, "strict": s, "none": n}

    summary = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "protocol": "context-constraint-ladder-v1",
        "rounds_per_tier": args.rounds_per_tier,
        "total_rounds_per_thread": total_rounds,
        "tiers": [
            {
                "id": t.id,
                "append_mode": t.append_mode,
                "decode": t.decode,
                "follow_up_decode": t.follow_up_decode,
                "score_mode": t.score_mode,
            }
            for t in TIERS
        ],
        "architectures": arch_results,
        "degradation_strict_to_none": degradation,
    }

    print("\n=== Degradation (strict → none rate drop) ===")
    for arch, d in degradation.items():
        print(f"  {arch}: strict {100*d['strict']:.1f}% → none {100*d['none']:.1f}% (Δ {100*d['strict_to_none_delta']:+.1f}pp)")

    REPORTS.mkdir(parents=True, exist_ok=True)
    out = REPORTS / "context_constraint_ladder.json"
    out.write_text(json.dumps(summary, indent=2, ensure_ascii=False), encoding="utf-8")
    print(f"\n→ {out}")


def run_ladder_thread_native_strict(thread_id: str, rounds_per_tier: int) -> dict:
    """All rounds use native decode (strict baseline across tiers)."""
    templates = EXTENDED_TURNS[thread_id]
    total_rounds = rounds_per_tier * len(TIERS)
    base = ValhallaBase(decode="native", follow_up_decode="inherit")
    base.set_context(SEEDS[thread_id])
    turns: list[dict] = []
    tier_stats: dict[str, dict] = {t.id: {"hits": 0, "n": 0} for t in TIERS}
    hits = 0

    for r in range(total_rounds):
        tier = tier_for_round(r, rounds_per_tier)
        aligned, question, hint, forbidden = templates[r % len(templates)]
        append_lines = build_appends(thread_id, aligned, tier, r)
        for line in append_lines:
            base.append_context(line)
        doc = base.run(
            [{"id": f"{thread_id}_r{r}", "text": question, "score_type": "open"}],
            append_only=base.session.session_state is not None,
        )
        ans = doc["rows"][0]["answer"] if doc.get("rows") else ""
        ok_fn = topic_ok_strict if tier.score_mode == "strict" else topic_ok_relaxed
        ok = ok_fn(ans, hint, forbidden)
        hits += int(ok)
        tier_stats[tier.id]["hits"] += int(ok)
        tier_stats[tier.id]["n"] += 1
        corpus = (doc.get("session") or {}).get("corpus") or []
        turns.append(
            {
                "round": r,
                "tier": tier.id,
                "question": question,
                "appends": append_lines,
                "answer": ans[:200],
                "on_topic": ok,
                "context_lines": len(corpus),
            }
        )

    return {
        "thread_id": thread_id,
        "architecture": "native_strict",
        "total_rounds": total_rounds,
        "on_topic_hits": hits,
        "on_topic_rate": hits / total_rounds,
        "by_tier": {
            tid: {
                "hits": tier_stats[tid]["hits"],
                "n": tier_stats[tid]["n"],
                "rate": tier_stats[tid]["hits"] / max(tier_stats[tid]["n"], 1),
            }
            for tid in tier_stats
        },
        "turns": turns,
    }


if __name__ == "__main__":
    main()
