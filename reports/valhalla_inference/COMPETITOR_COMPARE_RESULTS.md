# Competitor Compare Results

**Protocol:** competitor-compare-v1  
**Generated:** 2026-07-13T00:40:53.496018+00:00  
**Backbone:** `Qwen/Qwen2.5-3B-Instruct`

## Matrix

### locomo (n=15)

| System | Acc | Correct/n | Tok/q | Notes |
|--------|-----|-----------|-------|-------|
| full_context_lm | 60.0% | 9/15 | 17386.1 | |
| vanilla_rag | 60.0% | 9/15 | 17386.1 | |
| valhalla_base | 0.0% | 0/15 | 97.1 | |
| mem0 | 33.3% | 5/15 | 420.4 | |

### multihop (n=10)

| System | Acc | Correct/n | Tok/q | Notes |
|--------|-----|-----------|-------|-------|
| full_context_lm | 60.0% | 6/10 | 303.3 | |
| vanilla_rag | 60.0% | 6/10 | 303.3 | |
| valhalla_base | 60.0% | 6/10 | 244.0 | |
| mem0 | 60.0% | 6/10 | 303.3 | |

### longbench (n=10)

| System | Acc | Correct/n | Tok/q | Notes |
|--------|-----|-----------|-------|-------|
| full_context_lm | 20.0% | 2/10 | 3043.8 | |
| vanilla_rag | 20.0% | 2/10 | 3043.8 | |
| valhalla_base | 10.0% | 1/10 | 568.0 | |
| mem0 | 20.0% | 2/10 | 1196.4 | |

## Policy

- Mem0/Letta SKIP is honest when API/deps missing — do not invent scores.
- Primary claims: LoCoMo + tokens; MultiHop/LongBench = boundaries.
- See `docs/valhalla/COMPETITOR_COMPARE.md`.

