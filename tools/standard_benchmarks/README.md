# Standard third-party benchmarks

Recognized external benchmarks adapted for **ValhallaBase** (corpus ingest + hybrid decode).

| Benchmark | Script | Bundled data | Metric |
|-----------|--------|--------------|--------|
| **RAGTruth** (ACL 2024) | `test_ragtruth_v1.py` | `bundled/ragtruth_qa_test40.json` | faithful_rate, abstain_rate |
| **RAGTruth-100** | `test_ragtruth_full_v1.py` | `bundled/ragtruth_qa_test100.json` | faithful_rate |
| **Needle-in-haystack** (session) | `test_needle_session_v1.py` | generated | recall @ depth |
| **LongBench-v2** (THUDM) | `test_longbench_v2_v1.py` | `bundled/longbench_v2_mcq_short20.json` | MCQ accuracy |
| **FRAMES** (Google NAACL 2025) | `test_frames_v1.py` | `bundled/frames_slice25.json` | open QA accuracy |
| **MultiHop-RAG** | `test_multihop_rag_v1.py` | `bundled/multihop_rag_slice25.json` | multi-hop QA |
| **LoCoMo** | `test_locomo_v1.py` | `bundled/locomo_qa_slice30.json` | long dialogue QA |
| **LaRA-style** | `test_lara_v1.py` | `bundled/lara_rag_vs_lc_slice20.json` | RAG vs LC |
| **RULER-style** | `test_ruler_v1.py` | `bundled/ruler_v1_slice20.json` | needle recall |

## Run

```bash
# all recognized benchmarks (third-party + scaling + backbone + holdout)
python3 tools/valhalla_inference/run_recognized_benchmarks.py

# full industry suite (FRAMES, LoCoMo, MultiHop, LaRA, RULER, …)
python3 tools/valhalla_inference/run_industry_benchmarks.py --download-first --all

# third-party only (RAGTruth + Needle + LongBench)
python3 tools/valhalla_inference/run_standard_benchmarks.py

# quick smoke
python3 tools/valhalla_inference/run_standard_benchmarks.py --fast --limit 5

# multi-backbone scaling (0.5B / 2B / 3.8B)
python3 tools/valhalla_inference/test_model_scaling_v1.py --open-only --skip-training
```

## Model scaling ladder

| Model | Params | Role |
|-------|--------|------|
| Qwen2.5-0.5B-Instruct | 0.5B | Default · Rust lm_patch |
| google/gemma-2-2b-it | 2B | trad_lm scaling |
| microsoft/Phi-3-mini-4k-instruct | 3.8B | trad_lm scaling |

Results: [`MODEL_SCALING_20260630.md`](../../reports/valhalla_inference/MODEL_SCALING_20260630.md)

## Download bundles

```bash
# refresh bundled slices (needs .venv-llm + datasets)
.venv-llm/bin/python3 tools/standard_benchmarks/download_standard_benchmarks.py --all

# robust fetch: curl + GitHub mirrors + HF mirror + disk cache
# cache dir: tools/standard_benchmarks/cache/
HF_ENDPOINT=https://hf-mirror.com .venv-llm/bin/python3 tools/standard_benchmarks/download_standard_benchmarks.py --all

# offline: reuse HF cache + local github/wiki cache only
.venv-llm/bin/python3 tools/standard_benchmarks/download_standard_benchmarks.py --locomo-qa 30 --offline
```

Reports: `reports/valhalla_inference/{ragtruth,needle_session,longbench_v2}_v1_test.json`

**Valid interpretation:** [`VALID_BENCHMARKS_INTERPRETATION_20260630.md`](../../reports/valhalla_inference/VALID_BENCHMARKS_INTERPRETATION_20260630.md) — Tier A/B/C · runs · meaning · because

## Download reliability

