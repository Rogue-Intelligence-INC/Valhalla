# Valhalla Inference — Confidence Report v1

**Protocol:** `confidence-report-v1` · **Generated:** 2026-07-02

## Executive summary

| Domain | Key metric | 95% CI |
|--------|------------|--------|
| Grounding | fair_holdout_cold_rag | 87.1% [71.1%, 94.9%] |
| Session routing | transfer_vs_polluted_lift | +38.7pp [+22.6, +54.8] |
| vs Transformer | traditional_lm_rag | 100.0% [89.0%, 100.0%] |
| FOG quality | fog_emitted_precision | 100.0% [83.9%, 100.0%] |
| Proactive | UP_on_topic_given_emit | 100.0% [87.9%, 100.0%] |
| Crosslang | crosslang_accuracy | 100.0% [82.4%, 100.0%] |
| External | external_holdout_accuracy | 100.0% [79.6%, 100.0%] |
| Dialogue | extended_30turn_on_topic | 96.7% [83.3%, 99.4%] |
| Efficiency | prefill_tokens_ratio_valhalla_vs_rag | 0.291× |
| MCQ | mcq_production_max_pick | 65.2% [50.8%, 77.3%] |

## Methods

| Statistic | Interval |
|-----------|----------|
| Proportions (accuracy, precision, abstain) | Wilson 95% |
| Paired lifts (pp) | Bootstrap paired 95% |
| TPI / NPPI / PPI | Bootstrap index 95% |

Display format: **point% [lo%, hi%]** or **index [lo, hi]**

## TPI-v2

Source: `valhalla_base_training_potential_test_v2.json` · Protocol: `valhalla-base-training-potential-v2`

| Metric | Estimate [95% CI] | n |
|--------|-------------------|---|
| isolated_baseline_accuracy | 87.1% [71.1%, 94.9%] | 27/31 |
| polluted_session_accuracy | 48.4% [32.0%, 65.2%] | 15/31 |
| transfer_fresh_accuracy | 87.1% [71.1%, 94.9%] | 27/31 |
| transfer_vs_polluted_lift | +38.7pp [+22.6, +54.8] | paired n=31 |
| hybrid_vs_lm_patch_lift | -12.9pp [-25.8, -3.2] | paired n=31 |
| traditional_lm_rag | 100.0% [89.0%, 100.0%] | 31/31 |
| valhalla_over_traditional_lm | -12.9pp [-25.8, -3.2] | paired n=31 |
| TPI_v2 | 0.755 [0.640, 0.811] | bootstrap |

## NPPI

Source: `new_paradigm_potential_test.json` · Protocol: `valhalla-new-paradigm-potential-v1`

**Falsification flags:** `SESSION_SNAPSHOT_NO_HOLDOUT_LIFT_BEYOND_RAG`, `FAIR_HOLDOUT_RAG_EQUIVALENT`

| Metric | Estimate [95% CI] | n |
|--------|-------------------|---|
| fair_holdout_cold_rag | 87.1% [71.1%, 94.9%] | 27/31 |
| fair_holdout_transfer | 87.1% [71.1%, 94.9%] | 27/31 |
| fair_transfer_vs_polluted | +38.7pp [+22.6, +54.8] | paired n=31 |
| fair_transfer_vs_cold | +0.0pp [+0.0, +0.0] | paired n=31 |
| negative_control_targeted | 75.0% [46.8%, 91.1%] | 9/12 |
| negative_control_distractor | 0.0% [0.0%, 24.2%] | 0/12 |
| swap_fidelity | 100.0% [51.0%, 100.0%] | 4/4 |
| n5_traditional_lm_rag | 100.0% [89.0%, 100.0%] | 31/31 |
| n5_hybrid_over_traditional_lm | -12.9pp [-25.8, -3.2] | paired n=31 |
| NPPI | 0.848 [0.711, 0.924] | bootstrap |

## FOG-v2

Source: `fate_output_gate_v2_test.json` · Protocol: `fate-output-gate-v2`

| Metric | Estimate [95% CI] | n |
|--------|-------------------|---|
| always_emit_accuracy | 87.1% [71.1%, 94.9%] | 27/31 |
| fog_emitted_precision | 100.0% [83.9%, 100.0%] | 20/20 |
| fog_coverage | 64.5% [46.9%, 78.9%] | 20/31 |
| distractor_abstain_rate | 66.7% [39.1%, 86.2%] | 8/12 |

## Proactive-v2

