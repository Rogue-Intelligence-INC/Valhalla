# Hybrid Valhalla 2.0 — Investor Document Pack

**Product version**: hybrid-2.0 · **Engine**: hybrid-4.0-rust-pipeline · **Fair benchmark**: fair-1.2  
**Publisher**: Rogue Intelligence LNC. · **Date**: 2026-06-23  
**Benchmark anchor**: Fair Benchmark test split · 145 questions · Qwen2.5-0.5B-Instruct companion

---

## Headline metrics (test split)

| Metric | Hybrid unified |
|--------|----------------|
| **Total accuracy** | **80.7%** (117/145) |
| open | 93.4% |
| mcq | **60.9%** (28/46) |
| numeric | 84.2% |

**MCQ note**: +8.7 points vs memory-first deploy (52.2%) via structure-guided patch LM routing.

---

## Document index

| Type | English | 中文 |
|------|---------|------|
| **Day summary (2026-06-23)** | [DAY_SUMMARY_20260623.md](./DAY_SUMMARY_20260623.md) | 中英 |
| **Architecture reference** | [ARCHITECTURE_DETAILED.md](./ARCHITECTURE_DETAILED.md) | 中英对照 |
| **Pitch** | [EN_PITCH.md](./EN_PITCH.md) | [ZH_PITCH.md](./ZH_PITCH.md) |
| **Pitch deck** | [EN_PITCH_DECK.md](./EN_PITCH_DECK.md) | [ZH_PITCH_DECK.md](./ZH_PITCH_DECK.md) |
| **Technical whitepaper** | [EN_TECHNICAL_WHITEPAPER.md](./EN_TECHNICAL_WHITEPAPER.md) | [ZH_TECHNICAL_WHITEPAPER.md](./ZH_TECHNICAL_WHITEPAPER.md) |
| **Due diligence pack** | [EN_DUE_DILIGENCE.md](./EN_DUE_DILIGENCE.md) | [ZH_DUE_DILIGENCE.md](./ZH_DUE_DILIGENCE.md) |
| **Business plan** | [EN_BUSINESS_PLAN.md](./EN_BUSINESS_PLAN.md) | [ZH_BUSINESS_PLAN.md](./ZH_BUSINESS_PLAN.md) |
| **Paper draft (memory model)** | [../../papers/memory_hybrid/Hybrid_Structure_Memory_and_Routing.md](../../papers/memory_hybrid/Hybrid_Structure_Memory_and_Routing.md) | — |

---

## Related artifacts

| Artifact | Location |
|----------|----------|
| **Latest Hybrid JSON** | `reports/hybrid_valhalla_lm/hybrid_v4_mcq_patch_strengthen_final.json` |
| Prior unified JSON | `reports/hybrid_valhalla_lm/hybrid_unified_test_20260622_0345.json` |
| Public viz repo | [Rogue-Intelligence-INC/valhalla-transformer](https://github.com/Rogue-Intelligence-INC/valhalla-transformer) v2.0 |
| Rust runner | `./target/release/valhalla_hybrid --phase test --merge-mode deploy` |

**Confidential** — for qualified investors and design partners only.
