#!/usr/bin/env python3
"""
VPB Phase C — milestone velocity + updated PFI with scale headroom.

Usage:
  python run_vpb_forecast.py
  python run_vpb_forecast.py --scale-json reports/vpb/vpb_scale_test_*.json
"""

from __future__ import annotations

import argparse
import glob
import json
import sys
from datetime import datetime
from pathlib import Path
from typing import Any

REPO = Path(__file__).resolve().parents[2]
FAIR = Path(__file__).resolve().parent
OUT_DIR = REPO / "reports/vpb"

sys.path.insert(0, str(FAIR))

MILESTONES = [
    {
        "id": "M20260620_0859",
        "label": "fair_benchmark_baseline",
        "date": "2026-06-20",
        "source": "reports/fair_benchmark/fair_benchmark_test_20260620_0859.json",
        "metric": "triad_c1_persistent_acc",
    },
    {
        "id": "M20260620_1300",
        "label": "vpb_matrix_phase_a",
        "date": "2026-06-20",
        "source": "reports/vpb/vpb_matrix_baseline_20260620_1300.json",
        "metric": "pfi",
    },
]


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    return json.loads(path.read_text(encoding="utf-8"))


def latest_glob(pattern: str) -> Path | None:
    hits = sorted(glob.glob(pattern))
    return Path(hits[-1]) if hits else None


def extract_milestone_value(ms: dict) -> float | None:
    p = REPO / ms["source"]
    data = load_json(p)
    if not data:
        return None
    if ms["metric"] == "triad_c1_persistent_acc":
        for r in data.get("results", []):
            if r.get("arm") == "triad_c1_persistent":
                return r["summary"]["acc"]
        return None
    if ms["metric"] == "pfi":
        return data.get("pfi", {}).get("pfi")
    return None


def velocity_pp_per_month(m0: float, m1: float, days: float) -> float:
    if days <= 0:
        return 0.0
    months = days / 30.0
    return (m1 - m0) / months * 100 if months else 0.0


def updated_pfi(
    matrix_pfi: dict,
    scale_curve: dict | None,
    velocity_norm: float,
) -> dict[str, Any]:
    comp = matrix_pfi.get("components", {})
    scale_norm = (scale_curve or {}).get("scale_headroom_norm", 0.5)
    pfi = (
        0.25 * comp.get("recall_norm", 0.5)
        + 0.20 * comp.get("ingest_lift_norm", 0.5)
        + 0.20 * comp.get("wedge_score", 0.5)
        + 0.15 * min(max(comp.get("hybrid_minus_tf", 0) + 5, 0) / 10.0, 1.0)
        + 0.10 * scale_norm
        + 0.10 * velocity_norm
    )
    pfi = round(min(1.0, pfi), 4)
    label = "strong_potential" if pfi >= 0.70 else ("viable_wedge" if pfi >= 0.50 else "early_stage")
    return {
        "pfi": pfi,
        "interpretation": label,
        "components": {
            **comp,
            "scale_headroom_norm": scale_norm,
            "velocity_norm": velocity_norm,
        },
    }


def write_forecast_report(payload: dict, path: Path) -> None:
    pfi = payload["pfi_final"]
    lines = [
        "# VPB Forecast Report (Phase B + C)",
        "",
        f"**Run**: {payload['meta']['timestamp']}",
        "",
        "## Final PFI (with scale + velocity)",
        "",
        f"| PFI | {pfi['pfi']:.2f} ({pfi['interpretation']}) |",
        "",
        "### Components",
        "",
    ]
    for k, v in pfi["components"].items():
        lines.append(f"- {k}: {v}")
    lines.extend(["", "## Milestones", ""])
    for m in payload["milestones"]:
        lines.append(f"- **{m['id']}** ({m['label']}): {m.get('value')}")
    if payload.get("velocity"):
        v = payload["velocity"]
        lines.extend(
            [
                "",
                "## Velocity",
                "",
                f"- Δ hybrid proxy: {v.get('delta_pp', '—')} pp over {v.get('days', '—')} days",
                f"- velocity_norm: {v.get('velocity_norm')}",
            ]
        )
    if payload.get("scale"):
        sc = payload["scale"]
        lines.extend(
            [
                "",
                "## Scale curve (Fair L0)",
                "",
                f"- acc by tier: `{sc.get('corpus_supported_acc_by_tier')}`",
                f"- recall@3 by tier: `{sc.get('recall_at_3_by_tier')}`",
                f"- memory_horizon_tier: {sc.get('memory_horizon_tier')}",
            ]
        )
    if payload.get("memory_L1"):
        lines.extend(["", "## Memory L1 (Phase B)", ""])
        for t, m in payload["memory_L1"].items():
            lines.append(
                f"- **{t}**: acc {m['persistent']['acc']*100:.1f}% · "
                f"abstention {m.get('abstention_acc')} · evidence_recall {m.get('evidence_recall_at_3')}"
            )
    lines.extend(["", f"*VPB forecast · {payload['meta']['timestamp']}*"])
    path.write_text("\n".join(lines), encoding="utf-8")


