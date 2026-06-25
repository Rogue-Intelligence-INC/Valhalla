#!/usr/bin/env python3
"""Test ValhallaBase v1 extensions: (1) hybrid (2) session append (3) multimodal context."""

from __future__ import annotations

import json
import sys
from datetime import datetime, timezone
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

from valhalla_base import ValhallaBase  # noqa: E402

REPORTS = Path(__file__).resolve().parents[2] / "reports/valhalla_inference"


def load_sample_features() -> tuple[list[float], list[float]]:
    p = Path(__file__).resolve().parents[2] / "valhalla-unified-core/reports/cross_modal_samples_test.json"
    if p.is_file():
        s = json.loads(p.read_text())["samples"][0]
        return s["audio_features"][:64], s["image_features"][:64]
    return [0.01] * 64, [0.02] * 64


def main() -> None:
    audio, image = load_sample_features()
    base = ValhallaBase(decode="hybrid", max_new=96)
    report: dict = {"timestamp": datetime.now(timezone.utc).isoformat(), "tests": {}}

    # --- 1. Hybrid routing ---
    print("=== 1. Hybrid routing ===")
    base.set_context(["The SI unit of force is the Newton.", "F = ma relates force and acceleration."])
    hybrid_doc = base.run(
        [
            {
                "id": "hybrid_open",
                "text": "What is the SI unit of force?",
                "score_type": "open",
                "reference_answer": "Newton",
                "keywords": ["Newton", "N"],
            },
            {
                "id": "hybrid_mcq",
                "text": "What is the SI unit of force?\nA) Joule\nB) Newton\nC) Watt\nD) Pascal",
                "score_type": "mcq",
                "reference_answer": "B",
            },
        ]
    )
    for row in hybrid_doc["rows"]:
        print(f"  [{row['score_type']}] {row['id']} → {row['decode_path']}")
        print(f"    {row['answer'][:120]}")
    report["tests"]["hybrid"] = {
        "features": hybrid_doc.get("features"),
        "rows": [
            {"id": r["id"], "decode_path": r["decode_path"], "answer": r["answer"][:200]}
            for r in hybrid_doc["rows"]
        ],
    }

    # --- 2. Persistent session append ---
    print("\n=== 2. Session append (incremental context) ===")
    base.clear_context()
    base.set_context(["Planet Alpha: water boils at 100 degrees Celsius."])
    r1 = base.run([{"id": "s1", "text": "Boiling point on this planet?", "score_type": "open"}])
    fp1 = r1["context_fingerprint"]
    ans1 = r1["rows"][0]["answer"]
    print(f"  after init fp={fp1}: {ans1[:100]}")

    r2 = base.run(
        [{"id": "s2", "text": "Boiling point on this planet?", "score_type": "open"}],
        append_only=True,
        context_items=[{"text": "Correction: on Alpha water boils at 85C due to altitude."}],
    )
    fp2 = r2["context_fingerprint"]
    ans2 = r2["rows"][0]["answer"]
    print(f"  after append fp={fp2}: {ans2[:100]}")
    report["tests"]["session"] = {
        "fp_before": fp1,
        "fp_after": fp2,
        "fingerprint_changed": fp1 != fp2,
        "lines_ingested": r2["session"]["lines_ingested"],
        "features": r2.get("features"),
    }

    # --- 3. Multimodal context bundle ---
    print("\n=== 3. Multimodal context (text + audio + image features) ===")
    base.clear_context()
    mm_doc = base.run(
        [{"id": "mm1", "text": "What domain does this context describe?", "score_type": "open"}],
        context_items=[
            {"text": "Photosynthesis occurs in chloroplasts using light energy."},
            {"audio_features": audio},
            {"image_features": image},
        ],
    )
    print(f"  features: {mm_doc.get('features')}")
    print(f"  answer: {mm_doc['rows'][0]['answer'][:150]}")
    report["tests"]["multimodal"] = {
        "features": mm_doc.get("features"),
        "context_lines": mm_doc.get("context_lines"),
        "answer": mm_doc["rows"][0]["answer"][:300],
    }

    REPORTS.mkdir(parents=True, exist_ok=True)
    out = REPORTS / "valhalla_base_v1_extensions.json"
    out.write_text(json.dumps(report, indent=2, ensure_ascii=False), encoding="utf-8")
    print(f"\n→ {out}")


if __name__ == "__main__":
    main()
