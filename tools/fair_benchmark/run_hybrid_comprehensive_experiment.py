#!/usr/bin/env python3
"""
Hybrid Valhalla LM — comprehensive experiment (all comparison arms + record).

Arms:
  1. memory_tier_b persistent (triad_c1_persistent)
  2. patch_lm_full — Valhalla patch on ALL prompts (Tier A baseline)
  3. patch_lm_routed — patch on open+numeric subset only (hybrid cost model)
  4. hybrid_product — memory + routed patch LM
  5. transformer_zero_shot — unpatched Qwen (mature baseline)

Usage:
  .venv/bin/python tools/fair_benchmark/run_hybrid_comprehensive_experiment.py --phase test
  .venv/bin/python tools/fair_benchmark/run_hybrid_comprehensive_experiment.py --phase test --skip-transformer
"""

from __future__ import annotations

import argparse
import json
import os
import platform
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
FAIR = Path(__file__).resolve().parent
OUT_DIR = REPO / "reports/hybrid_valhalla_lm"
EXP_DIR = REPO / "reports/experiments"
NATIVE_BIN = REPO / "target/release/valhalla_native_qa"
EXPORT_BIN = REPO / "target/release/valhalla_model_export"

sys.path.insert(0, str(REPO / "tools/valhalla_model_bridge"))
sys.path.insert(0, str(REPO / "tools/valhalla_artifact"))
sys.path.insert(0, str(FAIR))

from artifact import load_artifact  # noqa: E402
from hybrid_valhalla_lm import MEMORY_TRIAD, build_hybrid_rows, hybrid_summary, patch_prompt_ids  # noqa: E402
from metrics import bootstrap_ci_paired, summarize_rows  # noqa: E402
from run_fair_benchmark import load_suite, run_memory_arm, select_corpus, select_prompts  # noqa: E402
from run_potential_analysis import classify_slices  # noqa: E402


def git_head() -> str:
    try:
        return (
            subprocess.run(
                ["git", "rev-parse", "--short", "HEAD"],
                capture_output=True,
                text=True,
                cwd=REPO,
                check=True,
            ).stdout.strip()
        )
    except Exception:
        return "unknown"


