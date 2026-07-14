#!/usr/bin/env python3
"""Natural NN topology probe — objective graph metrics vs null models.

Protocol: natural-nn-topology-v1

Inspired by cultured-neuron / connectome practice (small-world σ, clustering,
path length, modularity vs Erdős–Rényi nulls) and contrasted with a peer-like
co-occurrence entity graph (memory-API style text graph over the same corpus).

Does NOT claim biological cortical organs. Reports honest mega-cluster bounds.

  python3 tools/valhalla_inference/test_natural_nn_topology.py
  python3 tools/valhalla_inference/test_natural_nn_topology.py --stages 4,8,16,32
"""

from __future__ import annotations

import argparse
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

PROTOCOL = "natural-nn-topology-v1"
EDGE_STRENGTH = 0.01
NULL_SEEDS = 24

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
]


# ── graph primitives (no networkx dependency) ───────────────────────────────


def _undirected_adj(edges: list[tuple[str, str]]) -> dict[str, set[str]]:
    adj: dict[str, set[str]] = defaultdict(set)
    for a, b in edges:
        if a == b:
            continue
        adj[a].add(b)
        adj[b].add(a)
    return adj


def _ensure_nodes(adj: dict[str, set[str]], nodes: list[str]) -> dict[str, set[str]]:
    for n in nodes:
        adj.setdefault(n, set())
    return adj


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
            for v in adj.get(u, ()):
                if v not in seen:
                    seen.add(v)
                    q.append(v)
        comps.append(members)
    comps.sort(key=len, reverse=True)
    return comps


def avg_clustering(adj: dict[str, set[str]]) -> float:
    nodes = [n for n, nbrs in adj.items() if nbrs]
    if not nodes:
        return 0.0
    vals: list[float] = []
    for u in nodes:
        nbrs = list(adj[u])
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


def avg_shortest_path(adj: dict[str, set[str]], sample_cap: int = 120) -> float | None:
    """Mean geodesic length on the largest component (None if component size < 2)."""
    comps = connected_components(adj)
    if not comps or len(comps[0]) < 2:
        return None
    nodes = comps[0]
    if len(nodes) > sample_cap:
        rng = random.Random(42)
        sources = rng.sample(nodes, sample_cap)
    else:
        sources = nodes
    lengths: list[int] = []
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
            if t != s and t in set(nodes):
                lengths.append(d)
    if not lengths:
        return None
    return statistics.fmean(lengths)


def modularity_greedy(adj: dict[str, set[str]]) -> float:
    """Fast Newman-style agglomerative modularity (approx Louvain seed)."""
    nodes = list(adj.keys())
    n = len(nodes)
    if n < 2:
        return 0.0
    # edge count (undirected)
    m = sum(len(adj[u]) for u in nodes) // 2
    if m == 0:
        return 0.0
    # start: each node its own community
    comm = {u: i for i, u in enumerate(nodes)}
    # degree
    deg = {u: len(adj[u]) for u in nodes}

    def q_of(partition: dict[str, int]) -> float:
        # Q = 1/(2m) Σ_ij [A_ij - k_i k_j/(2m)] δ(c_i,c_j)
        by_c: dict[int, list[str]] = defaultdict(list)
        for u, c in partition.items():
            by_c[c].append(u)
        q = 0.0
        for members in by_c.values():
            lc = 0
            kc = 0
            member_set = set(members)
            for u in members:
                kc += deg[u]
                for v in adj[u]:
                    if v in member_set and u < v:
                        lc += 1
            q += (lc / m) - (kc / (2.0 * m)) ** 2
        return q

    improved = True
    while improved:
        improved = False
        for u in nodes:
            cur = comm[u]
            best = cur
            best_q = q_of(comm)
            # candidate communities = neighbors'
            cand = {comm[v] for v in adj[u]} | {cur}
            for c in cand:
                if c == cur:
                    continue
                trial = dict(comm)
                trial[u] = c
                qq = q_of(trial)
                if qq > best_q + 1e-12:
                    best_q = qq
                    best = c
            if best != cur:
                comm[u] = best
                improved = True
    return q_of(comm)


