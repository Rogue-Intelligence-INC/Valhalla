# Natural NN topology — comparative protocol (2026-07-14)

**Protocol:** `natural-nn-topology-v1`  
**Script:** `tools/valhalla_inference/test_natural_nn_topology.py`  
**Report:** `reports/valhalla_inference/natural_nn_topology.json`

## Why this test (not “feels neural”)

Peer agent-memory systems (Mem0 vector/graph, Zep/Graphiti bi-temporal KG, Letta hierarchical blocks) build **text / entity graphs**. They do **not** incubate StemCell–Tile physiology. Claiming a “natural neural network” therefore needs **connectomics-style graph statistics**, not RAG accuracy.

Literature analog (cultured neurons → small-world):

| Case | Metric | Reading |
|------|--------|---------|
| Downes et al. 2012 (PLoS Comput Biol) | C↑ with culture age; L stable → small-world | Development trajectory |
| Teller et al. 2014 | C, path length, efficiency vs random | Self-organizing clustered cultures |
| Watts–Strogatz | σ = (C/C_rand)/(L/L_rand) | σ > 1 → small-world vs ER |
| Connectome practice | Modularity Q vs null | Segregation beyond chance |

## Comparators in this battery

1. **Erdős–Rényi null** — same *N*, same edge count *M*, averaged over 24 draws → C_rand, L_rand, Q_rand.  
2. **Co-occurrence peer graph** — tokens co-occurring in the same corpus line (local proxy for Mem0-style entity/text graphs; **not** a live Mem0/Zep API run).  
3. **Growth ages** — staged line counts (default 6 → 12 → 18 → 24) mirroring DIV culture series.

Valhalla graphs rebuilt from fragment wire format:
- **Stem:** `fragments[].stem_cells[].connections` with strength > 0.01 (same as Rust `build_stem_organ_clusters`)
- **Tile:** `fragments[].completed_tiles[].connections`

## Pass ladder (honest)

| Verdict | Meaning |
|---------|---------|
| `NATURAL_NN_TOPOLOGY_PASS` | Structure formed + non-random vs ER + growth trajectory |
| `NATURAL_NN_TOPOLOGY_PARTIAL` | Formed + non-random; growth or organ split weak |
| `NATURAL_NN_TOPOLOGY_FORMED_RANDOMISH` | Cells/tiles exist but look ER-like |
| `NATURAL_NN_TOPOLOGY_FAIL` | No usable graph |

Soft flags (reported, not required for PASS):
- `functionally_differentiated_organs` — ≥2 components size≥2 and mega-CC < 95%
- `beats_cooccurrence_peer` — σ or Q ≥ co-occurrence peer

## Non-claims

- Not “biological cortex modules”
- Not LoCoMo / LongMemEval vs Mem0
- Not Transformer replacement

## Reproduce

```bash
python3 tools/valhalla_inference/test_natural_nn_topology.py
python3 tools/valhalla_inference/test_natural_nn_topology.py --stages 8,16,24
```

*Rogue Intelligence LNC.*
