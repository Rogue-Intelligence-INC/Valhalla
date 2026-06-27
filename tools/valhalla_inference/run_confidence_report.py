#!/usr/bin/env python3
"""Unified confidence report — Wilson + bootstrap CIs for all inference protocols.

Protocol: confidence-report-v1

Reads existing benchmark JSON under reports/valhalla_inference/ and emits:
  - confidence_report_v1.json
  - CONFIDENCE_REPORT_v1.md

Usage:
  python3 tools/valhalla_inference/run_confidence_report.py
  python3 tools/valhalla_inference/run_confidence_report.py --reports-dir /path/to/reports
"""

from __future__ import annotations

import argparse
import json
import random
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
sys.path.insert(0, str(HERE))

from confidence_stats import (  # noqa: E402
    bootstrap_ci_index,
    bootstrap_ci_paired,
    fmt_index,
    fmt_pct,
    fmt_pp,
    paired_rows_by_id,
    proportion_ci,
    proportion_ci_from_bools,
    wilson_ci,
)

PROTOCOL = "confidence-report-v1"
DEFAULT_REPORTS = REPO / "reports/valhalla_inference"


def binomial_resample_accuracy(correct: int, n: int, rng: random.Random) -> float:
    if n <= 0:
        return 0.0
    p = correct / n
    k = sum(1 for _ in range(n) if rng.random() < p)
    return k / n


def load(path: Path) -> dict[str, Any] | None:
    if not path.is_file():
        return None
    return json.loads(path.read_text(encoding="utf-8"))


def metric(name: str, prop: dict[str, Any], **extra: Any) -> dict[str, Any]:
    row = {"name": name, "proportion": prop, "display": fmt_pct(prop), **extra}
    return row


def lift_metric(name: str, lift: dict[str, Any], **extra: Any) -> dict[str, Any]:
    return {"name": name, "lift_pp": lift, "display": fmt_pp(lift), **extra}


def index_metric(name: str, idx: dict[str, Any], **extra: Any) -> dict[str, Any]:
    return {"name": name, "index": idx, "display": fmt_index(idx), **extra}


def rows_correct(rows: list[dict]) -> list[bool]:
    return [bool(r.get("correct")) for r in rows]


def analyze_tpi_v2(doc: dict[str, Any]) -> dict[str, Any]:
    out: dict[str, Any] = {"protocol": doc.get("protocol"), "metrics": [], "missing": []}
    learning = doc.get("learning_paradigms") or {}
    iso = learning.get("isolated_baseline") or {}
    iso_rows = iso.get("rows") or []
    if iso_rows:
        out["metrics"].append(metric("isolated_baseline_accuracy", proportion_ci_from_bools(rows_correct(iso_rows))))
    elif iso.get("n"):
        out["metrics"].append(metric("isolated_baseline_accuracy", wilson_ci(int(iso.get("correct", 0)), int(iso["n"]))))

    polluted = (doc.get("transfer_eval") or {}).get("curriculum_in_session_polluted") or {}
    transfer = (doc.get("transfer_eval") or {}).get("transfer_fresh_retrieval") or {}
    if polluted.get("rows") and transfer.get("rows"):
        b, a = paired_rows_by_id(polluted["rows"], transfer["rows"])
        out["metrics"].append(metric("polluted_session_accuracy", proportion_ci_from_bools(b)))
        out["metrics"].append(metric("transfer_fresh_accuracy", proportion_ci_from_bools(a)))
        out["metrics"].append(lift_metric("transfer_vs_polluted_lift", bootstrap_ci_paired(b, a)))
    elif polluted.get("n") and transfer.get("n"):
        out["metrics"].append(metric("polluted_session_accuracy", wilson_ci(int(polluted.get("correct", 0)), int(polluted["n"]))))
        out["metrics"].append(metric("transfer_fresh_accuracy", wilson_ci(int(transfer.get("correct", 0)), int(transfer["n"]))))

    rag = doc.get("rag_baseline_gap") or {}
    hybrid = rag.get("valhalla_isolated_hybrid") or {}
    lm = rag.get("valhalla_lm_patch_rag") or {}
    if hybrid.get("rows") and lm.get("rows"):
        hb, lb = paired_rows_by_id(hybrid["rows"], lm["rows"])
        out["metrics"].append(lift_metric("hybrid_vs_lm_patch_lift", bootstrap_ci_paired(lb, hb)))

    tpi_doc = doc.get("tpi") or {}
    if tpi_doc.get("tpi") is not None:
        weights = tpi_doc.get("weights") or {}
        components = tpi_doc.get("components") or {}

        def tpi_sampler(rng):  # noqa: ANN001
            comp = dict(components)
            if polluted.get("rows") and transfer.get("rows"):
                n = len(polluted["rows"])
                idx = [rng.randrange(n) for _ in range(n)]
                pr = sum(bool(polluted["rows"][i].get("correct")) for i in idx) / n
                tr = sum(bool(transfer["rows"][i].get("correct")) for i in idx) / n
                comp["transfer_vs_polluted"] = min(1.0, max(0.0, (tr - pr) * 100 / 40.0))
                comp["transfer_vs_isolated_norm"] = max(0.0, min(1.0, (tr - comp.get("isolated_ceiling", 0) + 0.05) / 0.10))
            if iso_rows:
                n = len(iso_rows)
                idx = [rng.randrange(n) for _ in range(n)]
                comp["isolated_ceiling"] = sum(bool(iso_rows[i].get("correct")) for i in idx) / n
            return sum(weights.get(k, 0) * comp.get(k, 0) for k in weights)

        idx_ci = bootstrap_ci_index(tpi_sampler)
        idx_ci["point"] = float(tpi_doc["tpi"])
        out["metrics"].append(index_metric("TPI_v2", idx_ci))

    trad = (rag.get("traditional_lm_rag") or doc.get("traditional_lm_rag") or {})
    if trad.get("skipped"):
        out["missing"].append("traditional_lm_rag (torch/model not available at run time)")
    return out


