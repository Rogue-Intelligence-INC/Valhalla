# Tile / StemCell 多轮 Cycle 优化实验 — 详细报告

**Run ID**: `20260620_1032` (test) · `20260620_1027` (smoke)  
**协议版本**: fair-1.1 · **Artifact**: `3beed48570c4`  
**出品**: Rogue Intelligence LNC. · **日期**: 2026-06-20

---

## 摘要

在 **Fair Benchmark 去污染语料**（49 行 ingest_clean）上，对 **Tile** 与 **StemCell** 两个单 body 分别测试 **1 / 5 / 10 / 20** 轮 `optimize_cycles`，测量结构（patch_hash、块/细胞数）与 QA 准确率是否随轮次提升。

**核心结论**：

| 维度 | Tile | StemCell |
|------|------|----------|
| 结构 patch_hash | **c1≡c20 完全不变** | **每档 cycle 不同**（内部演化） |
| 拓扑规模 | 177 tiles / 0 stem（恒定） | 0 tiles / 277 stem（恒定） |
| QA acc（test 145Q, isolated） | 34.5% → 34.5%（**+0.00 pp**） | 34.5% → 35.2%（**+0.69 pp**，仅 open） |
| 与 triad 对照 | 低于 triad_c1 35.9%、persistent 40.7% | 同左 |

**解释**：Fair 语料在 **第 1 轮即触发 Fate 收敛早停**（Tile）或 **QA 解码路径不敏感于 patch 微变**（StemCell）；多轮 cycle **不是**当前 fair 设定下的主要增益杠杆。Tier A patch 路径（legacy 0.08）仍可达 ~69%，与单 body Tier B 34.5% 的差距来自 **解码层 + triad 融合**，而非 cycle 数不足。

---

## 1. 实验动机

### 1.1 要回答的问题

1. **结构层**：增加 `corpus_cycles` / `incubation_cycles` 是否改变 `patch_hash`、`tile_count`、`stem_cell_count`？
2. **行为层**：结构变化（若有）是否转化为 QA 准确率提升？
3. **分体对比**：Tile-only / Stem-only 多轮优化，能否逼近 **triad** 或 **patch_stemcell_legacy 69%**？

### 1.2 背景假设（待证伪）

早期 Tier A/B 实验（200Q、pilot 12Q）已观察到 **c1≡c5≡c10** 的平台现象。本实验在 **更大 test split（145Q）** 与 **fair 去污染语料** 上系统复验，并分离 Tile / StemCell 两体，避免 triad 融合掩盖单 body 行为。

### 1.3 与相关实验的关系

| 实验 | 语料 | 题量 | 关注点 |
|------|------|------|--------|
| TF-front 累积（06 总结） | medium 56 行 | 12 pilot | Hub/Tile/Stem 分体 + patch 强度 |
| Tile/Stem 高难度（07） | medium 56 行 | 79 | direct_cumulative vs fresh_reload |
| **Fair Benchmark（0859）** | ingest_clean 49 行 | 145 test | memory / patch / TF 三轨 |
| **本实验（1032）** | 同 fair artifact | 145 test | **单 body × 多 cycle** |

---

## 2. 实验设计

### 2.1 语料与 Artifact

- **来源**: `checkpoints/incubation_artifacts/fair_triad_c1/`（manifest id `3beed48570c4`）
- **语料**: 49 行 `ingest_clean` — 去除数值 worked-answer 泄漏，保留领域事实
- **绑定**: 与 Fair Benchmark test run `20260620_0859` 同一 artifact，保证横向可比

### 2.2 评测集

| Phase | 题数 | 用途 |
|-------|------|------|
| smoke | 12 | 快速门禁 |
| **test** | **145** | 主结论 |

题型分布（test）：numeric 38 · mcq 46 · open 61

### 2.3 Cycle 档位

`cycles ∈ {1, 5, 10, 20}` — 覆盖「单轮收敛」到「远超早停阈值」的区间。

