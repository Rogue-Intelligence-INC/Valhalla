# ValhallaBase Benchmark Summary — 2026-07-08

**Backbone:** `Qwen/Qwen2.5-3B-Instruct` · CUDA · `VALHALLA_LM_F16` for memory-safe MCQ  
**Reports dir:** `reports/valhalla_inference/`

---

## 1. Industry suite @ 3B — ✅ complete (~29 min)

| Tier | Benchmark | Result | Verdict |
|------|-----------|--------|---------|
| **A** | RAGTruth-40 | **40/40** | RAGTRUTH_STRONG |
| **A** | RAGTruth-100 | **100/100** | RAGTRUTH_STRONG |
| **A** | Needle session | **4/4** | NEEDLE_STRONG |
| **A** | RULER-style | **20/20** | NEEDLE_STRONG |
| **A** | External holdout | **15/15** | EXTERNAL_HOLDOUT_STRONG |
| **B** | LongBench-v2 | **7/20 (35%)** | LONGBENCH_V2_VIABLE |
| **B** | MultiHop hybrid | **14/25 (56%)** | OPEN_QA_VIABLE |
| **B** | MultiHop v2 | **15/25 (60%)** | OPEN_QA_VIABLE |
| **B** | LaRA | RAG **8/20** · LC **13/20** | STRUCTURE_RAG_BELOW_LC |
| **B** | LoCoMo | **1/30 (3%)** | OPEN_QA_DEVELOPING |
| **B** | FRAMES | **0/25 (0%)** | OPEN_QA_DEVELOPING |
| **B** | MCQ ladder | **0.5B 63% · 3B 96% · 7B 98%** (F16 GPU) | MCQ_LADDER_SCALES |

**Docs:** [`INDUSTRY_3B_RUN_STATUS.md`](./INDUSTRY_3B_RUN_STATUS.md) · [`industry_benchmarks_summary.json`](./industry_benchmarks_summary.json)

---

## 2. Proactive agency — ✅ 审计 + 重跑（2026-07-08）

| 指标 | 结果 |
|------|------|
| **Agency Index** | **0.644** (AGENCY_VIABLE) |
| **PPI-v2 自动反应** | **0.650** · 68/72 pass |
| **门控 MAR / FIR** | **100% / 0%** |
| Crosslang CCU | **100%** |
| Multimodal | **67%** |
| Mega 3000轮 | 140s infra（**不计 Agency**）|

**四轨公式:** `0.25·long + 0.20·multimodal + 0.20·crosslang + 0.35·PPI-v2`

**审计:** [`PROACTIVE_AGENCY_AUDIT_20260708.md`](./PROACTIVE_AGENCY_AUDIT_20260708.md)  
**报告:** [`PROACTIVE_AGENCY_BENCHMARK.md`](./PROACTIVE_AGENCY_BENCHMARK.md)

---

## 3. MCQ ladder — memory-safe GPU F16 (补跑)

| Fix | Purpose |
|-----|---------|
| `--memory-safe` per-model serve | Cap VRAM across 0.5B / 3B / 7B |
| `VALHALLA_LM_F16=1` | Half weight memory |
| `wait_infer_ready()` | Avoid serve race on model load |
| Isolated session per MCQ | Prevent HTTP 32MB / broken pipe @ ~Q14 |

**Measured (2026-07-08 F16 GPU):** **0.5B 63% · 3B 96% · 7B 98%**

**Doc:** [`MCQ_LADDER_MEM_GPU_20260708.md`](./MCQ_LADDER_MEM_GPU_20260708.md)

---

## 4. Harness fixes (2026-07-08)

- `run_industry_benchmarks.py`: `bench_flags()` only on Rust demos (fixes Python exit 2)
- `benchmark_env.py`: `wait_infer_ready()` for serve warmup
- `test_mcq_ladder_v1.py`: `--memory-safe`, isolated long-batch MCQ
- `valhalla_base.py`: HTTP broken-pipe retry; `run_mcq_full_ladder.sh`: F16 + GPU defaults

---

*Rogue Intelligence LNC.*
