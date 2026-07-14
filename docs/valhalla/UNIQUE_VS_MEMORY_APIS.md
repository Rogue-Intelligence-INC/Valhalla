# Valhalla vs Memory APIs — What We Have That They Don’t

**Audience:** fundraise / diligence · product positioning  
**Date:** 2026-07-14  
**Peer set (2026 public narrative):** Mem0 · Zep/Graphiti · Letta (MemGPT)

> **Frame:** Do **not** sell Valhalla as “better LoCoMo than Mem0.”  
> Sell **structure substrate** capabilities peers lack: non-isolated multimodal, Fate-gated proactive + abstain, zero-weight pollution rescue, backbone-orthogonal decode, structure-cost append.

Industry same-Q (LoCoMo etc.) remains an **honest boundary** — see [`EVIDENCE_SUITE_SUMMARY_20260713.md`](../../reports/valhalla_inference/EVIDENCE_SUITE_SUMMARY_20260713.md).

Detailed capability catalog: [`DISTINCTIVE_CAPABILITIES_CN.md`](../../valhalla-unified-core/docs/DISTINCTIVE_CAPABILITIES_CN.md)

**Investor deep-dive (tests + demo script + honest bounds):** [`INVESTOR_STRENGTHS_SHOWCASE.md`](./INVESTOR_STRENGTHS_SHOWCASE.md)

---

## Capability matrix

| Capability | ValhallaBase | Mem0 | Zep / Graphiti | Letta | Engram (adjacent) |
|------------|--------------|------|----------------|-------|-------------------|
| **Non-isolated multimodal** (one Triad session · shared Fate patch across text/TTS/image) | **Yes** | Partial (store image/PDF as extra memories; late fuse in prompt) | No shared body | Text/block tiers; no Triad pulse | Org-knowledge compression (different bet) |
| **Proactive idle + FOG abstain** (`proactive_idle`, UP/MAR/FIR) | **Yes** | No (pull on query) | No (app schedules tips) | Agent loop can tip; **no algebraic emit/abstain gate** | N/A (learned memory layer) |
| **Zero-weight session learning / pollution rescue** (TPI transfer vs polluted dump) | **Yes** | Search-all still polluted | Graph may retain contradictions with validity windows — different problem | Agent edits blocks; no TPI protocol | Continual retention via trained engram |
| **Backbone-orthogonal open path** (swap LM head; structure stable) | **Yes** (native/open) | Memory tied to extraction/chat LM | Server + LM for answers | Runtime + model coupled | Typically model-training coupled |
| **Structure-cost append** (wire/prefill ≪ full resend **at accuracy floor**) | **Yes** (when U5 passes) | Compact facts help; still re-inject retrieved text | Graph retrieve → prompt | Tool paging still grows context | **Core marketing** (1–10% tokens) |
| Extract → vector/graph → inject | Secondary (optional LM head) | **Core product** | **Core product** | Self-managed tiers | Secondary |
| Temporal fact validity windows | Not the wedge | Optional Pro graph | **Core product** | Limited | Point-in-time via trained memory |

---

## What peers optimize (so we don’t compete on their field)

| Product | Their bet | Our honest response |
|---------|-----------|---------------------|
| **Mem0** | Fastest personalization layer on any stack | We are not a drop-in memory API; compare **unique-proof suite**, not LongMemEval marketing numbers |
| **Zep** | Temporal KG / “what was true when” | Cite Graphiti when temporal CRM is the job; Valhalla wedge is **body + gate + multimodal** |
| **Letta** | Agent-as-OS, self-editing memory blocks | Closest philosophically on “stateful agent”; still **text blocks + tools**, not Triad incubation |
| **Engram** | Learned enterprise memory → fewer tokens | Capital-category peer for “AI memory infra”; Valhalla is **runtime structure body**, not a trained engram SKU |

---

## Emphasize these five (proof protocols)

| # | Claim | Proof suite ID | Existing internal evidence |
|---|-------|----------------|----------------------------|
| 1 | Non-isolated multimodal | **U1** | cross-modal 12-task: shared patch **0.9999** vs iso **0.968**; wins **12/12** |
| 2 | Proactive + mismatch abstain | **U2** | proactive-fate-v2 UP/MAR/FIR · agency PPI |
| 3 | Pollution rescue (no SGD) | **U3** | TPI transfer vs polluted (+20–60pp class lifts) |
| 4 | Backbone orthogonality | **U4** | `VALHALLA_BASE_INDEPENDENT` |
| 5 | Cost @ accuracy floor | **U5** | token-efficiency-v1 ratios **with** on-topic floor |

Protocol: [`UNIQUE_PROOF_MICROBENCH.md`](./UNIQUE_PROOF_MICROBENCH.md)  
Runner: `./scripts/run_unique_proofs.sh --fast`

---

## Do **not** lead with

| Claim | Why not |
|-------|---------|
| Beat Mem0 LoCoMo / LongMemEval | Same-Q moderate: Valhalla **0/15** vs Mem0 **5/15** vs RAG **9/15** — honest gap |
| “Always cheaper tokens” without floor | Latest broken on-topic arm made cost-only cite invalid until U5 fixed |
| `lm_patch` is backbone-agnostic | Open/native is; MCQ patch is Qwen/Candle-bound |

---

## One-liner (CN / EN)

**中文：** Valhalla 不是又一个记忆 API，而是可孵化的结构身体：非隔离多模态、Fate 主动与拒答、零权重污染救援、换骨干解码头、结构化增量成本。

**EN:** ValhallaBase is a structure-trainable body — non-isolated multimodal, Fate-gated proactive/abstain, zero-weight pollution rescue, backbone-orthogonal decode, append-cost structure — not another extract-and-retrieve memory API.

---

*Rogue Intelligence LNC*
