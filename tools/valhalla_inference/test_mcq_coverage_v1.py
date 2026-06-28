#!/usr/bin/env python3
"""MCQ coverage battery — recent production stack on fair-1.2 MCQ slice.

Protocol: mcq-coverage-v1

Arms (all use targeted corpus ingest per question):
  - production_hybrid_lm_patch   decode=hybrid, mcq_decode=lm_patch (production default)
  - hybrid_structure_fate        decode=hybrid, mcq_decode=structure_fate
  - native_mcq_option            decode=native (structure mcq_option path)
  - lm_patch_all                 decode=lm_patch (all types via patch LM)
  - max_native_lm_patch          oracle: per-question max(native, lm_patch) — routing ceiling
  - traditional_lm_mcq           Qwen + corpus block (--with-trad-lm)

Usage:
  python3 tools/valhalla_inference/test_mcq_coverage_v1.py
  python3 tools/valhalla_inference/test_mcq_coverage_v1.py --fast
  python3 tools/valhalla_inference/test_mcq_coverage_v1.py --with-trad-lm  # full + trad
  python3 tools/valhalla_inference/test_mcq_coverage_v1.py --limit 10
"""

from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

REPO = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(REPO / "tools/fair_benchmark"))

from metrics import score_answer  # noqa: E402
from run_fair_benchmark import load_suite, select_corpus, select_prompts  # noqa: E402
from run_scale_benchmark import corpus_for_prompt  # noqa: E402
from run_strict_incubation_experiment import strict_user_text  # noqa: E402
from test_context_performance import context_block  # noqa: E402
from valhalla_base import ValhallaBase  # noqa: E402

PROTOCOL = "mcq-coverage-v1"
REPORTS = REPO / "reports/valhalla_inference"


def make_base(*, decode: str, mcq_decode: str = "lm_patch") -> ValhallaBase:
    return ValhallaBase(
        decode=decode,  # type: ignore[arg-type]
        mcq_decode=mcq_decode,  # type: ignore[arg-type]
        follow_up_decode="native_follow_up_aware",
        fate_ingress_routing="quad_cycle",
        fate_qa_feedback=False,
        incubation_cycles=2,
        max_new=128,
    )


def eval_arm(prompts: list[dict], corpus: list[str], *, decode: str, mcq_decode: str = "lm_patch") -> dict:
    base = make_base(decode=decode, mcq_decode=mcq_decode)
    rows: list[dict] = []
    decode_paths: dict[str, int] = {}
    for i, p in enumerate(prompts):
        print(f"    [{i + 1}/{len(prompts)}] {p['id']}", file=sys.stderr)
        ctx = corpus_for_prompt(p, corpus, k=8)
        q = {
            "id": p["id"],
            "text": strict_user_text(p),
            "score_type": "mcq",
            "reference_answer": p.get("reference_answer", ""),
            "keywords": p.get("keywords") or [],
        }
        base.clear_context()
        doc = base.run([q], context_items=[{"text": ln} for ln in ctx])
        row = doc["rows"][0]
        sc = score_answer(row["answer"], p)
        dp = str(row.get("decode_path") or "")
        decode_paths[dp] = decode_paths.get(dp, 0) + 1
        rows.append(
            {
                "id": p["id"],
                "correct": sc["correct"],
                "extracted": sc.get("extracted"),
                "reference": sc.get("reference"),
                "answer": str(row["answer"])[:200],
                "decode_path": dp,
                "context_lines": len(ctx),
            }
        )
    correct = sum(1 for r in rows if r["correct"])
    n = len(rows)
    return {
        "decode": decode,
        "mcq_decode": mcq_decode,
        "n": n,
        "correct": correct,
        "accuracy": correct / n if n else 0.0,
        "decode_path_counts": decode_paths,
        "rows": rows,
    }


def eval_trad_lm(prompts: list[dict], corpus: list[str]) -> dict:
    from test_context_arch_compare import resolve_lm_path  # noqa: E402
    from loader import ValhallaLLM  # noqa: E402

    lm = ValhallaLLM(resolve_lm_path("Qwen/Qwen2.5-0.5B-Instruct"))
    rows: list[dict] = []
    for i, p in enumerate(prompts):
        print(f"    trad [{i + 1}/{len(prompts)}] {p['id']}", file=sys.stderr)
        ctx = corpus_for_prompt(p, corpus, k=8)
        qtext = strict_user_text(p)
        ans = lm.generate([context_block(ctx) + qtext], max_tokens=128)[0].text.strip()
        sc = score_answer(ans, p)
        rows.append(
            {
                "id": p["id"],
                "correct": sc["correct"],
                "extracted": sc.get("extracted"),
                "reference": sc.get("reference"),
                "answer": ans[:200],
                "decode_path": "traditional_lm",
            }
        )
    correct = sum(1 for r in rows if r["correct"])
    n = len(rows)
    return {
        "arm": "traditional_lm_mcq",
        "n": n,
        "correct": correct,
        "accuracy": correct / n if n else 0.0,
        "rows": rows,
    }


