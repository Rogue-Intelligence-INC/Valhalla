#!/usr/bin/env python3
"""
Hybrid Valhalla — unified product runner (all question types).

Runs:
  - Triad persistent memory (whole-question batch)
  - Tile persistent + MCQ Fate cycles (best MCQ)
  - Valhalla-patched Qwen (open + numeric candidates)
  → merges via hybrid_valhalla_lm.build_hybrid_rows

Usage:
  python run_hybrid_valhalla_lm.py --phase test
  python run_hybrid_valhalla_lm.py --phase test --reuse-json reports/fair_benchmark/fair_benchmark_test_20260620_0859.json --skip-patch
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from datetime import datetime
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
FAIR = Path(__file__).resolve().parent
OUT_DIR = REPO / "reports/hybrid_valhalla_lm"
NATIVE_BIN = REPO / "target/release/valhalla_native_qa"
EXPORT_BIN = REPO / "target/release/valhalla_model_export"
ARTIFACT = REPO / "checkpoints/incubation_artifacts/fair_triad_c1"

sys.path.insert(0, str(REPO / "tools/valhalla_model_bridge"))
sys.path.insert(0, str(REPO / "tools/valhalla_artifact"))
sys.path.insert(0, str(FAIR))

from artifact import load_artifact  # noqa: E402
from hybrid_valhalla_lm import (  # noqa: E402
    FAIR_BENCHMARK_VERSION,
    HYBRID_POLICY_DOC,
    HYBRID_PRODUCT_VERSION,
    MEMORY_TILE_FATE,
    MEMORY_TRIAD,
    build_hybrid_rows,
    hybrid_summary,
    patch_prompt_ids,
)
from metrics import score_answer, summarize_rows  # noqa: E402
from run_fair_benchmark import load_suite, select_prompts  # noqa: E402
from run_potential_analysis import classify_slices  # noqa: E402
from run_strict_incubation_experiment import strict_user_text  # noqa: E402
from session import native_qa_batch_persistent  # noqa: E402


def run_persistent_memory(
    body: str,
    corpus: list[str],
    prompts: list[dict],
    *,
    mcq_fate: bool = False,
) -> list[dict]:
    prev = os.environ.get("VALHALLA_MCQ_FATE_CYCLES")
    if mcq_fate:
        os.environ["VALHALLA_MCQ_FATE_CYCLES"] = "1"
        os.environ.setdefault("VALHALLA_MCQ_FATE_MAX_CYCLES", "64")
        os.environ.setdefault("VALHALLA_MCQ_FATE_HARD_MAX", "128")
    else:
        os.environ["VALHALLA_MCQ_FATE_CYCLES"] = "0"
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
                    "domain": p.get("domain"),
                    "score_type": p.get("score_type"),
                    "correct": sc["correct"],
                    "score": sc,
                    "native_answer": resp.get("answer"),
                    "decode_path": resp.get("decode_path"),
                    "session": "persistent",
                    "memory_body": body,
                    "mcq_fate_cycles": resp.get("mcq_fate_cycles", 0),
                    "structure": {
                        "tile_count": resp.get("tile_count"),
                        "stem_cells": resp.get("stem_cells"),
                    },
                }
            )
        return rows
    finally:
        if prev is None:
            os.environ.pop("VALHALLA_MCQ_FATE_CYCLES", None)
        else:
            os.environ["VALHALLA_MCQ_FATE_CYCLES"] = prev


def load_patch_from_benchmark(json_path: Path, patch_ids: set[str], arm: str = "patch_triad_legacy") -> list[dict]:
    data = json.loads(json_path.read_text(encoding="utf-8"))
    for r in data["results"]:
        if r.get("arm") == arm:
            return [row for row in r["rows"] if row["id"] in patch_ids]
    raise KeyError(f"{arm} not in {json_path}")


def write_report(payload: dict, out_md: Path) -> None:
    meta = payload["meta"]
    s = payload["summary"]
    lines = [
        "# Hybrid Valhalla — Unified Product Report",
        "",
        f"**Phase**: {meta['phase']} · **Prompts**: {meta['prompt_count']} · **Time**: {meta['timestamp']}",
        f"**Hybrid**: {meta.get('hybrid_product_version', HYBRID_PRODUCT_VERSION)} · **Fair**: {meta.get('fair_benchmark_version', FAIR_BENCHMARK_VERSION)} · **hub-f64**: {meta.get('hub_f64_version', '1.1.0')}",
        f"**Corpus**: {meta['corpus_lines']} lines · **Patch body**: {meta['patch_body']} · **strength**: {meta['patch_strength']}",
        "",
        "## Summary (all types via Hybrid)",
        "",
        "| Track | Acc | numeric | mcq | open |",
        "|-------|-----|---------|-----|------|",
    ]

    def row_line(label: str, block: dict) -> None:
        st = block.get("by_score_type", {})
        lines.append(
            f"| {label} | {block['acc']:.2%} | "
            f"{st.get('numeric', {}).get('acc', 0):.2%} | "
            f"{st.get('mcq', {}).get('acc', 0):.2%} | "
            f"{st.get('open', {}).get('acc', 0):.2%} |"
        )

    row_line("**Hybrid product (unified)**", s["hybrid"])
    row_line("Triad memory only", s["memory_triad_persistent"])
    row_line("Tile+Fate memory only", s["memory_tile_fate_persistent"])
    row_line("patch LM (subset)", s["patch_lm_subset"])
    lines.extend(
        [
            "",
            "## By type",
            "",
            f"- **open** hybrid: **{s['open_only']['hybrid']['acc']:.2%}** (patch LM: {s['open_only']['patch_lm']['acc']:.2%})",
            f"- **mcq** hybrid: **{s['mcq_only']['hybrid']['acc']:.2%}** (tile+fate: {s['mcq_only']['tile_fate_memory']['acc']:.2%}, triad: {s['mcq_only']['triad_memory']['acc']:.2%})",
            f"- **numeric** hybrid: **{s['numeric_only']['hybrid']['acc']:.2%}**",
            "",
            f"Hybrid source mix: `{s['hybrid_source_counts']}`",
            "",
            "## Unified policy",
            "",
            HYBRID_POLICY_DOC,
            "",
        ]
    )
    out_md.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> None:
    ap = argparse.ArgumentParser(description="Hybrid Valhalla unified product")
    ap.add_argument("--phase", choices=["smoke", "val", "test"], default="test")
    ap.add_argument("--artifact", type=Path, default=ARTIFACT)
    ap.add_argument("--reuse-json", type=Path, default=None, help="Reuse patch rows from fair benchmark JSON")
    ap.add_argument("--patch-body", default="triad")
    ap.add_argument("--patch-strength", type=float, default=0.08)
    ap.add_argument("--model", default="Qwen/Qwen2.5-0.5B-Instruct")
    ap.add_argument("--max-new", type=int, default=192)
    ap.add_argument("--skip-patch", action="store_true")
    ap.add_argument("--skip-tile-fate", action="store_true", help="Use triad only for memory (debug)")
    args = ap.parse_args()

    if not NATIVE_BIN.exists():
        sys.exit(f"Missing {NATIVE_BIN}")
    if not args.skip_patch and not EXPORT_BIN.exists():
        sys.exit(f"Missing {EXPORT_BIN}")

    os.environ.setdefault("HF_ENDPOINT", "https://hf-mirror.com")
    OUT_DIR.mkdir(parents=True, exist_ok=True)

    suite = load_suite()
    prompts = select_prompts(suite, args.phase)
    corpus = load_artifact(args.artifact).corpus if args.artifact.exists() else suite["corpus"]["ingest_clean"]
    slices = classify_slices(prompts, corpus)
    patch_ids = patch_prompt_ids(prompts, slices)
    patch_prompts = [p for p in prompts if p["id"] in patch_ids]

    print(f"=== Unified Hybrid · {args.phase} · {len(prompts)}Q · patch on {len(patch_prompts)} ===", file=sys.stderr)

    print("=== Memory: triad persistent ===", file=sys.stderr)
    triad_rows = run_persistent_memory("triad", corpus, prompts, mcq_fate=False)

    if args.skip_tile_fate:
        tile_rows = triad_rows
    else:
        print("=== Memory: tile persistent + MCQ Fate ===", file=sys.stderr)
        tile_rows = run_persistent_memory("tile", corpus, prompts, mcq_fate=True)

    memory_sources = {
        MEMORY_TRIAD: triad_rows,
        MEMORY_TILE_FATE: tile_rows,
    }

    if args.skip_patch and args.reuse_json:
        patch_rows = load_patch_from_benchmark(args.reuse_json, patch_ids)
    elif args.skip_patch:
        sys.exit("--skip-patch requires --reuse-json")
    else:
        from patch_track import eval_patch_arm_fresh

        bundle = load_artifact(args.artifact) if args.artifact.exists() else None
        export_doc = bundle.export_doc if bundle and bundle.body == args.patch_body else None
        print(f"=== Patch LM: {args.patch_body} on {len(patch_prompts)} prompts ===", file=sys.stderr)
        patch_result = eval_patch_arm_fresh(
            "patch_triad_hybrid_unified",
            args.patch_body,
            corpus,
            patch_prompts,
            model_name=args.model,
            strength=args.patch_strength,
            patch_mode="legacy",
            max_new=args.max_new,
            export_doc=export_doc,
        )
        patch_rows = patch_result["rows"]

    hybrid_rows, _ = build_hybrid_rows(prompts, memory_sources, patch_rows, corpus)
    summary = hybrid_summary(hybrid_rows, memory_sources, patch_rows, slices)

    tag = datetime.now().strftime("%Y%m%d_%H%M")
    payload = {
        "meta": {
            "benchmark": "hybrid_valhalla_unified",
            "hybrid_product_version": HYBRID_PRODUCT_VERSION,
            "fair_benchmark_version": FAIR_BENCHMARK_VERSION,
            "hub_f64_version": "1.1.0",
            "phase": args.phase,
            "prompt_count": len(prompts),
            "patch_lm_prompt_count": len(patch_prompts),
            "corpus_lines": len(corpus),
            "timestamp": tag,
            "patch_body": args.patch_body,
            "patch_strength": args.patch_strength,
            "model": args.model,
            "memory_sources": [MEMORY_TRIAD, MEMORY_TILE_FATE],
            "artifact": str(args.artifact),
        },
        "summary": summary,
        "hybrid_rows": hybrid_rows,
        "memory_sources": memory_sources,
        "patch_rows": patch_rows,
    }

    out_json = OUT_DIR / f"hybrid_unified_{args.phase}_{tag}.json"
    out_md = REPO / "reports" / f"VALHALLA_HYBRID_UNIFIED_{args.phase}_{tag}.md"
    out_json.write_text(json.dumps(payload, indent=2, ensure_ascii=False), encoding="utf-8")
    write_report(payload, out_md)

    h = summary["hybrid"]
    st = h.get("by_score_type", {})
    print(f"\n=== Unified Hybrid ({args.phase}) ===", file=sys.stderr)
    print(f"Total:   {h['correct']}/{h['n']} ({h['acc']:.1%})", file=sys.stderr)
    print(f"open:    {st.get('open', {}).get('acc', 0):.1%}", file=sys.stderr)
    print(f"mcq:     {st.get('mcq', {}).get('acc', 0):.1%}", file=sys.stderr)
    print(f"numeric: {st.get('numeric', {}).get('acc', 0):.1%}", file=sys.stderr)
    print(f"Sources: {summary['hybrid_source_counts']}", file=sys.stderr)
    print(f"Wrote {out_md}", file=sys.stderr)


if __name__ == "__main__":
    main()
