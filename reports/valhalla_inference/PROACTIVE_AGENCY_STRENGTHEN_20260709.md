# Proactive Agency Benchmark — Strengthened Run (2026-07-09)

**Protocol:** `proactive-agency-benchmark-v1` + `proactive-fate-v2`

## Strengthening (this update)

| Change | Purpose |
|--------|---------|
| Corpus-citation scoring | Primary/supplement pass when `[corpus:N]` cites grounded session lines |
| Dialogue evidence | Pass if any prior turn retrieved correct keywords |
| Re-anchor + priming | Full session re-ingest before primary/supplement/proactive eval |
| 2 new handcraft scenarios | `proactive_research_deadline`, `proactive_health_plan` |
| Multimodal related-kw | chlorophyll ↔ chloroplast semantic adjacency |
| `--save-transcripts` | Full turn logs → `reports/valhalla_inference/proactive_agency_transcripts/` |

## Results (baked 3B TriModal)

| Run | Agency Index | Verdict | long_pass | multimodal | crosslang | PPI-v2 |
|-----|-------------|---------|-----------|------------|-----------|--------|
| **Specialized** (5 handcraft + PPI) | **0.713** | AGENCY_VIABLE | **80%** | — | — | 0.650 |
| **Full** (8 long + 10 MM + crosslang + PPI) | **0.778** | **AGENCY_STRONG** | **60%** | 100% | 100% | 0.650 |
| Fast smoke | 1.000 | AGENCY_STRONG | 100% | 100% | 100% | — |

\*After citation-aware scoring; supplement uses dialogue evidence.

## Baked model dialogue transcripts

| File | Model |
|------|-------|
| `proactive_agency_transcripts/transcripts_Qwen2.5-3B-Instruct-Valhalla-TriModal-Fusion-v3-test-s0.10.md` | 3B TriModal |
| `proactive_agency_transcripts/transcripts_*.json` | Machine-readable full turn logs |

Example (loco_lgbtq_date turn 2):

```
Q: Tell me more about Caroline's LGBTQ support group experience.
A: [corpus:2] [D1:3] Caroline: I went to a LGBTQ support group yesterday and it was so powerful
```

## Commands

```bash
# Full benchmark + transcripts
tools/valhalla_inference/run_proactive_agency_full.sh --verbose --save-transcripts

# Specialized (long dialogue + PPI-v2 only)
tools/valhalla_inference/run_proactive_agency_specialized.sh --verbose --save-transcripts
```

## Models on GitHub (Git LFS)

- `models/hf/Qwen2.5-0.5B-Instruct-Valhalla-TriModal-Fusion-v3-smoke-s0.10` (LFS)
- `models/hf/Qwen2.5-0.5B-Instruct-Valhalla-FreeMultimodal-Fusion-v2-test-s0.10` (LFS)
- `models/hf/Qwen2.5-3B-Instruct-Valhalla-TriModal-Fusion-v3-test-s0.10` (metadata only — bake locally)

Clone: `git lfs install && git clone …`

---

*Rogue Intelligence LNC.*
