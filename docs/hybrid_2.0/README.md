# Hybrid Valhalla 2.0 — Investor Document Pack

**Product version**: hybrid-2.0 · **Fair benchmark**: fair-1.2 · **Runtime**: hub-f64 1.1.0  
**Publisher**: Rogue Intelligence LNC. · **Date**: 2026-06-22  
**Benchmark anchor**: Fair Benchmark test split · 145 questions · Qwen2.5-0.5B-Instruct companion

---

## Headline metrics (test split)

| Metric | Hybrid unified |
|--------|----------------|
| **Total accuracy** | **80.0%** (116/145) |
| open | 95.1% |
| mcq | 54.4% |
| numeric | 86.8% |

---

## Document index

| Type | English | 中文 |
|------|---------|------|
| **Architecture reference** (charts + file map) | [ARCHITECTURE_DETAILED.md](./ARCHITECTURE_DETAILED.md) | 中英对照 |
| **Internal pipeline v4** (LM decompose → Valhalla → recompose) | [PIPELINE_V4_LM_DECOMPOSE_VALHALLA_RECOMPOSE.md](./PIPELINE_V4_LM_DECOMPOSE_VALHALLA_RECOMPOSE.md) | 内部 |
| **Engineering gap analysis** (松散性 · 收紧路线图) | [INTERNAL_ENGINEERING_GAP_ANALYSIS.md](./INTERNAL_ENGINEERING_GAP_ANALYSIS.md) | 内部 |
| **Pitch** (2-min / email) | [EN_PITCH.md](./EN_PITCH.md) | [ZH_PITCH.md](./ZH_PITCH.md) |
| **Pitch deck** (slide outline) | [EN_PITCH_DECK.md](./EN_PITCH_DECK.md) | [ZH_PITCH_DECK.md](./ZH_PITCH_DECK.md) |
| **Technical whitepaper** | [EN_TECHNICAL_WHITEPAPER.md](./EN_TECHNICAL_WHITEPAPER.md) | [ZH_TECHNICAL_WHITEPAPER.md](./ZH_TECHNICAL_WHITEPAPER.md) |
| **Due diligence pack** | [EN_DUE_DILIGENCE.md](./EN_DUE_DILIGENCE.md) | [ZH_DUE_DILIGENCE.md](./ZH_DUE_DILIGENCE.md) |
| **Business plan** | [EN_BUSINESS_PLAN.md](./EN_BUSINESS_PLAN.md) | [ZH_BUSINESS_PLAN.md](./ZH_BUSINESS_PLAN.md) |

---

## Related artifacts

| Artifact | Location |
|----------|----------|
| Unified Hybrid JSON | `reports/hybrid_valhalla_lm/hybrid_unified_test_20260622_0345.json` |
| MCQ & Fair summary | `reports/VALHALLA_MCQ_AND_FAIR_BENCHMARK_SUMMARY_20260622.md` |
| Public viz repo | [Rogue-Intelligence-INC/valhalla-transformer](https://github.com/Rogue-Intelligence-INC/valhalla-transformer) v2.0 |
| Runner | `python tools/fair_benchmark/run_hybrid_valhalla_lm.py --phase test` |

**Confidential** — for qualified investors and design partners only.
