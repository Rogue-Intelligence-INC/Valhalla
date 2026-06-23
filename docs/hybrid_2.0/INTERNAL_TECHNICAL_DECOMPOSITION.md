# Hybrid 2.0 — 内部技术分解（非对外）

**用途**：工程分析、瓶颈定位、复现审计  
**版本**：hybrid-2.0 · fair-1.2 · hub-f64 1.1.0  
**日期**：2026-06-22  
**性质**：内部文档 — 含实现细节、启发式、评测语义与已知 gap

---

## 0. 执行拓扑：谁跑什么

```
Python (orchestration)                    Rust (compute)
─────────────────────                    ────────────────
run_hybrid_valhalla_lm.py
  ├─ session.native_qa_batch_persistent  → subprocess valhalla_native_qa (JSON stdin/stdout)
  ├─ patch_track.eval_patch_arm_fresh    → HF Qwen + apply_valhalla_patch (Python/torch)
  └─ hybrid_valhalla_lm.build_hybrid_rows → 纯 Python 合并

tools/transformer_grinder/grind.py       → 预计算 corpus_ports / question_ports (hash 384-d)
valhalla_model_export (可选)             → 独立 export patch JSON
```

**关键点**：Tier B 无 torch 依赖；Tier A 每题 `load_model` + `del model` + `gc`（fair 协议）。Hybrid 合并在 Python，**不在 Rust 内**。

---

## 1. 数据类型与 JSON 契约

### 1.1 Batch QA 请求 (`NativeQaBatchRequest`)

| 字段 | 类型 | 含义 |
|------|------|------|
| `body` | str | `triad` / `tile` / `stemcell` / `hub` |
| `corpus` | `[str]` | 原始语料行 |
| `corpus_ports` | `[BodyFrontPorts]` | grinder 预编码，与 corpus 等长 |
| `questions[]` | | 每题 |
| `questions[].question` | str | 整题文本（MCQ 含 A–D 行） |
| `questions[].score_type` | str | `open` / `mcq` / `numeric` |
| `questions[].question_ports` | BodyFrontPorts | 整题 embedding + 三 port |
| `incubation_cycles` | usize | Phase1 optimize 轮数（Hybrid 用 1） |
| `qa_cycles` | usize | 非 MCQ-Fate 题的 optimize 轮数 |
| `ingress_mode` | str | `grinder`（默认） |

### 1.2 单题响应字段（batch item）

| 字段 | 来源 |
|------|------|
| `answer` | decode_* 输出 |
| `decode_path` | `mcq_option` / `mcq_fate_cycle` / `direct_math` / … |
| `tile_count`, `stem_cells` | `model_patch` |
| `mcq_fate_cycles` | Fate 管线总 cycle 数 |
| `mcq_fate_stopped` | 停止原因字符串 |
| `mcq_decompose_score`, `mcq_aggregate_score` | [0,1] 启发式 |

### 1.3 Hybrid 合并行

```python
{
  "id", "score_type", "correct",           # correct 由 score_answer() 对 reference 判定
  "hybrid_source",                         # tile_fate_persistent | triad_persistent | patch_lm
  "hybrid_policy",                         # patch_lm | memory_mcq_best | hybrid_max_all
  "answer",                                # assistant 或 native_answer
  ...base row fields from winning track...
}
```

---

## 2. Grinder：文本 → 向量（默认 hash）

**文件**：`tools/transformer_grinder/grind.py` · Rust 镜像 `dl_front.rs::hash_embed`

### 2.1 hash_embed 算法

```
seed = HASH_SEED_TEXT (0x544558545F464E54) 或 per-shard seed
for each byte b in utf-8(text):
    h = (h * 6364136223846793005 + b) mod 2^64
for i in 0..384:
    h = (h * 6364136223846793005 + i) mod 2^64
    out[i] = ((h >> 33) / (2^32-1)) * 2 - 1
return L2_normalize(out)
```

**性质**：确定性、无语义、同构文本同向量。fair benchmark 默认 backend=`hash`。

### 2.2 project_port(body_seed, cap=128)

从 384-d embedding 用 LCG 采样 + 权重投影到 **128 维** body port，再 L2 normalize。

三 port：
- `BODY_PORT_SEED_HUB = 0x4855425F464E54`
- `BODY_PORT_SEED_TILE = 0x54494C455F464E54`
- `BODY_PORT_SEED_STEM = 0x5354454D5F464E54`

