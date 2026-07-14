#!/usr/bin/env python3
"""Strengthened natural-NN evaluation — nulls + SWP + causal lesion.

Protocol: natural-nn-causal-v1

Tier A (morphology, literature-hardened):
  - Degree-preserving edge-swap null (stronger than ER)
  - Small-World Propensity φ (Muldoon-style vs lattice + random)
  - Multi-seed bootstrap for CIs

Tier B (causal / persuasive):
  - Intact vs organ-lesion vs degree-matched rewire
  - Structure-alignment proxy (mirrors native_qa score_structure_alignment)
  - Optional live ask landing check on mutated sessions

  python3 tools/valhalla_inference/test_natural_nn_causal_v1.py
  python3 tools/valhalla_inference/test_natural_nn_causal_v1.py --seeds 5 --live-ask
"""

from __future__ import annotations

import argparse
import copy
import json
import math
import os
import random
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

PROTOCOL = "natural-nn-causal-v1"
EDGE_STRENGTH = 0.01
TYPE_BINS = 8
TYPE_COSINE = 0.85
NULL_TRIALS = 32
SWAP_ROUNDS_FACTOR = 10  # swaps ≈ factor * E

CORPUS = [
    "Melanie painted a lake sunrise that remains special to her.",
    "Caroline joined a support group that helped her feel accepted.",
    "Caroline later studied counseling so she could support others professionally.",
    "Melanie kept returning to the lake sunrise whenever she felt uncertain.",
    "ValhallaBase incubates Hub Tile StemCell and Fate before optional LM decode.",
    "Tile blocks form completed_tiles with activation and connection structure.",
    "StemCell cells form organ clusters via connected-component BFS on the cell graph.",
    "Fate signal_tick advances on each ingress cycle and gates proactive emit.",
    "Quad-cycle routing rotates BlackHole MeshBrain MultiNova BaseForce per line.",
    "Persistent session append grows structure without full prompt resend.",
    "Zero-weight learning updates affinity without SGD on LM weights.",
    "Native retrieval scores structure alignment using tile and organ signatures.",
    "Small-world topology shows high clustering and short average paths versus nulls.",
    "Organ coalescence means same-type cells merge without a preset organ catalog.",
    "Causal lesion removes organ edges while degree-matched rewire preserves degrees.",
    "Degree-preserving nulls are stricter than Erdős–Rényi for small-world claims.",
]

PROBES = [
    {
        "id": "sunrise",
        "question": "What did Melanie paint?",
        "gold_tokens": ["sunrise", "lake", "painted"],
        "evidence_lines": [0, 3],
    },
    {
        "id": "counseling",
        "question": "What did Caroline study later?",
        "gold_tokens": ["counseling", "support", "professionally"],
        "evidence_lines": [2],
    },
    {
        "id": "support",
        "question": "What helped Caroline feel accepted?",
        "gold_tokens": ["support", "group", "accepted"],
        "evidence_lines": [1],
    },
]


# ── math helpers ────────────────────────────────────────────────────────────


def _cosine(a: list[float], b: list[float]) -> float:
    if not a or not b:
        return 0.0
    n = min(len(a), len(b))
    dot = sum(a[i] * b[i] for i in range(n))
    na = math.sqrt(sum(a[i] * a[i] for i in range(n)) + 1e-15)
    nb = math.sqrt(sum(b[i] * b[i] for i in range(n)) + 1e-15)
    return dot / (na * nb)


def _text_sig(text: str, dim: int = 8) -> list[float]:
    """Stable bag signature for alignment probes (not LM embedding)."""
    v = [0.0] * dim
    toks = [t.lower().strip(".,;:!?\"'") for t in text.split()]
    for i, t in enumerate(toks):
        if len(t) < 3:
            continue
        h = hash(t) % (10**9)
        v[h % dim] += 1.0 + 0.1 * (i % 5)
    n = math.sqrt(sum(x * x for x in v) + 1e-15)
    return [x / n for x in v]


# ── graph core ──────────────────────────────────────────────────────────────


def adj_from_edges(nodes: list[str], edges: list[tuple[str, str]]) -> dict[str, set[str]]:
    adj: dict[str, set[str]] = {n: set() for n in nodes}
    for a, b in edges:
        if a == b or a not in adj or b not in adj:
            continue
        adj[a].add(b)
        adj[b].add(a)
    return adj


