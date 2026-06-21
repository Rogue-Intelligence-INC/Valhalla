# Valhalla MCQ Experiments & Fair Benchmark — Project Summary  
# Valhalla 多选题实验与公平基准 — 项目总结

**Date / 日期**: 2026-06-22  
**Author / 出品**: Rogue Intelligence LNC.  
**Benchmark / 基准**: Fair Benchmark test split · 145 questions (46 multiple-choice) · 49-line clean corpus  
**Companion LLM / 配套语言模型**: Qwen2.5-0.5B-Instruct

---

## 1. What problem are we solving?  
## 1. 我们在解决什么问题？

**EN:** Valhalla is a Q&A system that does **not** rely on an LLM generating text token-by-token. Reference text and user questions are converted into numeric signals, “digested” by an internal structure built from the corpus, and answers are produced by rule-based scoring. The small LLM (Qwen 0.5B) is used mainly for **feature extraction** or **open-ended generation**, not as the only way to answer.

This phase focuses on **four-option multiple-choice (MCQ)**: the same 46 questions are used to compare three answering modes and to locate the real bottleneck.

**中文：** Valhalla 是一套**不依赖大模型逐 token 生成**的问答系统：参考文本和问题先变成数值信号，在由语料“长出来”的内部结构里消化信息，再用规则化打分给出答案。配套小模型（Qwen 0.5B）主要用于**特征提取**或**开放题生成**，不是唯一答题路径。

本阶段聚焦 **四选一多选题（MCQ）**：用同一套 46 道题对比三种答题方式，并找出真正的瓶颈。

---

## 2. Three ways to answer (plain language)  
## 2. 三种答题方式（日常语言）

| Mode / 方式 | Plain description / 说明 | MCQ output / 多选题输出 |
|-------------|--------------------------|-------------------------|
| **A. Native memory** / **原生记忆** | Corpus → internal structure → score each option → pick highest | Fixed `ANSWER: A/B/C/D` |
| **B. LLM only** / **纯语言模型** | Load Qwen per question; generate free text | Extract letter from generated text |
| **C. LLM + structure patch** / **模型 + 结构补丁** | Export structure from Valhalla corpus; tweak Qwen weights; then generate | Same as B |

**Fair Benchmark rules / 公平原则：**

- **EN:** Eval corpus **excludes** reference answers (no leakage). Native and LLM tracks are **scored separately**. Native mode has two session types: **per-question reload** (corpus re-ingested every time — slow, weak structure) vs **one-shot incubation + many questions** (corpus ingested once — much better for MCQ).
- **中文：** 评测语料**不含**标准答案。原生与 LLM **分开报分**。原生有两种会话：**逐题重载**（每题重读语料，结构弱）与 **一次孵化 + 多题连答**（语料只消化一次，多选题明显更好）。

---

## 3. Key numbers (46 MCQ, test split)  
## 3. 核心数字（46 道多选题）

**Source / 来源:** `reports/fair_benchmark/fair_benchmark_test_20260620_0859.json` and 2026-06-22 follow-up runs.

### 3.1 MCQ results / 多选题主结果

| Method / 方法 | Correct / 正确 | Accuracy / 准确率 | Notes / 备注 |
|---------------|------------------|-------------------|--------------|
| Native, no corpus / 原生·无语料 | 6/46 | 13.0% | ~random / 近随机 |
| Native, per-question reload / 原生·逐题重载 | 13/46 | 28.3% | Old default / 旧默认 |
| **Native, one-shot, all modules / 原生·一次孵化·三模块** | **21/46** | **45.7%** | Main baseline / 主 baseline |
| **Native, one-shot, tile module only / 原生·一次孵化·Tile** | **22/46** | **47.8%** | Best MCQ so far / 目前最高 |
| Native, one-shot, stem module only / 原生·一次孵化·Stem | 19/46 | 41.3% | 2026-06-22 |
| Qwen zero-shot / 纯 Qwen | 17/46 | 37.0% | 71% on 145Q via open / 开放题拉高总分 |
| Qwen + Valhalla patch / Qwen + 补丁 | 12–13/46 | 26–28% | Patch hurts MCQ / 补丁未帮 MCQ |

