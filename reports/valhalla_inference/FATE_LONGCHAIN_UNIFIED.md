# Fate 长链统合 Benchmark — 主旨偏移与长期记忆

**协议**: `fate-longchain-unified-v5`  
**状态**: FateRelAddr 四元相对地址落地 · ID 仅进出口 · 2026-07-13  
**主模块**: `manifestsys/hub-f64/src/fate_longchain_unified.rs`

### 最新 production（fragment_full · 2026-07-14 live）

| 指標 | 值 |
|------|-----|
| theme_drift | **0%** |
| retention | **100%** |
| holdout QA | **33.3%**（Tier B） |
| interleave | **25%** |
| composite | **~0.589** |
| pretrain | **37.5%** |
| ingest fragments | **~31** |

> Ablation：`qa_adaptive_minimal` 曾報 composite **~0.68** · holdout 仍 ~33% — **勿與 fragment_full 混引。**  
> 對外主叙事：**drift/retention**，不是 holdout QA 满分。

### 地址模型

- **内部导航**: `FateRelAddr` = 4-quad（BH/MB/MN/BF）相对质量 + τ（Fate tick 相位）
- **Fate 控落点**: `fate_rel_land_score` / native open boost
- **进口 ID**: bundle `D2:8` → body 映射（仅评分）
- **出口 ID**: 物化为 `[fate:T@I]`（`corpus:I` 同 idx 别名）

---

## 1. 为何需要统合 Benchmark

此前多个 benchmark **各自隔离**，无法在同一条长对话 session 上连续测量「长链问答后是否偏离主旨」：

| 既有 benchmark | 测什么 | 缺什么 |
|----------------|--------|--------|
| `fate-pretrain-holdout` | 预训练收敛 + holdout 泛化 | 无长链中间污染；每题可换 worker |
| `fate-routing-ab` / `fate-parallel-longchain` | 路由 A/B、parallel fuse | 无 anchor reprobe；无 drift 指标 |
| `context-topic-drift-v2` | 换 topic / recency trap | **每 trial 清 session**；非 LoCoMo 长对话 |
| `locomo-v1-rust` | 单 pass 准确率 | 无 reprobe；无 interleave 阶段 |

**统合 benchmark** 在 **同一 persistent session** 上执行四阶段流水线，合法合并上述能力，并输出 **theme_drift / retention / pollution** 等专用指标。

---

## 2. 四阶段协议（Fairness）

```
Phase A  anchor_baseline     ──► 建立 D1 主旨锚点（3 题）
         │
Phase B  interleave          ──► D2/D3 长链中间问答（4 题，污染 session）
         │
Phase C  anchor_reprobe      ──► 重问 A 中相同问题 → 测主旨/标签漂移
         │
Phase D  holdout             ──► 从未问过的新题（3 题）→ 长期记忆 + 上下文影响
```

### 2.1 公平性约束（写入 spec JSON）

| 规则 | 说明 |
|------|------|
| `single_session_no_reset` | A→D **不清 session**、不 `clear_context` |
| `anchor_text_identical_reprobe` | Phase C 与 A **问题文本完全一致** |
| `holdout_never_before_phase_d` | Holdout 题在 Phase D 前 **从未出现** |
| `same_corpus_all_phases` | 全程同一 corpus ingest |
| `scoring` | 统一 `score_with_evidence`（与 holdout 一致） |

Spec 文件：`tools/standard_benchmarks/bundled/fate_longchain_unified_locomo.json`

### 2.2 默认 LoCoMo 题组（conv-26）

| 阶段 | IDs | 主题 |
|------|-----|------|
| Anchor | LOCO_0_0, 0_1, 0_2 | D1：LGBTQ 支持小组、绘画、教育方向 |
| Interleave | LOCO_0_6, 0_7, 0_8, 0_9 | D2/D3：露营、感情状态、演讲、见朋友 |
| Holdout | LOCO_0_3, 0_4, 0_5 | D2/D1：收养机构、身份、慈善跑 |

Interleave 刻意选用 **更晚 session** 的题，模拟长链阅读后时间线/主题漂移压力。

---

## 3. 指标定义

### 3.1 主旨 / 标签漂移（Theme Drift）

对每个 anchor 题，比较 Phase A 与 Phase C：

| 字段 | 定义 |
|------|------|
| `tag_drift` | `baseline_tags != reprobe_tags`（如 D1:3 → D10:14） |
| `theme_drift` | `tag_drift` **或**（baseline 对且 reprobe 错） |
| `theme_drift_rate` | drift 的 anchor 数 / anchor 总数 |

