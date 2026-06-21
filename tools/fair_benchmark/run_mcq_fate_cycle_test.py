#!/usr/bin/env python3
"""
MCQ Fate multi-cycle test — task_tag mcq, decompose → aggregate, Fate-controlled rounds.

Compares:
  - baseline: persistent whole-question, qa_cycles=1, VALHALLA_MCQ_FATE_CYCLES=0
  - fate:     same protocol + VALHALLA_MCQ_FATE_CYCLES=1

Usage:
  python run_mcq_fate_cycle_test.py
  python run_mcq_fate_cycle_test.py --body triad stemcell tile
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from collections import Counter
from datetime import datetime
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
FAIR = Path(__file__).resolve().parent
OUT = REPO / "reports/experiments"
BASELINE = REPO / "reports/fair_benchmark/fair_benchmark_test_20260620_0859.json"
ARTIFACT = REPO / "checkpoints/incubation_artifacts/fair_triad_c1"

sys.path.insert(0, str(FAIR))
sys.path.insert(0, str(REPO / "tools/valhalla_artifact"))

from artifact import load_artifact  # noqa: E402
from metrics import score_answer, summarize_rows  # noqa: E402
from run_fair_benchmark import load_suite, select_prompts  # noqa: E402
from run_strict_incubation_experiment import strict_user_text  # noqa: E402
from session import native_qa_batch_persistent  # noqa: E402


def mcq_prompts() -> list[dict]:
    return [p for p in select_prompts(load_suite(), "test") if p.get("score_type") == "mcq"]


def baseline_mcq(arm: str = "triad_c1_persistent") -> tuple[int, int]:
    if not BASELINE.exists():
        return 21, 46
    data = json.loads(BASELINE.read_text())
    for r in data["results"]:
        if r.get("arm") == arm:
            sub = [x for x in r.get("rows", []) if x.get("score_type") == "mcq"]
            return sum(x.get("correct", False) for x in sub), len(sub)
    return 21, 46


def run_arm(body: str, corpus: list[str], prompts: list[dict], fate: bool) -> dict:
    prev_fate = os.environ.get("VALHALLA_MCQ_FATE_CYCLES")
    prev_max = os.environ.get("VALHALLA_MCQ_FATE_MAX_CYCLES")
    os.environ["VALHALLA_MCQ_FATE_CYCLES"] = "1" if fate else "0"
    if fate:
        os.environ.setdefault("VALHALLA_MCQ_FATE_MAX_CYCLES", "64")
        os.environ.setdefault("VALHALLA_MCQ_FATE_HARD_MAX", "128")
        os.environ.setdefault("VALHALLA_MCQ_FATE_MIN_CYCLES", "5")
    try:
        batch_q = [
            (strict_user_text(p), p.get("score_type", "open"), p["id"]) for p in prompts
        ]
        results = native_qa_batch_persistent(
            body,
            corpus,
            batch_q,
            incubation_cycles=1,
            qa_cycles=1,
        )
        by_id = {r.get("id"): r for r in results}
        rows = []
        for p in prompts:
            resp = by_id.get(p["id"], {})
            sc = score_answer(resp.get("answer", ""), p, strict_mcq=True)
            rows.append(
                {
                    "id": p["id"],
                    "correct": sc["correct"],
                    "extracted": sc.get("extracted"),
                    "decode_path": resp.get("decode_path"),
                    "mcq_fate_cycles": resp.get("mcq_fate_cycles", 0),
                    "mcq_fate_stopped": resp.get("mcq_fate_stopped", ""),
                    "mcq_decompose_score": resp.get("mcq_decompose_score", 0.0),
                    "mcq_aggregate_score": resp.get("mcq_aggregate_score", 0.0),
                }
            )
        mcq_rows = rows
        stop_reasons = Counter(r["mcq_fate_stopped"] for r in mcq_rows if r["mcq_fate_stopped"])
        cycles = [r["mcq_fate_cycles"] for r in mcq_rows if r["mcq_fate_cycles"]]
        return {
            "body": body,
            "fate_cycles": fate,
            "summary": summarize_rows(mcq_rows),
            "decode_paths": dict(Counter(r["decode_path"] for r in mcq_rows)),
            "stop_reasons": dict(stop_reasons),
            "avg_fate_cycles": sum(cycles) / len(cycles) if cycles else 0,
            "avg_decompose": sum(r["mcq_decompose_score"] for r in mcq_rows) / len(mcq_rows),
            "avg_aggregate": sum(r["mcq_aggregate_score"] for r in mcq_rows) / len(mcq_rows),
            "rows": mcq_rows,
        }
    finally:
        if prev_fate is None:
            os.environ.pop("VALHALLA_MCQ_FATE_CYCLES", None)
        else:
            os.environ["VALHALLA_MCQ_FATE_CYCLES"] = prev_fate
        if prev_max is None:
            os.environ.pop("VALHALLA_MCQ_FATE_MAX_CYCLES", None)
        else:
            os.environ["VALHALLA_MCQ_FATE_MAX_CYCLES"] = prev_max


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--body", nargs="+", default=["triad", "tile", "stemcell"])
    args = ap.parse_args()

    prompts = mcq_prompts()
    corpus = load_artifact(ARTIFACT).corpus
    b_ok, b_n = baseline_mcq()

    payload: dict = {
        "meta": {
            "timestamp": datetime.now().strftime("%Y%m%d_%H%M"),
            "mcq_count": len(prompts),
            "baseline_persistent": {"correct": b_ok, "total": b_n},
            "protocol": "persistent whole-question + mcq_fate decompose/aggregate",
        },
        "results": {},
    }

    for body in args.body:
        print(f"=== {body} baseline (fate off) ===", file=sys.stderr)
        base = run_arm(body, corpus, prompts, fate=False)
        print(f"=== {body} fate cycles (fate on) ===", file=sys.stderr)
        fate = run_arm(body, corpus, prompts, fate=True)
        payload["results"][body] = {"baseline": base, "fate": fate}

    tag = payload["meta"]["timestamp"]
    out_json = OUT / f"MCQ_FATE_CYCLE_{tag}.json"
    out_md = OUT / f"MCQ_FATE_CYCLE_{tag}.md"
    OUT.mkdir(parents=True, exist_ok=True)
    out_json.write_text(json.dumps(payload, indent=2, ensure_ascii=False), encoding="utf-8")

    lines = [
        "# MCQ Fate 多循环实验（task_tag + 分解/聚合）",
        "",
        f"**Time**: {tag} · **MCQ**: {len(prompts)}Q · baseline persistent: **{b_ok}/{b_n}**",
        "",
        "## 机制",
        "",
        "- **Tag**: `task_tag:mcq` 写入 Hub/Tile/Stem Triple-Fate",
        "- **分解**: stem + 各 option 分 shard ingress → optimize cycle",
        "- **聚合**: 整题 re-ingress → **Fate 动态决定轮次**（收敛投票 + 边际稳定）→ decode `mcq_option`",
        "- **开关**: `VALHALLA_MCQ_FATE_CYCLES=1`",
        "- **Env**: `MAX_CYCLES=64` (soft) · `HARD_MAX=128` · `MIN_CYCLES=5`",
        "",
        "## 结果",
        "",
        "| Body | baseline MCQ | fate MCQ | Δ | avg cycles | stop (top) | decode |",
        "|------|--------------|----------|---|------------|------------|--------|",
    ]

    for body, pair in payload["results"].items():
        b = pair["baseline"]["summary"]
        f = pair["fate"]["summary"]
        delta = f["correct"] - b["correct"]
        top_stop = max(pair["fate"]["stop_reasons"].items(), key=lambda x: x[1])[0] if pair["fate"]["stop_reasons"] else "—"
        lines.append(
            f"| **{body}** | {b['correct']}/{b['n']} ({b['acc']:.1%}) | "
            f"**{f['correct']}/{f['n']} ({f['acc']:.1%})** | {delta:+d} | "
            f"{pair['fate']['avg_fate_cycles']:.1f} | {top_stop} | `{pair['fate']['decode_paths']}` |"
        )

    lines.extend(["", f"JSON: `{out_json.relative_to(REPO)}`", ""])
    out_md.write_text("\n".join(lines), encoding="utf-8")

    print(f"\n{'body':<10} {'base':>8} {'fate':>8} {'Δ':>4} {'avg_cyc':>8}")
    for body, pair in payload["results"].items():
        b = pair["baseline"]["summary"]["correct"]
        f = pair["fate"]["summary"]["correct"]
        ac = pair["fate"]["avg_fate_cycles"]
        print(f"{body:<10} {b}/{len(prompts):>3} {f}/{len(prompts):>3} {f-b:+4d} {ac:8.1f}")
    print(f"Wrote {out_md}")


if __name__ == "__main__":
    main()
