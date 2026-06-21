#!/usr/bin/env python3
"""MCQ stem-only + (stem, option) joint compare decode — vs baseline 21/46."""

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
from session import native_qa_batch_persistent  # noqa: E402


def mcq_prompts() -> list[dict]:
    return [p for p in select_prompts(load_suite(), "test") if p.get("score_type") == "mcq"]


def baseline_mcq() -> tuple[int, int, dict[str, bool]]:
    if not BASELINE.exists():
        return 21, 46, {}
    data = json.loads(BASELINE.read_text())
    for r in data["results"]:
        if r.get("arm") == "triad_c1_persistent":
            sub = [x for x in r["rows"] if x.get("score_type") == "mcq"]
            by_id = {x["id"]: x.get("correct", False) for x in sub}
            return sum(by_id.values()), len(sub), by_id
    return 21, 46, {}


def main() -> None:
    corpus = load_artifact(ARTIFACT).corpus
    prompts = mcq_prompts()
    batch_q = [(strict_user_text(p), p.get("score_type", "open"), p["id"]) for p in prompts]
    results = native_qa_batch_persistent(
        "triad",
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
                "pred": sc.get("pred"),
                "gold": p.get("reference_answer"),
                "decode_path": resp.get("decode_path"),
                "domain": p.get("domain"),
            }
        )

    b_ok, b_n, b_by_id = baseline_mcq()
    summary = summarize_rows(rows)
    flips = []
    for r in rows:
        pid = r["id"]
        if pid in b_by_id and b_by_id[pid] != r["correct"]:
            flips.append(
                {
                    "id": pid,
                    "baseline": b_by_id[pid],
                    "now": r["correct"],
                    "pred": r["pred"],
                    "gold": r["gold"],
                }
            )

    tag = datetime.now().strftime("%Y%m%d_%H%M")
    payload = {
        "meta": {
            "timestamp": tag,
            "decode": "mcq_compare (stem-only query + joint stem+option)",
            "baseline_persistent_c1": {"correct": b_ok, "total": b_n},
            "delta_correct": summary["correct"] - b_ok,
        },
        "summary": summary,
        "decode_paths": dict(Counter(r["decode_path"] for r in rows)),
        "flips_vs_baseline": flips,
        "rows": rows,
    }

    OUT.mkdir(parents=True, exist_ok=True)
    out_json = OUT / f"MCQ_COMPARE_{tag}.json"
    out_md = OUT / f"MCQ_COMPARE_{tag}.md"
    out_json.write_text(json.dumps(payload, indent=2, ensure_ascii=False), encoding="utf-8")

    delta = summary["correct"] - b_ok
    lines = [
        "# MCQ Compare Decode Test",
        "",
        f"**Time**: {tag} · **MCQ**: {summary['n']}Q",
        "",
        "机制：ingress 整题不变；decode 用 stem-only query + `(stem, option)` joint embed 比较打分",
        "",
        f"| Arm | Acc |",
        f"|-----|-----|",
        f"| baseline persistent c1 | {b_ok}/{b_n} ({b_ok/b_n:.1%}) |",
        f"| **mcq_compare** | **{summary['correct']}/{summary['n']} ({summary['acc']:.1%})** |",
        "",
        f"**Δ vs baseline**: {delta:+d} ({delta/b_n*100:+.1f} pp)",
        "",
        f"decode paths: `{payload['decode_paths']}`",
        "",
    ]
    if flips:
        lines.extend(["## Flips vs baseline", ""])
        for f in flips:
            arrow = "✓→✗" if f["baseline"] and not f["now"] else "✗→✓"
            lines.append(
                f"- `{f['id']}` {arrow} pred={f['pred']} gold={f['gold']}"
            )
        lines.append("")

    lines.append(f"JSON: `{out_json.relative_to(REPO)}`")
    out_md.write_text("\n".join(lines), encoding="utf-8")

    print(f"baseline: {b_ok}/{b_n} ({b_ok/b_n:.1%})")
    print(f"compare:  {summary['correct']}/{summary['n']} ({summary['acc']:.1%})")
    print(f"delta:    {delta:+d} ({delta/b_n*100:+.1f} pp)")
    print(f"paths:    {payload['decode_paths']}")
    if flips:
        print(f"flips:    {len(flips)}")
        for f in flips:
            print(f"  {f['id']}: {f['baseline']} -> {f['now']} pred={f['pred']} gold={f['gold']}")
    print(f"Wrote {out_md}")


if __name__ == "__main__":
    main()