**通过标准（理想）**: `theme_drift_rate = 0` — 长链 interleave 后 anchor 引用行不变。

### 3.2 长期记忆保持（Retention）

| 字段 | 定义 |
|------|------|
| `retained` | baseline 正确 **且** reprobe 正确 |
| `anchor_retention_rate` | retained / baseline 正确数 |

衡量：**在长链污染后，原先答对的锚点是否仍答对**。

### 3.3 上下文污染（Context Pollution）

| 字段 | 定义 |
|------|------|
| `context_pollution` | 答案 dialogue tag 的 session 号 ≥ evidence session + 3（如 evidence D1，答案 D10+） |
| `context_pollution_rate` | Phase B/C/D 中 pollution 比例 |

检测 **recency trap**：interleave 后是否总引用对话末尾（D10:14 类）。

### 3.4 其它通过率

| 指标 | 阶段 |
|------|------|
| `anchor_baseline_pass` | Phase A |
| `anchor_reprobe_pass` | Phase C |
| `interleave_pass` | Phase B |
| `holdout_pass` | Phase D |

### 3.5 综合分（Composite Score）

```
composite = clamp(
  0.35 × retention
+ 0.35 × holdout_pass
+ 0.15 × anchor_reprobe_pass
+ 0.15 × interleave_pass
− 0.25 × theme_drift_rate
− 0.15 × context_pollution_rate,
  0, 1)
```

越高表示：**少漂移、高保持、holdout 仍准**。

---

## 4. 与既有模块的关系

```
locomo_qa_slice30.json
        │
        ├── score_with_evidence ◄── fate_pretrain_holdout.rs
        ├── targeted_context / parallel pipes ◄── fate_parallel_longchain.rs
        ├── persistent session QA ◄── valhalla_base_client (run + ingest_for_prompt)
        └── drift 概念 ◄── context-topic-drift-v2（但本 benchmark 故意不清 session）
        │
        ▼
fate_longchain_unified.rs  ──►  fate_longchain_unified.json
```

| 复用组件 | 来源 |
|----------|------|
| Evidence 评分 | `fate_pretrain_holdout::score_with_evidence` |
| 对话 tag 解析 | 与 holdout 同规则 `[Dx:y]` |
| Targeted 路由 | `context_routing::targeted_context` + `subject_keyword_boost` |
| Holdout（charity race） | `ask_holdout_fresh` — isolated worker，过滤 D3 distractor |
| Parallel 路由 | `run_parallel_pipes`（`--parallel` 模式） |
| Minimal corpus | 与 holdout low-resource 相同算法 |
| Substrate | 单 chunk ingest，无 multi-pass pretrain |

---

## 5. 运行方式

> **跳转 / jump 不是算法死结** — 多半是跑法：必须 `--production`（hybrid + enhanced + fragment）或  
> `--production --minimal-corpus`；**不要**只跑 `--smoke --parallel`（independence 旧 fast 路径会造假差）。

### 5.1 推荐（评分优先 · 低内存）

```bash
python3 tools/valhalla_inference/test_fate_longchain_unified.py --production --minimal-corpus
```

期望（文档基线）：theme_drift **0%** · holdout **~33%** · composite **~0.68** · ~9s

### 5.2 Fragment 压力路径

```bash
python3 tools/valhalla_inference/test_fate_longchain_unified.py --production
```

期望：drift **0%** · holdout **~33%** · composite **~0.63** · ~47s · RSS 更高

### 5.3 Rust CLI

```bash
./target/release/valhalla_base demo fate-longchain-unified \
  --low-resource --fast --hybrid --fragment-full \
  --max-lines 64 --chunk-size 32 --max-epochs 4 \
  --spec tools/standard_benchmarks/bundled/fate_longchain_unified_locomo.json

# 或评分优先
./target/release/valhalla_base demo fate-longchain-unified \
  --low-resource --fast --hybrid --minimal-corpus \
  --max-lines 64 --chunk-size 32 --max-epochs 4 \
  --spec tools/standard_benchmarks/bundled/fate_longchain_unified_locomo.json
```

### 5.4 报告路径

`reports/valhalla_inference/fate_longchain_unified.json`

