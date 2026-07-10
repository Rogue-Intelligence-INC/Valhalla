# Valhalla — AI Memory & Hybrid Inference Platform

<p align="center">
  <strong>ValhallaBase</strong> · structure-trainable memory · proactive agency · grounded context<br>
  L1/L2/L3 training stack · faithful retrieval · non-fixed-size LM product
</p>

<p align="center">
  <a href="#one-click-test">One-click test</a> ·
  <a href="#highlights">Highlights</a> ·
  <a href="#web-demo">Web demo</a> ·
  <a href="docs/README.md">Docs</a> ·
  <a href="https://github.com/Rogue-Intelligence-INC/valhalla-unified-core">VUC</a>
</p>

---

## Repository layout

**ValhallaBase product surface** — evaluate only these paths:

| Path | Role |
|------|------|
| [`manifestsys/`](manifestsys/) | Runtime — Hub · Tile · StemCell · Fate · `valhalla_base` |
| [`systems/`](systems/) | Triad metasystems |
| [`valhalla-unified-core/`](valhalla-unified-core/) | Product docs (submodule) |
| [`apps/valhalla_chat/`](apps/valhalla_chat/) | Web console |
| [`tools/valhalla_inference/`](tools/valhalla_inference/) | Benchmarks · Pareto matrix |
| [`scripts/run_highlight_test.sh`](scripts/run_highlight_test.sh) | One-click pillar smoke |
| [`scripts/demo_valhalla_chat.sh`](scripts/demo_valhalla_chat.sh) | One-click web demo |
| [`reports/valhalla_inference/`](reports/valhalla_inference/) | Citeable JSON + summaries |

Engineering checkout (NDA): full tree under license — product eval uses paths above only.

---

## Headline numbers

| Pillar | Metric | Verdict |
|--------|--------|---------|
| **Trainability (L1/L2/L3)** | TPI **0.755** · holdout **80%** (4/5 smoke) | `run_incubation_llm_benchmarks.py --fast` |
| **Faithfulness** | RAGTruth **99/100** · theme drift **0%** | Industry faithfulness |
| **Session context** | Needle **4/4** · RULER **20/20** · long mem **94.7%** | Append protocol |
| **Backbone orthogonality** | Qwen3B · RWKV · Phi-MoE open **4/4** · `MEMORY_INDEPENDENT` | `VALHALLA_BASE_INDEPENDENT` |
| **External Pareto matrix** | LoCoMo · LongBench · token · latency · proactive | `run_external_pareto_matrix.py --fast` |
| **Incubation / trainable LLM** | TPI **0.755** · holdout **80%** | `run_incubation_llm_benchmarks.py` |
| **Proactive agency** | Agency **0.778** · PPI **0.650** · MAR **100%** | Fate-gated reaction |

> **Not a 3B LM product.** Runtime default **0.5B**; **3B** = industry benchmark calibration only.

<a name="one-click-test"></a>
## One-click test (any machine)

Auto-detects **GPU/CPU**, **CUDA paths**, **Python venv**, and runs pillar smoke (~8–15 min GPU):

```bash
git clone --recurse-submodules https://github.com/Rogue-Intelligence-INC/Valhalla.git
cd Valhalla
./scripts/run_highlight_test.sh
```

| Flag | Effect |
|------|--------|
| `--train-proof` | + L2 Fate pretrain holdout · L1 TPI transfer · L3 bake artifact |
| `--full` | + backbone practical panel + fate-core smoke + industry aggregate |
| `--cpu-only` | Force CPU (no NVIDIA driver); train-proof skips L1 TPI |
| `--skip-build` | Reuse existing `target/release/valhalla_base` |

```bash
make full-test              # pillars + independence + industry + audit
make highlight-full         # --full pillars only
```

**Incubation / continual-learning LLM benchmarks** (MemoryBench · CL-Bench · TiC-LM proxies):

```bash
python3 tools/valhalla_inference/run_incubation_llm_benchmarks.py --fast
```

**External Pareto matrix** (full-context · vanilla RAG · ValhallaBase — same backbone):

```bash
python3 tools/valhalla_inference/run_external_pareto_matrix.py --fast
make pareto-matrix
```

**Integrated report:** `reports/valhalla_inference/VALHALLA_INTEGRATED_STATUS.md`  
**Experiment summary:** `reports/valhalla_inference/EXPERIMENT_SUMMARY_20260711.md`

