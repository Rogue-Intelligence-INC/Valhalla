# Tile / StemCell 多轮 Cycle 优化实验

**Run**: 20260620_1027 · **Phase**: smoke · **N**: 12
**Corpus**: 49 lines · **Cycles tested**: [1, 5, 10, 20]

## 1. 结构 export（patch 是否随 cycle 变）

| body | cycles | patch_hash | tiles | stem | trace_len |
|------|--------|------------|-------|------|-----------|
| tile | 1 | `2ff181b332274c5f` | 177 | 0 | 1 |
| tile | 5 | `2ff181b332274c5f` | 177 | 0 | 5 |
| tile | 10 | `2ff181b332274c5f` | 177 | 0 | 10 |
| tile | 20 | `2ff181b332274c5f` | 177 | 0 | 20 |
| stemcell | 1 | `af0f334aa497541b` | 0 | 277 | 1 |
| stemcell | 5 | `12650c8e7bcbe1e6` | 0 | 277 | 5 |
| stemcell | 10 | `f18e55bfa289c396` | 0 | 277 | 10 |
| stemcell | 20 | `ca5d1f21c36c66f7` | 0 | 277 | 20 |

## 2. QA acc（isolated · corpus_cycles）

| arm | acc | numeric | mcq | open | tile/stem unique |
|-----|-----|---------|-----|------|------------------|
| tile_c1_isolated | 75.0% | 100.0% | 25.0% | 0.0% | t[184, 185, 186, 188, 189, 192, 193] s[0] |
| tile_c5_isolated | 75.0% | 100.0% | 25.0% | 0.0% | t[183, 184, 186, 187, 188, 191, 192] s[0] |
| tile_c10_isolated | 75.0% | 100.0% | 25.0% | 0.0% | t[184, 185, 186, 188, 189, 192, 193] s[0] |
| tile_c20_isolated | 75.0% | 100.0% | 25.0% | 0.0% | t[184, 185, 186, 188, 189, 192, 193] s[0] |
| stemcell_c1_isolated | 75.0% | 100.0% | 25.0% | 0.0% | t[0] s[284, 285, 287, 288, 289, 293] |
| stemcell_c5_isolated | 75.0% | 100.0% | 25.0% | 0.0% | t[0] s[283, 284, 286, 287, 288, 289, 292] |
| stemcell_c10_isolated | 75.0% | 100.0% | 25.0% | 0.0% | t[0] s[284, 285, 286, 288, 289, 292, 293] |
| stemcell_c20_isolated | 75.0% | 100.0% | 25.0% | 0.0% | t[0] s[284, 285, 286, 288, 289, 292, 293] |

## 3. QA acc（batch · incubation_cycles）

| arm | acc | numeric | mcq | open |
|-----|-----|---------|-----|------|
| tile_c1_batch | 83.3% | 100.0% | 50.0% | 0.0% |
| tile_c5_batch | 83.3% | 100.0% | 50.0% | 0.0% |
| tile_c10_batch | 83.3% | 100.0% | 50.0% | 0.0% |
| tile_c20_batch | 83.3% | 100.0% | 50.0% | 0.0% |
| stemcell_c1_batch | 83.3% | 100.0% | 50.0% | 0.0% |
| stemcell_c5_batch | 83.3% | 100.0% | 50.0% | 0.0% |
| stemcell_c10_batch | 83.3% | 100.0% | 50.0% | 0.0% |
| stemcell_c20_batch | 83.3% | 100.0% | 50.0% | 0.0% |

## 4. Δ vs c1（isolated）

- **tile_c5_isolated**: +0.00 pp [+0.00, +0.00]
- **tile_c10_isolated**: +0.00 pp [+0.00, +0.00]
- **tile_c20_isolated**: +0.00 pp [+0.00, +0.00]
- **stemcell_c5_isolated**: +0.00 pp [+0.00, +0.00]
- **stemcell_c10_isolated**: +0.00 pp [+0.00, +0.00]
- **stemcell_c20_isolated**: +0.00 pp [+0.00, +0.00]

## 5. 结论

**tile structure**: patch_hash 不变 (c1≡c20) · counts c1→c20: (177, 0) → (177, 0). **stemcell structure**: patch_hash 随 cycle 变化 · counts c1→c20: (0, 277) → (0, 277). **tile QA isolated c1→c20**: +0.00 pp. **stemcell QA isolated c1→c20**: +0.00 pp.
