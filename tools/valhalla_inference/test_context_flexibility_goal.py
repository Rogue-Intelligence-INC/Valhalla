#!/usr/bin/env python3
"""Context flexibility MAX GOAL benchmark — improvements 1/2/3.

1. Hybrid follow-up decode (native first turn → LM patch on follow-ups)
2. Follow-up-aware native retrieval (prior question ranking)
3. Extended topic threads (8–12 turns, growing corpus)

Compares against baseline native + traditional LM prompt injection.

Usage:
  python3 tools/valhalla_inference/test_context_flexibility_goal.py
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from copy import deepcopy
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

from test_context_arch_compare import resolve_lm_path  # noqa: E402
from test_context_flexibility_deep import (  # noqa: E402
    GROWTH_APPENDS,
    GROWTH_SEED,
    MULTI_Q_BATCHES,
    measure_growth_curve,
    run_multi_q_new,
    run_multi_q_trad,
)
from test_context_performance import context_block  # noqa: E402
from valhalla_base import ValhallaBase  # noqa: E402

REPORTS = REPO / "reports/valhalla_inference"
DOCS = REPO / "valhalla-unified-core/docs"

EXTENDED_TURNS = {
    "australia": [
        ("Sydney is the largest city in Australia but not the capital.", "What is the capital of Australia?", "canberra", ["paris", "france"]),
        ("English is the de facto national language of Australia.", "Which continent is Australia located in?", "oceania", ["europe", "africa"]),
        ("The Great Barrier Reef lies off the coast of Queensland, Australia.", "What is the largest city in Australia?", "sydney", ["canberra", "paris"]),
        ("Australia uses the Australian dollar as currency.", "What language is most widely spoken in Australia?", "english", ["french", "german"]),
        ("Melbourne is the second largest city in Australia.", "What is the second largest city in Australia?", "melbourne", ["sydney", "paris"]),
        ("The Australian flag features the Union Jack and Southern Cross.", "What stars appear on the Australian flag?", "southern cross", ["north star", "orion"]),
        ("Uluru is a large sandstone monolith in central Australia.", "What famous rock formation is in central Australia?", "uluru", ["everest", "eiffel"]),
        ("Kangaroos are native marsupials found across Australia.", "What native marsupial is mentioned for Australia?", "kangaroo", ["panda", "lion"]),
        ("The Outback refers to remote arid interior regions of Australia.", "What term describes remote arid interior of Australia?", "outback", ["tundra", "savanna"]),
        ("Canberra was selected as capital as a compromise between Sydney and Melbourne.", "Why was Canberra chosen as capital?", "compromise", ["random", "paris"]),
    ],
    "chemistry": [
        ("Steam forms when liquid water reaches its boiling point.", "At what temperature does water boil at standard pressure?", "100", ["freez"]),
        ("Ice melts at 0 degrees Celsius when heated.", "At what temperature does water freeze?", "0", ["100", "boil"]),
        ("The chemical formula for water is H2O.", "What is the chemical formula for water?", "h2o", ["co2", "nacl"]),
        ("Salt water boils at a slightly higher temperature than pure water.", "Does salt water boil at same temperature as pure water?", "higher", ["lower", "same"]),
        ("Evaporation turns liquid water into water vapor.", "What process turns liquid water into vapor?", "evaporation", ["condensation", "freezing"]),
        ("Condensation forms clouds when water vapor cools.", "What process forms clouds from vapor?", "condensation", ["evaporation", "combustion"]),
        ("Hydrogen and oxygen combine to form water.", "Which elements combine to form water?", "hydrogen", ["carbon", "iron"]),
        ("Water expands when it freezes, unlike most liquids.", "Does water expand or contract when it freezes?", "expand", ["contract", "unchanged"]),
        ("Distillation separates water from dissolved salts by boiling.", "What process separates water from dissolved salts?", "distillation", ["filtration", "photosynthesis"]),
        ("pH 7 is neutral for pure water at 25C.", "What pH is neutral for pure water?", "7", ["0", "14"]),
    ],
    "physics": [
        ("One Newton equals one kilogram meter per second squared.", "What is the SI unit of force?", "newton", ["joule", "watt"]),
        ("Mass is measured in kilograms in the SI system.", "In F=ma, what does m represent?", "mass", ["meter", "momentum"]),
        ("Acceleration is the rate of change of velocity.", "In F=ma, what does a represent?", "acceleration", ["area", "ampere"]),
        ("Weight is force due to gravity on a mass.", "What is weight in terms of force and gravity?", "gravity", ["magnetism", "friction"]),
        ("Velocity is speed with direction.", "What quantity combines speed with direction?", "velocity", ["mass", "temperature"]),
        ("Kinetic energy is one half m v squared.", "What energy depends on mass and velocity squared?", "kinetic", ["potential", "thermal"]),
        ("Friction opposes relative motion between surfaces.", "What force opposes relative motion between surfaces?", "friction", ["buoyancy", "lift"]),
        ("Gravity on Earth is approximately 9.8 meters per second squared.", "What is approximate gravity on Earth in m/s²?", "9.8", ["1.6", "100"]),
        ("Momentum equals mass times velocity.", "What equals mass times velocity?", "momentum", ["energy", "power"]),
        ("Work is force times distance in the direction of force.", "What is force times distance in direction of force?", "work", ["power", "pressure"]),
    ],
}

SEEDS = {
    "australia": [
        "The capital of Australia is Canberra.",
        "Australia is a country in Oceania.",
    ],
    "chemistry": [
        "Water boils at 100 degrees Celsius at standard atmospheric pressure.",
        "Water freezes at 0 degrees Celsius at standard pressure.",
    ],
    "physics": [
        "The SI unit of force is the Newton.",
        "Newton's second law states F equals m times a.",
    ],
}


def norm(s: str) -> str:
    return re.sub(r"\s+", " ", s.lower().strip())


def topic_ok(answer: str, hint: str, forbidden: list[str]) -> bool:
    low = norm(answer)
    if hint.lower() not in low:
        return False
    for f in forbidden:
        f = f.strip().lower()
        if not f:
            continue
        if re.search(rf"\b{re.escape(f)}\b", low):
            return False
    return True


def make_base(mode: str) -> ValhallaBase:
    b = ValhallaBase(decode="hybrid")
    if mode == "native_baseline":
        b.decode = "native"
        b.follow_up_decode = "inherit"
    elif mode == "follow_up_aware":
        b.decode = "hybrid"
        b.follow_up_decode = "native_follow_up_aware"
    elif mode == "hybrid_follow_up_lm":
        b.decode = "hybrid"
        b.follow_up_decode = "lm_patch_on_follow_up"
    else:
        raise ValueError(mode)
    return b


def run_extended_thread(base: ValhallaBase, thread_id: str) -> dict:
    base.clear_context()
    base.set_context(SEEDS[thread_id])
    turns_out: list[dict] = []
    hits = 0
    max_chars = 0

    for i, (append, question, hint, forbidden) in enumerate(EXTENDED_TURNS[thread_id]):
        base.append_context(append)
        doc = base.run(
            [{"id": f"{thread_id}_{i}", "text": question, "score_type": "open"}],
            append_only=base.session.session_state is not None,
        )
        ans = doc["rows"][0]["answer"] if doc.get("rows") else ""
        path = doc["rows"][0].get("decode_path", "") if doc.get("rows") else ""
        ok = topic_ok(ans, hint, forbidden)
        hits += int(ok)
        corpus = (doc.get("session") or {}).get("corpus") or []
        chars = sum(len(l) for l in corpus)
        max_chars = max(max_chars, chars)
        turns_out.append(
            {
                "turn": i,
                "question": question,
                "answer": ans[:180],
                "on_topic": ok,
                "decode_path": path,
                "context_lines": len(corpus),
                "context_chars": chars,
            }
        )

    n = len(EXTENDED_TURNS[thread_id])
    return {
        "thread_id": thread_id,
        "turns": n,
        "on_topic_hits": hits,
        "on_topic_rate": hits / n,
        "final_context_lines": turns_out[-1]["context_lines"] if turns_out else 0,
        "final_context_chars": max_chars,
        "turn_details": turns_out,
    }


def run_extended_trad(lm, thread_id: str) -> dict:
    corpus = list(SEEDS[thread_id])
    turns_out: list[dict] = []
    hits = 0
    max_chars = 0

    for i, (append, question, hint, forbidden) in enumerate(EXTENDED_TURNS[thread_id]):
        corpus.append(append)
        q = context_block(corpus) + question
        ans = lm.generate([q], max_tokens=128)[0].text.strip()
        ok = topic_ok(ans, hint, forbidden)
        hits += int(ok)
        chars = sum(len(l) for l in corpus)
        max_chars = max(max_chars, chars)
        turns_out.append(
            {
                "turn": i,
                "question": question,
                "answer": ans[:180],
                "on_topic": ok,
                "context_lines": len(corpus),
                "context_chars": chars,
                "prompt_chars": len(q),
            }
        )

    n = len(EXTENDED_TURNS[thread_id])
    return {
        "thread_id": thread_id,
        "turns": n,
        "on_topic_hits": hits,
        "on_topic_rate": hits / n,
        "final_context_lines": len(corpus),
        "final_context_chars": max_chars,
        "turn_details": turns_out,
    }


def aggregate_threads(threads: list[dict]) -> dict:
    hits = sum(t["on_topic_hits"] for t in threads)
    n = sum(t["turns"] for t in threads)
    return {
        "total_on_topic": hits,
        "total_turns": n,
        "rate": hits / max(n, 1),
        "threads": threads,
    }


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--lm", default="Qwen/Qwen2.5-0.5B-Instruct")
    ap.add_argument("--growth-appends", type=int, default=12)
    args = ap.parse_args()

    from loader import ValhallaLLM

    lm_path = resolve_lm_path(args.lm)
    lm = ValhallaLLM(lm_path, engine="transformers")

    print("=== Context Flexibility MAX GOAL ===\n")

    print("A. Context growth (session append)")
    growth = measure_growth_curve(ValhallaBase(decode="native"), args.growth_appends)
    print(f"  {growth['final_context_lines']} lines / {growth['final_context_chars']} chars")
    print(f"  incremental ~{growth['new_arch_avg_incremental_payload_chars']} chars/step vs trad {growth['trad_final_full_prompt_chars']} chars\n")

    modes = ["native_baseline", "follow_up_aware", "hybrid_follow_up_lm"]
    extended: dict[str, dict] = {}
    print("B. Extended topic threads (10 turns × 3 topics)")
    for mode in modes:
        threads = [run_extended_thread(make_base(mode), tid) for tid in EXTENDED_TURNS]
        agg = aggregate_threads(threads)
        extended[mode] = agg
        print(f"  {mode:22s} {agg['total_on_topic']}/{agg['total_turns']} ({100*agg['rate']:.1f}%)")

    trad_ext = aggregate_threads([run_extended_trad(lm, tid) for tid in EXTENDED_TURNS])
    print(f"  {'traditional_lm':22s} {trad_ext['total_on_topic']}/{trad_ext['total_turns']} ({100*trad_ext['rate']:.1f}%)\n")

    print("C. Multi-question disambiguation")
    multi: dict[str, dict] = {}
    for mode in modes:
        batches = [run_multi_q_new(make_base(mode), b) for b in MULTI_Q_BATCHES]
        hits = sum(b["disambiguation_hits"] for b in batches)
        n = sum(b["n_questions"] for b in batches)
        multi[mode] = {"hits": hits, "n": n, "rate": hits / max(n, 1), "batches": batches}
        print(f"  {mode:22s} {hits}/{n}")

    trad_multi_batches = [run_multi_q_trad(lm, b) for b in MULTI_Q_BATCHES]
    trad_multi_hits = sum(b["disambiguation_hits"] for b in trad_multi_batches)
    trad_multi_n = sum(b["n_questions"] for b in trad_multi_batches)
    print(f"  {'traditional_lm':22s} {trad_multi_hits}/{trad_multi_n}\n")

    base_rate = extended["native_baseline"]["rate"]
    aware_rate = extended["follow_up_aware"]["rate"]
    hybrid_rate = extended["hybrid_follow_up_lm"]["rate"]
    trad_rate = trad_ext["rate"]

    improvements = {
        "follow_up_aware_vs_baseline": aware_rate - base_rate,
        "hybrid_follow_up_vs_baseline": hybrid_rate - base_rate,
        "hybrid_follow_up_vs_traditional": hybrid_rate - trad_rate,
        "follow_up_aware_vs_traditional": aware_rate - trad_rate,
    }

    if hybrid_rate >= trad_rate and hybrid_rate >= base_rate:
        verdict = "GOAL_ACHIEVED_HYBRID_FOLLOW_UP"
    elif aware_rate > base_rate + 0.05:
        verdict = "PARTIAL_FOLLOW_UP_AWARE_WINS"
    elif hybrid_rate > base_rate + 0.05:
        verdict = "PARTIAL_HYBRID_WINS"
    else:
        verdict = "GOAL_IN_PROGRESS"

    summary = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "protocol": "context-flexibility-max-goal-v1",
        "max_goal": "flexible context + topic continuity + multi-Q disambiguation",
        "improvements": {
            "1_hybrid_follow_up_lm": "decode=hybrid + follow_up_decode=lm_patch_on_follow_up",
            "2_follow_up_aware_retrieval": "decode=hybrid + follow_up_decode=native_follow_up_aware",
            "3_extended_threads": "10 turns × 3 topics with session append",
        },
        "growth_curve": growth,
        "extended_topic_continuity": {
            "native_baseline": extended["native_baseline"],
            "follow_up_aware": extended["follow_up_aware"],
            "hybrid_follow_up_lm": extended["hybrid_follow_up_lm"],
            "traditional_lm": trad_ext,
        },
        "multi_question_disambiguation": {
            **{m: multi[m] for m in modes},
            "traditional_lm": {
                "hits": trad_multi_hits,
                "n": trad_multi_n,
                "rate": trad_multi_hits / max(trad_multi_n, 1),
                "batches": trad_multi_batches,
            },
        },
        "improvement_deltas": improvements,
        "verdict": verdict,
    }

    print("Deltas vs baseline native:")
    for k, v in improvements.items():
        print(f"  {k}: {v:+.1%}")
    print(f"\nVerdict: {verdict}")

    REPORTS.mkdir(parents=True, exist_ok=True)
    out = REPORTS / "context_flexibility_max_goal.json"
    out.write_text(json.dumps(summary, indent=2, ensure_ascii=False), encoding="utf-8")
    print(f"→ {out}")

    DOCS.mkdir(parents=True, exist_ok=True)
    write_doc(summary, DOCS / "CONTEXT_FLEXIBILITY_MAX_GOAL.md")


def write_doc(summary: dict, path: Path) -> None:
    g = summary["growth_curve"]
    ext = summary["extended_topic_continuity"]
    imp = summary["improvement_deltas"]

    def row(name: str) -> str:
        d = ext[name]
        return f"| {name} | {d['total_on_topic']}/{d['total_turns']} | {100*d['rate']:.1f}% |"

    body = f"""# Context Flexibility — Maximum Goal

