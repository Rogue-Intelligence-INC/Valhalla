# Valhalla — AI Memory & Hybrid Inference Platform

<p align="center">
  <strong>ValhallaBase</strong> · backbone-agnostic structure memory · proactive agency · grounded context<br>
  <strong>结构记忆体</strong> · LM 无关 · 主动性 · 忠实检索 · 非固定尺寸 LM 产品
</p>

<p align="center">
  <a href="#one-click-test">One-click test</a> ·
  <a href="#highlights">Highlights</a> ·
  <a href="#english">English</a> ·
  <a href="#中文">中文</a> ·
  <a href="docs/README.md">Docs</a> ·
  <a href="https://github.com/Rogue-Intelligence-INC/valhalla-unified-core">VUC</a>
</p>

---

## Headline numbers

| Pillar | Metric | Verdict |
|--------|--------|---------|
| **Backbone independence** | RWKV · MoE · Dense open **4/4** | `VALHALLA_BASE_INDEPENDENT` |
| **Proactive agency** | Agency **0.778** · PPI **0.650** · MAR **100%** · FIR **0%** | Fate-gated reaction |
| **Anti-hallucination** | RAGTruth **99/100** · theme drift **0%** | Industry faithfulness |
| **Session context** | Needle **4/4** · RULER **20/20** · long mem **94.7%** | Append protocol |
| **Incubation** | TPI **0.755** · transfer **+38.7pp** | Zero LM gradients |

> **Not a 3B LM product.** Runtime default **0.5B**; **3B** = industry benchmark calibration only.

<a name="one-click-test"></a>
## One-click test（任意机器）

Auto-detects **GPU/CPU**, **CUDA paths**, **Python venv**, and runs pillar smoke (~8–15 min GPU):

```bash
git clone --recurse-submodules https://github.com/Rogue-Intelligence-INC/Valhalla.git
cd Valhalla
./scripts/run_highlight_test.sh
```

| Flag | Effect |
|------|--------|
| `--full` | + backbone orthogonality panel + fate-core suite |
| `--cpu-only` | Force CPU (no NVIDIA driver) |
| `--skip-build` | Reuse existing `target/release/valhalla_base` |

```bash
make highlight          # same as script
make highlight-full     # --full
```

**Checks:** Triad smoke · RAGTruth · Needle · Proactive gates · Fate holdout unit  
**Reports:** `reports/valhalla_inference/` · Guide: [`REPRODUCTION.md`](REPRODUCTION.md)

---

Valhalla is an **AI memory & hybrid inference platform** built around **ValhallaBase**: context is **incubated** into a **Triad structure snapshot** (Hub / Tile / StemCell / Fate) with **zero LM weight updates**; a companion LM acts as the **swappable decode head** — not a standalone foundation model.

> **Valhalla is the base. The LM is the decode head.** · **Structure path is backbone-orthogonal.**

**Investor one-liner:** ValhallaBase is **backbone-agnostic structure memory** with **proactive Fate agency**, **industry-grade RAG faithfulness**, and **session-native context** — zero weight updates. **3B is benchmark calibration only, not the product.**

---

