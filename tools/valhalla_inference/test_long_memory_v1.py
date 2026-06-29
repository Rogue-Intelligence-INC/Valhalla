#!/usr/bin/env python3
"""Long-session memory — recall, persistence, and config sweep.

Protocol: long-memory-v1

Batteries:
  A. early_anchor   — critical fact at session start; 30 filler appends; recall anchor
  B. extended_20    — 20-turn × 3 topic threads (persistent append)
  C. session_reload  — mid-thread save/restore session_state JSON
  D. corpus_depth   — 50-line mixed corpus; probe early / mid / late facts

Arms (config presets):
  production       — hybrid + native_follow_up_aware + fate_qa_feedback + cycles=2
  cycles4          — incubation_cycles=4
  no_qa_feedback   — fate_qa_feedback=False
  inherit_followup — follow_up_decode=inherit (ablation)

Usage:
  python3 tools/valhalla_inference/test_long_memory_v1.py
  python3 tools/valhalla_inference/test_long_memory_v1.py --fast
  python3 tools/valhalla_inference/test_long_memory_v1.py --arm cycles4
"""

from __future__ import annotations

import argparse
import copy
import json
import os
import re
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

REPO = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(REPO / "tools/fair_benchmark"))

from test_context_flexibility_goal import EXTENDED_TURNS, SEEDS, topic_ok  # noqa: E402
from valhalla_base import ValhallaBase  # noqa: E402

PROTOCOL = "long-memory-v1"
REPORTS = REPO / "reports/valhalla_inference"

FILLER_LINES = [
    "Paris is the capital of France and lies on the River Seine.",
    "Iron rusts when exposed to moist air over time.",
    "The speed of light in vacuum is approximately 299792458 meters per second.",
    "Photosynthesis converts light energy into chemical energy in plants.",
    "The Pacific Ocean is the largest ocean on Earth by surface area.",
    "Diamond is a crystalline form of carbon with high hardness.",
    "The human heart has four chambers: two atria and two ventricles.",
    "Mitochondria are often called the powerhouse of the cell.",
    "Shakespeare wrote Hamlet and Romeo and Juliet.",
    "The Great Wall of China is a series of fortifications in northern China.",
    "Jupiter is the largest planet in the Solar System.",
    "The chemical symbol for sodium is Na.",
    "Beethoven composed nine symphonies during his lifetime.",
    "The Amazon rainforest produces significant oxygen for the planet.",
    "Mount Everest is the highest mountain above sea level.",
    "The periodic table organizes elements by atomic number.",
    "DNA stores genetic information in living organisms.",
    "The Nile is one of the longest rivers in the world.",
    "Volcanoes form at tectonic plate boundaries.",
    "The Moon orbits Earth approximately every 27.3 days.",
    "Antarctica is the coldest continent on Earth.",
    "Copper is an excellent conductor of electricity.",
    "The Renaissance began in Italy in the 14th century.",
    "Black holes have gravitational fields so strong light cannot escape.",
    "The Sahara is the largest hot desert in the world.",
    "Oxygen makes up about 21 percent of Earth's atmosphere.",
    "The Eiffel Tower was completed in Paris in 1889.",
    "Proteins are polymers of amino acids.",
    "The Roman Empire once controlled much of Europe and the Mediterranean.",
    "Sound travels faster in water than in air.",
]

ANCHOR_CASES = [
    {
        "id": "codename",
        "seed": "The classified project codename is NEBULA-7.",
        "question": "What is the classified project codename?",
        "hint": "nebula-7",
        "forbidden": ["paris", "france", "iron"],
    },
    {
        "id": "employee_id",
        "seed": "Employee badge number for site access is AX-44291.",
        "question": "What is the employee badge number for site access?",
        "hint": "ax-44291",
        "forbidden": ["paris", "jupiter", "shakespeare"],
    },
    {
        "id": "policy_date",
        "seed": "The data retention policy effective date is 2019-03-15.",
        "question": "When does the data retention policy take effect?",
        "hint": "2019-03-15",
        "forbidden": ["1889", "14th century"],
    },
]