**EN takeaways:** (1) Native + corpus + one-shot beats LLM-only on MCQ (21 vs 17). (2) **Session protocol** matters more than small scoring tweaks (+9 MCQ from reload → one-shot). (3) Valhalla patch does **not** improve MCQ on this benchmark.

**中文结论：** (1) 原生 + 语料 + 一次孵化 优于 单独 Qwen（21 vs 17）。(2) **会话协议**影响大于小改打分公式。(3) 给 Qwen 打结构补丁未提高多选题。

### 3.2 Full 145 questions / 全量 145 题

| Method / 方法 | Correct / 正确 | Accuracy / 准确率 |
|---------------|------------------|-------------------|
| Native, one-shot, all modules | 59/145 | 40.7% |
| Qwen zero-shot | 103/145 | 71.0% |

**EN:** Qwen is strong on **open** (~93%) and **numeric** (~76%) questions. Native is competitive on numeric (~58% in benchmark) but weaker on open and MCQ decoding.

**中文：** Qwen 在**开放题**、**数字题**仍强；原生数字题不错，弱项是开放题与多选题解码。

---

## 4. How does native MCQ answering work?  
## 4. 原生多选题怎么答？

**EN (current `mcq_option` scorer):**

1. **Whole question input** — stem + four options + instructions, encoded once.
2. **Corpus incubation** — 49 reference lines build a shared memory field.
3. **Per-option scoring** (four independent scores, then argmax):
   - Option vector vs whole-question vector (cosine)
   - Option vs structure summary vector
   - Word overlap between option and corpus lines (weight reduced to limit “keyword guessing”)
   - Keyword overlap helpers
4. **Output:** highest score → `ANSWER: X`

This is **not** relative comparison (“A is better than B by Δ”). It is **not** chain-of-thought from an LLM. That limits accuracy around ~45–48%.

**中文（当前 `mcq_option` 打分）：** 整题一次编码 → 语料孵化 → 四选项独立打分取最大 → `ANSWER: X`。不是选项间相对排序，也不是 LLM 推理，这是 ~45–48% 天花板的主因之一。

**At 21/46 / 在 21/46 上：** Predictions A8·B15·C12·D11 vs gold A4·**B27**·C12·D3 — roughly uniform guesses, not aligned with label distribution. ~half of hits come from corpus word overlap + weak structure, not stable stem understanding.

---

## 5. How does Qwen alone perform on MCQ?  
## 5. 纯 Qwen 多选题表现

**EN:**

- Each question: load model, strict prompt (“end with `ANSWER: X`”).
- Model often writes **explanations** (“Photosynthesis occurs in chloroplasts…”) instead of a single letter.
- Scorer **extracts** A/B/C/D; **16/46** fail extraction → counted wrong.
- Typical error: answers “Paris” instead of picking option C about France.

Overall 71% on 145Q **must not** be read as MCQ ability — MCQ subset is **37%**, below native **~46%**.

**中文：** 常生成解释段而非选项字母；16/46 抽不到字母；总准确率 71% 不能代表多选题（仅 37%）。

---

## 6. Experiments run in this cycle  
## 6. 本轮实验与结论

All on **46 MCQ**; JSON under `reports/experiments/`.

