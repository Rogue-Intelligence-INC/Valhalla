# Hybrid Valhalla 2.0 — Investor Pitch

**Company**: Rogue Intelligence LNC.  
**Product**: Hybrid Valhalla **hybrid-2.0** (engine: **hybrid-4.0-rust-pipeline**)  
**Date**: 2026-06-23 · **Confidential**

---

## One-liner

**Hybrid Valhalla is a deployable Q&A product that routes each question to the best path—native structure memory or a small patched language model—delivering 80% accuracy on a decontaminated 145-question benchmark without relying on a single black-box LLM.**

---

## The problem

Enterprises building **vertical AI** face a false choice:

1. **Closed LLM APIs** — strong on open text, but API tax, data egress, and no auditable routing.  
2. **Self-hosted Transformers only** — good generation, weak on **persistent domain structure** and **offline audit**.  
3. **RAG / fine-tune loops** — expensive retraining; structure does not accumulate as a first-class artifact.

For regulated and industrial buyers, the real question is not “Can you beat GPT-4 on MMLU?” but:

> *Can you answer **our** questions **on our hardware**, with **traceable routing**, and **improve vertically without full retraining**?*

---

## Our solution: Hybrid Valhalla

One product router. All question types. No user-facing “mode switch.”

| Question type | Hybrid route | Test accuracy |
|---------------|--------------|---------------|
| **Open / generation** | Structure patch + Qwen generate | **93.44% (57/61)** |
| **Multiple choice (MCQ)** | Structure memory **or** patch LM (deploy router) | **60.87% (28/46)** |
| **Numeric** | max(structure memory, patch LM) | **84.21% (32/38)** |
| **All types (145Q)** | Unified router | **80.69% (117/145)** |

**Companion model**: Qwen2.5-0.5B-Instruct — deliberately small to prove the **structure layer** carries value, not parameter scale.

---

## Why it matters

### 1. Structure + language beats either alone

| Track (145Q test) | Accuracy |
|-------------------|----------|
| Qwen 0.5B zero-shot | 103/145 (71.03%) |
| Triad memory only | 59/145 (40.69%) |
| Tile+Fate memory only | 59/145 (40.69%) |
| **Hybrid unified** | **117/145 (80.69%)** |

Hybrid is not “another chat wrapper.” It is **auditable routing** over **incubated structure** and **optional LM generation**.

### 2. Honest wedge on MCQ

Structure memory (tile+fate persistent) reaches **23/46 (50.00%)** on MCQ decode alone; **hybrid deploy with structure-guided patch LM** reaches **28/46 (60.87%)** on the same fair test slice — **+8.70 pp** vs memory-first deploy **24/46 (52.17%)** (`hybrid_v4_test_deploy_post_decode_fix.json` → final). The bottleneck remains **option comparison**; we report by type with cited JSON.

### 3. Fair, reproducible benchmark

- **fair-1.2**: evaluation corpus **excludes** reference answers (no leakage).  
- Smoke / val / test splits; JSON artifacts published to [valhalla-transformer](https://github.com/Rogue-Intelligence-INC/valhalla-transformer).  
- Runnable in one command: `run_hybrid_valhalla_lm.py --phase test`.

---

## Product architecture (public)

```
Corpus (domain text, no answers)
        │
        ▼
One-shot incubation → Hub / Tile / StemCell / Fate structure
        │
        ▼
User question ──► Hybrid router (hybrid-2.0)
        │
        ├─ open      → patch LM + generate
        ├─ mcq       → structure memory ∨ patch LM (spread-gated router)
        └─ numeric   → max(native, patch LM)
        │
        ▼
Answer + routing trace (auditable)
```

**Deployable today**: Rust `valhalla_native_qa` binary, Python fair-benchmark harness, API + viz in public repo v2.0.

---

## Business model

| SKU | Buyer | Value |
|-----|-------|-------|
| **Hybrid Runtime License** | Enterprise / vertical SaaS | Owned Q&A stack, no API tax |
| **Incubation Bridge (Tier A)** | ML platform teams | Structure → patch export to customer weights |
| **Native QA (Tier B)** | Regulated on-prem | Retrieval / rule paths without HF forward |
| **Design-partner POC** | Manufacturing, edu, gov | 49–110 line corpus → measurable uplift |

We do **not** sell “beat GPT-4.” We sell **sovereign, auditable, structure-incubation infrastructure** with a **proven hybrid product score**.

---

## Traction (June 2026)

- **hybrid-4.0-rust-pipeline** shipped: unified router, structure-guided patch LM, MCQ deploy routing (+4 MCQ on fair test).  
- **80.69% / 93.44% / 60.87% / 84.21%** on type-split fair test (117/145 · 57/61 · 28/46 · 32/38).  
- Anchor JSON: `hybrid_v4_mcq_patch_strengthen_final.json` · day summary: `DAY_SUMMARY_20260623.md`.  
- Public repo **valhalla-transformer v2.0**: experiments JSON, `/api/hybrid-unified`, Fair Benchmark viz.  
- Draft paper: `papers/memory_hybrid/Hybrid_Structure_Memory_and_Routing.md` (memory-model framing, no codenames).

---

## Competition

| Approach | Strength | Gap Hybrid fills |
|----------|----------|------------------|
| OpenAI / Anthropic API | General IQ | Sovereignty, routing audit, no API tax |
| LangChain / LlamaIndex | Fast LLM apps | Native structure incubation, not wrapper-only |
| RAG + fine-tune | Familiar | Persistent structure checkpoint; hybrid routing |
| Pure on-prem LLM | Data local | Weak vertical structure accumulation |

**Positioning**: *Observable routing + owned infrastructure + hybrid synergy—not another chat API.*

---

## The ask

**Seed round: $3–5M**

| Use | % |
|-----|---|
| Hybrid product engineering (MCQ comparator, enterprise SDK) | 45% |
| Design-partner deployments (3 vertical POCs) | 25% |
| Fair benchmark expansion + third-party audit | 15% |
| GTM + compliance narrative (sovereignty, audit packs) | 15% |

**18-month milestones**: MCQ → 65%+ on fair test (option-comparator memory); 3 paid POCs; Hybrid SDK + SLA; optional FIPS path via Yaksha partner.

---

## Vision

Transformers solve **how to say it**. Valhalla solves **what structure to route on, whether it is auditable, and whether it improves without full retraining**.

Hybrid Valhalla is the **product proof** that the two layers belong together.

---

**Contact**: Rogue Intelligence LNC. · hybrid-2.0 · fair-1.2 · hub-f64 1.1.0
