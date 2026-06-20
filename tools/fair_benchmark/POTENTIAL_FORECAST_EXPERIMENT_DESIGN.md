# Valhalla 潜力预测实验设计

**实验 ID**: `potential_forecast_v1`  
**类型**: 设计文档（未实现）· **不上传**  
**日期**: 2026-06-20  
**前提**: Transformer（Qwen2.5-0.5B）是**成熟产品**；Valhalla 是**开发中**的孵化 runtime

---

## 0. 为什么要单独设计「潜力预测」实验

### 0.1 现状对比的误导性

| 对比方式 | 问题 |
|----------|------|
| 全量 acc：Valhalla 40.7% vs TF 71% | 把**终局产品**和**半成品 Tier B** 放在同一标尺 |
| 单次 snapshot | 看不到**改进斜率**；开发中系统应看导数 |
| open_generation 0% vs TF 94% | TF 的强项；Valhalla 尚未实现 Tier B 开放生成解码 |
| 多轮 cycle 无增益 | 说明瓶颈在**解码/语料规模**，不在「再跑 20 轮」 |

### 0.2 预测应回答的三个产品问题

1. **记忆平台**：Valhalla 能否成为「语料 ingress → 可审计检索 → 垂直 QA」的**独立产品**？（不必替代 TF）
2. **Hybrid 天花板**：memory（楔子）+ patch Tier A（open）组合，**理论上界**距 TF 多远？
3. **全量替代时间表**：若持续按当前楔子斜率开发，**何时**可能在 fair test 上追平 TF 71%？（或证明不可达）

### 0.3 与已有实验的关系

| 已有 | 本设计扩展 |
|------|------------|
| `potential_analysis_v1`（MPI/TRI snapshot） | 加入**轨迹、规模曲线、混合天花板、交叉点预测** |
| `fair_benchmark` test 145Q | **冻结**为 longitudinal 固定考卷 |
| Tile/Stem multi-cycle | 证明 cycle 非杠杆 → 预测模型**排除** cycle 维度 |
| Tier A patch 69% | 作为 hybrid **上界臂**，非「当前主产品」 |

---

## 1. 实验总览

```
                    ┌─────────────────────────────────────┐
  冻结考卷 145Q      │ fair test split（永不改题）          │
  冻结评分器         │ numeric / mcq / open + slices       │
                    └──────────────┬──────────────────────┘
                                   │
         ┌─────────────────────────┼─────────────────────────┐
         ▼                         ▼                         ▼
   轴 A 语料规模曲线          轴 B 开发里程碑序列        轴 C Hybrid 天花板
   (platform ceiling)        (velocity)                 (product ceiling)
         │                         │                         │
         └─────────────────────────┼─────────────────────────┘
                                   ▼
                    ┌─────────────────────────────────────┐
                    轴 D 成熟度校正 TRI    轴 E TF 交叉点预测
                    (wedge-only 替代)      (time-to-parity)
                                   │
                                   ▼
                    PFI 潜力预测指数 + 三场景报告
                    (pessimistic / base / optimistic)
```

**核心原则**：

- **TF 臂固定**：同一 checkpoint、同一 prompt、不再微调 — 代表成熟产品**静态基线**。
- **Valhalla 臂演进**：每 milestone 记录结构版本 + 解码版本 + artifact 版本。
- **预测看斜率与楔子**，不看单次全量 acc。

---

## 2. 冻结资产（所有轴共用）

| 资产 | 路径 / 规格 | 不可变 |
|------|-------------|--------|
| 考卷 | fair benchmark `test` 145Q | ✓ |
| 评分 | `tools/fair_benchmark/metrics.py` | ✓ |
| 切片 | corpus_supported / mcq / open_retrieval / open_generation / numeric | ✓ |
| TF 基线 | Qwen2.5-0.5B-Instruct zero-shot | ✓（除非换模型档位列） |
| 主 artifact | `fair_triad_c1`（可扩展 tier 语料） | 版本化 |

---

## 3. 五条测量轴

### 轴 A — 语料规模响应曲线（Memory Platform Ceiling）

**问题**：记忆平台潜力是否随 **ingress 规模** 超线性/对数增长？

**设计**：

