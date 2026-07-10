# ValhallaBase 独立性实验 — 2026-07-10

**协议:** `valhalla-independence-v1`  
**入口:** `tools/valhalla_inference/run_valhalla_independence.py`  
**统一 backbone 接口:** `tools/valhalla_inference/valhalla_backbone.py`

---

## 实验目的（回应 ChatGPT / 普世性质疑）

验证 **ValhallaBase 结构层（Tier B open）与 LM backbone 无关**：

1. 换 RWKV / MoE / Dense，**native** 与 **hybrid open** 答案是否与 reference 完全一致？
2. **trad_lm** 作为对照，允许随 backbone 变化。
3. **会话记忆**（needle）在 hybrid 配置下是否仍与 native reference 一致？
4. **融合 / 主动性 / 长链** 属于产品能力层，在固定 Qwen 栈上 smoke，**不纳入 backbone sweep**。

---

## 模型 panel（已缓存）

| 类 | Preset | HF |
|----|--------|-----|
| RWKV | `rwkv7_0.1b` | `fla-hub/rwkv7-0.1B-g1` |
| MoE | `phi_tiny_moe` | `microsoft/Phi-tiny-MoE-instruct` |
| Dense | `qwen_3b` | `Qwen/Qwen2.5-3B-Instruct` |

---

## 结果摘要

| Tier | 检查 | 结果 |
|------|------|------|
| **A** | open native/hybrid vs reference | **VALHALLA_BASE_INDEPENDENT** — 6/6 PASS（3 backbone × native+hybrid） |
| **A′** | trad 对照 | 3/3 均为 4/4（本批题目上未拉开差距） |
| **B** | needle session hybrid vs native | **MEMORY_INDEPENDENT** — 3/3 PASS（depth 5,10） |
| **C** | core_fusion smoke (2Q, Qwen3B) | **OK** — CORE_FUSION_TIE 2/2 |
| **D** | proactive gates-only | **OK** — GATES_OK 3/3 |
| **E** | longchain unified | **OK** — 缓存 composite **0.661**, theme_drift **0%**（live smoke 120s 超时后回退） |

**decode_path 证据：** 所有 open 题均为 `valhalla_base/native/.../open_retrieval_span`，与 backbone 标签无关。

---

## 内存 / 资源修复（本次）

| 措施 | 作用 |
|------|------|
| `VALHALLA_INGRESS_RESOURCE_PERCENT=70` | 默认 70% ingress |
| `VALHALLA_INGRESS_MAX_ITEMS=48/96` | smoke / 常规上限 |
| `ValhallaLLM.unload()` + `torch.cuda.empty_cache()` | trad 臂逐 backbone 释放 GPU |
| 子进程 `timeout` + 串行 smoke | 防止 longchain 挂死占满内存 |
| longchain `--smoke`（24 行 corpus） | 超轻量 live 尝试 |
| `theme_drift_rate=0` 缓存回退 | 修复 `0.0 or 1.0` bug，超时后用已有报告 |

---

## 是否应加入融合 / 主动性 / 长链？

| 测试 | 纳入 independence 主 verdict？ | 理由 |
|------|-------------------------------|------|
| **open + memory** | ✅ 是 | 直接检验 backbone 无关性 |
| **core_fusion** | ⚠️ 扩展 tier C | 依赖固定 LM + bake，非 backbone sweep |
| **proactive** | ⚠️ 扩展 tier D | 门控/传播依赖 hybrid 栈与 Fate 信号 |
| **longchain** | ⚠️ 扩展 tier E | 架构记忆能力；live 重跑重，用 smoke+cache |

**结论：** 主报告只承诺 **Tier A/B 独立性**；C/D/E 作为 **extended_capabilities** 附录，避免把产品特性与结构独立性混为一谈。

---

## 复现

```bash
# 完整独立性（open + memory + trad 对照）
VALHALLA_INGRESS_RESOURCE_PERCENT=70 VALHALLA_INGRESS_MAX_ITEMS=96 \
  .venv-llm/bin/python3 tools/valhalla_inference/run_valhalla_independence.py \
  --fast --with-trad --with-memory

# 扩展能力 smoke（串行、低内存）
.venv-llm/bin/python3 tools/valhalla_inference/run_valhalla_independence.py \
  --smokes-only --fast --extended

# 带入新模型
.venv-llm/bin/python3 tools/valhalla_inference/run_valhalla_independence.py \
  --hf org/new-model --fast --with-memory
```

**报告 JSON:** `reports/valhalla_inference/valhalla_independence_test.json`

---

## 统一 backbone 接口

```bash
# 列表 / 任意 HF / preset
.venv-llm/bin/python3 tools/valhalla_inference/valhalla_backbone.py --list
.venv-llm/bin/python3 tools/valhalla_inference/valhalla_backbone.py --hf org/model --fast
.venv-llm/bin/python3 tools/valhalla_inference/valhalla_backbone.py --preset rwkv7_0.1b --preset phi_tiny_moe --fast
```

注册表：`tools/standard_benchmarks/backbone_registry.py`
