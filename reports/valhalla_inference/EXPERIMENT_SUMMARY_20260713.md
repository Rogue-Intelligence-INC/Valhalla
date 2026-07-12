# Valhalla Experiment Summary — Unified I/O · Bench/Interface Split · Full Industry Run

**Date:** 2026-07-13  
**Commit:** `04a49ddf` · **Status:** Integrated · cite with Tier labels  
**Integrated report:** [`VALHALLA_INTEGRATED_STATUS.md`](./VALHALLA_INTEGRATED_STATUS.md)

---

## 1. 实验目标

在 **RelAddr + Fate adapt + multihop v2 compose（fatewithv2）** 统一栈上：

1. 将 **benchmark 编排** 与 **可编程 open-QA 接口** 分离，避免 bench 逻辑污染生产 API。
2. 修复 **interface → Fate 喂料**（小 corpus 全量 evidence，非 fragment top-1 截断）。
3. 以 **全量 Industry suite**（无 `--fast`）刷新第三方可比分数。

**公理栈不变：** `Φ²=Φ+1` · ingress 代数信号 · Fate 探索 · native decode · LM 仅 decode head。

---

## 2. 架构变动（新版）

```
┌─────────────────────────────────────────────────────────────┐
│  Bench 层    multihop_bench.rs · valhalla_base_bench.rs       │
│              bundle I/O · 并行跑分 · JSON 报告                 │
└───────────────────────────┬─────────────────────────────────┘
                            │ OpenQaRequest / run_open_qa()
┌───────────────────────────▼─────────────────────────────────┐
│  接口层      open_qa_interface.rs                             │
│              OpenQaFlags · run_open_qa · run_open_qa_scored   │
└───────────────────────────┬─────────────────────────────────┘
                            │ fate_ingress / fate_egress
┌───────────────────────────▼─────────────────────────────────┐
│  I/O 层      fate_io.rs                                     │
│              fate_ingress · fate_egress · fate_score_short  │
└───────────────────────────┬─────────────────────────────────┘
                            │ native_qa_batch
┌───────────────────────────▼─────────────────────────────────┐
│  运行时      native_qa · multihop_fate · natural_channels   │
│              elastic_resources (CUDA · parallel · budget)     │
└─────────────────────────────────────────────────────────────┘
```

| 模块 | 路径 | 职责 |
|------|------|------|
| **fate_io** | `manifestsys/hub-f64/src/fate_io.rs` | RelAddr ingress · egress cite · 统一 solution flags |
| **open_qa_interface** | `open_qa_interface.rs` | 单次 native open-QA 调用 API |
| **multihop_bench** | `multihop_bench.rs` | MultiHop / LaRA benchmark _runner |
| **elastic_resources** | `elastic_resources.rs` | `ElasticResourcePlan` · `par_map_items` · CUDA env |

**CLI 入口（bench 仍通过 demo 调用接口）：**

```bash
valhalla_base demo multihop          # v2 only  → multihop_rag_v2_test.json
valhalla_base demo multihop-fate     # Fate     → multihop_rag_fate_test.json
valhalla_base demo multihop-hybrid   # fatewithv2 → multihop_rag_hybrid_test.json
```

**Rust 程序 API：**

```rust
use hub_f64::{OpenQaRequest, OpenQaFlags, run_open_qa_scored, fate_score_short_answer};

let req = OpenQaRequest::new("q1", question, corpus)
    .with_flags(OpenQaFlags::unified());
let out = run_open_qa_scored(&req, gold);
```

---

## 3. 关键代码修复（与 peak 路径相关）

| 问题 | 修复 | 文件 |
|------|------|------|
| 小 corpus 只喂 `seg_1` fragment | `fate_full_corpus_feed()` 全 evidence | `native_qa.rs` |
| Fate 短答被 span 通道压过 | natural_channels grounding bonus | `natural_channels.rs` |
| multihop RelAddr 未进 hop adapt | RelAddr + entity span 对齐 | `multihop_fate.rs` |
| env ingress budget 被忽略 | `IngressBudget::incubation_with_env()` | `elastic_resources.rs` |
| 评分 Sam Altman 假阳性 | `fate_score_short_answer` 诚实 token 匹配 | `fate_io.rs` |

