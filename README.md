# Valhalla / 瓦尔哈拉 Research Platform

<p align="center">
  <strong>🏔️ AI incubation · Hybrid memory routing · Multimodal structure fusion</strong><br>
  <strong>AI 孵化 · 混合记忆路由 · 多模态结构融合</strong>
</p>

<p align="center">
  <a href="#valhalla-english">English</a> •
  <a href="#valhalla-中文">中文</a> •
  <a href="#yaksha-cipher">Yaksha Cipher</a>
</p>

---

<a name="valhalla-english"></a>
## 🇬🇧 Valhalla — Hybrid Memory & Multimodal Fusion

**Organization**: [Rogue Intelligence Inc.](https://github.com/Rogue-Intelligence-INC)

Valhalla is an **AI research platform** for structure-driven incubation: memory bodies (Tile / StemCell / Fate) fuse with companion LMs via hybrid routing—not a standalone foundation model.

### Hybrid 2.0 (Fair Benchmark, 145 questions)

Primary artifact: `reports/hybrid_valhalla_lm/hybrid_v4_mcq_patch_strengthen_final.json`

| Metric | Result |
|--------|--------|
| **Total accuracy** | **117/145 (80.69%)** |
| Open questions | 57/61 (93.44%) |
| MCQ | 28/46 (60.87%) |
| Numeric | 32/38 (84.21%) |

Baseline Qwen2.5-0.5B zero-shot: 103/145 (71.03%). Structure memory alone: 59/145 (40.69%).

```bash
RUSTFLAGS='-L /opt/cuda/lib64' cargo build -p hub-f64 --release --bin valhalla_hybrid
./target/release/valhalla_hybrid --phase test --merge-mode deploy \
  --model checkpoints/valhalla_patched/Qwen2.5-0.5B-Instruct_valhalla_s0.08 \
  --out reports/hybrid_valhalla_lm/hybrid_v4_mcq_patch_strengthen_final.json
```

Docs: [`docs/hybrid_2.0/`](docs/hybrid_2.0/) · [`docs/README.md`](docs/README.md) · [`docs/archive/`](docs/archive/)

### Multimodal Fusion Benchmark (NEW)

Text + TTS audio → DL front → **Tile + StemCell Triad** structure fusion.

```bash
RUSTFLAGS='-L /opt/cuda/lib64' cargo build -p hub-f64 --release --bin valhalla_multimodal_fusion_bench
VALHALLA_DL_FRONT=1 python3 tools/valhalla_dl_front/run_multimodal_fusion_bench.py --cycles 3
```

| Gate | Result |
|------|--------|
| Tile superadditivity (12 pairs) | **58.3%** pass |
| Stem superadditivity | **91.7%** pass |
| Modality complementarity | **0.285** pass |
| Audio/text tile ratio | **10.9×** |
| Corpus batch fusion | **6.7 tiles/pair** |
| **Overall** | **PASS** |

Report: [`reports/tts_tile_stem_fusion/MULTIMODAL_FUSION_BENCH.md`](reports/tts_tile_stem_fusion/MULTIMODAL_FUSION_BENCH.md)

Open-question memory pilot (text vs text+TTS):

```bash
VALHALLA_MULTIMODAL_FUSION=1 python3 tools/fair_benchmark/run_multimodal_memory_pilot.py
```

### Repository layout

```
Valhalla/
├── manifestsys/hub-f64/     # Hybrid pipeline, DL front, Tile/Stem ingress
├── tools/valhalla_dl_front/ # Multimodal encoder sidecar + fusion bench
├── tools/fair_benchmark/    # Fair QA benchmark harness
├── docs/hybrid_2.0/         # Investor / technical doc pack
├── s-wind-sim/              # S-wind symbolic discovery engine
└── valhalla-crypto/         # Yaksha Cipher (see below)
```

---

<a name="valhalla-中文"></a>
## 🇨🇳 Valhalla — 混合记忆与多模态融合

**组织**：[Rogue Intelligence Inc.](https://github.com/Rogue-Intelligence-INC)

Valhalla 是 **AI 孵化研究平台**：结构记忆体（Tile / StemCell / Fate）与 companion LM 通过混合路由融合——不是独立基座大模型。

### Hybrid 2.0（Fair Benchmark，145 题）

主 artifact：`reports/hybrid_valhalla_lm/hybrid_v4_mcq_patch_strengthen_final.json`

| 指标 | 结果 |
|------|------|
| **总准确率** | **117/145（80.69%）** |
| 开放题 | 57/61（93.44%） |
| 选择题 MCQ | 28/46（60.87%） |
| 数值题 | 32/38（84.21%） |

Qwen2.5-0.5B 零样本基线：103/145（71.03%）。纯结构记忆：59/145（40.69%）。

文档：[`docs/hybrid_2.0/`](docs/hybrid_2.0/)

### 多模态融合基准（新）

文本 + TTS 语音 → DL front → **Tile + StemCell Triad** 结构融合。

| 门禁 | 结果 |
|------|------|
| Tile 超加性（12 对） | **58.3%** 通过 |
| Stem 超加性 | **91.7%** 通过 |
| 模态互补性 | **0.285** 通过 |
| 音频/文本 Tile 比 | **10.9×** |
| 语料批量融合 | **6.7 tiles/对** |
| **总评** | **PASS** |

报告：[`reports/tts_tile_stem_fusion/MULTIMODAL_FUSION_BENCH.md`](reports/tts_tile_stem_fusion/MULTIMODAL_FUSION_BENCH.md)

---

<a name="yaksha-cipher"></a>

# Yaksha Cipher / 夜叉密码 / 夜叉暗号

<p align="center">
  <strong>🔐 Parameterized Symmetric Encryption with Stateful AEAD Protocol</strong>
</p>

<p align="center">
  <a href="#english">English</a> •
  <a href="#中文">中文</a> •
  <a href="#日本語">日本語</a>
</p>

---

<a name="english"></a>
## 🇬🇧 English

### Overview

**Yaksha Cipher** is a novel symmetric encryption framework featuring:

- **Parameterized Transformations**: Key-derived parameters control transformation behavior
- **Neural-Inspired Mode**: MeshBrain physics-inspired encryption
- **Stateful AEAD Protocol**: Session-differentiated authenticated encryption
- **GPU Acceleration**: Up to 1.4 GB/s throughput via wgpu/Vulkan

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                 Yaksha Cipher Protocol                      │
├─────────────────────────────────────────────────────────────┤
│  YakshaKdf (Key Derivation)    ← Session differentiation    │
│           ↓                                                 │
│  Yaksha Encryption             ← Deterministic primitive    │
│           ↓                                                 │
│  YakshaMac (Authentication)    ← Integrity verification     │
└─────────────────────────────────────────────────────────────┘
```

### Modules

| Module | Description |
|--------|-------------|
| `core` | Basic encryption primitive |
| `parametric` | Key-derived transformation parameters |
| `neural` | MeshBrain-inspired transformations |
| `aead` | Authenticated encryption protocol |

### Quick Start

```rust
use yaksha_cipher::{parametric, neural};

let key = b"secret_key_1234!";
let plaintext = b"Hello, Yaksha!";

// Parametric mode
let ciphertext = parametric::encrypt(plaintext, key);
let decrypted = parametric::decrypt(&ciphertext, key);

// Neural mode
let ciphertext = neural::encrypt(plaintext, key);
let decrypted = neural::decrypt(&ciphertext, key);
```

### Performance

| Mode | CPU Throughput | GPU Throughput |
|------|----------------|----------------|
| Core | ~1000 MB/s | - |
| Parametric | ~650 MB/s | ~1.4 GB/s |
| Neural | ~800 MB/s | ~1.3 GB/s |

### Security Properties

- **Avalanche Effect**: ~50% (near-optimal)
- **Key Sensitivity**: ~50%
- **Deterministic Primitive**: System-level differentiation via stateful KDF

---

<a name="中文"></a>
## 🇨🇳 中文

### 概述

**夜叉密码 (Yaksha Cipher)** 是一个创新的对称加密框架，特点包括：

- **参数化变换**：密钥派生参数控制变换行为
- **神经网络启发模式**：基于 MeshBrain 物理模型的加密
- **有状态 AEAD 协议**：会话差异化的认证加密
- **GPU 加速**：通过 wgpu/Vulkan 达到 1.4 GB/s 吞吐量

### 架构

```
┌─────────────────────────────────────────────────────────────┐
│                   夜叉密码协议                                │
├─────────────────────────────────────────────────────────────┤
│  YakshaKdf (密钥派生)           ← 会话差异化                   │
│           ↓                                                 │
│  Yaksha 加密                   ← 确定性原语                   │
│           ↓                                                 │
│  YakshaMac (认证)              ← 完整性验证                   │
└─────────────────────────────────────────────────────────────┘
```

### 模块

| 模块 | 描述 |
|------|------|
| `core` | 基础加密原语 |
| `parametric` | 参数化变换模式 |
| `neural` | 神经网络启发变换 |
| `aead` | 认证加密协议 |

### 快速开始

```rust
use yaksha_cipher::{parametric, neural};

let key = b"secret_key_1234!";
let plaintext = b"你好，夜叉！";

// 参数化模式
let ciphertext = parametric::encrypt(plaintext, key);
let decrypted = parametric::decrypt(&ciphertext, key);

// 神经模式
let ciphertext = neural::encrypt(plaintext, key);
let decrypted = neural::decrypt(&ciphertext, key);
```

### 性能

| 模式 | CPU 吞吐量 | GPU 吞吐量 |
|------|-----------|-----------|
| Core | ~1000 MB/s | - |
| Parametric | ~650 MB/s | ~1.4 GB/s |
| Neural | ~800 MB/s | ~1.3 GB/s |

### 安全特性

- **雪崩效应**：~50%（接近理想值）
- **密钥敏感性**：~50%
- **确定性原语**：通过有状态 KDF 实现系统级差异化

---

<a name="日本語"></a>
## 🇯🇵 日本語

### 概要

**夜叉暗号 (Yaksha Cipher)** は革新的な対称暗号フレームワークです：

- **パラメータ化変換**：鍵由来のパラメータが変換動作を制御
- **ニューラル着想モード**：MeshBrain物理モデルに基づく暗号化
- **ステートフルAEADプロトコル**：セッション差別化された認証暗号
- **GPUアクセラレーション**：wgpu/Vulkanで最大1.4 GB/sスループット

### アーキテクチャ

```
┌─────────────────────────────────────────────────────────────┐
│                   夜叉暗号プロトコル                          │
├─────────────────────────────────────────────────────────────┤
│  YakshaKdf (鍵導出)            ← セッション差別化             │
│           ↓                                                 │
│  Yaksha暗号化                  ← 決定性プリミティブ            │
│           ↓                                                 │
│  YakshaMac (認証)              ← 完全性検証                   │
└─────────────────────────────────────────────────────────────┘
```

### モジュール

| モジュール | 説明 |
|-----------|------|
| `core` | 基本暗号プリミティブ |
| `parametric` | パラメータ化変換モード |
| `neural` | ニューラルネットワーク着想変換 |
| `aead` | 認証暗号プロトコル |

### クイックスタート

```rust
use yaksha_cipher::{parametric, neural};

let key = b"secret_key_1234!";
let plaintext = b"こんにちは、夜叉！";

// パラメータ化モード
let ciphertext = parametric::encrypt(plaintext, key);
let decrypted = parametric::decrypt(&ciphertext, key);

// ニューラルモード
let ciphertext = neural::encrypt(plaintext, key);
let decrypted = neural::decrypt(&ciphertext, key);
```

### パフォーマンス

| モード | CPUスループット | GPUスループット |
|--------|----------------|----------------|
| Core | ~1000 MB/s | - |
| Parametric | ~650 MB/s | ~1.4 GB/s |
| Neural | ~800 MB/s | ~1.3 GB/s |

### セキュリティ特性

- **アバランシェ効果**：~50%（ほぼ理想値）
- **鍵感度**：~50%
- **決定性プリミティブ**：ステートフルKDFによるシステムレベルの差別化

---

## Project Structure / 项目结构 / プロジェクト構造

```
Valhalla/
├── valhalla-crypto/
│   ├── yaksha-cipher/        # Core library / 核心库 / コアライブラリ
│   └── yaksha-cipher-gpu/    # GPU acceleration / GPU加速 / GPUアクセラレーション
├── systems/                  # System implementations / 系统实现 / システム実装
├── s-wind-sim/              # Signal processing / 信号处理 / 信号処理
└── manifestsys/             # Manifest system / 清单系统 / マニフェストシステム
```

## Requirements / 依赖 / 依存関係

- Rust 1.70+
- rayon (CPU parallelism)
- wgpu 0.19 (optional, GPU)

## License / 许可证 / ライセンス

MIT

## Organization / 组织 / 組織

[Rogue Intelligence Inc.](https://github.com/Rogue-Intelligence-INC)

---

<p align="center">
  <strong>Yaksha Cipher</strong> - Advanced Symmetric Cryptography<br>
  <strong>夜叉密码</strong> - 先进对称加密<br>
  <strong>夜叉暗号</strong> - 先進的対称暗号
</p>
