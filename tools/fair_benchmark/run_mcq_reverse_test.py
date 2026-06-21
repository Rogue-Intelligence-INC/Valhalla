#!/usr/bin/env python3
"""Quick MCQ reverse + hybrid routing test vs fair baseline."""

from __future__ import annotations

import json
import sys
from datetime import datetime
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
FAIR = Path(__file__).resolve().parent
sys.path.insert(0, str(FAIR))
sys.path.insert(0, str(REPO / "tools/valhalla_artifact"))

from artifact import load_artifact
from hybrid_valhalla_lm import build_hybrid_rows, hybrid_summary, patch_prompt_ids
from metrics import summarize_rows
from run_fair_benchmark import load_suite, run_memory_arm, select_prompts

BASELINE = REPO / "reports/fair_benchmark/fair_benchmark_test_20260620_0859.json"
ARTIFACT = REPO / "checkpoints/incubation_artifacts/fair_triad_c1"
OUT = REPO / "reports/experiments"


def main() -> None:
    suite = load_suite()
    prompts = select_prompts(suite, "test")
    bundle = load_artifact(ARTIFACT)
    corpus = bundle.corpus

    arm_cfg = next(a for a in suite["arms"]["memory_tier_b"] if a["id"] == "triad_c1_persistent")
    print("=== memory (mcq_reverse decode) ===", file=sys.stderr)
    mem = run_memory_arm(arm_cfg, prompts, corpus)
    memory_rows = mem["rows"]

    baseline = json.loads(BASELINE.read_text())
    b_mem = next(r for r in baseline["results"] if r["arm"] == "triad_c1_persistent")
    b_patch = next(r for r in baseline["results"] if r["arm"] == "patch_triad_legacy")
    b_tf = next(r for r in baseline["results"] if r["arm"] == "transformer_zero_shot")

    def mcq_acc(rows):
        sub = [r for r in rows if r.get("score_type") == "mcq"]
        return sum(r["correct"] for r in sub), len(sub)

    def paths(rows):
        from collections import Counter
        sub = [r for r in rows if r.get("score_type") == "mcq"]
        return dict(Counter(r.get("decode_path") for r in sub))

    m_new = mcq_acc(memory_rows)
    m_old = mcq_acc(b_mem["rows"])
    p_old = mcq_acc(b_patch["rows"])
    t_old = mcq_acc(b_tf["rows"])

    # hybrid with baseline patch rows (routing change only for patch subset)
    patch_ids = patch_prompt_ids(prompts, __import__("run_potential_analysis", fromlist=["classify_slices"]).classify_slices(prompts, corpus))
    patch_rows = [r for r in b_patch["rows"] if r["id"] in patch_ids]
    hybrid_rows, slices = build_hybrid_rows(prompts, memory_rows, patch_rows, corpus)
    summary = hybrid_summary(hybrid_rows, memory_rows, patch_rows, slices)

    h_mcq = mcq_acc(hybrid_rows)
    h_all = summarize_rows(hybrid_rows)

    tag = datetime.now().strftime("%Y%m%d_%H%M")
    report = {
        "meta": {"timestamp": tag, "phase": "test", "mcq_reverse": True},
        "mcq": {
            "memory_before": {"correct": m_old[0], "total": m_old[1]},
            "memory_after_reverse": {"correct": m_new[0], "total": m_new[1]},
            "patch_baseline": {"correct": p_old[0], "total": p_old[1]},
            "tf_baseline": {"correct": t_old[0], "total": t_old[1]},
            "hybrid_max_routing": {"correct": h_mcq[0], "total": h_mcq[1]},
            "decode_paths_after": paths(memory_rows),
        },
        "hybrid_total_acc": h_all["acc"],
        "hybrid_by_type": h_all.get("by_score_type"),
        "hybrid_source_counts": summary["hybrid_source_counts"],
    }

    out_json = OUT / f"MCQ_REVERSE_TEST_{tag}.json"
    out_md = OUT / f"MCQ_REVERSE_EXPERIMENT_{tag}.md"
    OUT.mkdir(parents=True, exist_ok=True)
    out_json.write_text(json.dumps(report, indent=2, ensure_ascii=False), encoding="utf-8")

    delta = (m_new[0] - m_old[0]) / m_old[1] * 100 if m_old[1] else 0
    hy_delta = (h_mcq[0] - m_old[0]) / m_old[1] * 100 if m_old[1] else 0
    lines = [
        "# MCQ 逆向处理实验记录",
        "",
        f"**Time**: {tag} · **Phase**: test 145Q",
        "",
        "## 变更",
        "",
        "1. **Tier B `decode_mcq`**: evidence-first 逆向（`mcq_reverse`）→ fallback `mcq_option`",
        "2. **Hybrid 路由**: `mcq` → `max(memory, patch_lm)`（备份于 `backups/pre-mcq-reverse-20260621/`）",
        "",
        "## MCQ 结果 (46Q)",
        "",
        "| Arm | Before | After |",
        "|-----|--------|-------|",
        f"| memory | {m_old[0]}/{m_old[1]} ({m_old[0]/m_old[1]:.1%}) | **{m_new[0]}/{m_new[1]} ({m_new[0]/m_new[1]:.1%})** |",
        f"| patch (baseline) | {p_old[0]}/{p_old[1]} | — |",
        f"| TF (baseline) | {t_old[0]}/{t_old[1]} | — |",
        f"| **hybrid max** | — | **{h_mcq[0]}/{h_mcq[1]} ({h_mcq[0]/h_mcq[1]:.1%})** |",
        "",
        f"- memory Δ: **{delta:+.1f} pp**",
        f"- hybrid vs old memory on MCQ: **{hy_delta:+.1f} pp**",
        f"- decode paths: `{report['mcq']['decode_paths_after']}`",
        f"- hybrid total acc: **{h_all['acc']:.2%}**",
        f"- hybrid sources: `{summary['hybrid_source_counts']}`",
        "",
        f"JSON: `{out_json.relative_to(REPO)}`",
        "",
    ]
    out_md.write_text("\n".join(lines), encoding="utf-8")

    print(f"MCQ memory: {m_old[0]}/{m_old[1]} -> {m_new[0]}/{m_new[1]} ({delta:+.1f}pp)")
    print(f"MCQ hybrid max: {h_mcq[0]}/{h_mcq[1]}")
    print(f"Hybrid total: {h_all['acc']:.2%}")
    print(f"Wrote {out_md}")


if __name__ == "__main__":
    main()
