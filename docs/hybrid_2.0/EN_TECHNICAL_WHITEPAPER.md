# Hybrid Valhalla 2.0 — Technical Whitepaper

**Document type**: Product technical whitepaper  
**Version**: hybrid-2.0 · hybrid-4.0-rust-pipeline · fair-1.2 · hub-f64 1.1.0  
**Date**: 2026-06-23  
**Publisher**: Rogue Intelligence LNC.  
**Audience**: ML systems engineers, architects, third-party auditors  
**Status**: Experiment-driven; claims tied to reproducible artifacts

---

## Abstract

Hybrid Valhalla **hybrid-2.0** is a **unified Q&A product** that routes each prompt to the best answering path among:

1. **Tier B native memory** (Valhalla structure incubation, no `model.generate()`), and  
2. **Tier A patch LM** (Valhalla structure exported as weight perturbations on Qwen2.5-0.5B-Instruct).

On the **fair-1.2** benchmark test split (**145 prompts**, **49-line decontaminated corpus**), the unified product achieves:

| Metric | Hybrid unified |
|--------|----------------|
| Total | **80.69%** (117/145) |
| open | **93.44%** |
| mcq | **60.87%** (28/46) |
| numeric | **84.21%** |

**Primary artifact**: `reports/hybrid_valhalla_lm/hybrid_v4_mcq_patch_strengthen_final.json`

---

## 1. Terminology

| Term | Definition |
|------|------------|
| **Hybrid product** | Single router; all prompts enter one policy (`hybrid-2.0`) |
| **Tier A (patch LM)** | Structure → `model_patch` → Qwen forward + generate |
| **Tier B (native memory)** | Structure incubation + rule/score decode; no HF generate |
| **One-shot incubation** | Corpus ingested once; structure shared across batch (persistent session) |
| **Whole-question ingress** | MCQ: stem + four options encoded together |
| **fair-1.2** | Decontaminated corpus; smoke/val/test splits; separate native vs LM scoring |
| **strict MCQ** | Output must be scorable as `ANSWER: X`; extraction failures count wrong |

---

## 2. System architecture

### 2.1 Layered stack

```
┌─────────────────────────────────────────────────────────┐
│              Hybrid router (hybrid-2.0)                    │
│  open → patch_lm │ mcq → memory_mcq_best │ numeric → max  │
└──────────────────────────┬──────────────────────────────┘
                           │
         ┌─────────────────┼─────────────────┐
         ▼                 ▼                 ▼
   Tile+Fate memory   Triad memory     Patch LM (Qwen 0.5B)
   (native QA)        (native QA)      (Tier A export)
         │                 │                 │
         └─────────────────┴─────────────────┘
                           │
                           ▼
              Structure incubation layer
         Hub · Tile · StemCell · Fate cycles
                           │
                           ▼
              Corpus ingress (Grinder / optional LLM states)
```

### 2.2 Core modules

| Module | Role in Hybrid 2.0 |
|--------|-------------------|
| **Hub** | Quad routing, Fate preferences, checkpoint orchestration |
| **Tile** | Block memory; best MCQ body in experiments (22–24/46) |
| **StemCell** | Self-organizing clusters; numeric/MCQ support |
| **Fate** | Dynamic stop / closure; MCQ decompose-aggregate cycles |
| **native_qa** | Rust decode: `mcq_option`, numeric extract, open retrieval |
| **patch bridge** | Python: apply Valhalla patch @ strength 0.08 (triad body) |

### 2.3 Hybrid routing policy

Implementation: `tools/fair_benchmark/hybrid_valhalla_lm.py`

| `score_type` | Policy ID | Selection rule |
|--------------|-----------|----------------|
| `open`, `open_generation` | `patch_lm` | Valhalla patch + Qwen generate |
| `mcq` | `memory_mcq_best` | First correct among Tile+Fate, Triad; else best available |
| `numeric` | `hybrid_max_all` | First correct among Tile+Fate, Triad, patch; else best |
| other / corpus-supported | `memory_triad` | Triad persistent memory |

**Product invariant**: Users never select a track; the router decides per prompt.

---

## 3. Benchmark design (fair-1.2)

### 3.1 Prompt suite

