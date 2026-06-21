#!/usr/bin/env python3
"""
Tile / StemCell — 整题输入测试 (persistent batch + grinder).

整题输入协议:
  - 语料: 每行 grinder → hub/tile/stem ports，一次 incubation
  - 题目: strict_user_text 整题 (题干+选项+指令) → question_ports，一次 QA ingress
  - 不拆 stem/option ingress；decode 仍走 mcq_option

对比 fair benchmark 的 isolated (char ingress, 每题重 ingest 语料).
"""

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
sys.path.insert(0, str(REPO / "tools/valhalla_model_bridge"))
sys.path.insert(0, str(REPO / "tools/valhalla_artifact"))

from artifact import load_artifact  # noqa: E402
from metrics import score_answer, summarize_rows  # noqa: E402
from run_fair_benchmark import load_suite, select_prompts  # noqa: E402
from run_strict_incubation_experiment import strict_user_text  # noqa: E402
from session import native_qa_batch_persistent  # noqa: E402

BODIES = ("tile", "stemcell")
PHASE = "test"


def baseline_for(body: str) -> dict:
    """Fair benchmark isolated c1 arm for this body."""
    arm_id = f"{body}_c1"
    if not BASELINE.exists():
        return {}
    data = json.loads(BASELINE.read_text())
    for r in data["results"]:
        if r.get("arm") == arm_id:
            return {
                "arm": arm_id,
                "session": "isolated",
                "summary": r.get("summary", {}),
                "mcq": r.get("summary", {}).get("by_score_type", {}).get("mcq", {}),
            }
    return {}


def run_whole_question(body: str, corpus: list[str], prompts: list[dict]) -> dict:
    batch_q = [(strict_user_text(p), p.get("score_type", "open"), p["id"]) for p in prompts]
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
                "domain": p.get("domain"),
                "score_type": p.get("score_type"),
                "correct": sc["correct"],
                "extracted": sc.get("extracted"),
                "decode_path": resp.get("decode_path"),
                "tile_count": resp.get("tile_count"),
                "stem_cells": resp.get("stem_cells"),
                "tile_connections": resp.get("tile_connections"),
                "stem_organ_clusters": resp.get("stem_organ_clusters"),
            }
        )
    mcq_rows = [r for r in rows if r["score_type"] == "mcq"]
    return {
        "body": body,
        "protocol": "whole_question_persistent_grinder",
        "ingress": "整题 grinder ports → 一次 incubation + 每题整题 QA ingress",
        "summary": summarize_rows(rows),
        "mcq_summary": summarize_rows(mcq_rows),
        "decode_paths": dict(Counter(r["decode_path"] for r in rows)),
        "mcq_decode_paths": dict(Counter(r["decode_path"] for r in mcq_rows)),
        "structure_sample": {
            "tile_count": rows[0].get("tile_count") if rows else None,
            "stem_cells": rows[0].get("stem_cells") if rows else None,
        },
        "rows": rows,
    }