**Peak decode path（典型正确项）：**  
`multihop_fate_yesno+multihop_v2_yesno` · `multihop_fate_entity+multihop_v2_entity`

---

## 4. 全量 Industry 结果（2026-07-13 · 无 `--fast`）

**Runner:** `run_industry_benchmarks.py --all --fresh`  
**时长:** ~26 min · **LM 校准:** Qwen2.5-3B-Instruct · **Ingress:** 50% / max_items 48  
**Elastic:** `parallel_workers=2` · `cuda_ready=true`

| Benchmark | n | Result | Verdict |
|-----------|---|--------|---------|
| Needle session | 4 depths | **4/4 (100%)** | NEEDLE_STRONG |
| RAGTruth | 40 | **34/40 (85%)** | RAGTRUTH_STRONG |
| RAGTruth-100 | 100 | **88/100 (88%)** | RAGTRUTH_STRONG |
| LongBench-v2 | 20 | **4/20 (20%)** | LONGBENCH_V2_DEVELOPING |
| MultiHop v2 | 25 | **15/25 (60%)** | OPEN_QA_VIABLE |
| MultiHop Fate | 25 | **15/25 (60%)** | OPEN_QA_VIABLE |
| MultiHop fatewithv2 | 25 | **15/25 (60%)** | OPEN_QA_VIABLE |
| LoCoMo | 30 | **17/30 (57%)** | OPEN_QA_VIABLE |
| RULER-style | 20 | **15/20 (75%)** | NEEDLE_VIABLE |
| LaRA RAG vs LC | 20 | **RAG 11/20 · LC 13/20** | STRUCTURE_RAG_BELOW_LC |
| External holdout | 15 | **10/15 (67%)** | EXTERNAL_HOLDOUT_VIABLE |
| MCQ ladder @3B | 46 | **45/46 (98%)** | MCQ_LADDER_FLAT |
| FRAMES | — | **跳过** | Tier C（无 wiki extract） |

**Legitimacy:** citeable **10/11** · FRAMES bundle viable: **False**

---

## 5. 与历史 run 对照

| 项 | 2026-07-08 @3B | 2026-07-12 fast | **2026-07-13 全量** |
|----|----------------|-----------------|---------------------|
| Needle | 4/4 | 2/2 | **4/4** |
| RAGTruth-100 | 99/100 | 8/10 | **88/100** |
| LoCoMo | 14/30 | 1/3 | **17/30** |
| MultiHop hybrid | 14/25 | 17/25* | **15/25** |
| LongBench | 7/20 | 1/5 | **4/20** |

\* fast run 与并行/评分口径不同；**15/25** 为三臂一致、诚实 `fate_score_short_answer` 下的稳定读数。

**历史 peak（VUC 归档 hybrid）：** 16/25 (64%) — fate+fate+v2 混合路径，非纯 v2。

---

## 6. 复现

```bash
export RUSTFLAGS="-L /opt/cuda/lib64" LIBRARY_PATH=/opt/cuda/lib64
cargo build -p hub-f64 --release --features patch-lm --bin valhalla_base

# 全量 industry（含 MCQ ladder @3B）
python3 tools/valhalla_inference/run_industry_benchmarks.py --all --fresh --verbose

# MultiHop 三臂（Rust demo · n=25）
valhalla_base demo multihop-hybrid --repo . --limit 25
valhalla_base demo multihop --repo . --limit 25
valhalla_base demo multihop-fate --repo . --limit 25

# 整合报告
python3 tools/valhalla_inference/integrate_valhalla_reports.py
python3 tools/valhalla_inference/sync_valhalla_unified_core.py
```

---

## 7. 诚实边界（Tier B/C）

- **LongBench / MultiHop entity**：部分 MH 项 decode 正确文本但 gold 不匹配（如 MH_2275/2328）— 评分已收紧。
- **FRAMES**：需 Wikipedia extract prefetch；离线环境不可 cite。
- **Pareto matrix** 中 LoCoMo/LongBench 对照行仍为 fast slice — 与全量 industry 分开引用。

---

*Rogue Intelligence LNC.*
