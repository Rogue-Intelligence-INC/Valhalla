#!/usr/bin/env python3
"""Corpus ↔ eval contamination checks for Valhalla Fair Benchmark."""

from __future__ import annotations

import re
from typing import Any


def _norm(s: str) -> str:
    return re.sub(r"\s+", " ", s.strip().lower())


def _legacy_leak_patterns() -> dict[str, list[str]]:
    """Known answer-leaking paraphrases in old medium/large corpus (numeric/word)."""
    return {
        "GSM_01": ["seventy two", "totaling seventy two", "half as many clips in may as in april"],
        "GSM_02": ["janet sells duck eggs after eating and baking"],
        "GSM_04": ["twenty four apples gave six bought ten", "ending with twenty eight"],
        "MATH_01": ["seventeen times twenty three", "three hundred ninety one"],
        "MATH_02": ["one hundred forty four divided by twelve equals twelve"],
        "MATH_03": ["square root of eighty one is nine"],
        "MATH_04": ["x plus fifteen equals forty two then x equals twenty seven"],
    }


def line_leaks_prompt(line: str, prompt: dict) -> list[str]:
    """
    Return leak reasons. Only flags **eval cheating**, not legitimate domain facts.

    - numeric/word: block worked solutions & legacy paraphrases
    - mcq/open: corpus *should* contain facts (Canberra, 1945, etc.) — not flagged
    """
    reasons: list[str] = []
    low = _norm(line)
    pid = prompt.get("id", "")
    stype = prompt.get("score_type", "")

    for pat in _legacy_leak_patterns().get(pid, []):
        if pat in low:
            reasons.append(f"legacy_pattern:{pat[:32]}")

    if stype != "numeric":
        return reasons

    ref = str(prompt.get("reference_answer", "")).strip()
    if not ref:
        return reasons

    # Numeric: block lines that state the benchmark answer as a worked result
    plain = line.replace(",", "")
    if ref in plain:
        solution_markers = (
            "equals",
            "equal to",
            "times",
            "multiplied",
            "divided",
            "square root",
            "total",
            "altogether",
            "ending with",
            "结果是",
            "等于",
            "总共",
        )
        if any(m in low for m in solution_markers):
            reasons.append(f"numeric_worked_answer:{ref}")
        # Short lines dominated by the answer number (word-problem leak)
        elif len(line.split()) <= 18 and re.search(r"\d", line):
            if any(m in low for m in ("total", "altogether", "ending", "sold", "bought", "gave")):
                reasons.append(f"numeric_word_leak:{ref}")

    return reasons


def corpus_contamination_report(corpus: list[str], prompts: list[dict]) -> dict[str, Any]:
    line_hits: list[dict] = []
    prompt_hits: dict[str, list[str]] = {}

    for li, line in enumerate(corpus):
        for p in prompts:
            reasons = line_leaks_prompt(line, p)
            if reasons:
                line_hits.append(
                    {
                        "line_index": li,
                        "line_preview": line[:80],
                        "prompt_id": p["id"],
                        "reasons": reasons,
                    }
                )
                prompt_hits.setdefault(p["id"], []).append(f"L{li}")

    contaminated_lines = {h["line_index"] for h in line_hits}
    numeric_prompts = [p for p in prompts if p.get("score_type") == "numeric"]
    numeric_hits = {
        p["id"]
        for p in numeric_prompts
        if p["id"] in prompt_hits
    }

    return {
        "corpus_lines": len(corpus),
        "prompt_count": len(prompts),
        "contaminated_line_count": len(contaminated_lines),
        "contaminated_prompt_count": len(prompt_hits),
        "numeric_prompt_leak_count": len(numeric_hits),
        "clean": len(contaminated_lines) == 0,
        "line_hits": line_hits,
        "prompt_hits": prompt_hits,
    }


def filter_clean_corpus(corpus: list[str], prompts: list[dict]) -> tuple[list[str], list[dict]]:
    bad: set[int] = set()
    removed: list[dict] = []
    for li, line in enumerate(corpus):
        for p in prompts:
            reasons = line_leaks_prompt(line, p)
            if reasons:
                bad.add(li)
                removed.append({"line_index": li, "line_preview": line[:80], "prompt_id": p["id"], "reasons": reasons})
                break
    clean = [line for i, line in enumerate(corpus) if i not in bad]
    return clean, removed


def filter_padding_lines(corpus: list[str]) -> list[str]:
    """Remove generic padding lines with no domain content."""
    return [
        line
        for line in corpus
        if not line.startswith("Incubation knowledge line ")
    ]
