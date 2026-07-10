#!/usr/bin/env python3
"""Refresh bundled third-party benchmark slices.

Robust download (see download_utils.py):
  - curl + GitHub mirror chain for LoCoMo
  - HF_ENDPOINT auto (hf-mirror.com) with fallback
  - Wikipedia disk cache + REST/MediaWiki API + curl
  - --offline reuses cache only

Usage:
  .venv-llm/bin/python3 tools/standard_benchmarks/download_standard_benchmarks.py --all
  HF_ENDPOINT=https://hf-mirror.com .venv-llm/bin/python3 tools/standard_benchmarks/download_standard_benchmarks.py --all
  .venv-llm/bin/python3 tools/standard_benchmarks/download_standard_benchmarks.py --locomo-qa 30 --offline
"""

from __future__ import annotations

import argparse
import ast
import json
import random
import sys
from pathlib import Path
from typing import Any

from download_utils import (
    ensure_hf_endpoint,
    fetch_github_json,
    load_hf_dataset,
    wiki_summaries_parallel,
)

BUNDLED = Path(__file__).resolve().parent / "bundled"
FILLER = [
    "Iron rusts when exposed to moist air over time.",
    "The speed of light in vacuum is approximately 299792458 meters per second.",
    "Photosynthesis converts light energy into chemical energy in plants.",
    "Jupiter is the largest planet in the Solar System.",
    "DNA stores genetic information in living organisms.",
    "The Nile is one of the longest rivers in the world.",
    "Mercury is the closest planet to the Sun.",
    "Oxygen makes up about 21 percent of Earth's atmosphere.",
]


def _normalize_wiki_links(raw: Any) -> list[str]:
    """Parse wiki_links from HF row (str, list, or repr string) — never char-split."""
    if raw is None:
        return []
    if isinstance(raw, (list, tuple)):
        return [str(u).strip() for u in raw if u and str(u).strip()]
    if isinstance(raw, str):
        s = raw.strip()
        if not s:
            return []
        if s.startswith("http"):
            return [s]
        if s.startswith("["):
            try:
                parsed = json.loads(s)
            except json.JSONDecodeError:
                try:
                    parsed = ast.literal_eval(s)
                except (SyntaxError, ValueError):
                    parsed = None
            if isinstance(parsed, list):
                return [str(u).strip() for u in parsed if u and str(u).strip()]
        return []
    return [str(raw).strip()] if raw else []


