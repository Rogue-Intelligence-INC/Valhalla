# MCQ Weak Corpus Boost — All Bodies

**Time**: 20260622_0248 · **MCQ**: 46Q

机制：`mcq_corpus_option_boost` 0.35→0.12/词、cap 0.85、min word 5；`keyword_domain_boost` 0.5→0.2

| Arm | Session | Baseline MCQ | Weak boost | Δ | decode path |
|-----|---------|--------------|------------|---|-------------|
| **tile_c1** | isolated | 13/46 (28.3%) | **10/46 (21.7%)** | -3 (-6.5 pp) | `{'mcq_option': 46}` |
| **stemcell_c1** | isolated | 13/46 (28.3%) | **10/46 (21.7%)** | -3 (-6.5 pp) | `{'mcq_option': 46}` |
| **triad_c1** | isolated | 13/46 (28.3%) | **11/46 (23.9%)** | -2 (-4.3 pp) | `{'mcq_option': 46}` |
| **triad_c1_persistent** | persistent | 21/46 (45.7%) | **21/46 (45.6%)** | +0 (+0.0 pp) | `{'mcq_option': 46}` |
| **tile_c1_persistent** | persistent | — | **22/46 (47.8%)** | — | `{'mcq_option': 46}` |
| **stemcell_c1_persistent** | persistent | — | **19/46 (41.3%)** | — | `{'mcq_option': 46}` |

JSON: `reports/experiments/MCQ_WEAK_CORPUS_ALL_BODIES_20260622_0248.json`
