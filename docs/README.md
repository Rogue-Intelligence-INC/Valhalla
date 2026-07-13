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
| **U5 Structure-cost** | prefill **0.29×** | [`UNIQUE_PROOF_SUITE.md`](../reports/valhalla_inference/UNIQUE_PROOF_SUITE.md) |

**Matrix:** [`UNIQUE_VS_MEMORY_APIS.md`](valhalla/UNIQUE_VS_MEMORY_APIS.md) · **Protocol:** [`UNIQUE_PROOF_MICROBENCH.md`](valhalla/UNIQUE_PROOF_MICROBENCH.md)

```bash
./scripts/run_unique_proofs.sh --fast   # 5/5 citeable (2026-07-14)
```

---

## Supporting pillars

| Pillar | Result | Doc |
|--------|--------|-----|
| **Faithfulness** | RAGTruth **34/40 · 88/100** | [`EVIDENCE_SUITE_SUMMARY_20260713.md`](../reports/valhalla_inference/EVIDENCE_SUITE_SUMMARY_20260713.md) |
| **Session context** | Needle **4/4** · long mem **94.7%** | `long_memory_test_full.json` |
| **Trainability L2/L3** | holdout **6/6** · bake Δ≈298 | [TRAINABILITY_PROOF.md](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/TRAINABILITY_PROOF.md) |

### Industry calibration @ 3B (secondary · 2026-07-13)

| Highlight | Result | Doc |
|-----------|--------|-----|
| Needle | **4/4** | [`BENCHMARK_SUMMARY_20260713.md`](../reports/valhalla_inference/BENCHMARK_SUMMARY_20260713.md) |
| LoCoMo | **17/30 (57%)** — Tier B, not lead | same |
| MultiHop fatewithv2 | **15/25 (60%)** | [`EXPERIMENT_SUMMARY_20260713.md`](../reports/valhalla_inference/EXPERIMENT_SUMMARY_20260713.md) |
| MCQ ladder 46Q | **45/46 (98%)** | same |

**Runtime default LM:** `Qwen2.5-0.5B-Instruct`. Industry suite sets 3B explicitly.

---

## Start here

| Pack | Path | Use |
|------|------|-----|
| **Unique proofs** | [`scripts/run_unique_proofs.sh`](../scripts/run_unique_proofs.sh) | **Lead diligence** U1–U5 |
| **One-click pillars** | [`scripts/run_highlight_test.sh`](../scripts/run_highlight_test.sh) | Pillar smoke |
| **Evidence pack** | [`scripts/run_evidence_suite.sh`](../scripts/run_evidence_suite.sh) | Legitimacy · competitor · zip |
| **Web demo** | [`scripts/demo_valhalla_chat.sh`](../scripts/demo_valhalla_chat.sh) | Backend + Flutter web |
| **Product positioning** | [VALHALLA_BASE_PRODUCT_POSITIONING.md](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/VALHALLA_BASE_PRODUCT_POSITIONING.md) | Unique-first |
| **Investor one-pager** | [INVESTOR_ONE_PAGER.md](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/INVESTOR_ONE_PAGER.md) | Confidential memo |
| **Landing page** | [page/index.html](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/page/index.html) | GitHub Pages intro |
| **Product doc hub** | [`valhalla/README.md`](valhalla/README.md) | VUC links |
| **Incubation** | [INCUBATION_CAPABILITY.md](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/INCUBATION_CAPABILITY.md) | TPI / NPPI |
| **Platform reference** | [VALHALLA_BASE_PLATFORM.md](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/VALHALLA_BASE_PLATFORM.md) | Architecture |

---

## Benchmarks archive

| Benchmark | Tier | Result (latest honest) |
|-----------|------|------------------------|
| Unique proofs U1–U5 | A (capability) | **5/5** (2026-07-14) |
| RAGTruth 40/100 | A | **34/40 · 88/100** (2026-07-13) |
| Needle | A | **4/4** |
| MCQ ladder @ 3B | A (scaling) | **45/46** |
| LoCoMo / MultiHop / LongBench | B | See evidence summary |

**Master:** [`EVIDENCE_SUITE_SUMMARY_20260713.md`](../reports/valhalla_inference/EVIDENCE_SUITE_SUMMARY_20260713.md) · [`UNIQUE_PROOF_SUITE.md`](../reports/valhalla_inference/UNIQUE_PROOF_SUITE.md)

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
