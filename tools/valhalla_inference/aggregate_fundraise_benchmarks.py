#!/usr/bin/env python3
"""Aggregate investor-facing benchmark dashboard from reports/valhalla_inference/*.json.

Protocol: fundraise-aggregate-v1

Outputs:
  - FUNDRAISE_BENCHMARK_LATEST.json
  - FUNDRAISE_BENCHMARK_LATEST.md
  - fundraise_benchmark_history.jsonl (append one line per run)
"""

from __future__ import annotations

import json
import sys
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
REPORTS = REPO / "reports/valhalla_inference"

# (json file, display name, headline metric keys)
PANEL = [
    ("proactive_agency_benchmark.json", "Proactive Agency v1", "agency_index", None, None),
    ("proactive_fate_v2_test.json", "Proactive Fate v2", "accuracy", "correct", "n"),
    ("fate_output_gate_v2_test.json", "Fate output gate v2", "pass_rate", "passed", "n"),
    ("long_memory_test_full.json", "Long memory v1", "recall", "correct", "n"),
    ("ragtruth_v1_test.json", "RAGTruth", "faithful_rate", "faithful", "n"),
    ("ragtruth_full_v1_test.json", "RAGTruth-100", "faithful_rate", "faithful", "n"),
    ("needle_session_v1_test.json", "Needle session", "recall", "correct", "n"),
    ("external_holdout_test.json", "External holdout", "accuracy", "correct", "n"),
    ("ruler_v1_test.json", "RULER-style", "recall", "correct", "n"),
    ("lara_v1_test.json", "LaRA RAG", "accuracy", "correct", "n"),
    ("multihop_rag_v1_test.json", "MultiHop-RAG v1", "accuracy", "correct", "n"),
    ("multihop_rag_v2_test.json", "MultiHop-RAG v2", "accuracy", "correct", "n"),
    ("multihop_rag_fate_test.json", "MultiHop-RAG Fate", "accuracy", "correct", "n"),
    ("multihop_rag_hybrid_test.json", "MultiHop-RAG hybrid", "accuracy", "correct", "n"),
    ("valhalla_base_training_potential_v2_test.json", "TPI v2", "final_accuracy", None, "rounds"),
    ("new_paradigm_potential_test.json", "NPPI", "nppi_index", None, None),
]


def load(fname: str) -> dict | None:
    p = REPORTS / fname
    if not p.is_file():
        return None
    return json.loads(p.read_text(encoding="utf-8"))


def load_long_memory() -> dict | None:
    for fname in ("long_memory_v1_test.json", "long_memory_test_full.json", "long_memory_test_fast.json"):
        doc = load(fname)
        if doc is None:
            continue
        arms = doc.get("arms") or {}
        prod = arms.get("production") or {}
        summ = prod.get("summary") or {}
        overall = summ.get("overall") or {}
        if overall.get("n"):
            return {
                "protocol": doc.get("protocol", "long-memory-v1"),
                "n": overall["n"],
                "correct": overall.get("hits", overall.get("correct", 0)),
                "recall": overall.get("rate"),
                "verdict": "STRONG" if float(overall.get("rate", 0)) >= 0.9 else "VIABLE",
            }
    return None


def fmt_result(doc: dict, rate_k: str | None, num_k: str | None, n_k: str | None) -> str:
    n = doc.get(n_k) if n_k else None
    num = doc.get(num_k) if num_k else None
    rate = doc.get(rate_k) if rate_k else None
    if num is not None and n is not None:
        if rate is not None:
            return f"{num}/{n} ({100 * float(rate):.0f}%)"
        return f"{num}/{n}"
    if rate is not None:
        return f"{100 * float(rate):.1f}%"
    if num is not None:
        return str(num)
    return "—"


