# Valhalla Baked HF Models

Pre-baked causal LMs for proactive agency benchmark and TriModal fusion inference.

| Model | Base | Product | Size | Use |
|-------|------|---------|------|-----|
| `Qwen2.5-0.5B-Instruct-Valhalla-TriModal-Fusion-v3-smoke-s0.10` | Qwen2.5-0.5B-Instruct | trimodal-fusion-v3 | ~943 MB | Smoke / CI |
| `Qwen2.5-0.5B-Instruct-Valhalla-FreeMultimodal-Fusion-v2-test-s0.10` | Qwen2.5-0.5B-Instruct | free-multimodal-v2 | ~943 MB | TTS+LLM fusion |
| `Qwen2.5-3B-Instruct-Valhalla-TriModal-Fusion-v3-test-s0.10` | Qwen2.5-3B-Instruct | trimodal-fusion-v3 | ~5.8 GB | **Default proactive agency benchmark** |

> **Note:** The 3B checkpoint exceeds GitHub LFS single-file limit (2 GiB). Metadata is in-repo; bake locally (see below) or copy from a release mirror.

Weights for **0.5B** models are stored via **Git LFS** (`model.safetensors`). Clone with:

```bash
git lfs install
git clone https://github.com/Rogue-Intelligence-INC/Valhalla.git
```

## Load

```python
from transformers import AutoModelForCausalLM, AutoTokenizer
model_id = "models/hf/Qwen2.5-3B-Instruct-Valhalla-TriModal-Fusion-v3-test-s0.10"
tokenizer = AutoTokenizer.from_pretrained(model_id, trust_remote_code=True)
model = AutoModelForCausalLM.from_pretrained(model_id, trust_remote_code=True)
```

## Re-bake

```bash
python tools/fair_benchmark/bake_trimodal_fusion.py \
  --base-model Qwen/Qwen2.5-3B-Instruct --phase test --strength 0.10 --strong
```

## Benchmark

```bash
MODEL=models/hf/Qwen2.5-3B-Instruct-Valhalla-TriModal-Fusion-v3-test-s0.10 \
  tools/valhalla_inference/run_proactive_agency_full.sh --verbose --save-transcripts
```

Dialogue transcripts: `reports/valhalla_inference/proactive_agency_transcripts/`