| Tier | 行数 | 构造方式 |
|------|------|----------|
| S | 22 | ingest_clean 子集 |
| M | 49 | 当前 fair ingest_clean |
| L | 110 | M + 同域扩展（人工审核，无答案泄漏） |
| XL | 220 | L ×2 或垂直域打包 |

**每 tier 跑臂**（仅 memory track，控制变量）：

- `baseline_no_corpus`
- `triad_c1_isolated`
- `triad_c1_persistent`

**记录指标**：

- `acc(corpus_supported)` · `recall@3` · `ingest_lift_pp` · `MPI(tier)`

**拟合模型**（预测上界）：

```
acc_cs(L) = a + b · log(L)          # 对数饱和（常见记忆系统）
recall@3(L) = r_max · (1 - e^{-kL}) # 指数饱和
```

**外推**：

- 报告 `acc_cs @ L=500, 2000, 10000` 三档 **预测值 + 95% CI**
- 若 `b ≤ 0` 或 `recall@3` 在 M→L 增益 < 2pp → **platform ceiling 低**，预测文档中标记 `memory_saturates_early`

**通过信号（platform 可行）**：

- M→L：`ingest_lift_pp` 仍 ≥ +5pp
- `recall@3(L)` 斜率 > 0 且未饱和

---

### 轴 B — 开发里程碑时间序列（Improvement Velocity）

**问题**：Valhalla 作为开发中系统，**每月/每 milestone** 在冻结考卷上改进多快？

**设计**：

- 选定 **5–8 个 milestone**（已有 + 未来）：
  - M0: `20260620_0859` fair benchmark（已有）
  - M1: persistent retrieval 修复后
  - M2: native open 解码 v0.1
  - M3: artifact 增量 ingress
  - …
- 每 milestone **只改 Valhalla**，TF 不重跑（或季度校准一次）

**每 milestone 记录**：

| 字段 | 说明 |
|------|------|
| `git_tag` / `manifest_version` | 可复现 |
| `acc_total` · 各 slice acc | 145Q |
| `Δpp vs M0` · `Δpp vs M_{n-1}` | 进步速度 |
| `decode_paths` 分布 | 哪条解码路径在涨 |

**速度指标**：

```
velocity_slice = median( Δpp_slice / months_since_M0 )
```

**预测（12 个月）**：

```
acc_forecast_12m = acc_M0 + 12 × velocity_corpus_supported   # base case
CI: bootstrap over milestone pairs
```

**Interpretation**：

- `velocity > +1.5 pp/月` on corpus_supported → **high_development_momentum**
- `velocity ≈ 0` 连续 2 milestone → **plateau_risk**（需换杠杆：解码而非 cycle）

---

### 轴 C — Hybrid 产品天花板（非 TF 全替路径）

**问题**：若产品定位为 **Hybrid**（memory 楔子 + patch 生成），**理论上界**是多少？

**设计**：

对每题 `q`，定义 **oracle hybrid 臂**（同一 milestone 内可测，非作弊）：

| 题型 / 切片 | 选用臂 |
|-------------|--------|
| corpus_supported + mcq | `triad_c1_persistent` |
| corpus_supported + numeric | `max(persistent, patch_triad_legacy)` |
| open_retrieval | `patch_triad_legacy` |
| open_generation | `patch_triad_legacy` 或 TF（产品配置位） |

```
hybrid_oracle_acc = mean( correct_q( best_allowed_arm(q) ) )
gap_to_tf = TF_acc - hybrid_oracle_acc
```

**当前 snapshot 估计**（来自 0859 数据，可直接算）：

| 臂 | acc |
|----|-----|
| persistent memory | 40.7% |
| patch triad legacy | 68.3% |
| TF | 71.0% |
| **hybrid oracle（需逐题 max）** | **~72–75%（预测区间，待跑）** |

**预测含义**：

- 若 hybrid oracle **已 ≥ TF** → 产品路径是 **编排层** 而非单轨 Valhalla
- 若 hybrid oracle **< TF 但 gap ≤ 5pp** → **12 个月内可产品化**（成熟 TF + Valhalla 记忆层）

**必跑实验**（实现时）：

