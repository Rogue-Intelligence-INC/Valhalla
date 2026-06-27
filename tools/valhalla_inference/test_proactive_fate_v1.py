#!/usr/bin/env python3
"""Proactive Fate v1 — idle self-probe + FOG-gated nudge after dialogue warmup.

Protocol: proactive-fate-v1

Batteries:
  A. cold_idle        — proactive without warmup → must NOT trigger
  B. warmup_targeted  — 2+ turns on-topic → idle should emit grounded nudge
  C. warmup_distractor — wrong-topic context → idle should abstain/defer (low false interrupt)
  D. spread_gate      — 1 turn only → must NOT trigger (min_turns)

Usage:
  python3 tools/valhalla_inference/test_proactive_fate_v1.py
"""

from __future__ import annotations

import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

REPO = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

from valhalla_base import ValhallaBase  # noqa: E402

PROTOCOL = "proactive-fate-v1"
REPORTS = REPO / "reports/valhalla_inference"

TARGET_CTX = [
    "The capital of Australia is Canberra.",
    "Australia is a country in Oceania.",
    "Canberra is located in the Australian Capital Territory.",
    "Sydney is the largest city in Australia but not the capital.",
]

DISTRACTOR_CTX = [
    "The capital of France is Paris.",
    "France is in Western Europe.",
    "Paris is on the River Seine.",
    "The Eiffel Tower is in Paris.",
]

WARMUP_TARGET = [
    "What country is Canberra in?",
    "Is Sydney the capital of Australia?",
]

WARMUP_DISTRACTOR = [
    "What is the capital of Australia?",
    "Is Canberra located in Oceania?",
]


def make_base() -> ValhallaBase:
    return ValhallaBase(
        decode="hybrid",
        follow_up_decode="native_follow_up_aware",
        fate_ingress_routing="quad_cycle",
        fate_qa_feedback=True,
        proactive_enabled=True,
        proactive_min_turns=2,
        incubation_cycles=2,
    )


def nudge_grounded(nudge: str | None, keywords: list[str]) -> bool:
    if not nudge:
        return False
    low = nudge.lower()
    return any(k.lower() in low for k in keywords)


def run_battery_cold_idle() -> dict[str, Any]:
    base = make_base()
    base.set_context(TARGET_CTX)
    try:
        base.proactive_idle()
        ok = False
        reason = "should_have_failed_no_session"
    except RuntimeError as e:
        ok = "requires an active session" in str(e)
        reason = str(e)
    return {"id": "cold_idle", "pass": ok, "detail": reason}


def run_battery_spread_gate() -> dict[str, Any]:
    base = make_base()
    base.set_context(TARGET_CTX)
    base.ask(WARMUP_TARGET[0], qid="w0")
    doc = base.proactive_idle()
    pro = doc.get("proactive") or {}
    triggered = bool(pro.get("triggered"))
    return {
        "id": "spread_gate_one_turn",
        "pass": not triggered,
        "triggered": triggered,
        "trigger_reason": pro.get("trigger_reason"),
        "proactive": pro,
    }


def run_battery_warmup(ctx: list[str], warmup: list[str], *, case_id: str, expect_emit: bool, keywords: list[str]) -> dict[str, Any]:
    base = make_base()
    base.set_context(ctx)
    for i, q in enumerate(warmup):
        base.ask(q, qid=f"w{i}")
    tel = base.session.session_state or {}
    snap = tel.get("snapshot") or {}
    prefs = snap.get("hub_prefs") or []
    spread = max(prefs) - min(prefs) if prefs else 0.0

    doc = base.proactive_idle()
    pro = doc.get("proactive") or {}
    triggered = bool(pro.get("triggered"))
    emitted = bool(pro.get("emitted"))
    nudge = pro.get("nudge")
    grounded = nudge_grounded(nudge, keywords) if emitted else None

    if expect_emit:
        passed = triggered and emitted and grounded is True
    else:
        # Mismatch warmup: no emit, or nudge must not hallucinate Australia from France-only ctx
        passed = not emitted or not nudge_grounded(nudge, ["canberra", "australia", "oceania"])

    return {
        "id": case_id,
        "pass": passed,
        "triggered": triggered,
        "emitted": emitted,
        "fog_decision": pro.get("fog_decision"),
        "trigger_reason": pro.get("trigger_reason"),
        "probe_question": pro.get("probe_question"),
        "nudge": nudge,
        "grounded": grounded,
        "hub_prefs_spread": spread,
        "proactive": pro,
    }


def main() -> None:
    batteries = [
        run_battery_cold_idle(),
        run_battery_spread_gate(),
        run_battery_warmup(
            TARGET_CTX,
            WARMUP_TARGET,
            case_id="warmup_targeted",
            expect_emit=True,
            keywords=["canberra", "australia"],
        ),
        run_battery_warmup(
            DISTRACTOR_CTX,
            WARMUP_DISTRACTOR,
            case_id="warmup_distractor",
            expect_emit=False,
            keywords=["paris"],
        ),
    ]

    passed = sum(1 for b in batteries if b.get("pass"))
    n = len(batteries)
    emit_cases = [b for b in batteries if b.get("emitted")]
    unsolicited_precision = (
        sum(1 for b in emit_cases if b.get("grounded")) / len(emit_cases)
        if emit_cases
        else None
    )

    verdict: list[str] = []
    if passed == n:
        verdict.append("PROACTIVE_FATE_V1_OK")
    else:
        verdict.append("PROACTIVE_FATE_V1_PARTIAL")

    targeted = next(b for b in batteries if b["id"] == "warmup_targeted")
    if targeted.get("emitted"):
        verdict.append("UNSolicited_EMIT_AFTER_WARMUP")
    else:
        verdict.append("NO_EMIT_AFTER_WARMUP")

    distractor = next(b for b in batteries if b["id"] == "warmup_distractor")
    if not distractor.get("emitted"):
        verdict.append("DISTRACTOR_ABSTAIN")
    elif distractor.get("pass"):
        verdict.append("DISTRACTOR_NO_HALLUCINATION")
    else:
        verdict.append("DISTRACTOR_FALSE_EMIT")

    report: dict[str, Any] = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "protocol": PROTOCOL,
        "algebra": {
            "tau_0": "12/138",
            "proactive_min_spread": 12 / 138 * 5 / 140,
            "proactive_min_turns": 2,
        },
        "batteries": batteries,
        "summary": {
            "passed": passed,
            "total": n,
            "unsolicited_precision": unsolicited_precision,
        },
        "verdict": verdict,
    }

    REPORTS.mkdir(parents=True, exist_ok=True)
    out = REPORTS / "proactive_fate_v1_test.json"
    out.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n")

    print(f"Protocol: {PROTOCOL}")
    print(f"Batteries: {passed}/{n} pass")
    for b in batteries:
        flag = "PASS" if b.get("pass") else "FAIL"
        print(f"  [{flag}] {b['id']}: triggered={b.get('triggered')} emitted={b.get('emitted')}")
        if b.get("nudge"):
            print(f"         nudge: {str(b['nudge'])[:120]}")
    print(f"Unsolicited precision: {unsolicited_precision}")
    print(f"Verdict: {', '.join(verdict)}")
    print(f"Report: {out}")

    if passed < n:
        sys.exit(1)


if __name__ == "__main__":
    main()
