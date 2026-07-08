# Proactive Agency Benchmark — 合法性 / 合理性 / 全面性审计

**Date:** 2026-07-08 · **Protocols:** `proactive-agency-benchmark-v1` + `proactive-fate-v2`

---

## 结论（Executive）

| 维度 | 评级 | 说明 |
|------|------|------|
| **合法性** | ✅ 可引用（附协议边界） | 指标有明确定义；mega 与 agency 分数分离 |
| **合理性** | ⚠️→✅ 已修补 | 原缺 spread warmup → proactive 0%；已对齐 proactive-fate-v2 |
| **全面性** | ✅ 四轨覆盖 | A 长对话 · B 多模态 · C 跨语言 · **D 自动反应 (PPI-v2)** |

**对外引用建议：**
- **自动反应能力** → `proactive-fate-v2` **PPI-v2 0.85+**、门控 G/H/F 电池
- **主动性 AI 综合** → Agency Index（含 35% PPI-v2 权重）
- **长链基础设施** → mega 3000 轮（**不**计入 Agency Index）

---

## Track 审计

### A · long_dialogue（高轮次对话 + 补充 + proactive_idle）

| 项 | 评估 |
|----|------|
| 场景数 | 3 handcraft + LoCoMo 扩展 — **合理** 作 smoke，非穷尽 |
| 评分 | primary keyword ≥34% · supplement rich+grounded — **合法** |
| 原缺陷 | 无 spread-building warmup → `spread=0` 恒失败 |
| **修补** | `spread_warmup_for_proactive()` 3-turn + `score_proactive_pass()` defer 门控 |

### B · multimodal_natural（同题 text/TTS/image/tri-modal）

| 项 | 评估 |
|----|------|
| 样本 | 3 prompts — **偏少**，作一致性演示 |
| 指标 | keyword hits + cross-modality consistency — **合理** |
| 已知 | MM_BIO_01 偶发 0-hit（baked adapter 敏感）|

### C · crosslang（EN/ZH/FR 不切换栈）

| 项 | 评估 |
|----|------|
| 样本 | 6 FACTS × 3 语 — **全面** |
| 结果 | CCU **100%** — **可对外引用** |

### D · proactive_reaction（自动反应 · proactive-fate-v2）

| 项 | 评估 |
|----|------|
| 电池 | **G** 门控 3/3 · **H** handcraft 7/7 · **F** fair holdout 6/6 |
| 指标 | PPI-v2 · UP_on · MAR · FIR · TR — **行业级分解** |
| 负控 | mismatch abstain + false interrupt — **合法** |
| 纳入 | Agency Index **35% 权重**（自动反应为核心差异化）|

### Mega · 3000 轮（基础设施压测）

| 项 | 评估 |
|----|------|
| 目的 | session 分段 · append_only · 内存上界 — **工程验证** |
| **禁止** | 不得作为 Agency Index 或 proactive 能力分数 |
| 输出 | 独立 `proactive_agency_mega.json` |

---

## Agency Index 公式（2026-07-08 修订）

```
Agency = 0.25·long_pass + 0.20·multimodal_pass + 0.20·crosslang + 0.35·PPI-v2
```

| Verdict | 阈值 |
|---------|------|
| AGENCY_STRONG | ≥ 0.75 |
| AGENCY_VIABLE | ≥ 0.55 |
| AGENCY_WEAK | < 0.55 |

---

## 待补项（非阻塞）

1. multimodal 扩至 ≥10 prompts（fair multimodal slice）
2. long_dialogue 增 DailyDialog / LoCoMo 全对话 proactive 场景
3. mega + targeted reanchor（避免 primary 失忆）

---

*Rogue Intelligence LNC.*
