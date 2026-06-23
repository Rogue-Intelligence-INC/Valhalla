# Hybrid Valhalla 2.0 — Business Plan

**Company**: Rogue Intelligence LNC.  
**Product**: Hybrid Valhalla **hybrid-2.0**  
**Version**: 2.0 · **Date**: 2026-06-22  
**Type**: Business plan (Hybrid product edition)  
**Use**: Fundraising, enterprise partnership · **Confidential**

---

## Executive summary

Rogue Intelligence ships **Hybrid Valhalla hybrid-2.0** — a deployable Q&A product that routes each question to native structure memory or a patched small language model. On the **fair-1.2** decontaminated benchmark (145 test prompts, Qwen2.5-0.5B companion), the unified product achieves **80.0% total accuracy** with **95.1% open**, **54.4% MCQ**, and **86.8% numeric**.

**We sell**: sovereign, auditable, structure-incubation infrastructure with a **measured hybrid product score** — not "beat GPT-4."

**Fundraising**: Seed **$3–5M** · 18-month path to 3 paid design partners, MCQ 65%+, Hybrid SDK GA.

---

## 1. Market opportunity

### 1.1 Problem

| Pain | Market signal (2025–2026) | Hybrid response |
|------|---------------------------|-----------------|
| API tax & lock-in | Inference unit economics break at scale | On-prem Rust runtime + optional small LM |
| Black-box routing | LangChain hides internal state | Per-answer `hybrid_source` trace |
| Vertical structure | RAG is ephemeral; fine-tune is costly | One-shot incubation → persistent checkpoint |
| Data sovereignty | Regulated on-prem RFPs | No mandatory external API (Tier B paths) |
| Demo ≠ product | 2026 investor bar: runnable artifacts | fair-1.2 JSON + public API v2.0 |

### 1.2 Market sizing

| Layer | Description | Scale |
|-------|-------------|-------|
| **TAM** | Global enterprise AI infra + private MLOps | $100B+ category |
| **SAM** | Regulated + vertical SaaS needing on-prem Q&A with audit | Multi-billion |
| **SOM (24 mo)** | 3 design-partner POCs → Hybrid licenses | **$0.5–2M ARR** path |

### 1.3 Why now

- Capital shifts from "concept AI" to **verifiable infra + unit economics**  
- **Sovereignty + inference cost** are procurement hard requirements  
- Hybrid **2.0 shipped** with public reproducibility — not roadmap slideware  

---

## 2. Product & solution

### 2.1 Hybrid Valhalla 2.0 (flagship SKU)

**One router. All types. June 2026 production definition.**

| Type | Route | Test acc |
|------|-------|----------|
| open | patch LM + generate | 95.1% |
| mcq | best(Tile+Fate, Triad) | 54.4% |
| numeric | max(all three) | 86.8% |
| **Total** | unified | **80.0%** |

### 2.2 Product matrix (modular)

| SKU | Description | Hybrid relation |
|-----|-------------|-----------------|
| **Hybrid Runtime** | Router + native QA + patch bridge | **Flagship** — what we demo |
| **Hub Enterprise** | Quad routing + Fate + dashboard | Structure layer |
| **Tile / StemCell** | Structure growth engines | MCQ / numeric memory bodies |
| **Incubation Bridge (Tier A)** | Structure → patch export | Open + numeric patch path |
| **Native QA (Tier B)** | No HF forward | MCQ / numeric memory path |
| **Wave + Val-OS + Yaksha** | Language, OS, AEAD | Optional sovereign stack |

### 2.3 Value proposition

```
Closed API:     data → [black-box LLM] → answer → API fees forever
Private LLM:    data → [self-hosted LM + RAG] → GPU + retrain loops
Hybrid Valhalla: corpus → [incubate structure] → router picks memory OR patch LM
                 → auditable trace → no full retrain for vertical shift
```

### 2.4 Synergy narrative (validated)

| Track | 145Q test acc | Role |
|-------|---------------|------|
| Qwen 0.5B alone | 71.0% | Language baseline |
| Memory only | 39–41% | Structure baseline |
| **Hybrid unified** | **80.0%** | **Product proof** |

Transformers solve **how to say it**. Valhalla solves **what structure to route on** and **whether it is auditable offline**.

### 2.5 What we do NOT sell

- General benchmark SOTA vs frontier models  
- "Replaced GPT-4" messaging  
- Inflated MCQ claims without type split  

---

## 3. Go-to-market

### 3.1 Target personas

| Persona | US / global title | Job-to-be-done | Entry |
|---------|-------------------|----------------|-------|
| **P1 Sovereign AI** | VP Eng / CISO (gov, mfg, defense supply chain) | On-prem Q&A, routing audit | Hybrid Runtime |
| **P2 Vertical SaaS** | Head of ML / Platform | Domain corpus → measurable uplift | Incubation + Hybrid |
| **P3 Systems integrator** | Solutions architect | Modular SKUs, OEM | Hub / Tile / Hybrid |
| **P4 Lab / university** | Research computing | Runnable stack for grants & curriculum | Val-OS + Hybrid demo |

### 3.2 GTM phases

| Phase | Timeline | Activity | Success metric |
|-------|----------|----------|----------------|
| **Design partner** | Q3–Q4 2026 | 3 POCs (edu, mfg, gov-adjacent) | Signed POC + corpus ≥110 lines |
| **Product GA** | Q1 2027 | Hybrid SDK, SLA, Docker repro | 1st paid license |
| **Channel** | Q2 2027+ | SI OEM, regional sovereign cloud | 3+ ARR customers |

