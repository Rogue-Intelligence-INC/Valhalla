# Hybrid Valhalla 2.0 — Investor Document Pack

**Product version**: hybrid-2.0 · **Engine**: hybrid-4.0-rust-pipeline · **Fair benchmark**: fair-1.2  
**Publisher**: Rogue Intelligence LNC. · **Date**: 2026-06-23  
**Benchmark anchor**: Fair Benchmark test split · 145 questions · Qwen2.5-0.5B-Instruct companion

---

## Headline metrics (test split)

**Primary artifact**: `reports/hybrid_valhalla_lm/hybrid_v4_mcq_patch_strengthen_final.json`  
**Reproduce**: `valhalla_hybrid --phase test --merge-mode deploy` (see DAY_SUMMARY)

| Metric | Hybrid deploy | Fraction |
|--------|---------------|----------|
| **Total accuracy** | **80.69%** | **117/145** |
| open | 93.44% | 57/61 |
| mcq | **60.87%** | **28/46** |
| numeric | 84.21% | 32/38 |

**MCQ deploy uplift** (same split, `hybrid_v4_test_deploy_post_decode_fix.json` → final): **24/46 (52.17%) → 28/46 (60.87%)**, **+4 questions (+8.70 pp)**.

**Baselines (same fair test split, cited JSON)**:

| Track | Total | MCQ | Source |
|-------|-------|-----|--------|
| Qwen2.5-0.5B zero-shot | 103/145 (71.03%) | 17/46 (36.96%) | `fair_benchmark_test_20260620_0859.json` |
| Structure memory (tile+fate persistent) | 59/145 (40.69%) | 23/46 (50.00%) | final JSON `summary.memory_tile_fate_persistent` |
| Patch LM track (final run) | 117/145 (80.69%) | 28/46 (60.87%) | final JSON `summary.patch_lm_subset` |

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

## Multimodal fusion (structure layer)

Text + TTS → Tile/Stem Triad. Bench: `reports/tts_tile_stem_fusion/MULTIMODAL_FUSION_BENCH.md`

| Gate | Result |
|------|--------|
| Tile superadditivity | 58.3% |
| Stem superadditivity | 91.7% |
| Overall | **PASS** |

QA pilot (open memory): `tools/fair_benchmark/run_multimodal_memory_pilot.py`

**Hybrid open + v4 recompose (multimodal)**:

```bash
python3 tools/fair_benchmark/run_hybrid_multimodal_open.py --phase test
# → TTS audio JSON + valhalla_hybrid --multimodal-audio-json
# Pipeline: OpenRetrieval decompose + audio ingress + open_structure_score patch bias
```

Env: `VALHALLA_MULTIMODAL_FUSION=1` · `VALHALLA_OPEN_PATCH_BIAS_SCALE=0.15`

---

## Related artifacts

| Artifact | Location |
|----------|----------|
| Hybrid deploy JSON (final) | `reports/hybrid_valhalla_lm/hybrid_v4_mcq_patch_strengthen_final.json` |
| Memory-first deploy (MCQ baseline) | `reports/hybrid_valhalla_lm/hybrid_v4_test_deploy_post_decode_fix.json` |
| **Multimodal fusion bench** | `reports/tts_tile_stem_fusion/multimodal_fusion_bench.json` (**PASS** v1.0) |
| Fair test suite | `reports/fair_benchmark/fair_benchmark_test_20260620_0859.json` |
| Public mirror | [valhalla-transformer](https://github.com/Rogue-Intelligence-INC/valhalla-transformer) `experiments/hybrid_unified_latest.json` |
| Archived pre-2.0 docs | `docs/archive/pre-hybrid-2.0/` |

**Rule**: All percentages in this pack must match a cited JSON row (`correct/total`); do not round headline metrics for marketing.
