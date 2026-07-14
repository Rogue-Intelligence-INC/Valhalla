# Benchmark strategy — Valhalla (2026-07-14)

Inventory, legitimacy tiers, peer gaps, and runtime policy (GPU/CPU/memory).

## What we already run

| Family | Entry points | Role |
|--------|--------------|------|
| **Unique Proofs U1–U5** | `./scripts/run_unique_proofs.sh` | Structure-native claims (cite U1–U4; U5 cost@floor secondary) |
| **Evidence / independence** | `run_valhalla_independence.py`, `pack_evidence_run.py` | Continuity + isolation controls |
| **Industry suite** | `run_industry_benchmarks.py` | RAGTruth, Needle, FRAMES*, MultiHop, LoCoMo, LaRA, RULER, LongBench-v2, MCQ ladder |
| **Fate longchain** | `test_fate_longchain_unified.py` | theme_drift / retention over multi-phase pollution |
| **Fair / Pareto / competitors** | `tools/fair_benchmark/`, `run_competitor_compare.py` | Cross-backend calibration (not SOTA mem claim) |
| **Legitimacy audit** | `audit_benchmark_legitimacy.py` | Tier A/B/C citeability gate |

\*FRAMES is **not citeable** without Wikipedia extracts (`frames_bundle_preflight`).

## Legitimacy tiers (how to cite)

| Tier | Meaning | Examples |
|------|---------|----------|
| **A — citeable** | Protocol + bundle viable + report keys intact | Unique Proof U1–U4; RAGTruth with n≥10; longchain when drift≤5% and retention≥95% |
| **B — directional** | Fast slice / partial n / calibration LM | LoCoMo/RULER fast; industry 3B ladder |
| **C — do not cite** | Broken bundle, print-only, wrong claim shape | FRAMES without wiki; LoCoMo framed as “beat Mem0 SOTA” |

**Positioning rule:** sell structure substrate / Unique Proofs / drift-retention. Do **not** lead with LoCoMo vs Mem0 SOTA or U5 until cost@floor passes.

Audit command:

```bash
python3 tools/valhalla_inference/audit_benchmark_legitimacy.py
```

## Peers to add (harness roadmap — not claimed done)

From 2025–2026 memory-agent literature:

| Benchmark | Why for Valhalla | Effort |
|-----------|------------------|--------|
| **LongMemEval** (+ cleaned) | Session-incremental chat memory (ICLR 2025) | Adapter over Fate ingress |
| **LongMemEval-V2** | Agentic web/enterprise trajectories + latency frontier | Large haystacks; start Small tier |
| **MemoryAgentBench** | AR / TTL / LRU / conflict resolution (ICLR 2026) | Incremental multi-turn harness |
| **HaluMem / faithfulness overlays** | Memory-side hallucination (pair with RAGTruth) | Metric on existing traces |
| **ProActEval / LatentNeeds** (or FOG peers) | Proactive emit/abstain evaluation | Align with U2 FOG gate |

Keep existing **RAGTruth + Needle + Unique Proof drift** as the investor lead pack until those harnesses land.

## GPU / CPU concurrency policy

| Workload | Policy | Knobs |
|----------|--------|-------|
| **CUDA LM infer** | Outer loop **serial** by default (one locked infer path) | Do **not** set `VALHALLA_BENCH_PARALLEL_CUDA` unless intentional; `VALHALLA_LM_CACHE_MAX=1` |
| **Structure / native item loops** (MultiHop, LoCoMo map) | CPU fan-out OK | `apply_structure_parallel_policy()` → `VALHALLA_BENCH_PARALLEL=4` when CPU-only / `VALHALLA_USE_CPU=1` |
| **Fate longchain phases** | Parallel lchain inside budget | `VALHALLA_FATE_PARALLEL_LCHAIN=1`; ingress % cap |
| **Rayon pools** | **Cached by worker count** | `elastic_resources::par_map_items` — avoids rebuild per batch |

Industry runner now calls `apply_structure_parallel_policy()` after `apply_gpu_env()`.

## Memory optimizations (implemented / recommended)

| Change | Status | Effect |
|--------|--------|--------|
| Cache rayon pools in `par_map_items` | **Done** | Lower allocator / pool-build churn on Multihop/LoCoMo |
| `trim_fate_rollbacks()` between longchain phases A/B/C | **Done** | Cap rollback RSS before next phase |
| Resource budget defaults | **Done** | `VALHALLA_ROLLBACK_DEPTH_CAP=2`, fusion/adapt pulse caps, `LM_CACHE_MAX=1` |
| `release_suite_resources` / `release_resources` | **Done** | Drop LM cache + GC (/torch empty_cache) between suites |
| Fragment `Arc` remesh (reduce clone) | Later | Harder; measure RSS first |
| `VALHALLA_FATE_SPLIT_FRAGMENTS` | Keep **OFF** | Enabling hurt pretrain/holdout in prior runs |

## Recommended run order (legitimacy-first)

```bash
# 1. Citeable structure pack
./scripts/run_unique_proofs.sh --fast --only U1,U2,U3,U4
python3 tools/valhalla_inference/test_fate_longchain_unified.py --production

# 2. Faithfulness + needle
RUSTFLAGS="-L /opt/cuda/lib64" python3 tools/valhalla_inference/run_industry_benchmarks.py --fast

# 3. Gate
python3 tools/valhalla_inference/audit_benchmark_legitimacy.py

# 4. Optional: competitors / fair Pareto (label as calibration)
```

## Interpretation cheat-sheet

- **High shared_patch / FOG abstain / identity continuity** → Unique Proof story.
- **theme_drift≈0, retention≈1** → Fate longchain continuity (even if QA pass is mid).
- **RAGTruth faithful%** → grounding under retrieval pressure.
- **LoCoMo / MultiHop absolute %** → B-tier directional only unless full protocol + peer table with matched LM.
