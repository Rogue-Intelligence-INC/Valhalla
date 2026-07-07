# MCQ 缩放潜力 — 0.5B → 3B → 7B（实测）

**日期:** 2026-07-06 / 2026-07-07  
**协议:** `mcq-ladder-v1` · `valhalla_hybrid_lm_patch`  
**数据:** [`mcq_ladder_v1_full_test.json`](./mcq_ladder_v1_full_test.json) · 全阶梯报告 [`MCQ_LADDER_FULL_20260707.md`](./MCQ_LADDER_FULL_20260707.md)

---

## 核心结论（三点实测）

| 维度 | 0.5B | 3B | 7B |
|------|------|-----|-----|
| **Valhalla hybrid + lm_patch** | 29/46 (**63%**) | 45/46 (**98%**) | 44/46 (**96%**) |
| 仅换 LM，Valhalla 栈不变 | ✓ | ✓ | ✓ |

**0.5B→3B** 跃迁 +35 pp；**3B→7B** 无增益（触顶）。**生产默认保持 3B。**

---

## 分解：Valhalla 增益 vs LM 缩放

### 1. Valhalla 独立贡献（与参数量无关）

在 0.5B 上已观测：

- **trad_lm RAG MCQ** ≈ 15% → **Valhalla hybrid lm_patch** ≈ 63% → **+48 pp**
- 开放检索 holdout、RAGTruth、RULER 等：**不依赖 MCQ logprob ladder**，在 0.5B 上已接近饱和

→ Fate 会话 + hybrid decode + lm_patch 是 **与参数量正交的增益通道**。

### 2. LM 缩放贡献（同栈、换 backbone）

三点拟合（log₁₀ params，serve 模式 2026-07-07）：

```
acc ≈ 0.750 + 0.310 · log₁₀(B)     (R² = 0.865, n=3)
```

| B | 实测 MCQ |
|---|----------|
| 0.5B | 63% |
| 3B | **98%** ← 生产甜点 |
| 7B | 96%（−1 题 vs 3B） |

0.5B→3B 仍占绝大部分缩放收益；7B 外推 ~100% **未兑现** — MCQ logprob 在 3B 已封顶。

### 3. 潜力解读

| 层级 | 含义 |
|------|------|
| **结构潜力** | 0.5B 上 +45–50 pp → 小 LM 也能被 manifold 抬到可用 MCQ |
| **缩放潜力** | 结构固定时，**3B 为 lm_patch MCQ 最优**；更大 LM 无必要 |
| **产品默认** | **Qwen2.5-3B-Instruct** + CUDA + `valhalla_base serve` |

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

## 后续测试策略

1. **Industry:** `run_industry_benchmarks.py --all`（默认 3B + serve）
2. **MCQ 全阶梯:** `tools/valhalla_inference/run_mcq_full_ladder.sh`（0.5B/3B/7B）
3. **AGI / 训练代理:** `--agi-only`（与 industry 分开，避免 10h+ 串联）

```bash
export LIBRARY_PATH=/opt/cuda/targets/x86_64-linux/lib:$LIBRARY_PATH
export RUSTFLAGS="-L /opt/cuda/lib64"
python3 tools/valhalla_inference/run_industry_benchmarks.py --all
tools/valhalla_inference/run_mcq_full_ladder.sh   # optional 7B ladder
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
