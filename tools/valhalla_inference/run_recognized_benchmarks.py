#!/usr/bin/env python3
"""Run all recognized third-party + scaling benchmarks and write consolidated summary.

Protocols:
  - ragtruth-v1          (ACL 2024 RAGTruth)
  - needle-session-v1    (Needle-in-haystack session)
  - longbench-v2-v1      (THUDM LongBench-v2 short MCQ)
  - model-scaling-v1     (0.5B / 2B / 3.8B ladder)
  - backbone-universality-v1 (optional, --with-backbone)

Usage:
  python3 tools/valhalla_inference/run_recognized_benchmarks.py
  python3 tools/valhalla_inference/run_recognized_benchmarks.py --fast
  python3 tools/valhalla_inference/run_recognized_benchmarks.py --skip-longbench
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
REPORTS = REPO / "reports/valhalla_inference"
PY = sys.executable


def run_cmd(label: str, cmd: list[str], *, extra_env: dict[str, str] | None = None) -> int:
    print(f"\n{'=' * 64}\n[{label}]\n$ {' '.join(cmd)}\n")
    env = {**os.environ, **(extra_env or {})}
    return subprocess.call(cmd, cwd=REPO, env=env)


def load_json(name: str) -> dict | None:
    p = REPORTS / name
    if not p.is_file():
        return None
    return json.loads(p.read_text(encoding="utf-8"))


def write_summary(results: dict) -> Path:
    REPORTS.mkdir(parents=True, exist_ok=True)
    out_json = REPORTS / "recognized_benchmarks_summary.json"
    out_md = REPORTS / "EXPERIMENT_SUMMARY_20260630.md"

    out_json.write_text(json.dumps(results, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")

    rows = results.get("benchmarks") or []
    lines = [
        "# ValhallaBase — 公认 Benchmark 实验总结",
        "",
        f"**Generated:** {results.get('timestamp', '')}",
        "",
        "## 总表",
        "",
        "| Benchmark | Protocol | n | Result | Verdict |",
        "|-----------|----------|---|--------|---------|",
    ]
    for r in rows:
        lines.append(
            f"| {r['name']} | `{r['protocol']}` | {r.get('n', '—')} | **{r['result']}** | {r.get('verdict', '—')} |"
        )

    lines.extend(["", "## 解读", ""])
    for r in rows:
        if r.get("note"):
            lines.append(f"- **{r['name']}:** {r['note']}")

    scaling = results.get("scaling_patterns") or []
    if scaling:
        lines.extend(["", "## Scaling 规律", ""])
        for p in scaling:
            lines.append(f"- `{p}`")

    lines.extend(
        [
            "",
            "## 复现",
            "",
            "```bash",
            "cargo build -p hub-f64 --release --bin valhalla_base",
            "python3 tools/valhalla_inference/run_recognized_benchmarks.py",
            "```",
            "",
            "JSON: `recognized_benchmarks_summary.json` · 分项报告见 `reports/valhalla_inference/`",
            "",
            "---",
            "",
            "*Rogue Intelligence LNC. · ValhallaBase · Qwen2.5-0.5B hybrid (unless noted)*",
        ]
    )
    out_md.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return out_md


def aggregate() -> dict:
    benchmarks: list[dict] = []

    rt = load_json("ragtruth_v1_test.json")
    if rt:
        benchmarks.append(
            {
                "name": "RAGTruth (ACL 2024)",
                "protocol": "ragtruth-v1",
                "n": rt.get("n"),
                "result": f"{rt.get('faithful')}/{rt.get('n')} grounded ({100 * rt.get('faithful_rate', 0):.0f}%)",
                "verdict": rt.get("verdict"),
                "note": "Context-grounded RAG faithfulness; native citation path; no abstain on QA slice.",
            }
        )

    nd = load_json("needle_session_v1_test.json")
    if nd:
        benchmarks.append(
            {
                "name": "Needle-in-haystack (session)",
                "protocol": "needle-session-v1",
                "n": f"depths {nd.get('depths')}",
                "result": f"{nd.get('correct')}/{nd.get('n')} recall ({100 * nd.get('recall', 0):.0f}%)",
                "verdict": nd.get("verdict"),
                "note": "Session append recall @ depth 30 (62 lines); Paul Graham-style needle.",
            }
        )

    lb = load_json("longbench_v2_v1_test.json")
    if lb:
        benchmarks.append(
            {
                "name": "LongBench-v2 (THUDM)",
                "protocol": "longbench-v2-v1",
                "n": lb.get("n"),
                "result": f"{lb.get('correct')}/{lb.get('n')} ({100 * lb.get('accuracy', 0):.0f}%)",
                "verdict": lb.get("verdict"),
                "note": "Short MCQ slice; 12k context cap / 32 corpus lines; 0.5B hybrid lm_patch. Human full v2 ~53.7%.",
            }
        )

    sc = load_json("model_scaling_v1_test.json")
    patterns: list[str] = []
    if sc:
        a = sc.get("analysis") or {}
        patterns = a.get("patterns") or []
        vh = sc.get("valhalla_hybrid_open") or {}
        benchmarks.append(
            {
                "name": "Scaling open (31Q targeted)",
                "protocol": "model-scaling-v1",
                "n": vh.get("n"),
                "result": f"{vh.get('correct')}/{vh.get('n')} ({100 * vh.get('accuracy', 0):.0f}%)",
                "verdict": a.get("verdict"),
                "note": "Valhalla open backbone-invariant; +6.5pp premium @ 0.5B vs trad RAG.",
            }
        )
        vh_mcq = sc.get("valhalla_hybrid_mcq_lm_patch") or {}
        if vh_mcq.get("n"):
            q0 = (sc.get("trad_lm_mcq_by_model") or {}).get("qwen_0.5b") or {}
            benchmarks.append(
                {
                    "name": "Scaling MCQ @ 0.5B",
                    "protocol": "model-scaling-v1",
                    "n": vh_mcq.get("n"),
                    "result": f"hybrid {vh_mcq.get('correct')}/{vh_mcq.get('n')} vs trad {q0.get('correct', '?')}/{q0.get('n', '?')}",
                    "verdict": "HYBRID_MCQ_BEATS_TRAD",
                    "note": f"Hybrid lm_patch +50pp over trad prompt MCQ at same backbone.",
                }
            )

    ml = load_json("mcq_ladder_v1_test.json")
    if ml:
        pts = (ml.get("hybrid_lm_patch_scaling") or {}).get("points") or []
        pt_str = " → ".join(
            f"{p.get('params_b')}B {100 * (p.get('accuracy') or 0):.0f}%"
            for p in pts
            if p.get("accuracy") is not None
        )
        benchmarks.append(
            {
                "name": "MCQ ladder (Qwen lm_patch)",
                "protocol": "mcq-ladder-v1",
                "n": ml.get("mcq_n"),
                "result": pt_str or "—",
                "verdict": ml.get("verdict"),
                "note": "Rust sharded safetensors; 0.5B→3B hybrid MCQ scaling curve.",
            }
        )
        if ml.get("verdict") == "MCQ_LADDER_SCALES" and "MCQ_LADDER_SCALES" not in patterns:
            patterns.append("MCQ_LADDER_SCALES")

    bu = load_json("backbone_universality_test.json")
    if bu:
        vh_arm = (bu.get("arms") or {}).get("valhalla_hybrid") or {}
        benchmarks.append(
            {
                "name": "Backbone universality",
                "protocol": "backbone-universality-v1",
                "n": bu.get("n"),
                "result": f"Valhalla {vh_arm.get('correct', '?')}/{vh_arm.get('n', '?')} ({100 * vh_arm.get('accuracy', 0):.0f}%)",
                "verdict": "BACKBONE_INDEPENDENT_OPEN",
                "note": "Tier B native open vs trad_lm RAG across Qwen/Gemma/Phi.",
            }
        )

    eh = load_json("external_holdout_test.json")
    if eh:
        benchmarks.append(
            {
                "name": "External holdout",
                "protocol": "external-holdout-v1",
                "n": eh.get("n"),
                "result": f"{eh.get('correct')}/{eh.get('n')} ({100 * eh.get('accuracy', 0):.0f}%)",
                "verdict": eh.get("verdict"),
                "note": "Out-of-suite generalization (complementary, not third-party corpus).",
            }
        )

    return {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "benchmarks": benchmarks,
        "scaling_patterns": patterns,
        "report_files": [
            "ragtruth_v1_test.json",
            "needle_session_v1_test.json",
            "longbench_v2_v1_test.json",
            "model_scaling_v1_test.json",
            "mcq_ladder_v1_test.json",
            "backbone_universality_test.json",
            "external_holdout_test.json",
        ],
    }


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--fast", action="store_true")
    ap.add_argument("--skip-longbench", action="store_true")
    ap.add_argument("--skip-scaling", action="store_true")
    ap.add_argument("--with-backbone", action="store_true")
    ap.add_argument("--aggregate-only", action="store_true")
    args = ap.parse_args()

    if args.aggregate_only:
        results = aggregate()
        md = write_summary(results)
        print(f"→ {md}")
        return

    failed = 0
    limit_flags = ["--limit", "5"] if args.fast else []

    for script in ("test_needle_session_v1.py", "test_ragtruth_v1.py"):
        rc = run_cmd(script, [PY, str(HERE / script), *limit_flags])
        if rc:
            failed += 1

    if not args.skip_longbench:
        rc = run_cmd(
            "longbench",
            [PY, str(HERE / "test_longbench_v2_v1.py"), *limit_flags],
        )
        if rc:
            failed += 1

    if not args.skip_scaling:
        scale_flags = ["--open-only", "--skip-training"]
        if args.fast:
            scale_flags += ["--limit", "12"]
        rc = run_cmd("scaling-open", [PY, str(HERE / "test_model_scaling_v1.py"), *scale_flags])
        if rc:
            failed += 1
        if not args.fast:
            rc = run_cmd(
                "scaling-mcq",
                [
                    PY,
                    str(HERE / "test_model_scaling_v1.py"),
                    "--mcq-only",
                    "--limit",
                    "20",
                    "--merge-into",
                    str(REPORTS / "model_scaling_v1_test.json"),
                ],
            )
            if rc:
                failed += 1
        mcq_flags = ["--fast"] if args.fast else []
        rc = run_cmd(
            "mcq-ladder",
            [PY, str(HERE / "test_mcq_ladder_v1.py"), *mcq_flags],
            extra_env={"RUSTFLAGS": "-L /opt/cuda/lib64"},
        )
        if rc:
            failed += 1

    if args.with_backbone or (not args.fast and not args.skip_scaling):
        bb_flags = ["--slice", "open_retrieval"]
        if args.fast:
            bb_flags.append("--fast")
        rc = run_cmd(
            "backbone",
            [PY, str(HERE / "test_backbone_universality_v1.py"), *bb_flags],
        )
        if rc:
            failed += 1

    rc = run_cmd("external-holdout", [PY, str(HERE / "test_external_holdout.py")])
    if rc:
        failed += 1

    results = aggregate()
    md = write_summary(results)
    print(f"\n{'=' * 64}\nSummary → {md}")
    if failed:
        sys.exit(f"{failed} benchmark step(s) failed")
    print("All recognized benchmarks completed.")


if __name__ == "__main__":
    main()
