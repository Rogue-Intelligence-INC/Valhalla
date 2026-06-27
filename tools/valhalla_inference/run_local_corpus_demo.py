#!/usr/bin/env python3
"""Local corpus ingest demo — zero-weight teach + generation eval on real files.

Loads markdown/text from a local directory, ingests into ValhallaBase,
runs a short QA curriculum, then compares generation before vs after.

Protocol: local-corpus-demo-v1

Usage:
  python3 tools/valhalla_inference/run_local_corpus_demo.py
  python3 tools/valhalla_inference/run_local_corpus_demo.py --corpus-dir valhalla-unified-core/docs
"""

from __future__ import annotations

import argparse
import copy
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

REPO = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

from run_scale_benchmark import corpus_for_prompt  # noqa: E402
from valhalla_base import ValhallaBase  # noqa: E402

PROTOCOL = "local-corpus-demo-v1"
REPORTS = REPO / "reports/valhalla_inference"

DEFAULT_CORPUS_DIRS = (
    REPO / "valhalla-unified-core/docs",
    REPO / "docs",
)

PRIORITY_FILES = (
    "VALHALLA_BASE_PLATFORM.md",
    "FATE_INGRESS_ROUTING.md",
    "CONTEXT_FLEXIBILITY_MAX_GOAL.md",
    "EN_ENGINEERING_WHITEPAPER.md",
    "INTRODUCTION.md",
)

# Holdout probes — answers must appear in local docs, not in fair_benchmark ingest_clean
LOCAL_EVAL: list[dict] = [
    {
        "id": "L01",
        "text": "What is the primary ValhallaBase protocol name?",
        "reference_answer": "valhalla-base-v1",
        "keywords": ["valhalla-base-v1", "valhalla base v1"],
    },
    {
        "id": "L02",
        "text": "What fate ingress routing mode is the production default?",
        "reference_answer": "quad_cycle",
        "keywords": ["quad_cycle", "quad cycle"],
    },
    {
        "id": "L03",
        "text": "Which four meta-systems does Hub route corpus signals to?",
        "reference_answer": "BlackHole MeshBrain MultiNova BaseForce",
        "keywords": ["blackhole", "meshbrain", "multinova", "baseforce"],
    },
    {
        "id": "L04",
        "text": "For open questions in hybrid decode, which path is used by default?",
        "reference_answer": "native",
        "keywords": ["native", "structure retrieval"],
    },
    {
        "id": "L05",
        "text": "What follow-up decode mode is recommended for production?",
        "reference_answer": "native_follow_up_aware",
        "keywords": ["native_follow_up_aware", "follow-up-aware"],
    },
    {
        "id": "L06",
        "text": "What product line name is abbreviated VUC?",
        "reference_answer": "Valhalla Unified Core",
        "keywords": ["valhalla unified core", "vuc"],
    },
    {
        "id": "L07",
        "text": "What JSON interface does valhalla_base binary use?",
        "reference_answer": "stdin stdout",
        "keywords": ["stdin", "stdout", "json"],
    },
    {
        "id": "L08",
        "text": "What is the default LM model family mentioned for the decode head?",
        "reference_answer": "Qwen2.5",
        "keywords": ["qwen", "qwen2.5"],
    },
]


def make_base(*, feedback: bool = False) -> ValhallaBase:
    return ValhallaBase(
        decode="hybrid",
        follow_up_decode="native_follow_up_aware",
        fate_ingress_routing="quad_cycle",
        fate_qa_feedback=feedback,
        incubation_cycles=2,
        max_new=160,
    )