def main() -> None:
    ap = argparse.ArgumentParser(description="VPB forecast + milestone PFI")
    ap.add_argument("--matrix-json", type=Path, default=None)
    ap.add_argument("--scale-json", type=Path, default=None)
    args = ap.parse_args()

    matrix_path = args.matrix_json or latest_glob(str(REPO / "reports/vpb/vpb_matrix_baseline_*.json"))
    scale_path = args.scale_json or latest_glob(str(REPO / "reports/vpb/vpb_scale_test_*.json"))
    if not matrix_path:
        scale_path = scale_path or latest_glob(str(REPO / "reports/vpb/vpb_scale_smoke_*.json"))

    matrix = load_json(matrix_path) if matrix_path else None
    scale_doc = load_json(scale_path) if scale_path else None

    milestones = []
    for ms in MILESTONES:
        milestones.append({**ms, "value": extract_milestone_value(ms)})

    velocity = {"velocity_norm": 0.5, "delta_pp": None, "days": None}
    v0 = milestones[0].get("value")
    v1 = matrix["pfi"]["pfi"] if matrix else milestones[1].get("value")
    if v0 is not None and v1 is not None:
        # proxy: PFI vs acc on 0-1 scale — use hybrid delta from matrix
        if matrix:
            hybrid = matrix["matrix"]["S6_hybrid_product"]["M1_task_acc_hybrid"]
            tf = matrix["matrix"]["S6_hybrid_product"]["M1_task_acc_tf"]
            delta_pp = (hybrid - tf) * 100
            velocity = {
                "delta_pp": round(delta_pp, 2),
                "days": 1,
                "velocity_norm": min(1.0, max(0.0, 0.5 + delta_pp / 20.0)),
            }

    scale_curve = None
    memory_L1 = None
    if scale_doc:
        fair = scale_doc.get("results", {}).get("fair_L0", {})
        scale_curve = fair.get("scale_curve")
        memory_L1 = scale_doc.get("results", {}).get("memory_L1", {}).get("metrics_by_tier")

    pfi_final = updated_pfi(
        matrix["pfi"] if matrix else {"components": {}},
        scale_curve,
        velocity["velocity_norm"],
    )

    tag = datetime.now().strftime("%Y%m%d_%H%M")
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    payload = {
        "meta": {
            "benchmark": "valhalla_potential_forecast_v1",
            "timestamp": tag,
            "matrix_json": str(matrix_path) if matrix_path else None,
            "scale_json": str(scale_path) if scale_path else None,
        },
        "milestones": milestones,
        "velocity": velocity,
        "scale": scale_curve,
        "memory_L1": memory_L1,
        "pfi_final": pfi_final,
        "gates": matrix.get("gates") if matrix else None,
    }

    out_json = OUT_DIR / f"vpb_forecast_{tag}.json"
    out_md = REPO / "reports" / f"VALHALLA_VPB_FORECAST_{tag}.md"
    out_json.write_text(json.dumps(payload, indent=2, ensure_ascii=False), encoding="utf-8")
    write_forecast_report(payload, out_md)
    print(f"Wrote {out_json}", file=sys.stderr)
    print(f"PFI_final={pfi_final['pfi']:.2f} scale={scale_curve}", file=sys.stderr)


if __name__ == "__main__":
    main()
