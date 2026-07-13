# Unique Proof Suite

**Protocol:** `unique-proof-suite-v1`  
**Generated:** 2026-07-13T20:26:25.982986+00:00  
**Scale:** fast  

> Mechanism differentiation vs Mem0/Zep/Letta — **not** industry LoCoMo SOTA.

## Summary

- Passed: **5/5**
- Citeable: **5/5**

| ID | Claim | Pass | Citeable | Verdict | Peer |
|----|-------|------|----------|---------|------|
| U1 | non_isolated_multimodal | YES | YES | `NON_ISOLATED_BETTER` | `SKIP_NO_SHARED_BODY` |
| U2 | proactive_gate | YES | YES | `PROACTIVE_ABSTAIN_PASS` | `SKIP_NO_FOG` |
| U3 | pollution_rescue_tpi | YES | YES | `POLLUTION_RESCUE_PASS` | `SKIP_NO_TRANSFER_PROTOCOL` |
| U4 | backbone_orthogonality | YES | YES | `VALHALLA_BASE_INDEPENDENT` | `PROMPT_TIED_TO_LM` |
| U5 | cost_at_floor | YES | YES | `COST_RATIO_PASS_ACC_DEVELOPING` | `REINJECT_RETRIEVED_TEXT` |

## Metrics

### U1 — non_isolated_multimodal

```json
{
  "avg_shared_patch_isolated": 0.9684721809432261,
  "avg_shared_patch_non_isolated": 0.9999484330112433,
  "delta": 0.0315,
  "non_isolated_wins": 12,
  "sample_count": 12
}
```

### U2 — proactive_gate

```json
{
  "UP_on_topic": 0.0,
  "MAR": 1.0,
  "FIR": 0.0,
  "PPI_v2": 0.65,
  "gates_ok": true,
  "emit_ok": false,
  "abstain_ok": true
}
```

### U3 — pollution_rescue_tpi

```json
{
  "transfer_vs_polluted_pp": 38.70967741935484,
  "transfer_accuracy": 0.8709677419354839,
  "polluted_accuracy": 0.4838709677419355,
  "reused": true
}
```

### U4 — backbone_orthogonality

```json
{
  "summary": {
    "verdict": "VALHALLA_BASE_INDEPENDENT",
    "backbones_tested": 3,
    "backbones_total": 6,
    "causal_tested": 3,
    "causal_total": 4,
    "appendix_count": 1,
    "reference_accuracy": 1.0,
    "independent": true,
    "panel": "practical"
  },
  "reused": true
}
```

### U5 — cost_at_floor

```json
{
  "on_topic_rate": 0.6,
  "prefill_ratio_vs_rag_full": 0.291,
  "wire_ratio": 0.163,
  "floor_full": false,
  "floor_fast": true
}
```

## Docs

- [`UNIQUE_VS_MEMORY_APIS.md`](../../docs/valhalla/UNIQUE_VS_MEMORY_APIS.md)
- [`UNIQUE_PROOF_MICROBENCH.md`](../../docs/valhalla/UNIQUE_PROOF_MICROBENCH.md)

*Rogue Intelligence LNC*
