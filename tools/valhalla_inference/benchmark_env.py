#!/usr/bin/env python3
"""Shared benchmark defaults — 3B backbone + CUDA GPU for ValhallaBase lm_patch."""

from __future__ import annotations

import json
import os
import subprocess
import time
import urllib.error
import urllib.request
from contextlib import contextmanager
from pathlib import Path
from typing import Iterator

DEFAULT_LM_MODEL = "Qwen/Qwen2.5-3B-Instruct"
DEFAULT_LM_MODEL_ID = "qwen_3b"
LEGACY_LM_MODEL = "Qwen/Qwen2.5-0.5B-Instruct"
DEFAULT_SERVE_PORT = 8787

REPO = Path(__file__).resolve().parents[2]
VB_BIN = REPO / "target" / "release" / "valhalla_base"

CUDA_LIB = "/opt/cuda/targets/x86_64-linux/lib"
CUDA_RUSTFLAGS = "-L /opt/cuda/lib64"


def default_lm_model() -> str:
    return os.environ.get("VALHALLA_LM_MODEL", DEFAULT_LM_MODEL).strip()


def apply_gpu_env() -> dict[str, str]:
    """Set LIBRARY_PATH / RUSTFLAGS / CPU policy for Rust Candle CUDA."""
    lib = os.environ.get("LIBRARY_PATH", "")
    if CUDA_LIB not in lib:
        lib = f"{CUDA_LIB}:{lib}" if lib else CUDA_LIB
    env = {
        **os.environ,
        "LIBRARY_PATH": lib,
        "RUSTFLAGS": os.environ.get("RUSTFLAGS", CUDA_RUSTFLAGS),
        "VALHALLA_USE_CPU": os.environ.get("VALHALLA_USE_CPU", "0"),
        "VALHALLA_LM_MODEL": default_lm_model(),
    }
    os.environ.update(env)
    return env


def production_valhalla_base_kwargs(**overrides):
    """Kwargs aligned with long-memory + targeted production stack."""
    base = {
        "lm_model": default_lm_model(),
        "decode": "hybrid",
        "mcq_decode": "lm_patch",
        "follow_up_decode": "native_follow_up_aware",
        "fate_ingress_routing": "quad_cycle",
        "fate_qa_feedback": True,
        "context_ingest": "targeted",
        "reanchor_affinity": True,
        "fate_warmup_enabled": True,
        "fate_warmup_cycles": int(os.environ.get("VALHALLA_FATE_WARMUP_CYCLES", "4")),
        "incubation_cycles": 2,
        "max_new": 128,
    }
    base.update(overrides)
    return base


def serve_enabled() -> bool:
    if os.environ.get("VALHALLA_BASE_NO_SERVE", "").strip() in ("1", "true", "yes"):
        return False
    if os.environ.get("VALHALLA_BASE_URL", "").strip():
        return False
    return True


def kill_serve_on_port(port: int | None = None) -> None:
    """Terminate any process listening on the ValhallaBase serve port."""
    port = port or int(os.environ.get("VALHALLA_BASE_PORT", str(DEFAULT_SERVE_PORT)))
    for cmd in (
        ["fuser", "-k", f"{port}/tcp"],
        ["sh", "-c", f"lsof -ti:{port} | xargs -r kill -9 2>/dev/null || true"],
    ):
        try:
            subprocess.run(cmd, capture_output=True, timeout=15, check=False)
        except (subprocess.TimeoutExpired, FileNotFoundError, OSError):
            pass
    os.environ.pop("VALHALLA_BASE_URL", None)
    time.sleep(2.0)


def _serve_force_fresh() -> bool:
    return os.environ.get("VALHALLA_SERVE_FORCE_FRESH", "").strip().lower() in ("1", "true", "yes")


def _health_ok(url: str) -> bool:
    try:
        with urllib.request.urlopen(f"{url.rstrip('/')}/health", timeout=2.0) as resp:
            return resp.status == 200
    except (urllib.error.URLError, TimeoutError, OSError):
        return False


def _wait_health(url: str, *, timeout_s: float = 180.0) -> None:
    deadline = time.monotonic() + timeout_s
    health = f"{url.rstrip('/')}/health"
    last_err = "timeout"
    while time.monotonic() < deadline:
        if _health_ok(url):
            return
        last_err = f"waiting for {health}"
        time.sleep(0.5)
    raise RuntimeError(f"valhalla_base serve not healthy at {health}: {last_err}")


