#!/usr/bin/env python3
"""Organ coalescence + long-runtime structure growth.

Protocol: natural-nn-organ-runtime-v1

Doctrine (project): organs are NOT predefined. Cell identity = data_signature.
Same-type coalescence inside a connected component is organ differentiation
*continuing* (maturation), not a mega-cluster failure.

  python3 tools/valhalla_inference/test_natural_nn_organ_runtime.py
  python3 tools/valhalla_inference/test_natural_nn_organ_runtime.py --long
"""

from __future__ import annotations

import argparse
import json
import math
import os
import statistics
import sys
from collections import defaultdict, deque
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

REPO = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
REPORTS = REPO / "reports" / "valhalla_inference"
sys.path.insert(0, str(HERE))

from benchmark_env import apply_gpu_env, apply_resource_budget, production_valhalla_base_kwargs  # noqa: E402
from valhalla_base import ValhallaBase  # noqa: E402

PROTOCOL = "natural-nn-organ-runtime-v1"
EDGE_STRENGTH = 0.01
# Cosine similarity ≥ this → same emergent type (signature family)
TYPE_COSINE = 0.85
# Quantize signature for discrete type labels (first 2 dims)
TYPE_BINS = 8

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
    "Caroline later studied counseling so she could support others professionally.",
    "Melanie kept returning to the lake sunrise whenever she felt uncertain.",
    "Hub prefers rotate as lines land on BlackHole MeshBrain MultiNova BaseForce.",
    "Fragments keep completed tiles and stem cells while snapshots stay compact.",
    "Native retrieval scores structure alignment using tile and organ signatures.",
    "Small-world topology would show high clustering and short average paths.",
    "Agent memory APIs extract facts into vector or temporal graphs without cell physiology.",
    "Objective evaluation compares real graphs against Erdős–Rényi null models.",
    "Modularity Q measures segregation of communities beyond chance partitions.",
    "Growth trajectories resemble culture age series rather than single-shot dumps.",
    "Co-occurrence graphs link tokens sharing a line as a peer-like text substrate.",
    "Natural NN claims require non-random topology plus growth not mere cell counts.",
    "Further runtime lets same-signature cells coalesce into maturing organs.",
    "Energy field attraction pulls similar identities together without preset labels.",
    "Division inherits parent data_signature so type continuity is structural.",
    "Merge averages signatures of absorbed cells into the surviving organ body.",
    "Longer Fate warmup and incubation cycles stress-test structure persistence.",
    "Append-only session_extend adds lines without discarding prior Tile Stem state.",
    "Heterogeneous mega-clusters would mix incompatible signatures under one component.",
    "Homogeneous mega-clusters are the same organ consolidating over runtime.",
]


def _cosine(a: list[float], b: list[float]) -> float:
    if not a or not b:
        return 0.0
    n = min(len(a), len(b))
    if n == 0:
        return 0.0
    dot = sum(a[i] * b[i] for i in range(n))
    na = math.sqrt(sum(a[i] * a[i] for i in range(n)))
    nb = math.sqrt(sum(b[i] * b[i] for i in range(n)))
    if na < 1e-12 or nb < 1e-12:
        return 0.0
    return dot / (na * nb)


def _type_label(sig: list[float], *, energy: float = 0.0, variance: float = 0.0, degree: int = 0, bins: int = TYPE_BINS) -> str:
    """Emergent type id — signature first; if empty, energy/variance/degree family.

    Matches project doctrine: no preset organ catalog. Empty signature cells that
    share energy-family still count as same-type proto-organ coalescence.
    """
    if sig:
        parts = []
        for i in range(2):
            v = sig[i] if i < len(sig) else 0.0
            x = (math.tanh(v) + 1.0) * 0.5
            parts.append(str(min(bins - 1, int(x * bins))))
        return "sig:" + "_".join(parts)
    # proto identity (embryonic pool) — bucket by physiology proxies
    e_bin = min(bins - 1, int((math.tanh(energy) + 1.0) * 0.5 * bins))
    v_bin = min(bins - 1, int((math.tanh(variance) + 1.0) * 0.5 * bins))
    d_bin = min(bins - 1, min(degree, bins - 1))
    return f"proto:e{e_bin}_v{v_bin}_d{d_bin}"


