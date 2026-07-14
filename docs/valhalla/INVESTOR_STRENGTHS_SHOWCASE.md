# ValhallaBase · 投资人价值特征详解

**Audience:** 融资 / 尽职调查 / 技术合伙人  
**Date:** 2026-07-14  
**Entity:** Rogue Intelligence LNC · Valhalla  
**Evidence stamp:** Unique Proof Suite **U1–U4 = 4/4 pass · 4/4 citeable** · Live 4-beat showcase **4/4** · Longchain `theme_drift=0%` · `anchor_retention=100%`

> **一句话（EN）:** ValhallaBase is a **structure-trainable memory body** — Triad incubation (Hub / Tile / StemCell / Fate) with Fate-gated agency, grounded abstain, non-isolated multimodal fusion, and backbone-orthogonal decode — **not** another extract→retrieve memory API.  
> **一句话（CN）:** Valhalla 不是又一个记忆 API，而是可孵化的**结构身体**：非隔离多模态、Fate 主动与拒答、零权重污染救援、换骨干解码头、长链主体稳定。

## 图表（Demo 投影）

重新生成：`python3 scripts/generate_investor_figures.py` → `reports/valhalla_inference/figures/`

| 图 | 文件 |
|----|------|
| U1 非隔离融合 | [`investor_u1_shared_patch.png`](../../reports/valhalla_inference/figures/investor_u1_shared_patch.png) |
| U3 污染救援 | [`investor_u3_pollution_rescue.png`](../../reports/valhalla_inference/figures/investor_u3_pollution_rescue.png) |
| U2 + 主体稳定 | [`investor_u2_longchain_stability.png`](../../reports/valhalla_inference/figures/investor_u2_longchain_stability.png) |
| U1–U4 通过板 | [`investor_unique_proof_board.png`](../../reports/valhalla_inference/figures/investor_unique_proof_board.png) |
| Live 四拍 | [`investor_live_four_beats.png`](../../reports/valhalla_inference/figures/investor_live_four_beats.png) |
| 能力对照 | [`investor_capability_contrast.png`](../../reports/valhalla_inference/figures/investor_capability_contrast.png) |
| Longchain QA（诚实） | [`investor_longchain_qa_honest.png`](../../reports/valhalla_inference/figures/investor_longchain_qa_honest.png) |

![U1](../../reports/valhalla_inference/figures/investor_u1_shared_patch.png)

![U3](../../reports/valhalla_inference/figures/investor_u3_pollution_rescue.png)

![U2+retention](../../reports/valhalla_inference/figures/investor_u2_longchain_stability.png)

![U1–U4 board](../../reports/valhalla_inference/figures/investor_unique_proof_board.png)

![Live 4 beats](../../reports/valhalla_inference/figures/investor_live_four_beats.png)

![Capability contrast](../../reports/valhalla_inference/figures/investor_capability_contrast.png)

---

## 0. 怎么讲清楚（叙事纪律）

### 卖什么

| 卖点层 | 投资者应记住的话 |
|--------|------------------|
| **产品形态** | 结构可训练的 **ValhallaBase**；LM 是可换的 decode 头 |
| **差异化** | 同一 session 里长出 Hub/Tile/Stem + Fate 调度，而不是把文本塞进 prompt |
| **可证伪证据** | Unique Proof Suite（U1–U4）+ Live showcase + Longchain drift/retention |

### 不卖什么（诚实边界）

| 不要领先声称 | 原因 |
|--------------|------|
| 「LoCoMo / LongMemEval 全面碾压 Mem0」 | 同题竞赛不是我们优势面；见内部同题边界报告 |
| 「永远更省 token」 | U5 cost@floor 当前未过（on-topic 地板不足时不可单独引） |
| 「3B / 某固定底座即产品」 | 3B 仅 calibration；产品是结构层 |

对标框架文件：[`UNIQUE_VS_MEMORY_APIS.md`](./UNIQUE_VS_MEMORY_APIS.md)

---

