# Hybrid Valhalla 2.0 — Due Diligence Pack

**Document type**: Investor / partner due diligence  
**Version**: hybrid-2.0 · hybrid-4.0-rust-pipeline · fair-1.2 · hub-f64 1.1.0  
**Date**: 2026-06-23  
**Publisher**: Rogue Intelligence LNC.  
**Classification**: Confidential — qualified investors and design partners

---

## 1. Purpose

This pack supports **technical and commercial due diligence** on Hybrid Valhalla **hybrid-2.0** — the unified Q&A product shipping June 2026. It maps **claims → evidence → risks → open items** for audit-ready review.

---

## 2. Company snapshot

| Field | Detail |
|-------|--------|
| Legal entity | Rogue Intelligence LNC. (游侠智能有限公司) |
| Product | Hybrid Valhalla hybrid-2.0 |
| Stage | Seed / Pre-seed — runnable product, early commercialization |
| Core repos | Valhalla (private/internal) · [valhalla-transformer](https://github.com/Rogue-Intelligence-INC/valhalla-transformer) (public viz + JSON v2.0) |
| IP scope | Hub / Tile / StemCell / Fate runtime, Wave language, Val-OS, Yaksha (modular) |

*Team roster, cap table, and financial statements: provided under NDA in data room.*

---

## 3. Product claims verification matrix

| # | Claim | Metric | Source artifact | Verifier action | Status |
|---|-------|--------|-----------------|-----------------|--------|
| C1 | Unified Hybrid total accuracy | **80.69%** (117/145) | `hybrid_v4_mcq_patch_strengthen_final.json` | Re-run `valhalla_hybrid --phase test --merge-mode deploy` | ✓ |
| C2 | Open accuracy | **93.44%** (57/61) | Same JSON · `summary.hybrid.by_score_type.open` | Inspect row-level `patch_lm` source | ✓ |
| C3 | MCQ accuracy | **60.87%** (28/46) | Same JSON | Compare 46 MCQ ids vs gold | ✓ |
| C4 | Numeric accuracy | **84.21%** (32/38) | Same JSON | Numeric subset only | ✓ |
| C5 | MCQ deploy uplift vs memory-first | 52.2% → 60.9% (+4) | `hybrid_v4_test_deploy_post_decode_fix.json` vs final | Deploy policy diff | ✓ |
| C6 | Patch LM MCQ | 28/46 | Same JSON · patch subset | Independent patch track | ✓ |
| C7 | No corpus answer leakage | fair-1.2 design | `fair_benchmark_suite.json` + corpus files | Confirm no gold in corpus | ✓ |
| C8 | Routing trace per answer | `hybrid_source` field | Hybrid JSON rows | Sample 20 rows | ✓ |
| C9 | Public API serves latest | `/api/hybrid-unified/latest` | valhalla-transformer v2.0 | `curl` + schema check | ✓ |
| C10 | General LLM SOTA | — | — | Not claimed | N/A |

---

## 4. Artifact inventory

### 4.1 Primary (Hybrid 2.0)

| Artifact | Location |
|----------|----------|
| Hybrid unified JSON | `reports/hybrid_valhalla_lm/hybrid_v4_mcq_patch_strengthen_final.json` |
| Day summary (2026-06-23) | `docs/hybrid_2.0/DAY_SUMMARY_20260623.md` |
| Paper draft (memory model) | `papers/memory_hybrid/Hybrid_Structure_Memory_and_Routing.md` |
| Hybrid report (prior) | `reports/VALHALLA_HYBRID_UNIFIED_test_20260622_0345.md` |
| Router source | `manifestsys/hub-f64/src/hybrid.rs` |
| Patch LM + MCQ resolve | `manifestsys/hub-f64/src/patch_lm.rs` |
| Pipeline + merge | `manifestsys/hub-f64/src/pipeline.rs` |
| Option affinity | `manifestsys/hub-f64/src/task_decompose.rs` |
| Aggregate option scores | `manifestsys/hub-f64/src/mcq_fate.rs` |

### 4.2 Public mirror (valhalla-transformer v2.0)

| Artifact | Path in public repo |
|----------|---------------------|
| Latest hybrid JSON | `experiments/hybrid_unified_latest.json` |
| Timestamped run | `experiments/hybrid_unified_test_20260622_0345.json` |
| Docs 13/14 | `en/13_*`, `zh/13_*`, `en/14_*`, `zh/14_*` |
| API | `api/server.js` — `/api/hybrid-unified` |

### 4.3 Supporting fair benchmark

| Artifact | Location |
|----------|----------|
| Fair test base | `reports/fair_benchmark/fair_benchmark_test_20260620_0859.json` |
| Suite spec | `tools/fair_benchmark/fair_benchmark_suite.json` |

---

## 5. Reproduction protocol (auditor checklist)

```bash
# 1. Clone Valhalla (data room access)
# 2. Build native QA
cd manifestsys/hub-f64
RUSTFLAGS='-L /opt/cuda/lib64' cargo build -p hub-f64 --release --bin valhalla_native_qa

# 3. Set paths
export VALHALLA_NATIVE_QA=/path/to/target/release/valhalla_native_qa
export VALHALLA_MCQ_FATE_CYCLES=1   # for Tile MCQ memory track

# 4. Run Hybrid (may require Qwen weights for patch track)
python3 tools/fair_benchmark/run_hybrid_valhalla_lm.py \
  --phase test \
  --reuse-json reports/fair_benchmark/fair_benchmark_test_20260620_0859.json

# 5. Compare output JSON summary.hybrid.acc to 0.80 (± env tolerance)
```

**Expected runtime**: Depends on GPU/CPU; patch track dominates. Document machine spec in audit log.

**Skip-patch mode**: Validates memory-only merge logic; open/numeric will **not** match 80% headline.

---

## 6. Architecture & code maturity

| Component | Maturity | Notes |
|-----------|----------|-------|
| Hybrid router | **Product** | `HYBRID_PRODUCT_VERSION = "hybrid-2.0"` |
| Fair benchmark harness | **Product** | fair-1.2 splits, published JSON |
| native_qa MCQ decode | **Beta** | `mcq_option`; v2 decode reverted |
| MCQ Fate cycles | **Beta** | Tile-only best; env-gated |
| Patch bridge | **Product** | Tier A; open path proven |
| Enterprise SDK | **Alpha** | Routing trace in JSON; no formal SDK yet |
| Wave / Val-OS / Yaksha | **Parallel SKUs** | Optional; not required for Hybrid score |

---

## 7. Risk register

| ID | Risk | Likelihood | Impact | Mitigation |
|----|------|------------|--------|------------|
| R1 | MCQ accuracy plateau (~54%) | High | Medium | P0 relative comparator; stem-conditioned scoring |
| R2 | Small LM (0.5B) limits numeric/open ceiling | Medium | Medium | Customer-chosen LM; structure layer model-agnostic |
| R3 | Corpus scale (49 lines) not production | Medium | High | Design-partner corpora 110+ lines; fair-2.0 |
| R4 | Reproduction env sensitivity (CUDA, Qwen paths) | Medium | Medium | Docker repro package (roadmap); documented env |
| R5 | Investor over-reads 80% as "GPT killer" | Medium | High | **By-type reporting**; explicit non-goals in all docs |
| R6 | Tier B open alone weak (~25%) | Known | Low | Product uses Hybrid; documented in §3.3 summary |
| R7 | IP / export control (dual-use AI) | Low | Medium | Legal review per jurisdiction; on-prem default |
| R8 | Key-person dependency | Medium | High | Document architecture; modular bins; hire plan in BP |

---

## 8. What we explicitly do NOT claim

- SOTA on MMLU, HumanEval, or general reasoning vs frontier models  
- FIPS / Common Criteria certification (Yaksha has Industry Gate narrative, not cert)  
- +4.17 pp Hub patch gains (invalidated under strict MCQ — see correction record)  
- MCQ solved at human level  
- Single-track 80% without Hybrid routing  

---

## 9. Legal & compliance (overview)

| Topic | Status |
|-------|--------|
| Open-source components | Rust ecosystem, Qwen weights (Apache 2.0 model license — verify current HF card) |
| Customer data | On-prem processing default; no mandatory cloud inference |
| Benchmark integrity | Corpus decontamination; correction record maintained |
| Export / AI regulations | Assess per deployment geography (CN, US, EU) — **legal counsel required** |

---

## 10. Commercial DD hooks

| Question | Answer |
|----------|--------|
| Who buys first? | Sovereign AI / vertical SaaS / SI — see BP §4 |
| Pricing model? | License + POC; not finalized — indicative in BP |
| Competition? | API providers, RAG stacks, LangChain — Hybrid wedge is audit + structure |
| Moat? | Fair benchmark discipline + shipped hybrid product + modular Rust runtime |
| 18-mo milestone? | MCQ 65%+, 3 paid POCs, SDK GA |

---

## 11. Open items for data room (fill on request)

- [ ] Cap table & prior funding  
- [ ] Team bios & ESOP  
- [ ] 12-month cash forecast  
- [ ] Customer LOIs / POC pipeline  
- [ ] IP assignment chain  
- [ ] Docker repro image for Hybrid full run  
- [ ] Third-party benchmark audit letter  

---

## 12. Contact & document index

| Document | Path |
|----------|------|
| Pitch | `docs/hybrid_2.0/EN_PITCH.md` |
| Pitch deck | `docs/hybrid_2.0/EN_PITCH_DECK.md` |
| Technical whitepaper | `docs/hybrid_2.0/EN_TECHNICAL_WHITEPAPER.md` |
| Business plan | `docs/hybrid_2.0/EN_BUSINESS_PLAN.md` |
| This DD pack | `docs/hybrid_2.0/EN_DUE_DILIGENCE.md` |

---

*Rogue Intelligence LNC. · hybrid-2.0 · fair-1.2 · hub-f64 1.1.0 · Confidential · 2026-06-22*
