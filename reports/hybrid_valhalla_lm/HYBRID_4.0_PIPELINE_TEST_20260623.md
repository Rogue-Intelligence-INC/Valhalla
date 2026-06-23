# Hybrid 4.0 Pipeline v4 — Implementation & Test (2026-06-23)

**Product**: `hybrid-4.0-rust-pipeline`  
**Pipeline**: `pipeline-v4-decompose`

## Architecture (implemented)

```
LM/Grinder Front
  → task_decompose (DecomposeMode per score_type)
  → Valhalla (Tile / Stem / Hub / Triad + mcq_fate aggregate)
  → recompose (RecomposePatch + mcq_option_affinity)
  → patch_lm (legacy patch + MCQ lm_head bias + generate)
  → hybrid merge (memory_mcq_dual: max Tier B + Tier A)
```

### New modules

| Module | Status |
|--------|--------|
| `task_decompose.rs` | DecomposePlan, RecomposePatch, MCQ/Numeric/Open shards |
| `pipeline.rs` | `run_question_pipeline`, `decode_mcq_from_recompose` |
| `patch_export.rs` | `build_grinder_incubation_snapshot` |
| `patch_lm.rs` | `run_patch_track_v4`, `apply_mcq_affinity_bias` |
| `native_qa.rs` | `pipeline_v4` batch path |
| `hybrid.rs` | `memory_mcq_dual`, MCQ patch in patch track |

## Test results (145Q)

| Mode | Hybrid | open | mcq | numeric |
|------|--------|------|-----|---------|
| **v4 deploy** | **78.62%** (114/145) | 93.44% | **52.17%** (24/46) | 86.84% |
| **v4 oracle** | **83.45%** (121/145) | — | — | — |
| v3.1 deploy (baseline) | 80.00% (116/145) | 95.08% | 52.17% | 89.47% |

Deploy hybrid sources (v4): `tile_fate_persistent: 46`, `patch_lm: 99` (includes all MCQ patch candidates)

### MCQ status

- Memory MCQ restored to **52.17%** (24/46) — same as pre-v4
- Oracle **+3.45pp** vs v3.1 (83.45% vs 80%) from patch+memory best-of
- MCQ patch track + lm_head bias scaffold in place; next gain from affinity→lm_head tuning

## Run

```bash
RUSTFLAGS='-L /opt/cuda/lib64' cargo build -p hub-f64 --release --bin valhalla_hybrid

# Full v4 pipeline (default)
./target/release/valhalla_hybrid --phase test --merge-mode deploy --cpu

# Disable v4
VALHALLA_PIPELINE_V4=0 ./target/release/valhalla_hybrid --phase test --patch-json ...
```

## Next (P3)

- `McqPairwiseCompare` decompose mode A/B
- Stronger lm_head bias from `mcq_option_affinity`
- GPU patch LM when cudarc supports CUDA 13.3