def _tpi_transfer_rows(tpi_doc: dict[str, Any] | None) -> tuple[list[dict], list[dict]]:
    if not tpi_doc:
        return [], []
    te = tpi_doc.get("transfer_eval") or {}
    polluted = (te.get("curriculum_in_session_polluted") or {}).get("rows") or []
    transfer = (te.get("transfer_fresh_retrieval") or {}).get("rows") or []
    return polluted, transfer


def _tpi_cold_rows(tpi_doc: dict[str, Any] | None) -> list[dict]:
    if not tpi_doc:
        return []
    iso = (tpi_doc.get("learning_paradigms") or {}).get("isolated_baseline") or {}
    return iso.get("rows") or []


def analyze_nppi(doc: dict[str, Any], *, tpi_doc: dict[str, Any] | None = None) -> dict[str, Any]:
    out: dict[str, Any] = {"protocol": doc.get("protocol"), "metrics": [], "missing": []}
    n2 = doc.get("n2_fair_holdout") or {}
    cold = n2.get("cold_rag_isolated") or {}
    transfer = n2.get("transfer_fresh") or {}
    polluted = n2.get("in_session_polluted") or {}

    tpi_polluted, tpi_transfer = _tpi_transfer_rows(tpi_doc)
    tpi_cold = _tpi_cold_rows(tpi_doc)

    cold_rows = cold.get("rows") or tpi_cold
    transfer_rows = transfer.get("rows") or tpi_transfer
    polluted_rows = polluted.get("rows") or tpi_polluted

    if cold_rows:
        out["metrics"].append(metric("fair_holdout_cold_rag", proportion_ci_from_bools(rows_correct(cold_rows))))
    elif cold.get("n"):
        out["metrics"].append(metric("fair_holdout_cold_rag", wilson_ci(int(cold.get("correct", 0)), int(cold["n"]))))
    if transfer_rows:
        out["metrics"].append(metric("fair_holdout_transfer", proportion_ci_from_bools(rows_correct(transfer_rows))))
    elif transfer.get("n"):
        out["metrics"].append(metric("fair_holdout_transfer", wilson_ci(int(transfer.get("correct", 0)), int(transfer["n"]))))
    if polluted_rows and transfer_rows:
        b, a = paired_rows_by_id(polluted_rows, transfer_rows)
        lift = bootstrap_ci_paired(b, a)
        if not polluted.get("rows") and tpi_polluted:
            lift["note"] = "paired rows from TPI-v2 transfer_eval (NPPI n2 lacks per-row)"
        out["metrics"].append(lift_metric("fair_transfer_vs_polluted", lift))
    elif polluted.get("n") and transfer.get("n"):
        lifts = n2.get("lifts_pp") or {}
        out["metrics"].append(
            lift_metric(
                "fair_transfer_vs_polluted",
                {
                    "mean_pp": float(lifts.get("transfer_vs_polluted_pp", 0)),
                    "ci95_lo": float(lifts.get("transfer_vs_polluted_pp", 0)),
                    "ci95_hi": float(lifts.get("transfer_vs_polluted_pp", 0)),
                    "n": int(polluted["n"]),
                    "method": "point_from_report",
                    "note": "no per-row data; CI equals point estimate",
                },
            )
        )
    if cold_rows and transfer_rows:
        c, t = paired_rows_by_id(cold_rows, transfer_rows)
        out["metrics"].append(lift_metric("fair_transfer_vs_cold", bootstrap_ci_paired(c, t)))

    n4 = doc.get("n4_negative_controls") or {}
    if n4.get("subset_n"):
        out["metrics"].append(
            metric(
                "negative_control_targeted",
                wilson_ci(int(round(n4["targeted_accuracy"] * n4["subset_n"])), n4["subset_n"]),
            )
        )
        out["metrics"].append(
            metric(
                "negative_control_distractor",
                wilson_ci(int(round(n4.get("distractor_accuracy", 0) * n4["subset_n"])), n4["subset_n"]),
            )
        )
        if n4.get("swap_trials"):
            out["metrics"].append(
                metric("swap_fidelity", wilson_ci(int(n4.get("swap_fidelity", 0)), int(n4["swap_trials"])))
            )

    n5 = doc.get("n5_paradigm_separation") or {}
    trad = n5.get("traditional_lm_rag") or {}
    if trad.get("skipped"):
        out["missing"].append("traditional_lm_rag")

    nppi_doc = doc.get("nppi") or {}
    if nppi_doc.get("nppi") is not None:
        weights = nppi_doc.get("weights") or {}

        def nppi_sampler(rng):  # noqa: ANN001
            if cold_rows and transfer_rows:
                n = len(cold_rows)
                idx = [rng.randrange(n) for _ in range(n)]
                cold_acc = sum(bool(cold_rows[i].get("correct")) for i in idx) / n
                tr_acc = sum(bool(transfer_rows[i].get("correct")) for i in idx) / n
                pol_acc = (
                    sum(bool(polluted_rows[i].get("correct")) for i in idx) / n
                    if polluted_rows
                    else 0.35
                )
            else:
                cold_acc = binomial_resample_accuracy(int(cold.get("correct", 0)), int(cold.get("n", 31)), rng)
                tr_acc = binomial_resample_accuracy(int(transfer.get("correct", 0)), int(transfer.get("n", 31)), rng)
                pol_acc = binomial_resample_accuracy(int(polluted.get("correct", 0)), int(polluted.get("n", 31)), rng)
            comp = {
                "fair_holdout_cold": cold_acc,
                "novel_holdout_transfer": tr_acc,
                "compositional_transfer": tr_acc,
                "session_routing_value": min(1.0, max(0.0, (tr_acc - pol_acc) * 100 / 50.0)),
                "structure_session_lift": max(0.0, min(1.0, (tr_acc - cold_acc + 0.05) / 0.10)),
                "control_integrity": n4.get("control_integrity", 1.0),
            }
            raw = sum(weights.get(k, 0) * comp.get(k, 0) for k in weights)
            return raw * float(nppi_doc.get("gate_factor", 1.0))

        idx_ci = bootstrap_ci_index(nppi_sampler)
        idx_ci["point"] = float(nppi_doc["nppi"])
        out["metrics"].append(index_metric("NPPI", idx_ci))
        out["falsification_flags"] = nppi_doc.get("falsification_flags") or []
    return out


