#!/usr/bin/env python3
"""Context performance test for Valhalla fused models.

Measures how fusion models use (or ignore) fair corpus context:
  - open_retrieval: answer should come from ingest_clean corpus
  - open_generation: no corpus support (parametric knowledge only)
  - ablation: bare prompt vs explicit corpus block in prompt

Usage:
  python3 tools/valhalla_inference/test_context_performance.py
  python3 tools/valhalla_inference/test_context_performance.py --limit 20
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(REPO / "tools/fair_benchmark"))

from loader import ValhallaLLM  # noqa: E402
from metrics import score_answer  # noqa: E402
from run_fair_benchmark import load_suite, select_prompts  # noqa: E402
from run_potential_analysis import classify_slices  # noqa: E402
from run_strict_incubation_experiment import strict_user_text  # noqa: E402

DEFAULT_MODEL = REPO / "models/hf/Qwen2.5-0.5B-Instruct-Valhalla-FreeMultimodal-Fusion-v2-test-s0.10"
FUSED_BIN = REPO / "target/release/valhalla_fused_lm"
REPORTS = REPO / "reports/valhalla_inference"
V2_FULL = REPO / "reports/natural_multimodal_potential/fused_v2_full_test.json"


def corpus_hits(prompt: dict, corpus: list[str], k: int = 5) -> list[str]:
    ref = str(prompt.get("reference_answer", "")).lower()
    kws = [str(x).lower() for x in (prompt.get("keywords") or []) if x]
    scored: list[tuple[int, str]] = []
    for line in corpus:
        low = line.lower()
        score = 0
        if ref and len(ref) >= 2 and ref in low:
            score += 10
        score += sum(1 for kw in kws if kw and kw in low)
        if score > 0:
            scored.append((score, line))
    scored.sort(key=lambda x: -x[0])
    return [ln for _, ln in scored[:k]]


def context_block(lines: list[str]) -> str:
    if not lines:
        return ""
    body = "\n".join(f"- {ln}" for ln in lines)
    return f"Reference facts (use if relevant):\n{body}\n\n"


def analyze_existing(path: Path, prompts: list[dict], corpus: list[str]) -> dict:
    if not path.is_file():
        return {}
    rows = json.loads(path.read_text())["fused_rows"]
    slices = classify_slices(prompts, corpus)
    by_id = {p["id"]: p for p in prompts}

    def slice_acc(name: str) -> dict:
        ids = slices[name]
        rs = [r for r in rows if r["id"] in ids]
        c = sum(1 for r in rs if r.get("correct"))
        return {"n": len(rs), "correct": c, "acc": c / len(rs) if rs else 0.0}

    failures = []
    for r in rows:
        pid = r["id"]
        if pid not in slices["open_retrieval"]:
            continue
        if not r.get("correct"):
            p = by_id[pid]
            failures.append(
                {
                    "id": pid,
                    "reference": p.get("reference_answer"),
                    "output": (r.get("answer") or "")[:200],
                }
            )

    return {
        "source": str(path),
        "slices": {
            k: slice_acc(k)
            for k in (
                "corpus_supported",
                "open_retrieval",
                "open_generation",
                "mcq_routing",
                "numeric_compute",
            )
        },
        "open_retrieval_failures": failures[:8],
    }


def live_ablation(model_dir: Path, prompts: list[dict], corpus: list[str], limit: int) -> dict:
    slices = classify_slices(prompts, corpus)
    ids = sorted(slices["open_retrieval"])[:limit]
    subset = [p for p in prompts if p["id"] in ids]

    llm = ValhallaLLM(model_dir, engine="transformers")
    llm.load()

    bare_rows, ctx_rows = [], []
    for p in subset:
        q = strict_user_text(p)
        hits = corpus_hits(p, corpus)
        bare_out = llm.generate([q], max_tokens=128)[0].text
        ctx_out = llm.generate([context_block(hits) + q], max_tokens=128)[0].text
        bare_sc = score_answer(bare_out, p)
        ctx_sc = score_answer(ctx_out, p)
        bare_rows.append({"id": p["id"], "correct": bare_sc["correct"], "output": bare_out[:300]})
        ctx_rows.append({"id": p["id"], "correct": ctx_sc["correct"], "output": ctx_out[:300]})

    def acc(rows: list[dict]) -> dict:
        c = sum(1 for r in rows if r["correct"])
        return {"n": len(rows), "correct": c, "acc": c / len(rows) if rows else 0}

    return {
        "model": str(model_dir),
        "open_retrieval_subset": acc(bare_rows),
        "with_explicit_corpus_in_prompt": acc(ctx_rows),
        "delta_pp": round(
            100
            * (
                acc(ctx_rows)["acc"] - acc(bare_rows)["acc"]
            ),
            1,
        ),
        "samples": [
            {
                "id": p["id"],
                "reference": p.get("reference_answer"),
                "corpus_hits": corpus_hits(p, corpus, 3),
                "bare_correct": bare_rows[i]["correct"],
                "ctx_correct": ctx_rows[i]["correct"],
                "bare_out": bare_rows[i]["output"][:180],
                "ctx_out": ctx_rows[i]["output"][:180],
            }
            for i, p in enumerate(subset)
        ],
    }


def run_fused_test(model_dir: Path, phase: str) -> dict:
    if not FUSED_BIN.is_file():
        subprocess.run(
            ["cargo", "build", "-p", "hub-f64", "--release", "--bin", "valhalla_fused_lm"],
            check=True,
            cwd=REPO,
            env={**dict(__import__("os").environ), "RUSTFLAGS": "-L /opt/cuda/lib64"},
        )
    out = REPORTS / f"context_fused_{phase}.json"
    subprocess.run(
        [str(FUSED_BIN), "--phase", phase, "--model", str(model_dir), "--out", str(out)],
        check=True,
        cwd=REPO,
    )
    return {"path": str(out), "summary": json.loads(out.read_text())["summary"]}


def guided_base_recommendation(existing: dict, ablation: dict) -> dict:
    slices = existing.get("slices") or {}
    open_ret = slices.get("open_retrieval", {})
    open_gen = slices.get("open_generation", {})
    mcq = slices.get("mcq_routing", {})
    delta = ablation.get("delta_pp", 0)

    needs_orchestrator = []
    skip_guided = []

    # Context / retrieval
    if open_ret.get("acc", 0) >= 0.85:
        skip_guided.append("open_retrieval already ≥85% on baked fused — no RAG wrapper required for deploy")
    else:
        needs_orchestrator.append("open_retrieval weak — consider Valhalla memory track or corpus-in-prompt orchestrator")

    if abs(delta) < 3:
        skip_guided.append(f"explicit corpus-in-prompt Δ={delta:+.1f}pp — model not context-starved at inference")
    elif delta > 5:
        needs_orchestrator.append(f"explicit context helps +{delta:.1f}pp — lightweight prompt-RAG orchestrator worth it")
    else:
        needs_orchestrator.append("marginal context lift — optional prompt-RAG only")

    # MCQ
    if mcq.get("acc", 0) < 0.65:
        needs_orchestrator.append("MCQ <65% — keep valhalla_fused_lm logprob path; optional grammar-guided decode for vLLM")
    else:
        skip_guided.append("MCQ acceptable with baked logprob batch")

    # open_generation vs retrieval gap
    gap = (open_ret.get("acc", 0) - open_gen.get("acc", 0)) * 100
    if gap > 15:
        needs_orchestrator.append(f"retrieval-only wedge +{gap:.0f}pp over generation — corpus ingress is load-bearing")
    else:
        skip_guided.append("open_generation nearly matches retrieval — parametric+bake sufficient for many opens")

    verdict = (
        "BUILD_LIGHT_ORCHESTRATOR"
        if len(needs_orchestrator) > len(skip_guided)
        else "SKIP_FULL_GUIDED_BASE"
    )

    return {
        "verdict": verdict,
        "summary": (
            "Pre-baked fused LM + valhalla_fused_lm MCQ logprob is enough for MVP serving; "
            "full custom vLLM guided substrate NOT required unless you need runtime hybrid routing or live corpus ingress."
        ),
        "needs": needs_orchestrator,
        "skip": skip_guided,
        "recommended_stack": {
            "serving": "transformers or vLLM on baked HF weights",
            "mcq": "valhalla_fused_lm logprob_batch (not vLLM sample)",
            "open_with_corpus": "optional prompt-RAG wrapper if delta_pp > 5",
            "full_hybrid": "Valhalla orchestrator (Tier B memory + patch router) — separate from vLLM plugin",
        },
    }


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--model", type=Path, default=DEFAULT_MODEL)
    ap.add_argument("--phase", default="test")
    ap.add_argument("--limit", type=int, default=15, help="open_retrieval live ablation size")
    ap.add_argument("--skip-live", action="store_true")
    ap.add_argument("--skip-fused-rerun", action="store_true")
    args = ap.parse_args()

    suite = load_suite()
    prompts = select_prompts(suite, args.phase)
    corpus = suite["corpus"]["ingest_clean"]

    report: dict = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "model": str(args.model),
        "phase": args.phase,
    }

    print("=== Existing fused test (slice breakdown) ===")
    existing = analyze_existing(V2_FULL, prompts, corpus)
    report["existing_full_test"] = existing
    if existing:
        for k, v in existing["slices"].items():
            print(f"  {k:20} {v['correct']}/{v['n']} ({100*v['acc']:.1f}%)")

    if not args.skip_fused_rerun:
        print("\n=== valhalla_fused_lm (uses corpus snapshot at eval) ===")
        fused = run_fused_test(args.model, args.phase)
        report["fused_rerun"] = fused
        print(json.dumps(fused["summary"], indent=2))

    if not args.skip_live:
        print(f"\n=== Live ablation (open_retrieval n={args.limit}) ===")
        ablation = live_ablation(args.model, prompts, corpus, args.limit)
        report["live_ablation"] = ablation
        print(
            f"  bare: {ablation['open_retrieval_subset']['correct']}/{ablation['open_retrieval_subset']['n']} "
            f"({100*ablation['open_retrieval_subset']['acc']:.1f}%)"
        )
        print(
            f"  +ctx: {ablation['with_explicit_corpus_in_prompt']['correct']}/{ablation['with_explicit_corpus_in_prompt']['n']} "
            f"({100*ablation['with_explicit_corpus_in_prompt']['acc']:.1f}%) "
            f"Δ={ablation['delta_pp']:+.1f}pp"
        )

    rec = guided_base_recommendation(existing, report.get("live_ablation", {}))
    report["guided_base_analysis"] = rec
    print("\n=== Guided base analysis ===")
    print(f"Verdict: {rec['verdict']}")
    for line in rec["needs"]:
        print(f"  + {line}")
    for line in rec["skip"]:
        print(f"  − {line}")

    REPORTS.mkdir(parents=True, exist_ok=True)
    out = REPORTS / "context_performance.json"
    out.write_text(json.dumps(report, indent=2, ensure_ascii=False), encoding="utf-8")
    print(f"\n→ {out}")


if __name__ == "__main__":
    main()
