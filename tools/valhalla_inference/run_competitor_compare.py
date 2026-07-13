#!/usr/bin/env python3
"""Competitor-oriented compare — same questions · same backbone · memory/RAG peers.

Protocol: competitor-compare-v1

Default systems: full_context_lm · vanilla_rag · valhalla_base
Optional: mem0 (--with-mem0) · letta (stub until API)

Datasets (moderate): LoCoMo-15 · LongBench-10 · MultiHop-10 (Valhalla arms via ablation reuse notes)
Primary path reuses run_external_pareto_matrix backends + optional MultiHop short.

Usage:
  python3 tools/valhalla_inference/run_competitor_compare.py --moderate
  python3 tools/valhalla_inference/run_competitor_compare.py --moderate --with-mem0
  python3 tools/valhalla_inference/run_competitor_compare.py --fast
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
DOCS = REPO / "docs/valhalla"

sys.path.insert(0, str(HERE))
sys.path.insert(0, str(REPO / "tools/standard_benchmarks"))
sys.path.insert(0, str(REPO / "tools/fair_benchmark"))

from benchmark_env import apply_gpu_env, apply_resource_budget, benchmark_calibration_lm, release_resources  # noqa: E402
from common import format_mcq_prompt, load_bundled, score_short_answer, split_context_to_corpus  # noqa: E402
from metrics import score_answer  # noqa: E402
from pareto_backends import BACKENDS, RunMetrics  # noqa: E402

PROTOCOL = "competitor-compare-v1"


def locomo_items(limit: int) -> list[dict[str, Any]]:
    doc = load_bundled("locomo_qa_slice30.json")
    out: list[dict] = []
    for it in doc.get("items") or []:
        lines = [ln.strip() for ln in (it.get("session_lines") or []) if ln.strip()]
        out.append(
            {
                "id": it.get("id"),
                "question": it.get("question"),
                "reference_answer": it.get("answer"),
                "session_lines": lines,
                "corpus": lines,
                "score_type": "open",
            }
        )
        if limit > 0 and len(out) >= limit:
            break
    return out


def longbench_items(limit: int) -> list[dict[str, Any]]:
    doc = load_bundled("longbench_v2_mcq_short20.json")
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
                "corpus": corpus,
                "score_type": "mcq",
                "choice_A": it.get("choice_A"),
            }
        )
        if limit > 0 and len(out) >= limit:
            break
    return out


def multihop_items(limit: int) -> list[dict[str, Any]]:
    doc = load_bundled("multihop_rag_slice25.json")
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


def score_open(answer: str, item: dict) -> bool:
    return score_short_answer(answer, str(item.get("reference_answer") or ""))


def score_mcq(answer: str, item: dict) -> bool:
    return bool(
        score_answer(
            answer, {"reference_answer": item["reference_answer"], "score_type": "mcq"}
        ).get("correct")
    )


def run_systems(
    name: str,
    items: list[dict],
    systems: list[str],
    *,
    backbone: str,
    score_fn,
) -> dict[str, Any]:
    out: dict[str, Any] = {"n": len(items), "systems": {}}
    for sys_name in systems:
        fn = BACKENDS.get(sys_name)
        if fn is None:
            out["systems"][sys_name] = {"skipped": True, "skip_reason": "unknown system"}
            continue
        print(f"  [{name}] {sys_name} n={len(items)} …", flush=True)
        release_resources()
        kwargs: dict[str, Any] = {"items": items, "backbone": backbone, "score_fn": score_fn}
        if sys_name == "valhalla_base":
            kwargs["proactive"] = False
        try:
            metrics: RunMetrics = fn(**kwargs)
        except Exception as exc:  # noqa: BLE001
            m = RunMetrics()
            m.skipped = True
            m.skip_reason = str(exc)[:240]
            metrics = m
        summary = metrics.to_summary()
        out["systems"][sys_name] = summary
        if metrics.skipped:
            print(f"    SKIPPED: {metrics.skip_reason}", flush=True)
        else:
            print(
                f"    {summary['correct']}/{summary['n']} ({summary['accuracy']:.1%}) "
                f"tok/q={summary['avg_tokens_per_query']:.0f}",
                flush=True,
            )
    return out


def write_report(doc: dict[str, Any]) -> Path:
    REPORTS.mkdir(parents=True, exist_ok=True)
    jpath = REPORTS / "competitor_compare.json"
    mpath = REPORTS / "COMPETITOR_COMPARE_RESULTS.md"
    jpath.write_text(json.dumps(doc, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")

    lines = [
        "# Competitor Compare Results",
        "",
        f"**Protocol:** {PROTOCOL}  ",
        f"**Generated:** {doc['timestamp']}  ",
        f"**Backbone:** `{doc['backbone']}`",
        "",
        "## Matrix",
        "",
    ]
    for bench, block in (doc.get("benchmarks") or {}).items():
        lines.extend([f"### {bench} (n={block.get('n')})", "", "| System | Acc | Correct/n | Tok/q | Notes |", "|--------|-----|-----------|-------|-------|"])
        for sys_name, s in (block.get("systems") or {}).items():
            if s.get("skipped"):
                lines.append(f"| {sys_name} | SKIP | — | — | {s.get('skip_reason', '')[:60]} |")
            else:
                lines.append(
                    f"| {sys_name} | {s.get('accuracy', 0):.1%} | {s.get('correct')}/{s.get('n')} | "
                    f"{s.get('avg_tokens_per_query')} | |"
                )
        lines.append("")
    lines.extend(
        [
            "## Policy",
            "",
            "- Mem0/Letta SKIP is honest when API/deps missing — do not invent scores.",
            "- Primary claims: LoCoMo + tokens; MultiHop/LongBench = boundaries.",
            "- See `docs/valhalla/COMPETITOR_COMPARE.md`.",
            "",
        ]
    )
    mpath.write_text("\n".join(lines) + "\n", encoding="utf-8")
    # keep docs copy of policy; results stay in reports
    return mpath


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--moderate", action="store_true")
    ap.add_argument("--fast", action="store_true")
    ap.add_argument("--no-mem0", action="store_true", help="exclude mem0 arm (default: include local ST fallback)")
    ap.add_argument("--with-letta", action="store_true", help="include letta stub (usually SKIP)")
    ap.add_argument("--locomo-limit", type=int, default=0)
    ap.add_argument("--longbench-limit", type=int, default=0)
    ap.add_argument("--multihop-limit", type=int, default=0)
    ap.add_argument("--backbone", default="")
    args = ap.parse_args()

    apply_resource_budget()
    apply_gpu_env()
    os.environ.setdefault("VALHALLA_BASE_NO_SERVE", "1")
    backbone = args.backbone or benchmark_calibration_lm()
    os.environ["VALHALLA_LM_MODEL"] = backbone

    if args.fast:
        loc_n, lb_n, mh_n = 5, 3, 5
        scale = "fast"
    elif args.moderate:
        loc_n = args.locomo_limit or 15
        lb_n = args.longbench_limit or 10
        mh_n = args.multihop_limit or 10
        scale = "moderate"
    else:
        loc_n = args.locomo_limit or 30
        lb_n = args.longbench_limit or 20
        mh_n = args.multihop_limit or 15
        scale = "full"

    systems = ["full_context_lm", "vanilla_rag", "valhalla_base"]
    if not args.no_mem0:
        systems.append("mem0")
    if args.with_letta:
        systems.append("letta")

    print(f"=== {PROTOCOL} scale={scale} systems={systems} backbone={backbone} ===\n", flush=True)

    doc: dict[str, Any] = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "protocol": PROTOCOL,
        "scale": scale,
        "backbone": backbone,
        "systems": systems,
        "policy": str(DOCS / "COMPETITOR_COMPARE.md"),
        "benchmarks": {},
    }

    loc = locomo_items(loc_n)
    print(f"[LoCoMo] {len(loc)}", flush=True)
    doc["benchmarks"]["locomo"] = run_systems("locomo", loc, systems, backbone=backbone, score_fn=score_open)

    mh = multihop_items(mh_n)
    print(f"\n[MultiHop] {len(mh)}", flush=True)
    doc["benchmarks"]["multihop"] = run_systems("multihop", mh, systems, backbone=backbone, score_fn=score_open)

    lb = longbench_items(lb_n)
    print(f"\n[LongBench] {len(lb)}", flush=True)
    doc["benchmarks"]["longbench"] = run_systems("longbench", lb, systems, backbone=backbone, score_fn=score_mcq)

    path = write_report(doc)
    print(f"\n→ {path}")
    print(f"→ {REPORTS / 'competitor_compare.json'}")
    release_resources()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