### 2.4 三种协议

| 协议 | 实现 | 含义 |
|------|------|------|
| **export** | `valhalla_model_export --experiment` | 仅结构：patch_hash、tile/stem 计数、`cycle_trace` |
| **isolated** | `valhalla_native_qa` 每题独立 session | `corpus_cycles=N`, `qa_cycles=1`；无 cross-question 记忆 |
| **batch** | `native_qa_batch_persistent` | 一次孵化，`incubation_cycles=N`，再答全部题（hash grinder） |

**主报告数值以 test + isolated 为准**；batch 仅在 smoke 跑通（test 未跑 batch，耗时考虑）。

### 2.5 Body 与对照臂

- **实验臂**: `body=tile` · `body=stemcell`
- **Fair 对照**（同语料 test 145Q）:
  - `triad_c1` isolated: **35.9%**
  - `triad_c1_persistent`: **40.7%**
  - `patch_stemcell_legacy` (Tier A): **69.0%**
  - `baseline_no_corpus`: **20.0%**

---

## 3. 实现机制（为何多轮可能无效）

### 3.1 `run_optimize_cycles` 与 Fate 早停

源码 `manifestsys/hub-f64/src/signal_ingress.rs`：

```rust
for _ in 0..cycles {
    if run_tile {
        self.tile.process_tiles_cycle();
        if self.tile.has_fate_converged() {
            break;  // ← 收敛后不再继续
        }
    }
    if run_stem {
        self.stem.process_cells_cycle();
        // evolve ...
        if self.stem.has_fate_converged() {
            break;
        }
    }
}
```

- **Tile**：Fair 49 行语料 feed 后，**第 1 轮** `process_tiles_cycle` 即满足 `has_fate_converged()` → 请求 c5/c10/c20 时 **实际只跑 1 轮有效优化**。
- **StemCell**：Fate 早停条件较松或 evolve 持续改写 patch 向量 → **patch_hash 随 cycle 变化**，但 **细胞数锁定 277**（merge/divide 已达平衡）。

### 3.2 export 与 QA 路径差异

| 路径 | patch 用途 | 对 cycle 敏感度 |
|------|------------|-----------------|
| **export** | 完整 `model_patch` 指纹 | StemCell hash 变；Tile 不变 |
| **Tier B native_qa** | `tile_signatures` / `stem_signatures` + hub_prefs 检索解码 | 签名集合稳定 → acc 平台 |
| **Tier A patch+generate** | `patch_vector` + quad_layer_scales 注入 Qwen | legacy 69% — **不同解码栈** |

因此：**StemCell patch_hash 变化 ≠ QA 可观测变化**；解码层读的是 **离散签名与偏好权重**，不是 patch 向量的连续微扰。

### 3.3 Tile 结构 trace（test export）

| cycles | tile_completed | tile_merges | tile_splits | tile_dim |
|--------|----------------|-------------|-------------|----------|
| 1–20 | **177** | **0** | **0** | **48.95** |

177 块在 c1 已全部闭合，后续 cycle 无 merge/split 事件 → patch_hash 恒定 `2ff181b332274c5f`。

### 3.4 StemCell 结构 trace

| cycles | patch_hash | stem_cells | trace_len |
|--------|------------|------------|-----------|
| 1 | `af0f334aa497541b` | 277 | 1 |
| 5 | `12650c8e7bcbe1e6` | 277 | 5 |
| 10 | `f18e55bfa289c396` | 277 | 10 |
| 20 | `ca5d1f21c36c66f7` | 277 | 20 |

**解释**：`evolve(0.35/0.5)` 在固定拓扑上调整连接/场状态 → patch 向量漂移；但 **细胞数不变**，Tier B 检索的 stem 签名集合基本不变。

---

## 4. 结果

### 4.1 结构层（test · export）