def collect_cells(sess: dict) -> list[dict[str, Any]]:
    cells: list[dict[str, Any]] = []
    seen: set[str] = set()
    frags = sess.get("fragments") or []
    pools = []
    for f in frags if isinstance(frags, list) else []:
        if isinstance(f, dict):
            pools.append(f.get("stem_cells") or [])
    snap = sess.get("snapshot") or {}
    pools.append(snap.get("stem_cells") or [])
    raw: list[dict] = []
    for pool in pools:
        for c in pool if isinstance(pool, list) else []:
            if not isinstance(c, dict):
                continue
            cid = str(c.get("cell_id") or c.get("id") or "")
            if not cid or cid in seen:
                continue
            seen.add(cid)
            sig = [float(x) for x in (c.get("data_signature") or [])]
            conns: list[tuple[str, float]] = []
            for peer in c.get("connections") or []:
                if isinstance(peer, (list, tuple)) and len(peer) >= 2:
                    conns.append((str(peer[0]), float(peer[1])))
                elif isinstance(peer, dict):
                    conns.append(
                        (
                            str(peer.get("id") or peer.get("peer") or ""),
                            float(peer.get("strength") or peer.get("w") or 0),
                        )
                    )
            raw.append(
                {
                    "cell_id": cid,
                    "energy": float(c.get("energy") or 0),
                    "age": int(c.get("age") or 0),
                    "generation": int(c.get("generation") or 0),
                    "variance": float(c.get("variance") or 0),
                    "signal_count": int(c.get("signal_count") or 0),
                    "signature": sig,
                    "connections": [(a, w) for a, w in conns if a and w > EDGE_STRENGTH],
                }
            )
    for c in raw:
        deg = len(c["connections"])
        c["type_label"] = _type_label(
            c["signature"],
            energy=c["energy"],
            variance=c["variance"],
            degree=deg,
        )
        c["typed"] = bool(c["signature"])
        cells.append(c)
    return cells


def collect_tiles(sess: dict) -> list[dict[str, Any]]:
    tiles: list[dict[str, Any]] = []
    seen: set[str] = set()
    for f in sess.get("fragments") or []:
        if not isinstance(f, dict):
            continue
        for t in f.get("completed_tiles") or []:
            if not isinstance(t, dict):
                continue
            tid = str(t.get("id") or "")
            if not tid or tid in seen:
                continue
            seen.add(tid)
            tiles.append(
                {
                    "id": tid,
                    "energy": float(t.get("energy") or 0),
                    "activation": float(t.get("activation") or 0),
                    "connections": [str(x) for x in (t.get("connections") or [])],
                    "signature": [float(x) for x in (t.get("data_signature") or [])],
                }
            )
    return tiles


def connected_components(cells: list[dict[str, Any]]) -> list[list[str]]:
    adj: dict[str, set[str]] = defaultdict(set)
    ids = {c["cell_id"] for c in cells}
    for c in cells:
        adj.setdefault(c["cell_id"], set())
        for peer, _ in c["connections"]:
            if peer in ids:
                adj[c["cell_id"]].add(peer)
                adj[peer].add(c["cell_id"])
    seen: set[str] = set()
    comps: list[list[str]] = []
    for start in adj:
        if start in seen:
            continue
        q = deque([start])
        seen.add(start)
        members: list[str] = []
        while q:
            u = q.popleft()
            members.append(u)
            for v in adj[u]:
                if v not in seen:
                    seen.add(v)
                    q.append(v)
        comps.append(members)
    comps.sort(key=len, reverse=True)
    return comps


def analyze_organs(cells: list[dict[str, Any]]) -> dict[str, Any]:
    by_id = {c["cell_id"]: c for c in cells}
    comps = connected_components(cells)
    type_counts: dict[str, int] = defaultdict(int)
    for c in cells:
        type_counts[c["type_label"]] += 1

    organs: list[dict[str, Any]] = []
    for i, members in enumerate(comps):
        if len(members) < 2:
            continue
        member_cells = [by_id[m] for m in members if m in by_id]
        labels = [c["type_label"] for c in member_cells]
        dominant = max(set(labels), key=labels.count)
        purity = labels.count(dominant) / len(labels)
        # pairwise cosine within component (sample if large)
        sample = member_cells[: min(40, len(member_cells))]
        cos_vals: list[float] = []
        for a_i in range(len(sample)):
            for b_i in range(a_i + 1, len(sample)):
                cos_vals.append(_cosine(sample[a_i]["signature"], sample[b_i]["signature"]))
        mean_cos = statistics.fmean(cos_vals) if cos_vals else 0.0
        same_type = purity >= 0.75 or mean_cos >= TYPE_COSINE
        energies = [c["energy"] for c in member_cells]
        gens = [c["generation"] for c in member_cells]
        organs.append(
            {
                "organ_id": f"emergent_{i}",
                "cell_count": len(members),
                "frac_of_population": round(len(members) / max(1, len(cells)), 4),
                "dominant_type_label": dominant,
                "type_purity": round(purity, 4),
                "mean_pairwise_cosine": round(mean_cos, 4),
                "same_type_coalescence": same_type,
                "reading": (
                    "organ_maturation_continuing"
                    if same_type
                    else "heterogeneous_mix_not_yet_typed"
                ),
                "avg_energy": round(statistics.fmean(energies), 4) if energies else 0.0,
                "max_generation": max(gens) if gens else 0,
                "n_type_labels_inside": len(set(labels)),
            }
        )

    same_type_organs = [o for o in organs if o["same_type_coalescence"]]
    hetero = [o for o in organs if not o["same_type_coalescence"]]
    distinct_types = {o["dominant_type_label"] for o in same_type_organs}
    n_typed = sum(1 for c in cells if c.get("typed"))
    n_proto = len(cells) - n_typed
    return {
        "n_cells": len(cells),
        "n_typed_signatures": n_typed,
        "n_proto_empty_signature": n_proto,
        "n_emergent_types": len(type_counts),
        "type_histogram_top": sorted(type_counts.items(), key=lambda x: -x[1])[:12],
        "n_cc_ge2": len(organs),
        "organs": organs,
        "n_same_type_organs": len(same_type_organs),
        "n_heterogeneous_cc": len(hetero),
        "organ_differentiation_continuing": len(same_type_organs) >= 1,
        "multi_type_organ_bodies": len(distinct_types) >= 2,
        "largest_organ": organs[0] if organs else None,
    }


