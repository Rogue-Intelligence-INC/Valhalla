#!/usr/bin/env python3
"""New (ValhallaBase structure context) vs traditional (LM prompt injection) context test.

Stops early with NO_DIFFERENCE verdict if context swap + accuracy show no meaningful gap.

Usage:
  python3 tools/valhalla_inference/test_context_arch_compare.py --limit 31
"""

from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(REPO / "tools/fair_benchmark"))

from metrics import score_answer  # noqa: E402
from run_fair_benchmark import load_suite, select_prompts  # noqa: E402
from run_potential_analysis import classify_slices  # noqa: E402
from run_strict_incubation_experiment import strict_user_text  # noqa: E402
from test_context_performance import context_block, corpus_hits  # noqa: E402
from valhalla_base import ValhallaBase  # noqa: E402

REPORTS = REPO / "reports/valhalla_inference"
MODELS_HF = REPO / "models/hf"
DEFAULT_LM = Path(
    "/home/jouly/.cache/huggingface/hub/models--Qwen--Qwen2.5-0.5B-Instruct/snapshots"
)


def _weights_ok(d: Path) -> bool:
    st = d / "model.safetensors"
    if st.is_file() and st.stat().st_size > 1024:
        return True
    idx = d / "model.safetensors.index.json"
    return idx.is_file()


def _hub_cache_snap(repo_id: str) -> Path | None:
    slug = "models--" + repo_id.replace("/", "--")
    root = Path.home() / ".cache/huggingface/hub" / slug / "snapshots"
    if not root.is_dir():
        return None
    for snap in sorted(root.iterdir(), reverse=True):
        if _weights_ok(snap):
            return snap
    return None


def _modelscope_cache_snap(repo_id: str) -> Path | None:
    """ModelScope hub layout: ~/.cache/modelscope/hub/models/<org>/<name>/"""
    ms_id = repo_id  # same id for mapped models in our download script
    parts = ms_id.split("/")
    if len(parts) != 2:
        return None
    root = Path.home() / ".cache/modelscope/hub" / "models" / parts[0] / parts[1]
    if root.is_dir() and _weights_ok(root):
        return root
    # LLM-Research alias for Phi-3
    alt = Path.home() / ".cache/modelscope/hub" / "models" / "LLM-Research" / parts[1]
    if alt.is_dir() and _weights_ok(alt):
        return alt
    return None


def resolve_lm_path(hint: str) -> Path:
    """Resolve HF model dir: local path, models/hf/, or hub cache."""
    p = Path(hint)
    if p.is_dir() and _weights_ok(p):
        return p
    if "/" in hint or hint.startswith("models/"):
        repo = hint.split("/")[-1] if hint.startswith("models/") else hint
        local = MODELS_HF / repo.split("/")[-1]
        if local.is_dir() and _weights_ok(local):
            return local
        cached = _hub_cache_snap(hint if "/" in hint else repo)
        if cached is not None:
            return cached
        ms = _modelscope_cache_snap(hint if "/" in hint else repo)
        if ms is not None:
            return ms
    if DEFAULT_LM.is_dir():
        for snap in sorted(DEFAULT_LM.iterdir(), reverse=True):
            if _weights_ok(snap):
                return snap
    raise FileNotFoundError(f"No HF weights for {hint} (try models/hf/ or huggingface-cli download)")


SWAP_CASES = [
    (
        "What is the capital of Australia?",
        ["The capital of Australia is Canberra.", "Australia is in Oceania."],
        ["The capital of France is Paris.", "France is in Western Europe."],
        "canberra",
        "paris",
    ),
    (
        "At what temperature does water boil at standard pressure?",
        [
            "Water boils at 100 degrees Celsius at standard atmospheric pressure.",
        ],
        ["Water freezes at 0 degrees Celsius at standard pressure."],
        "100",
        "0",
    ),
]


def trad_generate(lm, question: str, ctx_lines: list[str]) -> str:
    q = context_block(ctx_lines) + question
    return lm.generate([q], max_tokens=128)[0].text.strip()


def new_generate(base: ValhallaBase, question: str, ctx_lines: list[str]) -> str:
    base.clear_context()
    doc = base.run(
        [{"id": "q", "text": question, "score_type": "open"}],
        context=ctx_lines,
    )
    return (doc["rows"][0]["answer"] if doc.get("rows") else "").strip()


