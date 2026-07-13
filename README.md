# ValhallaBase

**Structure-trainable memory substrate for persistent AI agents.**  
The LM is a **swappable decode head** — not the product.

<p align="center">
  <a href="#prove-it">Evidence pack</a> ·
  <a href="#one-click">One-click test</a> ·
  <a href="#benchmarks">Benchmarks</a> ·
  <a href="#competitor-compare">Competitor compare</a> ·
  <a href="https://github.com/Rogue-Intelligence-INC/valhalla-unified-core">Product docs (VUC)</a>
</p>

```text
session / corpus
    → Triad incubation (Hub · Tile · StemCell · Fate)
    → persistent structure snapshot
    → native / hybrid decode · proactive Fate
    → optional LM head (0.5B default · 3B = calibration only)
```

> **Investor one-liner:** Persistent structure memory + incubation (L1/L2/L3) for private agents — faithfulness and session state without resending full history.  
> **Not claimed:** Transformer replacement · open-domain QA SOTA · AGI.

---

## Prove it (logs · suite · legitimacy)

Every formal run can be packed into an **uploadable Evidence Pack**:

| Artifact | Role |
|----------|------|
| `tools/valhalla_inference/` + `tools/standard_benchmarks/bundled/` | Runnable suite |
| `tools/standard_benchmarks/provenance/` | Upstream citations (LoCoMo · RAGTruth · MultiHop …) |
| `reports/**/*.log` | Timestamped execution logs |
| `audit_benchmark_legitimacy.py` | Citeable vs Tier-C gate |
| `pack_evidence_run.py` | MANIFEST · HASHES · REPRODUCE.md · zip |

```bash
# Install bench deps (Mem0-style local ST · transformers · …)
python3 -m venv .venv-llm && .venv-llm/bin/pip install -r requirements-benchmark.txt

# Full evidence suite: legitimacy → competitor compare → token → pack zip
./scripts/run_evidence_suite.sh              # moderate
./scripts/run_evidence_suite.sh --fast       # smoke
./scripts/run_evidence_suite.sh --full       # + structure ablation
```

Output: `reports/valhalla_inference/evidence_<tag>/` + `.zip`  
Share **REPRODUCE.md + legitimacy_audit.json + HASHES.sha256** first.

Policy: [`docs/valhalla/COMPETITOR_COMPARE.md`](docs/valhalla/COMPETITOR_COMPARE.md) · [`docs/valhalla/FUNDRAISE_ABLATION_PROTOCOL.md`](docs/valhalla/FUNDRAISE_ABLATION_PROTOCOL.md)

---

<a name="one-click"></a>
## One-click smoke

```bash
git clone --recurse-submodules https://github.com/Rogue-Intelligence-INC/Valhalla.git
cd Valhalla
./scripts/run_highlight_test.sh          # pillars ~8–15 min GPU
./scripts/demo_valhalla_chat.sh          # web console
```

---

<a name="benchmarks"></a>
## Benchmarks (honest · locked protocols)

**Calibration LM for industry tables:** `Qwen2.5-3B-Instruct` (comparability only).  
**Product default:** `Qwen2.5-0.5B-Instruct`.

### Industry full suite — 2026-07-13

| Tier | Benchmark | Result | Cite? |
|------|-----------|--------|-------|
| A | RAGTruth-40 / 100 | **34/40 · 88/100** | Yes |
| A | Needle | **4/4** | Yes |
| A | MCQ @ 3B | **45/46** | Yes |
| B | LoCoMo | **17/30 (57%)** | Yes |
| B | MultiHop | **15/25 (60%)** | Yes |
| B | RULER-style | **15/20** | Yes |
| B | LongBench-v2 | **4/20 (20%)** | Boundary |
| B | LaRA | RAG 11/20 · LC 13/20 | Honest tradeoff |
| C | FRAMES | skipped (no wiki) | **No** |

