# Valhalla — AI Memory & Hybrid Inference Platform

<p align="center">
  <strong>ValhallaBase</strong> · structure-first context · hybrid decode · industry RAG benchmarks<br>
  <strong>结构记忆体</strong> · 混合路由 · 第三方 benchmark 全绿
</p>

<p align="center">
  <a href="#highlights">Highlights</a> ·
  <a href="#english">English</a> ·
  <a href="#中文">中文</a> ·
  <a href="docs/README.md">Docs</a> ·
  <a href="valhalla-unified-core/">Valhalla Unified Core</a>
</p>

**Organization:** [Rogue Intelligence Inc.](https://github.com/Rogue-Intelligence-INC)

Valhalla is an **AI research & product platform**. Context is incubated into a **Triad structure snapshot** (Hub / Tile / StemCell / Fate); a companion LM acts as the **decode head** — not a standalone foundation model.

> **Valhalla is the base. The LM is the decode head.**

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
| **MCQ ladder** (46Q) | **0.5B 65% → 3B 98%** | lm_patch scales with Qwen backbone |

Full interpretation: [`VALID_BENCHMARKS_INTERPRETATION_20260630.md`](reports/valhalla_inference/VALID_BENCHMARKS_INTERPRETATION_20260630.md)

### Native capabilities (ValhallaBase, 2026-06)

| Capability | Result |
|------------|--------|
| **Long memory** (append-then-ask) | **73/76 (96.0%)** |
| **Follow-up native** (30-turn) | **96.7%** |
| **Proactive Fate v2** | **71/72 · PPI 0.890** |
| **Open retrieval** (targeted) | **31/31 (100%)** |
| **Zero cross-topic drift** | **0/48** probes |

### Architecture edge

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
# Build inference binary
RUSTFLAGS='-L /opt/cuda/lib64' cargo build -p hub-f64 --release --bin valhalla_base

# Run industry benchmark suite
HF_ENDPOINT=https://hf-mirror.com .venv-llm/bin/python3 tools/standard_benchmarks/download_standard_benchmarks.py --all --no-wiki-fetch
RUSTFLAGS='-L /opt/cuda/lib64' python3 tools/valhalla_inference/run_industry_benchmarks.py --all
```

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
| [`valhalla-unified-core/docs/VALHALLA_BASE_PLATFORM.md`](valhalla-unified-core/docs/VALHALLA_BASE_PLATFORM.md) | Platform reference |
| [`reports/valhalla_inference/VALID_BENCHMARKS_INTERPRETATION_20260630.md`](reports/valhalla_inference/VALID_BENCHMARKS_INTERPRETATION_20260630.md) | **What to cite · runs · because** |
| [`reports/valhalla_inference/EXPERIMENT_SUMMARY_20260630.md`](reports/valhalla_inference/EXPERIMENT_SUMMARY_20260630.md) | Master experiment summary |

### Honest boundaries (Tier B)

LongBench **35%** · MultiHop **36%** · LaRA RAG **25%** vs LC **70%** — documented with protocol caveats. Do **not** cite FRAMES (no Wikipedia corpus) or LoCoMo 3% (64-line cap) as official scores.

---

<a name="中文"></a>
## 中文

### 是什么

Valhalla 是 **AI 记忆与混合推理平台**：语料 ingest 为 **Triad 结构快照**（Hub / Tile / StemCell / Fate），开放题走 **结构检索 + 引用**，选择题走 **Hybrid → LM logprob**。

### 亮点（对外可引用）

- **RAGTruth 100/100** — ACL 2024 RAG 忠实度 industry 级
- **Needle 4/4 · RULER 20/20** — 会话 append 长程召回
- **Holdout 15/15** — 套件外泛化
- **MCQ 0.5B→3B 65%→98%** — Qwen lm_patch 缩放
- **长久记忆 96% · Proactive 71/72 · PPI 0.890 · 30 轮 follow-up 96.7%**

解读文档：[`VALID_BENCHMARKS_INTERPRETATION_20260630.md`](reports/valhalla_inference/VALID_BENCHMARKS_INTERPRETATION_20260630.md)

### 快速复现

```bash
RUSTFLAGS='-L /opt/cuda/lib64' cargo build -p hub-f64 --release --bin valhalla_base
python3 tools/valhalla_inference/run_industry_benchmarks.py --all
```

产品文档：[`valhalla-unified-core/`](valhalla-unified-core/) · 索引：[`docs/README.md`](docs/README.md)

---

## Requirements

- Rust 1.70+ · CUDA optional (`RUSTFLAGS='-L /opt/cuda/lib64'`)
- Python 3.10+ · `.venv-llm` for trad LM / HF datasets
- Default backbone: `Qwen2.5-0.5B-Instruct`

## License

MIT — see [valhalla-unified-core/NON_OPEN_SOURCE.md](valhalla-unified-core/NON_OPEN_SOURCE.md) for proprietary runtime components.

---

<p align="center">
  <strong>Valhalla</strong> — Structure-first AI memory · Hybrid inference · Rogue Intelligence Inc.
</p>
