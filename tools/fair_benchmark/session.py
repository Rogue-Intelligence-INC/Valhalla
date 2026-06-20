#!/usr/bin/env python3
"""Native QA session protocols for fair benchmark."""

from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
GRINDER = REPO / "tools/transformer_grinder"
NATIVE_BIN = REPO / "target/release/valhalla_native_qa"

if str(GRINDER) not in sys.path:
    sys.path.insert(0, str(GRINDER))

from grind import grind_shard_for_native_qa, grind_text  # noqa: E402


def _run_native(req: dict) -> dict:
    proc = subprocess.run(
        [str(NATIVE_BIN)],
        input=json.dumps(req),
        capture_output=True,
        text=True,
        check=True,
        env={**os.environ, "RUSTFLAGS": "-L /opt/cuda/lib64"},
    )
    return json.loads(proc.stdout)


def native_qa_isolated(
    body: str,
    cycles: int,
    corpus: list[str],
    question: str,
    score_type: str,
) -> dict:
    """Per-question: full corpus re-ingest (legacy protocol)."""
    return _run_native(
        {
            "body": body,
            "cycles": cycles,
            "corpus": corpus,
            "question": question,
            "score_type": score_type,
        }
    )


def _grind_ports(lines: list[str], backend: str = "hash") -> list[dict]:
    return [grind_shard_for_native_qa(grind_text(line, backend=backend)) for line in lines]


def native_qa_batch_persistent(
    body: str,
    corpus: list[str],
    questions: list[tuple[str, str, str | None]],
    *,
    incubation_cycles: int = 1,
    qa_cycles: int = 1,
    grinder_backend: str = "hash",
) -> list[dict]:
    """
    One incubation over full corpus, then sequential QA without re-ingesting corpus.

    questions: list of (question_text, score_type, optional_id)
    """
    corpus_ports = _grind_ports(corpus, backend=grinder_backend)
    q_specs = []
    for qtext, stype, qid in questions:
        q_ports = grind_shard_for_native_qa(grind_text(qtext, backend=grinder_backend))
        q_specs.append(
            {
                "question": qtext,
                "score_type": stype,
                "question_ports": q_ports,
                "id": qid or "",
            }
        )

    resp = _run_native(
        {
            "body": body,
            "corpus": corpus,
            "corpus_ports": corpus_ports,
            "questions": q_specs,
            "incubation_cycles": incubation_cycles,
            "qa_cycles": qa_cycles,
            "ingress_mode": "grinder",
        }
    )
    return resp.get("results", [])
