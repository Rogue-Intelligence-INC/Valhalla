# Valhalla AI — Documentation Index

**Scope:** ValhallaBase inference, memory, benchmarks, and product docs only.  
**Updated:** 2026-07-10

> **Product definition:** ValhallaBase is **backbone-agnostic structure memory** — not a 3B LM product.  
> See [VALHALLA_BASE_PRODUCT_POSITIONING.md](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/VALHALLA_BASE_PRODUCT_POSITIONING.md).

> Non-AI work in this monorepo (cryptography, physics simulation, legacy Wave tooling) is **out of scope** for this index — not maintained here.  
> **Product docs** are in [valhalla-unified-core](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core) (submodule). Use [`valhalla/README.md`](valhalla/README.md) for working links — paths like `../valhalla-unified-core/docs/*.md` **404 on GitHub**.

---

## ✦ Product pillars / 产品亮点

| Pillar | Result | Doc |
|--------|--------|-----|
| **Backbone independence** | `VALHALLA_BASE_INDEPENDENT` | [`VALHALLA_INDEPENDENCE_EXPERIMENT_20260710.md`](../reports/valhalla_inference/VALHALLA_INDEPENDENCE_EXPERIMENT_20260710.md) |
| **Proactive agency** | Agency **0.778** · PPI **0.650** | [`PROACTIVE_AGENCY_BENCHMARK.md`](../reports/valhalla_inference/PROACTIVE_AGENCY_BENCHMARK.md) |
| **Faithfulness** | RAGTruth **99/100** · MAR **100%** | [`VALID_BENCHMARKS_INTERPRETATION_20260630.md`](../reports/valhalla_inference/VALID_BENCHMARKS_INTERPRETATION_20260630.md) |
| **Session context** | Needle **4/4** · long mem **94.7%** | `long_memory_test_full.json` |
| **Incubation** | TPI **0.755** | [INCUBATION_CAPABILITY.md](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/INCUBATION_CAPABILITY.md) |

### Industry calibration @ 3B (comparability — not product default)

| Highlight | Result | Doc |
|-----------|--------|-----|
| MCQ ladder 46Q | **45/46 (98%)** | [`INDUSTRY_3B_RUN_STATUS.md`](../reports/valhalla_inference/INDUSTRY_3B_RUN_STATUS.md) |
| LoCoMo | **14/30 (47%)** | same |

**Runtime default LM:** `Qwen2.5-0.5B-Instruct` (`benchmark_env.py`). Industry suite sets 3B explicitly.

---

## Start here

| Pack | Path | Use |
|------|------|-----|
| **One-click test** | [`scripts/run_highlight_test.sh`](../scripts/run_highlight_test.sh) | Pillar smoke on any Linux host |
| **Product positioning** | [VALHALLA_BASE_PRODUCT_POSITIONING.md](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/VALHALLA_BASE_PRODUCT_POSITIONING.md) | **What ValhallaBase is / is not** |
| **Release 2026-07-06** | [`valhalla/RELEASE_20260706.md`](valhalla/RELEASE_20260706.md) | Dynamic fragments · Fate convergence |
| **3-minute demo** | [`valhalla/DEMO.md`](valhalla/DEMO.md) | Corpus → ask → session → proactive |
| **ValhallaBase CLI & API** | [`valhalla/VALHALLA_BASE_CLI.md`](valhalla/VALHALLA_BASE_CLI.md) | **Protocol header · schemas · complete commands** |
| **Product doc hub** | [`valhalla/README.md`](valhalla/README.md) | VUC links that work on GitHub |
| **孵化能力总结** | [INCUBATION_CAPABILITY.md](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/INCUBATION_CAPABILITY.md) | **Triad 孵化 · TPI/NPPI · 零权重学习** |
| **Valhalla Unified Core (VUC)** | [valhalla-unified-core](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core) | Canonical product doc, protocols, landing page |
| **Investor one-pager** | [INVESTOR_ONE_PAGER.md](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/INVESTOR_ONE_PAGER.md) | Confidential memo |
| **Valid benchmark interpretation** | [`../reports/valhalla_inference/VALID_BENCHMARKS_INTERPRETATION_20260630.md`](../reports/valhalla_inference/VALID_BENCHMARKS_INTERPRETATION_20260630.md) | Runs · meaning · because |
| **Platform reference** | [VALHALLA_BASE_PLATFORM.md](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/VALHALLA_BASE_PLATFORM.md) | Architecture & production config |
| **Fair benchmark** | [`../tools/fair_benchmark/`](../tools/fair_benchmark/) | Reproduce hybrid / memory runs |
| **Industry benchmarks** | [`../tools/standard_benchmarks/`](../tools/standard_benchmarks/) | Download + third-party harness |
| **Inference reports** | [`../reports/valhalla_inference/`](../reports/valhalla_inference/) | Full JSON + summaries |

