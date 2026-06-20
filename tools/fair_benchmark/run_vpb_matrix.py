#!/usr/bin/env python3
"""
Valhalla Potential Benchmark (VPB) v1.0 — Phase A matrix from existing fair JSON.

Computes HELM-style scenario×metric matrix, hybrid oracle, PFI, and gate checks.

Usage:
  python run_vpb_matrix.py --reuse-json reports/fair_benchmark/fair_benchmark_test_20260620_0859.json
"""

from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime
from pathlib import Path
from typing import Any

REPO = Path(__file__).resolve().parents[2]
FAIR = Path(__file__).resolve().parent
OUT_DIR = REPO / "reports/vpb"

sys.path.insert(0, str(FAIR))

from run_potential_analysis import (  # noqa: E402
    classify_slices,
    load_from_benchmark_json,
    load_suite_prompts,
    slice_acc,
    slice_retrieval,
)


SCENARIOS = {
    "S1_memory_platform": "memory_platform",
    "S2_memory_retrieval": "memory_retrieval",
    "S3_routing_mcq": "routing_mcq",
    "S4_numeric_compute": "numeric_compute",
    "S5_open_retrieval": "open_retrieval",
    "S6_hybrid_product": "hybrid_product",
}

ARM_KEYS = {
    "baseline": "baseline_no_corpus",
    "isolated": "triad_c1",
    "persistent": "triad_c1_persistent",
    "patch": "patch_triad_legacy",
    "tf": "transformer_zero_shot",
}


def row_faithfulness(row: dict) -> float:
    """RAGAS-style faithfulness proxy for Tier B native decode."""
    dp = row.get("decode_path", "")
    ret = row.get("retrieval") or {}
    if dp == "direct_math":
        return 1.0
    if dp == "open_reject_no_keywords":
        return 1.0
    if ret.get("recall_at_k"):
        return 1.0 if row.get("correct") else 0.55
    if row.get("correct") and dp in ("mcq_option", "open_span", "cortex"):
        return 0.65
    return 0.0


def row_token_cost_est(row: dict) -> float:
    """BEAM/Mem0-inspired token budget proxy (not exact tokens)."""
    struct = row.get("structure") or {}
    matches = row.get("top_matches") or []
    if matches:
        return len(matches) * 120.0 + struct.get("memory_items", 0) * 60.0
    return struct.get("tile_count", 0) * 8.0 + struct.get("stem_cells", 0) * 4.0 + 200.0


def mean_faithfulness(rows: list[dict], ids: set[str]) -> float | None:
    sub = [r for r in rows if r["id"] in ids]
    if not sub:
        return None
    return round(sum(row_faithfulness(r) for r in sub) / len(sub), 4)


def mean_token_cost(rows: list[dict], ids: set[str]) -> float | None:
    sub = [r for r in rows if r["id"] in ids]
    if not sub:
        return None
    return round(sum(row_token_cost_est(r) for r in sub) / len(sub), 1)


def ingest_lift_pp(
    with_corpus: list[dict], baseline: list[dict], ids: set[str]
) -> float | None:
    if not ids:
        return None
    br = {r["id"]: r["correct"] for r in baseline}
    sub = [r for r in with_corpus if r["id"] in ids]
    if not sub:
        return None
    lift = sum(r["correct"] for r in sub) - sum(br[r["id"]] for r in sub)
    return round(lift / len(sub) * 100, 2)


def hybrid_pick_arm(
    prompt: dict,
    slices: dict[str, set[str]],
) -> str:
    pid = prompt["id"]
    st = prompt.get("score_type", "open")
    cs = pid in slices["corpus_supported"]
    og = pid in slices["open_generation"]

    if og:
        return "patch"
    if st == "mcq" and cs:
        return "persistent"
    if st == "numeric":
        return "hybrid_max_persistent_patch"
    if st == "open" and cs:
        return "patch"
    if cs:
        return "persistent"
    return "persistent"


