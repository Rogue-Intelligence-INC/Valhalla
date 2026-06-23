# Hybrid Valhalla 2.0 — Pitch Deck (Slide Outline)

**Format**: 14 slides · ~10 min · Export to Google Slides / Keynote / PDF  
**Version**: hybrid-2.0 · **Date**: 2026-06-23 · Rogue Intelligence LNC.

> Each slide: **Title · Bullets · Speaker notes · Visual hint**

---

## Slide 1 — Title

**Hybrid Valhalla 2.0**  
*Structure-incubated Q&A that beats a 0.5B LLM alone — auditable, on-prem, hybrid-routed.*

- Rogue Intelligence LNC.
- hybrid-2.0 · fair-1.2 · hub-f64 1.1.0
- June 2026

**Notes**: Open with **80.69% (117/145)** on fair test — hybrid deploy, cited JSON.

**Visual**: Logo + metric badge `80.69% · 117/145`

---

## Slide 2 — Problem

**Vertical AI is stuck between API tax and black-box LLMs**

- Closed APIs: strong generation, **no sovereignty**, recurring inference cost
- Self-hosted LLM only: **no persistent structure**, weak audit trail
- RAG/fine-tune: expensive; structure is not a **first-class asset**
- Buyers ask: *local? auditable? improves without full retrain?*

**Notes**: Do not attack GPT-4 IQ — attack **procurement gaps**.

**Visual**: Three-column pain icons (cloud $, black box, GPU fire)

---

## Slide 3 — Insight

**Two layers, one product**

| Layer | Role |
|-------|------|
| **Valhalla structure** | Corpus incubation, routing, native memory |
| **Transformer (small LM)** | Open generation, numeric fallback |

Neither alone wins the fair benchmark. **Hybrid router picks the best path per question type.**

**Visual**: Stack diagram (structure on top, LM below, router in middle)

---

## Slide 4 — Product demo frame

**One router · All question types · No user mode switch**

```
open → patch LM + generate
mcq  → best(Tile+Fate, Triad memory)
numeric → max(all three)
```

Live: `valhalla-transformer` viz · `/api/hybrid-unified/latest`

**Notes**: Offer 60-second live API pull if room has network.

**Visual**: Screenshot of Fair Benchmark viz tab

---

## Slide 5 — Results (headline)

**Fair Benchmark test · 145 questions · decontaminated corpus**

| | Hybrid deploy | Qwen 0.5B alone |
|---|--------|-----------------|
| **Total** | **80.69% (117/145)** | 71.03% (103/145) |
| open | **93.44% (57/61)** | 93.44% (57/61) |
| mcq | **60.87% (28/46)** | 36.96% (17/46) |
| numeric | **84.21% (32/38)** | 76.32% (29/38) |

*Source: `hybrid_v4_mcq_patch_strengthen_final.json` · `fair_benchmark_test_20260620_0859.json`*

**Notes**: Always show **by type** — MCQ is improving, not SOTA.

**Visual**: Bar chart, four metrics

---

## Slide 6 — Why Hybrid beats components

**Synergy is measured, not claimed**

| Track | Acc (145Q test) |
|-------|-----------------|
| Triad memory only | 40.69% (59/145) |
| Tile+Fate memory only | 40.69% (59/145) |
| Patch LM track (final) | 80.69% (117/145) |
| **Hybrid deploy** | **80.69% (117/145)** |

**Visual**: Component vs product comparison

---

## Slide 7 — MCQ story (honest wedge)

**Native structure beats 0.5B on multiple choice**

- 46 MCQ · same fair test split
- Qwen zero-shot: **17/46 (37%)**
- Tile one-shot whole-question: **22/46 (48%)**
- Tile + MCQ Fate cycles: **24/46 (52%)**
- Hybrid MCQ route: **~25/46 (54%)**

Bottleneck: **option comparator**, not ingestion.

**Visual**: Funnel: corpus → incubation → score A/B/C/D

---

## Slide 8 — Architecture

**Deployable stack today**

```
Corpus → one-shot incubation (Hub/Tile/Stem/Fate)
      → Hybrid router (hybrid_valhalla_lm.py)
      → valhalla_native_qa (Rust) + optional Qwen patch
```

Artifacts: `.hub` checkpoints · JSON experiment logs · routing trace per answer

**Visual**: Architecture block diagram (from whitepaper §3)

---

## Slide 9 — Fair benchmark & trust

**fair-1.2 — investor-grade reproducibility**

- Corpus **excludes** gold answers
- smoke / val / test splits
- Public JSON + API on GitHub (`valhalla-transformer` v2.0)
- One-command reproduce: `run_hybrid_valhalla_lm.py --phase test`

**Notes**: This is our answer to "demo ≠ product" in 2026.

**Visual**: GitHub repo screenshot + JSON schema snippet

---

## Slide 10 — Market

**Who pays first**

| Persona | Job-to-be-done | Entry SKU |
|---------|------------------|-----------|
| Sovereign AI lead (gov/mfg) | On-prem Q&A, audit | Hybrid Runtime |
| Vertical SaaS | Domain structure accumulates | Incubation + Hybrid |
| Systems integrator | Modular SKUs | Hub / Tile / Hybrid |

TAM: enterprise AI infra · SAM: regulated on-prem · SOM: 3 POCs → $0.5–2M ARR path

**Visual**: 3 persona cards

---

## Slide 11 — Business model

**Revenue lines**

1. **Hybrid Runtime License** (annual, per node or corpus size)
2. **Design-partner POC** (fixed fee → convert to license)
3. **Tier A patch export** (structure → customer weights)
4. **Optional**: Wave / Val-OS / Yaksha modules (OEM)

**We do not sell**: "beat GPT-4 on MMLU"

**Visual**: Pricing ladder sketch (POC → Team → Enterprise)

---

## Slide 12 — Competition

| | Them | Us |
|---|------|-----|
| OpenAI API | IQ | Sovereignty + routing audit |
| LangChain | Speed | Native incubation runtime |
| RAG stack | Familiar | Structure checkpoint + hybrid |
| Pure on-prem LLM | Local | Vertical structure growth |

**Moat**: Fair benchmark discipline + modular Rust runtime + hybrid product score **already shipped**

**Visual**: 2×2: sovereignty vs structure depth

---

## Slide 13 — Roadmap & ask

**Seed: $3–5M · 18 months**

| Milestone | Target |
|-----------|--------|
| MCQ fair test | 65%+ |
| Paid design partners | 3 |
| Hybrid SDK + SLA | GA |
| Benchmark audit | Third-party sign-off |

Use of funds: 45% product · 25% POCs · 15% benchmark · 15% GTM

**Visual**: Timeline Gantt (Q3'26 – Q2'28)

---

## Slide 14 — Close

**Transformers say it. Valhalla routes it. Hybrid proves both.**

- **80%** fair test · **95%** open · auditable · on-prem
- Runnable artifacts today — not slideware
- Rogue Intelligence LNC.

**Contact** · [Demo: valhalla-transformer API] · hybrid-2.0

**Notes**: End with live viz or pre-recorded 90s screen capture.

**Visual**: Full metrics dashboard + QR to public repo

---

## Appendix slides (optional)

- A1: Reproduction commands & env vars  
- A2: MCQ experiment matrix (what worked / reverted)  
- A3: Module map (Hub, Tile, StemCell, Fate, Wave, Val-OS, Yaksha)  
- A4: Risk register (see Due Diligence pack)  
- A5: Team & advisors (fill at presentation time)