`grind_shard_for_native_qa()` 输出 JSON：`{hub, tile, stem, embedding, encoder, embed_dim}`。

### 2.3 可选 backend

| backend | 条件 | 行为 |
|---------|------|------|
| `hash` | 默认 | 上式 |
| `minilm` | sentence-transformers | 隐层 quad mean → ports |
| `transformer` | HF AutoModel | Qwen 等 hidden states |

Rust `VALHALLA_DL_FRONT=1` 走 Python sidecar `encode.py`，与 grinder 同族 API。

---

## 3. TriadSession：结构计算核心

**文件**：`signal_ingress.rs`

### 3.1 IngressBudget

| 模式 | max_items | max_signals_total | max_signals_per_item |
|------|-----------|-------------------|----------------------|
| `default()` | ∞ | ∞ | 4096 |
| `experiment()` | 64 | 32768 | 512 |
| **`incubation()`** | 512 | 512000 | 512 |

- **isolated QA** (`run_native_qa`)：`IngressBudget::experiment()`，且默认最多 ingest **32** 行语料（`VALHALLA_NATIVE_MAX_CORPUS_LINES`）
- **batch persistent**：`IngressBudget::incubation()`，ingest **全部** corpus_ports

这是 MCQ 28% vs 46% 的**首要工程原因**（不仅是协议名不同）。

### 3.2 emit_body_ports 分流

```rust
feed_hub  = body ∈ {Hub, Triad}
feed_tile = body ∈ {Tile, Triad}
feed_stem = body ∈ {StemCell, Triad}
```

- **Hub 路径**：每个标量 → `HubRequest{ intensity: v.clamp(0,1), Learning }` → `route_index(message)` ∈ {0..3} → `hub_routed[]++`
- **Tile 路径**：`tile.receive_batch(ports.tile)` 或 `receive_data(v)`
- **Stem 路径**：值 clamp 到 [-1,1]；每 64 个 signal 注入 `0.0` 脉冲；末尾再 `receive_data(0.0)`

### 3.3 run_optimize_cycles(n)

```rust
for _ in 0..n {
    if tile: tile.process_tiles_cycle(); break if tile.has_fate_converged()
    if stem: stem.process_cells_cycle(); stem.evolve(0.5); maybe evolve(0.35); break if converged
}
if hub/triad: hub.mark_epoch_complete()
```

Incubation 阶段：`run_optimize_cycles(incubation_cycles)` 后才 `build_feed_report`。

### 3.4 Incubation snapshot fork

```rust
TriadIncubationSnapshot {
    completed_tiles, stem_cells, tile_total_operations,
    hub_routed[4], ingress_encoder, hub_prefs
}
```

`from_incubation_snapshot()` **复制** tile/stem 结构状态，**不**复制 hub Fate 全状态（hub 新建，prefs 从 snap 恢复路径见实现——实际上 snap 存 hub_prefs，session 重建时 tile/stem 恢复）。

每题 QA：
1. fork snapshot
2. `memories = corpus_memories.clone()` + push question MemoryEntry
3. `emit_body_ports(question_ports)` — 整题一次
4. MCQ+Fate 或 `build_feed_report(qa_cycles)`
5. `decode_from_report()`

**语料 MemoryEntry** 在 Phase1 `feed_grinder_corpus()` 时建立，每行 `{text, signals: embedding, source:"corpus", corpus_idx}`。

---

## 4. build_model_patch：结构 → 可导出 patch

**函数**：`signal_ingress.rs::build_model_patch`

### 4.1 patch_vector（dim = PATCH_VECTOR_DIM）

| body | 填入方式 |
|------|----------|
| Hub/Triad | `hub_routed[i]/total` → [0..3]；`hub_prefs[i]` → [32+i] |
| Tile/Triad | 所有 `tile.data_signature` 标量累加到 `patch_vector[idx % DIM]` |
| Stem/Triad | stem signatures ×0.5 累加 |
| Triad 额外 | hub_prefs 再加到 vector 首尾 |

最后 **L2 normalize** 整个 patch_vector。

### 4.2 quad_layer_scales[4]

| body | 公式 |
|------|------|
| Hub/Triad | `1.0 + 0.05 * (hub_prefs[i] - mean(prefs))` |
| Tile | 用 tile_signatures 首元素均值：`1.0 + 0.03 * (v - m)` |
| StemCell | 用 cell energy：`1.0 + 0.04 * (e/max_e - 0.5)` |

