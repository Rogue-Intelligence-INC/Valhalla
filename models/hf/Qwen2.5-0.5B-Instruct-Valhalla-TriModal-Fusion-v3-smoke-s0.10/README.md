---
license: apache-2.0
base_model: Qwen/Qwen2.5-0.5B-Instruct
tags:
  - valhalla
  - text-generation
  - qwen2
  - multimodal-fusion
library_name: transformers
pipeline_tag: text-generation

---

# Qwen2.5-0.5B-Instruct-Valhalla-TriModal-Fusion-v3-smoke-s0.10

Valhalla-baked causal LM derived from [Qwen/Qwen2.5-0.5B-Instruct](https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct).

| Field | Value |
|-------|-------|
| Product | `valhalla-trimodal-fusion-v3` |
| Bake mode | `full` |
| Strength | `0.1` |
| Body | `triad` |
| Baked at | `2026-06-25T17:55:27.933067+00:00` |
| Weight delta norm | `298.67698926400874` |

Structure incubation: tiles=72, stem=154, signals=152064.



## Load (standard Hugging Face)

```python
from transformers import AutoModelForCausalLM, AutoTokenizer

model_id = "/home/jouly/AI/Valhalla/models/hf/Qwen2.5-0.5B-Instruct-Valhalla-TriModal-Fusion-v3-smoke-s0.10"
tokenizer = AutoTokenizer.from_pretrained(model_id, trust_remote_code=True)
model = AutoModelForCausalLM.from_pretrained(model_id, trust_remote_code=True)
```

## Valhalla fused LM (optional, same weights)

```bash
valhalla_fused_lm --model /home/jouly/AI/Valhalla/models/hf/Qwen2.5-0.5B-Instruct-Valhalla-TriModal-Fusion-v3-smoke-s0.10 --phase test
```

`valhalla_manifest.json` documents incubation metadata; `transformers` ignores it at load time.

---
*Rogue Intelligence LNC. — Valhalla free multimodal fusion bake*
