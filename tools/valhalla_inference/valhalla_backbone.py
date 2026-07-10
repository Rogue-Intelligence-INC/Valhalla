#!/usr/bin/env python3
"""Valhalla × backbone — single entry for experiments.

  # ad-hoc HF model (no registry edit)
  python3 tools/valhalla_inference/valhalla_backbone.py --hf microsoft/Phi-tiny-MoE-instruct --fast

  # preset from backbone_registry.PRESETS
  python3 tools/valhalla_inference/valhalla_backbone.py --preset rwkv7_0.1b --fast

  # paper 6-class panel
  python3 tools/valhalla_inference/valhalla_backbone.py --paper --fast

  # probe load only
  python3 tools/valhalla_inference/valhalla_backbone.py --hf Qwen/Qwen2.5-3B-Instruct --probe

Python API:
  from valhalla_backbone import experiment, probe, Backbone
  experiment(Backbone.from_hf("org/model"), fast=True)
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Callable

REPO = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(REPO / "tools/fair_benchmark"))
sys.path.insert(0, str(REPO / "tools/standard_benchmarks"))

from backbone_registry import Backbone, CATEGORY_ORDER, list_presets, paper_panel, resolve  # noqa: E402
from benchmark_env import apply_gpu_env, production_valhalla_base_kwargs  # noqa: E402
from encoder_context import EncoderContextRouter  # noqa: E402
from metrics import score_answer  # noqa: E402
from run_fair_benchmark import load_suite, select_prompts  # noqa: E402
from run_potential_analysis import classify_slices  # noqa: E402
from run_scale_benchmark import targeted_context  # noqa: E402
from test_context_arch_compare import resolve_lm_path, trad_generate  # noqa: E402
from valhalla_base import ValhallaBase  # noqa: E402

PROTOCOL = "valhalla-backbone-v1"
REPORTS = REPO / "reports/valhalla_inference"


# ---------------------------------------------------------------------------
# ValhallaBase helpers
# ---------------------------------------------------------------------------

def _native_base() -> ValhallaBase:
    return ValhallaBase(
        decode="native",
        follow_up_decode="native_follow_up_aware",
        fate_ingress_routing="quad_cycle",
        fate_qa_feedback=False,
        incubation_cycles=1,
        max_new=128,
    )


def _hybrid_base(lm_model: str) -> ValhallaBase:
    return ValhallaBase(**production_valhalla_base_kwargs(lm_model=lm_model))


def _valhalla_batch(
    factory: Callable[[], ValhallaBase],
    corpus: list[str],
    prompts: list[dict],
    ctx_fn: Callable[[dict, list[str]], list[str]],
) -> dict[str, Any]:
    rows, ok = [], 0
    for p in prompts:
        base = factory()
        base.set_context(ctx_fn(p, corpus))
        out = base.run([{
            "id": p["id"], "text": p["text"], "score_type": "open",
            "reference_answer": p.get("reference_answer", ""),
            "keywords": p.get("keywords") or [],
        }])
        row = (out.get("rows") or [{}])[0]
        ans = str(row.get("answer") or "")
        hit = bool(score_answer(ans, p)["correct"])
        ok += int(hit)
        rows.append({"id": p["id"], "correct": hit, "answer": ans[:160],
                     "decode_path": row.get("decode_path", "")})
    n = len(prompts)
    return {"n": n, "correct": ok, "accuracy": ok / n if n else 0.0, "rows": rows}


def _trad_lm(
    bb: Backbone,
    corpus: list[str],
    prompts: list[dict],
    ctx_fn: Callable[[dict, list[str]], list[str]] | None = None,
) -> dict[str, Any]:
    from loader import ValhallaLLM

    ctx = ctx_fn or (lambda p, c: targeted_context(p, c))
    path = resolve_lm_path(bb.hf_id)
    lm = ValhallaLLM(path, load_in_4bit=bb.load_in_4bit, trust_remote_code=bb.trust_remote_code)
    try:
        lm.load()
        rows, ok = [], 0
        for p in prompts:
            ans = trad_generate(lm, p["text"], ctx(p, corpus))
            hit = bool(score_answer(ans, p)["correct"])
            ok += int(hit)
            rows.append({"id": p["id"], "correct": hit, "answer": ans[:160]})
        n = len(prompts)
        return {"n": n, "correct": ok, "accuracy": ok / n if n else 0.0, "model_path": str(path), "rows": rows}
    finally:
        lm.unload()


def _encoder_ctx(bb: Backbone) -> Callable[[dict, list[str]], list[str]]:
    router = EncoderContextRouter(bb.hf_id)
    return lambda p, c: router.select(p, c, k=8)


def _vision_smoke(bb: Backbone) -> dict[str, Any]:
    import torch
    from transformers import AutoModelForVision2Seq, AutoProcessor

    path = resolve_lm_path(bb.hf_id)
    proc = AutoProcessor.from_pretrained(str(path), trust_remote_code=bb.trust_remote_code)
    kw: dict[str, Any] = {"trust_remote_code": bb.trust_remote_code,
                          "torch_dtype": torch.float16 if torch.cuda.is_available() else torch.float32}
    if torch.cuda.is_available():
        kw["device_map"] = "auto"
    if "qwen2.5-vl" in bb.hf_id.lower() or "qwen2_5_vl" in bb.hf_id.lower():
        from transformers import Qwen2_5_VLForConditionalGeneration
        model = Qwen2_5_VLForConditionalGeneration.from_pretrained(str(path), **kw)
    else:
        model = AutoModelForVision2Seq.from_pretrained(str(path), **kw)
    model.eval()
    msg = [{"role": "user", "content": [{"type": "text", "text": "Hi"}]}]
    text = proc.apply_chat_template(msg, tokenize=False, add_generation_prompt=True)
    inp = proc(text=[text], return_tensors="pt", padding=True)
    dev = next(model.parameters()).device
    inp = {k: v.to(dev) if hasattr(v, "to") else v for k, v in inp.items()}
    with torch.no_grad():
        out = model.generate(**inp, max_new_tokens=8)
    sample = proc.batch_decode(out, skip_special_tokens=True)[0][-60:]
    return {"load_ok": True, "generate_sample": sample, "model_path": str(path)}


# ---------------------------------------------------------------------------
# Public API
# ---------------------------------------------------------------------------

def load_tasks(*, fast: bool = True, limit: int = 0) -> tuple[list[str], list[dict]]:
    suite = load_suite()
    corpus = suite["corpus"]["ingest_clean"]
    prompts = [p for p in select_prompts(suite, "test") if p.get("score_type") == "open"]
    ids = classify_slices(prompts, corpus)["open_retrieval"]
    tasks = [p for p in prompts if p["id"] in ids]
    if fast:
        tasks = tasks[:4]
    elif limit > 0:
        tasks = tasks[:limit]
    return corpus, tasks


def weights_ok(bb: Backbone) -> tuple[bool, str]:
    try:
        return True, str(resolve_lm_path(bb.hf_id))
    except FileNotFoundError as e:
        return False, str(e)


def probe(bb: Backbone) -> dict[str, Any]:
    ok, path_or_err = weights_ok(bb)
    out: dict[str, Any] = {"id": bb.id, "hf_id": bb.hf_id, "kind": bb.kind,
                           "category": bb.category, "weights_ok": ok}
    if not ok:
        out["error"] = path_or_err
        return out
    out["model_path"] = path_or_err
    if bb.kind == "causal_lm":
        from loader import ValhallaLLM
        lm = ValhallaLLM(path_or_err, load_in_4bit=bb.load_in_4bit, trust_remote_code=bb.trust_remote_code)
        out.update(lm.probe())
        if out.get("load_ok"):
            out["generate_sample"] = lm.generate(["2+2?"], max_tokens=12)[0].text[:80]
    elif bb.kind == "encoder":
        EncoderContextRouter(bb.hf_id)
        out["load_ok"] = True
    else:
        out.update(_vision_smoke(bb))
    return out


def experiment(
    bb: Backbone,
    corpus: list[str],
    prompts: list[dict],
    *,
    skip_missing: bool = True,
    arms: tuple[str, ...] | None = None,
    ctx_fn: Callable[[dict, list[str]], list[str]] | None = None,
) -> dict[str, Any]:
    """Run ValhallaBase arms for one backbone."""
    result: dict[str, Any] = {
        "id": bb.id, "hf_id": bb.hf_id, "kind": bb.kind, "category": bb.category, "arms": {},
    }
    ok, path = weights_ok(bb)
    result["model_path"] = path if ok else None
    if not ok:
        if skip_missing:
            result["skipped"] = True
            result["error"] = path
            return result
        raise FileNotFoundError(path)

    ctx = ctx_fn or (lambda p, c: targeted_context(p, c))
    for arm in arms or bb.valhalla_arms():
        try:
            if arm == "native":
                result["arms"]["native"] = _valhalla_batch(_native_base, corpus, prompts, ctx)
            elif arm == "hybrid":
                result["arms"]["hybrid"] = _valhalla_batch(
                    lambda: _hybrid_base(bb.hf_id), corpus, prompts, ctx)
            elif arm == "trad":
                result["arms"]["trad"] = _trad_lm(bb, corpus, prompts, ctx_fn=ctx)
            elif arm == "encoder":
                result["arms"]["encoder"] = _valhalla_batch(
                    _native_base, corpus, prompts, _encoder_ctx(bb))
            elif arm == "vision":
                result["arms"]["vision"] = _vision_smoke(bb)
        except Exception as e:
            result["arms"][arm] = {"error": str(e)}
    return result


def experiment_many(
    backbones: list[Backbone],
    *,
    fast: bool = True,
    limit: int = 0,
    skip_missing: bool = True,
) -> dict[str, Any]:
    corpus, prompts = load_tasks(fast=fast, limit=limit)
    specs = {}
    for bb in backbones:
        specs[bb.id] = experiment(bb, corpus, prompts, skip_missing=skip_missing)
    return {"protocol": PROTOCOL, "n_questions": len(prompts), "specs": specs}


def _fmt_arm(data: dict) -> str:
    if "error" in data:
        return "ERR"
    if "accuracy" in data:
        return f"{data['correct']}/{data['n']}"
    if data.get("load_ok") or data.get("generate_sample"):
        return "OK"
    return "?"


def _summarize(row: dict) -> str:
    if row.get("skipped"):
        return f"SKIP ({str(row.get('error', ''))[:50]})"
    return "  ".join(f"{k}={_fmt_arm(v)}" for k, v in (row.get("arms") or {}).items())


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def main() -> int:
    ap = argparse.ArgumentParser(description="Valhalla backbone experiment (single entry)")
    ap.add_argument("--hf", action="append", help="HF model id (ad-hoc, repeatable)")
    ap.add_argument("--preset", action="append", help="PRESETS key (repeatable)")
    ap.add_argument("--category", choices=CATEGORY_ORDER, help="All presets in category")
    ap.add_argument("--paper", action="store_true", help="Paper panel (6 classes)")
    ap.add_argument("--list", action="store_true", help="List PRESETS and exit")
    ap.add_argument("--probe", action="store_true", help="Load probe only")
    ap.add_argument("--fast", action="store_true")
    ap.add_argument("--limit", type=int, default=0)
    args = ap.parse_args()

    apply_gpu_env()
    os.environ.setdefault("VALHALLA_INGRESS_RESOURCE_PERCENT", "70")
    os.environ.setdefault("HF_ENDPOINT", os.environ.get("HF_ENDPOINT", "https://hf-mirror.com"))

    if args.list:
        for cat in CATEGORY_ORDER:
            items = list_presets(cat)  # type: ignore[arg-type]
            if items:
                print(f"[{cat}]")
                for b in items:
                    tag = " *" if b.paper else ""
                    print(f"  {b.id:22} {b.hf_id}{tag}")
        return 0

    backbones = resolve(
        hf=args.hf,
        preset=args.preset,
        category=args.category,  # type: ignore[arg-type]
        paper=args.paper or (not args.hf and not args.preset and not args.category),
    )
    if not backbones:
        print("No backbones selected. Use --hf, --preset, --category, or --paper.", file=sys.stderr)
        return 1

    print(f"=== {PROTOCOL} n={len(backbones)} {[b.id for b in backbones]} ===\n")

    if args.probe:
        report = {"protocol": PROTOCOL, "timestamp": datetime.now(timezone.utc).isoformat(), "probes": {}}
        ok = 0
        for bb in backbones:
            print(f"Probe {bb.id} ({bb.hf_id}) …")
            row = probe(bb)
            report["probes"][bb.id] = row
            passed = bool(row.get("load_ok") or row.get("weights_ok"))
            ok += int(passed)
            print(f"  {'PASS' if passed else 'FAIL'}: {row.get('generate_sample') or row.get('error') or row.get('model_path')}\n")
        report["summary"] = {"passed": ok, "n": len(backbones)}
        out = REPORTS / "valhalla_backbone_probe.json"
        out.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n")
        print(f"→ {out}")
        return 0 if ok == len(backbones) else 1

    corpus, prompts = load_tasks(fast=args.fast, limit=args.limit)
    report: dict[str, Any] = {
        "protocol": PROTOCOL,
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "n_questions": len(prompts),
        "specs": {},
    }
    for bb in backbones:
        print(f"--- [{bb.category}] {bb.id} ---")
        row = experiment(bb, corpus, prompts)
        report["specs"][bb.id] = row
        print(f"  {_summarize(row)}\n")

    out = REPORTS / "valhalla_backbone_test.json"
    out.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n")
    print(f"→ {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
