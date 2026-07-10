# Valhalla Inference Loader

Unified loading layer for Valhalla modules ↔ Transformer (vLLM-style API + **Valhalla-as-base**).

**2026-07-07 GPU acceleration (patch_lm):** MCQ logprob **prefill + suffix decode** (908→~16 forwards/query) · `PatchTensorScratch` weight reuse · generate first-step prefill. Measured 0.5B MCQ **~55s → ~3.7s/query** (serve + CUDA). Optional `VALHALLA_LM_F16=1` for half-precision weights.

```bash
# Industry (standard suite)
python3 tools/valhalla_inference/run_industry_benchmarks.py --all
# MCQ full ladder 0.5B / 3B / 7B
tools/valhalla_inference/run_mcq_full_ladder.sh
# Proactive agency (long dialogue + multimodal + cross-lingual)
tools/valhalla_inference/run_proactive_agency_benchmark.sh --verbose
tools/valhalla_inference/run_proactive_agency_benchmark.sh --mega --turns 3000
# AGI / training proxies (separate)
python3 tools/valhalla_inference/run_industry_benchmarks.py --agi-only
# Live demo question validation
python3 scripts/run_demo_showcase.py
```

## Valhalla-as-base (context-flexible substrate)

**Production stack (built-in defaults):** `hybrid` · `targeted` ingest · session affinity reanchor · `fate_qa_feedback` · `native_follow_up_aware`

```python
from valhalla_base import ValhallaBase

base = ValhallaBase()  # defaults = production stack
base.ingest_for_prompt(prompt_dict, corpus)  # targeted + reanchor
base.run([question], append_only=True)
# or batch:
base.run_batch(questions, corpus)
```

**Rust-first:** in-process `ValhallaBaseClient` · `valhalla_base demo v1|needle|corpus` · `valhalla_base serve`

Full CLI/API: [`docs/valhalla/VALHALLA_BASE_CLI.md`](../../docs/valhalla/VALHALLA_BASE_CLI.md) · `manifestsys/hub-f64/include/valhalla_base.h`

Python wrapper: defaults to **`valhalla_base serve`** HTTP (`benchmark_runtime()`); set `VALHALLA_BASE_NO_SERVE=1` to fall back to subprocess.

| | Standard LLM | ValhallaBase |
|--|--------------|--------------|
| Context | Concatenated in prompt window | ingress → Hub/Tile/Stem structure |
| Change context | Edit prompt | `set_context()` / swap bundle, rebuild fingerprint |
| LM role | Sole foundation model | **decode head** (runtime patch) |
| No LM | Unavailable | `decode=native` Tier B memory retrieval |

```bash
cargo build -p hub-f64 --release --bin valhalla_base
python3 tools/valhalla_inference/test_valhalla_base_context.py
python3 tools/valhalla_inference/test_valhalla_base_context.py --decode native
```

Protocol: `valhalla-base-v1` · **Full CLI/API:** [`docs/valhalla/VALHALLA_BASE_CLI.md`](../../docs/valhalla/VALHALLA_BASE_CLI.md) · `manifestsys/hub-f64/include/valhalla_base.h`

### v1 extensions (1 hybrid · 2 session · 3 multimodal)

| # | Capability | Usage |
|---|------------|-------|
| 1 | **Hybrid routing** | `decode=hybrid` — open→native (LM fallback on abstain), mcq/numeric→LM logprob |
| 2 | **Persistent session** | `session` + `append_lines` — incremental context without resending full corpus |
| 3 | **Multimodal context** | `context_items[]` — text / audio_features / image_features ingress |

```bash
python3 tools/valhalla_inference/test_valhalla_base_v1.py
```

```python
base = ValhallaBase(decode="hybrid")
base.set_context(["Fact A"])
base.ask("open question?", score_type="open")
base.append_context("Fact B")  # incremental session
base.run(questions, append_only=True)
base.append_multimodal(text="...", audio_features=[...], image_features=[...])
```

## Module stack → Transformer

| Module | Role | How it writes into Transformer |
|--------|------|--------------------------------|
| **hub** | Fate quad routing | `quad_layer_scales` → 24-layer LayerNorm group scaling |
| **tile** | Block signatures | → `patch_vector` → hidden embed delta |
| **stemcell** | Cell graph | → fused into `patch_vector` |
| **triad** | Triad parallel incubation | `build_model_patch()` → bake / runtime |
| **dl_front** | Multimodal ingress | 384-d before LM → three ports (non-generate path) |
| **patch_lm** | Candle Qwen2 | safetensors load + weight surgery |
| **fused_lm** | Single-track LM product | baked forward + MCQ logprob |

## Load modes

```
                    ┌─────────────────────────────────────┐
  Corpus / multimodal │  Hub + Tile + Stem (Triad)          │
                    └──────────────┬──────────────────────┘
                                   │ model_patch
                    ┌──────────────▼──────────────────────┐
                    │  bake → model.safetensors           │
                    │  + valhalla_manifest.json           │
                    └──────────────┬──────────────────────┘
           ┌───────────────────────┼───────────────────────┐
           ▼                       ▼                       ▼
   transformers              vLLM (optional)           valhalla_fused_lm
   AutoModel...              pre-bake as Qwen2          Rust Candle baked_lm
```