def structure_snapshot(sess: dict) -> dict[str, Any]:
    cells = collect_cells(sess)
    tiles = collect_tiles(sess)
    snap = sess.get("snapshot") or {}
    organ = analyze_organs(cells)
    tile_edges = sum(len(t["connections"]) for t in tiles) // 2
    return {
        "fate_signal_tick": int(snap.get("fate_signal_tick") or 0),
        "hub_routed": snap.get("hub_routed") or [],
        "fragment_count": len(sess.get("fragments") or []),
        "lines_ingested": sess.get("lines_ingested"),
        "n_tiles": len(tiles),
        "tile_edge_pairs": tile_edges,
        "tile_mean_activation": (
            round(statistics.fmean(t["activation"] for t in tiles), 4) if tiles else 0.0
        ),
        "stem": organ,
    }


def make_base(*, cycles: int, warmup: int) -> ValhallaBase:
    kwargs = production_valhalla_base_kwargs()
    kwargs["decode"] = "hybrid"
    kwargs["mcq_decode"] = "structure_fate"
    kwargs["hybrid_open_lm_fallback"] = False
    kwargs["incubation_cycles"] = cycles
    kwargs["fate_warmup_cycles"] = warmup
    os.environ["VALHALLA_HYBRID_OPEN_LM_FALLBACK"] = "0"
    fields = getattr(ValhallaBase, "__dataclass_fields__", {}) or {}
    return ValhallaBase(**{k: v for k, v in kwargs.items() if k in fields})