def wait_infer_ready(
    url: str,
    *,
    lm_model: str | None = None,
    timeout_s: float = 600.0,
) -> None:
    """Poll /v1/infer with fate_warmup until model weights are loaded (avoids broken pipe)."""
    base = url.rstrip("/")
    payload: dict[str, object] = {
        "context": [
            "The capital of France is Paris.",
            "Paris is on the Seine river.",
        ],
        "questions": [],
        "fate_warmup_only": True,
        "config": {
            "decode": "hybrid",
            "fate_warmup_enabled": True,
            "fate_warmup_cycles": 1,
            "lm_model": lm_model or default_lm_model(),
            "patch_body": "triad",
        },
    }
    data = json.dumps(payload).encode("utf-8")
    deadline = time.monotonic() + timeout_s
    last_err = "timeout"
    while time.monotonic() < deadline:
        req = urllib.request.Request(
            f"{base}/v1/infer",
            data=data,
            headers={"Content-Type": "application/json"},
            method="POST",
        )
        try:
            with urllib.request.urlopen(req, timeout=300.0) as resp:
                raw = resp.read().decode("utf-8").strip()
                if resp.status == 200 and raw:
                    doc = json.loads(raw)
                    if doc.get("ok") is not False:
                        return
                    last_err = str(doc.get("error") or raw)
        except urllib.error.HTTPError as exc:
            last_err = exc.read().decode("utf-8", errors="replace").strip() or str(exc)
        except (urllib.error.URLError, TimeoutError, OSError, json.JSONDecodeError) as exc:
            last_err = str(exc)
        time.sleep(3.0)
    raise RuntimeError(f"valhalla_base infer not ready at {base}: {last_err}")


def _ensure_bin(env: dict[str, str]) -> Path:
    if VB_BIN.is_file():
        return VB_BIN
    subprocess.run(
        ["cargo", "build", "-p", "hub-f64", "--release", "--bin", "valhalla_base"],
        check=True,
        cwd=REPO,
        env=env,
    )
    if not VB_BIN.is_file():
        raise RuntimeError("failed to build valhalla_base")
    return VB_BIN


@contextmanager
def valhalla_serve(
    *,
    port: int | None = None,
    extra_env: dict[str, str] | None = None,
) -> Iterator[str]:
    """Start valhalla_base HTTP serve; yield base URL; tear down on exit."""
    port = port or int(os.environ.get("VALHALLA_BASE_PORT", str(DEFAULT_SERVE_PORT)))
    url = f"http://127.0.0.1:{port}"
    if _serve_force_fresh():
        kill_serve_on_port(port)
    if _health_ok(url):
        # Reuse only if we did not start it (external serve); verify with short timeout.
        try:
            import urllib.request
            with urllib.request.urlopen(f"{url.rstrip('/')}/v1/protocol", timeout=3.0) as resp:
                if resp.status == 200:
                    print(f"valhalla_serve: reusing {url}")
                    os.environ["VALHALLA_BASE_URL"] = url
                    yield url
                    return
        except (urllib.error.URLError, TimeoutError, OSError):
            print(f"valhalla_serve: stale serve at {url}, starting fresh")

    env = {**os.environ, **(extra_env or {})}
    bin_path = _ensure_bin(env)
    proc = subprocess.Popen(
        [str(bin_path), "serve", "--port", str(port)],
        cwd=REPO,
        env=env,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.PIPE,
        text=True,
    )
    try:
        _wait_health(url)
        os.environ["VALHALLA_BASE_URL"] = url
        yield url
    finally:
        if proc.poll() is None:
            proc.terminate()
            try:
                proc.wait(timeout=15)
            except subprocess.TimeoutExpired:
                proc.kill()
                proc.wait(timeout=5)
        err = proc.stderr.read() if proc.stderr else ""
        if err.strip():
            print(err, end="", flush=True)
        os.environ.pop("VALHALLA_BASE_URL", None)
        time.sleep(1.5)


@contextmanager
def benchmark_runtime(*, extra_env: dict[str, str] | None = None) -> Iterator[dict[str, str]]:
    """GPU env + optional persistent valhalla_base serve for benchmark suites."""
    env = apply_gpu_env()
    if extra_env:
        env.update(extra_env)
        os.environ.update(extra_env)
    if serve_enabled():
        with valhalla_serve(extra_env=env) as url:
            env = {**os.environ, "VALHALLA_BASE_URL": url}
            yield env
    else:
        yield env
