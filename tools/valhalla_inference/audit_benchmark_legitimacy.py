#!/usr/bin/env python3
"""Audit benchmark bundles and recent reports for citeability (Tier A/B/C).

Usage:
  python3 tools/valhalla_inference/audit_benchmark_legitimacy.py
  python3 tools/valhalla_inference/audit_benchmark_legitimacy.py --json
"""

from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

REPO = Path(__file__).resolve().parents[2]
REPORTS = REPO / "reports" / "valhalla_inference"
STD = REPO / "tools" / "standard_benchmarks"
sys.path.insert(0, str(STD))

from common import frames_bundle_preflight  # noqa: E402


def _load_report(name: str) -> dict[str, Any] | None:
    p = REPORTS / name
    if not p.is_file():
        return None
    try:
        return json.loads(p.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return None


def check_frames() -> dict[str, Any]:
    pre = frames_bundle_preflight("frames_slice25.json")
    rep = _load_report("frames_v1_test.json")
    tier = pre["tier"]
    note = "Tier C — no Wikipedia extracts; score NOT citeable"
    if pre["viable"]:
        note = "Tier A/B — wiki corpus present; score citeable after re-run"
    if rep and not pre["viable"]:
        note += f"; stale report shows {rep.get('correct')}/{rep.get('n')} — discard"
    return {
        "benchmark": "FRAMES",
        "protocol": "frames-v1",
        "bundle_viable": pre["viable"],
        "tier": tier,
        "preflight": pre,
        "report": {
            "exists": rep is not None,
            "correct": rep.get("correct") if rep else None,
            "n": rep.get("n") if rep else None,
            "wiki_items_fetched": rep.get("wiki_items_fetched") if rep else None,
        },
        "citeable": pre["viable"],
        "note": note,
    }


def check_bundle(name: str, min_items: int = 1) -> dict[str, Any]:
    p = STD / "bundled" / name
    if not p.is_file():
        return {"file": name, "ok": False, "tier": "C", "note": "missing bundle", "n": 0}
    doc = json.loads(p.read_text(encoding="utf-8"))
    items = doc.get("items") or doc.get("questions") or doc.get("qa") or []
    n = len(items) if isinstance(items, list) else 0
    ok = n >= min_items
    return {
        "file": name,
        "ok": ok,
        "n": n,
        "tier": "A" if ok else "C",
        "note": "bundle present" if ok else "empty or corrupt",
    }


def check_report_legit(
    name: str,
    *,
    protocol: str,
    min_n: int = 1,
    min_n_fast: int = 0,
    score_key: str = "correct",
    required_keys: tuple[str, ...] = ("n", "verdict"),
) -> dict[str, Any]:
    rep = _load_report(name)
    if rep is None:
        return {
            "benchmark": name,
            "protocol": protocol,
            "report_exists": False,
            "tier": "—",
            "citeable": False,
            "note": "no report yet",
        }
    missing = [k for k in required_keys if k not in rep]
    n = int(rep.get("n") or rep.get("n_questions") or 0)
    score = rep.get(score_key)
    if score is None and score_key == "correct":
        score = rep.get("faithful")
    if score is None and name == "lara_v1_test.json":
        score = (rep.get("valhalla_rag") or {}).get("correct")
    valid = not missing and score is not None
    if valid:
        if n >= min_n:
            tier = "A"
        elif min_n_fast and n >= min_n_fast:
            tier = "B"
            valid = True
        else:
            tier = "C"
            valid = False
    else:
        tier = "C"
    # protocol-specific invalidation
    if name == "frames_v1_test.json":
        wf = int(rep.get("wiki_items_fetched") or 0)
        if wf == 0 or not frames_bundle_preflight("frames_slice25.json")["viable"]:
            valid = False
            tier = "C"
            note = "Tier C — no wiki corpus; discard stale scores"
        else:
            note = "valid report" if valid else f"invalid: missing={missing} n={n}"
    elif name == "valhalla_independence_test.json":
        summary = rep.get("summary") or {}
        valid = bool(summary.get("verdict")) and n >= min_n
        score = summary.get("backbones_tested")
        tier = "A" if valid else "C"
        note = "valid report" if valid else "missing summary verdict"
    else:
        note = "valid report" if valid else f"invalid: missing={missing} n={n}"
        if tier == "B":
            note = f"fast slice (n={n}); cite with --fast label"
    return {
        "benchmark": name.replace("_test.json", "").replace("_v1", ""),
        "protocol": protocol,
        "report_exists": True,
        "correct": score,
        "n": n,
        "verdict": rep.get("verdict") or (rep.get("summary") or {}).get("verdict"),
        "tier": tier,
        "citeable": valid,
        "fast_slice": tier == "B",
        "missing_keys": missing,
        "note": note,
    }


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()

    doc: dict[str, Any] = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "bundles": {
            "ragtruth": check_bundle("ragtruth_qa_test40.json", 10),
            "needle": check_bundle("ruler_v1_slice20.json", 1),
            "locomo": check_bundle("locomo_qa_slice30.json", 5),
            "multihop": check_bundle("multihop_rag_slice25.json", 5),
            "frames": check_frames(),
        },
        "reports": [
            check_report_legit("ragtruth_v1_test.json", protocol="ragtruth-v1", min_n=10, score_key="faithful"),
            check_report_legit("ragtruth_full_v1_test.json", protocol="ragtruth-full-v1", min_n=50, min_n_fast=10, score_key="faithful"),
            check_report_legit("needle_session_v1_test.json", protocol="needle-session-v1", min_n=1),
            check_report_legit("longbench_v2_v1_test.json", protocol="longbench-v2-v1", min_n=5),
            check_report_legit("frames_v1_test.json", protocol="frames-v1", min_n=5),
            check_report_legit("multihop_rag_hybrid_test.json", protocol="multihop-rag-v1", min_n=5),
            check_report_legit("locomo_v1_test.json", protocol="locomo-v1", min_n=5, min_n_fast=3),
            check_report_legit("ruler_v1_test.json", protocol="ruler-v1", min_n=5, min_n_fast=4),
            check_report_legit("lara_v1_test.json", protocol="lara-v1", min_n=5, required_keys=("n",)),
            check_report_legit("external_holdout_test.json", protocol="external-holdout-v1", min_n=5),
            check_report_legit(
                "valhalla_independence_test.json",
                protocol="valhalla-independence-v1",
                min_n=1,
                required_keys=(),
            ),
        ],
    }

    citeable = sum(1 for r in doc["reports"] if r.get("citeable"))
    total = len(doc["reports"])
    doc["summary"] = {
        "reports_citeable": citeable,
        "reports_total": total,
        "frames_bundle_viable": doc["bundles"]["frames"]["bundle_viable"],
    }

    if args.json:
        print(json.dumps(doc, indent=2, ensure_ascii=False))
    else:
        print(f"Benchmark legitimacy audit @ {doc['timestamp']}\n")
        print("## Bundles")
        for k, v in doc["bundles"].items():
            if k == "frames":
                print(f"  {k}: tier={v['tier']} viable={v['bundle_viable']} — {v['note']}")
            else:
                print(f"  {k}: tier={v.get('tier')} n={v.get('n')} ok={v.get('ok')}")
        print("\n## Reports (citeability)")
        for r in doc["reports"]:
            tag = "CITE" if r.get("citeable") else "NO"
            print(
                f"  [{tag}] {r.get('benchmark','?'):20} "
                f"tier={r.get('tier')} "
                f"result={r.get('correct','—')}/{r.get('n','—')} "
                f"verdict={r.get('verdict','—')}"
            )
            if r.get("note") and not r.get("citeable"):
                print(f"         → {r['note']}")
        print(f"\nSummary: {citeable}/{total} reports citeable; FRAMES bundle viable={doc['summary']['frames_bundle_viable']}")

    # exit 1 if FRAMES falsely citeable in reports but bundle bad
    frames_rep = next(r for r in doc["reports"] if "frames" in r.get("benchmark", ""))
    if frames_rep.get("citeable") and not doc["summary"]["frames_bundle_viable"]:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
