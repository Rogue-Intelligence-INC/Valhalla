#!/usr/bin/env python3
"""Consolidate Valhalla reports — pillars · independence · industry · legitimacy.

Usage:
  python3 tools/valhalla_inference/integrate_valhalla_reports.py
  python3 tools/valhalla_inference/integrate_valhalla_reports.py --json
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

REPO = Path(__file__).resolve().parents[2]
REPORTS = REPO / "reports" / "valhalla_inference"
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))


def _load(name: str) -> dict[str, Any] | None:
    p = REPORTS / name
    if not p.is_file():
        return None
    try:
        return json.loads(p.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return None


def _audit() -> dict[str, Any]:
    proc = subprocess.run(
        [sys.executable, str(HERE / "audit_benchmark_legitimacy.py"), "--json"],
        capture_output=True,
        text=True,
        cwd=REPO,
    )
    if proc.returncode == 0 and proc.stdout.strip():
        return json.loads(proc.stdout)
    return {"error": proc.stderr or "audit failed"}


def build_integration() -> dict[str, Any]:
    indep = _load("valhalla_independence_test.json")
    industry = _load("industry_benchmarks_summary.json")
    proactive = _load("proactive_fate_v2_test.json")
    ragtruth = _load("ragtruth_v1_test.json")
    needle = _load("needle_session_v1_test.json")
    pareto = _load("external_pareto_matrix.json")

    doc: dict[str, Any] = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "resource_budget": {
            "ingress_percent": 50,
            "ingress_max_items": 48,
            "lm_cache_max": 1,
            "serve_mode": "subprocess_default",
        },
        "pillars": {
            "ragtruth": {
                "faithful": ragtruth.get("faithful") if ragtruth else None,
                "n": ragtruth.get("n") if ragtruth else None,
                "verdict": ragtruth.get("verdict") if ragtruth else None,
            },
            "needle": {
                "correct": needle.get("correct") if needle else None,
                "n": needle.get("n") if needle else None,
                "verdict": needle.get("verdict") if needle else None,
            },
            "proactive": {
                "pass_rate": (proactive.get("summary") or {}).get("passed") if proactive else None,
                "ppi_v2": (proactive.get("metrics") or {}).get("PPI_v2") if proactive else None,
                "verdict": proactive.get("verdict") if proactive else None,
            },
        },
        "independence": {
            "protocol": indep.get("protocol") if indep else None,
            "panel": (indep.get("summary") or {}).get("panel") if indep else None,
            "verdict": (indep.get("summary") or {}).get("verdict") if indep else None,
            "causal_tested": (indep.get("summary") or {}).get("causal_tested") if indep else None,
            "appendix_count": (indep.get("summary") or {}).get("appendix_count") if indep else None,
            "memory_verdict": (indep.get("memory") or {}).get("verdict") if indep else None,
        },
        "industry": industry.get("benchmarks") if industry else [],
        "pareto_matrix": {
            "protocol": pareto.get("protocol") if pareto else None,
            "verdict": pareto.get("verdict") if pareto else None,
            "backbone": (pareto.get("constraints") or {}).get("backbone") if pareto else None,
            "matrix": pareto.get("matrix") if pareto else None,
        },
        "legitimacy": _audit(),
    }

    citeable = 0
    total = 0
    for r in (doc.get("legitimacy") or {}).get("reports") or []:
        total += 1
        if r.get("citeable"):
            citeable += 1
    doc["summary"] = {
        "independence": doc["independence"].get("verdict"),
        "industry_rows": len(doc.get("industry") or []),
        "reports_citeable": citeable,
        "reports_total": total,
        "frames_viable": (doc.get("legitimacy") or {}).get("summary", {}).get("frames_bundle_viable"),
    }
    return doc


def write_md(doc: dict[str, Any]) -> Path:
    out = REPORTS / "VALHALLA_INTEGRATED_STATUS.md"
    lines = [
        "# Valhalla — Integrated Status",
        "",
        f"**Generated:** {doc['timestamp']}",
        "",
        "## Resource budget",
        "",
        f"- Ingress: **{doc['resource_budget']['ingress_percent']}%** · max_items **{doc['resource_budget']['ingress_max_items']}**",
        f"- LM cache max: **{doc['resource_budget']['lm_cache_max']}** (serve `/v1/release`)",
        f"- Default: **subprocess** (`VALHALLA_BASE_NO_SERVE=1`) — no persistent 3B serve during suite",
        "",
        "## Pillars",
        "",
        "| Pillar | Result | Verdict |",
        "|--------|--------|---------|",
    ]
    for name, row in (doc.get("pillars") or {}).items():
        if name == "ragtruth":
            res = f"{row.get('faithful')}/{row.get('n')}"
        elif name == "proactive":
            res = f"PPI {row.get('ppi_v2')}"
        else:
            res = f"{row.get('correct')}/{row.get('n')}"
        lines.append(f"| {name} | {res} | {row.get('verdict', '—')} |")

    ind = doc.get("independence") or {}
    lines.extend(
        [
            "",
            "## Independence",
            "",
            f"- Panel: `{ind.get('panel', '—')}`",
            f"- Verdict: **{ind.get('verdict', '—')}**",
            f"- Causal tested: {ind.get('causal_tested', '—')}",
            f"- Appendix (encoder/vision): {ind.get('appendix_count', 0)}",
            f"- Memory: {ind.get('memory_verdict', '—')}",
            "",
            "## Industry (latest)",
            "",
            "| Benchmark | Result | Verdict |",
            "|-----------|--------|---------|",
        ]
    )
    for row in doc.get("industry") or []:
        lines.append(f"| {row.get('name')} | {row.get('result')} | {row.get('verdict')} |")

    pm = doc.get("pareto_matrix") or {}
    if pm.get("matrix"):
        lines.extend(
            [
                "",
                "## External Pareto matrix",
                "",
                f"- Backbone: `{pm.get('backbone')}`",
                f"- Verdict: **{', '.join(pm.get('verdict') or [])}**",
                "",
                "| System | LoCoMo | LongBench | Proactive |",
                "|--------|--------|-----------|-----------|",
            ]
        )
        for row in pm.get("matrix") or []:
            lines.append(
                f"| {row.get('system')} | {row.get('LoCoMo')} | {row.get('LongBench')} | {row.get('Proactive', 'N/A')} |"
            )

    s = doc.get("summary") or {}
    lines.extend(
        [
            "",
            "## Legitimacy",
            "",
            f"- Citeable reports: **{s.get('reports_citeable')}/{s.get('reports_total')}**",
            f"- FRAMES bundle viable: **{s.get('frames_viable')}**",
            "",
            "---",
            "",
            "*Rogue Intelligence LNC.*",
        ]
    )
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return out


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()

    doc = build_integration()
    json_path = REPORTS / "valhalla_integrated_status.json"
    json_path.write_text(json.dumps(doc, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    md_path = write_md(doc)

    if args.json:
        print(json.dumps(doc, indent=2, ensure_ascii=False))
    else:
        print(f"→ {json_path}")
        print(f"→ {md_path}")
        s = doc.get("summary") or {}
        print(
            f"integrated: independence={s.get('independence')} "
            f"citeable={s.get('reports_citeable')}/{s.get('reports_total')} "
            f"frames_viable={s.get('frames_viable')}"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
