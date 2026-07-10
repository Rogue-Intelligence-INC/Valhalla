"""Encoder-based context routing for Valhalla native ingress."""

from __future__ import annotations

from typing import Any

import numpy as np


def _cos(a: np.ndarray, b: np.ndarray) -> float:
    na = float(np.linalg.norm(a)) or 1e-9
    nb = float(np.linalg.norm(b)) or 1e-9
    return float(np.dot(a, b) / (na * nb))


def _prefix_text(hf_id: str, text: str, *, role: str) -> str:
    low = hf_id.lower()
    if "nomic-embed" in low:
        return f"search_{role}: {text}"
    if "bge-" in low and role == "query":
        return f"Represent this sentence for searching relevant passages: {text}"
    if "jina-embeddings" in low:
        return text
    return text


class EncoderContextRouter:
    """Semantic top-k line picker — feeds ValhallaBase.set_context()."""

    def __init__(self, hf_id: str = "nomic-ai/nomic-embed-text-v1") -> None:
        from test_context_arch_compare import resolve_lm_path  # noqa: WPS433

        path = resolve_lm_path(hf_id)
        from transformers import AutoModel, AutoTokenizer
        import torch

        self.hf_id = hf_id
        self.path = str(path)
        self.tok = AutoTokenizer.from_pretrained(str(path), trust_remote_code=True)
        dtype = torch.float16 if torch.cuda.is_available() else torch.float32
        kwargs: dict[str, Any] = {"trust_remote_code": True, "torch_dtype": dtype}
        if torch.cuda.is_available():
            kwargs["device_map"] = "auto"
        self.model = AutoModel.from_pretrained(str(path), **kwargs)
        self.model.eval()
        self._device = next(self.model.parameters()).device

    def _embed_batch(self, texts: list[str], *, role: str) -> np.ndarray:
        import torch

        prefixed = [_prefix_text(self.hf_id, t, role=role) for t in texts]
        batch = self.tok(prefixed, padding=True, truncation=True, return_tensors="pt")
        batch = {k: v.to(self._device) for k, v in batch.items()}
        with torch.no_grad():
            out = self.model(**batch)
        hidden = out.last_hidden_state
        mask = batch["attention_mask"].unsqueeze(-1).float()
        pooled = (hidden * mask).sum(dim=1) / mask.sum(dim=1).clamp(min=1e-6)
        vecs = pooled.float().cpu().numpy()
        if "nomic-embed" in self.hf_id.lower():
            norms = np.linalg.norm(vecs, axis=1, keepdims=True)
            vecs = vecs / np.clip(norms, 1e-9, None)
        return vecs

    def select(self, prompt: dict[str, Any], corpus: list[str], *, k: int = 8) -> list[str]:
        if not corpus:
            return []
        q = str(prompt.get("text") or "")
        qv = self._embed_batch([q], role="query")[0]
        cv = self._embed_batch(corpus, role="document")
        scored = sorted(
            ((float(_cos(qv, row)), ln) for row, ln in zip(cv, corpus)),
            key=lambda x: -x[0],
        )
        return [ln for _, ln in scored[:k]]