| Source | Strategy |
|--------|----------|
| **LoCoMo** | `curl` → GitHub raw → ghproxy → jsDelivr; cached under `cache/github/` |
| **HuggingFace** | `HF_ENDPOINT` default `hf-mirror.com`; fallback to huggingface.co |
| **Wikipedia (FRAMES)** | Disk cache `cache/wiki/`; curl + REST + MediaWiki API |
| **Offline** | `--offline` reuses HF dataset cache + local files |

See `download_utils.py` · `download_standard_benchmarks.py --offline`

## Notes

- **RAGTruth**: QA subset from `wandb/RAGTruth-processed` test split; faithfulness = context support + reference overlap.
- **Needle**: Paul Graham-style recall via session `append_context`, not raw 128K transformer window.
- **LongBench-v2**: `length=short` slice; context truncated to 12k chars / 32 corpus lines for Valhalla session limits. Full-context eval requires raising `VALHALLA_NATIVE_MAX_CORPUS_LINES` and chunked append (future protocol).

Human baselines (literature): LongBench-v2 overall ~53.7%; our slice is short-length MCQ only — compare cautiously.

## Results (2026-07-10 full industry run @ Qwen2.5-3B)

| Benchmark | Tier | Result | Verdict |
|-----------|------|--------|---------|
| RAGTruth 40Q / 100Q | A | **40/40 · 99/100** | RAGTRUTH_STRONG |
| Needle + RULER | A | **4/4 · 20/20** | NEEDLE_STRONG |
| MCQ ladder 46Q @ 3B | A | **45/46 (98%)** | MCQ_LADDER_FLAT |
| External holdout | A | **15/15** | EXTERNAL_HOLDOUT_STRONG |
| LoCoMo | B | **14/30 (47%)** | OPEN_QA_VIABLE |
| MultiHop hybrid / v2 | B | **14/25 · 15/25** | OPEN_QA_VIABLE |
| LongBench-v2 | B | **7/20 (35%)** | LONGBENCH_V2_VIABLE |
| LaRA | B | RAG 8/20 · LC 13/20 | STRUCTURE_RAG_BELOW_LC |
| FRAMES | C | **1/25 (4%)** | OPEN_QA_DEVELOPING — needs wiki corpus |

Run: `run_industry_benchmarks.py --all --fresh --verbose` (~22 min)

**Interpretation:** [`VALID_BENCHMARKS_INTERPRETATION_20260630.md`](../../reports/valhalla_inference/VALID_BENCHMARKS_INTERPRETATION_20260630.md)  
Full write-up: [`INDUSTRY_BENCHMARKS_20260630.md`](../../reports/valhalla_inference/INDUSTRY_BENCHMARKS_20260630.md) · [`INDUSTRY_3B_RUN_STATUS.md`](../../reports/valhalla_inference/INDUSTRY_3B_RUN_STATUS.md)

## Results (2026-06-30 full industry run)

| Benchmark | Tier | Result | Verdict |
|-----------|------|--------|---------|
| RAGTruth 40Q (×2) / 100Q | A | **40/40 · 100/100** | RAGTRUTH_STRONG |
| Needle (×2) + RULER | A | **4/4 · 20/20** | NEEDLE_STRONG |
| MCQ ladder 46Q | A | **65%→98%** | MCQ_LADDER_SCALES |
| External holdout | A | **15/15** | EXTERNAL_HOLDOUT_STRONG |
| MultiHop / LongBench / LaRA / LoCoMo | B | partial | OPEN_QA_DEVELOPING |
| FRAMES | C | invalid | FRAMES_NEEDS_WIKI |

**Interpretation:** [`VALID_BENCHMARKS_INTERPRETATION_20260630.md`](../../reports/valhalla_inference/VALID_BENCHMARKS_INTERPRETATION_20260630.md)  
Full write-up: [`INDUSTRY_BENCHMARKS_20260630.md`](../../reports/valhalla_inference/INDUSTRY_BENCHMARKS_20260630.md)