CORPUS_DEPTH = [
    ("line_1", "The vault access PIN is 7391.", "What is the vault access PIN?", "7391", []),
    ("line_25", "The backup server hostname is backup-east-04.internal.", "What is the backup server hostname?", "backup-east-04", []),
    ("line_50", "Emergency contact extension is 9912.", "What is the emergency contact extension?", "9912", []),
]


def make_base(arm: str) -> ValhallaBase:
    b = ValhallaBase(
        decode="hybrid",
        follow_up_decode="native_follow_up_aware",
        fate_qa_feedback=True,
        incubation_cycles=2,
    )
    if arm == "cycles4":
        b.incubation_cycles = 4
    elif arm == "no_qa_feedback":
        b.fate_qa_feedback = False
    elif arm == "inherit_followup":
        b.follow_up_decode = "inherit"
    elif arm == "production":
        pass
    else:
        raise ValueError(f"unknown arm: {arm}")
    return b


def ask_open(base: ValhallaBase, qid: str, question: str) -> dict[str, Any]:
    append_only = base.session.session_state is not None
    doc = base.run(
        [{"id": qid, "text": question, "score_type": "open"}],
        append_only=append_only,
    )
    row = (doc.get("rows") or [{}])[0]
    return {
        "answer": row.get("answer", ""),
        "decode_path": row.get("decode_path", ""),
        "corpus_lines": len((doc.get("session") or {}).get("corpus") or []),
        "history_len": len((doc.get("session") or {}).get("question_history") or []),
    }


def battery_early_anchor(base: ValhallaBase, n_filler: int) -> list[dict]:
    out: list[dict] = []
    for case in ANCHOR_CASES:
        base.clear_context()
        base.set_context([case["seed"]])
        for i, line in enumerate(FILLER_LINES[:n_filler]):
            base.append_context(line)
        r = ask_open(base, f"anchor_{case['id']}", case["question"])
        ok = topic_ok(r["answer"], case["hint"], case["forbidden"])
        out.append(
            {
                "battery": "early_anchor",
                "case_id": case["id"],
                "n_filler": n_filler,
                "ok": ok,
                **r,
            }
        )
    return out


def battery_extended_20(base: ValhallaBase) -> list[dict]:
    out: list[dict] = []
    for thread_id, turns in EXTENDED_TURNS.items():
        base.clear_context()
        base.set_context(SEEDS[thread_id])
        # Repeat 10-turn block twice → 20 turns
        for rep in range(2):
            for i, (append, question, hint, forbidden) in enumerate(turns):
                base.append_context(append)
                r = ask_open(base, f"{thread_id}_{rep}_{i}", question)
                ok = topic_ok(r["answer"], hint, forbidden)
                out.append(
                    {
                        "battery": "extended_20",
                        "thread": thread_id,
                        "turn": rep * len(turns) + i,
                        "ok": ok,
                        **r,
                    }
                )
    return out


def battery_session_reload(base: ValhallaBase) -> list[dict]:
    out: list[dict] = []
    thread_id = "australia"
    turns = EXTENDED_TURNS[thread_id]
    base.clear_context()
    base.set_context(SEEDS[thread_id])

    mid = len(turns) // 2
    for i, (append, question, hint, forbidden) in enumerate(turns[:mid]):
        base.append_context(append)
        r = ask_open(base, f"reload_a_{i}", question)
        out.append({"battery": "session_reload", "phase": "before_save", "turn": i, "ok": topic_ok(r["answer"], hint, forbidden), **r})

    saved = copy.deepcopy(base.session.session_state)

    base2 = make_base("production")
    base2.session.session_state = saved
    base2.session.corpus = list(saved.get("corpus") or [])
    base2.session.fingerprint = str(saved.get("fingerprint") or "")

    for i, (append, question, hint, forbidden) in enumerate(turns[mid:], start=mid):
        base2.append_context(append)
        r = ask_open(base2, f"reload_b_{i}", question)
        out.append({"battery": "session_reload", "phase": "after_restore", "turn": i, "ok": topic_ok(r["answer"], hint, forbidden), **r})
    return out


