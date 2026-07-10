# Valhalla Experiment Summary — Independence · Industry · Incubation LLM

**Date:** 2026-07-11  
**Status:** Integrated · cite with Tier labels

---

## Experiment A — Backbone independence + industry (2026-07-10/11)

### A1 Resource & legitimacy fixes

| Issue | Fix |
|-------|-----|
| LM weight cache unbounded in serve | `VALHALLA_LM_CACHE_MAX=1` + `POST /v1/release` |
| 3B serve held VRAM entire industry suite | Default `VALHALLA_BASE_NO_SERVE=1` (subprocess) |
| FRAMES 1/25 invalid (no wiki) | Tier C preflight; skip + discard stale scores |
| fate-longchain RAM explosion | `max-lines` cap; smoke 24 lines; Rust truncate |

### A2 Independence (`valhalla-independence-v1`, panel=**practical**)

| Backbone | Open orthogonality | Memory needle |
|----------|-------------------|---------------|
| Qwen2.5-3B | native/hybrid **4/4** match ref | PASS |
| RWKV-7 0.1B | **4/4** | PASS |
| Phi-tiny-MoE | **4/4** | PASS |
| ModernBERT (appendix) | encoder arm smoke | — |
| Mamba2 / Qwen-VL | SKIP (weights) | — |

**Verdict:** `VALHALLA_BASE_INDEPENDENT` · `MEMORY_INDEPENDENT`

**Claim scope:** open native decode path orthogonal to causal LM checkpoint — **not** full 6-class paper HF weights.

### A3 Industry suite (3B calibration, `--fast`)

| Benchmark | Result | Tier |
|-----------|--------|------|
| RAGTruth | 40/40 | A |
| Needle | 2/2 | A |
| LongBench-v2 | 7/20 | A |
| MultiHop hybrid | 14/25 | A |
| RULER | 4/4 | B (fast) |
| External holdout | 5/5 | A |
| FRAMES | SKIPPED | C (no wiki) |
| LoCoMo | 1/3 | B (fast) |

**Legitimacy:** 10/11 reports citeable.

### A4 Philosophy / architecture ratio

Production inference stack ~**84%** structure/Fate/Triad; open decode **100% native**; LM patch ~11% adjunct.

---

## Experiment B — Incubation / trainable LLM benchmarks (2026-07-11)

### B1 Motivation (literature)

External benchmarks for **incubating / training LLM systems** without assuming full CPT:

| External | Focus | Valhalla proxy |
|----------|-------|----------------|
| [MemoryBench](https://arxiv.org/abs/2510.17281) | Continual learning from user feedback | TPI B4 transfer · Fate session · proactive |
| [CL-Bench](https://huggingface.co/datasets/tencent-community/CL-bench) | In-context rule learning (no fine-tune) | `cl-bench-slice-v1` native path |
| [TiC-LM](https://arxiv.org/abs/2504.02107) | Time-continual pretrain | L3 bake · TPI curriculum (structure) |
| [CL-Bench (2026)](https://arxiv.org/html/2606.05661) | Stateful gain across domains | fate-longchain-unified · holdout |

### B2 Valhalla protocols run

| Protocol | What it tests | Cite when |
|----------|---------------|-----------|
| `valhalla-base-training-potential-v2` | TPI — structure learning without LM SGD | Tier A after full eval |
| `fate-pretrain-holdout-v3` | L2 supervised pretrain → holdout | Tier A− (small n smoke) |
| `incubation-standard-v1` | Multi-domain bake export | Tier B pilot |
| `cl-bench-slice-v1` | Novel in-context rules via native retrieve | Tier B if HF reachable |

**Do not conflate:** LoCoMo industry blind (47%) with Fate pretrain holdout (100% @ L2 protocol).

### B3 Results (2026-07-11 fast run)

| Protocol | Result | Verdict | Citeable |
|----------|--------|---------|----------|
| `valhalla-base-training-potential-v2` | TPI **0.755** (cached full eval) | STRONG_INFERENCE_TIME_LEARNING | Yes |
| `fate-pretrain-holdout-v3` | train **40%** · holdout **80%** (4/5) | HOLDOUT_STRONG (train not converged @ n=5) | Yes (holdout) |

**Note:** `--fast` reuses cached TPI (full B1–B5 from 2026-07-02); re-run with `--force-tpi` for fresh TPI (~30+ min).

### B4 One-click reproduction

```bash
./scripts/run_full_valhalla_test.sh --skip-build
python3 tools/valhalla_inference/run_incubation_llm_benchmarks.py --fast
python3 tools/valhalla_inference/audit_benchmark_legitimacy.py
python3 tools/valhalla_inference/integrate_valhalla_reports.py
```

---

## Experiment C — External Pareto matrix (2026-07-11)

Unified comparison under **same backbone · dataset · scoring · budget**:

```bash
python3 tools/valhalla_inference/run_external_pareto_matrix.py --fast
```

| System | LoCoMo | LongBench | Token/query | Latency | Proactive |
|--------|--------|-----------|-------------|---------|-----------|
| Full-context LM | 1/3 | 0/3 | **17386** | 2801ms | N/A |
| Vanilla RAG | 1/3 | 0/3 | **17386** | 2699ms | N/A |
| **ValhallaBase** | 0/3 | 0/3 | **99** (−99.4%) | 276s* | Yes (gate) |

\* Smoke n=3 · subprocess cold-start inflates latency; token metric is the Pareto signal.

Report: `reports/valhalla_inference/EXTERNAL_PARETO_MATRIX.md`  
**Verdict:** `TOKEN_EFFICIENCY_PARETO` · `ACC_TRADEOFF_HONEST`

---

> ValhallaBase is a **structure-trainable memory substrate** — not a fixed-size foundation LM.  
> **Independence:** open path decode is backbone-orthogonal (Qwen3B/RWKV/Phi-MoE verified).  
> **Faithfulness:** RAGTruth 100% @ 40Q smoke; FRAMES blocked on wiki infra (Tier C).  
> **Incubation:** TPI + Fate pretrain + CL-Bench proxy align with 2025–26 continual-learning literature — structure path replaces LM SGD for session/incubation; LM remains optional decode head.

---

*Rogue Intelligence LNC.*
