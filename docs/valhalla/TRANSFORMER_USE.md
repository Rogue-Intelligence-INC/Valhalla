# Transformer 在 Valhalla 里的用法

**日期：** 2026-10-30  
**代码：** `tools/model_decompose/shred_qwen_weights.py` · `manifestsys/hub-f64/src/model_fate.rs` · `manifestsys/hub-f64/src/bin/valhalla_model_fate.rs`

Transformer 在这里是一份已经存在的语言模型，被拆开，用来确认它能不能当作后续材料。它不是被反向传播训练的学生。

## 拆什么

`shred_qwen_weights.py` 读取一份现成的 Qwen（默认 `Qwen/Qwen2.5-0.5B-Instruct`）。解码层切成四段，映射到 Valhalla 的体：

| 段 | 层 | 去向 | 阶段 |
|---|---|---|---|
| Q0 | 前四分之一 | Hub | decompose |
| Q1 | 第二个四分之一 | Tile | decompose |
| Q2 | 第三个四分之一 | Stem | decompose |
| Q3 | 最后四分之一 | Hub | aggregate |

每一层的签名来自 `input_layernorm`、`post_attention_layernorm`，以及注意力 `q_proj` / `k_proj` / `v_proj` / `o_proj` 的行均值。签名归一化后投成 hub、tile、stem 三个端口。整网再追加两片：`MODEL_FATE_AGGREGATE`（Hub）和 `MODEL_RECOMPOSE`（Triad）。

拆出来的是层权重签名，不是从模型里抽出的文本句。协议名是 `valhalla-model-decompose-v1`。

## 怎样确认

`valhalla_model_fate` 把这些端口吃进 `TriadSession`，用 Fate 做多轮适应。确认读数是结构闭合，不是生成出来的句子。

`model_closure_score` 取各分片分数的均值和离散程度：`0.7 × 均值 + 0.3 × (1 − 离散)`。

适应可以在这些条件下停下：`fate_fixed_point`、`fate_converged`、`fate_marginal_stable`。其中 `model_recompose_ready` 要求同时满足：

- `aggregate_score ≥ 0.85`
- `decompose_score ≥ 0.75`
- Fate 不再处于探索

闭合之后，适应过的 Triad 补丁才允许写回一份**新的**检查点。原 Hugging Face 权重只读，不在原文件上改。

## 和文本语料的关系

文本行是另一条入口：`valhalla_model_export` 的 grinder 或 `normalized_f64`。那条路喂的是句子，用来孵化 Hub / Tile / Stem。

模型分解确认的是：这份现成 LLM 的层结构能不能在 Hub、Tile、Stem 上闭合。闭合了，才把它当作后面的材料。文本语料的质量不由这次分解打分。

## 写回不是训练

写回使用已有的权重手术（`patch_lm.rs` 的 `apply_legacy_patch_tensors`）：四元 scale 乘上各层 `input_layernorm`，结构向量按 `strength` 加到相关 token 的 `embed_tokens` 行上。这里没有损失，也没有 `backward`。

因此这条链路的监控是 `decompose_score`、`aggregate_score`、`model_closure_score`，以及停机原因是不是 `model_recompose_ready`。不要用自由生成的句子来判断分解有没有确认成功。

单题问答里的分解是另一条路：`docs/hybrid_2.0/PIPELINE_V4_LM_DECOMPOSE_VALHALLA_RECOMPOSE.md` 把一道题切成 stem / option 等分片再回写。那不是对整份现成模型做质量确认。

## 命令

```bash
python tools/model_decompose/shred_qwen_weights.py \
  --model Qwen/Qwen2.5-0.5B-Instruct \
  --out reports/model_decompose/qwen_shards.json

cargo build -p hub-f64 --release --bin valhalla_model_fate
valhalla_model_fate --shards reports/model_decompose/qwen_shards.json
```

Windows 桌面壳只在「模型」页选中这份已有的 Qwen。拆解和闭合确认在主项目运行时里做，不在壳里训练。
