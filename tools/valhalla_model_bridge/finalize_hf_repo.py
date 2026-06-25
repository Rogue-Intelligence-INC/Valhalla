#!/usr/bin/env python3
"""Finalize a Valhalla bake directory as a complete Hugging Face model repo.

Adds README model card, .gitattributes (LFS), optional QA metrics, and verifies
`transformers` can load the checkpoint. Optional upload via `huggingface_hub`.

Usage:
  python finalize_hf_repo.py checkpoints/valhalla_free_multimodal/test_fused_v2_s0.10 \\
    --repo-id Rogue-Intelligence/Qwen2.5-0.5B-Valhalla-FreeMultimodal-v2

  python finalize_hf_repo.py models/hf/my-model --push --private
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

REQUIRED_FILES = (
    "config.json",
    "model.safetensors",
    "tokenizer.json",
    "tokenizer_config.json",
)

GITATTRIBUTES = """*.7z filter=lfs diff=lfs merge=lfs -text
*.arrow filter=lfs diff=lfs merge=lfs -text
*.bin filter=lfs diff=lfs merge=lfs -text
*.bz2 filter=lfs diff=lfs merge=lfs -text
*.ckpt filter=lfs diff=lfs merge=lfs -text
*.ftz filter=lfs diff=lfs merge=lfs -text
*.gz filter=lfs diff=lfs merge=lfs -text
*.h5 filter=lfs diff=lfs merge=lfs -text
*.joblib filter=lfs diff=lfs merge=lfs -text
*.lfs.* filter=lfs diff=lfs merge=lfs -text
*.mlmodel filter=lfs diff=lfs merge=lfs -text
*.model filter=lfs diff=lfs merge=lfs -text
*.msgpack filter=lfs diff=lfs merge=lfs -text
*.npy filter=lfs diff=lfs merge=lfs -text
*.npz filter=lfs diff=lfs merge=lfs -text
*.onnx filter=lfs diff=lfs merge=lfs -text
*.ot filter=lfs diff=lfs merge=lfs -text
*.parquet filter=lfs diff=lfs merge=lfs -text
*.pb filter=lfs diff=lfs merge=lfs -text
*.pickle filter=lfs diff=lfs merge=lfs -text
*.pkl filter=lfs diff=lfs merge=lfs -text
*.pt filter=lfs diff=lfs merge=lfs -text
*.pth filter=lfs diff=lfs merge=lfs -text
*.rar filter=lfs diff=lfs merge=lfs -text
*.safetensors filter=lfs diff=lfs merge=lfs -text
saved_model/**/* filter=lfs diff=lfs merge=lfs -text
*.tar.* filter=lfs diff=lfs merge=lfs -text
*.tar filter=lfs diff=lfs merge=lfs -text
*.tflite filter=lfs diff=lfs merge=lfs -text
*.tgz filter=lfs diff=lfs merge=lfs -text
*.wasm filter=lfs diff=lfs merge=lfs -text
*.xz filter=lfs diff=lfs merge=lfs -text
*.zip filter=lfs diff=lfs merge=lfs -text
*.zst filter=lfs diff=lfs merge=lfs -text
*tfevents* filter=lfs diff=lfs merge=lfs -text
"""


def load_manifest(model_dir: Path) -> dict[str, Any]:
    path = model_dir / "valhalla_manifest.json"
    if not path.is_file():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def load_qa_metrics(qa_report: Path | None, model_dir: Path) -> dict[str, Any] | None:
    candidates: list[Path] = []
    if qa_report is not None:
        candidates.append(qa_report)
    candidates.extend(
        [
            model_dir / "qa_report.json",
            model_dir.parent.parent / "reports/natural_multimodal_potential/fused_v2_full_test.json",
        ]
    )
    for path in candidates:
        if path.is_file():
            doc = json.loads(path.read_text(encoding="utf-8"))
            if "summary" in doc:
                return doc["summary"]
    return None


def verify_hf_loadable(model_dir: Path, *, skip: bool = False) -> None:
    if skip:
        return
    os.environ.setdefault("HF_ENDPOINT", "https://hf-mirror.com")
    import torch
    from transformers import AutoModelForCausalLM, AutoTokenizer

    tokenizer = AutoTokenizer.from_pretrained(model_dir, trust_remote_code=True)
    dtype = torch.float16 if torch.cuda.is_available() else torch.float32
    model = AutoModelForCausalLM.from_pretrained(
        model_dir,
        torch_dtype=dtype,
        device_map="auto" if torch.cuda.is_available() else None,
        trust_remote_code=True,
    )
    ids = tokenizer.encode("Hello", add_special_tokens=False)
    if ids:
        dev = next(model.parameters()).device
        with torch.no_grad():
            model(torch.tensor([ids], device=dev))
    print(f"  HF verify OK: {model_dir}", file=sys.stderr)


def write_readme(
    model_dir: Path,
    *,
    repo_id: str | None,
    manifest: dict[str, Any],
    qa: dict[str, Any] | None,
) -> None:
    bake = manifest.get("bake") or {}
    incubation = manifest.get("incubation") or {}
    free_mm = manifest.get("free_multimodal") or {}
    base_model = bake.get("base_model_id", "Qwen/Qwen2.5-0.5B-Instruct")
    product = manifest.get("product", "valhalla-baked-lm")
    strength = bake.get("strength", "?")
    baked_at = bake.get("baked_at", datetime.now(timezone.utc).isoformat())

    tags = ["valhalla", "text-generation", "qwen2"]
    if "multimodal" in product or free_mm:
        tags.append("multimodal-fusion")

    qa_block = ""
    if qa and qa.get("total"):
        t = qa["total"]
        qa_block = f"""
