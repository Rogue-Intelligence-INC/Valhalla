# Valhalla AI — Documentation Index

**Scope:** ValhallaBase inference, memory, benchmarks, and product docs only.  
**Updated:** 2026-07-14 · **Narrative:** unique capabilities first (not LoCoMo-first).

> **Product definition:** ValhallaBase is a **structure-trainable body** — not a memory-API clone, not a 3B LM product.  
> See [VALHALLA_BASE_PRODUCT_POSITIONING.md](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/VALHALLA_BASE_PRODUCT_POSITIONING.md).

> **Canonical docs:** [valhalla-unified-core](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core) (submodule).  
> GitHub-safe mirror: [`valhalla/README.md`](valhalla/README.md).

---

## Unique capabilities (lead)

| Capability | Result | Doc |
|------------|--------|-----|
| **U1 Non-isolated multimodal** | shared **0.9999** vs iso **0.968** | [`UNIQUE_PROOF_SUITE.md`](../reports/valhalla_inference/UNIQUE_PROOF_SUITE.md) |
| **U2 FOG proactive + abstain** | MAR **1.0** · FIR **0.0** · PPI **0.65** | same |
| **U3 Pollution rescue** | **+38.7pp** · TPI **0.755** | [INCUBATION_CAPABILITY.md](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/INCUBATION_CAPABILITY.md) |
| **U4 Backbone orthogonality** | `VALHALLA_BASE_INDEPENDENT` | [`VALHALLA_INDEPENDENCE_EXPERIMENT_20260710.md`](../reports/valhalla_inference/VALHALLA_INDEPENDENCE_EXPERIMENT_20260710.md) |
| **U5 Structure-cost** | secondary / developing | [`CANONICAL_METRICS_20260714.md`](valhalla/CANONICAL_METRICS_20260714.md) |

**Matrix:** [`UNIQUE_VS_MEMORY_APIS.md`](valhalla/UNIQUE_VS_MEMORY_APIS.md) · **Protocol:** [`UNIQUE_PROOF_MICROBENCH.md`](valhalla/UNIQUE_PROOF_MICROBENCH.md)

```bash
./scripts/run_unique_proofs.sh --fast --only U1,U2,U3,U4   # 4/4 citeable (2026-07-14)
```

---

## Supporting pillars

| Pillar | Result | Doc |
|--------|--------|-----|
| **Faithfulness** | RAGTruth **38/40 · 88/100** | [`CANONICAL_METRICS_20260714.md`](valhalla/CANONICAL_METRICS_20260714.md) |
| **Continuity** | theme_drift **0%** · retention **100%** | [`FATE_LONGCHAIN_UNIFIED.md`](../reports/valhalla_inference/FATE_LONGCHAIN_UNIFIED.md) |
| **Session needle** | Strong / prior **4/4** | industry reports |
| **Trainability L2/L3** | protocol holdout (see TRAINABILITY_PROOF) | [TRAINABILITY_PROOF.md](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/TRAINABILITY_PROOF.md) |

### Industry calibration @ 3B (secondary · 2026-07-14)

| Highlight | Result | Doc |
|-----------|--------|-----|
| RAGTruth fast | **38/40** | industry summary |
| LoCoMo | Tier B — not lead | evidence / industry |
| MultiHop hybrid | **17/25** (fast industry) | same |
| MCQ ladder 46Q | **45/46 (98%)** @ 3B cal | ladder reports |

**Runtime default LM:** `Qwen2.5-0.5B-Instruct`. Industry suite sets 3B explicitly.

---

## Start here

| Pack | Path | Use |
|------|------|-----|
| **Unique proofs** | [`scripts/run_unique_proofs.sh`](../scripts/run_unique_proofs.sh) | **Lead diligence** U1–U4 |
| **Canonical metrics** | [`valhalla/CANONICAL_METRICS_20260714.md`](valhalla/CANONICAL_METRICS_20260714.md) | Single source of truth |
| **One-click pillars** | [`scripts/run_highlight_test.sh`](../scripts/run_highlight_test.sh) | Pillar smoke |
| **Evidence pack** | [`scripts/run_evidence_suite.sh`](../scripts/run_evidence_suite.sh) | Legitimacy · competitor · zip |
| **Web demo** | [`scripts/demo_valhalla_chat.sh`](../scripts/demo_valhalla_chat.sh) | Backend + Flutter web |
| **Product positioning** | [VALHALLA_BASE_PRODUCT_POSITIONING.md](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/VALHALLA_BASE_PRODUCT_POSITIONING.md) | Unique-first |
| **Investor one-pager** | [INVESTOR_ONE_PAGER.md](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/INVESTOR_ONE_PAGER.md) | Confidential memo |
| **Landing page** | [https://rogue-intelligence-inc.github.io/valhalla-unified-core/](https://rogue-intelligence-inc.github.io/valhalla-unified-core/) | GitHub Pages |
| **Paper page** | [paper.html](https://rogue-intelligence-inc.github.io/valhalla-unified-core/paper.html) | Technical note |
| **Product doc hub** | [`valhalla/README.md`](valhalla/README.md) | VUC links |
| **Incubation** | [INCUBATION_CAPABILITY.md](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/INCUBATION_CAPABILITY.md) | TPI / NPPI |
| **Platform reference** | [VALHALLA_BASE_PLATFORM.md](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/VALHALLA_BASE_PLATFORM.md) | Architecture |

---

## Benchmarks archive

| Benchmark | Tier | Result (canonical 2026-07-14) |
|-----------|------|-------------------------------|
| Unique proofs U1–U4 | A (capability) | **4/4** |
| RAGTruth 40/100 | A | **38/40 · 88/100** |
| Longchain continuity | A | drift **0%** · retention **100%** |
| Needle | A | Strong / prior 4/4 |
| MCQ ladder @ 3B | A (scaling) | **45/46** |
| LoCoMo / MultiHop / LongBench / U5 | B | See canonical + evidence |

**Master:** [`CANONICAL_METRICS_20260714.md`](valhalla/CANONICAL_METRICS_20260714.md) · [`UNIQUE_PROOF_SUITE.md`](../reports/valhalla_inference/UNIQUE_PROOF_SUITE.md)

---

## Run

```bash
./scripts/run_unique_proofs.sh --fast
./scripts/run_evidence_suite.sh --full   # optional
RUSTFLAGS='-L /opt/cuda/lib64' cargo build -p hub-f64 --release --bin valhalla_base
```

---

## Further reading

- **Product docs:** [valhalla-unified-core](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core)
- **Unique vs memory APIs:** [`valhalla/UNIQUE_VS_MEMORY_APIS.md`](valhalla/UNIQUE_VS_MEMORY_APIS.md)
- **3-minute demo:** [`valhalla/DEMO.md`](valhalla/DEMO.md)
- **Experiment JSON:** `reports/valhalla_inference/`