| body | c1 hash | c20 hash | tiles | stem | 判定 |
|------|---------|----------|-------|------|------|
| tile | `2ff181b3…` | **相同** | 177 | 0 | **结构冻结** |
| stemcell | `af0f334a…` | `ca5d1f21…` | 0 | 277 | **patch 演化、规模冻结** |

### 4.2 QA 层（test · isolated · 145Q）

| Arm | acc | numeric | mcq | open |
|-----|-----|---------|-----|------|
| tile_c1 | 34.5% | 57.9% | 28.3% | 24.6% |
| tile_c20 | 34.5% | 57.9% | 28.3% | 24.6% |
| stemcell_c1 | 34.5% | 57.9% | 28.3% | 24.6% |
| stemcell_c20 | **35.2%** | 57.9% | 28.3% | **26.2%** |

**Δ vs c1（bootstrap paired CI）**：

| Arm | Δ acc | 95% CI |
|-----|-------|--------|
| tile_c5/c10/c20 | +0.00 pp | [+0.00, +0.00] |
| stemcell_c10/c20 | +0.69 pp | [+0.00, +2.07] |

StemCell +0.69 pp **全部来自 open**（24.6%→26.2%），numeric/mcq 零变化；CI 含 0，**不能拒绝「无效应」原假设**。

### 4.3 Smoke 对照（12Q · 含 batch）

| 协议 | tile c1→c20 | stem c1→c20 |
|------|-------------|-------------|
| isolated | 75.0% → 75.0% | 75.0% → 75.0% |
| batch | 83.3% → 83.3% | 83.3% → 83.3% |

batch 比 isolated 高约 **8.3 pp**（持久 session / 跨题上下文），但 **batch 内多 cycle 仍无增益** — 与 Fair Benchmark 中 persistent +7.3 pp on corpus_supported 一致：**session 机制 > cycle 数**。

### 4.4 与 Fair Benchmark 对照表

| Arm | Track | acc (145Q) | 相对 tile_c1 |
|-----|-------|------------|--------------|
| baseline_no_corpus | memory | 20.0% | −14.5 pp |
| **tile_c1 / stem_c1** | **Tier B isolated** | **34.5%** | baseline |
| triad_c1 | memory | 35.9% | +1.4 pp |
| triad_c1_persistent | memory | **40.7%** | +6.2 pp |
| patch_tile_legacy | patch Tier A | 68.3% | +33.8 pp |
| patch_stemcell_legacy | patch Tier A | **69.0%** | +34.5 pp |
| generate_baseline (Qwen) | TF | 71.0% | +36.5 pp |

**解读**：

1. 单 body Tier B（34.5%）≈ triad isolated（35.9%）— triad 融合仅 +1.4 pp，**Hub 路由 + 三体签名并集** 贡献有限。
2. **Persistent session**（40.7%）> 任何 cycle 倍增 — 跨题记忆是 Tier B 主杠杆。
3. **Tier A patch**（69%）≈ Transformer（71%）— 差距在 **generate 解码**，不在 incubation cycle 数。

### 4.5 活跃 Tile / Stem 索引（test c20）

- **Tile**：活跃 tile 索引 ~178–192（14 个 unique），stem=0
- **StemCell**：活跃 stem 索引 ~278–293（14 个 unique），tile=0

多轮 cycle **不改变** 活跃签名索引集合（c1 与 c20 列表相同或仅 1 个索引 swap）→ 解码路径稳定。

---

## 5. 讨论

### 5.1 为何 Tile 完全平台？

Fair 语料信息量在 **一次 feed + 一轮 tile 闭合** 内已被 Fate 判定为「边际效应递减」。`has_fate_converged()` 早停使 c5–c20 成为 **空转**（trace_len 记录轮次，但结构指标不变）。

这与 1308 条对话语料上「10 周期未收敛」不矛盾：**语料规模与熵** 决定收敛速度；49 行 clean corpus 属于 **低熵、快收敛**  regime。