> **Status:** {summary['verdict']}  
> **Protocol:** `{summary['protocol']}`  
> **Report:** `reports/valhalla_inference/context_flexibility_max_goal.json`

## Goal

One of Valhalla Unified Core's **maximum product goals**:

1. **Flexible context** — session append without re-sending full corpus; internal context grows while wire payload stays small.
2. **Topic continuity** — consecutive questions on the same thread stay on-topic as corpus grows.
3. **Multi-question disambiguation** — batch questions on mixed topics each get the correct sub-answer.

## Three Improvements (v1)

| # | Mechanism | Config |
|---|-----------|--------|
| 1 | **Hybrid follow-up LM** | `decode=hybrid`, `follow_up_decode=lm_patch_on_follow_up` — first turn native retrieval; follow-ups use structure→LM patch on incubated snapshot |
| 2 | **Follow-up-aware retrieval** | `follow_up_decode=native_follow_up_aware` — rank corpus with prior question history (boost current-Q keywords, penalize prior-only hits, recency bias) |
| 3 | **Extended threads** | 10-turn × 3-topic drill with incremental session append |

## A. Context Growth

| Metric | Value |
|--------|-------|
| Seed lines | {g['seed_lines']} |
| After {g['append_steps']} appends | **{g['final_context_lines']} lines / {g['final_context_chars']} chars** |
| Incremental payload (avg) | **~{g['new_arch_avg_incremental_payload_chars']} chars/step** |
| Traditional full prompt | {g['trad_final_full_prompt_chars']} chars ({g['trad_growth_ratio']}× initial) |

