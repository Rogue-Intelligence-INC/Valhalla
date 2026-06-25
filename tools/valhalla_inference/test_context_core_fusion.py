#!/usr/bin/env python3
"""Context × Core fusion test — baseline vs structure→LM core_fusion path.

Pipeline per task:
  1. Context bundle (fair corpus + sample text/TTS/image)
  2. Universal core metrics (non-isolated, reference)
  3. ValhallaBase hybrid QA: baseline vs core_fusion=true

Usage:
  python3 tools/valhalla_inference/test_context_core_fusion.py --limit 12
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
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(REPO / "tools/fair_benchmark"))

from run_fair_benchmark import load_suite, select_prompts  # noqa: E402
from run_potential_analysis import classify_slices  # noqa: E402
from run_strict_incubation_experiment import strict_user_text  # noqa: E402
from valhalla_base import ValhallaBase  # noqa: E402

REPORTS = REPO / "reports/valhalla_inference"
CORE_BIN = REPO / "target/release/valhalla_universal_core"
BASE_BIN = REPO / "target/release/valhalla_base"
SAMPLES_DEFAULT = REPO / "valhalla-unified-core/reports/cross_modal_samples_test.json"


def ensure_bins() -> None:
    for name in ("valhalla_universal_core", "valhalla_base"):
        path = REPO / "target/release" / name
        if path.is_file():
            continue
        subprocess.run(
            ["cargo", "build", "-p", "hub-f64", "--release", "--bin", name],
            check=True,
            cwd=REPO,
            env={**os.environ, "RUSTFLAGS": "-L /opt/cuda/lib64"},
        )


def corpus_for_prompt(prompt: dict, corpus: list[str], k: int = 8) -> list[str]:
    ref = str(prompt.get("reference_answer", "")).lower()
    kws = [str(x).lower() for x in (prompt.get("keywords") or []) if x]
    scored: list[tuple[int, str]] = []
    for line in corpus:
        low = line.lower()
        score = 0
        if ref and len(ref) >= 2 and ref in low:
            score += 10
        score += sum(1 for kw in kws if kw and kw in low)
        if score > 0:
            scored.append((score, line))
    scored.sort(key=lambda x: -x[0])
    lines = [ln for _, ln in scored[:k]]
    if not lines:
        lines = corpus[: min(k, len(corpus))]
    return lines


def run_universal_core(sample: dict) -> dict:
    proc = subprocess.run(
        [str(CORE_BIN)],
        input=json.dumps({"samples": [sample], "layout": "non_isolated"}),
        capture_output=True,
        text=True,
        env={
            **os.environ,
            "RUSTFLAGS": "-L /opt/cuda/lib64",
            "VALHALLA_FREE_MULTIMODAL": "1",
        },
    )
    if proc.stderr:
        print(proc.stderr, file=sys.stderr, end="")
    proc.check_returncode()
    doc = json.loads(proc.stdout)
    m = doc["samples"][0]["metrics"]
    return {
        "shared_patch": m["shared_patch_pairwise_avg"],
        "universal_score": m["universal_core_score"],
        "universal_formed": m["universal_core_formed"],
        "text_tts": m["patch_text_audio"],
        "text_image": m["patch_text_image"],
        "tts_image": m["patch_audio_image"],
    }


def context_items_from_sample(sample: dict, extra_lines: list[str]) -> list[dict]:
    items: list[dict] = []
    for ln in extra_lines:
        items.append({"text": ln})
    items.append({"text": sample["text"].split("\n")[0][:200]})
    items.append({"audio_features": sample["audio_features"]})
    items.append({"image_features": sample["image_features"]})
    return items


def question_payload(prompt: dict, sample: dict) -> dict:
    return {
        "id": prompt["id"],
        "text": strict_user_text(prompt),
        "score_type": prompt.get("score_type", "open"),
        "reference_answer": prompt.get("reference_answer", ""),
        "keywords": prompt.get("keywords") or [],
        "audio_features": sample["audio_features"],
        "image_features": sample["image_features"],
    }


def run_qa(base: ValhallaBase, prompt: dict, sample: dict, ctx_lines: list[str]) -> dict:
    items = context_items_from_sample(sample, ctx_lines)
    base.clear_context()
    doc = base.run([question_payload(prompt, sample)], context_items=items)
    row = doc["rows"][0]
    return {
        "correct": row["correct"],
        "decode_path": row["decode_path"],
        "answer": row["answer"][:300],
        "core_fusion": row.get("core_fusion"),
        "context_fingerprint": doc["context_fingerprint"],
        "features": doc.get("features"),
    }


def main() -> None:
    ap = argparse.ArgumentParser(description="Context × core fusion test")
    ap.add_argument("--limit", type=int, default=12)
    ap.add_argument("--samples-json", type=Path, default=SAMPLES_DEFAULT)
    ap.add_argument("--lm", default="Qwen/Qwen2.5-0.5B-Instruct")
    args = ap.parse_args()

    if not args.samples_json.is_file():
        sys.exit(f"Missing samples: {args.samples_json}")

    ensure_bins()
    samples_doc = json.loads(args.samples_json.read_text(encoding="utf-8"))
    samples = samples_doc["samples"][: args.limit]

    suite = load_suite()
    prompts = {p["id"]: p for p in select_prompts(suite, "test")}
    corpus = suite["corpus"]["ingest_clean"]
    slices = classify_slices(list(prompts.values()), corpus)

    baseline = ValhallaBase(decode="hybrid", lm_model=args.lm, max_new=128, core_fusion=False)
    fused = ValhallaBase(decode="hybrid", lm_model=args.lm, max_new=128, core_fusion=True)
    tasks: list[dict] = []

    print(f"=== Context × Core Fusion (n={len(samples)}, baseline vs core_fusion) ===\n")

    for i, sample in enumerate(samples):
        sid = sample["id"]
        prompt = prompts.get(sid)
        if not prompt:
            print(f"  skip {sid}: not in fair test", file=sys.stderr)
            continue

        ctx_lines = corpus_for_prompt(prompt, corpus)
        print(f"[{i+1}/{len(samples)}] {sid}")
        core = run_universal_core(sample)
        qa_base = run_qa(baseline, prompt, sample, ctx_lines)
        qa_fused = run_qa(fused, prompt, sample, ctx_lines)

        tasks.append(
            {
                "id": sid,
                "score_type": prompt.get("score_type"),
                "corpus_supported": sid in slices["corpus_supported"],
                "core_reference": core,
                "baseline": qa_base,
                "core_fusion_path": qa_fused,
                "fusion_improved": (not qa_base["correct"] and qa_fused["correct"]),
                "fusion_regressed": (qa_base["correct"] and not qa_fused["correct"]),
            }
        )
        print(
            f"  core ref: uni={core['universal_score']:.3f} shared={core['shared_patch']:.3f}"
        )
        print(
            f"  baseline: correct={qa_base['correct']}  "
            f"fused: correct={qa_fused['correct']} "
            f"uni={((qa_fused.get('core_fusion') or {}).get('universal_score'))}"
        )

    core_ok = sum(1 for t in tasks if t["core_reference"]["universal_formed"])
    base_ok = sum(1 for t in tasks if t["baseline"]["correct"])
    fused_ok = sum(1 for t in tasks if t["core_fusion_path"]["correct"])
    improved = sum(1 for t in tasks if t["fusion_improved"])
    regressed = sum(1 for t in tasks if t["fusion_regressed"])
    avg_uni = sum(t["core_reference"]["universal_score"] for t in tasks) / max(len(tasks), 1)

    swap: dict | None = None
    if tasks and samples:
        s0 = samples[0]
        alt = [
            "Water boils at 100 degrees Celsius at standard atmospheric pressure.",
            "The capital of Australia is Canberra.",
        ]
        fused.clear_context()
        d_a = fused.run(
            [{"id": "swap", "text": "What is the capital of Australia?", "score_type": "open"}],
            context_items=context_items_from_sample(s0, alt),
        )
        fused.clear_context()
        d_b = fused.run(
            [{"id": "swap", "text": "What is the capital of Australia?", "score_type": "open"}],
            context_items=[{"text": "The capital of France is Paris."}],
        )
        swap = {
            "question": "What is the capital of Australia?",
            "with_canberra_context": d_a["rows"][0]["answer"][:200],
            "with_paris_context": d_b["rows"][0]["answer"][:200],
        }

    summary = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "protocol": "context-core-fusion-v2",
        "task_count": len(tasks),
        "core_formation_rate": core_ok / max(len(tasks), 1),
        "baseline_qa_accuracy": base_ok / max(len(tasks), 1),
        "core_fusion_qa_accuracy": fused_ok / max(len(tasks), 1),
        "fusion_improved_count": improved,
        "fusion_regressed_count": regressed,
        "avg_universal_score": avg_uni,
        "context_swap_ablation": swap,
        "tasks": tasks,
        "verdict": (
            "CORE_FUSION_WINS"
            if fused_ok > base_ok
            else "CORE_FUSION_TIE"
            if fused_ok == base_ok and core_ok >= len(tasks) * 0.8
            else "BASELINE_WINS"
            if base_ok > fused_ok
            else "PARTIAL"
        ),
    }

    print(f"\n=== Summary (n={len(tasks)}) ===")
    print(f"Core formed: {core_ok}/{len(tasks)}  avg uni={avg_uni:.3f}")
    print(f"Baseline QA:  {base_ok}/{len(tasks)} ({100*summary['baseline_qa_accuracy']:.0f}%)")
    print(f"Core fusion:  {fused_ok}/{len(tasks)} ({100*summary['core_fusion_qa_accuracy']:.0f}%)")
    print(f"Improved: {improved}  Regressed: {regressed}")
    print(f"Verdict: {summary['verdict']}")

    REPORTS.mkdir(parents=True, exist_ok=True)
    out = REPORTS / "context_core_fusion.json"
    out.write_text(json.dumps(summary, indent=2, ensure_ascii=False), encoding="utf-8")
    print(f"\n→ {out}")


if __name__ == "__main__":
    main()
