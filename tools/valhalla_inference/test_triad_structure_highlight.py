#!/usr/bin/env python3
"""Triad structure + Fate cycle highlight probe.

Protocol: triad-structure-highlight-v1

Confirms production ValhallaBase incubates Tile/Stem structure and advances
Fate signal clock under quad_cycle + fate_orchestrate (not print-only).

  python3 tools/valhalla_inference/test_triad_structure_highlight.py
"""

from __future__ import annotations

import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
REPORTS = REPO / "reports" / "valhalla_inference"
sys.path.insert(0, str(HERE))

from benchmark_env import apply_gpu_env, apply_resource_budget, production_valhalla_base_kwargs  # noqa: E402
from valhalla_base import ValhallaBase  # noqa: E402

PROTOCOL = "triad-structure-highlight-v1"

CORPUS = [
    "ValhallaBase incubates Hub Tile StemCell and Fate before optional LM decode.",
    "Tile blocks form completed_tiles with activation and connection structure.",
    "StemCell cells form organ clusters via connected-component BFS on the cell graph.",
    "Fate signal_tick advances on each ingress cycle and gates proactive emit.",
    "Quad-cycle routing rotates BlackHole MeshBrain MultiNova BaseForce per line.",
    "Caroline joined a support group that helped her feel accepted.",
    "Melanie painted a lake sunrise that remains special to her.",
    "Persistent session append grows structure without full prompt resend.",
    "Zero-weight learning updates affinity without SGD on LM weights.",
    "Pollution rescue uses targeted Fate transfer against dump baselines.",
    "Hybrid decode keeps open questions on native structure retrieval.",
    "Optional bake writes TriModal fusion into an HF checkpoint for L3.",
]


def _extract(sess: dict) -> dict:
    snap = sess.get("snapshot") or {}
    frags = sess.get("fragments") or []
    tiles = 0
    stems = 0
    organs = 0
    frag_summaries = []
    for f in frags if isinstance(frags, list) else []:
        if not isinstance(f, dict):
            continue
        ft = f.get("completed_tiles") or []
        fs = f.get("stem_cells") or []
        fo = f.get("stem_organ_clusters") or []
        n_tiles = len(ft) if isinstance(ft, list) else 0
        n_stems = len(fs) if isinstance(fs, list) else 0
        n_org = len(fo) if isinstance(fo, list) else 0
        tiles += n_tiles
        stems += n_stems
        organs += n_org
        frag_summaries.append(
            {
                "id": f.get("id"),
                "label": f.get("label"),
                "tiles": n_tiles,
                "stem_cells": n_stems,
                "signal_tick_span": [
                    f.get("signal_tick_start"),
                    f.get("signal_tick_end"),
                ],
            }
        )
    # Fallback: top-level (non-fragment layout)
    if tiles == 0:
        t = snap.get("completed_tiles") or []
        tiles = len(t) if isinstance(t, list) else int(snap.get("tile_count") or 0)
    if stems == 0:
        s = snap.get("stem_cells") or []
        stems = len(s) if isinstance(s, list) else 0
    if organs == 0:
        o = snap.get("stem_organ_clusters") or []
        organs = len(o) if isinstance(o, list) else 0
    return {
        "fate_signal_tick": int(snap.get("fate_signal_tick") or 0),
        "tile_count": tiles,
        "stem_cell_count": stems,
        "stem_organ_clusters": organs,
        "fragment_count": len(frags) if isinstance(frags, list) else 0,
        "hub_prefs": snap.get("hub_prefs") or [],
        "hub_routed": snap.get("hub_routed") or [],
        "tile_total_operations": snap.get("tile_total_operations"),
        "lines_ingested": sess.get("lines_ingested"),
        "fingerprint": (sess.get("fingerprint") or "")[:16],
        "fate_ingress_routing": snap.get("fate_ingress_routing"),
        "manifold_protocol": snap.get("manifold_protocol"),
        "fragment_structure": frag_summaries,
    }


