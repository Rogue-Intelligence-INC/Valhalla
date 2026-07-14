# Design → Properties Map — ValhallaBase (2026-07-14)

**Purpose:** Separate **what we designed** from **what we measured** (and what we must not overclaim).  
**Audience:** diligence, engineering, landing “technical core” readers.  
**Status:** Canonical reading guide · Rogue Intelligence LNC.

Many cited “characteristics” are not marketing inventions after the fact — they are **consequences of explicit design choices**. Emergence tests ask a narrower question: *given that design, does the resulting graph behave non-randomly / casually?*

```
┌──────────────────────────┐
│  Design (intentional)    │  architecture, defaults, scoring weights
└────────────┬─────────────┘
             ▼
┌──────────────────────────┐
│  Runtime substrate       │  Hub / Tile / Stem / Fate / fragments
└────────────┬─────────────┘
             ▼
┌──────────────────────────┐
│  Measured properties     │  σ, φ, organ coalescence, Unique Proofs…
└──────────────────────────┘
```

---

## 1. Three layers (use this vocabulary)

| Layer | Meaning | Example |
|-------|---------|---------|
| **D — Designed** | We built this on purpose; it runs when defaults are on | `quad_cycle` ingress; FOG; hybrid decode |
| **E — Emergent under design** | Design does not hardcode the outcome; structure forms and we **measure** it | Stem small-world σ; same-type coalescence |
| **P — Product proof** | Mechanism difference vs memory APIs on frozen protocols | Unique Proofs U1–U4 |

Do **not** call a **D** item “discovered biology.”  
Do **not** call an **E** item “guaranteed by theorem” without the named probe.  
Do **not** sell a **P** item as LoCoMo SOTA.

---

## 2. Design catalog → expected properties

| Design (code / default) | Where | Intended consequence | Typical measured property |
|-------------------------|-------|----------------------|---------------------------|
| Triad body: Hub + Tile + StemCell | `patch_body=triad` | One session grows multimodal structure before LM | Tile aggregates · Stem cells in fragments |
| Fate `quad_cycle` ingress | `fate_ingress_routing` | Rotate four Hub organs per line | Balanced `hub_routed` · advancing `fate_signal_tick` |
| Fate orchestrate / warmup / adapt | production kwargs | Iterative affinity & fixed-point seeking | `manifold_orchestrator_smoke` fixed_point |
| FOG + `proactive_idle` | output gate | Speak only when algebra clears; else abstain | U2 MAR/FIR/PPI |
| Non-isolated multimodal session | single Triad wire | Modalities share structure, not late-fuse stores | U1 shared patch ≫ isolated |
| Targeted / reanchor ingest | `context_ingest` | Avoid dump pollution | U3 pollution rescue (+38.7pp) |
| Native open + optional LM MCQ | `decode=hybrid` | Backbone-orthogonal open path | U4 independence |
| `score_structure_alignment` weights | `native_qa.rs` (0.15 / 0.08 / 0.12) | Retrieval **uses** Tile/Stem/organ | Lesion of organ channel hurts that term |
| Fragment-full + compact snapshot | production layout | Structure in fragments; wire stays small | Top-level tiles may be **empty by design** |
| No preset organ labels | `Cell` / Stem merge | Identity = `data_signature` (+ proto energy family) | Same-type CC = maturation continuing |
| L1 zero-weight session | affinity / snapshot | Learn without LM SGD | TPI / transfer |
| L2 Fate pretrain | affinity epochs | Supervised structure tighten | hub_spread trajectory |
| L3 HF bake | TriModal export | Optional weight write | `weight_delta_norm` |
| Append / `session_extend` | longchain | Persist structure without full resend | drift 0% · retention 100% |

---

## 3. Observed property → which design it depends on

| You saw… | It comes from… | Class | Probe |
|----------|----------------|-------|-------|
| Hub 104×4 balanced | `quad_cycle` rotation | **D** | triad highlight |
| Tiles / stems grow on ask | Triad incubation + cycles | **D→E** | triad / organ runtime |
| Snapshot empty, fragments full | compact fragment_full wire | **D** | layout note |
| Stem σ ≫ 1 vs ER | Stem connection dynamics under Triad+Fate | **E** | `natural-nn-topology-v1` |
| Config-null p≈0 · φ supporting | Non-random topology under stricter null | **E** | `natural-nn-causal-v1` |
| Large same-type body (proto) | No preset organs + coalescence rules | **D→E** | organ-runtime |
| Lesion ≻ degree-rewire on organ channel | Scoring weight 0.12 + CC structure | **D + E** | causal-v1 |
| FOG abstain / proactive | Gate thresholds | **D** (P = U2) | Unique Proof U2 |
| +38.7pp pollution rescue | Targeted + transfer protocol | **D** (P = U3) | Unique Proof U3 |
| Bake Δw≈298 | L3 fusion strength | **D** | train smoke L3 |

---

## 4. What is *not* a free-floating “feature”

Clarify confusions that appear in reviews:

1. **“Natural NN”** — not a separate product SKU. It is the **measured graph behavior** of the Stem/Tile substrate we designed to incubate.  
2. **“Organ”** — not a catalog of named organs in code. Design refuses presets; coalescence is **allowed** and then **measured**.  
3. **Empty top-level snapshot** — intentional compact export, not failure to form structure.  
4. **Unique Proofs** — proof that the **design differs from memory APIs**, not that topology σ is higher than Mem0’s text graph.  
5. **Holdout QA 33%** — not a design goal for the substrate wedge; continuity (drift/retention) is the designed longchain claim.

---

## 5. One-page story for investors / DD

**We designed** a structure-first body (Triad + Fate + FOG + hybrid decode + structure-aware retrieve).  
**That design grows** Tile/Stem graphs at runtime (not extract→vector inject).  
**We then measured** that those graphs are non-random (nulls, SWP support) and that organ-channel scoring is causally sensitive to lesion (vs degree-matched rewire).  
**Separately, Unique Proofs** show peers cannot express the same session body (U1–U4).

Order of claims in docs/landing should stay:

1. Design map (this file)  
2. Technical core measurements (topology / organ / causal / train)  
3. Unique Proofs U1–U4  
4. Continuity secondary metrics  

---

## 6. Related documents

| Doc | Role |
|-----|------|
| [TECH_HIGHLIGHTS_TRIAD_TRAIN_20260714.md](./TECH_HIGHLIGHTS_TRIAD_TRAIN_20260714.md) | Measurement core |
| [NATURAL_NN_TOPOLOGY_PROTOCOL_20260714.md](./NATURAL_NN_TOPOLOGY_PROTOCOL_20260714.md) | E: topology |
| [NATURAL_NN_ORGAN_RUNTIME_20260714.md](./NATURAL_NN_ORGAN_RUNTIME_20260714.md) | E: coalescence doctrine |
| [NATURAL_NN_CAUSAL_PROTOCOL_20260714.md](./NATURAL_NN_CAUSAL_PROTOCOL_20260714.md) | E: lesion |
| [TRAINABILITY_PROOF.md](./TRAINABILITY_PROOF.md) | D: L1–L3 |
| [CANONICAL_METRICS_20260714.md](./CANONICAL_METRICS_20260714.md) | Cite table |
| [UNIQUE_VS_MEMORY_APIS.md](./UNIQUE_VS_MEMORY_APIS.md) | P: peer matrix |

*Rogue Intelligence LNC.*