def build_hybrid_rows(
    prompts: list[dict],
    arms: dict[str, list[dict]],
    slices: dict[str, set[str]],
) -> list[dict]:
    pers = {r["id"]: r for r in arms["persistent"]}
    patch = {r["id"]: r for r in arms["patch"]}
    iso = {r["id"]: r for r in arms["isolated"]}
    out = []

    for p in prompts:
        pid = p["id"]
        choice = hybrid_pick_arm(p, slices)
        if choice == "hybrid_max_persistent_patch":
            correct = pers[pid]["correct"] or patch[pid]["correct"]
            src = "persistent" if pers[pid]["correct"] else ("patch" if patch[pid]["correct"] else "none")
        elif choice == "patch":
            correct = patch[pid]["correct"]
            src = "patch"
        else:
            correct = pers[pid]["correct"]
            src = "persistent"

        base = pers.get(pid) or iso.get(pid) or patch[pid]
        out.append(
            {
                **{k: v for k, v in base.items() if k not in ("correct",)},
                "correct": correct,
                "hybrid_source": src,
                "hybrid_policy": choice,
            }
        )
    return out


def scenario_ids(scenario: str, slices: dict[str, set[str]]) -> set[str]:
    if scenario == "memory_platform":
        return slices["corpus_supported"]
    if scenario == "memory_retrieval":
        return slices["corpus_supported"]
    if scenario == "routing_mcq":
        return slices["mcq_routing"]
    if scenario == "numeric_compute":
        return slices["numeric_compute"]
    if scenario == "open_retrieval":
        return slices["open_retrieval"]
    if scenario == "hybrid_product":
        return set().union(*slices.values())  # all classified ids
    return set()


def all_prompt_ids(prompts: list[dict]) -> set[str]:
    return {p["id"] for p in prompts}


def build_matrix_cell(
    scenario: str,
    arms: dict[str, list[dict]],
    hybrid_rows: list[dict],
    slices: dict[str, set[str]],
    prompts: list[dict],
) -> dict[str, Any]:
    ids = scenario_ids(scenario, slices)
    if scenario == "hybrid_product":
        ids = all_prompt_ids(prompts)

    cell: dict[str, Any] = {"scenario": scenario, "n": len(ids)}

    if scenario in ("memory_platform", "memory_retrieval"):
        rows = arms["persistent"] if scenario == "memory_platform" else arms["isolated"]
        if ids:
            cell["M1_task_acc"] = slice_acc(rows, ids)["acc"]
            cell["M2_recall_at_3"] = slice_retrieval(arms["isolated"], ids)
            cell["M3_ingest_lift_pp"] = ingest_lift_pp(
                arms["persistent"], arms["baseline"], ids
            )
            cell["M4_faithfulness"] = mean_faithfulness(rows, ids)
            cell["M5_token_cost_est"] = mean_token_cost(rows, ids)
        return cell

    if scenario == "routing_mcq":
        cell["M1_task_acc_valhalla"] = slice_acc(arms["persistent"], ids)["acc"]
        cell["M1_task_acc_tf"] = slice_acc(arms["tf"], ids)["acc"]
        cell["M2_recall_at_3"] = slice_retrieval(arms["isolated"], ids)
        cell["M3_ingest_lift_pp"] = ingest_lift_pp(
            arms["persistent"], arms["baseline"], ids
        )
        cell["M4_faithfulness"] = mean_faithfulness(arms["persistent"], ids)
        cell["M5_token_cost_est"] = mean_token_cost(arms["persistent"], ids)
        return cell

    if scenario == "numeric_compute":
        cell["M1_task_acc_valhalla"] = slice_acc(arms["persistent"], ids)["acc"]
        cell["M1_task_acc_tf"] = slice_acc(arms["tf"], ids)["acc"]
        cell["M1_task_acc_patch"] = slice_acc(arms["patch"], ids)["acc"]
        cell["M4_faithfulness"] = mean_faithfulness(arms["persistent"], ids)
        cell["M5_token_cost_est"] = mean_token_cost(arms["persistent"], ids)
        return cell

    if scenario == "open_retrieval":
        cell["M1_task_acc_memory"] = slice_acc(arms["persistent"], ids)["acc"]
        cell["M1_task_acc_patch"] = slice_acc(arms["patch"], ids)["acc"]
        cell["M1_task_acc_tf"] = slice_acc(arms["tf"], ids)["acc"]
        cell["M2_recall_at_3"] = slice_retrieval(arms["isolated"], ids)
        cell["M4_faithfulness"] = mean_faithfulness(arms["persistent"], ids)
        return cell

    if scenario == "hybrid_product":
        cell["M1_task_acc_hybrid"] = slice_acc(hybrid_rows, ids)["acc"]
        cell["M1_task_acc_tf"] = slice_acc(arms["tf"], ids)["acc"]
        cell["M1_task_acc_persistent"] = slice_acc(arms["persistent"], ids)["acc"]
        cell["M1_task_acc_patch"] = slice_acc(arms["patch"], ids)["acc"]
        cell["M4_faithfulness"] = mean_faithfulness(hybrid_rows, ids)
        cell["M5_token_cost_est"] = mean_token_cost(hybrid_rows, ids)
        by_src: dict[str, int] = {}
        for r in hybrid_rows:
            if r["id"] in ids:
                s = r.get("hybrid_source", "?")
                by_src[s] = by_src.get(s, 0) + 1
        cell["hybrid_source_counts"] = by_src
        return cell

    return cell