| Mode | Condition | Notes |
|------|-----------|-------|
| **baked_hf** | manifest present, `runtime_patch=false` | Structure in weights; vLLM can serve directly |
| **runtime_patch** | bare Qwen | Per-query Triad → runtime patch |
| **vllm_compat** | pre-bake directory | `vllm serve models/hf/...` as ordinary Qwen2 |

## Quick test

```bash
source .venv-llm/bin/activate
export HF_ENDPOINT=https://hf-mirror.com

# vLLM-style API (transformers backend)
python3 tools/valhalla_inference/test_model_load.py

# Specify engine
python3 tools/valhalla_inference/test_model_load.py --engine transformers

# If vLLM installed
pip install vllm   # optional
python3 tools/valhalla_inference/test_model_load.py --try-vllm

# + Rust fused_lm smoke
python3 tools/valhalla_inference/test_model_load.py --fused-smoke
```

## Python API (vLLM-like)

```python
import sys
sys.path.insert(0, "tools/valhalla_inference")
from loader import ValhallaLLM

llm = ValhallaLLM("models/hf/Qwen2.5-0.5B-Instruct-Valhalla-FreeMultimodal-Fusion-v2-test-s0.10")
out = llm.generate(["What is Newton's first law?"])
print(out[0].text)
```

## vLLM CLI (pre-baked models)

Baked weights are standard Qwen2 HF format; `valhalla_manifest.json` is ignored:

```bash
vllm serve models/hf/Qwen2.5-0.5B-Instruct-Valhalla-FreeMultimodal-Fusion-v2-test-s0.10 \
  --trust-remote-code --max-model-len 4096
```

Hybrid routing / per-query runtime patch **is not** inside the vLLM graph; use an external orchestrator or pre-bake.

## Reports

`reports/valhalla_inference/model_load_test.json`

## Inference paradigm experiments (zero-weight / non fine-tune)

Full record: [EXPERIMENT_RECORD_INFERENCE_PARADIGM.md](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/EXPERIMENT_RECORD_INFERENCE_PARADIGM.md).

| Script | Protocol | Metric |
|--------|----------|--------|
| `test_valhalla_base_training_potential.py --protocol v2` | training-potential-v2 | TPI-v2 |
| `test_new_paradigm_potential.py` | new-paradigm-v1 | NPPI |
| `test_context_topic_drift.py` | context-topic-drift | swap / drift |
| `run_local_corpus_demo.py` | local-corpus-demo-v1 | local corpus ingest |
| `test_fate_output_gate.py` | fate-output-gate-v2 | FOG precision/coverage |
| `test_proactive_fate_v2.py` | proactive-fate-v2 | PPI-v2 / UP / MAR · **Fate signal_tick warmup** |
| `run_proactive_agency_benchmark.py` | **proactive-agency-v1** | **Agency Index 0.639** (full) · 10 multimodal · 8 long · CCU 100% · PPI-v2 0.650 |

```bash
# New paradigm potential (fair holdout + synthetic novel + negative controls)
python3 tools/valhalla_inference/test_new_paradigm_potential.py

# Local markdown corpus → teach → generation compare
python3 tools/valhalla_inference/run_local_corpus_demo.py \
  --corpus-dir valhalla-unified-core/docs

# Full proactive agency benchmark (baked 3B default)
tools/valhalla_inference/run_proactive_agency_benchmark.sh --verbose
# Extended (10 multimodal + 5 LoCoMo + full PPI-v2)
tools/valhalla_inference/run_proactive_agency_full.sh --verbose
# Specialized (handcraft long_dialogue + PPI-v2)
tools/valhalla_inference/run_proactive_agency_specialized.sh --verbose
# Mega 3000 turns (memory-bounded: segment-decodes + corpus cap + serve)
VALHALLA_BASE_URL=http://127.0.0.1:8788 \
  tools/valhalla_inference/run_proactive_agency_benchmark.sh \
  --mega --turns 3000 --extend-chunk 8 --segment-decodes 4 --verbose

# Unified confidence intervals (Wilson + bootstrap, all experiment JSON)
python3 tools/valhalla_inference/run_confidence_report.py

# traditional_lm control + refresh TPI/NPPI (.venv-llm + HF Qwen cache)
.venv-llm/bin/python3 tools/valhalla_inference/test_valhalla_base_training_potential.py --only-rag
.venv-llm/bin/python3 tools/valhalla_inference/test_new_paradigm_potential.py --only-n5

# Proactive Fate — full fair holdout (--fair-limit 0 = all open_retrieval)
python3 tools/valhalla_inference/test_proactive_fate_v2.py --fair-limit 0

# Cross-language same core (no stack switch) + external holdout
python3 tools/valhalla_inference/test_crosslang_core_unified.py
python3 tools/valhalla_inference/test_external_holdout.py

# MCQ suite (fair-1.2 n=46) + max_pick routing
python3 tools/valhalla_inference/test_mcq_coverage_v1.py --fast
.venv-llm/bin/python3 tools/valhalla_inference/test_mcq_coverage_v1.py --only-trad-lm

# Token / wire efficiency (30-turn compare)
python3 tools/valhalla_inference/test_token_efficiency.py

# Fate output gate (emit / abstain / defer)
python3 tools/valhalla_inference/test_fate_output_gate.py
```
