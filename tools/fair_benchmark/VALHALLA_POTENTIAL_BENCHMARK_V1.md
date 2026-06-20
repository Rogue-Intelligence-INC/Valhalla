# Valhalla Potential Benchmark (VPB) v1.0

**实验 ID**: `valhalla_potential_benchmark_v1`  
**状态**: 设计规范（可基于现有 `fair_benchmark` 增量实现）  
**日期**: 2026-06-20  
**定位**: 评估 **开发中** Valhalla 相对 **成熟** Transformer 的**潜力**，而非单次 acc 胜负

---

## 1. 设计动机：向业界学什么

| 来源 | 核心思想 | VPB 采纳 |
|------|----------|----------|
| [HELM](https://arxiv.org/abs/2211.09110) (Stanford CRFM) | **分数矩阵** scenario×metric，拒绝单一 leaderboard 数字 | §4 六场景 × 五指标矩阵 |
| [LoCoMo](https://snap-research.github.io/locomo/) (Snap ACL 2024) | 长对话记忆：QA 分 **single/multi-hop/temporal/adversarial**；报告 **retrieval 命中率** | §5 记忆任务 taxonomy |
| [LONGMEMEVAL](https://github.com/xiaowu0162/LongMemEval) | **Recall@k** + evidence 标注；语料 **S/M 两档规模**；五能力（提取/多会话/时序/更新/拒答） | §6 语料规模轴 + 中间检索指标 |
| [RAGAS](https://arxiv.org/abs/2309.15217) | 检索与生成分离：**context precision/recall, faithfulness, answer relevancy** | §4.2 记忆轨四指标 |
| [METR](https://metr.org/) Time Horizon | 用 **任务尺度**（人类完成时长）追踪能力指数增长，而非静态题集 | §6 语料 token/行数作为 **memory horizon** |
| [ForecastBench](https://arxiv.org/pdf/2409.19839) | **动态/时间索引** benchmark，防泄漏、测预测能力 | §8 里程碑序列（frozen test + 版本轴） |
| [Mem0 / BEAM](https://mem0.ai/blog/what-is-beam-memory-benchmark) | 大规模下测 **token 效率**（检索 token vs 全上下文） | §4.2 效率指标 |
| AI Product 成熟度框架 | **Leading indicators**（eval pass rate、velocity）预测 4–8 周后 outcome | §7 开发速度轴 |

**VPB 不做的事**（向 HELM 学习「显式缺失」）：

- 不用一个 **Potential Score** 替代矩阵（只提供加权 **PFI** 作为摘要，矩阵为主）
- 不把 open_generation 与 TF 生成 **同一标尺硬比**（分轨报告）
- 不用泄漏语料「刷分」（继承 fair-1.1 去污染）

---

## 2. 比较哲学：成熟 vs 开发中

```
┌─────────────────────────────────────────────────────────────┐
│  Transformer（成熟对照臂）                                    │
│  · 固定 checkpoint · 零样本 · 无 fair 语料微调               │
│  · 报告：acc, latency, tokens/query（静态基线）               │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  Valhalla（被测系统）                                         │
│  · 允许 fair ingress（= 客户灌 KB 的真实用法）                 │
│  · 报告：acc + 检索 + 结构 + **Δmilestone + scale curve**    │
│  · 潜力 = 楔子能力 + 改进速度 + hybrid 天花板，非全量 acc      │
└─────────────────────────────────────────────────────────────┘
```

**合理比较单元**（ borrowed from HELM + LoCoMo ）：

1. **场景**（scenario）：记忆平台 / 路由 / 数值 / 开放检索 / 开放生成 / hybrid
2. **指标**（metric）：acc、recall@k、ingest lift、faithfulness、token cost
3. **尺度**（scale）：语料 S/M/L（LONGMEMEVAL 式）

---

## 3. 基准套件构成

### 3.1 三层套件（增量构建）

| 层 | 名称 | 题量 | 来源 | 用途 |
|----|------|------|------|------|
| L0 | `fair_test` | 145 | 现有 fair benchmark test | **冻结主考卷**，纵向对比 |
| L1 | `vpb_memory` | +80 | 仿 LoCoMo/LONGMEMEVAL 构造 | 记忆 taxonomy 覆盖 |
| L2 | `vpb_scale` | L0 子集 ×3 语料档 | 同题不同 corpus tier | 规模响应曲线 |

**总计**：~225 题（L0+L1），语料 3 档（S/M/L）。

### 3.2 L1 记忆题 taxonomy（对齐 LoCoMo + LONGMEMEVAL）

每题标注 `memory_task`（必填）：

| `memory_task` | 定义 | 题数目标 | 测什么 |
|---------------|------|----------|--------|
| `single_hop` | 答案在语料 **一行** | 20 | 基础 ingress 检索 |
| `multi_hop` | 需 **2+ 行** 合成 | 15 | Tile/Stem 结构联结 |
| `temporal` | 依赖顺序/时间词 | 10 | Fate 时序偏好 |
| `knowledge_update` | 语料后部覆盖前部 | 10 | 增量 ingress |
| `abstention` | 语料 **无** 答案，应拒答 | 15 | 抗幻觉（LONGMEMEVAL abstention） |
| `adversarial` | 相似干扰项在语料 | 10 | 检索 precision |

另保留 fair 原有：`score_type` ∈ numeric | mcq | open。

**证据标注**（LONGMEMEVAL 式，L1 必做）：

```json
{
  "id": "VPB_MH_01",
  "memory_task": "multi_hop",
  "evidence_line_ids": ["corpus_12", "corpus_31"],
  "abstention_expected": false
}
```

---

## 4. 评分矩阵（HELM 式，VPB 核心输出）

### 4.1 六场景（Scenarios）

| # | Scenario ID | 含义 | 主要 arms |
|---|-------------|------|-----------|
| S1 | `memory_platform` | 语料支撑 QA + 持久 session | persistent, isolated |
| S2 | `memory_retrieval` | 检索质量（不论最终 acc） | persistent + evidence |
| S3 | `routing_mcq` | 选项路由楔子 | persistent, patch |
| S4 | `numeric_compute` | 规则/皮层数值 | native, patch |
| S5 | `open_retrieval` | 开放题·语料可检索 | memory, patch |
| S6 | `hybrid_product` | 产品编排 max(memory, patch) | oracle hybrid |

**TF 对照臂**仅进入 S3–S5 的「生成基线」列，**不**进入 S1/S2（TF 无 Valhalla ingress 等价物 unless RAG baseline）。

### 4.2 五指标（Metrics）

| Metric ID | 定义 | 借鉴 | 适用场景 |
|-----------|------|------|----------|
| **M1 TaskAcc** | 任务正确率 | HELM accuracy | S1,S3–S6 |
| **M2 Recall@k** | evidence 行是否出现在 top-k | LoCoMo, LONGMEMEVAL | S1,S2 |
| **M3 IngestLift** | acc(with corpus) − acc(no corpus) | Valhalla fair | S1 |
| **M4 Faithfulness** | 答案 claim 可追溯到 retrieved 结构 | RAGAS | S1,S2,S5 |
| **M5 TokenCost** | 检索/解码消耗 token 或 f64 ops | BEAM, Mem0 | 全部 |

**RAGAS 适配 Valhalla Tier B**（无 LLM 生成时）：

| RAGAS 原指标 | VPB 操作化 |
|--------------|------------|
| Context Recall | `Recall@k`：evidence_line_ids ⊆ top_matches |
| Context Precision | top-k 中 relevant 行占比 |
| Faithfulness | 答案 token/数值是否来自 top_matches 或 decode_path 可追溯 |
| Answer Relevancy | 答案是否 addressing question（mcq 选项命中 / numeric 非空 / open keyword） |

### 4.3 输出格式：Potential Matrix

```
                 M1      M2      M3      M4      M5
S1 memory_plat   0.41    0.75    +31pp   0.82    4.2k
S2 retrieval     —       0.75    —       0.79    3.1k
S3 mcq           0.46    0.68    +18pp   0.71    2.8k
S4 numeric       0.58    —       +0pp    0.95    1.2k
S5 open_retr     0.52    0.61    +22pp   0.55    5.0k
S6 hybrid        0.73    —       —       0.88    6.5k

TF baseline (S3-S5 only):  mcq 0.37  numeric 0.76  open 0.97
```

**禁止**只报一个 `VPB Score`；矩阵 + PFI 摘要并存。

---

## 5. 语料规模轴（LONGMEMEVAL S/M/L）

| Tier | 行数 | ~tokens | 角色 |
|------|------|---------|------|
| **S** | 22 | ~3k | smoke / CI |
| **M** | 49 | ~8k | 当前 fair default |
| **L** | 120 | ~20k | 规模外推 |
| **XL** | 250 | ~45k | 可选 stress（BEAM 式） |

**每 tier 必报**：

- `TaskAcc(corpus_supported)` vs tier
- `Recall@3` vs tier
- `IngestLift_pp` vs tier

**拟合**（METR 式 horizon 隐喻）：

```
memory_horizon = min tier where Recall@3 ≥ 0.85
```

若 M 已 ≥0.85 而 L 下降 → **scale fragility**；若 L 仍升 → **platform headroom**。

---

## 6. 对照臂（Arms）完整列表

### 6.1 Valhalla arms

| Arm | Track | 说明 |
|-----|-------|------|
| `baseline_no_corpus` | B | 无 ingress |
| `triad_c1_isolated` | B | 每题重孵化 |
| `triad_c1_persistent` | B | batch session |
| `patch_triad_legacy` | A | Tier A 0.08 |
| `hybrid_oracle` | A+B | 逐题 max(persistent, patch) 按 §6.3 策略 |

### 6.2 Transformer 对照臂（成熟产品）

| Arm | 约束 |
|-----|------|
| `tf_zero_shot` | Qwen2.5-0.5B-Instruct，**无** fair 语料 |
| `tf_rag_oracle` | （可选）oracle 检索 + TF 生成，作 **RAG 上限** 非 Valhalla 对手 |

### 6.3 Hybrid oracle 策略（S6）

| 条件 | 选用臂 |
|------|--------|
| `memory_task=abstention` | 拒答正确 > 任意生成 |
| `corpus_supported` + mcq | persistent |
| `corpus_supported` + numeric | max(persistent, patch) |
| open_retrieval | patch |
| open_generation | patch 或 TF（配置位，分报） |

---

## 7. 开发速度轴（ForecastBench + Product Velocity）

**冻结**：L0 `fair_test` 145Q 永不改题。  
**可变**：Valhalla 版本 `milestone_id`。

每 milestone 记录：

| 字段 | 说明 |
|------|------|
| `milestone_id` | e.g. `M20260620` |
| `matrix_snapshot` | §4.3 全矩阵 |
| `PFI` | §9 综合指数 |
| `delta_vs_prev` | 矩阵逐格 Δ |
| `velocity_7d` | 若有 CI 门禁：eval pass rate |

**潜力预测**（仅 Valhalla 臂）：

```
forecast_acc_S1(t+12w) = acc_S1(now) + 12 × weekly_velocity_S1
```

TF **不**做 forecast（成熟产品假设斜率≈0）。

---

## 8. 综合潜力指数 PFI（摘要，非唯一 KPI）

```
PFI = 0.25 × norm(M2 Recall@3 @ M tier)
    + 0.20 × norm(M3 IngestLift on corpus_supported)
    + 0.20 × wedge_score          # S3 mcq + S5 open_retr vs TF
    + 0.15 × norm(hybrid_S6 - TF) # 相对成熟产品缺口
    + 0.10 × scale_headroom        # L tier Recall slope
    + 0.10 × velocity_norm        # milestone 改进速度
```

| PFI | 解释 |
|-----|------|
| ≥ 0.70 | **strong_potential** — 记忆平台 + hybrid 路径清晰 |
| 0.50–0.70 | **viable_wedge** — 垂直场景可产品化 |
| < 0.50 | **early_stage** — 先补 Tier B 解码与规模曲线 |

**wedge_score**（借鉴 HELM 分场景）：

```
wedge_score = weighted_mean( 1[Valhalla ≥ TF] on S3, S5 corpus_supported )
```

当前粗估（0859 数据）：mcq +8.7pp → wedge 部分胜；open_retr 平；→ wedge_score ≈ 0.55–0.65。

---

## 9. 通过门槛（MVP 潜力叙事）

| 门槛 ID | 条件 | 产品含义 |
|---------|------|----------|
| G1 | M2 Recall@3 ≥ 0.70 @ M tier | 记忆检索可用 |
| G2 | M3 IngestLift ≥ +15pp @ corpus_supported | ingress 有行为改变 |
| G3 | S3 mcq ≥ TF + 5pp | 路由楔子成立 |
| G4 | S6 hybrid ≥ TF − 5pp | hybrid 接近成熟产品 |
| G5 | L tier Recall 斜率 ≥ 0 | 规模仍有 headroom |
| G6 | abstention acc ≥ 0.60 | 抗幻觉及格 |

**潜力 benchmark 通过**：G1∧G2∧(G3∨G4) — 即 **平台 + (楔子或 hybrid)**。

---

## 10. 实验协议（三阶段）

### Phase A — 矩阵基线（2–3 天，可复用 0859）

1. 跑 L0 fair_test 全 arms（已有）
2. 计算 L0 的 §4.3 矩阵 + PFI v0
3. 跑 `hybrid_oracle` 离线逐题 max

```bash
python tools/fair_benchmark/run_vpb_matrix.py \
  --reuse-json reports/fair_benchmark/fair_benchmark_test_20260620_0859.json \
  --phase baseline
```

### Phase B — L1 记忆 taxonomy（1 周）

1. 发布 `vpb_memory_suite.json`（80 题 + evidence）
2. 并入去污染语料构造
3. 跑 S1/S2 + abstention/adversarial 分报

```bash
python tools/fair_benchmark/build_vpb_suite.py --extend fair_benchmark_suite.json
python tools/fair_benchmark/run_vpb_matrix.py --phase memory --corpus-tier M
```

### Phase C — 规模 + 里程碑（持续）

1. S/M/L 语料档各跑 persistent
2. 每 release 打 milestone，重跑 L0 矩阵
3. 输出 `vpb_forecast_report.md`

```bash
python tools/fair_benchmark/run_vpb_matrix.py \
  --phase scale --corpus-tiers S,M,L --protocols persistent
python tools/fair_benchmark/run_vpb_forecast.py --milestones M20260620,M20260720
```

---

## 11. 与 fair benchmark 关系

```
fair-1.1 (现有)          VPB v1.0 (扩展)
─────────────────        ─────────────────
145Q test         →      L0 冻结主考卷
ingest_clean 49   →      M tier
recall@3, acc     →      M2, M1
slices            →      scenarios S1-S5
patch + TF        →      S6 hybrid + TF 对照
—                 →      L1 memory taxonomy
—                 →      S/M/L scale axis
—                 →      milestone velocity
—                 →      PFI 摘要
```

**向后兼容**：VPB 不替换 fair leaderboard；fair 是 VPB 的 L0 子集。

---

## 12. 公平性检查清单

| # | 检查 | 方法 |
|---|------|------|
| 1 | 无答案泄漏 | `validate_suite.py` + abstention 题 |
| 2 | TF 未用 fair 微调 | 配置锁定 |
| 3 | Valhalla ingress 可审计 | artifact manifest |
| 4 | 检索与 acc 分报 | 矩阵 S1 vs S2 |
| 5 | 成熟/开发分轨叙事 | TF 仅 S3–S5；velocity 仅 Valhalla |
| 6 | 置信区间 | bootstrap paired on L0 |

---

## 13. 预期基线（20260620 方向性，Phase A 待验）

| 格 | Valhalla | TF | 潜力解读 |
|----|----------|-----|----------|
| S1 M1 | ~41% | N/A | 平台 acc |
| S2 M2 | ~75% | N/A | 检索强 |
| S3 M1 mcq | ~46% | 37% | **楔子胜** |
| S5 open | ~52% mem / ~97% patch | 97% | hybrid 路径 |
| S6 hybrid | **~72%（估）** | 71% | 可超 TF |
| PFI | **~0.62–0.68（估）** | — | viable_wedge |

---

## 14. 实现文件规划

```
tools/fair_benchmark/
  VALHALLA_POTENTIAL_BENCHMARK_V1.md     ← 本文
  POTENTIAL_FORECAST_EXPERIMENT_DESIGN.md
  vpb_memory_suite.json                 ← Phase B
  run_vpb_matrix.py                     ← 矩阵 + PFI
  run_vpb_forecast.py                   ← milestone 外推
  build_vpb_suite.py                    ← L1 题构造
```

---

## 15. 参考文献

- Bommasani et al., *Holistic Evaluation of Language Models (HELM)*, 2022. https://arxiv.org/abs/2211.09110  
- Maharana et al., *Evaluating Very Long-Term Conversational Memory (LoCoMo)*, ACL 2024. https://snap-research.github.io/locomo/  
- Wu et al., *LONGMEMEVAL*, 2024. https://github.com/xiaowu0162/LongMemEval  
- Es et al., *RAGAS*, 2023. https://arxiv.org/abs/2309.15217  
- METR, *Time Horizon*, 2024–2025. https://metr.org/  
- Kokotajlo et al., *ForecastBench*, 2024. https://arxiv.org/pdf/2409.19839  
- Mem0, *BEAM Memory Benchmark* blog, 2025. https://mem0.ai/blog/what-is-beam-memory-benchmark  

---

*Rogue Intelligence LNC. · VPB v1.0 设计 · 2026-06-20*
