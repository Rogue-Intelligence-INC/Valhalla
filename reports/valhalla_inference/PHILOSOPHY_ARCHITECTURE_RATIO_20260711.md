# Valhalla 哲学在架构中的占比

**Date:** 2026-07-11  
**Method:** `wc -l` 模块统计 + `decode_path` 实测 + 平台文档对照

---

## 1. 哲学定义（本报告口径）

| 轴 | 含义 |
|----|------|
| **结构优先** | Triad (Hub/Tile/StemCell/Fate) · session incubation · native structure decode |
| **Fate 代数** | `fate_adaptive_algebra` · affinity/hub_spread 门控 · proactive |
| **LM 头** | `patch_lm` / trad — 可替换 decode head，非产品本体 |
| **非哲学** | legacy train bins · 纯 benchmark harness · UI shell |

公理 **Φ²=Φ+1** 的完整 s-wind 探索在 `s-wind-sim/`；生产 hot path 用 Fate 代数子集。

---

## 2. 生产推理栈 LOC 占比（hub-f64 `src/`）

**总计 43 文件 · 26,971 LOC**

| 类别 | 模块数 | LOC | 占比 |
|------|--------|-----|------|
| **A 结构/Triad/Fate/ingress/native** | 24 | 17,787 | **66.0%** |
| **E ValhallaBase 编排** (session/routing) | 4 | 3,487 | 12.9% |
| **B LM decode/patch** | 5 | 3,024 | 11.2% |
| **C 库内 benchmark** | 6 | 2,393 | 8.9% |
| **D UI/viz** | 3 | 280 | 1.0% |

**哲学核心 (A+E) + Triad/Fate 子 crate：**

| 组件 | LOC |
|------|-----|
| hub-f64 A+E | 21,274 |
| tile-quattro + stem-cell + chamber | 4,525 |
| fate-f64 meta | 3,642 |
| **哲学核心合计** | **29,441** |

**完整推理栈 (A+E+B+C+D)：** 35,138 LOC → **哲学占 83.8%**

### 若计入 legacy bins（93 文件 · 33k LOC）

全 crate 59,969 LOC → 哲学 **49.1%**（legacy `stemcell_*`/`tile_*` train 稀释）

---

## 3. 推理路径：结构 vs LM 决策占比

### 代码路由 (`effective_decode`)

| 题型 | 生产默认 `hybrid` | 结构 native | LM patch |
|------|-------------------|-------------|----------|
| Open QA | ✓ | **100%** | 0% |
| Follow-up | `native_follow_up_aware` | **100%** | 0% |
| MCQ/numeric | `mcq_decode=lm_patch` | 0% | **100%** |
| **Triad incubation** | 所有请求 | **100%** | 0% |

**结论：** 哲学「结构优先」在 **ingest 层 100%**；open chat **decode 100% native**；MCQ 才走 LM 头。

### Benchmark JSON 聚合（1,065 条 `decode_path`）

| 路径类 | 占比 |
|--------|------|
| Structure-native | **57.8%** |
| LM patch | 27.1% |
| trad/baked baseline | 15.1% |

偏 MCQ arm + trad 对照；**不代表生产 open-chat 比例**。

---

## 4. 哲学 ↔ 架构对齐

| 哲学支柱 | 代码 | 强度 |
|----------|------|------|
| Triad 快照 | `signal_ingress.rs` (1558) + tile/stem | **强** |
| Fate 探索/会话 | `fate_orchestrator.rs` (2203) | **强** |
| Native decode | `native_qa.rs` (2854) vs `patch_lm.rs` (1456) | **强** — 2:1 |
| 可训练结构 L1/L2/L3 | fate_pretrain · model_export | **强** |
| Φ 代数穷尽 | s-wind-sim 独立 | 理论层，非 runtime 主路径 |

**文档一致：** `VALHALLA_BASE_PLATFORM.md` 双层模型（结构基座 + 可选 LM）= `lib.rs` 模块图 + `effective_decode()`。

---

## 5. Benchmark 层占比（外部）

| 路径 | LOC | 角色 |
|------|-----|------|
| `tools/valhalla_inference/*.py` | ~19,298 | 第三方协议 · 非产品本体 |
| `apps/valhalla_chat` | ~2,381 | 演示/UI |

Benchmark 体量大但 **orthogonal** — 验证哲学主张，不定义哲学。

---

## 6. 一句话

> **Valhalla 哲学（结构/Fate/Triad/native decode）占生产推理栈 ~84% LOC；open 路径 decode 100% 结构；LM 是 ~11% 的可替换头。**  
> 全 repo 若含 legacy train bins，占比降至 ~49% — 那是历史训练代码，不是 ValhallaBase 产品面。

---

*Rogue Intelligence LNC.*
