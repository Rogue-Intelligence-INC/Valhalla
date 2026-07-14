# Fundraise · DD · BP · Paper — analysis (2026-07-14)

**Purpose:** Align diligence packs and investor narrative with **live** Unique Proofs + longchain + industry honesty.  
**Canonical product line:** ValhallaBase / VUC — not Hybrid-2.0 / Yaksha archive decks unless asked.

---

## 1. Diligence stack (what to hand investors)

| Layer | Path | Status |
|-------|------|--------|
| **Lead memo** | `valhalla-unified-core/docs/INVESTOR_ONE_PAGER.md` | Unique-first · U1–U4 |
| **Deep showcase** | `docs/valhalla/INVESTOR_STRENGTHS_SHOWCASE.md` | Charts + live 4-beat |
| **Positioning** | `VALHALLA_BASE_PRODUCT_POSITIONING.md` + `MARKET_POSITIONING_20260714.md` | Category map |
| **Engineering DD** | `EN_ENGINEERING_DUE_DILIGENCE.md` | **Supplement §A below**; body still 2026-06-26 |
| **Business plan** | `EN_ENGINEERING_BUSINESS_PLAN.md` | **Supplement §B**; reframe away from “memory platform” alone |
| **Paper** | `papers/ValhallaBase_Distinctive_Inference_Paradigm.{tex,pdf}` | Distinctive inference paradigm — keep; do not claim LoCoMo SOTA |
| **Archive** | `docs/hybrid_2.0/*`, `docs/YAKSHA_BP_*`, `pitch/*` | **Do not lead** — prior generation SKUs |

### Auditor one-shot

```bash
./scripts/run_unique_proofs.sh --fast --only U1,U2,U3,U4
python3 tools/valhalla_inference/audit_benchmark_legitimacy.py
# optional: industry --fast · longchain --production
```

---

## 2. What clears DD today (cite)

| Claim | Proof | Risk if mis-cited |
|-------|-------|-------------------|
| Non-isolated multimodal | U1 shared_patch ≈0.9999 | Calling it “better RAG” |
| Proactive + abstain | U2 MAR 1.0 · FIR 0.0 | “Never hallucinates” |
| Pollution rescue | U3 +38.7pp | Claiming forever beats clean RAG |
| Backbone-orthogonal | U4 independent | Claiming lm_patch is universal |
| Continuity after long chain | drift **0%** · retention **100%** | Equating to holdout QA 100% |
| Faithfulness | RAGTruth **38/40** (2026-07-14 fast) | Stale “100/100” repo blurbs |

---

## 3. What fails or weakens DD if led first

| Topic | Live reality | Investor framing |
|-------|--------------|------------------|
| LoCoMo / MultiHop absolute | Tier B / developing | Competitive context, not wedge |
| Longchain holdout QA | **33%** (fragment_full) | Continuity metrics lead; QA is roadmap |
| Interleave QA | **25%** | Same |
| U5 cost@floor | Not lead | “Append cheaper when floor met” only after pass |
| “5/5 Unique Proofs” | Live lead suite is **U1–U4 = 4/4** | Fix all public pages |

Capital comps: Engram’s ~$98M is a **token-economics / learned memory** story — useful TAM signal for “memory infra,” **not** a claim we share their SKU.

---

## 4. DD pack gaps to close (engineering)

1. Refresh `EN_ENGINEERING_DUE_DILIGENCE.md` claims table with Unique Proof protocols (done via §A).  
2. Keep FRAMES offline unless wiki prefetched.  
3. Dual ablatoins: `fragment_full` (substrate) vs `qa_adaptive_minimal` (score ablation) — label clearly.  
4. Remove latex `.aux/.log/.out` from git; ship `.tex` + `.pdf` only.  
5. Repo description: drop “RAGTruth 100/100” if stale.

---

## 5. BP narrative (recommended slide order)

1. **Problem:** Memory APIs store/retrieve text; persistent agents need a **body** + **gates**.  
2. **Product:** ValhallaBase — Triad + Fate; LM optional.  
3. **Proof:** U1–U4 board + live four beats.  
4. **Continuity:** drift 0% / retention 100%.  
5. **Honest boundary:** industry same-Q Tier B · holdout QA Tier B.  
6. **GTM:** licensed on-prem structure runtime · bake optional.  
7. **Ask:** evaluation license + design partner track.

---

## §A — DD supplement (paste into diligence pack)

**Version:** vuc-dd-1.1-supplement · **2026-07-14**

| Claim | Metric | Protocol | Artifact |
|-------|--------|----------|----------|
| Non-isolated multimodal | shared 0.9999 vs iso 0.968 | unique-proof-suite U1 | `UNIQUE_PROOF_SUITE.md` |
| Proactive + abstain | MAR 1.0 · FIR 0.0 · PPI 0.65 | U2 / proactive-agency | unique + proactive reports |
| Pollution rescue | +38.7pp | U3 / TPI | training_potential / unique |
| Backbone orthogonality | VALHALLA_BASE_INDEPENDENT | U4 | `valhalla_independence_test.json` |
| Theme continuity | drift 0% · retention 100% | fate-longchain-unified-v5 | `fate_longchain_unified.json` |
| Faithfulness | 38/40 · 88/100 | ragtruth-v1 | industry reports |
| Legitimacy gate | 13/13 citeable (live audit) | audit_benchmark_legitimacy | run locally |

**Pre-registered non-claims:** LoCoMo SOTA vs Mem0 · LongMemEval win · U5 lead · holdout QA 100% on fragment_full.

---

## §B — BP supplement (replace §2.3 framing)

**Market positioning (2026):** Compete in **structure substrate / persistent agent body**, adjacent to but not substitutable for Mem0/Zep/Letta memory APIs. Capital category overlap with “AI memory infra” (Mem0 YC, Zep Series A, Engram Series A) validates buyer pain; Valhalla differentiation is **incubation + algebraic gates + multimodal shared body**, evidenced by Unique Proofs — not by LoCoMo leaderboard ownership.

---

*Rogue Intelligence LNC.*