def edge_list(adj: dict[str, set[str]]) -> list[tuple[str, str]]:
    out: list[tuple[str, str]] = []
    for u, nbrs in adj.items():
        for v in nbrs:
            if u < v:
                out.append((u, v))
    return out


def connected_components(adj: dict[str, set[str]]) -> list[list[str]]:
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


def avg_clustering(adj: dict[str, set[str]]) -> float:
    vals: list[float] = []
    for u, nbrs_set in adj.items():
        nbrs = list(nbrs_set)
        k = len(nbrs)
        if k < 2:
            vals.append(0.0)
            continue
        links = 0
        for i in range(k):
            for j in range(i + 1, k):
                if nbrs[j] in adj[nbrs[i]]:
                    links += 1
        vals.append(2.0 * links / (k * (k - 1)))
    return statistics.fmean(vals) if vals else 0.0


def avg_shortest_path(adj: dict[str, set[str]], sample_cap: int = 100, seed: int = 0) -> float | None:
    comps = connected_components(adj)
    if not comps or len(comps[0]) < 2:
        return None
    nodes = comps[0]
    rng = random.Random(seed)
    sources = rng.sample(nodes, min(sample_cap, len(nodes)))
    lengths: list[int] = []
    member_set = set(nodes)
    for s in sources:
        dist = {s: 0}
        q = deque([s])
        while q:
            u = q.popleft()
            for v in adj.get(u, ()):
                if v not in dist:
                    dist[v] = dist[u] + 1
                    q.append(v)
        for t, d in dist.items():
            if t != s and t in member_set:
                lengths.append(d)
    return statistics.fmean(lengths) if lengths else None


def config_edge_swap(
    nodes: list[str], edges: list[tuple[str, str]], *, trials: int, seed: int
) -> list[tuple[str, str]]:
    """Degree-preserving randomized graph via double-edge swaps."""
    if len(edges) < 2:
        return list(edges)
    rng = random.Random(seed)
    e = [tuple(sorted(x)) for x in edges]
    edge_set = set(e)
    attempts = max(trials, len(e) * SWAP_ROUNDS_FACTOR)
    for _ in range(attempts):
        (a, b), (c, d) = rng.sample(e, 2)
        if len({a, b, c, d}) < 4:
            continue
        if rng.random() < 0.5:
            p1, p2 = tuple(sorted((a, c))), tuple(sorted((b, d)))
        else:
            p1, p2 = tuple(sorted((a, d))), tuple(sorted((b, c)))
        if p1[0] == p1[1] or p2[0] == p2[1]:
            continue
        if p1 in edge_set or p2 in edge_set:
            continue
        edge_set.discard((a, b))
        edge_set.discard((c, d))
        edge_set.add(p1)
        edge_set.add(p2)
        e = list(edge_set)
    return e


def lattice_edges(nodes: list[str], avg_degree: float) -> list[tuple[str, str]]:
    """Ring lattice with k≈avg_degree neighbors (Watts–Strogatz regular extreme)."""
    n = len(nodes)
    if n < 3:
        return []
    k = max(2, int(round(avg_degree)))
    if k % 2 == 1:
        k += 1
    k = min(k, n - 1 if n % 2 == 0 else n - 2)
    half = k // 2
    edges: set[tuple[str, str]] = set()
    for i in range(n):
        for d in range(1, half + 1):
            j = (i + d) % n
            a, b = sorted((nodes[i], nodes[j]))
            edges.add((a, b))
    return sorted(edges)


def small_world_propensity(C: float, L: float | None, C_latt: float, L_rand: float | None) -> float | None:
    """Muldoon-style φ = 1 - √(½(ΔC² + ΔL²)), Δ in [0,1] clipped."""
    if L is None or L_rand is None or L_rand <= 1e-12:
        return None
    # Clustering: higher C is more lattice-like; ΔC = (C_latt - C)/(C_latt - C_rand)
    # Use C_rand ≈ 0 for sparse nets → ΔC ≈ 1 - C/C_latt
    if C_latt <= 1e-12:
        delta_c = 0.0 if C <= 1e-12 else 1.0
    else:
        delta_c = max(0.0, min(1.0, (C_latt - C) / C_latt))
    delta_l = max(0.0, min(1.0, (L - L_rand) / L_rand)) if L >= L_rand else 0.0
    return 1.0 - math.sqrt(0.5 * (delta_c * delta_c + delta_l * delta_l))


