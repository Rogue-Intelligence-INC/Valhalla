#!/usr/bin/env python3
"""Shared benchmark CLI helpers — progress bar, checkpoint, pause, resume."""

from __future__ import annotations

import argparse
import json
import os
import sys
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Iterator

SUITE_CHECKPOINT_PROTOCOL = "industry-suite-checkpoint-v1"


class ProgressBar:
    """ASCII progress bar for suite-level and item-level tracking."""

    def __init__(self, label: str, total: int, *, width: int = 32, enabled: bool = True) -> None:
        self.label = label
        self.total = max(total, 1)
        self.current = 0
        self.width = width
        self.enabled = enabled and total > 0
        self._start = time.monotonic()
        self._item_times: list[float] = []

    def set(self, current: int) -> None:
        self.current = min(current, self.total)
        self._render()

    def inc(self, n: int = 1) -> None:
        self.set(self.current + n)

    def tick_item(self) -> None:
        """Call after each benchmark item completes (for ETA)."""
        self._item_times.append(time.monotonic() - self._start)
        if len(self._item_times) > 20:
            self._item_times = self._item_times[-20:]

    def eta_seconds(self) -> float | None:
        if len(self._item_times) < 2 or self.current <= 0:
            return None
        # average time per completed item
        per_item = self._item_times[-1] / max(self.current, 1)
        remaining = self.total - self.current
        return per_item * remaining

    def _eta_str(self) -> str:
        eta = self.eta_seconds()
        if eta is None:
            return "ETA —"
        if eta < 60:
            return f"ETA {eta:.0f}s"
        if eta < 3600:
            return f"ETA {eta/60:.1f}m"
        return f"ETA {eta/3600:.1f}h"

    def _render(self) -> None:
        if not self.enabled:
            return
        pct = min(100.0, self.current / self.total * 100.0)
        filled = min(self.width, round(self.current / self.total * self.width))
        bar = "█" * filled + "░" * (self.width - filled)
        elapsed = time.monotonic() - self._start
        msg = (
            f"\r[{self.label}] [{bar}] {self.current}/{self.total} ({pct:.0f}%) "
            f"{elapsed:.0f}s {self._eta_str()}   "
        )
        sys.stderr.write(msg)
        sys.stderr.flush()

    def finish(self, msg: str = "done") -> None:
        if not self.enabled:
            return
        self._render()
        sys.stderr.write(f" {msg}\n")
        sys.stderr.flush()


def pause_requested(pause_file: str | Path | None = None) -> bool:
    if pause_file is not None:
        path = str(pause_file).strip()
    else:
        path = os.environ.get("VALHALLA_PAUSE_FILE", "").strip()
    return bool(path) and Path(path).is_file()


def add_benchmark_args(ap: argparse.ArgumentParser) -> None:
    """Common flags for individual benchmark scripts."""
    ap.add_argument("--verbose", "-v", action="store_true", help="progress + per-item logs")
    ap.add_argument("--checkpoint", type=Path, help="checkpoint file path")
    ap.add_argument("--checkpoint-every", type=int, default=1, help="save every N items")
    ap.add_argument("--resume", "--resume-from", dest="resume", type=Path, help="resume from checkpoint")
    ap.add_argument("--pause-file", type=Path, help="create this file to pause gracefully")


def benchmark_extra_flags(args: argparse.Namespace) -> list[str]:
    """Translate argparse namespace → valhalla_base demo flags."""
    flags: list[str] = []
    if getattr(args, "verbose", False):
        flags.append("--verbose")
    if getattr(args, "checkpoint", None):
        flags.extend(["--checkpoint", str(args.checkpoint)])
    if getattr(args, "checkpoint_every", 0):
        flags.extend(["--checkpoint-every", str(args.checkpoint_every)])
    if getattr(args, "resume", None):
        flags.extend(["--resume", str(args.resume)])
    if getattr(args, "pause_file", None):
        flags.extend(["--pause-file", str(args.pause_file)])
    return flags


def load_suite_checkpoint(path: Path) -> dict[str, Any]:
    if not path.is_file():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def save_suite_checkpoint(path: Path, doc: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    doc["last_updated"] = datetime.now(timezone.utc).isoformat()
    path.write_text(json.dumps(doc, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def init_suite_checkpoint(
    path: Path,
    *,
    model: str,
    tests: list[str],
    resume: bool,
) -> dict[str, Any]:
    if resume and path.is_file():
        doc = load_suite_checkpoint(path)
        if doc.get("protocol") == SUITE_CHECKPOINT_PROTOCOL:
            return doc
    return {
        "protocol": SUITE_CHECKPOINT_PROTOCOL,
        "model": model,
        "started_at": datetime.now(timezone.utc).isoformat(),
        "completed": [],
        "failed": [],
        "pending": list(tests),
        "paused": False,
    }


def mark_suite_done(doc: dict[str, Any], label: str, *, exit_code: int, report: str | None = None) -> None:
    entry = {
        "label": label,
        "exit_code": exit_code,
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "report": report,
    }
    if exit_code == 0:
        doc["completed"] = [e for e in doc.get("completed", []) if e.get("label") != label]
        doc["completed"].append(entry)
        doc["failed"] = [e for e in doc.get("failed", []) if e.get("label") != label]
    else:
        doc["failed"] = [e for e in doc.get("failed", []) if e.get("label") != label]
        doc["failed"].append(entry)
    pending = doc.get("pending") or []
    if label in pending:
        pending.remove(label)
    doc["pending"] = pending


def suite_completed_labels(doc: dict[str, Any]) -> set[str]:
    return {e["label"] for e in doc.get("completed", []) if e.get("exit_code") == 0}


def iter_with_progress(
    label: str,
    items: list[Any],
    *,
    verbose: bool = False,
    start: int = 0,
) -> Iterator[tuple[int, Any]]:
    """Yield (index, item) with optional per-item progress bar."""
    total = len(items)
    bar = ProgressBar(label, total, enabled=verbose or start > 0)
    bar.set(start)
    for i in range(start, total):
        yield i, items[i]
        bar.inc()
    bar.finish()