def hint_in_answer(hint: str, answer: str) -> bool:
    return hint.lower() in answer.lower()


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--limit", type=int, default=31)
    ap.add_argument("--lm", default="Qwen/Qwen2.5-0.5B-Instruct")
    ap.add_argument("--min-delta-acc", type=float, default=0.05,
                    help="Min accuracy gap to claim difference")
    ap.add_argument("--min-swap-gap", type=int, default=1,
                    help="Min extra swap successes for new vs trad to claim difference")
    args = ap.parse_args()

    from loader import ValhallaLLM

    lm_path = resolve_lm_path(args.lm)

    suite = load_suite()
    corpus = suite["corpus"]["ingest_clean"]
    open_prompts = [p for p in select_prompts(suite, "test") if p.get("score_type") == "open"]
    slices = classify_slices(open_prompts, corpus)
    retrieval_ids = slices["open_retrieval"]
    tasks = [p for p in open_prompts if p["id"] in retrieval_ids][: args.limit]

    print(f"=== Context arch compare (open_retrieval n={len(tasks)}) ===\n")
    base = ValhallaBase(decode="native")
    lm = ValhallaLLM(lm_path, engine="transformers")

    new_rows: list[dict] = []
    trad_rows: list[dict] = []

    for i, p in enumerate(tasks):
        hits = corpus_hits(p, corpus)
        q = strict_user_text(p)
        na = new_generate(base, q, hits)
        ta = trad_generate(lm, q, hits)
        ns = score_answer(na, p)
        ts = score_answer(ta, p)
        new_rows.append({"id": p["id"], "correct": ns["correct"], "answer": na[:200]})
        trad_rows.append({"id": p["id"], "correct": ts["correct"], "answer": ta[:200]})
        if (i + 1) % 10 == 0 or i + 1 == len(tasks):
            print(f"  [{i+1}/{len(tasks)}] …", flush=True)

    new_ok = sum(1 for r in new_rows if r["correct"])
    trad_ok = sum(1 for r in trad_rows if r["correct"])
    new_acc = new_ok / max(len(tasks), 1)
    trad_acc = trad_ok / max(len(tasks), 1)
    acc_delta = new_acc - trad_acc

    swap_results: list[dict] = []
    new_swap_ok = trad_swap_ok = 0
    swap_agree = 0
    for q, ctx_a, ctx_b, hint_a, hint_b in SWAP_CASES:
        na = new_generate(base, q, ctx_a)
        nb = new_generate(base, q, ctx_b)
        ta = trad_generate(lm, q, ctx_a)
        tb = trad_generate(lm, q, ctx_b)
        n_ok = hint_in_answer(hint_a, na) and hint_in_answer(hint_b, nb)
        t_ok = hint_in_answer(hint_a, ta) and hint_in_answer(hint_b, tb)
        new_swap_ok += int(n_ok)
        trad_swap_ok += int(t_ok)
        same_behavior = (
            hint_in_answer(hint_a, na) == hint_in_answer(hint_a, ta)
            and hint_in_answer(hint_b, nb) == hint_in_answer(hint_b, tb)
        )
        swap_agree += int(same_behavior)
        swap_results.append(
            {
                "question": q,
                "new_a": na[:120],
                "new_b": nb[:120],
                "trad_a": ta[:120],
                "trad_b": tb[:120],
                "new_swap_ok": n_ok,
                "trad_swap_ok": t_ok,
                "same_swap_behavior": same_behavior,
            }
        )

    answer_match = sum(
        1 for n, t in zip(new_rows, trad_rows)
        if n["correct"] == t["correct"] and n["answer"][:80] == t["answer"][:80]
    )

    has_acc_diff = abs(acc_delta) >= args.min_delta_acc
    has_swap_diff = (new_swap_ok - trad_swap_ok) >= args.min_swap_gap or (
        trad_swap_ok - new_swap_ok
    ) >= args.min_swap_gap
    has_behavior_diff = swap_agree < len(SWAP_CASES)

    if not has_acc_diff and not has_swap_diff and not has_behavior_diff:
        verdict = "NO_CONTEXT_DIFFERENCE"
    elif new_acc > trad_acc + args.min_delta_acc:
        verdict = "NEW_ARCH_BETTER"
    elif trad_acc > new_acc + args.min_delta_acc:
        verdict = "TRADITIONAL_BETTER"
    else:
        verdict = "MIXED_SMALL_GAP"

    summary = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "protocol": "context-arch-compare-v1",
        "open_retrieval_n": len(tasks),
        "new_architecture": {
            "name": "ValhallaBase native (structure context)",
            "correct": new_ok,
            "accuracy": new_acc,
        },
        "traditional": {
            "name": "LM prompt injection (explicit corpus block)",
            "correct": trad_ok,
            "accuracy": trad_acc,
        },
        "accuracy_delta_new_minus_trad": acc_delta,
        "context_swap": swap_results,
        "new_swap_ok": new_swap_ok,
        "trad_swap_ok": trad_swap_ok,
        "swap_behavior_agree": swap_agree,
        "answer_prefix_match_rate": answer_match / max(len(tasks), 1),
        "verdict": verdict,
    }

    print(f"\nNew (ValhallaBase):  {new_ok}/{len(tasks)} ({100*new_acc:.1f}%)")
    print(f"Trad (LM + prompt):  {trad_ok}/{len(tasks)} ({100*trad_acc:.1f}%)")
    print(f"Δ accuracy: {acc_delta:+.1%}")
    print(f"Context swap: new {new_swap_ok}/{len(SWAP_CASES)}  trad {trad_swap_ok}/{len(SWAP_CASES)}")
    print(f"Swap behavior agree: {swap_agree}/{len(SWAP_CASES)}")
    print(f"Verdict: {verdict}")

    REPORTS.mkdir(parents=True, exist_ok=True)
    out = REPORTS / "context_arch_compare.json"
    out.write_text(json.dumps(summary, indent=2, ensure_ascii=False), encoding="utf-8")
    print(f"→ {out}")

    if verdict == "NO_CONTEXT_DIFFERENCE":
        sys.exit(0)


if __name__ == "__main__":
    main()
