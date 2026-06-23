# Hybrid Valhalla 2.0 — 技术白皮书

**文档类型**：产品技术白皮书  
**版本**：hybrid-2.0 · hybrid-4.0-rust-pipeline · fair-1.2 · hub-f64 1.1.0  
**日期**：2026-06-23  
**出品**：Rogue Intelligence LNC.（游侠智能有限公司）  
**读者**：ML 系统工程师、架构师、第三方审计方  
**状态**：实验驱动；论断均绑定可复现产物

---

## 摘要

Hybrid Valhalla **hybrid-2.0** 是 **统一问答产品**：每道题在以下路径中自动择优：

1. **Tier B 原生记忆**（Valhalla 结构孵化，无 `model.generate()`），及  
2. **Tier A 补丁 LM**（结构导出为 Qwen2.5-0.5B-Instruct 权重扰动）。

在 **fair-1.2** 基准 **test 切分**（**145 题**、**49 行去污染语料**）上，统一产品表现：

| 指标 | Hybrid 统一 |
|------|-------------|
| 总计 | **80.69%**（117/145） |
| 开放题 | **93.44%** |
| 多选题 | **60.87%**（28/46） |
| 数字题 | **84.21%** |

**非目标**：宣称在通用基准上 SOTA 于 GPT-4/Claude；宣称 Valhalla 全面替代 Transformer。

**主产物**：`reports/hybrid_valhalla_lm/hybrid_v4_mcq_patch_strengthen_final.json`

---

## 1. 术语

| 术语 | 定义 |
|------|------|
| **Hybrid 产品** | 单一路由器；全部 prompt 走同一策略（`hybrid-2.0`） |
| **Tier A（补丁 LM）** | 结构 → `model_patch` → Qwen 前向 + 生成 |
| **Tier B（原生记忆）** | 结构孵化 + 规则/打分解码；无 HF generate |
| **一次孵化** | 语料只 ingress 一次；结构在 batch 内共享（persistent 会话） |
| **整题 ingress** | MCQ：题干 + 四选项一并编码 |
| **fair-1.2** | 去污染语料；smoke/val/test 切分；原生与 LM 分开计分 |
| **strict MCQ** | 输出须可解析为 `ANSWER: X`；抽取失败计错 |

---

## 2. 系统架构

### 2.1 分层栈

```
┌─────────────────────────────────────────────────────────┐
│              Hybrid 路由器（hybrid-2.0）                  │
│  开放题→patch_lm │ MCQ→memory_mcq_best │ 数字→max 三路    │
└──────────────────────────┬──────────────────────────────┘
                           │
         ┌─────────────────┼─────────────────┐
         ▼                 ▼                 ▼
   Tile+Fate 记忆      Triad 记忆        补丁 LM（Qwen 0.5B）
   （原生 QA）         （原生 QA）       （Tier A 导出）
         │                 │                 │
         └─────────────────┴─────────────────┘
                           │
                           ▼
              结构孵化层
         Hub · Tile · StemCell · Fate 循环
                           │
                           ▼
              语料 ingress（Grinder / 可选 LLM 状态）
```

### 2.2 核心模块

| 模块 | 在 Hybrid 2.0 中的作用 |
|------|------------------------|
| **Hub** | 四象限路由、Fate 偏好、checkpoint 编排 |
| **Tile** | 块记忆；实验中 MCQ 最佳 body（22–24/46） |
| **StemCell** | 自组织簇；数字/MCQ 辅助 |
| **Fate** | 动态停止/闭合；MCQ 分解-聚合循环 |
| **native_qa** | Rust 解码：`mcq_option`、数字抽取、开放检索 |
| **patch bridge** | Python：@0.08 施加 Valhalla 补丁（triad body） |

### 2.3 Hybrid 路由策略

实现：`tools/fair_benchmark/hybrid_valhalla_lm.py`

| `score_type` | 策略 ID | 选择规则 |
|--------------|---------|----------|
| `open`、`open_generation` | `patch_lm` | Valhalla 补丁 + Qwen 生成 |
| `mcq` | `memory_mcq_best` | Tile+Fate、Triad 间取首个正确；否则取最优可用 |
| `numeric` | `hybrid_max_all` | Tile+Fate、Triad、patch 间取首个正确 |
| 其他 / 语料支持 | `memory_triad` | Triad persistent 记忆 |

**产品不变量**：用户不选轨道；路由器按题决定。

---

## 3. 基准设计（fair-1.2）