def write_bundle(name: str, doc: dict[str, Any]) -> Path:
    BUNDLED.mkdir(parents=True, exist_ok=True)
    path = BUNDLED / name
    path.write_text(
        __import__("json").dumps(doc, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    print(f"Wrote {path} ({doc.get('n', len(doc.get('items', [])))} items)")
    return path


def download_ragtruth(n_faithful: int, n_halluc: int, out_name: str, *, offline: bool) -> None:
    ds = load_hf_dataset("wandb/RAGTruth-processed", split="test", offline=offline)
    qa = [r for r in ds if r["task_type"] == "QA" and r["quality"] == "good"]
    faithful = [
        r
        for r in qa
        if r["hallucination_labels_processed"]["evident_conflict"] == 0
        and r["hallucination_labels_processed"]["baseless_info"] == 0
    ]
    halluc = [
        r
        for r in qa
        if r["hallucination_labels_processed"]["evident_conflict"] == 1
        or r["hallucination_labels_processed"]["baseless_info"] == 1
    ]
    rng = random.Random(42)
    sel = rng.sample(faithful, min(n_faithful, len(faithful))) + rng.sample(
        halluc, min(n_halluc, len(halluc))
    )
    items = []
    for r in sel:
        h = r["hallucination_labels_processed"]
        items.append(
            {
                "id": r["id"],
                "query": r["query"],
                "context": r["context"],
                "reference_output": r["output"],
                "evident_conflict": int(h["evident_conflict"]),
                "baseless_info": int(h["baseless_info"]),
                "has_hallucination": int(h["evident_conflict"] or h["baseless_info"]),
            }
        )
    write_bundle(
        out_name,
        {
            "source": "wandb/RAGTruth-processed",
            "split": "test",
            "citation": "ACL 2024 RAGTruth",
            "n": len(items),
            "items": items,
        },
    )


def download_longbench_v2(n: int, max_context_chars: int, *, offline: bool) -> None:
    ds = load_hf_dataset("zai-org/LongBench-v2", split="train", offline=offline)
    short = [r for r in ds if r.get("length") == "short"]
    rng = random.Random(42)
    pick = rng.sample(short, min(n, len(short)))
    items = []
    for r in pick:
        ctx = r["context"]
        trunc = len(ctx) > max_context_chars
        if trunc:
            ctx = ctx[:max_context_chars]
        items.append(
            {
                "_id": r["_id"],
                "domain": r["domain"],
                "sub_domain": r["sub_domain"],
                "difficulty": r["difficulty"],
                "length": r["length"],
                "question": r["question"],
                "choice_A": r["choice_A"],
                "choice_B": r["choice_B"],
                "choice_C": r["choice_C"],
                "choice_D": r["choice_D"],
                "answer": r["answer"],
                "context": ctx,
                "context_truncated": trunc,
            }
        )
    write_bundle(
        "longbench_v2_mcq_short20.json",
        {
            "source": "THUDM/LongBench-v2",
            "split": "train",
            "filter": f"length=short, context<={max_context_chars} chars",
            "n": len(items),
            "items": items,
        },
    )


def download_frames(n: int, *, fetch_wiki: bool, offline: bool) -> None:
    ds = load_hf_dataset("google/frames-benchmark", split="test", offline=offline)
    rng = random.Random(42)
    pick = rng.sample(list(range(len(ds))), min(n, len(ds)))
    items: list[dict[str, Any]] = []
    wiki_hits = 0
    for i, idx in enumerate(pick):
        r = ds[int(idx)]
        links = _normalize_wiki_links(r.get("wiki_links"))
        if not links:
            for k in range(1, 12):
                lk = r.get(f"wikipedia_link_{k}")
                if lk:
                    links.append(lk)
        links = [str(u) for u in links if u][:8]
        corpus: list[str] = []
        if fetch_wiki and links:
            corpus = wiki_summaries_parallel(links, offline=offline)
            if corpus:
                wiki_hits += 1
        if not corpus:
            corpus = [f"Reference: {u}" for u in links]
        items.append(
            {
                "id": f"FRAMES_{idx}",
                "question": r["Prompt"],
                "answer": r["Answer"],
                "reasoning_types": r.get("reasoning_types"),
                "wiki_links": links,
                "corpus": corpus,
                "wiki_fetched": bool(corpus and not str(corpus[0]).startswith("Reference:")),
            }
        )
        if (i + 1) % 5 == 0:
            print(f"  FRAMES {i + 1}/{len(pick)} (wiki_ok={wiki_hits}) …")
    fname = "frames_slice25.json" if n <= 30 else f"frames_slice{n}.json"
    write_bundle(
        fname,
        {
            "source": "google/frames-benchmark",
            "citation": "NAACL 2025 FRAMES — multi-hop RAG",
            "baseline_no_rag": 0.408,
            "baseline_oracle": 0.729,
            "wiki_items_with_extract": wiki_hits,
            "n": len(items),
            "items": items,
        },
    )
    if fetch_wiki and wiki_hits == 0 and not offline:
        print("  WARNING: no Wikipedia extracts fetched — check network or use cached cache/wiki/")


def download_multihop(n: int, *, offline: bool) -> None:
    ds = load_hf_dataset("yixuantt/MultiHopRAG", config="MultiHopRAG", split="train", offline=offline)
    rng = random.Random(42)
    indices = list(range(len(ds)))
    rng.shuffle(indices)
    items = []
    for idx in indices:
        if len(items) >= n:
            break
        r = ds[int(idx)]
        evidence = r.get("evidence_list") or []
        corpus = []
        for ev in evidence:
            fact = str(ev.get("fact") or "").strip()
            title = str(ev.get("title") or "").strip()
            if fact:
                corpus.append(f"{title}: {fact}" if title else fact)
        if not corpus:
            continue
        items.append(
            {
                "id": f"MH_{idx}",
                "question": r["query"],
                "answer": r["answer"],
                "question_type": r.get("question_type"),
                "corpus": corpus,
            }
        )
    fname = "multihop_rag_slice25.json" if n <= 30 else f"multihop_rag_slice{n}.json"
    write_bundle(
        fname,
        {
            "source": "yixuantt/MultiHopRAG",
            "citation": "COLM 2024 MultiHop-RAG",
            "n": len(items),
            "items": items,
        },
    )


def _parse_locomo(raw: list, max_qa: int, max_conversations: int) -> list[dict[str, Any]]:
    items: list[dict[str, Any]] = []
    q_count = 0
    for ci, sample in enumerate(raw[:max_conversations]):
        conv = sample.get("conversation") or {}
        lines: list[str] = []
        for key in sorted(conv.keys()):
            if not key.startswith("session_") or key.endswith("_date_time"):
                continue
            turns = conv[key]
            if not isinstance(turns, list):
                continue
            for turn in turns:
                text = str(turn.get("text") or "").strip()
                dia = str(turn.get("dia_id") or "")
                speaker = str(turn.get("speaker") or "")
                if text:
                    prefix = f"[{dia}] " if dia else ""
                    lines.append(f"{prefix}{speaker}: {text}".strip())
        for qa in sample.get("qa") or []:
            if q_count >= max_qa:
                break
            items.append(
                {
                    "id": f"LOCO_{ci}_{q_count}",
                    "sample_id": sample.get("sample_id", ci),
                    "question": qa["question"],
                    "answer": qa["answer"],
                    "category": qa.get("category"),
                    "evidence": qa.get("evidence"),
                    "session_lines": lines,
                }
            )
            q_count += 1
        if q_count >= max_qa:
            break
    return items


def download_locomo(max_qa: int, max_conversations: int = 10, *, offline: bool = False) -> None:
    raw = fetch_github_json(
        "snap-research/locomo",
        "data/locomo10.json",
        branch="main",
        offline=offline,
    )
    items = _parse_locomo(raw, max_qa, max_conversations)
    max_lines = max((len(i["session_lines"]) for i in items), default=0)
    print(f"  LoCoMo: {len(items)} QA, max session_lines={max_lines}")
    write_bundle(
        "locomo_qa_slice30.json" if max_qa <= 40 else f"locomo_qa_slice{max_qa}.json",
        {
            "source": "snap-research/locomo locomo10.json",
            "citation": "ACL 2024 LoCoMo long conversational memory",
            "n": len(items),
            "max_session_lines": max_lines,
            "items": items,
        },
    )


def download_lara_style(n: int, *, offline: bool) -> None:
    ds = load_hf_dataset("yixuantt/MultiHopRAG", config="MultiHopRAG", split="train", offline=offline)
    rng = random.Random(7)
    indices = list(range(len(ds)))
    rng.shuffle(indices)
    items = []
    for idx in indices:
        if len(items) >= n:
            break
        r = ds[int(idx)]
        evidence = r.get("evidence_list") or []
        rag_lines = []
        long_parts = []
        for ev in evidence:
            fact = str(ev.get("fact") or "").strip()
            title = str(ev.get("title") or "").strip()
            line = f"{title}: {fact}" if title and fact else (fact or title)
            if line:
                rag_lines.append(line)
                long_parts.append(line)
        if not rag_lines:
            continue
        rng2 = random.Random(idx)
        noise = [FILLER[i % len(FILLER)] for i in range(40)]
        insert_at = rng2.randint(10, 30)
        long_ctx = noise[:insert_at] + long_parts + noise[insert_at:]
        items.append(
            {
                "id": f"LARA_{idx}",
                "question": r["query"],
                "answer": r["answer"],
                "rag_corpus": rag_lines,
                "long_context_lines": long_ctx,
            }
        )
    write_bundle(
        "lara_rag_vs_lc_slice20.json" if n <= 25 else f"lara_rag_vs_lc_slice{n}.json",
        {
            "source": "LaRA-style from MultiHopRAG",
            "citation": "Alibaba LaRA — RAG vs long-context routing",
            "n": len(items),
            "items": items,
        },
    )


def download_ruler_v1(n: int) -> None:
    rng = random.Random(99)
    items: list[dict[str, Any]] = []
    needles = [
        ("The secret vault code is ALPHA-991.", "ALPHA-991", "What is the secret vault code?"),
        ("Project Nebula launch key: ZETA-4402.", "ZETA-4402", "What is the Project Nebula launch key?"),
        ("Registration token for Site 7: KAPPA-18.", "KAPPA-18", "What is the registration token for Site 7?"),
        ("Emergency override PIN: 7734-OMEGA.", "7734-OMEGA", "What is the emergency override PIN?"),
    ]
    for i in range(n):
        needle_text, hint, question = needles[i % len(needles)]
        depth = rng.choice([5, 10, 20, 30, 40])
        multi = i % 3 == 2
        lines = [FILLER[j % len(FILLER)] for j in range(depth)]
        if multi and depth >= 10:
            n2_text, n2_hint, _ = needles[(i + 1) % len(needles)]
            lines[depth // 3] = needle_text
            lines[(2 * depth) // 3] = n2_text
            items.append(
                {
                    "id": f"RULER_{i}",
                    "task": "multi_needle",
                    "depth": depth,
                    "question": question + " Also report the second secret code if present.",
                    "hints": [hint, n2_hint],
                    "session_lines": lines,
                }
            )
        else:
            lines[depth // 2] = needle_text
            items.append(
                {
                    "id": f"RULER_{i}",
                    "task": "single_needle",
                    "depth": depth,
                    "question": question,
                    "hints": [hint],
                    "session_lines": lines,
                }
            )
    write_bundle(
        "ruler_v1_slice20.json" if n <= 25 else f"ruler_v1_slice{n}.json",
        {
            "source": "synthetic RULER-style (NIAH + multi-needle)",
            "citation": "RULER / U-NIAH long-context retrieval family",
            "n": len(items),
            "items": items,
        },
    )


def main() -> None:
    ap = argparse.ArgumentParser(description="Download bundled benchmark slices (robust fetch)")
    ap.add_argument("--all", action="store_true")
    ap.add_argument("--offline", action="store_true", help="Use cache only (HF reuse + local files)")
    ap.add_argument("--hf-endpoint", default="", help="HF endpoint (default: hf-mirror.com if unset)")
    ap.add_argument("--ragtruth-faithful", type=int, default=0)
    ap.add_argument("--ragtruth-halluc", type=int, default=0)
    ap.add_argument("--ragtruth-full-faithful", type=int, default=0)
    ap.add_argument("--ragtruth-full-halluc", type=int, default=0)
    ap.add_argument("--longbench-n", type=int, default=0)
    ap.add_argument("--max-context-chars", type=int, default=12000)
    ap.add_argument("--frames-n", type=int, default=0)
    ap.add_argument("--multihop-n", type=int, default=0)
    ap.add_argument("--locomo-qa", type=int, default=0)
    ap.add_argument("--lara-n", type=int, default=0)
    ap.add_argument("--ruler-n", type=int, default=0)
    ap.add_argument("--no-wiki-fetch", action="store_true")
    args = ap.parse_args()

    if args.all:
        args.frames_n = args.frames_n or 25
        args.multihop_n = args.multihop_n or 25
        args.locomo_qa = args.locomo_qa or 30
        args.lara_n = args.lara_n or 20
        args.ruler_n = args.ruler_n or 20
        args.ragtruth_faithful = args.ragtruth_faithful or 20
        args.ragtruth_halluc = args.ragtruth_halluc or 20
        args.ragtruth_full_faithful = args.ragtruth_full_faithful or 50
        args.ragtruth_full_halluc = args.ragtruth_full_halluc or 50
        args.longbench_n = args.longbench_n or 20

    ep = ensure_hf_endpoint(args.hf_endpoint or None)
    print(f"HF_ENDPOINT={ep} offline={args.offline}")

    BUNDLED.mkdir(parents=True, exist_ok=True)
    try:
        kw = {"offline": args.offline}
        if args.all or args.ragtruth_faithful or args.ragtruth_halluc:
            download_ragtruth(args.ragtruth_faithful, args.ragtruth_halluc, "ragtruth_qa_test40.json", **kw)
        if args.all or args.ragtruth_full_faithful or args.ragtruth_full_halluc:
            download_ragtruth(
                args.ragtruth_full_faithful or 50,
                args.ragtruth_full_halluc or 50,
                "ragtruth_qa_test100.json",
                **kw,
            )
        if args.all or args.longbench_n:
            download_longbench_v2(args.longbench_n or 20, args.max_context_chars, **kw)
        if args.frames_n:
            download_frames(args.frames_n, fetch_wiki=not args.no_wiki_fetch, **kw)
        if args.multihop_n:
            download_multihop(args.multihop_n, **kw)
        if args.locomo_qa:
            download_locomo(args.locomo_qa, offline=args.offline)
        if args.lara_n:
            download_lara_style(args.lara_n, **kw)
        if args.ruler_n:
            download_ruler_v1(args.ruler_n)
    except ImportError:
        sys.exit("pip install datasets pyarrow (use project .venv-llm)")


if __name__ == "__main__":
    main()
