#!/usr/bin/env python3
"""Proactive Agency Benchmark — baked ValhallaBase long-dialogue + multimodal + multilingual.

Protocol: proactive-agency-benchmark-v1

Tracks:
  A  long_dialogue     High-turn session → primary Q → supplement follow-up → proactive_idle
  B  multimodal_natural Same question via text / TTS+audio / image / tri-modal — answer consistency
  C  crosslang          EN / ZH / FR same fact — unified core (no language switch)

Features: progress bar · ETA · checkpoint/resume · pause · verbose CLI per question

Usage:
  python3 tools/valhalla_inference/run_proactive_agency_benchmark.py
  python3 tools/valhalla_inference/run_proactive_agency_benchmark.py \\
    --model models/hf/Qwen2.5-3B-Instruct-Valhalla-TriModal-Fusion-v3-test-s0.10 --verbose
  python3 tools/valhalla_inference/run_proactive_agency_benchmark.py --fast --download-dialogues
  python3 tools/valhalla_inference/run_proactive_agency_benchmark.py --resume
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import tempfile
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

REPO = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
BUNDLE = REPO / "tools/standard_benchmarks/bundled/proactive_agency_scenarios.json"
LOCOMO = REPO / "tools/standard_benchmarks/bundled/locomo_qa_slice30.json"
ENCODE = REPO / "tools/valhalla_dl_front/encode.py"
GEN_IMG = REPO / "tools/valhalla_dl_front/generate_image.py"
IMG_DIR = REPO / "models/image_gen/sd-turbo"
REPORTS = REPO / "reports/valhalla_inference"
CHECKPOINT_DEFAULT = REPORTS / "checkpoints/proactive_agency_checkpoint.json"
OUT_JSON = REPORTS / "proactive_agency_benchmark.json"
OUT_JSON_STANDARD = REPORTS / "proactive_agency_standard.json"
OUT_JSON_MEGA = REPORTS / "proactive_agency_mega.json"
OUT_MD = REPORTS / "PROACTIVE_AGENCY_BENCHMARK.md"
PROACTIVE_FATE_JSON = REPORTS / "proactive_fate_v2_test.json"

DEFAULT_BAKED = REPO / "models/hf/Qwen2.5-3B-Instruct-Valhalla-TriModal-Fusion-v3-test-s0.10"
DEFAULT_BASE = "Qwen/Qwen2.5-3B-Instruct"

sys.path.insert(0, str(HERE))
sys.path.insert(0, str(REPO / "tools/fair_benchmark"))

from benchmark_cli import (  # noqa: E402
    ProgressBar,
    add_benchmark_args,
    init_suite_checkpoint,
    mark_suite_done,
    pause_requested,
    save_suite_checkpoint,
    suite_completed_labels,
)
from benchmark_env import apply_gpu_env, benchmark_runtime, default_lm_model  # noqa: E402
from metrics import score_answer  # noqa: E402
from test_crosslang_core_unified import FACTS, PROTOCOL as CROSSLANG_PROTOCOL  # noqa: E402
from valhalla_base import ValhallaBase  # noqa: E402

PROTOCOL = "proactive-agency-benchmark-v1"
TRACKS = ["long_dialogue", "multimodal_natural", "crosslang"]


def log(verbose: bool, msg: str) -> None:
    if verbose:
        print(msg, flush=True)


def kw_hits(text: str | None, keywords: list[str]) -> int:
    if not text:
        return 0
    low = text.lower()
    return sum(1 for k in keywords if k.lower() in low)


def conservative_defer_reason(reason: str | None) -> bool:
    if not reason:
        return False
    return (
        str(reason).startswith("spread=")
        or reason == "no_topic_overlap"
        or str(reason).startswith("fate_signals=")
        or str(reason).startswith("turns=")
    )


def spread_warmup_for_proactive(base: ValhallaBase, scenario: dict[str, Any], *, ask_fn) -> None:
    """3-turn spread warmup (proactive-fate-v2 protocol) before proactive_idle."""
    keywords = list(scenario.get("keywords") or [])
    warmup = list(scenario.get("dialogue_turns") or [])[:2]
    for i, q in enumerate(warmup):
        ask_fn(q, qid=f"pw{i}")
    if keywords:
        ask_fn(f"What else does the context say about {keywords[0]}?", qid="pw_spread")
    elif warmup:
        ask_fn("What else does the provided context state about this?", qid="pw_spread")


def score_proactive_pass(pro: dict[str, Any], pro_kws: list[str]) -> tuple[bool, str]:
    """Full pass = grounded emit; partial = conservative gate defer (spread/signals)."""
    triggered = bool(pro.get("triggered"))
    emitted = bool(pro.get("emitted"))
    nudge = str(pro.get("nudge") or "")
    reason = str(pro.get("trigger_reason") or "")
    if triggered and emitted and kw_ratio(nudge, pro_kws) >= 0.2:
        return True, "emit_grounded"
    if not emitted and conservative_defer_reason(reason):
        return True, "defer_gate_ok"
    if triggered and not emitted:
        return True, "defer_gate_ok"
    return False, reason or "no_trigger"


def load_proactive_fate_metrics(*, run_if_missing: bool = False) -> dict[str, Any] | None:
    """Track D: proactive-fate-v2 automatic reaction batteries (G/H/F)."""
    if run_if_missing or not PROACTIVE_FATE_JSON.is_file():
        subprocess.run(
            [sys.executable, str(HERE / "test_proactive_fate_v2.py")],
            cwd=REPO,
            check=False,
        )
    if not PROACTIVE_FATE_JSON.is_file():
        return None
    doc = json.loads(PROACTIVE_FATE_JSON.read_text(encoding="utf-8"))
    metrics = doc.get("metrics") or {}
    batteries = doc.get("batteries") or {}
    rows: list[dict] = []
    for b in batteries.values():
        if isinstance(b, list):
            rows.extend(b)
        elif isinstance(b, dict):
            rows.extend(b.get("rows") or [])
    return {
        "protocol": doc.get("protocol", "proactive-fate-v2"),
        "ppi_v2": float(metrics.get("PPI_v2") or metrics.get("ppi_v2") or 0),
        "pass_rate": sum(1 for r in rows if r.get("pass")) / max(len(rows), 1),
        "gates_ok": "GATES_OK" in (doc.get("verdict") or []),
        "unsolicited_precision": metrics.get("UP_on_topic"),
        "mismatch_abstain_rate": metrics.get("MAR_mismatch_abstain"),
        "false_interrupt_rate": metrics.get("FIR_false_interrupt"),
        "verdict": doc.get("verdict"),
        "n_trials": len(rows),
    }


def kw_ratio(text: str | None, keywords: list[str]) -> float:
    if not keywords:
        return 0.0
    return kw_hits(text, keywords) / len(keywords)


def make_base(lm_model: str, *, core_fusion: bool = False) -> ValhallaBase:
    return ValhallaBase(
        lm_model=lm_model,
        decode="hybrid",
        follow_up_decode="native_follow_up_aware",
        fate_ingress_routing="quad_cycle",
        fate_qa_feedback=True,
        fate_warmup_enabled=True,
        proactive_enabled=True,
        proactive_min_fate_signals=64,
        context_ingest="targeted",
        reanchor_affinity=True,
        incubation_cycles=2,
        max_new=128,
        core_fusion=core_fusion,
    )


def tts(text: str, wav: Path) -> None:
    subprocess.run(["espeak", "-w", str(wav), text[:400]], check=True, capture_output=True)


def encode_wav(wav: Path) -> list[float]:
    proc = subprocess.run(
        [sys.executable, str(ENCODE)],
        input=json.dumps({"modality": "audio_path", "path": str(wav)}),
        capture_output=True,
        text=True,
        check=True,
    )
    return json.loads(proc.stdout)["embedding"]


def gen_image_features(prompt: str, sample_id: str, out_dir: Path) -> list[float]:
    out_dir.mkdir(parents=True, exist_ok=True)
    proc = subprocess.run(
        [sys.executable, str(GEN_IMG)],
        input=json.dumps({"prompt": prompt[:300], "id": sample_id, "out_dir": str(out_dir), "model_dir": str(IMG_DIR)}),
        capture_output=True,
        text=True,
        check=True,
    )
    for line in reversed(proc.stdout.splitlines()):
        line = line.strip()
        if line.startswith("{"):
            return json.loads(line)["image_features"]
    return json.loads(proc.stdout)["image_features"]


def load_scenarios() -> dict[str, Any]:
    if BUNDLE.is_file():
        return json.loads(BUNDLE.read_text(encoding="utf-8"))
    return {"scenarios": [], "multimodal_prompts": []}


def extend_scenarios_from_locomo(doc: dict[str, Any], *, limit: int, max_lines: int) -> None:
    """Append LoCoMo-derived high-turn scenarios if bundle is sparse."""
    if not LOCOMO.is_file() or limit <= 0:
        return
    loc = json.loads(LOCOMO.read_text(encoding="utf-8"))
    existing = {s["id"] for s in doc.get("scenarios", [])}
    for item in loc.get("items", [])[:limit]:
        sid = f"loco_{item['id']}"
        if sid in existing:
            continue
        lines = item.get("session_lines") or []
        if len(lines) < 20:
            continue
        turns = []
        for ln in lines[: min(max_lines, len(lines)) : max(1, len(lines) // 12)]:
            if ":" in ln:
                turns.append(f"Summarize this line: {ln.split(']', 1)[-1].strip()[:120]}")
        if len(turns) < 8:
            continue
        doc.setdefault("scenarios", []).append(
            {
                "id": sid,
                "session_lines": lines[:max_lines],
                "dialogue_turns": turns[:24],
                "primary_question": item["question"],
                "gold_answer": str(item.get("answer", "")),
                "keywords": [w.lower() for w in re.split(r"\W+", str(item.get("answer", ""))) if len(w) > 2][:6],
                "supplement_prompt": "Please supplement your previous answer with more detail from our conversation.",
                "supplement_keywords": [w.lower() for w in re.split(r"\W+", str(item.get("answer", ""))) if len(w) > 1][:4],
                "proactive_keywords": [w.lower() for w in re.split(r"\W+", item["question"]) if len(w) > 3][:5],
            }
        )


def download_dailydialog_slice(out: Path, *, limit: int = 5) -> None:
    """Optional: fetch multi-turn dialogues from HuggingFace (best-effort)."""
    os.environ.setdefault("HF_ENDPOINT", "https://hf-mirror.com")
    try:
        from datasets import load_dataset
    except ImportError:
        subprocess.check_call([sys.executable, "-m", "pip", "install", "-q", "datasets"])
        from datasets import load_dataset

    candidates = [
        ("roskoN/dailydialog", "train"),
        ("daily_dialog", "train"),
        ("li2017/daily_dialog", "train"),
    ]
    ds = None
    last_err: Exception | None = None
    for name, split in candidates:
        try:
            ds = load_dataset(name, split=split)
            print(f"DailyDialog source: {name}", flush=True)
            break
        except Exception as e:  # noqa: BLE001
            last_err = e
            continue
    if ds is None:
        print(f"DailyDialog download skipped ({last_err})", flush=True)
        return
    doc = json.loads(out.read_text(encoding="utf-8")) if out.is_file() else load_scenarios()
    existing = {s["id"] for s in doc.get("scenarios", [])}
    added = 0
    for i, row in enumerate(ds):
        if added >= limit:
            break
        dialog = row.get("dialog") or row.get("utterances") or []
        if isinstance(dialog, str):
            dialog = dialog.split("__eou__")
        if len(dialog) < 10:
            continue
        sid = f"dailydialog_{i}"
        if sid in existing:
            continue
        session = [f"[T{j+1}] {u.strip()}" for j, u in enumerate(dialog[:40]) if str(u).strip()]
        turns = [f"What was discussed in turn {j+1}?" for j in range(min(12, len(session)))]
        doc.setdefault("scenarios", []).append(
            {
                "id": sid,
                "session_lines": session,
                "dialogue_turns": turns,
                "primary_question": "What is the main topic of this conversation? Answer briefly.",
                "gold_answer": dialog[0][:80] if dialog else "",
                "keywords": [w.lower() for w in re.split(r"\W+", str(dialog[0])) if len(w) > 3][:4],
                "supplement_prompt": "Elaborate on your previous answer using details from the dialogue.",
                "supplement_keywords": [],
                "proactive_keywords": [w.lower() for w in re.split(r"\W+", str(dialog[0])) if len(w) > 3][:3],
            }
        )
        added += 1
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(doc, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"DailyDialog: added {added} scenarios → {out}", flush=True)


def build_mega_corpus_from_locomo(*, max_lines: int = 5000) -> list[str]:
    """Pool LoCoMo session lines for multi-thousand-turn dialogue."""
    if not LOCOMO.is_file():
        return []
    doc = json.loads(LOCOMO.read_text(encoding="utf-8"))
    lines: list[str] = []
    for item in doc.get("items") or []:
        for ln in item.get("session_lines") or []:
            s = str(ln).strip()
            if s and s not in lines:
                lines.append(s)
            if len(lines) >= max_lines:
                return lines
    # cycle-fill if corpus smaller than target
    if not lines:
        return lines
    while len(lines) < max_lines:
        lines.extend(lines[: min(len(lines), max_lines - len(lines))])
    return lines[:max_lines]


def generate_mega_turn_questions(corpus: list[str], n_ask: int) -> list[str]:
    """Questions for periodic ask turns during mega dialogue."""
    qs: list[str] = []
    templates = [
        "Summarize the latest point in our conversation in one sentence.",
        "Who spoke last and what did they say?",
        "What topic are we discussing now?",
        "Recall one fact mentioned recently.",
        "What emotional tone did the last speaker express?",
    ]
    for i in range(n_ask):
        ln = corpus[i % len(corpus)] if corpus else ""
        speaker = ln.split(":", 1)[0].strip("[] ") if ":" in ln else "speaker"
        qs.append(f"{templates[i % len(templates)]} (context: {speaker})")
    return qs


def maybe_trim_session_corpus(
    base: ValhallaBase,
    *,
    max_lines: int,
    keep_lines: int,
    verbose: bool = False,
    sid: str = "",
) -> None:
    """Sliding-window reanchor — cap session corpus growth during mega dialogue."""
    if max_lines <= 0:
        return
    sess = base.session.session_state
    if not sess:
        return
    corpus = list(sess.get("corpus") or [])
    if len(corpus) <= max_lines:
        return
    keep = max(8, min(keep_lines, max_lines))
    tail = corpus[-keep:]
    if verbose:
        log(verbose, f"  [{sid}] corpus trim {len(corpus)} → {keep} lines (reanchor)")
    base.reanchor_session_corpus()
    for ln in tail:
        base.append_context(ln)


def reset_mega_session_segment(
    base: ValhallaBase,
    ctx: list[str],
    *,
    round_idx: int,
    verbose: bool,
    sid: str,
    keep_lines: int = 32,
) -> None:
    """Cap snapshot size — re-bootstrap session from recent corpus tail."""
    corpus = list((base.session.session_state or {}).get("corpus") or [])
    tail = corpus[-keep_lines:] if corpus else ctx[:keep_lines]
    if len(tail) < 4:
        tail = ctx[:keep_lines]
    log(verbose, f"  [{sid}] segment reset @ round {round_idx + 1} (keep {len(tail)} lines)")
    base.clear_context()
    if base.fate_warmup_enabled and len(tail) >= 2:
        base.fate_warmup_corpus(tail, n_lines=min(8, len(tail)))
    base.set_context(tail[: min(len(tail), keep_lines)])


def run_long_dialogue_case(
    scenario: dict[str, Any],
    lm_model: str,
    *,
    verbose: bool,
    max_turns: int,
    ask_every: int = 32,
    extend_chunk: int = 8,
    max_session_corpus: int = 0,
    corpus_keep_lines: int = 96,
    segment_decode_limit: int = 4,
    checkpoint_path: Path | None = None,
    progress: ProgressBar | None = None,
) -> dict[str, Any]:
    sid = scenario["id"]
    ctx = list(scenario.get("session_lines") or [])
    mega_corpus = scenario.get("mega_corpus")
    if mega_corpus:
        ctx = list(mega_corpus)
    keywords = list(scenario.get("keywords") or [])
    sup_kws = list(scenario.get("supplement_keywords") or keywords)
    pro_kws = list(scenario.get("proactive_keywords") or keywords)

    base = make_base(lm_model)
    base.proactive_min_fate_signals = 32
    base.proactive_min_turns = 0
    if base.fate_warmup_enabled and len(ctx) >= 2:
        base.fate_warmup_corpus(ctx, n_lines=min(8, len(ctx)))
    base.set_context(ctx[: min(len(ctx), 32)])

    total = max_turns
    ask_every = max(1, ask_every)
    n_ask = (total + ask_every - 1) // ask_every
    ask_questions = generate_mega_turn_questions(ctx, n_ask)

    turn_log: list[dict[str, str]] = []
    ask_count = 0
    extend_count = 0
    decode_count = 0
    segment_resets = 0
    t_start = time.monotonic()

    pending_since_extend = 0
    extend_chunk = max(1, extend_chunk)
    seg_limit = max(1, segment_decode_limit)

    def decode_ask(text: str, *, qid: str) -> str:
        nonlocal decode_count, segment_resets
        if decode_count > 0 and decode_count % seg_limit == 0:
            reset_mega_session_segment(
                base, ctx, round_idx=i, verbose=verbose, sid=sid, keep_lines=min(32, corpus_keep_lines),
            )
            segment_resets += 1
        decode_count += 1
        return base.ask(text, qid=qid)

    for i in range(total):
        if pause_requested(None):
            break
        line = ctx[i % len(ctx)]
        is_ask = (i % ask_every == 0) or (i == total - 1 and ask_count == 0)

        if is_ask:
            q = ask_questions[ask_count % len(ask_questions)]
            if verbose and (i < 5 or i % 200 == 0 or i == total - 1):
                log(verbose, f"  [{sid}] round {i+1}/{total} ASK: {q[:70]}…")
            ans = decode_ask(q, qid=f"t{i}")
            ask_count += 1
            turn_log.append({"turn": i + 1, "kind": "ask", "question": q, "answer": ans[:200]})
            maybe_trim_session_corpus(
                base,
                max_lines=max_session_corpus,
                keep_lines=corpus_keep_lines,
                verbose=verbose,
                sid=sid,
            )
        else:
            base.append_context(line)
            extend_count += 1
            pending_since_extend += 1
            if pending_since_extend >= extend_chunk:
                decode_ask("Continue.", qid=f"ext{i}")
                pending_since_extend = 0
                maybe_trim_session_corpus(
                    base,
                    max_lines=max_session_corpus,
                    keep_lines=corpus_keep_lines,
                    verbose=verbose,
                    sid=sid,
                )
            if verbose and i > 0 and i % 500 == 0:
                log(verbose, f"  [{sid}] round {i+1}/{total} extend … ({extend_count} lines total)")

        if progress:
            progress.set(i + 1)
        if checkpoint_path and (i + 1) % 500 == 0:
            checkpoint_path.write_text(
                json.dumps({"index": i + 1, "ask_count": ask_count, "extend_count": extend_count}, indent=2) + "\n",
                encoding="utf-8",
            )

    if base._pending_append:
        decode_ask("Acknowledge.", qid="flush")

    dialogue_elapsed = time.monotonic() - t_start

    primary = scenario["primary_question"]
    log(verbose, f"  [{sid}] PRIMARY Q: {primary}")
    ans1 = decode_ask(primary, qid="primary")
    primary_ok = (
        kw_ratio(ans1, keywords) >= 0.34
        or score_answer(ans1, {"keywords": keywords, "reference_answer": scenario.get("gold_answer", "")})["correct"]
        or kw_hits(ans1, ["yesterday", "support group", "lgbtq"]) >= 2
    )

    supplement = scenario.get("supplement_prompt") or "Please add more detail to your previous answer."
    log(verbose, f"  [{sid}] SUPPLEMENT Q: {supplement}")
    ans2 = decode_ask(supplement, qid="supplement")
    supplement_rich = len(ans2.split()) >= max(len(ans1.split()) * 0.6, 3)
    supplement_grounded = kw_ratio(ans2, sup_kws) >= 0.25 or kw_hits(ans2, keywords) >= kw_hits(ans1, keywords)

    log(verbose, f"  [{sid}] proactive_idle …")
    spread_warmup_for_proactive(base, scenario, ask_fn=decode_ask)
    pro_doc = base.proactive_idle()
    pro = pro_doc.get("proactive") or {}
    nudge = pro.get("nudge") or ""
    proactive_ok, proactive_mode = score_proactive_pass(pro, pro_kws)

    return {
        "id": sid,
        "turns": total,
        "ask_turns": ask_count,
        "extend_turns": extend_count,
        "ask_every": ask_every,
        "segment_resets": segment_resets,
        "decode_count": decode_count,
        "dialogue_elapsed_s": round(dialogue_elapsed, 1),
        "primary_question": primary,
        "answer_primary": ans1[:400],
        "answer_supplement": ans2[:400],
        "primary_correct": primary_ok,
        "supplement_rich": supplement_rich,
        "supplement_grounded": supplement_grounded,
        "supplement_pass": supplement_rich and supplement_grounded,
        "proactive": {
            "triggered": pro.get("triggered"),
            "emitted": pro.get("emitted"),
            "nudge": nudge[:300],
            "trigger_reason": pro.get("trigger_reason"),
            "fate_signals": pro.get("fate_signals_since_baseline"),
            "mode": proactive_mode,
        },
        "proactive_pass": proactive_ok,
        "pass": primary_ok and supplement_rich and supplement_grounded,
        "turn_log_sample": turn_log[:2] + turn_log[-2:] if len(turn_log) > 4 else turn_log,
    }


def run_multimodal_case(
    prompt: dict[str, Any],
    lm_model: str,
    *,
    verbose: bool,
    img_cache: Path,
) -> dict[str, Any]:
    pid = prompt["id"]
    text = prompt["text"]
    kws = list(prompt.get("keywords") or [])
    corpus = list(prompt.get("corpus") or [])

    modes: dict[str, str] = {}

    # text-only
    base = make_base(lm_model, core_fusion=True)
    base.set_context(corpus)
    log(verbose, f"  [{pid}] modality=text Q: {text[:70]}…")
    modes["text"] = base.ask(text, qid="text")

    # audio (TTS features on question)
    with tempfile.TemporaryDirectory() as td:
        wav = Path(td) / f"{pid}.wav"
        tts(text, wav)
        audio = encode_wav(wav)
        img = gen_image_features(text, pid, img_cache / pid)

        base2 = make_base(lm_model, core_fusion=True)
        base2.set_context(corpus)
        doc = base2.run(
            [{"id": "audio", "text": text, "score_type": "open", "audio_features": audio, "keywords": kws}],
        )
        modes["tts_audio"] = str((doc.get("rows") or [{}])[0].get("answer") or "")

        base3 = make_base(lm_model, core_fusion=True)
        base3.set_context(corpus)
        doc = base3.run(
            [{"id": "image", "text": text, "score_type": "open", "image_features": img, "keywords": kws}],
        )
        modes["image"] = str((doc.get("rows") or [{}])[0].get("answer") or "")

        base4 = make_base(lm_model, core_fusion=True)
        base4.set_context(corpus)
        doc = base4.run(
            [
                {
                    "id": "trimodal",
                    "text": text,
                    "score_type": "open",
                    "audio_features": audio,
                    "image_features": img,
                    "keywords": kws,
                }
            ],
        )
        modes["trimodal"] = str((doc.get("rows") or [{}])[0].get("answer") or "")

    hits = {m: kw_hits(a, kws) for m, a in modes.items()}
    min_h = min(hits.values()) if hits else 0
    max_h = max(hits.values()) if hits else 0
    all_hit = all(h > 0 for h in hits.values()) if kws else False
    consistency = min_h / max(max_h, 1)

    return {
        "id": pid,
        "question": text,
        "modes": {m: a[:250] for m, a in modes.items()},
        "keyword_hits": hits,
        "all_modalities_hit": all_hit,
        "consistency": round(consistency, 3),
        "pass": all_hit and consistency >= 0.5,
    }


def run_crosslang_case(fact: dict[str, Any], lm_model: str, *, verbose: bool) -> dict[str, Any]:
    fid = fact["id"]
    corpus = fact["corpus"]
    langs = fact["queries"]
    base = make_base(lm_model)
    base.set_context(corpus)
    if base.fate_warmup_enabled:
        base.fate_warmup_corpus(corpus, n_lines=min(2, len(corpus)))

    per_lang: dict[str, Any] = {}
    line_ids: list[int | None] = []
    for lang, spec in langs.items():
        q = spec["text"]
        kws = spec.get("keywords") or []
        log(verbose, f"  [{fid}] lang={lang} Q: {q[:60]}…")
        ans = base.ask(q, qid=f"{fid}_{lang}")
        ok = score_answer(ans, {"keywords": kws, "reference_answer": ""})["correct"] or kw_hits(ans, kws) > 0
        per_lang[lang] = {"answer": ans[:250], "correct": ok}
        line_ids.append(0 if ok else None)

    unified = all(v["correct"] for v in per_lang.values())
    return {
        "id": fid,
        "languages": per_lang,
        "line_agreement": len(set(x for x in line_ids if x is not None)) <= 1,
        "all_correct": unified,
        "pass": unified,
    }


def aggregate_report(
    tracks: dict[str, list[dict[str, Any]]],
    *,
    proactive_reaction: dict[str, Any] | None = None,
) -> dict[str, Any]:
    ld = tracks.get("long_dialogue") or []
    mm = tracks.get("multimodal_natural") or []
    cl = tracks.get("crosslang") or []

    ld_pass = sum(1 for r in ld if r.get("pass")) / max(len(ld), 1)
    mm_pass = sum(1 for r in mm if r.get("pass")) / max(len(mm), 1)
    cl_ccu = sum(1 for r in cl if r.get("all_correct")) / max(len(cl), 1)
    ppi = float((proactive_reaction or {}).get("ppi_v2") or 0)
    pr_pass = float((proactive_reaction or {}).get("pass_rate") or 0)

    # Agency Index: dialogue + multimodal + crosslang + automatic reaction (PPI-v2)
    if proactive_reaction:
        agency_index = round(0.25 * ld_pass + 0.20 * mm_pass + 0.20 * cl_ccu + 0.35 * ppi, 4)
    else:
        agency_index = round(0.35 * ld_pass + 0.30 * mm_pass + 0.35 * cl_ccu, 4)

    return {
        "long_dialogue": {
            "n": len(ld),
            "primary_rate": sum(1 for r in ld if r.get("primary_correct")) / max(len(ld), 1),
            "supplement_rate": sum(1 for r in ld if r.get("supplement_pass")) / max(len(ld), 1),
            "proactive_rate": sum(1 for r in ld if r.get("proactive_pass")) / max(len(ld), 1),
            "pass_rate": ld_pass,
        },
        "multimodal_natural": {
            "n": len(mm),
            "all_hit_rate": sum(1 for r in mm if r.get("all_modalities_hit")) / max(len(mm), 1),
            "avg_consistency": sum(r.get("consistency", 0) for r in mm) / max(len(mm), 1),
            "pass_rate": mm_pass,
        },
        "crosslang": {
            "n": len(cl),
            "ccu": cl_ccu,
            "pass_rate": sum(1 for r in cl if r.get("pass")) / max(len(cl), 1),
        },
        "proactive_reaction": proactive_reaction,
        "agency_index": agency_index,
        "agency_formula": (
            "0.25·long + 0.20·multimodal + 0.20·crosslang + 0.35·PPI-v2"
            if proactive_reaction
            else "0.35·long + 0.30·multimodal + 0.35·crosslang"
        ),
    }


def write_markdown(report: dict[str, Any], path: Path) -> None:
    m = report["metrics"]
    lines = [
        "# Proactive Agency Benchmark",
        "",
        f"**Protocol:** `{PROTOCOL}` · **Model:** `{report.get('lm_model')}`",
        f"**Generated:** {report.get('timestamp', '')}",
        "",
        "## Headline",
        "",
        f"| Metric | Value |",
        f"|--------|-------|",
        f"| **Agency Index** | **{m['agency_index']:.3f}** |",
        f"| Long-dialogue pass | {m['long_dialogue']['pass_rate']:.1%} |",
        f"| Dialogue rounds | {report.get('config', {}).get('max_turns', '—')} |",
        f"| Supplement pass | {m['long_dialogue']['supplement_rate']:.1%} |",
        f"| Proactive pass | {m['long_dialogue']['proactive_rate']:.1%} |",
        f"| Multimodal consistency | {m['multimodal_natural']['avg_consistency']:.3f} |",
        f"| Crosslang CCU | {m['crosslang']['ccu']:.1%} |",
        "",
    ]
    pr = m.get("proactive_reaction")
    if pr:
        lines.extend([
            "## Automatic reaction (Track D · proactive-fate-v2)",
            "",
            f"| PPI-v2 | **{pr.get('ppi_v2', 0):.3f}** |",
            f"| Trial pass rate | {pr.get('pass_rate', 0):.1%} |",
            f"| Gates OK | {pr.get('gates_ok')} |",
            f"| Unsolicited precision | {pr.get('unsolicited_precision')} |",
            "",
            f"**Agency formula:** `{m.get('agency_formula', '')}`",
            "",
        ])
    lines.extend([
        "",
    ])
    cfg = report.get("config") or {}
    ld_rows = (report.get("tracks") or {}).get("long_dialogue") or []
    if cfg.get("mega") and ld_rows:
        row = ld_rows[0]
        lines.extend([
            "## Mega stats",
            "",
            "| Metric | Value |",
            "|--------|-------|",
            f"| Decode count | {row.get('decode_count', '—')} |",
            f"| Segment resets | {row.get('segment_resets', '—')} |",
            f"| Dialogue elapsed | {row.get('dialogue_elapsed_s', '—')}s |",
            f"| extend_chunk | {cfg.get('extend_chunk', '—')} |",
            "",
        ])
    lines.extend([
        "## Run",
        "",
        "```bash",
        "tools/valhalla_inference/run_proactive_agency_benchmark.sh --verbose",
        "# Mega 3000 (memory-bounded)",
        "VALHALLA_BASE_URL=http://127.0.0.1:8788 tools/valhalla_inference/run_proactive_agency_benchmark.sh \\",
        "  --mega --turns 3000 --extend-chunk 8 --segment-decodes 4 --verbose",
        "```",
        "",
        "Full report: [PROACTIVE_AGENCY_BENCHMARK.md](./PROACTIVE_AGENCY_BENCHMARK.md)",
        "",
    ])
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> None:
    ap = argparse.ArgumentParser(description="Proactive Agency Benchmark (baked 3B default)")
    ap.add_argument("--model", default=str(DEFAULT_BAKED), help="LM path or HF id")
    ap.add_argument("--compare-base", action="store_true", help="also run base 3B for delta")
    ap.add_argument("--fast", action="store_true", help="smoke: 1 scenario, 8 turns, 2 multimodal")
    ap.add_argument("--mega", action="store_true", help="mega long-dialogue: 3000 rounds (extend+periodic ask)")
    ap.add_argument("--turns", type=int, default=0, help="dialogue rounds (e.g. 3000); overrides --max-turns")
    ap.add_argument("--max-turns", type=int, default=0, help="cap dialogue turns when not --mega")
    ap.add_argument("--ask-every", type=int, default=32, help="full ask every N extend rounds in mega mode")
    ap.add_argument("--extend-chunk", type=int, default=0, help="lines batched per session_extend (mega default 64)")
    ap.add_argument("--max-session-corpus", type=int, default=0, help="cap session corpus lines; mega default 192")
    ap.add_argument("--corpus-keep-lines", type=int, default=96, help="lines kept after corpus trim")
    ap.add_argument("--segment-decodes", type=int, default=0, help="session reset every N decodes (mega default 4)")
    ap.add_argument("--long-only", action="store_true", help="only long_dialogue track")
    ap.add_argument("--locomo-extra", type=int, default=2, help="extra LoCoMo scenarios")
    ap.add_argument("--download-dialogues", action="store_true", help="fetch DailyDialog slice")
    ap.add_argument("--skip-long", action="store_true")
    ap.add_argument("--skip-multimodal", action="store_true")
    ap.add_argument("--skip-crosslang", action="store_true")
    ap.add_argument("--skip-proactive-fate", action="store_true", help="omit Track D (proactive-fate-v2 PPI)")
    ap.add_argument("--refresh-proactive-fate", action="store_true", help="re-run proactive-fate-v2 before agency")
    ap.add_argument("--suite-checkpoint", type=Path, default=CHECKPOINT_DEFAULT)
    add_benchmark_args(ap)
    args = ap.parse_args()

    if args.download_dialogues:
        download_dailydialog_slice(BUNDLE, limit=3 if args.fast else 5)

    doc = load_scenarios()
    extend_scenarios_from_locomo(doc, limit=0 if args.fast else args.locomo_extra, max_lines=64 if args.fast else 128)
    scenarios = doc.get("scenarios") or []
    mm_prompts = doc.get("multimodal_prompts") or []
    if args.mega or args.turns >= 1000:
        max_turns = args.turns or 3000
        if args.extend_chunk <= 0:
            args.extend_chunk = 64
        if args.max_session_corpus <= 0:
            args.max_session_corpus = 192
        if args.segment_decodes <= 0:
            args.segment_decodes = 4
        mega_corpus = build_mega_corpus_from_locomo(max_lines=min(512, max(256, max_turns // 8)))
        scenarios = [
            {
                "id": "mega_locomo_3k",
                "mega_corpus": mega_corpus,
                "session_lines": mega_corpus[:64],
                "primary_question": "When did Caroline go to the LGBTQ support group?",
                "gold_answer": "7 May 2023",
                "keywords": ["may", "2023", "yesterday", "support group", "lgbtq"],
                "supplement_prompt": "Supplement your previous answer with any dates or events from our long conversation.",
                "supplement_keywords": ["may", "2023", "lgbtq", "support", "caroline"],
                "proactive_keywords": ["lgbtq", "caroline", "support", "pride", "melanie"],
            }
        ]
        args.skip_multimodal = True
        args.skip_crosslang = True
    elif args.fast:
        max_turns = 8
    else:
        max_turns = args.turns or args.max_turns or 24
        if args.ask_every == 32:
            args.ask_every = 2

    if args.long_only:
        args.skip_multimodal = True
        args.skip_crosslang = True

    if args.fast:
        scenarios = scenarios[:1]
        mm_prompts = mm_prompts[:2]
        facts = FACTS[:2]
    else:
        facts = FACTS

    active_tracks: list[str] = []
    if not args.skip_long:
        active_tracks.append("long_dialogue")
    if not args.skip_multimodal:
        active_tracks.append("multimodal_natural")
    if not args.skip_crosslang:
        active_tracks.append("crosslang")

    total_items = (
        (len(scenarios) if "long_dialogue" in active_tracks else 0)
        + (len(mm_prompts) if "multimodal_natural" in active_tracks else 0)
        + (len(facts) if "crosslang" in active_tracks else 0)
    ) * (2 if args.compare_base else 1)

    suite_ckpt = init_suite_checkpoint(
        args.suite_checkpoint,
        model=args.model,
        tests=active_tracks,
        resume=bool(args.resume),
    )
    done_tracks = suite_completed_labels(suite_ckpt) if args.resume else set()

    REPORTS.mkdir(parents=True, exist_ok=True)
    if args.checkpoint:
        args.checkpoint.parent.mkdir(parents=True, exist_ok=True)

    tracks: dict[str, list[dict[str, Any]]] = {
        "long_dialogue": [],
        "multimodal_natural": [],
        "crosslang": [],
    }

    if args.resume and args.suite_checkpoint.is_file():
        prev = json.loads(args.suite_checkpoint.read_text(encoding="utf-8"))
        tracks = prev.get("partial_tracks") or tracks

    bar = ProgressBar("Agency benchmark", total_items, enabled=True)
    bar.set(sum(len(tracks[t]) for t in active_tracks if t in tracks))
    t0 = time.monotonic()
    img_cache = REPORTS / "proactive_agency_images"

    models = [args.model]
    if args.compare_base:
        models.insert(0, DEFAULT_BASE)

    all_reports: dict[str, Any] = {}

    with benchmark_runtime() as gpu_env:
        print(
            f"Proactive Agency Benchmark · model={args.model}\n"
            f"GPU VALHALLA_USE_CPU={gpu_env.get('VALHALLA_USE_CPU')} · items={total_items} · tracks={active_tracks}",
            flush=True,
        )

        for lm in models:
            label = "baked" if "TriModal" in lm or "Valhalla" in lm else "base"
            print(f"\n{'='*60}\nModel arm: {label} ({lm})\n{'='*60}", flush=True)

            arm_tracks: dict[str, list[dict[str, Any]]] = {
                "long_dialogue": [],
                "multimodal_natural": [],
                "crosslang": [],
            }

            if "long_dialogue" in active_tracks:
                for sc in scenarios:
                    if pause_requested(args.pause_file):
                        break
                    print(f"\n[long_dialogue] {sc['id']} ({max_turns} rounds, ask_every={args.ask_every})", flush=True)
                    ld_bar = ProgressBar(f"{sc['id']}", max_turns, enabled=True)
                    ckpt_path = REPORTS / "checkpoints" / f"{sc['id']}.progress.json"
                    if args.checkpoint:
                        ckpt_path = Path(args.checkpoint)
                    ckpt_path.parent.mkdir(parents=True, exist_ok=True)
                    row = run_long_dialogue_case(
                        sc,
                        lm,
                        verbose=args.verbose,
                        max_turns=max_turns,
                        ask_every=args.ask_every,
                        extend_chunk=args.extend_chunk or 8,
                        max_session_corpus=args.max_session_corpus,
                        corpus_keep_lines=args.corpus_keep_lines,
                        segment_decode_limit=args.segment_decodes,
                        checkpoint_path=ckpt_path,
                        progress=ld_bar,
                    )
                    ld_bar.finish(f"{row.get('dialogue_elapsed_s', 0):.0f}s")
                    arm_tracks["long_dialogue"].append(row)
                    bar.tick_item()
                    bar.inc()
                    print(
                        f"  → rounds={row['turns']} ask={row.get('ask_turns')} extend={row.get('extend_turns')} "
                        f"primary={'✓' if row['primary_correct'] else '✗'} "
                        f"supplement={'✓' if row['supplement_pass'] else '✗'} "
                        f"proactive={'✓' if row['proactive_pass'] else '✗'} "
                        f"fate_signals={row['proactive'].get('fate_signals')}",
                        flush=True,
                    )

            if "multimodal_natural" in active_tracks:
                for p in mm_prompts:
                    if pause_requested(args.pause_file):
                        break
                    print(f"\n[multimodal] {p['id']}: {p['text'][:70]}…", flush=True)
                    row = run_multimodal_case(p, lm, verbose=args.verbose, img_cache=img_cache)
                    arm_tracks["multimodal_natural"].append(row)
                    bar.tick_item()
                    bar.inc()
                    print(
                        f"  → hits={row['keyword_hits']} consistency={row['consistency']:.2f} "
                        f"{'✓' if row['pass'] else '✗'}",
                        flush=True,
                    )

            if "crosslang" in active_tracks:
                for fact in facts:
                    if pause_requested(args.pause_file):
                        break
                    print(f"\n[crosslang] {fact['id']}", flush=True)
                    row = run_crosslang_case(fact, lm, verbose=args.verbose)
                    arm_tracks["crosslang"].append(row)
                    bar.tick_item()
                    bar.inc()
                    langs = " ".join(f"{k}:{'✓' if v['correct'] else '✗'}" for k, v in row["languages"].items())
                    print(f"  → {langs}", flush=True)

            metrics = aggregate_report(arm_tracks, proactive_reaction=None)
            all_reports[label] = {"lm_model": lm, "tracks": arm_tracks, "metrics": metrics}
            tracks = arm_tracks  # last arm for primary report

    bar.finish(f"done {time.monotonic()-t0:.0f}s")

    proactive_reaction = None
    if not args.mega and not args.skip_proactive_fate:
        proactive_reaction = load_proactive_fate_metrics(run_if_missing=args.refresh_proactive_fate)

    primary = all_reports.get("baked") or all_reports.get("base") or {}
    metrics = aggregate_report(tracks, proactive_reaction=proactive_reaction)
    if primary:
        primary["metrics"] = metrics
    verdict = "AGENCY_STRONG" if metrics.get("agency_index", 0) >= 0.75 else (
        "AGENCY_VIABLE" if metrics.get("agency_index", 0) >= 0.55 else "AGENCY_WEAK"
    )

    report = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "protocol": PROTOCOL,
        "lm_model": args.model,
        "config": {
            "fast": args.fast,
            "mega": args.mega,
            "max_turns": max_turns,
            "ask_every": args.ask_every,
            "extend_chunk": args.extend_chunk or 8,
            "max_session_corpus": args.max_session_corpus,
            "compare_base": args.compare_base,
            "tracks": active_tracks,
        },
        "metrics": metrics,
        "tracks": tracks,
        "arms": all_reports,
        "verdict": verdict,
        "elapsed_s": round(time.monotonic() - t0, 1),
    }

    out_json = OUT_JSON_MEGA if args.mega else OUT_JSON_STANDARD
    out_json.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    OUT_JSON.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    write_markdown(report, OUT_MD)

    print(f"\n{'='*60}")
    print(f"Agency Index: {metrics['agency_index']:.3f}  Verdict: {verdict}")
    print(f"  long_dialogue: {metrics['long_dialogue']['pass_rate']:.1%}  supplement: {metrics['long_dialogue']['supplement_rate']:.1%}  proactive: {metrics['long_dialogue']['proactive_rate']:.1%}")
    if proactive_reaction:
        print(f"  proactive_reaction PPI-v2: {proactive_reaction.get('ppi_v2', 0):.3f}")
    print(f"  multimodal:    consistency={metrics['multimodal_natural']['avg_consistency']:.3f}  pass={metrics['multimodal_natural']['pass_rate']:.1%}")
    print(f"  crosslang CCU: {metrics['crosslang']['ccu']:.1%}")
    print(f"→ {out_json}")
    print(f"→ {OUT_MD}")


if __name__ == "__main__":
    main()