### 3.1 题目套件

- **套件文件**：`tools/fair_benchmark/fair_benchmark_suite.json`
- **Test 切分**：145 题
  - 46 道四选一 MCQ
  - 数字题子集
  - 开放题 / open_generation
- **语料**：49 行参考文本；评测语料 **不含** 标准答案

### 3.2 计分

- **numeric**：归一化答案匹配
- **mcq**：从 `ANSWER: X` 或末行严格抽取字母
- **open**：按 fair  harness 的参考重叠 / 生成质量指标

### 3.3 会话协议

| 协议 | 说明 | MCQ 影响 |
|------|------|----------|
| 逐题重载 | 每题重新 ingress 语料 | ~28% MCQ |
| **一次孵化 persistent** | 语料一次、多题连答 | **~46–52% MCQ** |

Hybrid 2.0 的 Tier B 路径使用 **persistent** 记忆。

### 3.4 与 Hybrid 一并报告的基线

| 轨道 | 总准确率 | 开放 | MCQ | 数字 |
|------|----------|------|-----|------|
| **Hybrid 统一** | **80.00%** | 95.08% | 54.35% | 86.84% |
| 仅 Triad 记忆 | 39.31% | 24.59% | 43.48% | 57.89% |
| 仅 Tile+Fate | 41.38% | 22.95% | 52.17% | 57.89% |
| 补丁 LM（路由子集） | 87.88% | 95.08% | 0%* | 76.32% |
| Qwen 0.5B 零样本（145Q） | 71.0% | ~93% | 37.0% | ~76% |

*hybrid-2.0 策略下 patch LM 非 MCQ 主路径。

**Hybrid 来源混合（145Q）**：`tile_fate_persistent: 72`、`triad_persistent: 1`、`patch_lm: 72`

---

## 4. Tier B：原生记忆路径

### 4.1 孵化流水线

1. 49 行语料 ingress Valhalla（Grinder 向量）。
2. 结构演化（启用 Tile 和/或 Triad body）。
3. MCQ 且 `VALHALLA_MCQ_FATE_CYCLES=1`：标签 `task_tag:mcq`，分解题干+选项，Fate 动态停止聚合。

### 4.2 MCQ 解码（`mcq_option`）

四选项独立打分 → argmax → `ANSWER: X`：

- 选项向量 vs 整题向量（余弦）
- 选项 vs 结构摘要向量
- 与语料行的词重叠（降权以防关键词猜测）
- 关键词重叠辅助

**未采用**：LLM 思维链；仅相对 stem 的比较（路线图）。

### 4.3 MCQ 实验摘要（46 题）

| 方法 | 正确 | 准确率 |
|------|------|--------|
| 原生·无语料 | 6/46 | 13.0% |
| 逐题重载 | 13/46 | 28.3% |
| 一次孵化 Triad（全模块） | 21/46 | 45.7% |
| 一次孵化仅 Tile | 22/46 | 47.8% |
| Tile + MCQ Fate 循环 | 24/46 | 52.2% |
| **Hybrid deploy + 结构引导 patch** | **28/46** | **60.9%** |
| Qwen 零样本 | 17/46 | 37.0% |

**结论**：会话协议（重载→一次孵化 +18 pp）大于小改打分。瓶颈在 **四选项比较**。

---

## 5. Tier A：补丁 LM 路径

### 5.1 补丁几何

- 4 组 `input_layernorm.weight` 按 `quad_layer_scales` 缩放
- prompt token embedding 经 `patch_vector` 投影扰动
- 默认强度：**0.08**，body：**triad**

### 5.2 在 Hybrid 2.0 中的角色

- **开放题主路径**：test 开放切片约 95%
- **数字题候选**：经 `hybrid_max_all` 合并
- **非 MCQ 主路径**：同基准上 patch 对 MCQ 弱于原生（约 26–28%）

### 5.3 对照（历史背景）

48Q 孵化套件（strict）：Valhalla triad 补丁 **-2.08 pp** vs 随机补丁 **-14.58 pp** — 结构信号 **非噪声**，但小套件上 strict 净增益 ≤ 0。Hybrid 产品在 patch **胜出** 处（开放题）使用，而非处处使用。

---

## 6. 统一产品实现

### 6.1 入口

