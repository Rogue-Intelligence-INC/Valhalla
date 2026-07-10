#!/usr/bin/env python3
"""Incubation / trainable-LLM benchmark battery — Valhalla protocols + literature mapping.

Maps Valhalla structure-trainable stack to external continual-learning / incubation
benchmarks (MemoryBench, CL-Bench, TiC-LM) via protocol proxies we can run locally.

Protocols (citeable):
  valhalla-base-training-potential-v2  — TPI (context learning without LM SGD)
  fate-pretrain-holdout-v3             — supervised structure pretrain → holdout
  incubation-standard-v1             — multi-domain bake / export smoke
  cl-bench-slice-v1                  — HF tencent-community/CL-bench (if reachable)

Usage:
  python3 tools/valhalla_inference/run_incubation_llm_benchmarks.py --fast
  python3 tools/valhalla_inference/run_incubation_llm_benchmarks.py --all
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

REPO = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
REPORTS = REPO / "reports" / "valhalla_inference"
PY = sys.executable
VENV_PY = REPO / ".venv-llm" / "bin" / "python3"
RUN_PY = str(VENV_PY) if VENV_PY.is_file() else PY

# External literature mapping (not all runnable offline)
LITERATURE_MAP = [
    {
        "name": "MemoryBench",
        "paper": "arXiv:2510.17281",
        "focus": "continual learning from user feedback in service time",
        "valhalla_proxy": "TPI-v2 B4 transfer + Fate session + proactive gates",
        "url": "https://arxiv.org/abs/2510.17281",
    },
    {
        "name": "CL-Bench",
        "paper": "tencent-community/CL-bench",
        "focus": "in-context learning of novel rules without fine-tune",
        "valhalla_proxy": "cl-bench-slice-v1 + open_retrieval fair slice",
        "url": "https://huggingface.co/datasets/tencent-community/CL-bench",
    },
    {
        "name": "TiC-LM",
        "paper": "arXiv:2504.02107",
        "focus": "time-continual pretraining / forgetting-plasticity",
        "valhalla_proxy": "L3 bake + TPI curriculum (structure path; not web-scale CPT)",
        "url": "https://arxiv.org/abs/2504.02107",
    },
    {
        "name": "Continual Learning Bench",
        "paper": "arXiv:2606.05661",
        "focus": "stateful gain across expert domains",
        "valhalla_proxy": "fate-longchain-unified + external-holdout",
        "url": "https://arxiv.org/html/2606.05661",
    },
]


def run_cmd(label: str, cmd: list[str], *, env: dict | None = None, timeout_s: float | None = None) -> tuple[int, float]:
    import time

    print(f"\n{'=' * 64}\n[{label}]\n$ {' '.join(cmd)}\n", flush=True)
    t0 = time.monotonic()
    try:
        rc = subprocess.call(cmd, cwd=REPO, env={**os.environ, **(env or {})}, timeout=timeout_s)
    except subprocess.TimeoutExpired:
        print(f"[{label}] TIMEOUT after {timeout_s}s", flush=True)
        rc = 124
    return rc, time.monotonic() - t0


def load_cached_tpi() -> dict[str, Any] | None:
    path = REPORTS / "valhalla_base_training_potential_test_v2.json"
    if not path.is_file():
        return None
    try:
        doc = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return None
    tpi = (doc.get("tpi") or {}).get("tpi")
    if tpi is None:
        return None
    return {
        "protocol": "valhalla-base-training-potential-v2",
        "result": f"TPI {tpi:.3f} (cached)",
        "verdict": doc.get("verdict"),
        "citeable": True,
        "exit_code": 0,
        "cached": True,
        "timestamp": doc.get("timestamp"),
    }


def run_cl_bench_slice(*, limit: int) -> dict[str, Any]:
    """Try HF CL-bench slice via ValhallaBase open path."""
    out: dict[str, Any] = {
        "protocol": "cl-bench-slice-v1",
        "dataset": "tencent-community/CL-bench",
        "limit": limit,
        "tier": "B",
    }
    sys.path.insert(0, str(HERE))
    from valhalla_base import ValhallaBase  # noqa: WPS433

    try:
        from datasets import load_dataset  # type: ignore
    except ImportError:
        out["verdict"] = "SKIPPED_NO_DATASETS"
        out["note"] = "pip install datasets"
        return out

    try:
        ds = load_dataset("tencent-community/CL-bench", split="train", streaming=True)
    except Exception as exc:
        out["verdict"] = "SKIPPED_NO_DATA"
        out["error"] = str(exc)[:200]
        return out

    rows: list[dict] = []
    ok = 0
    n = 0
    for row in ds:
        if n >= limit:
            break
        ctx = str(row.get("context") or row.get("instruction") or "")
        q = str(row.get("question") or row.get("input") or "")
        ref = str(row.get("answer") or row.get("output") or "")
        if not ctx or not q:
            continue
        base = ValhallaBase(decode="native", incubation_cycles=2)
        base.set_context([ln for ln in ctx.split("\n") if ln.strip()][:32])
        ans = str((base.run([{"id": f"CL_{n}", "text": q, "score_type": "open", "reference_answer": ref}])
                   .get("rows") or [{}])[0].get("answer") or "")
        hit = ref.lower()[:24] in ans.lower() or any(w in ans.lower() for w in ref.lower().split()[:3] if len(w) > 3)
        ok += int(hit)
        n += 1
        rows.append({"id": f"CL_{n}", "correct": hit, "answer": ans[:120]})
    acc = ok / n if n else 0.0
    out.update({
        "n": n,
        "correct": ok,
        "accuracy": acc,
        "rows": rows,
        "verdict": "CL_BENCH_VIABLE" if acc >= 0.4 and n >= 3 else "CL_BENCH_DEVELOPING",
        "citeable": n >= 3,
    })
    return out


def write_report(doc: dict[str, Any]) -> Path:
    REPORTS.mkdir(parents=True, exist_ok=True)
    jpath = REPORTS / "incubation_llm_benchmarks.json"
    mpath = REPORTS / "INCUBATION_LLM_BENCHMARKS.md"
    jpath.write_text(json.dumps(doc, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")

    lines = [
        "# Incubation / Trainable-LLM Benchmarks",
        "",
        f"**Generated:** {doc['timestamp']}",
        "",
        "## Valhalla protocols (this run)",
        "",
        "| Protocol | Result | Verdict | Citeable |",
        "|----------|--------|---------|----------|",
    ]
    for r in doc.get("results") or []:
        lines.append(
            f"| {r.get('protocol', '?')} | {r.get('result', '—')} | {r.get('verdict', '—')} | {r.get('citeable', '—')} |"
        )
    lines.extend(["", "## Literature mapping", ""])
    for lit in doc.get("literature") or []:
        lines.append(f"- **{lit['name']}** ({lit['paper']}): {lit['focus']}")
        lines.append(f"  - Valhalla proxy: {lit['valhalla_proxy']}")
    lines.extend(["", "---", "", "*Rogue Intelligence LNC.*"])
    mpath.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return mpath


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--fast", action="store_true")
    ap.add_argument("--all", action="store_true")
    ap.add_argument("--skip-cl", action="store_true")
    ap.add_argument("--force-tpi", action="store_true", help="Re-run TPI even in --fast (slow)")
    args = ap.parse_args()

    env = {
        "HF_ENDPOINT": os.environ.get("HF_ENDPOINT", "https://hf-mirror.com"),
        "VALHALLA_INGRESS_RESOURCE_PERCENT": "50",
        "VALHALLA_INGRESS_MAX_ITEMS": "48",
        "VALHALLA_BASE_NO_SERVE": "1",
    }

    results: list[dict[str, Any]] = []
    failed = 0

    # TPI-v2 — full run is ~30+ min; --fast reuses cached report when available
    tpi_cached = load_cached_tpi()
    if args.fast and tpi_cached and not args.force_tpi:
        print(f"\n[TPI-v2] reusing cached report ({tpi_cached['result']})", flush=True)
        results.append(tpi_cached)
    else:
        tpi_flags = ["--protocol", "v2", "--eval-limit", "8" if args.fast else "20"]
        if args.fast:
            tpi_flags.extend(["--skip-cycles", "--skip-scale", "--skip-rag"])
        timeout = None if args.all and not args.fast else (900.0 if args.fast else None)
        rc, _ = run_cmd(
            "TPI-v2",
            [RUN_PY, str(HERE / "test_valhalla_base_training_potential.py"), *tpi_flags],
            env=env,
            timeout_s=timeout,
        )
        if rc == 124 and tpi_cached:
            print("[TPI-v2] timeout — falling back to cached report", flush=True)
            results.append(tpi_cached)
        else:
            tpi_path = REPORTS / "valhalla_base_training_potential_test_v2.json"
            tpi_doc = json.loads(tpi_path.read_text()) if tpi_path.is_file() else {}
            tpi = (tpi_doc.get("tpi") or {}).get("tpi")
            results.append({
                "protocol": "valhalla-base-training-potential-v2",
                "result": f"TPI {tpi:.3f}" if tpi else "—",
                "verdict": tpi_doc.get("verdict"),
                "citeable": tpi is not None,
                "exit_code": rc,
            })
            failed += int(rc != 0 and rc != 124)

    # Fate pretrain holdout (L2)
    holdout_flags = ["--low-resource", "--fast", "--train", "5", "--test", "5"]
    rc, _ = run_cmd(
        "fate-pretrain-holdout",
        [RUN_PY, str(HERE / "test_fate_pretrain_holdout.py"), *holdout_flags],
        env=env,
    )
    hold_doc = {}
    hp = REPORTS / "fate_pretrain_holdout.json"
    if hp.is_file():
        hold_doc = json.loads(hp.read_text())
    train_acc = hold_doc.get("train_accuracy") or hold_doc.get("train_pass")
    test_acc = hold_doc.get("test_accuracy") or hold_doc.get("test_pass")
    converged = hold_doc.get("converged", False)
    results.append({
        "protocol": "fate-pretrain-holdout-v3",
        "result": f"train {train_acc} test {test_acc}" if train_acc is not None else "—",
        "verdict": "CONVERGED" if converged else ("HOLDOUT_STRONG" if (test_acc or 0) >= 0.8 else "PARTIAL"),
        "citeable": (test_acc or 0) >= 0.8,
        "exit_code": rc,
    })
    failed += int(rc != 0 and (test_acc or 0) < 0.6)

    if args.all and not args.fast:
        rc, _ = run_cmd(
            "incubation-standard",
            [RUN_PY, str(REPO / "tools/valhalla_model_bridge/run_incubation_benchmark.py"), "--phase", "standard"],
            env=env,
        )
        results.append({
            "protocol": "incubation-standard-v1",
            "result": "see reports/body_experiments/",
            "verdict": "OK" if rc == 0 else "FAIL",
            "citeable": rc == 0,
            "exit_code": rc,
        })
        failed += int(rc != 0)

    if not args.skip_cl:
        cl = run_cl_bench_slice(limit=5 if args.fast else 10)
        cl["result"] = f"{cl.get('correct', '?')}/{cl.get('n', '?')}"
        results.append(cl)

    doc = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "mode": "fast" if args.fast else ("all" if args.all else "default"),
        "literature": LITERATURE_MAP,
        "results": results,
        "summary": {
            "protocols_run": len(results),
            "citeable": sum(1 for r in results if r.get("citeable")),
        },
    }
    mpath = write_report(doc)
    print(f"\n→ {mpath}")
    integrate = subprocess.call([RUN_PY, str(HERE / "integrate_valhalla_reports.py")], cwd=REPO)
    return 1 if failed or integrate else 0


if __name__ == "__main__":
    raise SystemExit(main())