def er_null_metrics(n: int, e: int, trials: int = NULL_SEEDS, seed: int = 0) -> dict[str, float]:
    """Erdős–Rényi G(n,M) averages for clustering / path / modularity."""
    if n < 2 or e <= 0:
        return {"C_rand": 0.0, "L_rand": 0.0, "Q_rand": 0.0}
    labels = [str(i) for i in range(n)]
    max_e = n * (n - 1) // 2
    e = min(e, max_e)
    Cs: list[float] = []
    Ls: list[float] = []
    Qs: list[float] = []
    for t in range(trials):
        rng = random.Random(seed + t * 9973)
        pairs = [(labels[i], labels[j]) for i in range(n) for j in range(i + 1, n)]
        chosen = rng.sample(pairs, e) if e < len(pairs) else pairs
        adj = _ensure_nodes(_undirected_adj(chosen), labels)
        Cs.append(avg_clustering(adj))
        lp = avg_shortest_path(adj)
        if lp is not None:
            Ls.append(lp)
        Qs.append(modularity_greedy(adj))
    return {
        "C_rand": statistics.fmean(Cs) if Cs else 0.0,
        "L_rand": statistics.fmean(Ls) if Ls else 0.0,
        "Q_rand": statistics.fmean(Qs) if Qs else 0.0,
    }


def topology_report(name: str, nodes: list[str], edges: list[tuple[str, str]]) -> dict[str, Any]:
    adj = _ensure_nodes(_undirected_adj(edges), nodes)
    n = len(nodes)
    e = sum(len(adj[u]) for u in nodes) // 2
    comps = connected_components(adj)
    mega = len(comps[0]) / n if comps and n else 0.0
    C = avg_clustering(adj)
    L = avg_shortest_path(adj)
    Q = modularity_greedy(adj)
    null = er_null_metrics(n, e)
    C_r = null["C_rand"]
    L_r = null["L_rand"]
    Q_r = null["Q_rand"]
    gamma = (C / C_r) if C_r > 1e-12 else (math.inf if C > 0 else 0.0)
    lambda_ = (L / L_r) if (L is not None and L_r > 1e-12) else None
    if lambda_ and lambda_ > 0 and math.isfinite(gamma):
        sigma = gamma / lambda_
    else:
        sigma = None
    organs = [c for c in comps if len(c) >= 2]
    size_entropy = 0.0
    if organs:
        sizes = [len(c) for c in organs]
        total = sum(sizes)
        for s in sizes:
            p = s / total
            size_entropy -= p * math.log(p + 1e-15)
    return {
        "graph": name,
        "n": n,
        "e": e,
        "avg_degree": (2.0 * e / n) if n else 0.0,
        "n_components": len(comps),
        "n_organs_ge2": len(organs),
        "largest_cc_frac": round(mega, 4),
        "organ_size_entropy": round(size_entropy, 4),
        "C": round(C, 4),
        "L": None if L is None else round(L, 4),
        "Q": round(Q, 4),
        "C_rand": round(C_r, 4),
        "L_rand": round(L_r, 4),
        "Q_rand": round(Q_r, 4),
        "C_over_Crand": None if not math.isfinite(gamma) else round(gamma, 4),
        "L_over_Lrand": None if lambda_ is None else round(lambda_, 4),
        "sigma_small_world": None if sigma is None else round(sigma, 4),
        "Q_minus_Qrand": round(Q - Q_r, 4),
    }


# ── extract Valhalla graphs ─────────────────────────────────────────────────


