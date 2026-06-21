#!/usr/bin/env python3
"""
MCQ experiments — (3) multi-cycle persistent vs (4) line-level entailment scorer.

Usage:
  python run_mcq_experiments_3_4.py
  python run_mcq_experiments_3_4.py --exp 3
  python run_mcq_experiments_3_4.py --exp 4
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

sys.path.insert(0, str(FAIR))
sys.path.insert(0, str(REPO / "tools/valhalla_artifact"))

from artifact import load_artifact
from metrics import score_answer, summarize_rows
from run_fair_benchmark import load_suite, select_prompts
from run_strict_incubation_experiment import strict_user_text
from session import native_qa_batch_persistent

CYCLES = (1, 5, 10, 20)
ARTIFACT = REPO / "checkpoints/incubation_artifacts/fair_triad_c1"


def mcq_prompts() -> list[dict]:
    return [p for p in select_prompts(load_suite(), "test") if p.get("score_type") == "mcq"]


def run_persistent_mcq(incubation_cycles: int, entailment: bool = False) -> dict:
    corpus = load_artifact(ARTIFACT).corpus
    prompts = mcq_prompts()
    prev = os.environ.get("VALHALLA_MCQ_ENTAILMENT")
    os.environ["VALHALLA_MCQ_ENTAILMENT"] = "1" if entailment else "0"
    try:
        batch_q = [
            (strict_user_text(p), p.get("score_type", "open"), p["id"]) for p in prompts
        ]
        results = native_qa_batch_persistent(
            "triad",
            corpus,
            batch_q,
            incubation_cycles=incubation_cycles,
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
                    "decode_path": resp.get("decode_path"),
                    "domain": p.get("domain"),
                }
            )
        return {
            "incubation_cycles": incubation_cycles,
            "entailment": entailment,
            "summary": summarize_rows(rows),
            "decode_paths": dict(Counter(r["decode_path"] for r in rows)),
            "rows": rows,
        }
    finally:
        if prev is None:
            os.environ.pop("VALHALLA_MCQ_ENTAILMENT", None)
        else:
            os.environ["VALHALLA_MCQ_ENTAILMENT"] = prev


def baseline_mcq() -> tuple[int, int]:
    if not BASELINE.exists():
        return 21, 46
    data = json.loads(BASELINE.read_text())
    for r in data["results"]:
        if r.get("arm") == "triad_c1_persistent":
            sub = [x for x in r["rows"] if x.get("score_type") == "mcq"]
            return sum(x["correct"] for x in sub), len(sub)
    return 21, 46


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--exp", choices=("3", "4", "both"), default="both")
    args = ap.parse_args()

    b_ok, b_n = baseline_mcq()
    payload: dict = {
        "meta": {
            "timestamp": datetime.now().strftime("%Y%m%d_%H%M"),
            "mcq_count": len(mcq_prompts()),
            "baseline_persistent_c1": {"correct": b_ok, "total": b_n},
        },
        "experiment_3_multicycle": [],
        "experiment_4_entailment": None,
    }

    if args.exp in ("3", "both"):
        print("=== Exp 3: persistent multi-cycle incubation (MCQ) ===", file=sys.stderr)
        for c in CYCLES:
            print(f"  incubation_cycles={c} ...", file=sys.stderr)
            payload["experiment_3_multicycle"].append(run_persistent_mcq(c, entailment=False))

    if args.exp in ("4", "both"):
        print("=== Exp 4: MCQ line-level entailment (c1 persistent) ===", file=sys.stderr)
        payload["experiment_4_entailment"] = {
            "baseline_c1": run_persistent_mcq(1, entailment=False),
            "entailment_c1": run_persistent_mcq(1, entailment=True),
        }

    tag = payload["meta"]["timestamp"]
    out_json = OUT / f"MCQ_EXPERIMENTS_3_4_{tag}.json"
    out_md = OUT / f"MCQ_EXPERIMENTS_3_4_{tag}.md"
    OUT.mkdir(parents=True, exist_ok=True)
    out_json.write_text(json.dumps(payload, indent=2, ensure_ascii=False), encoding="utf-8")

    lines = [
        "# MCQ 实验 3 & 4",
        "",
        f"**Time**: {tag} · **MCQ**: {payload['meta']['mcq_count']}Q · "
        f"baseline persistent c1: **{b_ok}/{b_n} ({b_ok/b_n:.1%})**",
        "",
    ]

    if payload["experiment_3_multicycle"]:
        lines.extend(
            [
                "## 实验 3 — Persistent 多 cycle incubation",
                "",
                "triad · persistent batch · `VALHALLA_MCQ_ENTAILMENT=0` · decode 不变",
                "",
                "| incubation_cycles | Acc | decode path |",
                "|-------------------|-----|-------------|",
            ]
        )
        for r in payload["experiment_3_multicycle"]:
            s = r["summary"]
            lines.append(
                f"| **{r['incubation_cycles']}** | **{s['acc']:.1%}** ({s['correct']}/{s['n']}) | `{r['decode_paths']}` |"
            )
        lines.append("")

    if payload["experiment_4_entailment"]:
        base = payload["experiment_4_entailment"]["baseline_c1"]["summary"]
        ent = payload["experiment_4_entailment"]["entailment_c1"]["summary"]
        delta = (ent["correct"] - base["correct"]) / base["n"] * 100
        lines.extend(
            [
                "## 实验 4 — MCQ line-level entailment scorer",
                "",
                "triad persistent c1 · `VALHALLA_MCQ_ENTAILMENT=1`",
                "",
                "机制：top 语料行 → 选项 keyword/数字对齐 entailment；削弱全局 `mcq_corpus_option_boost`",
                "",
                "| Config | Acc | decode path |",
                "|--------|-----|-------------|",
                f"| forward (c1) | {base['acc']:.1%} ({base['correct']}/{base['n']}) | `{payload['experiment_4_entailment']['baseline_c1']['decode_paths']}` |",
                f"| **entailment (c1)** | **{ent['acc']:.1%}** ({ent['correct']}/{ent['n']}) | `{payload['experiment_4_entailment']['entailment_c1']['decode_paths']}` |",
                "",
                f"**Δ entailment vs forward**: {delta:+.1f} pp",
                "",
            ]
        )

    lines.append(f"JSON: `{out_json.relative_to(REPO)}`")
    out_md.write_text("\n".join(lines), encoding="utf-8")

    if payload["experiment_3_multicycle"]:
        for r in payload["experiment_3_multicycle"]:
            s = r["summary"]
            print(f"Exp3 c{r['incubation_cycles']}: {s['acc']:.1%} ({s['correct']}/{s['n']})")
    if payload["experiment_4_entailment"]:
        b = payload["experiment_4_entailment"]["baseline_c1"]["summary"]
        e = payload["experiment_4_entailment"]["entailment_c1"]["summary"]
        print(f"Exp4 forward: {b['acc']:.1%} ({b['correct']}/{b['n']})")
        print(f"Exp4 entail:  {e['acc']:.1%} ({e['correct']}/{e['n']})")
    print(f"Wrote {out_md}")


if __name__ == "__main__":
    main()
