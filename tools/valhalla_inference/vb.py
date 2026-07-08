#!/usr/bin/env python3
"""Valhalla Base unified CLI — programmatic reproduction entry point.

  vb.py health
  vb.py build
  vb.py demo v1
  vb.py serve [--with-dashboard]
  vb.py dashboard
  vb.py suite [fate-core|rust-unit|standard-fast|context-drift|--all]
  vb.py repro
"""
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import urllib.error
import urllib.request
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
BIN = REPO / "target" / "release" / "valhalla_base"
DASH = REPO / "target" / "release" / "valhalla_dashboard"
RUSTFLAGS = os.environ.get("RUSTFLAGS", "-L /opt/cuda/lib64")
BASE_URL = os.environ.get("VALHALLA_BASE_URL", "http://127.0.0.1:8787")


def env() -> dict[str, str]:
    return {**os.environ, "RUSTFLAGS": RUSTFLAGS}


def ensure_bin() -> None:
    if BIN.is_file():
        return
    subprocess.check_call(
        [
            "cargo", "build", "-p", "hub-f64", "--release",
            "--bin", "valhalla_base", "--features", "patch-lm",
        ],
        cwd=REPO,
        env=env(),
    )


def ensure_dashboard() -> None:
    if DASH.is_file():
        return
    subprocess.check_call(
        [
            "cargo", "build", "-p", "hub-f64", "--release",
            "--bin", "valhalla_dashboard", "--features", "patch-lm",
        ],
        cwd=REPO,
        env=env(),
    )


def curl_json(path: str) -> dict:
    url = f"{BASE_URL.rstrip('/')}{path}"
    try:
        with urllib.request.urlopen(url, timeout=5) as resp:
            return json.loads(resp.read().decode())
    except urllib.error.URLError as e:
        print(f"HTTP failed ({url}): {e}", file=sys.stderr)
        sys.exit(1)


def cmd_health(_: argparse.Namespace) -> int:
    doc = curl_json("/health")
    print(json.dumps(doc, indent=2))
    return 0 if doc.get("ok") else 1


def cmd_repro(_: argparse.Namespace) -> int:
    return subprocess.call(["make", "repro"], cwd=REPO, env=env())


def cmd_build(_: argparse.Namespace) -> int:
    ensure_bin()
    ensure_dashboard()
    print(BIN)
    return 0


def cmd_demo(args: argparse.Namespace) -> int:
    ensure_bin()
    return subprocess.call([str(BIN), "demo", *args.demo_args], cwd=REPO, env=env())


def cmd_serve(args: argparse.Namespace) -> int:
    ensure_bin()
    cmd = [str(BIN), "serve", "--port", str(args.port)]
    if args.with_dashboard:
        cmd.append("--with-dashboard")
    return subprocess.call(cmd, cwd=REPO, env=env())


def cmd_dashboard(args: argparse.Namespace) -> int:
    ensure_dashboard()
    e = env()
    e["VALHALLA_DASHBOARD_PORT"] = str(args.port)
    return subprocess.call([str(DASH)], cwd=REPO, env=e)


def cmd_suite(args: argparse.Namespace) -> int:
    ensure_bin()
    runner = REPO / "tools" / "valhalla_inference" / "run_suite_sequential.py"
    cmd = [sys.executable, str(runner)]
    if args.all_suites:
        cmd.append("--all")
    elif args.suite:
        for s in args.suite:
            cmd.extend(["--suite", s])
    else:
        cmd.extend(["--suite", "fate-core"])
    return subprocess.call(cmd, cwd=REPO, env=env())


def main() -> int:
    ap = argparse.ArgumentParser(prog="vb", description="Valhalla Base reproduction CLI")
    sub = ap.add_subparsers(dest="cmd", required=True)

    sub.add_parser("health", help="GET /health")
    sub.add_parser("build", help="cargo build valhalla_base + dashboard")
    sub.add_parser("repro", help="make repro (full smoke path)")

    p_demo = sub.add_parser("demo", help="valhalla_base demo …")
    p_demo.add_argument("demo_args", nargs=argparse.REMAINDER)

    p_serve = sub.add_parser("serve", help="HTTP infer API")
    p_serve.add_argument("--port", type=int, default=8787)
    p_serve.add_argument("--with-dashboard", action="store_true")

    p_dash = sub.add_parser("dashboard", help="standalone triad dashboard")
    p_dash.add_argument("--port", type=int, default=8765)

    p_suite = sub.add_parser("suite", help="run_suite_sequential.py")
    p_suite.add_argument("--all", dest="all_suites", action="store_true")
    p_suite.add_argument("--suite", action="append")

    args = ap.parse_args()
    handlers = {
        "health": cmd_health,
        "build": cmd_build,
        "repro": cmd_repro,
        "demo": cmd_demo,
        "serve": cmd_serve,
        "dashboard": cmd_dashboard,
        "suite": cmd_suite,
    }
    return handlers[args.cmd](args)


if __name__ == "__main__":
    raise SystemExit(main())
