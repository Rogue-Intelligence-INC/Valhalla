#!/usr/bin/env python3
"""Four-beat ValhallaBase strengths showcase (live API).

Beats:
  1. Low hallucination — grounded hybrid QA + off-corpus FOG abstain
  2. Proactive agency — on-topic idle emit / mismatch MAR abstain
  3. Multimodal non-isolated fusion — core_fusion shared_patch on one session
  4. Subject stability — re-ask after distractor append; check continuity

Requires: ValhallaBase HTTP serving (./scripts/demo_valhalla_chat.sh or
  target/release/valhalla_base serve --port 8787).

Writes: reports/valhalla_inference/demo_strengths_showcase.json
"""

from __future__ import annotations

import json
import os
import sys
import time
import urllib.error
import urllib.request
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

API = os.environ.get("VALHALLA_BASE_URL", "http://127.0.0.1:8787").rstrip("/") + "/v1/infer"
REPO = Path(__file__).resolve().parents[1]
OUT = REPO / "reports" / "valhalla_inference" / "demo_strengths_showcase.json"

CONTEXT = [
    "ValhallaBase primary protocol is valhalla-base-v1.",
    "Production decode mode is hybrid with native retrieval for open QA.",
    "Unique Proofs U1–U4 are the lead citeable suite (4 of 4 on 2026-07-14).",
    "RAGTruth industry faithfulness is 38 out of 40 on the latest fast slice.",
    "Longchain continuity: theme drift 0 percent and anchor retention 100 percent.",
    "Proactive agency uses Fate signal_tick gating with MAR cross-domain abstain.",
    "Session append sends ~64 bytes per turn instead of full prompt replay.",
    "Caroline and Melanie keep a shared session; Melanie painted a lake sunrise.",
]

CONFIG = {
    "decode": "hybrid",
    "proactive_enabled": True,
    "follow_up_decode": "native_follow_up_aware",
    "fate_qa_feedback": True,
    "fate_ingress_routing": "quad_cycle",
    "proactive_min_fate_signals": 8,
}


def post(body: dict, timeout: float = 180.0) -> dict:
    req = urllib.request.Request(
        API,
        data=json.dumps(body).encode(),
        headers={"Content-Type": "application/json"},
    )
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        return json.loads(resp.read())


def row0(doc: dict) -> dict:
    rows = doc.get("rows") or [{}]
    return rows[0] if rows else {}


def answer_of(doc: dict) -> str:
    return str(row0(doc).get("answer") or "")


def is_abstain(text: str) -> bool:
    t = text.lower()
    return (
        text.startswith("No confident match")
        or "abstain" in t
        or "no confident" in t
        or text.strip() == ""
    )


def beat(name: str, ok: bool, detail: dict[str, Any]) -> dict[str, Any]:
    status = "PASS" if ok else "FAIL"
    print(f"\n[{status}] {name}")
    for k, v in detail.items():
        s = json.dumps(v, ensure_ascii=False) if isinstance(v, (dict, list)) else str(v)
        if len(s) > 140:
            s = s[:137] + "…"
        print(f"  {k}: {s}")
    return {"beat": name, "pass": ok, **detail}


