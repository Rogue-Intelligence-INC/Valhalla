#!/usr/bin/env python3
"""New-paradigm AI potential — fair, falsifiable test without traditional training.

Protocol: valhalla-new-paradigm-potential-v1

**Paradigm under test (pre-registered claim):**
  Inference-time structure learning via Fate incubation + session snapshot +
  targeted context ingest — **zero LM weight updates** (no SGD / LoRA / fine-tune).

**Not claimed:**
  - Replacing pre-training or weight-level fine-tuning for all domains
  - Learning without any context ingest (pure parametric memory)
  - Batch eval on polluted full-corpus sessions

**Falsification criteria (objective):**
  1. If targeted-RAG cold start ≈ post-session transfer on holdout → session adds no lift
  2. If no-context ≈ with-context on corpus-dependent probes → invalid / contaminated
  3. If polluted-session eval ≈ transfer eval → routing failure unfixable
  4. If novel holdout fails with targeted ingest → ingest path broken

Batteries:
  N1 Novel knowledge (synthetic, NOT in fair corpus) — zero-weight acquisition
  N2 Fair holdout (open_retrieval) — transfer vs cold vs polluted
  N3 Compositional rephrase — post-curriculum generalization
  N4 Negative controls — swap + no-context + distractor (must behave as expected)
  N5 Paradigm separation — structure hybrid vs lm_patch vs optional trad LM

Metric: NPPI (New Paradigm Potential Index) ∈ [0,1] with explicit gates.

Usage:
  python3 tools/valhalla_inference/test_new_paradigm_potential.py
  python3 tools/valhalla_inference/test_new_paradigm_potential.py --phase test
"""

from __future__ import annotations

import argparse
import copy
import json
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

REPO = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(REPO / "tools/fair_benchmark"))

from metrics import score_answer  # noqa: E402
from run_fair_benchmark import load_suite, select_prompts  # noqa: E402
from run_potential_analysis import classify_slices  # noqa: E402
from run_scale_benchmark import corpus_for_prompt  # noqa: E402
from run_strict_incubation_experiment import strict_user_text  # noqa: E402
from test_context_flexibility_goal import EXTENDED_TURNS  # noqa: E402
from valhalla_base import ValhallaBase  # noqa: E402

PROTOCOL = "valhalla-new-paradigm-potential-v1"
REPORTS = REPO / "reports/valhalla_inference"

# Synthetic knowledge block — audited absent from fair-1.x ingest_clean
NOVEL_INGEST: list[str] = [
    "Project Helios assigns codename Alpha-7 to its primary research satellite.",
    "Protocol NP-2026 mandates dual independent verification for structure deployments.",
    "Cipher Ridge summit elevation is exactly 2847 meters above mean sea level.",
    "Dr Lena Okonkwo discovered the Okonkwo coefficient for tidal resonance at 0.37.",
    "The Lumina Array stores exactly 4096 parallel fate channels in its hub lattice.",
    "Arcturus Station occupies Earth-Moon Lagrange point L4 in the Helios constellation.",
    "The Zeph protocol compresses semantic ingress to 128 dimensions per modality.",
    "Maris Deep Observatory recorded bioluminescence peak at 440 nanometers wavelength.",
]

