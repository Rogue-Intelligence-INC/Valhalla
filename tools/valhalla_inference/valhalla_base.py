#!/usr/bin/env python3
"""Valhalla-as-base — thin wrapper over Rust `valhalla_base` binary.

**Production / long-chain / batch eval: use Rust in-process or CLI:**
  hub_f64::ValhallaBaseClient
  valhalla_base demo locomo|needle-prod
  cargo test -p hub-f64 --lib valhalla_base_client::tests --release --features patch-lm

Python `run()` remains JSON stdin/stdout to the same Rust core (not a separate logic path).
"""

from __future__ import annotations

import copy
import json
import os
import subprocess
import time
import urllib.error
import urllib.request
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Literal

REPO = Path(__file__).resolve().parents[2]
BIN = REPO / "target/release/valhalla_base"
DEFAULT_LM = "Qwen/Qwen2.5-0.5B-Instruct"


def default_lm_model() -> str:
    return os.environ.get("VALHALLA_LM_MODEL", DEFAULT_LM).strip()


DecodeMode = Literal["native", "lm_patch", "hybrid"]
McqDecodeMode = Literal["lm_patch", "structure_fate", "max_pick"]
FollowUpDecodeMode = Literal["inherit", "lm_patch_on_follow_up", "native_follow_up_aware"]
ContextIngestMode = Literal["targeted", "retrieval", "full"]


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
    """Valhalla is the base; LM is decode head.

    Production stack (default / unified solution):
      hybrid decode · native_follow_up_aware · quad_cycle ingress
      open_fate + multihop_v2 (fatewithv2) · RelAddr land · fate_qa_feedback
    """

    lm_model: str = field(default_factory=default_lm_model)
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
    open_fate: bool = True
    multihop_fate: bool = True
    multihop_v2: bool = True
    context_ingest: ContextIngestMode = "targeted"
    reanchor_affinity: bool = True
    fate_warmup_enabled: bool = True
    fate_warmup_cycles: int = 4
    fate_warmup_prelearn_lines: int = 0
    session: ValhallaBaseSession = field(default_factory=ValhallaBaseSession)
    context_items: list[dict[str, Any]] = field(default_factory=list)
    _pending_append: list[dict[str, Any]] = field(default_factory=list, repr=False)
    _use_append_only: bool = field(default=False, repr=False)

    def set_context(self, lines: list[str], *, preserve_session: bool = False) -> None:
        """Replace wire context. Clears session unless preserve_session=True."""
        if not preserve_session:
            self.session = ValhallaBaseSession()
        self.context_items = [{"text": ln} for ln in lines if ln.strip()]
        self._pending_append.clear()
        self._use_append_only = False

    def reanchor_session_corpus(self) -> None:
        """Clear session corpus fingerprint but keep snapshot affinity + history."""
        if not self.session.session_state:
            return
        sess = copy.deepcopy(self.session.session_state)
        sess["corpus"] = []
        sess["fingerprint"] = ""
        self.session.session_state = sess
        self.context_items.clear()
        self._pending_append.clear()

    def ingest_for_prompt(
        self,
        prompt: dict[str, Any],
        corpus: list[str],
        *,
        k: int = 8,
        mode: ContextIngestMode | None = None,
    ) -> list[str]:
        """Production per-question ingest: targeted lines + optional affinity reanchor."""
        from context_routing import resolve_context_lines  # noqa: WPS433

        ingest = mode or self.context_ingest
        lines = resolve_context_lines(prompt, corpus, mode=ingest, k=k)
        if self.reanchor_affinity and self.session.session_state is not None:
            self.reanchor_session_corpus()
            for ln in lines:
                self.append_context(ln)
            self._use_append_only = True
        else:
            self.set_context(lines)
        return lines

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
            self._pending_append.append(item)

    def session_extend(self) -> dict[str, Any]:
        """Extend session snapshot from pending append lines (no QA — long-chain ingest)."""
        self._ensure_bin()
        if not self.session.session_state:
            raise RuntimeError("session_extend requires an active session")
        pending = list(self._pending_append)
        if not pending:
            raise RuntimeError("session_extend requires pending append lines")
        payload: dict[str, Any] = {
            "questions": [],
            "session_extend_only": True,
            "session": self.session.session_state,
            "append_items": pending,
            "append_lines": [str(it["text"]) for it in pending if it.get("text")],
            "config": self._config_payload(),
        }
        doc = self._run_bin(payload)
        self.session.apply_response(doc)
        self._pending_append.clear()
        return doc

    def clear_context(self) -> None:
        self.session = ValhallaBaseSession()
        self.context_items.clear()
        self._pending_append.clear()
        self._use_append_only = False

    def _config_payload(self) -> dict[str, Any]:
        return {
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
            "fate_warmup_enabled": self.fate_warmup_enabled,
            "fate_warmup_cycles": self.fate_warmup_cycles,
            "fate_warmup_prelearn_lines": self.fate_warmup_prelearn_lines,
            "core_fusion": self.core_fusion,
            "proactive_enabled": self.proactive_enabled,
            "proactive_min_fate_signals": self.proactive_min_fate_signals,
            "proactive_min_turns": self.proactive_min_turns,
            "proactive_min_spread": self.proactive_min_spread,
            "open_fate": self.open_fate,
            "multihop_fate": self.multihop_fate,
            "multihop_v2": self.multihop_v2,
        }

    def _pause_requested(self, pause_file: str | None = None) -> bool:
        path = (pause_file or os.environ.get("VALHALLA_PAUSE_FILE", "")).strip()
        return bool(path) and Path(path).is_file()

    def save_checkpoint(self, path: str | Path, *, index: int, rows: list[dict[str, Any]], warmup_done: bool = False) -> None:
        """Persist batch progress for resume."""
        doc = {
            "protocol": "valhalla-base-checkpoint-v1",
            "index": index,
            "questions_done": index,
            "rows": rows,
            "session_state": self.session.session_state,
            "warmup_done": warmup_done,
            "timestamp": datetime.now(timezone.utc).isoformat(),
        }
        p = Path(path)
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(json.dumps(doc, ensure_ascii=False, indent=2), encoding="utf-8")

    def load_checkpoint(self, path: str | Path) -> dict[str, Any]:
        """Restore session from checkpoint; returns checkpoint doc."""
        doc = json.loads(Path(path).read_text(encoding="utf-8"))
        sess = doc.get("session_state")
        if sess:
            self.session.session_state = sess
            self.session.corpus = list(sess.get("corpus") or [])
            self.session.fingerprint = str(sess.get("fingerprint") or "")
        return doc

    def fate_warmup_corpus(self, corpus: list[str], *, n_lines: int = 4) -> dict[str, Any]:
        """Fate prelearn + warmup on seed corpus lines (no QA decode)."""
        self._ensure_bin()
        lines = [ln.strip() for ln in corpus if ln.strip()]
        if not lines:
            raise ValueError("fate_warmup_corpus: corpus is empty")
        if len(lines) == 1:
            lines = lines * 2
        else:
            lines = lines[: max(2, n_lines)]
        payload: dict[str, Any] = {
            "context": lines,
            "questions": [],
            "fate_warmup_only": True,
            "config": self._config_payload(),
        }
        if self.session.session_state:
            payload["session"] = self.session.session_state
        doc = self._run_bin(payload)
        self.session.apply_response(doc)
        return doc

    def dialogue_warmup(self, turns: list[str]) -> None:
        """Open-turn dialogue warmup before eval (builds hub_prefs spread)."""
        for i, q in enumerate(turns):
            self.ask(q, qid=f"dw{i}")

    def run_batch(
        self,
        questions: list[dict[str, Any]],
        corpus: list[str],
        *,
        ingest_mode: ContextIngestMode | None = None,
        context_k: int = 8,
        warmup_corpus_lines: int = 4,
        dialogue_warmup: list[str] | None = None,
        checkpoint_path: str | Path | None = None,
        checkpoint_every: int = 0,
        resume_from: str | Path | None = None,
        pause_file: str | None = None,
    ) -> dict[str, Any]:
        """Production eval: Fate corpus warmup → optional dialogue → per-question ingest + QA."""
        all_rows: list[dict[str, Any]] = []
        last_doc: dict[str, Any] = {}
        start_idx = 0
        warmup_done = False

        if resume_from:
            ckpt = self.load_checkpoint(resume_from)
            all_rows = list(ckpt.get("rows") or [])
            start_idx = int(ckpt.get("index") or len(all_rows))
            warmup_done = bool(ckpt.get("warmup_done"))
            if checkpoint_path is None:
                checkpoint_path = resume_from

        if not warmup_done and self.fate_warmup_enabled and warmup_corpus_lines > 0:
            last_doc = self.fate_warmup_corpus(corpus, n_lines=warmup_corpus_lines)
            warmup_done = True

        if dialogue_warmup:
            self.dialogue_warmup(dialogue_warmup)

        paused = False
        for i in range(start_idx, len(questions)):
            if self._pause_requested(pause_file):
                paused = True
                if checkpoint_path:
                    self.save_checkpoint(
                        checkpoint_path,
                        index=i,
                        rows=all_rows,
                        warmup_done=warmup_done,
                    )
                break
            q = questions[i]
            self.ingest_for_prompt(q, corpus, k=context_k, mode=ingest_mode)
            doc = self.run([q], append_only=self._use_append_only)
            self._use_append_only = False
            last_doc = doc
            all_rows.extend(doc.get("rows") or [])
            if checkpoint_path and checkpoint_every > 0 and (i + 1) % checkpoint_every == 0:
                self.save_checkpoint(
                    checkpoint_path,
                    index=i + 1,
                    rows=all_rows,
                    warmup_done=warmup_done,
                )

        if not last_doc:
            return {"rows": [], "protocol": "valhalla-base-v1", "paused": paused}
        out = dict(last_doc)
        out["rows"] = all_rows
        out["paused"] = paused
        out["questions_done"] = len(all_rows)
        if checkpoint_path and not paused and len(all_rows) >= len(questions):
            self.save_checkpoint(
                checkpoint_path,
                index=len(questions),
                rows=all_rows,
                warmup_done=warmup_done,
            )
        return out

    def _ensure_bin(self) -> None:
        if BIN.is_file():
            return
        subprocess.run(
            ["cargo", "build", "-p", "hub-f64", "--release", "--bin", "valhalla_base"],
            check=True,
            cwd=REPO,
            env={**os.environ, "RUSTFLAGS": "-L /opt/cuda/lib64", "LIBRARY_PATH": os.environ.get("LIBRARY_PATH", f"/opt/cuda/targets/x86_64-linux/lib:{os.environ.get('LIBRARY_PATH', '')}")},
        )

    def _serve_url(self) -> str | None:
        url = os.environ.get("VALHALLA_BASE_URL", "").strip().rstrip("/")
        return url or None

    def _run_http(self, payload: dict[str, Any]) -> dict[str, Any]:
        base_url = self._serve_url()
        if not base_url:
            raise RuntimeError("VALHALLA_BASE_URL not set")
        data = json.dumps(payload).encode("utf-8")
        last_exc: Exception | None = None
        for attempt in range(5):
            req = urllib.request.Request(
                f"{base_url}/v1/infer",
                data=data,
                headers={"Content-Type": "application/json"},
                method="POST",
            )
            try:
                with urllib.request.urlopen(req, timeout=3600) as resp:
                    raw = resp.read().decode("utf-8").strip()
                break
            except (urllib.error.URLError, OSError) as exc:
                last_exc = exc
                if attempt < 4 and ("Broken pipe" in str(exc) or "Connection reset" in str(exc)):
                    time.sleep(2.0 * (attempt + 1))
                    continue
                raise
            except urllib.error.HTTPError as exc:
                raw = exc.read().decode("utf-8", errors="replace").strip()
                if raw:
                    doc = json.loads(raw)
                    raise RuntimeError(doc.get("error") or raw) from exc
                raise RuntimeError(str(exc)) from exc
        else:
            raise RuntimeError(str(last_exc) or "valhalla_base serve failed")
        if not raw:
            raise RuntimeError("valhalla_base serve returned empty body")
        doc = json.loads(raw)
        if doc.get("ok") is False:
            raise RuntimeError(doc.get("error") or "valhalla_base serve failed")
        return doc

    def _run_bin(self, payload: dict[str, Any]) -> dict[str, Any]:
        if self._serve_url():
            return self._run_http(payload)
        proc = subprocess.run(
            [str(BIN), "--json-error"],
            input=json.dumps(payload),
            capture_output=True,
            text=True,
            env={**os.environ, "RUSTFLAGS": "-L /opt/cuda/lib64", "LIBRARY_PATH": os.environ.get("LIBRARY_PATH", f"/opt/cuda/targets/x86_64-linux/lib:{os.environ.get('LIBRARY_PATH', '')}")},
        )
        if proc.stderr:
            print(proc.stderr, end="", flush=True)
        raw = proc.stdout.strip()
        if not raw:
            raise RuntimeError(proc.stderr or "valhalla_base returned empty stdout")
        doc = json.loads(raw)
        if proc.returncode != 0 or doc.get("ok") is False:
            raise RuntimeError(doc.get("error") or proc.stderr or "valhalla_base failed")
        return doc

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
            "config": self._config_payload(),
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

        proc = self._run_bin(payload)
        doc = proc
        self.session.apply_response(doc)
        return doc

    def ask(self, text: str, *, score_type: str = "open", qid: str = "q0") -> str:
        append_only = bool(self._pending_append and self.session.session_state)
        doc = self.run(
            [{"id": qid, "text": text, "score_type": score_type}],
            append_only=append_only,
        )
        rows = doc.get("rows") or []
        return rows[0]["answer"] if rows else ""

    def enable_unified_solution(self) -> "ValhallaBase":
        """Programmable: RelAddr land + Fate adapt + multihop v2 (fatewithv2)."""
        self.open_fate = True
        self.multihop_fate = True
        self.multihop_v2 = True
        self.decode = "hybrid"
        self.follow_up_decode = "native_follow_up_aware"
        self.fate_ingress_routing = "quad_cycle"
        self.fate_qa_feedback = True
        return self

    def fate_ingress_for(
        self,
        prompt: dict[str, Any],
        corpus: list[str],
        *,
        k: int = 8,
    ) -> list[str]:
        """Generic RelAddr-style ingress via targeted routing (Python adapter).

        Rust `fate_ingress` is the canonical adapter; this mirrors the entry for
        programmable ValhallaBase scripts without spawning a second binary RPC.
        """
        return self.ingest_for_prompt(prompt, corpus, k=k, mode="targeted")

    def ask_unified(
        self,
        text: str,
        corpus: list[str] | None = None,
        *,
        use_ingress: bool = True,
        qid: str = "q0",
    ) -> str:
        """Unified solution ask: enable fatewithv2 flags, optional ingress, then ask."""
        self.enable_unified_solution()
        if corpus is not None and use_ingress:
            self.fate_ingress_for({"id": qid, "text": text, "score_type": "open"}, corpus)
        elif corpus is not None and not self.session.session_state and not self.context_items:
            self.set_context(corpus)
        return self.ask(text, qid=qid)

    def proactive_idle(self) -> dict[str, Any]:
        """No user question — self-probe + FOG-gated nudge after dialogue warmup."""
        self._ensure_bin()
        if not self.session.session_state:
            raise RuntimeError("proactive_idle requires an active session (warmup dialogue first)")
        payload: dict[str, Any] = {
            "questions": [],
            "proactive_idle": True,
            "session": self.session.session_state,
            "config": self._config_payload(),
        }
        doc = self._run_bin(payload)
        self.session.apply_response(doc)
        return doc