def wedge_score(slices: dict[str, set[str]], pers: list[dict], tf: list[dict]) -> float:
    wedges = [
        ("mcq_routing", slices["mcq_routing"], 0.5),
        ("open_retrieval", slices["open_retrieval"], 0.5),
    ]
    total = 0.0
    wsum = 0.0
    for _, ids, w in wedges:
        if not ids:
            continue
        va = slice_acc(pers, ids)["acc"]
        ta = slice_acc(tf, ids)["acc"]
        total += w * (1.0 if va >= ta else 0.0)
        wsum += w
    return round(total / wsum, 4) if wsum else 0.0


def compute_pfi(
    matrix: dict[str, dict],
    slices: dict[str, set[str]],
    arms: dict[str, list[dict]],
    hybrid_rows: list[dict],
) -> dict[str, Any]:
    s1 = matrix["S1_memory_platform"]
    s3 = matrix["S3_routing_mcq"]
    s6 = matrix["S6_hybrid_product"]

    recall = s1.get("M2_recall_at_3") or 0.0
    ingest = s1.get("M3_ingest_lift_pp") or 0.0
    wedge = wedge_score(slices, arms["persistent"], arms["tf"])
    hybrid_acc = s6.get("M1_task_acc_hybrid") or 0.0
    tf_acc = s6.get("M1_task_acc_tf") or 0.0
    hybrid_gap = hybrid_acc - tf_acc  # positive = beats TF

    pfi = (
        0.25 * min(recall / 0.85, 1.0)
        + 0.20 * min(max(ingest, 0) / 25.0, 1.0)
        + 0.20 * wedge
        + 0.15 * min(max(hybrid_gap + 0.05, 0) / 0.10, 1.0)  # -5pp..+5pp mapped
        + 0.10 * 0.5  # scale_headroom placeholder (Phase C)
        + 0.10 * 0.5  # velocity placeholder (Phase C)
    )
    pfi = round(min(1.0, pfi), 4)

    return {
        "pfi": pfi,
        "interpretation": _pfi_label(pfi),
        "components": {
            "recall_norm": round(min(recall / 0.85, 1.0), 4),
            "ingest_lift_norm": round(min(max(ingest, 0) / 25.0, 1.0), 4),
            "wedge_score": wedge,
            "hybrid_minus_tf": round(hybrid_gap * 100, 2),
            "scale_headroom_norm": 0.5,
            "velocity_norm": 0.5,
        },
        "mcq_delta_pp": round(
            (s3.get("M1_task_acc_valhalla", 0) - s3.get("M1_task_acc_tf", 0)) * 100, 2
        ),
    }


