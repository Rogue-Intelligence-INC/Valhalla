# Fundraise Benchmark — Latest Pointer

**Date:** 2026-07-14  
**Canonical numbers:** [`../../docs/valhalla/CANONICAL_METRICS_20260714.md`](../../docs/valhalla/CANONICAL_METRICS_20260714.md)

## Read first

1. [`CANONICAL_METRICS_20260714.md`](../../docs/valhalla/CANONICAL_METRICS_20260714.md) — **single source of truth**
2. [`UNIQUE_VS_MEMORY_APIS.md`](../../docs/valhalla/UNIQUE_VS_MEMORY_APIS.md) — what peers lack
3. [`UNIQUE_PROOF_SUITE.md`](./UNIQUE_PROOF_SUITE.md) — U1–U4 **4/4 pass · 4/4 citeable**
4. [`FATE_LONGCHAIN_UNIFIED.md`](./FATE_LONGCHAIN_UNIFIED.md) — drift **0%** · retention **100%**
5. [`EVIDENCE_SUITE_SUMMARY_20260713.md`](./EVIDENCE_SUITE_SUMMARY_20260713.md) — industry same-Q
6. [`legitimacy_audit.json`](./legitimacy_audit.json)
7. [`COMPETITOR_COMPARE_RESULTS.md`](./COMPETITOR_COMPARE_RESULTS.md)

## Unique-proof headline (lead)

| ID | Verdict | Number |
|----|---------|--------|
| U1 Non-iso MM | NON_ISOLATED_BETTER | shared 0.9999 vs iso 0.968 · wins 12/12 |
| U2 Proactive FOG | PROACTIVE_ABSTAIN_PASS | MAR 1.0 · FIR 0.0 · PPI 0.65 |
| U3 Pollution rescue | PASS | **+38.7pp** transfer vs polluted |
| U4 Backbone ortho | VALHALLA_BASE_INDEPENDENT | practical panel |
| U5 Cost@floor | **Secondary / developing** | do not lead |

## Continuity + faithfulness

| Claim surface | Number |
|---------------|--------|
| theme_drift / retention | **0% / 100%** (`fragment_full`) |
| holdout QA / interleave | **33% / 25%** (Tier B) |
| RAGTruth | **38/40 · 88/100** |
| Competitor LoCoMo | Valhalla **0/15** vs full/vanilla **9/15** (honest gap) |

## Superseded — do not cite as current

- Unique Proofs “**5/5**”
- RAGTruth “**100/100**” as live headline
- Longchain composite **0.95** / holdout **100%** without labeling **v1 minimal historical**

## Memory leak fix (2026-07-14)

JSON session round-trip no longer carries `fate_rollback_stack` (full Triad snapshots).  
See Rust `skip_serializing` + Python `slim_session_for_wire()`.