## 1. 今日复跑 · 证据摘要（2026-07-14）

### 1.1 Unique Proof Suite（机制差异，对标 Mem0 / Zep / Letta）

| ID | 主张 | Verdict | 关键数字 | 对标缺什么 |
|----|------|---------|----------|------------|
| **U1** | 非隔离多模态 | `NON_ISOLATED_BETTER` | shared_patch **0.9999** vs Isolated **0.9685**（Δ≈0.0315，**12/12** wins） | 对方多为「存成另一条记忆再 prompt 晚融合」 |
| **U2** | Fate 主动 + 拒答闸 | `PROACTIVE_ABSTAIN_PASS` | **MAR = 1.0** · PPI_v2 **0.65** · `abstain_ok` · `gates_ok` | 无代数 emit/abstain；多为 App 定时 tip |
| **U3** | 零权重污染救援 | `POLLUTION_RESCUE_PASS` | transfer **87.1%** vs polluted **48.4%**（**+38.7 pp**） | 无 TPI / transfer-fresh 协议 |
| **U4** | 骨干正交开路径 | `VALHALLA_BASE_INDEPENDENT` | practical panel **independent=true** · 多骨干开路径匹配 | 记忆绑定抽取/对话 LM |

复现：

```bash
./scripts/run_unique_proofs.sh --fast --only U1,U2,U3,U4
# → reports/valhalla_inference/UNIQUE_PROOF_SUITE.md
```

### 1.2 Live 四拍对话 showcase（可在 Demo Day 实操）

| Beat | 观众看到什么 | 结果（本次） |
|------|--------------|--------------|
| 1 低幻觉 | 库内题 grounded · 库外题 FOG `abstain` | **PASS** · fog.decision=`abstain` |
| 2 主体稳定 | 插入干扰后同题引用保持 `[corpus:5]` | **PASS** · continuity=true |
| 3 主动 | `proactive_idle` 闸门运行（可 `defer` / `emit` / `abstain`） | **PASS** · fog_decision=`defer`（闸门工作中） |
| 4 非隔离融合 | text+audio+image · `core_fusion` | **PASS** · **shared_patch = 1.0** · universal_formed=true |

复现：

```bash
./scripts/demo_valhalla_chat.sh          # UI + STRENGTHS HUD
python3 scripts/run_demo_showcase.py     # JSON 证据
# → reports/valhalla_inference/demo_strengths_showcase.json
```

### 1.3 长链主体稳定（同一 persistent session）

| 指标 | 值（`--production` / fragment_full 64L） |
|------|------------------------------------------|
| **theme_drift_rate** | **0%** |
| **anchor_retention_rate** | **100%** |
| pretrain / interleave / holdout | 37.5% / 25% / 33.3%（QA 仍可增强；**不是**今日对外主叙事） |
| composite | ~0.59 |

复现：

```bash
python3 tools/valhalla_inference/test_fate_longchain_unified.py --production
# → reports/valhalla_inference/FATE_LONGCHAIN_UNIFIED.md
```

---

## 2. 六大有价值特征 · 详细解释

### 2.1 结构先行 · LM 仅作解码头（产品根定义）

**问题：** 传统栈把「记忆」和「生成」绑在同一权重矩阵；换领域要么全量微调，要么把 corpus 塞进 context window。

**Valhalla：** 文本/多模态经 grinder → DlFront → **Triad 孵化为 Hub / Tile / StemCell**，Fate 调度 affinity 与路由；Qwen 等仅作 hybrid 备用或 MCQ patch 头。开放题默认 **native 结构检索 + corpus 索引**，答案可审计 `decode_path`。

**投资者翻译：** 你们买的是 **可训练的记忆身体**，不是「又一个 0.5B/3B chat 包装」。

**证据路径：** session snapshot · fragment tracks · `decode_path`（showcase beat 1）。

---

### 2.2 主动回应 · Fate 闸门（不是永远插嘴）

