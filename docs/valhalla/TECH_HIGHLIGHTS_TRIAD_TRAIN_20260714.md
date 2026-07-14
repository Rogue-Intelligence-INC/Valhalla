# Technical core — Triad natural structure + structure training (2026-07-14)

**Status:** Citeable **technical core** · backed by live probes + Unique Proofs  
**Audience:** diligence, paper page, product landing, investor one-pager  
**Not claimed:** cortical organ nameplate · LM SGD as the default product path

> **Read first:** many cited properties are **consequences of design**, then measured.  
> See [`DESIGN_TO_PROPERTIES_20260714.md`](./DESIGN_TO_PROPERTIES_20260714.md) — **D**esigned / **E**mergent / **P**roof layers.  
> **Unified scorecard:** `reports/valhalla_inference/PROOF_SCORECARD.md` · **17/17 PROVEN** · `TECHNICAL_CORE_FULLY_PROVEN`  
> Reproduce: `./scripts/run_technical_core_proof_suite.sh --fast`

---

## Verdict (one screen)

| Core | Class | Evidence | Bound |
|------|-------|----------|--------|
| **C1 — Fate cycle + Tile/Stem formation** | **D→E** | `TRIAD_STRUCTURE_CYCLE_PASS` · manifold PASS | Fragments hold structure |
| **C2 — Natural NN topology vs ER** | **E** | `NATURAL_NN_TOPOLOGY_PASS` · Stem σ≈9.27 | Not Mem0 API bake-off |
| **C3 — Organ coalescence** | **D→E** | `ORGAN_COALESCENCE_RUNTIME_PASS` · body 64 @ ~0.78 · tiles 5→10 | No preset organ catalog; same-type merge = maturation continuing |
| **C3b — Causal lesion (strengthened)** | **D+E** | `NATURAL_NN_CAUSAL_PASS` · config-null p=0 · organ-channel lesion ≻ rewire | Hash-sig proxy; massΔ 234 vs 0 |
| **C4 — Structure-first train (L1–L3)** | **D** (P via U3) | L1 U3 **+38.7pp** · L3 bake **Δw≈298** | Not LoCoMo SOTA |

These four form the **technical core**, beside Unique Proofs U1–U4.

---

## H1 — Fate circulation · Tile/Stem as structured substrate

### What runs (production path)

Default ingress is **`quad_cycle`**: BlackHole → MeshBrain → MultiNova → BaseForce per line.  
`fate_orchestrate` · warmup / adapt / pulse · FOG · `signal_tick` advance are **on** in production Triad sessions (not print-only).

This is **Valhalla Fate** (ingress + affinity + relative addressing) — not s-wind Alg `Fate.explore` / Space.

### What forms at runtime

Under `fragment_full`, **Tile aggregates** and **Stem cells** incubate **inside fragments**. Top-level `snapshot.completed_tiles` / `stem_cells` may be empty by design (compact wire format).

**Live probe (2026-07-14)** · `reports/valhalla_inference/triad_structure_highlight.json`:

| Metric | Value |
|--------|-------|
| Verdict | `TRIAD_STRUCTURE_CYCLE_PASS` |
| Aggregated tiles | **16** across 4 fragments |
| Aggregated stem cells | **108** |
| Hub quad routed | **104 / 104 / 104 / 104** |
| `fate_ingress_routing` | `quad_cycle` |
| Manifold smoke | adapt cycles 3 · `fate_fixed_point` · **PASS** |

Reproduction:

```bash
python3 tools/valhalla_inference/test_triad_structure_highlight.py
./target/release/manifold_orchestrator_smoke
```

### Objective topology test (natural NN generation)

Protocol `natural-nn-topology-v1` (cultured-neuron / Watts–Strogatz style — not RAG accuracy):

| Comparator | Role |
|------------|------|
| Erdős–Rényi null (same N,M) | Classical small-world / modularity null |
| Co-occurrence peer graph | Local Mem0/Zep-style text-graph proxy (same corpus) |
| Age stages 6→24 lines | DIV-like growth trajectory |

**Live (2026-07-14):** `NATURAL_NN_TOPOLOGY_PASS` — Stem **σ≈9.27** vs ER, C≈0.65, Q≈0.58, growth across ages.  

**Organ doctrine (corrected):** organs are **not predefined**. Same-type coalescence = *differentiation continuing*. Long-runtime probe `natural-nn-organ-runtime-v1`: **`ORGAN_COALESCENCE_RUNTIME_PASS`** — largest body 64 cells @ purity ~0.78 (`proto:e5_v4_d7`), tiles **5→10**, Fate tick **+768** with longer pulses. See `NATURAL_NN_ORGAN_RUNTIME_20260714.md`.

```bash
python3 tools/valhalla_inference/test_natural_nn_topology.py
python3 tools/valhalla_inference/test_natural_nn_organ_runtime.py --long
```

### “Natural neural network” — honest wording

| Language that is OK | Language that is not |
|---------------------|----------------------|
| Runtime **structure substrate**: Tile merge/split · Stem BFS clusters · Hub routing | “Formed a biological brain / cortical modules” |
| **Small-world vs ER** (σ≫1) with growth ages | “Functionally differentiated perception/memory organs” |
| Structure enters **native retrieve scoring** | “Natural NN = Transformer replacement / beats Mem0 LoCoMo” |
---

## H2 — Structure-trainable body (new LM-adjacent mode)

Valhalla trains **structure first**; LM weight update is **optional L3**.

```
L1  Session incubation — zero LM gradients · affinity / snapshot learning
L2  Fate supervised pretrain — hub_spread / line affinity epochs
L3  HF bake — TriModal fusion writes checkpoint (controlled strength)
```

| Layer | Primary cite | Metric |
|-------|--------------|--------|
| **L1** | Unique Proof **U3** + TPI-v2 | Transfer **+38.7pp** vs polluted · TPI **0.755** |
| **L2** | `fate-pretrain-holdout-v1` | Smoke: hub_spread tightens + train>0; full: train 100% · holdout ≥2/3 when converged |
| **L3** | `bake_crossmodal12` | `weight_delta_norm ≈ 298` @ strength 0.10 + `valhalla_manifest.json` |

Canonical narrative: `docs/TRAINABILITY_PROOF.md` (VUC) / same title under engineering docs.

Smoke:

```bash
python3 tools/valhalla_inference/test_train_proof_smoke.py --skip-tpi   # L2 mechanism + L3
# L1 lead cite remains Unique Proof U3 (full TPI is heavier)
```

**Product sentence:** ValhallaBase is a **structure-trainable memory body** with an **optional LM decode head** — not “fine-tune a chat model and bolt on RAG.”

---

## How to sell (landing / paper)

1. Lead with Unique Proofs **U1–U4** (peers lack).  
2. Add **H1** as “structure actually grows under Fate cycles.”  
3. Add **H2** as “three-layer train stack; LM optional.”  
4. Continuity: longchain drift **0%** / retention **100%**.  
5. Do **not** lead with LoCoMo vs Mem0.

---

## Reproduction bundle

```bash
./scripts/run_unique_proofs.sh --fast --only U1,U2,U3,U4
python3 tools/valhalla_inference/test_triad_structure_highlight.py
python3 tools/valhalla_inference/test_train_proof_smoke.py --skip-tpi
python3 tools/valhalla_inference/test_fate_longchain_unified.py --production
python3 tools/valhalla_inference/audit_benchmark_legitimacy.py
```

*Rogue Intelligence LNC. · 2026-07-14*
