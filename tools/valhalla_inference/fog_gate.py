"""Fate Output Gate — fixed v1 and E7 algebraic v2.

Algebra (v2, from E7 integers r=7, δ=5, dim=133):
  τ(k) = (r + δ) / (dim + δ + k)  = 12 / (138 + k)
  Δ_req(s₁) = s₁ · δ / (dim + r)       = s₁ · 5/140 = s₁/28

emit ⟺ cycles ≥ C_min ∧ s₁ ≥ τ(k) ∧ (s₁ − s₂) ≥ Δ_req(s₁) ∧ ¬native_reject
"""

from __future__ import annotations

from dataclasses import asdict, dataclass
from fractions import Fraction
from typing import Any

# E7 algebraic integers (no fitted floats in definition)
FOG_R = 7
FOG_DELTA = 5
FOG_DIM = 133
FOG_BASE = FOG_DIM + FOG_DELTA  # 138 = base_α

DEFAULT_MIN_SCORE = 0.12
DEFAULT_MIN_MARGIN = 0.03
DEFAULT_MIN_STRUCTURE_CYCLES = 2


@dataclass
class FogDecisionDoc:
    decision: str
    structure_ready: bool
    evidence_ready: bool
    top_score: float
    margin: float
    tau: float
    delta_required: float
    reason: str
    formula: str = "v1_fixed"

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


def tau_algebraic(keyword_count: int) -> Fraction:
    """τ(k) = (r+δ)/(dim+δ+k)"""
    return Fraction(FOG_R + FOG_DELTA, FOG_BASE + max(keyword_count, 0))


def delta_required_algebraic(top_score: float) -> Fraction:
    """Δ_req = s₁ · δ/(dim+r)"""
    return Fraction(FOG_DELTA, FOG_DIM + FOG_R) * Fraction(str(top_score)).limit_denominator(10_000)


def fog_decide_algebraic(
    *,
    top_score: float,
    second_score: float,
    keyword_count: int,
    incubation_cycles: int,
    decode_path: str,
    answer: str,
    min_structure_cycles: int = DEFAULT_MIN_STRUCTURE_CYCLES,
    trigger_present: bool = True,
) -> FogDecisionDoc:
    margin = top_score - second_score
    tau_f = tau_algebraic(keyword_count)
    tau = float(tau_f)
    delta_req_f = delta_required_algebraic(top_score) if top_score > 0 else Fraction(0)
    delta_req = float(delta_req_f)
    formula = f"τ=12/(138+{keyword_count}), Δ≥s₁·5/140"

    structure_ready = incubation_cycles >= min_structure_cycles
    if not structure_ready:
        return FogDecisionDoc(
            "defer", False, False, top_score, margin, tau, delta_req,
            f"cycles={incubation_cycles}<{min_structure_cycles}", formula,
        )
    if not trigger_present:
        return FogDecisionDoc(
            "defer", True, False, top_score, margin, tau, delta_req, "no_trigger", formula,
        )

    native_reject = "reject" in decode_path or answer.strip().startswith("No confident match")
    if native_reject:
        return FogDecisionDoc(
            "abstain", True, False, top_score, margin, tau, delta_req, "native_reject", formula,
        )
    if top_score < tau:
        return FogDecisionDoc(
            "abstain", True, False, top_score, margin, tau, delta_req,
            f"s1={top_score:.4f}<tau={tau:.4f}", formula,
        )
    if margin < delta_req:
        return FogDecisionDoc(
            "abstain", True, False, top_score, margin, tau, delta_req,
            f"margin={margin:.4f}<delta_req={delta_req:.4f}", formula,
        )
    return FogDecisionDoc(
        "emit", True, True, top_score, margin, tau, delta_req, "algebraic_ready", formula,
    )


def fog_decide_v1_fixed(
    *,
    top_score: float,
    margin: float,
    keyword_count: int,
    incubation_cycles: int,
    decode_path: str,
    answer: str,
    min_structure_cycles: int = DEFAULT_MIN_STRUCTURE_CYCLES,
    min_score: float = DEFAULT_MIN_SCORE,
    min_margin: float = DEFAULT_MIN_MARGIN,
    trigger_present: bool = True,
) -> FogDecisionDoc:
    structure_ready = incubation_cycles >= min_structure_cycles
    if not structure_ready:
        return FogDecisionDoc(
            "defer", False, False, top_score, margin, min_score, min_margin,
            f"cycles={incubation_cycles}<{min_structure_cycles}", "v1_fixed",
        )
    if not trigger_present:
        return FogDecisionDoc(
            "defer", True, False, top_score, margin, min_score, min_margin, "no_trigger", "v1_fixed",
        )
    native_reject = "reject" in decode_path or answer.strip().startswith("No confident match")
    if native_reject:
        return FogDecisionDoc(
            "abstain", True, False, top_score, margin, min_score, min_margin, "native_reject", "v1_fixed",
        )
    if top_score < min_score:
        return FogDecisionDoc(
            "abstain", True, False, top_score, margin, min_score, min_margin, "low_top_score", "v1_fixed",
        )
    if margin < min_margin:
        return FogDecisionDoc(
            "abstain", True, False, top_score, margin, min_score, min_margin, "low_margin", "v1_fixed",
        )
    return FogDecisionDoc(
        "emit", True, True, top_score, margin, min_score, min_margin, "ready", "v1_fixed",
    )


def fog_from_base_row(
    row: dict,
    tel: dict,
    *,
    mode: str = "algebraic",
) -> FogDecisionDoc:
    """Use native fog_* from ValhallaBase row when present."""
    fe = row.get("fog_evidence") or {}
    fd = row.get("fog_decision")
    cycles = int(tel.get("incubation_cycles") or 2)
    s1 = float(fe.get("top_score") or 0)
    s2 = float(fe.get("second_score") or 0)
    k = int(fe.get("keyword_count") or 0)

    if mode == "native" and fd:
        return FogDecisionDoc(
            fd.get("decision", "emit"),
            bool(fd.get("structure_ready")),
            bool(fd.get("evidence_ready")),
            s1,
            s1 - s2,
            float(fd.get("tau") or 0),
            float(fd.get("delta_required") or 0),
            str(fd.get("reason", "")),
            "v2_rust_algebraic",
        )

    return fog_decide_algebraic(
        top_score=s1,
        second_score=s2,
        keyword_count=k,
        incubation_cycles=cycles,
        decode_path=str(row.get("decode_path", "")),
        answer=str(row.get("answer", "")),
    )
