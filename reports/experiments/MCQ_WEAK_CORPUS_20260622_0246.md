# MCQ Weak Corpus Boost Test

**Time**: 20260622_0246 · **MCQ**: 46Q

机制：`mcq_corpus_option_boost` 0.35→0.12/词、cap 0.85、min word 5、每词首条语料命中；`keyword_domain_boost` 权重 0.5→0.2

| Arm | Acc |
|-----|-----|
| baseline persistent c1 | 21/46 (45.7%) |
| **weak corpus boost** | **21/46 (45.6%)** |

**Δ vs baseline**: +0 (+0.0 pp)

decode paths: `{'mcq_option': 46}`

## Flips vs baseline

- `MCQ_F14` ✗→✓ pred=B gold=B
- `MCQ_F42` ✓→✗ pred=D gold=B
- `MCQ_F44` ✓→✗ pred=C gold=B
- `MCQ_F48` ✗→✓ pred=B gold=B

JSON: `reports/experiments/MCQ_WEAK_CORPUS_20260622_0246.json`