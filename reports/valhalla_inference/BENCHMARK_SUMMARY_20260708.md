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
| **B** | LoCoMo | **14/30 (47%)** | OPEN_QA_VIABLE |
| **B** | FRAMES | **0/25 (0%)** | OPEN_QA_DEVELOPING |
| **B** | MCQ ladder | **0.5B 63% · 3B 96% · 7B 98%** (F16 GPU) | MCQ_LADDER_SCALES |

**Docs:** [`INDUSTRY_3B_RUN_STATUS.md`](./INDUSTRY_3B_RUN_STATUS.md) · [`industry_benchmarks_summary.json`](./industry_benchmarks_summary.json)

---

## 2. Proactive agency — ✅ AGENCY_STRONG（2026-07-09）

| 指标 | 结果 |
|------|------|
| **Agency Index** | **0.778** (AGENCY_STRONG) — full 版 |
| **PPI-v2 自动反应** | **0.650** · 68/72 pass |
| **门控 MAR / FIR** | **100% / 0%** |
| Long-dialogue | n=**10** · pass **60%** · proactive gate **100%** |
| Multimodal | n=**10** · pass **100%** · consistency **1.0** |
| Crosslang CCU | **100%** |
| Mega 3000轮 | 140s infra（**不计 Agency**）|

**四轨公式:** `0.25·long + 0.20·multimodal + 0.20·crosslang + 0.35·PPI-v2`

```bash
tools/valhalla_inference/run_proactive_agency_full.sh --verbose
tools/valhalla_inference/run_proactive_agency_specialized.sh --verbose
```

**审计:** [`PROACTIVE_AGENCY_AUDIT_20260708.md`](./PROACTIVE_AGENCY_AUDIT_20260708.md)  
**补齐报告:** [`PROACTIVE_AGENCY_FULL_20260708.md`](./PROACTIVE_AGENCY_FULL_20260708.md)  
**报告:** [`PROACTIVE_AGENCY_BENCHMARK.md`](./PROACTIVE_AGENCY_BENCHMARK.md)

---

## 3. Backbone orthogonality — ✅（2026-07-10）

| Layer | Panel | Result |
|-------|-------|--------|
| `open_path_orthogonality` | RWKV-7 · Phi-MoE · Qwen2.5-3B | native/hybrid **4/4** match → **VALHALLA_BASE_INDEPENDENT** |
| `session_memory_orthogonality` | needle d=5,10 | hybrid = native ref → **MEMORY_INDEPENDENT** |
| `stack_capability_smokes` | fusion · proactive gates · longchain | all OK（fixed Qwen stack） |

```bash
.venv-llm/bin/python3 tools/valhalla_inference/run_valhalla_independence.py --fast --with-trad --with-memory
.venv-llm/bin/python3 tools/valhalla_inference/valhalla_backbone.py --list
```

**Doc:** [`VALHALLA_INDEPENDENCE_EXPERIMENT_20260710.md`](./VALHALLA_INDEPENDENCE_EXPERIMENT_20260710.md)

---

## 4. MCQ ladder — memory-safe GPU F16 (补跑)

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
- `run_proactive_agency_benchmark.py`: `--full` / `--specialized-reaction` · 10 multimodal · LoCoMo 扩展修复

---

## 5. Fate 长链套件回归（2026-07-09）

顺序 runner：`python3 tools/valhalla_inference/run_suite_sequential.py --all`

| 模块 | 结果 |
|------|------|
| fate-pretrain-holdout | train/holdout **100%** |
| fate-longchain-unified | theme_drift **0%** · holdout **100%** · composite **0.950** |
| fate-routing-ab | parallel/targeted **67%** (+33pp) low-resource |
| fate-parallel-longchain | parallel **100%** (+50pp) |
| standard-fast (8项) | 8/8 PASS |
| context-topic-drift | extended on-topic **96.7%** |

**修复**: `score_locomo_answer`（evidence-aware）、LoCoMo `--fast` minimal corpus、holdout charity-race isolated 检索。

**汇总**: [`suite_summaries/INDEX_FULL_20260709.md`](./suite_summaries/INDEX_FULL_20260709.md) · [`PRODUCTIZATION_ASSESSMENT_20260709.md`](./PRODUCTIZATION_ASSESSMENT_20260709.md)

---

## 6. 产品化 / 融资就绪（2026-07-09）

| 维度 | 状态 |
|------|------|
| 第三方复现 | ✅ `REPRODUCTION.md` · `make repro` · Docker · `/v1/repro` |
| 全套件 benchmark | ✅ fate + rust-unit + industry + proactive（见 INDEX_FULL） |
| 模型分发 | 🟡 0.5B Git LFS · 3B metadata + 本地 bake |
| 开放 QA 长链 | 🟡 LoCoMo 47% ↑ · FRAMES 仍 developing |

---

*Rogue Intelligence LNC.*
