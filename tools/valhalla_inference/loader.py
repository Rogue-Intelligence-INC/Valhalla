"""Valhalla vLLM-style model loader — multi-backend (transformers / vllm / fused)."""

from __future__ import annotations

import json
import shutil
import subprocess
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Literal

from stack import ValhallaModelInfo, inspect_model

Engine = Literal["auto", "transformers", "vllm", "fused"]

REPO = Path(__file__).resolve().parents[2]
FUSED_BIN = REPO / "target/release/valhalla_fused_lm"


@dataclass
class GenerateResult:
    prompt: str
    text: str
    backend: str
    meta: dict[str, Any] = field(default_factory=dict)


def _has_vllm() -> bool:
    try:
        import vllm  # noqa: F401

        return True
    except ImportError:
        return bool(shutil.which("vllm"))


def _validate_hf_dir(model_dir: Path) -> None:
    st = model_dir / "model.safetensors"
    idx = model_dir / "model.safetensors.index.json"
    if idx.is_file():
        return
    if st.is_file() and st.stat().st_size >= 1024:
        return
    raise ValueError(f"Invalid or empty weights under {model_dir}")
    cfg = model_dir / "config.json"
    if not cfg.is_file() or cfg.stat().st_size == 0:
        raise ValueError(f"Missing config.json in {model_dir}")


class ValhallaLLM:
    """vLLM-like API for Valhalla-baked (or base) causal LMs.

    Usage:
        llm = ValhallaLLM("models/hf/Qwen2.5-0.5B-Instruct-Valhalla-FreeMultimodal-Fusion-v2-test-s0.10")
        out = llm.generate(["What is the SI unit of force?"])
    """

    def __init__(
        self,
        model: str | Path,
        *,
        engine: Engine = "auto",
        trust_remote_code: bool = True,
        dtype: str = "auto",
        max_model_len: int = 4096,
    ) -> None:
        self.model_dir = Path(model).resolve()
        self.info: ValhallaModelInfo = inspect_model(self.model_dir)
        self.engine = self._resolve_engine(engine)
        self.trust_remote_code = trust_remote_code
        self.dtype = dtype
        self.max_model_len = max_model_len
        self._backend: Any = None
        self._tokenizer: Any = None

    def _resolve_engine(self, engine: Engine) -> Engine:
        if engine != "auto":
            return engine
        if self.info.load_mode == "baked_hf":
            return "transformers"
        if _has_vllm():
            return "vllm"
        return "transformers"

    @property
    def backend_name(self) -> str:
        return self.engine

    def load(self) -> None:
        _validate_hf_dir(self.model_dir)
        if self.engine == "vllm":
            self._load_vllm()
        elif self.engine == "fused":
            self._load_fused_stub()
        else:
            self._load_transformers()

    def _load_transformers(self) -> None:
        import torch
        from transformers import AutoConfig, AutoModelForCausalLM, AutoTokenizer

        self._tokenizer = AutoTokenizer.from_pretrained(
            str(self.model_dir), trust_remote_code=self.trust_remote_code
        )
        torch_dtype = torch.float16 if torch.cuda.is_available() else torch.float32
        if self.dtype == "float32":
            torch_dtype = torch.float32
        cfg = AutoConfig.from_pretrained(str(self.model_dir))
        # Prefer built-in architecture (avoids stale remote modeling_phi3.py on Phi-3).
        use_remote = self.trust_remote_code and cfg.model_type not in (
            "phi3",
            "gemma2",
            "llama",
            "qwen2",
        )
        self._backend = AutoModelForCausalLM.from_pretrained(
            str(self.model_dir),
            torch_dtype=torch_dtype,
            device_map="auto" if torch.cuda.is_available() else None,
            trust_remote_code=use_remote,
            attn_implementation="eager",
        )
        self._backend.eval()
        self.engine = "transformers"

    def _load_vllm(self) -> None:
        if not _has_vllm():
            raise RuntimeError("vllm not installed; pip install vllm or use engine='transformers'")
        from vllm import LLM

        self._backend = LLM(
            model=str(self.model_dir),
            trust_remote_code=self.trust_remote_code,
            max_model_len=self.max_model_len,
        )
        self.engine = "vllm"

    def _load_fused_stub(self) -> None:
        if not FUSED_BIN.is_file():
            raise RuntimeError(f"Build fused bin: cargo build -p hub-f64 --release --bin valhalla_fused_lm")
        self.engine = "fused"

    def _format_chat(self, prompt: str) -> str:
        if self._tokenizer is None:
            from transformers import AutoTokenizer

            self._tokenizer = AutoTokenizer.from_pretrained(
                str(self.model_dir), trust_remote_code=self.trust_remote_code
            )
        messages = [{"role": "user", "content": prompt}]
        return self._tokenizer.apply_chat_template(
            messages, tokenize=False, add_generation_prompt=True
        )

    def generate(
        self,
        prompts: list[str],
        *,
        max_tokens: int = 128,
        temperature: float = 0.0,
    ) -> list[GenerateResult]:
        if self._backend is None and self.engine != "fused":
            self.load()

        if self.engine == "vllm":
            from vllm import SamplingParams

            sp = SamplingParams(max_tokens=max_tokens, temperature=temperature)
            outs = self._backend.generate(prompts, sp)
            return [
                GenerateResult(
                    prompt=p,
                    text=o.outputs[0].text.strip(),
                    backend="vllm",
                    meta={"finish_reason": o.outputs[0].finish_reason},
                )
                for p, o in zip(prompts, outs)
            ]

        if self.engine == "fused":
            return self._generate_fused(prompts)

        import torch

        results: list[GenerateResult] = []
        for prompt in prompts:
            text_in = self._format_chat(prompt)
            inputs = self._tokenizer(text_in, return_tensors="pt")
            device = next(self._backend.parameters()).device
            inputs = {k: v.to(device) for k, v in inputs.items()}
            with torch.no_grad():
                out = self._backend.generate(
                    **inputs,
                    max_new_tokens=max_tokens,
                    do_sample=temperature > 0,
                    temperature=temperature if temperature > 0 else None,
                    pad_token_id=self._tokenizer.eos_token_id,
                )
            gen = self._tokenizer.decode(
                out[0][inputs["input_ids"].shape[1] :], skip_special_tokens=True
            )
            results.append(
                GenerateResult(prompt=prompt, text=gen.strip(), backend="transformers")
            )
        return results

    def _generate_fused(self, prompts: list[str]) -> list[GenerateResult]:
        raise NotImplementedError(
            "Use test_model_load.py --fused for batch fair eval via valhalla_fused_lm"
        )

    def probe(self) -> dict[str, Any]:
        """Load smoke — return stack info without keeping model in memory."""
        out: dict[str, Any] = {
            "info": self.info.to_dict(),
            "engine_requested": self.engine,
            "vllm_available": _has_vllm(),
        }
        try:
            _validate_hf_dir(self.model_dir)
            out["weights_ok"] = True
        except ValueError as e:
            out["weights_ok"] = False
            out["weights_error"] = str(e)
            return out
        if self.engine == "vllm" and not _has_vllm():
            out["load_skipped"] = "vllm not installed"
            return out
        try:
            self.load()
            if self.engine == "transformers" and self._backend is not None:
                nparams = sum(p.numel() for p in self._backend.parameters())
                out["param_count_M"] = round(nparams / 1e6, 1)
                out["device"] = str(next(self._backend.parameters()).device)
            elif self.engine == "vllm":
                out["vllm_loaded"] = True
            out["load_ok"] = True
        except Exception as e:
            out["load_ok"] = False
            out["load_error"] = str(e)
        return out