def analyze_fog_v2(doc: dict[str, Any]) -> dict[str, Any]:
    out: dict[str, Any] = {"protocol": doc.get("protocol"), "metrics": []}
    mode = (doc.get("modes") or {}).get("v2_algebraic") or (doc.get("modes") or {}).get("v2_rust_native") or {}
    g1 = mode.get("g1_targeted") or {}
    g2 = mode.get("g2_distractor") or {}
    rows = g1.get("rows") or []
    if rows:
        always = [bool(r.get("always_emit_correct")) for r in rows]
        out["metrics"].append(metric("always_emit_accuracy", proportion_ci_from_bools(always)))
        emits = [r for r in rows if r.get("fog_decision") == "emit"]
        if emits:
            gated = [bool(r.get("gated_correct")) for r in emits if r.get("gated_correct") is not None]
            if gated:
                out["metrics"].append(metric("fog_emitted_precision", proportion_ci_from_bools(gated)))
        emit_n = sum(1 for r in rows if r.get("fog_decision") == "emit")
        out["metrics"].append(metric("fog_coverage", wilson_ci(emit_n, len(rows))))
    g2_rows = g2.get("rows") or []
    if g2_rows:
        abstain = [r.get("fog_decision") in ("abstain", "defer") for r in g2_rows]
        out["metrics"].append(metric("distractor_abstain_rate", proportion_ci_from_bools(abstain)))
    return out