**Organization:** [Rogue Intelligence Inc.](https://github.com/Rogue-Intelligence-INC)

**Canonical positioning:** [VALHALLA_BASE_PRODUCT_POSITIONING.md](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/VALHALLA_BASE_PRODUCT_POSITIONING.md)

### 自动反应 / Automatic Reaction

| 能力 | 结果 | 说明 |
|------|------|------|
| **Agency Index** | **0.778** | 四轨 proactive AI |
| **PPI-v2** | **0.650** | Fate 门控自动插话 · mismatch 不乱发 |
| **MAR** | **100%** | 错域会话主动 abstain |
| **FIR** | **0%** | 无幻觉式打断 |
| **Crosslang CCU** | **100%** | EN/ZH/FR 同一 Triad 核心 |

详见 [`PROACTIVE_AGENCY_BENCHMARK.md`](reports/valhalla_inference/PROACTIVE_AGENCY_BENCHMARK.md) · [`VALHALLA_INDEPENDENCE_EXPERIMENT_20260710.md`](reports/valhalla_inference/VALHALLA_INDEPENDENCE_EXPERIMENT_20260710.md)

> **Repo note:** Product docs live in **[valhalla-unified-core](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core)** (git submodule). Crypto, physics, legacy Wave, and patent trees in this monorepo are **out of scope** for ValhallaBase — see [`docs/README.md`](docs/README.md).

---

<a name="highlights"></a>
## ✦ Highlights / 亮点

### Product pillars (what we sell)

| Pillar | Result | What it proves |
|--------|--------|----------------|
| **Backbone independence** | `VALHALLA_BASE_INDEPENDENT` | RWKV · Phi-MoE · Dense — open path 4/4 match |
| **Session memory** | `MEMORY_INDEPENDENT` | Needle hybrid = native @ d=5,10 |
| **Proactive agency** | Agency **0.778** · PPI **0.650** | Fate-gated automatic reaction |
| **Anti-hallucination** | RAGTruth **40/40 · 99/100** | Industry faithfulness · MAR 100% |
| **Context / memory** | Needle **4/4** · RULER **20/20** · long mem **94.7%** | Session append, not prompt window |
| **Incubation** | TPI **0.755** · transfer **+38.7pp** | Zero-weight session learning |

### Industry calibration @ 3B (comparability only — not product default)

| Benchmark | Result | Note |
|-----------|--------|------|
| RAGTruth-100 | **99/100** | Tier A |
| MCQ ladder 46Q | **45/46 (98%)** | lm_patch @ calibration size |
| LoCoMo | **14/30 (47%)** | OPEN_QA_VIABLE |
| LongBench | **7/20 (35%)** | Honest boundary |

Runtime default LM: **Qwen2.5-0.5B** when `VALHALLA_LM_MODEL` unset. Industry suite sets **3B** explicitly.

[`BENCHMARK_SUMMARY_20260708.md`](reports/valhalla_inference/BENCHMARK_SUMMARY_20260708.md) · [`INDUSTRY_3B_RUN_STATUS.md`](reports/valhalla_inference/INDUSTRY_3B_RUN_STATUS.md)

### 孵化能力（Incubation — zero-weight learning）

| 指标 | 结果 | 说明 |
|------|------|------|
| **TPI-v2** | **0.755** | 推理期结构学习（无 LM 梯度） |
| **NPPI** | **0.848** | 新范式潜力 · novel **6/6** |
| Transfer vs polluted | **+38.7pp** | Fate snapshot 救援 session 污染 |

详见 [INCUBATION_CAPABILITY.md](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/INCUBATION_CAPABILITY.md)

### Native capabilities (ValhallaBase, 2026-07)

| Capability | Result |
|------------|--------|
| **Long memory** (append-then-ask) | **72/76 (94.7%)** |
| **Follow-up native** (30-turn) | **96.7%** |
| **Proactive Fate v2** | **68/72 · PPI 0.650** |
| **Open retrieval** (targeted) | **31/31 (100%)** |
| **Zero cross-topic drift** | **0/48** probes |

### Architecture edge

| | Transformer prompt stack | **ValhallaBase** |
|--|--------------------------|------------------|
| Context | Tokens in window | Corpus → **structure snapshot** |
| Backbone | Fixed checkpoint product | **Orthogonal** — RWKV / MoE / Dense |
| Multi-turn | Resend full history | **Session append** (~64 chars/step) |
| Proactive | N/A | **Fate-gated** `proactive_idle` |
| Faithfulness | Generate from weights | **Grounded retrieval** + RAGTruth-strong |

---

<a name="english"></a>
## English

### Quick start

```bash
git submodule update --init valhalla-unified-core
./scripts/run_highlight_test.sh    # one-click pillar test
```

Build + CLI demo:

```bash
export LIBRARY_PATH=/opt/cuda/targets/x86_64-linux/lib:$LIBRARY_PATH
RUSTFLAGS='-L /opt/cuda/lib64' cargo build -p hub-f64 --release --bin valhalla_base
./tools/valhalla_inference/valhalla_base_cli.sh open
```

Industry benchmarks (3B **calibration**):

```bash
python3 tools/valhalla_inference/run_industry_benchmarks.py --all
```

Full API: [`docs/valhalla/VALHALLA_BASE_CLI.md`](docs/valhalla/VALHALLA_BASE_CLI.md)

### Documentation

| Doc | Purpose |
|-----|---------|
| [**Product positioning**](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/VALHALLA_BASE_PRODUCT_POSITIONING.md) | **What ValhallaBase is / is not** |
| [`docs/README.md`](docs/README.md) | Documentation index |
| [VALHALLA_BASE_PLATFORM.md](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/VALHALLA_BASE_PLATFORM.md) | Platform reference (VUC) |
| [`docs/valhalla/DEMO.md`](docs/valhalla/DEMO.md) | 3-minute demo |

### Honest boundaries (Tier B)

LongBench **7/20** · MultiHop **14/25** · FRAMES **1/25** (wiki pending) · LaRA RAG **8/20** — documented with protocol caveats.

**Positioning:** Strong on **independence, proactive agency, faithfulness, session memory** — not a fixed-size LM SKU.

---

<a name="中文"></a>
## 中文

### 是什么

ValhallaBase 是 **LM 无关的结构记忆基座**：语料经 Triad 孵化为结构快照，**不更新 LM 权重**；LM 仅为可替换 decode 头。**3B 仅用于行业 benchmark 标定，不是产品默认尺寸。**

### 一键验证

```bash
./scripts/run_highlight_test.sh
```

### 五大亮点

1. **结构正交** — RWKV / MoE / Dense 同 verdict  
2. **主动性** — Agency 0.778 · PPI 0.65 · MAR 100%  
3. **抗幻觉** — RAGTruth 99/100 · theme drift 0%  
4. **上下文记忆** — Needle/RULER 100% · 长久记忆 94.7%  
5. **零权重孵化** — TPI 0.755

解读：[`BENCHMARK_SUMMARY_20260708.md`](reports/valhalla_inference/BENCHMARK_SUMMARY_20260708.md) · [产品定位](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/VALHALLA_BASE_PRODUCT_POSITIONING.md)

---

## Requirements

- Rust 1.70+ · CUDA optional (`RUSTFLAGS='-L /opt/cuda/lib64'`)
- Python 3.10+ · `.venv-llm` for trad LM / HF datasets
- **Runtime default LM:** `Qwen2.5-0.5B-Instruct` · override via `VALHALLA_LM_MODEL`

## License

MIT — see [valhalla-unified-core/NON_OPEN_SOURCE.md](valhalla-unified-core/NON_OPEN_SOURCE.md) for proprietary runtime components.