### 5.2 为何 StemCell patch 变而 QA 不变？

StemCell 的 `evolve` 在 **固定 277 细胞** 上连续调整场与连接 → `patch_vector` 指纹变化。Tier B `decode_mcq` / `decode_open` 依赖 **stem_signatures 离散集合 + cosine 检索**，细胞 ID 与签名桶不变时，**排序几乎不变**。

c10/c20 上 open +1.6 pp 可能来自 **场状态微扰** 改变 open keyword 检索边界 — 样本量 61 题、CI 宽，宜视为 **探索性信号** 而非稳定增益。

### 5.3 batch vs isolated 的含义

| 模式 | 机制 | smoke acc |
|------|------|-----------|
| isolated | 每题重新孵化 | 75.0% |
| batch | 一次孵化 + 多题 | 83.3% |

**产品含义**：Incubation 产物应 **一次构建、多次查询**（artifact + persistent session），而非 per-question 重复 cycle。

### 5.4 与「多轮优化」产品叙事的对齐

| 叙事 | 本实验支持？ |
|------|-------------|
| 「更多 cycle → 更高 QA」 | **否**（fair corpus, Tier B） |
| 「c1 即收敛 → 省算力」 | **是**（Tile 结构；Stem 规模） |
| 「persistent / artifact 复用 → 更高 QA」 | **是**（+6–8 pp） |
| 「Tier A patch 接近 TF」 | **是**（69% vs 71%，与 cycle 无关） |

---

## 6. 局限与后续

| 局限 | 说明 |
|------|------|
| 语料单一 | 仅 fair ingest_clean 49 行；大语料可能延迟收敛 |
| Tier B only | 未在本实验扫 patch Tier A × multi-cycle |
| test 未跑 batch | 仅 smoke 有 batch 数据 |
| 早停不可关 | 当前 API 无 `force_cycles` 旁路 |

**后续实验方向**（未在本报告执行）：

1. **增量 ingress**：cycle 间插入新语料行，而非空转 optimize
2. **`force_full_cycles` 标志**：禁用 `has_fate_converged` break，测结构上限
3. **Tier A × cycles**：patch_vector 注入 Qwen 是否对 cycle 更敏感
4. **大语料 regime**：medium/large corpus 上复测 c1 vs c20

---

## 7. 复现

```bash
cd /path/to/Valhalla
RUSTFLAGS='-L /opt/cuda/lib64' cargo build -p hub-f64 --release \
  --bin valhalla_model_export --bin valhalla_native_qa

# Smoke（export + isolated + batch）
python3 tools/fair_benchmark/run_tile_stemcell_cycle_experiment.py --phase smoke

# Test 主结论（export + isolated，~4.5 min）
python3 tools/fair_benchmark/run_tile_stemcell_cycle_experiment.py \
  --phase test --protocols export,isolated \
  --artifact checkpoints/incubation_artifacts/fair_triad_c1/

# 同步到 valhalla-transformer
python3 tools/fair_benchmark/sync_tile_stemcell_cycles_to_vt.py
```

**输出**：

- JSON: `reports/tile_stemcell_cycles/tile_stemcell_cycles_test_20260620_1032.json`
- 短报告: `reports/VALHALLA_TILE_STEMCELL_CYCLES_test_20260620_1032.md`
- 本详版: `reports/VALHALLA_TILE_STEMCELL_MULTI_CYCLE_EXPERIMENT_20260620.md`

---

## 8. 结论（一句话）

在 Fair Benchmark 设定下，**Tile 第 1 轮即结构收敛，StemCell patch 虽演化但 QA 平台**；多轮 `optimize_cycles` **不是** Tier B 准确率的主杠杆 — **artifact 持久化 + triad/patch Tier A 解码** 才是接近 Transformer 的路径。

---

*Rogue Intelligence LNC. · Valhalla · 2026-06-20*
