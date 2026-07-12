# Industry benchmarks — ValhallaBase

**Generated:** 2026-07-12T23:25:28.969688+00:00

## Results

| Benchmark | Protocol | Result | Verdict |
|-----------|----------|--------|---------|
| RAGTruth | `ragtruth-v1` | **34/40 (85%)** | RAGTRUTH_STRONG |
| RAGTruth-100 | `ragtruth-full-v1` | **88/100 (88%)** | RAGTRUTH_STRONG |
| Needle session | `needle-session-v1` | **4/4 (100%)** | NEEDLE_STRONG |
| LongBench-v2 | `longbench-v2-fragmentwithfate-light-v1` | **4/20 (20%)** | LONGBENCH_V2_DEVELOPING |
| FRAMES | `frames-v1-rust` | **1/25 (4%) · Tier C (no wiki extracts)** | TIER_C_INVALID |
| MultiHop-RAG | `multihop-rag-v1` | **9/25 (36%)** | OPEN_QA_DEVELOPING |
| MultiHop-RAG v2 | `multihop-rag-v2` | **15/25 (60%)** | OPEN_QA_VIABLE |
| MultiHop-RAG Fate | `multihop-rag-fate` | **15/25 (60%)** | OPEN_QA_VIABLE |
| MultiHop-RAG fatewithv2 | `multihop-rag-fatewithv2` | **15/25 (60%)** | OPEN_QA_VIABLE |
| LoCoMo | `locomo-v1-rust` | **17/30 (57%)** | OPEN_QA_VIABLE |
| RULER-style | `ruler-v1` | **15/20 (75%)** | NEEDLE_VIABLE |
| LaRA RAG vs LC | `lara-v1` | **RAG 11/20 · LC 13/20** | STRUCTURE_RAG_BELOW_LC |
| MCQ ladder | `mcq-ladder-v1` | **3.0B 98%** | MCQ_LADDER_FLAT |
| External holdout | `external-holdout-v1` | **10/15 (67%)** | EXTERNAL_HOLDOUT_VIABLE |
| TPI-v2 | `valhalla-base-training-potential-v2` | **TPI 0.755** | ['STRONG_INFERENCE_TIME_LEARNING', 'FATE_TRANSFER_RESCUES_SESSION'] |

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
RUSTFLAGS="-L /opt/cuda/lib64" python3 tools/valhalla_inference/run_industry_benchmarks.py --all --verbose
```

---

*Rogue Intelligence LNC.*