def battery_corpus_depth(base: ValhallaBase) -> list[dict]:
    out: list[dict] = []
    lines: list[str] = []
    for i, line in enumerate(FILLER_LINES[:49], start=1):
        lines.append(line)
    # Inject probes at positions
    lines.insert(0, CORPUS_DEPTH[0][1])  # vault PIN at start
    if len(lines) >= 25:
        lines[24] = CORPUS_DEPTH[1][1]  # backup hostname ~line 25
    while len(lines) < 49:
        lines.append(FILLER_LINES[len(lines) % len(FILLER_LINES)])
    lines.append(CORPUS_DEPTH[2][1])  # emergency ext at end

    base.clear_context()
    base.set_context(lines[:1])
    for ln in lines[1:]:
        base.append_context(ln)

    for probe_id, _fact, question, hint, forbidden in CORPUS_DEPTH:
        r = ask_open(base, f"depth_{probe_id}", question)
        ok = topic_ok(r["answer"], hint, forbidden)
        out.append({"battery": "corpus_depth", "probe": probe_id, "ok": ok, **r})
    return out


def summarize(rows: list[dict]) -> dict[str, Any]:
    by_battery: dict[str, list[dict]] = {}
    for r in rows:
        by_battery.setdefault(r["battery"], []).append(r)

    summary: dict[str, Any] = {}
    for bat, items in by_battery.items():
        hits = sum(1 for x in items if x.get("ok"))
        summary[bat] = {
            "n": len(items),
            "hits": hits,
            "rate": round(hits / len(items), 4) if items else 0.0,
        }
    summary["overall"] = {
        "n": len(rows),
        "hits": sum(1 for x in rows if x.get("ok")),
        "rate": round(sum(1 for x in rows if x.get("ok")) / len(rows), 4) if rows else 0.0,
    }
    return summary


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--fast", action="store_true", help="early_anchor only, 15 filler")
    ap.add_argument(
        "--arm",
        default="all",
        choices=["all", "production", "cycles4", "no_qa_feedback", "inherit_followup"],
    )
    ap.add_argument("--filler", type=int, default=30, help="filler lines for early_anchor")
    args = ap.parse_args()

    arms = (
        ["production", "cycles4", "no_qa_feedback", "inherit_followup"]
        if args.arm == "all" and not args.fast
        else [args.arm if args.arm != "all" else "production"]
    )

    all_results: dict[str, Any] = {
        "protocol": PROTOCOL,
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "env": {
            "VALHALLA_NATIVE_MAX_CORPUS_LINES": os.environ.get("VALHALLA_NATIVE_MAX_CORPUS_LINES", "32(default)"),
            "VALHALLA_INGEST_ALL_CORPUS": os.environ.get("VALHALLA_INGEST_ALL_CORPUS", ""),
        },
        "arms": {},
    }

    for arm in arms:
        base = make_base(arm)
        rows: list[dict] = []
        n_filler = 15 if args.fast else args.filler
        rows.extend(battery_early_anchor(base, n_filler))
        if not args.fast:
            rows.extend(battery_extended_20(base))
            rows.extend(battery_session_reload(base))
            rows.extend(battery_corpus_depth(base))
        for r in rows:
            r["arm"] = arm
        all_results["arms"][arm] = {"summary": summarize(rows), "rows": rows}
        s = all_results["arms"][arm]["summary"]["overall"]
        print(f"[{arm}] overall {s['hits']}/{s['n']} ({100*s['rate']:.1f}%)")

    out = REPORTS / ("long_memory_test_fast.json" if args.fast else "long_memory_test_full.json")
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(all_results, indent=2, ensure_ascii=False) + "\n")
    print(f"Wrote {out}")


if __name__ == "__main__":
    main()
