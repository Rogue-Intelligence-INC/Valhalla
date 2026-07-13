# ValhallaBase Benchmark Summary — 2026-07-13

**Product:** Backbone-agnostic structure memory · LM = decode head only  
**Industry calibration LM:** `Qwen2.5-3B-Instruct` (third-party suite only)  
**Architecture:** `fate_io` · `open_qa_interface` · `multihop_bench` · `elastic_resources`  
**Reports dir:** `reports/valhalla_inference/`

**Positioning:** [VALHALLA_BASE_PRODUCT_POSITIONING.md](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/VALHALLA_BASE_PRODUCT_POSITIONING.md)

---

## Industry calibration @ 3B — full suite (~26 min)

*3B is benchmark comparability size, **not** the product definition.*

| Tier | Benchmark | Result | Verdict |
|------|-----------|--------|---------|
| **A** | RAGTruth-40 | **34/40 (85%)** | RAGTRUTH_STRONG |
| **A** | RAGTruth-100 | **88/100 (88%)** | RAGTRUTH_STRONG |
| **A** | Needle session | **4/4 (100%)** | NEEDLE_STRONG |
| **A** | MCQ ladder @ 3B | **45/46 (98%)** | MCQ_LADDER_FLAT |
| **B** | External holdout | **10/15 (67%)** | EXTERNAL_HOLDOUT_VIABLE |
| **B** | RULER-style | **15/20 (75%)** | NEEDLE_VIABLE |
| **B** | LoCoMo | **17/30 (57%)** | OPEN_QA_VIABLE |
| **B** | MultiHop v2 | **15/25 (60%)** | OPEN_QA_VIABLE |
| **B** | MultiHop Fate | **15/25 (60%)** | OPEN_QA_VIABLE |
| **B** | MultiHop fatewithv2 | **15/25 (60%)** | OPEN_QA_VIABLE |
| **B** | LongBench-v2 | **4/20 (20%)** | LONGBENCH_V2_DEVELOPING |
| **B** | LaRA | RAG **11/20** · LC **13/20** | STRUCTURE_RAG_BELOW_LC |
| **B** | FRAMES | **0/5 smoke** · wiki enriched (parasail) | OPEN_QA_DEVELOPING (Tier B; full n≥10 pending) |

**Evidence suite `--full` (same day):** competitor + 7-arm ablation + token pack — see [`EVIDENCE_SUITE_SUMMARY_20260713.md`](./EVIDENCE_SUITE_SUMMARY_20260713.md) · pack `evidence_evidence_20260713/`

**Legitimacy:** **11/11 citeable** · FRAMES bundle viable=True

**Elastic runtime:** ingress 50% · max_items 48 · parallel_workers 2 · cuda_ready true

**Docs:** [`EXPERIMENT_SUMMARY_20260713.md`](./EXPERIMENT_SUMMARY_20260713.md) · [`VALHALLA_INTEGRATED_STATUS.md`](./VALHALLA_INTEGRATED_STATUS.md) · [`industry_benchmarks_summary.json`](./industry_benchmarks_summary.json) · [`COMPETITOR_COMPARE_RESULTS.md`](./COMPETITOR_COMPARE_RESULTS.md) · [`FUNDRAISE_ABLATION_MATRIX.md`](./FUNDRAISE_ABLATION_MATRIX.md)

---

## Reproduce

```bash
export RUSTFLAGS="-L /opt/cuda/lib64" LIBRARY_PATH=/opt/cuda/lib64
./scripts/prefetch_frames_wiki.sh
./scripts/run_evidence_suite.sh --full
python3 tools/valhalla_inference/run_industry_benchmarks.py --all --fresh --verbose
python3 tools/valhalla_inference/integrate_valhalla_reports.py
```

---

## Module map (bench vs interface)

| Layer | Rust module | Role |
|-------|-------------|------|
| I/O | `fate_io` | ingress / egress / scoring |
| API | `open_qa_interface` | `OpenQaRequest` · `run_open_qa` |
| Bench | `multihop_bench` | bundle · parallel · JSON reports |
| Resources | `elastic_resources` | CUDA · worker pool · ingress budget |

See [`docs/valhalla/VALHALLA_BASE_CLI.md`](../../docs/valhalla/VALHALLA_BASE_CLI.md) § Architecture.

---

*Rogue Intelligence LNC.*
