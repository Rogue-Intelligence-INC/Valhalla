# Hybrid Valhalla — Unified Product Report

**Phase**: test · **Prompts**: 145 · **Time**: 20260622_0345
**Corpus**: 49 lines · **Patch body**: triad · **strength**: 0.08

## Summary (all types via Hybrid)

| Track | Acc | numeric | mcq | open |
|-------|-----|---------|-----|------|
| **Hybrid product (unified)** | 80.00% | 86.84% | 54.35% | 95.08% |
| Triad memory only | 39.31% | 57.89% | 43.48% | 24.59% |
| Tile+Fate memory only | 41.38% | 57.89% | 52.17% | 22.95% |
| patch LM (subset) | 87.88% | 76.32% | 0.00% | 95.08% |

## By type

- **open** hybrid: **95.08%** (patch LM: 95.08%)
- **mcq** hybrid: **54.35%** (tile+fate: 52.17%, triad: 43.48%)
- **numeric** hybrid: **86.84%**

Hybrid source mix: `{'tile_fate_persistent': 72, 'triad_persistent': 1, 'patch_lm': 72}`

## Unified policy

Unified Hybrid Valhalla — all 145 questions through one product router:

| Type | Route |
|------|-------|
| open / open_generation | Valhalla patch + Qwen generate |
| mcq | best(Tile+Fate memory, Triad memory) |
| numeric | max(Tile+Fate, Triad, patch LM) |
| other | Triad memory |

Memory protocol: persistent batch + whole-question grinder ingress.
MCQ memory: Tile body + VALHALLA_MCQ_FATE_CYCLES=1 (decompose/aggregate).