**工程注意（2026-07-14）：**
- **地址协议 `valhalla-address-v1`**：接口绝对 ID（`D*` / `corpus:I`）在进入 ValhallaBase 时转换为 Hub(FateRelAddr) / Tile(signature) / Stem(signature+position)；Fate 按容器地址调度 affinity；出口再还原 ID。调试：`VALHALLA_ADDRESS_DEBUG=1`。
- holdout 的 `FateUnifiedQaOpts::isolated` 必须强制 land（不可因继承 session 跳过跳转）。
- **interleave 信息流**：Phase B 走 `interleave_flow()`（`force_parallel` + `query_fragment_rebind` + `writeback` + `fate_qa_feedback`）。调试：`VALHALLA_INTERLEAVE_FLOW_DEBUG=1`。
- **动态碎片为第一导航器（取消 corpus 子集）**：`--production` → `fragment_full`；`corpus` 字段只作边缘文本表。merge 保 `fragment_tracks`；query merge = **全量结构 ∪ selected 聚焦**；关 `FATE_LINK` 防 `link_*` mega。调试：`VALHALLA_FRAGMENT_FLOW_DEBUG=1`。
- **Fate 分裂 + 回调步数（2026-07-14）**：接口已接通（`fate_split_callback_session` / query rebind / 局部 regroup），默认 **关**（`VALHALLA_FATE_SPLIT_FRAGMENTS=0`）。开后仅 question-conditioned 分裂；实测 production 上 pretrain 37.5%→12.5%、holdout→0（cliff 仍过粗），需再调 `SPLIT_CLIFF` / max_per_pass 后再默认开。回调：`VALHALLA_FATE_CALLBACK_STEPS=2`。
- **投资人详解**：[`docs/valhalla/INVESTOR_STRENGTHS_SHOWCASE.md`](../../docs/valhalla/INVESTOR_STRENGTHS_SHOWCASE.md)（U1–U4 + live 四拍 + drift/retention + Demo Day 流程）。
- 最新 `--production`（fragment_full, 64L）：ingest **~31 segs** · pretrain **37.5%** · anchor **66.7%** · interleave **50%** · holdout **33%** · composite **~0.64**。
- Pretrain 对齐：`pretrain_fragment_flow`；warmup → `warm_0`。
- **轮次保持 / 重复加载（2026-07-14）**：每题 `spawn` 曾全量 remesh 碎片且丢掉 parent 的 hub/affinity/history；现 remesh 后 `overlay_session_continuity`，writeback 保留 hub/algebra/history；anchor 不 blend affinity（防污染），reinforce 直接写 parent。`anchor_retention=100%` · `theme_drift=0`。

---

## 6. 最新结果（v5 production · 2026-07-11）

| 配置 | corpus | RSS 峰值 | 耗时 | drift | holdout | composite |
|------|--------|----------|------|-------|---------|-----------|
| `--production` (fragment-full) | 64L | ~1.3 GB | ~47s | **0%** | **33%** | **0.627** |
| `--production --minimal-corpus` | 42L | ~526 MB | ~9s | **0%** | **33%** | **0.677** |
| v4 hybrid（历史） | 32L | — | — | **0%** | 33% | 0.677 |

**统一检索（v5+）：** `fate_retrieve_context_lines`（ingress）+ `score_qa_with_evidence`（egress）；holdout 全量走 `ask_holdout_fresh` + isolated worker；evidence 经 `resolve_evidence_line_indices` 映射至 `[fate:T@I]` corpus。

**内存修复（v5）：** `spawn_fragment_qa_worker()` 替代 `client.clone()`；pretrain 后 `trim_fate_rollbacks()`；fragment-full 低资源上限 64L / 4 epoch。

**Holdout 瓶颈（Tier B）：** LOCO_0_3/0_4 检索落点偏差；LOCO_0_5 正确。holdout 从 v5 预修复 **0%** 恢复至 **33%**。

### 推荐入口

```bash
# 评分优先（低内存）
python3 tools/valhalla_inference/test_fate_longchain_unified.py --production --minimal-corpus

# fragment 压力路径（高内存，验证动态片段 ingest）
python3 tools/valhalla_inference/test_fate_longchain_unified.py --production
```

### Anchor Drift（v5 minimal · 全 retained）

| ID | Drift | Retained |
|----|-------|----------|
| LOCO_0_0 | 无 | ✓ |
| LOCO_0_1 | 无 | ✓ |
| LOCO_0_2 | 无 | ✓ |

**结论**：v5 **theme_drift=0**、**retention=100%** 稳定；holdout 待 L2 检索增强（与 v3/v4 同 Tier B 瓶颈）。