**问题：** 市场「proactive agent」多为定时 tip / 模型总想发言 → 骚扰与幻觉风险。

**Valhalla：** `proactive_idle` 在 session 上自探针；**FOG（Fate Output Gate）** 按代数阈值决定 `emit` / `abstain` / `defer`。跨域错投用 **MAR（Mismatch Abstain Rate）** 约束。

| 指标 | 含义 | 今日 U2 |
|------|------|---------|
| MAR | 错域应拒答 | **100%** |
| PPI_v2 | 主动质量综合 | **0.65** |
| gates_ok | 闸门可运行 | **true** |
| abstain_ok | 拒答通路 | **true** |

**Demo 话术：** 「看侧栏 STRENGTHS HUD 的 `fog=` —— 没把握就不说。」

**对照：** Mem0/Zep = 查询时才抽记忆；Letta 可 tip，但无 E7/Fate 代数 emit/abstain 协议。

---

### 2.3 低幻觉 · Grounded + FOG 拒答

**两层防护：**

1. **结构 grounding：** 答案带 `[corpus:i]` / `[fate:…]` 引用，优先检索孵化结构而非凭空生成。  
2. **FOG abstain：** 证据不足或 margin 不够 → 拒答 / No confident match（showcase 库外题 `fog.decision=abstain`）。

**进阶（产品差异）：污染救援 U3**  
同问题在「定向 transfer」 vs「污染 dump」：准确率 **87.1% vs 48.4%（+38.7pp）** —— **零权重 session learning**，不跑 SGD/LoRA。

**投资者翻译：** 不是靠更大模型「少胡说」，而是 **身体结构 + 闸门 + 干净 ingest**。

---

### 2.4 主体稳定 · theme_drift / retention

**问题：** 长对话插题、干扰 corpus、重复加载后「认错人 / 换引用」。

**Valhalla：** 同一 persistent session · 动态碎片导航 · remesh 后 `overlay_session_continuity` · writeback 保留 hub/algebra/history。

| 指标 | 今日 |
|------|------|
| theme_drift | **0%** |
| anchor_retention | **100%** |
| Live beat 2 | 干扰后 `[corpus:5]` 连续 |

**Demo：** 问主事实 → 插无关 EURUSD → 再问 → HUD `subject=held` / cite 不变。

**注意：** interleave QA **正确率**仍是工程瓶颈（~25%）；对外强调 **引用/主体不漂**，勿夸成「长问答满分」。

---

### 2.5 多模态非隔离融合（不是三条 RAG 再拼 prompt）

**问题：** 竞品把 image/audio 当另一条文本记忆，**晚融合**进 prompt。

**Valhalla：** text / audio_features / image_features → **同一 Triad session**（DlFront ports）→ 可选 `core_fusion` 形成 universal core。

| 模式 | avg shared_patch | 含义 |
|------|------------------|------|
| Isolated | ~0.968 | 分身体 |
| **Non-isolated** | **~0.9999**（Live **1.0**） | 共享 Fate patch |

**Demo：** 开 Core fusion + 填 audio/image → HUD `shared_patch≈1` · `fusion=strong`。

---

### 2.6 骨干正交 · 解码头可换（U4）

**问题：** 记忆系统深度耦合某一 chat checkpoint → 供应商锁定。

**Valhalla：** 结构层稳定；**开路径**上可换 RWKV / Phi-MoE / Dense 等（`VALHALLA_BASE_INDEPENDENT`）。  
**边界：** `lm_patch` MCQ 路径仍可能绑特定 Candle/Qwen —— 对外说 **open/native 正交**，不要说「所有路径完全无关底座」。

---

## 3. 竞争定位一页纸

