"""Lightweight guided decode helpers (MCQ / JSON) without external deps."""

from __future__ import annotations

import re
from dataclasses import dataclass

MCQ_PATTERN = re.compile(r"\bANSWER:\s*([A-D])\b", re.IGNORECASE)
LETTER_ONLY = re.compile(r"^[A-D]$", re.IGNORECASE)


@dataclass
class GuidedMCQResult:
    raw: str
    letter: str | None
    guided: bool


def extract_mcq_letter(text: str) -> str | None:
    m = MCQ_PATTERN.search(text)
    if m:
        return m.group(1).upper()
    tail = text.strip().splitlines()[-1].strip() if text.strip() else ""
    if LETTER_ONLY.match(tail):
        return tail.upper()
    for ch in reversed(text):
        if ch.upper() in "ABCD":
            return ch.upper()
    return None


def guided_mcq_postprocess(raw: str) -> GuidedMCQResult:
    letter = extract_mcq_letter(raw)
    return GuidedMCQResult(raw=raw, letter=letter, guided=letter is not None)


def mcq_guided_prompt_suffix() -> str:
    return (
        "\n\nThink briefly, then end with exactly one line: "
        "ANSWER: X (X must be A, B, C, or D)."
    )