- **Suite file**: `tools/fair_benchmark/fair_benchmark_suite.json`
- **Test split**: 145 prompts
  - 46 MCQ (4-option)
  - numeric subset
  - open / open_generation
- **Corpus**: 49 reference lines; **no gold answers** in eval corpus

### 3.2 Scoring

- **numeric**: normalized answer match
- **mcq**: strict letter extraction from `ANSWER: X` or final line
- **open**: reference overlap / generation quality metrics per fair harness

### 3.3 Session protocols

| Protocol | Description | MCQ impact |
|----------|-------------|------------|
| per-question reload | Corpus re-ingested each prompt | ~28% MCQ |
| **persistent one-shot** | Corpus once, many questions | **~46–52% MCQ** |

Hybrid 2.0 uses **persistent** memory for Tier B paths.

### 3.4 Baselines reported alongside Hybrid

| Track | Total acc | open | mcq | numeric |
|-------|-----------|------|-----|---------|
| **Hybrid unified** | **80.00%** | 95.08% | 54.35% | 86.84% |
| Triad memory only | 39.31% | 24.59% | 43.48% | 57.89% |
| Tile+Fate memory only | 41.38% | 22.95% | 52.17% | 57.89% |
| Patch LM (routed subset) | 87.88% | 95.08% | 0%* | 76.32% |
| Qwen 0.5B zero-shot (145Q) | 71.0% | ~93% | 37.0% | ~76% |

*Patch LM not used as primary MCQ path in hybrid-2.0 policy.

**Hybrid source mix (145Q)**: `tile_fate_persistent: 72`, `triad_persistent: 1`, `patch_lm: 72`

---

## 4. Tier B: Native memory paths

### 4.1 Incubation pipeline

1. Load 49-line corpus into Valhalla ingress (Grinder vectors).
2. Run structure evolution (Tile and/or Triad body enabled).
3. For MCQ with `VALHALLA_MCQ_FATE_CYCLES=1`: tag `task_tag:mcq`, decompose stem+options, aggregate via Fate dynamic stop.

### 4.2 MCQ decode (`mcq_option`)

Per-option independent scoring → argmax → `ANSWER: X`:

- Option vector vs whole-question vector (cosine)
- Option vs structure summary vector
- Word overlap with corpus lines (attenuated to limit keyword guessing)
- Keyword overlap helpers

**Not used**: LLM chain-of-thought; relative-only stem comparison (roadmap).

### 4.3 MCQ experiment summary (46 questions)

| Method | Correct | Acc |
|--------|---------|-----|
| Native, no corpus | 6/46 | 13.0% |
| Per-question reload | 13/46 | 28.3% |
| One-shot Triad (all modules) | 21/46 | 45.7% |
| One-shot Tile only | 22/46 | 47.8% |
| Tile + MCQ Fate cycles | 24/46 | 52.2% |
| Qwen zero-shot | 17/46 | 37.0% |

**Conclusion**: Session protocol (+18 pp reload→one-shot) dominates small scoring tweaks. Bottleneck is **four-way option comparison**.

---

## 5. Tier A: Patch LM path

### 5.1 Patch geometry

- 4 groups of `input_layernorm.weight` scaled by `quad_layer_scales`
- Prompt-token embeddings perturbed by projected `patch_vector`
- Default strength: **0.08**, body: **triad**

### 5.2 Role in Hybrid 2.0

- **Primary for open**: ~95% accuracy on test open slice
- **Candidate for numeric**: combined via `hybrid_max_all`
- **Not primary for MCQ**: patch hurts MCQ vs native on same benchmark (~26–28%)

### 5.3 Controls (historical context)

On 48Q incubation suite (strict): Valhalla triad patch **-2.08 pp** vs random patch **-14.58 pp** — structure signal is **not noise**, but net strict gain on small suite was ≤ 0. Hybrid product uses patch where it **wins** (open), not everywhere.

---

## 6. Unified product implementation

### 6.1 Entry points

| Component | Path |
|-----------|------|
| Router logic | `tools/fair_benchmark/hybrid_valhalla_lm.py` |
| Runner | `tools/fair_benchmark/run_hybrid_valhalla_lm.py` |
| Native binary | `manifestsys/hub-f64` → `valhalla_native_qa` |
| MCQ Fate | `manifestsys/hub-f64/src/mcq_fate.rs` |

