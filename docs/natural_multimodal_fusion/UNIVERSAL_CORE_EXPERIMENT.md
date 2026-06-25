# 通用核心实验 — 同一任务 × 三模态隔离

**协议**: `valhalla-universal-core-v1`

## 问题

同一任务，分别用 **文字 / TTS / 生图** 独立完成（非三模型并行喂入），问：

1. **是否共用一个核心？** — 三模态隔离跑 Valhalla 后，Fate patch 是否对齐
2. **融合是否形成通用核心？** — 三模态融合后 patch 是否收敛到隔离 patch 的质心（centroid）

## 流程（每题）

```
同一 task_text
    ├─ task_via_text   → 仅文字 ports → 隔离 Fate → patch_text
    ├─ task_via_tts    → 仅 TTS ports  → 隔离 Fate → patch_audio
    └─ task_via_image  → 仅生图 ports → 隔离 Fate → patch_image

centroid = normalize(mean(patch_text, patch_audio, patch_image))

tri-modal 融合 → patch_fused

指标:
  shared_patch     = avg(cos(text,audio), cos(text,image), cos(audio,image))
  fusion↔centroid  = cos(patch_fused, centroid)
  core_tightening  = fusion↔centroid − shared_patch
```

## Smoke 结果（4 题）

| 指标 | 值 |
|------|-----|
| Avg shared patch | **0.848** |
| 共用核心 rate | **100%**（patch 对齐） |
| 通用核心形成 rate | **50%** |
| Avg fusion↔centroid | **0.863** |

**结论**：同一任务三模态隔离后 Fate patch **高度对齐（~0.85）**，说明共用结构核心；融合后约半数样本进一步收敛到质心，形成更紧的通用核心。

## 复现

```bash
python3 tools/fair_benchmark/run_universal_core_experiment.py --phase smoke --limit 4
python3 tools/fair_benchmark/run_universal_core_experiment.py --phase test --limit 20
```
