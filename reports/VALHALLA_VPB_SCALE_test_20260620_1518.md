# VPB Phase C — Corpus Scale Axis

**Run**: 20260620_1518 · **Phase**: test
**Tiers**: {'S': 22, 'M': 49, 'L': 107}

## Fair L0 (frozen test) — persistent by tier

| Tier | acc | corpus_supported | recall@3 | ingest_lift |
|------|-----|------------------|----------|-------------|
| S | 36.5% | 47.7% | 75.0% | 27.91 pp |
| M | 40.7% | 49.0% | 75.0% | 31.25 pp |
| L | 40.7% | 50.5% | 75.0% | 29.13 pp |

**scale_headroom_norm**: 0.5564
**memory_horizon_tier** (Recall≥0.85): None

## Memory L1 taxonomy — by tier

### Tier S
- acc: 64.1%
- abstention: 0.9167
- evidence_recall: 0.7
- tasks: `{'single_hop': {'n': 16, 'acc': 0.6875}, 'multi_hop': {'n': 12, 'acc': 0.5}, 'temporal': {'n': 8, 'acc': 0.25}, 'knowledge_update': {'n': 8, 'acc': 0.875}, 'abstention': {'n': 12, 'acc': 0.9167}, 'adversarial': {'n': 8, 'acc': 0.5}}`

### Tier M
- acc: 89.1%
- abstention: 0.9167
- evidence_recall: 0.9
- tasks: `{'single_hop': {'n': 16, 'acc': 1.0}, 'multi_hop': {'n': 12, 'acc': 1.0}, 'temporal': {'n': 8, 'acc': 0.5}, 'knowledge_update': {'n': 8, 'acc': 0.875}, 'abstention': {'n': 12, 'acc': 0.9167}, 'adversarial': {'n': 8, 'acc': 0.875}}`

### Tier L
- acc: 89.1%
- abstention: 0.9167
- evidence_recall: 0.9
- tasks: `{'single_hop': {'n': 16, 'acc': 1.0}, 'multi_hop': {'n': 12, 'acc': 1.0}, 'temporal': {'n': 8, 'acc': 0.5}, 'knowledge_update': {'n': 8, 'acc': 0.875}, 'abstention': {'n': 12, 'acc': 0.9167}, 'adversarial': {'n': 8, 'acc': 0.875}}`
