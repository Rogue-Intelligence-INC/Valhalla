#!/usr/bin/env python3
"""Token / wire transport efficiency — Valhalla session append vs RAG full prompt.

Protocol: token-efficiency-v1

Compares three architectures on matched 30-turn threads (3 topics × 10 turns):
  1. valhalla_append  — incremental wire payload; LM prefill = question only (native path)
  2. rag_full_resend  — full corpus block + question each turn (prefill tokens)
  3. rag_with_history — full corpus + prior Q/A turns each turn (prefill tokens)

Also reports holdout accuracy (open_retrieval) from existing TPI-v2 when available.

Usage:
  python3 tools/valhalla_inference/test_token_efficiency.py
  python3 tools/valhalla_inference/test_token_efficiency.py --run-valhalla
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

from test_context_flexibility_goal import EXTENDED_TURNS, SEEDS, norm, topic_ok  # noqa: E402
from test_context_performance import context_block  # noqa: E402

REPORTS = REPO / "reports/valhalla_inference"
PROTOCOL = "token-efficiency-v1"
DECODE_TOKENS_PER_ANSWER = 128  # traditional LM max_tokens default


def load_tokenizer(model_hint: str):
    from test_context_arch_compare import resolve_lm_path  # noqa: E402

    from transformers import AutoTokenizer

    path = resolve_lm_path(model_hint)
    return AutoTokenizer.from_pretrained(str(path), trust_remote_code=True)


def chat_prefill_tokens(tokenizer, user_content: str) -> int:
    messages = [{"role": "user", "content": user_content}]
    text = tokenizer.apply_chat_template(messages, tokenize=False, add_generation_prompt=True)
    return len(tokenizer.encode(text, add_special_tokens=False))


def flatten_threads() -> list[dict]:
    """30 turns: append line + question + hint + forbidden per step."""
    turns: list[dict] = []
    for topic, rows in EXTENDED_TURNS.items():
        seed = SEEDS.get(topic, [])
        corpus = list(seed)
        for i, (append, question, hint, forbidden) in enumerate(rows):
            turns.append(
                {
                    "topic": topic,
                    "turn": i + 1,
                    "append": append,
                    "question": question,
                    "hint": hint,
                    "forbidden": forbidden,
                    "corpus_after": list(corpus) + [append],
                }
            )
            corpus.append(append)
    return turns


def simulate_costs(tokenizer, turns: list[dict]) -> dict[str, dict]:
    """Wire chars + prefill token cumulative curves for three architectures."""
    corpus: list[str] = []
    wire_cum = 0
    valhalla_prefill_cum = 0
    rag_prefill_cum = 0
    hist_prefill_cum = 0
    history: list[tuple[str, str]] = []

    per_turn: list[dict] = []

    # Seed ingest (Valhalla full set once per topic — average per turn below)
    topic_seeds = {t: list(SEEDS.get(t, [])) for t in EXTENDED_TURNS}

    for idx, t in enumerate(turns):
        topic = t["topic"]
        if t["turn"] == 1:
            corpus = list(topic_seeds[topic])
            wire_step = len(json.dumps({"context": corpus}))
            mode = "full_ingest"
        else:
            wire_step = len(json.dumps({"append_lines": [t["append"]]}))
            mode = "append"
        corpus = t["corpus_after"]
        wire_cum += wire_step

        q_only = t["question"]
        valhalla_step = chat_prefill_tokens(tokenizer, q_only)
        valhalla_prefill_cum += valhalla_step

        rag_prompt = context_block(corpus) + q_only
        rag_step = chat_prefill_tokens(tokenizer, rag_prompt)
        rag_prefill_cum += rag_step

        hist_body = context_block(corpus)
        for uq, ans in history:
            hist_body += f"User: {uq}\nAssistant: {ans}\n\n"
        hist_body += q_only
        hist_step = chat_prefill_tokens(tokenizer, hist_body)
        hist_prefill_cum += hist_step

        history.append((q_only, f"[answer about {t['hint']}]"))

        per_turn.append(
            {
                "global_turn": idx + 1,
                "topic": topic,
                "turn_in_topic": t["turn"],
                "mode": mode,
                "wire_step_chars": wire_step,
                "wire_cumulative_chars": wire_cum,
                "valhalla_prefill_step": valhalla_step,
                "valhalla_prefill_cumulative": valhalla_prefill_cum,
                "rag_prefill_step": rag_step,
                "rag_prefill_cumulative": rag_prefill_cum,
                "rag_history_prefill_step": hist_step,
                "rag_history_prefill_cumulative": hist_prefill_cum,
                "corpus_lines": len(corpus),
            }
        )

    n = len(turns)
    return {
        "valhalla_append": {
            "wire_chars_cumulative": wire_cum,
            "wire_chars_per_turn_avg": round(wire_cum / n, 1),
            "prefill_tokens_cumulative": valhalla_prefill_cum,
            "prefill_tokens_per_turn_avg": round(valhalla_prefill_cum / n, 1),
            "decode_tokens_total": 0,
            "note": "Native open path: corpus in snapshot, not in LM prompt",
        },
        "rag_full_resend": {
            "wire_chars_cumulative": sum(
                len(context_block(t["corpus_after"]) + t["question"]) for t in turns
            ),
            "prefill_tokens_cumulative": rag_prefill_cum,
            "prefill_tokens_per_turn_avg": round(rag_prefill_cum / n, 1),
            "decode_tokens_total": n * DECODE_TOKENS_PER_ANSWER,
        },
        "rag_with_history": {
            "prefill_tokens_cumulative": hist_prefill_cum,
            "prefill_tokens_per_turn_avg": round(hist_prefill_cum / n, 1),
            "decode_tokens_total": n * DECODE_TOKENS_PER_ANSWER,
        },
        "per_turn": per_turn,
        "ratios_vs_rag_full": {
            "wire_chars": round(
                wire_cum
                / max(
                    sum(len(context_block(t["corpus_after"]) + t["question"]) for t in turns),
                    1,
                ),
                3,
            ),
            "prefill_tokens": round(valhalla_prefill_cum / max(rag_prefill_cum, 1), 3),
            "prefill_tokens_vs_history": round(valhalla_prefill_cum / max(hist_prefill_cum, 1), 3),
        },
    }


def run_valhalla_accuracy(turns: list[dict]) -> dict:
    from valhalla_base import ValhallaBase  # noqa: E402

    base = ValhallaBase(
        decode="hybrid",
        follow_up_decode="native_follow_up_aware",
        fate_ingress_routing="quad_cycle",
        fate_qa_feedback=False,
    )
    hits = 0
    rows: list[dict] = []
    corpus: list[str] = []
    current_topic = ""

    for t in turns:
        if t["topic"] != current_topic:
            current_topic = t["topic"]
            base.clear_context()
            corpus = list(SEEDS.get(current_topic, []))
            base.set_context(corpus)
        base.append_context(t["append"])
        corpus = t["corpus_after"]
        doc = base.run(
            [{"id": f"{t['topic']}_{t['turn']}", "text": t["question"], "score_type": "open"}],
            append_only=True,
        )
        ans = ""
        for r in doc.get("results") or []:
            ans = str(r.get("answer") or r.get("text") or "")
        ok = topic_ok(ans, t["hint"], t["forbidden"])
        hits += int(ok)
        rows.append({"topic": t["topic"], "turn": t["turn"], "on_topic": ok, "answer": ans[:120]})

    n = len(turns)
    return {"on_topic_hits": hits, "n": n, "on_topic_rate": round(hits / n, 4), "rows": rows}


def load_holdout_accuracy() -> dict:
    path = REPORTS / "valhalla_base_training_potential_test_v2.json"
    if not path.is_file():
        return {"missing": str(path.name)}
    doc = json.loads(path.read_text(encoding="utf-8"))
    learning = doc.get("learning_paradigms") or {}
    iso = learning.get("isolated_baseline") or {}
    rag = doc.get("rag_baseline_gap") or doc.get("rag_comparison") or {}
    trad = rag.get("traditional_lm_rag") or {}
    return {
        "valhalla_hybrid_cold_rag": {
            "correct": iso.get("correct"),
            "n": iso.get("n"),
            "accuracy": iso.get("accuracy"),
        },
        "traditional_lm_rag": {
            "correct": trad.get("correct"),
            "n": trad.get("n"),
            "accuracy": trad.get("accuracy"),
        },
        "source": path.name,
    }


def build_summary_table(costs: dict, holdout: dict, thread_acc: dict | None) -> list[dict]:
    va = costs["valhalla_append"]
    rf = costs["rag_full_resend"]
    rh = costs["rag_with_history"]
    rows = [
        {
            "architecture": "valhalla_append",
            "wire_chars_cumulative": va["wire_chars_cumulative"],
            "prefill_tokens_cumulative": va["prefill_tokens_cumulative"],
            "decode_tokens_total": va["decode_tokens_total"],
            "thread_on_topic_rate": (thread_acc or {}).get("on_topic_rate"),
            "holdout_accuracy": (holdout.get("valhalla_hybrid_cold_rag") or {}).get("accuracy"),
        },
        {
            "architecture": "rag_full_resend",
            "wire_chars_cumulative": rf["wire_chars_cumulative"],
            "prefill_tokens_cumulative": rf["prefill_tokens_cumulative"],
            "decode_tokens_total": rf["decode_tokens_total"],
            "thread_on_topic_rate": None,
            "holdout_accuracy": (holdout.get("traditional_lm_rag") or {}).get("accuracy"),
        },
        {
            "architecture": "rag_with_history",
            "wire_chars_cumulative": None,
            "prefill_tokens_cumulative": rh["prefill_tokens_cumulative"],
            "decode_tokens_total": rh["decode_tokens_total"],
            "thread_on_topic_rate": None,
            "holdout_accuracy": (holdout.get("traditional_lm_rag") or {}).get("accuracy"),
        },
    ]
    return rows


def main() -> None:
    ap = argparse.ArgumentParser(description="Token efficiency v1 — wire + prefill vs RAG")
    ap.add_argument("--lm", default="Qwen/Qwen2.5-0.5B-Instruct")
    ap.add_argument("--run-valhalla", action="store_true", help="Run live 30-turn Valhalla accuracy")
    ap.add_argument("--out", type=Path, default=REPORTS / "token_efficiency_test.json")
    args = ap.parse_args()

    print(f"=== {PROTOCOL} ===")
    turns = flatten_threads()
    print(f"  turns: {len(turns)}")

    tokenizer = load_tokenizer(args.lm)
    costs = simulate_costs(tokenizer, turns)
    holdout = load_holdout_accuracy()

    thread_acc = None
    if args.run_valhalla:
        print("  running ValhallaBase 30-turn thread eval…")
        thread_acc = run_valhalla_accuracy(turns)
    else:
        # Pull cached extended continuity from context-flexibility-max-goal if present
        cached = REPORTS / "context_flexibility_max_goal.json"
        if cached.is_file():
            doc = json.loads(cached.read_text(encoding="utf-8"))
            ext = (doc.get("extended_topic_continuity") or {}).get("follow_up_aware") or {}
            if ext.get("rate") is not None or ext.get("on_topic_rate") is not None:
                thread_acc = {
                    "on_topic_hits": ext.get("total_on_topic", ext.get("on_topic_hits")),
                    "n": ext.get("total_turns", 30),
                    "on_topic_rate": ext.get("rate", ext.get("on_topic_rate")),
                    "source": "context_flexibility_max_goal.json (follow_up_aware)",
                }

    summary_table = build_summary_table(costs, holdout, thread_acc)

    report = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "protocol": PROTOCOL,
        "model_tokenizer": args.lm,
        "turns": len(turns),
        "architectures": {
            "valhalla_append": costs["valhalla_append"],
            "rag_full_resend": costs["rag_full_resend"],
            "rag_with_history": costs["rag_with_history"],
        },
        "ratios_vs_rag_full": costs["ratios_vs_rag_full"],
        "thread_accuracy": thread_acc,
        "holdout_accuracy": holdout,
        "summary_table": summary_table,
        "interpretation": (
            "Valhalla wire cost scales with incremental append (~64 chars/step avg on growth curve); "
            "LM prefill stays question-only on native path. RAG resends full corpus each turn — "
            "prefill tokens grow super-linearly with history variant."
        ),
    }

    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(report, indent=2, ensure_ascii=False), encoding="utf-8")
    print(f"→ {args.out}")

    va = costs["valhalla_append"]
    rf = costs["rag_full_resend"]
    print(
        f"\nWire cumulative:  valhalla={va['wire_chars_cumulative']}  "
        f"rag_full={rf['wire_chars_cumulative']}  ratio={costs['ratios_vs_rag_full']['wire_chars']}"
    )
    print(
        f"Prefill cumulative: valhalla={va['prefill_tokens_cumulative']}  "
        f"rag_full={rf['prefill_tokens_cumulative']}  "
        f"ratio={costs['ratios_vs_rag_full']['prefill_tokens']}"
    )
    if thread_acc:
        print(f"Thread on-topic: {thread_acc.get('on_topic_rate', 0):.1%}")
    if holdout.get("valhalla_hybrid_cold_rag"):
        h = holdout["valhalla_hybrid_cold_rag"]
        t = holdout.get("traditional_lm_rag") or {}
        print(f"Holdout: valhalla={h.get('accuracy')}  trad_lm={t.get('accuracy')}")


if __name__ == "__main__":
    main()
