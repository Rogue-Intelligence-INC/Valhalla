# Manifold Fragment Runtime v1

**Protocol:** `valhalla-manifold-v1` · **Library:** `valhalla-fragment-library-v1`  
**Date:** 2026-07-06 · **Status:** Production path in `hub-f64`

---

## 概述

将 ValhallaBase 的 **monolithic snapshot**（整 corpus 一次 freeze + 每题 fork）升级为 **Fate 驱动的可组合片段流形**：

| 旧路径 | 新路径 |
|--------|--------|
| 固定 N 轮 incubation | Fate-owned adapt（收敛/边际稳定停止） |
| 每 corpus 行一个 fragment（可选） | **密度分段**：Fate 信号密集度下降时切段 |
| 每题 clone 全 snapshot | Live session + **query-conditioned merge** |
| 浅层 Fate restore | 完整 `preference_history` + `triple_fate` + `patch_vector` |

---

## 架构

```
corpus ingress → [density segments seg_*] → manifold snapshot
                      ↓
question → rank fragments (affinity) → merge top-k → QA pulse → decode
```

### 核心类型

- **`SnapshotFragment`** — Tile/Stem + patch + Fate convergence + line/tick 元数据
- **`FateDensitySegmenter`** — 观测 `Δsystem_counts/Δtick`、preference 输出率下降 → 关闭片段
- **`merge_fragments_for_query`** — 按问题 affinity 选 top-k 片段再 compose

### 源码

| 模块 | 路径 |
|------|------|
| Orchestrator | `manifestsys/hub-f64/src/fate_orchestrator.rs` |
| Snapshot 扩展 | `manifestsys/hub-f64/src/signal_ingress.rs` |
| Batch QA | `manifestsys/hub-f64/src/native_qa.rs` |
| MultiHop bench | `manifestsys/hub-f64/src/valhalla_base_bench.rs` |
| Smoke / pretrain | `bin/manifold_orchestrator_smoke`, `bin/manifold_pretrain` |

---

## 环境变量

| 变量 | 默认 | 说明 |
|------|------|------|
| `VALHALLA_FATE_ORCHESTRATE` | `1` | Fate-owned adapt |
| `VALHALLA_FRAGMENT_MODE` | `density` | `density` \| `line` \| `off` |
| `VALHALLA_FRAGMENT_DENSITY_DROP` | `0.35` | 边际密度下降触发切段 |
| `VALHALLA_FRAGMENT_MIN_LINES` | `2` | 最短段行数 |
| `VALHALLA_FRAGMENT_MAX_LINES` | `48` | 最长段行数 |
| `VALHALLA_LIVE_SESSION` | `1` | Batch QA 持久 session |
| **`VALHALLA_DYNAMIC_CORPUS_FRAGMENTS`** | **`1`（随 `FRAGMENT_MODE≠off`）** | **统一功能：全 corpus ingest + query merge + keyword anchor** |
| `VALHALLA_QUERY_FRAGMENT_TOP_K` | `3` | 每题 fragment top-k（>32 行时自动放大至 `ceil(n/20)`） |
| `VALHALLA_FRAGMENT_KEYWORD_ANCHOR_FRAC` | `0.35` | 高 keyword 重叠行强制进检索候选（needle 保底） |
| `VALHALLA_NATIVE_MAX_CORPUS_LINES` | `32` | **仅 legacy**（`DYNAMIC_CORPUS_FRAGMENTS=0` 时 tail ingest） |
| `VALHALLA_INGEST_ALL_CORPUS` | `0` | legacy 强制全 ingest（已被 dynamic 取代） |

回退 legacy：`VALHALLA_DYNAMIC_CORPUS_FRAGMENTS=0`（恢复 tail-32 ingest，merge 默认关）

---

## Benchmark（2026-07-06）

### 行业全套（manifold 默认）

| Benchmark | 结果 |
|-----------|------|
| RAGTruth / RAGTruth-100 | 40/40 · 100/100 |
| Needle / RULER | 4/4 · 20/20 |
| External holdout | 15/15 |
| MultiHop hybrid | **16/25 (64%)** |
| MCQ ladder | 0.5B 61% → 3B 98% |
| LoCoMo | 1/30 |
| FRAMES | 0/25 |

### MultiHop 路径升级

- **Before:** `ValhallaBaseClient.run()` → 外部 snapshot，无 query merge
- **After:** `run_native_qa_batch()` → 密度分段 + query merge，输出 `query_fragments`

Hybrid 得分暂 **16/25**（与 7/5 持平；7/4 peak 17/25）。瓶颈在 multihop decode，非 fragment 选择。

---

## 命令

```bash
# 构建
LIBRARY_PATH=/opt/cuda/targets/x86_64-linux/lib:$LIBRARY_PATH \
  cargo build -p hub-f64 --release --bin valhalla_base \
  --bin manifold_orchestrator_smoke --bin manifold_pretrain

# Smoke
./target/release/manifold_orchestrator_smoke

# 离线预训练 → fragment library
./target/release/manifold_pretrain --out reports/manifold_pretrain_library.json

# MultiHop hybrid（native batch + query merge）
python3 tools/valhalla_inference/test_multihop_rag_v1.py --hybrid --limit 25

# 行业全套
python3 tools/valhalla_inference/run_industry_benchmarks.py --all
```

---

## 备份

Pre-change snapshot：`backups/fate_orchestrator_20260705/`

---

*Rogue Intelligence LNC.*