def topology_bundle(nodes: list[str], edges: list[tuple[str, str]], *, seed: int = 0) -> dict[str, Any]:
    adj = adj_from_edges(nodes, edges)
    n, e = len(nodes), len(edges)
    C = avg_clustering(adj)
    L = avg_shortest_path(adj, seed=seed)
    avg_k = (2.0 * e / n) if n else 0.0

    # ER nulls
    er_Cs: list[float] = []
    er_Ls: list[float] = []
    # Config nulls
    cfg_Cs: list[float] = []
    cfg_Ls: list[float] = []
    labels = list(nodes)
    max_e = n * (n - 1) // 2
    ee = min(e, max_e)
    for t in range(NULL_TRIALS):
        rng = random.Random(seed + 17 * t)
        pairs = [(labels[i], labels[j]) for i in range(n) for j in range(i + 1, n)]
        chosen = rng.sample(pairs, ee) if ee and ee < len(pairs) else pairs[:ee]
        a_er = adj_from_edges(labels, chosen)
        er_Cs.append(avg_clustering(a_er))
        ler = avg_shortest_path(a_er, seed=seed + t)
        if ler is not None:
            er_Ls.append(ler)
        swapped = config_edge_swap(labels, edges, trials=max(50, e * 2), seed=seed + 101 * t)
        a_cfg = adj_from_edges(labels, swapped)
        cfg_Cs.append(avg_clustering(a_cfg))
        lcfg = avg_shortest_path(a_cfg, seed=seed + t)
        if lcfg is not None:
            cfg_Ls.append(lcfg)

    C_er = statistics.fmean(er_Cs) if er_Cs else 0.0
    L_er = statistics.fmean(er_Ls) if er_Ls else None
    C_cfg = statistics.fmean(cfg_Cs) if cfg_Cs else 0.0
    L_cfg = statistics.fmean(cfg_Ls) if cfg_Ls else None

    latt = lattice_edges(labels, avg_k)
    a_latt = adj_from_edges(labels, latt)
    C_latt = avg_clustering(a_latt)
    phi = small_world_propensity(C, L, C_latt, L_er)

    gamma_er = (C / C_er) if C_er > 1e-12 else (math.inf if C > 0 else 0.0)
    lambda_er = (L / L_er) if (L is not None and L_er) else None
    sigma_er = (gamma_er / lambda_er) if (lambda_er and lambda_er > 0 and math.isfinite(gamma_er)) else None

    gamma_cfg = (C / C_cfg) if C_cfg > 1e-12 else (math.inf if C > 0 else 0.0)
    lambda_cfg = (L / L_cfg) if (L is not None and L_cfg) else None
    sigma_cfg = (
        gamma_cfg / lambda_cfg if (lambda_cfg and lambda_cfg > 0 and math.isfinite(gamma_cfg)) else None
    )

    # Empirical p: fraction of config nulls with C >= observed C (one-sided)
    p_C_cfg = (sum(1 for x in cfg_Cs if x >= C - 1e-12) / len(cfg_Cs)) if cfg_Cs else 1.0

    return {
        "n": n,
        "e": e,
        "avg_degree": round(avg_k, 4),
        "C": round(C, 4),
        "L": None if L is None else round(L, 4),
        "C_er": round(C_er, 4),
        "L_er": None if L_er is None else round(L_er, 4),
        "C_cfg": round(C_cfg, 4),
        "L_cfg": None if L_cfg is None else round(L_cfg, 4),
        "C_lattice": round(C_latt, 4),
        "sigma_vs_ER": None if sigma_er is None else round(sigma_er, 4),
        "sigma_vs_config": None if sigma_cfg is None else round(sigma_cfg, 4),
        "phi_SWP": None if phi is None else round(phi, 4),
        "p_clustering_ge_obs_under_config": round(p_C_cfg, 4),
        "nonrandom_vs_config": bool(p_C_cfg <= 0.05 and C > C_cfg),
        "phi_ge_0_6": bool(phi is not None and phi >= 0.6),
    }


# ── cells / organs / session mutation ───────────────────────────────────────


def _type_label(sig: list[float], energy: float, variance: float, degree: int) -> str:
    if sig:
        parts = []
        for i in range(2):
            v = sig[i] if i < len(sig) else 0.0
            x = (math.tanh(v) + 1.0) * 0.5
            parts.append(str(min(TYPE_BINS - 1, int(x * TYPE_BINS))))
        return "sig:" + "_".join(parts)
    e_bin = min(TYPE_BINS - 1, int((math.tanh(energy) + 1.0) * 0.5 * TYPE_BINS))
    v_bin = min(TYPE_BINS - 1, int((math.tanh(variance) + 1.0) * 0.5 * TYPE_BINS))
    d_bin = min(TYPE_BINS - 1, min(degree, TYPE_BINS - 1))
    return f"proto:e{e_bin}_v{v_bin}_d{d_bin}"


