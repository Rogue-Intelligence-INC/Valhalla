# External Pareto Matrix

**Protocol:** external-pareto-matrix-v1  
**Generated:** 2026-07-10T22:12:10.287142+00:00

## Locked constraints

| Field | Value |
|-------|-------|
| Backbone | `Qwen/Qwen2.5-3B-Instruct` |
| LoCoMo items | 3 |
| LongBench items | 3 |
| Scoring | LoCoMo: `score_short_answer` · LongBench: MCQ letter |
| Budget | ingress 50% · LM cache max 1 |

## Matrix

| System | LoCoMo | LongBench | Token/query | Latency ms | Session reload | Proactive |
|--------|--------|-----------|-------------|------------|----------------|-----------|
| full_context_lm | 1/3 | 0/3 | 17386.0 | 2801.3 | 199299 | N/A |
| vanilla_rag | 1/3 | 0/3 | 17386.0 | 2699.1 | 199299 | N/A |
| mem0 | ?/? | ?/? | None | None | None | N/A |
| letta | ?/? | ?/? | None | None | None | N/A |
| valhalla_base | 0/3 | 0/3 | 98.7 | 276208.3 | 200463 | 0 |

**Verdict:** TOKEN_EFFICIENCY_PARETO, ACC_TRADEOFF_HONEST

---

