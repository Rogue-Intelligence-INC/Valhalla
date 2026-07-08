# 3B 全行业 Benchmark 套件 — 已完成

**完成:** 2026-07-08 · **耗时:** 1736s (~29 min)  
**主干:** `Qwen/Qwen2.5-3B-Instruct` · GPU · serve @8787  
**日志:** `reports/valhalla_inference/industry_suite_run.log`

## Tier A — 强项（100%）

| Benchmark | 结果 | Verdict |
|-----------|------|---------|
| RAGTruth-40 | **40/40** | RAGTRUTH_STRONG |
| RAGTruth-100 | **100/100** | RAGTRUTH_STRONG |
| Needle session | **4/4** | NEEDLE_STRONG |
| RULER-style | **20/20** | NEEDLE_STRONG |
| External holdout | **15/15** | EXTERNAL_HOLDOUT_STRONG |

## Tier B — 可用 / 发展中

| Benchmark | 结果 | Verdict |
|-----------|------|---------|
| LongBench-v2 | **7/20 (35%)** | LONGBENCH_V2_VIABLE |
| MultiHop hybrid | **14/25 (56%)** | OPEN_QA_VIABLE |
| MultiHop v2 | **15/25 (60%)** | OPEN_QA_VIABLE |
| LaRA | RAG **8/20** · LC **13/20** | STRUCTURE_RAG_BELOW_LC |
| LoCoMo | **1/30 (3%)** | OPEN_QA_DEVELOPING |
| FRAMES | **0/25 (0%)** | OPEN_QA_DEVELOPING |

## MCQ ladder — GPU F16 补跑（2026-07-08）

| 模型 | 结果 | 状态 |
|------|------|------|
| 0.5B | **29/46 (63.0%)** | ✅ F16 · isolated session |
| 3B | **44/46 (95.7%)** | ✅ F16 GPU |
| 7B | **45/46 (97.8%)** | ✅ F16 GPU |

**根因修复:** 长 batch session snapshot >32MB → broken pipe；改为 per-question `clear_context()` + per-model serve restart。

**文档:** [`MCQ_LADDER_MEM_GPU_20260708.md`](./MCQ_LADDER_MEM_GPU_20260708.md)

## 汇总

- JSON: `industry_benchmarks_summary.json`
- Markdown: `INDUSTRY_BENCHMARKS_20260630.md`
- 总览: `BENCHMARK_SUMMARY_20260708.md`
- Checkpoint: `checkpoints/industry_suite_checkpoint.json`（11/11 ✅）

## 复现

```bash
RUSTFLAGS='-L /opt/cuda/lib64' VALHALLA_USE_CPU=0 \
  .venv-llm/bin/python3 tools/valhalla_inference/run_industry_benchmarks.py \
  --all --verbose --checkpoint-dir reports/valhalla_inference/checkpoints

# MCQ ladder（GPU F16 · memory-safe）
export VALHALLA_LM_F16=1 VALHALLA_USE_CPU=0
tools/valhalla_inference/run_mcq_full_ladder.sh
```