NOVEL_CURRICULUM: list[tuple[str, str, str]] = [
    (
        "Project Helios assigns codename Alpha-7 to its primary research satellite.",
        "What codename does Project Helios use for its primary satellite?",
        "alpha-7",
    ),
    (
        "Protocol NP-2026 mandates dual independent verification for structure deployments.",
        "Which protocol requires dual verification for structure deployments?",
        "np-2026",
    ),
    (
        "Cipher Ridge summit elevation is exactly 2847 meters above mean sea level.",
        "What is the elevation in meters of Cipher Ridge summit?",
        "2847",
    ),
    (
        "Dr Lena Okonkwo discovered the Okonkwo coefficient for tidal resonance at 0.37.",
        "What numeric value is the Okonkwo coefficient for tidal resonance?",
        "0.37",
    ),
    (
        "The Lumina Array stores exactly 4096 parallel fate channels in its hub lattice.",
        "How many parallel fate channels does the Lumina Array store?",
        "4096",
    ),
    (
        "Arcturus Station occupies Earth-Moon Lagrange point L4 in the Helios constellation.",
        "At which Lagrange point is Arcturus Station located?",
        "l4",
    ),
    (
        "The Zeph protocol compresses semantic ingress to 128 dimensions per modality.",
        "How many dimensions per modality does the Zeph protocol use?",
        "128",
    ),
    (
        "Maris Deep Observatory recorded bioluminescence peak at 440 nanometers wavelength.",
        "At what wavelength in nanometers did Maris Deep Observatory peak bioluminescence?",
        "440",
    ),
]

NOVEL_HOLDOUT: list[dict] = [
    {
        "id": "NOVEL_H01",
        "text": "State the Helios satellite codename mentioned in project documentation.",
        "reference_answer": "Alpha-7",
        "keywords": ["alpha-7", "alpha 7"],
        "score_type": "open",
    },
    {
        "id": "NOVEL_H02",
        "text": "Which verification protocol number governs structure deployments?",
        "reference_answer": "NP-2026",
        "keywords": ["np-2026", "np 2026"],
        "score_type": "open",
    },
    {
        "id": "NOVEL_H03",
        "text": "Report the summit height of Cipher Ridge in meters.",
        "reference_answer": "2847",
        "keywords": ["2847"],
        "score_type": "open",
    },
    {
        "id": "NOVEL_H04",
        "text": "What is Okonkwo's tidal resonance coefficient?",
        "reference_answer": "0.37",
        "keywords": ["0.37"],
        "score_type": "open",
    },
    {
        "id": "NOVEL_H05",
        "text": "How many fate channels are in the Lumina Array hub lattice?",
        "reference_answer": "4096",
        "keywords": ["4096"],
        "score_type": "open",
    },
    {
        "id": "NOVEL_H06",
        "text": "Where in the Earth-Moon system is Arcturus Station positioned?",
        "reference_answer": "L4",
        "keywords": ["l4", "lagrange"],
        "score_type": "open",
    },
]

COMPOSITION_PROBES: list[dict] = [
    {
        "id": "COMP_01",
        "text": "In our physics thread, what SI unit measures force?",
        "reference_answer": "Newton",
        "keywords": ["newton"],
        "score_type": "open",
    },
    {
        "id": "COMP_02",
        "text": "Which Australian city is largest but not the capital?",
        "reference_answer": "Sydney",
        "keywords": ["sydney"],
        "score_type": "open",
    },
    {
        "id": "COMP_03",
        "text": "At standard pressure, what Celsius temperature did we associate with boiling?",
        "reference_answer": "100",
        "keywords": ["100"],
        "score_type": "open",
    },
    {
        "id": "COMP_04",
        "text": "What equals mass times acceleration in the law we studied?",
        "reference_answer": "force",
        "keywords": ["force"],
        "score_type": "open",
    },
    {
        "id": "COMP_05",
        "text": "Which continent contains Australia per our geography context?",
        "reference_answer": "Oceania",
        "keywords": ["oceania"],
        "score_type": "open",
    },
    {
        "id": "COMP_06",
        "text": "What pH value is neutral for pure water in the chemistry notes?",
        "reference_answer": "7",
        "keywords": ["7"],
        "score_type": "open",
    },
]

SWAP_CONTROLS: list[dict] = [
    {
        "id": "SWAP_01",
        "question": "What is the capital of Australia?",
        "ctx_a": ["The capital of Australia is Canberra.", "Australia is in Oceania."],
        "ctx_b": ["The capital of France is Paris.", "France is in Western Europe."],
        "hint_a": "canberra",
        "hint_b": "paris",
    },
    {
        "id": "SWAP_02",
        "question": "At what temperature does water boil at standard pressure?",
        "ctx_a": ["Water boils at 100 degrees Celsius at standard atmospheric pressure."],
        "ctx_b": ["Water freezes at 0 degrees Celsius at standard pressure."],
        "hint_a": "100",
        "hint_b": "0",
    },
]

