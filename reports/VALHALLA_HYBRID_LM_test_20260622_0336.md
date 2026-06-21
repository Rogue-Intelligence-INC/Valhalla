# Hybrid Valhalla LM Report

**Phase**: test · **Prompts**: 145 · **Time**: 20260622_0336
**Corpus**: 49 lines · **Patch body**: triad · **strength**: 0.08

## Summary

| Track | Acc | numeric | mcq | open |
|-------|-----|---------|-----|------|
| **Hybrid product** | 77.24% | 86.84% | 45.65% | 95.08% |
| memory persistent | 40.69% | 57.89% | 45.65% | 26.23% |
| patch LM (subset) | 87.88% | 76.32% | 0.00% | 95.08% |

## Open slice (LM fixes retrieval gap)

- memory open: **26.23%**
- Valhalla patch LM open: **95.08%**
- hybrid open: **95.08%**
- open_retrieval: **96.77%**
- open_generation: **93.33%**

Hybrid source mix: `{'memory': 68, 'patch_lm': 72, 'none': 5}`

## Policy

- `open` / `open_generation` → **Valhalla-patched LM** (Tier A generate)
- `mcq` + corpus_supported → **memory** (Tier B)
- `numeric` → **max(memory, patch_lm)**

