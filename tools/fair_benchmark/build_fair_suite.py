#!/usr/bin/env python3
"""Build fair_benchmark_suite.json — decontaminated corpus + val/test splits."""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
LEGACY_SUITE = REPO / "tools/valhalla_model_bridge/incubation_benchmark_suite.json"
OUT = Path(__file__).resolve().parent / "fair_benchmark_suite.json"

# Fixed smoke set (12) — fast regression
SMOKE_IDS = [
    "GSM_01", "GSM_02", "GSM_03", "GSM_04",
    "MATH_01", "MATH_02", "MATH_03", "MATH_04",
    "MCQ_PHYS_01", "MCQ_PHYS_02", "MCQ_CHEM_01", "MCQ_CHEM_02",
]

SPLIT_SEED = 20260619
VAL_RATIO = 0.24  # ~48 of 200


def _split_bucket(prompt_id: str, score_type: str) -> str:
    h = hashlib.sha256(f"{SPLIT_SEED}:{prompt_id}:{score_type}".encode()).hexdigest()
    v = int(h[:8], 16) / 0xFFFFFFFF
    if prompt_id in SMOKE_IDS:
        return "smoke"
    return "val" if v < VAL_RATIO else "test"


def build() -> dict:
    from contamination import corpus_contamination_report, filter_clean_corpus, filter_padding_lines

    legacy = json.loads(LEGACY_SUITE.read_text(encoding="utf-8"))
    prompts = legacy["prompts"]
    legacy_medium = legacy["corpus"]["medium"]
    legacy_large = legacy["corpus"]["large"]

    clean_medium, removed = filter_clean_corpus(legacy_medium, prompts)
    clean_large, _ = filter_clean_corpus(legacy_large, prompts)
    # Dedupe, drop padding filler lines
    seen = set()
    clean_ingest = []
    for line in filter_padding_lines(clean_large):
        if line not in seen:
            seen.add(line)
            clean_ingest.append(line)

    audit_clean = corpus_contamination_report(clean_ingest, prompts)
    audit_legacy = corpus_contamination_report(legacy_medium, prompts)

    splits: dict[str, list[str]] = {"smoke": [], "val": [], "test": []}
    for p in prompts:
        splits[_split_bucket(p["id"], p.get("score_type", "open"))].append(p["id"])

    # Ensure smoke exactly 12
    assert len(splits["smoke"]) == len(SMOKE_IDS)

    return {
        "meta": {
            "version": "fair-1.1",
            "date": "2026-06-19",
            "name": "Valhalla Fair Benchmark",
            "prompt_count": len(prompts),
            "split_seed": SPLIT_SEED,
            "val_ratio": VAL_RATIO,
            "principles": [
                "ingest_corpus must not contain eval reference answers",
                "val split for development/tuning only; test is held-out",
                "memory and generate tracks reported separately",
            ],
            "contamination_audit": {
                "clean_ingest": {
                    "lines": len(clean_ingest),
                    "clean": audit_clean["clean"],
                    "removed_lines": len(removed),
                    "numeric_eval_leaks": audit_clean.get("numeric_prompt_leak_count", 0),
                },
                "legacy_medium": {
                    "lines": len(legacy_medium),
                    "contaminated_line_count": audit_legacy["contaminated_line_count"],
                    "contaminated_prompt_count": audit_legacy["contaminated_prompt_count"],
                },
            },
        },
        "prompts": prompts,
        "splits": splits,
        "corpus": {
            "ingest_clean": clean_ingest,
            "ingest_legacy_contaminated": legacy_medium,
        },
        "phases": {
            "smoke": {
                "split": "smoke",
                "corpus": "ingest_clean",
                "description": "12Q fast regression; CI gate",
            },
            "val": {
                "split": "val",
                "corpus": "ingest_clean",
                "description": "Development/tuning only — do not report as final",
            },
            "test": {
                "split": "test",
                "corpus": "ingest_clean",
                "description": "Held-out primary report split",
            },
            "test_legacy_ablation": {
                "split": "test",
                "corpus": "ingest_legacy_contaminated",
                "description": "Ablation only: shows leakage inflation vs fair corpus",
            },
        },
        "arms": {
            "memory_tier_b": [
                {"id": "baseline_no_corpus", "body": "triad", "cycles": 1, "use_corpus": False, "session": "isolated"},
                {"id": "hub_c1", "body": "hub", "cycles": 1, "use_corpus": True, "session": "isolated"},
                {"id": "tile_c1", "body": "tile", "cycles": 1, "use_corpus": True, "session": "isolated"},
                {"id": "stemcell_c1", "body": "stemcell", "cycles": 1, "use_corpus": True, "session": "isolated"},
                {"id": "triad_c1", "body": "triad", "cycles": 1, "use_corpus": True, "session": "isolated"},
                {
                    "id": "triad_c1_persistent",
                    "body": "triad",
                    "cycles": 1,
                    "use_corpus": True,
                    "session": "persistent",
                    "incubation_cycles": 1,
                    "qa_cycles": 1,
                },
            ],
            "patch_tier_a": [
                {
                    "id": "patch_tile_legacy",
                    "body": "tile",
                    "patch_mode": "legacy",
                    "strength": 0.08,
                    "cycles": 1,
                },
                {
                    "id": "patch_stemcell_legacy",
                    "body": "stemcell",
                    "patch_mode": "legacy",
                    "strength": 0.08,
                    "cycles": 1,
                },
                {
                    "id": "patch_triad_legacy",
                    "body": "triad",
                    "patch_mode": "legacy",
                    "strength": 0.08,
                    "cycles": 1,
                },
            ],
            "generate_baseline": [
                {"id": "transformer_zero_shot", "model": "Qwen/Qwen2.5-0.5B-Instruct"},
            ],
        },
        "removed_corpus_lines": removed[:20],
    }


def main() -> None:
    suite = build()
    OUT.write_text(json.dumps(suite, indent=2, ensure_ascii=False), encoding="utf-8")
    m = suite["meta"]
    print(f"Wrote {OUT}", file=sys.stderr)
    print(
        f"  prompts={m['prompt_count']} smoke={len(suite['splits']['smoke'])} "
        f"val={len(suite['splits']['val'])} test={len(suite['splits']['test'])}",
        file=sys.stderr,
    )
    ca = m["contamination_audit"]
    print(
        f"  clean_corpus={ca['clean_ingest']['lines']} lines, clean={ca['clean_ingest']['clean']}",
        file=sys.stderr,
    )
    print(
        f"  legacy_medium leaks: {ca['legacy_medium']['contaminated_line_count']} lines "
        f"→ {ca['legacy_medium']['contaminated_prompt_count']} prompts",
        file=sys.stderr,
    )


if __name__ == "__main__":
    main()