Tier A 应用（Python `apply_valhalla_patch`）：
```python
group_size = n_layers // 4
for layer li:
    qi = min(3, li // group_size)
    scale = 1.0 + strength * (scales[qi] - 1.0)   # strength=0.08
    layer.input_layernorm.weight *= scale
for tid in prompt_token_ids:
    embed_tokens[tid] += strength * project_patch(patch_vector, hidden_size)
```

**每题 fresh reload**：patch 不跨题累积（fair 要求；防 collapse）。

---

## 5. 解码器：公式级分解

**文件**：`native_qa.rs`

### 5.1 共用：dot_cosine

标准 cosine similarity，零向量 → 0。

### 5.2 score_structure_alignment(sig)

```
s = Σ_tile [ 0.15 * fate_tile_weight(hub_prefs, ti) * cos(sig, tile_sig) * energy * (1+|act|) ]
  + Σ_stem_sig [ 0.08 * stem_w * cos(sig, stem_sig) ]
  + Σ_cluster [ 0.12 * stem_w * cos(sig, cluster_sig) * max(ce, 0.01) ]

stem_w = (hub_prefs[1] + hub_prefs[3]) / 2  或 1.0
```

### 5.3 score_memories（open / fallback）

对每条 memory（可 exclude source）：
```
s = cos(m.signals, query_sig)
  + 0.35 * cos(m.signals, patch)
  + score_structure_alignment(m.signals, ...)
```

### 5.4 decode_mcq — 核心公式（每选项独立）

对选项 letter，令 `opt_sig = decode_signal(opt_line)`：

```
forward = cos(opt_sig, query_sig)
        + 0.5 * cos(opt_sig, patch)
        + score_structure_alignment(opt_sig, ...)
        + Σ_{m∈corpus} [ 0.2*cos(opt_sig, m.signals) + 0.2*keyword_domain_boost(q, m.text) ]
        + mcq_corpus_option_boost(opt_line, corpus)    # 或 ×0.1 若 entailment
        + keyword_overlap_count(q, opt_line) * 0.08
        + 0.1 * mean(hub_prefs[0..4])
```

**mcq_corpus_option_boost**（weak boost，当前保留）：
```
per_hit = env VALHALLA_MCQ_CORPUS_BOOST_HIT (default 0.12)
cap     = env VALHALLA_MCQ_CORPUS_BOOST_CAP (default 0.85)
min_word= env VALHALLA_MCQ_CORPUS_BOOST_MIN_WORD (default 5)
对 option 每个 ≥5 字符词，若任一 corpus 行包含该词 → +per_hit，上限 cap
```

**Entailment 分支**（`VALHALLA_MCQ_ENTAILMENT=1`，默认 off）：
```
s = forward * 0.48 + entail * 0.58
entail = Σ_rank 1/(1+rank) * mcq_line_entails_option(...) * mem_score
```

**输出**：`argmax_letter forward` → `"ANSWER: {letter}"`  
**没有**：选项间相对差分 `score(A)-score(B)`，没有 stem-only query 条件化比较。

### 5.5 decode_numeric — 优先级队列

候选 `(priority_score, value, text, path)`：

| 路径 | 优先级 | 机制 |
|------|--------|------|
| `direct_math` | 100 | 正则启发式：乘除、sqrt、简单方程 |
| `word_problem` | 95 | 固定英文模板：half as many, per day, each...total |
| `cortex` | 40 | 对题干数字做 Difference/Ratio/ sum/积 等 Space 变换候选 |
| `memory:*` | mem_score×10 | 从 top-8 memory 文本抽数字 |
| `keyword` | kb×10 | keyword_domain_boost |

取 **priority_score 最大** 的 `format_num(value)` 为答案。

**特点**：大量 **硬编码英文 pattern**；对 fair numeric 子集有效，非通用 CAS。

### 5.6 decode_open

1. `score_memories(exclude question)`
2. 过滤：`keyword_overlap >= min_open_keyword_overlap()`，`score >= min_retrieval_score()`
3. 返回 corpus 行文本或 `best_span` 片段
4. 低分 → `"No confident match..."` / `open_reject_low_score`

**alone ~25%**：检索+关键词，无生成。

---

