#!/usr/bin/env python3
"""Valhalla-as-base Python API — hybrid, persistent session, multimodal context."""

from __future__ import annotations

import json
import os
import subprocess
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Literal

REPO = Path(__file__).resolve().parents[2]
BIN = REPO / "target/release/valhalla_base"

DecodeMode = Literal["native", "lm_patch", "hybrid"]
McqDecodeMode = Literal["lm_patch", "structure_fate", "max_pick"]
FollowUpDecodeMode = Literal["inherit", "lm_patch_on_follow_up", "native_follow_up_aware"]


@dataclass
class ValhallaBaseSession:
    """Persistent context session — incremental append without full re-ingest."""

    corpus: list[str] = field(default_factory=list)
    fingerprint: str = ""
    session_state: dict[str, Any] | None = None

    def apply_response(self, doc: dict[str, Any]) -> None:
        sess = doc.get("session") or {}
        self.corpus = list(sess.get("corpus") or [])
        self.fingerprint = str(sess.get("fingerprint") or "")
        self.session_state = sess


@dataclass
class ValhallaBase:
    """Valhalla is the base; LM is decode head. Supports hybrid / session / multimodal."""

    lm_model: str = "Qwen/Qwen2.5-0.5B-Instruct"
    patch_body: str = "triad"
    strength: float = 0.08
    max_new: int = 128
    incubation_cycles: int = 2
    decode: DecodeMode = "hybrid"
    mcq_decode: McqDecodeMode = "lm_patch"
    follow_up_decode: FollowUpDecodeMode = "native_follow_up_aware"
    fate_weight_mode: str = "fixed"
    fate_routing_free: bool = False
    fate_ingress_routing: str = "quad_cycle"
    fate_qa_feedback: bool = True
    core_fusion: bool = False
    proactive_enabled: bool = True
    proactive_min_fate_signals: int = 64
    proactive_min_turns: int = 2  # legacy alias → min_fate_signals = turns * 32
    proactive_min_spread: float = 12 / 138 * 5 / 140  # τ(0)·δ/(dim+r)
    session: ValhallaBaseSession = field(default_factory=ValhallaBaseSession)
    context_items: list[dict[str, Any]] = field(default_factory=list)
    _pending_append: list[dict[str, Any]] = field(default_factory=list, repr=False)

    def set_context(self, lines: list[str]) -> None:
        self.session = ValhallaBaseSession()
        self.context_items = [{"text": ln} for ln in lines if ln.strip()]
        self._pending_append.clear()

    def append_context(self, line: str) -> None:
        line = line.strip()
        if line:
            item = {"text": line}
            self.context_items.append(item)
            self._pending_append.append(item)

    def append_multimodal(
        self,
        *,
        text: str | None = None,
        audio_features: list[float] | None = None,
        image_features: list[float] | None = None,
    ) -> None:
        item: dict[str, Any] = {}
        if text and text.strip():
            item["text"] = text.strip()
        if audio_features:
            item["audio_features"] = audio_features
        if image_features:
            item["image_features"] = image_features
        if item:
            self.context_items.append(item)

    def clear_context(self) -> None:
        self.session = ValhallaBaseSession()
        self.context_items.clear()
        self._pending_append.clear()

    def _ensure_bin(self) -> None:
        if BIN.is_file():
            return
        subprocess.run(
            ["cargo", "build", "-p", "hub-f64", "--release", "--bin", "valhalla_base"],
            check=True,
            cwd=REPO,
            env={**os.environ, "RUSTFLAGS": "-L /opt/cuda/lib64"},
        )

    def run(
        self,
        questions: list[dict[str, Any]],
        *,
        context: list[str] | None = None,
        context_items: list[dict[str, Any]] | None = None,
        append_only: bool = False,
    ) -> dict[str, Any]:
        self._ensure_bin()
        payload: dict[str, Any] = {
            "questions": questions,
            "config": {
                "lm_model": self.lm_model,
                "patch_body": self.patch_body,
                "strength": self.strength,
                "max_new": self.max_new,
                "incubation_cycles": self.incubation_cycles,
                "decode": self.decode,
                "mcq_decode": self.mcq_decode,
                "follow_up_decode": self.follow_up_decode,
                "fate_weight_mode": self.fate_weight_mode,
                "fate_routing_free": self.fate_routing_free,
                "fate_ingress_routing": self.fate_ingress_routing,
                "fate_qa_feedback": self.fate_qa_feedback,
                "core_fusion": self.core_fusion,
                "proactive_enabled": self.proactive_enabled,
                "proactive_min_fate_signals": self.proactive_min_fate_signals,
                "proactive_min_turns": self.proactive_min_turns,
                "proactive_min_spread": self.proactive_min_spread,
            },
        }

        if append_only and self.session.session_state:
            payload["session"] = self.session.session_state
            pending = context_items if context_items is not None else self._pending_append
            payload["append_items"] = pending
            payload["append_lines"] = [
                str(it["text"]) for it in pending if it.get("text")
            ]
            self._pending_append.clear()
        else:
            has_fresh_context = context is not None or context_items is not None
            continuing = self.session.session_state is not None and not has_fresh_context
            if continuing:
                payload["session"] = self.session.session_state
            else:
                items = context_items if context_items is not None else self.context_items
                if context is not None:
                    payload["context"] = context
                elif items:
                    payload["context"] = [
                        it["text"] for it in items if it.get("text")
                    ]
                if items:
                    payload["context_items"] = items
                if self.session.session_state and not has_fresh_context:
                    payload["session"] = self.session.session_state

        proc = subprocess.run(
            [str(BIN)],
            input=json.dumps(payload),
            capture_output=True,
            text=True,
            check=True,
            env={**os.environ, "RUSTFLAGS": "-L /opt/cuda/lib64"},
        )
        if proc.stderr:
            print(proc.stderr, end="", flush=True)
        doc = json.loads(proc.stdout)
        self.session.apply_response(doc)
        return doc

    def ask(self, text: str, *, score_type: str = "open", qid: str = "q0") -> str:
        doc = self.run([{"id": qid, "text": text, "score_type": score_type}])
        rows = doc.get("rows") or []
        return rows[0]["answer"] if rows else ""

    def proactive_idle(self) -> dict[str, Any]:
        """No user question — self-probe + FOG-gated nudge after dialogue warmup."""
        self._ensure_bin()
        if not self.session.session_state:
            raise RuntimeError("proactive_idle requires an active session (warmup dialogue first)")
        payload: dict[str, Any] = {
            "questions": [],
            "proactive_idle": True,
            "session": self.session.session_state,
            "config": {
                "lm_model": self.lm_model,
                "patch_body": self.patch_body,
                "strength": self.strength,
                "max_new": self.max_new,
                "incubation_cycles": self.incubation_cycles,
                "decode": self.decode,
                "mcq_decode": self.mcq_decode,
                "follow_up_decode": self.follow_up_decode,
                "fate_weight_mode": self.fate_weight_mode,
                "fate_routing_free": self.fate_routing_free,
                "fate_ingress_routing": self.fate_ingress_routing,
                "fate_qa_feedback": self.fate_qa_feedback,
                "proactive_enabled": self.proactive_enabled,
                "proactive_min_fate_signals": self.proactive_min_fate_signals,
                "proactive_min_turns": self.proactive_min_turns,
                "proactive_min_spread": self.proactive_min_spread,
            },
        }
        proc = subprocess.run(
            [str(BIN)],
            input=json.dumps(payload),
            capture_output=True,
            text=True,
            check=True,
            env={**os.environ, "RUSTFLAGS": "-L /opt/cuda/lib64"},
        )
        if proc.stderr:
            print(proc.stderr, end="", flush=True)
        doc = json.loads(proc.stdout)
        self.session.apply_response(doc)
        return doc
