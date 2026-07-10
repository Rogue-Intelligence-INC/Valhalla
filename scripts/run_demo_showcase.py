#!/usr/bin/env python3
"""Validate 3-minute demo question set against live ValhallaBase."""

from __future__ import annotations

import json
import os
import sys
import urllib.error
import urllib.request

API = os.environ.get("VALHALLA_BASE_URL", "http://127.0.0.1:8787").rstrip("/") + "/v1/infer"

CONTEXT = [
    "ValhallaBase primary protocol is valhalla-base-v1.",
    "Production decode mode is hybrid with native retrieval for open QA.",
    "Tier A RAGTruth score is 100 out of 100 grounded answers.",
    "Proactive agency uses Fate signal_tick gating with MAR cross-domain abstain.",
    "Session append sends ~64 bytes per turn instead of full prompt replay.",
]

CONFIG = {
    "decode": "hybrid",
    "proactive_enabled": True,
    "follow_up_decode": "native_follow_up_aware",
    "fate_qa_feedback": True,
}


def post(body: dict) -> dict:
    req = urllib.request.Request(
        API,
        data=json.dumps(body).encode(),
        headers={"Content-Type": "application/json"},
    )
    with urllib.request.urlopen(req, timeout=180) as resp:
        return json.loads(resp.read())


def main() -> int:
    print("=== ValhallaBase demo showcase ===")
    print(f"API: {API}")

    # Warmup
    warm = post(
        {
            "questions": [],
            "context": CONTEXT,
            "fate_warmup_only": True,
            "config": {**CONFIG, "fate_warmup_enabled": True, "fate_warmup_cycles": 4},
        }
    )
    session = warm.get("session")
    print(f"warmup: lines={warm.get('session', {}).get('lines_ingested', '?')}")

    steps = [
        ("Hybrid grounded QA", "What decode mode does ValhallaBase use?", None, False),
        ("Session follow-up", "What RAGTruth score was mentioned in our context?", session, False),
        (
            "Multimodal ingress",
            "Summarize the scene for the ingress pipeline.",
            session,
            True,
        ),
        ("Session depth", "Tell me more about session append.", session, False),
        ("Hybrid LM fallback", "hi there", session, False),
    ]

    ok = True
    for label, question, sess, multimodal in steps:
        body: dict = {
            "questions": [{"id": "q", "text": question, "score_type": "open"}],
            "config": CONFIG,
        }
        if sess:
            body["session"] = sess
        else:
            body["context"] = CONTEXT
        if multimodal:
            body["context_items"] = [{"text": "Lab demo: triad viz + protocol log visible."}]
            body["questions"][0]["audio_features"] = [0.12, 0.34, 0.56, 0.78]
            body["questions"][0]["image_features"] = [0.01, 0.02, 0.03, 0.04]

        doc = post(body)
        session = doc.get("session") or session
        row = (doc.get("rows") or [{}])[0]
        answer = str(row.get("answer") or "")
        path = str(row.get("decode_path") or "")
        reject = answer.startswith("No confident match")
        status = "PASS" if answer and not reject else "FAIL"
        if status == "FAIL":
            ok = False
        print(f"\n[{status}] {label}")
        print(f"  Q: {question}")
        print(f"  A: {answer[:120]}")
        print(f"  path: {path[:100]}")

    print("\n=== " + ("ALL PASS" if ok else "SOME FAILED") + " ===")
    return 0 if ok else 1


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except urllib.error.URLError as exc:
        print(f"ERROR: cannot reach ValhallaBase — start ./scripts/demo_valhalla_chat.sh first\n  {exc}", file=sys.stderr)
        raise SystemExit(2) from exc