## 6. MCQ Fate 管线 — 逐步语义

**文件**：`mcq_fate.rs` · 仅 `score_type==mcq" && VALHALLA_MCQ_FATE_CYCLES=1`

### 6.1 Shard 构造

```rust
shards = ["MCQ_STEM {stem}", "MCQ_OPTION {line_A}", ..., "MCQ_OPTION {line_D}"]
// 若无选项: ["MCQ_WHOLE {full question}"]
```

每 shard：`text_to_ports` = **独立 hash_embed**（非整题 embedding）。

### 6.2 Decompose 阶段

对每个 shard：
1. `emit_body_ports(ports)`
2. `run_optimize_cycles(1)`
3. `mark_epoch_if_hub`
4. 更新 `decompose_score`：

```
decompose_score = clamp(
    (parts_fed / n_shards) * 0.6 + min(structural_delta * 0.06, 0.4),
    0, 1
)
structural_delta = max(0, Δtile_count) + max(0, Δstem_count)
```

**Polish 循环**（最多 `soft_max/4`，且 decompose_score < 0.92）：
- 重复 feed 最后一个 shard + optimize
- `decompose_score += 0.04` 直到平台

### 6.3 Aggregate 阶段

```
while cycle < hard_max (128):
    emit_body_ports(whole_ports)      # 整题 ports 再 feed 一次
    run_optimize_cycles(1)
    aggregate_score = compute_aggregate_score(session)
    if fate_should_stop(...): break
```

**compute_aggregate_score**：
```
pref_spread = max(hub_prefs) / sum(hub_prefs)
activation_conc = max(tile_energy) / sum(tile_energy)
stem_cohesion = min(1, avg_connections/8)
return 0.4*pref_spread + 0.35*activation_conc + 0.25*stem_cohesion
```

### 6.4 fate_should_stop 条件（cycle ≥ min_cycles=5）

按序检查：
1. `still_exploring()` → marginal_delta > 0.03 且 stable_streak==0 → **不停止**
2. hub pattern `fixed_point` 且非 exploring → `fate_fixed_point`
3. pattern `converging` + any body converged → `fate_converged`
4. `fate_closure_score > 0` 且 `aggregate_score ≥ floor(stable_streak)` → `fate_converged`
5. `stable_streak ≥ streak_need(decompose_score)` → `fate_marginal_stable`
6. `aggregate_score ≥ 0.88` → `aggregate_ready`

**fate_closure_score**：Hub/Tile/Stem 收敛加权，权重来自 hub_prefs。

### 6.5 与 decode 的关系

Fate 管线修改的是 **Tile/Stem 结构状态**；最终仍走同一 `decode_mcq()`。  
Fate 不直接改选项分数公式，只改 `tile_sigs / stem_sigs / hub_prefs` 输入。

---

## 7. Hybrid 合并语义（关键：评测 vs 运行时）

**文件**：`hybrid_valhalla_lm.py`

### 7.1 路由表（静态，不看内容）

```python
open / open_generation → "patch_lm"
mcq                    → "memory_mcq_best"
numeric                → "hybrid_max_all"
else                   → "memory_triad"
```

`open_generation` = open 题且 reference/keywords **不在** corpus 中（`classify_slices`）。

### 7.2 _pick_best_row 语义

```python
# 1. 按 prefer 顺序，返回第一个 correct==True 的 row
# 2. 否则返回 prefer 顺序第一个存在的 row（即使 wrong）
# 3. 否则任意存在 row
```

| 策略 | prefer 顺序 | 运行时含义 | 评测含义 |
|------|-------------|------------|----------|
| memory_mcq_best | tile_fate, triad | **总是用 tile_fate 答案**（若存在） | 若 tile 错 triad 对，计 triad 对 |
| hybrid_max_all | tile_fate, triad, patch | 需跑三路，**无 oracle 时应用 prefer 第一条** | 任一路对则计对（**numeric 上界型合并**） |
| patch_lm | 单路 | 只用 patch | 同 |

**内部结论**：
- **MCQ hybrid 54%** 在运行时 ≈ tile_fate 准确率（~52%）+ 少量 triad 救回（prefer 只在 tile 错 triad 对时改变 **计分**，不改变 tile 优先输出的产品行为——若产品只展示 prefer 第一条，则展示 tile 答案）。
- **numeric 84.21% (32/38)** 为 deploy 合并结果（`hybrid_v4_mcq_patch_strengthen_final.json`）。若只跑单轨 memory 或无 patch 路由，数字分会低于该值。
- **open 95%** ≈ patch 子集（open 全走 patch），与 merge 逻辑一致。

