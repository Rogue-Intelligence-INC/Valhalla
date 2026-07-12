#!/usr/bin/env python3
"""Run benchmark suites sequentially; write per-suite summary markdown.

Usage:
  python3 tools/valhalla_inference/run_suite_sequential.py --suite fate-core
  python3 tools/valhalla_inference/run_suite_sequential.py --all --fast
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
REPORTS = REPO / "reports" / "valhalla_inference"
SUMMARIES = REPORTS / "suite_summaries"
PY = sys.executable
RUSTFLAGS = "-L /opt/cuda/lib64"

# (suite_id, label, command list, report_json optional)
SUITES: dict[str, list[tuple[str, list[str], str | None]]] = {
    "fate-core": [
        (
            "fate_parallel_longchain",
            [PY, str(HERE / "test_fate_parallel_longchain.py"), "--verbose"],
            "fate_parallel_longchain_test.json",
        ),
        (
            "fate_pretrain_holdout",
            [
                PY,
                str(HERE / "test_fate_pretrain_holdout.py"),
                "--train",
                "5",
                "--test",
                "5",
                "--max-epochs",
                "6",
                "--low-resource",
                "--fast",
            ],
            "fate_pretrain_holdout.json",
        ),
        (
            "fate_routing_ab",
            [
                PY,
                str(HERE / "test_fate_routing_ab.py"),
                "--limit",
                "10",
                "--low-resource",
                "--fast",
            ],
            "fate_routing_ab_locomo.json",
        ),
        (
            "fate_longchain_unified",
            [PY, str(HERE / "test_fate_longchain_unified.py"), "--production"],
            "fate_longchain_unified.json",
        ),
    ],
    "rust-unit": [
        (
            "cargo_test_fate_holdout_unified",
            [
                "cargo",
                "test",
                "-p",
                "hub-f64",
                "--lib",
                "--features",
                "patch-lm",
                "--",
                "fate_pretrain_holdout",
                "fate_longchain_unified",
            ],
            None,
        ),
    ],
    "standard-fast": [
        (
            "needle",
            [PY, str(HERE / "test_needle_session_v1.py"), "--fast"],
            "needle_session_v1_test.json",
        ),
        (
            "locomo",
            [PY, str(HERE / "test_locomo_v1.py"), "--rust", "--fast", "--limit", "3"],
            "locomo_v1_rust_test.json",
        ),
        (
            "frames",
            [PY, str(HERE / "test_frames_v1.py"), "--rust", "--fast", "--limit", "3"],
            "frames_v1_rust_test.json",
        ),
        (
            "multihop",
            [PY, str(HERE / "test_multihop_rag_v1.py"), "--hybrid", "--limit", "5", "--fast"],
            "multihop_rag_hybrid_test.json",
        ),
        (
            "ragtruth",
            [PY, str(HERE / "test_ragtruth_v1.py"), "--fast", "--limit", "5"],
            "ragtruth_v1_test.json",
        ),
        (
            "longbench",
            [PY, str(HERE / "test_longbench_v2_v1.py"), "--fast", "--limit", "5"],
            "longbench_v2_v1_test.json",
        ),
        (
            "ruler",
            [PY, str(HERE / "test_ruler_v1.py"), "--fast", "--limit", "5"],
            "ruler_v1_test.json",
        ),
        (
            "lara",
            [PY, str(HERE / "test_lara_v1.py"), "--fast", "--limit", "5"],
            "lara_v1_test.json",
        ),
    ],
    "context-drift": [
        (
            "context_topic_drift",
            [PY, str(HERE / "test_context_topic_drift.py"), "--skip-trad"],
            "context_topic_drift_test.json",
        ),
    ],
}

SUITE_ORDER = ["fate-core", "rust-unit", "standard-fast", "context-drift"]


def ts() -> str:
    return datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")


def extract_metrics(report_path: Path) -> dict:
    if not report_path.is_file():
        return {}
    try:
        return json.loads(report_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return {}


def summarize_benchmark(label: str, doc: dict, rc: int, elapsed: float) -> str:
    lines = [
        f"# Suite Item Summary: {label}",
        "",
        f"- **Time (UTC)**: {datetime.now(timezone.utc).isoformat()}",
        f"- **Exit code**: {rc}",
        f"- **Elapsed**: {elapsed:.1f}s",
        f"- **Status**: {'PASS' if rc == 0 else 'FAIL'}",
        "",
        "## Key metrics",
        "",
    ]
    if not doc:
        lines.append("_No JSON report or parse failed._")
        return "\n".join(lines) + "\n"

    protocol = doc.get("protocol", doc.get("name", "—"))
    lines.append(f"- **Protocol**: `{protocol}`")

    for key in (
        "baseline_pass",
        "parallel_pass",
        "targeted_pass",
        "train_pass",
        "test_pass",
        "converged",
        "theme_drift_rate",
        "anchor_retention_rate",
        "holdout_pass",
        "composite_score",
        "accuracy",
        "correct",
        "n",
    ):
        if key in doc:
            val = doc[key]
            if isinstance(val, float):
                lines.append(f"- **{key}**: {val:.4f}")
            else:
                lines.append(f"- **{key}**: {val}")

    rows = doc.get("rows") or doc.get("test_rows") or doc.get("anchor_drift_rows")
    if rows and isinstance(rows, list):
        ok = sum(1 for r in rows if r.get("correct") or r.get("baseline_correct"))
        lines.append(f"- **Row sample**: {len(rows)} items")
        if any("correct" in (r or {}) for r in rows):
            lines.append(f"- **Rows correct (where applicable)**: {ok}/{len(rows)}")

    lines.extend(["", "## Verdict", ""])
    if rc != 0:
        lines.append("**FAIL** — non-zero exit; inspect stderr in terminal log.")
    elif label == "fate_longchain_unified":
        drift = float(doc.get("theme_drift_rate") or 1.0)
        composite = float(doc.get("composite_score") or 0.0)
        if drift <= 0.01 and composite >= 0.85:
            lines.append("**PASS** — no theme drift after long-chain interleave.")
        elif drift <= 0.01:
            lines.append(f"**PASS** — no drift; composite {composite:.3f} (check holdout tier).")
        else:
            lines.append(
                f"**WARN** — theme_drift {drift:.1%}, composite {composite:.3f}; inspect anchor_drift_rows."
            )
    elif doc.get("converged") is True:
        lines.append("**PASS** — training converged.")
    elif doc.get("accuracy") is not None:
        lines.append(f"**DONE** — accuracy {doc['accuracy']:.1%}.")
    else:
        lines.append("**PASS** — completed without error.")

    return "\n".join(lines) + "\n"


def summarize_suite(suite_id: str, results: list[dict]) -> str:
    passed = sum(1 for r in results if r["rc"] == 0)
    total = len(results)
    lines = [
        f"# Suite Summary: `{suite_id}`",
        "",
        f"- **Completed (UTC)**: {datetime.now(timezone.utc).isoformat()}",
        f"- **Items**: {passed}/{total} passed",
        "",
        "## Items",
        "",
        "| Benchmark | Status | Elapsed | Notes |",
        "|-----------|--------|---------|-------|",
    ]
    for r in results:
        status = "PASS" if r["rc"] == 0 else "FAIL"
        note = r.get("note", "")
        lines.append(
            f"| {r['label']} | {status} | {r['elapsed']:.1f}s | {note} |"
        )
    lines.extend(["", "## Per-item reports", ""])
    for r in results:
        if r.get("summary_path"):
            lines.append(f"- [{r['label']}]({r['summary_path'].name})")
    return "\n".join(lines) + "\n"


def run_item(
    label: str,
    cmd: list[str],
    report_file: str | None,
    env: dict[str, str],
) -> dict:
    print(f"\n{'=' * 60}\n[{label}] $ {' '.join(cmd)}\n", flush=True)
    t0 = time.monotonic()
    proc = subprocess.run(cmd, cwd=REPO, env=env, capture_output=True, text=True)
    elapsed = time.monotonic() - t0
    if proc.stdout:
        print(proc.stdout[-4000:] if len(proc.stdout) > 4000 else proc.stdout)
    if proc.stderr:
        print(proc.stderr[-2000:], file=sys.stderr)

    doc = {}
    report_path = REPORTS / report_file if report_file else None
    if report_path and report_path.is_file():
        doc = extract_metrics(report_path)

    note = ""
    if doc.get("protocol"):
        note = str(doc["protocol"])
    elif proc.returncode == 0:
        note = "ok"

    item_summary = summarize_benchmark(label, doc, proc.returncode, elapsed)
    item_path = SUMMARIES / f"{label}_{ts()}.md"
    item_path.write_text(item_summary, encoding="utf-8")
    print(f"→ summary {item_path}", flush=True)

    return {
        "label": label,
        "rc": proc.returncode,
        "elapsed": elapsed,
        "note": note,
        "summary_path": item_path,
        "report_file": report_file,
    }


def run_suite(suite_id: str, env: dict[str, str]) -> int:
    items = SUITES.get(suite_id)
    if not items:
        print(f"Unknown suite: {suite_id}", file=sys.stderr)
        return 1

    SUMMARIES.mkdir(parents=True, exist_ok=True)
    results: list[dict] = []
    suite_t0 = time.monotonic()

    for label, cmd, report_file in items:
        if cmd[0] == "cargo":
            full_cmd = cmd
        else:
            full_cmd = cmd
        results.append(run_item(label, full_cmd, report_file, env))

    suite_elapsed = time.monotonic() - suite_t0
    suite_body = summarize_suite(suite_id, results)
    suite_body += f"\n- **Total elapsed**: {suite_elapsed:.1f}s\n"
    suite_path = SUMMARIES / f"SUITE_{suite_id}_{ts()}.md"
    suite_path.write_text(suite_body, encoding="utf-8")
    print(f"\n=== Suite `{suite_id}` → {suite_path}", flush=True)

    failed = sum(1 for r in results if r["rc"] != 0)
    return 1 if failed else 0


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--suite", choices=list(SUITES.keys()), action="append")
    ap.add_argument("--all", action="store_true")
    ap.add_argument("--list", action="store_true")
    args = ap.parse_args()

    if args.list:
        for sid in SUITE_ORDER:
            print(f"{sid}: {[x[0] for x in SUITES[sid]]}")
        return 0

    suites = SUITE_ORDER if args.all else (args.suite or ["fate-core"])
    env = {**os.environ, "RUSTFLAGS": RUSTFLAGS}
    env.setdefault("VALHALLA_INGRESS_RESOURCE_PERCENT", "70")
    env.setdefault("VALHALLA_INGRESS_MAX_ITEMS", "128")
    env.setdefault("VALHALLA_FATE_PARALLEL_LCHAIN", "1")
    SUMMARIES.mkdir(parents=True, exist_ok=True)

    # Ensure valhalla_base built once
    vb = REPO / "target" / "release" / "valhalla_base"
    if not vb.is_file():
        subprocess.check_call(
            [
                "cargo",
                "build",
                "-p",
                "hub-f64",
                "--release",
                "--bin",
                "valhalla_base",
                "--features",
                "patch-lm",
            ],
            cwd=REPO,
            env=env,
        )

    rc_total = 0
    for sid in suites:
        rc = run_suite(sid, env)
        if rc:
            rc_total = rc

    # Master index
    index_path = SUMMARIES / f"INDEX_{ts()}.md"
    index_lines = [
        "# Benchmark Suite Run Index",
        "",
        f"- **UTC**: {datetime.now(timezone.utc).isoformat()}",
        f"- **Suites run**: {', '.join(suites)}",
        "",
        "See `suite_summaries/SUITE_*.md` for per-suite rollups.",
        "",
    ]
    for p in sorted(SUMMARIES.glob("SUITE_*.md"), key=lambda x: x.stat().st_mtime, reverse=True)[:10]:
        index_lines.append(f"- [{p.name}]({p.name})")
    index_path.write_text("\n".join(index_lines) + "\n", encoding="utf-8")
    print(f"→ index {index_path}", flush=True)
    return rc_total


if __name__ == "__main__":
    raise SystemExit(main())
