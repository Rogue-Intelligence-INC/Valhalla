#!/usr/bin/env python3
"""Unified long-chain + long-term memory benchmark (theme drift after interleave).

Phases on ONE session (no reset):
  0) fate pretrain     — configurable items × epochs (reversible epoch log)
  A) anchor baseline   — D1 theme anchors
  B) interleave        — D2/D3 long-chain questions
  C) anchor reprobe    — same anchors → theme/tag drift
  D) holdout           — fresh probes never asked in A/B

  python3 tools/valhalla_inference/test_fate_longchain_unified.py --low-resource
  python3 tools/valhalla_inference/test_fate_longchain_unified.py --low-resource --enhanced
"""
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
REPORTS = REPO / "reports" / "valhalla_inference"
BIN = REPO / "target" / "release" / "valhalla_base"
SPEC = REPO / "tools/standard_benchmarks/bundled/fate_longchain_unified_locomo.json"

sys.path.insert(0, str(REPO / "tools" / "valhalla_inference"))
from benchmark_env import apply_gpu_env, apply_resource_budget, release_resources  # noqa: E402

LOCK = REPO / "reports" / "valhalla_inference" / ".fate_longchain_unified.lock"


def _acquire_lock() -> int:
    """Exclusive flock — refuse concurrent longchain runs (VRAM / RAM contention)."""
    import fcntl

    LOCK.parent.mkdir(parents=True, exist_ok=True)
    fd = os.open(str(LOCK), os.O_CREAT | os.O_RDWR, 0o644)
    try:
        fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
    except BlockingIOError:
        os.close(fd)
        raise SystemExit(
            f"another fate-longchain-unified run holds {LOCK}; refuse concurrent execution"
        )
    os.write(fd, f"{os.getpid()}\n".encode())
    return fd