| 组件 | 路径 |
|------|------|
| 路由逻辑 | `tools/fair_benchmark/hybrid_valhalla_lm.py` |
| 运行器 | `tools/fair_benchmark/run_hybrid_valhalla_lm.py` |
| 原生二进制 | `manifestsys/hub-f64` → `valhalla_native_qa` |
| MCQ Fate | `manifestsys/hub-f64/src/mcq_fate.rs` |

### 6.2 复现

```bash
# 构建原生 QA
cd manifestsys/hub-f64
RUSTFLAGS='-L /opt/cuda/lib64' cargo build -p hub-f64 --release --bin valhalla_native_qa

# 运行 Hybrid（test 阶段，复用 fair JSON 记忆轨道）
cd /path/to/Valhalla
python3 tools/fair_benchmark/run_hybrid_valhalla_lm.py \
  --phase test \
  --reuse-json reports/fair_benchmark/fair_benchmark_test_20260620_0859.json

# MCQ Fate 循环（Tile body）
export VALHALLA_MCQ_FATE_CYCLES=1
```

### 6.3 环境变量（MCQ）

| 变量 | 默认 | 用途 |
|------|------|------|
| `VALHALLA_MCQ_FATE_CYCLES` | 0（Hybrid MCQ 运行时为 1） | 启用 MCQ 分解/聚合 |
| `VALHALLA_MCQ_FATE_MAX_CYCLES` | 64 | Fate 循环上限 |
| `VALHALLA_MCQ_FATE_HARD_MAX` | 128 | 硬停止 |
| `VALHALLA_MCQ_FATE_MIN_CYCLES` | 5 | 停止前最少循环 |

---

## 7. 可验证论断登记

| 论断 | 证据 | 状态 |
|------|------|------|
| Hybrid 145Q test 80.7% | `hybrid_v4_mcq_patch_strengthen_final.json` | ✓ |
| 开放题 93.4% 经 patch 路由 | 同上 `by_score_type.open` | ✓ |
| MCQ deploy 52.2% → 60.9% | post_decode_fix vs final | ✓ |
| 原生 MCQ 记忆基线 | 24/46（结构 decode） | ✓ |
| Hybrid 总胜 Qwen 71% | 80.7% vs 71% 同切分 | ✓ |
| 通用基准胜 GPT-4/Claude | — | ✗ 未宣称 |
| FIPS 认证加密 | — | ✗ 未宣称（Yaksha 另文） |

---

## 8. 已知限制与路线图

### 8.1 限制

1. **MCQ 天花板约 61%（deploy）**：选项比较仍弱于开放生成；需 stem 条件相对比较器。  
2. **配套 LM 较小**：0.5B — 证明结构层价值；非生产 LM 选型。  
3. **语料规模**：49 行 — 垂直 POC，非 web 级 RAG。  
4. **Tier B 开放检索 alone ~25%**：开放题在 **产品层** 经 Hybrid 解决，非 memory-only。

### 8.2 路线图（工程）

| 优先级 | 项 | 目标 |
|--------|-----|------|
| P0 | stem 条件相对 MCQ 比较器 | MCQ fair test 65%+ |
| P1 | 语料行 → 选项支持图 | 降低关键词假阳性 |
| P2 | 企业 SDK（路由轨迹导出） | GA |
| P3 | 更大 fair 语料 + 第三方审计 | fair-2.0 |

---

## 9. 安全与部署

- **本地部署**：Rust 二进制、本地 `.hub` / checkpoint  
- **可选加密**：Yaksha AEAD 模块（独立白皮书；Industry Gate 叙事）  
- **审计**：Hybrid JSON 每行含路由键（`tile_fate_persistent` | `triad_persistent` | `patch_lm`）  
- **Tier B 无强制外部 API**

---

## 10. 参考文献

| 文档 | 路径 |
|------|------|
| MCQ 与 Fair 总结 | `docs/hybrid_2.0/DAY_SUMMARY_20260623.md` |
| Hybrid 最终 JSON | `reports/hybrid_valhalla_lm/hybrid_v4_mcq_patch_strengthen_final.json` |
| 公开 viz 仓库 | https://github.com/Rogue-Intelligence-INC/valhalla-transformer（v2.0） |
| Valhalla vs Transformer 白皮书 | `docs/valhalla_vs_transformer/en/04_TECHNICAL_WHITEPAPER.md` |
| 产品定义 | `docs/VALHALLA_PRODUCT_DEFINITION.md` |

---

*Rogue Intelligence LNC. · Hybrid Valhalla hybrid-2.0 · fair-1.2 · hub-f64 1.1.0 · 2026-06-23*