```bash
python tools/fair_benchmark/run_hybrid_ceiling.py \
  --reuse-json reports/fair_benchmark/fair_benchmark_test_20260620_0859.json \
  --policy wedge_memory_plus_patch_open
```

---

### 轴 D — 成熟度校正替代指数（Maturity-Adjusted TRI）

**问题**：TRI=0.27 是否**低估**了开发中系统的「可替代楔子」潜力？

**设计**：拆分 TRI 为三层，**仅对已实现能力计分**：

| 层级 | 切片 | 权重 | 成熟 TF 要求 | Valhalla 现状 |
|------|------|------|--------------|---------------|
| L1 已交付 | mcq_routing | 0.30 | 路由 | **+8.7 pp 胜 TF** |
| L2 已交付 | open_retrieval + patch | 0.35 | 检索 | patch≈TF |
| L3 开发中 | corpus_supported memory | 0.25 | 记忆 QA | -14.6 pp |
| L4 未实现 | open_generation native | 0.10 | 生成 | 0% vs 94% |

**成熟度校正**：

```
TRI_delivered = Σ w_i × 1[win]   for L1+L2 only
TRI_full      = TRI_delivered + Σ w_j × potential_j
```

其中 `potential_j` 来自轴 B 的 velocity 外推，**不是**假设全赢。

**预测输出**：

| 指标 | 含义 |
|------|------|
| `TRI_delivered` | 今天已能卖的楔子 |
| `TRI_12m_base` | 12 个月后 L3 追平假设下的 TRI |
| `TRI_ceiling` | L1+L2+L3 全赢、L4 仍用 patch/TF 的 hybrid TRI |

**当前粗估**（基于 0906 报告）：

- L1 胜 → 贡献 ~0.30
- L2 patch 平 → 贡献 ~0.35
- L3 负 → 贡献 0
- **TRI_delivered ≈ 0.65**（vs snapshot TRI 0.27）

→ 投资叙事：**不是「27% 替代 TF」**，而是 **「65% 楔子已可交付 + 记忆层仍在追」**。

---

### 轴 E — TF 交叉点预测（Time-to-Parity）

**问题**：按开发斜率，fair 145Q 上 **何时** hybrid ≥ TF 71%？

**模型**（三场景）：

| 场景 | 假设 | hybrid 年化增益 |
|------|------|-----------------|
| 悲观 | 轴 A 饱和 + 轴 B velocity→0 | +2 pp/年 |
| 基准 | velocity 维持 M0→M1 估计 | +8 pp/年 |
| 乐观 | L3 corpus_supported 追平 + native open v1 | +18 pp/年 |

**交叉点**：

```
months_to_parity = max(0, (TF_acc - hybrid_now) / (monthly_gain)) 
```

**若 hybrid oracle 已 > TF**：`months_to_parity = 0`（产品焦点转 **成本/审计/数据主权** 而非 acc）

**若基准场景 > 36 个月**：结论为 **不追求全量 parity**；定位 memory platform + TF 编排。

---

## 4. 综合指数：PFI（Potential Forecast Index）

```
PFI = 0.40 × MPI_ceiling_norm          # 轴 A 外推，归一化到 [0,1]
    + 0.25 × velocity_norm              # 轴 B，归一化（cap 3 pp/月）
    + 0.20 × hybrid_gap_closure         # 轴 C，1 - gap/hybrid_ceiling_gap_max
    + 0.15 × TRI_delivered              # 轴 D
```

| PFI | 解释 |
|-----|------|
| ≥ 0.70 | **strong_platform_forecast** — 记忆平台 + hybrid 可产品化 |
| 0.50–0.70 | **viable_niche** — 垂直楔子 + TF 共生 |
| < 0.50 | **research_stage** — 需 Tier B 突破后再预测 |

**与 snapshot MPI/TRI 区别**：

- MPI/TRI = **现在拍照**
- PFI = **轨迹 + 天花板 + 成熟度校正**

---

## 5. 实验执行计划（分三期）

### Phase 0 — 离线复算（1 天，无需新跑 native）

| 任务 | 输入 | 输出 |
|------|------|------|
| hybrid oracle | `fair_benchmark_test_20260620_0859.json` | `hybrid_ceiling_acc` |
| TRI_delivered | 同上 + slices | 校正 TRI |
| PFI snapshot | MPI 0.76 + 上两项 | PFI v0 |

