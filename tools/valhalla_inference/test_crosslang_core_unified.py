#!/usr/bin/env python3
"""Cross-language unified core — same ValhallaBase stack, no language switching.

Protocol: crosslang-core-unified-v1

Tests whether EN / ZH / FR (etc.) queries against a **shared multilingual corpus**
route through the **same production core** (hybrid + quad_cycle + native_follow_up)
and retrieve the **same factual line** — no per-language model or decode switch.

Metrics:
  CCU     crosslang core unified index ∈ [0,1]
  line_agreement   fraction of facts where all langs hit the same corpus line
  decode_agreement all langs use native path on open QA
  fate_agreement   dominant Fate system index matches across langs (±0)

Usage:
  python3 tools/valhalla_inference/test_crosslang_core_unified.py
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

REPO = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(REPO / "tools/fair_benchmark"))

from metrics import score_answer  # noqa: E402
from valhalla_base import ValhallaBase  # noqa: E402

PROTOCOL = "crosslang-core-unified-v1"
REPORTS = REPO / "reports/valhalla_inference"

# One canonical corpus line per fact — cross-lang queries must hit the same [corpus:0] core.
FACTS: list[dict[str, Any]] = [
    {
        "id": "yangtze",
        "corpus": [
            "The Yangtze River originates from glaciers on the Tibetan Plateau in China.",
        ],
        "queries": {
            "en": {
                "text": "Where does the Yangtze River originate? Answer briefly.",
                "keywords": ["tibet", "plateau", "glacier", "yangtze"],
            },
            "zh": {
                "text": "长江发源于哪里？请简要回答。",
                "keywords": ["tibet", "plateau", "glacier", "tibetan", "yangtze", "青藏"],
            },
            "fr": {
                "text": "D'où prend sa source le fleuve Yangtsé ? Réponse brève.",
                "keywords": ["tibet", "plateau", "glacier", "tibetan", "yangtze"],
            },
        },
    },
    {
        "id": "qin_dynasty",
        "corpus": [
            "The Qin dynasty was the first unified centralised feudal dynasty in Chinese history.",
        ],
        "queries": {
            "en": {
                "text": "Which was the first unified centralised dynasty in Chinese history?",
                "keywords": ["qin", "dynasty", "unified"],
            },
            "zh": {
                "text": "中国历史上第一个统一的中央集权封建王朝是哪个？",
                "keywords": ["qin", "dynasty", "unified", "秦"],
            },
            "fr": {
                "text": "Quelle dynastie a unifié la Chine en premier ?",
                "keywords": ["qin", "dynasty", "unified"],
            },
        },
    },
    {
        "id": "ww2_end",
        "corpus": [
            "World War Two ended in 1945 with Germany's unconditional surrender.",
        ],
        "queries": {
            "en": {"text": "In which year did World War II end?", "keywords": ["1945"]},
            "zh": {"text": "第二次世界大战在哪一年结束？", "keywords": ["1945"]},
            "fr": {
                "text": "En quelle année la Seconde Guerre mondiale s'est-elle terminée ?",
                "keywords": ["1945"],
            },
        },
    },
    {
        "id": "canberra",
        "corpus": [
            "The capital city of Australia is Canberra not Sydney or Melbourne.",
        ],
        "queries": {
            "en": {"text": "What is the capital of Australia?", "keywords": ["canberra"]},
            "zh": {"text": "澳大利亚的首都是哪里？", "keywords": ["canberra"]},
            "fr": {"text": "Quelle est la capitale de l'Australie ?", "keywords": ["canberra"]},
        },
    },
    {
        "id": "water_boil",
        "corpus": [
            "Water boils at 100 degrees Celsius at standard atmospheric pressure.",
        ],
        "queries": {
            "en": {
                "text": "At standard pressure, at what temperature does water boil in Celsius?",
                "keywords": ["100", "celsius", "boil"],
            },
            "zh": {
                "text": "标准大气压下水的沸点是多少摄氏度？",
                "keywords": ["100", "celsius", "boil"],
            },
            "fr": {
                "text": "À quelle température l'eau bout-elle à pression standard (°C) ?",
                "keywords": ["100", "celsius", "boil"],
            },
        },
    },
    {
        "id": "solar_terms",
        "corpus": [
            "There are 24 solar terms in the traditional Chinese calendar.",
        ],
        "queries": {
            "en": {
                "text": "How many solar terms are in the traditional Chinese calendar?",
                "keywords": ["24", "solar"],
            },
            "zh": {"text": "一年有多少个节气？", "keywords": ["24", "solar", "term"]},
            "fr": {
                "text": "Combien de termes solaires compte le calendrier chinois traditionnel ?",
                "keywords": ["24", "solar", "term"],
            },
        },
    },
]


def make_base() -> ValhallaBase:
    return ValhallaBase(
        decode="hybrid",
        follow_up_decode="native_follow_up_aware",
        fate_ingress_routing="quad_cycle",
        fate_qa_feedback=False,
        incubation_cycles=2,
        max_new=128,
    )


def parse_corpus_idx(answer: str) -> int | None:
    m = re.search(r"\[corpus:(\d+)\]", answer)
    return int(m.group(1)) if m else None


def answer_body(answer: str) -> str:
    return re.sub(r"^\[corpus:\d+\]\s*", "", answer).strip()


def line_hit(answer: str, corpus: list[str]) -> int | None:
    idx = parse_corpus_idx(answer)
    if idx is not None and 0 <= idx < len(corpus):
        return idx
    ans = answer.lower()
    for i, line in enumerate(corpus):
        low = line.lower()
        if low in ans or ans in low:
            return i
        cjk = "".join(c for c in line if ord(c) > 127)
        if len(cjk) >= 2 and cjk in answer:
            return i
    return None


def dominant_fate(prefs: list[float]) -> int:
    if not prefs:
        return -1
    return max(range(len(prefs)), key=lambda i: prefs[i])


def eval_lang(base: ValhallaBase, fact: dict, lang: str, q: dict) -> dict[str, Any]:
    payload = {
        "id": f"{fact['id']}_{lang}",
        "text": q["text"],
        "score_type": "open",
        "reference_answer": q["keywords"][0],
        "keywords": q["keywords"],
    }
    doc = base.run([payload])
    row = (doc.get("rows") or [{}])[0]
    ans = str(row.get("answer") or "")
    sc = score_answer(ans, payload)
    fate = doc.get("fate_telemetry") or {}
    prefs = fate.get("hub_prefs") or []
    return {
        "fact_id": fact["id"],
        "lang": lang,
        "question": q["text"],
        "answer": ans[:200],
        "correct": bool(sc["correct"]),
        "decode_path": row.get("decode_path"),
        "line_idx": line_hit(ans, fact["corpus"]),
        "dominant_fate": dominant_fate(prefs),
        "hub_prefs_spread": fate.get("hub_prefs_spread"),
    }


def run_fact(fact: dict) -> dict[str, Any]:
    rows: list[dict[str, Any]] = []
    for lang, q in fact["queries"].items():
        base = make_base()
        base.set_context(fact["corpus"])
        rows.append(eval_lang(base, fact, lang, q))

    line_idxs = [r["line_idx"] for r in rows if r["line_idx"] is not None]
    line_agree = len(set(line_idxs)) == 1 if len(line_idxs) == len(rows) else False
    bodies = [answer_body(r["answer"]) for r in rows]
    text_agree = len(set(bodies)) == 1
    decode_paths = [r["decode_path"] for r in rows]
    decode_native = all("native" in (p or "") for p in decode_paths)
    fate_idxs = [r["dominant_fate"] for r in rows if r["dominant_fate"] >= 0]
    fate_agree = len(set(fate_idxs)) <= 1 if fate_idxs else False
    all_correct = all(r["correct"] for r in rows)

    return {
        "fact_id": fact["id"],
        "rows": rows,
        "line_agreement": line_agree,
        "text_agreement": text_agree,
        "decode_native_all": decode_native,
        "fate_agreement": fate_agree,
        "all_correct": all_correct,
        "core_unified": line_agree and text_agree and all_correct and decode_native,
    }


def compute_ccu(results: list[dict]) -> float:
    if not results:
        return 0.0
    w = 0.35 * sum(1 for r in results if r["all_correct"]) / len(results)
    w += 0.25 * sum(1 for r in results if r["line_agreement"]) / len(results)
    w += 0.20 * sum(1 for r in results if r["decode_native_all"]) / len(results)
    w += 0.15 * sum(1 for r in results if r["fate_agreement"]) / len(results)
    return round(w, 4)


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--limit", type=int, default=0, help="0 = all facts")
    args = ap.parse_args()

    facts = FACTS[: args.limit] if args.limit > 0 else FACTS
    print(f"=== {PROTOCOL} facts={len(facts)} langs=3 stack=production (no switch) ===\n")

    results = [run_fact(f) for f in facts]
    ccu = compute_ccu(results)
    unified = sum(1 for r in results if r["core_unified"])

    report: dict[str, Any] = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "protocol": PROTOCOL,
        "stack": {
            "decode": "hybrid",
            "follow_up_decode": "native_follow_up_aware",
            "fate_ingress_routing": "quad_cycle",
            "language_switch": False,
        },
        "facts": results,
        "metrics": {
            "CCU": ccu,
            "core_unified_facts": f"{unified}/{len(results)}",
            "accuracy_all_langs": sum(
                sum(1 for row in r["rows"] if row["correct"]) for r in results
            )
            / max(sum(len(r["rows"]) for r in results), 1),
        },
        "verdict": (
            ["CROSSLANG_CORE_UNIFIED_STRONG"]
            if ccu >= 0.85
            else ["CROSSLANG_CORE_UNIFIED_VIABLE"]
            if ccu >= 0.65
            else ["CROSSLANG_CORE_DEVELOPING"]
        ),
    }

    REPORTS.mkdir(parents=True, exist_ok=True)
    out = REPORTS / "crosslang_core_unified_test.json"
    out.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")

    print(f"CCU = {ccu:.3f}  core_unified = {unified}/{len(results)}")
    for r in results:
        langs = ", ".join(
            f"{row['lang']}:{'✓' if row['correct'] else '✗'} line={row['line_idx']}"
            for row in r["rows"]
        )
        flag = "UNIFIED" if r["core_unified"] else "split"
        print(f"  [{r['fact_id']}] {flag} — {langs}")
    print(f"\n→ {out}")


if __name__ == "__main__":
    main()
