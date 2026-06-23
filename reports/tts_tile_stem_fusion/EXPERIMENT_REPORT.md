# TTS × Tile/StemCell 多模态融合实验报告

**Date**: 2026-06-19  
**Status**: smoke / structure probe（非 QA benchmark）  
**Primary artifact**: `fusion_report.json`（本目录）  
**WAV samples**: `s1.wav` … `s5.wav`

---

## 1. 实验目的

验证 **同一语义内容** 经 TTS 合成语音后，通过 DL front 分别/联合进入 **Tile + StemCell（Triad）** 时，结构孵化是否出现可测量的 **多模态融合** 效应（相对 text-only、audio-only 单模态基线）。

---

## 2. 管线

```
文本 ──┬── ModalityInput::Text ──────────► DL front (384-d) ──► hub / tile / stem ports
       │
       └── TTS (espeak) ── WAV ── mel bands ── ModalityInput::AudioFeatures ──► 同上
```

| 组件 | 路径 |
|------|------|
| TTS + 编排脚本 | `tools/valhalla_dl_front/run_tts_tile_stem_fusion.py` |
| 音频编码 sidecar | `tools/valhalla_dl_front/encode.py`（`audio_path` → 64 mel bands + 元数据 → 384-d） |
| Rust 融合对比 bin | `manifestsys/hub-f64/src/bin/valhalla_tile_stem_tts_fusion.rs` |
| Ingress | `TriadSession::feed_modality` → `BodyTarget::Triad` |

### 2.1 运行配置（本次记录）

| 参数 | 值 |
|------|-----|
| TTS backend | **espeak**（离线；Piper 模型下载因网络不可达未使用） |
| `VALHALLA_DL_FRONT` | **1**（Python sidecar；文本走 MiniLM/hash，音频走 `wav_mel_bands`） |
| `optimize_cycles` | **2** |
| 样本数 | **5** |

### 2.2 复现

```bash
RUSTFLAGS='-L /opt/cuda/lib64' cargo build -p hub-f64 --release --bin valhalla_tile_stem_tts_fusion

VALHALLA_DL_FRONT=1 python3 tools/valhalla_dl_front/run_tts_tile_stem_fusion.py \
  --tts espeak --skip-download --cycles 2
```

联网后可尝试 Piper / edge-tts：

```bash
python3 tools/valhalla_dl_front/run_tts_tile_stem_fusion.py --tts auto
```

---

## 3. 测试样本

| ID | 文本 |
|----|------|
| s1 | Love is eternal |
| s2 | The container holds many items |
| s3 | What is the capital of France |
| s4 | Water boils at one hundred degrees Celsius |
| s5 | A quick brown fox jumps over the lazy dog |

每种样本跑三种模式：`text_only` · `audio_only` · `text+audio_fusion`（同一 Triad session 顺序喂入 text 再 audio）。

---

## 4. 结果（来源：`fusion_report.json`）

### 4.1 汇总

| 指标 | 值 |
|------|-----|
| `avg_text_audio_patch_cosine` | **0.9954** |
| `avg_fusion_vs_text_cosine` | **0.9949** |
| `fusion_grows_structure` | **true** |

> patch 向量：融合后导出的 `model_patch.patch_vector` 与单模态 patch 的余弦相似度。

### 4.2 逐样本 — Tile 结构

| ID | text tiles | audio tiles | **fusion tiles** | text dim | audio dim | **fusion dim** |
|----|------------|-------------|------------------|----------|-----------|----------------|
| s1 | 3 | 45 | 43 | 4.0 | 14.61 | 14.61 |
| s2 | 4 | 44 | 43 | 4.0 | 14.05 | 12.95 |
| s3 | 6 | 43 | **52** | 4.0 | 15.71 | **16.25** |
| s4 | 5 | 40 | **52** | 4.0 | 15.16 | **16.25** |
| s5 | 5 | 44 | **53** | 4.0 | 15.16 | **16.80** |

**观察**：
- 纯文本：Tile 完成数 **3–6**，维度恒为 **4.0**（基线维）。
- TTS 音频：Tile 完成数 **40–45**，维度 **~14–16**（结构孵化显著强于文本）。
- 融合：**3/5** 样本 fusion tile 数 **超过** text 与 audio 单模态最大值（s3 +9、s4 +12、s5 +9）。