### Phase 1 — 语料规模轴（3–5 天）

1. 构造 L(110) / XL(220) 去污染语料（同域扩展）
2. 每 tier 跑 persistent + isolated
3. 拟合 `acc_cs(L)` · 写 ceiling 报告

### Phase 2 — 里程碑序列（持续 6–12 月）

- 每 feature merge 打 tag，重跑 fair 145Q Valhalla 臂
- TF 每季度校准一次
- 自动更新 velocity · 滚动 PFI

---

## 6. 公平性约束（相对成熟 TF）

| 约束 | 原因 |
|------|------|
| TF 不用 fair 语料微调 | 模拟**客户已有模型**，非专门为考卷训练 |
| Valhalla 可用 fair 语料 ingress | 模拟**产品用法**（客户灌入自己的 KB） |
| 分开报 **wedge acc** 与 **full acc** | 避免 open_generation 拉低叙事 |
| 预测必须带 **三场景 CI** | 开发中系统不确定性大 |
| 禁止「假设 cycle→∞」外推 | 1032 实验已证伪 |

---

## 7. 预期结论模板（基于现有数据的方向性判断）

> **非最终跑数结果**，供 Phase 0 验证方向。

| 维度 | 方向性判断 |
|------|------------|
| Memory platform | MPI=0.76，ingest +31pp → **平台信号强**；需轴 A 验证规模 ceiling |
| TF 全替 | open_generation 鸿沟 → **12–24 个月内不全替** 是合理基线 |
| Hybrid 产品 | patch 69% + memory MCQ 楔子 → **hybrid oracle 可能已 ≈ TF** |
| 最佳产品形态 | **Valhalla 记忆/审计层 + TF 生成**；非二选一 |
| 开发优先级 | ① persistent/session ② corpus 规模 ③ native open ④ ~~加 cycle~~ |

---

## 8. 实现清单（将来可做，本次不上传）

```
tools/fair_benchmark/
  POTENTIAL_FORECAST_EXPERIMENT_DESIGN.md   ← 本文
  run_hybrid_ceiling.py                     ← Phase 0
  run_corpus_scaling_forecast.py            ← Phase 1
  run_milestone_tracker.py                  ← Phase 2
  run_potential_forecast.py                 ← 汇总 PFI
```

**Phase 0 最小命令（设计）**：

```bash
# 1. Hybrid 天花板（仅读已有 JSON）
python tools/fair_benchmark/run_hybrid_ceiling.py \
  --reuse-json reports/fair_benchmark/fair_benchmark_test_20260620_0859.json

# 2. 成熟度校正 TRI + PFI v0
python tools/fair_benchmark/run_potential_forecast.py \
  --reuse-json reports/fair_benchmark/fair_benchmark_test_20260620_0859.json \
  --reuse-potential reports/potential_analysis/potential_analysis_20260620_0906.json
```

---

## 9. 报告输出结构

```markdown
# Valhalla 潜力预测报告 (potential_forecast_v1)

## Executive Summary
- PFI: 0.xx (scenario)
- 12m hybrid acc forecast: xx% [lo, hi]
- months_to_parity: xx | never (pessimistic)
- 产品建议: platform | hybrid | research

## 1. 成熟度 framing
## 2. 轴 A 语料 ceiling 曲线
## 3. 轴 B 开发 velocity
## 4. 轴 C hybrid oracle vs TF
## 5. 轴 D TRI_delivered vs TRI_snapshot
## 6. 三场景预测表
## 7. 风险与反证（何种结果推翻预测）
```

---

## 10. 何种结果会推翻本预测

| 观测 | 推翻 |
|------|------|
| 轴 A：L→XL ingest_lift < 2pp | memory platform ceiling 低 |
| 轴 B：连续 3 milestone Δ≈0 | velocity 外推无效 |
| hybrid oracle < TF - 10pp 且 12m 无改善 | hybrid 产品叙事不成立 |
| native open v1 仍 0% on open_generation | L4 长期依赖 TF |

---

*Rogue Intelligence LNC. · 设计稿 · 不上传 · 2026-06-20*
