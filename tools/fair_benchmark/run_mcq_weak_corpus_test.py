#!/usr/bin/env python3
"""MCQ weakened corpus boost — all body arms vs fair-benchmark baselines."""

from __future__ import annotations

import json
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
from session import native_qa_batch_persistent, native_qa_isolated  # noqa: E402

ARMS = [
    {"id": "tile_c1", "body": "tile", "session": "isolated", "cycles": 1},
    {"id": "stemcell_c1", "body": "stemcell", "session": "isolated", "cycles": 1},
    {"id": "triad_c1", "body": "triad", "session": "isolated", "cycles": 1},
    {"id": "triad_c1_persistent", "body": "triad", "session": "persistent", "cycles": 1},
    {"id": "tile_c1_persistent", "body": "tile", "session": "persistent", "cycles": 1},
    {"id": "stemcell_c1_persistent", "body": "stemcell", "session": "persistent", "cycles": 1},
]


def mcq_prompts() -> list[dict]:
    return [p for p in select_prompts(load_suite(), "test") if p.get("score_type") == "mcq"]


def baseline_mcq_by_arm() -> dict[str, tuple[int, int]]:
    out: dict[str, tuple[int, int]] = {}
    if not BASELINE.exists():
        return out
    data = json.loads(BASELINE.read_text())
    for r in data["results"]:
        arm = r.get("arm", "")
        sub = [x for x in r.get("rows", []) if x.get("score_type") == "mcq"]
        if sub:
            out[arm] = (sum(x.get("correct", False) for x in sub), len(sub))
    return out


def run_arm(arm: dict, corpus: list[str], prompts: list[dict]) -> dict:
    body = arm["body"]
    session = arm["session"]
    cycles = arm["cycles"]
    rows = []

    if session == "persistent":
        batch_q = [(strict_user_text(p), p.get("score_type", "open"), p["id"]) for p in prompts]
        results = native_qa_batch_persistent(
            body,
            corpus,
            batch_q,
            incubation_cycles=cycles,
            qa_cycles=1,
        )
        by_id = {r.get("id"): r for r in results}
        for p in prompts:
            resp = by_id.get(p["id"], {})
            sc = score_answer(resp.get("answer", ""), p, strict_mcq=True)
            rows.append(
                {
                    "id": p["id"],
                    "correct": sc["correct"],
                    "extracted": sc.get("extracted"),
                    "gold": p.get("reference_answer"),
                    "decode_path": resp.get("decode_path"),
                    "domain": p.get("domain"),
                    "tile_count": resp.get("tile_count"),
                    "stem_cells": resp.get("stem_cells"),
                }
            )
    else:
        for p in prompts:
            resp = native_qa_isolated(
                body, cycles, corpus, strict_user_text(p), p.get("score_type", "open")
            )
            sc = score_answer(resp["answer"], p, strict_mcq=True)
            rows.append(
                {
                    "id": p["id"],
                    "correct": sc["correct"],
                    "extracted": sc.get("extracted"),
                    "gold": p.get("reference_answer"),
                    "decode_path": resp.get("decode_path"),
                    "domain": p.get("domain"),
                    "tile_count": resp.get("tile_count"),
                    "stem_cells": resp.get("stem_cells"),
                }
            )

    return {
        "arm_id": arm["id"],
        "body": body,
        "session": session,
        "cycles": cycles,
        "summary": summarize_rows(rows),
        "decode_paths": dict(Counter(r["decode_path"] for r in rows)),
        "rows": rows,
    }


def main() -> None:
    corpus = load_artifact(ARTIFACT).corpus
    prompts = mcq_prompts()
    baselines = baseline_mcq_by_arm()

    arm_results = []
    for arm in ARMS:
        print(f"=== {arm['id']} ({arm['session']}) ===", file=sys.stderr)
        arm_results.append(run_arm(arm, corpus, prompts))

    tag = datetime.now().strftime("%Y%m%d_%H%M")
    payload = {
        "meta": {
            "timestamp": tag,
            "decode": "mcq_option + weakened corpus boost",
            "boost": {
                "per_hit": 0.12,
                "cap": 0.85,
                "min_word_len": 5,
                "keyword_domain_weight": 0.2,
            },
            "mcq_count": len(prompts),
        },
        "arms": arm_results,
    }

    OUT.mkdir(parents=True, exist_ok=True)
    out_json = OUT / f"MCQ_WEAK_CORPUS_ALL_BODIES_{tag}.json"
    out_md = OUT / f"MCQ_WEAK_CORPUS_ALL_BODIES_{tag}.md"
    out_json.write_text(json.dumps(payload, indent=2, ensure_ascii=False), encoding="utf-8")

    lines = [
        "# MCQ Weak Corpus Boost — All Bodies",
        "",
        f"**Time**: {tag} · **MCQ**: {len(prompts)}Q",
        "",
        "机制：`mcq_corpus_option_boost` 0.35→0.12/词、cap 0.85、min word 5；`keyword_domain_boost` 0.5→0.2",
        "",
        "| Arm | Session | Baseline MCQ | Weak boost | Δ | decode path |",
        "|-----|---------|--------------|------------|---|-------------|",
    ]
    for r in arm_results:
        s = r["summary"]
        b = baselines.get(r["arm_id"])
        if b:
            delta = s["correct"] - b[0]
            bstr = f"{b[0]}/{b[1]} ({b[0]/b[1]:.1%})"
            dstr = f"{delta:+d} ({delta/b[1]*100:+.1f} pp)"
        else:
            bstr = "—"
            dstr = "—"
        dp = r["decode_paths"]
        lines.append(
            f"| **{r['arm_id']}** | {r['session']} | {bstr} | "
            f"**{s['correct']}/{s['n']} ({s['acc']:.1%})** | {dstr} | `{dp}` |"
        )
    lines.extend(["", f"JSON: `{out_json.relative_to(REPO)}`", ""])
    out_md.write_text("\n".join(lines), encoding="utf-8")

    print(f"\n{'arm':<26} {'session':<12} {'baseline':>10} {'weak':>10} {'delta':>8}")
    for r in arm_results:
        s = r["summary"]
        b = baselines.get(r["arm_id"])
        bstr = f"{b[0]}/{b[1]}" if b else "—"
        delta = s["correct"] - b[0] if b else None
        dstr = f"{delta:+d}" if delta is not None else "—"
        print(
            f"{r['arm_id']:<26} {r['session']:<12} {bstr:>10} "
            f"{s['correct']}/{s['n']:>7} {dstr:>8}"
        )
    print(f"Wrote {out_md}")


if __name__ == "__main__":
    main()
