# Industry 3B Benchmark Summary — ValhallaBase

**Date:** 2026-07-06  
**Backbone:** `Qwen/Qwen2.5-3B-Instruct` · CUDA (`VALHALLA_USE_CPU=0`)  
**Stack:** ValhallaBase hybrid · dynamic corpus fragments · Fate convergence · lm_patch MCQ  
**Run:** `run_industry_benchmarks.py --all` (~6.4h, subprocess mode)  
**Artifacts:** `industry_benchmarks_summary.json` · per-protocol `*_test.json`

---

## Executive summary / 一句话

ValhallaBase on **Qwen2.5-3B** reaches **industry-grade grounded RAG** (RAGTruth 100/100), **perfect needle recall** (4/4 + RULER 20/20), **suite-external holdout** (15/15), and **near-ceiling MCQ lm_patch** (**45/46 = 98%**). Open long-chain benchmarks (LongBench, FRAMES, LoCoMo) remain developing; **multi-hop hybrid** is viable at **64%**.

---

## Tier A — cite externally

| Benchmark | Protocol | Result | Verdict |
|-----------|----------|--------|---------|
| **RAGTruth** | ragtruth-v1 | **40/40 (100%)** | RAGTRUTH_STRONG |
| **RAGTruth-100** | ragtruth-full-v1 | **100/100 (100%)** | RAGTRUTH_STRONG |
| **Needle session** | needle-session-v1 | **4/4 (100%)** @ depth 30 | NEEDLE_STRONG |
| **RULER-style** | ruler-v1 | **20/20 (100%)** | NEEDLE_STRONG |
| **External holdout** | external-holdout-v1 | **15/15 (100%)** | EXTERNAL_HOLDOUT_STRONG |
| **MCQ ladder** | mcq-ladder-v1 | **0.5B 63% → 3B 98% → 7B 96%** (29/45/44 of 46) | MCQ_LADDER_SCALES |

Full ladder (serve, 2026-07-07): [`MCQ_LADDER_FULL_20260707.md`](./MCQ_LADDER_FULL_20260707.md)

### What this proves

- **Faithful RAG** at ACL 2024 RAGTruth level — zero abstain on measured slice.
- **Session long-range recall** — dynamic fragments + keyword anchor fix needle @ corpus index 30 (**4/4** without disabling query merge).
- **Generalization** — holdout GSM8K/MMLU-style proxy **15/15** outside fair-1.2 training corpus.
- **Backbone scaling** — same Valhalla stack: **+35pp** 0.5B→3B; **3B optimal** (7B measured 44/46, no gain).

Three-point fit (2026-07-07): `acc ≈ 0.750 + 0.310·log₁₀(B)` (R²=0.865) — see [`MCQ_LADDER_FULL_20260707.md`](./MCQ_LADDER_FULL_20260707.md).

---

## Tier B — measured, protocol caveats

| Benchmark | Result | Notes |
|-----------|--------|-------|
| **LongBench-v2** (short slice) | **4/20 (20%)** | ≤12k char context · max 32 corpus lines · hybrid MCQ |
| **MultiHop-RAG** v1 | **9/25 (36%)** | Baseline hybrid |
| **MultiHop-RAG** v2 | **15/25 (60%)** | Compose path |
| **MultiHop-RAG** Fate | **13/25 (52%)** | Fate adapt |
| **MultiHop-RAG** hybrid | **16/25 (64%)** | fate-v2-hybrid — **best open multi-hop** |
| **LaRA** RAG vs LC | **RAG 12/20 (60%) · LC 13/20 (65%)** | Structure RAG slightly below long-context trad |
| **FRAMES** | **0/25** | No Wikipedia fetch — **do not cite** as official |
| **LoCoMo** | **0/30** | 64-line cap slice — **do not cite** as official |

**Positioning:** Strong on **grounded retrieval, session memory, MCQ patch scaling** — not a CoT long-reasoning replacement today. Next: multi-hop structure composition.

---

## Incubation / AGI proxy (partial run)

| Metric | Result | Status |
|--------|--------|--------|
| **TPI-v2** | **0.755** | Completed in summary aggregate |
| **B2 learning paradigms** | — | AGI `--agi-only` interrupted (serve teardown) — re-run with `run_industry_benchmarks.py --agi-only` |

Verdicts: `STRONG_INFERENCE_TIME_LEARNING` · `FATE_TRANSFER_RESCUES_SESSION`

---

## Configuration (reproduce)

```bash
export LIBRARY_PATH=/opt/cuda/targets/x86_64-linux/lib:$LIBRARY_PATH
export RUSTFLAGS='-L /opt/cuda/lib64'
export VALHALLA_LM_MODEL=Qwen/Qwen2.5-3B-Instruct
export VALHALLA_USE_CPU=0

python3 tools/valhalla_inference/run_industry_benchmarks.py --all
# AGI proxies separately:
python3 tools/valhalla_inference/run_industry_benchmarks.py --agi-only
```

**2026-07-06 infra update:** `benchmark_runtime()` auto-starts `valhalla_base serve` + in-process **LM weight cache** (`patch_lm.rs`) — avoids per-question 3B cold reload. Re-validation log: `industry_3b_serve_20260706.log`.

---

## Honest boundaries

| Do cite | Do not cite |
|---------|-------------|
| RAGTruth 100/100 | FRAMES 0/25 (no wiki corpus) |
| Needle 4/4 · RULER 20/20 | LoCoMo 0/30 (truncated slice) |
| Holdout 15/15 | LongBench 20% without noting truncation |
| MCQ 63%→98% ladder | LaRA as “beats LC” (60% vs 65%) |

Full interpretation: [`VALID_BENCHMARKS_INTERPRETATION_20260630.md`](./VALID_BENCHMARKS_INTERPRETATION_20260630.md)

---

## Related docs

| Doc | Path |
|-----|------|
| Release notes | [`docs/valhalla/RELEASE_20260706.md`](../../docs/valhalla/RELEASE_20260706.md) |
| MCQ scaling analysis | [`MCQ_SCALING_POTENTIAL_20260706.md`](./MCQ_SCALING_POTENTIAL_20260706.md) |
| Manifold fragments | [`docs/valhalla/MANIFOLD_FRAGMENT_V1.md`](../../docs/valhalla/MANIFOLD_FRAGMENT_V1.md) |
| JSON aggregate | [`industry_benchmarks_summary.json`](./industry_benchmarks_summary.json) |

---

*Rogue Intelligence LNC · ValhallaBase `valhalla-base-v1`*