def max_pick(native: dict, lm_patch: dict) -> dict:
    by_n = {r["id"]: r for r in native["rows"]}
    by_l = {r["id"]: r for r in lm_patch["rows"]}
    rows: list[dict] = []
    for pid in by_n:
        rn, rl = by_n[pid], by_l.get(pid, {})
        ok = bool(rn["correct"] or rl.get("correct"))
        pick = "native" if rn["correct"] else ("lm_patch" if rl.get("correct") else "none")
        rows.append(
            {
                "id": pid,
                "correct": ok,
                "picked": pick,
                "native_correct": rn["correct"],
                "lm_patch_correct": rl.get("correct", False),
            }
        )
    correct = sum(1 for r in rows if r["correct"])
    n = len(rows)
    return {
        "arm": "max_native_lm_patch",
        "n": n,
        "correct": correct,
        "accuracy": correct / n if n else 0.0,
        "note": "oracle routing ceiling — not deployable",
        "rows": rows,
    }


def disagreement_analysis(native: dict, lm_patch: dict) -> dict:
    disagree = 0
    native_only = 0
    lm_only = 0
    both = 0
    neither = 0
    for rn, rl in zip(native["rows"], lm_patch["rows"], strict=True):
        nc, lc = bool(rn["correct"]), bool(rl["correct"])
        if nc and lc:
            both += 1
        elif nc:
            native_only += 1
            disagree += 1
        elif lc:
            lm_only += 1
            disagree += 1
        else:
            neither += 1
            disagree += 1
    return {
        "disagree_count": disagree,
        "both_correct": both,
        "native_only_wins": native_only,
        "lm_patch_only_wins": lm_only,
        "both_wrong": neither,
    }


def verdict(production: dict, max_pick: dict, trad: dict | None) -> str:
    acc = production["accuracy"]
    if acc >= 0.65:
        base = "MCQ_COVERAGE_STRONG"
    elif acc >= 0.55:
        base = "MCQ_COVERAGE_MODERATE"
    else:
        base = "MCQ_COVERAGE_WEAK"
    gap = (max_pick["accuracy"] - acc) * 100.0
    if gap > 5.0:
        base += "_ROUTING_GAP"
    if trad and trad.get("accuracy", 0) > acc:
        base += "_BELOW_TRAD_LM"
    return base