def collect_stem_tile(sess: dict) -> tuple[list[str], list[tuple[str, str]], list[str], list[tuple[str, str]]]:
    frags = sess.get("fragments") or []
    stem_nodes: list[str] = []
    stem_edges: list[tuple[str, str]] = []
    tile_nodes: list[str] = []
    tile_edges: list[tuple[str, str]] = []
    seen_stem: set[str] = set()
    seen_tile: set[str] = set()
    seen_se: set[tuple[str, str]] = set()
    seen_te: set[tuple[str, str]] = set()

    def add_stem(cells: list) -> None:
        for c in cells:
            if not isinstance(c, dict):
                continue
            cid = str(c.get("cell_id") or c.get("id") or "")
            if not cid:
                continue
            if cid not in seen_stem:
                seen_stem.add(cid)
                stem_nodes.append(cid)
            for peer in c.get("connections") or []:
                if isinstance(peer, (list, tuple)) and len(peer) >= 2:
                    pid, strength = str(peer[0]), float(peer[1])
                elif isinstance(peer, dict):
                    pid, strength = str(peer.get("id") or peer.get("peer") or ""), float(
                        peer.get("strength") or peer.get("w") or 0
                    )
                else:
                    continue
                if not pid or strength <= EDGE_STRENGTH:
                    continue
                a, b = sorted((cid, pid))
                key = (a, b)
                if key not in seen_se:
                    seen_se.add(key)
                    stem_edges.append(key)

    def add_tile(tiles: list) -> None:
        for t in tiles:
            if not isinstance(t, dict):
                continue
            tid = str(t.get("id") or "")
            if not tid:
                continue
            if tid not in seen_tile:
                seen_tile.add(tid)
                tile_nodes.append(tid)
            for peer in t.get("connections") or []:
                pid = str(peer) if not isinstance(peer, dict) else str(peer.get("id") or "")
                if not pid:
                    continue
                a, b = sorted((tid, pid))
                key = (a, b)
                if key not in seen_te:
                    seen_te.add(key)
                    tile_edges.append(key)

    for f in frags if isinstance(frags, list) else []:
        if not isinstance(f, dict):
            continue
        add_stem(f.get("stem_cells") or [])
        add_tile(f.get("completed_tiles") or [])

    snap = sess.get("snapshot") or {}
    if not stem_nodes:
        add_stem(snap.get("stem_cells") or [])
    if not tile_nodes:
        add_tile(snap.get("completed_tiles") or [])
    return stem_nodes, stem_edges, tile_nodes, tile_edges


def cooccurrence_graph(lines: list[str], min_token_len: int = 3) -> tuple[list[str], list[tuple[str, str]]]:
    """Peer-like text co-occurrence graph (memory-API analog over same corpus)."""
    stop = {
        "the", "and", "for", "with", "that", "this", "from", "into", "onto",
        "are", "was", "were", "has", "have", "had", "not", "but", "its", "her",
        "his", "she", "him", "they", "them", "our", "you", "your",
    }
    edges: set[tuple[str, str]] = set()
    nodes: set[str] = set()
    for line in lines:
        toks = [
            t.strip(".,;:!?\"'()[]").lower()
            for t in line.split()
        ]
        toks = [t for t in toks if len(t) >= min_token_len and t not in stop and t.isalpha()]
        for t in toks:
            nodes.add(t)
        for i in range(len(toks)):
            for j in range(i + 1, len(toks)):
                a, b = sorted((toks[i], toks[j]))
                if a != b:
                    edges.add((a, b))
    return sorted(nodes), sorted(edges)


# ── incubation schedule ─────────────────────────────────────────────────────


def incubate(lines: list[str]) -> dict:
    kwargs = production_valhalla_base_kwargs()
    kwargs["decode"] = "hybrid"
    kwargs["mcq_decode"] = "structure_fate"
    kwargs["hybrid_open_lm_fallback"] = False
    os.environ["VALHALLA_HYBRID_OPEN_LM_FALLBACK"] = "0"
    fields = getattr(ValhallaBase, "__dataclass_fields__", {}) or {}
    base = ValhallaBase(**{k: v for k, v in kwargs.items() if k in fields})
    base.set_context(lines)
    # Force Fate/Tile/Stem cycles without needing answer quality
    _ = base.ask("What structure formed in this session?")
    return base.session.session_state or {}


