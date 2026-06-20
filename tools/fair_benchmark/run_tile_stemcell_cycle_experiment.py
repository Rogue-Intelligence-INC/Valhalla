#!/usr/bin/env python3
"""
Tile / StemCell multi-cycle optimization experiment.

Tests whether additional optimize cycles change structure and QA acc.

Protocols:
  export    — structure-only (patch_hash, tile/stem counts, cycle_trace)
  isolated  — native_qa per question: corpus_cycles=N, qa_cycles=1
  batch     — one incubation, many questions: incubation_cycles=N (hash grinder)

Usage:
  python run_tile_stemcell_cycle_experiment.py --phase smoke
  python run_tile_stemcell_cycle_experiment.py --phase test --protocols export,isolated
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
OUT_DIR = REPO / "reports/tile_stemcell_cycles"
NATIVE_BIN = REPO / "target/release/valhalla_native_qa"
EXPORT_BIN = REPO / "target/release/valhalla_model_export"

sys.path.insert(0, str(BRIDGE))
sys.path.insert(0, str(FAIR))
sys.path.insert(0, str(REPO / "tools/valhalla_artifact"))

from artifact import load_artifact, patch_fingerprint  # noqa: E402
from metrics import bootstrap_ci_paired, score_answer, summarize_rows  # noqa: E402
from run_fair_benchmark import load_suite, select_prompts  # noqa: E402
from run_strict_incubation_experiment import strict_user_text  # noqa: E402
from session import native_qa_batch_persistent, native_qa_isolated  # noqa: E402

BODIES = ("tile", "stemcell")
DEFAULT_CYCLES = (1, 5, 10, 20)


def export_structure(body: str, cycles: int, corpus: list[str]) -> dict:
    proc = subprocess.run(
        [
            str(EXPORT_BIN),
            "--body",
            body,
            "--mode",
            "normalized_f64",
            "--cycles",
            str(cycles),
            "--experiment",
        ],
        input="\n".join(corpus) + "\n",
        capture_output=True,
        text=True,
        check=True,
        env={**os.environ, "RUSTFLAGS": "-L /opt/cuda/lib64"},
    )
    doc = json.loads(proc.stdout)
    mp = doc["report"]["model_patch"]
    trace = doc["report"].get("cycle_trace") or []
    return {
        "body": body,
        "cycles": cycles,
        "patch_hash": patch_fingerprint(doc),
        "optimize_cycles": doc["report"].get("optimize_cycles", cycles),
        "tile_count": mp["tile_count"],
        "stem_cell_count": mp["stem_cell_count"],
        "tile_ops": doc["report"].get("tile_ops"),
        "stem_cells_reported": doc["report"].get("stem_cells"),
        "cycle_trace_len": len(trace),
        "cycle_trace_tail": trace[-3:] if trace else [],
        "patch_vector_head": mp["patch_vector"][:8],
    }


def native_qa_cycles(
    body: str,
    corpus: list[str],
    question: str,
    score_type: str,
    corpus_cycles: int,
    qa_cycles: int = 1,
) -> dict:
    req = {
        "body": body,
        "cycles": corpus_cycles,
        "corpus_cycles": corpus_cycles,
        "qa_cycles": qa_cycles,
        "corpus": corpus,
        "question": question,
        "score_type": score_type,
    }
    proc = subprocess.run(
        [str(NATIVE_BIN)],
        input=json.dumps(req),
        capture_output=True,
        text=True,
        check=True,
        env={**os.environ, "RUSTFLAGS": "-L /opt/cuda/lib64"},
    )
    return json.loads(proc.stdout)


def run_isolated_arm(
    body: str,
    cycles: int,
    prompts: list[dict],
    corpus: list[str],
) -> dict:
    arm = f"{body}_c{cycles}_isolated"
    print(f"=== {arm} ===", file=sys.stderr)
    rows = []
    structures = []
    for p in prompts:
        user_text = strict_user_text(p)
        resp = native_qa_cycles(body, corpus, user_text, p.get("score_type", "open"), corpus_cycles=cycles)
        sc = score_answer(resp["answer"], p, strict_mcq=True)
        rows.append(
            {
                "id": p["id"],
                "score_type": p.get("score_type"),
                "correct": sc["correct"],
                "decode_path": resp.get("decode_path"),
                "tile_count": resp.get("tile_count"),
                "stem_cells": resp.get("stem_cells"),
            }
        )
        structures.append(
            {
                "tile_count": resp.get("tile_count"),
                "stem_cells": resp.get("stem_cells"),
                "cycles_executed": resp.get("cycles_executed"),
            }
        )
    # structure stable across questions?
    tc = {s["tile_count"] for s in structures}
    sc = {s["stem_cells"] for s in structures}
    return {
        "arm": arm,
        "body": body,
        "protocol": "isolated",
        "cycles": cycles,
        "structure_unique": {"tile_count": sorted(tc), "stem_cells": sorted(sc)},
        "summary": summarize_rows(rows),
        "rows": rows,
    }


def run_batch_arm(
    body: str,
    incubation_cycles: int,
    prompts: list[dict],
    corpus: list[str],
) -> dict:
    arm = f"{body}_c{incubation_cycles}_batch"
    print(f"=== {arm} ===", file=sys.stderr)
    batch_q = [(strict_user_text(p), p.get("score_type", "open"), p["id"]) for p in prompts]
    results = native_qa_batch_persistent(
        body, corpus, batch_q, incubation_cycles=incubation_cycles, qa_cycles=1
    )
    by_id = {r.get("id"): r for r in results}
    rows = []
    for p in prompts:
        resp = by_id.get(p["id"], {})
        sc = score_answer(resp.get("answer", ""), p, strict_mcq=True)
        rows.append(
            {
                "id": p["id"],
                "score_type": p.get("score_type"),
                "correct": sc["correct"],
                "decode_path": resp.get("decode_path"),
                "tile_count": resp.get("tile_count"),
                "stem_cells": resp.get("stem_cells"),
            }
        )
    tc = {r.get("tile_count") for r in results}
    sc = {r.get("stem_cells") for r in results}
    return {
        "arm": arm,
        "body": body,
        "protocol": "batch",
        "cycles": incubation_cycles,
        "structure_unique": {"tile_count": sorted(tc), "stem_cells": sorted(sc)},
        "summary": summarize_rows(rows),
        "rows": rows,
    }


def write_report(payload: dict, path: Path) -> None:
    meta = payload["meta"]
    lines = [
        "# Tile / StemCell 多轮 Cycle 优化实验",
        "",
        f"**Run**: {meta['timestamp']} · **Phase**: {meta['phase']} · **N**: {meta['prompt_count']}",
        f"**Corpus**: {meta['corpus_lines']} lines · **Cycles tested**: {meta['cycles_tested']}",
        "",
        "## 1. 结构 export（patch 是否随 cycle 变）",
        "",
        "| body | cycles | patch_hash | tiles | stem | trace_len |",
        "|------|--------|------------|-------|------|-----------|",
    ]
    for r in payload.get("export_structure", []):
        lines.append(
            f"| {r['body']} | {r['cycles']} | `{r['patch_hash']}` | {r['tile_count']} | "
            f"{r['stem_cell_count']} | {r['cycle_trace_len']} |"
        )

    lines.extend(["", "## 2. QA acc（isolated · corpus_cycles）", ""])
    iso = [r for r in payload["results"] if r["protocol"] == "isolated"]
    if iso:
        lines.append("| arm | acc | numeric | mcq | open | tile/stem unique |")
        lines.append("|-----|-----|---------|-----|------|------------------|")
        for r in iso:
            st = r["summary"].get("by_score_type", {})
            su = r.get("structure_unique", {})
            lines.append(
                f"| {r['arm']} | {r['summary']['acc']:.1%} | "
                f"{st.get('numeric', {}).get('acc', 0):.1%} | "
                f"{st.get('mcq', {}).get('acc', 0):.1%} | "
                f"{st.get('open', {}).get('acc', 0):.1%} | "
                f"t{su.get('tile_count')} s{su.get('stem_cells')} |"
            )

    batch = [r for r in payload["results"] if r["protocol"] == "batch"]
    if batch:
        lines.extend(["", "## 3. QA acc（batch · incubation_cycles）", ""])
        lines.append("| arm | acc | numeric | mcq | open |")
        lines.append("|-----|-----|---------|-----|------|")
        for r in batch:
            st = r["summary"].get("by_score_type", {})
            lines.append(
                f"| {r['arm']} | {r['summary']['acc']:.1%} | "
                f"{st.get('numeric', {}).get('acc', 0):.1%} | "
                f"{st.get('mcq', {}).get('acc', 0):.1%} | "
                f"{st.get('open', {}).get('acc', 0):.1%} |"
            )

    lines.extend(["", "## 4. Δ vs c1（isolated）", ""])
    for body in BODIES:
        base = next((r for r in iso if r["body"] == body and r["cycles"] == 1), None)
        if not base:
            continue
        br = {row["id"]: row["correct"] for row in base["rows"]}
        for r in iso:
            if r["body"] != body or r["cycles"] == 1:
                continue
            after = [row["correct"] for row in r["rows"]]
            before = [br[row["id"]] for row in r["rows"]]
            ci = bootstrap_ci_paired(before, after)
            lines.append(
                f"- **{r['arm']}**: {ci['mean_pp']:+.2f} pp [{ci['ci95_lo']:+.2f}, {ci['ci95_hi']:+.2f}]"
            )

    lines.extend(["", "## 5. 结论", "", payload.get("verdict", ""), ""])
    path.write_text("\n".join(lines), encoding="utf-8")


def build_verdict(export_rows: list[dict], results: list[dict]) -> str:
    parts = []
    by_body: dict[str, list] = {}
    for r in export_rows:
        by_body.setdefault(r["body"], []).append(r)

    for body, rows in by_body.items():
        hashes = {r["cycles"]: r["patch_hash"] for r in rows}
        counts = {r["cycles"]: (r["tile_count"], r["stem_cell_count"]) for r in rows}
        stable = len(set(hashes.values())) == 1
        parts.append(
            f"**{body} structure**: patch_hash {'不变 (c1≡c20)' if stable else '随 cycle 变化'} · "
            f"counts c1→c20: {counts.get(1)} → {counts.get(20, counts.get(max(counts)))}."
        )

    iso = [r for r in results if r["protocol"] == "isolated"]
    for body in BODIES:
        c1 = next((r for r in iso if r["body"] == body and r["cycles"] == 1), None)
        c20 = next((r for r in iso if r["body"] == body and r["cycles"] == 20), None)
        if c1 and c20:
            d = (c20["summary"]["acc"] - c1["summary"]["acc"]) * 100
            parts.append(f"**{body} QA isolated c1→c20**: {d:+.2f} pp.")

    return " ".join(parts)


def main() -> None:
    ap = argparse.ArgumentParser(description="Tile/StemCell multi-cycle experiment")
    ap.add_argument("--phase", choices=["smoke", "test"], default="smoke")
    ap.add_argument("--artifact", type=Path, default=REPO / "checkpoints/incubation_artifacts/fair_triad_c1")
    ap.add_argument("--cycles", default="1,5,10,20", help="Comma-separated cycle counts")
    ap.add_argument(
        "--protocols",
        default="export,isolated,batch",
        help="export,isolated,batch",
    )
    args = ap.parse_args()

    if not EXPORT_BIN.exists() or not NATIVE_BIN.exists():
        sys.exit("Build valhalla_model_export and valhalla_native_qa first")

    cycle_list = [int(x.strip()) for x in args.cycles.split(",") if x.strip()]
    protocols = {p.strip() for p in args.protocols.split(",")}

    bundle = load_artifact(args.artifact) if args.artifact.exists() else None
    corpus = bundle.corpus if bundle else json.loads((FAIR / "fair_benchmark_suite.json").read_text())["corpus"]["ingest_clean"]
    prompts = select_prompts(load_suite(), args.phase)

    OUT_DIR.mkdir(parents=True, exist_ok=True)
    export_rows = []
    results = []

    if "export" in protocols:
        for body in BODIES:
            for c in cycle_list:
                export_rows.append(export_structure(body, c, corpus))

    if "isolated" in protocols:
        for body in BODIES:
            for c in cycle_list:
                results.append(run_isolated_arm(body, c, prompts, corpus))

    if "batch" in protocols:
        for body in BODIES:
            for c in cycle_list:
                results.append(run_batch_arm(body, c, prompts, corpus))

    tag = datetime.now().strftime("%Y%m%d_%H%M")
    payload = {
        "meta": {
            "experiment": "tile_stemcell_multi_cycle",
            "timestamp": tag,
            "phase": args.phase,
            "prompt_count": len(prompts),
            "corpus_lines": len(corpus),
            "cycles_tested": cycle_list,
            "protocols": sorted(protocols),
            "artifact_id": bundle.manifest["artifact_id"] if bundle else None,
        },
        "export_structure": export_rows,
        "results": results,
    }
    payload["verdict"] = build_verdict(export_rows, results)

    out_json = OUT_DIR / f"tile_stemcell_cycles_{args.phase}_{tag}.json"
    out_md = REPO / "reports" / f"VALHALLA_TILE_STEMCELL_CYCLES_{args.phase}_{tag}.md"
    out_json.write_text(json.dumps(payload, indent=2, ensure_ascii=False), encoding="utf-8")
    write_report(payload, out_md)
    print(f"Wrote {out_json}", file=sys.stderr)
    print(f"Wrote {out_md}", file=sys.stderr)
    print(payload["verdict"], file=sys.stderr)


if __name__ == "__main__":
    main()