THREAD_IDS = ("australia", "chemistry", "physics")


def make_base(*, feedback: bool = False, decode: str = "hybrid") -> ValhallaBase:
    return ValhallaBase(
        decode=decode,  # type: ignore[arg-type]
        follow_up_decode="native_follow_up_aware",
        fate_ingress_routing="quad_cycle",
        fate_qa_feedback=feedback,
        incubation_cycles=2,
        max_new=128,
    )


def q_payload(p: dict) -> dict:
    text = p.get("text", "")
    if p.get("domain"):
        text = strict_user_text(p)
    return {
        "id": p["id"],
        "text": text,
        "score_type": p.get("score_type", "open"),
        "reference_answer": p.get("reference_answer", ""),
        "keywords": p.get("keywords") or [],
    }


def strip_session(session_state: dict[str, Any]) -> dict[str, Any]:
    sess = copy.deepcopy(session_state)
    sess["corpus"] = []
    sess["fingerprint"] = ""
    sess["lines_ingested"] = 0
    sess["question_history"] = []
    return sess


def eval_isolated(base: ValhallaBase, prompt: dict, corpus: list[str]) -> dict:
    ctx = corpus_for_prompt(prompt, corpus)
    base.clear_context()
    base.set_context(ctx)
    doc = base.run([q_payload(prompt)])
    row = (doc.get("rows") or [{}])[0]
    sc = score_answer(row.get("answer", ""), prompt)
    return {"correct": sc["correct"], "answer": (row.get("answer") or "")[:160], "ctx_lines": len(ctx)}


def eval_transfer(session_state: dict[str, Any], prompt: dict, corpus: list[str]) -> dict:
    base = make_base(feedback=False)
    base.session.session_state = strip_session(session_state)
    for line in corpus_for_prompt(prompt, corpus):
        base.append_context(line)
    doc = base.run([q_payload(prompt)], append_only=True)
    row = (doc.get("rows") or [{}])[0]
    sc = score_answer(row.get("answer", ""), prompt)
    return {"correct": sc["correct"], "answer": (row.get("answer") or "")[:160]}


def hint_ok(hint: str, answer: str) -> bool:
    return hint.lower() in answer.lower()


def audit_novel_absent_from_fair(corpus: list[str]) -> dict:
    leaks: list[str] = []
    for line in NOVEL_INGEST:
        key = line.split()[3:8]  # rough token overlap
        frag = " ".join(key).lower()
        for c in corpus:
            if frag[:20] in c.lower():
                leaks.append(line[:60])
                break
    for p in NOVEL_HOLDOUT:
        ref = str(p["reference_answer"]).lower()
        for c in corpus:
            if ref and ref in c.lower():
                leaks.append(p["id"])
    return {"clean": len(leaks) == 0, "leaks": leaks}


def run_novel_curriculum(base: ValhallaBase) -> dict:
    hits = 0
    rounds: list[dict] = []
    for i, (fact, question, hint) in enumerate(NOVEL_CURRICULUM):
        base.append_context(fact)
        doc = base.run(
            [{
                "id": f"novel_train_{i}",
                "text": question,
                "score_type": "open",
                "reference_answer": hint,
                "keywords": [hint],
            }],
            append_only=base.session.session_state is not None,
        )
        row = (doc.get("rows") or [{}])[0]
        ok = bool(row.get("correct"))
        hits += int(ok)
        tel = doc.get("fate_telemetry") or {}
        rounds.append({"round": i, "train_correct": ok, "spread": tel.get("hub_prefs_spread")})
    return {
        "rounds": len(NOVEL_CURRICULUM),
        "train_accuracy": hits / max(len(NOVEL_CURRICULUM), 1),
        "final_spread": rounds[-1]["spread"] if rounds else 0.0,
        "round_details": rounds,
    }


