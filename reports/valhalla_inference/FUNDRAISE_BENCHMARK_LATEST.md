# ValhallaBase — Fundraise Benchmark Dashboard

**Generated:** 2026-07-07T23:17:17.600297+00:00

**Investor gate:** 7 PASS · 5 WATCH · 3 GAP
**Fundraise-ready heuristic:** YES

## Headline metrics

| Capability | Result | Verdict | Gate |
|------------|--------|---------|------|
| Proactive Agency v1 | **Agency 0.000 · 3000 rounds** | AGENCY_WEAK | GAP |
| Proactive Fate v2 | **16/16 (100.0%)** | PROACTIVE_FATE_V2_STRONG | PASS |
| Fate output gate v2 | **50% checks** | FOG_V2_NEEDS_TUNING | WATCH |
| Long memory v1 | **76/76 (100%)** | STRONG | PASS |
| RAGTruth | **40/40 (100%)** | RAGTRUTH_STRONG | PASS |
| RAGTruth-100 | **100/100 (100%)** | RAGTRUTH_STRONG | PASS |
| Needle session | **4/4 (100%)** | NEEDLE_STRONG | PASS |
| External holdout | **15/15 (100%)** | EXTERNAL_HOLDOUT_STRONG | PASS |
| RULER-style | **20/20 (100%)** | NEEDLE_STRONG | PASS |
| LaRA RAG | **12/20 (60%)** | STRUCTURE_RAG_BELOW_LC | WATCH |
| MultiHop-RAG v1 | **9/25 (36%)** | OPEN_QA_DEVELOPING | GAP |
| MultiHop-RAG v2 | **15/25 (60%)** | OPEN_QA_VIABLE | WATCH |
| MultiHop-RAG Fate | **13/25 (52%)** | OPEN_QA_VIABLE | WATCH |
| MultiHop-RAG hybrid | **16/25 (64%)** | OPEN_QA_VIABLE | WATCH |
| NPPI | **—** | NEW_PARADIGM_VIABLE | GAP |

## Continuous run

```bash
# Full cycle (~30–45 min)
./tools/valhalla_inference/run_fundraise_continuous.sh

# Every 6 hours (background)
nohup ./tools/valhalla_inference/run_fundraise_continuous.sh --loop 6 >> reports/valhalla_inference/fundraise_cron.log 2>&1 &
```

---

*Rogue Intelligence LNC — for investor due diligence.*
