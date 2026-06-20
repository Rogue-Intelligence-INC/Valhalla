# Valhalla 潜力分析实验报告

**Run**: 20260620_0906 · **Split**: test · **N**: 145
**Artifact**: 3beed48570c4

## A. 记忆模型潜力 (MPI)

| 指标 | 值 |
|------|-----|
| **Memory Potential Index** | **0.76** (strong_memory_platform_signal) |
| recall@3 (corpus_supported) | 75.00% |
| ingest lift (corpus_supported) | +31.25 pp |
| persistent vs isolated | +7.29 pp |

## B. Transformer 替代潜力 (TRI)

| 指标 | 值 |
|------|-----|
| **Replacement Index** | **0.27** (not_a_transformer_replacement) |
| open_generation gap (TF − memory) | +90.00 pp |

### 楔子切片 vs Transformer

| Slice | N | Valhalla | TF | Δ pp | 胜 |
|-------|---|----------|-----|------|-----|
| corpus_supported | 96 | 49.0% | 63.5% | -14.6 | ✗ |
| mcq_routing | 46 | 45.6% | 37.0% | +8.7 | ✓ |
| open_retrieval | 31 | 51.6% | 96.8% | -45.2 | ✗ |

## 切片明细（persistent memory）

- **corpus_supported**: 49.0% (47/96)
- **numeric_compute**: 57.9% (22/38)
- **mcq_routing**: 45.6% (21/46)
- **open_retrieval**: 51.6% (16/31)
- **open_generation**: 0.0% (0/30)

## Patch triad vs Memory vs TF（楔子切片）

- **corpus_supported**: memory 49.0% · patch 58.3% · TF 63.5%
- **mcq_routing**: memory 45.6% · patch 26.1% · TF 37.0%
- **open_retrieval**: memory 51.6% · patch 96.8% · TF 96.8%

## 结论

**记忆模型**：MPI=0.76（strong_memory_platform_signal）— clean 语料 ingest 在 corpus-supported 子集上有 +31.2pp  lift，持久 session 再 +7.3pp。 **TF 替代**：TRI=0.27（not_a_transformer_replacement）— 楔子域可部分替代；open_generation 仍落后 TF 90.0pp，需 hybrid。