---

## Benchmarks (2026-06-30)

| Benchmark | Tier | Runs | Result |
|-----------|------|------|--------|
| RAGTruth 40Q | A | ×2 | **40/40** |
| RAGTruth 100Q | A | ×1 | **100/100** |
| Needle session | A | ×2 | **4/4** |
| RULER-style | A | ×1 | **20/20** |
| MCQ ladder | A | ×1 (46Q) | **61% → 98%** (0.5B→3B) |
| External holdout | A | ×1 | **15/15** |
| LongBench-v2 | B | ×1 | 35% (0.5B + trunc) |
| MultiHop-RAG | B | ×1 | 36% |
| LaRA RAG vs LC | B | ×1 | 25% vs 70% |

**Tier B positioning:** strong on **grounded retrieval & session memory**; weak on **multi-hop composition** today — next: structure-level multi-hop, not CoT in prompt window.
| LoCoMo | B† | ×1 | 3% (64-line cap) |
| FRAMES | C | — | **invalid** (no wiki) |

**Master summary:** [`FULL_BENCHMARK_SUMMARY_20260702.md`](../reports/valhalla_inference/FULL_BENCHMARK_SUMMARY_20260702.md) · [`EXPERIMENT_SUMMARY_20260630.md`](../reports/valhalla_inference/EXPERIMENT_SUMMARY_20260630.md) · [`DAILY_SUMMARY_20260701.md`](valhalla/DAILY_SUMMARY_20260701.md)  
**Runner (2026-07-06):**

```bash
export LIBRARY_PATH=/opt/cuda/targets/x86_64-linux/lib:$LIBRARY_PATH
export VALHALLA_LM_MODEL=Qwen/Qwen2.5-3B-Instruct
python3 tools/valhalla_inference/run_industry_benchmarks.py --all          # industry + MCQ ladder
python3 tools/valhalla_inference/run_industry_benchmarks.py --agi-only     # TPI · fair-bench · incubation
```

Defaults: `benchmark_env.py` · see [`docs/valhalla/RELEASE_20260706.md`](../../docs/valhalla/RELEASE_20260706.md)

Legacy Hybrid 2.0 fair-145 (**80%**) is historical — **ValhallaBase** line uses protocols above.

---

## Run

```bash
RUSTFLAGS='-L /opt/cuda/lib64' cargo build -p hub-f64 --release --bin valhalla_base
HF_ENDPOINT=https://hf-mirror.com .venv-llm/bin/python3 tools/standard_benchmarks/download_standard_benchmarks.py --all --no-wiki-fetch
RUSTFLAGS='-L /opt/cuda/lib64' python3 tools/valhalla_inference/run_industry_benchmarks.py --all
```

---

## Out of scope (not in this index)

| Area | Note |
|------|------|
| `valhalla-crypto/` | Yaksha Cipher — separate project |
| `s-wind-sim/` | Physics / symbolic discovery — separate |
| `archive/` · `wave/` | Legacy Wave compiler tooling |
| `docs/hybrid_2.0/` | Historical investor pack; superseded by VUC |

## 中文

- **产品文档：** [valhalla-unified-core](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core) · hub [`valhalla/README.md`](valhalla/README.md)
- **3 分钟演示：** [`valhalla/DEMO.md`](valhalla/DEMO.md)
- **有效 benchmark 解读：** `reports/valhalla_inference/VALID_BENCHMARKS_INTERPRETATION_20260630.md`
- **实验 JSON：** `reports/valhalla_inference/`
