#!/usr/bin/env python3
"""
Valhalla potential analysis — Memory Model + Transformer Replacement.

Usage:
  python run_potential_analysis.py --artifact checkpoints/incubation_artifacts/fair_triad_c1/
  python run_potential_analysis.py --reuse-json reports/fair_benchmark/fair_benchmark_test_20260620_0859.json
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from collections import defaultdict
from datetime import datetime
from pathlib import Path
from typing import Any

REPO = Path(__file__).resolve().parents[2]
FAIR = Path(__file__).resolve().parent
BRIDGE = REPO / "tools/valhalla_model_bridge"
OUT_DIR = REPO / "reports/potential_analysis"
NATIVE_BIN = REPO / "target/release/valhalla_native_qa"

sys.path.insert(0, str(BRIDGE))
sys.path.insert(0, str(FAIR))
sys.path.insert(0, str(REPO / "tools/valhalla_artifact"))

from artifact import load_artifact  # noqa: E402
from metrics import bootstrap_ci_paired, retrieval_hit, score_answer, summarize_rows  # noqa: E402
from run_fair_benchmark import run_memory_arm  # noqa: E402
from run_strict_incubation_experiment import strict_user_text  # noqa: E402


def load_suite_prompts(phase: str = "test") -> tuple[list[dict], list[str]]:
    from run_fair_benchmark import load_suite, select_prompts

    suite = load_suite()
    return select_prompts(suite, phase), suite["corpus"]["ingest_clean"]


def classify_slices(prompts: list[dict], corpus: list[str]) -> dict[str, set[str]]:
    def supported(p: dict) -> bool:
        ref = str(p.get("reference_answer", "")).lower()
        kws = [str(k).lower() for k in p.get("keywords", []) if k]
        for line in corpus:
            low = line.lower()
            if ref and len(ref) >= 2 and ref in low:
                return True
            if sum(1 for k in kws if k in low) >= max(1, len(kws) // 2):
                return True
        return False

    slices: dict[str, set[str]] = {
        "corpus_supported": set(),
        "numeric_compute": set(),
        "mcq_routing": set(),
        "open_retrieval": set(),
        "open_generation": set(),
    }
    for p in prompts:
        pid = p["id"]
        st = p.get("score_type", "open")
        if st == "numeric":
            slices["numeric_compute"].add(pid)
        if st == "mcq":
            slices["mcq_routing"].add(pid)
        if supported(p):
            slices["corpus_supported"].add(pid)
            if st == "open":
                slices["open_retrieval"].add(pid)
        elif st == "open":
            slices["open_generation"].add(pid)
    return slices


def slice_acc(rows: list[dict], ids: set[str]) -> dict[str, Any]:
    sub = [r for r in rows if r["id"] in ids]
    if not sub:
        return {"n": 0, "acc": 0.0, "correct": 0}
    c = sum(1 for r in sub if r["correct"])
    return {"n": len(sub), "acc": round(c / len(sub), 4), "correct": c}


def slice_retrieval(rows: list[dict], ids: set[str]) -> float:
    sub = [r for r in rows if r["id"] in ids and r.get("retrieval")]
    if not sub:
        return 0.0
    hits = [r["retrieval"].get("recall_at_k", False) for r in sub]
    return round(sum(hits) / len(hits), 4)


def memory_potential_index(
    persistent_rows: list[dict],
    baseline_rows: list[dict],
    isolated_rows: list[dict],
    slices: dict[str, set[str]],
) -> dict[str, Any]:
    cs = slices["corpus_supported"]
    br = {r["id"]: r["correct"] for r in baseline_rows}
    pr = [r for r in persistent_rows if r["id"] in cs]
    br_sub = [br[r["id"]] for r in pr]

    # persistent batch omits top_matches — use isolated session for retrieval@3
    recall = slice_retrieval(isolated_rows, cs)
    ingest_lift = (sum(r["correct"] for r in pr) - sum(br_sub)) / max(len(pr), 1) * 100
    iso_sub = [r for r in isolated_rows if r["id"] in cs]
    pers_acc = sum(r["correct"] for r in pr) / max(len(pr), 1)
    iso_acc = sum(r["correct"] for r in iso_sub) / max(len(iso_sub), 1)
    persistent_gain = (pers_acc - iso_acc) * 100

    mpi = min(1.0, 0.35 * recall + 0.35 * min(ingest_lift / 25, 1.0) + 0.30 * min(persistent_gain / 15, 1.0))
    return {
        "memory_potential_index": round(mpi, 4),
        "recall_at_3_corpus_supported": recall,
        "ingest_lift_pp_corpus_supported": round(ingest_lift, 2),
        "persistent_vs_isolated_pp_corpus_supported": round(persistent_gain, 2),
        "interpretation": _mpi_label(mpi),
    }


def _mpi_label(mpi: float) -> str:
    if mpi >= 0.65:
        return "strong_memory_platform_signal"
    if mpi >= 0.45:
        return "moderate_memory_potential"
    if mpi >= 0.25:
        return "early_memory_wedge"
    return "memory_platform_not_ready"


def replacement_potential_index(
    valhalla_rows: list[dict],
    tf_rows: list[dict],
    slices: dict[str, set[str]],
) -> dict[str, Any]:
    wedge_slices = [
        ("corpus_supported", slices["corpus_supported"]),
        ("mcq_routing", slices["mcq_routing"]),
        ("open_retrieval", slices["open_retrieval"]),
    ]
    tf_map = {r["id"]: r["correct"] for r in tf_rows}
    total_w = sum(len(ids) for _, ids in wedge_slices)
    tri_sum = 0.0
    slice_compare = []

    for name, ids in wedge_slices:
        if not ids:
            continue
        va = slice_acc(valhalla_rows, ids)
        ta = slice_acc(tf_rows, ids)
        wins = va["acc"] >= ta["acc"]
        w = len(ids) / total_w
        tri_sum += w * (1.0 if wins else 0.0)
        slice_compare.append(
            {
                "slice": name,
                "n": va["n"],
                "valhalla_acc": va["acc"],
                "transformer_acc": ta["acc"],
                "delta_pp": round((va["acc"] - ta["acc"]) * 100, 2),
                "valhalla_wins_or_tie": wins,
            }
        )

    og = slices["open_generation"]
    mem_og = slice_acc(valhalla_rows, og)
    tf_og = slice_acc(tf_rows, og)

    tri = round(tri_sum, 4)
    return {
        "transformer_replacement_index": tri,
        "wedge_slice_compare": slice_compare,
        "open_generation_gap_pp": round((tf_og["acc"] - mem_og["acc"]) * 100, 2),
        "open_generation": {"memory": mem_og, "transformer": tf_og},
        "interpretation": _tri_label(tri, mem_og["acc"], tf_og["acc"]),
    }


def _tri_label(tri: float, mem_og: float, tf_og: float) -> str:
    if tri >= 0.67 and tf_og - mem_og > 0.3:
        return "partial_replacement_hybrid_required"
    if tri >= 0.5:
        return "wedge_substitution_possible"
    if tri >= 0.33:
        return "niche_replacement_only"
    return "not_a_transformer_replacement"


def load_from_benchmark_json(path: Path) -> dict[str, list[dict]]:
    data = json.loads(path.read_text(encoding="utf-8"))
    out = {}
    for r in data["results"]:
        out[r["arm"]] = r["rows"]
    return out


def run_live_memory(prompts: list[dict], corpus: list[str], suite: dict) -> dict[str, list[dict]]:
    arms = {
        "baseline_no_corpus": next(a for a in suite["arms"]["memory_tier_b"] if a["id"] == "baseline_no_corpus"),
        "triad_c1": next(a for a in suite["arms"]["memory_tier_b"] if a["id"] == "triad_c1"),
        "triad_c1_persistent": next(a for a in suite["arms"]["memory_tier_b"] if a["id"] == "triad_c1_persistent"),
    }
    return {name: run_memory_arm(cfg, prompts, corpus)["rows"] for name, cfg in arms.items()}


def run_live_transformer(prompts: list[dict], model: str, max_new: int) -> list[dict]:
    from patch_track import eval_transformer_fresh

    return eval_transformer_fresh(prompts, model_name=model, max_new=max_new)["rows"]


def write_report(payload: dict, path: Path) -> None:
    m = payload["meta"]
    mem = payload["memory_potential"]
    rep = payload["replacement_potential"]
    lines = [
        "# Valhalla 潜力分析实验报告",
        "",
        f"**Run**: {m['timestamp']} · **Split**: test · **N**: {m['prompt_count']}",
        f"**Artifact**: {m.get('artifact_id', '—')}",
        "",
        "## A. 记忆模型潜力 (MPI)",
        "",
        f"| 指标 | 值 |",
        f"|------|-----|",
        f"| **Memory Potential Index** | **{mem['memory_potential_index']:.2f}** ({mem['interpretation']}) |",
        f"| recall@3 (corpus_supported) | {mem['recall_at_3_corpus_supported']:.2%} |",
        f"| ingest lift (corpus_supported) | {mem['ingest_lift_pp_corpus_supported']:+.2f} pp |",
        f"| persistent vs isolated | {mem['persistent_vs_isolated_pp_corpus_supported']:+.2f} pp |",
        "",
        "## B. Transformer 替代潜力 (TRI)",
        "",
        f"| 指标 | 值 |",
        f"|------|-----|",
        f"| **Replacement Index** | **{rep['transformer_replacement_index']:.2f}** ({rep['interpretation']}) |",
        f"| open_generation gap (TF − memory) | {rep['open_generation_gap_pp']:+.2f} pp |",
        "",
        "### 楔子切片 vs Transformer",
        "",
        "| Slice | N | Valhalla | TF | Δ pp | 胜 |",
        "|-------|---|----------|-----|------|-----|",
    ]
    for s in rep["wedge_slice_compare"]:
        win = "✓" if s["valhalla_wins_or_tie"] else "✗"
        lines.append(
            f"| {s['slice']} | {s['n']} | {s['valhalla_acc']:.1%} | {s['transformer_acc']:.1%} | "
            f"{s['delta_pp']:+.1f} | {win} |"
        )

    lines.extend(["", "## 切片明细（persistent memory）", ""])
    for name, stats in payload.get("slice_breakdown_persistent", {}).items():
        lines.append(f"- **{name}**: {stats['acc']:.1%} ({stats['correct']}/{stats['n']})")

    pt = payload.get("patch_triad_on_wedge")
    if pt:
        lines.extend(["", "## Patch triad vs Memory vs TF（楔子切片）", ""])
        for slice_name, v in pt.items():
            lines.append(
                f"- **{slice_name}**: memory {v['memory_acc']:.1%} · patch {v['patch_acc']:.1%} · TF {v['tf_acc']:.1%}"
            )

    lines.extend(["", "## 结论", "", payload.get("verdict", ""), ""])
    path.write_text("\n".join(lines), encoding="utf-8")


def build_verdict(mpi: dict, rep: dict) -> str:
    parts = []
    parts.append(
        f"**记忆模型**：MPI={mpi['memory_potential_index']:.2f}（{mpi['interpretation']}）— "
        f"clean 语料 ingest 在 corpus-supported 子集上有 +{mpi['ingest_lift_pp_corpus_supported']:.1f}pp  lift，"
        f"持久 session 再 +{mpi['persistent_vs_isolated_pp_corpus_supported']:.1f}pp。"
    )
    parts.append(
        f"**TF 替代**：TRI={rep['transformer_replacement_index']:.2f}（{rep['interpretation']}）— "
        f"楔子域可部分替代；open_generation 仍落后 TF {rep['open_generation_gap_pp']:.1f}pp，需 hybrid。"
    )
    return " ".join(parts)


def main() -> None:
    ap = argparse.ArgumentParser(description="Valhalla potential analysis experiment")
    ap.add_argument("--artifact", type=Path, default=REPO / "checkpoints/incubation_artifacts/fair_triad_c1")
    ap.add_argument("--reuse-json", type=Path, default=None, help="Reuse arms from fair benchmark JSON")
    ap.add_argument("--run-transformer", action="store_true", help="Re-run TF (slow); default reuse JSON")
    ap.add_argument("--model", default="Qwen/Qwen2.5-0.5B-Instruct")
    ap.add_argument("--max-new", type=int, default=192)
    args = ap.parse_args()

    os.environ.setdefault("HF_ENDPOINT", "https://hf-mirror.com")
    OUT_DIR.mkdir(parents=True, exist_ok=True)

    from run_fair_benchmark import load_suite

    suite = load_suite()
    prompts, default_corpus = load_suite_prompts("test")
    artifact_id = None
    if args.artifact.exists():
        bundle = load_artifact(args.artifact)
        corpus = bundle.corpus
        artifact_id = bundle.manifest["artifact_id"]
    else:
        corpus = default_corpus

    slices = classify_slices(prompts, corpus)

    if args.reuse_json and args.reuse_json.exists():
        rows_by_arm = load_from_benchmark_json(args.reuse_json)
        print(f"Reused arms from {args.reuse_json}", file=sys.stderr)
    else:
        if not NATIVE_BIN.exists():
            sys.exit(f"Missing {NATIVE_BIN}")
        print("Running live memory arms...", file=sys.stderr)
        rows_by_arm = run_live_memory(prompts, corpus, suite)

    need = ["baseline_no_corpus", "triad_c1", "triad_c1_persistent", "transformer_zero_shot"]
    for k in ["baseline_no_corpus", "triad_c1", "triad_c1_persistent"]:
        if k not in rows_by_arm:
            sys.exit(f"Missing arm {k} in results")

    if "transformer_zero_shot" not in rows_by_arm or args.run_transformer:
        print("Running transformer_zero_shot (145Q fresh reload)...", file=sys.stderr)
        rows_by_arm["transformer_zero_shot"] = run_live_transformer(prompts, args.model, args.max_new)

    pers = rows_by_arm["triad_c1_persistent"]
    base = rows_by_arm["baseline_no_corpus"]
    iso = rows_by_arm["triad_c1"]
    tf = rows_by_arm["transformer_zero_shot"]

    mpi = memory_potential_index(pers, base, iso, slices)
    rep = replacement_potential_index(pers, tf, slices)

    slice_breakdown = {}
    for name, ids in slices.items():
        if ids:
            slice_breakdown[name] = slice_acc(pers, ids)

    br = {r["id"]: r["correct"] for r in base}
    ci = bootstrap_ci_paired([br[r["id"]] for r in pers], [r["correct"] for r in pers])

    tag = datetime.now().strftime("%Y%m%d_%H%M")
    payload = {
        "meta": {
            "experiment": "potential_analysis_v1",
            "timestamp": tag,
            "prompt_count": len(prompts),
            "artifact_id": artifact_id,
            "corpus_lines": len(corpus),
            "slice_sizes": {k: len(v) for k, v in slices.items()},
        },
        "slices": {k: sorted(v) for k, v in slices.items()},
        "memory_potential": {**mpi, "ingest_ci95_pp": ci},
        "replacement_potential": rep,
        "slice_breakdown_persistent": slice_breakdown,
        "patch_triad_on_wedge": None,
    }

    if "patch_triad_legacy" in rows_by_arm:
        pt = rows_by_arm["patch_triad_legacy"]
        payload["patch_triad_on_wedge"] = {
            s["slice"]: {
                "patch_acc": slice_acc(pt, slices[s["slice"]])["acc"],
                "memory_acc": slice_acc(pers, slices[s["slice"]])["acc"],
                "tf_acc": slice_acc(tf, slices[s["slice"]])["acc"],
            }
            for s in rep["wedge_slice_compare"]
        }

    payload["verdict"] = build_verdict(mpi, rep)

    out_json = OUT_DIR / f"potential_analysis_{tag}.json"
    out_md = REPO / "reports" / f"VALHALLA_POTENTIAL_ANALYSIS_{tag}.md"
    out_json.write_text(json.dumps(payload, indent=2, ensure_ascii=False), encoding="utf-8")
    write_report(payload, out_md)
    print(f"MPI={mpi['memory_potential_index']:.2f} TRI={rep['transformer_replacement_index']:.2f}", file=sys.stderr)
    print(f"Wrote {out_json}", file=sys.stderr)
    print(f"Wrote {out_md}", file=sys.stderr)


if __name__ == "__main__":
    main()