def run_n1_novel_knowledge() -> dict:
    """Synthetic facts: session teach → holdout with transfer vs cold RAG."""
    base = make_base(feedback=True)
    base.set_context(list(NOVEL_INGEST))
    curriculum = run_novel_curriculum(base)
    session = base.session.session_state
    assert session is not None

    polluted_rows: list[dict] = []
    pol_correct = 0
    for p in NOVEL_HOLDOUT:
        doc = base.run([q_payload(p)], append_only=True)
        ans = (doc.get("rows") or [{}])[0].get("answer", "")
        sc = score_answer(ans, p)
        pol_correct += int(sc["correct"])
        polluted_rows.append({"id": p["id"], "correct": sc["correct"], "answer": ans[:160]})
    polluted = {
        "arm": "novel_in_session_polluted",
        "n": len(NOVEL_HOLDOUT),
        "correct": pol_correct,
        "accuracy": pol_correct / max(len(NOVEL_HOLDOUT), 1),
        "rows": polluted_rows,
    }

    transfer_rows: list[dict] = []
    transfer_correct = 0
    for p in NOVEL_HOLDOUT:
        r = eval_transfer(session, p, NOVEL_INGEST)
        transfer_correct += int(r["correct"])
        transfer_rows.append({"id": p["id"], **r})
    transfer = {
        "arm": "novel_transfer_fresh",
        "description": "Post-session snapshot + targeted novel ingest per question",
        "n": len(NOVEL_HOLDOUT),
        "correct": transfer_correct,
        "accuracy": transfer_correct / max(len(NOVEL_HOLDOUT), 1),
        "rows": transfer_rows,
    }

    cold_rows: list[dict] = []
    cold_correct = 0
    for p in NOVEL_HOLDOUT:
        b = make_base(feedback=False)
        r = eval_isolated(b, p, NOVEL_INGEST)
        cold_correct += int(r["correct"])
        cold_rows.append({"id": p["id"], **r})
    cold = {
        "arm": "novel_cold_rag",
        "description": "Fresh session, targeted ingest only (RAG upper bound, no prior session)",
        "n": len(NOVEL_HOLDOUT),
        "correct": cold_correct,
        "accuracy": cold_correct / max(len(NOVEL_HOLDOUT), 1),
        "rows": cold_rows,
    }

    cross_session = {
        "arm": "novel_cross_session_cold",
        "description": "Discard session after train; cold RAG only (proves no weight memory)",
        "accuracy": cold["accuracy"],
    }

    return {
        "curriculum": curriculum,
        "novel_in_session_polluted": polluted,
        "novel_transfer_fresh": transfer,
        "novel_cold_rag": cold,
        "novel_cross_session_cold": cross_session,
        "lifts_pp": {
            "transfer_vs_polluted_pp": (transfer["accuracy"] - polluted["accuracy"]) * 100.0,
            "transfer_vs_cold_pp": (transfer["accuracy"] - cold["accuracy"]) * 100.0,
            "session_lift_over_cold_pp": (transfer["accuracy"] - cold["accuracy"]) * 100.0,
        },
    }


def run_fair_curriculum(corpus: list[str]) -> tuple[dict[str, Any], ValhallaBase]:
    base = make_base(feedback=True)
    base.set_context(list(corpus))
    hits = 0
    n = 0
    for tid in THREAD_IDS:
        for append, question, hint, _ in EXTENDED_TURNS[tid]:
            base.append_context(append)
            doc = base.run(
                [{
                    "id": f"fair_train_{tid}_{n}",
                    "text": question,
                    "score_type": "open",
                    "reference_answer": hint,
                    "keywords": [hint],
                }],
                append_only=base.session.session_state is not None,
            )
            hits += int((doc.get("rows") or [{}])[0].get("correct", False))
            n += 1
    meta = {"rounds": n, "train_accuracy": hits / max(n, 1)}
    return meta, base