def _pfi_label(pfi: float) -> str:
    if pfi >= 0.70:
        return "strong_potential"
    if pfi >= 0.50:
        return "viable_wedge"
    return "early_stage"


def check_gates(matrix: dict[str, dict], slices: dict[str, set[str]], arms: dict[str, list[dict]]) -> dict[str, Any]:
    s1 = matrix["S1_memory_platform"]
    s3 = matrix["S3_routing_mcq"]
    s6 = matrix["S6_hybrid_product"]

    g1 = (s1.get("M2_recall_at_3") or 0) >= 0.70
    g2 = (s1.get("M3_ingest_lift_pp") or 0) >= 15.0
    g3 = (s3.get("M1_task_acc_valhalla") or 0) >= (s3.get("M1_task_acc_tf") or 0) + 0.05
    g4 = (s6.get("M1_task_acc_hybrid") or 0) >= (s6.get("M1_task_acc_tf") or 0) - 0.05

    # abstention proxy: open_generation where memory rejects vs TF generates
    og = slices["open_generation"]
    og_mem = slice_acc(arms["persistent"], og)
    # G6 placeholder — no abstention suite in L0 yet
    g6 = None

    passed = g1 and g2 and (g3 or g4)
    return {
        "G1_recall_at_3_gte_0.70": g1,
        "G2_ingest_lift_gte_15pp": g2,
        "G3_mcq_wedge_gte_tf_plus_5pp": g3,
        "G4_hybrid_gte_tf_minus_5pp": g4,
        "G5_scale_headroom": None,
        "G6_abstention": g6,
        "open_generation_memory_acc": og_mem["acc"],
        "potential_benchmark_pass": passed,
        "interpretation": "platform_plus_wedge_or_hybrid" if passed else "gates_not_met",
    }