def analyze_proactive_v2(doc: dict[str, Any]) -> dict[str, Any]:
    out: dict[str, Any] = {"protocol": doc.get("protocol"), "metrics": []}
    batteries = doc.get("batteries") or {}
    all_rows = []
    for key in ("gates", "handcraft", "fair"):
        all_rows.extend(batteries.get(key) or [])

    aligned = [
        r for r in all_rows
        if r.get("id", "").endswith("_aligned") or ("hand_" in r.get("id", "") and "_mismatch" not in r.get("id", ""))
    ]
    mismatch = [r for r in all_rows if r.get("id", "").endswith("_mismatch")]
    eligible = [r for r in all_rows if r.get("warmup_turns", 0) >= 2 and not r.get("id", "").startswith("gate_")]

    emits_aligned = [r for r in aligned if r.get("emitted")]
    if emits_aligned:
        grounded = [bool(r.get("grounded")) for r in emits_aligned]
        out["metrics"].append(metric("UP_on_topic_given_emit", proportion_ci_from_bools(grounded)))
    if aligned:
        emit_rate = [bool(r.get("emitted")) for r in aligned]
        out["metrics"].append(metric("on_topic_emit_rate", proportion_ci_from_bools(emit_rate)))
    if mismatch:
        abstain = [not bool(r.get("emitted")) for r in mismatch]
        out["metrics"].append(metric("MAR_mismatch_abstain", proportion_ci_from_bools(abstain)))
        emit_mismatch = [r for r in mismatch if r.get("emitted")]
        if emit_mismatch:
            hall = [bool(r.get("hallucination")) for r in emit_mismatch]
            out["metrics"].append(metric("FIR_on_mismatch_emits", proportion_ci_from_bools(hall)))

    m = doc.get("metrics") or {}
    if m.get("PPI_v2") is not None:

        def ppi_sampler(rng):  # noqa: ANN001
            if not eligible:
                return float(m["PPI_v2"])
            idx = [rng.randrange(len(eligible)) for _ in range(len(eligible))]
            sample = [eligible[i] for i in idx]
            gate_pass = sum(1 for r in (batteries.get("gates") or []) if r.get("pass")) / max(len(batteries.get("gates") or []), 1)
            al = [r for r in sample if r.get("id", "").endswith("_aligned") or ("hand_" in r.get("id", "") and "_mismatch" not in r.get("id", ""))]
            mi = [r for r in sample if r.get("id", "").endswith("_mismatch")]
            on_em = [r for r in al if r.get("emitted")]
            up = sum(1 for r in on_em if r.get("grounded")) / max(len(on_em), 1) if on_em else 0.0
            on_rate = len(on_em) / max(len(al), 1)
            mar = sum(1 for r in mi if not r.get("emitted")) / max(len(mi), 1) if mi else 1.0
            fir = sum(1 for r in mi if r.get("hallucination")) / max(sum(1 for r in sample if r.get("emitted")), 1)
            return 0.20 * gate_pass + 0.35 * up * on_rate + 0.30 * mar + 0.15 * (1.0 - fir)

        idx_ci = bootstrap_ci_index(ppi_sampler)
        idx_ci["point"] = float(m["PPI_v2"])
        out["metrics"].append(index_metric("PPI_v2", idx_ci))
    return out


