"""Fate Output Gate (FOG) v1 — when to emit vs abstain vs defer."""

from __future__ import annotations

from dataclasses import asdict, dataclass
from typing import Any

DEFAULT_MIN_SCORE = 0.12
DEFAULT_MIN_MARGIN = 0.03
DEFAULT_MIN_STRUCTURE_CYCLES = 2


@dataclass
class FogDecision:
    decision: str  # emit | abstain | defer
    structure_ready: bool
    evidence_ready: bool
    top_score: float
    margin: float
    reason: str

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


def score_lines_for_question(question: str, corpus: list[str]) -> list[tuple[str, float]]:
    """Keyword overlap ranking (aligned with fair corpus_for_prompt heuristic)."""
    q = question.lower()
    kws = [w for w in q.split() if len(w) >= 3 or w.isdigit()]
    scored: list[tuple[str, float]] = []
    for line in corpus:
        low = line.lower()
        kw_hits = sum(1 for k in kws if k in low)
        score = kw_hits * 0.12
        for k in kws:
            if len(k) >= 4 and k in low:
                score += 0.08
        if score > 0:
            scored.append((line, score))
    scored.sort(key=lambda x: -x[1])
    return scored


def fog_decide(
    *,
    question: str,
    corpus: list[str],
    decode_path: str,
    answer: str,
    incubation_cycles: int,
    min_structure_cycles: int = DEFAULT_MIN_STRUCTURE_CYCLES,
    min_score: float = DEFAULT_MIN_SCORE,
    min_margin: float = DEFAULT_MIN_MARGIN,
    trigger_present: bool = True,
) -> FogDecision:
    structure_ready = incubation_cycles >= min_structure_cycles
    if not structure_ready:
        return FogDecision(
            "defer",
            False,
            False,
            0.0,
            0.0,
            f"incubation_cycles={incubation_cycles}<{min_structure_cycles}",
        )

    if not trigger_present:
        return FogDecision("defer", True, False, 0.0, 0.0, "no_trigger")

    native_reject = (
        "reject" in decode_path
        or answer.strip().startswith("No confident match")
    )
    ranked = score_lines_for_question(question, corpus)
    top_score = ranked[0][1] if ranked else 0.0
    margin = (
        ranked[0][1] - ranked[1][1] if len(ranked) >= 2 else top_score
    )

    if native_reject:
        return FogDecision(
            "abstain", True, False, top_score, margin, "native_reject"
        )
    if top_score < min_score:
        return FogDecision(
            "abstain", True, False, top_score, margin, "low_top_score"
        )
    if margin < min_margin:
        return FogDecision(
            "abstain", True, False, top_score, margin, "low_margin"
        )

    return FogDecision(
        "emit", True, True, top_score, margin, "ready"
    )