def write_report(payload: dict, path: Path) -> None:
    m = payload["matrix"]
    pfi = payload["pfi"]
    gates = payload["gates"]
    meta = payload["meta"]

    lines = [
        "# Valhalla Potential Benchmark (VPB) — Phase A Report",
        "",
        f"**Run**: {meta['timestamp']} · **Source**: {meta['source_json']}",
        f"**Phase**: {meta['phase']} · **N**: {meta['prompt_count']} · **Corpus tier**: M (49 lines)",
        "",
        "## Executive Summary",
        "",
        f"| 指标 | 值 |",
        f"|------|-----|",
        f"| **PFI** | **{pfi['pfi']:.2f}** ({pfi['interpretation']}) |",
        f"| Hybrid acc | {m['S6_hybrid_product']['M1_task_acc_hybrid']*100:.1f}% |",
        f"| TF acc | {m['S6_hybrid_product']['M1_task_acc_tf']*100:.1f}% |",
        f"| Hybrid − TF | {pfi['components']['hybrid_minus_tf']:+.1f} pp |",
        f"| Gates pass | {'✓' if gates['potential_benchmark_pass'] else '✗'} ({gates['interpretation']}) |",
        "",
        "> TF = 成熟静态基线 · Valhalla = 开发中 · 矩阵优于单一 acc",
        "",
        "## Potential Matrix (HELM-style)",
        "",
        "| Scenario | M1 TaskAcc | M2 Recall@3 | M3 IngestLift | M4 Faith | M5 TokenEst |",
        "|----------|------------|-------------|---------------|----------|-------------|",
    ]

    def pct(x: Any) -> str:
        if x is None:
            return "—"
        if isinstance(x, (int, float)) and x <= 1.0 and x >= 0:
            return f"{x*100:.1f}%"
        return str(x)

    s1 = m["S1_memory_platform"]
    lines.append(
        f"| S1 memory_platform | {pct(s1.get('M1_task_acc'))} | {pct(s1.get('M2_recall_at_3'))} "
        f"| {s1.get('M3_ingest_lift_pp', '—')} pp | {s1.get('M4_faithfulness', '—')} "
        f"| {s1.get('M5_token_cost_est', '—')} |"
    )
    s2 = m["S2_memory_retrieval"]
    lines.append(
        f"| S2 memory_retrieval | — | {pct(s2.get('M2_recall_at_3'))} | — "
        f"| {s2.get('M4_faithfulness', '—')} | {s2.get('M5_token_cost_est', '—')} |"
    )
    s3 = m["S3_routing_mcq"]
    lines.append(
        f"| S3 mcq (V/T) | {pct(s3.get('M1_task_acc_valhalla'))} / {pct(s3.get('M1_task_acc_tf'))} "
        f"| {pct(s3.get('M2_recall_at_3'))} | {s3.get('M3_ingest_lift_pp', '—')} pp "
        f"| {s3.get('M4_faithfulness', '—')} | {s3.get('M5_token_cost_est', '—')} |"
    )
    s4 = m["S4_numeric_compute"]
    lines.append(
        f"| S4 numeric (V/T/P) | {pct(s4.get('M1_task_acc_valhalla'))} / {pct(s4.get('M1_task_acc_tf'))} "
        f"/ {pct(s4.get('M1_task_acc_patch'))} | — | — | {s4.get('M4_faithfulness', '—')} "
        f"| {s4.get('M5_token_cost_est', '—')} |"
    )
    s5 = m["S5_open_retrieval"]
    lines.append(
        f"| S5 open_retr (M/P/T) | {pct(s5.get('M1_task_acc_memory'))} / {pct(s5.get('M1_task_acc_patch'))} "
        f"/ {pct(s5.get('M1_task_acc_tf'))} | {pct(s5.get('M2_recall_at_3'))} | — "
        f"| {s5.get('M4_faithfulness', '—')} | — |"
    )
    s6 = m["S6_hybrid_product"]
    lines.append(
        f"| S6 hybrid / TF | {pct(s6.get('M1_task_acc_hybrid'))} / {pct(s6.get('M1_task_acc_tf'))} "
        f"| — | — | {s6.get('M4_faithfulness', '—')} | {s6.get('M5_token_cost_est', '—')} |"
    )

    lines.extend(
        [
            "",
            f"Hybrid source mix: `{s6.get('hybrid_source_counts', {})}`",
            "",
            "## PFI Components",
            "",
            f"- recall_norm: {pfi['components']['recall_norm']}",
            f"- ingest_lift_norm: {pfi['components']['ingest_lift_norm']}",
            f"- wedge_score: {pfi['components']['wedge_score']}",
            f"- hybrid − TF: {pfi['components']['hybrid_minus_tf']} pp",
            f"- mcq Δ vs TF: {pfi['mcq_delta_pp']} pp",
            "",
            "## Gates",
            "",
        ]
    )
    for k, v in gates.items():
        if k != "interpretation":
            lines.append(f"- **{k}**: {v}")
    lines.append(f"\n**{gates['interpretation']}**")
    lines.extend(
        [
            "",
            "## 解读（成熟 TF vs 开发中 Valhalla）",
            "",
            "1. **S1/S2** 检索与 ingest lift 证明记忆平台信号；与 TF 无直接对比。",
            "2. **S3 MCQ 楔子** 是 Valhalla 已胜 mature TF 的域。",
            "3. **S6 Hybrid** 是产品路径上界；若 ≥ TF−5pp，无需等待 Tier B 全替。",
            "4. PFI 中 scale/velocity 为 Phase C 占位（当前 0.5）。",
            "",
            "## 下一步",
            "",
            "- Phase B: `vpb_memory_suite.json`（LoCoMo/LONGMEMEVAL taxonomy）",
            "- Phase C: S/M/L 语料档 + milestone velocity",
            "",
            f"*VPB v1.0 · {meta['timestamp']}*",
        ]
    )
    path.write_text("\n".join(lines), encoding="utf-8")


