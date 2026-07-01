#!/usr/bin/env python3
"""Needle-in-a-haystack — session append protocol (recognized long-memory probe).

Protocol: needle-session-v1

Standard Paul Graham-style needle test adapted to ValhallaBase persistent session:
  append filler → insert needle → append filler → ask recall question.

Depths: 5, 10, 20, 30 filler lines before/after needle (configurable).

Usage:
  python3 tools/valhalla_inference/test_needle_session_v1.py
  python3 tools/valhalla_inference/test_needle_session_v1.py --fast
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

from valhalla_base import ValhallaBase  # noqa: E402

PROTOCOL = "needle-session-v1"
REPORTS = REPO / "reports/valhalla_inference"

NEEDLE = "The magic registration code for Project Valhalla is XK-7742."
QUESTION = "What is the magic registration code for Project Valhalla?"
HINT = "xk-7742"

FILLER = [
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
    "Mercury is the closest planet to the Sun.",
    "Helium is the second lightest element in the periodic table.",
    "The Amazon River discharges more water than any other river.",
    "Granite is an igneous rock formed from cooled magma.",
    "The human skeleton has 206 bones in adulthood.",
]


def recall_ok(answer: str) -> bool:
    low = answer.lower()
    return HINT in low or "7742" in low


def run_depth(depth: int, *, fast: bool) -> dict[str, Any]:
    base = ValhallaBase(
        decode="hybrid",
        follow_up_decode="native_follow_up_aware",
        fate_ingress_routing="quad_cycle",
        fate_qa_feedback=False,
        incubation_cycles=1 if fast else 2,
        max_new=64,
    )
    before = [FILLER[i % len(FILLER)] for i in range(depth)]
    after = [FILLER[(i + depth) % len(FILLER)] for i in range(depth)]

    base.set_context(before[:1] if before else [])
    for line in before[1:] + [NEEDLE] + after:
        base.append_context(line)

    out = base.run([{"id": f"needle_d{depth}", "text": QUESTION, "score_type": "open"}])
    ans = str((out.get("rows") or [{}])[0].get("answer") or "")
    ok = recall_ok(ans)
    return {
        "depth": depth,
        "session_lines": 1 + len(before) + 1 + len(after),
        "correct": ok,
        "answer": ans[:160],
    }


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--fast", action="store_true", help="depths 5,10 only")
    ap.add_argument("--depths", type=str, default="", help="comma depths e.g. 5,10,20")
    args = ap.parse_args()

    if args.depths:
        depths = [int(x) for x in args.depths.split(",") if x.strip()]
    elif args.fast:
        depths = [5, 10]
    else:
        depths = [5, 10, 20, 30]

    print(f"=== {PROTOCOL} depths={depths} ===\n")

    rows = [run_depth(d, fast=args.fast) for d in depths]
    correct = sum(1 for r in rows if r["correct"])
    n = len(rows)
    recall = correct / max(n, 1)

    for r in rows:
        print(f"  {'✓' if r['correct'] else '✗'} depth={r['depth']:2d} lines={r['session_lines']}")

    verdict = "NEEDLE_STRONG" if recall >= 0.90 else "NEEDLE_VIABLE" if recall >= 0.70 else "NEEDLE_DEVELOPING"
    report = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "protocol": PROTOCOL,
        "benchmark": "Needle-in-a-haystack (session append)",
        "needle": NEEDLE,
        "question": QUESTION,
        "depths": depths,
        "n": n,
        "correct": correct,
        "recall": round(recall, 4),
        "rows": rows,
        "verdict": verdict,
    }

    REPORTS.mkdir(parents=True, exist_ok=True)
    out_path = REPORTS / "needle_session_v1_test.json"
    out_path.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"\nRecall: {correct}/{n} ({recall:.1%})  verdict={verdict}")
    print(f"→ {out_path}")


if __name__ == "__main__":
    main()