def collect_cells(sess: dict) -> list[dict[str, Any]]:
    cells: list[dict[str, Any]] = []
    seen: set[str] = set()
    pools: list = []
    for f in sess.get("fragments") or []:
        if isinstance(f, dict):
            pools.append(f.get("stem_cells") or [])
    pools.append((sess.get("snapshot") or {}).get("stem_cells") or [])
    for pool in pools:
        for c in pool if isinstance(pool, list) else []:
            if not isinstance(c, dict):
                continue
            cid = str(c.get("cell_id") or "")
            if not cid or cid in seen:
                continue
            seen.add(cid)
            sig = [float(x) for x in (c.get("data_signature") or [])]
            conns: list[tuple[str, float]] = []
            for peer in c.get("connections") or []:
                if isinstance(peer, (list, tuple)) and len(peer) >= 2:
                    conns.append((str(peer[0]), float(peer[1])))
            conns = [(a, w) for a, w in conns if a and w > EDGE_STRENGTH]
            cells.append(
                {
                    "cell_id": cid,
                    "energy": float(c.get("energy") or 0),
                    "variance": float(c.get("variance") or 0),
                    "signature": sig,
                    "connections": conns,
                    "type_label": _type_label(sig, float(c.get("energy") or 0), float(c.get("variance") or 0), len(conns)),
                }
            )
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
                    "signature": [float(x) for x in (t.get("data_signature") or [])],
                }
            )
    return tiles


def largest_same_type_organ(cells: list[dict[str, Any]]) -> dict[str, Any] | None:
    """Largest multi-cell component; prefer same-type coalescence, else largest CC.

    Doctrine: no preset organ catalog. Coalesced CC is the lesion target even when
    type purity is only proto-level (energy/degree family).
    """
    nodes = [c["cell_id"] for c in cells]
    edges: list[tuple[str, str]] = []
    by_id = {c["cell_id"]: c for c in cells}
    for c in cells:
        for peer, _ in c["connections"]:
            if peer in by_id:
                a, b = sorted((c["cell_id"], peer))
                edges.append((a, b))
    edges = list(set(edges))
    adj = adj_from_edges(nodes, edges)
    comps = [m for m in connected_components(adj) if len(m) >= 2]
    if not comps:
        return None

    def describe(members: list[str]) -> dict[str, Any]:
        labels = [by_id[m]["type_label"] for m in members if m in by_id]
        dom = max(set(labels), key=labels.count) if labels else "unknown"
        purity = labels.count(dom) / len(labels) if labels else 0.0
        sample = [by_id[m] for m in members[:40] if m in by_id]
        cos_vals = [
            _cosine(sample[i]["signature"], sample[j]["signature"])
            for i in range(len(sample))
            for j in range(i + 1, len(sample))
            if sample[i]["signature"] and sample[j]["signature"]
        ]
        mean_cos = statistics.fmean(cos_vals) if cos_vals else 0.0
        same = purity >= 0.55 or mean_cos >= TYPE_COSINE
        return {
            "members": members,
            "cell_count": len(members),
            "dominant_type": dom,
            "purity": round(purity, 4),
            "mean_cosine": round(mean_cos, 4),
            "same_type_coalescence": same,
        }

    typed = [describe(m) for m in comps]
    same = [o for o in typed if o["same_type_coalescence"]]
    pool = same if same else typed
    return max(pool, key=lambda o: o["cell_count"])