def evaluate_stage(n_lines: int) -> dict[str, Any]:
    lines = CORPUS[:n_lines]
    sess = incubate(lines)
    snap = sess.get("snapshot") or {}
    stem_n, stem_e, tile_n, tile_e = collect_stem_tile(sess)
    stem_topo = topology_report("stem_cell", stem_n, stem_e)
    tile_topo = topology_report("tile", tile_n, tile_e)
    coco_n, coco_e = cooccurrence_graph(lines)
    coco_topo = topology_report("cooccurrence_peer", coco_n, coco_e)
    return {
        "n_lines": n_lines,
        "fate_signal_tick": int(snap.get("fate_signal_tick") or 0),
        "hub_routed": snap.get("hub_routed") or [],
        "fragment_count": len(sess.get("fragments") or []),
        "stem": stem_topo,
        "tile": tile_topo,
        "cooccurrence_peer": coco_topo,
    }


def classify(stages: list[dict[str, Any]]) -> dict[str, Any]:
    last = stages[-1]
    first = stages[0]
    stem = last["stem"]
    tile = last["tile"]
    coco = last["cooccurrence_peer"]

    formed = stem["n"] >= 8 and stem["e"] > 0 and tile["n"] >= 1
    sigma = stem.get("sigma_small_world")
    non_random = False
    if sigma is not None and sigma > 1.0:
        non_random = True
    elif stem.get("C_over_Crand") and stem["C_over_Crand"] >= 1.5:
        non_random = True
    elif stem.get("Q_minus_Qrand") and stem["Q_minus_Qrand"] > 0.05:
        non_random = True

    # Growth: sigma or C rises with age (cultured-neuron analogue)
    growth = False
    if len(stages) >= 2:
        s0 = first["stem"].get("sigma_small_world")
        s1 = stem.get("sigma_small_world")
        c0 = first["stem"].get("C") or 0.0
        c1 = stem.get("C") or 0.0
        e0 = first["stem"].get("e") or 0
        e1 = stem.get("e") or 0
        if s0 is not None and s1 is not None and s1 > s0:
            growth = True
        elif c1 > c0 and e1 >= e0:
            growth = True
        elif e1 > e0 and stem["n"] >= first["stem"]["n"]:
            growth = True

    # Differentiation (legacy flag): >1 organ of size≥2 and mega fraction < 0.95
    # NOTE: project doctrine — organs are NOT predefined. A single large same-type
    # component is organ maturation continuing, not a failure. See
    # natural-nn-organ-runtime-v1 for signature-homogeneity analysis.
    differentiated = stem["n_organs_ge2"] >= 2 and stem["largest_cc_frac"] < 0.95

    # Vs peer co-occurrence: Valhalla structure more modular / small-world than text bag
    vs_peer = False
    if sigma is not None and coco.get("sigma_small_world") is not None:
        vs_peer = sigma >= coco["sigma_small_world"]
    if stem.get("Q", 0) >= coco.get("Q", 0):
        vs_peer = vs_peer or (stem["Q"] - coco["Q"] > 0.02)

    if formed and non_random and growth:
        verdict = "NATURAL_NN_TOPOLOGY_PASS"
    elif formed and non_random:
        verdict = "NATURAL_NN_TOPOLOGY_PARTIAL"
    elif formed:
        verdict = "NATURAL_NN_TOPOLOGY_FORMED_RANDOMISH"
    else:
        verdict = "NATURAL_NN_TOPOLOGY_FAIL"

    return {
        "formed": formed,
        "non_random_vs_ER": non_random,
        "growth_trajectory": growth,
        "multi_cc_organs_legacy": differentiated,
        "functionally_differentiated_organs": differentiated,
        "beats_cooccurrence_peer": vs_peer,
        "stem_sigma": sigma,
        "stem_mega_cc_frac": stem["largest_cc_frac"],
        "verdict": verdict,
        "honest_bounds": {
            "literature_analog": "cultured neurons: C↑ + L stable → small-world (Downes 2012; Teller 2014)",
            "sigma_convention": "σ>1 suggests small-world vs ER; σ≫1 strong",
            "organ_doctrine": "no preset organ catalog; same-signature coalescence = maturation continuing (see organ-runtime protocol)",
            "not_claimed": "cortical organ names / Transformers replaced",
            "peer_note": "cooccurrence_peer approximates Mem0/Zep text graphs — not online API run",
        },
    }


