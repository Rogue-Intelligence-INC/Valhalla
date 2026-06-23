# Hybrid Valhalla 2.0 — 尽职调查包（DD）

**文档类型**：投资人 / 伙伴尽职调查  
**版本**：hybrid-2.0 · hybrid-4.0-rust-pipeline · fair-1.2 · hub-f64 1.1.0  
**日期**：2026-06-23  
**出品**：Rogue Intelligence LNC.（游侠智能有限公司）  
**密级**：机密 — 仅限合格投资人与设计伙伴

---

## 1. 目的

本包支持对 Hybrid Valhalla **hybrid-2.0**（2026 年 6 月交付的统一问答产品）进行 **技术与商业尽职调查**，提供 **论断 → 证据 → 风险 → 待办** 映射，便于审计式审阅。

---

## 2. 公司快照

| 字段 | 内容 |
|------|------|
| 法律实体 | Rogue Intelligence LNC.（游侠智能有限公司） |
| 产品 | Hybrid Valhalla hybrid-2.0 |
| 阶段 | 种子 / Pre-seed — 产品可运行，商业化早期 |
| 核心仓库 | Valhalla（内部）· [valhalla-transformer](https://github.com/Rogue-Intelligence-INC/valhalla-transformer)（公开 viz + JSON v2.0） |
| IP 范围 | Hub / Tile / StemCell / Fate 运行时、Wave 语言、Val-OS、Yaksha（模块化） |

*团队名册、 cap table、财务报表：NDA 数据室提供。*

---

## 3. 产品论断验证矩阵

| # | 论断 | 指标 | 来源产物 | 验证动作 | 状态 |
|---|------|------|----------|----------|------|
| C1 | Hybrid 统一总准确率 | **80.69%**（117/145） | `hybrid_v4_mcq_patch_strengthen_final.json` | 重跑 `valhalla_hybrid --phase test --merge-mode deploy` | ✓ |
| C2 | 开放题准确率 | **93.44%**（57/61） | 同上 · `summary.hybrid.by_score_type.open` | 抽查 `patch_lm` 来源行 | ✓ |
| C3 | MCQ 准确率 | **60.87%**（28/46） | 同上 | 46 道 MCQ id 对 gold | ✓ |
| C4 | 数字题准确率 | **84.21%**（32/38） | 同上 | 仅数字子集 | ✓ |
| C5 | MCQ deploy 相对 memory-first 提升 | 24/46（52.17%）→ 28/46（60.87%） | post_decode_fix vs final JSON | deploy 策略对比 | ✓ |
| C6 | 补丁 LM MCQ | 28/46 | 同上 · patch 子集 | 独立 patch 轨道 | ✓ |
| C7 | 语料无答案泄漏 | fair-1.2 设计 | suite + 语料文件 | 确认语料无 gold | ✓ |
| C8 | 每答路由轨迹 | `hybrid_source` 字段 | Hybrid JSON 行 | 抽样 20 行 | ✓ |
| C9 | 公开 API 提供 latest | `/api/hybrid-unified/latest` | valhalla-transformer v2.0 | `curl` + schema | ✓ |
| C10 | 通用 LLM SOTA | — | — | 未宣称 | N/A |

---

## 4. 产物清单

### 4.1 主产物（Hybrid 2.0）

| 产物 | 位置 |
|------|------|
| Hybrid 统一 JSON | `reports/hybrid_valhalla_lm/hybrid_v4_mcq_patch_strengthen_final.json` |
| 日结报告 | `docs/hybrid_2.0/DAY_SUMMARY_20260623.md` |
| MCQ + Fair 总结 v1.3 | `reports/VALHALLA_MCQ_AND_FAIR_BENCHMARK_SUMMARY_20260622.md` |
| 路由源码 | `tools/fair_benchmark/hybrid_valhalla_lm.py` |
| 运行器 | `tools/fair_benchmark/run_hybrid_valhalla_lm.py` |
| 原生 QA | `manifestsys/hub-f64/src/native_qa.rs` |
| MCQ Fate | `manifestsys/hub-f64/src/mcq_fate.rs` |

### 4.2 公开镜像（valhalla-transformer v2.0）

| 产物 | 公开仓库路径 |
|------|--------------|
| 最新 hybrid JSON | `experiments/hybrid_unified_latest.json` |
| 带时间戳运行（历史） | `experiments/hybrid_unified_test_20260622_0345.json` — 已被 **final** 取代 |
| **Canonical deploy** | `experiments/hybrid_unified_latest.json` ← 由 final JSON 同步 |
| 文档 13/14 | `en/13_*`、`zh/13_*`、`en/14_*`、`zh/14_*` |
| API | `api/server.js` — `/api/hybrid-unified` |

### 4.3 Fair 基准支撑

| 产物 | 位置 |
|------|------|
| Fair test 基线 | `reports/fair_benchmark/fair_benchmark_test_20260620_0859.json` |
| 套件规格 | `tools/fair_benchmark/fair_benchmark_suite.json` |

---

## 5. 复现协议（审计清单）

```bash
# 1. 获取 Valhalla（数据室权限）
# 2. 构建原生 QA
cd manifestsys/hub-f64
RUSTFLAGS='-L /opt/cuda/lib64' cargo build -p hub-f64 --release --bin valhalla_native_qa

# 3. 设置路径
export VALHALLA_NATIVE_QA=/path/to/target/release/valhalla_native_qa
export VALHALLA_MCQ_FATE_CYCLES=1   # Tile MCQ 记忆轨道

# 4. 运行 Hybrid（patch 轨道需 Qwen 权重）
python3 tools/fair_benchmark/run_hybrid_valhalla_lm.py \
  --phase test \
  --reuse-json reports/fair_benchmark/fair_benchmark_test_20260620_0859.json

# 5. 对比输出 JSON summary.hybrid.acc 与 0.807（± 环境容差）
```

**预期耗时**：视 GPU/CPU；patch 轨道占主导。审计日志须记录机器规格。

**skip-patch 模式**：仅验证 memory 合并逻辑；开放/数字 **不会** 达到 80% headline。

---

## 6. 架构与代码成熟度

| 组件 | 成熟度 | 说明 |
|------|--------|------|
| Hybrid 路由器 | **产品** | `HYBRID_PRODUCT_VERSION = "hybrid-2.0"` |
| Fair 基准 harness | **产品** | fair-1.2 切分，JSON 已发布 |
| native_qa MCQ 解码 | **Beta** | `mcq_option`；v2 decode 已回滚 |
| MCQ Fate 循环 | **Beta** | Tile 最优；env 门控 |
| Patch bridge | **产品** | Tier A；开放路径已验证 |
| 企业 SDK | **Alpha** | JSON 含路由轨迹；无正式 SDK |
| Wave / Val-OS / Yaksha | **并行 SKU** | 可选；非 Hybrid 分数必需 |

---

## 7. 风险登记

| ID | 风险 | 可能性 | 影响 | 缓解 |
|----|------|--------|------|------|
| R1 | MCQ 准确率平台期（deploy ~61%，结构 alone ~52%） | 高 | 中 | P0 选项比较器；结构引导 patch 路由 |
| R2 | 小 LM（0.5B）限制数字/开放上限 | 中 | 中 | 客户自选 LM；结构层与模型无关 |
| R3 | 语料规模（49 行）非生产级 | 中 | 高 | 设计伙伴语料 110+ 行；fair-2.0 |
| R4 | 复现环境敏感（CUDA、Qwen 路径） | 中 | 中 | Docker 复现包（路线图）；文档化 env |
| R5 | 投资人误读 80% 为「GPT 杀手」 | 中 | 高 | **分题型披露**；全文档 explicit 非目标 |
| R6 | Tier B 开放 alone 弱（~25%） | 已知 | 低 | 产品层 Hybrid 解决；总结 §3.3 已说明 |
| R7 | IP / 出口管制（双用途 AI） | 低 | 中 | 按司法辖区法律审查；默认本地部署 |
| R8 | 关键人依赖 | 中 | 高 | 架构文档化；模块化 bin；BP 招聘计划 |

---

## 8. 明确不宣称的内容

- MMLU、HumanEval 或通用推理相对前沿模型 SOTA  
- FIPS / 通用准则认证（Yaksha 为 Industry Gate 叙事，非认证）  
- Hub 补丁 +4.17 pp（strict MCQ 下已作废 — 见 correction record）  
- MCQ 达到人类水平  
- 无 Hybrid 路由的单轨道 80%  

---

## 9. 法律与合规（概览）

| 主题 | 状态 |
|------|------|
| 开源组件 | Rust 生态、Qwen 权重（Apache 2.0 模型许可 — 以当前 HF 卡片为准） |
| 客户数据 | 默认本地处理；无强制云推理 |
| 基准完整性 | 语料去污染；correction record 维护 |
| 出口 / AI 法规 | 按部署地（中、美、欧）评估 — **须法律顾问** |

---

## 10. 商业 DD 要点

| 问题 | 回答 |
|------|------|
| 谁先付费？ | 主权 AI / 垂直 SaaS / SI — 见 BP §4 |
| 定价模型？ | 授权 + POC；未定稿 — BP  indicative |
| 竞争？ | API 商、RAG 栈、LangChain — Hybrid 楔子是审计 + 结构 |
| 壁垒？ | Fair 基准纪律 + 已交付 Hybrid 产品 + 模块化 Rust 运行时 |
| 18 个月里程碑？ | MCQ 65%+、3 付费 POC、SDK GA |

---

## 11. 数据室待补项（按需）

- [ ] Cap table 与历史融资  
- [ ] 团队简介与 ESOP  
- [ ] 12 个月现金流预测  
- [ ] 客户 LOI / POC 管线  
- [ ] IP 归属链  
- [ ] Hybrid 全跑 Docker 镜像  
- [ ] 第三方基准审计函  

---

## 12. 联系与文档索引

| 文档 | 路径 |
|------|------|
| Pitch | `docs/hybrid_2.0/ZH_PITCH.md` |
| Pitch deck | `docs/hybrid_2.0/ZH_PITCH_DECK.md` |
| 技术白皮书 | `docs/hybrid_2.0/ZH_TECHNICAL_WHITEPAPER.md` |
| 商业计划书 | `docs/hybrid_2.0/ZH_BUSINESS_PLAN.md` |
| 本 DD 包 | `docs/hybrid_2.0/ZH_DUE_DILIGENCE.md` |

---

*Rogue Intelligence LNC. · hybrid-2.0 · fair-1.2 · hub-f64 1.1.0 · 机密 · 2026-06-23*
