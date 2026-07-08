#!/usr/bin/env python3
"""Run industry / third-party benchmark suite (max coverage).

Default backbone: Qwen2.5-3B-Instruct + CUDA GPU (see benchmark_env.py).

Protocols included:
  ragtruth-v1, ragtruth-full-v1, needle-session-v1, longbench-v2-v1
  frames-v1, multihop-rag-v1, locomo-v1, lara-v1, ruler-v1
  mcq-ladder-v1 @ 3B (serve mode, single model)

Usage:
  python3 tools/valhalla_inference/run_industry_benchmarks.py --fast
  python3 tools/valhalla_inference/run_industry_benchmarks.py --download-first --all
  python3 tools/valhalla_inference/run_industry_benchmarks.py --aggregate-only
  python3 tools/valhalla_inference/run_industry_benchmarks.py --all --verbose --checkpoint-dir reports/valhalla_inference/checkpoints
  python3 tools/valhalla_inference/run_industry_benchmarks.py --resume --pause-file /tmp/vb_pause
  touch /tmp/vb_pause   # graceful pause between benchmarks
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

from benchmark_cli import (  # noqa: E402
    ProgressBar,
    init_suite_checkpoint,
    mark_suite_done,
    pause_requested,
    save_suite_checkpoint,
    suite_completed_labels,
)
from benchmark_env import apply_gpu_env, benchmark_runtime, default_lm_model  # noqa: E402

REPORTS = REPO / "reports/valhalla_inference"
CHECKPOINTS_DEFAULT = REPORTS / "checkpoints"
SUITE_CHECKPOINT = CHECKPOINTS_DEFAULT / "industry_suite_checkpoint.json"
PY = sys.executable
VENV_PY = REPO / ".venv-llm" / "bin" / "python3"
VB_BIN = REPO / "target" / "release" / "valhalla_base"

INDUSTRY_TESTS = [
    ("needle", None, "rust"),
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

# Per-benchmark report filenames (for suite checkpoint metadata)
REPORT_FILES: dict[str, str] = {
    "needle": "needle_session_v1_test.json",
    "ragtruth": "ragtruth_v1_test.json",
    "ragtruth-full": "ragtruth_full_v1_test.json",
    "longbench": "longbench_v2_v1_test.json",
    "frames": "frames_v1_test.json",
    "multihop": "multihop_rag_hybrid_test.json",
    "locomo": "locomo_v1_test.json",
    "ruler": "ruler_v1_test.json",
    "lara": "lara_v1_test.json",
    "external-holdout": "external_holdout_test.json",
    "mcq-ladder": "mcq_ladder_v1_test.json",
}


def ensure_valhalla_base(extra_env: dict[str, str]) -> Path:
    if VB_BIN.is_file():
        return VB_BIN
    print("Building valhalla_base (Rust)...", flush=True)
    rc = subprocess.call(
        ["cargo", "build", "-p", "hub-f64", "--release", "--bin", "valhalla_base"],
        cwd=REPO,
        env={**os.environ, **extra_env},
    )
    if rc or not VB_BIN.is_file():
        raise RuntimeError("failed to build valhalla_base")
    return VB_BIN


def bench_flags(args: argparse.Namespace, label: str, *, rust: bool = False) -> list[str]:
    """Build per-benchmark flags: fast, checkpoint (Rust demos only), pause."""
    flags: list[str] = []
    if args.fast:
        flags.append("--fast")
    # Suite-level checkpoint handles resume between benchmarks; in-run checkpoint is Rust-only.
    if rust and args.checkpoint_dir:
        ckpt = args.checkpoint_dir / f"{label}.checkpoint.json"
        flags.extend(["--checkpoint", str(ckpt), "--checkpoint-every", str(args.checkpoint_every)])
        if args.verbose:
            flags.append("--verbose")
        if args.resume and ckpt.is_file():
            flags.extend(["--resume", str(ckpt)])
    if args.pause_file:
        flags.extend(["--pause-file", str(args.pause_file)])
    return flags


def run_cmd(label: str, cmd: list[str], *, extra_env: dict[str, str] | None = None, verbose: bool = False) -> int:
    print(f"\n{'=' * 64}\n[{label}]\n$ {' '.join(cmd)}\n", flush=True)
    env = {**os.environ, **(extra_env or {})}
    t0 = time.monotonic()
    if verbose:
        return subprocess.call(cmd, cwd=REPO, env=env)
    proc = subprocess.run(cmd, cwd=REPO, env=env, capture_output=True, text=True)
    if proc.stdout:
        print(proc.stdout, end="", flush=True)
    if proc.stderr:
        print(proc.stderr, end="", file=sys.stderr, flush=True)
    elapsed = time.monotonic() - t0
    print(f"[{label}] exit={proc.returncode} elapsed={elapsed:.1f}s", flush=True)
    return proc.returncode


def run_rust_demo(
    label: str,
    demo: str,
    flags: list[str],
    extra_env: dict[str, str],
    *,
    verbose: bool,
) -> int:
    vb = ensure_valhalla_base(extra_env)
    if not verbose:
        flags = [*flags, "--verbose"]  # Rust progress bar on stderr
    return run_cmd(label, [str(vb), "demo", demo, *flags], extra_env=extra_env, verbose=verbose)


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
            "RUSTFLAGS=\"-L /opt/cuda/lib64\" python3 tools/valhalla_inference/run_industry_benchmarks.py --all --verbose",
            "```",
            "",
            "---",
            "",
            "*Rogue Intelligence LNC.*",
        ]
    )
    out_md.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return out_md


def build_test_list(args: argparse.Namespace) -> list[tuple[str, str | None, str | list]]:
    tests: list[tuple[str, str | None, str | list]] = []
    if not args.agi_only:
        tests.extend(INDUSTRY_TESTS)
        if args.all and not args.skip_scaling:
            tests.append(("mcq-ladder", "test_mcq_ladder_v1.py", ["--model", args.model]))
    if args.agi or args.agi_only:
        tests.extend(AGI_TRAINING_TESTS)
    return [(l, s, e) for l, s, e in tests if l not in args.skip]


def main() -> None:
    ap = argparse.ArgumentParser(description="ValhallaBase industry benchmark suite (3B default)")
    ap.add_argument("--fast", action="store_true")
    ap.add_argument("--download-first", action="store_true")
    ap.add_argument("--offline-download", action="store_true", help="reuse download cache only")
    ap.add_argument("--all", action="store_true", help="include mcq-ladder @ 3B (full suite)")
    ap.add_argument("--agi", action="store_true", help="include AGI/training proxies")
    ap.add_argument("--agi-only", action="store_true", help="AGI proxies only")
    ap.add_argument("--skip-scaling", action="store_true")
    ap.add_argument("--aggregate-only", action="store_true")
    ap.add_argument("--skip", action="append", dest="skip", default=[])
    ap.add_argument("--verbose", "-v", action="store_true", help="progress bars + detailed output")
    ap.add_argument("--checkpoint-dir", type=Path, default=None, help="per-benchmark checkpoint dir")
    ap.add_argument("--checkpoint-every", type=int, default=1, help="save every N items (in-benchmark)")
    ap.add_argument("--suite-checkpoint", type=Path, default=SUITE_CHECKPOINT, help="suite-level resume file")
    ap.add_argument("--resume", action="store_true", help="skip completed benchmarks (suite checkpoint)")
    ap.add_argument("--pause-file", type=Path, help="touch this file to pause between benchmarks")
    ap.add_argument("--model", default="qwen_3b", help="MCQ ladder model id (default qwen_3b)")
    args = ap.parse_args()

    if args.checkpoint_dir is None and (args.resume or args.verbose):
        args.checkpoint_dir = CHECKPOINTS_DEFAULT

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
        rc = run_cmd("download-bundles", dl_cmd, extra_env=dl_env, verbose=args.verbose)
        if rc:
            sys.exit("download failed")

    test_list = build_test_list(args)
    labels = [t[0] for t in test_list]
    suite_doc = init_suite_checkpoint(
        args.suite_checkpoint,
        model=default_lm_model(),
        tests=labels,
        resume=args.resume,
    )
    done = suite_completed_labels(suite_doc) if args.resume else set()

    failed = 0
    paused = False
    suite_start = time.monotonic()

    with benchmark_runtime() as gpu_env:
        print(
            f"Backbone: {default_lm_model()}  GPU: VALHALLA_USE_CPU={gpu_env.get('VALHALLA_USE_CPU')}"
            f"  serve={gpu_env.get('VALHALLA_BASE_URL', 'subprocess')}",
            flush=True,
        )
        if args.checkpoint_dir:
            args.checkpoint_dir.mkdir(parents=True, exist_ok=True)
            print(f"Checkpoints → {args.checkpoint_dir}", flush=True)
        if args.pause_file:
            print(f"Pause: touch {args.pause_file}", flush=True)

        pending = [t for t in test_list if t[0] not in done]
        suite_bar = ProgressBar("Industry suite", len(test_list), enabled=True)
        suite_bar.set(len(done))

        for label, script, extra in test_list:
            if label in done:
                print(f"[skip] {label} (already completed)", flush=True)
                continue

            if pause_requested(args.pause_file):
                suite_doc["paused"] = True
                save_suite_checkpoint(args.suite_checkpoint, suite_doc)
                suite_bar.finish("paused")
                print(f"\nPause requested — suite checkpoint → {args.suite_checkpoint}", flush=True)
                paused = True
                break

            flags = bench_flags(args, label, rust=(script is None and extra == "rust"))
            if script is None and extra == "rust":
                rc = run_rust_demo(label, label, flags, gpu_env, verbose=args.verbose)
            else:
                py = VENV_PY if VENV_PY.is_file() else PY
                script_path = Path(script) if script and Path(script).is_absolute() else HERE / str(script)
                extra_list = list(extra) if isinstance(extra, list) else []
                cmd = [str(py), str(script_path), *flags, *extra_list]
                rc = run_cmd(label, cmd, extra_env=gpu_env, verbose=args.verbose)

            report_file = REPORT_FILES.get(label)
            mark_suite_done(suite_doc, label, exit_code=rc, report=report_file)
            save_suite_checkpoint(args.suite_checkpoint, suite_doc)
            write_summary(aggregate())

            suite_bar.inc()
            if rc:
                failed += 1
                if not args.verbose:
                    print(f"[{label}] FAILED (continuing suite)", flush=True)

        if not paused:
            suite_bar.finish(f"done ({time.monotonic() - suite_start:.0f}s)")

    md = write_summary(aggregate())
    print(f"\n{'=' * 64}\nSummary → {md}", flush=True)
    print(f"Suite checkpoint → {args.suite_checkpoint}", flush=True)

    if paused:
        print("Suite paused — resume with: --resume", flush=True)
        sys.exit(0)
    if failed:
        sys.exit(f"{failed} industry benchmark step(s) failed")
    print("Industry benchmark suite completed.", flush=True)


if __name__ == "__main__":
    main()
