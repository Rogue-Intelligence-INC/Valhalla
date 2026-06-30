#!/usr/bin/env python3
"""Scaling-law analysis for Valhalla vs trad_lm RAG."""

from __future__ import annotations

import math
from typing import Any


def log10_params(params_b: float) -> float:
    return math.log10(max(params_b, 0.1))


def linear_fit(xs: list[float], ys: list[float]) -> dict[str, float]:
    """OLS y = a + b*x."""
    n = len(xs)
    if n < 2:
        return {"intercept": ys[0] if ys else 0.0, "slope": 0.0, "r2": 0.0}
    mx = sum(xs) / n
    my = sum(ys) / n
    num = sum((x - mx) * (y - my) for x, y in zip(xs, ys))
    den = sum((x - mx) ** 2 for x in xs) or 1e-9
    slope = num / den
    intercept = my - slope * mx
    ss_tot = sum((y - my) ** 2 for y in ys) or 1e-9
    ss_res = sum((y - (intercept + slope * x)) ** 2 for x, y in zip(xs, ys))
    r2 = 1.0 - ss_res / ss_tot
    return {"intercept": intercept, "slope": slope, "r2": max(0.0, min(1.0, r2))}


def analyze_scaling(report: dict[str, Any]) -> dict[str, Any]:
    """Derive structure premium, trad scaling slope, training substrate claims."""
    vh_open = report.get("valhalla_hybrid_open") or {}
    vh_acc = float(vh_open.get("accuracy") or 0.0)

    trad_open = report.get("trad_lm_open_by_model") or {}
    trad_mcq = report.get("trad_lm_mcq_by_model") or {}
    premiums: list[dict[str, Any]] = []
    xs: list[float] = []
    ys_open: list[float] = []
    ys_mcq: list[float] = []

    for mid, arm in trad_open.items():
        params = float((arm.get("params_b") or 0.5))
        trad_acc = float(arm.get("accuracy") or 0.0)
        prem_pp = (vh_acc - trad_acc) * 100.0
        premiums.append(
            {
                "model_id": mid,
                "params_b": params,
                "valhalla_open": vh_acc,
                "trad_lm_open": trad_acc,
                "structure_premium_pp": round(prem_pp, 2),
            }
        )
        xs.append(log10_params(params))
        ys_open.append(trad_acc)
        mcq_arm = trad_mcq.get(mid) or {}
        ys_mcq.append(float(mcq_arm.get("accuracy") or 0.0))

    open_fit = linear_fit(xs, ys_open)
    mcq_fit = linear_fit(xs, ys_mcq) if any(ys_mcq) else open_fit

    learning = report.get("training_substrate") or {}
    lifts = {}
    if isinstance(learning.get("learning_paradigms"), dict):
        lifts = learning["learning_paradigms"].get("lifts_pp") or {}
    elif not learning.get("skipped"):
        lifts = learning.get("lifts_pp") or {}
    feedback_pp = float(lifts.get("feedback_lift_pp") or 0.0)
    curriculum_pp = float(lifts.get("curriculum_gain_pp") or 0.0)
    transfer = learning.get("transfer_eval") or {}
    tf = transfer.get("transfer_fresh_retrieval") or transfer.get("transfer_fresh_retrieval_no_feedback") or {}
    transfer_acc = float(tf.get("accuracy") or 0.0)
    polluted = transfer.get("curriculum_in_session_polluted") or {}
    polluted_acc = float(polluted.get("accuracy") or 0.0)
    transfer_lift_pp = (transfer_acc - polluted_acc) * 100.0 if polluted_acc else 0.0

    prem_sorted = sorted(premiums, key=lambda x: x["params_b"])
    premium_shrink = None
    if len(prem_sorted) >= 2:
        premium_shrink = prem_sorted[0]["structure_premium_pp"] - prem_sorted[-1]["structure_premium_pp"]

    patterns: list[str] = []
    if open_fit["slope"] > 0.02:
        patterns.append("TRAD_RAG_SCALES_WITH_PARAMS")
    if vh_acc >= 0.85 and all(p["structure_premium_pp"] >= 0 for p in premiums):
        patterns.append("STRUCTURE_PREMIUM_NONNEGATIVE")
    if premium_shrink is not None and premium_shrink > 3.0:
        patterns.append("STRUCTURE_PREMIUM_LARGEST_AT_SMALL_BACKBONE")
    vh_mcq = report.get("valhalla_hybrid_mcq_lm_patch") or {}
    if vh_mcq.get("accuracy") and trad_mcq.get("qwen_0.5b", {}).get("accuracy"):
        if vh_mcq["accuracy"] > trad_mcq["qwen_0.5b"]["accuracy"] + 0.10:
            patterns.append("HYBRID_MCQ_BEATS_TRAD_AT_SAME_BACKBONE")
    if mcq_fit["slope"] > 0.05:
        patterns.append("TRAD_MCQ_SCALES_WITH_PARAMS")
    if vh_acc >= 0.90:
        patterns.append("TIER_B_OPEN_CEILING_STABLE")
    if feedback_pp >= 1.0 or curriculum_pp >= 2.0 or transfer_acc >= 0.85 or transfer_lift_pp >= 30.0:
        patterns.append("ZERO_WEIGHT_TRAINING_LIFT")
    if transfer_lift_pp >= 30.0:
        patterns.append("TRANSFER_BEATS_POLLUTED_SESSION")

    verdict = "SCALING_EVIDENCE_STRONG"
    if len(patterns) < 3:
        verdict = "SCALING_EVIDENCE_PARTIAL"
    if not patterns:
        verdict = "SCALING_EVIDENCE_WEAK"

    projected: list[dict[str, Any]] = []
    for target_b in (7.0, 14.0, 70.0):
        x = log10_params(target_b)
        proj_trad = open_fit["intercept"] + open_fit["slope"] * x
        proj_trad = max(0.0, min(1.0, proj_trad))
        projected.append(
            {
                "params_b": target_b,
                "trad_lm_open_projected": round(proj_trad, 4),
                "valhalla_open_assumed": round(vh_acc, 4),
                "structure_premium_projected_pp": round((vh_acc - proj_trad) * 100, 1),
            }
        )

    return {
        "valhalla_open_accuracy": vh_acc,
        "structure_premium_by_model": premiums,
        "premium_shrink_small_to_large_pp": premium_shrink,
        "trad_lm_open_scaling": {
            "x": "log10(params_b)",
            **{k: round(v, 4) for k, v in open_fit.items()},
            "interpretation": "+0.05 acc per 10× params (slope on log10 scale)",
        },
        "trad_lm_mcq_scaling": {
            "x": "log10(params_b)",
            **{k: round(v, 4) for k, v in mcq_fit.items()},
        },
        "training_substrate_lifts_pp": lifts,
        "transfer_lift_vs_polluted_pp": round(transfer_lift_pp, 2),
        "scale_projection": projected,
        "patterns": patterns,
        "verdict": verdict,
    }