def load_corpus_from_dirs(dirs: list[Path], *, max_lines: int = 80) -> tuple[list[str], dict]:
    patterns = ("*.md", "*.txt")
    seen: set[str] = set()
    lines: list[str] = []
    sources: dict[str, int] = {}

    def ingest_file(fp: Path) -> None:
        nonlocal lines
        if "archive" in fp.parts or "patents" in fp.parts:
            return
        try:
            text = fp.read_text(encoding="utf-8", errors="ignore")
        except OSError:
            return
        for raw in text.splitlines():
            if len(lines) >= max_lines:
                return
            s = raw.strip()
            s = re.sub(r"^#+\s*", "", s)
            s = re.sub(r"^\*\*|\*\*$", "", s)
            s = re.sub(r"^[-*]\s*", "", s)
            s = re.sub(r"`+", "", s)
            s = s.strip()
            if len(s) < 24 or len(s) > 220:
                continue
            if s.startswith("|") or s.startswith("---"):
                continue
            if "http://" in s or "https://" in s:
                continue
            key = s.lower()
            if key in seen:
                continue
            seen.add(key)
            lines.append(s)
            sources[str(fp.relative_to(REPO))] = sources.get(str(fp.relative_to(REPO)), 0) + 1

    collected: list[Path] = []
    for d in dirs:
        if not d.is_dir():
            continue
        for pat in patterns:
            collected.extend(sorted(d.rglob(pat)))

    by_name: dict[str, Path] = {}
    for fp in collected:
        by_name.setdefault(fp.name, fp)

    for name in PRIORITY_FILES:
        fp = by_name.get(name)
        if fp:
            ingest_file(fp)
    for fp in collected:
        if len(lines) >= max_lines:
            break
        if fp.name in PRIORITY_FILES:
            continue
        ingest_file(fp)

    return lines, {"files": sources, "line_count": len(lines)}


