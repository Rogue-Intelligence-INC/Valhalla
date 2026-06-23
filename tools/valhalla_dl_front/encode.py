#!/usr/bin/env python3
"""Valhalla DL front sidecar — text / image / audio → embedding JSON on stdout.

Reads one JSON object from stdin, writes {"embedding": [...], "encoder": "..."}.

Optional deps (install for real semantic vectors):
  pip install sentence-transformers pillow torchvision

Without deps: deterministic hash embedding (384-d, normalized) — same family as Rust fallback.
"""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path


EMBED_DIM = 384


def _normalize(v: list[float]) -> list[float]:
    n = sum(x * x for x in v) ** 0.5 or 1e-9
    return [x / n for x in v]


def hash_embed(text: str, dim: int = EMBED_DIM, seed: int = 0x544558545F464E54) -> list[float]:
    h = seed & 0xFFFFFFFFFFFFFFFF
    for b in text.encode("utf-8", errors="replace"):
        h = (h * 6364136223846793005 + b) & 0xFFFFFFFFFFFFFFFF
    out: list[float] = []
    for i in range(dim):
        h = (h * 6364136223846793005 + i) & 0xFFFFFFFFFFFFFFFF
        x = ((h >> 33) / float(2**32 - 1)) * 2.0 - 1.0
        out.append(x)
    return _normalize(out)


def encode_text(content: str) -> tuple[list[float], str]:
    try:
        from sentence_transformers import SentenceTransformer

        model = SentenceTransformer("all-MiniLM-L6-v2")
        vec = model.encode(content, normalize_embeddings=True)
        emb = [float(x) for x in vec]
        if len(emb) != EMBED_DIM:
            emb = (emb + [0.0] * EMBED_DIM)[:EMBED_DIM]
            emb = _normalize(emb)
        return emb, "sentence_transformers/MiniLM-L6-v2"
    except Exception:
        return hash_embed(content), "hash_text"


def encode_image_path(path: str) -> tuple[list[float], str]:
    p = Path(path)
    if not p.is_file():
        return hash_embed(f"image_missing:{path}"), "hash_image_missing"

    try:
        from PIL import Image
        import torch
        from torchvision import models, transforms

        img = Image.open(p).convert("RGB")
        preprocess = transforms.Compose(
            [
                transforms.Resize(256),
                transforms.CenterCrop(224),
                transforms.ToTensor(),
                transforms.Normalize(mean=[0.485, 0.456, 0.406], std=[0.229, 0.224, 0.225]),
            ]
        )
        resnet = models.resnet18(weights=models.ResNet18_Weights.DEFAULT)
        resnet.eval()
        with torch.no_grad():
            t = preprocess(img).unsqueeze(0)
            feat = resnet(t)
            emb = feat.squeeze().tolist()
        if len(emb) > EMBED_DIM:
            emb = emb[:EMBED_DIM]
        else:
            emb = emb + [0.0] * (EMBED_DIM - len(emb))
        return _normalize(emb), "torchvision/resnet18"
    except Exception:
        try:
            from PIL import Image

            img = Image.open(p).convert("L").resize((32, 32))
            pixels = list(img.getdata())
            emb = [float(x) / 255.0 for x in pixels]
            emb = (emb + [0.0] * EMBED_DIM)[:EMBED_DIM]
            return _normalize(emb), "pillow_grayscale_32"
        except Exception:
            digest = hashlib.sha256(p.read_bytes()).hexdigest()
            return hash_embed(digest, seed=0x494D475F50415448), "hash_image_bytes"


def encode_audio_path(path: str, n_mels: int = 64) -> tuple[list[float], str]:
    """WAV → mel-band energy vector (384-d). Falls back to byte hash."""
    p = Path(path)
    if not p.is_file():
        return hash_embed(f"audio_missing:{path}", seed=0x415544494F504154), "hash_audio_missing"

    try:
        import numpy as np

        try:
            from scipy.io import wavfile

            rate, data = wavfile.read(str(p))
        except Exception:
            import wave

            with wave.open(str(p), "rb") as wf:
                rate = wf.getframerate()
                raw = wf.readframes(wf.getnframes())
                sampwidth = wf.getsampwidth()
            if sampwidth == 2:
                data = np.frombuffer(raw, dtype=np.int16)
            elif sampwidth == 4:
                data = np.frombuffer(raw, dtype=np.int32)
            else:
                data = np.frombuffer(raw, dtype=np.uint8).astype(np.float32)

        if data.ndim > 1:
            data = data.mean(axis=1)
        x = data.astype(np.float64)
        if np.max(np.abs(x)) > 1.5:
            x = x / (np.max(np.abs(x)) + 1e-9)

        # Simple mel-like band energies (no librosa required)
        n_fft = 512
        hop = 256
        if len(x) < n_fft:
            x = np.pad(x, (0, n_fft - len(x)))
        frames = []
        for start in range(0, len(x) - n_fft + 1, hop):
            frame = x[start : start + n_fft] * np.hanning(n_fft)
            spec = np.abs(np.fft.rfft(frame)) ** 2
            frames.append(spec)
        if not frames:
            return hash_embed(p.read_bytes().hex()[:128], seed=0x415544494F), "hash_audio_short"
        power = np.mean(np.stack(frames, axis=0), axis=0)
        edges = np.linspace(0, len(power), n_mels + 1, dtype=int)
        mel = []
        for i in range(n_mels):
            chunk = power[edges[i] : edges[i + 1]]
            mel.append(float(np.log1p(np.mean(chunk) + 1e-9)))
        duration = float(len(x) / max(rate, 1))
        rms = float(np.sqrt(np.mean(x * x)))
        emb = mel + [duration, rms, float(rate), float(len(frames)), float(np.std(x))]
        if len(emb) > EMBED_DIM:
            emb = emb[:EMBED_DIM]
        else:
            emb = emb + [0.0] * (EMBED_DIM - len(emb))
        return _normalize(emb), "wav_mel_bands"
    except Exception:
        digest = hashlib.sha256(p.read_bytes()).hexdigest()
        return hash_embed(digest, seed=0x415544494F), "hash_audio_bytes"


def encode_features(values: list[float], label: str) -> tuple[list[float], str]:
    emb = [float(v) for v in values]
    if len(emb) > EMBED_DIM:
        emb = emb[:EMBED_DIM]
    else:
        emb = emb + [0.0] * (EMBED_DIM - len(emb))
    return _normalize(emb), label


def main() -> None:
    raw = sys.stdin.read()
    req = json.loads(raw) if raw.strip() else {}
    modality = req.get("modality", "text")

    if modality == "text":
        content = req.get("content", "")
        emb, enc = encode_text(content)
    elif modality == "image_path":
        emb, enc = encode_image_path(req.get("path", ""))
    elif modality == "image_features":
        emb, enc = encode_features(req.get("values", []), "image_features")
    elif modality == "audio_path":
        emb, enc = encode_audio_path(req.get("path", ""))
    elif modality == "audio_features":
        emb, enc = encode_features(req.get("values", []), "audio_features")
    elif modality == "embedding":
        emb, enc = encode_features(req.get("values", []), "precomputed")
    else:
        emb, enc = hash_embed(json.dumps(req, sort_keys=True)), "hash_unknown"

    json.dump({"embedding": emb, "encoder": enc}, sys.stdout)


if __name__ == "__main__":
    main()
