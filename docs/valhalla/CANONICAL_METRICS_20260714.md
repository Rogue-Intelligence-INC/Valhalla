# Canonical metrics — ValhallaBase (2026-07-14)

**Single source of truth for public / diligence claims.**  
Dated archive reports may show higher historical numbers under *different protocols* — do not remix without labeling.

## Design → properties (read before the metric table)

Many capabilities people notice are **designed first**, then **measured**. Vocabulary:

| Tag | Meaning |
|-----|---------|
| **D** | Intentional architecture / default |
| **E** | Emergent under that design (measured, not hardcoded outcome) |
| **P** | Product proof vs memory-API peers |

Full map: [`DESIGN_TO_PROPERTIES_20260714.md`](./DESIGN_TO_PROPERTIES_20260714.md)  
**Unified proof scorecard (17 claims):** `reports/valhalla_inference/PROOF_SCORECARD.md` · `TECHNICAL_CORE_FULLY_PROVEN`
**Heavy investment (time+methods+info):** `reports/valhalla_inference/PROOF_INVESTMENT_HEAVY.md` · wall ~11min · 56-line corpus · 14 topology ages · 8 causal seeds + recovery


```bash
python3 tools/valhalla_inference/run_technical_core_proof_suite.py --fast
```

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
| Longchain holdout QA | **100%** (3/3) | same · production **128L** pin-evidence · see **Legitimacy** |
| Longchain interleave QA | **75%** (3/4) | same · content land egress (was 25%) |
| Longchain composite | **≈0.882** | continuity-weighted |
| RAGTruth (live industry) | **38/40** · full cache **88/100** | ragtruth-v1 |

## Legitimacy (why numbers moved · how to cite)

**SSOT:** this file. Landing / diligence / paper pages must match these rows. Archive reports that still say holdout **33%** / composite **≈0.589** / interleave **25%** are **pre-opt snapshots** — label them historical, do not remix as “current.”

| Snapshot | Cite | What changed (must disclose if asked) |
|----------|------|----------------------------------------|
| ≤ morning 2026-07-14 | holdout 33% · interleave 25% · composite ≈0.589 | Truncate-before-cap starved evidence; sticky address-schedule |
| mid-day (pin + holdout egress) | holdout 100% · interleave 25% · composite ≈0.822 | Pin-evidence corpus + holdout content RelAddr egress |
| **now (opt)** | holdout **100%** · interleave **75%** · composite **≈0.882** | + content-gate schedule · interleave content land · tighter temporal scoring (camping ≠ charity-race) |

**Still true / still separate:**

| Claim | Status |
|-------|--------|
| theme_drift **0%** · retention **100%** | Unchanged lead continuity |
| Interleave residual (1/4) | `LOCO_0_6` June camping plan still miss — developing |
| Industry LoCoMo / MultiHop vs Mem0 | Still Tier B — **≠** protocol holdout |
| U5 full on-topic floor (≥0.80) | Still open (fast citeable developing only) |
| Holdout **n=3** / interleave **n=4** | Small slices — protocol passes, not market SOTA |
| Soft scorer edges | Some interleave True without gold substring (evidence path) — keep auditing |

**Land-opt audit (same logic elsewhere):** [`LAND_OPT_AUDIT_20260714.md`](./LAND_OPT_AUDIT_20260714.md)

**Reproduce:** `python3 tools/valhalla_inference/test_fate_longchain_unified.py --production`  
**Artifact:** `reports/valhalla_inference/fate_longchain_unified.json` + `FATE_LONGCHAIN_UNIFIED.md` (top table = current).

## Technical highlights (mechanism — publish with Unique Proofs)

| Claim | Value | Protocol / artifact |
|-------|--------|---------------------|
| Fate cycle + Tile/Stem formation | **TRIAD_STRUCTURE_CYCLE_PASS** · 16 tiles · 108 stems · hub 104×4 | `triad-structure-highlight-v1` |
| Natural NN topology (vs ER null) | **NATURAL_NN_TOPOLOGY_PASS** · Stem σ≈9.27 · growth 6→24 lines | `natural-nn-topology-v1` |
| Organ coalescence (no preset catalog) | **ORGAN_COALESCENCE_RUNTIME_PASS** · body 64 @ ~0.78 · tiles 5→10 · tick +768 | `natural-nn-organ-runtime-v1` |
| Strengthened causal NN (config null + lesion) | **NATURAL_NN_CAUSAL_PASS** · config p=0 · organ-channel lesion≻rewire · massΔ 234 vs 0 | `natural-nn-causal-v1` |
| Manifold Fate adapt | **PASS** · fixed_point · 3 adapt cycles | `manifold_orchestrator_smoke` |
| Structure train L1 | **+38.7pp** pollution rescue · TPI **0.755** | U3 / TPI-v2 |
| Structure train L2/L3 | hub_spread tightens · bake **Δw≈298** | `valhalla-train-proof-smoke-v1` |

Narrative: `docs/valhalla/TECH_HIGHLIGHTS_TRIAD_TRAIN_20260714.md`  
Honest bound: Tile/Stem are **structure substrate**, not differentiated cortical organs.

## Secondary / Tier B (publish, do not lead)

| Claim | Value | Note |
|-------|-------|------|
| U5 cost@floor | **Citeable developing** | prefill ~0.29× · on-topic ~0.55 (fast); full floor ≥0.80 open — **not Unique Proof lead** |
| LoCoMo / MultiHop absolute | Tier B | Industry same-Q vs Mem0 — not Unique Proof |

## Forbidden public phrasing

- “Unique Proofs **5/5**” (lead suite is U1–U4)
- “RAGTruth **100/100**” as *current* headline (superseded; use 38/40 · 88/100)
- “Longchain composite **0.95**” as current (that was **v1 minimal historical**)
- “Beat Mem0 on LoCoMo / LongMemEval”
- “Holdout 100%” **without** saying `fate-longchain-unified-v5` · `fragment_full` production (and without separating industry LoCoMo)
- Remixing **pre-fix 33%** tables as if they were the current live result

## Reproduction

```bash
./scripts/run_unique_proofs.sh --fast --only U1,U2,U3,U4
python3 tools/valhalla_inference/test_triad_structure_highlight.py
python3 tools/valhalla_inference/test_natural_nn_topology.py
python3 tools/valhalla_inference/test_natural_nn_organ_runtime.py --long
python3 tools/valhalla_inference/test_natural_nn_causal_v1.py --seeds 3
python3 tools/valhalla_inference/test_train_proof_smoke.py --skip-tpi
python3 tools/valhalla_inference/test_fate_longchain_unified.py --production
python3 tools/valhalla_inference/audit_benchmark_legitimacy.py
```

*Rogue Intelligence LNC.*
