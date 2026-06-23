# Multimodal Fusion Benchmark

**Version**: fusion-bench-1.0 · **Date**: 2026-06-19  
**Artifact**: `multimodal_fusion_bench.json`  
**Status**: **OVERALL PASS**

---

## Purpose

Stronger test of **cross-modality fusion** (text + TTS audio → Tile + StemCell Triad) beyond the 5-sample smoke test. Validates structure superadditivity, modality complementarity, corpus batch incubation, and utterance diversity.

## Run

```bash
RUSTFLAGS='-L /opt/cuda/lib64' cargo build -p hub-f64 --release --bin valhalla_multimodal_fusion_bench
VALHALLA_DL_FRONT=1 python3 tools/valhalla_dl_front/run_multimodal_fusion_bench.py --cycles 3
```

## Gates (12 pairs, espeak TTS, 3 optimize cycles)

| Gate | Metric | Threshold | Result |
|------|--------|-----------|--------|
| Tile superadditivity | fusion tiles > max(text, audio) | ≥ 50% pairs | **58.3%** ✓ |
| Stem superadditivity | fusion stem > max(text, audio) | ≥ 35% pairs | **91.7%** ✓ |
| Complementarity | tile sig cos(text, audio) | 0.05–0.50 | **0.285** ✓ |
| Modality ratio | mean audio tiles / text tiles | ≥ 5× | **10.9×** ✓ |
| Corpus batch | tiles per pair in batch session | ≥ 5 | **6.7** ✓ |
| Utterance diversity | duration std (seconds) | ≥ 0.4 | **0.54** ✓ |

**Overall**: `summary.overall_pass_with_embedding = true`

## Key structure metrics

| Metric | Value |
|--------|-------|
| Mean fusion tiles (single) | 48.5 |
| Batch fusion tiles (12 pairs, one session) | 80 |
| Mean fusion stem cells | 143.3 |
| Batch fusion stem cells | 292 |

## Interpretation

1. **Audio drives structure** (~11× more Tile activity than text alone).
2. **Fusion is superadditive** on Tile (58%) and especially Stem (92%).
3. **Modalities complement** (tile signature cos ≈ 0.29, not redundant).
4. **Corpus-level fusion** consolidates 12 utterances into shared structure (80 tiles, 292 stem cells).
5. Not yet on QA path — structure/incubation layer only.

## Open-question memory pilot (2026-06-19)

Triad memory, 61 open Q, espeak + `VALHALLA_MULTIMODAL_FUSION=1`:

| Track | Correct |
|-------|---------|
| text_only | 4/61 |
| text + TTS audio | 4/61 |

See `multimodal_memory_pilot.json` — wired, **no task delta yet**.

## Files

| Path | Role |
|------|------|
| `manifestsys/hub-f64/src/bin/valhalla_multimodal_fusion_bench.rs` | Rust gates |
| `tools/valhalla_dl_front/run_multimodal_fusion_bench.py` | TTS + orchestration |
| `tools/valhalla_dl_front/encode.py` | `audio_path` → mel bands |
| `bench_wav/` | Generated utterances |

---

## 中文摘要

**多模态融合基准 v1.0** — 12 条 text+TTS 配对，Triad（Tile+Stem）结构级验收。

- Tile 超加性：**58.3%** 样本融合超过单模态  
- Stem 超加性：**91.7%**  
- 模态互补（tile sig 余弦）：**0.285**  
- 音频/文本 Tile 比：**10.9×**  
- 语料批量融合：**6.7 tiles/pair**  
- **总评：PASS**

复现见上方命令；完整 JSON 见 `multimodal_fusion_bench.json`。