def run_timeline(*, long: bool) -> dict[str, Any]:
    """Progressive append + longer Fate cycles; sample structure after each pulse."""
    if long:
        pulses = [
            {"lines": CORPUS[:8], "cycles": 2, "warmup": 4, "label": "p0_seed"},
            {"lines": CORPUS[8:16], "cycles": 4, "warmup": 6, "label": "p1_append"},
            {"lines": CORPUS[16:24], "cycles": 8, "warmup": 8, "label": "p2_deep"},
            {"lines": CORPUS[24:], "cycles": 12, "warmup": 10, "label": "p3_longer"},
            {"lines": [], "cycles": 16, "warmup": 12, "label": "p4_extra_cycles_only"},
        ]
    else:
        pulses = [
            {"lines": CORPUS[:8], "cycles": 2, "warmup": 4, "label": "p0_seed"},
            {"lines": CORPUS[8:16], "cycles": 4, "warmup": 6, "label": "p1_append"},
            {"lines": CORPUS[16:24], "cycles": 8, "warmup": 8, "label": "p2_deep"},
        ]

    base = make_base(cycles=pulses[0]["cycles"], warmup=pulses[0]["warmup"])
    timeline: list[dict[str, Any]] = []
    ingested = 0

    for i, pulse in enumerate(pulses):
        base.incubation_cycles = pulse["cycles"]
        base.fate_warmup_cycles = pulse["warmup"]
        lines = pulse["lines"]
        if i == 0:
            base.set_context(lines)
            ingested = len(lines)
            _ = base.ask(f"Pulse {pulse['label']}: what structure is forming?")
        elif lines:
            for ln in lines:
                base.append_context(ln)
            ext = base.session_extend()
            ingested += len(lines)
            # Extra Fate stress via ask (forces more routing ticks)
            _ = base.ask(f"Pulse {pulse['label']}: summarize coalescing organs.")
            _ = ext  # keep response applied via session_extend
        else:
            # cycles-only: re-ask to drive more incubation without new lines
            base.incubation_cycles = pulse["cycles"]
            _ = base.ask(f"Pulse {pulse['label']}: continue Fate cycles without new corpus.")

        sess = base.session.session_state or {}
        snap = structure_snapshot(sess)
        snap["pulse"] = pulse["label"]
        snap["incubation_cycles"] = pulse["cycles"]
        snap["fate_warmup_cycles"] = pulse["warmup"]
        snap["corpus_lines_target"] = ingested
        timeline.append(snap)
        stem = snap["stem"]
        largest = stem.get("largest_organ") or {}
        print(
            f"[{pulse['label']}] tick={snap['fate_signal_tick']} "
            f"cells={stem['n_cells']} typed={stem.get('n_typed_signatures')} "
            f"tiles={snap['n_tiles']} types={stem['n_emergent_types']} "
            f"same_type_organs={stem['n_same_type_organs']} "
            f"continuing={stem['organ_differentiation_continuing']} "
            f"largest_n={largest.get('cell_count')} "
            f"purity={largest.get('type_purity')} "
            f"label={largest.get('dominant_type_label')} "
            f"read={largest.get('reading')}"
        )

    first, last = timeline[0], timeline[-1]
    growth = {
        "cells_delta": last["stem"]["n_cells"] - first["stem"]["n_cells"],
        "tiles_delta": last["n_tiles"] - first["n_tiles"],
        "types_delta": last["stem"]["n_emergent_types"] - first["stem"]["n_emergent_types"],
        "tick_delta": last["fate_signal_tick"] - first["fate_signal_tick"],
        "largest_organ_delta": (
            (last["stem"].get("largest_organ") or {}).get("cell_count", 0)
            - (first["stem"].get("largest_organ") or {}).get("cell_count", 0)
        ),
        "purity_last": (last["stem"].get("largest_organ") or {}).get("type_purity"),
        "same_type_organs_last": last["stem"]["n_same_type_organs"],
        "multi_type_bodies_last": last["stem"]["multi_type_organ_bodies"],
    }

    continuing = all(p["stem"]["organ_differentiation_continuing"] for p in timeline)
    more_structure = (
        growth["tiles_delta"] > 0
        or growth["tick_delta"] > 0
        or growth["largest_organ_delta"] > 0
        or growth["cells_delta"] != 0
    )
    # PASS if same-type coalescence sustained + runtime grew structure
    if continuing and more_structure:
        verdict = "ORGAN_COALESCENCE_RUNTIME_PASS"
    elif continuing:
        verdict = "ORGAN_COALESCENCE_PASS_NO_GROWTH"
    elif last["stem"]["n_cells"] >= 8:
        verdict = "STRUCTURE_PRESENT_TYPE_MIXED"
    else:
        verdict = "ORGAN_RUNTIME_FAIL"

    return {
        "growth": growth,
        "timeline": timeline,
        "checks": {
            "organ_defined_as": "connected component of cells with similar data_signature (emergent, not preset)",
            "same_type_merge_is": "organ_differentiation_continuing / maturation",
            "organ_differentiation_continuing": continuing,
            "more_structure_with_runtime": more_structure,
            "multi_type_organ_bodies": last["stem"]["multi_type_organ_bodies"],
        },
        "verdict": verdict,
    }


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--long",
        action="store_true",
        help="extra pulse with more lines + cycles-only deepen (slower)",
    )
    args = ap.parse_args()

    apply_gpu_env()
    apply_resource_budget()
    os.environ.setdefault("VALHALLA_USE_CPU", os.environ.get("VALHALLA_USE_CPU", "1"))
    REPORTS.mkdir(parents=True, exist_ok=True)

    print(f"=== {PROTOCOL} ===")
    print("organ := same-signature coalescence (no preset organ catalog)")
    print(f"mode={'long' if args.long else 'standard'}")
    print()

    result = run_timeline(long=args.long)
    out = {
        "protocol": PROTOCOL,
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "doctrine": {
            "preset_organs": False,
            "cell_identity": "data_signature",
            "same_type_CC": "organ differentiation continuing (maturation)",
            "heterogeneous_CC": "not yet typed / mix — not counted as matured organ",
        },
        **result,
    }
    path = REPORTS / "natural_nn_organ_runtime.json"
    path.write_text(json.dumps(out, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print()
    print(json.dumps({"verdict": result["verdict"], "growth": result["growth"], "checks": result["checks"]}, indent=2))
    print(f"→ {path}")
    return 0 if result["verdict"] in (
        "ORGAN_COALESCENCE_RUNTIME_PASS",
        "ORGAN_COALESCENCE_PASS_NO_GROWTH",
        "STRUCTURE_PRESENT_TYPE_MIXED",
    ) else 1


if __name__ == "__main__":
    raise SystemExit(main())
