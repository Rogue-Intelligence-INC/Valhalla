# Image gen models for tri-modal fusion

Download SD-Turbo (~1.5GB):

```bash
source .venv-llm/bin/activate
pip install diffusers transformers accelerate safetensors
python tools/valhalla_dl_front/download_image_model.py
```

Default model: `stabilityai/sd-turbo` → `models/image_gen/sd-turbo/`
