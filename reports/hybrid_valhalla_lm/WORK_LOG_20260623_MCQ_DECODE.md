# 工作记录 — MCQ Decode 修复与 Corpus 实验 (2026-06-23)

**产品版本**: `hybrid-4.0-rust-pipeline` / `pipeline-v4-decompose`  
**负责人**: Jouly + Cursor Agent  
**分支**: `main`

---

## 1. 目标

在 Hybrid v4 pipeline 基线（Hybrid deploy **78.6%**，MCQ **52.2%**）上：

1. **回滚**无效 MCQ Fate tune / bypass / audit 实验代码  
2. **修复** decode 层（affinity 全平 → 总选 A、`decode_path` 误标）  
3. **改进** memory `decode_mcq` 的 corpus / entailment 权重（用户选择路径二）

---

## 2. 代码变更摘要

### 2.1 回滚（无效测试路径）

| 移除项 | 说明 |
|--------|------|
| `mcq_fate_tune.rs` / `valhalla_mcq_fate_tune` | 多轮 guidance tune、JSON 参数 |
| `--bypass-decode` / `decode_mcq_structure` | 结构旁路测试 |
| `mcq_reference_seed` | 离线 gold peek |
| audit 字段 | `mcq_memory_letter` / `mcq_affinity_letter` / `mcq_merge_rule` |
| whole-question / pairwise 混入 stem-options | `plan_mcq_fate_shards` 等 |

### 2.2 Decode 层修复（保留）

| 模块 | 变更 |
|------|------|
| `mcq_fate.rs` | `option_scores: Vec<(char,f64)>` 每选项 tile energy |
| `pipeline.rs` | `shard_scores_from_mcq_fate`、相对 affinity、spread≥0.012 时 merge |
| `task_decompose.rs` | stem + options + aggregate；affinity 相对均值 |
| `native_qa.rs` | `decode_mcq` 相对打分；`pipeline::merge_mcq_pipeline_decode` |

### 2.3 Corpus / Entailment 改进（路径二）

| 功能 | 说明 |
|------|------|
| `mcq_unique_corpus_lift` | top-6 corpus 行上，仅对**独占关键词证据**的选项加分 |
| `mcq_stem` + entailment | `mcq_line_entails_option` 增加 stem 关键词权重 |
| 环境变量 | `VALHALLA_MCQ_UNIQUE_LIFT`（默认 0.14）、`VALHALLA_MCQ_TOP_CORPUS`（6） |
| Entailment | **默认关闭**；`VALHALLA_MCQ_ENTAILMENT=1` 显式开启 |

---

## 3. Benchmark 结果（145Q, deploy）

### 3.1 基线 vs Decode 修复后

| 指标 | v4 baseline (`20260623`) | post_decode_fix |
|------|--------------------------|-----------------|
| Hybrid | **78.6%** (114/145) | 77.9% (113/145) |
| MCQ | **52.2%** (24/46) | **52.2%** (24/46) |
| Open | 93.4% | 93.4% |
| Numeric | 86.8% | 84.2% |
| Patch LM MCQ | 39.1% | **58.7%** |

Decode 修复主要纠正 `decode_path` 标注；Hybrid MCQ 仍全走 `tile_fate_persistent`（46/46）。

### 3.2 Corpus / Entailment 实验矩阵

| 版本 | Hybrid MCQ | 备注 |
|------|------------|------|
| post_decode_fix | **24/46 = 52.2%** | baseline |
| v2 强 entailment 默认开 | 21/46 = 45.7% | 有害 |
| v3 自适应 entailment | 20/46 = 43.5% | 有害 |
| v4 对比式 corpus | 20/46 = 43.5% | 有害 |
| **v5 unique corpus lift** | **23/46 = 50.0%** | 最接近 baseline，仅多错 `MCQ_F42` |

### 3.3 根因结论

1. **Affinity 全平** → `max_by` 总选 A：已通过 per-option `option_scores` + 相对 affinity 修复  
2. **`decode_path` 被 `po.decode_hint` 覆盖**：已修复，路径与答案一致  
3. **Memory decode 准确率瓶颈**不在路径标注，而在 corpus 信号区分度；独占证据 lift 略逊于 baseline（−1 题）  
4. **Entailment 默认开启**显著伤害 MCQ（−3～4 题），须保持 off 或高阈值自适应  

---

## 4. 产物文件

```
manifestsys/hub-f64/src/
  pipeline.rs, task_decompose.rs, hybrid.rs, patch_lm.rs, ...
  native_qa.rs, mcq_fate.rs

reports/hybrid_valhalla_lm/
  hybrid_v4_test_deploy_20260623.json          # v4 baseline
  hybrid_v4_test_deploy_post_decode_fix.json   # decode 修复
  hybrid_v4_test_deploy_mcq_decode_v5.json     # unique corpus lift
  HYBRID_4.0_PIPELINE_TEST_20260623.md
  WORK_LOG_20260623_MCQ_DECODE.md              # 本文档
```

---

## 5. 复现命令

```bash
cd /home/jouly/AI/Valhalla
RUSTFLAGS='-L /opt/cuda/lib64' cargo build -p hub-f64 --release --bin valhalla_hybrid

./target/release/valhalla_hybrid \
  --phase test --merge-mode deploy \
  --model checkpoints/valhalla_patched/Qwen2.5-0.5B-Instruct_valhalla_s0.08 \
  --out reports/hybrid_valhalla_lm/hybrid_v4_test_deploy.json
```

可选 corpus 调参：

```bash
export VALHALLA_MCQ_UNIQUE_LIFT=0.14
export VALHALLA_MCQ_UNIQUE_LIFT_CAP=0.55
export VALHALLA_MCQ_ENTAILMENT=0   # 默认 off，勿全开
```

---

## 6. 下一步（未做）

- **Hybrid 路由**（路径一）：memory vs patch 分歧时按 affinity spread 选轨  
- `McqPairwiseCompare` decompose A/B  
- lm_head bias 与 `mcq_option_affinity` 更强耦合  

---

*Generated 2026-06-23 — Valhalla Hybrid MCQ decode session.*
