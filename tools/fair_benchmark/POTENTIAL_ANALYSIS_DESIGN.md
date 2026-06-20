# Valhalla 潜力分析实验 — 设计说明

**实验 ID**: `potential_analysis_v1`  
**Split**: fair benchmark `test` (145Q) · clean corpus · artifact `fair_triad_c1`

---

## 双轨问题

| 轨道 | 核心问题 | 成功标准 |
|------|----------|----------|
| **A. 记忆模型** | 语料 ingress 后能否 **可靠检索** 并改变 QA 行为？ | recall@3↑、ingest lift 显著、persistent > isolated |
| **B. Transformer 替代** | 在哪些子域可用 Valhalla **代替** TF forward？ | corpus-supported 楔子上 Valhalla ≥ TF |

---

## 切片（Slices）

| Slice | 定义 | 测试什么 |
|-------|------|----------|
| `corpus_supported` | ref 或 ≥50% keywords 出现在 ingest_clean | 记忆平台主战场 |
| `numeric_compute` | score_type=numeric | 规则算式 vs 生成 |
| `mcq_routing` | score_type=mcq | 选项路由 |
| `open_retrieval` | open 且 corpus_supported | 检索型 open |
| `open_generation` | open 且 **不**在语料中 | TF 必赢区 |

---

## 臂（Arms）

**记忆轨**: `baseline_no_corpus`, `triad_c1`, `triad_c1_persistent`  
**替代轨**: `triad_c1_persistent`, `transformer_zero_shot`, `patch_triad_legacy`

---

## 指标

### Memory Potential Index (MPI) ∈ [0,1]

```
MPI = 0.35 × recall@3(corpus_supported)
    + 0.35 × ingest_lift_pp / 25        (cap 25pp)
    + 0.30 × persistent_gain_pp / 15    (cap 15pp)
```

### Transformer Replacement Index (TRI) ∈ [0,1]

```
TRI = Σ_slice w_slice × 1[best_valhalla_acc ≥ TF_acc]
    slices = corpus_supported, mcq_routing, open_retrieval
    w = slice_size / total_wedge_questions
penalty: open_generation gap (TF - memory) reported separately
```

---

## 命令

```bash
python tools/fair_benchmark/run_potential_analysis.py --artifact checkpoints/incubation_artifacts/fair_triad_c1/
python tools/fair_benchmark/run_potential_analysis.py --reuse-json reports/fair_benchmark/fair_benchmark_test_20260620_0859.json
```

*Rogue Intelligence LNC.*
