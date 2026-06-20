# VPB Phase C — Corpus Scale Axis

**Run**: 20260620_1516 · **Phase**: smoke
**Tiers**: {'S': 22, 'M': 49, 'L': 107}

## Fair L0 (frozen test) — persistent by tier

| Tier | acc | corpus_supported | recall@3 | ingest_lift |
|------|-----|------------------|----------|-------------|
| S | 83.3% | 66.7% | 100.0% | — pp |
| M | 83.3% | 71.4% | 100.0% | — pp |
| L | 83.3% | 81.8% | 100.0% | — pp |

**scale_headroom_norm**: 0.803
**memory_horizon_tier** (Recall≥0.85): S

## Memory L1 taxonomy — by tier

### Tier S
- acc: 81.2%
- abstention: 1.0
- evidence_recall: 0.8462
- tasks: `{'single_hop': {'n': 4, 'acc': 0.5}, 'multi_hop': {'n': 3, 'acc': 1.0}, 'temporal': {'n': 2, 'acc': 1.0}, 'knowledge_update': {'n': 2, 'acc': 0.5}, 'abstention': {'n': 3, 'acc': 1.0}, 'adversarial': {'n': 2, 'acc': 1.0}}`

### Tier M
- acc: 93.8%
- abstention: 1.0
- evidence_recall: 0.9231
- tasks: `{'single_hop': {'n': 4, 'acc': 1.0}, 'multi_hop': {'n': 3, 'acc': 1.0}, 'temporal': {'n': 2, 'acc': 1.0}, 'knowledge_update': {'n': 2, 'acc': 0.5}, 'abstention': {'n': 3, 'acc': 1.0}, 'adversarial': {'n': 2, 'acc': 1.0}}`

### Tier L
- acc: 93.8%
- abstention: 1.0
- evidence_recall: 0.8462
- tasks: `{'single_hop': {'n': 4, 'acc': 1.0}, 'multi_hop': {'n': 3, 'acc': 1.0}, 'temporal': {'n': 2, 'acc': 1.0}, 'knowledge_update': {'n': 2, 'acc': 0.5}, 'abstention': {'n': 3, 'acc': 1.0}, 'adversarial': {'n': 2, 'acc': 1.0}}`
