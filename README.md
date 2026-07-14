# ValhallaBase

**Not another memory API.**  
A **structure-trainable body** for persistent agents — Triad incubation + Fate gate — with the LM as an optional decode head.

<p align="center">
  <a href="#unique">What peers lack</a> ·
  <a href="#prove-unique">Unique proofs</a> ·
  <a href="#prove-it">Evidence pack</a> ·
  <a href="#industry">Industry (honest)</a> ·
  <a href="https://github.com/Rogue-Intelligence-INC/valhalla-unified-core">Product docs (VUC)</a>
</p>

```text
multimodal ingress (text · TTS · image)
    → one Triad session (Hub · Tile · StemCell · Fate)
    → structure snapshot
    → FOG proactive / abstain · native or hybrid decode
    → optional LM head (0.5B default · 3B = calibration only)
```

> **Investor one-liner:** Structure substrate that Mem0 / Zep / Letta cannot express — non-isolated multimodal, Fate-gated proactive+abstain, zero-weight pollution rescue, backbone-orthogonal open path, append-cost memory.  
> **Not claimed:** LoCoMo SOTA · Transformer replacement · AGI · “drop-in Mem0 clone.”

---

<a name="unique"></a>
## What we have that memory APIs don’t

Mem0 · Zep/Graphiti · Letta optimize **extract → store → search → inject**.  
ValhallaBase is a **body**: incubate structure once, then gate, retrieve, and optionally decode.

| Capability | ValhallaBase | Mem0 / Zep / Letta |
|------------|--------------|---------------------|
| **Non-isolated multimodal** — one session, shared Fate patch across text/TTS/image | **Yes** · shared **0.9999** vs iso **0.968** | Late-fuse / separate memories |
| **Fate-gated proactive + abstain** — `proactive_idle` · FOG emit/refuse | **Yes** · MAR **1.0** · FIR **0.0** | Pull-on-query / app cron / no algebraic gate |
| **Zero-weight pollution rescue** — TPI transfer vs polluted dump | **Yes** · **+38.7pp** | Search-all stays polluted |
| **Backbone-orthogonal open path** — swap LM, structure holds | **Yes** · `VALHALLA_BASE_INDEPENDENT` | Prompt/runtime tied to LM |
| **Structure-cost append** — wire/prefill ≪ full resend | **Secondary** · ~0.29× when floor met (U5 developing) | Full resend / re-inject retrieved text |

Full matrix: [`docs/valhalla/UNIQUE_VS_MEMORY_APIS.md`](docs/valhalla/UNIQUE_VS_MEMORY_APIS.md) · Canonical numbers: [`docs/valhalla/CANONICAL_METRICS_20260714.md`](docs/valhalla/CANONICAL_METRICS_20260714.md)

<a name="prove-unique"></a>
### Prove unique capabilities (lead suite)

```bash
./scripts/run_unique_proofs.sh --fast --only U1,U2,U3,U4
```

**Latest `--fast` (2026-07-14): U1–U4 = 4/4 pass · 4/4 citeable** — [`UNIQUE_PROOF_SUITE.md`](reports/valhalla_inference/UNIQUE_PROOF_SUITE.md)

| ID | Claim | Verdict |
|----|-------|---------|
| U1 | Non-isolated multimodal | `NON_ISOLATED_BETTER` |
| U2 | FOG proactive / abstain | `PROACTIVE_ABSTAIN_PASS` |
| U3 | Pollution rescue (TPI) | `POLLUTION_RESCUE_PASS` (+38.7pp) |
| U4 | Backbone orthogonality | `VALHALLA_BASE_INDEPENDENT` |
| U5 | Cost @ floor | **Secondary / developing** — do not lead |

Protocol: [`UNIQUE_PROOF_MICROBENCH.md`](docs/valhalla/UNIQUE_PROOF_MICROBENCH.md)

---

<a name="prove-it"></a>
## Evidence pack (legitimacy · logs · zip)

