# Proactive Agency Benchmark

**Protocol:** `proactive-agency-benchmark-v1`  
**Model:** `Qwen2.5-3B-Instruct-Valhalla-TriModal-Fusion-v3-test-s0.10` (baked tri-modal)  
**Updated:** 2026-07-08

---

## 最大亮点 — 自动反应（Automatic Reaction）

ValhallaBase 的 **主动性 AI** 不靠 prompt 触发，而靠 **Fate 结构门控** 在 `proactive_idle` 时自动探测会话、择机插话：

| 能力 | 结果 | 协议 |
|------|------|------|
| **自动反应 PPI-v2** | **0.650** · 68/72 pass | `proactive-fate-v2` Track D |
| **门控安全** | MAR **100%** · FIR **0%** | mismatch 不乱插话 |
| **跨语言 CCU** | **100%** | EN/ZH/FR 同一核心 |
| **Agency Index** | **0.644** | AGENCY_VIABLE |

> **对外引用：** 自动反应能力用 **PPI-v2 + G/H/F 电池**；Agency Index 含 **35% PPI 权重**。  
> 审计：[`PROACTIVE_AGENCY_AUDIT_20260708.md`](./PROACTIVE_AGENCY_AUDIT_20260708.md)

---

## Executive summary

| Run | Rounds | Duration | Agency Index | Verdict | Highlight |
|-----|--------|----------|--------------|---------|-----------|
| **Standard（4 tracks）** | 24 × 3 scenarios | 18s | **0.644** | AGENCY_VIABLE | PPI-v2 **0.65** · CCU **100%** · proactive gate **100%** |
| **Mega（infra only）** | **3000** | **140s** | — | infra | 460 decode · 114 segment resets · **不计 Agency** |

---

## Tracks

| Track | 内容 | 指标 |
|-------|------|------|
| **A** long_dialogue | 高轮次 session → primary → supplement → `proactive_idle` | primary / supplement / proactive gate |
| **B** multimodal_natural | 同题 text / TTS / image / tri-modal | consistency |
| **C** crosslang | EN / ZH / FR 同事实 | CCU |
| **D** proactive_reaction | **自动反应** G/H/F 电池 | **PPI-v2** · UP · MAR · FIR |

**Agency Index** = 0.25×long_pass + 0.20×multimodal + 0.20×crosslang + **0.35×PPI-v2**

---

## Standard run（2026-07-08 重跑）

| Metric | Value |
|--------|-------|
| Agency Index | **0.644** |
| Long-dialogue pass | 33.3% |
| Proactive gate pass | **100%** (spread defer — 门控正常) |
| Multimodal consistency | 0.667 |
| Crosslang CCU | **100%** |
| PPI-v2 (Track D) | **0.650** |

---

## Mega run（3000 轮 · 基础设施 · 不计分）

见 [`proactive_agency_mega.json`](./proactive_agency_mega.json) · 140s · segment reset 内存有界

---

## Run

```bash
# Standard（四轨含自动反应 PPI）
tools/valhalla_inference/run_proactive_agency_benchmark.sh --verbose

# 仅自动反应电池
python3 tools/valhalla_inference/test_proactive_fate_v2.py

# Mega 压测（不计 Agency Index）
tools/valhalla_inference/run_proactive_agency_benchmark.sh --mega --turns 3000 --extend-chunk 8
```

**Artifacts:** `proactive_agency_standard.json` · `proactive_fate_v2_test.json`

---

*Rogue Intelligence LNC.*