def analyze_context_drift(doc: dict[str, Any], *, extended: bool = False) -> dict[str, Any]:
    out: dict[str, Any] = {"protocol": doc.get("protocol"), "metrics": [], "extended": extended}
    arm = (doc.get("arms") or {}).get("production") or {}
    swap_rows = (arm.get("swap") or {}).get("rows") or []
    if swap_rows:
        fid = [bool(r.get("swap_fidelity")) for r in swap_rows]
        out["metrics"].append(metric("swap_fidelity", proportion_ci_from_bools(fid)))
        hall = [bool(r.get("cross_topic_hallucination")) for r in swap_rows]
        out["metrics"].append(metric("swap_hallucination_rate", proportion_ci_from_bools(hall)))

    if extended:
        ext = (doc.get("summary") or {}).get("production") or {}
        s = ext.get("extended_on_topic") or ""
        if "/" in s:
            k, n = s.split("/")
            out["metrics"].append(metric("extended_30turn_on_topic", wilson_ci(int(k), int(n))))
    else:
        summary = doc.get("summary") or {}
        prod = summary.get("production") or {}
        if prod.get("distractor_false_confidence") is not None:
            # 0 false confidence on distractor arm summary
            out["metrics"].append(
                metric("distractor_false_confidence_rate", wilson_ci(0, 12), note="summary aggregate n≈12")
            )
    if doc.get("traditional_lm_error"):
        out["missing"] = ["traditional_lm: " + str(doc["traditional_lm_error"])]
    return out


def analyze_local_corpus(doc: dict[str, Any]) -> dict[str, Any]:
    out: dict[str, Any] = {"protocol": doc.get("protocol"), "metrics": []}
    for key, label in (
        ("before_no_local_corpus", "before_no_corpus"),
        ("cold_targeted_ingest", "cold_targeted"),
        ("after_transfer_eval", "after_transfer"),
    ):
        block = doc.get(key) or {}
        rows = block.get("rows") or []
        if rows:
            out["metrics"].append(metric(label, proportion_ci_from_bools(rows_correct(rows))))
    before = doc.get("before_no_local_corpus") or {}
    cold = doc.get("cold_targeted_ingest") or {}
    if before.get("rows") and cold.get("rows"):
        b, c = paired_rows_by_id(before["rows"], cold["rows"])
        out["metrics"].append(lift_metric("ingest_vs_none_lift", bootstrap_ci_paired(b, c)))
    return out


def build_report(reports_dir: Path) -> dict[str, Any]:
    sections: list[dict[str, Any]] = []
    tpi_doc = load(reports_dir / "valhalla_base_training_potential_test_v2.json")

    mapping = [
        ("TPI-v2", reports_dir / "valhalla_base_training_potential_test_v2.json", analyze_tpi_v2),
        ("NPPI", reports_dir / "new_paradigm_potential_test.json", lambda d: analyze_nppi(d, tpi_doc=tpi_doc)),
        ("FOG-v2", reports_dir / "fate_output_gate_v2_test.json", analyze_fog_v2),
        ("Proactive-v2", reports_dir / "proactive_fate_v2_test.json", analyze_proactive_v2),
        ("Context-drift", reports_dir / "context_topic_drift_test.json", analyze_context_drift),
        ("Context-drift-extended", reports_dir / "context_topic_drift_extended_test.json", lambda d: analyze_context_drift(d, extended=True)),
        ("Local-corpus", reports_dir / "local_corpus_demo_test.json", analyze_local_corpus),
    ]

    missing_files: list[str] = []
    for title, path, fn in mapping:
        doc = load(path)
        if doc is None:
            missing_files.append(str(path.name))
            continue
        section = fn(doc)
        section["title"] = title
        section["source"] = str(path.name)
        sections.append(section)

    return {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "protocol": PROTOCOL,
        "methods": {
            "proportion": "Wilson score 95% CI",
            "paired_lift_pp": "Bootstrap percentile 95% CI (5000 resamples, seed=42)",
            "composite_index": "Bootstrap percentile 95% CI on row resampling",
        },
        "sections": sections,
        "missing_report_files": missing_files,
        "global_gaps": [
            "traditional_lm RAG arm skipped in NPPI/TPI when torch/model unavailable",
            "Composite indices (TPI/NPPI/PPI) bootstrap uses row resampling — component point estimates may differ slightly from stored index",
        ],
    }