**Checks:** Triad smoke · RAGTruth · Needle · Proactive gates · Fate holdout unit  
**Train-proof:** [TRAINABILITY_PROOF.md](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/TRAINABILITY_PROOF.md)  
**Reports:** `reports/valhalla_inference/` · Guide: [`REPRODUCTION.md`](REPRODUCTION.md)

---

Valhalla is an **AI memory & hybrid inference platform** built around **ValhallaBase**: context is **incubated** into a **Triad structure snapshot** (Hub / Tile / StemCell / Fate); structure is **trainable** via session learning (L1), Fate pretrain (L2), and HF bake (L3). A companion LM acts as the **swappable decode head** — not a standalone foundation model.

> **Valhalla is the base. The LM is the decode head.** · **Structure is trainable; LM SGD is optional.**

**Investor one-liner:** ValhallaBase is a **structure-trainable memory substrate** — session learning, Fate pretrain, and HF bake — with **industry-grade RAG faithfulness**, **proactive Fate agency**, and **backbone-orthogonal** decode. **3B is benchmark calibration only, not the product.**

**Organization:** [Rogue Intelligence Inc.](https://github.com/Rogue-Intelligence-INC)

**Canonical positioning:** [VALHALLA_BASE_PRODUCT_POSITIONING.md](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/VALHALLA_BASE_PRODUCT_POSITIONING.md)

> **Product docs:** [valhalla-unified-core](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core) · [`docs/valhalla/README.md`](docs/valhalla/README.md)

---

<a name="web-demo"></a>
## Web demo (interactive console)

One-click: backend + release Flutter web (HTML renderer, no CanvasKit CDN):

```bash
./scripts/demo_valhalla_chat.sh
```

| URL | Purpose |
|-----|---------|
| http://127.0.0.1:8888/ | Chat UI — session warmup · hybrid QA · multimodal ingress |
| http://127.0.0.1:8787/dashboard/ | Triad visualization dashboard |

**What to verify:** connected status · Fate warmup · grounded hybrid answer · multimodal badge on user bubble · protocol log panel.

Dev mode: `./scripts/run_valhalla_chat_web.sh` · release: `./scripts/run_valhalla_chat_web.sh --release`

---

### Automatic reaction (proactive agency)

| Capability | Result | Notes |
|------------|--------|-------|
| **Agency Index** | **0.778** | Four-track proactive AI |
| **PPI-v2** | **0.650** | Fate-gated nudge · no mismatch spam |
| **MAR** | **100%** | Cross-domain sessions abstain |
| **FIR** | **0%** | No hallucinated interruptions |
| **Crosslang CCU** | **100%** | EN/ZH/FR share one Triad core |

See [`PROACTIVE_AGENCY_BENCHMARK.md`](reports/valhalla_inference/PROACTIVE_AGENCY_BENCHMARK.md) · [`VALHALLA_INDEPENDENCE_EXPERIMENT_20260710.md`](reports/valhalla_inference/VALHALLA_INDEPENDENCE_EXPERIMENT_20260710.md)

---

<a name="highlights"></a>
## Highlights

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

### Incubation (zero-weight learning)

| Metric | Result | Notes |
|--------|--------|-------|
| **TPI-v2** | **0.755** | Inference-time structure learning (no LM gradients) |
| **NPPI** | **0.848** | Novel paradigm potential · novel **6/6** |
| Transfer vs polluted | **+38.7pp** | Fate snapshot rescues session pollution |

See [INCUBATION_CAPABILITY.md](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/INCUBATION_CAPABILITY.md)

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

## Quick start

```bash
git submodule update --init valhalla-unified-core
./scripts/run_highlight_test.sh    # one-click pillar test
./scripts/demo_valhalla_chat.sh    # web console demo
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

## Requirements

- Rust 1.70+ · CUDA optional (`RUSTFLAGS='-L /opt/cuda/lib64'`)
- Python 3.10+ · `.venv-llm` for trad LM / HF datasets
- Flutter 3.24+ (web demo only) · `FLUTTER_ROOT` or `~/flutter`
- **Runtime default LM:** `Qwen2.5-0.5B-Instruct` · override via `VALHALLA_LM_MODEL`

## License

MIT — see [valhalla-unified-core/NON_OPEN_SOURCE.md](valhalla-unified-core/NON_OPEN_SOURCE.md) for proprietary runtime components.
