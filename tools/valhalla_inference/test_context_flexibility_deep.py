#!/usr/bin/env python3
"""Deep context flexibility: growth curve, topic continuity, multi-question disambiguation.

Compares ValhallaBase (structure session context) vs traditional LM prompt injection.

Usage:
  python3 tools/valhalla_inference/test_context_flexibility_deep.py
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

from test_context_arch_compare import resolve_lm_path  # noqa: E402
from test_context_performance import context_block  # noqa: E402
from valhalla_base import ValhallaBase  # noqa: E402

REPORTS = REPO / "reports/valhalla_inference"

# Incremental append corpus for growth curve (flexible session).
GROWTH_SEED = [
    "The capital of France is Paris.",
    "France is in Western Europe.",
    "The Eiffel Tower is in Paris.",
    "French is the official language of France.",
]
GROWTH_APPENDS = [
    "The Louvre Museum is located in Paris.",
    "The Seine river flows through Paris.",
    "France borders Belgium, Germany, and Spain.",
    "The currency of France is the Euro.",
    "Versailles Palace is near Paris.",
    "Bordeaux is famous for wine in southwestern France.",
    "Marseille is a major port city in southern France.",
    "The Alps form part of France's eastern border.",
    "Normandy is a region in northern France.",
    "Lyon is known for gastronomy in east-central France.",
    "Strasbourg hosts the European Parliament.",
    "Nice is a resort city on the French Riviera.",
]

TOPIC_THREADS = [
    {
        "id": "australia",
        "seed": [
            "The capital of Australia is Canberra.",
            "Australia is a country in Oceania.",
        ],
        "turns": [
            {
                "append": "Sydney is the largest city in Australia but not the capital.",
                "question": "What is the capital of Australia?",
                "hint": "canberra",
                "forbidden": ["paris", "france", "london"],
            },
            {
                "append": "English is the de facto national language of Australia.",
                "question": "Which continent is Australia located in?",
                "hint": "oceania",
                "forbidden": ["europe", "africa", "paris"],
            },
            {
                "append": "The Great Barrier Reef lies off the coast of Queensland, Australia.",
                "question": "What is the largest city in Australia?",
                "hint": "sydney",
                "forbidden": ["canberra", "paris", "tokyo"],
            },
            {
                "append": "Australia uses the Australian dollar as currency.",
                "question": "What language is most widely spoken in Australia?",
                "hint": "english",
                "forbidden": ["french", "german", "paris"],
            },
        ],
    },
    {
        "id": "chemistry",
        "seed": [
            "Water boils at 100 degrees Celsius at standard atmospheric pressure.",
            "Water freezes at 0 degrees Celsius at standard pressure.",
        ],
        "turns": [
            {
                "append": "Steam forms when liquid water reaches its boiling point.",
                "question": "At what temperature does water boil at standard pressure?",
                "hint": "100",
                "forbidden": ["0 degree", "freez"],
            },
            {
                "append": "Ice melts at 0 degrees Celsius when heated.",
                "question": "At what temperature does water freeze?",
                "hint": "0",
                "forbidden": ["100", "boil"],
            },
            {
                "append": "The chemical formula for water is H2O.",
                "question": "What is the chemical formula for water?",
                "hint": "h2o",
                "forbidden": ["co2", "nacl"],
            },
            {
                "append": "Salt water boils at a slightly higher temperature than pure water.",
                "question": "What happens when water reaches 100C at standard pressure?",
                "hint": "boil",
                "forbidden": ["freez", "solid"],
            },
        ],
    },
    {
        "id": "physics",
        "seed": [
            "The SI unit of force is the Newton.",
            "Newton's second law states F equals m times a.",
        ],
        "turns": [
            {
                "append": "One Newton equals one kilogram meter per second squared.",
                "question": "What is the SI unit of force?",
                "hint": "newton",
                "forbidden": ["joule", "watt", "pascal"],
            },
            {
                "append": "Mass is measured in kilograms in the SI system.",
                "question": "In F=ma, what does m represent?",
                "hint": "mass",
                "forbidden": ["meter", "mole", "momentum"],
            },
            {
                "append": "Acceleration is the rate of change of velocity.",
                "question": "In F=ma, what does a represent?",
                "hint": "acceleration",
                "forbidden": ["area", "ampere", "angle"],
            },
            {
                "append": "Weight is force due to gravity on a mass.",
                "question": "What law relates force, mass, and acceleration?",
                "hint": "second",
                "forbidden": ["third", "thermo", "ohm"],
            },
        ],
    },
]

MULTI_Q_BATCHES = [
    {
        "id": "france_water",
        "context": [
            "The capital of France is Paris.",
            "France is in Western Europe.",
            "Water boils at 100 degrees Celsius at standard atmospheric pressure.",
            "Water freezes at 0 degrees Celsius at standard pressure.",
        ],
        "questions": [
            ("q_fr", "What is the capital of France?", "paris", ["100", "boil", "freez"]),
            ("q_boil", "At what temperature does water boil?", "100", ["paris", "france", "europe"]),
        ],
    },
    {
        "id": "triple_mix",
        "context": [
            "The capital of Australia is Canberra.",
            "The SI unit of force is the Newton.",
            "Photosynthesis occurs in chloroplasts using light energy.",
            "Gold has chemical symbol Au.",
        ],
        "questions": [
            ("q_au_cap", "What is the capital of Australia?", "canberra", ["newton", "chloroplast", " gold"]),
            ("q_force", "What is the SI unit of force?", "newton", ["canberra", "photosynth", "chloroplast"]),
            ("q_photo", "Where does photosynthesis occur?", "chloroplast", ["canberra", "newton", " paris"]),
        ],
    },
    {
        "id": "four_way",
        "context": [
            "Mercury is the closest planet to the Sun.",
            "The human heart has four chambers.",
            "Python is a programming language.",
            "The Pacific Ocean is the largest ocean on Earth.",
        ],
        "questions": [
            ("q_merc", "Which planet is closest to the Sun?", "mercury", ["heart", "python", "pacific"]),
            ("q_heart", "How many chambers does the human heart have?", "four", ["mercury", "python", "ocean"]),
            ("q_py", "Name one programming language mentioned in the context.", "python", ["mercury", "heart", "ocean"]),
            ("q_pac", "What is the largest ocean?", "pacific", ["mercury", "heart", "python"]),
        ],
    },
]


def norm(s: str) -> str:
    return re.sub(r"\s+", " ", s.lower().strip())


def topic_ok(answer: str, hint: str, forbidden: list[str]) -> bool:
    low = norm(answer)
    if hint.lower() not in low:
        return False
    return not any(f.strip() and f.lower() in low for f in forbidden)


def trad_generate(lm, question: str, ctx_lines: list[str]) -> str:
    q = context_block(ctx_lines) + question
    return lm.generate([q], max_tokens=128)[0].text.strip()


def measure_growth_curve(base: ValhallaBase, n_appends: int) -> dict:
    """Track how new-arch context grows under incremental session append."""
    base.clear_context()
    base.set_context(GROWTH_SEED)
    steps: list[dict] = []

    def snapshot(step: int, mode: str, payload_chars: int, doc: dict) -> None:
        sess = doc.get("session") or {}
        corpus = list(sess.get("corpus") or base.session.corpus or [])
        total_chars = sum(len(ln) for ln in corpus)
        steps.append(
            {
                "step": step,
                "mode": mode,
                "context_lines": len(corpus),
                "context_chars": total_chars,
                "fingerprint": str(sess.get("fingerprint") or base.session.fingerprint),
                "payload_chars": payload_chars,
                "incremental": mode == "append",
            }
        )

    # Initial full ingest
    doc = base.run(
        [{"id": "g0", "text": "What is the capital of France?", "score_type": "open"}]
    )
    init_payload = len(json.dumps({"context": GROWTH_SEED}))
    snapshot(0, "full_ingest", init_payload, doc)

    trad_corpus = list(GROWTH_SEED)
    trad_prompt_sizes: list[int] = [len(context_block(trad_corpus) + "What is the capital of France?")]

    for i, line in enumerate(GROWTH_APPENDS[:n_appends]):
        base.append_context(line)
        append_payload = len(json.dumps({"append_lines": [line]}))
        doc = base.run(
            [{"id": f"g{i+1}", "text": "What is the capital of France?", "score_type": "open"}],
            append_only=True,
        )
        snapshot(i + 1, "append", append_payload, doc)

        trad_corpus.append(line)
        trad_prompt_sizes.append(
            len(context_block(trad_corpus) + "What is the capital of France?")
        )

    last = steps[-1]
    return {
        "seed_lines": len(GROWTH_SEED),
        "append_steps": min(n_appends, len(GROWTH_APPENDS)),
        "final_context_lines": last["context_lines"],
        "final_context_chars": last["context_chars"],
        "fingerprint_changes": len({s["fingerprint"] for s in steps if s["fingerprint"]}),
        "new_arch_avg_incremental_payload_chars": round(
            sum(s["payload_chars"] for s in steps if s["incremental"])
            / max(1, sum(1 for s in steps if s["incremental"])),
            1,
        ),
        "new_arch_final_incremental_payload_chars": steps[-1]["payload_chars"]
        if steps[-1]["incremental"]
        else append_payload,
        "trad_final_full_prompt_chars": trad_prompt_sizes[-1],
        "trad_growth_ratio": round(trad_prompt_sizes[-1] / max(trad_prompt_sizes[0], 1), 2),
        "steps": steps,
        "trad_prompt_sizes": trad_prompt_sizes,
    }


def run_topic_thread_new(base: ValhallaBase, thread: dict) -> dict:
    base.clear_context()
    base.set_context(thread["seed"])
    turns_out: list[dict] = []
    hits = 0

    for i, turn in enumerate(thread["turns"]):
        base.append_context(turn["append"])
        doc = base.run(
            [{"id": f"{thread['id']}_{i}", "text": turn["question"], "score_type": "open"}],
            append_only=base.session.session_state is not None,
        )

        ans = doc["rows"][0]["answer"] if doc.get("rows") else ""
        ok = topic_ok(ans, turn["hint"], turn["forbidden"])
        hits += int(ok)
        turns_out.append(
            {
                "turn": i,
                "question": turn["question"],
                "answer": ans[:160],
                "hint": turn["hint"],
                "on_topic": ok,
                "context_lines": doc.get("context_lines"),
                "context_chars": sum(len(l) for l in (doc.get("session") or {}).get("corpus") or []),
            }
        )

    return {
        "thread_id": thread["id"],
        "turns": len(thread["turns"]),
        "on_topic_hits": hits,
        "on_topic_rate": hits / max(len(thread["turns"]), 1),
        "turn_details": turns_out,
    }


def run_topic_thread_trad(lm, thread: dict) -> dict:
    corpus = list(thread["seed"])
    turns_out: list[dict] = []
    hits = 0

    for i, turn in enumerate(thread["turns"]):
        if turn.get("append"):
            corpus.append(turn["append"])
        ans = trad_generate(lm, turn["question"], corpus)
        ok = topic_ok(ans, turn["hint"], turn["forbidden"])
        hits += int(ok)
        turns_out.append(
            {
                "turn": i,
                "question": turn["question"],
                "answer": ans[:160],
                "hint": turn["hint"],
                "on_topic": ok,
                "context_lines": len(corpus),
                "context_chars": sum(len(l) for l in corpus),
                "prompt_chars": len(context_block(corpus) + turn["question"]),
            }
        )

    return {
        "thread_id": thread["id"],
        "turns": len(thread["turns"]),
        "on_topic_hits": hits,
        "on_topic_rate": hits / max(len(thread["turns"]), 1),
        "turn_details": turns_out,
    }


def run_multi_q_new(base: ValhallaBase, batch: dict) -> dict:
    base.clear_context()
    doc = base.run(
        [
            {"id": qid, "text": qtext, "score_type": "open"}
            for qid, qtext, _, _ in batch["questions"]
        ],
        context=batch["context"],
    )
    rows = {r["id"]: r for r in doc.get("rows") or []}
    details: list[dict] = []
    hits = 0
    for qid, qtext, hint, forbidden in batch["questions"]:
        ans = rows.get(qid, {}).get("answer", "")
        ok = topic_ok(ans, hint, forbidden)
        hits += int(ok)
        details.append({"id": qid, "question": qtext, "answer": ans[:160], "correct_topic": ok})
    return {
        "batch_id": batch["id"],
        "n_questions": len(batch["questions"]),
        "disambiguation_hits": hits,
        "disambiguation_rate": hits / max(len(batch["questions"]), 1),
        "details": details,
    }


def run_multi_q_trad(lm, batch: dict) -> dict:
    ctx = batch["context"]
    details: list[dict] = []
    hits = 0
    for qid, qtext, hint, forbidden in batch["questions"]:
        ans = trad_generate(lm, qtext, ctx)
        ok = topic_ok(ans, hint, forbidden)
        hits += int(ok)
        details.append({"id": qid, "question": qtext, "answer": ans[:160], "correct_topic": ok})
    return {
        "batch_id": batch["id"],
        "n_questions": len(batch["questions"]),
        "disambiguation_hits": hits,
        "disambiguation_rate": hits / max(len(batch["questions"]), 1),
        "details": details,
    }


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--growth-appends", type=int, default=12)
    ap.add_argument("--lm", default="Qwen/Qwen2.5-0.5B-Instruct")
    args = ap.parse_args()

    from loader import ValhallaLLM

    lm_path = resolve_lm_path(args.lm)
    base = ValhallaBase(decode="native")
    lm = ValhallaLLM(lm_path, engine="transformers")

    print("=== A. Context growth (flexible session append) ===")
    growth = measure_growth_curve(base, args.growth_appends)
    print(
        f"  seed {growth['seed_lines']} → final {growth['final_context_lines']} lines, "
        f"{growth['final_context_chars']} chars"
    )
    print(
        f"  incremental payload ~{growth['new_arch_avg_incremental_payload_chars']} chars/step; "
        f"trad full prompt {growth['trad_final_full_prompt_chars']} chars "
        f"({growth['trad_growth_ratio']}× initial)"
    )

    print("\n=== B. Topic continuity (same-thread follow-ups) ===")
    new_threads = [run_topic_thread_new(ValhallaBase(decode="native"), t) for t in TOPIC_THREADS]
    trad_threads = [run_topic_thread_trad(lm, t) for t in TOPIC_THREADS]
    new_topic = sum(t["on_topic_hits"] for t in new_threads)
    trad_topic = sum(t["on_topic_hits"] for t in trad_threads)
    new_topic_n = sum(t["turns"] for t in new_threads)
    print(f"  New: {new_topic}/{new_topic_n} on-topic")
    print(f"  Trad: {trad_topic}/{new_topic_n} on-topic")

    print("\n=== C. Multi-question disambiguation ===")
    new_multi = [run_multi_q_new(ValhallaBase(decode="native"), b) for b in MULTI_Q_BATCHES]
    trad_multi = [run_multi_q_trad(lm, b) for b in MULTI_Q_BATCHES]
    new_dis = sum(b["disambiguation_hits"] for b in new_multi)
    trad_dis = sum(b["disambiguation_hits"] for b in trad_multi)
    dis_n = sum(b["n_questions"] for b in new_multi)
    print(f"  New: {new_dis}/{dis_n} correct-topic")
    print(f"  Trad: {trad_dis}/{dis_n} correct-topic")

    topic_delta = (new_topic - trad_topic) / max(new_topic_n, 1)
    dis_delta = (new_dis - trad_dis) / max(dis_n, 1)

    if topic_delta >= 0.10 and dis_delta >= 0.10:
        verdict = "NEW_ARCH_FLEXIBILITY_WINS"
    elif topic_delta <= -0.10 and dis_delta <= -0.10:
        verdict = "TRADITIONAL_WINS"
    elif abs(topic_delta) < 0.05 and abs(dis_delta) < 0.05:
        verdict = "NO_FLEXIBILITY_GAP"
    else:
        verdict = "MIXED_FLEXIBILITY_GAP"

    summary = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "protocol": "context-flexibility-deep-v1",
        "growth_curve": growth,
        "topic_continuity": {
            "new_architecture": {
                "name": "ValhallaBase session",
                "total_on_topic": new_topic,
                "total_turns": new_topic_n,
                "rate": new_topic / max(new_topic_n, 1),
                "threads": new_threads,
            },
            "traditional": {
                "name": "LM prompt injection",
                "total_on_topic": trad_topic,
                "total_turns": new_topic_n,
                "rate": trad_topic / max(new_topic_n, 1),
                "threads": trad_threads,
            },
            "delta_rate_new_minus_trad": topic_delta,
        },
        "multi_question_disambiguation": {
            "new_architecture": {
                "total_hits": new_dis,
                "total_questions": dis_n,
                "rate": new_dis / max(dis_n, 1),
                "batches": new_multi,
            },
            "traditional": {
                "total_hits": trad_dis,
                "total_questions": dis_n,
                "rate": trad_dis / max(dis_n, 1),
                "batches": trad_multi,
            },
            "delta_rate_new_minus_trad": dis_delta,
        },
        "verdict": verdict,
    }

    print(f"\nTopic continuity Δ: {topic_delta:+.1%}")
    print(f"Multi-Q disambiguation Δ: {dis_delta:+.1%}")
    print(f"Verdict: {verdict}")

    REPORTS.mkdir(parents=True, exist_ok=True)
    out = REPORTS / "context_flexibility_deep.json"
    out.write_text(json.dumps(summary, indent=2, ensure_ascii=False), encoding="utf-8")
    print(f"→ {out}")


if __name__ == "__main__":
    main()
