# Valhalla Potential Benchmark (VPB) — Phase A Report

**Run**: 20260620_1300 · **Source**: reports/fair_benchmark/fair_benchmark_test_20260620_0859.json
**Phase**: baseline · **N**: 145 · **Corpus tier**: M (49 lines)

## Executive Summary

| 指标 | 值 |
|------|-----|
| **PFI** | **0.77** (strong_potential) |
| Hybrid acc | 77.2% |
| TF acc | 71.0% |
| Hybrid − TF | +6.2 pp |
| Gates pass | ✓ (platform_plus_wedge_or_hybrid) |

> TF = 成熟静态基线 · Valhalla = 开发中 · 矩阵优于单一 acc

## Potential Matrix (HELM-style)

| Scenario | M1 TaskAcc | M2 Recall@3 | M3 IngestLift | M4 Faith | M5 TokenEst |
|----------|------------|-------------|---------------|----------|-------------|
| S1 memory_platform | 49.0% | 75.0% | 31.25 pp | 0.3474 | 921.8 |
| S2 memory_retrieval | — | 75.0% | — | 0.6 | 3360.0 |
| S3 mcq (V/T) | 45.6% / 37.0% | 84.8% | 32.61 pp | 0.2967 | 922.1 |
| S4 numeric (V/T/P) | 57.9% / 76.3% / 76.3% | — | — | 0.5329 | 920.6 |
| S5 open_retr (M/P/T) | 51.6% / 96.8% / 96.8% | 61.3% | — | 0.3355 | — |
| S6 hybrid / TF | 77.2% / 71.0% | — | — | 0.5431 | 921.1 |

Hybrid source mix: `{'persistent': 68, 'patch': 72, 'none': 5}`

## PFI Components

- recall_norm: 0.8824
- ingest_lift_norm: 1.0
- wedge_score: 0.5
- hybrid − TF: 6.21 pp
- mcq Δ vs TF: 8.69 pp

## Gates

- **G1_recall_at_3_gte_0.70**: True
- **G2_ingest_lift_gte_15pp**: True
- **G3_mcq_wedge_gte_tf_plus_5pp**: True
- **G4_hybrid_gte_tf_minus_5pp**: True
- **G5_scale_headroom**: None
- **G6_abstention**: None
- **open_generation_memory_acc**: 0.0
- **potential_benchmark_pass**: True

**platform_plus_wedge_or_hybrid**

## 解读（成熟 TF vs 开发中 Valhalla）

1. **S1/S2** 检索与 ingest lift 证明记忆平台信号；与 TF 无直接对比。
2. **S3 MCQ 楔子** 是 Valhalla 已胜 mature TF 的域。
3. **S6 Hybrid** 是产品路径上界；若 ≥ TF−5pp，无需等待 Tier B 全替。
4. PFI 中 scale/velocity 为 Phase C 占位（当前 0.5）。

## 下一步

- Phase B: `vpb_memory_suite.json`（LoCoMo/LONGMEMEVAL taxonomy）
- Phase C: S/M/L 语料档 + milestone velocity

*VPB v1.0 · 20260620_1300*