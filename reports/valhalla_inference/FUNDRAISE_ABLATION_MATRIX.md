# Fundraise Ablation Matrix

**Protocol:** fundraise-ablation-matrix-v1  
**Generated:** 2026-07-13T01:58:57.802589+00:00

## Constraints

| Field | Value |
|-------|-------|
| Backbone | `Qwen/Qwen2.5-3B-Instruct` |
| LoCoMo n | 15 |
| MultiHop n | 15 |
| RAGTruth n | 40 |
| LongBench n | 10 |
| Scale | moderate |
| Stop on error | True |

## Matrix (correct/n)

| Arm | LoCoMo | MultiHop | RAGTruth | LongBench |
|-----|--------|----------|----------|-----------|
| LM full-context | 9/15 | 8/15 | 0/40 | 2/10 |
| Vanilla RAG | 9/15 | 8/15 | 0/40 | 2/10 |
| Valhalla lm_patch | 1/15 | 8/15 | 4/40 | 1/10 |
| Valhalla native | 0/15 | 9/15 | 34/40 | 3/10 |
| Valhalla hybrid (full) | 0/15 | 9/15 | 34/40 | 1/10 |
| Hybrid − Fate | 0/15 | 9/15 | 34/40 | 1/10 |
| Hybrid − Triad (tile) | 0/15 | 9/15 | 37/40 | 1/10 |

## Component notes

- Fate lift on LoCoMo: +0.0pp (full vs −Fate)
- Triad lift on LoCoMo: +0.0pp (triad vs tile)
- Structure premium on LoCoMo: -60.0pp (hybrid vs vanilla RAG)
- Fate lift on MultiHop: +0.0pp (full vs −Fate)
- Triad lift on MultiHop: +0.0pp (triad vs tile)
- Structure premium on MultiHop: +6.7pp (hybrid vs vanilla RAG)

## Control

- State: **ok**
- Pause file: `/home/jouly/AI/Valhalla/reports/valhalla_inference/ablation.pause`

---

