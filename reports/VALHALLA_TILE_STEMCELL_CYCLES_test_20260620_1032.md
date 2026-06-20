# Tile / StemCell 多轮 Cycle 优化实验

**Run**: 20260620_1032 · **Phase**: test · **N**: 145
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
| tile_c1_isolated | 34.5% | 57.9% | 28.3% | 24.6% | t[178, 179, 180, 181, 182, 183, 184, 186, 187, 188, 189, 190, 191, 192] s[0] |
| tile_c5_isolated | 34.5% | 57.9% | 28.3% | 24.6% | t[177, 178, 179, 180, 181, 182, 183, 185, 186, 187, 188, 189, 190, 192] s[0] |
| tile_c10_isolated | 34.5% | 57.9% | 28.3% | 24.6% | t[178, 179, 180, 181, 182, 183, 184, 186, 187, 188, 189, 190, 191, 192] s[0] |
| tile_c20_isolated | 34.5% | 57.9% | 28.3% | 24.6% | t[178, 179, 180, 181, 182, 183, 184, 186, 187, 188, 189, 190, 191, 192] s[0] |
| stemcell_c1_isolated | 34.5% | 57.9% | 28.3% | 24.6% | t[0] s[278, 279, 280, 281, 282, 283, 284, 286, 287, 288, 289, 290, 291, 293] |
| stemcell_c5_isolated | 34.5% | 57.9% | 28.3% | 24.6% | t[0] s[277, 278, 279, 280, 281, 282, 283, 285, 286, 287, 288, 289, 290, 292] |
| stemcell_c10_isolated | 35.2% | 57.9% | 28.3% | 26.2% | t[0] s[278, 279, 280, 281, 282, 283, 284, 286, 287, 288, 289, 290, 291, 292] |
| stemcell_c20_isolated | 35.2% | 57.9% | 28.3% | 26.2% | t[0] s[278, 279, 280, 281, 282, 283, 284, 286, 287, 288, 289, 290, 291, 292] |

## 4. Δ vs c1（isolated）

- **tile_c5_isolated**: +0.00 pp [+0.00, +0.00]
- **tile_c10_isolated**: +0.00 pp [+0.00, +0.00]
- **tile_c20_isolated**: +0.00 pp [+0.00, +0.00]
- **stemcell_c5_isolated**: +0.00 pp [+0.00, +0.00]
- **stemcell_c10_isolated**: +0.69 pp [+0.00, +2.07]
- **stemcell_c20_isolated**: +0.69 pp [+0.00, +2.07]

## 5. 结论

**tile structure**: patch_hash 不变 (c1≡c20) · counts c1→c20: (177, 0) → (177, 0). **stemcell structure**: patch_hash 随 cycle 变化 · counts c1→c20: (0, 277) → (0, 277). **tile QA isolated c1→c20**: +0.00 pp. **stemcell QA isolated c1→c20**: +0.69 pp.
