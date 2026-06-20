#!/usr/bin/env python3
"""
Valhalla Fair Benchmark — complete runner (memory + patch + generate).

Usage:
  python build_fair_suite.py
  python validate_suite.py
  python run_fair_benchmark.py --phase smoke
  python run_fair_benchmark.py --phase test --full
  python run_fair_benchmark.py --phase test --with-patch --with-transformer
  python sync_to_valhalla_transformer.py
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from datetime import datetime
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
FAIR = Path(__file__).resolve().parent
BRIDGE = REPO / "tools/valhalla_model_bridge"
SUITE_FILE = FAIR / "fair_benchmark_suite.json"
OUT_DIR = REPO / "reports/fair_benchmark"
NATIVE_BIN = REPO / "target/release/valhalla_native_qa"
EXPORT_BIN = REPO / "target/release/valhalla_model_export"
VT_REPO = Path("/home/jouly/AI/valhalla-transformer")

sys.path.insert(0, str(BRIDGE))
sys.path.insert(0, str(FAIR))
sys.path.insert(0, str(REPO / "tools/valhalla_artifact"))

from artifact import attach_eval_report, load_artifact, write_artifact  # noqa: E402

from metrics import bootstrap_ci_paired, retrieval_hit, score_answer, summarize_rows  # noqa: E402
from run_strict_incubation_experiment import strict_user_text  # noqa: E402
from session import native_qa_batch_persistent, native_qa_isolated  # noqa: E402


def load_suite() -> dict:
    if not SUITE_FILE.exists():
        subprocess.run([sys.executable, str(FAIR / "build_fair_suite.py")], check=True)
    return json.loads(SUITE_FILE.read_text(encoding="utf-8"))


def select_prompts(suite: dict, phase: str) -> list[dict]:
    phase_cfg = suite["phases"][phase]
    ids = set(suite["splits"][phase_cfg["split"]])
    return [p for p in suite["prompts"] if p["id"] in ids]


def select_corpus(suite: dict, phase: str) -> list[str]:
    return suite["corpus"][suite["phases"][phase]["corpus"]]


def run_memory_arm(arm_cfg: dict, prompts: list[dict], corpus: list[str]) -> dict:
    arm_id = arm_cfg["id"]
    body = arm_cfg["body"]
    cycles = arm_cfg["cycles"]
    use_corpus = arm_cfg["use_corpus"]
    session = arm_cfg.get("session", "isolated")
    c = corpus if use_corpus else []

    print(f"=== memory: {arm_id} ({session}) ===", file=sys.stderr)
    rows = []

    if session == "persistent":
        if not c:
            raise ValueError("persistent session requires corpus")
        batch_q = [
            (strict_user_text(p), p.get("score_type", "open"), p["id"]) for p in prompts
        ]
        batch_results = native_qa_batch_persistent(
            body,
            c,
            batch_q,
            incubation_cycles=arm_cfg.get("incubation_cycles", 1),
            qa_cycles=arm_cfg.get("qa_cycles", 1),
        )
        by_id = {r.get("id"): r for r in batch_results}
        for p in prompts:
            resp = by_id.get(p["id"], {})
            sc = score_answer(resp.get("answer", ""), p, strict_mcq=True)
            rows.append(
                {
                    "id": p["id"],
                    "domain": p["domain"],
                    "score_type": p.get("score_type"),
                    "correct": sc["correct"],
                    "score": sc,
                    "native_answer": resp.get("answer"),
                    "decode_path": resp.get("decode_path"),
                    "retrieval": {"recall_at_k": False, "mrr": 0.0},
                    "session": "persistent",
                    "structure": {
                        "tile_count": resp.get("tile_count"),
                        "stem_cells": resp.get("stem_cells"),
                    },
                }
            )
    else:
        for p in prompts:
            user_text = strict_user_text(p)
            resp = native_qa_isolated(body, cycles, c, user_text, p.get("score_type", "open"))
            sc = score_answer(resp["answer"], p, strict_mcq=True)
            top = resp.get("top_matches") or []
            rows.append(
                {
                    "id": p["id"],
                    "domain": p["domain"],
                    "score_type": p.get("score_type"),
                    "correct": sc["correct"],
                    "score": sc,
                    "native_answer": resp["answer"],
                    "decode_path": resp.get("decode_path"),
                    "retrieval": retrieval_hit(p, top, k=3),
                    "top_matches": top[:3],
                    "session": "isolated",
                    "structure": {
                        "tile_count": resp.get("tile_count"),
                        "stem_cells": resp.get("stem_cells"),
                        "memory_items": resp.get("memory_items"),
                    },
                }
            )

    return {
        "arm": arm_id,
        "track": "memory_tier_b",
        **arm_cfg,
        "summary": summarize_rows(rows),
        "rows": rows,
    }


def write_report(payload: dict, out_md: Path) -> None:
    meta = payload["meta"]
    lines = [
        "# Valhalla Fair Benchmark Report",
        "",
        f"**Phase**: {meta['phase']} · **Split**: {meta['split']} · **Corpus**: {meta['corpus_tier']}",
        f"**Time**: {meta['timestamp']} · **Prompts**: {meta['prompt_count']}",
        f"**Suite**: {meta.get('version', 'fair-1.0')} · **Clean corpus**: {meta.get('corpus_clean', True)}",
        f"**Tracks**: memory={meta.get('with_memory')} patch={meta.get('with_patch')} generate={meta.get('with_transformer')}",
        "",
    ]

    def track_table(track: str, title: str) -> None:
        arms = [r for r in payload["results"] if r.get("track") == track]
        if not arms:
            return
        lines.extend([f"## {title}", "", "| Arm | Acc | numeric | mcq | open |", "|-----|-----|---------|-----|------|"])
        for r in arms:
            s = r["summary"]
            st = s.get("by_score_type", {})
            lines.append(
                f"| {r['arm']} | {s['acc']:.2%} | "
                f"{st.get('numeric', {}).get('acc', 0):.2%} | "
                f"{st.get('mcq', {}).get('acc', 0):.2%} | "
                f"{st.get('open', {}).get('acc', 0):.2%} |"
            )
        lines.append("")

    track_table("memory_tier_b", "Memory track (Tier B)")
    track_table("patch_tier_a", "Patch track (Tier A · fresh reload / Q)")
    track_table("generate_baseline", "Generate track (Transformer zero-shot)")

    memory = [r for r in payload["results"] if r.get("track") == "memory_tier_b"]
    baseline = next((r for r in memory if r["arm"] == "baseline_no_corpus"), None)
    if baseline and meta.get("paired_ci"):
        lines.extend(["## Paired Δ vs baseline (memory, pp 95% CI)", ""])
        base_rows = {row["id"]: row["correct"] for row in baseline["rows"]}
        for r in memory:
            if r["arm"] == "baseline_no_corpus":
                continue
            before = [base_rows[row["id"]] for row in r["rows"]]
            after = [row["correct"] for row in r["rows"]]
            ci = bootstrap_ci_paired(before, after)
            lines.append(
                f"- **{r['arm']}**: {ci['mean_pp']:+.2f} pp [{ci['ci95_lo']:+.2f}, {ci['ci95_hi']:+.2f}]"
            )
        lines.append("")

    mem_isolated = [r for r in memory if r.get("session") == "isolated" and "decode_path" in str(r.get("summary", {}))]
    if memory:
        lines.extend(["## Decode path (memory, isolated arms)", ""])
        for r in memory:
            if r.get("session") != "isolated":
                continue
            paths = r["summary"].get("by_decode_path", {})
            if paths:
                parts = ", ".join(f"{k}={v['acc']:.0%}" for k, v in sorted(paths.items()))
                lines.append(f"- **{r['arm']}**: {parts}")
        lines.append("")

    out_md.write_text("\n".join(lines) + "\n", encoding="utf-8")


def sync_vt(payload: dict, out_json: Path) -> None:
    if not VT_REPO.exists():
        return
    exp_dir = VT_REPO / "experiments"
    exp_dir.mkdir(parents=True, exist_ok=True)
    tag = payload["meta"]["timestamp"]
    phase = payload["meta"]["phase"]
    slim = {
        "meta": payload["meta"],
        "results": [
            {
                "arm": r["arm"],
                "track": r.get("track"),
                "session": r.get("session"),
                "acc": r["summary"]["acc"],
                "by_score_type": r["summary"].get("by_score_type"),
                "retrieval": r["summary"].get("retrieval"),
            }
            for r in payload["results"]
        ],
    }
    dst = exp_dir / f"fair_benchmark_{phase}_{tag}.json"
    dst.write_text(json.dumps(slim, indent=2, ensure_ascii=False), encoding="utf-8")
    latest = exp_dir / "fair_benchmark_latest.json"
    latest.write_text(json.dumps(payload, indent=2, ensure_ascii=False), encoding="utf-8")
    print(f"Synced -> {dst}", file=sys.stderr)


def main() -> None:
    ap = argparse.ArgumentParser(description="Valhalla Fair Benchmark (complete)")
    ap.add_argument("--phase", choices=["smoke", "val", "test", "test_legacy_ablation"], default="smoke")
    ap.add_argument("--full", action="store_true", help="memory + patch + generate")
    ap.add_argument("--with-patch", action="store_true")
    ap.add_argument("--with-transformer", action="store_true")
    ap.add_argument("--skip-memory", action="store_true")
    ap.add_argument("--artifact", type=Path, default=None, help="Use incubation artifact corpus (+ save eval)")
    ap.add_argument("--save-artifact", type=Path, default=None, help="Write triad artifact after memory run")
    ap.add_argument("--skip-sync", action="store_true")
    ap.add_argument("--model", default="Qwen/Qwen2.5-0.5B-Instruct")
    ap.add_argument("--max-new", type=int, default=192)
    ap.add_argument("--patch-strength", type=float, default=0.08)
    args = ap.parse_args()

    with_memory = not args.skip_memory
    with_patch = args.with_patch or args.full
    with_transformer = args.with_transformer or args.full

    if with_memory and not NATIVE_BIN.exists():
        print(f"Missing {NATIVE_BIN}", file=sys.stderr)
        sys.exit(1)
    if with_patch and not EXPORT_BIN.exists():
        print(f"Missing {EXPORT_BIN}", file=sys.stderr)
        sys.exit(1)

    os.environ.setdefault("HF_ENDPOINT", "https://hf-mirror.com")
    OUT_DIR.mkdir(parents=True, exist_ok=True)

    suite = load_suite()
    phase = args.phase
    artifact_bundle = None
    if args.artifact:
        artifact_bundle = load_artifact(args.artifact)
        corpus = artifact_bundle.corpus
        prompts = select_prompts(suite, phase)
        phase_meta = suite["phases"][phase]
        print(
            f"Using artifact {args.artifact} id={artifact_bundle.manifest['artifact_id']} "
            f"corpus={len(corpus)} patch_hash={artifact_bundle.patch_hash}",
            file=sys.stderr,
        )
    else:
        prompts = select_prompts(suite, phase)
        corpus = select_corpus(suite, phase)
        phase_meta = suite["phases"][phase]

    results = []
    if with_memory:
        for arm_cfg in suite["arms"]["memory_tier_b"]:
            results.append(run_memory_arm(arm_cfg, prompts, corpus))

    if with_patch:
        from patch_track import eval_patch_arm_fresh  # noqa: E402

        for arm_cfg in suite["arms"]["patch_tier_a"]:
            print(f"=== patch: {arm_cfg['id']} ===", file=sys.stderr)
            pre_export = None
            if artifact_bundle and artifact_bundle.body == arm_cfg["body"]:
                pre_export = artifact_bundle.export_doc
            results.append(
                eval_patch_arm_fresh(
                    arm_cfg["id"],
                    arm_cfg["body"],
                    corpus,
                    prompts,
                    model_name=args.model,
                    strength=arm_cfg.get("strength", args.patch_strength),
                    patch_mode=arm_cfg.get("patch_mode", "legacy"),
                    max_new=args.max_new,
                    cycles=arm_cfg.get("cycles", 1),
                    export_doc=pre_export,
                )
            )

    if with_transformer:
        from patch_track import eval_transformer_fresh  # noqa: E402

        print(f"=== generate: transformer_zero_shot ===", file=sys.stderr)
        results.append(eval_transformer_fresh(prompts, model_name=args.model, max_new=args.max_new))

    tag = datetime.now().strftime("%Y%m%d_%H%M")
    payload = {
        "meta": {
            "benchmark": "valhalla_fair",
            "version": suite["meta"]["version"],
            "phase": phase,
            "split": phase_meta["split"],
            "corpus_tier": phase_meta["corpus"],
            "corpus_lines": len(corpus),
            "corpus_clean": phase_meta["corpus"] == "ingest_clean",
            "prompt_count": len(prompts),
            "timestamp": tag,
            "paired_ci": phase_meta["split"] == "test",
            "with_memory": with_memory,
            "with_patch": with_patch,
            "with_transformer": with_transformer,
            "artifact_id": artifact_bundle.manifest["artifact_id"] if artifact_bundle else None,
        },
        "results": results,
    }

    out_json = OUT_DIR / f"fair_benchmark_{phase}_{tag}.json"
    out_md = REPO / "reports" / f"VALHALLA_FAIR_BENCHMARK_{phase}_{tag}.md"
    out_json.write_text(json.dumps(payload, indent=2, ensure_ascii=False), encoding="utf-8")
    write_report(payload, out_md)

    if args.save_artifact and with_memory:
        from run_incubation_benchmark import export_body  # noqa: E402

        triad_export = export_body("triad", 1, corpus)
        write_artifact(
            args.save_artifact,
            triad_export,
            corpus,
            meta={"label": f"fair_{phase}", "benchmark_run": tag},
            eval_report=payload,
        )
        print(f"Wrote artifact {args.save_artifact}", file=sys.stderr)
    elif artifact_bundle is not None:
        attach_eval_report(artifact_bundle.root, payload)

    if not args.skip_sync:
        sync_vt(payload, out_json)
    print(f"Wrote {out_json}", file=sys.stderr)
    print(f"Wrote {out_md}", file=sys.stderr)


if __name__ == "__main__":
    main()