def score_open(answer: str, prompt: dict) -> bool:
    low = answer.lower()
    ref = str(prompt.get("reference_answer", "")).lower()
    if ref and ref in low:
        return True
    kws = [str(k).lower() for k in (prompt.get("keywords") or []) if k]
    hits = sum(1 for k in kws if k in low)
    return hits >= max(1, len(kws) // 2)


def ask(base: ValhallaBase, prompt: dict, *, append_only: bool = False) -> str:
    doc = base.run(
        [{
            "id": prompt["id"],
            "text": prompt["text"],
            "score_type": "open",
            "reference_answer": prompt.get("reference_answer", ""),
            "keywords": prompt.get("keywords") or [],
        }],
        append_only=append_only,
    )
    return (doc.get("rows") or [{}])[0].get("answer", "")


def eval_arm(
    prompts: list[dict],
    corpus: list[str],
    *,
    mode: str,
    trained_session: dict | None = None,
) -> dict:
    rows: list[dict] = []
    correct = 0
    for p in prompts:
        if mode == "no_local_corpus":
            base = make_base()
            base.set_context([
                "The capital of France is Paris.",
                "Water boils at 100 degrees Celsius.",
            ])
            ans = ask(base, p)
        elif mode == "cold_targeted":
            base = make_base()
            base.set_context(corpus_for_prompt(p, corpus))
            ans = ask(base, p)
        elif mode == "transfer_after_teach":
            assert trained_session is not None
            base = make_base()
            sess = copy.deepcopy(trained_session)
            sess["corpus"] = []
            sess["fingerprint"] = ""
            sess["question_history"] = []
            base.session.session_state = sess
            for ln in corpus_for_prompt(p, corpus):
                base.append_context(ln)
            ans = ask(base, p, append_only=True)
        else:
            raise ValueError(mode)
        ok = score_open(ans, p)
        correct += int(ok)
        rows.append({"id": p["id"], "correct": ok, "answer": ans[:200]})
    n = len(prompts)
    return {"mode": mode, "n": n, "correct": correct, "accuracy": correct / max(n, 1), "rows": rows}


def build_curriculum(corpus: list[str], n_rounds: int = 12) -> list[tuple[str, str, str]]:
    """Simple teach pairs from corpus lines (first N lines)."""
    rounds: list[tuple[str, str, str]] = []
    templates = [
        ("What fact is stated about: {snippet}?", "{snippet}", "{hint}"),
    ]
    for i, line in enumerate(corpus[:n_rounds]):
        snippet = line[:60].rstrip()
        hint = line.split()[0].lower() if line.split() else "fact"
        if len(hint) < 3:
            hint = "valhalla"
        q = f"According to the ingested documentation, what does this line say: {snippet}?"
        rounds.append((line, q, hint))
    return rounds


def run_teach(base: ValhallaBase, curriculum: list[tuple[str, str, str]]) -> dict:
    hits = 0
    for i, (fact, question, hint) in enumerate(curriculum):
        base.append_context(fact)
        doc = base.run(
            [{
                "id": f"teach_{i}",
                "text": question,
                "score_type": "open",
                "reference_answer": hint,
                "keywords": [hint],
            }],
            append_only=base.session.session_state is not None,
        )
        hits += int((doc.get("rows") or [{}])[0].get("correct", False))
    return {
        "rounds": len(curriculum),
        "train_accuracy": hits / max(len(curriculum), 1),
    }


def main() -> None:
    ap = argparse.ArgumentParser(description="Local corpus ValhallaBase ingest demo")
    ap.add_argument(
        "--corpus-dir",
        action="append",
        default=[],
        help="Directory to load .md/.txt (repeatable)",
    )
    ap.add_argument("--max-lines", type=int, default=60)
    ap.add_argument("--teach-rounds", type=int, default=12)
    args = ap.parse_args()

    dirs = [Path(p).resolve() for p in args.corpus_dir] if args.corpus_dir else list(DEFAULT_CORPUS_DIRS)
    corpus, meta = load_corpus_from_dirs(dirs, max_lines=args.max_lines)

    print(f"=== {PROTOCOL} ===")
    print(f"  corpus_dirs: {[str(d) for d in dirs if d.is_dir()]}")
    print(f"  ingested_lines: {len(corpus)}  source_files: {len(meta.get('files', {}))}")
    print(f"  eval_questions: {len(LOCAL_EVAL)}\n")

    if len(corpus) < 5:
        print("ERROR: too few corpus lines; check --corpus-dir", file=sys.stderr)
        sys.exit(1)

    print("[1] Before ingest (wrong context — no local docs) …")
    before = eval_arm(LOCAL_EVAL, corpus, mode="no_local_corpus")
    print(f"  accuracy: {before['correct']}/{before['n']} ({before['accuracy']:.1%})\n")

    print("[2] After targeted ingest (cold RAG, no teach) …")
    cold = eval_arm(LOCAL_EVAL, corpus, mode="cold_targeted")
    print(f"  accuracy: {cold['correct']}/{cold['n']} ({cold['accuracy']:.1%})\n")

    print(f"[3] Teach session ({args.teach_rounds} rounds, fate_qa_feedback=true) …")
    base = make_base(feedback=True)
    base.set_context(corpus)
    curriculum = build_curriculum(corpus, args.teach_rounds)
    teach = run_teach(base, curriculum)
    session = base.session.session_state
    print(f"  train_accuracy: {teach['train_accuracy']:.1%}\n")

    print("[4] After teach — transfer eval (fresh targeted retrieval) …")
    after = eval_arm(
        LOCAL_EVAL,
        corpus,
        mode="transfer_after_teach",
        trained_session=session,
    )
    print(f"  accuracy: {after['correct']}/{after['n']} ({after['accuracy']:.1%})\n")

    print("--- Sample generations (cold vs after teach) ---")
    for p in LOCAL_EVAL[:4]:
        cold_row = next(r for r in cold["rows"] if r["id"] == p["id"])
        after_row = next(r for r in after["rows"] if r["id"] == p["id"])
        print(f"\nQ: {p['text']}")
        print(f"  ref: {p['reference_answer']}")
        print(f"  cold:  {cold_row['answer'][:140]}")
        print(f"  teach: {after_row['answer'][:140]}")

    report: dict[str, Any] = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "protocol": PROTOCOL,
        "corpus_meta": meta,
        "corpus_sample": corpus[:8],
        "eval": LOCAL_EVAL,
        "before_no_local_corpus": before,
        "cold_targeted_ingest": cold,
        "teach_session": teach,
        "after_transfer_eval": after,
        "lifts_pp": {
            "ingest_vs_none_pp": (cold["accuracy"] - before["accuracy"]) * 100.0,
            "transfer_vs_cold_pp": (after["accuracy"] - cold["accuracy"]) * 100.0,
        },
        "zero_weight_updates": True,
    }

    REPORTS.mkdir(parents=True, exist_ok=True)
    out = REPORTS / "local_corpus_demo_test.json"
    out.write_text(json.dumps(report, indent=2, ensure_ascii=False), encoding="utf-8")
    print(f"\n→ {out}")
    print(
        f"\nSummary: none={before['accuracy']:.1%} → cold={cold['accuracy']:.1%} → "
        f"transfer={after['accuracy']:.1%}  "
        f"(ingest +{report['lifts_pp']['ingest_vs_none_pp']:.1f}pp  "
        f"teach +{report['lifts_pp']['transfer_vs_cold_pp']:.1f}pp)"
    )


if __name__ == "__main__":
    main()
