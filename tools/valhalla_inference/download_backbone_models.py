#!/usr/bin/env python3
"""Download cross-architecture LM backbones for universality tests.

Default pair:
  - google/gemma-2-2b-it          (~5.2 GB)
  - microsoft/Phi-3-mini-4k-instruct (~7.6 GB)

Endpoints (auto if --endpoint auto):
  - https://hf-mirror.com     (China mirror — often works when huggingface.co is blocked)
  - https://huggingface.co    (official)

Usage:
  .venv-llm/bin/python3 tools/valhalla_inference/download_backbone_models.py --mirror
  HF_ENDPOINT=https://hf-mirror.com .venv-llm/bin/python3 tools/valhalla_inference/download_backbone_models.py
  .venv-llm/bin/python3 tools/valhalla_inference/download_backbone_models.py --endpoint modelscope --only phi
"""

from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
DEST = REPO / "models/hf"

SIZE_GB = {
    "google/gemma-2-2b-it": 5.2,
    "microsoft/Phi-3-mini-4k-instruct": 7.6,
    "meta-llama/Llama-3.2-1B-Instruct": 2.4,
    "HuggingFaceTB/SmolLM2-360M-Instruct": 0.7,
}

DEFAULT_MODELS = [
    "google/gemma-2-2b-it",
    "microsoft/Phi-3-mini-4k-instruct",
]

# ModelScope ids (fallback when HF hub unreachable)
MODELSCOPE_MAP = {
    "microsoft/Phi-3-mini-4k-instruct": "LLM-Research/Phi-3-mini-4k-instruct",
    "google/gemma-2-2b-it": "google/gemma-2-2b-it",
    "Qwen/Qwen2.5-0.5B-Instruct": "Qwen/Qwen2.5-0.5B-Instruct",
    "Qwen/Qwen2.5-3B-Instruct": "Qwen/Qwen2.5-3B-Instruct",
}

HF_ENDPOINTS = [
    "https://hf-mirror.com",
    "https://huggingface.co",
]


def _free_gb(path: Path) -> float:
    return shutil.disk_usage(path).free / (1024**3)


def _probe_endpoint(url: str, timeout: float = 8.0) -> bool:
    import urllib.request

    try:
        req = urllib.request.Request(f"{url.rstrip('/')}/", method="HEAD")
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            return resp.status < 500
    except Exception:
        return False


def pick_endpoint(preferred: str) -> str:
    if preferred not in ("auto", "modelscope"):
        return preferred.rstrip("/")
    for url in HF_ENDPOINTS:
        if _probe_endpoint(url):
            print(f"Auto-selected endpoint: {url}")
            return url
    raise RuntimeError(
        "No HuggingFace endpoint reachable. Try: --endpoint https://hf-mirror.com "
        "or --endpoint modelscope"
    )


def _hub_cache_snap(repo_id: str) -> Path | None:
    slug = "models--" + repo_id.replace("/", "--")
    root = Path.home() / ".cache/huggingface/hub" / slug / "snapshots"
    if not root.is_dir():
        return None
    for snap in sorted(root.iterdir(), reverse=True):
        if _weights_ok(snap):
            return snap
    return None


def _modelscope_cache_snap(repo_id: str) -> Path | None:
    parts = repo_id.split("/")
    if len(parts) != 2:
        return None
    ms_id = MODELSCOPE_MAP.get(repo_id, repo_id)
    ms_parts = ms_id.split("/")
    if len(ms_parts) == 2:
        root = Path.home() / ".cache/modelscope/hub" / "models" / ms_parts[0] / ms_parts[1]
        if root.is_dir() and _weights_ok(root):
            return root
    root = Path.home() / ".cache/modelscope/hub" / "models" / parts[0] / parts[1]
    if root.is_dir() and _weights_ok(root):
        return root
    return None


def _weights_ok(d: Path) -> bool:
    if (d / "model.safetensors").is_file():
        return True
    return (d / "model.safetensors.index.json").is_file()


def download_hf(repo: str, *, endpoint: str, local_dir: Path | None) -> Path:
    os.environ["HF_ENDPOINT"] = endpoint
    # Large LFS files on hf-mirror may redirect to xethub; allow long resume.
    os.environ.setdefault("HF_HUB_DOWNLOAD_TIMEOUT", "600")
    from huggingface_hub import snapshot_download

    kwargs: dict = {"repo_id": repo, "endpoint": endpoint}
    if local_dir is not None:
        local_dir.mkdir(parents=True, exist_ok=True)
        kwargs["local_dir"] = str(local_dir)
        print(f"Downloading {repo} -> {local_dir} via {endpoint}")
    else:
        print(f"Downloading {repo} -> ~/.cache/huggingface/hub/ via {endpoint}")

    path = snapshot_download(**kwargs)
    return Path(path)