### 7.3 patch_prompt_ids

仅 `patch_lm` 和 `hybrid_max_all` 路由到的 id 需要跑 Qwen（约 72+numeric 重叠）。

---

## 8. 计分器（Python）

**文件**：`metrics.py`

### 8.1 MCQ strict

```python
extract_mcq_letter: ANSWER: X / 末行单字母
correct = (extracted == ref_letter)  # 无 substring fallback
```

### 8.2 numeric

```python
extract_number = 文本中最后一个数字串
correct = (num == ref) or (ref in num) or (ref.lower() in text.lower())
```

### 8.3 open

```python
correct = ref in text or 半数 keywords in text
```

---

## 9. 端到端 call graph（Hybrid test run）

```
run_hybrid_valhalla_lm.main()
├─ load_suite(phase=test) → 145 prompts, ingest_clean corpus
├─ run_persistent_memory("triad", mcq_fate=False)
│   └─ native_qa_batch_persistent → valhalla_native_qa
│       └─ run_native_qa_batch [§3.4]
├─ run_persistent_memory("tile", mcq_fate=True)
│   └─ VALHALLA_MCQ_FATE_CYCLES=1 → run_mcq_fate_pipeline per mcq [§6]
├─ eval_patch_arm_fresh(triad, patch_prompts)
│   └─ per prompt: load Qwen → apply patch → generate → score_answer
├─ build_hybrid_rows → hybrid_summary
└─ write JSON + markdown
```

---

## 10. Body 维度实测差异（145Q test）

| 轨道 | body | MCQ Fate | 总 acc | MCQ acc |
|------|------|----------|--------|---------|
| triad_persistent | triad | off | 39.31% | 43.48% |
| tile_fate_persistent | tile | on | 41.38% | 52.17% |
| patch subset | triad | N/A | 87.88%* | 0% |

*patch 子集不含 mcq 主路由题。

Hybrid 来源计数：tile_fate **72** · patch **72** · triad **1**  
→ 72 题 open 走 patch；MCQ 46 题几乎全 tile_fate；numeric 在 merge 时三路比。

---

## 11. 已知实现 gap 与实验结论（内部）

### 11.1 已证实有效（可复现）

| 改动 | 效应 |
|------|------|
| persistent vs isolated | MCQ +~18pp |
| whole-question ports | +9pp vs 拆分 ingress（历史实验） |
| body=tile only | MCQ +1–2 vs triad |
| MCQ Fate cycles | +2/46 vs tile alone |
| weak corpus boost | 持平或略稳；保留 |

### 11.2 已回滚 / 无效

| 改动 | 结果 |
|------|------|
| task-tag 路由 / 内部 reroute | 21→21 |
| 三因子 structural scorer | 更差或持平 |
| stem-only query | 21→16 |
| decode v2 / skip_blackhole | 18–22，已 revert |
| patch for MCQ | 26–28%，低于 native |

### 11.3 MCQ 瓶颈（算法层）

1. **独立 argmax**：四选项分数绝对值比较，非 `P(opt|stem)` 相对模型  
2. **hash embedding**：无语义，cosine 近似随机 + corpus boost 凑分  
3. **gold 分布**：46 题中 B 选项 27/46；模型预测近似均匀 → ~50% 天花板理论  
4. **Fate 只改结构输入**：未改比较器形式

### 11.4 numeric 瓶颈

- `direct_math` / `word_problem` 为 **英文模板硬编码**  
- 非模板题依赖 memory 抽数字 → 与 corpus 覆盖相关

### 11.5 open 路径分裂

| 路径 | 机制 | fair open acc |
|------|------|---------------|
| Tier B decode_open | 检索 | ~25% |
| Tier A patch+gen | Qwen generate | ~95% |
| Hybrid | 全 patch | ~95% |

---

## 12. 环境变量完整表（调参面）

### Ingress / Grinder
| 变量 | 默认 | 作用 |
|------|------|------|
| VALHALLA_GRINDER_SIGNALS_PER_PORT | 128 | port 维度 |
| VALHALLA_NATIVE_MAX_CORPUS_LINES | 32 | isolated 模式 ingest 上限 |
| VALHALLA_INGEST_ALL_CORPUS | off | isolated 强制全 ingest |