def gate_status(name: str, doc: dict, acc: float | None = None) -> str:
    raw = doc.get("verdict")
    if isinstance(raw, list):
        v = (raw[0] if raw else "").upper()
    else:
        v = (raw or "").upper()
    if acc is None:
        acc = doc.get("accuracy") or doc.get("faithful_rate") or doc.get("recall") or doc.get("pass_rate")
        summ = doc.get("summary") or {}
        if acc is None and summ.get("pass_rate") is not None:
            acc = summ["pass_rate"]
        ev = doc.get("evaluation") or {}
        if acc is None and ev.get("pass_rate") is not None:
            acc = ev["pass_rate"]
    if "STRONG" in v or (acc is not None and float(acc) >= 0.95):
        return "PASS"
    if "VIABLE" in v or (acc is not None and float(acc) >= 0.5):
        return "WATCH"
    return "GAP"


def row_from_doc(fname: str, label: str, rate_k: str | None, num_k: str | None, n_k: str | None) -> dict | None:
    if fname.startswith("long_memory"):
        doc = load_long_memory()
        if not doc:
            return None
        rate = doc.get("recall")
        result = fmt_result(doc, rate_k, num_k, n_k)
        return {
            "name": label,
            "file": fname,
            "protocol": doc.get("protocol"),
            "result": result,
            "verdict": doc.get("verdict"),
            "gate": gate_status(label, doc, rate),
            "timestamp": doc.get("timestamp"),
        }

    doc = load(fname)
    if not doc:
        return None

    if fname == "proactive_agency_benchmark.json":
        metrics = doc.get("metrics") or {}
        rate = metrics.get("agency_index")
        cfg = doc.get("config") or {}
        mega = cfg.get("mega")
        turns = cfg.get("max_turns", "—")
        result = f"Agency {float(rate):.3f}" + (f" · {turns} rounds" if mega else "")
        return {
            "name": label,
            "file": fname,
            "protocol": doc.get("protocol"),
            "result": result,
            "verdict": doc.get("verdict"),
            "gate": gate_status(label, doc, rate),
            "timestamp": doc.get("timestamp"),
        }

    if fname == "proactive_fate_v2_test.json":
        summ = doc.get("summary") or {}
        passed, total = summ.get("passed"), summ.get("total")
        rate = summ.get("pass_rate")
        result = f"{passed}/{total} ({100 * float(rate):.1f}%)" if rate is not None else "—"
        verdict = (doc.get("verdict") or ["—"])[0] if isinstance(doc.get("verdict"), list) else doc.get("verdict")
        return {
            "name": label,
            "file": fname,
            "protocol": doc.get("protocol"),
            "result": result,
            "verdict": verdict,
            "gate": gate_status(label, doc, rate),
            "timestamp": doc.get("timestamp"),
        }

    if fname == "lara_v1_test.json":
        rag = doc.get("valhalla_rag") or {}
        rate = rag.get("accuracy")
        correct = rag.get("correct")
        n = doc.get("n")
        result = f"{correct}/{n} ({100 * float(rate):.0f}%)" if rate is not None and n else "—"
        return {
            "name": label,
            "file": fname,
            "protocol": doc.get("protocol"),
            "result": result,
            "verdict": doc.get("verdict", "VIABLE"),
            "gate": gate_status(label, {"accuracy": rate}),
            "timestamp": doc.get("timestamp"),
        }

    if fname == "fate_output_gate_v2_test.json":
        ev = doc.get("evaluation") or {}
        rate = ev.get("pass_rate")
        result = f"{100 * float(rate):.0f}% checks" if rate is not None else "—"
        return {
            "name": label,
            "file": fname,
            "protocol": doc.get("protocol"),
            "result": result,
            "verdict": ev.get("verdict"),
            "gate": gate_status(label, {"verdict": ev.get("verdict"), "pass_rate": rate}),
            "timestamp": doc.get("timestamp"),
        }

    if fname == "new_paradigm_potential_test.json":
        idx = doc.get("nppi_index")
        verdict = (doc.get("verdict") or ["—"])[0] if isinstance(doc.get("verdict"), list) else doc.get("verdict")
        return {
            "name": label,
            "file": fname,
            "protocol": doc.get("protocol"),
            "result": f"NPPI {idx:.3f}" if idx is not None else "—",
            "verdict": verdict,
            "gate": "WATCH" if idx and float(idx) >= 0.5 else "GAP",
            "timestamp": doc.get("timestamp"),
        }

    return {
        "name": label,
        "file": fname,
        "protocol": doc.get("protocol"),
        "result": fmt_result(doc, rate_k, num_k, n_k),
        "verdict": doc.get("verdict") if not isinstance(doc.get("verdict"), list) else (doc.get("verdict") or ["—"])[0],
        "gate": gate_status(label, doc),
        "timestamp": doc.get("timestamp"),
    }


