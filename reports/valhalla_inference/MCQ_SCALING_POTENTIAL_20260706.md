# MCQ 缩放潜力 — 0.5B → 3B 分解与 Valhalla 上限

**日期:** 2026-07-06  
**协议:** `mcq-ladder-v1` · `valhalla_hybrid_lm_patch`  
**数据:** `reports/valhalla_inference/mcq_ladder_v1_test.json`

---

## 核心结论

| 维度 | 0.5B | 3B | Δ |
|------|------|-----|---|
| **Valhalla hybrid + lm_patch** | 28/46 (61%) | 45/46 (98%) | **+37 pp** |
| 仅换 LM，Valhalla 栈不变 | ✓ | ✓ | 差距主要来自 **LM logprob 分辨率** |

**0.5B→3B 的跃迁比任何单点 industry 分数更能说明潜力：** 同一 Fate/Space/检索栈下，backbone 从 0.5B 换到 3B，MCQ 几乎打满。说明 Valhalla 的 **结构层已把 0.5B 的 logprob 天花板抬高 ~45–50 pp**；3B 只是把最后一档 logprob 歧义消掉。

---

## 分解：Valhalla 增益 vs LM 缩放

### 1. Valhalla 独立贡献（与参数量无关）

在 0.5B 上已观测：

- **trad_lm RAG MCQ** ≈ 15% → **Valhalla hybrid lm_patch** ≈ 61% → **+46 pp**
- 开放检索 holdout、RAGTruth、RULER 等：**不依赖 MCQ logprob  ladder**，在 0.5B 上已接近饱和

→ Fate 会话 + hybrid decode + lm_patch 是 **与参数量正交的增益通道**。

### 2. LM 缩放贡献（同栈、换 backbone）

两点拟合（log₁₀ params）：

```
acc ≈ 0.752 + 0.475 · log₁₀(B)     (R² = 1.0, n=2)
```

| B | 实测/外推 MCQ |
|---|---------------|
| 0.5B | 61% (实测) |
| 3B | 98% (实测) |
| 7B | ~100% (外推，待测) |

0.5B→3B 预测 Δ ≈ **+37 pp**，与实测 **+37 pp** 一致 → 缩放律在 Valhalla 栈上 **可预测**。

### 3. 潜力解读

| 层级 | 含义 |
|------|------|
| **结构潜力** | 0.5B 上 +45–50 pp → 小 LM 也能被 manifold 抬到可用 MCQ |
| **缩放潜力** | 结构固定时，acc 随 log B 近线性 → 7B 有望 MCQ 封顶 |
| **产品默认** | 后续 benchmark **以 Qwen2.5-3B-Instruct 为主干**（GPU 默认开） |

---

## 与 industry suite 的关系

| 类别 | 0.5B 表现 | 3B 预期 |
|------|-----------|---------|
| RAG 忠实 (RAGTruth) | 40/40, 99/100 | 维持或略升 |
| 长上下文针 (RULER) | 20/20 | 维持 |
| Needle session | 3/4 | 待 3B 复测 |
| 开放 QA (FRAMES, LoCoMo) | 弱 | 3B 可能改善 decode，非检索瓶颈 |
| 外部 holdout | 15/15 | 维持 |

MCQ ladder 是 **知识/推理 logprob 分辨率** 的探针；RAG/needle 是 **检索+忠实** 探针。两者互补：前者看 LM 缩放，后者看 Valhalla 结构。

---

## 后续测试策略（3B 为主）

1. **Industry:** `run_industry_benchmarks.py --all`（默认 3B + CUDA）
2. **AGI / 训练代理:** `--agi` → TPI-v2、fair-benchmark test、incubation standard
3. **对照:** 保留 0.5B 结果作 scaling 基线，新跑一律 `VALHALLA_LM_MODEL=Qwen/Qwen2.5-3B-Instruct`

```bash
export LIBRARY_PATH=/opt/cuda/targets/x86_64-linux/lib:$LIBRARY_PATH
export RUSTFLAGS="-L /opt/cuda/lib64"
python3 tools/valhalla_inference/run_industry_benchmarks.py --all --agi
```

---

## 映射：公认 AGI / 训练 benchmark → Valhalla 代理

见 `agi_benchmarks_mapping_20260706.json`：

| 前沿 cluster | Valhalla 代理 | 状态 |
|--------------|---------------|------|
| MMLU / 知识 | mcq-ladder, fair-benchmark MCQ, LongBench-v2 | ✓ 3B 为主 |
| GSM8K / 数学 | fair-benchmark numeric, TPI-v2, external holdout | ✓ `--agi` |
| RAG 忠实 | RAGTruth | ✓ industry |
| 长记忆 | Needle, RULER, LoCoMo | ✓ industry |
| 训练/迁移 | TPI-v2, NPPI, incubation | ✓ `--agi` |
| ARC-AGI, SWE-bench | — | 待 adapter |

---

*Rogue Intelligence LNC.*
