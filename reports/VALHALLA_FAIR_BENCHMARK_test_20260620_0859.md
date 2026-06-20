# Valhalla Fair Benchmark Report

**Phase**: test · **Split**: test · **Corpus**: ingest_clean
**Time**: 20260620_0859 · **Prompts**: 145
**Suite**: fair-1.0 · **Clean corpus**: True
**Tracks**: memory=True patch=True generate=True

## Memory track (Tier B)

| Arm | Acc | numeric | mcq | open |
|-----|-----|---------|-----|------|
| baseline_no_corpus | 20.00% | 57.89% | 13.04% | 1.64% |
| hub_c1 | 35.17% | 57.89% | 32.61% | 22.95% |
| tile_c1 | 34.48% | 57.89% | 28.26% | 24.59% |
| stemcell_c1 | 34.48% | 57.89% | 28.26% | 24.59% |
| triad_c1 | 35.86% | 57.89% | 28.26% | 27.87% |
| triad_c1_persistent | 40.69% | 57.89% | 45.65% | 26.23% |

## Patch track (Tier A · fresh reload / Q)

| Arm | Acc | numeric | mcq | open |
|-----|-----|---------|-----|------|
| patch_tile_legacy | 68.28% | 76.32% | 28.26% | 93.44% |
| patch_stemcell_legacy | 68.97% | 76.32% | 28.26% | 95.08% |
| patch_triad_legacy | 68.28% | 76.32% | 26.09% | 95.08% |

## Generate track (Transformer zero-shot)

| Arm | Acc | numeric | mcq | open |
|-----|-----|---------|-----|------|
| transformer_zero_shot | 71.03% | 76.32% | 36.96% | 93.44% |

## Paired Δ vs baseline (memory, pp 95% CI)

- **hub_c1**: +15.17 pp [+8.97, +21.38]
- **tile_c1**: +14.48 pp [+8.28, +20.69]
- **stemcell_c1**: +14.48 pp [+8.28, +20.69]
- **triad_c1**: +15.86 pp [+9.66, +22.76]
- **triad_c1_persistent**: +20.69 pp [+13.79, +28.28]

## Decode path (memory, isolated arms)

- **baseline_no_corpus**: cortex=24%, direct_math=100%, mcq_option=13%, open_reject_no_keywords=2%
- **hub_c1**: cortex=24%, direct_math=100%, mcq_option=33%, open_span=23%
- **tile_c1**: cortex=24%, direct_math=100%, mcq_option=28%, open_span=25%
- **stemcell_c1**: cortex=24%, direct_math=100%, mcq_option=28%, open_span=25%
- **triad_c1**: cortex=24%, direct_math=100%, mcq_option=28%, open_span=28%

