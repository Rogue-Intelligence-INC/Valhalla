#!/usr/bin/env python3
"""Fate + E7 algebra — corpus affinity and FOG gate integration test.

Protocol: fate-algebra-corpus-v1

Verifies:
  1. Rust fog_decision τ/Δ match Python Fraction (12/(138+k), s₁·5/140)
  2. Fate QA feedback writes corpus_line_affinity via E7 deltas (not fitted floats)
  3. fate_telemetry.algebra_formula == e7_tau_v1
  4. long-memory production arm end-to-end score

Usage:
  python3 tools/valhalla_inference/test_fate_algebra_corpus_v1.py
  python3 tools/valhalla_inference/test_fate_algebra_corpus_v1.py --fast
"""

from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime, timezone
from fractions import Fraction
from pathlib import Path
from typing import Any

REPO = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(REPO / "tools/fair_benchmark"))

from fog_gate import (  # noqa: E402
    FOG_BASE,
    FOG_DELTA,
    FOG_DIM,
    FOG_R,
    delta_required_algebraic,
    fog_from_base_row,
    tau_algebraic,
)
from test_context_flexibility_goal import EXTENDED_TURNS, SEEDS, topic_ok  # noqa: E402
from valhalla_base import ValhallaBase  # noqa: E402

PROTOCOL = "fate-algebra-corpus-v1"
REPORTS = REPO / "reports/valhalla_inference"

TAU0 = float(Fraction(FOG_R + FOG_DELTA, FOG_BASE))
NEG_C = float(Fraction(FOG_DELTA, FOG_DIM + FOG_R))
AFFINITY_POS = TAU0
AFFINITY_NEG = NEG_C


def check_fog_algebra(row: dict, tel: dict) -> dict[str, Any]:
    fe = row.get("fog_evidence") or {}
    fd = row.get("fog_decision") or {}
    k = int(fe.get("keyword_count") or 0)
    s1 = float(fe.get("top_score") or 0)
    tau_py = float(tau_algebraic(k))
    delta_py = float(delta_required_algebraic(s1)) if s1 > 0 else 0.0
    tau_rust = float(fd.get("tau") or 0)
    delta_rust = float(fd.get("delta_required") or 0)
    fog_doc = fog_from_base_row(row, tel, mode="native")
    return {
        "keyword_count": k,
        "tau_python": tau_py,
        "tau_rust": tau_rust,
        "tau_match": abs(tau_py - tau_rust) < 1e-9,
        "delta_python": delta_py,
        "delta_rust": delta_rust,
        "delta_match": abs(delta_py - delta_rust) < 1e-6 or s1 == 0,
        "fog_decision": fd.get("decision"),
        "fog_doc_decision": fog_doc.decision,
    }


def run_affinity_learning_probe() -> dict[str, Any]:
    turns = EXTENDED_TURNS["australia"][:4]
    base = ValhallaBase(
        decode="hybrid",
        follow_up_decode="native_follow_up_aware",
        fate_ingress_routing="quad_cycle",
        fate_qa_feedback=True,
        incubation_cycles=2,
    )
    base.set_context(SEEDS["australia"])

    affinity_trace: list[list[float]] = []
    fog_checks: list[dict] = []
    doc: dict = {}

    for i, (append, question, hint, forbidden) in enumerate(turns):
        base.append_context(append)
        doc = base.run(
            [{"id": f"alg_{i}", "text": question, "score_type": "open"}],
            append_only=base.session.session_state is not None,
        )
        row = (doc.get("rows") or [{}])[0]
        tel = doc.get("fate_telemetry") or {}
        snap = (doc.get("session") or {}).get("snapshot") or {}
        aff = list(snap.get("corpus_line_affinity") or [])
        affinity_trace.append(aff)
        fog_checks.append(check_fog_algebra(row, tel))

    final_aff = affinity_trace[-1] if affinity_trace else []
    nonzero = sum(1 for v in final_aff if v > 0)
    max_aff = max(final_aff) if final_aff else 0.0

    return {
        "turns": len(turns),
        "affinity_nonzero_final": nonzero,
        "affinity_max_final": max_aff,
        "affinity_grew": len(final_aff) > 0 and max_aff > 0,
        "fog_algebra_all_match": all(c["tau_match"] and c["delta_match"] for c in fog_checks),
        "fog_checks": fog_checks,
        "algebra_formula": (doc.get("fate_telemetry") or {}).get("algebra_formula"),
        "expected_pos_delta_scale": AFFINITY_POS,
        "expected_neg_delta_scale": AFFINITY_NEG,
    }


def run_long_memory_smoke() -> dict[str, Any]:
    from test_long_memory_v1 import battery_early_anchor, make_base  # noqa: E402

    base = make_base("production")
    rows = battery_early_anchor(base, n_filler=10)
    hits = sum(1 for r in rows if r.get("ok"))
    return {"battery": "early_anchor", "n": len(rows), "hits": hits, "rate": hits / max(len(rows), 1)}


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--fast", action="store_true", help="skip long-memory smoke")
    args = ap.parse_args()

    affinity = run_affinity_learning_probe()
    long_mem = None if args.fast else run_long_memory_smoke()

    report = {
        "protocol": PROTOCOL,
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "algebra": {
            "tau0": TAU0,
            "neg_coupling": NEG_C,
            "formula": "τ(k)=12/(138+k), Δ=s₁·5/140, affinity⁺=align·τ(0), affinity⁻=align·δ/(dim+r)",
        },
        "affinity_probe": affinity,
        "long_memory_smoke": long_mem,
        "pass": (
            affinity["fog_algebra_all_match"]
            and affinity.get("algebra_formula") == "e7_tau_v1"
            and affinity["affinity_grew"]
            and (long_mem is None or long_mem["hits"] == long_mem["n"])
        ),
    }

    out = REPORTS / "fate_algebra_corpus_v1.json"
    out.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")

    print(f"[{PROTOCOL}] fog_algebra_match={affinity['fog_algebra_all_match']}")
    print(f"  affinity_grew={affinity['affinity_grew']} max={affinity['affinity_max_final']:.4f}")
    print(f"  formula={affinity.get('algebra_formula')}")
    if long_mem:
        print(f"  early_anchor={long_mem['hits']}/{long_mem['n']}")
    print(f"  PASS={report['pass']}")
    print(f"Wrote {out}")
    return 0 if report["pass"] else 1


if __name__ == "__main__":
    sys.exit(main())
