"""Confidence intervals for Valhalla inference benchmarks.

Methods:
  - Wilson score interval (binomial proportions, all n)
  - Bootstrap percentile CI (paired lifts, composite indices)
"""

from __future__ import annotations

import math
import random
from typing import Any, Callable, Sequence

Z975 = 1.959963984540054  # two-sided 95%


def wilson_ci(successes: int, n: int, *, alpha: float = 0.05) -> dict[str, Any]:
    """Wilson score interval for binomial proportion."""
    if n <= 0:
        return {"k": 0, "n": 0, "point": None, "ci95_lo": None, "ci95_hi": None, "method": "wilson"}
    k = max(0, min(successes, n))
    p = k / n
    if n < 1:
        return {"k": k, "n": n, "point": p, "ci95_lo": p, "ci95_hi": p, "method": "wilson"}
    z = Z975 if alpha == 0.05 else 1.959963984540054
    z2 = z * z
    denom = 1 + z2 / n
    center = (p + z2 / (2 * n)) / denom
    margin = (z / denom) * math.sqrt(p * (1 - p) / n + z2 / (4 * n * n))
    lo = max(0.0, center - margin)
    hi = min(1.0, center + margin)
    return {
        "k": k,
        "n": n,
        "point": round(p, 6),
        "ci95_lo": round(lo, 6),
        "ci95_hi": round(hi, 6),
        "method": "wilson",
    }


def proportion_ci(successes: int, n: int) -> dict[str, Any]:
    return wilson_ci(successes, n)


def proportion_ci_from_bools(values: Sequence[bool]) -> dict[str, Any]:
    n = len(values)
    k = sum(1 for v in values if v)
    return wilson_ci(k, n)


def bootstrap_ci_paired(
    before: Sequence[bool],
    after: Sequence[bool],
    *,
    n_boot: int = 5000,
    seed: int = 42,
) -> dict[str, Any]:
    """Bootstrap CI for paired accuracy lift (after - before) in percentage points."""
    assert len(before) == len(after)
    n = len(before)
    if n == 0:
        return {"mean_pp": 0.0, "ci95_lo": 0.0, "ci95_hi": 0.0, "n": 0, "method": "bootstrap_paired"}
    rng = random.Random(seed)
    deltas: list[float] = []
    for _ in range(n_boot):
        idx = [rng.randrange(n) for _ in range(n)]
        bb = sum(before[i] for i in idx) / n
        aa = sum(after[i] for i in idx) / n
        deltas.append((aa - bb) * 100.0)
    deltas.sort()
    lo = deltas[int(0.025 * n_boot)]
    hi = deltas[int(0.975 * n_boot)]
    mean = (sum(after) / n - sum(before) / n) * 100.0
    return {
        "mean_pp": round(mean, 3),
        "ci95_lo": round(lo, 3),
        "ci95_hi": round(hi, 3),
        "n": n,
        "method": "bootstrap_paired",
    }


def bootstrap_ci_index(
    sampler: Callable[[random.Random], float],
    *,
    n_boot: int = 5000,
    seed: int = 42,
) -> dict[str, Any]:
    """Bootstrap percentile CI for a scalar index via resampling callback."""
    rng = random.Random(seed)
    samples = [sampler(rng) for _ in range(n_boot)]
    samples.sort()
    lo = samples[int(0.025 * n_boot)]
    hi = samples[int(0.975 * n_boot)]
    point = samples[int(0.5 * n_boot)]
    return {
        "point": round(point, 4),
        "ci95_lo": round(lo, 4),
        "ci95_hi": round(hi, 4),
        "n_boot": n_boot,
        "method": "bootstrap_percentile",
    }


def fmt_pct(ci: dict[str, Any]) -> str:
    if ci.get("point") is None:
        return "—"
    p = ci["point"] * 100
    lo = ci["ci95_lo"] * 100
    hi = ci["ci95_hi"] * 100
    return f"{p:.1f}% [{lo:.1f}%, {hi:.1f}%]"


def fmt_pp(ci: dict[str, Any]) -> str:
    if not ci.get("n"):
        return "—"
    return f"{ci['mean_pp']:+.1f}pp [{ci['ci95_lo']:+.1f}, {ci['ci95_hi']:+.1f}]"


def fmt_index(ci: dict[str, Any]) -> str:
    return f"{ci['point']:.3f} [{ci['ci95_lo']:.3f}, {ci['ci95_hi']:.3f}]"


def paired_rows_by_id(a_rows: list[dict], b_rows: list[dict]) -> tuple[list[bool], list[bool]]:
    am = {r["id"]: bool(r.get("correct")) for r in a_rows if r.get("id")}
    bm = {r["id"]: bool(r.get("correct")) for r in b_rows if r.get("id")}
    ids = sorted(set(am) & set(bm))
    return [am[i] for i in ids], [bm[i] for i in ids]


def attach_metric_block(metrics: dict[str, Any]) -> dict[str, Any]:
    """Add human-readable strings for report markdown."""
    out = dict(metrics)
    if "proportion" in out:
        out["display"] = fmt_pct(out["proportion"])
    if "lift_pp" in out:
        out["lift_display"] = fmt_pp(out["lift_pp"])
    if "index" in out:
        out["index_display"] = fmt_index(out["index"])
    return out
