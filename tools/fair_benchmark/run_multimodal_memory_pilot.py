#!/usr/bin/env python3
"""Pilot: open-question memory with text+TTS audio fusion vs text-only (Triad).

Usage:
  VALHALLA_MULTIMODAL_FUSION=1 python tools/fair_benchmark/run_multimodal_memory_pilot.py
"""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
FAIR = Path(__file__).resolve().parent
ENCODE = REPO / "tools/valhalla_dl_front/encode.py"
OUT = REPO / "reports/tts_tile_stem_fusion/multimodal_memory_pilot.json"

sys.path.insert(0, str(FAIR))
from metrics import score_answer  # noqa: E402
from run_fair_benchmark import load_suite, select_prompts  # noqa: E402
from run_strict_incubation_experiment import strict_user_text  # noqa: E402
from session import native_qa_batch_persistent  # noqa: E402


def tts_espeak(text: str, wav: Path) -> None:
    subprocess.run(["espeak", "-w", str(wav), text], check=True, capture_output=True)


def encode_wav(wav: Path) -> list[float]:
    proc = subprocess.run(
        [sys.executable, str(ENCODE)],
        input=json.dumps({"modality": "audio_path", "path": str(wav)}),
        capture_output=True,
        text=True,
        check=True,
    )
    return json.loads(proc.stdout)["embedding"]


def run_track(
    corpus: list[str],
    prompts: list[dict],
    audio_by_id: dict[str, list[float]] | None,
    label: str,
) -> dict:
    batch_q = [
        (strict_user_text(p), p.get("score_type", "open"), p["id"]) for p in prompts
    ]
    results = native_qa_batch_persistent(
        "triad",
        corpus,
        batch_q,
        incubation_cycles=1,
        qa_cycles=1,
        audio_by_id=audio_by_id,
    )
    by_id = {r.get("id"): r for r in results}
    rows = []
    correct = 0
    for p in prompts:
        r = by_id.get(p["id"], {})
        ans = r.get("answer", "")
        scored = score_answer(ans, p)
        ok = scored.get("correct", False)
        correct += int(ok)
        rows.append({"id": p["id"], "correct": ok, "answer": ans, "reference": p.get("reference")})
    return {
        "label": label,
        "correct": correct,
        "total": len(prompts),
        "accuracy": correct / max(len(prompts), 1),
        "rows": rows,
    }


def main() -> None:
    suite = load_suite()
    prompts = [p for p in select_prompts(suite, "test") if p.get("score_type") == "open"]
    corpus = [strict_user_text(p) for p in select_prompts(suite, "test")]

    audio_by_id: dict[str, list[float]] = {}
    with tempfile.TemporaryDirectory() as td:
        tmp = Path(td)
        for p in prompts:
            wav = tmp / f"{p['id']}.wav"
            tts_espeak(strict_user_text(p), wav)
            audio_by_id[p["id"]] = encode_wav(wav)

    text_only = run_track(corpus, prompts, None, "text_only")
    import os

    os.environ["VALHALLA_MULTIMODAL_FUSION"] = "1"
    fusion = run_track(corpus, prompts, audio_by_id, "text_plus_tts_audio")

    report = {
        "suite": "fair_benchmark_test_open_only",
        "n_open": len(prompts),
        "text_only": text_only,
        "text_plus_tts_audio": fusion,
        "delta_correct": fusion["correct"] - text_only["correct"],
        "delta_pp": (fusion["accuracy"] - text_only["accuracy"]) * 100,
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(json.dumps({k: report[k] for k in report if k != "text_only" and k != "text_plus_tts_audio"}, indent=2))
    print(f"text_only: {text_only['correct']}/{text_only['total']}")
    print(f"fusion:    {fusion['correct']}/{fusion['total']}  (delta {report['delta_correct']:+d})")
    print(f"Wrote {OUT}")


if __name__ == "__main__":
    main()
