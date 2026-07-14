# Multimodal mixed answering — substrate proof (2026-07-14)

**Positioning:** deepen Triad substrate proof — **not** generative model catalog expansion.

## Product rule

| Rule | Default | Option |
|------|---------|--------|
| Ingress | Text + audio + image → **one** Triad session (non-isolated) | — |
| Answer evidence | **`answer_mix=mixed`** — all modality-tagged lines can land | `text` · `audio` · `image` |
| Decode head | Optional / swappable (U4) | Any supported LM |

## Why this (not “fuse three more small models”)

- U1 already proves **non-isolated** shared body (`shared_patch` ≈ 1).
- Buyers who need multimodal need **one body + mixed land**, not a voice+vision SDK soup.
- Channel options exist for ablation / preference; **mixed stays production default**.

## Proof

```bash
PYTHONPATH=tools/valhalla_inference \
  python3 tools/valhalla_inference/test_multimodal_mixed_answer.py
```

Artifacts: `reports/valhalla_inference/multimodal_mixed_answer.json` · `MULTIMODAL_MIXED_ANSWER.md`

## Config

```python
from benchmark_env import production_multimodal_kwargs
from valhalla_base import ValhallaBase

base = ValhallaBase(**production_multimodal_kwargs())  # core_fusion + answer_mix=mixed
# Optional:
base = ValhallaBase(**production_multimodal_kwargs(answer_mix="audio"))
```

Rust: `ValhallaAnswerMix::{Mixed,Text,Audio,Image}` · `corpus_modality_mask` on session.
