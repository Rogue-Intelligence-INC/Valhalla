# Industry benchmarks — ValhallaBase

**Generated:** 2026-07-02T08:04:31.719562+00:00

## Results

| Benchmark | Protocol | Result | Verdict |
|-----------|----------|--------|---------|
| RAGTruth | `ragtruth-v1` | **40/40 (100%)** | RAGTRUTH_STRONG |
| RAGTruth-100 | `ragtruth-full-v1` | **100/100 (100%)** | RAGTRUTH_STRONG |
| Needle session | `needle-session-v1` | **4/4 (100%)** | NEEDLE_STRONG |
| LongBench-v2 | `longbench-v2-v1` | **7/20 (35%)** | LONGBENCH_V2_VIABLE |
| FRAMES | `frames-v1` | **0/25 (0%)** | OPEN_QA_DEVELOPING |
| MultiHop-RAG | `multihop-rag-v1` | **9/25 (36%)** | OPEN_QA_DEVELOPING |
| LoCoMo | `locomo-v1` | **1/30 (3%)** | OPEN_QA_DEVELOPING |
| RULER-style | `ruler-v1` | **20/20 (100%)** | NEEDLE_STRONG |
| LaRA RAG vs LC | `lara-v1` | **RAG 5/20 · LC 14/20** | STRUCTURE_RAG_BELOW_LC |
| MCQ ladder | `mcq-ladder-v1` | **0.5B 65% → 3.0B 98%** | MCQ_LADDER_SCALES |
| External holdout | `external-holdout-v1` | **15/15 (100%)** | EXTERNAL_HOLDOUT_STRONG |

## Roadmap (pending)

- U-NIAH multi-needle (extend ruler-v2)
- RAGBench end-to-end (galileo-ai/ragbench)
- FRAMES full 824Q + wiki fetch
- LoCoMo full 10 conversations all QA
- Qwen 7B MCQ ladder measured
- RAGAS reference-free on production traces

## Run

```bash
HF_ENDPOINT=https://hf-mirror.com .venv-llm/bin/python3 tools/standard_benchmarks/download_standard_benchmarks.py --all
RUSTFLAGS="-L /opt/cuda/lib64" python3 tools/valhalla_inference/run_industry_benchmarks.py
```

---

*Rogue Intelligence LNC.*
