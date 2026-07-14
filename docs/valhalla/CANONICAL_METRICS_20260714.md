# Canonical metrics — ValhallaBase (2026-07-14)

**Single source of truth for public / diligence claims.**  
Dated archive reports may show higher historical numbers under *different protocols* — do not remix without labeling.

## Lead (cite)

| Claim | Value | Protocol / artifact |
|-------|--------|---------------------|
| Unique Proofs **U1–U4** | **4/4 pass · 4/4 citeable** | `unique-proof-suite-v1` · `UNIQUE_PROOF_SUITE.md` |
| U1 shared patch | **0.9999** vs iso **0.968** | U1 |
| U2 MAR / FIR / PPI | **1.0 / 0.0 / 0.65** | U2 |
| U3 pollution rescue | **+38.7pp** | U3 |
| U4 orthogonality | `VALHALLA_BASE_INDEPENDENT` | U4 |
| Longchain theme_drift | **0%** | `fate-longchain-unified-v5` · `fragment_full` |
| Longchain retention | **100%** | same |
| RAGTruth (live industry) | **38/40** · full cache **88/100** | ragtruth-v1 |

## Technical highlights (mechanism — publish with Unique Proofs)

| Claim | Value | Protocol / artifact |
|-------|--------|---------------------|
| Fate cycle + Tile/Stem formation | **TRIAD_STRUCTURE_CYCLE_PASS** · 16 tiles · 108 stems · hub 104×4 | `triad-structure-highlight-v1` |
| Natural NN topology (vs ER null) | **NATURAL_NN_TOPOLOGY_PASS** · Stem σ≈9.27 · growth 6→24 lines | `natural-nn-topology-v1` |
| Organ coalescence (no preset catalog) | **ORGAN_COALESCENCE_RUNTIME_PASS** · body 64 @ ~0.78 · tiles 5→10 · tick +768 | `natural-nn-organ-runtime-v1` |
| Manifold Fate adapt | **PASS** · fixed_point · 3 adapt cycles | `manifold_orchestrator_smoke` |
| Structure train L1 | **+38.7pp** pollution rescue · TPI **0.755** | U3 / TPI-v2 |
| Structure train L2/L3 | hub_spread tightens · bake **Δw≈298** | `valhalla-train-proof-smoke-v1` |

Narrative: `docs/valhalla/TECH_HIGHLIGHTS_TRIAD_TRAIN_20260714.md`  
Honest bound: Tile/Stem are **structure substrate**, not differentiated cortical organs.

## Secondary / Tier B (publish, do not lead)

| Claim | Value | Note |
|-------|--------|------|
| Longchain holdout QA | **33%** | fragment_full production |
| Longchain interleave | **25%** | same |
| Longchain composite | **≈0.589** | continuity-weighted |
| U5 cost@floor | **Developing** | prefill ~0.29× exists; accuracy floor not met — **not Unique Proof lead** |
| LoCoMo / MultiHop absolute | Tier B | Not vs-Mem0 SOTA |

## Forbidden public phrasing

- “Unique Proofs **5/5**” (lead suite is U1–U4)
- “RAGTruth **100/100**” as *current* headline (superseded; use 38/40 · 88/100)
- “Longchain composite **0.95** / holdout **100%**” without saying **v1 minimal historical**
- “Beat Mem0 on LoCoMo / LongMemEval”

## Reproduction

```bash
./scripts/run_unique_proofs.sh --fast --only U1,U2,U3,U4
python3 tools/valhalla_inference/test_triad_structure_highlight.py
python3 tools/valhalla_inference/test_natural_nn_topology.py
python3 tools/valhalla_inference/test_natural_nn_organ_runtime.py --long
python3 tools/valhalla_inference/test_train_proof_smoke.py --skip-tpi
python3 tools/valhalla_inference/test_fate_longchain_unified.py --production
python3 tools/valhalla_inference/audit_benchmark_legitimacy.py
```

*Rogue Intelligence LNC.*