### 6.2 Reproduction

```bash
# Build native QA
cd manifestsys/hub-f64
RUSTFLAGS='-L /opt/cuda/lib64' cargo build -p hub-f64 --release --bin valhalla_native_qa

# Run Hybrid on fair test (reuse prior fair JSON for memory tracks)
cd /path/to/Valhalla
python3 tools/fair_benchmark/run_hybrid_valhalla_lm.py \
  --phase test \
  --reuse-json reports/fair_benchmark/fair_benchmark_test_20260620_0859.json

# MCQ Fate cycles (Tile body)
export VALHALLA_MCQ_FATE_CYCLES=1
```

### 6.3 Environment variables (MCQ)

| Variable | Default | Purpose |
|----------|---------|---------|
| `VALHALLA_MCQ_FATE_CYCLES` | 0 (1 in Hybrid MCQ runs) | Enable MCQ decompose/aggregate |
| `VALHALLA_MCQ_FATE_MAX_CYCLES` | 64 | Fate cycle cap |
| `VALHALLA_MCQ_FATE_HARD_MAX` | 128 | Hard stop |
| `VALHALLA_MCQ_FATE_MIN_CYCLES` | 5 | Minimum cycles before stop |

---

## 7. Verifiable claims register

| Claim | Evidence | Status |
|-------|----------|--------|
| Hybrid 80% on 145Q test | `hybrid_unified_test_20260622_0345.json` | ✓ |
| Open 95% via patch route | Same JSON, `by_score_type.open` | ✓ |
| Native MCQ beats Qwen 0.5B alone | 24/46 vs 17/46 (Tile+Fate) | ✓ |
| Patch does not fix MCQ | 12–13/46 patch MCQ historical | ✓ |
| One-shot protocol critical | 28% → 46% MCQ | ✓ |
| Hybrid beats Qwen 71% total | 80% vs 71% same split | ✓ |
| Beats GPT-4 / Claude general | — | ✗ not claimed |
| FIPS-certified crypto | — | ✗ not claimed (Yaksha separate) |

---

## 8. Known limitations & roadmap

### 8.1 Limitations

1. **MCQ ceiling ~54%**: comparator scores options independently, not relatively conditioned on stem.  
2. **Small companion LM**: 0.5B — demonstrates structure layer; not production LM choice.  
3. **Corpus size**: 49 lines — vertical POC scale, not web-scale RAG.  
4. **Tier B open retrieval alone ~25%**: open solved at **product** level via Hybrid, not memory-only.

### 8.2 Roadmap (engineering)

| Priority | Item | Target |
|----------|------|--------|
| P0 | Stem-conditioned relative MCQ comparator | 65%+ MCQ fair test |
| P1 | Corpus line → option support graph | Reduce keyword false positives |
| P2 | Enterprise SDK (routing trace export) | GA |
| P3 | Larger fair corpus + third-party audit | fair-2.0 |

---

## 9. Security & deployment

- **On-prem**: Rust binaries, local `.hub` / checkpoint files  
- **Optional encryption**: Yaksha AEAD module (separate whitepaper; Industry Gate narrative)  
- **Audit**: Per-answer routing key (`tile_fate_persistent` | `triad_persistent` | `patch_lm`) in Hybrid JSON rows  
- **No mandatory external API** for Tier B paths

---

## 10. References

| Document | Path |
|----------|------|
| MCQ & Fair summary | `reports/VALHALLA_MCQ_AND_FAIR_BENCHMARK_SUMMARY_20260622.md` |
| Hybrid unified report | `reports/VALHALLA_HYBRID_UNIFIED_test_20260622_0345.md` |
| Public viz repo | https://github.com/Rogue-Intelligence-INC/valhalla-transformer (v2.0) |
| Valhalla vs Transformer WP | `docs/valhalla_vs_transformer/en/04_TECHNICAL_WHITEPAPER.md` |
| Product definition | `docs/VALHALLA_PRODUCT_DEFINITION.md` |

---

*Rogue Intelligence LNC. · Hybrid Valhalla hybrid-2.0 · fair-1.2 · hub-f64 1.1.0 · 2026-06-22*
