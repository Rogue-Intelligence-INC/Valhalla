# Evidence Suite Summary — 2026-07-13

**Status:** COMPLETE (`./scripts/run_evidence_suite.sh --full`)  
**Finished:** 2026-07-13T16:04:09+08:00  
**Git at pack:** `c314282a` (+ docs/results commit after this summary)  
**Pack:** `reports/valhalla_inference/evidence_evidence_20260713/` (+ `.zip`)  
**Log:** `reports/valhalla_inference/evidence_suite_run.log`

**Calibration LM:** `Qwen/Qwen2.5-3B-Instruct` (comparability only)  
**Product default LM:** `Qwen/Qwen2.5-0.5B-Instruct`  
**Legitimacy:** **11/11 citeable** · FRAMES wiki bundle **viable=True**

---

## Verdict (one paragraph)

Evidence suite finished cleanly. Industry pillars remain strong on faithfulness / needle (RAGTruth 34/40 · 88/100, Needle 4/4). Same-backbone competitor compare shows Valhalla **tied on MultiHop (6/10)** and **far cheaper on tokens**, but **behind full-context / vanilla RAG on LoCoMo (0/15 vs 9/15)** and slightly behind on LongBench (1/10 vs 2/10). Ablation confirms the product wedge is **faithfulness under structure (native/hybrid 34–37/40 RAGTruth)** while session open-QA on LoCoMo is still the honesty gap — not a SOTA claim.

---

## 1. Legitimacy

| Report | Tier | Score | Cite |
|--------|------|-------|------|
| ragtruth / ragtruth_full | A | 34/40 · 88/100 | Yes |
| needle_session | A | 4/4 | Yes |
| locomo / multihop / ruler / lara | A | 17/30 · 15/25 · 15/20 · 11/20 | Yes |
| longbench / holdout / independence | A | 4/20 · 10/15 · 3/4 | Yes |
| **FRAMES** | **B** | **0/5 smoke** (wiki enriched via parasail) | Yes (label `--fast`; full n≥10 preferred) |

FRAMES is no longer Tier-C invalid: bundle has 25/25 Wikipedia extracts (`./scripts/prefetch_frames_wiki.sh`).

---

## 2. Competitor compare (moderate · same questions)

| Bench | full_context | vanilla_rag | valhalla_base | mem0 (local ST) |
|-------|--------------|-------------|---------------|-----------------|
| LoCoMo-15 | **9/15 (60%)** | **9/15** | 0/15 (0%) | 5/15 (33%) |
| MultiHop-10 | 6/10 (60%) | 6/10 | **6/10** | 6/10 |
| LongBench-10 | **2/10 (20%)** | **2/10** | 1/10 (10%) | 2/10 |

**Token note (from run log):** LoCoMo tok/q ≈ full/vanilla **17386** · valhalla **97** · mem0 **420**.  
MultiHop tok/q ≈ 303 vs valhalla **244**. LongBench ≈ 3044 vs valhalla **568**.

Artifacts: `competitor_compare.json` · `COMPETITOR_COMPARE_RESULTS.md`

---

## 3. Fundraise ablation (7 arms · moderate)

| Arm | LoCoMo | MultiHop | RAGTruth | LongBench |
|-----|--------|----------|----------|-----------|
| LM full-context | 9/15 | 8/15 | 0/40 | 2/10 |
| Vanilla RAG | 9/15 | 8/15 | 0/40 | 2/10 |
| Valhalla lm_patch | 1/15 | 8/15 | 4/40 | 1/10 |
| Valhalla native | 0/15 | 9/15 | **34/40** | **3/10** |
| Valhalla hybrid (full) | 0/15 | 9/15 | **34/40** | 1/10 |
| Hybrid − Fate | 0/15 | 9/15 | 34/40 | 1/10 |
| Hybrid − Triad (tile) | 0/15 | 9/15 | **37/40** | 1/10 |

**Component notes (recorded):** Fate/Triad lift on LoCoMo ≈ 0pp this slice; structure premium on MultiHop **+6.7pp** vs vanilla RAG; LoCoMo structure premium **−60pp** (honest gap).

Artifacts: `fundraise_ablation_matrix.json` · `FUNDRAISE_ABLATION_MATRIX.md`

---

## 4. Token efficiency (mechanism)

30-turn protocol — Valhalla append vs RAG full resend / history:

| Metric | Valhalla append | RAG full resend | Ratio |
|--------|-----------------|-----------------|-------|
| Prefill tok/turn avg | **37.5** | 128.9 | **0.291×** |
| vs RAG+history | **37.5** | 394.8 | **0.095×** |
| Wire chars ratio vs full | — | — | **0.163×** |

On-topic thread rate in this token harness was **0%** (empty answers) — treat as **cost** evidence only, not accuracy.

Artifact: `token_efficiency_test.json`

---

## 5. Industry calibration (locked prior + FRAMES refresh)

| Tier | Benchmark | Result |
|------|-----------|--------|
| A | RAGTruth-40 / 100 | **34/40 · 88/100** |
| A | Needle | **4/4** |
| A | MCQ @ 3B | **45/46** |
| B | LoCoMo / MultiHop / RULER | **17/30 · 15/25 · 15/20** |
| B | LongBench / LaRA | **4/20** · RAG 11/20 vs LC 13/20 |
| B | FRAMES | **0/5 smoke · wiki OK** (full re-run pending) |
| A | Independence | **VALHALLA_BASE_INDEPENDENT** |

---

## 6. What to cite / what not to cite

**Cite on pitch:**
- RAGTruth faithfulness + Needle
- Legitimacy 11/11 + FRAMES wiki viable
- Token efficiency ratios
- Ablation RAGTruth native/hybrid vs LM/RAG 0/40
- Competitor MultiHop tie + token delta

**Do not cite as SOTA:**
- LoCoMo competitor 0/15 (honesty gap)
- FRAMES 0/5 smoke as full-suite accuracy
- Token harness on-topic 0%

---

## 7. Reproduce

```bash
export RUSTFLAGS="-L /opt/cuda/lib64" LIBRARY_PATH=/opt/cuda/lib64 HF_ENDPOINT=https://hf-mirror.com
./scripts/prefetch_frames_wiki.sh
./scripts/run_evidence_suite.sh --full
.venv-llm/bin/python3 tools/valhalla_inference/audit_benchmark_legitimacy.py
.venv-llm/bin/python3 tools/valhalla_inference/pack_evidence_run.py --tag evidence_20260713 --zip
```

---

*Rogue Intelligence LNC · Evidence suite 2026-07-13*