### 4.3 逐样本 — StemCell 结构

| ID | text cells | audio cells | **fusion cells** | text conn | audio conn | **fusion conn** |
|----|------------|-------------|------------------|-----------|------------|-----------------|
| s1 | 106 | 138 | **147** | 426 | 508 | 468 |
| s2 | 102 | 139 | 133 | 416 | 508 | 456 |
| s3 | 102 | 139 | 138 | 416 | 508 | 460 |
| s4 | 108 | 134 | **140** | 430 | 508 | 466 |
| s5 | 104 | 135 | **140** | 422 | 508 | 466 |

**观察**：
- Stem 细胞数：**3/5** 样本融合后 **高于** 两路单模态（s1、s4、s5）。
- 单样本 incubation 下 `stem_cluster_count` 均为 **1**，`stem_text_audio_sig_cosine` 均为 **0**（cluster 粒度不足，尚不能从 signature 层面比较 Stem 跨模态分化）。

### 4.4 跨模态相似度

| ID | text↔audio patch cos | fusion↔text | fusion↔audio | tile sig text↔audio |
|----|----------------------|---------------|----------------|---------------------|
| s1 | 0.996 | 0.996 | 0.992 | 0.276 |
| s2 | 0.996 | 0.996 | 0.995 | 0.330 |
| s3 | 0.995 | 0.994 | 0.989 | 0.351 |
| s4 | 0.995 | 0.994 | 0.989 | 0.279 |
| s5 | 0.995 | 0.994 | 0.989 | 0.301 |

**观察**：
- **Patch 层**：text 与 audio 高度相关（~0.995），因同一句话语义一致。
- **Tile signature 层**：text↔audio 仅 **0.28–0.35**，结构表征明显分化 → 融合有信息互补空间，而非简单重复。

---

## 5. 结论

1. **TTS 音频经 DL front 进入 Tile 后**，单模态结构规模（~40+ tiles、dim ~15）远高于纯文本（~3–6 tiles、dim 4.0）。
2. **text+audio 联合 Triad 喂入** 在多数样本上产生 **超加性 Tile 增长**（s3/s4/s5），并在部分样本上提升 Stem 细胞数（s1/s4/s5）；`fusion_grows_structure = true`。
3. Patch 向量高相似、Tile signature 低相似，说明 **语义对齐 + 结构互补** 同时存在，符合多模态融合预期。
4. 本次为 **结构 smoke**，未接 fair benchmark / hybrid QA；Stem cluster 需更长 corpus 或多轮 incubation 才能进一步分化。

---

## 6. 限制与后续

| 限制 | 说明 |
|------|------|
| TTS 质量 | espeak 机械音；Piper/edge-tts 待网络恢复后复测 |
| 单句 incubation | 每样本独立 session，非 corpus 级训练 |
| Stem 指标 | 单 cluster，cross-modal sig 无效 |
| QA 未测 | 尚未验证融合对 MCQ/open 准确率的贡献 |

**建议下一步**：
- 下载 Piper `en_US-lessac-medium`，对比 espeak 下 Tile/Stem 增益是否一致。
- 将 text+TTS-audio 双端口接入 `valhalla_hybrid` 开放题 incubation 路径。
- LibriSpeech 配对 corpus 批量跑 fusion，观察 `stem_cluster_count > 1` 是否出现。

---

## 7. English summary

**Goal**: Probe whether TTS-derived audio features fused with text in the Triad (Tile + StemCell) produce super-additive structure vs single-modality baselines.

**Setup**: 5 English sentences → espeak WAV → 384-d mel-band features → `valhalla_tile_stem_tts_fusion` with `VALHALLA_DL_FRONT=1`, 2 optimize cycles.

**Findings**:
- Audio-only drives **~10×** more Tile completions than text-only (40–45 vs 3–6) and raises Tile dimension from 4.0 to ~15.
- Fusion exceeds both modalities on Tile count in **3/5** samples (up to +12 tiles on s4).
- Patch cosine text↔audio ≈ **0.995**; Tile signature cosine ≈ **0.28–0.35** → aligned semantics, divergent structure.
- `fusion_grows_structure: true` in aggregate summary.

**Artifact**: `reports/tts_tile_stem_fusion/fusion_report.json`
