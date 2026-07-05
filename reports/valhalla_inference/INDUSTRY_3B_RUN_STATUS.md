# 3B 全表 benchmark — 运行中

**启动:** 2026-07-06 06:31  
**主干:** `Qwen/Qwen2.5-3B-Instruct` · GPU · `VALHALLA_USE_CPU=0`  
**命令:** `run_industry_benchmarks.py --all --agi`  
**日志:** `industry_3b_20260706_063100.log`  
**健康检查:** 2026-07-06 — 前次 run 在 LongBench 中断；Needle 已修复 **4/4**；全表已重跑

## 前次中断结果（063100.log）

| 步骤 | 3B 结果 |
|------|---------|
| Needle | 3/4（已修复 → **4/4**） |
| RAGTruth-40 | 40/40 |
| RAGTruth-100 | 99/100 |
| LongBench-v2 | 中断（未完成） |
| 其余 + AGI | 未跑 |

## 进度

| 步骤 | 状态 | 3B 结果 |
|------|------|---------|
| Needle session | ✅ | 3/4 (75%) |
| RAGTruth-40 | ✅ | 40/40 (100%) |
| RAGTruth-100 | ✅ | 99/100 (99%) |
| LongBench-v2 | 🔄 进行中 (~2/20) | — |
| FRAMES | ⏳ | — |
| MultiHop hybrid | ⏳ | — |
| LoCoMo | ⏳ | — |
| RULER | ⏳ | — |
| LaRA | ⏳ | — |
| External holdout | ⏳ | — |
| MCQ ladder | ⏳ | — |
| TPI-v2 | ⏳ | — |
| fair-benchmark | ⏳ | — |
| incubation standard | ⏳ | — |

## 健康检查 (07:00)

- 日志无 `Error` / `Traceback` / `FAIL` / `panic`
- 主进程 PID 64093 存活 (~30min)
- LongBench 子进程 `valhalla_base` CPU ~115%，3B 模型已加载
- 已完成 JSON 校验：`needle` / `ragtruth` / `ragtruth-full` verdict 正常
- LongBench 单题 ~10–15min（长上下文 ingest + 3B lm_patch），属预期慢，非卡死

## 监控

```bash
tail -f reports/valhalla_inference/industry_3b_20260706_063100.log
ps -p $(pgrep -f 'run_industry_benchmarks.py --all --agi' | head -1) -o etime=
nvidia-smi
```

## 完成后

汇总 JSON: `industry_benchmarks_summary.json`  
Markdown: `INDUSTRY_BENCHMARKS_20260630.md`