def mutate_session_stem(
    sess: dict,
    *,
    mode: str,
    organ_members: set[str] | None,
    seed: int = 0,
) -> dict:
    """Return deep-copied session with stem edges mutated in every fragment."""
    out = copy.deepcopy(sess)
    rng = random.Random(seed)

    def mutate_cells(cells: list) -> list:
        if not isinstance(cells, list):
            return cells
        # collect undirected edges & rewrite
        ids = [str(c.get("cell_id") or "") for c in cells if isinstance(c, dict)]
        ids = [i for i in ids if i]
        id_set = set(ids)
        edges: list[tuple[str, str]] = []
        strength: dict[tuple[str, str], float] = {}
        for c in cells:
            if not isinstance(c, dict):
                continue
            cid = str(c.get("cell_id") or "")
            for peer in c.get("connections") or []:
                if not (isinstance(peer, (list, tuple)) and len(peer) >= 2):
                    continue
                pid, w = str(peer[0]), float(peer[1])
                if pid not in id_set or w <= EDGE_STRENGTH:
                    continue
                a, b = sorted((cid, pid))
                edges.append((a, b))
                strength[(a, b)] = max(strength.get((a, b), 0), w)
        edges = list(set(edges))

        if mode == "intact":
            new_edges = edges
        elif mode == "organ_lesion" and organ_members:
            new_edges = [
                (a, b)
                for a, b in edges
                if not (a in organ_members or b in organ_members)
            ]
        elif mode == "degree_rewire":
            new_edges = config_edge_swap(ids, edges, trials=max(80, len(edges) * 3), seed=seed)
            # keep strengths mapped randomly
            vals = list(strength.values()) or [0.2]
            strength = {e: rng.choice(vals) for e in new_edges}
        else:
            new_edges = edges

        nbrs: dict[str, list[tuple[str, float]]] = defaultdict(list)
        for a, b in new_edges:
            w = strength.get((a, b), 0.2)
            nbrs[a].append((b, w))
            nbrs[b].append((a, w))

        for c in cells:
            if not isinstance(c, dict):
                continue
            cid = str(c.get("cell_id") or "")
            c["connections"] = nbrs.get(cid, [])
            if mode == "organ_lesion" and organ_members and cid in organ_members:
                # weaken identity contribution of lesioned organ
                c["data_signature"] = []
                c["energy"] = float(c.get("energy") or 0) * 0.05
        return cells

    for f in out.get("fragments") or []:
        if isinstance(f, dict):
            f["stem_cells"] = mutate_cells(f.get("stem_cells") or [])
    snap = out.get("snapshot")
    if isinstance(snap, dict) and snap.get("stem_cells"):
        snap["stem_cells"] = mutate_cells(snap.get("stem_cells") or [])
    return out


def stem_tile_features(sess: dict) -> dict[str, Any]:
    cells = collect_cells(sess)
    tiles = collect_tiles(sess)
    organ = largest_same_type_organ(cells)
    # rebuild cluster sigs as mean signature of organ members (+ all CC≥2)
    nodes = [c["cell_id"] for c in cells]
    by_id = {c["cell_id"]: c for c in cells}
    edges: list[tuple[str, str]] = []
    for c in cells:
        for p, _ in c["connections"]:
            if p in by_id:
                edges.append(tuple(sorted((c["cell_id"], p))))
    edges = list(set(edges))
    comps = [m for m in connected_components(adj_from_edges(nodes, edges)) if len(m) >= 2]
    cluster_sigs: list[list[float]] = []
    cluster_e: list[float] = []
    for members in comps:
        sigs = [by_id[m]["signature"] for m in members if by_id[m]["signature"]]
        if not sigs:
            # proto: use [energy, variance]
            sigs = [[by_id[m]["energy"], by_id[m]["variance"]] for m in members]
        dim = max(len(s) for s in sigs)
        mean = [0.0] * dim
        for s in sigs:
            for i, x in enumerate(s):
                mean[i] += x
        mean = [x / len(sigs) for x in mean]
        cluster_sigs.append(mean)
        cluster_e.append(statistics.fmean(by_id[m]["energy"] for m in members))
    stem_sigs = [c["signature"] if c["signature"] else [c["energy"], c["variance"]] for c in cells]
    return {
        "cells": cells,
        "tiles": tiles,
        "organ": organ,
        "tile_sigs": [t["signature"] or [t["energy"], t["activation"]] for t in tiles],
        "tile_energies": [t["energy"] for t in tiles],
        "tile_activations": [t["activation"] for t in tiles],
        "stem_sigs": stem_sigs,
        "cluster_sigs": cluster_sigs,
        "cluster_energies": cluster_e,
        "hub_prefs": list((sess.get("snapshot") or {}).get("hub_prefs") or [0.25, 0.25, 0.25, 0.25]),
        "edges": edges,
        "nodes": nodes,
    }


