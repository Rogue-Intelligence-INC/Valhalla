#!/usr/bin/env python3
"""Valhalla model load test — transformers / vLLM / fused (vLLM-style API).

Usage:
  python3 tools/valhalla_inference/test_model_load.py
  python3 tools/valhalla_inference/test_model_load.py --engine transformers
  python3 tools/valhalla_inference/test_model_load.py --engine vllm   # if installed
  python3 tools/valhalla_inference/test_model_load.py --try-vllm
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
sys.path.insert(0, str(HERE))

from guided import guided_mcq_postprocess, mcq_guided_prompt_suffix  # noqa: E402
from loader import ValhallaLLM, _has_vllm  # noqa: E402
from stack import stack_report  # noqa: E402

DEFAULT_MODEL = (
    REPO / "models/hf/Qwen2.5-0.5B-Instruct-Valhalla-FreeMultimodal-Fusion-v2-test-s0.10"
)
REPORTS = REPO / "reports/valhalla_inference"
FUSED_BIN = REPO / "target/release/valhalla_fused_lm"

SMOKE_PROMPTS = [
    "What is the SI unit of force? Answer in one sentence.",
    (
        "Which organelle is the site of photosynthesis?\n"
        "A) Mitochondria\nB) Ribosome\nC) Chloroplast\nD) Nucleus"
        + mcq_guided_prompt_suffix()
    ),
]


def test_fused_smoke(model_dir: Path, phase: str = "smoke") -> dict:
    if not FUSED_BIN.is_file():
        subprocess.run(
            ["cargo", "build", "-p", "hub-f64", "--release", "--bin", "valhalla_fused_lm"],
            check=True,
            cwd=REPO,
            env={**dict(__import__("os").environ), "RUSTFLAGS": "-L /opt/cuda/lib64"},
        )
    out = REPORTS / f"fused_{phase}.json"
    REPORTS.mkdir(parents=True, exist_ok=True)
    subprocess.run(
        [
            str(FUSED_BIN),
            "--phase",
            phase,
            "--model",
            str(model_dir),
            "--out",
            str(out),
        ],
        check=True,
        cwd=REPO,
    )
    doc = json.loads(out.read_text(encoding="utf-8"))
    return {
        "backend": "valhalla_fused_lm (Candle baked_lm)",
        "summary": doc.get("summary", {}).get("total"),
        "path": str(out),
    }


def main() -> None:
    ap = argparse.ArgumentParser(description="Valhalla vLLM-style load test")
    ap.add_argument("--model", type=Path, default=DEFAULT_MODEL)
    ap.add_argument("--engine", choices=("auto", "transformers", "vllm"), default="auto")
    ap.add_argument("--try-vllm", action="store_true", help="Also probe vllm if installed")
    ap.add_argument("--fused-smoke", action="store_true", help="Run valhalla_fused_lm smoke")
    ap.add_argument("--max-tokens", type=int, default=96)
    args = ap.parse_args()

    model_dir = args.model.resolve()
    print(stack_report(model_dir))
    print()

    report: dict = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "model": str(model_dir),
        "vllm_installed": _has_vllm(),
        "engines": {},
        "generations": [],
    }

    # Primary engine test
    llm = ValhallaLLM(model_dir, engine=args.engine)
    probe = llm.probe()
    report["engines"]["primary"] = probe
    print("--- Load probe ---")
    print(json.dumps(probe, indent=2, ensure_ascii=False))

    if probe.get("load_ok"):
        print("\n--- Generate (vLLM-style API) ---")
        results = llm.generate(SMOKE_PROMPTS, max_tokens=args.max_tokens)
        for r in results:
            row = {"prompt": r.prompt[:120], "backend": r.backend, "output": r.text[:400]}
            if "ANSWER:" in r.prompt or "A)" in r.prompt:
                g = guided_mcq_postprocess(r.text)
                row["guided_mcq"] = g.letter
            report["generations"].append(row)
            print(f"\n[{r.backend}] {r.prompt[:80]}...")
            print(f"  → {r.text[:300]}")

    if args.try_vllm and _has_vllm() and args.engine != "vllm":
        print("\n--- vLLM optional probe ---")
        vllm_llm = ValhallaLLM(model_dir, engine="vllm")
        vprobe = vllm_llm.probe()
        report["engines"]["vllm"] = vprobe
        print(json.dumps(vprobe, indent=2, ensure_ascii=False))
        if vprobe.get("load_ok"):
            vout = vllm_llm.generate([SMOKE_PROMPTS[0]], max_tokens=64)
            report["generations"].append(
                {"prompt": SMOKE_PROMPTS[0], "backend": "vllm", "output": vout[0].text[:400]}
            )
            print(f"  vllm → {vout[0].text[:200]}")
    elif args.try_vllm and not _has_vllm():
        report["engines"]["vllm"] = {"skipped": "pip install vllm to enable"}
        print("\n(vLLM not installed — baked HF still loads via transformers / vllm serve <dir>)")

    if args.fused_smoke:
        print("\n--- valhalla_fused_lm (Rust Candle) ---")
        fused = test_fused_smoke(model_dir)
        report["engines"]["fused"] = fused
        print(json.dumps(fused, indent=2))

    REPORTS.mkdir(parents=True, exist_ok=True)
    out_path = REPORTS / "model_load_test.json"
    out_path.write_text(json.dumps(report, indent=2, ensure_ascii=False), encoding="utf-8")
    print(f"\n→ {out_path}")

    if not probe.get("load_ok"):
        sys.exit(1)


if __name__ == "__main__":
    main()
