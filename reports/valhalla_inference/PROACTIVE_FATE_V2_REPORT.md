# Proactive Fate v2 — Experiment Report

**Protocol:** `proactive-fate-v2` · **Date:** 2026-07-10

## Headline

| Metric | Value |
|--------|-------|
| **PPI-v2** (composite) | **0.350** |
| On-topic unsolicited precision | 0.0% |
| On-topic emit rate | 0.0% |
| Mismatch abstain rate | 0.0% |
| False interrupt rate | 0.0% |
| Trigger rate (eligible) | 0.0% |

## Verdict

- `PROACTIVE_FATE_V2_WEAK`
- `NO_HALLUCINATION_ON_MISMATCH`
- `GATES_OK`

## Batteries

- Gates: 3/3
## Known gaps

- Time gate: Fate `signal_tick` since `fate_baseline_tick` (default min=64), not turn count.
- Fair aligned uses `targeted_context` + 3-turn warmup; conservative defer counts as pass.
- 1 fair mismatch economics emit (FIR 3.1%); exit: PPI-v2 ≥ 0.75 + gates OK + FIR ≤ 5%.

## Reproduce

```bash
RUSTFLAGS="-L /opt/cuda/lib64" cargo build -p hub-f64 --release --bin valhalla_base
python3 tools/valhalla_inference/test_proactive_fate_v2.py
```

JSON: `reports/valhalla_inference/proactive_fate_v2_test.json`