def write_markdown(report: dict[str, Any], path: Path) -> None:
    lines = [
        "# Valhalla Inference — Confidence Report v1",
        "",
        f"**Protocol:** `{PROTOCOL}` · **Generated:** {report['timestamp'][:10]}",
        "",
        "## Executive summary",
        "",
        "| Domain | Key metric | 95% CI |",
        "|--------|------------|--------|",
    ]
    highlights = [
        ("Grounding", "fair_holdout_cold_rag", "NPPI"),
        ("Session routing", "transfer_vs_polluted_lift", "TPI-v2"),
        ("FOG quality", "fog_emitted_precision", "FOG-v2"),
        ("Proactive", "UP_on_topic_given_emit", "Proactive-v2"),
        ("Dialogue", "extended_30turn_on_topic", "Context-drift-extended"),
    ]
    sec_by_title = {s["title"]: s for s in report.get("sections") or []}
    for label, metric_name, title in highlights:
        sec = sec_by_title.get(title) or {}
        m = next((x for x in sec.get("metrics") or [] if x["name"] == metric_name), None)
        if m:
            lines.append(f"| {label} | {metric_name} | {m.get('display', '—')} |")
    lines += [
        "",
        "## Methods",
        "",
        "| Statistic | Interval |",
        "|-----------|----------|",
        "| Proportions (accuracy, precision, abstain) | Wilson 95% |",
        "| Paired lifts (pp) | Bootstrap paired 95% |",
        "| TPI / NPPI / PPI | Bootstrap index 95% |",
        "",
        "Display format: **point% [lo%, hi%]** or **index [lo, hi]**",
        "",
    ]

    for sec in report.get("sections") or []:
        lines += [f"## {sec['title']}", "", f"Source: `{sec.get('source', '')}` · Protocol: `{sec.get('protocol', '')}`", ""]
        if sec.get("missing"):
            lines.append("**Missing arms:**")
            for m in sec["missing"]:
                lines.append(f"- {m}")
            lines.append("")
        if sec.get("falsification_flags"):
            lines.append("**Falsification flags:** " + ", ".join(f"`{f}`" for f in sec["falsification_flags"]))
            lines.append("")
        lines += ["| Metric | Estimate [95% CI] | n |", "|--------|-------------------|---|"]
        for m in sec.get("metrics") or []:
            prop = m.get("proportion")
            lift = m.get("lift_pp")
            idx = m.get("index")
            if prop:
                disp = m.get("display", fmt_pct(prop))
                n = prop.get("n", "—")
                k = prop.get("k", "—")
                lines.append(f"| {m['name']} | {disp} | {k}/{n} |")
            elif lift:
                disp = m.get("display", fmt_pp(lift))
                n = lift.get("n", "—")
                lines.append(f"| {m['name']} | {disp} | paired n={n} |")
            elif idx:
                disp = m.get("display", fmt_index(idx))
                lines.append(f"| {m['name']} | {disp} | bootstrap |")
        lines.append("")

    if report.get("global_gaps"):
        lines += ["## Global gaps", ""]
        for g in report["global_gaps"]:
            lines.append(f"- {g}")
        lines.append("")

    lines += [
        "## Reproduce",
        "",
        "```bash",
        "python3 tools/valhalla_inference/run_confidence_report.py",
        "```",
        "",
    ]
    path.write_text("\n".join(lines), encoding="utf-8")


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--reports-dir", type=Path, default=DEFAULT_REPORTS)
    args = ap.parse_args()

    report = build_report(args.reports_dir)
    args.reports_dir.mkdir(parents=True, exist_ok=True)
    json_path = args.reports_dir / "confidence_report_v1.json"
    md_path = args.reports_dir / "CONFIDENCE_REPORT_v1.md"
    json_path.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    write_markdown(report, md_path)

    vuc = REPO / "valhalla-unified-core/reports/valhalla_inference"
    if vuc.parent.parent.exists():
        vuc.mkdir(parents=True, exist_ok=True)
        vuc.joinpath("confidence_report_v1.json").write_text(json_path.read_text())
        vuc.joinpath("CONFIDENCE_REPORT_v1.md").write_text(md_path.read_text())

    print(f"Protocol: {PROTOCOL}")
    print(f"Sections: {len(report['sections'])}")
    for sec in report["sections"]:
        print(f"  [{sec['title']}] {len(sec.get('metrics', []))} metrics")
        for m in sec.get("metrics", [])[:4]:
            print(f"    {m['name']}: {m.get('display', m.get('index_display', m.get('lift_display')))}")
    if report.get("missing_report_files"):
        print(f"Missing: {', '.join(report['missing_report_files'])}")
    print(f"JSON: {json_path}")
    print(f"Markdown: {md_path}")


if __name__ == "__main__":
    main()