def main() -> None:
    ap = argparse.ArgumentParser(description="MCQ coverage v1 — fair-1.2 test slice")
    ap.add_argument("--phase", default="test")
    ap.add_argument("--limit", type=int, default=0)
    ap.add_argument("--with-trad-lm", action="store_true")
    ap.add_argument(
        "--only-trad-lm",
        action="store_true",
        help="Load existing report and run traditional_lm_mcq arm only",
    )
    ap.add_argument(
        "--fast",
        action="store_true",
        help="Skip lm_patch_all duplicate arm (production lm_patch used for max_pick baseline)",
    )
    ap.add_argument("--out", type=Path, default=REPORTS / "mcq_coverage_test.json")
    args = ap.parse_args()

    if args.only_trad_lm:
        if not args.out.is_file():
            sys.exit(f"Missing report: {args.out} — run full battery first")
        report = json.loads(args.out.read_text(encoding="utf-8"))
        suite = load_suite()
        corpus = select_corpus(suite, report.get("phase", args.phase))
        prompts = [p for p in select_prompts(suite, report.get("phase", args.phase)) if p.get("score_type") == "mcq"]
        if args.limit > 0:
            prompts = prompts[: args.limit]
        print(f"=== {PROTOCOL} --only-trad-lm n={len(prompts)} ===\n")
        try:
            trad = eval_trad_lm(prompts, corpus)
            report.setdefault("arms", {})["traditional_lm_mcq"] = trad
            prod = report["arms"].get("production_hybrid_lm_patch") or {}
            report["summary"]["traditional_lm_mcq"] = f"{trad['correct']}/{trad['n']} ({trad['accuracy']:.1%})"
            if prod.get("accuracy") is not None:
                report["summary"]["production_vs_trad_lm_pp"] = round(
                    (prod["accuracy"] - trad["accuracy"]) * 100, 1
                )
        except Exception as exc:  # noqa: BLE001
            report.setdefault("arms", {})["traditional_lm_mcq"] = {"skipped": True, "error": str(exc)}
        args.out.write_text(json.dumps(report, indent=2, ensure_ascii=False), encoding="utf-8")
        print(f"→ {args.out}")
        return

    suite = load_suite()
    corpus = select_corpus(suite, args.phase)
    prompts = [p for p in select_prompts(suite, args.phase) if p.get("score_type") == "mcq"]
    if args.limit > 0:
        prompts = prompts[: args.limit]

    print(f"=== {PROTOCOL} n={len(prompts)} corpus={len(corpus)} ===\n")

    arms: dict[str, Any] = {}

    print("[1/5] production_hybrid_lm_patch …")
    arms["production_hybrid_lm_patch"] = eval_arm(
        prompts, corpus, decode="hybrid", mcq_decode="lm_patch"
    )

    print("[2/5] production_hybrid_max_pick …")
    arms["production_hybrid_max_pick"] = eval_arm(
        prompts, corpus, decode="hybrid", mcq_decode="max_pick"
    )

    print("[3/5] hybrid_structure_fate …")
    arms["hybrid_structure_fate"] = eval_arm(
        prompts, corpus, decode="hybrid", mcq_decode="structure_fate"
    )

    print("[4/5] native_mcq_option …")
    arms["native_mcq_option"] = eval_arm(prompts, corpus, decode="native")

    if args.fast:
        print("[5/5] lm_patch_all skipped (--fast; production serves as lm_patch arm)")
        arms["lm_patch_all"] = {
            **arms["production_hybrid_lm_patch"],
            "note": "alias of production_hybrid_lm_patch under --fast",
        }
    else:
        print("[5/5] lm_patch_all …")
        arms["lm_patch_all"] = eval_arm(prompts, corpus, decode="lm_patch")

    arms["max_native_lm_patch"] = max_pick(arms["native_mcq_option"], arms["lm_patch_all"])
    arms["disagreement"] = disagreement_analysis(
        arms["native_mcq_option"], arms["lm_patch_all"]
    )

    trad = None
    if args.with_trad_lm:
        print("[+] traditional_lm_mcq …")
        try:
            trad = eval_trad_lm(prompts, corpus)
            arms["traditional_lm_mcq"] = trad
        except Exception as exc:  # noqa: BLE001
            arms["traditional_lm_mcq"] = {"skipped": True, "error": str(exc)}

    prod = arms["production_hybrid_max_pick"]
    prod_lm = arms["production_hybrid_lm_patch"]
    mx = arms["max_native_lm_patch"]
    v = verdict(prod, mx, trad)

    report = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "protocol": PROTOCOL,
        "phase": args.phase,
        "n": len(prompts),
        "corpus_lines": len(corpus),
        "production_stack": {
            "decode": "hybrid",
            "mcq_decode": "lm_patch",
            "follow_up_decode": "native_follow_up_aware",
            "fate_ingress_routing": "quad_cycle",
        },
        "arms": arms,
        "summary": {
            "production_hybrid_lm_patch": (
                f"{prod_lm['correct']}/{prod_lm['n']} ({prod_lm['accuracy']:.1%})"
            ),
            "production_hybrid_max_pick": f"{prod['correct']}/{prod['n']} ({prod['accuracy']:.1%})",
            "max_pick_vs_lm_patch_pp": round((prod["accuracy"] - prod_lm["accuracy"]) * 100, 1),
            "hybrid_structure_fate": (
                f"{arms['hybrid_structure_fate']['correct']}/{arms['hybrid_structure_fate']['n']}"
                f" ({arms['hybrid_structure_fate']['accuracy']:.1%})"
            ),
            "native_mcq_option": (
                f"{arms['native_mcq_option']['correct']}/{arms['native_mcq_option']['n']}"
                f" ({arms['native_mcq_option']['accuracy']:.1%})"
            ),
            "lm_patch_all": (
                f"{arms['lm_patch_all']['correct']}/{arms['lm_patch_all']['n']}"
                f" ({arms['lm_patch_all']['accuracy']:.1%})"
            ),
            "max_native_lm_patch": f"{mx['correct']}/{mx['n']} ({mx['accuracy']:.1%})",
            "routing_ceiling_gap_pp": round((mx["accuracy"] - prod["accuracy"]) * 100, 1),
        },
        "verdict": v,
        "known_gaps": [
            "MCQ slice not in confidence-report until this battery",
            "max_native_lm_patch is oracle — deploy hybrid may lag",
            "structure_fate vs lm_patch tradeoff varies by item",
        ],
    }

    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(report, indent=2, ensure_ascii=False), encoding="utf-8")
    print(f"\n→ {args.out}")
    print(f"Verdict: {v}")
    for k, v in report["summary"].items():
        print(f"  {k}: {v}")


if __name__ == "__main__":
    main()