### MCQ
| 变量 | 默认 | 作用 |
|------|------|------|
| VALHALLA_MCQ_FATE_CYCLES | 0 | 1=启用 Fate 管线 |
| VALHALLA_MCQ_FATE_MAX_CYCLES | 64 | decompose polish 软预算 |
| VALHALLA_MCQ_FATE_HARD_MAX | 128 | 硬顶 |
| VALHALLA_MCQ_FATE_MIN_CYCLES | 5 | aggregate 最少轮 |
| VALHALLA_MCQ_CORPUS_BOOST_HIT | 0.12 | 词命中加分 |
| VALHALLA_MCQ_CORPUS_BOOST_CAP | 0.85 | boost 上限 |
| VALHALLA_MCQ_CORPUS_BOOST_MIN_WORD | 5 | 词长阈值 |
| VALHALLA_MCQ_ENTAILMENT | off | entailment 分支 |

### Open decode
| 变量 | 作用 |
|------|------|
| VALHALLA_OPEN_MIN_SCORE | min_retrieval_score |
| VALHALLA_OPEN_MIN_KW | min keyword overlap |

---

## 13. 源码索引（函数级）

| 功能 | 位置 |
|------|------|
| Batch incubation | `native_qa.rs:run_native_qa_batch` |
| Isolated QA | `native_qa.rs:run_native_qa` |
| MCQ 打分 | `native_qa.rs:decode_mcq` |
| MCQ Fate | `mcq_fate.rs:run_mcq_fate_pipeline` |
| Numeric 规则 | `native_qa.rs:try_direct_math`, `try_word_problem`, `cortex_numeric_candidates` |
| Patch 构建 | `signal_ingress.rs:build_model_patch` |
| Patch 应用 | `apply_and_compare.py:apply_valhalla_patch` |
| Hybrid 路由 | `hybrid_valhalla_lm.py:hybrid_route` |
| Hybrid 合并 | `hybrid_valhalla_lm.py:build_hybrid_rows`, `_pick_best_row` |
| 计分 | `metrics.py:score_answer` |
| Grinder | `grind.py:hash_embed`, `project_port` |
| Slice 分类 | `run_potential_analysis.py:classify_slices` |

---

## 14. 复现检查清单（审计用）

```bash
# 1. 构建
RUSTFLAGS='-L /opt/cuda/lib64' cargo build -p hub-f64 --release --bin valhalla_native_qa

# 2. 全 Hybrid（含 patch，需 GPU + Qwen）
python3 tools/fair_benchmark/run_hybrid_valhalla_lm.py --phase test \
  --reuse-json reports/fair_benchmark/fair_benchmark_test_20260620_0859.json

# 3. 仅 memory 轨（验证 Tier B）
python3 tools/fair_benchmark/run_hybrid_valhalla_lm.py --phase test --skip-patch \
  --reuse-json reports/fair_benchmark/fair_benchmark_test_20260620_0859.json
# → 总 acc 应显著低于 80%（无 open patch）

# 4. 单轨 MCQ Fate
export VALHALLA_MCQ_FATE_CYCLES=1
python3 tools/fair_benchmark/run_mcq_fate_cycle_test.py --body tile
```

验证点：
- [ ] `hybrid_source_counts` 是否 tile_fate≈46+numeric、patch≈open+numeric
- [ ] MCQ 行 `decode_path` 是否 `mcq_fate_cycle`
- [ ] numeric 三路 row 是否都存在再 merge
- [ ] `--skip-patch` 时 open acc 是否跌至 ~25%

---

## 15. 与对外文档的差异（ deliberately omitted in external docs）

| 内部事实 | 对外处理 |
|----------|----------|
| numeric merge 是 best-of-3 oracle | 常简化为「三路取 max」 |
| hash grinder 无语义 | 称「Grinder / 可选 LLM 隐层」 |
| decode_numeric 英文硬编码 | 称「numeric 推理」 |
| MCQ 独立 argmax 天花板 | 称「选项比较器待改进」 |
| Hub quad 路由靠字符串 match | 不展开 |
| isolated 只 ingest 32 行 | 仅提「协议差异」 |

---

*内部技术分解 v1.0 · 非对外分发 · hybrid-2.0 · 2026-06-22*
