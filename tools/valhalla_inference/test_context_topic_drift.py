#!/usr/bin/env python3
"""Context swap, topic drift, and cross-topic hallucination — fair experiment.

Protocol: context-topic-drift-v1

Fairness rules (all arms):
  - Fixed question set; hints/forbidden labels fixed in spec (not model-derived).
  - Matched context size: swap/distractor arms use k=8 lines where applicable.
  - Isolated session per trial (clear_context); no cross-trial session bleed.
  - Text-only context (no multimodal) to isolate context routing.
  - Traditional LM control receives byte-identical context via context_block().
  - open_retrieval slice from fair-1.2 test split for corpus-linked probes.

Arms:
  - production     hybrid + native_follow_up_aware (ValhallaBase production stack)
  - native_inherit native + inherit (structure retrieval ablation)
  - traditional_lm Qwen2.5-0.5B + full prompt context (transformer control)

Batteries:
  A. context_swap      — same Q, two mutually exclusive contexts; must match active hint
  B. distractor_open   — targeted vs distractor vs minimal (fair open_retrieval)
  C. interleave_drift  — mixed-topic appends; probe on-topic + forbidden cross-topic
  D. recency_trap      — long wrong-topic block then one correct line; recency bias test
  E. extended_followup — 10-turn × 3 topic threads (30 total); session persistence +
                         cross-topic forbidden drift under follow-up decode modes

Usage:
  python3 tools/valhalla_inference/test_context_topic_drift.py
  python3 tools/valhalla_inference/test_context_topic_drift.py --skip-trad
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass
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
from run_scale_benchmark import corpus_for_prompt, distractor_context  # noqa: E402
from run_strict_incubation_experiment import strict_user_text  # noqa: E402
from test_context_arch_compare import resolve_lm_path  # noqa: E402
from test_context_flexibility_deep import norm  # noqa: E402
from test_context_flexibility_goal import EXTENDED_TURNS, SEEDS  # noqa: E402
from test_context_performance import context_block  # noqa: E402
from valhalla_base import ValhallaBase  # noqa: E402

PROTOCOL = "context-topic-drift-v2"
REPORTS = REPO / "reports/valhalla_inference"

SWAP_PAIRS = [
    {
        "id": "swap_au_fr",
        "question": "What is the capital of Australia?",
        "ctx_a": [
            "The capital of Australia is Canberra.",
            "Australia is a country in Oceania.",
            "Canberra is located in the Australian Capital Territory.",
        ],
        "ctx_b": [
            "The capital of France is Paris.",
            "France is in Western Europe.",
            "Paris is on the River Seine.",
        ],
        "hint_a": "canberra",
        "hint_b": "paris",
        "forbidden_a": ["paris", "france", "seine"],
        "forbidden_b": ["canberra", "australia", "oceania"],
    },
    {
        "id": "swap_boil_freeze",
        "question": "At what temperature does water boil at standard pressure?",
        "ctx_a": [
            "Water boils at 100 degrees Celsius at standard atmospheric pressure.",
            "Steam forms when water reaches its boiling point.",
        ],
        "ctx_b": [
            "Water freezes at 0 degrees Celsius at standard pressure.",
            "Ice melts at 0 degrees Celsius when heated.",
        ],
        "hint_a": "100",
        "hint_b": "0",
        "forbidden_a": ["freez", "ice melt", "re:(?<![\\d])0 degrees?"],
        "forbidden_b": ["100", "boil", "steam"],
    },
    {
        "id": "swap_au_fe",
        "question": "What is the chemical symbol for gold?",
        "ctx_a": [
            "The chemical symbol for gold is Au.",
            "Gold is a precious metal.",
        ],
        "ctx_b": [
            "The chemical symbol for iron is Fe.",
            "Iron rusts in moist air.",
        ],
        "hint_a": "au",
        "hint_b": "fe",
        "forbidden_a": [" iron", "fe ", "rust"],
        "forbidden_b": ["gold", " au", "precious"],
    },
    {
        "id": "swap_newton_joule",
        "question": "What is the SI unit of force?",
        "ctx_a": [
            "The SI unit of force is the Newton.",
            "Newton's second law relates force, mass, and acceleration.",
        ],
        "ctx_b": [
            "The SI unit of energy is the Joule.",
            "One Joule equals one Newton meter.",
        ],
        "hint_a": "newton",
        "hint_b": "joule",
        "forbidden_a": ["joule", "energy", "watt"],
        "forbidden_b": ["newton", "force", "mass times"],
    },
    {
        "id": "swap_chloro_mito",
        "question": "Which organelle is the site of photosynthesis?",
        "ctx_a": [
            "Photosynthesis occurs in chloroplasts using light energy.",
            "Chloroplasts contain chlorophyll pigment.",
        ],
        "ctx_b": [
            "Mitochondria produce ATP through cellular respiration.",
            "The mitochondrion is the powerhouse of the cell.",
        ],
        "hint_a": "chloroplast",
        "hint_b": "mitochondri",
        "forbidden_a": ["mitochondri", "atp", "respiration"],
        "forbidden_b": ["chloroplast", "photosynth", "chlorophyll"],
    },
    {
        "id": "swap_ww2_ww1",
        "question": "In what year did World War Two end?",
        "ctx_a": [
            "World War Two ended in 1945 with Germany's unconditional surrender.",
            "The Pacific war concluded in 1945 after atomic bombings.",
        ],
        "ctx_b": [
            "World War One ended in 1918 with the Armistice.",
            "The Treaty of Versailles was signed in 1919.",
        ],
        "hint_a": "1945",
        "hint_b": "1918",
        "forbidden_a": ["1918", "1919", "versailles", "world war one"],
        "forbidden_b": ["1945", "world war two", "atomic"],
    },
]

INTERLEAVE_SESSIONS = [
    {
        "id": "capitals_interleave",
        "seed": ["The capital of France is Paris.", "France is in Western Europe."],
        "appends": [
            "The capital of Australia is Canberra.",
            "English is widely spoken in Australia.",
            "The Eiffel Tower is in Paris.",
            "Sydney is the largest city in Australia but not the capital.",
            "The Louvre Museum is in Paris.",
            "Canberra is in the Australian Capital Territory.",
        ],
        "probes": [
            {
                "after_append": 2,
                "question": "What is the capital of Australia?",
                "hint": "canberra",
                "forbidden": ["paris", "france", "eiffel", "louvre"],
            },
            {
                "after_append": 4,
                "question": "What is the capital of France?",
                "hint": "paris",
                "forbidden": ["canberra", "sydney", "australia"],
            },
            {
                "after_append": 6,
                "question": "What is the capital of Australia?",
                "hint": "canberra",
                "forbidden": ["paris", "france", "louvre"],
            },
        ],
    },
    {
        "id": "physics_chem_interleave",
        "seed": [
            "Water boils at 100 degrees Celsius at standard atmospheric pressure.",
            "The chemical formula for water is H2O.",
        ],
        "appends": [
            "The SI unit of force is the Newton.",
            "Newton's second law states F equals m times a.",
            "Water freezes at 0 degrees Celsius at standard pressure.",
            "Mass is measured in kilograms in the SI system.",
            "Ice melts at 0 degrees Celsius when heated.",
            "Acceleration is the rate of change of velocity.",
        ],
        "probes": [
            {
                "after_append": 2,
                "question": "What is the SI unit of force?",
                "hint": "newton",
                "forbidden": ["100", "boil", "h2o", "water boil"],
            },
            {
                "after_append": 4,
                "question": "At what temperature does water freeze?",
                "hint": "0",
                "forbidden": ["newton", "force", "f=ma", "100"],
            },
            {
                "after_append": 6,
                "question": "In F=ma, what does a represent?",
                "hint": "acceleration",
                "forbidden": ["0 degree", "freez", "h2o", "paris"],
            },
        ],
    },
]

RECENCY_TRAPS = [
    {
        "id": "france_then_canberra",
        "block": [
            "The capital of France is Paris.",
            "France is in Western Europe.",
            "The Eiffel Tower is in Paris.",
            "French is the official language of France.",
            "The Louvre Museum is located in Paris.",
            "The Seine river flows through Paris.",
        ],
        "correct_line": "The capital of Australia is Canberra.",
        "question": "What is the capital of Australia?",
        "hint": "canberra",
        "forbidden": ["paris", "france", "eiffel", "louvre", "seine"],
    },
    {
        "id": "boil_then_freeze",
        "block": [
            "Water boils at 100 degrees Celsius at standard atmospheric pressure.",
            "Steam forms when liquid water reaches its boiling point.",
            "At sea level water boils at one hundred degrees Celsius.",
        ],
        "correct_line": "Water freezes at 0 degrees Celsius at standard pressure.",
        "question": "At what temperature does water freeze at standard pressure?",
        "hint": "0",
        "forbidden": ["100", "boil", "steam", "one hundred"],
    },
]


@dataclass(frozen=True)
class Arm:
    arm_id: str
    decode: str
    follow_up_decode: str

    def make_base(self) -> ValhallaBase:
        return ValhallaBase(
            decode=self.decode,  # type: ignore[arg-type]
            follow_up_decode=self.follow_up_decode,  # type: ignore[arg-type]
            core_fusion=False,
            max_new=128,
        )


ARMS = [
    Arm("production", "hybrid", "native_follow_up_aware"),
    Arm("native_inherit", "native", "inherit"),
]

EXTENDED_ARMS = ARMS + [
    Arm("hybrid_follow_up_lm", "hybrid", "lm_patch_on_follow_up"),
]

THREAD_IDS = tuple(EXTENDED_TURNS.keys())


def trad_answer(lm, question: str, ctx: list[str]) -> str:
    prompt = context_block(ctx) + question
    return lm.generate([prompt], max_tokens=128)[0].text.strip()


def valhalla_answer(base: ValhallaBase, question: str, ctx: list[str], *, append_only: bool) -> str:
    doc = base.run(
        [{"id": "probe", "text": question, "score_type": "open"}],
        context=ctx if not append_only else None,
        append_only=append_only,
    )
    return (doc.get("rows") or [{}])[0].get("answer", "")


def cross_topic_hallucination(answer: str, forbidden: list[str]) -> bool:
    low = norm(answer)
    for f in forbidden:
        f = f.strip().lower()
        if not f:
            continue
        if f.startswith("re:"):
            if re.search(f[3:], low):
                return True
        elif f.isdigit():
            if re.search(rf"(?<!\d){re.escape(f)}(?!\d)", low):
                return True
        elif f in low:
            return True
    return False


def topic_ok_strict(answer: str, hint: str, forbidden: list[str]) -> bool:
    low = norm(answer)
    if hint.isdigit():
        if not re.search(rf"(?<!\d){re.escape(hint.lower())}(?!\d)", low):
            return False
    elif hint.lower() not in low:
        return False
    return not cross_topic_hallucination(answer, forbidden)


def run_swap_arm(arm: Arm, lm=None) -> dict:
    rows: list[dict] = []
    fidelity_hits = 0
    hallucination_hits = 0
    n = 0

    for pair in SWAP_PAIRS:
        for side, ctx, hint, forbidden in (
            ("a", pair["ctx_a"], pair["hint_a"], pair["forbidden_a"]),
            ("b", pair["ctx_b"], pair["hint_b"], pair["forbidden_b"]),
        ):
            n += 1
            if arm.arm_id == "traditional_lm":
                assert lm is not None
                ans = trad_answer(lm, pair["question"], ctx)
            else:
                base = arm.make_base()
                base.clear_context()
                ans = valhalla_answer(base, pair["question"], ctx, append_only=False)

            ok = topic_ok_strict(ans, hint, forbidden)
            hall = cross_topic_hallucination(ans, forbidden)
            fidelity_hits += int(ok)
            hallucination_hits += int(hall)
            rows.append(
                {
                    "pair_id": pair["id"],
                    "side": side,
                    "question": pair["question"],
                    "hint": hint,
                    "answer": ans[:200],
                    "swap_fidelity": ok,
                    "cross_topic_hallucination": hall,
                }
            )

    return {
        "trials": n,
        "swap_fidelity": fidelity_hits,
        "swap_fidelity_rate": fidelity_hits / max(n, 1),
        "cross_topic_hallucination_count": hallucination_hits,
        "cross_topic_hallucination_rate": hallucination_hits / max(n, 1),
        "rows": rows,
    }


def run_distractor_arm(arm: Arm, prompts: list[dict], corpus: list[str], lm=None) -> dict:
    modes = ("targeted", "distractor", "minimal")
    out: dict[str, Any] = {"modes": {}}

    for mode in modes:
        rows: list[dict] = []
        correct = 0
        halluc = 0
        false_conf = 0

        for p in prompts:
            good = corpus_for_prompt(p, corpus, k=8)
            if mode == "targeted":
                ctx = good
            elif mode == "distractor":
                ctx = distractor_context(corpus, good, k=8)
            else:
                ctx = corpus[:2]

            qtext = strict_user_text(p)
            if arm.arm_id == "traditional_lm":
                assert lm is not None
                ans = trad_answer(lm, qtext, ctx)
            else:
                base = arm.make_base()
                base.clear_context()
                ans = valhalla_answer(base, qtext, ctx, append_only=False)

            sc = score_answer(ans, p)
            ref = str(p.get("reference_answer", "")).lower()
            kws = [str(k).lower() for k in (p.get("keywords") or []) if k]

            # Hallucination: confident wrong anchor from distractor lines when targeted should apply
            distractor_anchor = False
            if mode == "targeted":
                wrong_lines = distractor_context(corpus, good, k=8)
                for ln in wrong_lines:
                    low_ln = ln.lower()
                    if any(k in low_ln for k in kws[:2]) and any(k in ans.lower() for k in kws[:2]):
                        if ln not in good and ref not in ln.lower():
                            distractor_anchor = False
                    for kw in kws:
                        if kw in ln.lower() and kw in ans.lower() and not sc["correct"]:
                            if ln not in good:
                                distractor_anchor = True
                                break
                hall = distractor_anchor or (
                    not sc["correct"] and cross_topic_hallucination(ans, [ref] if ref else [])
                )
            elif mode == "distractor":
                # Answering as if correct while context lacks support
                hall = sc["correct"] and not any(
                    ref in ln.lower() or any(k in ln.lower() for k in kws)
                    for ln in ctx
                )
                false_conf = int(sc["correct"])
            else:
                hall = sc["correct"]  # minimal shouldn't reliably hit ref

            correct += int(sc["correct"])
            halluc += int(hall)

            rows.append(
                {
                    "id": p["id"],
                    "correct": sc["correct"],
                    "hallucination_flag": hall,
                    "answer": ans[:180],
                    "context_mode": mode,
                }
            )

        out["modes"][mode] = {
            "n": len(rows),
            "accuracy": correct / max(len(rows), 1),
            "hallucination_rate": halluc / max(len(rows), 1),
            "false_confidence_rate": false_conf / max(len(rows), 1) if mode == "distractor" else None,
            "rows": rows,
        }

    return out


def run_interleave_arm(arm: Arm, lm=None) -> dict:
    session_rows: list[dict] = []
    on_topic = 0
    halluc = 0
    n = 0

    for sess in INTERLEAVE_SESSIONS:
        if arm.arm_id == "traditional_lm":
            assert lm is not None
            corpus = list(sess["seed"])
            for i, line in enumerate(sess["appends"], start=1):
                corpus.append(line)
                for probe in sess["probes"]:
                    if probe["after_append"] != i:
                        continue
                    n += 1
                    ans = trad_answer(lm, probe["question"], corpus)
                    ok = topic_ok_strict(ans, probe["hint"], probe["forbidden"])
                    hall = cross_topic_hallucination(ans, probe["forbidden"])
                    on_topic += int(ok)
                    halluc += int(hall)
                    session_rows.append(
                        {
                            "session_id": sess["id"],
                            "after_append": i,
                            "question": probe["question"],
                            "answer": ans[:180],
                            "on_topic": ok,
                            "cross_topic_hallucination": hall,
                        }
                    )
        else:
            base = arm.make_base()
            base.clear_context()
            base.set_context(sess["seed"])
            for i, line in enumerate(sess["appends"], start=1):
                base.append_context(line)
                for probe in sess["probes"]:
                    if probe["after_append"] != i:
                        continue
                    n += 1
                    ans = valhalla_answer(base, probe["question"], [], append_only=True)
                    ok = topic_ok_strict(ans, probe["hint"], probe["forbidden"])
                    hall = cross_topic_hallucination(ans, probe["forbidden"])
                    on_topic += int(ok)
                    halluc += int(hall)
                    session_rows.append(
                        {
                            "session_id": sess["id"],
                            "after_append": i,
                            "question": probe["question"],
                            "answer": ans[:180],
                            "on_topic": ok,
                            "cross_topic_hallucination": hall,
                        }
                    )

    return {
        "probes": n,
        "on_topic": on_topic,
        "on_topic_rate": on_topic / max(n, 1),
        "cross_topic_hallucination_count": halluc,
        "cross_topic_hallucination_rate": halluc / max(n, 1),
        "rows": session_rows,
    }


def run_recency_arm(arm: Arm, lm=None) -> dict:
    rows: list[dict] = []
    on_topic = 0
    halluc = 0

    for trap in RECENCY_TRAPS:
        ctx = trap["block"] + [trap["correct_line"]]
        if arm.arm_id == "traditional_lm":
            assert lm is not None
            ans = trad_answer(lm, trap["question"], ctx)
        else:
            base = arm.make_base()
            base.clear_context()
            for ln in trap["block"]:
                base.append_context(ln)
            base.append_context(trap["correct_line"])
            ans = valhalla_answer(base, trap["question"], [], append_only=True)

        ok = topic_ok_strict(ans, trap["hint"], trap["forbidden"])
        hall = cross_topic_hallucination(ans, trap["forbidden"])
        on_topic += int(ok)
        halluc += int(hall)
        rows.append(
            {
                "trap_id": trap["id"],
                "question": trap["question"],
                "answer": ans[:180],
                "on_topic": ok,
                "cross_topic_hallucination": hall,
                "recency_bias_failure": not ok,
            }
        )

    n = len(rows)
    return {
        "trials": n,
        "on_topic": on_topic,
        "on_topic_rate": on_topic / max(n, 1),
        "cross_topic_hallucination_count": halluc,
        "cross_topic_hallucination_rate": halluc / max(n, 1),
        "recency_bias_failure_rate": (n - on_topic) / max(n, 1),
        "rows": rows,
    }


def run_extended_thread_arm(arm: Arm, thread_id: str, lm=None) -> dict:
    turns_out: list[dict] = []
    on_topic = 0
    halluc = 0

    if arm.arm_id == "traditional_lm":
        assert lm is not None
        corpus = list(SEEDS[thread_id])
        for i, (append, question, hint, forbidden) in enumerate(EXTENDED_TURNS[thread_id]):
            corpus.append(append)
            ans = trad_answer(lm, question, corpus)
            ok = topic_ok_strict(ans, hint, forbidden)
            hall = cross_topic_hallucination(ans, forbidden)
            on_topic += int(ok)
            halluc += int(hall)
            turns_out.append(
                {
                    "turn": i,
                    "question": question,
                    "answer": ans[:180],
                    "hint": hint,
                    "on_topic": ok,
                    "cross_topic_hallucination": hall,
                    "topic_drift": not ok,
                }
            )
    else:
        base = arm.make_base()
        base.clear_context()
        base.set_context(SEEDS[thread_id])
        for i, (append, question, hint, forbidden) in enumerate(EXTENDED_TURNS[thread_id]):
            base.append_context(append)
            doc = base.run(
                [{"id": f"{thread_id}_{i}", "text": question, "score_type": "open"}],
                append_only=base.session.session_state is not None,
            )
            row = (doc.get("rows") or [{}])[0]
            ans = row.get("answer", "")
            ok = topic_ok_strict(ans, hint, forbidden)
            hall = cross_topic_hallucination(ans, forbidden)
            on_topic += int(ok)
            halluc += int(hall)
            turns_out.append(
                {
                    "turn": i,
                    "question": question,
                    "answer": ans[:180],
                    "hint": hint,
                    "on_topic": ok,
                    "cross_topic_hallucination": hall,
                    "topic_drift": not ok,
                    "decode_path": row.get("decode_path"),
                }
            )

    n = len(turns_out)
    return {
        "thread_id": thread_id,
        "turns": n,
        "on_topic": on_topic,
        "on_topic_rate": on_topic / max(n, 1),
        "cross_topic_hallucination_count": halluc,
        "cross_topic_hallucination_rate": halluc / max(n, 1),
        "topic_drift_count": n - on_topic,
        "topic_drift_rate": (n - on_topic) / max(n, 1),
        "turn_details": turns_out,
    }


def run_extended_followup_battery(arm: Arm, lm=None) -> dict:
    threads = [run_extended_thread_arm(arm, tid, lm) for tid in THREAD_IDS]
    total_on = sum(t["on_topic"] for t in threads)
    total_hall = sum(t["cross_topic_hallucination_count"] for t in threads)
    total_n = sum(t["turns"] for t in threads)
    return {
        "thread_count": len(threads),
        "total_turns": total_n,
        "on_topic": total_on,
        "on_topic_rate": total_on / max(total_n, 1),
        "cross_topic_hallucination_count": total_hall,
        "cross_topic_hallucination_rate": total_hall / max(total_n, 1),
        "topic_drift_count": total_n - total_on,
        "topic_drift_rate": (total_n - total_on) / max(total_n, 1),
        "threads": threads,
    }


def summarize_arms(report: dict) -> dict:
    summary: dict[str, dict] = {}
    for arm_id, data in report["arms"].items():
        summary[arm_id] = {}
        if "swap" in data:
            summary[arm_id].update(
                {
                    "swap_fidelity_rate": data["swap"]["swap_fidelity_rate"],
                    "swap_hallucination_rate": data["swap"]["cross_topic_hallucination_rate"],
                    "distractor_targeted_acc": data["distractor"]["modes"]["targeted"]["accuracy"],
                    "distractor_false_confidence": data["distractor"]["modes"]["distractor"].get(
                        "false_confidence_rate"
                    ),
                    "interleave_on_topic_rate": data["interleave"]["on_topic_rate"],
                    "interleave_hallucination_rate": data["interleave"]["cross_topic_hallucination_rate"],
                    "recency_on_topic_rate": data["recency"]["on_topic_rate"],
                    "recency_bias_failure_rate": data["recency"]["recency_bias_failure_rate"],
                }
            )
        if "extended_followup" in data:
            ef = data["extended_followup"]
            summary[arm_id].update(
                {
                    "extended_on_topic_rate": ef["on_topic_rate"],
                    "extended_hallucination_rate": ef["cross_topic_hallucination_rate"],
                    "extended_topic_drift_rate": ef["topic_drift_rate"],
                    "extended_on_topic": f"{ef['on_topic']}/{ef['total_turns']}",
                }
            )
    return summary


def main() -> None:
    ap = argparse.ArgumentParser(description="Context topic drift & hallucination fair experiment")
    ap.add_argument("--phase", default="test")
    ap.add_argument("--lm", default="Qwen/Qwen2.5-0.5B-Instruct")
    ap.add_argument("--skip-trad", action="store_true", help="Skip traditional LM control arm")
    ap.add_argument("--open-limit", type=int, default=0, help="0 = all open_retrieval in phase")
    ap.add_argument(
        "--battery-e-only",
        action="store_true",
        help="Only run battery E (30-turn extended follow-up threads)",
    )
    ap.add_argument(
        "--skip-battery-e",
        action="store_true",
        help="Skip battery E even when running full suite",
    )
    args = ap.parse_args()

    suite = load_suite()
    corpus = suite["corpus"]["ingest_clean"]
    prompts = [p for p in select_prompts(suite, args.phase) if p.get("score_type") == "open"]
    slices = classify_slices(prompts, corpus)
    open_retrieval = [p for p in prompts if p["id"] in slices["open_retrieval"]]
    if args.open_limit > 0:
        open_retrieval = open_retrieval[: args.open_limit]

    lm = None
    trad_error: str | None = None
    if not args.skip_trad:
        try:
            from loader import ValhallaLLM

            lm = ValhallaLLM(resolve_lm_path(args.lm), engine="transformers")
        except Exception as exc:  # noqa: BLE001
            trad_error = str(exc)
            print(f"  [warn] traditional_lm unavailable: {trad_error}", file=sys.stderr)

    if args.battery_e_only:
        arms_to_run = list(EXTENDED_ARMS)
        if lm is not None:
            arms_to_run.append(Arm("traditional_lm", "native", "inherit"))
    else:
        arms_to_run = list(ARMS)
        if lm is not None:
            arms_to_run.append(Arm("traditional_lm", "native", "inherit"))

    print(f"=== {PROTOCOL} phase={args.phase} ===")
    if args.battery_e_only:
        print("  mode: battery E only (30-turn extended follow-up)")
    print(f"  open_retrieval probes: {len(open_retrieval)}")
    print(f"  arms: {[a.arm_id for a in arms_to_run]}\n")

    report: dict[str, Any] = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "protocol": PROTOCOL,
        "fair_benchmark_version": suite.get("version", "fair-1.2"),
        "phase": args.phase,
        "fairness": {
            "isolated_session_per_trial": not args.battery_e_only,
            "persistent_session_battery_e": args.battery_e_only or not args.skip_battery_e,
            "text_only_context": True,
            "matched_context_lines_k": 8,
            "fixed_question_spec": True,
            "open_retrieval_count": len(open_retrieval),
            "extended_turns_per_thread": 10,
            "extended_thread_count": len(THREAD_IDS),
            "arms": [a.arm_id for a in arms_to_run],
        },
        "traditional_lm_error": trad_error,
        "arms": {},
    }

    for arm in arms_to_run:
        print(f"--- arm: {arm.arm_id} ---")
        use_lm = lm if arm.arm_id == "traditional_lm" else None
        arm_report: dict[str, Any] = {
            "config": {"decode": arm.decode, "follow_up_decode": arm.follow_up_decode},
        }

        if not args.battery_e_only:
            swap = run_swap_arm(arm, use_lm)
            print(
                f"  [A swap] fidelity {swap['swap_fidelity']}/{swap['trials']} "
                f"hallucination {swap['cross_topic_hallucination_count']}/{swap['trials']}"
            )
            distractor = run_distractor_arm(arm, open_retrieval, corpus, use_lm)
            dt = distractor["modes"]["targeted"]
            dd = distractor["modes"]["distractor"]
            print(
                f"  [B distractor] targeted acc {dt['accuracy']:.0%} "
                f"distractor false-conf {dd.get('false_confidence_rate', 0):.0%}"
            )
            interleave = run_interleave_arm(arm, use_lm)
            print(
                f"  [C interleave] on-topic {interleave['on_topic']}/{interleave['probes']} "
                f"hallucination {interleave['cross_topic_hallucination_count']}/{interleave['probes']}"
            )
            recency = run_recency_arm(arm, use_lm)
            print(
                f"  [D recency] on-topic {recency['on_topic']}/{recency['trials']} "
                f"bias-fail {recency['recency_bias_failure_rate']:.0%}"
            )
            arm_report.update(
                {
                    "swap": swap,
                    "distractor": distractor,
                    "interleave": interleave,
                    "recency": recency,
                }
            )

        if args.battery_e_only or not args.skip_battery_e:
            extended = run_extended_followup_battery(arm, use_lm)
            print(
                f"  [E extended 30-turn] on-topic {extended['on_topic']}/{extended['total_turns']} "
                f"({extended['on_topic_rate']:.1%}) "
                f"drift {extended['topic_drift_count']} "
                f"hallucination {extended['cross_topic_hallucination_count']}"
            )
            for th in extended["threads"]:
                print(
                    f"    {th['thread_id']:10s} "
                    f"{th['on_topic']}/{th['turns']} "
                    f"drift={th['topic_drift_count']} hall={th['cross_topic_hallucination_count']}"
                )
            arm_report["extended_followup"] = extended

        report["arms"][arm.arm_id] = arm_report

    report["summary"] = summarize_arms(report)
    report["verdict"] = []

    if not args.battery_e_only:
        prod = report["summary"].get("production", {})
        nat = report["summary"].get("native_inherit", {})
        if prod.get("swap_fidelity_rate", 0) >= 0.9:
            report["verdict"].append("SWAP_FIDELITY_OK")
        if prod.get("interleave_on_topic_rate", 0) >= nat.get("interleave_on_topic_rate", 0):
            report["verdict"].append("PRODUCTION_INTERLEAVE_GTE_NATIVE")
        if prod.get("distractor_targeted_acc", 0) >= 0.85:
            report["verdict"].append("TARGETED_RETRIEVAL_OK")
        if prod.get("recency_bias_failure_rate", 1) <= 0.5:
            report["verdict"].append("RECENCY_TRAP_PARTIAL_OK")

    if args.battery_e_only or not args.skip_battery_e:
        prod_e = report["summary"].get("production", {})
        nat_e = report["summary"].get("native_inherit", {})
        hfu = report["summary"].get("hybrid_follow_up_lm", {})
        if prod_e.get("extended_on_topic_rate", 0) >= 0.9:
            report["verdict"].append("EXTENDED_ON_TOPIC_OK")
        if prod_e.get("extended_on_topic_rate", 0) >= nat_e.get("extended_on_topic_rate", 0):
            report["verdict"].append("PRODUCTION_EXTENDED_GTE_NATIVE")
        if hfu and prod_e.get("extended_on_topic_rate", 0) > hfu.get("extended_on_topic_rate", 0):
            report["verdict"].append("FOLLOW_UP_AWARE_BEATS_LM_PATCH")
        if prod_e.get("extended_hallucination_rate", 1) <= 0.05:
            report["verdict"].append("EXTENDED_HALLUCINATION_LOW")

    if not report["verdict"]:
        report["verdict"] = ["NEEDS_IMPROVEMENT"]

    REPORTS.mkdir(parents=True, exist_ok=True)
    out_name = (
        "context_topic_drift_extended_test.json"
        if args.battery_e_only
        else "context_topic_drift_test.json"
    )
    out = REPORTS / out_name
    out.write_text(json.dumps(report, indent=2, ensure_ascii=False), encoding="utf-8")

    print(f"\nVerdict: {report['verdict']}")
    print(f"→ {out}")


if __name__ == "__main__":
    main()