## Fair benchmark (145)

| Split | Score |
|-------|-------|
| Total | {t.get('correct', '?')}/{t.get('n', '?')} ({100 * float(t.get('acc', 0)):.1f}%) |
| Open | {qa.get('open', {}).get('correct', '?')}/{qa.get('open', {}).get('n', '?')} |
| MCQ | {qa.get('mcq', {}).get('correct', '?')}/{qa.get('mcq', {}).get('n', '?')} |
| Numeric | {qa.get('numeric', {}).get('correct', '?')}/{qa.get('numeric', {}).get('n', '?')} |
"""

    readme = f"""---
license: apache-2.0
base_model: {base_model}
tags:
{chr(10).join(f'  - {t}' for t in tags)}
library_name: transformers
pipeline_tag: text-generation
{('model-index:' + chr(10) + f'  - name: {repo_id}' + chr(10) + f'    results: []') if repo_id else ''}
---

# {repo_id or model_dir.name}

Valhalla-baked causal LM derived from [{base_model}](https://huggingface.co/{base_model}).

| Field | Value |
|-------|-------|
| Product | `{product}` |
| Bake mode | `{bake.get('bake_mode', 'full')}` |
| Strength | `{strength}` |
| Body | `{incubation.get('body', 'triad')}` |
| Baked at | `{baked_at}` |
| Weight delta norm | `{bake.get('weight_delta_norm', 'n/a')}` |

Structure incubation: tiles={incubation.get('tile_count', '?')}, stem={incubation.get('stem_cell_count', '?')}, signals={incubation.get('signals_emitted', '?')}.

{qa_block}

## Load (standard Hugging Face)

```python
from transformers import AutoModelForCausalLM, AutoTokenizer

model_id = "{repo_id or str(model_dir)}"
tokenizer = AutoTokenizer.from_pretrained(model_id, trust_remote_code=True)
model = AutoModelForCausalLM.from_pretrained(model_id, trust_remote_code=True)
```

## Valhalla fused LM (optional, same weights)

```bash
valhalla_fused_lm --model {model_dir} --phase test
```

`valhalla_manifest.json` documents incubation metadata; `transformers` ignores it at load time.

---
*Rogue Intelligence LNC. — Valhalla free multimodal fusion bake*
"""
    (model_dir / "README.md").write_text(readme, encoding="utf-8")


def write_hf_metadata(model_dir: Path, *, repo_id: str | None) -> None:
    meta = {
        "format": "huggingface-transformers",
        "finalized_at": datetime.now(timezone.utc).isoformat(),
        "repo_id": repo_id,
        "loadable_via": ["transformers.AutoModelForCausalLM", "valhalla_fused_lm"],
    }
    (model_dir / "hf_export.json").write_text(json.dumps(meta, indent=2), encoding="utf-8")
    (model_dir / ".gitattributes").write_text(GITATTRIBUTES, encoding="utf-8")


def ensure_required_files(model_dir: Path) -> None:
    missing = [f for f in REQUIRED_FILES if not (model_dir / f).is_file()]
    if missing:
        raise FileNotFoundError(
            f"{model_dir} is not HF-complete; missing: {', '.join(missing)}. Run bake first."
        )


def push_to_hub(model_dir: Path, repo_id: str, *, private: bool) -> str:
    os.environ.setdefault("HF_ENDPOINT", "https://hf-mirror.com")
    from huggingface_hub import HfApi, create_repo

    api = HfApi()
    create_repo(repo_id, exist_ok=True, private=private)
    api.upload_folder(
        folder_path=str(model_dir),
        repo_id=repo_id,
        repo_type="model",
        commit_message="Valhalla HF export",
    )
    url = f"https://huggingface.co/{repo_id}"
    print(f"  pushed → {url}", file=sys.stderr)
    return url


def default_hf_out_name(manifest: dict[str, Any], *, phase: str = "test") -> str:
    bake = manifest.get("bake") or {}
    base = (bake.get("base_model_id") or "Qwen/Qwen2.5-0.5B-Instruct").split("/")[-1]
    product = manifest.get("product", "valhalla-baked")
    slug = product.replace("valhalla-", "").replace("_", "-")
    strength = bake.get("strength", 0.08)
    return f"{base}-Valhalla-{slug}-{phase}-s{strength:.2f}"


def finalize_hf_repo(
    model_dir: Path,
    *,
    repo_id: str | None = None,
    qa_report: Path | None = None,
    verify: bool = True,
    push: bool = False,
    private: bool = True,
) -> dict[str, Any]:
    model_dir = model_dir.resolve()
    ensure_required_files(model_dir)
    manifest = load_manifest(model_dir)
    qa = load_qa_metrics(qa_report, model_dir)

    write_readme(model_dir, repo_id=repo_id, manifest=manifest, qa=qa)
    write_hf_metadata(model_dir, repo_id=repo_id)
    verify_hf_loadable(model_dir, skip=not verify)

    hub_url = None
    if push:
        if not repo_id:
            raise ValueError("--push requires --repo-id org/model-name")
        hub_url = push_to_hub(model_dir, repo_id, private=private)

    return {
        "model_dir": str(model_dir),
        "repo_id": repo_id,
        "hub_url": hub_url,
        "files": sorted(p.name for p in model_dir.iterdir() if p.is_file()),
    }


def copy_to_hf_models(
    src: Path,
    *,
    repo_root: Path,
    name: str | None = None,
    manifest: dict[str, Any] | None = None,
) -> Path:
    """Copy baked checkpoint into models/hf/{name} for HF hub layout."""
    manifest = manifest or load_manifest(src)
    dest_name = name or default_hf_out_name(manifest)
    dest = repo_root / "models" / "hf" / dest_name
    if dest.resolve() == src.resolve():
        return dest
    if dest.exists():
        shutil.rmtree(dest)
    shutil.copytree(src, dest)
    return dest


def main() -> None:
    ap = argparse.ArgumentParser(description="Finalize Valhalla bake as Hugging Face model repo")
    ap.add_argument("model_dir", type=Path, help="Directory with model.safetensors from bake")
    ap.add_argument("--repo-id", default=None, help="HF hub repo id, e.g. org/model-name")
    ap.add_argument("--qa-report", type=Path, default=None, help="QA JSON with summary block")
    ap.add_argument("--copy-to-models-hf", action="store_true", help="Copy to models/hf/{name}/")
    ap.add_argument("--name", default=None, help="Folder name under models/hf/")
    ap.add_argument("--skip-verify", action="store_true")
    ap.add_argument("--push", action="store_true", help="Upload to Hugging Face Hub")
    ap.add_argument("--public", action="store_true", help="Create public hub repo (default private)")
    args = ap.parse_args()

    model_dir = args.model_dir
    if args.copy_to_models_hf:
        repo = Path(__file__).resolve().parents[2]
        model_dir = copy_to_hf_models(model_dir, repo_root=repo, name=args.name)

    result = finalize_hf_repo(
        model_dir,
        repo_id=args.repo_id,
        qa_report=args.qa_report,
        verify=not args.skip_verify,
        push=args.push,
        private=not args.public,
    )
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
