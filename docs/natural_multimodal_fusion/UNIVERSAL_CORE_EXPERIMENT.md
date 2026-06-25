# 通用核心实验 — 同一任务 × 三模态

**协议**: `valhalla-universal-core-v1` · 不隔离: `valhalla-universal-core-v1-non-isolated`

## 问题

同一任务，用 **文字 / TTS / 生图** 完成，问：

1. **是否共用一个核心？** — 三模态 Fate patch 是否对齐
2. **融合是否形成通用核心？** — 融合 patch 是否收敛到质心（centroid）

## 两种布局

| 布局 | 做法 | 测什么 |
|------|------|--------|
| **隔离** `isolated` | 三个独立 Triad session，各跑一种模态 | 跨 session 是否共用核心（~0.85） |
| **不隔离** `non_isolated` | 一个 session 内 text→tts→image 交错 pulse | 并发融合是否更快形成通用核心 |

## 隔离流程

```
同一 task_text
    ├─ task_via_text   → 独立 session → patch_text
    ├─ task_via_tts    → 独立 session → patch_audio
    └─ task_via_image  → 独立 session → patch_image
centroid → tri-modal 融合 → patch_fused
```

## 不隔离流程

```
同一 task_text → 单 Triad session
每轮: pulse(text) → pulse(tts) → pulse(image) → pulse(mixed)
末轮 snapshot 作为 text/tts/image track → 与 final patch 算指标
```

## 对比结果（smoke 4 题）

| 指标 | 隔离 | 不隔离 |
|------|------|--------|
| Avg shared patch | **0.850** | 1.000* |
| Avg universal score | 0.763 | **0.880** |
| 胜场 | 0 | **4** |

\*不隔离 track 为同一 session 顺序快照，天然高度相关；隔离 ~0.85 才是「跨模态独立运行仍对齐」的硬证据。

**结论**：

- **隔离** → 证明同一任务三模态**各自独立完成**仍指向共用 Fate 核心
- **不隔离** → 通用核心分更高（+0.12），单 session 融合更紧
- 两者互补：隔离验证存在性，不隔离优化收敛

## 复现

```bash
# 对比（默认）
python3 tools/fair_benchmark/run_universal_core_experiment.py --layout compare --phase smoke --limit 4

# 仅隔离 / 仅不隔离
python3 tools/fair_benchmark/run_universal_core_experiment.py --layout isolated --phase smoke --limit 4
python3 tools/fair_benchmark/run_universal_core_experiment.py --layout non_isolated --phase smoke --limit 4
```