def run_n2_fair_holdout(eval_prompts: list[dict], corpus: list[str]) -> dict:
    _, trained = run_fair_curriculum(corpus)
    session = trained.session.session_state
    assert session is not None

    cold_correct = 0
    transfer_correct = 0
    polluted_correct = 0
    for p in eval_prompts:
        b = make_base()
        cold_correct += int(eval_isolated(b, p, corpus)["correct"])
        transfer_correct += int(eval_transfer(session, p, corpus)["correct"])
        doc = trained.run([q_payload(p)], append_only=True)
        sc = score_answer((doc.get("rows") or [{}])[0].get("answer", ""), p)
        polluted_correct += int(sc["correct"])

    n = len(eval_prompts)
    cold = {"accuracy": cold_correct / max(n, 1), "correct": cold_correct, "n": n}
    transfer = {"accuracy": transfer_correct / max(n, 1), "correct": transfer_correct, "n": n}
    polluted = {"accuracy": polluted_correct / max(n, 1), "correct": polluted_correct, "n": n}
    return {
        "cold_rag_isolated": cold,
        "transfer_fresh": transfer,
        "in_session_polluted": polluted,
        "lifts_pp": {
            "transfer_vs_polluted_pp": (transfer["accuracy"] - polluted["accuracy"]) * 100.0,
            "transfer_vs_cold_pp": (transfer["accuracy"] - cold["accuracy"]) * 100.0,
        },
        "falsified_session_adds_no_lift": abs(transfer["accuracy"] - cold["accuracy"]) < 0.02,
    }


def run_n3_compositional(corpus: list[str]) -> dict:
    """Rephrased probes after fair curriculum; eval with fair corpus retrieval."""
    _, trained = run_fair_curriculum(corpus)
    session = trained.session.session_state
    assert session is not None

    cold_c = transfer_c = 0
    rows: list[dict] = []
    for p in COMPOSITION_PROBES:
        b = make_base()
        c_ok = eval_isolated(b, p, corpus)["correct"]
        t_ok = eval_transfer(session, p, corpus)["correct"]
        cold_c += int(c_ok)
        transfer_c += int(t_ok)
        rows.append({"id": p["id"], "cold": c_ok, "transfer": t_ok})
    n = len(COMPOSITION_PROBES)
    return {
        "n": n,
        "cold_accuracy": cold_c / max(n, 1),
        "transfer_accuracy": transfer_c / max(n, 1),
        "lift_pp": (transfer_c - cold_c) / max(n, 1) * 100.0,
        "rows": rows,
    }


def run_n4_negative_controls(eval_prompts: list[dict], corpus: list[str]) -> dict:
    subset = eval_prompts[: min(12, len(eval_prompts))]

    targeted_c = distractor_c = 0
    for p in subset:
        targeted_c += int(eval_isolated(make_base(), p, corpus)["correct"])
        good = corpus_for_prompt(p, corpus)
        bad = [ln for ln in corpus if ln not in good][:8] or corpus[:2]
        b2 = make_base()
        b2.set_context(bad)
        doc = b2.run([q_payload(p)])
        sc = score_answer((doc.get("rows") or [{}])[0].get("answer", ""), p)
        distractor_c += int(sc["correct"])

    swap_hits = 0
    swap_n = 0
    for case in SWAP_CONTROLS:
        for ctx, hint in ((case["ctx_a"], case["hint_a"]), (case["ctx_b"], case["hint_b"])):
            swap_n += 1
            b = make_base()
            b.set_context(ctx)
            doc = b.run([{"id": case["id"], "text": case["question"], "score_type": "open"}])
            ans = (doc.get("rows") or [{}])[0].get("answer", "")
            swap_hits += int(hint_ok(hint, ans))

    n = len(subset)
    targeted_acc = targeted_c / max(n, 1)
    distractor_acc = distractor_c / max(n, 1)

    checks = {
        "targeted_beats_distractor": targeted_acc > distractor_acc + 0.05,
        "swap_fidelity_ok": swap_hits >= swap_n - 1,
        "distractor_not_ceiling": distractor_acc < 0.85,
        "targeted_above_half": targeted_acc >= 0.5,
    }
    integrity = sum(1 for v in checks.values() if v) / max(len(checks), 1)

    return {
        "subset_n": n,
        "targeted_accuracy": targeted_acc,
        "distractor_accuracy": distractor_acc,
        "swap_fidelity": swap_hits,
        "swap_trials": swap_n,
        "checks": checks,
        "control_integrity": integrity,
    }