def write_experiment_record(payload: dict, out_md: Path) -> None:
    meta = payload["meta"]
    arms = payload["arms"]
    slices = payload["slices"]
    tag = meta["timestamp"]

    def fmt_arm(key: str) -> str:
        a = arms[key]["summary"]
        st = a.get("by_score_type", {})
        return (
            f"| {arms[key]['label']} | {a['acc']:.2%} | "
            f"{st.get('numeric', {}).get('acc', 0):.2%} | "
            f"{st.get('mcq', {}).get('acc', 0):.2%} | "
            f"{st.get('open', {}).get('acc', 0):.2%} |"
        )

    hy = arms["hybrid"]["summary"]
    mem = arms["memory"]["summary"]
    ci = payload.get("paired_ci", {})

    lines = [
        "# Hybrid Valhalla LM — 全面实验记录",
        "",
        f"**实验 ID**: `hybrid_valhalla_lm_comprehensive_{tag}`",
        f"**日期**: {meta['date_iso']} · **Git**: `{meta['git_head']}`",
        f"**Phase**: {meta['phase']} · **题量**: {meta['prompt_count']} · **语料**: {meta['corpus_lines']} 行 (ingest_clean)",
        "",
        "---",
        "",
        "## 1. 实验目的",
        "",
        "验证 **Valhalla 结构 patch 后的 LM（Tier A）+ Tier B memory 混合编排** 能否解决 open QA 无生成能力问题，",
        "并与纯 memory、纯 patch LM、零样本 Transformer 对照。",
        "",
        "---",
        "",
        "## 2. 协议",
        "",
        "| 项 | 值 |",
        "|----|-----|",
        f"| 模型 | `{meta['model']}` |",
        f"| Patch body | `{meta['patch_body']}` · strength `{meta['patch_strength']}` · mode `legacy` |",
        f"| Memory arm | `{meta['memory_arm']}` · persistent session |",
        f"| Export | `valhalla_model_export` → `apply_valhalla_patch` |",
        f"| Generate | fresh HF reload **per question** (fair protocol) |",
        f"| Artifact | `{meta.get('artifact_id') or 'suite corpus'}` |",
        f"| 环境 | {meta['platform']} |",
        "",
        "**Hybrid 路由**（`hybrid_valhalla_lm.hybrid_pick_arm`）：",
        "",
        "- `open` / `open_generation` → Valhalla patch LM",
        "- `mcq` + corpus_supported → memory",
        "- `numeric` → max(memory, patch_lm)",
        "",
        f"Patch routed 子集: **{meta['patch_routed_count']}** / {meta['prompt_count']} 题",
        "",
        "---",
        "",
        "## 3. 主结果",
        "",
        "| Arm | Acc | numeric | mcq | open |",
        "|-----|-----|---------|-----|------|",
        fmt_arm("memory"),
        fmt_arm("patch_routed"),
        fmt_arm("hybrid"),
    ]
    if "patch_full" in arms:
        lines.insert(-2, fmt_arm("patch_full"))
    if "transformer" in arms:
        lines.append(fmt_arm("transformer"))
    pf = arms.get("patch_full", arms["patch_routed"])
    pf_key = "patch_full" if "patch_full" in arms else "patch_routed"
    lines.extend(
        [
            "",
            "### 3.1 Open 切片",
            "",
            "| 切片 | memory | patch_full | hybrid |",
            "|------|--------|------------|--------|",
            f"| open 全体 ({slices['open_total']}Q) | "
            f"{arms['memory']['open_slice']['acc']:.2%} | "
            f"{pf['open_slice']['acc']:.2%} | "
            f"{arms['hybrid']['open_slice']['acc']:.2%} |",
            f"| open_retrieval ({len(slices['open_retrieval'])}Q) | "
            f"{slice_acc_from_rows(arms['memory']['rows'], slices['open_retrieval']):.2%} | "
            f"{slice_acc_from_rows(pf['rows'], slices['open_retrieval']):.2%} | "
            f"{slice_acc_from_rows(arms['hybrid']['rows'], slices['open_retrieval']):.2%} |",
            f"| open_generation ({len(slices['open_generation'])}Q) | "
            f"{slice_acc_from_rows(arms['memory']['rows'], slices['open_generation']):.2%} | "
            f"{slice_acc_from_rows(pf['rows'], slices['open_generation']):.2%} | "
            f"{slice_acc_from_rows(arms['hybrid']['rows'], slices['open_generation']):.2%} |",
            "",
            "### 3.2 Hybrid vs 对照",
            "",
            f"- Hybrid − memory: **{payload['deltas']['hybrid_minus_memory_pp']:+.2f} pp**",
        ]
    )
    if payload["deltas"].get("hybrid_minus_patch_full_pp") is not None:
        lines.append(
            f"- Hybrid − patch_full: **{payload['deltas']['hybrid_minus_patch_full_pp']:+.2f} pp**"
        )
    if "transformer" in arms:
        lines.append(
            f"- Hybrid − TF zero-shot: **{payload['deltas']['hybrid_minus_transformer_pp']:+.2f} pp**"
        )
    if ci:
        lines.extend(
            [
                "",
                f"- Hybrid vs memory 95% CI: **[{ci['ci95_lo']:+.2f}, {ci['ci95_hi']:+.2f}] pp**",
            ]
        )
    src = payload.get("hybrid_source_counts", {})
    lines.extend(
        [
            "",
            f"Hybrid source mix: `{src}`",
            "",
            "---",
            "",
            "## 4. 结论",
            "",
            payload["conclusion"],
            "",
            "---",
            "",
            "## 5. 产物",
            "",
            f"- JSON: `reports/hybrid_valhalla_lm/hybrid_comprehensive_{meta['phase']}_{tag}.json`",
            f"- 本记录: `{out_md.relative_to(REPO)}`",
            "",
            "## 6. 复现",
            "",
            "```bash",
            "cd /home/jouly/AI/Valhalla",
            "HF_ENDPOINT=https://hf-mirror.com .venv/bin/python3 \\",
            "  tools/fair_benchmark/run_hybrid_comprehensive_experiment.py \\",
            f"  --phase {meta['phase']} \\",
            "  --artifact checkpoints/incubation_artifacts/fair_triad_c1/",
            "```",
            "",
        ]
    )
    out_md.write_text("\n".join(lines) + "\n", encoding="utf-8")


def slice_acc_from_rows(rows: list[dict], ids: set[str]) -> float:
    sub = [r for r in rows if r["id"] in ids]
    if not sub:
        return 0.0
    return sum(1 for r in sub if r["correct"]) / len(sub)


