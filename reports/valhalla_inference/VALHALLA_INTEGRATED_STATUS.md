# Valhalla — Integrated Status

**Generated:** 2026-07-10T22:12:28.441995+00:00

## Resource budget

- Ingress: **50%** · max_items **48**
- LM cache max: **1** (serve `/v1/release`)
- Default: **subprocess** (`VALHALLA_BASE_NO_SERVE=1`) — no persistent 3B serve during suite

## Pillars

| Pillar | Result | Verdict |
|--------|--------|---------|
| ragtruth | 40/40 | RAGTRUTH_STRONG |
| needle | 2/2 | NEEDLE_STRONG |
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
| RAGTruth | 40/40 (100%) | RAGTRUTH_STRONG |
| RAGTruth-100 | 10/10 (100%) | RAGTRUTH_STRONG |
| Needle session | 2/2 (100%) | NEEDLE_STRONG |
| LongBench-v2 | 7/20 (35%) | LONGBENCH_V2_VIABLE |
| FRAMES | SKIPPED (Tier C — no wiki corpus) | TIER_C_INVALID |
| MultiHop-RAG | 9/25 (36%) | OPEN_QA_DEVELOPING |
| MultiHop-RAG v2 | 15/25 (60%) | OPEN_QA_VIABLE |
| MultiHop-RAG Fate | 13/25 (52%) | OPEN_QA_VIABLE |
| MultiHop-RAG hybrid | 14/25 (56%) | OPEN_QA_VIABLE |
| LoCoMo | 1/3 (33%) | OPEN_QA_DEVELOPING |
| RULER-style | 4/4 (100%) | NEEDLE_STRONG |
| LaRA RAG vs LC | ?/5 | STRUCTURE_RAG_BELOW_LC |
| MCQ ladder | 3.0B 98% | MCQ_LADDER_FLAT |
| External holdout | 5/5 (100%) | EXTERNAL_HOLDOUT_STRONG |
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

## Legitimacy

- Citeable reports: **10/11**
- FRAMES bundle viable: **False**

---

*Rogue Intelligence LNC.*