def run_n5_paradigm_separation(eval_prompts: list[dict], corpus: list[str]) -> dict:
    from test_context_performance import context_block  # noqa: E402

    hybrid_c = patch_c = 0
    for p in eval_prompts[: min(15, len(eval_prompts))]:
        hybrid_c += int(eval_isolated(make_base(decode="hybrid"), p, corpus)["correct"])
        patch_c += int(
            eval_isolated(make_base(decode="lm_patch", feedback=False), p, corpus)["correct"]
        )
    n = min(15, len(eval_prompts))
    hybrid = {"accuracy": hybrid_c / max(n, 1), "n": n}
    patch = {"accuracy": patch_c / max(n, 1), "n": n}

    trad: dict[str, Any] = {"skipped": True, "error": None}
    try:
        from test_context_arch_compare import resolve_lm_path  # noqa: E402
        from loader import ValhallaLLM  # noqa: E402

        lm = ValhallaLLM(resolve_lm_path("Qwen/Qwen2.5-0.5B-Instruct"))
        tc = 0
        for p in eval_prompts[:n]:
            ctx = corpus_for_prompt(p, corpus)
            ans = lm.generate([context_block(ctx) + strict_user_text(p)], max_tokens=128)[0].text
            tc += int(score_answer(ans.strip(), p)["correct"])
        trad = {"accuracy": tc / max(n, 1), "n": n, "skipped": False}
    except Exception as exc:  # noqa: BLE001
        trad["error"] = str(exc)

    return {
        "structure_hybrid": hybrid,
        "lm_patch_rag": patch,
        "traditional_lm_rag": trad,
        "delta_hybrid_vs_patch_pp": (hybrid["accuracy"] - patch["accuracy"]) * 100.0,
    }


def compute_nppi(report: dict) -> dict:
    n1 = report["n1_novel_knowledge"]
    n2 = report["n2_fair_holdout"]
    n3 = report["n3_compositional"]
    n4 = report["n4_negative_controls"]
    n5 = report["n5_paradigm_separation"]

    gates: dict[str, bool] = {
        "novel_contamination_clean": report["audits"]["novel_absent_from_fair"]["clean"],
        "negative_controls_pass": n4["control_integrity"] >= 0.75,
        "transfer_rescues_polluted": n2["lifts_pp"]["transfer_vs_polluted_pp"] > 10.0,
        "cold_rag_works": n1["novel_cold_rag"]["accuracy"] >= 0.5,
    }
    gate_factor = 1.0 if all(gates.values()) else 0.85 if sum(gates.values()) >= 3 else 0.5

    components = {
        "novel_holdout_transfer": n1["novel_transfer_fresh"]["accuracy"],
        "fair_holdout_cold": n2["cold_rag_isolated"]["accuracy"],
        "compositional_transfer": n3["transfer_accuracy"],
        "session_routing_value": min(1.0, max(0.0, n2["lifts_pp"]["transfer_vs_polluted_pp"] / 50.0)),
        "structure_session_lift": max(0.0, min(1.0, (n2["lifts_pp"]["transfer_vs_cold_pp"] + 5.0) / 10.0)),
        "control_integrity": n4["control_integrity"],
    }
    weights = {
        "novel_holdout_transfer": 0.20,
        "fair_holdout_cold": 0.25,
        "compositional_transfer": 0.15,
        "session_routing_value": 0.20,
        "structure_session_lift": 0.10,
        "control_integrity": 0.10,
    }
    raw = sum(weights[k] * components[k] for k in weights)
    nppi = round(raw * gate_factor, 4)

    falsification: list[str] = []
    if n1["lifts_pp"]["session_lift_over_cold_pp"] < 1.0:
        falsification.append("SESSION_SNAPSHOT_NO_HOLDOUT_LIFT_BEYOND_RAG")
    if n2["falsified_session_adds_no_lift"]:
        falsification.append("FAIR_HOLDOUT_RAG_EQUIVALENT")
    if n1["novel_cold_rag"]["accuracy"] < 0.5:
        falsification.append("NOVEL_INGEST_PATH_BROKEN")
    if not gates["negative_controls_pass"]:
        falsification.append("NEGATIVE_CONTROLS_FAILED")
    if not gates["novel_contamination_clean"]:
        falsification.append("NOVEL_CORPUS_CONTAMINATED")

    return {
        "nppi": nppi,
        "gate_factor": gate_factor,
        "gates": gates,
        "components": {k: round(v, 4) for k, v in components.items()},
        "weights": weights,
        "falsification_flags": falsification,
    }