| Experiment / 实验 | Idea / 做法 | vs baseline | Verdict / 结论 |
|-------------------|-------------|-------------|----------------|
| Task tag / internal routing / 任务标签·路由 | Route MCQ differently | 21→21 | Not the bottleneck / 非瓶颈 |
| 3-factor structural scorer / 三因子打分 | support − contradiction + intent | Worse / flat | Wrong decomposition / 分解错位 |
| Negation branch / 否定分支 | NOT / EXCEPT | +1 | Few such items / 题少 |
| Stem-only query + joint embed / 仅题干+联合向量 | Less option pollution in query | 21→16 | Structure too weak alone / 结构不够 |
| Weaker corpus word boost / 削弱词面加分 | Cap keyword matching | 21 flat | OK for one-shot / 一次孵化可保留 |
| One-shot whole question / 整题·一次孵化 | Tile 22, Stem 19 | +9 vs reload | **Protocol wins / 协议有效** |
| Decode v2 + skip channel / decode v2 | Mixed stem, relative score | 18–22 | **Reverted / 已回滚** |

**Works / 有效:** one-shot incubation, whole-question ingress, toned-down keyword boost.  
**Does not / 无效:** routing patches, wrong-layer decomposition, stem-only query (today).  
**Next / 待做:** stem-conditioned **relative** option comparison; corpus line → which option; module-specific weights.

---

## 7. Architecture (public, no internal codenames)  
## 7. 系统架构（对外版）

```
Corpus (49 lines, no answers / 语料，无答案)
        │
        ▼
Text → numeric vectors (Grinder; optional LLM hidden states)
        │  文本 → 数值向量（Grinder；可选用 LLM 隐层）
        ▼
Internal incubation (router + graph tiles + cell clusters)
        │  内部孵化（路由中枢 + 图块 + 细胞群）
        ▼
User question (whole: stem + options / 整题)
        │
        ├─► Native: score A/B/C/D → ANSWER: X
        └─► LLM: Qwen generate → extract letter
```

**Grinder / 文本前端：** Text → fixed-size vectors for internal modules; benchmark default = deterministic hash (same API as LLM layers).

**One-shot incubation (persistent batch) / 一次孵化：** Corpus fed once; structure shared across questions — MCQ **28% → ~46%**.

---

## 8. Reproduce / 复现

| Item / 内容 | Path / 路径 |
|-------------|-------------|
| Native QA / 原生问答 | `manifestsys/hub-f64/src/native_qa.rs` |
| Fair benchmark / 公平基准 | `tools/fair_benchmark/` |
| Main results / 主结果 | `reports/fair_benchmark/fair_benchmark_test_20260620_0859.json` |
| MCQ experiments / MCQ 实验 | `reports/experiments/MCQ_*`, `TILE_STEM_WHOLE_QUESTION_*` |
| Build / 构建 | `cargo build -p hub-f64 --release --bin valhalla_native_qa` |
| Whole-question test / 整题测试 | `python tools/fair_benchmark/run_tile_stem_whole_question_test.py` |

**Current code state / 当前代码：** decode v2 **reverted**; weakened corpus word boost **kept**; path `mcq_option`.

---

## 9. Recommended next steps / 下一步建议

1. **Lock protocol / 锁定协议:** one-shot + whole question; tile module best on MCQ (22/46).
2. **Fix the comparator / 改比较器:** stem-conditioned relative scores + corpus-to-option support (not more routing rules).
3. **Split roles / 分工:** Qwen for open text; native for MCQ/numeric; do not expect patch to fix MCQ.
4. **Report by type / 分题型报告:** never use 145Q accuracy alone for MCQ claims.

---

## 10. One sentence / 一句话

**EN:** On 46 MCQs, Valhalla native (“corpus incubation + whole question + option scoring”) reaches **~22/46**, above **17/46** for Qwen 0.5B alone. The bottleneck is **how to compare four options**, not whether the question entered the system.

**中文：** 在 46 道多选题上，Valhalla 原生（语料孵化 + 整题输入 + 选项打分）约 **22/46**，高于单独 Qwen 的 **17/46**；瓶颈在**如何比较四个选项**，而非题目有没有进入系统。

---

*Rogue Intelligence LNC. · Report v1.1 bilingual · 2026-06-22*