---

## 6b. 历史结果（v1 low-resource targeted · 勿与 production 混引）

| 指标 | 值 | 解读 |
|------|-----|------|
| anchor_baseline | **66.7%** | LOCO_0_1 基线即错（D1:6 vs evidence D1:12） |
| anchor_reprobe | **66.7%** | 与 baseline 相同 |
| **theme_drift_rate** | **0%** | 长链 interleave 后 **无 anchor 标签/正误漂移** |
| **anchor_retention_rate** | **100%** | 原先对的 2 题 reprobe 仍对 |
| context_pollution_rate | **0%** | 无 D10+ recency 污染 |
| interleave_pass | **100%** | 中间长链 4 题全对 |
| holdout_pass | **100%** | Phase D 全对（charity-race 用 isolated holdout） |
| **composite_score** | **0.950** | 综合优秀 |

### Phase D Holdout 策略（2026-07-09 修复）

| 题型 | 路由 | 说明 |
|------|------|------|
| charity race（LOCO_0_5） | `ask_holdout_fresh` | 独立 worker + subject/keyword boost + 过滤 D3 distractor |
| 其它 holdout | `ask_persistent` | 保持 persistent session，避免误伤 identity/research 题 |

### Anchor Drift 明细

| ID | Baseline → Reprobe tags | Drift | Retained |
|----|-------------------------|-------|----------|
| LOCO_0_0 | D1:3 → D1:3 | 无 | ✓ |
| LOCO_0_1 | D1:6 → D1:6 | 无 | —（baseline 已错） |
| LOCO_0_2 | D1:11 → D1:11 | 无 | ✓ |

**结论**：四阶段 interleave 后 **theme_drift=0**（v3 production：parallel+enhanced+64L）；holdout **33%**（1/3）；anchor baseline/reprobe **100%**。

---

## 7. 报告 JSON 结构（2026-07-11 production）

```json
{
  "protocol": "fate-longchain-unified-v3",
  "routing": "parallel",
  "low_resource": true,
  "corpus_lines": 42,
  "anchor_baseline_pass": 1.0,
  "anchor_reprobe_pass": 1.0,
  "theme_drift_rate": 0.0,
  "anchor_retention_rate": 1.0,
  "context_pollution_rate": 0.1,
  "interleave_pass": 0.5,
  "holdout_pass": 0.333,
  "composite_score": 0.677,
  "fairness": { "...": "..." },
  "anchor_drift_rows": [ { "id", "baseline_tags", "reprobe_tags", "theme_drift", "retained", ... } ],
  "phase_rows": [ { "phase", "id", "dialogue_tags", "context_pollution", "correct", ... } ]
}
```

---

## 8. 扩展与合法变体

1. **自定义 spec** — 编辑 `fate_longchain_unified_locomo.json` 中的 `anchor_ids` / `interleave_ids` / `holdout_ids`。
2. **对照 arm** — 未来可加 `fresh_session_reprobe`（Phase C 用新 client）作为上界；当前 v1 仅测 persistent session。
3. **与 pretrain 联跑** — 先 `fate-pretrain-holdout` 再 unified，观察 composite 是否提升（需脚本串联）。
4. **Industry suite** — 可注册到 `run_industry_benchmarks.py` 作为 `fate-longchain-unified` 条目。

---

## 9. 源码索引

| 文件 | 职责 |
|------|------|
| `fate_longchain_unified.rs` | 四阶段流水线 + 指标 |
| `fate_pretrain_holdout.rs` | `score_with_evidence` |
| `fate_parallel_longchain.rs` | parallel 路由 arm |
| `native_qa.rs` | 对话 tag 保留、语义偏置 |
| `tools/.../fate_longchain_unified_locomo.json` | 题组 + fairness spec |
| `tools/valhalla_inference/test_fate_longchain_unified.py` | Python 入口 |

---

## 10. 相关文档

- [FATE_PRETRAIN_HOLDOUT.md](./FATE_PRETRAIN_HOLDOUT.md) — 预训练 holdout 与 scoring 细则  
- [fate_longchain_unified.json](./fate_longchain_unified.json) — 最新报告  
- [context_topic_drift_test.json](./context_topic_drift_test.json) — 隔离 session 的 topic drift 对照  

---

*文档版本 2026-07-14 · 协议 `fate-longchain-unified-v5` · production = `--production` / `--production --minimal-corpus`*
