#!/usr/bin/env python3
"""External Pareto comparison matrix — unified backbone · dataset · scoring · budget.

Protocol: external-pareto-matrix-v1

Compares memory systems under identical constraints:
  full_context_lm · vanilla_rag · valhalla_base · mem0 (optional) · letta (stub)

Metrics per cell: LoCoMo acc · LongBench acc · avg tokens/query · latency · session reload · proactive

Usage:
  python3 tools/valhalla_inference/run_external_pareto_matrix.py --fast
  python3 tools/valhalla_inference/run_external_pareto_matrix.py --locomo-limit 10 --longbench-limit 10
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

REPO = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
REPORTS = REPO / "reports/valhalla_inference"
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(REPO / "tools/standard_benchmarks"))
sys.path.insert(0, str(REPO / "tools/fair_benchmark"))

from benchmark_env import apply_resource_budget, benchmark_calibration_lm, release_resources  # noqa: E402
from common import format_mcq_prompt, load_bundled, score_short_answer, split_context_to_corpus  # noqa: E402
from metrics import score_answer  # noqa: E402
from pareto_backends import BACKENDS, RunMetrics  # noqa: E402

PROTOCOL = "external-pareto-matrix-v1"
LOCOMO_BUNDLE = "locomo_qa_slice30.json"
LONGBENCH_BUNDLE = "longbench_v2_mcq_short20.json"
SYSTEM_ORDER = ["full_context_lm", "vanilla_rag", "mem0", "letta", "valhalla_base"]


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


def longbench_items(limit: int) -> list[dict[str, Any]]:
    doc = load_bundled(LONGBENCH_BUNDLE)
    out: list[dict] = []
    for it in doc.get("items") or []:
        corpus = split_context_to_corpus(it.get("context") or "", max_lines=32)
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
            }
        )
        if limit > 0 and len(out) >= limit:
            break
    return out


def score_locomo(answer: str, item: dict) -> bool:
    ref = item.get("reference_answer") or item.get("gold") or ""
    return score_short_answer(answer, str(ref))


def score_longbench(answer: str, item: dict) -> bool:
    sc = score_answer(answer, {"reference_answer": item["reference_answer"], "score_type": "mcq"})
    return bool(sc.get("correct"))


def run_dataset(
    name: str,
    items: list[dict],
    *,
    backbone: str,
    systems: list[str],
    score_fn,
) -> dict[str, Any]:
    results: dict[str, Any] = {}
    for sys_name in systems:
        fn = BACKENDS.get(sys_name)
        if fn is None:
            continue
        print(f"  [{name}] {sys_name} n={len(items)} …", flush=True)
        release_resources()
        kwargs: dict[str, Any] = {"items": items, "backbone": backbone, "score_fn": score_fn}
        if sys_name == "valhalla_base":
            kwargs["proactive"] = True
        metrics: RunMetrics = fn(**kwargs)
        results[sys_name] = metrics.to_summary()
        if not metrics.skipped:
            s = results[sys_name]
            print(
                f"    acc={s['accuracy']:.1%} tok/q={s['avg_tokens_per_query']:.0f} "
                f"lat={s['avg_latency_ms']:.0f}ms reload={s['session_reload_bytes_cum']} "
                f"proactive={s['proactive_emitted']}",
                flush=True,
            )
        else:
            print(f"    SKIPPED: {metrics.skip_reason}", flush=True)
    return results


def build_matrix(locomo: dict, longbench: dict, *, systems_run: list[str] | None = None) -> list[dict[str, Any]]:
    rows: list[dict] = []
    order = systems_run or SYSTEM_ORDER
    for sys_name in order:
        lc = locomo.get(sys_name) or {}
        lb = longbench.get(sys_name) or {}
        if lc.get("skipped") and lb.get("skipped"):
            continue
        lc_disp = "SKIPPED" if lc.get("skipped") else f"{lc.get('correct', '?')}/{lc.get('n', '?')}"
        lb_disp = "SKIPPED" if lb.get("skipped") else f"{lb.get('correct', '?')}/{lb.get('n', '?')}"
        rows.append(
            {
                "system": sys_name,
                "LoCoMo": lc_disp,
                "LongBench": lb_disp,
                "Token_cost_avg": lc.get("avg_tokens_per_query") or lb.get("avg_tokens_per_query"),
                "Latency_ms_avg": lc.get("avg_latency_ms") or lb.get("avg_latency_ms"),
                "Session_reload_bytes": lc.get("session_reload_bytes_cum"),
                "Proactive": lc.get("proactive_emitted") if sys_name == "valhalla_base" else "N/A",
                "locomo_accuracy": lc.get("accuracy"),
                "longbench_accuracy": lb.get("accuracy"),
            }
        )
    return rows


def pareto_verdict(matrix: list[dict]) -> list[str]:
    tags: list[str] = []
    vb = next((r for r in matrix if r["system"] == "valhalla_base"), {})
    fc = next((r for r in matrix if r["system"] == "full_context_lm"), {})
    if vb.get("Proactive") not in (None, "N/A", 0) or vb.get("proactive_emitted"):
        tags.append("PROACTIVE_UNIQUE")
    vb_tok = vb.get("Token_cost_avg") or 1e9
    fc_tok = fc.get("Token_cost_avg") or vb_tok
    if fc_tok and vb_tok < fc_tok * 0.6:
        tags.append("TOKEN_EFFICIENCY_PARETO")
    vb_lc = vb.get("locomo_accuracy") or 0
    fc_lc = fc.get("locomo_accuracy") or 0
    if vb_lc >= fc_lc and vb_tok <= fc_tok:
        tags.append("LOCOMO_PARETO_DOMINANCE")
    elif vb_lc < fc_lc:
        tags.append("ACC_TRADEOFF_HONEST")
    return tags or ["NEEDS_FULL_RUN"]


def write_report(doc: dict[str, Any]) -> Path:
    REPORTS.mkdir(parents=True, exist_ok=True)
    jpath = REPORTS / "external_pareto_matrix.json"
    mpath = REPORTS / "EXTERNAL_PARETO_MATRIX.md"
    jpath.write_text(json.dumps(doc, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")

    lines = [
        "# External Pareto Matrix",
        "",
        f"**Protocol:** {PROTOCOL}  ",
        f"**Generated:** {doc['timestamp']}",
        "",
        "## Locked constraints",
        "",
        f"| Field | Value |",
        f"|-------|-------|",
        f"| Backbone | `{doc['constraints']['backbone']}` |",
        f"| LoCoMo items | {doc['constraints']['locomo_n']} |",
        f"| LongBench items | {doc['constraints']['longbench_n']} |",
        f"| Scoring | LoCoMo: `score_short_answer` · LongBench: MCQ letter |",
        f"| Budget | ingress 50% · LM cache max 1 |",
        "",
        "## Matrix",
        "",
        "| System | LoCoMo | LongBench | Token/query | Latency ms | Session reload | Proactive |",
        "|--------|--------|-----------|-------------|------------|----------------|-----------|",
    ]
    for row in doc["matrix"]:
        lines.append(
            f"| {row['system']} | {row['LoCoMo']} | {row['LongBench']} | "
            f"{row.get('Token_cost_avg', '—')} | {row.get('Latency_ms_avg', '—')} | "
            f"{row.get('Session_reload_bytes', '—')} | {row.get('Proactive', 'N/A')} |"
        )
    lines.extend(["", f"**Verdict:** {', '.join(doc.get('verdict') or [])}", "", "---", ""])
    mpath.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return mpath


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--fast", action="store_true")
    ap.add_argument("--locomo-limit", type=int, default=0)
    ap.add_argument("--longbench-limit", type=int, default=0)
    ap.add_argument("--backbone", default="")
    ap.add_argument("--systems", default=",".join(SYSTEM_ORDER))
    args = ap.parse_args()

    apply_resource_budget()
    os.environ.setdefault("VALHALLA_BASE_NO_SERVE", "1")
    backbone = args.backbone or benchmark_calibration_lm()
    os.environ["VALHALLA_LM_MODEL"] = backbone

    loc_limit = args.locomo_limit or (3 if args.fast else 10)
    lb_limit = args.longbench_limit or (3 if args.fast else 10)
    systems = [s.strip() for s in args.systems.split(",") if s.strip()]
    if args.fast and args.systems == ",".join(SYSTEM_ORDER):
        systems = ["full_context_lm", "vanilla_rag", "valhalla_base"]

    print(f"=== {PROTOCOL} backbone={backbone} ===\n", flush=True)
    loc_items = locomo_items(loc_limit)
    lb_items = longbench_items(lb_limit)

    print(f"[LoCoMo] {len(loc_items)} items", flush=True)
    loc_results = run_dataset("LoCoMo", loc_items, backbone=backbone, systems=systems, score_fn=score_locomo)

    print(f"\n[LongBench-v2] {len(lb_items)} items", flush=True)
    lb_results = run_dataset("LongBench", lb_items, backbone=backbone, systems=systems, score_fn=score_longbench)

    matrix = build_matrix(loc_results, lb_results, systems_run=systems)
    doc = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "protocol": PROTOCOL,
        "constraints": {
            "backbone": backbone,
            "locomo_n": len(loc_items),
            "longbench_n": len(lb_items),
            "systems": systems,
            "scoring": "locomo: score_short_answer; longbench: mcq letter",
            "hardware_note": "local run — record GPU model in diligence pack",
        },
        "locomo": loc_results,
        "longbench": lb_results,
        "matrix": matrix,
        "verdict": pareto_verdict(matrix),
    }
    path = write_report(doc)
    print(f"\n→ {path}")
    integrate = __import__("subprocess").call(
        [sys.executable, str(HERE / "integrate_valhalla_reports.py")], cwd=REPO
    )
    return integrate


if __name__ == "__main__":
    raise SystemExit(main())
