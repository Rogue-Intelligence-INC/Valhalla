#!/usr/bin/env python3
"""Substrate proof: non-isolated multimodal + default mixed answering.

Claim (product):
  Modalities enter ONE Triad session (non-isolated). Answer-time default is
  **mixed** — all modality-tagged lines can land. User may choose text|audio|image
  for ablation; mixed remains the production default.

Does NOT expand generative model catalog — decode head stays optional/swappable.

Usage:
  PYTHONPATH=tools/valhalla_inference python3 tools/valhalla_inference/test_multimodal_mixed_answer.py
"""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path

from benchmark_env import production_multimodal_kwargs
from valhalla_base import ValhallaBase

REPO = Path(__file__).resolve().parents[2]
REPORTS = REPO / "reports/valhalla_inference"

# Three facts on three modality channels — same session, non-isolated.
ITEMS = [
    {"text": "TEXT_NOTE: meeting host is Nora."},
    {
        "text": "AUDIO_CAPTION: the badge color is amber.",
        "audio_features": [0.12, 0.34, 0.56, 0.78, 0.21, 0.43],
    },
    {
        "text": "IMAGE_CAPTION: the room plaque says Z-17.",
        "image_features": [0.91, 0.12, 0.03, 0.44, 0.55, 0.66],
    },
]

CASES = [
    {
        "id": "Q_HOST_TEXT",
        "q": "Who is the meeting host?",
        "need": ["nora"],
        "expect": {"mixed": True, "text": True, "audio": False, "image": False},
    },
    {
        "id": "Q_BADGE_AUDIO",
        "q": "What badge color is mentioned?",
        "need": ["amber"],
        "expect": {"mixed": True, "text": False, "audio": True, "image": False},
    },
    {
        "id": "Q_ROOM_IMAGE",
        "q": "What does the room plaque say?",
        "need": ["z-17", "z17"],
        "expect": {"mixed": True, "text": False, "audio": False, "image": True},
        "any_need": True,
    },
]


def _hit(answer: str, need: list[str], any_need: bool) -> bool:
    low = answer.lower()
    if any_need:
        return any(tok in low for tok in need)
    return all(tok in low for tok in need)


def run_mode(mode: str) -> dict:
    base = ValhallaBase(
        **production_multimodal_kwargs(
            answer_mix=mode,
            decode="hybrid",
            max_new=64,
        )
    )
    rows = []
    for c in CASES:
        base.clear_context()
        doc = base.run(
            [{"id": f"{c['id']}_{mode}", "text": c["q"], "score_type": "open"}],
            context_items=ITEMS,
        )
        r0 = doc["rows"][0]
        ans = (r0.get("answer") or "").strip()
        ok = _hit(ans, c["need"], c.get("any_need", False))
        expected = bool(c["expect"].get(mode))
        rows.append(
            {
                "id": c["id"],
                "mode": mode,
                "pass": ok == expected,
                "hit": ok,
                "expected_hit": expected,
                "answer": ans[:200],
                "answer_mix": r0.get("answer_mix"),
                "answer_mix_eligible": r0.get("answer_mix_eligible"),
                "features": doc.get("features"),
                "decode_path": r0.get("decode_path"),
            }
        )
    return {
        "mode": mode,
        "features": rows[0]["features"] if rows else [],
        "rows": rows,
        "all_pass": all(r["pass"] for r in rows),
    }


def main() -> int:
    modes = ["mixed", "text", "audio", "image"]
    by_mode = {m: run_mode(m) for m in modes}

    default_mixed = by_mode["mixed"]
    has_default_flag = any(
        "answer_mix_default_mixed" in (f or "")
        for f in (default_mixed.get("features") or [])
    ) or any(
        str(f) == "answer_mix_mixed" for f in (default_mixed.get("features") or [])
    )

    # Non-isolated: all three channels share one session (production_multimodal core_fusion).
    non_isolated = "core_fusion" in (default_mixed.get("features") or []) and (
        "multimodal_context" in (default_mixed.get("features") or [])
    )

    mode_pass = {m: by_mode[m]["all_pass"] for m in modes}
    all_ok = all(mode_pass.values()) and has_default_flag and non_isolated

    out = {
        "protocol": "multimodal-mixed-answer-v1",
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "product_claim": (
            "Non-isolated multimodal Triad; answer_mix default=mixed; "
            "optional text|audio|image gates; decode head orthogonal (not expanded)."
        ),
        "non_isolated": non_isolated,
        "default_mixed_feature": has_default_flag,
        "mode_pass": mode_pass,
        "by_mode": by_mode,
        "verdict": "MULTIMODAL_MIXED_ANSWER_PASS" if all_ok else "MULTIMODAL_MIXED_ANSWER_FAIL",
    }

    REPORTS.mkdir(parents=True, exist_ok=True)
    path = REPORTS / "multimodal_mixed_answer.json"
    path.write_text(json.dumps(out, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")

    md = REPORTS / "MULTIMODAL_MIXED_ANSWER.md"
    lines = [
        "# Multimodal mixed-answer proof",
        "",
        f"**Protocol:** `{out['protocol']}`  ",
        f"**Verdict:** **{out['verdict']}**  ",
        f"**Timestamp:** {out['timestamp']}",
        "",
        "## Claim",
        "",
        out["product_claim"],
        "",
        "## Results",
        "",
        f"- Non-isolated features (`core_fusion` + `multimodal_context`): `{non_isolated}`",
        f"- Default mixed feature flag: `{has_default_flag}`",
        "",
        "| Mode | Pass |",
        "|------|------|",
    ]
    for m in modes:
        lines.append(f"| `{m}` | `{mode_pass[m]}` |")
    lines.extend(
        [
            "",
            "## Reproduce",
            "",
            "```bash",
            "PYTHONPATH=tools/valhalla_inference \\",
            "  python3 tools/valhalla_inference/test_multimodal_mixed_answer.py",
            "```",
            "",
            f"Artifact: `{path.relative_to(REPO)}`",
            "",
            "*Rogue Intelligence LNC.*",
            "",
        ]
    )
    md.write_text("\n".join(lines), encoding="utf-8")

    print(f"Verdict: {out['verdict']}")
    for m in modes:
        print(f"  {m}: {'PASS' if mode_pass[m] else 'FAIL'}")
        for r in by_mode[m]["rows"]:
            mark = "✓" if r["pass"] else "✗"
            print(f"    {mark} {r['id']} hit={r['hit']} expect={r['expected_hit']} → {r['answer'][:80]!r}")
    print(f"→ {path}")
    print(f"→ {md}")
    return 0 if all_ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