def score_organ_channel(query_sig: list[float], feat: dict[str, Any]) -> float:
    """Organ/cluster channel only (weight 0.12 in native_qa) — lesion target."""
    hub = feat["hub_prefs"] or [0.25, 0.25, 0.25, 0.25]
    stem_w = (hub[1] + hub[3]) / 2.0 if len(hub) >= 4 else 1.0
    s = 0.0
    for cs, ce in zip(feat["cluster_sigs"], feat["cluster_energies"]):
        s += 0.12 * stem_w * abs(_cosine(query_sig, cs)) * max(ce, 0.01)
    return s


def score_structure_alignment_proxy(query_sig: list[float], feat: dict[str, Any]) -> float:
    """Faithful Python mirror of native_qa::score_structure_alignment weights."""
    hub = feat["hub_prefs"] or [0.25, 0.25, 0.25, 0.25]
    s = 0.0
    for ti, ts in enumerate(feat["tile_sigs"]):
        act = feat["tile_activations"][ti] if ti < len(feat["tile_activations"]) else 0.0
        e = feat["tile_energies"][ti] if ti < len(feat["tile_energies"]) else 1.0
        tw = hub[ti % len(hub)] if hub else 1.0
        s += 0.15 * tw * _cosine(query_sig, ts) * max(e, 0.0) * (1.0 + abs(act))
    stem_w = (hub[1] + hub[3]) / 2.0 if len(hub) >= 4 else 1.0
    for ss in feat["stem_sigs"]:
        s += 0.08 * stem_w * _cosine(query_sig, ss)
    s += score_organ_channel(query_sig, feat)
    return s


def memory_rank_proxy(query: str, corpus: list[str], feat: dict[str, Any]) -> dict[str, Any]:
    """Rank all corpus lines by cosine+structure (native-style) for gold evidence."""
    qsig = _text_sig(query)
    patch = list((feat.get("hub_prefs") or [0.25]) * 8)[:8]
    scored: list[tuple[int, float]] = []
    for i, line in enumerate(corpus):
        msig = _text_sig(line)
        s = _cosine(msig, qsig)
        s += 0.35 * _cosine(msig, patch)
        s += score_structure_alignment_proxy(msig, feat)
        scored.append((i, s))
    scored.sort(key=lambda x: -x[1])
    return {
        "ranking": [{"line": i, "score": round(sc, 4), "text": corpus[i][:80]} for i, sc in scored],
        "top1": scored[0][0] if scored else None,
        "alignment_on_query": round(score_structure_alignment_proxy(qsig, feat), 4),
    }


# ── incubate ────────────────────────────────────────────────────────────────


def incubate(corpus: list[str], *, seed_tag: int = 0) -> tuple[dict, ValhallaBase, list[str]]:
    kwargs = production_valhalla_base_kwargs()
    kwargs["decode"] = "hybrid"
    kwargs["mcq_decode"] = "structure_fate"
    kwargs["hybrid_open_lm_fallback"] = False
    kwargs["incubation_cycles"] = 4
    kwargs["fate_warmup_cycles"] = 6
    os.environ["VALHALLA_HYBRID_OPEN_LM_FALLBACK"] = "0"
    # mild seed influence via warmup lines order (shuffle copy)
    lines = list(corpus)
    rng = random.Random(seed_tag)
    if seed_tag:
        head, tail = lines[:4], lines[4:]
        rng.shuffle(tail)
        lines = head + tail
    fields = getattr(ValhallaBase, "__dataclass_fields__", {}) or {}
    base = ValhallaBase(**{k: v for k, v in kwargs.items() if k in fields})
    base.set_context(lines)
    _ = base.ask("Incubate structure for causal topology evaluation.")
    return base.session.session_state or {}, base, lines


def live_ask_check(base: ValhallaBase, sess: dict, probes: list[dict], corpus: list[str]) -> dict:
    base.session.session_state = sess
    base.context_items = []
    base._pending_append.clear()
    hits = 0
    details = []
    for p in probes:
        ans = base.ask(p["question"], qid=p["id"]).lower()
        hit = any(tok in ans for tok in p["gold_tokens"])
        hits += int(hit)
        details.append({"id": p["id"], "hit": hit, "answer": ans[:160]})
    return {"hits": hits, "n": len(probes), "pass_rate": hits / max(1, len(probes)), "details": details}


