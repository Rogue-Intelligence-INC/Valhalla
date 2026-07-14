# Unique Proof Microbench — U1–U5

**Protocol:** `unique-proof-suite-v1`  
**Goal:** Prove capabilities **Mem0 / Zep / Letta lack**, not re-fight LoCoMo.  
**Product frame:** [`UNIQUE_VS_MEMORY_APIS.md`](./UNIQUE_VS_MEMORY_APIS.md)

```bash
./scripts/run_unique_proofs.sh --fast    # ~15–40 min GPU
./scripts/run_unique_proofs.sh --full    # deeper limits
./scripts/run_unique_proofs.sh --fast --only U2,U5
```

Outputs:

- `reports/valhalla_inference/unique_proof_suite.json`
- `reports/valhalla_inference/UNIQUE_PROOF_SUITE.md`

Each experiment records: `verdict` · `citeable` · `peer_status` · metrics · source report.

---

## Experiments

### U1 — Non-isolated multimodal

| Field | Value |
|-------|-------|
| Claim | One Triad session fuses text/TTS/image better than isolated late fusion |
| Valhalla | `valhalla-unified-core/scripts/run_cross_modal.py --limit N` (or cached summary when `--allow-cached`) |
| Peer | Mem0/Letta: **`SKIP_NO_SHARED_BODY`** — cannot share one incubating body; late-fuse proxy only |
| Pass (fast) | `avg_shared_patch_non_isolated ≥ avg_shared_patch_isolated + 0.02` **and** `non_isolated_wins ≥ max(10, 0.8·N)` |
| Full | `--limit 12` (or 107 when explicitly requested) |

### U2 — Proactive gate (emit + abstain)

| Field | Value |
|-------|-------|
| Claim | Idle can emit on-topic; mismatch abstains (FOG) |
| Valhalla | `test_proactive_fate_v2.py` — fast: `--gates-only` or handcraft+`--fair-limit 8`; full: default |
| Peer | Cron/tip agents: **`SKIP_NO_FOG`** or forced-answer FIR proxy if wired |
| Pass | `UP_on_topic ≥ 0.80` · `MAR_mismatch_abstain ≥ 0.70` · `FIR_false_interrupt ≤ 0.15` |

### U3 — Pollution rescue (TPI)

| Field | Value |
|-------|-------|
| Claim | Zero-weight transfer recovers accuracy vs polluted session dump |
| Valhalla | L1 path from `test_train_proof_smoke.run_tpi_transfer_smoke` / `test_valhalla_base_training_potential.py` |
| Peer | Mem0 search-all / RAG dump: **`SKIP_NO_TRANSFER_PROTOCOL`** — polluted retrieval stays polluted |
| Pass | `transfer_vs_polluted_pp ≥ +20` |

### U4 — Backbone orthogonality

| Field | Value |
|-------|-------|
| Claim | Open/native structure path independent of LM checkpoint |
| Valhalla | `run_valhalla_independence.py --fast` (optionally `--smokes-only`) |
| Peer | Answers drift with backbone when memory is prompt-only |
| Pass | Summary verdict contains `VALHALLA_BASE_INDEPENDENT` **or** open consistency ≥ configured threshold |

### U5 — Cost @ accuracy floor

| Field | Value |
|-------|-------|
| Claim | Append/prefill cheaper **while** staying on-topic |
| Valhalla | `test_token_efficiency.py --run-valhalla` (fast: fewer turns) |
| Peer | RAG full-resend / Mem0 re-inject |
| Pass | `on_topic_rate ≥ 0.80` **and** `prefill_tokens` ratio vs RAG full ≤ `0.35` |

**Status (2026-07-14 fast):** citeable developing — `COST_RATIO_PASS_ACC_DEVELOPING` (on-topic ~0.55, prefill ratio ~0.29). Full floor (≥0.80) still open. Suite `--fast` uses `--max-turns 20` so the cost claim is reproducible without a full floor.

---

## Peer policy (honest)

- Prefer **SKIP + reason** over invented peer scores.
- Unique-proof ≠ industry same-Q. Cite LoCoMo gaps separately.
- `citeable=true` only when Valhalla arm meets pass **and** report artifact exists.

## Memory note (2026-07-14)

Curriculum / multi-turn Python JSON path used to OOM by re-sending `fate_rollback_stack`
(up to 12 full `TriadIncubationSnapshot`s per turn). Fixed:

- Rust: `fate_rollback_stack` is `skip_serializing` on the wire
- Python: `ValhallaBaseSession.slim_session_for_wire()` strips stack + caps `question_history`
- Suite: `release_resources()` between U experiments; U3 `--fast` reuses cached TPI

---

## Separation of claims

| Track | Question |
|-------|----------|
| **Unique-proof** | Can peers even express this capability? |
| **Industry same-Q** | Who scores higher on LoCoMo / MultiHop today? |

---

*Rogue Intelligence LNC*