| Artifact | Role |
|----------|------|
| `./scripts/run_unique_proofs.sh` | **Primary** — unique vs memory APIs |
| `./scripts/run_evidence_suite.sh` | Legitimacy · same-Q competitor · ablation · zip |
| `audit_benchmark_legitimacy.py` | Citeable vs Tier-C gate |
| `pack_evidence_run.py` | MANIFEST · HASHES · REPRODUCE.md |

```bash
python3 -m venv .venv-llm && .venv-llm/bin/pip install -r requirements-benchmark.txt
./scripts/run_unique_proofs.sh --fast
./scripts/run_evidence_suite.sh --full   # optional diligence pack
```

Share first: **UNIQUE_PROOF_SUITE.md** · legitimacy_audit.json · HASHES.sha256.

---

<a name="one-click"></a>
## One-click smoke

```bash
git clone --recurse-submodules https://github.com/Rogue-Intelligence-INC/Valhalla.git
cd Valhalla
./scripts/run_highlight_test.sh          # pillars
./scripts/demo_valhalla_chat.sh          # web console
```

---

<a name="industry"></a>
## Industry same-Q (secondary · honest)

Same-backbone compare is **not** our wedge — we publish it so diligence isn’t surprising.

**Calibration LM:** `Qwen2.5-3B-Instruct` · **Product default:** `Qwen2.5-0.5B-Instruct`

### Tier A faithfulness / needle (cite)

| Benchmark | Result |
|-----------|--------|
| RAGTruth-40 / 100 | **38/40 · 88/100** |
| Needle | Strong (session append; 4/4 on prior full needle suite) |
| Ablation RAGTruth | native/hybrid **34–37/40** vs LM/RAG **0/40** (2026-07-13 evidence) |
| Longchain continuity | theme_drift **0%** · retention **100%** (`fragment_full`) |

### Tier B open-QA / long context (boundary)

| Benchmark | Result |
|-----------|--------|
| LoCoMo | **17/30** industry · competitor moderate **0/15** vs RAG **9/15** (honest gap) |
| MultiHop | **15/25** · competitor **6/10** (tie) |
| LongBench / LaRA / FRAMES | Developing / wiki-enriched smoke |

Details: [`EVIDENCE_SUITE_SUMMARY_20260713.md`](reports/valhalla_inference/EVIDENCE_SUITE_SUMMARY_20260713.md) · [`COMPETITOR_COMPARE_RESULTS.md`](reports/valhalla_inference/COMPETITOR_COMPARE_RESULTS.md)

---

## Architecture

| | Memory API (Mem0/Zep/Letta) | ValhallaBase |
|--|-----------------------------|--------------|
| Product unit | Records / graph / blocks | **Triad structure snapshot** |
| Multimodal | Separate stores → late fuse | **One session · non-isolated** |
| Initiative | Query-time retrieve | **FOG proactive + abstain** |
| Learning | Extract facts / optional FT | **L1 session · L2 Fate · L3 bake** |
| Cost | Re-inject retrieved text | **Append wire · question-only prefill** |

Positioning: [VALHALLA_BASE_PRODUCT_POSITIONING.md](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/VALHALLA_BASE_PRODUCT_POSITIONING.md)

### Surfaces

| Path | Role |
|------|------|
| [`manifestsys/`](manifestsys/) | Runtime — `valhalla_base` |
| [`tools/valhalla_inference/`](tools/valhalla_inference/) | Unique proofs · industry benches · evidence |
| [`apps/valhalla_chat/`](apps/valhalla_chat/) | Web console |
| [`valhalla-unified-core/`](valhalla-unified-core/) | Product docs submodule |

---

## Requirements

- Rust 1.70+ · CUDA optional (`RUSTFLAGS='-L /opt/cuda/lib64'`)
- Python 3.10+ · `.venv-llm` + [`requirements-benchmark.txt`](requirements-benchmark.txt)

## License

Runtime: [valhalla-unified-core/NON_OPEN_SOURCE.md](valhalla-unified-core/NON_OPEN_SOURCE.md).  
Benchmark harness & reports follow repository notices.

---

*Rogue Intelligence LNC · [Organization](https://github.com/Rogue-Intelligence-INC)*