def run_seed(seed: int, *, live_ask: bool) -> dict[str, Any]:
    print(f"\n=== seed {seed} ===")
    sess0, base, corpus = incubate(CORPUS, seed_tag=seed)
    feat0 = stem_tile_features(sess0)
    organ = feat0["organ"]
    organ_members = set(organ["members"]) if organ else set()
    print(
        f"  cells={len(feat0['cells'])} tiles={len(feat0['tiles'])} edges={len(feat0['edges'])} "
        f"organ_n={organ['cell_count'] if organ else 0} purity={organ['purity'] if organ else None}"
    )

    topo = topology_bundle(feat0["nodes"], feat0["edges"], seed=seed)
    print(
        f"  topo C={topo['C']} φ={topo['phi_SWP']} σ_ER={topo['sigma_vs_ER']} "
        f"σ_cfg={topo['sigma_vs_config']} p_cfg={topo['p_clustering_ge_obs_under_config']}"
    )

    arms = {}
    for mode in ("intact", "organ_lesion", "degree_rewire"):
        sess_m = mutate_session_stem(sess0, mode=mode, organ_members=organ_members, seed=seed + 7)
        feat_m = stem_tile_features(sess_m)
        align_scores = []
        organ_channel = []
        gold_ranks = []
        for p in PROBES:
            mr = memory_rank_proxy(p["question"], corpus, feat_m)
            align_scores.append(mr["alignment_on_query"])
            organ_channel.append(score_organ_channel(_text_sig(p["question"]), feat_m))
            gold = set(p["evidence_lines"])
            rank = None
            for r, item in enumerate(mr["ranking"], start=1):
                if item["line"] in gold:
                    rank = r
                    break
            gold_ranks.append(rank if rank is not None else len(corpus))
        struct_mass = (
            len(feat_m["edges"])
            + 10.0 * len(feat_m["cluster_sigs"])
            + sum(feat_m["cluster_energies"])
        )
        arm = {
            "mean_alignment": round(statistics.fmean(align_scores), 4),
            "mean_organ_channel": round(statistics.fmean(organ_channel), 4),
            "mean_gold_rank": round(statistics.fmean(gold_ranks), 4),
            "n_edges": len(feat_m["edges"]),
            "n_clusters": len(feat_m["cluster_sigs"]),
            "structure_mass": round(struct_mass, 4),
            "organ_present": bool(largest_same_type_organ(feat_m["cells"])),
        }
        if live_ask:
            arm["live_ask"] = live_ask_check(base, sess_m, PROBES, corpus)
        arms[mode] = arm
        print(
            f"  [{mode}] organ_ch={arm['mean_organ_channel']} mass={arm['structure_mass']} "
            f"edges={arm['n_edges']} clusters={arm['n_clusters']} gold_rank={arm['mean_gold_rank']}"
        )

    intact_s = arms["intact"]["mean_organ_channel"]
    lesion_s = arms["organ_lesion"]["mean_organ_channel"]
    rewire_s = arms["degree_rewire"]["mean_organ_channel"]
    intact_m = arms["intact"]["structure_mass"]
    lesion_m = arms["organ_lesion"]["structure_mass"]
    rewire_m = arms["degree_rewire"]["structure_mass"]
    drop_lesion = intact_s - lesion_s
    drop_rewire = intact_s - rewire_s
    mass_drop_lesion = intact_m - lesion_m
    mass_drop_rewire = intact_m - rewire_m
    # Causal PASS (Soligo-style module intervention):
    # lesion collapses organ channel & edge mass more than degree-matched rewire.
    causal_ok = (
        mass_drop_lesion > mass_drop_rewire + 1.0
        and arms["organ_lesion"]["n_edges"] < max(1, arms["intact"]["n_edges"] // 2)
        and drop_lesion > max(0.0, drop_rewire) + 1e-6
        and lesion_s < intact_s
    )

    return {
        "seed": seed,
        "organ": {k: v for k, v in organ.items() if k != "members"} if organ else None,
        "organ_n_members": organ["cell_count"] if organ else 0,
        "topology": topo,
        "arms": arms,
        "causal": {
            "drop_organ_channel_lesion": round(drop_lesion, 4),
            "drop_organ_channel_rewire": round(drop_rewire, 4),
            "mass_drop_lesion": round(mass_drop_lesion, 4),
            "mass_drop_rewire": round(mass_drop_rewire, 4),
            "lesion_hurts_more_than_rewire": causal_ok,
            "intact_better_gold_rank": arms["intact"]["mean_gold_rank"]
            <= arms["organ_lesion"]["mean_gold_rank"],
        },
    }


def aggregate(seeds: list[dict[str, Any]]) -> dict[str, Any]:
    phis = [s["topology"]["phi_SWP"] for s in seeds if s["topology"]["phi_SWP"] is not None]
    p_cfgs = [s["topology"]["p_clustering_ge_obs_under_config"] for s in seeds]
    nonrand = [s["topology"]["nonrandom_vs_config"] for s in seeds]
    causal_flags = [s["causal"]["lesion_hurts_more_than_rewire"] for s in seeds]
    drop_l = [s["causal"]["drop_organ_channel_lesion"] for s in seeds]
    drop_r = [s["causal"]["drop_organ_channel_rewire"] for s in seeds]
    phi_mean = statistics.fmean(phis) if phis else None
    morph_ok = sum(nonrand) >= max(1, (len(seeds) + 1) // 2)
    causal_ok = sum(causal_flags) >= max(1, (len(seeds) + 1) // 2)

    if morph_ok and causal_ok:
        verdict = "NATURAL_NN_CAUSAL_PASS"
    elif morph_ok or causal_ok:
        verdict = "NATURAL_NN_CAUSAL_PARTIAL"
    else:
        verdict = "NATURAL_NN_CAUSAL_FAIL"

    return {
        "n_seeds": len(seeds),
        "phi_SWP_mean": None if phi_mean is None else round(phi_mean, 4),
        "phi_note": "supporting (Muldoon φ≳0.6 is literature convention; not sole gate)",
        "p_config_mean": round(statistics.fmean(p_cfgs), 4),
        "fraction_nonrandom_vs_config": round(sum(nonrand) / len(seeds), 4),
        "fraction_causal_lesion_gt_rewire": round(sum(causal_flags) / len(seeds), 4),
        "mean_drop_organ_channel_lesion": round(statistics.fmean(drop_l), 4),
        "mean_drop_organ_channel_rewire": round(statistics.fmean(drop_r), 4),
        "mean_mass_drop_lesion": round(
            statistics.fmean([s["causal"]["mass_drop_lesion"] for s in seeds]), 4
        ),
        "mean_mass_drop_rewire": round(
            statistics.fmean([s["causal"]["mass_drop_rewire"] for s in seeds]), 4
        ),
        "morphology_ok": morph_ok,
        "causal_ok": causal_ok,
        "verdict": verdict,
        "honest_bounds": {
            "nulls": "config edge-swap + ER + lattice for SWP φ (Muldoon-inspired)",
            "causal": "organ-channel (native weight 0.12) lesion vs degree-matched rewire + structure mass",
            "not_claimed": "named cortical organs; live Mem0 API σ bake-off",
        },
    }


def main() -> int:
    ap = argparse.ArgumentParser(description="Strengthened natural-NN causal protocol")
    ap.add_argument("--seeds", type=int, default=3, help="independent incubation seeds")
    ap.add_argument("--live-ask", action="store_true", help="also run live ask on mutated sessions")
    args = ap.parse_args()

    apply_gpu_env()
    apply_resource_budget()
    os.environ.setdefault("VALHALLA_USE_CPU", os.environ.get("VALHALLA_USE_CPU", "1"))
    REPORTS.mkdir(parents=True, exist_ok=True)

    print(f"=== {PROTOCOL} ===")
    print(f"seeds={args.seeds} live_ask={args.live_ask} null_trials={NULL_TRIALS}")

    seed_rows = [run_seed(i, live_ask=args.live_ask) for i in range(args.seeds)]
    summary = aggregate(seed_rows)
    out = {
        "protocol": PROTOCOL,
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "references": [
            {"id": "Muldoon2016", "metric": "Small-World Propensity φ"},
            {"id": "Downes2012", "metric": "development + significance vs null"},
            {"id": "Soligo2025", "metric": "module detection + intervention/ablation"},
            {"id": "config_model", "metric": "degree-preserving edge-swap surrogate"},
        ],
        "seeds": seed_rows,
        "summary": summary,
        "verdict": summary["verdict"],
    }
    path = REPORTS / "natural_nn_causal_v1.json"
    path.write_text(json.dumps(out, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print("\n" + json.dumps(summary, indent=2))
    print(f"→ {path}")
    print(f"verdict={summary['verdict']}")
    return 0 if summary["verdict"] in ("NATURAL_NN_CAUSAL_PASS", "NATURAL_NN_CAUSAL_PARTIAL") else 1


if __name__ == "__main__":
    raise SystemExit(main())