def _release_lock(fd: int) -> None:
    import fcntl

    try:
        fcntl.flock(fd, fcntl.LOCK_UN)
    finally:
        os.close(fd)
        try:
            LOCK.unlink(missing_ok=True)
        except OSError:
            pass


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--low-resource", action="store_true")
    ap.add_argument("--parallel", action="store_true", help="use parallel pipes (all phases)")
    ap.add_argument(
        "--hybrid",
        action="store_true",
        help="hybrid routing: targeted anchor/reprobe, parallel interleave, fresh holdout",
    )
    ap.add_argument("--max-lines", type=int, default=0)
    ap.add_argument("--chunk-size", type=int, default=64)
    ap.add_argument("--pretrain-passes", type=int, default=None)
    ap.add_argument("--max-epochs", type=int, default=None)
    ap.add_argument(
        "--enhanced",
        action="store_true",
        help="stronger pretrain: 8 items × 8 epochs, Fate adaptive multi-pass warmup",
    )
    ap.add_argument("--verbose", action="store_true")
    ap.add_argument(
        "--smoke",
        action="store_true",
        help="light smoke: 64 corpus lines, chunk 32 (ingress cap 48)",
    )
    ap.add_argument(
        "--fragment-full",
        action="store_true",
        help="ingest full session; query fragment merge at QA (default when dynamic fragments on)",
    )
    ap.add_argument(
        "--minimal-corpus",
        action="store_true",
        help="legacy qa_adaptive minimal line subset (overrides fragment-full)",
    )
    ap.add_argument(
        "--production",
        action="store_true",
        help="production: hybrid + enhanced + fragment-full + max-lines 64 + epochs 4",
    )
    args = ap.parse_args()

    if args.production:
        args.low_resource = True
        args.enhanced = True
        args.hybrid = True
        # Default substrate = fragment_full; allow --minimal-corpus as ablation.
        if args.minimal_corpus:
            args.fragment_full = False
            print(
                "[info] --production --minimal-corpus: qa_adaptive_minimal ablation",
                flush=True,
            )
        else:
            args.fragment_full = True
            args.minimal_corpus = False
        if args.max_lines == 0:
            args.max_lines = 64 if args.fragment_full else 42
        if args.chunk_size == 64:
            args.chunk_size = 32
        if args.max_epochs is None:
            args.max_epochs = 4
    elif args.smoke:
        args.low_resource = True
        if args.max_lines == 0:
            args.max_lines = 32
        if args.chunk_size == 64:
            args.chunk_size = 32
    elif args.low_resource and args.max_lines == 0:
        # Bug fix: max-lines=0 loads full LoCoMo corpus → unbounded RAM
        args.max_lines = 64
        if args.chunk_size == 64:
            args.chunk_size = 32

    if args.enhanced:
        pretrain_passes = args.pretrain_passes if args.pretrain_passes is not None else 0
        max_epochs = args.max_epochs if args.max_epochs is not None else 8
    else:
        pretrain_passes = args.pretrain_passes if args.pretrain_passes is not None else 0
        max_epochs = args.max_epochs if args.max_epochs is not None else 0

    if not BIN.is_file():
        subprocess.check_call(
            [
                "cargo", "build", "-p", "hub-f64", "--release", "--bin", "valhalla_base",
                "--features", "patch-lm",
            ],
            cwd=REPO,
            env={**os.environ, "RUSTFLAGS": "-L /opt/cuda/lib64"},
        )

    cmd = [
        str(BIN), "demo", "fate-longchain-unified",
        "--max-lines", str(args.max_lines),
        "--chunk-size", str(args.chunk_size),
        "--pretrain-passes", str(pretrain_passes),
        "--max-epochs", str(max_epochs),
        "--spec", str(SPEC.relative_to(REPO)),
    ]
    if args.low_resource:
        cmd.append("--low-resource")
        cmd.append("--fast")
    if args.hybrid:
        cmd.append("--hybrid")
    elif args.parallel:
        cmd.append("--parallel")
    if args.minimal_corpus:
        cmd.append("--minimal-corpus")
    elif args.fragment_full:
        cmd.append("--fragment-full")

    print(" ".join(cmd), flush=True)
    # Kill any stale valhalla_base demos before starting (single occupant).
    subprocess.run(
        ["pkill", "-f", "valhalla_base demo fate-longchain-unified"],
        capture_output=True,
        check=False,
    )
    lock_fd = _acquire_lock()
    apply_gpu_env()
    env = apply_resource_budget()
    env.setdefault("VALHALLA_INGRESS_RESOURCE_PERCENT", "50" if args.production else "70")
    if args.production or args.fragment_full:
        # Dynamic-fragment primary: keep density segs; do not fate_link mega-merge.
        env.setdefault("VALHALLA_FATE_LINK_FRAGMENTS", "0")
        env.setdefault("VALHALLA_FRAGMENT_CHUNK_STRIDE", "8")
        env.setdefault("VALHALLA_FRAGMENT_MAX_LINES", "24")
        # Clear prior ablation overrides from this shell if any.
        env.pop("VALHALLA_FRAGMENT_MIN_LINES", None)
        env.pop("VALHALLA_FRAGMENT_DENSITY_DROP", None)
        env["VALHALLA_FRAGMENT_FLOW_DEBUG"] = env.get(
            "VALHALLA_FRAGMENT_FLOW_DEBUG", "1"
        )
    if args.low_resource or args.smoke or args.production:
        env.setdefault("VALHALLA_INGRESS_MAX_ITEMS", "48")
        env.setdefault("VALHALLA_FUSION_ROUND_CAP", "1")
        env.setdefault("VALHALLA_ROLLBACK_DEPTH_CAP", "2")
        env.setdefault("VALHALLA_LM_CACHE_MAX", "1")
        env.setdefault("VALHALLA_HYBRID_OPEN_LM_FALLBACK", "0")
    stream = args.production or args.verbose
    try:
        proc = subprocess.run(
            cmd,
            cwd=REPO,
            env=env,
            timeout=900,
            **({} if stream else {"capture_output": True, "text": True}),
        )
    except subprocess.TimeoutExpired:
        print("fate-longchain-unified timed out (900s)", file=sys.stderr)
        release_resources()
        _release_lock(lock_fd)
        return 1
    except Exception:
        _release_lock(lock_fd)
        raise
    if not stream and (args.verbose or args.low_resource or args.enhanced):
        if proc.stdout:
            print(proc.stdout)
        if proc.stderr:
            print(proc.stderr, file=sys.stderr)
    if proc.returncode != 0:
        if not stream:
            print(proc.stderr or proc.stdout, file=sys.stderr)
        release_resources()
        _release_lock(lock_fd)
        return proc.returncode

    out = REPORTS / "fate_longchain_unified.json"
    doc = json.loads(out.read_text(encoding="utf-8"))
    print(
        f"protocol {doc['protocol']}  routing={doc['routing']}  corpus_build={doc.get('corpus_build', '?')}  corpus={doc['corpus_lines']} lines\n"
        f"  pretrain {doc.get('pretrain_pass', 0):.1%} "
        f"({doc.get('pretrain_epochs', 0)}/{doc.get('pretrain_max_epochs', 0)} epochs, "
        f"passes={doc.get('pretrain_passes', 0)}, conv={doc.get('pretrain_converged')})\n"
        f"  anchor baseline {doc['anchor_baseline_pass']:.1%}  reprobe {doc['anchor_reprobe_pass']:.1%}\n"
        f"  theme_drift {doc['theme_drift_rate']:.1%}  retention {doc['anchor_retention_rate']:.1%}\n"
        f"  pollution {doc['context_pollution_rate']:.1%}  interleave {doc['interleave_pass']:.1%}\n"
        f"  holdout {doc['holdout_pass']:.1%}  composite {doc['composite_score']:.3f}"
    )
    for ep in doc.get("pretrain_epoch_log") or []:
        print(f"  pretrain epoch {ep['epoch']}: {ep['train_correct']}/{ep['train_n']}")
    print("--- anchor drift ---")
    for row in doc.get("anchor_drift_rows") or []:
        drift = "DRIFT" if row.get("theme_drift") else "ok"
        ret = "retained" if row.get("retained") else "lost"
        print(
            f"  {row.get('id')}  {drift}  {ret}  "
            f"base={row.get('baseline_tags')} → reprobe={row.get('reprobe_tags')}"
        )
    print(f"→ {out}")
    release_resources()
    _release_lock(lock_fd)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