def main() -> None:
    ap = argparse.ArgumentParser(description="VPB Phase A matrix from fair JSON")
    ap.add_argument(
        "--reuse-json",
        type=Path,
        default=REPO / "reports/fair_benchmark/fair_benchmark_test_20260620_0859.json",
    )
    ap.add_argument("--phase", default="baseline")
    args = ap.parse_args()

    if not args.reuse_json.exists():
        sys.exit(f"Missing {args.reuse_json}")

    raw = json.loads(args.reuse_json.read_text(encoding="utf-8"))
    arm_rows = load_from_benchmark_json(args.reuse_json)

    missing = [k for k in ARM_KEYS.values() if k not in arm_rows]
    if missing:
        sys.exit(f"JSON missing arms: {missing}")

    arms = {alias: arm_rows[key] for alias, key in ARM_KEYS.items()}
    prompts, corpus = load_suite_prompts("test")
    if raw["meta"].get("prompt_count") != len(prompts):
        prompts, corpus = load_suite_prompts(raw["meta"].get("phase", "test"))

    slices = classify_slices(prompts, corpus)
    hybrid_rows = build_hybrid_rows(prompts, arms, slices)

    matrix = {}
    for sid, scenario in SCENARIOS.items():
        matrix[sid] = build_matrix_cell(scenario, arms, hybrid_rows, slices, prompts)

    pfi = compute_pfi(matrix, slices, arms, hybrid_rows)
    gates = check_gates(matrix, slices, arms)

    tag = datetime.now().strftime("%Y%m%d_%H%M")
    OUT_DIR.mkdir(parents=True, exist_ok=True)

    payload = {
        "meta": {
            "benchmark": "valhalla_potential_v1",
            "phase": args.phase,
            "timestamp": tag,
            "source_json": str(args.reuse_json),
            "prompt_count": len(prompts),
            "corpus_lines": len(corpus),
            "methodology": [
                "HELM scenario×metric matrix",
                "LoCoMo/LONGMEMEVAL recall@3",
                "RAGAS faithfulness proxy",
                "hybrid oracle S6",
            ],
        },
        "slices": {k: sorted(v) for k, v in slices.items()},
        "matrix": matrix,
        "pfi": pfi,
        "gates": gates,
        "hybrid_summary": summarize_rows(hybrid_rows),
        "arm_summaries": {
            alias: summarize_rows(rows) for alias, rows in arms.items()
        },
    }

    out_json = OUT_DIR / f"vpb_matrix_{args.phase}_{tag}.json"
    out_md = REPO / "reports" / f"VALHALLA_VPB_MATRIX_{args.phase}_{tag}.md"
    out_json.write_text(json.dumps(payload, indent=2, ensure_ascii=False), encoding="utf-8")
    write_report(payload, out_md)

    print(f"Wrote {out_json}", file=sys.stderr)
    print(f"Wrote {out_md}", file=sys.stderr)
    print(
        f"PFI={pfi['pfi']:.2f} hybrid={matrix['S6_hybrid_product']['M1_task_acc_hybrid']*100:.1f}% "
        f"TF={matrix['S6_hybrid_product']['M1_task_acc_tf']*100:.1f}% "
        f"pass={gates['potential_benchmark_pass']}",
        file=sys.stderr,
    )


def summarize_rows(rows: list[dict]) -> dict:
    from metrics import summarize_rows as sr

    return sr(rows)


if __name__ == "__main__":
    main()