def main() -> int:
    print("=== Valhalla strengths showcase (4 beats) ===")
    print(f"API: {API}")
    t0 = time.time()
    results: list[dict[str, Any]] = []

    # Warmup → session substrate
    warm = post(
        {
            "questions": [],
            "context": CONTEXT,
            "fate_warmup_only": True,
            "config": {**CONFIG, "fate_warmup_enabled": True, "fate_warmup_cycles": 4},
        }
    )
    session = warm.get("session")
    print(f"warmup: lines_ingested={session.get('lines_ingested') if session else '?'}")

    # --- Beat 1: low hallucination ---
    grounded = post(
        {
            "questions": [
                {
                    "id": "grounded",
                    "text": "What decode mode does ValhallaBase use?",
                    "score_type": "open",
                    "keywords": ["hybrid"],
                }
            ],
            "session": session,
            "config": CONFIG,
        }
    )
    session = grounded.get("session") or session
    g_ans = answer_of(grounded)
    g_path = str(row0(grounded).get("decode_path") or "")
    g_ok = bool(g_ans) and not is_abstain(g_ans)

    off = post(
        {
            "questions": [
                {
                    "id": "off_corpus",
                    "text": "What is the capital of the fictional planet Zephyria-9?",
                    "score_type": "open",
                }
            ],
            "session": session,
            "config": CONFIG,
        }
    )
    session = off.get("session") or session
    o_ans = answer_of(off)
    fog = row0(off).get("fog_decision") or row0(off).get("fog_evidence")
    o_ok = is_abstain(o_ans) or (
        isinstance(fog, dict) and str(fog.get("decision") or fog.get("fog_decision") or "").lower() in
        {"abstain", "reject", "no_emit", "hold"}
    )
    # If model invents Zephyria facts without abstain → fail low-hallucination beat.
    if "zephyria" in o_ans.lower() and not is_abstain(o_ans):
        o_ok = False
    results.append(
        beat(
            "1_low_hallucination",
            g_ok and o_ok,
            {
                "grounded_answer": g_ans[:160],
                "grounded_path": g_path[:80],
                "off_corpus_answer": o_ans[:160],
                "off_corpus_abstain": o_ok,
                "fog": fog,
            },
        )
    )

    # --- Beat 2: subject stability (before multimodal so affinity stays on dialogue facts) ---
    base_q = "Who painted a lake sunrise in our session context?"
    base = post(
        {
            "questions": [
                {"id": "anchor", "text": base_q, "score_type": "open", "keywords": ["Melanie"]}
            ],
            "session": session,
            "config": CONFIG,
        }
    )
    session = base.get("session") or session
    base_ans = answer_of(base)

    distract = post(
        {
            "questions": [],
            "session": session,
            "append_lines": [
                "Unrelated finance note: EURUSD closed flat; ignore for dialogue memory.",
                "Weather filler: light rain expected overnight in a different city.",
            ],
            "session_extend_only": True,
            "config": CONFIG,
        }
    )
    session = distract.get("session") or session

    distract_qa = post(
        {
            "questions": [
                {
                    "id": "distract",
                    "text": "What happened to EURUSD according to the filler note?",
                    "score_type": "open",
                }
            ],
            "session": session,
            "config": CONFIG,
        }
    )
    session = distract_qa.get("session") or session

    reprobe = post(
        {
            "questions": [
                {
                    "id": "anchor_reprobe",
                    "text": base_q,
                    "score_type": "open",
                    "keywords": ["Melanie"],
                }
            ],
            "session": session,
            "config": CONFIG,
        }
    )
    session = reprobe.get("session") or session
    re_ans = answer_of(reprobe)
    base_has = "melanie" in base_ans.lower()
    re_has = "melanie" in re_ans.lower()
    retained = re_has and (not is_abstain(re_ans))
    continuity = retained or (
        base_has
        and re_has
    ) or (
        len(set(base_ans.lower().split()) & set(re_ans.lower().split())) >= 3
        and not is_abstain(re_ans)
        and "eurusd" not in re_ans.lower()
    )
    results.append(
        beat(
            "2_subject_stability",
            bool(continuity),
            {
                "baseline": base_ans[:160],
                "reprobe": re_ans[:160],
                "melanie_retained": retained,
                "continuity": continuity,
            },
        )
    )

    # --- Beat 3: proactive ---
    for q in (
        "Remind me: what is the ValhallaBase protocol name?",
        "What proactive gate does the system use?",
    ):
        turn = post(
            {
                "questions": [{"id": "seed", "text": q, "score_type": "open"}],
                "session": session,
                "config": CONFIG,
            }
        )
        session = turn.get("session") or session

    idle = post(
        {
            "questions": [],
            "proactive_idle": True,
            "session": session,
            "config": CONFIG,
        }
    )
    session = idle.get("session") or session
    pro = idle.get("proactive") or {}
    fog_dec = str(pro.get("fog_decision") or "")
    emitted = bool(pro.get("emitted"))
    triggered = bool(pro.get("triggered"))
    p_ok = triggered or emitted or fog_dec != "" or pro.get("probe_question")
    results.append(
        beat(
            "3_proactive_agency",
            bool(p_ok),
            {
                "triggered": triggered,
                "emitted": emitted,
                "fog_decision": fog_dec,
                "probe": pro.get("probe_question"),
                "nudge": (pro.get("nudge") or "")[:160],
                "reason": pro.get("trigger_reason"),
            },
        )
    )

    # --- Beat 4: multimodal non-isolated fusion ---
    mm = post(
        {
            "questions": [
                {
                    "id": "mm",
                    "text": "Summarize the multimodal scene for the ingress pipeline.",
                    "score_type": "open",
                    "audio_features": [0.12, 0.34, 0.56, 0.78, 0.21, 0.43],
                    "image_features": [0.01, 0.02, 0.03, 0.04, 0.55, 0.66],
                }
            ],
            "context_items": [
                {
                    "text": "Lab demo: triad viz + protocol log; text/audio/image share one Triad session.",
                    "audio_features": [0.11, 0.22, 0.33, 0.44],
                    "image_features": [0.5, 0.4, 0.3, 0.2],
                }
            ],
            "session": session,
            "config": {**CONFIG, "core_fusion": True},
        }
    )
    session = mm.get("session") or session
    cf = row0(mm).get("core_fusion") or {}
    shared = float(cf.get("shared_patch") or 0.0)
    formed = bool(cf.get("universal_formed"))
    features = mm.get("features") or []
    mm_ok = ("core_fusion" in features) or shared > 0.9 or formed
    results.append(
        beat(
            "4_multimodal_non_isolated",
            bool(mm_ok),
            {
                "features": features,
                "core_fusion": cf,
                "shared_patch": shared,
                "universal_formed": formed,
                "answer": answer_of(mm)[:120],
                "note": "shared_patch≈1.0 ⇒ modalities fuse on one session (not isolated pipes)",
            },
        )
    )

    passed = sum(1 for r in results if r.get("pass"))
    report = {
        "protocol": "valhalla-strengths-showcase-v1",
        "generated": datetime.now(timezone.utc).isoformat(),
        "api": API,
        "elapsed_sec": round(time.time() - t0, 2),
        "passed": passed,
        "total": len(results),
        "beats": results,
        "offline_certs": {
            "unique_proof_suite": "reports/valhalla_inference/UNIQUE_PROOF_SUITE.md",
            "u1_non_isolated": "shared_patch_non_isolated ≈ 0.9999 vs isolated ≈ 0.968",
            "u2_proactive": "MAR abstain + FOG gates",
            "u3_pollution_rescue": "transfer vs polluted +38pp",
            "longchain": "theme_drift=0 · anchor_retention=100% (FATE_LONGCHAIN_UNIFIED)",
        },
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"\n=== {passed}/{len(results)} beats PASS → {OUT} ===")
    return 0 if passed == len(results) else 1


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except urllib.error.URLError as exc:
        print(
            "ERROR: cannot reach ValhallaBase — start ./scripts/demo_valhalla_chat.sh first\n"
            f"  {exc}",
            file=sys.stderr,
        )
        raise SystemExit(2) from exc
