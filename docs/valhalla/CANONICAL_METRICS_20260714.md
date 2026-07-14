# Canonical metrics — ValhallaBase (2026-07-14)

**Single source of truth for public / diligence claims.**  
Dated archive reports may show higher historical numbers under *different protocols* — do not remix without labeling.

## Lead (cite)

| Claim | Value | Protocol / artifact |
|-------|--------|---------------------|
| Unique Proofs **U1–U4** | **4/4 pass · 4/4 citeable** | `unique-proof-suite-v1` · `UNIQUE_PROOF_SUITE.md` |
| U1 shared patch | **0.9999** vs iso **0.968** | U1 |
| U2 MAR / FIR / PPI | **1.0 / 0.0 / 0.65** | U2 |
| U3 pollution rescue | **+38.7pp** | U3 |
| U4 orthogonality | `VALHALLA_BASE_INDEPENDENT` | U4 |
| Longchain theme_drift | **0%** | `fate-longchain-unified-v5` · `fragment_full` |
| Longchain retention | **100%** | same |
| RAGTruth (live industry) | **38/40** · full cache **88/100** | ragtruth-v1 |

## Secondary / Tier B (publish, do not lead)

| Claim | Value | Note |
|-------|--------|------|
| Longchain holdout QA | **33%** | fragment_full production |
| Longchain interleave | **25%** | same |
| Longchain composite | **≈0.589** | continuity-weighted |
| U5 cost@floor | **Developing** | prefill ~0.29× exists; accuracy floor not met — **not Unique Proof lead** |
| LoCoMo / MultiHop absolute | Tier B | Not vs-Mem0 SOTA |

## Forbidden public phrasing

- “Unique Proofs **5/5**” (lead suite is U1–U4)
- “RAGTruth **100/100**” as *current* headline (superseded; use 38/40 · 88/100)
- “Longchain composite **0.95** / holdout **100%**” without saying **v1 minimal historical**
- “Beat Mem0 on LoCoMo / LongMemEval”

## Reproduction

```bash
./scripts/run_unique_proofs.sh --fast --only U1,U2,U3,U4
python3 tools/valhalla_inference/test_fate_longchain_unified.py --production
python3 tools/valhalla_inference/audit_benchmark_legitimacy.py
```

*Rogue Intelligence LNC.*