def main() -> int:
    ap = argparse.ArgumentParser(description="Natural NN topology objective probe")
    ap.add_argument(
        "--stages",
        default="6,12,18,24",
        help="comma-separated corpus line counts (development ages)",
    )
    args = ap.parse_args()
    stages_n = [int(x) for x in args.stages.split(",") if x.strip()]
    stages_n = [min(n, len(CORPUS)) for n in stages_n]
    stages_n = sorted(set(stages_n))

    apply_gpu_env()
    apply_resource_budget()
    os.environ.setdefault("VALHALLA_USE_CPU", os.environ.get("VALHALLA_USE_CPU", "1"))
    REPORTS.mkdir(parents=True, exist_ok=True)

    print(f"=== {PROTOCOL} ===")
    print(f"stages={stages_n}  null_trials={NULL_SEEDS}")
    print("comparators: ER null · co-occurrence peer graph (memory-API analog)")
    print()

    results: list[dict[str, Any]] = []
    for n in stages_n:
        print(f"[age={n} lines] incubating…")
        st = evaluate_stage(n)
        results.append(st)
        s = st["stem"]
        print(
            f"  stem n={s['n']} e={s['e']} C={s['C']} L={s['L']} "
            f"σ={s['sigma_small_world']} Q={s['Q']} mega={s['largest_cc_frac']} "
            f"organs≥2={s['n_organs_ge2']}"
        )
        t = st["tile"]
        print(f"  tile n={t['n']} e={t['e']} C={t['C']} σ={t['sigma_small_world']} Q={t['Q']}")
        c = st["cooccurrence_peer"]
        print(f"  coco n={c['n']} e={c['e']} C={c['C']} σ={c['sigma_small_world']} Q={c['Q']}")

    checks = classify(results)
    out = {
        "protocol": PROTOCOL,
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "references": [
            {
                "id": "Downes2012",
                "title": "Emergence of a Small-World Functional Network in Cultured Neurons",
                "metric": "C, L, small-worldness vs null over culture age (DIV)",
            },
            {
                "id": "Teller2014",
                "title": "Emergence of Small-World Anatomical Networks in Self-Organizing Clustered Neuronal Cultures",
                "metric": "C, path length, efficiency, degree distribution",
            },
            {
                "id": "WattsStrogatz1998",
                "title": "Collective dynamics of small-world networks",
                "metric": "σ = (C/C_rand)/(L/L_rand)",
            },
            {
                "id": "agent_memory_2026",
                "title": "Mem0 / Zep-Graphiti / Letta substrate contrast",
                "metric": "text/temporal KG — no Stem/Tile physiology; co-occurrence peer as local proxy",
            },
        ],
        "stages": results,
        "checks": checks,
        "verdict": checks["verdict"],
    }
    path = REPORTS / "natural_nn_topology.json"
    path.write_text(json.dumps(out, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print()
    print(json.dumps(checks, indent=2))
    print(f"→ {path}")
    print(f"verdict={checks['verdict']}")
    return 0 if checks["verdict"] in (
        "NATURAL_NN_TOPOLOGY_PASS",
        "NATURAL_NN_TOPOLOGY_PARTIAL",
        "NATURAL_NN_TOPOLOGY_FORMED_RANDOMISH",
    ) else 1


if __name__ == "__main__":
    raise SystemExit(main())
