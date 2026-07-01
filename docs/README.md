# Valhalla AI — Documentation Index

**Scope:** ValhallaBase inference, memory, benchmarks, and product docs only.  
**Updated:** 2026-06-30

> Non-AI work in this monorepo (cryptography, physics simulation, legacy Wave tooling) is **out of scope** for this index — not maintained here.

---

## ✦ Highlights / 亮点（对外引用）

### Tier A — Industry & third-party (valid, cite)

| Highlight | Result | Doc |
|-----------|--------|-----|
| **RAGTruth** faithful RAG | **100/100** · 0 abstain | [`VALID_BENCHMARKS_INTERPRETATION_20260630.md`](../reports/valhalla_inference/VALID_BENCHMARKS_INTERPRETATION_20260630.md) |
| **Needle + RULER** session recall | **4/4 · 20/20** | same |
| **External holdout** | **15/15** | `external_holdout_test.json` |
| **MCQ ladder** Qwen scaling | **65% → 98%** (46Q) | [`MCQ_LADDER_20260630.md`](../reports/valhalla_inference/MCQ_LADDER_20260630.md) |

### Native ValhallaBase

| Highlight | Result | Report |
|-----------|--------|--------|
| **Long memory** append-then-ask | **96.0%** | `long_memory_test_full.json` |
| **Follow-up native** 30-turn | **96.7%** | `context_flexibility_max_goal.json` |
| **Proactive Fate v2** | **72/72 · PPI 0.921** | `proactive_fate_v2_test.json` |
| **Open retrieval** targeted | **31/31** | `model_scaling_v1_test.json` |
| **Zero cross-topic drift** | **0/48** | `context_topic_drift_test.json` |

**One-line (Tier A only):** RAGTruth 100/100 · Needle 4/4 · RULER 20/20 · Holdout 15/15 · MCQ 65%→98%

---

## Start here

| Pack | Path | Use |
|------|------|-----|
| **Valhalla Unified Core (VUC)** | [`../valhalla-unified-core/`](../valhalla-unified-core/) | Canonical product doc, protocols, landing page |
| **Valid benchmark interpretation** | [`../reports/valhalla_inference/VALID_BENCHMARKS_INTERPRETATION_20260630.md`](../reports/valhalla_inference/VALID_BENCHMARKS_INTERPRETATION_20260630.md) | Runs · meaning · because |
| **Platform reference** | [`../valhalla-unified-core/docs/VALHALLA_BASE_PLATFORM.md`](../valhalla-unified-core/docs/VALHALLA_BASE_PLATFORM.md) | Architecture & production config |
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
| MCQ ladder | A | ×1 (46Q) | **65% → 98%** |
| External holdout | A | ×1 | **15/15** |
| LongBench-v2 | B | ×1 | 35% (0.5B + trunc) |
| MultiHop-RAG | B | ×1 | 36% |
| LaRA RAG vs LC | B | ×1 | 25% vs 70% |
| LoCoMo | B† | ×1 | 3% (64-line cap) |
| FRAMES | C | — | **invalid** (no wiki) |

**Master summary:** [`EXPERIMENT_SUMMARY_20260630.md`](../reports/valhalla_inference/EXPERIMENT_SUMMARY_20260630.md)  
**Runner:** `python3 tools/valhalla_inference/run_industry_benchmarks.py`

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

- **产品文档：** `valhalla-unified-core/docs/`
- **有效 benchmark 解读：** `reports/valhalla_inference/VALID_BENCHMARKS_INTERPRETATION_20260630.md`
- **实验 JSON：** `reports/valhalla_inference/`