Summaries: [`BENCHMARK_SUMMARY_20260713.md`](reports/valhalla_inference/BENCHMARK_SUMMARY_20260713.md) · [`EXPERIMENT_SUMMARY_20260713.md`](reports/valhalla_inference/EXPERIMENT_SUMMARY_20260713.md)

### Mechanism (secondary — not industry SOTA)

| Metric | Result | Meaning |
|--------|--------|---------|
| TPI-v2 | **0.755** | Zero-weight session structure learning |
| Transfer vs polluted | **+38.7pp** | Snapshot rescue — not “RAG +38pp” |
| NPPI | **0.848** | Synthetic novel ingest (labeled synthetic) |
| Agency / PPI | **0.778 / 0.650** | Fate-gated proactive |

### Reproduce industry

```bash
export RUSTFLAGS="-L /opt/cuda/lib64" LIBRARY_PATH=/opt/cuda/lib64
.venv-llm/bin/python3 tools/valhalla_inference/run_industry_benchmarks.py --all --fresh --verbose
.venv-llm/bin/python3 tools/valhalla_inference/audit_benchmark_legitimacy.py
.venv-llm/bin/python3 tools/valhalla_inference/pack_evidence_run.py --tag industry --zip
```

---

<a name="competitor-compare"></a>
## Competitor compare (same questions · same backbone)

| System | Role |
|--------|------|
| `full_context_lm` | Entire corpus in prompt |
| `vanilla_rag` | Traditional RAG block |
| `mem0` | Mem0 cloud if `OPENAI_API_KEY`; else **local ST top-k + same LM** |
| `valhalla_base` | Structure snapshot runtime |
| Ablation −Fate / −Triad | Causal contribution |

```bash
.venv-llm/bin/pip install -r requirements-benchmark.txt
.venv-llm/bin/python3 tools/valhalla_inference/run_competitor_compare.py --moderate
./tools/valhalla_inference/run_fundraise_ablation.sh --industry-only   # 7-arm ablation
```

We do **not** claim Mem0’s published LongMemEval numbers; we report **same-slice, same-backbone** scores (and SKIP when deps/API missing).

---

## What to evaluate (product surface)

| Path | Role |
|------|------|
| [`manifestsys/`](manifestsys/) | Runtime — `valhalla_base` |
| [`tools/valhalla_inference/`](tools/valhalla_inference/) | Benchmarks · evidence pack |
| [`tools/standard_benchmarks/`](tools/standard_benchmarks/) | Bundled industry slices + provenance |
| [`reports/valhalla_inference/`](reports/valhalla_inference/) | JSON · logs · evidence_* |
| [`apps/valhalla_chat/`](apps/valhalla_chat/) | Web console |
| [`valhalla-unified-core/`](valhalla-unified-core/) | Product docs submodule |

Archive / backups / historical experiments are **not** the product path.

---

## Architecture (vs prompt RAG)

| | Typical RAG / memory API | ValhallaBase |
|--|--------------------------|--------------|
| State | Records / vectors / prompts | **Triad structure snapshot** |
| Multi-turn | Resend or re-retrieve | **Session append + incubate** |
| Learning | Optional fine-tune | **L1 session · L2 Fate pretrain · L3 bake** |
| Proactive | Rare | **Fate-gated** `proactive_idle` |
| Backbone | Often fixed | **Orthogonal** decode head |

Positioning: [VALHALLA_BASE_PRODUCT_POSITIONING.md](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/VALHALLA_BASE_PRODUCT_POSITIONING.md)

---

## Requirements

- Rust 1.70+ · CUDA optional (`RUSTFLAGS='-L /opt/cuda/lib64'`)
- Python 3.10+ · `.venv-llm` + [`requirements-benchmark.txt`](requirements-benchmark.txt)
- Flutter (web demo only)

## License

Runtime components: see [valhalla-unified-core/NON_OPEN_SOURCE.md](valhalla-unified-core/NON_OPEN_SOURCE.md).  
Benchmark harness & reports under this tree follow repository license notices.

---

*Rogue Intelligence LNC · [Organization](https://github.com/Rogue-Intelligence-INC)*