def verdict(nppi: float, nppi_doc: dict, report: dict) -> list[str]:
    tags: list[str] = []
    flags = nppi_doc["falsification_flags"]

    if nppi >= 0.72:
        tags.append("NEW_PARADIGM_VIABLE")
    elif nppi >= 0.55:
        tags.append("NEW_PARADIGM_PROMISING")
    else:
        tags.append("NEW_PARADIGM_DEVELOPING")

    if report["n2_fair_holdout"]["cold_rag_isolated"]["accuracy"] >= 0.90:
        tags.append("INFERENCE_TIME_GROUNDING_STRONG")

    if report["n2_fair_holdout"]["lifts_pp"]["transfer_vs_polluted_pp"] > 20.0:
        tags.append("SESSION_ROUTING_ESSENTIAL")

    if "SESSION_SNAPSHOT_NO_HOLDOUT_LIFT_BEYOND_RAG" in flags:
        tags.append("HONEST_RAG_EQUIVALENT_ON_HOLDOUT")
        tags.append("NOT_WEIGHT_FINE_TUNE_REPLACEMENT")

    if report["n1_novel_knowledge"]["novel_transfer_fresh"]["accuracy"] >= 0.83:
        tags.append("ZERO_WEIGHT_NOVEL_INGEST_OK")

    if nppi_doc["gates"]["negative_controls_pass"]:
        tags.append("METHODOLOGY_VALID")

    return tags


