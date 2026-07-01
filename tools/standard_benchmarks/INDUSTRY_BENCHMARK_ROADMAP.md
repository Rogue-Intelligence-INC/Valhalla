# Industry benchmark roadmap — ValhallaBase

**Master runner:** `tools/valhalla_inference/run_industry_benchmarks.py`  
**Download bundles:** `download_standard_benchmarks.py --all` (robust: `download_utils.py`)  
**Valid interpretation:** [`../../reports/valhalla_inference/VALID_BENCHMARKS_INTERPRETATION_20260630.md`](../../reports/valhalla_inference/VALID_BENCHMARKS_INTERPRETATION_20260630.md)

## Implemented (harness + bundled slice)

| Tier | Benchmark | Protocol | Script | Bundle | Literature baseline |
|------|-----------|----------|--------|--------|---------------------|
| A | **RAGTruth** | `ragtruth-v1` | `test_ragtruth_v1.py` | `ragtruth_qa_test40.json` | ACL 2024 faithfulness |
| A | **RAGTruth-100** | `ragtruth-full-v1` | `test_ragtruth_full_v1.py` | `ragtruth_qa_test100.json` | expanded slice |
| A | **Needle session** | `needle-session-v1` | `test_needle_session_v1.py` | synthetic | NIAH family |
| A | **FRAMES** | `frames-v1` | `test_frames_v1.py` | `frames_slice25.json` | naive 40.8% · **needs wiki corpus** |
| A | **MultiHop-RAG** | `multihop-rag-v1` | `test_multihop_rag_v1.py` | `multihop_rag_slice25.json` | COLM 2024 |
| A | **LoCoMo** | `locomo-v1` | `test_locomo_v1.py` | `locomo_qa_slice30.json` | ACL 2024 long memory |
| A | **LaRA-style** | `lara-v1` | `test_lara_v1.py` | `lara_rag_vs_lc_slice20.json` | RAG vs LC routing |
| A | **RULER-style** | `ruler-v1` | `test_ruler_v1.py` | `ruler_v1_slice20.json` | RULER / U-NIAH |
| B | **LongBench-v2** | `longbench-v2-v1` | `test_longbench_v2_v1.py` | `longbench_v2_mcq_short20.json` | human ~53.7% |
| B | **MCQ ladder** | `mcq-ladder-v1` | `test_mcq_ladder_v1.py` | — | Qwen 0.5B→7B |
| B | **Scaling** | `model-scaling-v1` | `test_model_scaling_v1.py` | — | multi-backbone |
| C | **External holdout** | `external-holdout-v1` | `test_external_holdout.py` | internal | generalization |

## Pending (next harness)

| Benchmark | Why | Effort |
|-----------|-----|--------|
| **FRAMES full 824** | NAACL gold standard | wiki fetch ~2h |
| **U-NIAH** | unified RAG vs LC needle | extend `ruler-v2` |
| **RAGBench** | 100K end-to-end pipeline | HF download + adapter |
| **RAGAS** | reference-free prod metrics | trace export |
| **LaRA official** | Alibaba 2326 cases | GitHub clone |
| **Qwen 7B ladder** | complete scaling curve | ~15GB download |
| **MMLU-with-context** | fair RAG variant only | lm-eval harness |

## Quick start

```bash
# 1. Refresh bundled slices (needs .venv-llm + network)
HF_ENDPOINT=https://hf-mirror.com .venv-llm/bin/python3 tools/standard_benchmarks/download_standard_benchmarks.py --all

# 2. Fast smoke all industry tests
RUSTFLAGS="-L /opt/cuda/lib64" python3 tools/valhalla_inference/run_industry_benchmarks.py --fast

# 3. Full run + aggregate
RUSTFLAGS="-L /opt/cuda/lib64" python3 tools/valhalla_inference/run_industry_benchmarks.py --all

# 4. Summary only
python3 tools/valhalla_inference/run_industry_benchmarks.py --aggregate-only
```

Reports: `reports/valhalla_inference/industry_benchmarks_summary.json` · `INDUSTRY_BENCHMARKS_20260630.md`
