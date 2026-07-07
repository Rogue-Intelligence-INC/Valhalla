# MCQ Ladder Full — 0.5B → 3B → 7B (Measured)

**Date:** 2026-07-07  
**Protocol:** `mcq-ladder-v1` · `valhalla_hybrid_lm_patch` · 46 fair MCQ  
**Run:** `run_mcq_full_ladder.sh` · `valhalla_base serve` + LM weight cache · ~10h11m  
**Artifact:** [`mcq_ladder_v1_full_test.json`](./mcq_ladder_v1_full_test.json)

---

## Results

| Model | Params | Correct | Accuracy |
|-------|--------|---------|----------|
| Qwen2.5-0.5B-Instruct | 0.5B | 29/46 | **63.0%** |
| Qwen2.5-3B-Instruct | 3.0B | 45/46 | **97.8%** |
| Qwen2.5-7B-Instruct | 7.0B | 44/46 | **95.7%** |

**Verdict:** `MCQ_LADDER_SCALES`

---

## Scaling fit (n=3, log₁₀ params)

```
acc ≈ 0.750 + 0.310 · log₁₀(B)     R² = 0.865
```

| B | Measured | Notes |
|---|----------|-------|
| 0.5B | 63% | Structure + lm_patch baseline |
| 3B | **98%** | **Production sweet spot** — near ceiling |
| 7B | 96% | Slightly below 3B (−1 question) — **no further gain at scale** |

### Interpretation

1. **0.5B → 3B:** +35 pp — backbone logprob resolution dominates; Valhalla stack fixed.
2. **3B → 7B:** −2 pp — at ceiling; larger LM does not improve MCQ lm_patch on this slice (possible tie-break noise on 2 items).
3. **Product default:** keep **Qwen2.5-3B-Instruct** — best accuracy/cost/latency tradeoff.

Same Valhalla stack at 0.5B already adds ~+45–50 pp vs trad_lm RAG (see [`MCQ_SCALING_POTENTIAL_20260706.md`](./MCQ_SCALING_POTENTIAL_20260706.md)).

---

## Reproduce

```bash
# Download 7B (once)
.venv-llm/bin/python3 tools/valhalla_inference/download_backbone_models.py --mirror --only qwen --with-7b

# Full ladder (serve + cache)
tools/valhalla_inference/run_mcq_full_ladder.sh
```

---

## Related

- Industry 3B suite: [`INDUSTRY_3B_BENCHMARK_SUMMARY_20260706.md`](./INDUSTRY_3B_BENCHMARK_SUMMARY_20260706.md)
- 0.5B→3B analysis: [`MCQ_SCALING_POTENTIAL_20260706.md`](./MCQ_SCALING_POTENTIAL_20260706.md)

*Rogue Intelligence LNC.*