**Takeaway:** High flexibility → internal context scales linearly; transport cost stays ~flat per append.

## B. Extended Topic Continuity (10 turns × 3 topics = 30)

| Architecture | On-topic | Rate |
|--------------|----------|------|
{row('native_baseline')}
{row('follow_up_aware')}
{row('hybrid_follow_up_lm')}
{row('traditional_lm')}

### Improvement deltas

| Comparison | Δ rate |
|------------|--------|
| follow_up_aware vs baseline | {imp['follow_up_aware_vs_baseline']:+.1%} |
| hybrid_follow_up vs baseline | {imp['hybrid_follow_up_vs_baseline']:+.1%} |
| hybrid_follow_up vs traditional | {imp['hybrid_follow_up_vs_traditional']:+.1%} |
| follow_up_aware vs traditional | {imp['follow_up_aware_vs_traditional']:+.1%} |

## C. Multi-Question Disambiguation

See JSON report for per-batch details. All architectures tested on 9 mixed-topic questions (3 batches).

## Architecture Notes

- **Native baseline** excels at single-shot retrieval and corpus swap sensitivity.
- **Follow-up-aware retrieval** adds session `question_history` + `follow_up_retrieval_boost()` in `native_qa.rs`.
- **Hybrid follow-up LM** routes open follow-ups to `run_lm_one` with pre-incubated `TriadIncubationSnapshot` (structure substrate, not raw prompt RAG).
- **Traditional LM** remains strong on conversational follow-ups when explicit corpus block is injected each turn.

## Reproduce

```bash
cd /path/to/Valhalla
RUSTFLAGS="-L /opt/cuda/lib64" cargo build -p hub-f64 --release --bin valhalla_base
python3 tools/valhalla_inference/test_context_flexibility_goal.py
```

## Next Steps

- Longer threads (12–20 turns) under larger fair-benchmark corpus
- Combine `core_fusion` with hybrid follow-up for multimodal sessions
- Production default: `hybrid` + `lm_patch_on_follow_up` for dialogue; native for single-shot retrieval APIs
"""
    path.write_text(body, encoding="utf-8")
    print(f"→ {path}")


if __name__ == "__main__":
    main()