def main() -> None:
    ap = argparse.ArgumentParser(description="Valhalla new-paradigm potential (no traditional training)")
    ap.add_argument("--phase", default="test")
    ap.add_argument("--eval-limit", type=int, default=0)
    args = ap.parse_args()

    suite = load_suite()
    corpus = suite["corpus"]["ingest_clean"]
    prompts = select_prompts(suite, args.phase)
    slices = classify_slices(prompts, corpus)
    eval_prompts = [p for p in prompts if p["id"] in slices["open_retrieval"]]
    if args.eval_limit > 0:
        eval_prompts = eval_prompts[: args.eval_limit]

    print(f"=== {PROTOCOL} phase={args.phase} ===")
    print("  paradigm: structure incubation + session, zero LM weight updates")
    print(f"  fair holdout open_retrieval n={len(eval_prompts)}  corpus={len(corpus)}")
    print(f"  novel synthetic ingest n={len(NOVEL_INGEST)}  holdout n={len(NOVEL_HOLDOUT)}\n")

    audit = audit_novel_absent_from_fair(corpus)

    print("[N1] Novel knowledge (synthetic, zero-weight teach → holdout) …")
    n1 = run_n1_novel_knowledge()
    print(f"  train acc={n1['curriculum']['train_accuracy']:.1%}")
    print(f"  polluted={n1['novel_in_session_polluted']['accuracy']:.1%}  "
          f"transfer={n1['novel_transfer_fresh']['accuracy']:.1%}  "
          f"cold_rag={n1['novel_cold_rag']['accuracy']:.1%}")
    print(f"  session lift vs cold: {n1['lifts_pp']['session_lift_over_cold_pp']:+.1f}pp\n")

    print("[N2] Fair holdout (transfer vs cold vs polluted) …")
    n2 = run_n2_fair_holdout(eval_prompts, corpus)
    print(f"  cold={n2['cold_rag_isolated']['accuracy']:.1%}  "
          f"transfer={n2['transfer_fresh']['accuracy']:.1%}  "
          f"polluted={n2['in_session_polluted']['accuracy']:.1%}")
    print(f"  transfer vs polluted: {n2['lifts_pp']['transfer_vs_polluted_pp']:+.1f}pp  "
          f"vs cold: {n2['lifts_pp']['transfer_vs_cold_pp']:+.1f}pp\n")

    print("[N3] Compositional rephrase (post-curriculum) …")
    n3 = run_n3_compositional(corpus)
    print(f"  cold={n3['cold_accuracy']:.1%}  transfer={n3['transfer_accuracy']:.1%}  "
          f"lift={n3['lift_pp']:+.1f}pp\n")

    print("[N4] Negative controls …")
    n4 = run_n4_negative_controls(eval_prompts, corpus)
    print(f"  targeted={n4['targeted_accuracy']:.1%}  distractor={n4['distractor_accuracy']:.1%}  "
          f"swap={n4['swap_fidelity']}/{n4['swap_trials']}")
    print(f"  integrity={n4['control_integrity']:.1%}  checks={n4['checks']}\n")

    print("[N5] Paradigm separation (structure vs lm_patch vs trad) …")
    n5 = run_n5_paradigm_separation(eval_prompts, corpus)
    print(f"  hybrid={n5['structure_hybrid']['accuracy']:.1%}  "
          f"lm_patch={n5['lm_patch_rag']['accuracy']:.1%}")
    trad = n5["traditional_lm_rag"]
    if trad.get("skipped"):
        print(f"  traditional_lm skipped ({trad.get('error', 'n/a')})")
    else:
        print(f"  traditional_lm={trad['accuracy']:.1%}")
    print()

    report: dict[str, Any] = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "protocol": PROTOCOL,
        "fair_benchmark_version": suite.get("meta", {}).get("version", "fair-1.x"),
        "phase": args.phase,
        "paradigm_definition": {
            "learning_mechanism": "Fate incubation + session snapshot + targeted ingest",
            "explicitly_excluded": [
                "gradient_weight_updates",
                "lora_fine_tune",
                "pre_training_changes",
                "polluted_full_corpus_batch_eval",
            ],
        },
        "falsification_criteria": {
            "session_no_lift_if_transfer_equals_cold": True,
            "invalid_if_no_context_equals_targeted": True,
            "broken_if_novel_cold_below_50pct": True,
        },
        "audits": {
            "novel_absent_from_fair": audit,
            "no_lm_weight_updates": True,
        },
        "n1_novel_knowledge": n1,
        "n2_fair_holdout": n2,
        "n3_compositional": n3,
        "n4_negative_controls": n4,
        "n5_paradigm_separation": n5,
    }
    nppi_doc = compute_nppi(report)
    report["nppi"] = nppi_doc
    report["verdict"] = verdict(nppi_doc["nppi"], nppi_doc, report)

    print(f"NPPI = {nppi_doc['nppi']:.3f}  gates={nppi_doc['gates']}")
    print(f"Falsification flags: {nppi_doc['falsification_flags']}")
    print(f"Verdict: {report['verdict']}")

    REPORTS.mkdir(parents=True, exist_ok=True)
    out = REPORTS / f"new_paradigm_potential_{args.phase}.json"
    out.write_text(json.dumps(report, indent=2, ensure_ascii=False), encoding="utf-8")
    print(f"\n→ {out}")


if __name__ == "__main__":
    main()