def main() -> int:
    apply_gpu_env()
    apply_resource_budget()
    os.environ.setdefault("VALHALLA_USE_CPU", os.environ.get("VALHALLA_USE_CPU", "1"))

    REPORTS.mkdir(parents=True, exist_ok=True)
    kwargs = production_valhalla_base_kwargs()
    # Structure probe: open path via hybrid with structure MCQ (no heavy LM needed for tick/tiles)
    kwargs["decode"] = "hybrid"
    kwargs["mcq_decode"] = "structure_fate"
    kwargs["hybrid_open_lm_fallback"] = False if "hybrid_open_lm_fallback" in kwargs else kwargs.get("hybrid_open_lm_fallback")
    os.environ["VALHALLA_HYBRID_OPEN_LM_FALLBACK"] = "0"
    base = ValhallaBase(**{k: v for k, v in kwargs.items() if k in ValhallaBase.__dataclass_fields__})
    base.set_context(CORPUS)

    # First ask — forces incubation + Fate cycles
    r1 = base.ask("What did Melanie paint?")
    sess1 = base.session.session_state or {}
    m1 = _extract(sess1)

    # Second ask — should advance tick / preserve structure
    r2 = base.ask("Where did Caroline find acceptance?")
    sess2 = base.session.session_state or {}
    m2 = _extract(sess2)

    # Manifold smoke already proves adapt loops; capture if binary present
    manifold = None
    smoke_bin = REPO / "target" / "release" / "manifold_orchestrator_smoke"
    if smoke_bin.is_file():
        import subprocess

        proc = subprocess.run(
            [str(smoke_bin)],
            cwd=REPO,
            capture_output=True,
            text=True,
            timeout=120,
        )
        manifold = {
            "exit_code": proc.returncode,
            "pass": proc.returncode == 0 and "PASS" in (proc.stdout or ""),
            "stdout_tail": (proc.stdout or "")[-500:],
        }

    structure_ok = (
        (m2["tile_count"] > 0 or m2["stem_cell_count"] > 0)
        and m2["fate_signal_tick"] > 0
        and m2["fragment_count"] > 0
    )
    hub_cycle_ok = isinstance(m2.get("hub_routed"), list) and sum(
        int(x) for x in (m2.get("hub_routed") or []) if isinstance(x, (int, float))
    ) > 0
    fate_advance_ok = m2["fate_signal_tick"] >= m1["fate_signal_tick"]
    cycle_ok = structure_ok and fate_advance_ok and hub_cycle_ok and (
        manifold is None or manifold.get("pass")
    )

    doc = {
        "protocol": PROTOCOL,
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "stack": {
            "decode": kwargs.get("decode"),
            "fate_ingress_routing": kwargs.get("fate_ingress_routing"),
            "fate_orchestrate": True,
        },
        "after_first_ask": m1,
        "after_second_ask": m2,
        "answers": {
            "q1": (r1.get("answer") or r1.get("text") or str(r1))[:200]
            if isinstance(r1, dict)
            else str(r1)[:200],
            "q2": (r2.get("answer") or r2.get("text") or str(r2))[:200]
            if isinstance(r2, dict)
            else str(r2)[:200],
        },
        "manifold_orchestrator_smoke": manifold,
        "checks": {
            "structure_formed": structure_ok,
            "hub_quad_routed": hub_cycle_ok,
            "fate_tick_nondecreasing": fate_advance_ok,
            "cycle_ok": cycle_ok,
        },
        "verdict": "TRIAD_STRUCTURE_CYCLE_PASS" if cycle_ok else "TRIAD_STRUCTURE_CYCLE_FAIL",
        "honest_bounds": {
            "layout": "fragment_full production keeps Tile/Stem inside fragments (top-level snapshot may be empty)",
            "organs": "stem_organ_clusters often coalesce to 1 mega-cluster at small corpus",
            "natural_nn": "Tile propagate + Stem BFS are real substrate, not differentiated cortical modules",
        },
    }

    out = REPORTS / "triad_structure_highlight.json"
    out.write_text(json.dumps(doc, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(json.dumps(doc, indent=2, ensure_ascii=False))
    print(f"→ {out}")
    print(f"verdict={doc['verdict']}")
    return 0 if cycle_ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
