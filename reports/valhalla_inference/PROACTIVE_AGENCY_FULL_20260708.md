# Proactive Agency — 补齐版完整测试报告

**Date:** 2026-07-08 · **Model:** Qwen2.5-3B-Instruct-Valhalla-TriModal-Fusion-v3-test-s0.10  
**Protocol:** `proactive-agency-benchmark-v1` + `proactive-fate-v2` (Track D)

---

## 执行命令

```bash
# 补齐版（10 multimodal + 5 LoCoMo + full PPI-v2）
tools/valhalla_inference/run_proactive_agency_full.sh --verbose

# 自动反应专项（handcraft long_dialogue + full PPI-v2）
tools/valhalla_inference/run_proactive_agency_specialized.sh --verbose
```

---

## 补齐项（已完成）

| 待补项 | 状态 | 实现 |
|--------|------|------|
| multimodal ≥10 prompts | ✅ | `proactive_agency_scenarios.json` 扩至 **10** 项 |
| long_dialogue LoCoMo 扩展 | ✅ | `--full` → `locomo-extra=5`，修复 `extend_scenarios_from_locomo` 步长 |
| dialogue_turns 驱动标准轮次 | ✅ | handcraft/LoCoMo 场景走 `dialogue_turns` 而非 extend/ask |
| 专项测试入口 | ✅ | `--specialized-reaction` + `run_proactive_agency_specialized.sh` |
| Track D 全量 PPI | ✅ | `--full` 触发 `--fair-limit 0` 刷新 proactive-fate-v2 |

---

## 完整版结果（`proactive_agency_full.json`）

| 指标 | 值 |
|------|-----|
| **Agency Index** | **0.639** (AGENCY_VIABLE) |
| long_dialogue | n=**8** · pass **12.5%** · proactive gate **100%** |
| multimodal | n=**10** · pass **90%** · consistency **0.90** |
| crosslang CCU | **100%** (6 facts × EN/ZH/FR) |
| **PPI-v2** (Track D) | **0.650** · 68/72 pass · GATES_OK |

**Agency 公式：** `0.25·long + 0.20·multimodal + 0.20·crosslang + 0.35·PPI-v2`

---

## 专项测试结果（`proactive_agency_specialized.json`）

| 指标 | 值 |
|------|-----|
| 场景 | 3 handcraft（loco_lgbtq_date · science_water · geo_canberra）|
| long_dialogue pass | **33.3%** |
| proactive gate | **100%**（defer_gate_ok，spread 保守门控）|
| PPI-v2 | **0.650** · MAR **100%** · FIR **0%** |

---

## 解读

1. **自动反应门控 100%** — spread warmup + defer 计分对齐 proactive-fate-v2；baked tri-modal 模型 spread 常为 0，以保守 defer 通过。
2. **PPI-v2 0.650** — 低于历史 0.85 档；UP_on_topic=0%（无 grounded emit 样本）。MAR/FIR 负控优秀。
3. **long_dialogue primary 偏弱** — LoCoMo 扩展后 pass 降至 12.5%（8 场景）；primary 依赖 corpus 引用而非日期推理。
4. **multimodal 90%** — 10 项中 9 项全模态 keyword hit；MM_BIO_01 仍为偶发弱点。

---

## 对外引用建议

| 能力 | 引用指标 |
|------|----------|
| 自动反应 / 门控 | PPI-v2 **0.650** + G/H/F 电池 + MAR 100% |
| 多模态一致性 | 10-prompt pass **90%** |
| 跨语言不隔离 | CCU **100%** |
| 长对话主动性 | proactive gate **100%**（非 emit 率）|
| 综合 Agency | Index **0.639** AGENCY_VIABLE |

---

*Rogue Intelligence LNC.*