def build_conclusion(payload: dict) -> str:
    d = payload["deltas"]
    hy = payload["arms"]["hybrid"]["summary"]["acc"]
    mem_open = payload["arms"]["memory"]["open_slice"]["acc"]
    hy_open = payload["arms"]["hybrid"]["open_slice"]["acc"]
    parts = [
        f"1. **Open 从 memory {mem_open:.1%} → hybrid {hy_open:.1%}**：Valhalla patch LM 补齐无生成短板。",
        f"2. **Hybrid 总 acc {hy:.1%}**，相对纯 memory {d['hybrid_minus_memory_pp']:+.1f} pp。",
    ]
    if "transformer" in payload["arms"]:
        tf = payload["arms"]["transformer"]["summary"]["acc"]
        parts.append(
            f"3. Hybrid {hy:.1%} vs TF zero-shot {tf:.1%}（{d['hybrid_minus_transformer_pp']:+.1f} pp）— "
            + ("混合路径有效。" if d["hybrid_minus_transformer_pp"] >= 0 else "仍低于 TF，MCQ 楔子待加强。")
        )
    parts.append(
        "4. **产品建议**：open 走 Valhalla patch LM；MCQ/numeric 保留 Tier B memory 楔子。"
    )
    return "\n".join(parts)


def main() -> None:
    ap = argparse.ArgumentParser(description="Comprehensive Hybrid Valhalla LM experiment")
    ap.add_argument("--phase", choices=["smoke", "val", "test"], default="test")
    ap.add_argument("--artifact", type=Path, default=REPO / "checkpoints/incubation_artifacts/fair_triad_c1")
    ap.add_argument("--memory-arm", default="triad_c1_persistent")
    ap.add_argument("--patch-body", default="triad")
    ap.add_argument("--patch-strength", type=float, default=0.08)
    ap.add_argument("--model", default="Qwen/Qwen2.5-0.5B-Instruct")
    ap.add_argument("--max-new", type=int, default=192)
    ap.add_argument("--skip-transformer", action="store_true")
    ap.add_argument("--skip-patch-full", action="store_true", help="Skip full 145Q patch (saves ~15min)")
    args = ap.parse_args()

    if not NATIVE_BIN.exists() or not EXPORT_BIN.exists():
        sys.exit("Build valhalla_native_qa and valhalla_model_export first")

    os.environ.setdefault("HF_ENDPOINT", "https://hf-mirror.com")
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    EXP_DIR.mkdir(parents=True, exist_ok=True)

    suite = load_suite()
    bundle = load_artifact(args.artifact) if args.artifact.exists() else None
    corpus = bundle.corpus if bundle else select_corpus(suite, args.phase)
    export_doc = bundle.export_doc if bundle and bundle.body == args.patch_body else None

    prompts = select_prompts(suite, args.phase)
    slices_cls = classify_slices(prompts, corpus)
    patch_ids = patch_prompt_ids(prompts, slices_cls)
    patch_prompts = [p for p in prompts if p["id"] in patch_ids]
    open_ids = {p["id"] for p in prompts if p.get("score_type") == "open"}

    print(f"=== comprehensive {args.phase}: {len(prompts)}Q, routed patch {len(patch_prompts)}Q ===", file=sys.stderr)

    arm_cfg = next(a for a in suite["arms"]["memory_tier_b"] if a["id"] == args.memory_arm)
    print("=== [1/5] memory persistent ===", file=sys.stderr)
    mem_result = run_memory_arm(arm_cfg, prompts, corpus)
    memory_rows = mem_result["rows"]

    from patch_track import eval_patch_arm_fresh, eval_transformer_fresh

    patch_full_rows: list[dict] = []
    if not args.skip_patch_full:
        print(f"=== [2/5] patch LM full ({len(prompts)}Q) ===", file=sys.stderr)
        patch_full_result = eval_patch_arm_fresh(
            "patch_triad_full",
            args.patch_body,
            corpus,
            prompts,
            model_name=args.model,
            strength=args.patch_strength,
            export_doc=export_doc,
        )
        patch_full_rows = patch_full_result["rows"]
    else:
        print("=== [2/5] patch full SKIPPED ===", file=sys.stderr)

    print(f"=== [3/5] patch LM routed ({len(patch_prompts)}Q) ===", file=sys.stderr)
    patch_routed_result = eval_patch_arm_fresh(
        "patch_triad_routed",
        args.patch_body,
        corpus,
        patch_prompts,
        model_name=args.model,
        strength=args.patch_strength,
        export_doc=export_doc,
    )
    patch_routed_rows = patch_routed_result["rows"]

    print("=== [4/5] hybrid merge ===", file=sys.stderr)
    mem_src = {MEMORY_TRIAD: memory_rows}
    hybrid_rows, _ = build_hybrid_rows(prompts, mem_src, patch_routed_rows, corpus)
    summary = hybrid_summary(hybrid_rows, mem_src, patch_routed_rows, slices_cls)

    transformer_result = None
    if not args.skip_transformer:
        print(f"=== [5/5] transformer zero-shot ({len(prompts)}Q) ===", file=sys.stderr)
        transformer_result = eval_transformer_fresh(prompts, model_name=args.model, max_new=args.max_new)

    tag = datetime.now().strftime("%Y%m%d_%H%M")
    date_iso = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M UTC")

    def arm_block(label: str, rows: list[dict], key: str) -> dict:
        open_slice = summarize_rows([r for r in rows if r["id"] in open_ids])
        return {"label": label, "key": key, "summary": summarize_rows(rows), "open_slice": open_slice, "rows": rows}

    arms = {
        "memory": arm_block("memory persistent", memory_rows, "memory"),
        "patch_routed": arm_block("patch LM (routed subset)", patch_routed_rows, "patch_routed"),
        "hybrid": arm_block("**hybrid product**", hybrid_rows, "hybrid"),
    }
    if patch_full_rows:
        arms["patch_full"] = arm_block("patch LM (full 145Q)", patch_full_rows, "patch_full")
    if transformer_result:
        arms["transformer"] = arm_block(
            "TF zero-shot", transformer_result["rows"], "transformer"
        )

    hy_acc = arms["hybrid"]["summary"]["acc"]
    mem_acc = arms["memory"]["summary"]["acc"]
    deltas = {
        "hybrid_minus_memory_pp": round((hy_acc - mem_acc) * 100, 2),
        "hybrid_minus_patch_full_pp": round(
            (hy_acc - arms["patch_full"]["summary"]["acc"]) * 100, 2
        )
        if patch_full_rows
        else None,
    }
    if transformer_result:
        tf_acc = arms["transformer"]["summary"]["acc"]
        deltas["hybrid_minus_transformer_pp"] = round((hy_acc - tf_acc) * 100, 2)

    paired_ci = None
    if args.phase == "test":
        before = [r["correct"] for r in memory_rows]
        after = [r["correct"] for r in hybrid_rows]
        paired_ci = bootstrap_ci_paired(before, after)

    slice_info = {
        "open_retrieval": slices_cls["open_retrieval"],
        "open_generation": slices_cls["open_generation"],
        "corpus_supported": slices_cls["corpus_supported"],
        "open_total": len(open_ids),
    }

    payload = {
        "meta": {
            "experiment_id": f"hybrid_valhalla_lm_comprehensive_{tag}",
            "benchmark": "hybrid_valhalla_lm_comprehensive",
            "phase": args.phase,
            "prompt_count": len(prompts),
            "corpus_lines": len(corpus),
            "patch_routed_count": len(patch_prompts),
            "timestamp": tag,
            "date_iso": date_iso,
            "git_head": git_head(),
            "platform": platform.platform(),
            "model": args.model,
            "patch_body": args.patch_body,
            "patch_strength": args.patch_strength,
            "memory_arm": args.memory_arm,
            "artifact_id": bundle.manifest["artifact_id"] if bundle else None,
        },
        "slices": {k: (list(v) if isinstance(v, set) else v) for k, v in slice_info.items()},
        "arms": {k: {kk: vv for kk, vv in v.items() if kk != "rows"} for k, v in arms.items()},
        "hybrid_source_counts": summary["hybrid_source_counts"],
        "deltas": deltas,
        "paired_ci": paired_ci,
        "rows": {k: v["rows"] for k, v in arms.items()},
    }
    payload["conclusion"] = build_conclusion({**payload, "arms": arms})

    out_json = OUT_DIR / f"hybrid_comprehensive_{args.phase}_{tag}.json"
    # slim JSON without all rows for git; full rows in separate file
    slim = {k: v for k, v in payload.items() if k != "rows"}
    slim["row_counts"] = {k: len(v) for k, v in payload["rows"].items()}
    out_json.write_text(json.dumps({**slim, "rows": payload["rows"]}, indent=2, ensure_ascii=False), encoding="utf-8")

    out_md = EXP_DIR / f"HYBRID_VALHALLA_LM_COMPREHENSIVE_{args.phase}_{tag}.md"
    write_experiment_record({**payload, "arms": arms, "paired_ci": paired_ci}, out_md)

    also = REPO / "reports" / f"VALHALLA_HYBRID_LM_COMPREHENSIVE_{args.phase}_{tag}.md"
    also.write_text(out_md.read_text(encoding="utf-8"), encoding="utf-8")

    print(f"\n=== Comprehensive results ({args.phase}) ===", file=sys.stderr)
    for key in ("memory", "patch_full", "patch_routed", "hybrid", "transformer"):
        if key in arms:
            print(f"{arms[key]['label']:30s} {arms[key]['summary']['acc']:.2%}", file=sys.stderr)
    print(f"Wrote {out_json}", file=sys.stderr)
    print(f"Wrote {out_md}", file=sys.stderr)


if __name__ == "__main__":
    main()
