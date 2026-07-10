# Gap Analysis — FRAMES · Backbone · ChatGPT Critique

**Date:** 2026-07-11  
**Scope:** First-hand root cause · fixes applied · backbone orthogonality clarification

---

## 1. FRAMES 1/25 — 根因（一手）

### 1.1 现象

| 字段 | 值 |
|------|-----|
| Industry 3B 标定 | **1/25 (4%)** |
| `wiki_items_fetched` | **0** |
| 典型错误答案 | `[corpus:0] org/wiki/Mona_Lisa#...`（URL 片段，非数值/实体） |
| Paper baseline | naive **40.8%** · oracle **72.9%** |

### 1.2 根因链（已验证）

```
google/frames-benchmark (HF)
        ↓ download_frames()
Wikipedia REST/API 拉取 extract
        ↓ 失败（本机 en.wikipedia.org 超时 / 未缓存）
bundle corpus = ["Reference: https://en.wikipedia.org/wiki/..."]  ← 无正文
        ↓ ingest + retrieve
模型只能 cite URL placeholder → 几乎全错
```

**关键 bug（已修）：** `run_industry_benchmarks.py` 的 `ensure_frames_bundle()` 曾带 **`--no-wiki-fetch`** 重下 bundle —— 即使网络可用也不会拉 wiki，导致 **Tier C 无效分数仍被跑**。

### 1.3 不是模型能力问题（在此状态下）

在 corpus 仅为 URL placeholder 时：

- FRAMES **不能**反映 multi-hop RAG 能力  
- 分数 **禁止对外 cite**（Tier C）  
- ChatGPT 若说「FRAMES 证明长链/multi-hop 不行」→ **在此 bundle 状态下成立，但是基础设施问题，不是算法上限**

### 1.4 已做修复

| 修复 | 文件 |
|------|------|
| Wiki URL 去 `#fragment` / `?query` | `download_utils.py`, `frames_wiki.rs` |
| FRAMES preflight（无 extract 则 exit 2） | `common.py`, `test_frames_v1.py` |
| Industry 下载 **默认拉 wiki**（去掉 `--no-wiki-fetch`） | `run_industry_benchmarks.py` |
| Prefetch 脚本 | `scripts/prefetch_frames_wiki.sh` |

### 1.5 恢复有效 FRAMES 的步骤

```bash
# 需能访问 Wikipedia API
./scripts/prefetch_frames_wiki.sh

# 验证 + 快测
python3 tools/valhalla_inference/test_frames_v1.py --rust --fast
```

预期：`wiki_items_with_extract` ≥ 50% 后分数才有意义；可与 paper naive 40.8% 对照。

---

## 2. Backbone — ChatGPT 是否理解有误？

### 2.1 我们**实际**证明的是什么

**协议：** `valhalla-independence-v1`

| 主张 | 含义 | 一手证据 |
|------|------|----------|
| **Open path 正交** | Tier-B **open** 题：`native`/`hybrid` 答案与 backbone **字节级一致** | RWKV + MoE + Qwen3B **6/6** |
| **Session memory 正交** | Needle hybrid 换 `lm_model` 仍等于 native ref | **3/3** |
| **decode_path** | 均为 `valhalla_base/native/.../open_retrieval_span` | independence JSON |

**不**包含在 primary verdict 内：

- MCQ logprob 缩放（0.5B→3B 98%）  
- `trad_lm` RAG 对照  
- core_fusion / proactive / longchain（固定 Qwen 栈 appendix smoke）

### 2.2 常见**误读**（ChatGPT / 外部）

| 误读 | 实际 |
|------|------|
| 「换任何 backbone 产品能力一样」 | 仅 **open native 结构路径** 一致；MCQ/chat 质量仍随 LM 变 |
| 「证明不需要 LM」 | LM 是 **decode head**；结构负责 retrieve + session |
| 「六类都测完了」 | Registry 有 6 类 `paper_panel()`；independence **只跑了 3 preset** |
| 「hybrid = 用 LM 生成 open 答案」 | open 题走 **structure native**；hybrid 指 MCQ 臂 + 配置标签 |

### 2.3 Registry vs 实测差距

`backbone_registry.py` **paper_panel** 六类：

| 类 | Preset | Independence 已跑？ |
|----|--------|---------------------|
| Dense | qwen3_4b / qwen_3b | ✅ qwen_3b |
| SSM (Mamba) | mamba2_2.7b | ❌ |
| RWKV | rwkv7_0.1b | ✅ |
| MoE | phi_tiny_moe | ✅ |
| Encoder | modernbert | ❌（不同 arm：`encoder`） |
| Vision | qwen2_5_vl_3b | ❌（不同 arm：`vision`） |

**结论：** ChatGPT 要求六类 panel **方向正确**；我们 **registry 就绪但未跑全**。这不否定已做的 3 类正交，但 **不能**对外说「六架构 universal proof done」。

### 2.4 Encoder / Multimodal 与「正交」关系

- **Encoder-only**（ModernBERT/Jina）：不测 chat；测 **Fragment → embedding → retrieval**（结构 ingress 的另一臂）  
- **Multimodal**：`context_items` 已有 API + Flutter UI；与 **causal LM backbone sweep** 是不同维度  

ChatGPT 把六类合成一个 verdict **过于粗糙**；工程上应拆：

1. **Causal LM orthogonality**（open path）  
2. **Encoder retrieval arm**  
3. **Multimodal ingress arm**

---

## 3. 其他短板 — 原因与是否「误解」

| 短板 | 一手原因 | 是否误解 |
|------|----------|----------|
| LoCoMo 47% vs pretrain 100% | **不同协议**（full session blind vs 17 行 minimal + 监督 pretrain） | 混读 = 误解 |
| trad_lm 100% vs hybrid 87% | open decode 弱于 LM 直接读 context | 非误解 |
| LaRA RAG 8/20 | citation snippet 非 Yes/No compose | 非误解；需 multihop compose |
| LongBench 35% | 0.5B/短 slice + hybrid | 部分环境/规模问题 |
| TPI transfer = isolated | 优势在 **pollution rescue** 非 ceiling | 对外只报 TPI 数字 = 我们表述问题 |

---

## 4. 行动优先级

| P | 行动 | 回应 ChatGPT |
|---|------|--------------|
| **P0** | `./scripts/prefetch_frames_wiki.sh` + 重跑 FRAMES | 无效分数 → 可 cite 分数 |
| **P1** | `run_valhalla_independence.py --paper`（六类 smoke，能加载则跑） | 补 universal panel |
| **P1** | Encoder arm benchmark（ModernBERT + native retrieval） | 第 5 类 |
| **P2** | LoCoMo industry + pretrain-adapt 同协议对照 | 消掉 100% vs 47% 混读 |
| **P2** | LaRA / MultiHop → hybrid compose 默认路径 | 补 long-doc 弱项 |

---

## 5. 对外一句话（修正版）

> **FRAMES 1/25 是 wiki corpus 缺失导致的 Tier C，不是 multi-hop 能力定论。**  
> **Backbone 正交指 open 结构 decode 不随 checkpoint 变，不是「任意 backbone 全能力等价」。**  
> 六类 architecture panel：**registry 有、independence 未穷尽** — ChatGPT 这条仍是我们短板。

---

*Rogue Intelligence LNC.*
