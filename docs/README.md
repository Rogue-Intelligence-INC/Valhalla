# Valhalla documentation index

**Updated**: 2026-06-30

## Active (cite these)

| Pack | Path | Use |
|------|------|-----|
| **Valhalla Unified Core (VUC)** | [`../valhalla-unified-core/`](../valhalla-unified-core/) | **Canonical** ValhallaBase platform doc, production config, JSON reports |
| **Fair benchmark** | [`../tools/fair_benchmark/`](../tools/fair_benchmark/) | Reproduce hybrid / memory runs |
| **Inference benchmarks** | [`../reports/valhalla_inference/`](../reports/valhalla_inference/) | Full JSON (monorepo mirror; subset synced to VUC `reports/`) |
| **Memory-model paper** | [`../papers/memory_hybrid/`](../papers/memory_hybrid/) | External-facing draft |

**Start here:** [`valhalla-unified-core/docs/VALHALLA_BASE_PLATFORM.md`](../valhalla-unified-core/docs/VALHALLA_BASE_PLATFORM.md)

## Canonical metrics (ValhallaBase, 2026-06)

| Capability | Result | Report |
|------------|--------|--------|
| Open retrieval (31) | **97%** | `scale_benchmark_test.json` |
| MCQ LM logprob (46, text-only) | **63%** | `scale_benchmark_test.json` |
| Core fusion MCQ (5, multimodal) | **40% → 60%** | `context_core_fusion.json` |
| Follow-up native (30-turn) | **96.7%** | `context_flexibility_max_goal.json` |
| **Long memory** (append-then-ask) | **73/76 (96.0%)** | `long_memory_test_full.json` |
| **External holdout** | **15/15 (100%)** | `external_holdout_test.json` |
| **Proactive Fate v2** | **72/72 · PPI 0.921** | `proactive_fate_v2_test.json` |
| Fate quad_cycle ingress P1 | **93.3%** | `fate_weight_ladder_compare_30.json` |

### Third-party & scaling benchmarks (2026-06-30) — **all run complete**

| Benchmark | Result | Report |
|-----------|--------|--------|
| **RAGTruth** (ACL 2024) | **40/40** context-grounded | `ragtruth_v1_test.json` |
| **Needle session** | **4/4** @ depth 30 | `needle_session_v1_test.json` |
| **LongBench-v2** short MCQ | **7/20 (35%)** | `longbench_v2_v1_test.json` |
| **Scaling open** @ 0.5B | **100%** · **+6.5pp** vs trad | `model_scaling_v1_test.json` |
| **Scaling MCQ** @ 0.5B | **65% vs 15%** (+50pp) | `MODEL_SCALING_20260630.md` |
| **MCQ ladder** (Qwen lm_patch) | **0.5B 70% → 3B 100%** | `MCQ_LADDER_20260630.md` |
| **Backbone universality** | Valhalla 87% · trad 100% | `backbone_universality_test.json` |
| **External holdout** | **15/15** | `external_holdout_test.json` |

**Master summary:** [`EXPERIMENT_SUMMARY_20260630.md`](../reports/valhalla_inference/EXPERIMENT_SUMMARY_20260630.md)  
**Runner:** `python3 tools/valhalla_inference/run_recognized_benchmarks.py`

Legacy Hybrid 2.0 fair-145 totals (**80%**) remain valid for **Tier A+B combined** history but are **not** the ValhallaBase product line.

## Removed / do not use

| Path | Reason |
|------|--------|
| ~~`docs/valhalla_vs_transformer/`~~ | **Deleted locally** — stale v1.3 mirror; use [valhalla-transformer](https://github.com/Rogue-Intelligence-INC/valhalla-transformer) repo |
| `hybrid_2.0/` | Historical investor pack (fair-145); superseded by VUC for base/context/fusion narrative |
| `archive/` | Pre–Hybrid 2.0 pitch and experiment write-ups |

## 中文

- **当前产品文档**：`valhalla-unified-core/docs/`
- **实验 JSON**：`reports/valhalla_inference/`（完整）· VUC `reports/`（对外同步子集）
- **勿引用**：已删的 `valhalla_vs_transformer` 镜像、`archive/` 内旧数字