| 能力 | ValhallaBase | Mem0 | Zep/Graphiti | Letta |
|------|--------------|------|--------------|-------|
| 非隔离多模态共享 body | **是** | 晚融合 | 无 | 无 Triad pulse |
| Fate 主动 + FOG 拒答 | **是** | 无 | App 调度 tip | 可 tip，无代数闸 |
| 零权重污染救援（TPI） | **是** | 搜索仍脏 | 有效期窗（不同问题） | 无 TPI |
| 骨干正交开路径 | **是** | 绑抽取 LM | Server+LM | Runtime+model |
| 结构增量 append | 机制在（U5 地板待稳） | 仍注入检索文本 | Graph→prompt | Tool paging |
| Temporal KG「何时为真」 | 非楔子 | Optional | **核心** | 有限 |

**他们优化的是「更快记住句子」；我们优化的是「长出一个可调度、可闸控、可多模共体的结构」。**

---

## 4. Demo Day · 15 分钟流程（推荐）

| 分 | 动作 | HUD / 证据 |
|----|------|------------|
| 0–2 | 开场一句话 + 「不卖 LoCoMo」 | — |
| 2–5 | `demo_valhalla_chat.sh` Warmup | fragments / fate_tick |
| 5–7 | 库内题 → 库外题 | fog=abstain |
| 7–9 | 干扰话题 → 原题 | cite / subject held |
| 9–11 | Proactive probe | fog emit\|defer\|abstain |
| 11–13 | Multimodal + core_fusion | shared_patch≈1 |
| 13–15 | 投影 U1–U4 表 + longchain drift=0 | UNIQUE_PROOF_SUITE |

脚本与 UI checklist：`scripts/demo_valhalla_chat.sh` 头注释；可视化旁板 Canvas：`valhalla-strengths-showcase.canvas.tsx`。

---

## 5. 风险与路线图（尽调诚实页）

| 项 | 现状 | 计划叙事 |
|----|------|----------|
| Interleave QA 准确率 | ~25%（结构连续强、答题弱） | L2 检索 / fragment split 调参；**不**假装已 SOTA |
| U5 cost@floor | 未过 | 修好 on-topic 地板后再当成本楔子 |
| U2 emit_ok | 本次 fast 偏 abstain/defer | 闸门正确优先于「爱说话」；演示可加 on-topic warmup 抬 emit |
| LoCoMo 同题 | 非优势面 | 坚持 Unique Proof 叙事 |

---

## 6. 复现与产物清单

| 产物 | 路径 |
|------|------|
| Unique Proof 报告 | `reports/valhalla_inference/UNIQUE_PROOF_SUITE.md` |
| Unique Proof JSON | `reports/valhalla_inference/unique_proof_suite.json` |
| Live 四拍 JSON | `reports/valhalla_inference/demo_strengths_showcase.json` |
| Longchain 报告 | `reports/valhalla_inference/FATE_LONGCHAIN_UNIFIED.md` |
| 对标一页 | `docs/valhalla/UNIQUE_VS_MEMORY_APIS.md` |
| 能力目录（中） | `valhalla-unified-core/docs/DISTINCTIVE_CAPABILITIES_CN.md` |
| 本文件 | `docs/valhalla/INVESTOR_STRENGTHS_SHOWCASE.md` |

命令总表：

```bash
./scripts/run_unique_proofs.sh --fast --only U1,U2,U3,U4
python3 scripts/run_demo_showcase.py
python3 tools/valhalla_inference/test_fate_longchain_unified.py --production
./scripts/demo_valhalla_chat.sh
```

---

## 7. 附录 · 关键指标原样摘录

**U1 shared_patch**

- isolated: `0.9684721809432261`
- non_isolated: `0.9999484330112433`
- wins: `12/12`

**U2**

- MAR: `1.0` · PPI_v2: `0.65` · abstain_ok: `true`

**U3**

- transfer_accuracy: `0.8709677419354839`
- polluted_accuracy: `0.4838709677419355`
- lift: `+38.709… pp`

**Live fusion**

- shared_patch: `1.0` · universal_formed: `true` · decode_snap: `context+task_non_isolated`

**Longchain**

- theme_drift_rate: `0.0` · anchor_retention_rate: `1.0`

---

*Rogue Intelligence LNC · Valhalla · Confidential for diligence*
