# Valhalla — AI Memory & Hybrid Inference Platform

<p align="center">
  <strong>ValhallaBase</strong> · dynamic corpus fragments · Qwen2.5-3B · industry benchmarks<br>
  <strong>结构记忆体</strong> · 全语料动态片段 · 3B 生产默认 · 第三方 benchmark
</p>

<p align="center">
  <a href="#highlights">Highlights</a> ·
  <a href="#english">English</a> ·
  <a href="#中文">中文</a> ·
  <a href="docs/README.md">Docs</a> ·
  <a href="docs/valhalla/DEMO.md">Demo</a> ·
  <a href="https://github.com/Rogue-Intelligence-INC/valhalla-unified-core">Valhalla Unified Core</a>
</p>

**Organization:** [Rogue Intelligence Inc.](https://github.com/Rogue-Intelligence-INC)

Valhalla is an **AI memory & hybrid inference platform** built around **ValhallaBase**: context is **incubated** into a **Triad structure snapshot** (Hub / Tile / StemCell / Fate) with **zero LM weight updates**; a companion LM acts as the **decode head** — not a standalone foundation model.

> **Valhalla is the base. The LM is the decode head.** · **Incubation replaces prompt-window RAG for structure memory.**

**Investor one-liner:** ValhallaBase is a **structure-first AI memory substrate** for grounded retrieval, session learning, proactive routing, and hybrid LM decoding without weight updates.

> **Repo note:** Product docs live in **[valhalla-unified-core](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core)** (git submodule). Links below use the VUC repo so they work on GitHub. Crypto, physics, legacy Wave, and patent trees in this monorepo are **out of scope** for ValhallaBase — see [`docs/README.md`](docs/README.md).

---

<a name="highlights"></a>
## ✦ Highlights / 亮点

### Industry benchmarks (Tier A — cite externally)

| Benchmark | Result | What it proves |
|-----------|--------|----------------|
| **RAGTruth** (ACL 2024) | **100/100** grounded · 0 abstain | RAG faithfulness at industry level |
| **Needle session** | **4/4** @ depth 30 | Session-level long-range recall |
| **RULER-style** | **20/20** single + multi-needle | Deep needle retrieval on append protocol |
| **External holdout** | **15/15** | Out-of-suite generalization |
| **MCQ ladder** (46Q) | **0.5B 63% → 3B 96% → 7B 98%** | lm_patch scales to 3B; F16 GPU verified 2026-07-08 |

**2026-07-08:** Industry 3B suite **11/11** (~29 min) · Proactive agency **3000-round mega** (140s, memory-bounded) · MCQ **GPU F16 + memory-safe** harness ([`BENCHMARK_SUMMARY_20260708.md`](reports/valhalla_inference/BENCHMARK_SUMMARY_20260708.md))

**2026-07-07 full ladder (0.5B/3B/7B):** [`MCQ_LADDER_FULL_20260707.md`](reports/valhalla_inference/MCQ_LADDER_FULL_20260707.md) · **2026-07-08 F16 rerun:** [`MCQ_LADDER_MEM_GPU_20260708.md`](reports/valhalla_inference/MCQ_LADDER_MEM_GPU_20260708.md) · **2026-07-06 industry 3B:** [`INDUSTRY_3B_BENCHMARK_SUMMARY_20260706.md`](reports/valhalla_inference/INDUSTRY_3B_BENCHMARK_SUMMARY_20260706.md)

Full interpretation: [`BENCHMARK_SUMMARY_20260708.md`](reports/valhalla_inference/BENCHMARK_SUMMARY_20260708.md) · [`MCQ_LADDER_FULL_20260707.md`](reports/valhalla_inference/MCQ_LADDER_FULL_20260707.md) · [`INDUSTRY_3B_RUN_STATUS.md`](reports/valhalla_inference/INDUSTRY_3B_RUN_STATUS.md)

### 孵化能力（Incubation — zero-weight learning）

| 指标 | 结果 | 说明 |
|------|------|------|
| **TPI-v2** | **0.755** | 推理期结构学习（无 LM 梯度） |
| **NPPI** | **0.848** | 新范式潜力 · novel **6/6** |
| Transfer vs polluted | **+38.7pp** | Fate snapshot 救援 session 污染 |
| Data efficiency @10% | **100%** | 少量 targeted context 即 ceiling |
| Incubation cycles 1→8 | flat **87.1%** | depth 非当前瓶颈 |

详见 [INCUBATION_CAPABILITY.md](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/INCUBATION_CAPABILITY.md)

### Native capabilities (ValhallaBase, 2026-07)

| Capability | Result |
|------------|--------|
| **Long memory** (append-then-ask) | **72/76 (94.7%)** |
| **Follow-up native** (30-turn) | **96.7%** |
| **Proactive Fate v2** | **71/72 · PPI 0.890** |
| **Open retrieval** (targeted) | **31/31 (100%)** |
| **Zero cross-topic drift** | **0/48** probes |

### Architecture edge (2026-07-06)

| | Legacy prompt RAG | **ValhallaBase v2026-07** |
|--|-------------------|---------------------------|
| Long corpus | Truncate / last-N lines | **Dynamic fragments** — full ingest + query merge |
| Needle @ depth 30 | Fragment filter gap | **Keyword anchor** + scaled top-k → **4/4** |
| Default LM | 0.5B | **Qwen2.5-3B-Instruct + CUDA** |
| Adapt cycles | Fixed count | **Fate convergence** (warmup + adapt) |

| | Transformer prompt stack | **ValhallaBase** |
|--|--------------------------|------------------|
| Context | Tokens in window | Corpus → **structure snapshot** |
| Multi-turn | Resend full history | **Session append** (~64 chars/step) |
| Open QA | Generate from weights | **Native corpus retrieval** + citation |
| MCQ | Same generate path | Hybrid → **LM logprob** |
| Multimodal | Late encoder fusion | **Non-isolated core fusion** in one session |

---

<a name="english"></a>
## English

### Quick start

```bash
git submodule update --init valhalla-unified-core
export LIBRARY_PATH=/opt/cuda/targets/x86_64-linux/lib:$LIBRARY_PATH
RUSTFLAGS='-L /opt/cuda/lib64' cargo build -p hub-f64 --release --bin valhalla_base
./tools/valhalla_inference/valhalla_base_cli.sh open    # or: valhalla_base --example open | valhalla_base --pretty
```

Industry benchmarks (3B, split from AGI):

```bash
python3 tools/valhalla_inference/run_industry_benchmarks.py --all          # regular suite
python3 tools/valhalla_inference/run_industry_benchmarks.py --agi-only     # TPI · fair-bench · incubation
```

Full API: [`docs/valhalla/VALHALLA_BASE_CLI.md`](docs/valhalla/VALHALLA_BASE_CLI.md)

### Repository layout (AI)

```
Valhalla/
├── manifestsys/hub-f64/          # ValhallaBase runtime (native_qa, patch_lm, Fate ingress)
├── valhalla-unified-core/        # Product docs, protocols, public reports (VUC)
├── tools/
│   ├── valhalla_inference/       # Benchmark harness & test protocols
│   ├── standard_benchmarks/      # Third-party bundles + robust download
│   └── fair_benchmark/           # Fair QA suite (145Q history)
├── reports/valhalla_inference/   # JSON reports + interpretation docs
└── docs/                         # Documentation index
```

### Documentation

| Doc | Purpose |
|-----|---------|
| [`docs/README.md`](docs/README.md) | Documentation index |
| [VALHALLA_BASE_PLATFORM.md](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/VALHALLA_BASE_PLATFORM.md) | Platform reference (VUC) |
| [`docs/valhalla/VALHALLA_BASE_CLI.md`](docs/valhalla/VALHALLA_BASE_CLI.md) | **CLI · JSON schema · programmatic API** |
| [`docs/valhalla/DEMO.md`](docs/valhalla/DEMO.md) | **3-minute demo** (corpus → ask → session → proactive) |
| [`docs/valhalla/RELEASE_20260706.md`](docs/valhalla/RELEASE_20260706.md) | **Latest release** — dynamic fragments · 3B · Fate convergence |
| [`docs/valhalla/MANIFOLD_FRAGMENT_V1.md`](docs/valhalla/MANIFOLD_FRAGMENT_V1.md) | Fragment manifold protocol |
| [`docs/valhalla/README.md`](docs/valhalla/README.md) | Product doc hub (fixes submodule 404s) |
| [`reports/valhalla_inference/VALID_BENCHMARKS_INTERPRETATION_20260630.md`](reports/valhalla_inference/VALID_BENCHMARKS_INTERPRETATION_20260630.md) | **What to cite · runs · because** |
| [`reports/valhalla_inference/EXPERIMENT_SUMMARY_20260630.md`](reports/valhalla_inference/EXPERIMENT_SUMMARY_20260630.md) | Master experiment summary |

### Honest boundaries (Tier B — 3B run 2026-07-08)

LongBench **7/20 (35%)** · MultiHop hybrid **14/25 (56%)** · LaRA RAG **8/20** vs LC **13/20** · LoCoMo **1/30 (3%)** · FRAMES **0/25** — documented with protocol caveats. Do **not** cite truncated slices as official production scores.

**Proactive agency (2026-07-08):** Standard 24-turn **Agency Index 0.600** · Crosslang **100%** · 3000-round mega **140s** with segment reset (proactive spread gate still open).

**Positioning:** ValhallaBase is strong on **grounded retrieval, session memory, and MCQ patch scaling** — not a long-chain reasoning replacement today. Next phase: **multi-hop structure composition** (not prompt-window chain-of-thought).

---

<a name="中文"></a>
## 中文

### 是什么

Valhalla 是 **AI 记忆与混合推理平台**：语料经 **Triad 孵化** 为结构快照（Hub / Tile / StemCell / Fate），**不更新 LM 权重** 即可 session 学习、长久记忆与 targeted 检索。

### 孵化能力（2026-07-02）

- **TPI-v2 0.755** · **NPPI 0.848** — 零权重推理期学习
- **Novel 6/6** · 数据效率 @10% **100%**
- **Polluted 48% → Transfer 87%**（+38.7pp）— Fate snapshot 救援
- 文档：[INCUBATION_CAPABILITY.md](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/INCUBATION_CAPABILITY.md) · 演示：[`docs/valhalla/DEMO.md`](docs/valhalla/DEMO.md)

### 亮点（对外可引用）

- **RAGTruth 100/100** — ACL 2024 RAG 忠实度 industry 级
- **Needle 4/4 · RULER 20/20** — 会话 append 长程召回
- **Holdout 15/15** — 套件外泛化
- **MCQ 0.5B→3B→7B 63%→96%→98%** — F16 GPU 2026-07-08 实测；3B 生产默认
- **Industry 3B 2026-07-08** — Tier A 全 100% · LongBench 35% · MultiHop hybrid 56%
- **Proactive agency** — 3000 轮 mega 140s · Agency Index 0.600（24 轮标准）
- **长久记忆 94.7% · Proactive 71/72 · PPI 0.890 · 30 轮 follow-up 96.7%**

解读文档：[`BENCHMARK_SUMMARY_20260708.md`](reports/valhalla_inference/BENCHMARK_SUMMARY_20260708.md) · [`INDUSTRY_3B_RUN_STATUS.md`](reports/valhalla_inference/INDUSTRY_3B_RUN_STATUS.md)

### 快速复现

```bash
export LIBRARY_PATH=/opt/cuda/targets/x86_64-linux/lib:$LIBRARY_PATH
RUSTFLAGS='-L /opt/cuda/lib64' cargo build -p hub-f64 --release --bin valhalla_base
python3 tools/valhalla_inference/run_industry_benchmarks.py --all
```

产品文档：[valhalla-unified-core](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core) · 索引：[`docs/README.md`](docs/README.md) · 演示：[`docs/valhalla/DEMO.md`](docs/valhalla/DEMO.md)

---

## Requirements

- Rust 1.70+ · CUDA optional (`RUSTFLAGS='-L /opt/cuda/lib64'`)
- Python 3.10+ · `.venv-llm` for trad LM / HF datasets
- Default backbone: **`Qwen2.5-3B-Instruct`** · CUDA recommended (`benchmark_env.py`)

## License

MIT — see [valhalla-unified-core/NON_OPEN_SOURCE.md](valhalla-unified-core/NON_OPEN_SOURCE.md) for proprietary runtime components.

---

<p align="center">
  <strong>Valhalla</strong> — Structure-first AI memory · Hybrid inference · Rogue Intelligence Inc.
</p>
