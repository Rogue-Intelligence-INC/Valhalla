# Valhalla Potential Benchmark — Phase B + C 完整报告

**Run**: 20260620_1521 · **VPB v1.0**

---

## 1. 交付物

| 组件 | 路径 |
|------|------|
| VPB 规范 | `tools/fair_benchmark/VALHALLA_POTENTIAL_BENCHMARK_V1.md` |
| Phase A 矩阵 | `reports/VALHALLA_VPB_MATRIX_baseline_20260620_1300.md` |
| **Phase B 记忆题集** | `tools/fair_benchmark/vpb_memory_suite.json` (80Q) |
| **语料规模轴 S/M/L** | `tools/fair_benchmark/vpb_corpus_tiers.json` |
| Phase C 规模实验 test | `reports/VALHALLA_VPB_SCALE_test_20260620_1518.md` |
| **Forecast 终版 PFI** | `reports/VALHALLA_VPB_FORECAST_20260620_1521.md` |

### 工具链

```bash
python3 tools/fair_benchmark/build_vpb_suite.py
python3 tools/fair_benchmark/validate_vpb_suite.py
python3 tools/fair_benchmark/run_vpb_matrix.py --reuse-json reports/fair_benchmark/fair_benchmark_test_20260620_0859.json
python3 tools/fair_benchmark/run_vpb_scale.py --suite both --phase test --tiers S,M,L
python3 tools/fair_benchmark/run_vpb_forecast.py
```

---

## 2. Phase B — 记忆 Taxonomy（80 题）

对齐 LoCoMo / LONGMEMEVAL：

| memory_task | 题数 | 测什么 |
|-------------|------|--------|
| single_hop | 20 | 单行检索 |
| multi_hop | 15 | 多行合成 |
| temporal | 10 | 时序推理 |
| knowledge_update | 10 | 语料更新/双证据 |
| abstention | 15 | 应拒答（语料无答案） |
| adversarial | 10 | 干扰项误导 |

- **smoke**: 16 题（CI 门禁）
- **test**: 64 题
- 每题含 `evidence_line_ids`（LONGMEMEVAL 式）

---

## 3. Phase C — 语料规模轴（Fair L0 · 145Q）

| Tier | 行数 | persistent acc | corpus_supported | recall@3 | ingest_lift |
|------|------|----------------|------------------|----------|-------------|
| **S** | 22 | 36.5% | 47.7% | 75.0% | +27.9 pp |
| **M** | 49 | **40.7%** | 49.0% | 75.0% | **+31.3 pp** |
| **L** | 107 | 40.7% | **50.5%** | 75.0% | +29.1 pp |

**解读**：

- **M→L 全量 acc 平台**（40.7%），但 **corpus_supported +1.5 pp** — 记忆楔子仍有规模 headroom。
- **Recall@3 恒 75%** — 未达 memory_horizon 0.85；规模轴对检索饱和，需 Phase B 证据标注优化或增量 ingress。
- **S 档 acc 更低**（36.5%）— 语料过小损失 MCQ/开放覆盖，符合 LONGMEMEVAL 规模敏感预期。

`scale_headroom_norm`: **0.56**

---

## 4. Phase B 实测 — Memory L1（64 test 题 · persistent）

| Tier | acc | abstention | evidence_recall |
|------|-----|------------|-----------------|
| S | 64.1% | 91.7% | 70.0% |
| **M** | **89.1%** | **91.7%** | **90.0%** |
| L | 89.1% | 91.7% | 90.0% |

**任务级（M tier）**：

- single_hop / multi_hop: **100%**
- temporal: 50%
- knowledge_update: 87.5%
- abstention: **91.7%**（G6 接近通过）
- adversarial: 87.5%

M→L 记忆题集 **无增益** — 80 题证据均在 M 语料覆盖内；L 档 filler 不改变检索。

---

## 5. 综合 PFI（含 scale + velocity）

| 指标 | Phase A | **Forecast 终版** |
|------|---------|-------------------|
| PFI | 0.77 | **0.81** (strong_potential) |
| Hybrid vs TF | +6.2 pp | +6.2 pp |
| scale 分量 | 0.50 占位 | **0.56** 实测 |
| Gates | ✓ | ✓ |

---

## 6. 产品结论（成熟 TF vs 开发中 Valhalla）

| 路径 | 证据 |
|------|------|
| **记忆平台** | ingest +31pp · Memory L1 @ M **89%** · abstention **92%** |
| **Hybrid 产品** | oracle **77.2%** > TF **71%** |
| **规模扩展** | Fair L corpus_supported S→L **+2.8 pp**；全量 acc 平台 |
| **勿堆 cycle** | 与 1032 多轮实验一致 |
| **TF 全替** | open_generation 仍 0% memory；用 patch 层 |

**推荐 MVP**：fair M 语料 artifact → persistent session → hybrid 编排（MCQ/memory 走 Valhalla，open 走 patch/TF）。

---

## 7. 已知局限

1. L tier 107 行（非 120）— 来自去污染后 large 可用行
2. Fair scale 上 recall@3 未随 tier 提升 — persistent 批模式无 top_matches
3. temporal/adversarial 在 L1 test 上 M tier 仅 50%–87% — 可继续加题
4. velocity 仍单 milestone 代理 — 需后续 release 打 M202607xx

---

*Rogue Intelligence LNC. · VPB Phase B+C · 2026-06-20*