def download_modelscope(repo: str, *, cache_dir: Path | None) -> Path:
    try:
        from modelscope import snapshot_download as ms_download
    except ImportError:
        print("Installing modelscope …", flush=True)
        subprocess.check_call(
            [sys.executable, "-m", "pip", "install", "-q", "modelscope"],
        )
        from modelscope import snapshot_download as ms_download

    ms_id = MODELSCOPE_MAP.get(repo, repo)
    kwargs: dict = {"model_id": ms_id}
    if cache_dir is not None:
        kwargs["cache_dir"] = str(cache_dir)
    print(f"Downloading ModelScope {ms_id} (HF id {repo})")
    path = ms_download(**kwargs)
    return Path(path)


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--models", nargs="*", default=DEFAULT_MODELS)
    ap.add_argument("--only", choices=["gemma", "phi", "llama", "smollm"])
    ap.add_argument(
        "--endpoint",
        default="auto",
        help="HF endpoint URL, 'auto', or 'modelscope' (overrides --prefer)",
    )
    ap.add_argument(
        "--mirror",
        action="store_true",
        help="Shortcut for --endpoint https://hf-mirror.com",
    )
    ap.add_argument(
        "--local-dir",
        action="store_true",
        help="Also copy into models/hf/ (uses extra disk)",
    )
    ap.add_argument("--min-free-gb", type=float, default=14.0)
    args = ap.parse_args()

    presets = {
        "gemma": ["google/gemma-2-2b-it"],
        "phi": ["microsoft/Phi-3-mini-4k-instruct"],
        "llama": ["meta-llama/Llama-3.2-1B-Instruct"],
        "smollm": ["HuggingFaceTB/SmolLM2-360M-Instruct"],
    }
    models = presets.get(args.only, args.models)

    endpoint_arg = "https://hf-mirror.com" if args.mirror else args.endpoint

    need_gb = sum(SIZE_GB.get(m, 4.0) for m in models)
    free = _free_gb(Path.home())
    print(f"Disk: {free:.1f} GB free · estimated download ~{need_gb:.1f} GB")
    if free < need_gb + 2:
        print("WARNING: tight disk — omit --local-dir to use HF cache only")

    results: list[tuple[str, Path]] = []
    use_ms = endpoint_arg == "modelscope"

    if not use_ms:
        endpoint = pick_endpoint(endpoint_arg)
    else:
        endpoint = "modelscope"

    for repo in models:
        name = repo.split("/")[-1]
        local = (DEST / name) if args.local_dir else None

        # Skip if already in cache
        existing = None
        if local and local.is_dir() and _weights_ok(local):
            existing = local
        else:
            existing = _hub_cache_snap(repo) or _modelscope_cache_snap(repo)
        if existing is not None:
            print(f"SKIP {repo} — already at {existing}\n")
            results.append((repo, existing))
            continue

        try:
            if use_ms:
                path = download_modelscope(repo, cache_dir=local)
            else:
                path = download_hf(repo, endpoint=endpoint, local_dir=local)
        except Exception as e:
            print(f"FAILED {repo} via {endpoint}: {e}")
            if not use_ms and endpoint != "https://hf-mirror.com":
                print("Retrying via hf-mirror.com …")
                path = download_hf(
                    repo, endpoint="https://hf-mirror.com", local_dir=local
                )
            elif not use_ms:
                print("Retrying via ModelScope …")
                path = download_modelscope(repo, cache_dir=local)
            else:
                raise

        if not _weights_ok(path):
            raise FileNotFoundError(f"Missing weights under {path}")
        print(f"  OK: {path}\n")
        results.append((repo, path))

    print("=== Download complete ===")
    for repo, path in results:
        print(f"  {repo}\n    -> {path}")
    print("\nVerify:")
    print("  python3 -c \"import sys; sys.path.insert(0,'tools/valhalla_inference');")
    print("  from test_context_arch_compare import resolve_lm_path;")
    print("  print(resolve_lm_path('microsoft/Phi-3-mini-4k-instruct'))\"")


if __name__ == "__main__":
    main()
