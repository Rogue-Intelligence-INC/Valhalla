#!/usr/bin/env python3
"""Shared helpers for third-party (RAGTruth, LongBench, Needle) benchmarks."""

from __future__ import annotations

import json
import re
from pathlib import Path
from typing import Any

BUNDLED = Path(__file__).resolve().parent / "bundled"
CACHE = Path(__file__).resolve().parent / "cache"

ABSTAIN_MARKERS = (
    "open_fog_abstain",
    "cannot answer",
    "insufficient evidence",
    "not enough information",
    "do not have enough",
    "unable to answer",
    "abstain",
    "defer",
    "i don't know",
    "i do not know",
)


def load_bundled(name: str) -> dict[str, Any]:
    path = BUNDLED / name
    if not path.is_file():
        raise FileNotFoundError(f"Bundled benchmark missing: {path}")
    return json.loads(path.read_text(encoding="utf-8"))


def corpus_has_wiki_extract(corpus: list[str]) -> bool:
    """True if corpus lines contain real Wikipedia text, not URL placeholders."""
    for line in corpus:
        s = str(line).strip()
        if not s:
            continue
        if s.startswith("Reference:") and "wikipedia.org" in s:
            continue
        if len(s) >= 40:
            return True
    return False


def frames_bundle_preflight(bundle_name: str = "frames_slice25.json") -> dict[str, Any]:
    """Check FRAMES bundle wiki corpus readiness before citing scores."""
    doc = load_bundled(bundle_name)
    items = doc.get("items") or []
    ok_items = 0
    for it in items:
        corpus = it.get("corpus") or []
        if it.get("wiki_fetched") or corpus_has_wiki_extract(corpus):
            ok_items += 1
    meta_hits = int(doc.get("wiki_items_with_extract") or 0)
    n = len(items)
    viable = ok_items >= max(1, n // 2) or meta_hits >= max(1, n // 2)
    return {
        "bundle": bundle_name,
        "n": n,
        "wiki_items_with_extract_meta": meta_hits,
        "items_with_real_corpus": ok_items,
        "viable": viable,
        "tier": "A" if viable else "C",
        "fix": "./scripts/prefetch_frames_wiki.sh  # or: python3 tools/standard_benchmarks/enrich_frames_wiki.py",
    }


def split_context_to_corpus(context: str, *, max_lines: int = 0) -> list[str]:
    """Split long context into corpus lines. max_lines=0 → no artificial cap (algebra full ingest)."""
    text = context.strip()
    if not text:
        return []
    parts: list[str] = []
    for para in re.split(r"\n\s*\n+", text):
        para = " ".join(para.split())
        if not para:
            continue
        if len(para) <= 480:
            parts.append(para)
        else:
            for sent in re.split(r"(?<=[.!?])\s+", para):
                sent = sent.strip()
                if sent:
                    parts.append(sent)
    if not parts:
        parts = [text[:480]]
    if max_lines > 0:
        return parts[:max_lines]
    return parts


def is_abstain(answer: str) -> bool:
    low = answer.lower().strip()
    if len(low) < 4:
        return True
    return any(m in low for m in ABSTAIN_MARKERS)


def content_tokens(text: str, *, min_len: int = 4) -> set[str]:
    return {
        w.lower()
        for w in re.findall(r"[A-Za-z0-9][A-Za-z0-9\-']+", text)
        if len(w) >= min_len
    }


def context_support_ratio(answer: str, context: str) -> float:
    """Fraction of answer content tokens that appear in context."""
    ans_t = content_tokens(answer)
    if not ans_t:
        return 0.0
    ctx = context.lower()
    hit = sum(1 for t in ans_t if t in ctx)
    return hit / len(ans_t)


def reference_overlap(answer: str, reference: str) -> float:
    ref_t = content_tokens(reference)
    if not ref_t:
        return 0.0
    ans = answer.lower()
    hit = sum(1 for t in ref_t if t in ans)
    return hit / len(ref_t)


def score_open_answers(answer: str, answers: list[str]) -> bool:
    low = answer.lower()
    for ref in answers:
        ref = str(ref).strip()
        if not ref:
            continue
        if ref.lower() in low:
            return True
        ref_t = content_tokens(ref, min_len=3)
        if ref_t and sum(1 for t in ref_t if t in low) >= max(1, len(ref_t) // 2):
            return True
    return False


def score_ragtruth_item(answer: str, item: dict[str, Any]) -> dict[str, Any]:
    ctx = item["context"]
    ref = item.get("reference_output", "")
    support = context_support_ratio(answer, ctx)
    overlap = reference_overlap(answer, ref)
    abstain = is_abstain(answer)
    has_hall = bool(item.get("has_hallucination"))
    citation_native = "[corpus:" in answer.lower()
    # RAGTruth faithfulness = grounded in context (not summarization match).
    grounded = support >= 0.55 and not abstain
    if citation_native and support >= 0.70:
        grounded = True
    aligned = overlap >= 0.25 if ref else grounded
    faithful = grounded
    return {
        "abstain": abstain,
        "context_support": round(support, 4),
        "reference_overlap": round(overlap, 4),
        "context_grounded": grounded,
        "reference_aligned": aligned,
        "faithful": faithful,
        "native_citation": citation_native,
        "has_hallucination_label": has_hall,
    }


def format_mcq_prompt(item: dict[str, Any]) -> str:
    q = item["question"].strip()
    opts = [
        ("A", item["choice_A"]),
        ("B", item["choice_B"]),
        ("C", item["choice_C"]),
        ("D", item["choice_D"]),
    ]
    body = "\n".join(f"{letter}. {text.strip()}" for letter, text in opts)
    return f"{q}\n\n{body}\n\nAnswer with the letter only (A, B, C, or D)."


def verdict_ragtruth(faithful_rate: float, abstain_rate: float) -> str:
    if faithful_rate >= 0.75:
        return "RAGTRUTH_STRONG"
    if faithful_rate >= 0.55:
        return "RAGTRUTH_VIABLE"
    return "RAGTRUTH_DEVELOPING"


def verdict_longbench(acc: float) -> str:
    if acc >= 0.50:
        return "LONGBENCH_V2_COMPETITIVE"
    if acc >= 0.35:
        return "LONGBENCH_V2_VIABLE"
    return "LONGBENCH_V2_DEVELOPING"


def verdict_needle(recall: float) -> str:
    if recall >= 0.90:
        return "NEEDLE_STRONG"
    if recall >= 0.70:
        return "NEEDLE_VIABLE"
    return "NEEDLE_DEVELOPING"


def score_short_answer(answer: str, gold: str | list[str]) -> bool:
    """Match short-form QA (FRAMES, LoCoMo, MultiHop)."""
    refs = [gold] if isinstance(gold, str) else list(gold)
    low = answer.lower()
    for ref in refs:
        ref_s = str(ref).strip()
        if not ref_s:
            continue
        if ref_s.lower() in low:
            return True
        ref_t = content_tokens(ref_s, min_len=2)
        if not ref_t:
            continue
        hit = sum(1 for t in ref_t if t in low)
        if hit >= max(1, (len(ref_t) + 1) // 2):
            return True
    return False


def verdict_open_qa(acc: float, *, strong: float = 0.65, viable: float = 0.40) -> str:
    if acc >= strong:
        return "OPEN_QA_STRONG"
    if acc >= viable:
        return "OPEN_QA_VIABLE"
    return "OPEN_QA_DEVELOPING"


def verdict_rag_vs_lc(rag_acc: float, lc_acc: float) -> str:
    if rag_acc >= lc_acc + 0.05:
        return "STRUCTURE_RAG_BEATS_LC"
    if rag_acc >= lc_acc - 0.02:
        return "STRUCTURE_RAG_PARITY_LC"
    return "STRUCTURE_RAG_BELOW_LC"