def build() -> dict:
    rows: list[dict] = []
    for fname, label, rate_k, num_k, n_k in PANEL:
        row = row_from_doc(fname, label, rate_k, num_k, n_k)
        if row:
            rows.append(row)

    passes = sum(1 for r in rows if r["gate"] == "PASS")
    watches = sum(1 for r in rows if r["gate"] == "WATCH")
    gaps = sum(1 for r in rows if r["gate"] == "GAP")

    return {
        "protocol": "fundraise-aggregate-v1",
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "summary": {
            "panels": len(rows),
            "pass": passes,
            "watch": watches,
            "gap": gaps,
            "fundraise_ready": passes >= 6 and gaps <= 3,
        },
        "benchmarks": rows,
        "report_dir": str(REPORTS),
        "rerun": "./tools/valhalla_inference/run_fundraise_continuous.sh",
    }


def write_md(doc: dict) -> str:
    s = doc.get("summary") or {}
    lines = [
        "# ValhallaBase — Fundraise Benchmark Dashboard",
        "",
        f"**Generated:** {doc.get('timestamp', '')}",
        "",
        f"**Investor gate:** {s.get('pass', 0)} PASS · {s.get('watch', 0)} WATCH · {s.get('gap', 0)} GAP",
        f"**Fundraise-ready heuristic:** {'YES' if s.get('fundraise_ready') else 'NO — close MultiHop / LoCoMo gaps first'}",
        "",
        "## Headline metrics",
        "",
        "| Capability | Result | Verdict | Gate |",
        "|------------|--------|---------|------|",
    ]
    for r in doc.get("benchmarks") or []:
        lines.append(
            f"| {r['name']} | **{r['result']}** | {r.get('verdict', '—')} | {r['gate']} |"
        )
    lines.extend(
        [
            "",
            "## Continuous run",
            "",
            "```bash",
            "# Full cycle (~30–45 min)",
            "./tools/valhalla_inference/run_fundraise_continuous.sh",
            "",
            "# Every 6 hours (background)",
            "nohup ./tools/valhalla_inference/run_fundraise_continuous.sh --loop 6 >> reports/valhalla_inference/fundraise_cron.log 2>&1 &",
            "```",
            "",
            "---",
            "",
            "*Rogue Intelligence LNC — for investor due diligence.*",
        ]
    )
    return "\n".join(lines) + "\n"


def main() -> None:
    REPORTS.mkdir(parents=True, exist_ok=True)
    doc = build()
    out_json = REPORTS / "FUNDRAISE_BENCHMARK_LATEST.json"
    out_md = REPORTS / "FUNDRAISE_BENCHMARK_LATEST.md"
    hist = REPORTS / "fundraise_benchmark_history.jsonl"

    out_json.write_text(json.dumps(doc, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    out_md.write_text(write_md(doc), encoding="utf-8")

    with hist.open("a", encoding="utf-8") as f:
        f.write(
            json.dumps(
                {
                    "timestamp": doc["timestamp"],
                    "pass": doc["summary"]["pass"],
                    "watch": doc["summary"]["watch"],
                    "gap": doc["summary"]["gap"],
                    "fundraise_ready": doc["summary"]["fundraise_ready"],
                },
                ensure_ascii=False,
            )
            + "\n"
        )

    print(f"→ {out_md}")
    print(f"→ {out_json}")
    if not doc.get("benchmarks"):
        sys.exit("no benchmark JSON found — run tests first")


if __name__ == "__main__":
    main()
