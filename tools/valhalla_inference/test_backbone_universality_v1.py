#!/usr/bin/env python3
"""Backbone universality — Valhalla structure vs trad_lm RAG across LM families.

Protocol: backbone-universality-v1

Valhalla hybrid (native open) is **LM-backbone independent** for Tier B retrieval.
This battery checks whether trad_lm RAG baselines differ by backbone while
Valhalla holds fair-holdout / external-holdout accuracy.

Arms per question batch:
  - valhalla_hybrid (fixed production stack)
  - trad_lm_rag per --backbone (transformers generate + full context prompt)

Note: Rust lm_patch / Candle path remains **Qwen2-only**; not included here.

Usage:
  # After: download_backbone_models.py (Gemma + Phi-3)
  .venv-llm/bin/python3 tools/valhalla_inference/test_backbone_universality_v1.py --fast \\
    --backbone Qwen/Qwen2.5-0.5B-Instruct \\
    --backbone google/gemma-2-2b-it \\
    --backbone microsoft/Phi-3-mini-4k-instruct
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
from run_fair_benchmark import load_suite, select_prompts  # noqa: E402
from run_potential_analysis import classify_slices  # noqa: E402
from run_scale_benchmark import corpus_for_prompt, targeted_context  # noqa: E402
from test_context_arch_compare import resolve_lm_path, trad_generate  # noqa: E402
from valhalla_base import ValhallaBase  # noqa: E402

PROTOCOL = "backbone-universality-v1"
REPORTS = REPO / "reports/valhalla_inference"
DEFAULT_BACKBONES = [
    "Qwen/Qwen2.5-0.5B-Instruct",
    "google/gemma-2-2b-it",
    "microsoft/Phi-3-mini-4k-instruct",
]


def production_base() -> ValhallaBase:
    return ValhallaBase(
        decode="hybrid",
        follow_up_decode="native_follow_up_aware",
        fate_ingress_routing="quad_cycle",
        fate_qa_feedback=False,
        incubation_cycles=2,
        max_new=128,
    )


def load_holdout(fast: bool, slice_name: str | None = None) -> tuple[list[str], list[dict], list[dict]]:
    suite = load_suite()
    corpus = suite["corpus"]["ingest_clean"]
    prompts = select_prompts(suite, "test")
    open_only = [p for p in prompts if p.get("score_type") == "open"]
    if slice_name == "open_retrieval":
        ids = classify_slices(prompts, corpus)["open_retrieval"]
        open_only = [p for p in open_only if p["id"] in ids]
    if fast:
        open_only = open_only[:8]
    return corpus, open_only, prompts


def load_external(limit: int) -> tuple[list[str], list[dict]]:
    path = REPO / "tools/valhalla_model_bridge/external_domain_prompts.json"
    doc = json.loads(path.read_text(encoding="utf-8"))
    corpus = doc["corpus"]
    prompts = doc["prompts"]
    if limit > 0:
        prompts = prompts[:limit]
    return corpus, prompts


def run_valhalla(corpus: list[str], prompts: list[dict], external: bool) -> dict[str, Any]:
    rows: list[dict] = []
    correct = 0
    for p in prompts:
        base = production_base()
        if external:
            ctx = targeted_context(p, corpus)
        else:
            ctx = corpus_for_prompt(p, corpus)
        base.set_context(ctx)
        q = {
            "id": p["id"],
            "text": p["text"],
            "score_type": p.get("score_type", "open"),
            "reference_answer": p.get("reference_answer", ""),
            "keywords": p.get("keywords") or [],
        }
        out = base.run([q])
        row = (out.get("rows") or [{}])[0]
        ans = str(row.get("answer") or "")
        sc = score_answer(ans, p)
        ok = sc.get("correct", False)
        correct += int(ok)
        rows.append(
            {
                "id": p["id"],
                "correct": ok,
                "answer": ans[:200],
                "decode_path": row.get("decode_path"),
            }
        )
    n = len(prompts)
    return {"n": n, "correct": correct, "accuracy": correct / n if n else 0.0, "rows": rows}


def run_trad_lm(lm, corpus: list[str], prompts: list[dict], external: bool) -> dict[str, Any]:
    rows: list[dict] = []
    correct = 0
    for p in prompts:
        if external:
            ctx = targeted_context(p, corpus)
        else:
            ctx = corpus_for_prompt(p, corpus)
        ans = trad_generate(lm, p["text"], ctx)
        sc = score_answer(ans, p)
        ok = sc.get("correct", False)
        correct += int(ok)
        rows.append({"id": p["id"], "correct": ok, "answer": ans[:200]})
    n = len(prompts)
    return {"n": n, "correct": correct, "accuracy": correct / n if n else 0.0, "rows": rows}


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--suite", choices=["fair_open", "external"], default="fair_open")
    ap.add_argument(
        "--slice",
        choices=["open_retrieval"],
        help="fair_open only: corpus-supported open subset (~31Q, aligns with holdout)",
    )
    ap.add_argument("--fast", action="store_true")
    ap.add_argument("--limit", type=int, default=0)
    ap.add_argument(
        "--backbone",
        action="append",
        dest="backbones",
        help="HF model id or models/hf path (repeatable)",
    )
    ap.add_argument("--skip-valhalla", action="store_true")
    args = ap.parse_args()

    backbones = args.backbones or DEFAULT_BACKBONES

    if args.suite == "external":
        corpus, prompts = load_external(args.limit or (6 if args.fast else 0))
        external = True
    else:
        corpus, prompts, _ = load_holdout(args.fast, args.slice)
        external = False

    suite_label = args.suite if not args.slice else f"{args.suite}/{args.slice}"

    print(f"=== {PROTOCOL} suite={suite_label} n={len(prompts)} backbones={backbones} ===\n")

    report: dict[str, Any] = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "protocol": PROTOCOL,
        "suite": suite_label,
        "slice": args.slice,
        "n": len(prompts),
        "backbones": backbones,
        "arms": {},
        "notes": [
            "Valhalla hybrid open path is backbone-independent (Tier B native).",
            "trad_lm uses transformers AutoModel per backbone.",
            "lm_patch (Candle Qwen2) not in this protocol.",
        ],
    }

    if not args.skip_valhalla:
        print("Running valhalla_hybrid …")
        v = run_valhalla(corpus, prompts, external)
        report["arms"]["valhalla_hybrid"] = {
            "accuracy": v["accuracy"],
            "correct": v["correct"],
            "n": v["n"],
        }
        print(f"  valhalla_hybrid: {v['correct']}/{v['n']} ({100*v['accuracy']:.1f}%)\n")

    for bb in backbones:
        print(f"Running trad_lm RAG ({bb}) …")
        try:
            path = resolve_lm_path(bb)
        except FileNotFoundError as e:
            print(f"  SKIP: {e}")
            report["arms"][f"trad_lm/{bb}"] = {"error": str(e)}
            continue
        from loader import ValhallaLLM

        lm = ValhallaLLM(path, engine="transformers")
        lm.load()
        t = run_trad_lm(lm, corpus, prompts, external)
        key = f"trad_lm/{bb}"
        report["arms"][key] = {
            "model_path": str(path),
            "accuracy": t["accuracy"],
            "correct": t["correct"],
            "n": t["n"],
        }
        print(f"  {key}: {t['correct']}/{t['n']} ({100*t['accuracy']:.1f}%)\n")

    if "valhalla_hybrid" in report["arms"]:
        v_acc = report["arms"]["valhalla_hybrid"]["accuracy"]
        for k, arm in report["arms"].items():
            if k.startswith("trad_lm/") and "accuracy" in arm:
                arm["delta_pp_vs_valhalla"] = round((v_acc - arm["accuracy"]) * 100, 1)

    out_name = "backbone_universality_test.json"
    if args.slice == "open_retrieval":
        out_name = "backbone_universality_open_retrieval_full.json"
    out = REPORTS / out_name
    out.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"Wrote {out}")


if __name__ == "__main__":
    main()
