#!/usr/bin/env python3
"""Run industry / third-party benchmark suite (max coverage).

Default backbone: Qwen2.5-3B-Instruct + CUDA GPU (see benchmark_env.py).

Protocols included:
  ragtruth-v1, ragtruth-full-v1, needle-session-v1, longbench-v2-v1
  frames-v1, multihop-rag-v1, locomo-v1, lara-v1, ruler-v1
  external-holdout-v1 (GSM8K/MMLU AGI proxy), mcq-ladder-v1 @ 3B

Usage:
  python3 tools/valhalla_inference/run_industry_benchmarks.py --fast
  python3 tools/valhalla_inference/run_industry_benchmarks.py --download-first --all
  python3 tools/valhalla_inference/run_industry_benchmarks.py --aggregate-only
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

from benchmark_env import apply_gpu_env, default_lm_model  # noqa: E402
REPORTS = REPO / "reports/valhalla_inference"
PY = sys.executable
VENV_PY = REPO / ".venv-llm" / "bin" / "python3"
VB_BIN = REPO / "target" / "release" / "valhalla_base"

INDUSTRY_TESTS = [
    ("needle", None, "rust"),  # valhalla_base demo needle
    ("ragtruth", "test_ragtruth_v1.py", []),
    ("ragtruth-full", "test_ragtruth_full_v1.py", []),
    ("longbench", "test_longbench_v2_v1.py", []),
    ("frames", "test_frames_v1.py", []),
    ("multihop", "test_multihop_rag_v1.py", ["--hybrid", "--limit", "25"]),
    ("locomo", "test_locomo_v1.py", []),
    ("ruler", "test_ruler_v1.py", []),
    ("lara", "test_lara_v1.py", ["--with-trad"]),
    ("external-holdout", "test_external_holdout.py", []),
]

AGI_TRAINING_TESTS = [
    ("TPI-v2", "test_valhalla_base_training_potential.py", ["--protocol", "v2"]),
    ("fair-benchmark", str(REPO / "tools/fair_benchmark/run_fair_benchmark.py"), ["--phase", "test"]),
    ("incubation-standard", str(REPO / "tools/valhalla_model_bridge/run_incubation_benchmark.py"), ["--phase", "standard"]),
]

ROADMAP_PENDING = [
    "U-NIAH multi-needle (extend ruler-v2)",
    "RAGBench end-to-end (galileo-ai/ragbench)",
    "FRAMES full 824Q + wiki fetch",
    "LoCoMo full 10 conversations all QA",
    "Qwen 7B MCQ ladder measured",
    "RAGAS reference-free on production traces",
]


def ensure_valhalla_base(extra_env: dict[str, str]) -> Path:
    if VB_BIN.is_file():
        return VB_BIN
    print("Building valhalla_base (Rust)...")
    rc = subprocess.call(
        ["cargo", "build", "-p", "hub-f64", "--release", "--bin", "valhalla_base"],
        cwd=REPO,
        env={**os.environ, **extra_env},
    )
    if rc or not VB_BIN.is_file():
        raise RuntimeError("failed to build valhalla_base")
    return VB_BIN


def run_rust_demo(label: str, demo: str, flags: list[str], extra_env: dict[str, str]) -> int:
    vb = ensure_valhalla_base(extra_env)
    return run_cmd(label, [str(vb), "demo", demo, *flags], extra_env=extra_env)


def run_cmd(label: str, cmd: list[str], *, extra_env: dict[str, str] | None = None) -> int:
    print(f"\n{'=' * 64}\n[{label}]\n$ {' '.join(cmd)}\n")
    env = {**os.environ, **(extra_env or {})}
    return subprocess.call(cmd, cwd=REPO, env=env)


def load_json(name: str) -> dict | None:
    p = REPORTS / name
    return json.loads(p.read_text(encoding="utf-8")) if p.is_file() else None


def aggregate() -> dict:
    entries: list[dict] = []

    mapping = [
        ("ragtruth_v1_test.json", "RAGTruth", "faithful_rate", "faithful", "n", "verdict"),
        ("ragtruth_full_v1_test.json", "RAGTruth-100", "faithful_rate", "faithful", "n", "verdict"),
        ("needle_session_v1_test.json", "Needle session", "recall", "correct", "n", "verdict"),
        ("longbench_v2_v1_test.json", "LongBench-v2", "accuracy", "correct", "n", "verdict"),
        ("frames_v1_test.json", "FRAMES", "accuracy", "correct", "n", "verdict"),
        ("multihop_rag_v1_test.json", "MultiHop-RAG", "accuracy", "correct", "n", "verdict"),
        ("multihop_rag_v2_test.json", "MultiHop-RAG v2", "accuracy", "correct", "n", "verdict"),
        ("multihop_rag_fate_test.json", "MultiHop-RAG Fate", "accuracy", "correct", "n", "verdict"),
        ("multihop_rag_hybrid_test.json", "MultiHop-RAG hybrid", "accuracy", "correct", "n", "verdict"),
        ("locomo_v1_test.json", "LoCoMo", "accuracy", "correct", "n", "verdict"),
        ("ruler_v1_test.json", "RULER-style", "recall", "correct", "n", "verdict"),
        ("lara_v1_test.json", "LaRA RAG vs LC", None, None, "n", "verdict"),
        ("mcq_ladder_v1_test.json", "MCQ ladder", None, None, "mcq_n", "verdict"),
        ("external_holdout_test.json", "External holdout", "accuracy", "correct", "n", "verdict"),
        ("valhalla_base_training_potential_test_v2.json", "TPI-v2", "tpi", None, "eval_count", "verdict"),
    ]

    for fname, name, rate_k, num_k, n_k, v_k in mapping:
        doc = load_json(fname)
        if not doc:
            continue
        if fname == "lara_v1_test.json":
            rag = doc.get("valhalla_rag") or {}
            result = f"RAG {rag.get('correct', '?')}/{doc.get('n', '?')}"
            lc = doc.get("trad_long_context")
            if lc:
                result += f" · LC {lc.get('correct', '?')}/{doc.get('n', '?')}"
        elif fname == "mcq_ladder_v1_test.json":
            pts = (doc.get("hybrid_lm_patch_scaling") or {}).get("points") or []
            result = " → ".join(
                f"{p.get('params_b')}B {100*(p.get('accuracy') or 0):.0f}%"
                for p in pts
                if p.get("accuracy") is not None
            )
        elif fname == "valhalla_base_training_potential_test_v2.json":
            tpi = (doc.get("tpi") or {}).get("tpi")
            result = f"TPI {tpi:.3f}" if tpi is not None else "—"
        else:
            n = doc.get(n_k, "?")
            num = doc.get(num_k, "?") if num_k else "?"
            rate = doc.get(rate_k) if rate_k else None
            if rate is not None:
                result = f"{num}/{n} ({100*float(rate):.0f}%)"
            else:
                result = f"{num}/{n}"
        entries.append(
            {
                "name": name,
                "protocol": doc.get("protocol"),
                "result": result,
                "verdict": doc.get(v_k),
                "file": fname,
            }
        )

    return {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "benchmarks": entries,
        "pending": ROADMAP_PENDING,
        "report_dir": str(REPORTS),
    }


def write_summary(results: dict) -> Path:
    REPORTS.mkdir(parents=True, exist_ok=True)
    out_json = REPORTS / "industry_benchmarks_summary.json"
    out_md = REPORTS / "INDUSTRY_BENCHMARKS_20260630.md"
    out_json.write_text(json.dumps(results, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")

    lines = [
        "# Industry benchmarks — ValhallaBase",
        "",
        f"**Generated:** {results.get('timestamp', '')}",
        "",
        "## Results",
        "",
        "| Benchmark | Protocol | Result | Verdict |",
        "|-----------|----------|--------|---------|",
    ]
    for r in results.get("benchmarks") or []:
        lines.append(
            f"| {r['name']} | `{r.get('protocol', '—')}` | **{r['result']}** | {r.get('verdict', '—')} |"
        )
    lines.extend(["", "## Roadmap (pending)", ""])
    for p in results.get("pending") or []:
        lines.append(f"- {p}")
    lines.extend(
        [
            "",
            "## Run",
            "",
            "```bash",
            "HF_ENDPOINT=https://hf-mirror.com .venv-llm/bin/python3 tools/standard_benchmarks/download_standard_benchmarks.py --all",
            "RUSTFLAGS=\"-L /opt/cuda/lib64\" python3 tools/valhalla_inference/run_industry_benchmarks.py",
            "```",
            "",
            "---",
            "",
            "*Rogue Intelligence LNC.*",
        ]
    )
    out_md.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return out_md


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--fast", action="store_true")
    ap.add_argument("--download-first", action="store_true")
    ap.add_argument("--offline-download", action="store_true", help="reuse download cache only")
    ap.add_argument("--all", action="store_true", help="include mcq-ladder @ 3B")
    ap.add_argument("--agi", action="store_true", help="include AGI/training proxies (TPI, fair-benchmark, incubation)")
    ap.add_argument("--agi-only", action="store_true", help="run AGI/training proxies only (skip industry suite)")
    ap.add_argument("--skip-scaling", action="store_true")
    ap.add_argument("--aggregate-only", action="store_true")
    ap.add_argument("--skip", action="append", dest="skip", default=[])
    args = ap.parse_args()

    if args.aggregate_only:
        md = write_summary(aggregate())
        print(f"→ {md}")
        return

    if args.download_first:
        dl_py = VENV_PY if VENV_PY.is_file() else PY
        dl_cmd = [str(dl_py), str(REPO / "tools/standard_benchmarks/download_standard_benchmarks.py"), "--all"]
        if args.offline_download:
            dl_cmd.append("--offline")
        dl_env = {
            **os.environ,
            "HF_ENDPOINT": os.environ.get("HF_ENDPOINT", "https://hf-mirror.com"),
        }
        rc = run_cmd("download-bundles", dl_cmd, extra_env=dl_env)
        if rc:
            sys.exit("download failed")

    flags = ["--fast"] if args.fast else []
    gpu_env = apply_gpu_env()
    print(f"Backbone: {default_lm_model()}  GPU: VALHALLA_USE_CPU={gpu_env.get('VALHALLA_USE_CPU')}")
    failed = 0

    if not args.agi_only:
        for label, script, extra in INDUSTRY_TESTS:
            if label in args.skip:
                continue
            if script is None and extra == "rust":
                rc = run_rust_demo(label, label, flags, gpu_env)
            else:
                py = VENV_PY if label == "lara" and VENV_PY.is_file() else PY
                rc = run_cmd(label, [str(py), str(HERE / script), *flags, *extra], extra_env=gpu_env)
            if rc:
                failed += 1

        if args.all and not args.skip_scaling:
            rc = run_cmd(
                "mcq-ladder",
                [str(PY), str(HERE / "test_mcq_ladder_v1.py"), *flags],
                extra_env=gpu_env,
            )
            if rc:
                failed += 1

    if args.agi or args.agi_only:
        agi_py = VENV_PY if VENV_PY.is_file() else PY
        for label, script, extra in AGI_TRAINING_TESTS:
            if label in args.skip:
                continue
            script_path = Path(script)
            if not script_path.is_absolute():
                script_path = HERE / script
            rc = run_cmd(label, [str(agi_py), str(script_path), *flags, *extra], extra_env=gpu_env)
            if rc:
                failed += 1

    md = write_summary(aggregate())
    print(f"\n{'=' * 64}\nSummary → {md}")
    if failed:
        sys.exit(f"{failed} industry benchmark step(s) failed")
    print("Industry benchmark suite completed.")


if __name__ == "__main__":
    main()
