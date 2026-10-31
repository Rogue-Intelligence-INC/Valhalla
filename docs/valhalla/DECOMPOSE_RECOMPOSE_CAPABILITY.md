# 拆解重组后的能力测试

**日期：** 2026-10-30  
**范围：** 已有记录的汇总。本文件不新跑测试。  
**和训练分开：** 空 session、`fate_pretrain_holdout`、TPI、层签名上的 `model_fate` 不在这里。那些量的是孵化或从零长结构。下表量的是重组之后答对、召回、拒答。

各行协议不同，百分点不能相加，也不能当成同一套 145 题上的一条曲线。对外引用仍以 [`CANONICAL_METRICS_20260714.md`](./CANONICAL_METRICS_20260714.md) 为准；本文件只把散落的能力测试收到一处。

原始记录在三个本地目录：

- `C:\Ai\rogue-products\Valhalla`
- `C:\Ai\rogue-products\valhalla-unified-core`
- `C:\Ai\rogue-products\valhalla-transformer`

---

## 结果

| 测的能力 | 重组后的结果 | 记录 |
|---|---|---|
| MCQ | 结构补丁 + 路由 **30/46（65.2%）**，传统 RAG **13/46（28.3%）**，差 **+37.0 pp** | `valhalla-unified-core\docs\EXPERIMENT_RECORD_INFERENCE_PARADIGM.md` §13，协议 `mcq-coverage-v1` |
| 同规模 MCQ | 0.5B hybrid **65%**，对照 **15%**，差 **+50 pp** | `Valhalla\docs\valhalla\DAILY_SUMMARY_20260630.md` §7 |
| 多模态融合 | 12 题 **8/12 → 9/12**；其中 MCQ **2/5 → 3/5（+20 pp）**，翻过来的是 MCQ_BIO_01 | `valhalla-unified-core\docs\VALHALLA_BASE_PLATFORM.md` §4.2，`reports\context_core_fusion.json`，协议 `context-core-fusion-v2` |
| 开放题结构溢价 | 0.5B 相对传统 RAG **+6.5 pp**；定向检索 **31/31** | 同一份日报 §7，`reports\valhalla_inference\model_scaling_v1_test.json` |
| 会话抗污染 | transfer **87.1%**，相对污染 dump **+38.7 pp** | `Valhalla\docs\valhalla\CANONICAL_METRICS_20260714.md`（U3），`valhalla-unified-core\docs\TRAINABILITY_PROOF.md` |
| 记忆 | 语料 ingest **+31.3 pp**（M 档，49 行）；Memory L1 在 M 档 **89.1%**，拒答 **91.7%** | `valhalla-transformer\zh\12_VPB_POTENTIAL_BENCHMARK.md` |
| 一次孵化 | 逐题重载 MCQ **13/46**，一次孵化 Tile **22/46** | `Valhalla\reports\VALHALLA_MCQ_AND_FAIR_BENCHMARK_SUMMARY_20260622.md` |
| 层签名写回 | Tile legacy 在 12 题试点上 **+8.33 pp**，GSM_03 从 `45` 改到 `18` | `valhalla-transformer\zh\06_TF前置与孵化实验总结.md`，run `20260618` 一带，strength 0.08 |
| 行业套件 | RAGTruth **40/40** 与 **100/100**，Needle **4/4**，RULER **20/20**，套件外 **15/15** | `Valhalla\docs\valhalla\DAILY_SUMMARY_20260630.md` §行业；JSON 在 `Valhalla\reports\valhalla_inference\` |

2026-07-14 的规范表把 RAGTruth 记为 **38/40**，全量缓存 **88/100**（`ragtruth-v1`）。日报里的 40/40、100/100 是 2026-06-30 的快照，引用时带上日期。

---

## 原始文件

逐题行不在本页，在这些目录：

- `valhalla-transformer\experiments\`：`hybrid_unified_latest.json`、`fair_benchmark_test_20260620_0859.json`、`tf_front_body_matrix_pilot_*.json`
- `Valhalla\reports\valhalla_inference\`，`valhalla-unified-core\reports\valhalla_inference\` 有一份同样的拷贝
- `Valhalla\reports\fair_benchmark\`
- `valhalla-unified-core\reports\context_core_fusion.json`
- `valhalla-unified-core\fixtures\crossmodal12_suite.json` 是 12 条提示本身，不是分数
