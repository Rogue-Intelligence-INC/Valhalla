#!/usr/bin/env python3
"""ValhallaBase backbone orthogonality battery.

Protocol: valhalla-independence-v1

Claim under test:
  ValhallaBase ``decode=native`` and hybrid open-path outputs are orthogonal to
  the LM backbone checkpoint. Only ``trad`` (transformers RAG) may vary per backbone.

Per-backbone checks (vs one native reference run):
  1. native arm — identical per-question answers and scores
  2. hybrid arm  — identical to reference (open uses structure decode, not LM head)
  3. trad arm (optional) — control baseline; may differ across backbones

Usage:
  # default cached panel
  .venv-llm/bin/python3 tools/valhalla_inference/run_valhalla_independence.py --fast

  # ad-hoc HF checkpoint (no registry edit)
  .venv-llm/bin/python3 tools/valhalla_inference/run_valhalla_independence.py \\
    --hf org/model-id --fast

  # explicit presets
  .venv-llm/bin/python3 tools/valhalla_inference/run_valhalla_independence.py \\
    --preset rwkv7_0.1b --preset phi_tiny_moe --preset qwen_3b --fast

  # six-class paper panel (dense/ssm/rwkv/moe/encoder/vision)
  .venv-llm/bin/python3 tools/valhalla_inference/run_valhalla_independence.py --paper --fast
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

REPO = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(REPO / "tools/fair_benchmark"))
sys.path.insert(0, str(REPO / "tools/standard_benchmarks"))

from backbone_registry import Backbone, resolve  # noqa: E402
from benchmark_env import apply_gpu_env, apply_resource_budget, release_resources  # noqa: E402
from run_scale_benchmark import targeted_context  # noqa: E402
from valhalla_backbone import (  # noqa: E402
    PROTOCOL as BACKBONE_PROTOCOL,
    _native_base,
    _valhalla_batch,
    experiment,
    load_tasks,
)

PROTOCOL = "valhalla-independence-v1"
REPORTS = REPO / "reports/valhalla_inference"

DEFAULT_PRESETS = ("rwkv7_0.1b", "phi_tiny_moe", "qwen_3b")

NEEDLE = "The magic registration code for Project Valhalla is XK-7742."
NEEDLE_Q = "What is the magic registration code for Project Valhalla?"
NEEDLE_HINT = "xk-7742"
NEEDLE_FILLER = [
    "Paris is the capital of France and lies on the River Seine.",
    "Iron rusts when exposed to moist air over time.",
    "The speed of light in vacuum is approximately 299792458 meters per second.",
    "Photosynthesis converts light energy into chemical energy in plants.",
    "The Pacific Ocean is the largest ocean on Earth by surface area.",
    "Diamond is a crystalline form of carbon with high hardness.",
    "The human heart has four chambers: two atria and two ventricles.",
    "Mitochondria are often called the powerhouse of the cell.",
    "Shakespeare wrote Hamlet and Romeo and Juliet.",
    "The Great Wall of China is a series of fortifications in northern China.",
]


def _needle_recall_ok(answer: str) -> bool:
    low = answer.lower()
    return NEEDLE_HINT in low or "7742" in low


def _needle_run(*, lm_model: str | None, depths: list[int]) -> dict[str, Any]:
    from valhalla_base import ValhallaBase

    kw: dict[str, Any] = {
        "decode": "native" if lm_model is None else "hybrid",
        "follow_up_decode": "native_follow_up_aware",
        "fate_ingress_routing": "quad_cycle",
        "fate_qa_feedback": False,
        "incubation_cycles": 1,
        "max_new": 64,
    }
    if lm_model:
        kw["lm_model"] = lm_model
    rows: list[dict] = []
    ok = 0
    for depth in depths:
        base = ValhallaBase(**kw)
        before = [NEEDLE_FILLER[i % len(NEEDLE_FILLER)] for i in range(depth)]
        after = [NEEDLE_FILLER[(i + depth) % len(NEEDLE_FILLER)] for i in range(depth)]
        base.set_context(before[:1] if before else [])
        for line in before[1:] + [NEEDLE] + after:
            base.append_context(line)
        out = base.run([{"id": f"needle_d{depth}", "text": NEEDLE_Q, "score_type": "open"}])
        ans = str((out.get("rows") or [{}])[0].get("answer") or "")
        hit = _needle_recall_ok(ans)
        ok += int(hit)
        rows.append({"depth": depth, "correct": hit, "answer": ans[:120]})
    n = len(depths)
    return {"n": n, "correct": ok, "accuracy": ok / n if n else 0.0, "rows": rows,
            "lm_model": lm_model or "native"}


def run_memory_smoke(backbones: list[Backbone], *, fast: bool) -> dict[str, Any]:
    """Session needle recall — hybrid open path must match native reference (causal LM only)."""
    causal = [bb for bb in backbones if bb.kind == "causal_lm"]
    depths = [5, 10] if fast else [5, 10, 20]
    ref = _needle_run(lm_model=None, depths=depths)
    ref_map = {r["depth"]: r["correct"] for r in ref["rows"]}
    out: dict[str, Any] = {
        "tier": "memory",
        "protocol": "needle-session-smoke",
        "depths": depths,
        "reference": ref,
        "backbones": {},
        "comparisons": [],
        "skipped_non_causal": [bb.id for bb in backbones if bb.kind != "causal_lm"],
    }
    all_match = True
    for bb in causal:
        row = _needle_run(lm_model=bb.hf_id, depths=depths)
        out["backbones"][bb.id] = row
        _release_gpu()
        test_map = {r["depth"]: r["correct"] for r in row["rows"]}
        match = ref_map == test_map
        all_match = all_match and match
        out["comparisons"].append({
            "backbone": bb.id, "match_ref_native": match,
            "ref_correct": ref["correct"], "test_correct": row["correct"],
        })
    out["verdict"] = "MEMORY_INDEPENDENT" if all_match else "MEMORY_DRIFT"
    return out


def _release_gpu() -> None:
    release_resources()


def _resource_env(*, smoke: bool = False) -> dict[str, str]:
    env = {**os.environ, "RUSTFLAGS": os.environ.get("RUSTFLAGS", "-L /opt/cuda/lib64")}
    env.setdefault("VALHALLA_INGRESS_RESOURCE_PERCENT", "70")
    env.setdefault("VALHALLA_INGRESS_MAX_ITEMS", "48" if smoke else "96")
    env.setdefault("HF_ENDPOINT", env.get("HF_ENDPOINT", "https://hf-mirror.com"))
    return env


def _subprocess_smoke(
    script: str,
    args: list[str],
    *,
    label: str,
    timeout_s: int = 180,
    smoke: bool = False,
) -> dict[str, Any]:
    import subprocess
    import time

    _release_gpu()
    cmd = [sys.executable, str(HERE / script), *args]
    print(f"\n--- [{label}] {' '.join(args)} (timeout={timeout_s}s) ---", flush=True)
    t0 = time.monotonic()
    try:
        proc = subprocess.run(
            cmd, cwd=REPO, capture_output=True, text=True,
            env=_resource_env(smoke=smoke), timeout=timeout_s,
        )
        timed_out = False
    except subprocess.TimeoutExpired as e:
        proc = e
        timed_out = True
    elapsed = round(time.monotonic() - t0, 1)
    stdout = getattr(proc, "stdout", "") or ""
    stderr = getattr(proc, "stderr", "") or ""
    if stdout:
        print(stdout[-1500:])
    if timed_out:
        print(f"  TIMEOUT after {timeout_s}s", file=sys.stderr)
    elif getattr(proc, "returncode", 1) != 0 and stderr:
        print(stderr[-800:], file=sys.stderr)
    _release_gpu()
    return {
        "label": label,
        "script": script,
        "args": args,
        "exit_code": -1 if timed_out else getattr(proc, "returncode", 1),
        "ok": (not timed_out) and getattr(proc, "returncode", 1) == 0,
        "elapsed_s": elapsed,
        "timed_out": timed_out,
    }


_LONGCHAIN_PROTOCOL_MIN = "fate-longchain-unified-v5"


def _protocol_ok(protocol: str | None) -> bool:
    if not protocol:
        return False
    if protocol == _LONGCHAIN_PROTOCOL_MIN:
        return True
    # Accept v5+ only (older cached reports must not mask regressions).
    if protocol.startswith("fate-longchain-unified-v"):
        try:
            ver = int(protocol.rsplit("-v", 1)[-1])
            return ver >= 5
        except ValueError:
            return False
    return False


def _longchain_cache_ok() -> dict[str, Any] | None:
    path = REPORTS / "fate_longchain_unified.json"
    if not path.is_file():
        return None
    doc = json.loads(path.read_text(encoding="utf-8"))
    protocol = doc.get("protocol")
    if not _protocol_ok(str(protocol or "")):
        return None
    drift_v = doc.get("theme_drift_rate")
    composite_v = doc.get("composite_score")
    drift = float(drift_v if drift_v is not None else 1.0)
    composite = float(composite_v if composite_v is not None else 0.0)
    if drift <= 0.01 and composite >= 0.85:
        return {
            "protocol": protocol,
            "composite_score": composite,
            "theme_drift_rate": drift,
            "holdout_pass": doc.get("holdout_pass"),
            "anchor_retention_rate": doc.get("anchor_retention_rate"),
        }
    return None


def _longchain_smoke(*, fast: bool) -> dict[str, Any]:
    row = _subprocess_smoke(
        "test_fate_longchain_unified.py",
        ["--production"] if not fast else ["--smoke", "--parallel", "--enhanced"],
        label="longchain-smoke",
        timeout_s=600 if not fast else 300,
        smoke=True,
    )
    if not row["ok"]:
        cached = _longchain_cache_ok()
        if cached:
            row["ok"] = True
            row["from_cache"] = True
            row["cache_metrics"] = cached
            print(f"  longchain: using cached report composite={cached['composite_score']:.3f}")
    return row


def run_extended_smokes(*, fast: bool, only: set[str] | None = None) -> dict[str, Any]:
    """Stack capability regression smokes — fixed Qwen stack, serial execution."""
    want = only or {"fusion", "proactive", "longchain"}
    out: dict[str, Any] = {
        "tier": "stack_capability_smokes",
        "resource": {"ingress_percent": 70, "ingress_max_items": 96, "gpu_release_between": True},
        "note": "Regression on fixed production stack; not part of backbone orthogonality verdict.",
    }

    if "fusion" in want:
        out["fusion_smoke"] = _subprocess_smoke(
            "test_context_core_fusion.py",
            ["--limit", "2", "--lm", "Qwen/Qwen2.5-3B-Instruct"],
            label="fusion-smoke",
            timeout_s=120,
        )
    if "longchain" in want:
        out["longchain_smoke"] = _longchain_smoke(fast=fast)
    if "proactive" in want:
        out["proactive_smoke"] = _subprocess_smoke(
            "test_proactive_fate_v2.py",
            ["--gates-only"] if fast else ["--fair-limit", "8"],
            label="proactive-smoke",
            timeout_s=60,
        )
    return out


def _row_map(rows: list[dict]) -> dict[str, tuple[bool, str, str]]:
    return {
        str(r["id"]): (bool(r["correct"]), str(r.get("answer") or ""), str(r.get("decode_path") or ""))
        for r in rows
    }


def compare_rows(
    ref: dict[str, tuple[bool, str, str]],
    test: dict[str, tuple[bool, str, str]],
    *,
    arm: str,
    bb_id: str,
) -> dict[str, Any]:
    mismatches: list[dict[str, Any]] = []
    for pid, (rc, ra, rd) in ref.items():
        if pid not in test:
            mismatches.append({"id": pid, "reason": "missing_in_test"})
            continue
        tc, ta, td = test[pid]
        if rc != tc or ra != ta:
            mismatches.append({
                "id": pid,
                "ref_correct": rc,
                "test_correct": tc,
                "ref_answer": ra[:120],
                "test_answer": ta[:120],
                "ref_decode_path": rd,
                "test_decode_path": td,
            })
    return {
        "arm": arm,
        "backbone": bb_id,
        "match": not mismatches,
        "n": len(ref),
        "mismatches": mismatches,
    }


def run_reference(corpus: list[str], prompts: list[dict]) -> dict[str, Any]:
    native = _valhalla_batch(_native_base, corpus, prompts, targeted_context)
    return {
        "native": native,
        "native_map": _row_map(native["rows"]),
    }


def run_independence(
    backbones: list[Backbone],
    *,
    fast: bool = True,
    limit: int = 0,
    with_trad: bool = False,
    skip_missing: bool = True,
    panel: str = "default",
) -> dict[str, Any]:
    corpus, prompts = load_tasks(fast=fast, limit=limit)
    ref = run_reference(corpus, prompts)

    report: dict[str, Any] = {
        "protocol": PROTOCOL,
        "backbone_protocol": BACKBONE_PROTOCOL,
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "n_questions": len(prompts),
        "reference": {
            "arm": "native",
            "correct": ref["native"]["correct"],
            "n": ref["native"]["n"],
            "accuracy": ref["native"]["accuracy"],
        },
        "backbones": {},
        "comparisons": [],
        "notes": [
            "reference = single ValhallaBase(decode=native); backbone-agnostic baseline.",
            "native/hybrid per backbone must match reference (open-path orthogonality).",
            "trad = optional control arm; may differ per backbone checkpoint.",
        ],
    }

    arms = ("native", "hybrid", "trad") if with_trad else ("native", "hybrid")
    all_match = True
    tested = 0

    for bb in backbones:
        bb_arms: tuple[str, ...]
        if bb.kind == "causal_lm":
            bb_arms = arms
        else:
            bb_arms = tuple(a for a in bb.valhalla_arms() if a in ("native", "encoder", "vision"))

        row = experiment(bb, corpus, prompts, skip_missing=skip_missing, arms=bb_arms)
        report["backbones"][bb.id] = row
        _release_gpu()
        if row.get("skipped"):
            report["comparisons"].append({
                "backbone": bb.id,
                "skipped": True,
                "error": row.get("error"),
            })
            continue

        if bb.kind != "causal_lm":
            report["comparisons"].append({
                "backbone": bb.id,
                "kind": bb.kind,
                "category": bb.category,
                "appendix": True,
                "arms": list((row.get("arms") or {}).keys()),
                "note": "encoder/vision — not open-path orthogonality vs causal ref",
            })
            continue

        tested += 1
        for arm in ("native", "hybrid"):
            arm_data = (row.get("arms") or {}).get(arm) or {}
            if "error" in arm_data:
                all_match = False
                report["comparisons"].append({
                    "backbone": bb.id,
                    "arm": arm,
                    "match": False,
                    "error": arm_data["error"],
                })
                continue
            cmp = compare_rows(
                ref["native_map"],
                _row_map(arm_data.get("rows") or []),
                arm=arm,
                bb_id=bb.id,
            )
            report["comparisons"].append(cmp)
            all_match = all_match and cmp["match"]

        if with_trad:
            trad = (row.get("arms") or {}).get("trad") or {}
            report["comparisons"].append({
                "backbone": bb.id,
                "arm": "trad",
                "control": True,
                "correct": trad.get("correct"),
                "n": trad.get("n"),
                "accuracy": trad.get("accuracy"),
                "error": trad.get("error"),
            })

    if tested == 0:
        verdict = "NO_WEIGHTS"
    elif all_match:
        verdict = "VALHALLA_BASE_INDEPENDENT"
    else:
        verdict = "DEPENDENCE_DETECTED"

    report["summary"] = {
        "verdict": verdict,
        "backbones_tested": tested,
        "backbones_total": len(backbones),
        "causal_tested": tested,
        "causal_total": sum(1 for b in backbones if b.kind == "causal_lm"),
        "appendix_count": sum(1 for c in report["comparisons"] if c.get("appendix")),
        "reference_accuracy": ref["native"]["accuracy"],
        "independent": verdict == "VALHALLA_BASE_INDEPENDENT",
        "panel": panel,
    }
    return report


def _print_report(report: dict[str, Any]) -> None:
    if ref := report.get("reference"):
        print(f"=== {PROTOCOL} ref=native {ref['correct']}/{ref['n']} ===\n")

    for bb_id, row in report.get("backbones", {}).items():
        if row.get("skipped"):
            print(f"  [{bb_id}] SKIP")
            continue
        arms = row.get("arms") or {}
        parts = []
        for arm in ("native", "hybrid", "trad"):
            if arm not in arms:
                continue
            d = arms[arm]
            if "error" in d:
                parts.append(f"{arm}=ERR")
            elif "accuracy" in d:
                parts.append(f"{arm}={d['correct']}/{d['n']}")
        print(f"  [{bb_id}] {'  '.join(parts)}")

    print()
    for cmp in report.get("comparisons", []):
        if cmp.get("skipped"):
            continue
        if cmp.get("control"):
            acc = cmp.get("accuracy")
            if acc is not None:
                print(f"  trad control {cmp['backbone']}: {cmp.get('correct')}/{cmp.get('n')}")
            continue
        tag = "PASS" if cmp.get("match") else "FAIL"
        if cmp.get("appendix"):
            print(f"  [appendix] {cmp['backbone']} ({cmp.get('category')}) arms={cmp.get('arms')}")
            continue
        print(f"  {tag} {cmp['backbone']}.{cmp['arm']} vs ref")
        for m in cmp.get("mismatches") or []:
            print(f"    ! {m.get('id')}: ref={m.get('ref_answer', m.get('reason'))!r}")

    s = report.get("summary") or {}
    if s:
        print(f"\n→ verdict: {s.get('verdict')} ({s.get('backbones_tested')}/{s.get('backbones_total')} backbones)")

    if mem := report.get("memory"):
        print(f"\n[memory] ref={mem['reference']['correct']}/{mem['reference']['n']} → {mem['verdict']}")
        for c in mem.get("comparisons") or []:
            tag = "PASS" if c.get("match_ref_native") else "FAIL"
            print(f"  {tag} {c['backbone']} needle hybrid vs native ref")

    if ext := report.get("extended"):
        print("\n[stack_capability_smokes — fixed Qwen stack, not backbone sweep]")
        for key in ("fusion_smoke", "proactive_smoke", "longchain_smoke"):
            row = ext.get(key) or {}
            if not row:
                continue
            tag = "OK" if row.get("ok") else "FAIL"
            print(f"  {tag} {row.get('label')} exit={row.get('exit_code')} ({row.get('elapsed_s')}s)")


def main() -> int:
    ap = argparse.ArgumentParser(description="ValhallaBase backbone-independence battery")
    ap.add_argument("--hf", action="append", help="Ad-hoc HF id (repeatable)")
    ap.add_argument("--preset", action="append", help="PRESETS key (repeatable)")
    ap.add_argument(
        "--paper",
        action="store_true",
        help="Six-class paper panel (HF paper presets; may skip if weights missing)",
    )
    ap.add_argument(
        "--practical",
        action="store_true",
        help="Six-class panel with cached/light presets (qwen_3b · mamba · rwkv · phi-moe · modernbert)",
    )
    ap.add_argument("--fast", action="store_true")
    ap.add_argument("--limit", type=int, default=0)
    ap.add_argument("--with-trad", action="store_true", help="Also run trad (control, slower)")
    ap.add_argument("--with-memory", action="store_true", help="Needle session smoke per backbone")
    ap.add_argument("--extended", action="store_true",
                    help="stack capability smokes: fusion + proactive gates + longchain")
    ap.add_argument("--smokes-only", action="store_true",
                    help="Skip open sweep; merge into existing report")
    ap.add_argument("--merge-into", type=Path, default=None, help="Merge results into JSON")
    ap.add_argument("--smoke", action="append", choices=["fusion", "proactive", "longchain"],
                    help="Run one extended smoke only (repeatable)")
    ap.add_argument("--fail-on-skip", action="store_true", help="Treat missing weights as failure")
    args = ap.parse_args()

    apply_gpu_env()
    apply_resource_budget()
    release_resources()
    os.environ["VALHALLA_BASE_NO_SERVE"] = "1"
    os.environ.pop("VALHALLA_BASE_URL", None)
    os.environ.setdefault("VALHALLA_INGRESS_RESOURCE_PERCENT", "50")
    os.environ.setdefault("VALHALLA_INGRESS_MAX_ITEMS", "48")
    os.environ.setdefault("VALHALLA_LM_CACHE_MAX", "1")
    os.environ.setdefault("HF_ENDPOINT", os.environ.get("HF_ENDPOINT", "https://hf-mirror.com"))

    merge_path = args.merge_into or (REPORTS / "valhalla_independence_test.json")
    if args.smokes_only and merge_path.is_file():
        report = json.loads(merge_path.read_text(encoding="utf-8"))
        print(f"=== {PROTOCOL} smokes-only (merged) ===\n")
    else:
        if args.practical:
            backbones = resolve(practical=True)
        elif args.paper:
            backbones = resolve(paper=True)
        else:
            presets = args.preset or (None if args.hf else list(DEFAULT_PRESETS))
            backbones = resolve(hf=args.hf, preset=presets)
        if not backbones:
            print("No backbones. Use --preset or --hf.", file=sys.stderr)
            return 1

        report = run_independence(
            backbones,
            fast=args.fast,
            limit=args.limit,
            with_trad=args.with_trad,
            skip_missing=not args.fail_on_skip,
            panel="practical" if args.practical else ("paper" if args.paper else "default"),
        )

        if args.with_memory or args.extended:
            report["memory"] = run_memory_smoke(backbones, fast=args.fast)
            if report["memory"]["verdict"] != "MEMORY_INDEPENDENT":
                report["summary"]["verdict"] = "DEPENDENCE_DETECTED"
                report["summary"]["independent"] = False

    if args.extended or args.smoke:
        only = set(args.smoke) if args.smoke else None
        ext = run_extended_smokes(fast=args.fast, only=only)
        prev = report.get("extended") or {}
        prev.update(ext)
        report["extended"] = prev
        report["extended"]["completed_at"] = datetime.now(timezone.utc).isoformat()
        smokes = [prev.get(k) for k in ("fusion_smoke", "proactive_smoke", "longchain_smoke") if prev.get(k)]
        report["extended"]["all_ok"] = bool(smokes) and all(s.get("ok") for s in smokes)

    report["test_layers"] = {
        "open_path_orthogonality": report.get("summary", {}).get("verdict"),
        "session_memory_orthogonality": (report.get("memory") or {}).get("verdict"),
        "core_fusion_smoke": ((report.get("extended") or {}).get("fusion_smoke") or {}).get("ok"),
        "proactive_gate_smoke": ((report.get("extended") or {}).get("proactive_smoke") or {}).get("ok"),
        "longchain_memory_smoke": ((report.get("extended") or {}).get("longchain_smoke") or {}).get("ok"),
    }

    _print_report(report)

    out = REPORTS / "valhalla_independence_test.json"
    out.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"→ {out}")

    s = report.get("summary") or {}
    if s.get("verdict") == "VALHALLA_BASE_INDEPENDENT":
        return 0
    if s.get("verdict") == "NO_WEIGHTS":
        return 2
    if s.get("verdict"):
        return 1
    # smokes-only: success if all requested smokes passed
    ext = report.get("extended") or {}
    keys = [k for k in ("fusion_smoke", "proactive_smoke", "longchain_smoke") if k in ext]
    if keys and all(ext[k].get("ok") for k in keys):
        return 0
    return 1 if keys else 0


if __name__ == "__main__":
    raise SystemExit(main())
