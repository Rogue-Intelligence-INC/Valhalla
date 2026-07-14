# Land / corpus opt audit — same logic, legitimate reuse (2026-07-14)

**Question:** Where else can we apply the unified longchain fixes  
(pin-then-cap · content-gate schedule · content RelAddr egress · gold-aligned temporal score)  
**without** breaking reproducibility or silently rewriting claim surfaces?

**SSOT for cited longchain numbers:** [`CANONICAL_METRICS_20260714.md`](./CANONICAL_METRICS_20260714.md)

---

## Shared fix triad (already proven on `fate-longchain-unified-v5`)

| Fix | What it stops | Cite rule |
|-----|---------------|-----------|
| **Pin-then-cap** (`fragment_full_capped`) | Head-truncate dropping late evidence (e.g. D2:8 @~253) | Disclose “pin-evidence corpus” |
| **Content-gate schedule** | Sticky address/Tile peaks on early name lines | Disclose if land path changed |
| **Content RelAddr egress** | `worker.run` schedule overwrite on absolute QA | Holdout / absolute test only — not continuity arms |
| **Gold-aligned temporal score** | Camping “last year” ≠ charity-race date | Disclose scorer tightening |

---

## Applied this pass (same protocol family)

| Path | Change | Repro |
|------|--------|-------|
| `fate_pretrain_holdout` non–low_resource corpus | pin-then-cap instead of `.truncate` | `valhalla_base demo fate-pretrain-holdout` |
| `fate_pretrain_holdout` **test** split | `holdout_isolated()` content land (train stays pretrain flow) | same |
| `fate_parallel_longchain` routing A/B corpus | pin-then-cap per item | `fate-routing-ab` |
| `valhalla_base_eval` LoCoMo adaptive fallback | pin-then-cap | `locomo-v1-rust` |
| `fate_structure_land` | content-gate Tile∩Stem boost by `line_query_score` | any open land fallback |
| Unified / parallel / open `worker.run` | already content-gated (prior pass) | `--production` |

**Reproduce unified spine (must not regress):**

```bash
python3 tools/valhalla_inference/test_fate_longchain_unified.py --production
# expect: drift 0% · retention 100% · holdout 100% · interleave ≥75% · composite ≈0.88
```

---

## Intentionally **not** changed (legitimacy / claim-surface)

| Path | Why leave alone |
|------|-----------------|
| Unified **anchor** `persistent()` | Continuity arms must stay schedule-stable (theme_drift claim) |
| Routing A/B **baseline** = raw `client.run` | Changing to content egress redefines A/B; only with recalibrated report |
| Multihop / Frames / `fate_score_short_answer` | Different published scorers — tighten only with **protocol version bump** + labeled delta |
| LongBench / RAGTruth / intentional short windows | Not full-dialog LoCoMo; head-cap is the claim |
| Proactive agency locomo head-slice | Agency protocol, not absolute LoCoMo QA |
| Parallel **smoke** truncate | Toy corpora; not citeable |

---

## Candidate next (disclose before cite)

1. Multihop → migrate to `score_qa_with_evidence` under `multihop-vN+1`, keep old JSON historical.  
2. Python `test_locomo_v1.py` head-slice → pin-then-cap for parity with rust.  
3. Optional `content_land` flag on routing “targeted” arm for absolute LoCoMo A/B (recalibrate).  

---

## Reasonableness checks

- Optimizations only expand **evidence reach** or **content alignment** — no gold injection into answers.  
- Small-n protocol passes (holdout 3, interleave 4) stay labeled — not Mem0 LoCoMo SOTA.  
- Scorer hardening that **drops** false True is considered legitimate honesty, not a regression.

*Rogue Intelligence LNC.*
