#!/usr/bin/env python3
"""Fundraise ablation matrix — paired arms × industry benchmarks.

Protocol: fundraise-ablation-matrix-v1

CLI:
  --moderate / --fast / full
  --verbose (default on) · --pause-file · --resume · --checkpoint-dir
  --stop-on-error (default) · --continue-on-error
  --with-paradigm

Status file: reports/valhalla_inference/fundraise_ablation.status
Pause:       touch $VALHALLA_PAUSE_FILE  (or --pause-file PATH)
Resume:      --resume --checkpoint-dir DIR
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import traceback
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Callable

REPO = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
REPORTS = REPO / "reports/valhalla_inference"
DOCS = REPO / "docs/valhalla"
STATUS_FILE = REPORTS / "fundraise_ablation.status"
CHECKPOINT_PROTOCOL = "fundraise-ablation-checkpoint-v1"

sys.path.insert(0, str(HERE))
sys.path.insert(0, str(REPO / "tools/standard_benchmarks"))
sys.path.insert(0, str(REPO / "tools/fair_benchmark"))

from ablation_arms import (  # noqa: E402
    ARM_ORDER,
    ARM_SPECS,
    ArmError,
    ArmPaused,
    run_arm,
)
from benchmark_cli import ProgressBar, pause_requested  # noqa: E402
from benchmark_env import (  # noqa: E402
    apply_gpu_env,
    apply_resource_budget,
    benchmark_calibration_lm,
    release_resources,
)
from common import (  # noqa: E402
    format_mcq_prompt,
    load_bundled,
    score_ragtruth_item,
    score_short_answer,
    split_context_to_corpus,
)
from metrics import score_answer  # noqa: E402
from pareto_backends import RunMetrics  # noqa: E402

PROTOCOL = "fundraise-ablation-matrix-v1"
LOCOMO_BUNDLE = "locomo_qa_slice30.json"
MULTIHOP_BUNDLE = "multihop_rag_slice25.json"
RAGTRUTH_BUNDLE = "ragtruth_qa_test40.json"
LONGBENCH_BUNDLE = "longbench_v2_mcq_short20.json"

MODERATE_DEFAULTS = {"locomo": 15, "multihop": 15, "ragtruth": 40, "longbench": 10}


def write_status(**fields: Any) -> None:
    REPORTS.mkdir(parents=True, exist_ok=True)
    lines = [f"{k}={v}" for k, v in fields.items()]
    STATUS_FILE.write_text("\n".join(lines) + "\n", encoding="utf-8")


def locomo_items(limit: int) -> list[dict[str, Any]]:
    doc = load_bundled(LOCOMO_BUNDLE)
    out: list[dict] = []
    for it in doc.get("items") or []:
        lines = [ln.strip() for ln in (it.get("session_lines") or []) if ln.strip()]
        out.append(
            {
                "id": it.get("id"),
                "question": it.get("question"),
                "reference_answer": it.get("answer"),
                "gold": it.get("answer"),
                "session_lines": lines,
                "corpus": lines,
                "score_type": "open",
            }
        )
        if limit > 0 and len(out) >= limit:
            break
    return out


def multihop_items(limit: int) -> list[dict[str, Any]]:
    doc = load_bundled(MULTIHOP_BUNDLE)
    out: list[dict] = []
    for it in doc.get("items") or []:
        if not it.get("corpus"):
            continue
        out.append(
            {
                "id": it.get("id"),
                "question": it.get("question"),
                "reference_answer": it.get("answer"),
                "corpus": list(it.get("corpus") or []),
                "score_type": "open",
            }
        )
        if limit > 0 and len(out) >= limit:
            break
    return out


def ragtruth_items(limit: int) -> list[dict[str, Any]]:
    doc = load_bundled(RAGTRUTH_BUNDLE)
    out: list[dict] = []
    for it in doc.get("items") or []:
        out.append(dict(it))
        if limit > 0 and len(out) >= limit:
            break
    return out


def longbench_items(limit: int) -> list[dict[str, Any]]:
    doc = load_bundled(LONGBENCH_BUNDLE)
    out: list[dict] = []
    for it in doc.get("items") or []:
        corpus = split_context_to_corpus(it.get("context") or "", max_lines=0)
        prompt = format_mcq_prompt(it)
        out.append(
            {
                "id": it.get("_id"),
                "question": prompt,
                "prompt": prompt,
                "reference_answer": it.get("answer"),
                "answer": it.get("answer"),
                "corpus": corpus,
                "score_type": "mcq",
                "choice_A": it.get("choice_A"),
            }
        )
        if limit > 0 and len(out) >= limit:
            break
    return out


def score_locomo(answer: str, item: dict) -> bool:
    return score_short_answer(answer, str(item.get("reference_answer") or item.get("gold") or ""))


def score_multihop(answer: str, item: dict) -> bool:
    return score_short_answer(answer, str(item.get("reference_answer") or item.get("answer") or ""))


def score_ragtruth(answer: str, item: dict) -> bool:
    return bool(score_ragtruth_item(answer, item).get("faithful"))


def score_longbench(answer: str, item: dict) -> bool:
    sc = score_answer(answer, {"reference_answer": item["reference_answer"], "score_type": "mcq"})
    return bool(sc.get("correct"))


def cell_key(bench: str, arm: str) -> str:
    return f"{bench}::{arm}"


def load_checkpoint(path: Path) -> dict[str, Any]:
    if not path.is_file():
        return {}
    try:
        doc = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return {}
    if doc.get("protocol") != CHECKPOINT_PROTOCOL:
        return {}
    return doc


def save_checkpoint(path: Path, doc: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    doc["protocol"] = CHECKPOINT_PROTOCOL
    doc["last_updated"] = datetime.now(timezone.utc).isoformat()
    path.write_text(json.dumps(doc, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def build_matrix_table(doc: dict[str, Any]) -> list[dict[str, Any]]:
    rows: list[dict] = []
    benches = doc.get("benchmarks") or {}
    for arm in doc.get("arms_run") or ARM_ORDER:
        row: dict[str, Any] = {
            "arm": arm,
            "label": ARM_SPECS.get(arm, ARM_SPECS["hybrid_full"]).label,
        }
        for bkey, bname in (
            ("locomo", "LoCoMo"),
            ("multihop", "MultiHop"),
            ("ragtruth", "RAGTruth"),
            ("longbench", "LongBench"),
        ):
            arm_data = (benches.get(bkey) or {}).get("arms", {}).get(arm) or {}
            if arm_data.get("skipped"):
                row[bname] = "SKIP"
                row[f"{bname}_acc"] = None
            elif arm_data.get("n"):
                row[bname] = f"{arm_data['correct']}/{arm_data['n']}"
                row[f"{bname}_acc"] = arm_data.get("accuracy")
            else:
                row[bname] = "—"
                row[f"{bname}_acc"] = None
        rows.append(row)
    return rows


def component_deltas(matrix: list[dict]) -> list[str]:
    notes: list[str] = []
    full = next((r for r in matrix if r["arm"] == "hybrid_full"), {})
    no_fate = next((r for r in matrix if r["arm"] == "hybrid_no_fate"), {})
    tile = next((r for r in matrix if r["arm"] == "hybrid_tile"), {})
    vanilla = next((r for r in matrix if r["arm"] == "vanilla_rag"), {})
    for label, key in (("LoCoMo", "LoCoMo_acc"), ("MultiHop", "MultiHop_acc")):
        f_acc, nf_acc, t_acc, v_acc = full.get(key), no_fate.get(key), tile.get(key), vanilla.get(key)
        if f_acc is not None and nf_acc is not None:
            notes.append(f"Fate lift on {label}: {(f_acc - nf_acc) * 100:+.1f}pp (full vs −Fate)")
        if f_acc is not None and t_acc is not None:
            notes.append(f"Triad lift on {label}: {(f_acc - t_acc) * 100:+.1f}pp (triad vs tile)")
        if f_acc is not None and v_acc is not None:
            notes.append(
                f"Structure premium on {label}: {(f_acc - v_acc) * 100:+.1f}pp (hybrid vs vanilla RAG)"
            )
    return notes


def write_reports(doc: dict[str, Any]) -> tuple[Path, Path]:
    REPORTS.mkdir(parents=True, exist_ok=True)
    DOCS.mkdir(parents=True, exist_ok=True)
    jpath = REPORTS / "fundraise_ablation_matrix.json"
    mpath = REPORTS / "FUNDRAISE_ABLATION_MATRIX.md"
    dpath = DOCS / "FUNDRAISE_ABLATION_MATRIX.md"
    if doc.get("benchmarks"):
        doc["matrix"] = build_matrix_table(doc)
        doc["component_notes"] = component_deltas(doc["matrix"])
    jpath.write_text(json.dumps(doc, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")

    c = doc["constraints"]
    lines = [
        "# Fundraise Ablation Matrix",
        "",
        f"**Protocol:** {PROTOCOL}  ",
        f"**Generated:** {doc['timestamp']}",
        "",
        "## Constraints",
        "",
        "| Field | Value |",
        "|-------|-------|",
        f"| Backbone | `{c['backbone']}` |",
        f"| LoCoMo n | {c['locomo_n']} |",
        f"| MultiHop n | {c['multihop_n']} |",
        f"| RAGTruth n | {c['ragtruth_n']} |",
        f"| LongBench n | {c['longbench_n']} |",
        f"| Scale | {c.get('scale', 'custom')} |",
        f"| Stop on error | {c.get('stop_on_error', True)} |",
        "",
        "## Matrix (correct/n)",
        "",
        "| Arm | LoCoMo | MultiHop | RAGTruth | LongBench |",
        "|-----|--------|----------|----------|-----------|",
    ]
    for row in doc.get("matrix") or []:
        lines.append(
            f"| {row['label']} | {row.get('LoCoMo', '—')} | {row.get('MultiHop', '—')} | "
            f"{row.get('RAGTruth', '—')} | {row.get('LongBench', '—')} |"
        )
    lines.extend(["", "## Component notes", ""])
    for note in doc.get("component_notes") or []:
        lines.append(f"- {note}")
    if doc.get("paradigm"):
        lines.extend(["", "## Paradigm batteries", ""])
        for k, v in (doc.get("paradigm") or {}).items():
            lines.append(f"- **{k}**: {v}")
    if doc.get("control"):
        lines.extend(["", "## Control", ""])
        ctrl = doc["control"]
        lines.append(f"- State: **{ctrl.get('state', '—')}**")
        if ctrl.get("failed_cell"):
            lines.append(f"- Failed cell: `{ctrl['failed_cell']}`")
        if ctrl.get("pause_file"):
            lines.append(f"- Pause file: `{ctrl['pause_file']}`")
    lines.extend(["", "---", ""])
    text = "\n".join(lines) + "\n"
    mpath.write_text(text, encoding="utf-8")
    dpath.write_text(text, encoding="utf-8")
    return jpath, mpath


def run_benchmark(
    bench_key: str,
    items: list[dict],
    *,
    arms: list[str],
    backbone: str,
    score_fn: Callable[[str, dict], bool],
    checkpoint: dict[str, Any],
    ckpt_path: Path,
    doc: dict[str, Any],
    verbose: bool,
    pause_file: str | None,
    stop_on_error: bool,
) -> dict[str, Any]:
    results: dict[str, Any] = {"n": len(items), "arms": {}}
    # Restore completed arms from checkpoint
    prior = (checkpoint.get("cells") or {}).get(bench_key) or {}
    for arm, summary in (prior.get("arms") or {}).items():
        if summary.get("n") and not summary.get("partial"):
            results["arms"][arm] = summary

    suite_bar = ProgressBar(f"suite:{bench_key}", len(arms), enabled=verbose)
    suite_bar.set(len(results["arms"]))

    for arm in arms:
        key = cell_key(bench_key, arm)
        if arm in results["arms"]:
            print(f"  [{bench_key}] {arm} RESUME-SKIP (done)", flush=True)
            suite_bar.inc()
            continue
        if pause_requested(pause_file):
            write_status(state="paused", cell=key, updated=datetime.now(timezone.utc).isoformat())
            doc["control"] = {"state": "paused", "pause_file": pause_file, "failed_cell": None}
            write_reports(doc)
            save_checkpoint(ckpt_path, checkpoint)
            raise ArmPaused(RunMetrics(), 0)

        spec = ARM_SPECS.get(arm, ARM_SPECS["hybrid_full"])
        print(f"  [{bench_key}] {arm} ({spec.label}) n={len(items)} …", flush=True)
        write_status(
            state="running",
            cell=key,
            bench=bench_key,
            arm=arm,
            updated=datetime.now(timezone.utc).isoformat(),
            pid=os.getpid(),
        )
        release_resources()
        try:
            metrics = run_arm(
                arm,
                items,
                backbone=backbone,
                score_fn=score_fn,
                label=f"{bench_key}/{arm}",
                verbose=verbose,
                pause_file=pause_file,
                stop_on_error=stop_on_error,
            )
        except ArmPaused as exc:
            summary = exc.metrics.to_summary()
            summary["partial"] = True
            results["arms"][arm] = summary
            checkpoint.setdefault("cells", {})[bench_key] = results
            save_checkpoint(ckpt_path, checkpoint)
            doc.setdefault("benchmarks", {})[bench_key] = results
            doc["control"] = {"state": "paused", "pause_file": pause_file, "failed_cell": key}
            write_reports(doc)
            write_status(state="paused", cell=key, updated=datetime.now(timezone.utc).isoformat())
            raise
        except ArmError as exc:
            summary = (exc.metrics or RunMetrics()).to_summary()
            summary["error"] = str(exc)
            results["arms"][arm] = summary
            checkpoint.setdefault("cells", {})[bench_key] = results
            checkpoint.setdefault("failed", []).append({"cell": key, "error": str(exc)})
            save_checkpoint(ckpt_path, checkpoint)
            doc.setdefault("benchmarks", {})[bench_key] = results
            doc["control"] = {"state": "error", "failed_cell": key, "error": str(exc)}
            write_reports(doc)
            write_status(
                state="error",
                cell=key,
                error=str(exc)[:200],
                updated=datetime.now(timezone.utc).isoformat(),
            )
            if stop_on_error:
                raise
            print(f"    ERROR (continue): {exc}", flush=True)
            suite_bar.inc()
            continue

        summary = metrics.to_summary()
        results["arms"][arm] = summary
        checkpoint.setdefault("cells", {})[bench_key] = results
        completed = set(checkpoint.get("completed_cells") or [])
        completed.add(key)
        checkpoint["completed_cells"] = sorted(completed)
        save_checkpoint(ckpt_path, checkpoint)
        doc.setdefault("benchmarks", {})[bench_key] = results
        write_reports(doc)

        if not metrics.skipped:
            print(
                f"    {summary['correct']}/{summary['n']} ({summary['accuracy']:.1%}) "
                f"tok/q={summary['avg_tokens_per_query']:.0f} lat={summary['avg_latency_ms']:.0f}ms",
                flush=True,
            )
        else:
            print(f"    SKIPPED: {metrics.skip_reason}", flush=True)
            if stop_on_error and metrics.skip_reason:
                raise ArmError(f"skipped {key}: {metrics.skip_reason}", metrics)
        suite_bar.inc()
    suite_bar.finish(f"{bench_key} done")
    return results


def run_paradigm_batteries(
    *,
    moderate: bool,
    stop_on_error: bool,
    pause_file: str | None,
) -> dict[str, str]:
    py = sys.executable
    out: dict[str, str] = {}
    eval_limit = "15" if moderate else "0"
    steps = [
        (
            "TPI-v2",
            [
                py,
                str(HERE / "test_valhalla_base_training_potential.py"),
                "--protocol",
                "v2",
                "--skip-cycles",
            ]
            + (["--eval-limit", eval_limit] if moderate else []),
        ),
        ("NPPI", [py, str(HERE / "test_new_paradigm_potential.py")]),
        (
            "token_efficiency",
            [py, str(HERE / "test_token_efficiency.py")]
            + (["--run-valhalla"] if moderate else []),
        ),
        (
            "external_pareto",
            [
                py,
                str(HERE / "run_external_pareto_matrix.py"),
                "--locomo-limit",
                eval_limit if moderate else "10",
                "--longbench-limit",
                "10",
            ],
        ),
    ]
    for name, cmd in steps:
        if pause_requested(pause_file):
            out[name] = "paused"
            write_status(state="paused", cell=f"paradigm::{name}")
            break
        print(f"\n=== {name} (paradigm) ===", flush=True)
        write_status(state="running", cell=f"paradigm::{name}", pid=os.getpid())
        rc = subprocess.call(cmd, cwd=REPO)
        out[name] = "ok" if rc == 0 else f"exit {rc}"
        if rc != 0 and stop_on_error:
            write_status(state="error", cell=f"paradigm::{name}", error=f"exit {rc}")
            raise SystemExit(rc)
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description="Fundraise ablation matrix with monitor/pause/error-stop")
    ap.add_argument("--moderate", action="store_true")
    ap.add_argument("--fast", action="store_true")
    ap.add_argument("--locomo-limit", type=int, default=0)
    ap.add_argument("--multihop-limit", type=int, default=0)
    ap.add_argument("--ragtruth-limit", type=int, default=0)
    ap.add_argument("--longbench-limit", type=int, default=0)
    ap.add_argument("--backbone", default="")
    ap.add_argument("--arms", default=",".join(ARM_ORDER))
    ap.add_argument("--with-paradigm", action="store_true")
    ap.add_argument("--skip-industry", action="store_true")
    ap.add_argument("--verbose", "-v", action="store_true", default=True)
    ap.add_argument("--quiet", action="store_true", help="disable progress bars")
    ap.add_argument("--pause-file", type=Path, default=None)
    ap.add_argument("--checkpoint-dir", type=Path, default=REPORTS / "ablation_ckpt")
    ap.add_argument("--resume", action="store_true", help="resume from checkpoint-dir")
    ap.add_argument("--stop-on-error", action="store_true", default=True)
    ap.add_argument("--continue-on-error", action="store_true")
    args = ap.parse_args()

    verbose = not args.quiet
    stop_on_error = not args.continue_on_error
    pause_file = str(args.pause_file) if args.pause_file else os.environ.get("VALHALLA_PAUSE_FILE", "").strip() or None
    if pause_file:
        os.environ["VALHALLA_PAUSE_FILE"] = pause_file

    apply_resource_budget()
    apply_gpu_env()
    os.environ.setdefault("VALHALLA_BASE_NO_SERVE", "1")
    os.environ.setdefault("VALHALLA_CUDA_DEVICE", "0")
    backbone = args.backbone or benchmark_calibration_lm()
    os.environ["VALHALLA_LM_MODEL"] = backbone

    if args.fast:
        loc_n = mh_n = lb_n = 5
        rt_n = 10
        scale = "fast"
    elif args.moderate:
        loc_n = args.locomo_limit or MODERATE_DEFAULTS["locomo"]
        mh_n = args.multihop_limit or MODERATE_DEFAULTS["multihop"]
        rt_n = args.ragtruth_limit or MODERATE_DEFAULTS["ragtruth"]
        lb_n = args.longbench_limit or MODERATE_DEFAULTS["longbench"]
        scale = "moderate"
    else:
        loc_n = args.locomo_limit or 30
        mh_n = args.multihop_limit or 25
        rt_n = args.ragtruth_limit or 40
        lb_n = args.longbench_limit or 20
        scale = "full"

    arms = [a.strip() for a in args.arms.split(",") if a.strip()]
    ckpt_dir = args.checkpoint_dir
    ckpt_path = ckpt_dir / "fundraise_ablation_checkpoint.json"
    checkpoint: dict[str, Any] = load_checkpoint(ckpt_path) if args.resume else {
        "protocol": CHECKPOINT_PROTOCOL,
        "started_at": datetime.now(timezone.utc).isoformat(),
        "cells": {},
        "completed_cells": [],
        "failed": [],
    }
    if args.resume and checkpoint:
        print(f"RESUME from {ckpt_path} ({len(checkpoint.get('completed_cells') or [])} cells)", flush=True)

    doc: dict[str, Any] = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "protocol": PROTOCOL,
        "constraints": {
            "backbone": backbone,
            "locomo_n": loc_n,
            "multihop_n": mh_n,
            "ragtruth_n": rt_n,
            "longbench_n": lb_n,
            "arms": arms,
            "scale": scale,
            "stop_on_error": stop_on_error,
            "cuda_device": os.environ.get("VALHALLA_CUDA_DEVICE", "0"),
        },
        "arms_run": arms,
        "benchmarks": {},
        "control": {
            "state": "running",
            "pause_file": pause_file,
            "checkpoint": str(ckpt_path),
        },
    }
    # Seed prior benchmark results into doc for matrix continuity
    for bkey, cell in (checkpoint.get("cells") or {}).items():
        doc["benchmarks"][bkey] = cell

    write_status(state="running", scale=scale, backbone=backbone, pid=os.getpid())
    print(
        f"=== {PROTOCOL} scale={scale} backbone={backbone} "
        f"stop_on_error={stop_on_error} pause={pause_file or '—'} ===\n",
        flush=True,
    )

    try:
        if not args.skip_industry:
            benches: list[tuple[str, list[dict], Callable]] = [
                ("locomo", locomo_items(loc_n), score_locomo),
                ("multihop", multihop_items(mh_n), score_multihop),
                ("ragtruth", ragtruth_items(rt_n), score_ragtruth),
                ("longbench", longbench_items(lb_n), score_longbench),
            ]
            for bkey, items, score_fn in benches:
                print(f"[{bkey}] {len(items)} items", flush=True)
                doc["benchmarks"][bkey] = run_benchmark(
                    bkey,
                    items,
                    arms=arms,
                    backbone=backbone,
                    score_fn=score_fn,
                    checkpoint=checkpoint,
                    ckpt_path=ckpt_path,
                    doc=doc,
                    verbose=verbose,
                    pause_file=pause_file,
                    stop_on_error=stop_on_error,
                )
                write_reports(doc)

        if args.with_paradigm:
            doc["paradigm"] = run_paradigm_batteries(
                moderate=args.moderate or args.fast,
                stop_on_error=stop_on_error,
                pause_file=pause_file,
            )

        doc["control"]["state"] = "ok"
        write_status(state="ok", updated=datetime.now(timezone.utc).isoformat())
    except ArmPaused:
        print("\nPAUSED — touch removed pause file and re-run with --resume", flush=True)
        jpath, mpath = write_reports(doc)
        print(f"→ {jpath}\n→ {mpath}")
        return 75  # EX_TEMPFAIL
    except (ArmError, SystemExit) as exc:
        print(f"\nABORT (stop-on-error): {exc}", flush=True)
        traceback.print_exc()
        jpath, mpath = write_reports(doc)
        print(f"→ {jpath}\n→ {mpath}")
        return 1 if not isinstance(exc, SystemExit) else int(getattr(exc, "code", 1) or 1)

    jpath, mpath = write_reports(doc)
    print(f"\n→ {jpath}")
    print(f"→ {mpath}")
    release_resources()
    subprocess.call([sys.executable, str(HERE / "integrate_valhalla_reports.py")], cwd=REPO)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
