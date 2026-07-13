# Valhalla — Integrated Status

**Generated:** 2026-07-13T08:04:09.763328+00:00

## Resource budget

- Ingress: **50%** · max_items **48**
- LM cache max: **1** (serve `/v1/release`)
- Default: **subprocess** (`VALHALLA_BASE_NO_SERVE=1`) — no persistent 3B serve during suite

## Pillars

| Pillar | Result | Verdict |
|--------|--------|---------|
| ragtruth | 34/40 | RAGTRUTH_STRONG |
| needle | 4/4 | NEEDLE_STRONG |
| proactive | PPI 0.35 | ['PROACTIVE_FATE_V2_WEAK', 'NO_HALLUCINATION_ON_MISMATCH', 'GATES_OK'] |

## Independence

- Panel: `practical`
- Verdict: **VALHALLA_BASE_INDEPENDENT**
- Causal tested: 3
- Appendix (encoder/vision): 1
- Memory: MEMORY_INDEPENDENT

## Industry (latest)

| Benchmark | Result | Verdict |
|-----------|--------|---------|
| RAGTruth | 34/40 (85%) | RAGTRUTH_STRONG |
| RAGTruth-100 | 88/100 (88%) | RAGTRUTH_STRONG |
| Needle session | 4/4 (100%) | NEEDLE_STRONG |
| LongBench-v2 | 4/20 (20%) | LONGBENCH_V2_DEVELOPING |
| FRAMES | 1/25 (4%) · Tier C (no wiki extracts) | TIER_C_INVALID |
| MultiHop-RAG | 9/25 (36%) | OPEN_QA_DEVELOPING |
| MultiHop-RAG v2 | 15/25 (60%) | OPEN_QA_VIABLE |
| MultiHop-RAG Fate | 15/25 (60%) | OPEN_QA_VIABLE |
| MultiHop-RAG fatewithv2 | 15/25 (60%) | OPEN_QA_VIABLE |
| LoCoMo | 17/30 (57%) | OPEN_QA_VIABLE |
| RULER-style | 15/20 (75%) | NEEDLE_VIABLE |
| LaRA RAG vs LC | RAG 11/20 · LC 13/20 | STRUCTURE_RAG_BELOW_LC |
| MCQ ladder | 3.0B 98% | MCQ_LADDER_FLAT |
| External holdout | 10/15 (67%) | EXTERNAL_HOLDOUT_VIABLE |
| TPI-v2 | TPI 0.755 | ['STRONG_INFERENCE_TIME_LEARNING', 'FATE_TRANSFER_RESCUES_SESSION'] |

## External Pareto matrix

- Backbone: `Qwen/Qwen2.5-3B-Instruct`
- Verdict: **TOKEN_EFFICIENCY_PARETO, ACC_TRADEOFF_HONEST**

| System | LoCoMo | LongBench | Proactive |
|--------|--------|-----------|-----------|
| full_context_lm | 1/3 | 0/3 | N/A |
| vanilla_rag | 1/3 | 0/3 | N/A |
| mem0 | ?/? | ?/? | N/A |
| letta | ?/? | ?/? | N/A |
| valhalla_base | 0/3 | 0/3 | 0 |

## Fundraise ablation matrix

- Scale: **moderate**

| Arm | LoCoMo | MultiHop | RAGTruth | LongBench |
|-----|--------|----------|----------|-----------|
| LM full-context | 9/15 | 8/15 | 0/40 | 2/10 |
| Vanilla RAG | 9/15 | 8/15 | 0/40 | 2/10 |
| Valhalla lm_patch | 1/15 | 8/15 | 4/40 | 1/10 |
| Valhalla native | 0/15 | 9/15 | 34/40 | 3/10 |
| Valhalla hybrid (full) | 0/15 | 9/15 | 34/40 | 1/10 |
| Hybrid − Fate | 0/15 | 9/15 | 34/40 | 1/10 |
| Hybrid − Triad (tile) | 0/15 | 9/15 | 37/40 | 1/10 |
- Fate lift on LoCoMo: +0.0pp (full vs −Fate)
- Triad lift on LoCoMo: +0.0pp (triad vs tile)
- Structure premium on LoCoMo: -60.0pp (hybrid vs vanilla RAG)
- Fate lift on MultiHop: +0.0pp (full vs −Fate)
- Triad lift on MultiHop: +0.0pp (triad vs tile)
- Structure premium on MultiHop: +6.7pp (hybrid vs vanilla RAG)

## Legitimacy

- Citeable reports: **11/11**
- FRAMES bundle viable: **True**

---

*Rogue Intelligence LNC.*
