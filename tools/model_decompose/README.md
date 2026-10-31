# 模型分解

把一份已经存在的 Qwen 拆成 Hub / Tile / Stem 端口，用 Fate 闭合来确认它能不能当作后续材料。完整说明见 [`docs/valhalla/TRANSFORMER_USE.md`](../../docs/valhalla/TRANSFORMER_USE.md)。

```bash
python tools/model_decompose/shred_qwen_weights.py \
  --model Qwen/Qwen2.5-0.5B-Instruct \
  --out reports/model_decompose/qwen_shards.json
```

四段映射：Q0 → Hub，Q1 → Tile，Q2 → Stem，Q3 → Hub（aggregate）。签名来自各层 LayerNorm 和注意力投影的行均值。

`valhalla_model_fate --shards` 吃进这份 JSON。`decompose_score ≥ 0.75` 且 `aggregate_score ≥ 0.85`、并且 Fate 不再探索时，停机原因是 `model_recompose_ready`。

这不是反向传播，也不从模型里抽出文本句。文本语料走 `valhalla_model_export`。
