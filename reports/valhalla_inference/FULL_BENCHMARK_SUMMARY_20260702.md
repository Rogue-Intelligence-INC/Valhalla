# ValhallaBase — Full Benchmark Summary

**Date:** 2026-07-02 · **Suite:** `run_full_benchmark_suite.sh`  
**Log:** `full_benchmark_run_20260702_114524.log` · **Duration:** ~4.7h  
**Stack:** hybrid + native_follow_up_aware + quad_cycle + fate_qa_feedback · Fate `signal_tick` proactive gate

---

## Executive summary

| Tier | Highlights |
|------|------------|
| **A — Industry** | RAGTruth **100/100** · Needle **4/4** · RULER **20/20** · Holdout **15/15** · MCQ ladder **65%→98%** |
| **Training potential** | TPI-v2 **0.755** · NPPI **0.848** · transfer **+38.7pp** vs polluted session |
| **Native** | Long memory **72/76 (94.7%)** · Proactive **71/72 · PPI 0.890** · Crosslang **18/18** |
| **Honest limits** | LongBench **35%** · MultiHop **36%** · LaRA RAG **5/20** · LoCoMo **3%** · FRAMES **invalid** |

**vs Transformer RAG (Qwen2.5-0.5B):** hybrid **87.1%** · lm_patch / trad_lm **100%** on fair open holdout — structure wins on **session routing**, not raw RAG ceiling.

---

## 1. Industry benchmarks (third-party)

| Benchmark | Protocol | Result | Verdict | Cite? |
|-----------|----------|--------|---------|-------|
| RAGTruth | `ragtruth-v1` | **40/40 (100%)** | RAGTRUTH_STRONG | ✅ Tier A |
| RAGTruth-100 | `ragtruth-full-v1` | **100/100 (100%)** | RAGTRUTH_STRONG | ✅ Tier A |
| Needle session | `needle-session-v1` | **4/4 (100%)** | NEEDLE_STRONG | ✅ Tier A |
| RULER-style | `ruler-v1` | **20/20 (100%)** | NEEDLE_STRONG | ✅ Tier A |
| External holdout | `external-holdout-v1` | **15/15 (100%)** | EXTERNAL_HOLDOUT_STRONG | ✅ Tier A |
| MCQ ladder | `mcq-ladder-v1` | **0.5B 65% → 3B 98%** (46Q) | MCQ_LADDER_SCALES | ✅ Tier A |
| LongBench-v2 | `longbench-v2-v1` | **7/20 (35%)** | LONGBENCH_V2_VIABLE | ⚠️ Tier B (0.5B + trunc) |
| MultiHop-RAG | `multihop-rag-v1` | **9/25 (36%)** | OPEN_QA_DEVELOPING | ⚠️ Tier B |
| LaRA RAG vs LC | `lara-v1` | **RAG 5/20 · LC 14/20** | STRUCTURE_RAG_BELOW_LC | ⚠️ Tier B |
| LoCoMo | `locomo-v1` | **1/30 (3%)** | OPEN_QA_DEVELOPING | ❌ slice only |
| FRAMES | `frames-v1` | **0/25 (0%)** | OPEN_QA_DEVELOPING | ❌ no Wikipedia |

---

## 2. Training potential (Valhalla-native)

| Protocol | Metric | Result | Verdict |
|----------|--------|--------|---------|
| `valhalla-base-training-potential-v2` | **TPI-v2** | **0.755** [0.640, 0.811] | STRONG_INFERENCE_TIME_LEARNING |
| | isolated baseline | **27/31 (87.1%)** | |
| | polluted session | **15/31 (48.4%)** | |
| | transfer fresh | **27/31 (87.1%)** | **+38.7pp** vs polluted |
| | data efficiency @10% | **100%** | |
| | hybrid vs lm_patch vs trad_lm | **87.1% / 100% / 100%** | |
| `valhalla-new-paradigm-potential-v1` | **NPPI** | **0.848** [0.711, 0.924] | NEW_PARADIGM_VIABLE |
| | novel knowledge transfer | **6/6 (100%)** | ZERO_WEIGHT_NOVEL_INGEST_OK |
| | falsification | transfer ≈ cold RAG on holdout | NOT_WEIGHT_FINE_TUNE_REPLACEMENT |

---

## 3. Native capability batteries

| Protocol | Result | Notes |
|----------|--------|-------|
| `proactive-fate-v2` | **71/72 · PPI 0.890** | Fate `signal_tick` warmup (min=64); Gates 3/3 |
| `long-memory-v1` production | **72/76 (94.7%)** | extended_20 56/60 |
| `fate-output-gate-v2` | emit precision **100%** (20/20) | coverage 64.5%; FOG_V2_NEEDS_TUNING |
| `crosslang-core-unified-v1` | **18/18 · CCU 0.95** | |
| `context-topic-drift-v1` | **skipped** | needs `.venv-llm` (torch) |
| `token-efficiency-v1` | **skipped** | needs `.venv-llm` (transformers) |

---

## 4. Confidence report (Wilson + bootstrap)

See `CONFIDENCE_REPORT_v1.md` — key 95% CIs:

| Domain | Metric | CI |
|--------|--------|-----|
| Grounding | fair_holdout_cold_rag | 87.1% [71.1%, 94.9%] |
| Session routing | transfer_vs_polluted_lift | +38.7pp [+22.6, +54.8] |
| vs Transformer | traditional_lm_rag | 100.0% [89.0%, 100.0%] |
| Proactive | UP_on_topic_given_emit | 100.0% [87.9%, 100.0%] |
| External | external_holdout | 100.0% [79.6%, 100.0%] |

---

## 5. Suite status

| Step | Status |
|------|--------|
| download-bundles | ✅ |
| industry-benchmarks --all | ✅ |
| TPI-v2 + NPPI + trad_lm | ✅ |
| proactive-fate-v2 | ✅ |
| long-memory-v1 | ✅ |
| fate-output-gate-v2 | ✅ |
| crosslang-core-unified | ✅ |
| context-topic-drift | ⚠️ env (use `.venv-llm/bin/python3`) |
| token-efficiency | ⚠️ env (use `.venv-llm/bin/python3`) |
| confidence-report | ✅ |

---

## Reproduce

```bash
RUSTFLAGS="-L /opt/cuda/lib64" tools/valhalla_inference/run_full_benchmark_suite.sh
```

Or stepwise: `run_industry_benchmarks.py --download-first --all` + individual `test_*.py` scripts.

---

*Rogue Intelligence LNC.*