def main() -> None:
    suite = load_suite()
    prompts = select_prompts(suite, PHASE)
    mcq_n = sum(1 for p in prompts if p.get("score_type") == "mcq")
    corpus = load_artifact(ARTIFACT).corpus

    results = {}
    for body in BODIES:
        print(f"=== {body} whole-question persistent ({len(prompts)}Q, MCQ {mcq_n}) ===", file=sys.stderr)
        results[body] = run_whole_question(body, corpus, prompts)

    tag = datetime.now().strftime("%Y%m%d_%H%M")
    payload = {
        "meta": {
            "timestamp": tag,
            "phase": PHASE,
            "prompt_count": len(prompts),
            "mcq_count": mcq_n,
            "protocol": "whole_question_persistent_grinder",
        },
        "results": results,
        "baselines_isolated": {body: baseline_for(body) for body in BODIES},
    }

    OUT.mkdir(parents=True, exist_ok=True)
    out_json = OUT / f"TILE_STEM_WHOLE_QUESTION_{tag}.json"
    out_md = OUT / f"TILE_STEM_WHOLE_QUESTION_{tag}.md"
    out_json.write_text(json.dumps(payload, indent=2, ensure_ascii=False), encoding="utf-8")

    lines = [
        "# Tile / StemCell 整题输入测试",
        "",
        f"**Time**: {tag} · **Test**: {len(prompts)}Q (MCQ {mcq_n})",
        "",
        "## 协议",
        "",
        "- **Ingress**: Grinder hash embed，整题文本（题干+选项+指令）一次送入 body ports",
        "- **Session**: Persistent batch — 语料 incubation 一次，46+ 题 sequential QA 不重 ingest 语料",
        "- **Decode**: `mcq_option`（整题 query_sig，选项单独 hash 打分）",
        "",
        "## 结果",
        "",
        "### 全量 test",
        "",
        "| Body | 整题 persistent | Fair isolated baseline | Δ overall | tiles | stem |",
        "|------|-----------------|------------------------|-----------|-------|------|",
    ]

    for body in BODIES:
        r = results[body]
        s = r["summary"]
        b = payload["baselines_isolated"].get(body, {})
        bsum = b.get("summary", {})
        bacc = bsum.get("acc")
        bstr = f"{bsum.get('correct', '?')}/{bsum.get('n', '?')} ({bacc:.1%})" if bacc is not None else "—"
        delta = s["correct"] - bsum.get("correct", 0) if bsum else None
        dstr = f"{delta:+d} ({delta/bsum['n']*100:+.1f} pp)" if delta is not None and bsum.get("n") else "—"
        st = r["structure_sample"]
        lines.append(
            f"| **{body}** | **{s['correct']}/{s['n']} ({s['acc']:.1%})** | {bstr} | {dstr} | "
            f"{st.get('tile_count')} | {st.get('stem_cells')} |"
        )

    lines.extend(
        [
            "",
            "### MCQ subset",
            "",
            "| Body | 整题 persistent MCQ | Fair isolated MCQ | Δ MCQ | decode path |",
            "|------|---------------------|-------------------|-------|-------------|",
        ]
    )

    for body in BODIES:
        r = results[body]
        ms = r["mcq_summary"]
        b = payload["baselines_isolated"].get(body, {})
        bm = b.get("mcq", {})
        bacc = bm.get("acc")
        bstr = f"{bm.get('correct', '?')}/{bm.get('total', '?')} ({bacc:.1%})" if bacc is not None else "—"
        delta = ms["correct"] - bm.get("correct", 0) if bm else None
        dstr = f"{delta:+d} ({delta/bm['total']*100:+.1f} pp)" if delta is not None and bm.get("total") else "—"
        lines.append(
            f"| **{body}** | **{ms['correct']}/{ms['n']} ({ms['acc']:.1%})** | {bstr} | {dstr} | `{r['mcq_decode_paths']}` |"
        )

    lines.extend(["", f"JSON: `{out_json.relative_to(REPO)}`", ""])
    out_md.write_text("\n".join(lines), encoding="utf-8")

    print(f"\n{'body':<12} {'all':>12} {'mcq':>12} {'iso_mcq':>12} {'Δmcq':>8}")
    for body in BODIES:
        r = results[body]
        s = r["summary"]
        ms = r["mcq_summary"]
        bm = payload["baselines_isolated"].get(body, {}).get("mcq", {})
        iso = f"{bm.get('correct', '?')}/{bm.get('total', '?')}" if bm else "—"
        delta = ms["correct"] - bm.get("correct", 0) if bm else None
        dstr = f"{delta:+d}" if delta is not None else "—"
        print(f"{body:<12} {s['correct']}/{s['n']:>7} {ms['correct']}/{ms['n']:>7} {iso:>12} {dstr:>8}")
    print(f"Wrote {out_md}")


if __name__ == "__main__":
    main()