### 3.3 Sales motion

1. **Demo**: valhalla-transformer viz + `/api/hybrid-unified/latest`  
2. **POC**: Customer corpus (50–200 lines) → fair-style eval report  
3. **Convert**: Annual Hybrid Runtime license + optional Yaksha  

---

## 4. Business model & pricing (indicative)

| Tier | Includes | Indicative price |
|------|----------|-------------------|
| **POC** | 8-week eval, fair report, 1 corpus | $40–80K fixed |
| **Team** | Hybrid Runtime, 1 node, email support | $60–120K / year |
| **Enterprise** | Multi-node, SDK, routing audit export, SLA | $150–400K / year |
| **OEM / SI** | Module bundle (Hub+Tile+Hybrid) | Custom |

**Revenue mix target (24 mo)**: 60% license · 30% POC/services · 10% optional modules (Yaksha, Val-OS)

---

## 5. Competition

| Competitor | Category | Their strength | Hybrid wedge |
|------------|----------|----------------|--------------|
| OpenAI / Anthropic | Closed LLM API | General IQ | Sovereignty, no API tax, routing audit |
| AWS Bedrock / Azure OpenAI | Managed AI | SLAs on their models | Customer-owned binaries + checkpoints |
| LangChain / LlamaIndex | Orchestration | Ecosystem | Native incubation runtime, not wrapper-only |
| RAG + fine-tune stack | Vertical adapt | Familiar workflow | Structure checkpoint + hybrid routing |
| Pure on-prem LLM (HF) | Local inference | Data stays local | Weak persistent structure |

**Positioning**: *Observable routing + owned infrastructure + hybrid synergy at 80% fair test.*

---

## 6. Traction & milestones

### 6.1 Shipped (June 2026)

- hybrid-2.0 unified router  
- fair-1.2 benchmark + public valhalla-transformer v2.0  
- MCQ Fate cycles (Tile 24/46)  
- Bilingual investor doc pack  

### 6.2 18-month roadmap

| Quarter | Engineering | Commercial |
|---------|-------------|--------------|
| Q3 2026 | MCQ relative comparator | 3 POC conversations |
| Q4 2026 | Docker repro · fair-2.0 corpus | 1 signed POC |
| Q1 2027 | Hybrid SDK GA | 1 paid license |
| Q2 2027 | MCQ 65%+ fair test | 3 ARR customers |

---

## 7. Financial plan (high level)

### 7.1 Use of funds (Seed $3–5M)

| Category | % | Purpose |
|----------|---|---------|
| Product engineering | 45% | MCQ comparator, SDK, enterprise hardening |
| Design partners | 25% | 3 vertical POC deployments |
| Benchmark & audit | 15% | fair-2.0, third-party sign-off |
| GTM & ops | 15% | Sovereign narrative, compliance packs, hire 2 AE |

### 7.2 Headcount plan (18 mo)

| Role | Count | Timing |
|------|-------|--------|
| Founding eng (Rust + ML systems) | 2–3 | Existing + 1 hire |
| Applied ML / eval | 1 | Q4 2026 |
| Enterprise AE / solutions | 1 | Q1 2027 |
| DevRel / benchmark ops | 1 | Q2 2027 |

### 7.3 Revenue projection (illustrative)

| Year | ARR | Assumption |
|------|-----|------------|
| Y1 (2026–27) | $200–500K | 2–4 POC converts |
| Y2 (2027–28) | $1–2M | 6–12 enterprise seats |
| Y3 (2028–29) | $3–6M | SI channel + OEM |

*Illustrative only — not a forecast guarantee.*

---

## 8. Risks & mitigations

See **Due Diligence pack** §7. Top three:

1. **MCQ plateau** → P0 engineering on relative comparator  
2. **Misread 80% headline** → mandatory by-type reporting in all materials  
3. **Repro friction** → Docker package + audited env spec  

---

## 9. Team & organization

*Founder and core team bios: data room / NDA.*

**Organizational principle**: Modular SKUs (Hub, Tile, Hybrid) allow SI partners to sell subsets without full-stack mandate.

**Advisors** (to be listed): enterprise AI GTM, regulated-industry compliance.

---

## 10. The ask

| Item | Detail |
|------|--------|
| **Round** | Seed |
| **Amount** | **$3–5M** |
| **Instrument** | SAFE or priced equity (negotiable) |
| **Runway** | 18–24 months to ARR proof |
| **Lead use case** | Hybrid Runtime for sovereign / vertical Q&A |

**Contact**: Rogue Intelligence LNC. · hybrid-2.0 · fair-1.2 · hub-f64 1.1.0

---

## Appendix A — Metric reference

| Metric | Value | Source |
|--------|-------|--------|
| Hybrid total | 80.0% | hybrid_unified_test_20260622_0345.json |
| open | 95.1% | same |
| mcq | 54.4% | same |
| numeric | 86.8% | same |
| Qwen 0.5B baseline | 71.0% | fair_benchmark_test_20260620_0859.json |

## Appendix B — Document cross-reference

| Doc | Path |
|-----|------|
| Pitch | `docs/hybrid_2.0/EN_PITCH.md` |
| Pitch deck | `docs/hybrid_2.0/EN_PITCH_DECK.md` |
| Technical whitepaper | `docs/hybrid_2.0/EN_TECHNICAL_WHITEPAPER.md` |
| Due diligence | `docs/hybrid_2.0/EN_DUE_DILIGENCE.md` |

---

*Rogue Intelligence LNC. · Confidential · 2026-06-22*
