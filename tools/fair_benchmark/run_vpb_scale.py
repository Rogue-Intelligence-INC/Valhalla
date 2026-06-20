#!/usr/bin/env python3
"""
VPB Phase C — corpus scale axis (S/M/L) on frozen fair test + memory suite.

Usage:
  python run_vpb_scale.py --suite fair --phase test --tiers S,M
  python run_vpb_scale.py --suite memory --phase smoke --tiers S,M,L
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from datetime import datetime
from pathlib import Path
from typing import Any

REPO = Path(__file__).resolve().parents[2]
FAIR = Path(__file__).resolve().parent
OUT_DIR = REPO / "reports/vpb"
NATIVE_BIN = REPO / "target/release/valhalla_native_qa"

sys.path.insert(0, str(FAIR))

from metrics import retrieval_hit, score_answer, summarize_rows  # noqa: E402
from run_fair_benchmark import load_suite, select_prompts  # noqa: E402
from run_potential_analysis import classify_slices, slice_acc, slice_retrieval  # noqa: E402
from run_strict_incubation_experiment import strict_user_text  # noqa: E402
from session import native_qa_batch_persistent, native_qa_isolated  # noqa: E402


def load_vpb_memory() -> dict:
    p = FAIR / "vpb_memory_suite.json"
    if not p.exists():
        import subprocess

        subprocess.run([sys.executable, str(FAIR / "build_vpb_suite.py")], check=True)
    return json.loads(p.read_text(encoding="utf-8"))


def load_tiers() -> dict:
    p = FAIR / "vpb_corpus_tiers.json"
    if not p.exists():
        import subprocess

        subprocess.run([sys.executable, str(FAIR / "build_vpb_suite.py")], check=True)
    return json.loads(p.read_text(encoding="utf-8"))


def select_vpb_prompts(suite: dict, phase: str) -> list[dict]:
    phase_cfg = suite["phases"][phase]
    ids = set(suite["splits"][phase_cfg["split"]])
    return [p for p in suite["prompts"] if p["id"] in ids]


def score_row(prompt: dict, native: dict) -> dict:
    ans = native.get("answer", "")
    if prompt.get("abstention_expected"):
        low = ans.lower()
        kws = [str(k).lower() for k in prompt.get("keywords", []) if k]
        leaked = any(k in low for k in kws if len(k) >= 4)
        reject = native.get("decode_path") == "open_reject_no_keywords" or len(ans.strip()) < 3
        correct = reject or not leaked
        score = {"score_type": "abstention", "correct": correct, "reject": reject, "leaked": leaked}
    else:
        score = score_answer(ans, prompt)
    ret = None
    if native.get("top_matches"):
        ret = retrieval_hit(prompt, native["top_matches"], k=3)
    return {
        "id": prompt["id"],
        "memory_task": prompt.get("memory_task"),
        "domain": prompt.get("domain", ""),
        "score_type": prompt.get("score_type", "open"),
        "correct": score["correct"],
        "score": score,
        "native_answer": ans,
        "decode_path": native.get("decode_path"),
        "retrieval": ret,
        "top_matches": native.get("top_matches"),
        "session": "persistent",
    }


def evidence_recall(prompt: dict, corpus: list[str], top_matches: list[dict] | None) -> bool:
    """LONGMEMEVAL-style: evidence lines appear in retrieved context."""
    ev_ids = prompt.get("evidence_line_ids") or []
    if not ev_ids or not top_matches:
        return False
    tiers = load_tiers()
    idx = tiers.get("line_index_M") or {}
    id_to_line = {k: v for k, v in idx.items()}
    # rebuild index for current corpus length
    id_to_line = {f"c{i:03d}": line for i, line in enumerate(corpus)}
    targets = {id_to_line[e].lower() for e in ev_ids if e in id_to_line}
    if not targets:
        return False
    for m in top_matches[:3]:
        blob = (str(m.get("text", "")) + " " + str(m.get("source", ""))).lower()
        if any(t[:40] in blob or any(w in blob for w in t.split()[:4]) for t in targets):
            return True
    return False


def run_persistent_tier(
    prompts: list[dict],
    corpus: list[str],
    tier: str,
    body: str = "triad",
) -> list[dict]:
    batch_q = [(strict_user_text(p), p.get("score_type", "open"), p["id"]) for p in prompts]
    results = native_qa_batch_persistent(body, corpus, batch_q, incubation_cycles=1, qa_cycles=1)
    rows = []
    for p, native in zip(prompts, results):
        row = score_row(p, native)
        row["corpus_tier"] = tier
        rows.append(row)
    return rows


def run_isolated_sample(
    prompts: list[dict],
    corpus: list[str],
    tier: str,
    sample_ids: set[str],
    body: str = "triad",
) -> list[dict]:
    rows = []
    for p in prompts:
        if p["id"] not in sample_ids:
            continue
        native = native_qa_isolated(
            body, 1, corpus, strict_user_text(p), p.get("score_type", "open")
        )
        row = score_row(p, native)
        row["corpus_tier"] = tier
        row["session"] = "isolated"
        if row.get("retrieval") is None and native.get("top_matches"):
            row["retrieval"] = retrieval_hit(p, native["top_matches"], k=3)
        rows.append(row)
    return rows


def task_breakdown(rows: list[dict]) -> dict[str, Any]:
    by: dict[str, list] = {}
    for r in rows:
        t = r.get("memory_task") or "unknown"
        by.setdefault(t, []).append(r)
    out = {}
    for t, rs in by.items():
        c = sum(1 for x in rs if x["correct"])
        out[t] = {"n": len(rs), "acc": round(c / len(rs), 4) if rs else 0.0}
    return out


def tier_metrics(
    pers_rows: list[dict],
    iso_rows: list[dict],
    baseline_rows: list[dict] | None,
    slices: dict[str, set[str]] | None,
) -> dict[str, Any]:
    summary = summarize_rows(pers_rows)
    m: dict[str, Any] = {"persistent": summary}
    if slices:
        cs = slices["corpus_supported"]
        m["corpus_supported_acc"] = slice_acc(pers_rows, cs)["acc"]
        m["recall_at_3_isolated"] = slice_retrieval(iso_rows, cs) if iso_rows else None
        if baseline_rows:
            br = {r["id"]: r["correct"] for r in baseline_rows}
            sub = [r for r in pers_rows if r["id"] in cs]
            sub = [r for r in sub if r["id"] in br]
            if sub:
                lift = sum(r["correct"] for r in sub) - sum(br[r["id"]] for r in sub)
                m["ingest_lift_pp"] = round(lift / len(sub) * 100, 2)
    if any(r.get("memory_task") for r in pers_rows):
        m["by_memory_task"] = task_breakdown(pers_rows)
        ab = [r for r in pers_rows if r.get("memory_task") == "abstention"]
        if ab:
            m["abstention_acc"] = round(sum(r["correct"] for r in ab) / len(ab), 4)
    return m


def fit_scale_curve(tier_results: dict[str, dict]) -> dict[str, Any]:
    tiers = ["S", "M", "L"]
    accs = []
    recalls = []
    for t in tiers:
        if t not in tier_results:
            continue
        accs.append((t, tier_results[t].get("corpus_supported_acc") or tier_results[t]["persistent"]["acc"]))
        r = tier_results[t].get("recall_at_3_isolated")
        if r is not None:
            recalls.append((t, r))
    headroom = 0.5
    if len(accs) >= 2:
        delta = accs[-1][1] - accs[0][1]
        headroom = min(1.0, max(0.0, 0.5 + delta * 2))
    return {
        "corpus_supported_acc_by_tier": {t: a for t, a in accs},
        "recall_at_3_by_tier": {t: r for t, r in recalls},
        "scale_headroom_norm": round(headroom, 4),
        "memory_horizon_tier": next((t for t, r in recalls if r >= 0.85), None),
    }


def load_baseline_from_fair_json() -> list[dict] | None:
    p = REPO / "reports/fair_benchmark/fair_benchmark_test_20260620_0859.json"
    if not p.exists():
        return None
    data = json.loads(p.read_text(encoding="utf-8"))
    for r in data["results"]:
        if r["arm"] == "baseline_no_corpus":
            return [{"id": row["id"], "correct": row["correct"]} for row in r["rows"]]
    return None


def main() -> None:
    ap = argparse.ArgumentParser(description="VPB corpus scale experiment")
    ap.add_argument("--suite", choices=["fair", "memory", "both"], default="both")
    ap.add_argument("--phase", default="test")
    ap.add_argument("--tiers", default="S,M,L")
    ap.add_argument("--isolated-sample", type=int, default=40, help="Max isolated runs per tier for recall")
    args = ap.parse_args()

    if not NATIVE_BIN.exists():
        sys.exit("Build valhalla_native_qa first")

    tier_names = [t.strip() for t in args.tiers.split(",") if t.strip()]
    tiers_doc = load_tiers()
    tag = datetime.now().strftime("%Y%m%d_%H%M")
    OUT_DIR.mkdir(parents=True, exist_ok=True)

    payload: dict[str, Any] = {
        "meta": {
            "experiment": "vpb_scale_axis",
            "timestamp": tag,
            "phase": args.phase,
            "tiers": tier_names,
            "suite": args.suite,
        },
        "tier_line_counts": tiers_doc["meta"]["tiers"],
        "results": {},
    }

    if args.suite in ("fair", "both"):
        fair = load_suite()
        prompts = select_prompts(fair, args.phase)
        corpus_m = fair["corpus"]["ingest_clean"]
        slices = classify_slices(prompts, corpus_m)
        baseline_rows = load_baseline_from_fair_json()
        cs_ids = slices["corpus_supported"]

        fair_tiers: dict[str, Any] = {}
        sample_ids = set(list(cs_ids)[: args.isolated_sample])

        for tier in tier_names:
            corpus = tiers_doc["tiers"][tier]
            print(f"=== fair persistent tier {tier} ({len(corpus)} lines) ===", file=sys.stderr)
            pers = run_persistent_tier(prompts, corpus, tier)
            print(f"=== fair isolated sample tier {tier} ===", file=sys.stderr)
            iso = run_isolated_sample(prompts, corpus, tier, sample_ids)
            slices_t = classify_slices(prompts, corpus)
            fair_tiers[tier] = tier_metrics(pers, iso, baseline_rows, slices_t)
            fair_tiers[tier]["rows_persistent"] = pers
            fair_tiers[tier]["rows_isolated"] = iso

        payload["results"]["fair_L0"] = {
            "prompt_count": len(prompts),
            "metrics_by_tier": {
                t: {k: v for k, v in fair_tiers[t].items() if not k.startswith("rows_")}
                for t in tier_names
            },
            "scale_curve": fit_scale_curve({t: fair_tiers[t] for t in tier_names}),
        }

    if args.suite in ("memory", "both"):
        mem_suite = load_vpb_memory()
        mem_prompts = select_vpb_prompts(mem_suite, args.phase if args.phase in mem_suite["phases"] else "test")
        mem_tiers: dict[str, Any] = {}

        for tier in tier_names:
            corpus = tiers_doc["tiers"][tier]
            print(f"=== memory persistent tier {tier} ===", file=sys.stderr)
            pers = run_persistent_tier(mem_prompts, corpus, tier)
            ev_recall = []
            iso_subset = [p for p in mem_prompts if p.get("evidence_line_ids")][:20]
            for p in iso_subset:
                iso = native_qa_isolated(
                    "triad", 1, corpus, strict_user_text(p), p.get("score_type", "open")
                )
                ev_recall.append(evidence_recall(p, corpus, iso.get("top_matches")))
            mem_tiers[tier] = {
                "persistent": summarize_rows(pers),
                "by_memory_task": task_breakdown(pers),
                "evidence_recall_at_3": round(sum(ev_recall) / len(ev_recall), 4) if ev_recall else None,
            }
            ab = [r for r in pers if r.get("memory_task") == "abstention"]
            mem_tiers[tier]["abstention_acc"] = (
                round(sum(r["correct"] for r in ab) / len(ab), 4) if ab else None
            )

        payload["results"]["memory_L1"] = {
            "prompt_count": len(mem_prompts),
            "metrics_by_tier": mem_tiers,
        }

    out_json = OUT_DIR / f"vpb_scale_{args.phase}_{tag}.json"
    out_json.write_text(json.dumps(payload, indent=2, ensure_ascii=False), encoding="utf-8")
    write_scale_report(payload, REPO / "reports" / f"VALHALLA_VPB_SCALE_{args.phase}_{tag}.md")
    print(f"Wrote {out_json}", file=sys.stderr)


def write_scale_report(payload: dict, path: Path) -> None:
    lines = [
        "# VPB Phase C — Corpus Scale Axis",
        "",
        f"**Run**: {payload['meta']['timestamp']} · **Phase**: {payload['meta']['phase']}",
        f"**Tiers**: {payload['tier_line_counts']}",
        "",
    ]
    if "fair_L0" in payload["results"]:
        r = payload["results"]["fair_L0"]
        lines.append("## Fair L0 (frozen test) — persistent by tier")
        lines.append("")
        lines.append("| Tier | acc | corpus_supported | recall@3 | ingest_lift |")
        lines.append("|------|-----|------------------|----------|-------------|")
        for t, m in r["metrics_by_tier"].items():
            pers = m["persistent"]
            lines.append(
                f"| {t} | {pers['acc']*100:.1f}% | "
                f"{(m.get('corpus_supported_acc') or 0)*100:.1f}% | "
                f"{(m.get('recall_at_3_isolated') or 0)*100:.1f}% | "
                f"{m.get('ingest_lift_pp', '—')} pp |"
            )
        sc = r.get("scale_curve", {})
        lines.extend(
            [
                "",
                f"**scale_headroom_norm**: {sc.get('scale_headroom_norm')}",
                f"**memory_horizon_tier** (Recall≥0.85): {sc.get('memory_horizon_tier')}",
                "",
            ]
        )
    if "memory_L1" in payload["results"]:
        r = payload["results"]["memory_L1"]
        lines.append("## Memory L1 taxonomy — by tier")
        lines.append("")
        for t, m in r["metrics_by_tier"].items():
            lines.append(f"### Tier {t}")
            lines.append(f"- acc: {m['persistent']['acc']*100:.1f}%")
            lines.append(f"- abstention: {m.get('abstention_acc')}")
            lines.append(f"- evidence_recall: {m.get('evidence_recall_at_3')}")
            lines.append(f"- tasks: `{m.get('by_memory_task')}`")
            lines.append("")
    path.write_text("\n".join(lines), encoding="utf-8")
    print(f"Wrote {path}", file=sys.stderr)


if __name__ == "__main__":
    main()
