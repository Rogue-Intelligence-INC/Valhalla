# Valhalla → 烘焙新模型（不修改原 checkpoint）

**日期**: 2026-06-24（bake 接口）· 2026-06-16（legacy patch demo）

Valhalla 从 **Hub / Tile / StemCell / Triad** 孵化结构，**导出并烘焙成独立新模型**。
原 HuggingFace 权重只读加载，**不在原模型上 patch**。

> Transformer 在这里是被拆开的现成模型。用法见 [`docs/valhalla/TRANSFORMER_USE.md`](../../docs/valhalla/TRANSFORMER_USE.md)。  
> **TF 前置 + 累积 patch 实验**见 [`../tier_a_tf_front/`](../tier_a_tf_front/README.md)

## 推荐流程（bake → HF → hybrid / fused）

```
fair 语料 / TTS+LLM 融合
    ↓
valhalla_model_export / valhalla_free_multimodal_incubate
    ↓
bake_valhalla_model.py / bake_free_multimodal_fusion.py
    ↓
models/hf/{ModelName}/              ← 完整 Hugging Face 格式
    ├── config.json
    ├── model.safetensors
    ├── tokenizer.json
    ├── chat_template.jinja
    ├── README.md                   ← model card
    ├── .gitattributes              ← LFS（上传 Hub 用）
    └── valhalla_manifest.json      ← 可选元数据，transformers 忽略
    ↓
AutoModelForCausalLM.from_pretrained(...)   或  finalize_hf_repo.py --push-hub
    ↓
valhalla_fused_lm / valhalla_hybrid
```

### Body 与 Hybrid 对应

| `--body` | 结构来源 | Hybrid 用途 |
|----------|----------|---------------|
| `hub` | Fate 四元路由 | Hub 全局偏好 |
| `tile` | Tile 块签名 | MCQ / 结构记忆 |
| `stemcell` | StemCell 细胞 | MCQ / 能量分层 |
| `triad` | Hub+Tile+Stem 融合 | 默认 patch / open 路径 |

## 命令

```bash
# 编译 export
RUSTFLAGS='-L /opt/cuda/lib64' cargo build -p hub-f64 --release --bin valhalla_model_export

# 烘焙新模型（fair test 语料，triad body）
source .venv-llm/bin/activate
export HF_ENDPOINT=https://hf-mirror.com
python tools/valhalla_model_bridge/bake_valhalla_model.py \
  --body triad \
  --phase test \
  --strength 0.08 \
  --out checkpoints/valhalla_baked/triad_fair_test_s0.08

# 用 baked 模型跑 fused（无 runtime patch）
./target/release/valhalla_fused_lm \
  --model checkpoints/valhalla_baked/triad_fair_test_s0.08 \
  --phase smoke
```

### 仅导出结构 JSON（不烘焙权重）

```bash
valhalla_model_export --body tile --corpus-file corpus.txt --grinder \
  --manifest-out checkpoints/my_bake/
```

## Legacy demo（apply_and_compare.py）

旧 demo 仍在内存中 patch 后 `--save` 到新目录；新工程请优先 `bake_valhalla_model.py`。

| 指标 | 值 |
|------|-----|
| Tile 块 | 29 |
| StemCell 细胞 | 138 |
| Fate 四元 scale | [0.9999, 1.0, 1.0, 1.0] |
| 修改 LayerNorm 层数 | 24 |
| 修改 embedding 行数 | 43 |
| 权重变化 L2 | 0.525 |
| **生成文本是否变化** | **是** |

Patch 前: *"...Neural Network Optimization (NNO) to improve efficiency..."*  
Patch 后: *"...dynamically adjust hyperparameters and architecture during training..."*

## 设计说明

- **quad_layer_scales**：Fate 四元偏好 → 四层 Transformer 组的 LayerNorm 乘子
- **patch_vector**：Tile + StemCell 签名融合（256 维）→ 投影到 hidden_size → 加到 prompt token embedding
- **strength**：控制改动幅度；过大可能损模型质量

与 Valhalla 哲学一致：结构来自 Fate/Tile/StemCell，**非** 反向传播训练栈。
