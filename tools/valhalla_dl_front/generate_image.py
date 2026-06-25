#!/usr/bin/env python3
"""Generate image from prompt via small SD model → features for Valhalla tri-modal fusion.

stdin JSON:
  {"prompt": "...", "id": "p1", "out_dir": "/tmp/imgs"}

stdout JSON:
  {"image_path": "...", "image_features": [...], "encoder": "...", "latent_dim": N}

Uses local model at models/image_gen/sd-turbo (run download_image_model.py first).
Falls back to hash embedding if diffusers/torch unavailable.
"""

from __future__ import annotations

import json
import logging
import os
import subprocess
import sys
import warnings
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
ENCODE = REPO / "tools/valhalla_dl_front/encode.py"
DEFAULT_MODEL_DIR = REPO / "models" / "image_gen" / "sd-turbo"
EMBED_DIM = 384

os.environ.setdefault("HF_ENDPOINT", "https://hf-mirror.com")
os.environ.setdefault("TRANSFORMERS_VERBOSITY", "error")
os.environ.setdefault("DIFFUSERS_VERBOSITY", "error")
warnings.filterwarnings("ignore")
logging.getLogger("transformers").setLevel(logging.ERROR)
logging.getLogger("diffusers").setLevel(logging.ERROR)

_PIPE = None


def _normalize(v: list[float]) -> list[float]:
    n = sum(x * x for x in v) ** 0.5 or 1e-9
    return [x / n for x in v]


def hash_embed(text: str, dim: int = EMBED_DIM) -> list[float]:
    import hashlib

    h = hashlib.sha256(text.encode()).hexdigest()
    out = []
    for i in range(dim):
        chunk = h[(i * 2) % len(h) : (i * 2 + 2) % len(h) + 2] or "00"
        out.append(int(chunk[:2], 16) / 255.0 * 2 - 1)
    return _normalize(out)


def encode_image_path(path: str) -> list[float]:
    proc = subprocess.run(
        [sys.executable, str(ENCODE)],
        input=json.dumps({"modality": "image_path", "path": path}),
        capture_output=True,
        text=True,
        check=True,
    )
    return json.loads(proc.stdout)["embedding"]


def merge_features(a: list[float], b: list[float], w_a: float = 0.55) -> list[float]:
    dim = max(len(a), len(b), EMBED_DIM)
    out = []
    for i in range(dim):
        av = a[i % len(a)] if a else 0.0
        bv = b[i % len(b)] if b else 0.0
        out.append(av * w_a + bv * (1.0 - w_a))
    if len(out) > EMBED_DIM:
        out = out[:EMBED_DIM]
    else:
        out = out + [0.0] * (EMBED_DIM - len(out))
    return _normalize(out)


def generate_sd(prompt: str, out_path: Path, model_dir: Path) -> tuple[list[float], str]:
    global _PIPE
    import torch
    from diffusers import AutoPipelineForText2Image
    from PIL import Image

    model_id_file = model_dir / "model_id.txt"
    model_id = (
        model_id_file.read_text(encoding="utf-8").strip()
        if model_id_file.is_file()
        else "stabilityai/sd-turbo"
    )
    load_from = str(model_dir) if (model_dir / "model_index.json").is_file() else model_id

    if _PIPE is None:
        dtype = torch.float16 if torch.cuda.is_available() else torch.float32
        _PIPE = AutoPipelineForText2Image.from_pretrained(
            load_from,
            torch_dtype=dtype,
            variant="fp16" if dtype == torch.float16 else None,
        )
        if torch.cuda.is_available():
            _PIPE = _PIPE.to("cuda")
        _PIPE.set_progress_bar_config(disable=True)

    pipe = _PIPE

    steps = 1 if "turbo" in model_id.lower() else 4
    guidance = 0.0 if "turbo" in model_id.lower() else 7.5
    result = pipe(prompt, num_inference_steps=steps, guidance_scale=guidance)
    image: Image.Image = result.images[0]
    out_path.parent.mkdir(parents=True, exist_ok=True)
    image.save(out_path)

    latent_feat: list[float] = []
    encoder = f"diffusers/{model_id.split('/')[-1]}"
    try:
        import numpy as np
        from torchvision import transforms

        t = transforms.Compose(
            [
                transforms.Resize(512),
                transforms.CenterCrop(512),
                transforms.ToTensor(),
                transforms.Normalize([0.5], [0.5]),
            ]
        )
        with torch.no_grad():
            pix = t(image).unsqueeze(0)
            if torch.cuda.is_available():
                pix = pix.cuda()
            if hasattr(pipe, "vae") and pipe.vae is not None:
                lat = pipe.vae.encode(pix).latent_dist.mean
                flat = lat.flatten().float().cpu().numpy()
                stats = [
                    float(flat.mean()),
                    float(flat.std()),
                    float(np.percentile(flat, 25)),
                    float(np.percentile(flat, 75)),
                ]
                sample = flat[:: max(1, len(flat) // (EMBED_DIM - 4))][: EMBED_DIM - 4]
                latent_feat = _normalize([float(x) for x in sample] + stats)
            else:
                latent_feat = hash_embed(prompt + ":latent")
    except Exception:
        latent_feat = hash_embed(prompt + ":latent_fallback")

    pixel_emb = encode_image_path(str(out_path))
    merged = merge_features(latent_feat, pixel_emb, 0.45)
    return merged, encoder


def main() -> None:
    raw = sys.stdin.read()
    req = json.loads(raw) if raw.strip() else {}
    prompt = req.get("prompt", "")
    sample_id = req.get("id", "img0")
    out_dir = Path(req.get("out_dir", "/tmp/valhalla_trimodal_imgs"))
    model_dir = Path(req.get("model_dir", str(DEFAULT_MODEL_DIR)))

    out_path = out_dir / f"{sample_id}.png"

    try:
        features, encoder = generate_sd(prompt, out_path, model_dir)
    except Exception as e:
        print(f"SD fallback ({e})", file=sys.stderr)
        out_dir.mkdir(parents=True, exist_ok=True)
        # placeholder PNG via PIL if possible
        try:
            from PIL import Image, ImageDraw

            img = Image.new("RGB", (64, 64), color=(40, 40, 80))
            d = ImageDraw.Draw(img)
            d.text((4, 24), sample_id[:8], fill=(200, 200, 255))
            img.save(out_path)
            features = encode_image_path(str(out_path))
            encoder = "hash_sd_fallback"
        except Exception:
            features = hash_embed(prompt)
            encoder = "hash_prompt_only"
            out_path = Path("")

    json.dump(
        {
            "image_path": str(out_path) if out_path else "",
            "image_features": features,
            "encoder": encoder,
            "latent_dim": len(features),
        },
        sys.stdout,
    )
    sys.stdout.write("\n")
    sys.stdout.flush()


if __name__ == "__main__":
    main()
