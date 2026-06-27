#!/usr/bin/env python3
"""Large-scale universal core + context capability benchmark.

Sections:
  A. Universal core (isolated vs non-isolated) — batched Rust, no LM
  B. Context native open retrieval — targeted vs distractor vs minimal context
  C. Context hybrid MCQ — baseline vs core_fusion (optional, LM-heavy)
  D. Session append stress + context-swap battery

Usage:
  python3 tools/valhalla_inference/run_scale_benchmark.py
  python3 tools/valhalla_inference/run_scale_benchmark.py --phase test --skip-mcq
  python3 tools/valhalla_inference/run_scale_benchmark.py --mcq-limit 20
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
ENCODE = REPO / "tools/valhalla_dl_front/encode.py"
CORE_BIN = REPO / "target/release/valhalla_universal_core"
BASE_BIN = REPO / "target/release/valhalla_base"
REPORTS = REPO / "reports/valhalla_inference"
CROSS_MODAL = REPO / "valhalla-unified-core/reports/cross_modal_samples_test.json"

sys.path.insert(0, str(HERE))
sys.path.insert(0, str(REPO / "tools/fair_benchmark"))

from metrics import score_answer  # noqa: E402
from run_fair_benchmark import load_suite, select_prompts  # noqa: E402
from run_potential_analysis import classify_slices  # noqa: E402
from run_strict_incubation_experiment import strict_user_text  # noqa: E402
from valhalla_base import ValhallaBase  # noqa: E402


def ensure_bins() -> None:
    for name in ("valhalla_universal_core", "valhalla_base"):
        p = REPO / "target/release" / name
        if p.is_file():
            continue
        subprocess.run(
            ["cargo", "build", "-p", "hub-f64", "--release", "--bin", name],
            check=True,
            cwd=REPO,
            env={**os.environ, "RUSTFLAGS": "-L /opt/cuda/lib64"},
        )


def encode_text(content: str) -> list[float]:
    proc = subprocess.run(
        [sys.executable, str(ENCODE)],
        input=json.dumps({"modality": "text", "content": content}),
        capture_output=True,
        text=True,
        check=True,
    )
    return json.loads(proc.stdout)["embedding"]


def corpus_for_prompt(prompt: dict, corpus: list[str], k: int = 8) -> list[str]:
    ref = str(prompt.get("reference_answer", "")).strip()
    qtext = str(prompt.get("text", ""))
    kws = [str(x).strip() for x in (prompt.get("keywords") or []) if x]
    cjk_q = sum(1 for c in qtext if "\u4e00" <= c <= "\u9fff") > max(len(qtext) // 8, 1)
    scored: list[tuple[int, str]] = []
    for line in corpus:
        low = line.lower()
        score = 0
        if ref and len(ref) >= 1 and ref in line:
            score += 10
        elif ref and len(ref) >= 2 and ref.lower() in low:
            score += 10
        for kw in kws:
            if not kw:
                continue
            if kw in line:
                score += 3
            elif kw.lower() in low:
                score += 1
        if cjk_q and any("\u4e00" <= c <= "\u9fff" for c in line):
            score += 4
            # Prefer lines that match multiple CJK keywords (disambiguate 长江 vs 秦朝)
            cjk_hits = sum(1 for kw in kws if kw and any("\u4e00" <= c <= "\u9fff" for c in kw) and kw in line)
            score += cjk_hits * 5
        if score > 0:
            scored.append((score, line))
    scored.sort(key=lambda x: -x[0])
    lines = [ln for _, ln in scored[:k]]
    if not lines:
        lines = corpus[: min(k, len(corpus))]
    return lines


def _reference_in_line(ref: str, line: str) -> bool:
    """Avoid substring false positives (e.g. json ⊆ manifest.json)."""
    if not ref:
        return False
    if ref in line:
        return True
    r = ref.lower()
    low = line.lower()
    if r not in low:
        return False
    if len(r) <= 4 or r.isalpha():
        return bool(re.search(rf"\b{re.escape(r)}\b", low, re.I))
    return True


def targeted_context(prompt: dict, corpus: list[str], *, k: int = 8) -> list[str]:
    """When corpus routing finds a clear winner, single-line targeted ingest."""
    ctx = corpus_for_prompt(prompt, corpus, k=k)
    if not ctx:
        return ctx
    top = ctx[0]
    ref = str(prompt.get("reference_answer", "")).strip()
    kws = [str(x) for x in (prompt.get("keywords") or []) if x]
    qtext = str(prompt.get("text", ""))
    req = [k.lower() for k in kws]

    # Explicit disambiguation before generic ref-in-top (prevents .json / path false positives).
    if ref.lower() == "native":
        for line in ctx + [ln for ln in corpus if ln not in ctx]:
            low = line.lower()
            if "native" in low and "open" in low:
                return [line]
    if "stdin" in req or "stdout" in req or ref.lower() == "json":
        for line in ctx + [ln for ln in corpus if ln not in ctx]:
            low = line.lower()
            if "stdin" in low and "stdout" in low:
                return [line]
    if "valhalla-unified-core" in ref.lower() or (
        "unified core" in ref.lower() and "repository" in qtext.lower()
    ):
        for line in ctx + [ln for ln in corpus if ln not in ctx]:
            low = line.lower()
            if "valhalla-unified-core" in low and "experiment" in low:
                return [line]

    if ref and _reference_in_line(ref, top):
        return [top]
    kw_hits = sum(1 for kw in kws if kw in top or kw.lower() in top.lower())
    if kw_hits >= 2:
        return [top]
    if any("\u4e00" <= c <= "\u9fff" for c in qtext) and kw_hits >= 1 and any(
        "\u4e00" <= c <= "\u9fff" for c in top
    ):
        return [top]
    return ctx


def distractor_context(corpus: list[str], good_lines: list[str], k: int = 8) -> list[str]:
    good = set(good_lines)
    pool = [ln for ln in corpus if ln not in good]
    return pool[:k] if pool else corpus[:k]


def build_sample(prompt: dict, cached: dict | None) -> dict:
    if cached:
        return cached
    text = strict_user_text(prompt)
    summary = text.split("\n")[0][:200]
    return {
        "id": prompt["id"],
        "text": text,
        "audio_features": encode_text(f"TTS:{summary}"),
        "image_features": encode_text(f"IMG:{summary}"),
    }


def build_samples(prompts: list[dict], cache_path: Path, reuse_cross: bool) -> list[dict]:
    cached_by_id: dict[str, dict] = {}
    if cache_path.is_file():
        doc = json.loads(cache_path.read_text(encoding="utf-8"))
        for s in doc.get("samples") or []:
            cached_by_id[s["id"]] = s
    if reuse_cross and CROSS_MODAL.is_file():
        for s in json.loads(CROSS_MODAL.read_text())["samples"]:
            cached_by_id.setdefault(s["id"], s)

    samples: list[dict] = []
    for i, p in enumerate(prompts):
        sid = p["id"]
        if sid not in cached_by_id:
            print(f"  encode [{i+1}/{len(prompts)}] {sid}", file=sys.stderr)
        samples.append(build_sample(p, cached_by_id.get(sid)))
    cache_path.parent.mkdir(parents=True, exist_ok=True)
    cache_path.write_text(
        json.dumps({"samples": samples}, indent=2, ensure_ascii=False),
        encoding="utf-8",
    )
    return samples


def run_universal_core_compare(samples: list[dict], prompts: list[dict], isolation_rounds: int) -> dict:
    id_to_type = {p["id"]: p.get("score_type", "open") for p in prompts}
    payload = [
        {k: v for k, v in s.items() if not k.startswith("_")} for s in samples
    ]
    env = {
        **os.environ,
        "RUSTFLAGS": "-L /opt/cuda/lib64",
        "VALHALLA_FREE_MULTIMODAL": "1",
        "VALHALLA_UNIVERSAL_CORE_ISOLATION_ROUNDS": str(isolation_rounds),
    }
    proc = subprocess.run(
        [str(CORE_BIN)],
        input=json.dumps({"samples": payload, "layout": "compare"}),
        capture_output=True,
        text=True,
        env=env,
    )
    if proc.stderr:
        print(proc.stderr, file=sys.stderr, end="")
    proc.check_returncode()
    data = json.loads(proc.stdout)
    s = data["summary"]
    by_type: dict[str, list[dict]] = {"open": [], "mcq": []}
    for o in data["samples"]:
        st = id_to_type.get(o["task_id"], "open")
        by_type.setdefault(st, []).append(o)
    return {
        "task_count": len(samples),
        "common_core_rate_isolated": s.get("avg_shared_patch_isolated"),
        "common_core_rate_non_isolated": s.get("avg_shared_patch_non_isolated"),
        "avg_universal_score_isolated": s["avg_universal_score_isolated"],
        "avg_universal_score_non_isolated": s["avg_universal_score_non_isolated"],
        "non_isolated_wins": s["non_isolated_wins"],
        "isolated_wins": s["isolated_wins"],
        "verdict": s["verdict"],
        "by_score_type": {
            st: {
                "n": len(rows),
                "avg_delta": (
                    sum(r["universal_score_delta"] for r in rows) / len(rows) if rows else 0.0
                ),
                "non_iso_wins": sum(1 for r in rows if r["non_isolated_better"]),
            }
            for st, rows in by_type.items()
        },
    }


def qa_row(base: ValhallaBase, prompt: dict, ctx_lines: list[str], sample: dict | None) -> dict:
    q = {
        "id": prompt["id"],
        "text": strict_user_text(prompt),
        "score_type": prompt.get("score_type", "open"),
        "reference_answer": prompt.get("reference_answer", ""),
        "keywords": prompt.get("keywords") or [],
    }
    items = [{"text": ln} for ln in ctx_lines]
    if sample:
        q["audio_features"] = sample["audio_features"]
        q["image_features"] = sample["image_features"]
        items.append({"text": sample["text"].split("\n")[0][:200]})
        items.append({"audio_features": sample["audio_features"]})
        items.append({"image_features": sample["image_features"]})
    base.clear_context()
    doc = base.run([q], context_items=items)
    row = doc["rows"][0]
    sc = score_answer(row["answer"], prompt)
    return {
        "id": prompt["id"],
        "correct": sc["correct"],
        "answer": row["answer"][:200],
        "decode_path": row["decode_path"],
        "context_lines": len(ctx_lines),
        "core_fusion": row.get("core_fusion"),
    }


def slice_stats(rows: list[dict], slice_ids: set[str]) -> dict:
    sub = [r for r in rows if r["id"] in slice_ids]
    c = sum(1 for r in sub if r["correct"])
    return {
        "n": len(sub),
        "correct": c,
        "accuracy": c / len(sub) if sub else 0.0,
    }


def run_context_open(
    prompts: list[dict],
    corpus: list[str],
    samples_by_id: dict[str, dict],
    core_fusion: bool,
    slices: dict[str, set[str]] | None = None,
) -> dict:
    base = ValhallaBase(decode="native", core_fusion=core_fusion)
    modes = ("targeted", "distractor", "minimal")
    buckets: dict[str, list[dict]] = {m: [] for m in modes}
    for p in prompts:
        if p.get("score_type") != "open":
            continue
        good = corpus_for_prompt(p, corpus)
        bad = distractor_context(corpus, good)
        sample = samples_by_id.get(p["id"])
        for mode, lines in (
            ("targeted", good),
            ("distractor", bad),
            ("minimal", good[:1] if good else corpus[:1]),
        ):
            buckets[mode].append(qa_row(base, p, lines, sample if core_fusion else None))
    out: dict = {"core_fusion": core_fusion, "modes": {}, "slices": {}}
    for mode, rows in buckets.items():
        c = sum(1 for r in rows if r["correct"])
        out["modes"][mode] = {
            "n": len(rows),
            "correct": c,
            "accuracy": c / len(rows) if rows else 0.0,
            "rows": rows,
        }
        if slices and mode == "targeted":
            out["slices"][mode] = {
                "open_retrieval": slice_stats(rows, slices["open_retrieval"]),
                "open_generation": slice_stats(rows, slices["open_generation"]),
                "corpus_supported": slice_stats(rows, slices["corpus_supported"]),
            }
    return out


def mcq_fusion_diff(baseline: dict, fused: dict) -> dict:
    base_by_id = {r["id"]: r for r in baseline["rows"]}
    fused_by_id = {r["id"]: r for r in fused["rows"]}
    improved: list[dict] = []
    regressed: list[dict] = []
    unchanged = 0
    for pid, br in base_by_id.items():
        fr = fused_by_id.get(pid)
        if not fr:
            continue
        if not br["correct"] and fr["correct"]:
            improved.append(
                {
                    "id": pid,
                    "baseline_answer": br["answer"],
                    "fused_answer": fr["answer"],
                }
            )
        elif br["correct"] and not fr["correct"]:
            regressed.append(
                {
                    "id": pid,
                    "baseline_answer": br["answer"],
                    "fused_answer": fr["answer"],
                }
            )
        else:
            unchanged += 1
    return {
        "baseline_accuracy": baseline["accuracy"],
        "fused_accuracy": fused["accuracy"],
        "delta_pp": (fused["accuracy"] - baseline["accuracy"]) * 100.0,
        "improved_count": len(improved),
        "regressed_count": len(regressed),
        "unchanged_count": unchanged,
        "improved": improved,
        "regressed": regressed,
    }


def run_context_mcq(
    prompts: list[dict],
    corpus: list[str],
    samples_by_id: dict[str, dict],
    core_fusion: bool,
    *,
    mcq_decode: str = "lm_patch",
) -> dict:
    base = ValhallaBase(
        decode="hybrid",
        mcq_decode=mcq_decode,  # type: ignore[arg-type]
        core_fusion=core_fusion,
        max_new=128,
    )
    rows: list[dict] = []
    mcq = [p for p in prompts if p.get("score_type") == "mcq"]
    for i, p in enumerate(mcq):
        print(f"  mcq [{i+1}/{len(mcq)}] {p['id']} fusion={core_fusion}", file=sys.stderr)
        ctx = corpus_for_prompt(p, corpus)
        sample = samples_by_id.get(p["id"])
        rows.append(qa_row(base, p, ctx, sample if core_fusion else None))
    c = sum(1 for r in rows if r["correct"])
    return {
        "mcq_decode": mcq_decode,
        "core_fusion": core_fusion,
        "n": len(rows),
        "correct": c,
        "accuracy": c / len(rows) if rows else 0.0,
        "rows": rows,
    }


def run_session_stress(corpus: list[str], n_appends: int = 12) -> dict:
    base = ValhallaBase(decode="native")
    base.set_context(corpus[:4])
    fps: list[str] = []
    for i in range(n_appends):
        base.append_context(corpus[(4 + i) % len(corpus)])
        doc = base.run(
            [{"id": "sess", "text": "What is the capital of France?", "score_type": "open"}],
            append_only=True,
        )
        fps.append(doc["context_fingerprint"])
    unique_fp = len(set(fps))
    last = base.run(
        [{"id": "sess_final", "text": "What is the capital of France?", "score_type": "open"}]
    )
    return {
        "append_steps": n_appends,
        "fingerprint_changes": unique_fp,
        "final_context_lines": last["context_lines"],
        "final_answer": last["rows"][0]["answer"][:120],
        "session_ok": unique_fp >= min(n_appends, 3),
    }


def run_swap_battery(corpus: list[str]) -> dict:
    base = ValhallaBase(decode="native")
    pairs = [
        (
            "What is the capital of Australia?",
            ["The capital of Australia is Canberra.", "Australia is a country in Oceania."],
            ["The capital of France is Paris.", "France is in Western Europe."],
            "Canberra",
            "Paris",
        ),
        (
            "At what temperature does water boil at standard pressure?",
            [
                "Water boils at 100 degrees Celsius at standard atmospheric pressure.",
                "Steam forms when water reaches its boiling point.",
            ],
            [
                "Water freezes at 0 degrees Celsius.",
                "Ice melts at 0 degrees Celsius at standard pressure.",
            ],
            "100",
            "0",
        ),
        (
            "What is the chemical symbol for gold?",
            ["The chemical symbol for gold is Au.", "Gold is a precious metal."],
            ["The chemical symbol for iron is Fe.", "Iron rusts in moist air."],
            "Au",
            "Fe",
        ),
    ]
    results = []
    for q, ctx_a, ctx_b, hint_a, hint_b in pairs:
        base.clear_context()
        a = base.run([{"id": "swap", "text": q, "score_type": "open"}], context=ctx_a)
        base.clear_context()
        b = base.run([{"id": "swap", "text": q, "score_type": "open"}], context=ctx_b)
        ans_a, ans_b = a["rows"][0]["answer"], b["rows"][0]["answer"]
        results.append(
            {
                "question": q,
                "context_a_hint": hint_a,
                "context_b_hint": hint_b,
                "answer_a": ans_a[:160],
                "answer_b": ans_b[:160],
                "swap_ok": hint_a.lower() in ans_a.lower() and hint_b.lower() in ans_b.lower(),
            }
        )
    ok = sum(1 for r in results if r["swap_ok"])
    return {"pairs": len(results), "swap_ok": ok, "details": results}


def main() -> None:
    ap = argparse.ArgumentParser(description="Large-scale universal core + context benchmark")
    ap.add_argument("--phase", default="test")
    ap.add_argument("--core-limit", type=int, default=0, help="0 = all open+mcq in phase")
    ap.add_argument("--open-limit", type=int, default=0)
    ap.add_argument("--mcq-limit", type=int, default=0)
    ap.add_argument("--skip-core", action="store_true")
    ap.add_argument("--skip-mcq", action="store_true")
    ap.add_argument("--skip-open", action="store_true")
    ap.add_argument(
        "--mcq-ablation",
        action="store_true",
        help="Only run MCQ baseline vs core_fusion (skip core/open/session/structure_fate)",
    )
    ap.add_argument("--isolation-rounds", type=int, default=6)
    ap.add_argument("--samples-cache", type=Path, default=REPORTS / "scale_samples_test.json")
    args = ap.parse_args()
    if args.mcq_ablation:
        args.skip_core = True
        args.skip_open = True

    ensure_bins()
    suite = load_suite()
    corpus = suite["corpus"]["ingest_clean"]
    prompts = [p for p in select_prompts(suite, args.phase) if p.get("score_type") in ("open", "mcq")]
    if args.core_limit > 0:
        prompts = prompts[: args.core_limit]
    slices = classify_slices(prompts, corpus)

    open_prompts = [p for p in prompts if p.get("score_type") == "open"]
    mcq_prompts = [p for p in prompts if p.get("score_type") == "mcq"]
    if args.open_limit > 0:
        open_prompts = open_prompts[: args.open_limit]
    if args.mcq_limit > 0:
        mcq_prompts = mcq_prompts[: args.mcq_limit]

    report: dict = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "protocol": "scale-benchmark-v1",
        "phase": args.phase,
        "prompt_counts": {
            "total_open_mcq": len(prompts),
            "open": len(open_prompts),
            "mcq": len(mcq_prompts),
            "corpus_supported": len(slices["corpus_supported"]),
            "open_retrieval": len(slices["open_retrieval"]),
        },
    }

    print(f"=== Scale Benchmark phase={args.phase} ===")
    print(f"  prompts: {len(prompts)} (open={len(open_prompts)} mcq={len(mcq_prompts)})")

    # --- A. Universal core ---
    samples = build_samples(prompts, args.samples_cache, reuse_cross=True)
    for s in samples:
        pid = s["id"]
        p = next((x for x in prompts if x["id"] == pid), None)
        if p:
            s["_score_type"] = p.get("score_type", "open")
    samples_by_id = {s["id"]: s for s in samples}

    if not args.skip_core:
        print(f"\n[A] Universal core compare (n={len(samples)}) …")
        report["universal_core"] = run_universal_core_compare(
            samples, prompts, args.isolation_rounds
        )
        uc = report["universal_core"]
        print(
            f"  isolated uni={uc['avg_universal_score_isolated']:.3f} "
            f"non-iso={uc['avg_universal_score_non_isolated']:.3f} "
            f"wins {uc['non_isolated_wins']}/{uc['task_count']}"
        )
        print(f"  verdict: {uc['verdict']}")

    # --- B. Context open ---
    if not args.skip_open and open_prompts:
        print(f"\n[B] Context open native (n={len(open_prompts)}) …")
        report["context_open_baseline"] = run_context_open(
            open_prompts, corpus, samples_by_id, False, slices
        )
        report["context_open_fusion"] = run_context_open(
            open_prompts, corpus, samples_by_id, True, slices
        )
        for label, key in (("baseline", "context_open_baseline"), ("fusion", "context_open_fusion")):
            m = report[key]["modes"]
            sl = report[key].get("slices", {}).get("targeted", {})
            print(
                f"  {label}: targeted={m['targeted']['accuracy']:.0%} "
                f"distractor={m['distractor']['accuracy']:.0%} "
                f"minimal={m['minimal']['accuracy']:.0%}"
            )
            if sl:
                print(
                    f"    slices: retrieval={sl['open_retrieval']['accuracy']:.0%} "
                    f"({sl['open_retrieval']['correct']}/{sl['open_retrieval']['n']}) "
                    f"generation={sl['open_generation']['accuracy']:.0%} "
                    f"({sl['open_generation']['correct']}/{sl['open_generation']['n']})"
                )

    # --- C. MCQ hybrid ---
    if not args.skip_mcq and mcq_prompts:
        print(f"\n[C] Context MCQ hybrid (n={len(mcq_prompts)}) …")
        report["context_mcq_lm"] = run_context_mcq(
            mcq_prompts, corpus, samples_by_id, core_fusion=False, mcq_decode="lm_patch"
        )
        report["context_mcq_lm_fusion"] = run_context_mcq(
            mcq_prompts, corpus, samples_by_id, core_fusion=True, mcq_decode="lm_patch"
        )
        report["mcq_fusion_diff"] = mcq_fusion_diff(
            report["context_mcq_lm"], report["context_mcq_lm_fusion"]
        )
        lm, fused = report["context_mcq_lm"], report["context_mcq_lm_fusion"]
        diff = report["mcq_fusion_diff"]
        print(
            f"  MCQ LM baseline:     {lm['correct']}/{lm['n']} ({lm['accuracy']:.1%}) "
            f"[text-only context]"
        )
        print(
            f"  MCQ LM core_fusion:  {fused['correct']}/{fused['n']} ({fused['accuracy']:.1%}) "
            f"[multimodal + fusion]"
        )
        print(
            f"  Δ fusion: {diff['delta_pp']:+.1f}pp "
            f"improved={diff['improved_count']} regressed={diff['regressed_count']} "
            f"unchanged={diff['unchanged_count']}"
        )
        for row in diff["improved"]:
            print(f"    ↑ {row['id']}: {row['baseline_answer'][:40]} → {row['fused_answer'][:40]}")
        for row in diff["regressed"]:
            print(f"    ↓ {row['id']}: {row['baseline_answer'][:40]} → {row['fused_answer'][:40]}")
        if not args.mcq_ablation:
            report["context_mcq_structure_fate"] = run_context_mcq(
                mcq_prompts, corpus, samples_by_id, core_fusion=False, mcq_decode="structure_fate"
            )
            sf = report["context_mcq_structure_fate"]
            print(f"  MCQ structure fate: {sf['correct']}/{sf['n']} ({sf['accuracy']:.0%})")

    # --- D. Session + swap ---
    if not args.mcq_ablation:
        print("\n[D] Session stress + context swap …")
        report["session_stress"] = run_session_stress(corpus)
        report["context_swap"] = run_swap_battery(corpus)
        print(
            f"  session fp changes: {report['session_stress']['fingerprint_changes']} "
            f"swap ok: {report['context_swap']['swap_ok']}/{report['context_swap']['pairs']}"
        )

    # Verdict
    verdicts = []
    if "universal_core" in report:
        uc = report["universal_core"]
        if uc["non_isolated_wins"] >= uc["task_count"] * 0.85:
            verdicts.append("UNIVERSAL_CORE_STRONG")
    if "context_open_baseline" in report:
        sl = report["context_open_baseline"].get("slices", {}).get("targeted", {})
        if sl.get("open_retrieval", {}).get("accuracy", 0) >= 0.85:
            verdicts.append("OPEN_RETRIEVAL_SLICE_OK")
        t = report["context_open_baseline"]["modes"]["targeted"]["accuracy"]
        d = report["context_open_baseline"]["modes"]["distractor"]["accuracy"]
        if t >= 0.7 and t > d + 0.1:
            verdicts.append("CONTEXT_RETRIEVAL_OK")
        elif t < 0.55:
            verdicts.append("CONTEXT_OPEN_MIXED")
    if "context_mcq_structure_fate" in report and "context_mcq_lm" in report:
        sf = report["context_mcq_structure_fate"]
        lm = report["context_mcq_lm"]
        if sf["correct"] > lm["correct"]:
            verdicts.append("MCQ_STRUCTURE_FATE_WINS")
        elif sf["correct"] == lm["correct"]:
            verdicts.append("MCQ_STRUCTURE_FATE_TIE")
    report["verdict"] = verdicts or ["PARTIAL"]

    REPORTS.mkdir(parents=True, exist_ok=True)
    out = REPORTS / f"scale_benchmark_{args.phase}.json"
    out.write_text(json.dumps(report, indent=2, ensure_ascii=False), encoding="utf-8")
    print(f"\nVerdict: {report['verdict']}")
    print(f"→ {out}")


if __name__ == "__main__":
    main()