Source: `proactive_fate_v2_test.json` · Protocol: `proactive-fate-v2`

| Metric | Estimate [95% CI] | n |
|--------|-------------------|---|
| UP_on_topic_given_emit | 100.0% [87.9%, 100.0%] | 28/28 |
| on_topic_emit_rate | 80.0% [64.1%, 90.0%] | 28/35 |
| MAR_mismatch_abstain | 88.2% [73.4%, 95.3%] | 30/34 |
| FIR_on_mismatch_emits | 25.0% [4.6%, 69.9%] | 1/4 |
| PPI_v2 | 0.890 [0.826, 0.948] | bootstrap |

## Context-drift

Source: `context_topic_drift_test.json` · Protocol: `context-topic-drift-v2`

| Metric | Estimate [95% CI] | n |
|--------|-------------------|---|
| swap_fidelity | 100.0% [75.8%, 100.0%] | 12/12 |
| swap_hallucination_rate | 0.0% [0.0%, 24.2%] | 0/12 |
| distractor_false_confidence_rate | 0.0% [0.0%, 24.2%] | 0/12 |

## Context-drift-extended

Source: `context_topic_drift_extended_test.json` · Protocol: `context-topic-drift-v2`

| Metric | Estimate [95% CI] | n |
|--------|-------------------|---|
| extended_30turn_on_topic | 96.7% [83.3%, 99.4%] | 29/30 |

## Local-corpus

Source: `local_corpus_demo_test.json` · Protocol: `local-corpus-demo-v1`

| Metric | Estimate [95% CI] | n |
|--------|-------------------|---|
| before_no_corpus | 0.0% [0.0%, 16.1%] | 0/20 |
| cold_targeted | 100.0% [83.9%, 100.0%] | 20/20 |
| after_transfer | 100.0% [83.9%, 100.0%] | 20/20 |
| ingest_vs_none_lift | +100.0pp [+100.0, +100.0] | paired n=20 |

## Crosslang-core

Source: `crosslang_core_unified_test.json` · Protocol: `crosslang-core-unified-v1`

| Metric | Estimate [95% CI] | n |
|--------|-------------------|---|
| crosslang_accuracy | 100.0% [82.4%, 100.0%] | 18/18 |
| core_unified_fact_rate | 100.0% [61.0%, 100.0%] | 6/6 |
| line_agreement_rate | 100.0% [61.0%, 100.0%] | 6/6 |
| CCU | 0.950 [0.950, 0.950] | bootstrap |

## External-holdout

Source: `external_holdout_test.json` · Protocol: `external-holdout-v1`

| Metric | Estimate [95% CI] | n |
|--------|-------------------|---|
| external_holdout_accuracy | 100.0% [79.6%, 100.0%] | 15/15 |

## Token-efficiency

Source: `token_efficiency_test.json` · Protocol: `token-efficiency-v1`

| Metric | Estimate [95% CI] | n |
|--------|-------------------|---|
| wire_chars_ratio_valhalla_vs_rag | 0.163× |  |
| prefill_tokens_ratio_valhalla_vs_rag | 0.291× |  |
| prefill_tokens_rag_with_history | 11845 | 30-turn cumulative prefill |
| thread_on_topic_rate | 100.0% [88.6%, 100.0%] | 30/30 |
| holdout_valhalla_accuracy | 96.8% [83.8%, 99.4%] | 30/31 |

## MCQ-coverage

Source: `mcq_coverage_test.json` · Protocol: `mcq-coverage-v1`

| Metric | Estimate [95% CI] | n |
|--------|-------------------|---|
| mcq_production_hybrid | 65.2% [50.8%, 77.3%] | 30/46 |
| mcq_production_max_pick | 65.2% [50.8%, 77.3%] | 30/46 |
| mcq_structure_fate | 30.4% [19.1%, 44.8%] | 14/46 |
| mcq_native_structure | 30.4% [19.1%, 44.8%] | 14/46 |
| mcq_lm_patch_all | 65.2% [50.8%, 77.3%] | 30/46 |
| mcq_oracle_max_pick | 69.6% [55.2%, 80.9%] | 32/46 |
| mcq_oracle_vs_production_gap | -4.3pp [-10.9, +0.0] | paired n=46 |

## Global gaps

- local demo n=8 → ingest lift CI degenerates at +100pp
- Composite indices bootstrap uses row resampling — point may differ slightly from stored index

## Reproduce

```bash
python3 tools/valhalla_inference/run_confidence_report.py
```
