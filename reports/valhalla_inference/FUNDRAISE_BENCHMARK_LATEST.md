# Fundraise Benchmark — Latest Pointer

**Date:** 2026-07-14  
**Suites:** evidence `--full` (2026-07-13) · **unique-proof `--fast` DONE** (2026-07-14)

## Read first

1. [`UNIQUE_VS_MEMORY_APIS.md`](../../docs/valhalla/UNIQUE_VS_MEMORY_APIS.md) — what peers lack
2. [`UNIQUE_PROOF_SUITE.md`](./UNIQUE_PROOF_SUITE.md) — U1–U5 **5/5 pass · 5/5 citeable**
3. [`EVIDENCE_SUITE_SUMMARY_20260713.md`](./EVIDENCE_SUITE_SUMMARY_20260713.md) — industry same-Q
4. [`legitimacy_audit.json`](./legitimacy_audit.json) — **11/11 citeable**
5. [`COMPETITOR_COMPARE_RESULTS.md`](./COMPETITOR_COMPARE_RESULTS.md)
6. [`FUNDRAISE_ABLATION_MATRIX.md`](./FUNDRAISE_ABLATION_MATRIX.md)

## Unique-proof headline (lead with these)

| ID | Verdict | Number |
|----|---------|--------|
| U1 Non-iso MM | NON_ISOLATED_BETTER | shared 0.9999 vs iso 0.968 · wins 12/12 |
| U2 Proactive FOG | PROACTIVE_ABSTAIN_PASS | MAR 1.0 · FIR 0.0 · PPI 0.65 (emit starved on short warmup) |
| U3 Pollution rescue | PASS | **+38.7pp** transfer vs polluted |
| U4 Backbone ortho | VALHALLA_BASE_INDEPENDENT | 3 backbones |
| U5 Cost@floor | COST_RATIO_PASS_ACC_DEVELOPING | prefill **0.291×** · on-topic 60% (fast) |

## Industry same-Q (honest, not the lead)

| Claim surface | Number |
|---------------|--------|
| RAGTruth | 34/40 · 88/100 |
| Needle | 4/4 |
| Competitor LoCoMo | valhalla 0/15 vs full/vanilla 9/15 |
| Ablation RAGTruth | native/hybrid 34–37/40 vs LM/RAG 0/40 |

## Memory leak fix (2026-07-14)

JSON session round-trip no longer carries `fate_rollback_stack` (full Triad snapshots).  
See Rust `skip_serializing` + Python `slim_session_for_wire()`.
