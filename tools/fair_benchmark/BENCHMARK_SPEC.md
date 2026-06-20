# Valhalla Fair Benchmark — Specification (fair-1.0)

**Purpose**: A reproducible, contamination-free evaluation protocol for Valhalla as a **memory platform** and **LLM incubation runtime**, with fair comparison to Transformer baselines.

---

## 1. Design principles

| Principle | Implementation |
|-----------|----------------|
| **No answer leakage** | `ingest_clean` corpus filtered against all eval `reference_answer` + keywords + known paraphrase patterns |
| **Held-out test** | `test` split (~76%) never used for tuning; `val` (~24%) for development only |
| **Track separation** | Memory (Tier B) and Generate (Transformer) reported on separate rows |
| **Decode transparency** | Report `by_decode_path` — rule-based vs memory vs retrieval |
| **Retrieval metrics** | recall@3, MRR on top_matches (memory platform standard) |
| **Honest open QA** | Open items scored by keyword/ref match; **not** compared 1:1 to generative LM without label |

---

## 2. Splits

| Split | Size | Use |
|-------|------|-----|
| `smoke` | 12 | CI fast gate (~2 min) |
| `val` | ~48 | Patch strength / body selection — **do not publish** |
| `test` | ~140 | **Primary report** split |

Assignment: fixed `SMOKE_IDS` + deterministic hash split (`SPLIT_SEED=20260619`, `VAL_RATIO=0.24`).

---

## 3. Corpus tiers

| Tier | Lines | Role |
|------|-------|------|
| `ingest_clean` | ~49 | **Default** — domain facts, numeric answer leaks removed |
| `ingest_legacy_contaminated` | ~81 | Ablation — includes 7 numeric solution-leak lines |

**Tier 1 — Hard ban (numeric eval cheating)**:
- Legacy paraphrase lines (e.g. `"17×23=391"`, `"Natalia... seventy two"`)
- Worked-solution lines that state a **numeric** benchmark answer

**Allowed (legitimate memory)**:
- Domain facts for open/mcq (Canberra, 1945, Newton…) — retrieval of these is the intended memory-platform behavior

---

## 4. Tracks & arms

### Memory track (`memory_tier_b`)

| Arm | Session | Description |
|-----|---------|-------------|
| baseline_no_corpus | isolated | triad, no ingest |
| hub/tile/stem/triad_c1 | isolated | per-question corpus re-ingest |
| triad_c1_persistent | **batch** | one incubation → all questions (hash grinder) |

### Patch track (`patch_tier_a`)

Optional `--with-patch` or `--full`. **Fresh HF reload per question** (fair eval).

| Arm | Body | Mode | Strength |
|-----|------|------|----------|
| patch_tile_legacy | tile | legacy | 0.08 |
| patch_stemcell_legacy | stemcell | legacy | 0.08 |
| patch_triad_legacy | triad | legacy | 0.08 |

### Generate track

`--with-transformer` or `--full`: Qwen2.5-0.5B zero-shot, fresh reload per question.

---

## 5. Metrics

### Task accuracy
- `acc` overall
- `by_score_type`: numeric | mcq | open
- `by_domain`, `by_decode_path`

### Memory retrieval
- `recall_at_3`: reference or ≥50% keywords in top-3 matches
- `mrr_mean`: mean reciprocal rank of first hit

### Statistical
- Paired bootstrap 95% CI vs `baseline_no_corpus` on **test** split only

---

## 6. Commands

```bash
# Regenerate suite
python tools/fair_benchmark/build_fair_suite.py

# CI validation (zero leakage)
python tools/fair_benchmark/validate_suite.py

# Fast smoke
cargo build -p hub-f64 --release --bin valhalla_native_qa
python tools/fair_benchmark/run_fair_benchmark.py --phase smoke

# Primary report (memory only)
python tools/fair_benchmark/run_fair_benchmark.py --phase test

# Complete (memory + patch + generate)
python tools/fair_benchmark/run_fair_benchmark.py --phase test --full

# Leakage ablation
python tools/fair_benchmark/run_fair_benchmark.py --phase test_legacy_ablation

# Sync to valhalla-transformer
python tools/fair_benchmark/sync_to_valhalla_transformer.py
```

---

## 7. What to publish

**May publish** (test split, clean corpus):
- Memory track acc + retrieval + decode_path breakdown
- Paired CI vs baseline
- Generate track numeric/mcq acc

**Do not publish**:
- val split numbers as product capability
- legacy contaminated corpus as primary metric
- open acc as "beats Transformer"

---

*Rogue Intelligence LNC. · fair-1.0 · 2026-06-19*
