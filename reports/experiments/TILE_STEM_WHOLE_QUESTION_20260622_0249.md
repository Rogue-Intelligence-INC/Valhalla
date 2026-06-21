# Tile / StemCell 整题输入测试

**Time**: 20260622_0249 · **Test**: 145Q (MCQ 46)

## 协议

- **Ingress**: Grinder hash embed，整题文本（题干+选项+指令）一次送入 body ports
- **Session**: Persistent batch — 语料 incubation 一次，46+ 题 sequential QA 不重 ingest 语料
- **Decode**: `mcq_option`（整题 query_sig，选项单独 hash 打分）

## 结果

### 全量 test

| Body | 整题 persistent | Fair isolated baseline | Δ overall | tiles | stem |
|------|-----------------|------------------------|-----------|-------|------|
| **tile** | **60/145 (41.4%)** | 50/145 (34.5%) | +10 (+6.9 pp) | 11 | 0 |
| **stemcell** | **55/145 (37.9%)** | 50/145 (34.5%) | +5 (+3.4 pp) | 0 | 159 |

### MCQ subset

| Body | 整题 persistent MCQ | Fair isolated MCQ | Δ MCQ | decode path |
|------|---------------------|-------------------|-------|-------------|
| **tile** | **22/46 (47.8%)** | 13/46 (28.3%) | +9 (+19.6 pp) | `{'mcq_option': 46}` |
| **stemcell** | **19/46 (41.3%)** | 13/46 (28.3%) | +6 (+13.0 pp) | `{'mcq_option': 46}` |

JSON: `reports/experiments/TILE_STEM_WHOLE_QUESTION_20260622_0249.json`
