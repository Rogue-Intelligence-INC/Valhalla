# Industry benchmarks — Qwen2.5-3B full suite

**Date:** 2026-07-10  
**Backbone:** `Qwen/Qwen2.5-3B-Instruct` (`qwen_3b`)  
**Command:** `run_industry_benchmarks.py --all --fresh --verbose`

## Fixes applied

| Issue | Fix |
|-------|-----|
| FRAMES 0% (char-split `wiki_links`) | `_normalize_wiki_links()` in `download_standard_benchmarks.py` |
| MCQ broken pipe after LoCoMo | `--memory-safe` + serve restart before mcq-ladder |
| MCQ `models=[]` | `qwen_3b` preset id + `ladder_by_id()` resolution |
| LongBench `--production` crash | Production path via `production_base` + full corpus ingest |
| Suite resume | `--fresh` flag; FRAMES bundle preflight |

## Results (3B, post-fix)

| Benchmark | Result | Verdict |
|-----------|--------|---------|
| Needle session | 4/4 (100%) | NEEDLE_STRONG |
| RAGTruth | 40/40 (100%) | RAGTRUTH_STRONG |
| RAGTruth-100 | 99/100 (99%) | RAGTRUTH_STRONG |
| LongBench-v2 | 7/20 (35%) legacy hybrid · 4/20 production | LONGBENCH_V2_VIABLE |
| FRAMES | 1/25 (4%) | OPEN_QA_DEVELOPING — wiki fetch blocked (no cache) |
| MultiHop hybrid | 14/25 (56%) | OPEN_QA_VIABLE |
| LoCoMo | **14/30 (47%)** ↑ from 1/30 | OPEN_QA_VIABLE |
| RULER | 20/20 (100%) | NEEDLE_STRONG |
| LaRA | RAG 8/20 · LC 13/20 | STRUCTURE_RAG_BELOW_LC |
| External holdout | 15/15 (100%) | EXTERNAL_HOLDOUT_STRONG |
| **MCQ ladder @ 3B** | **45/46 (98%)** | MCQ_LADDER_FLAT (single point) |

## Pending / known gaps

- **FRAMES:** Wikipedia REST fetch times out in this environment; bundle has valid URLs but no extracts. Re-run with network wiki access or pre-populate `tools/standard_benchmarks/cache/wiki/`.
- **LongBench production stack:** targeted/full ingest under per-item isolation scores below legacy hybrid (4/20 vs 7/20). Industry suite uses legacy path for LongBench until Rust MCQ batch slice lands.
- **LoCoMo:** 47% viable; adaptive routing helps vs 128-line truncate-only fallback.

## Re-run failed items

```bash
HF_ENDPOINT=https://hf-mirror.com RUSTFLAGS="-L /opt/cuda/lib64" \
  .venv-llm/bin/python3 tools/valhalla_inference/test_longbench_v2_v1.py

HF_ENDPOINT=https://hf-mirror.com RUSTFLAGS="-L /opt/cuda/lib64" \
  .venv-llm/bin/python3 tools/valhalla_inference/test_mcq_ladder_v1.py --model qwen_3b --memory-safe
```

Full suite:

```bash
VALHALLA_INGRESS_RESOURCE_PERCENT=70 HF_ENDPOINT=https://hf-mirror.com RUSTFLAGS="-L /opt/cuda/lib64" \
  .venv-llm/bin/python3 tools/valhalla_inference/run_industry_benchmarks.py --all --fresh --verbose
```
