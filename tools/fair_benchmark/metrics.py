#!/usr/bin/env python3
"""Scoring and aggregation for Valhalla Fair Benchmark."""

from __future__ import annotations

import re
from collections import defaultdict
from typing import Any


def extract_mcq_letter(text: str) -> str | None:
    m = re.search(r"(?i)(?:ANSWER|答案)[:：\s]*\*?\*?\s*([A-D])\b", text)
    if m:
        return m.group(1).upper()
    m = re.search(r"(?i)(?:correct answer|answer is)[:：\s]*([A-D])\b", text)
    if m:
        return m.group(1).upper()
    for line in reversed(text.strip().splitlines()):
        t = line.strip()
        if re.fullmatch(r"[A-Da-d]", t):
            return t.upper()
    return None


def extract_number(text: str) -> str | None:
    nums = re.findall(r"-?\d+(?:\.\d+)?", text.replace(",", ""))
    return nums[-1] if nums else None


def score_answer(text: str, prompt: dict, *, strict_mcq: bool = True) -> dict:
    ref = prompt.get("reference_answer", "")
    keywords = prompt.get("keywords", [])
    stype = prompt.get("score_type", "open")
    low = text.lower()

    if stype == "mcq":
        letter = extract_mcq_letter(text)
        ref_letter = ref.strip().upper()[:1]
        hit = letter == ref_letter if letter else False
        return {
            "score_type": "mcq",
            "correct": hit,
            "extracted": letter,
            "reference": ref_letter,
            "strict": strict_mcq,
        }

    if stype == "numeric":
        num = extract_number(text)
        hit = num == ref or ref in (num or "")
        if not hit:
            hit = ref.lower() in low
        return {
            "score_type": "numeric",
            "correct": hit,
            "extracted": num,
            "reference": ref,
        }

    ref_hit = ref.lower() in low or any(k.lower() in low for k in ref.split() if len(k) > 2)
    if not ref_hit:
        ref_hit = sum(1 for k in keywords if k.lower() in low) >= max(1, len(keywords) // 2)
    return {"score_type": "open", "correct": ref_hit, "extracted": None, "reference": ref}


def retrieval_hit(prompt: dict, top_matches: list[dict], k: int = 3) -> dict:
    """Memory-platform retrieval metrics (independent of decode path)."""
    ref = str(prompt.get("reference_answer", "")).lower()
    keywords = [str(k).lower() for k in (prompt.get("keywords") or []) if k]
    hits = []
    for i, m in enumerate(top_matches[:k]):
        text = str(m.get("text", "")).lower()
        src = str(m.get("source", "")).lower()
        blob = text + " " + src
        ref_ok = ref and ref in blob
        kw_ok = sum(1 for kw in keywords if kw in blob) >= max(1, len(keywords) // 2)
        hits.append({"rank": i + 1, "ref_hit": ref_ok, "kw_hit": kw_ok, "any_hit": ref_ok or kw_ok})
    recall_at_k = any(h["any_hit"] for h in hits) if hits else False
    mrr = 0.0
    for h in hits:
        if h["any_hit"]:
            mrr = 1.0 / h["rank"]
            break
    return {"recall_at_k": recall_at_k, "mrr": round(mrr, 4), "top_hits": hits}


def bootstrap_ci_paired(
    before: list[bool], after: list[bool], n_boot: int = 5000, seed: int = 42
) -> dict:
    import random

    assert len(before) == len(after)
    n = len(before)
    if n == 0:
        return {"mean_pp": 0.0, "ci95_lo": 0.0, "ci95_hi": 0.0, "n": 0}
    rng = random.Random(seed)
    deltas = []
    for _ in range(n_boot):
        idx = [rng.randrange(n) for _ in range(n)]
        bb = sum(before[i] for i in idx) / n
        aa = sum(after[i] for i in idx) / n
        deltas.append((aa - bb) * 100)
    deltas.sort()
    lo = deltas[int(0.025 * n_boot)]
    hi = deltas[int(0.975 * n_boot)]
    mean = (sum(after) / n - sum(before) / n) * 100
    return {"mean_pp": round(mean, 2), "ci95_lo": round(lo, 2), "ci95_hi": round(hi, 2), "n": n}


def summarize_rows(rows: list[dict]) -> dict[str, Any]:
    """Aggregate acc, by score_type, decode_path, retrieval."""
    n = len(rows)
    correct = sum(1 for r in rows if r.get("correct"))
    by_st: dict[str, list[bool]] = defaultdict(list)
    by_path: dict[str, list[bool]] = defaultdict(list)
    by_domain: dict[str, list[bool]] = defaultdict(list)
    recall_hits = []
    mrr_vals = []

    for r in rows:
        by_st[r.get("score_type", "unknown")].append(r["correct"])
        by_path[r.get("decode_path") or "unknown"].append(r["correct"])
        by_domain[r.get("domain", "unknown")].append(r["correct"])
        ret = r.get("retrieval") or {}
        if "recall_at_k" in ret:
            recall_hits.append(ret["recall_at_k"])
            mrr_vals.append(ret.get("mrr", 0.0))

    def acc_dict(d: dict[str, list[bool]]) -> dict:
        return {
            k: {
                "correct": sum(v),
                "total": len(v),
                "acc": round(sum(v) / len(v), 4) if v else 0.0,
            }
            for k, v in sorted(d.items())
        }

    out: dict[str, Any] = {
        "n": n,
        "correct": correct,
        "acc": round(correct / n, 4) if n else 0.0,
        "by_score_type": acc_dict(by_st),
        "by_decode_path": acc_dict(by_path),
        "by_domain": acc_dict(by_domain),
    }
    if recall_hits:
        out["retrieval"] = {
            "recall_at_3": round(sum(recall_hits) / len(recall_hits), 4),
            "mrr_mean": round(sum(mrr_vals) / len(mrr_vals), 4),
        }
    return out
