# Yaksha Cipher / 夜叉密码 / 夜叉暗号

<p align="center">
  <a href="#english">English</a> •
  <a href="#中文">中文</a> •
  <a href="#日本語">日本語</a>
</p>

---

<a name="english"></a>
## 🇬🇧 English

**Yaksha Cipher** is a Rust symmetric encryption library with parameterized transformations and stateful AEAD.

### Installation

```toml
[dependencies]
yaksha-cipher = "1.0"
```

### Usage

```rust
use yaksha_cipher::{core, parametric, neural, aead};

let key = b"your_secret_key!";
let data = b"Sensitive data";

// Core mode (basic)
let encrypted = core::encrypt(data, key);
let decrypted = core::decrypt(&encrypted, key);

// Parametric mode (key-derived parameters)
let encrypted = parametric::encrypt(data, key);
let decrypted = parametric::decrypt(&encrypted, key);

// Neural mode (MeshBrain-inspired)
let encrypted = neural::encrypt(data, key);
let decrypted = neural::decrypt(&encrypted, key);

// AEAD mode (authenticated encryption)
let mut cipher = aead::YakshaAead::new(key);
let (ciphertext, tag) = cipher.encrypt(data, b"aad");
```

### Security

| Property | Value |
|----------|-------|
| Avalanche Effect | ~50% |
| Key Sensitivity | ~50% |
| Deterministic | Yes (primitive level) |
| AEAD Support | Yes (via YakshaAead) |

---

<a name="中文"></a>
## 🇨🇳 中文

**夜叉密码** 是一个 Rust 对称加密库，具有参数化变换和有状态 AEAD。

### 安装

```toml
[dependencies]
yaksha-cipher = "1.0"
```

### 使用

```rust
use yaksha_cipher::{core, parametric, neural, aead};

let key = b"your_secret_key!";
let data = b"敏感数据";

// 核心模式（基础）
let encrypted = core::encrypt(data, key);
let decrypted = core::decrypt(&encrypted, key);

// 参数化模式（密钥派生参数）
let encrypted = parametric::encrypt(data, key);
let decrypted = parametric::decrypt(&encrypted, key);

// 神经模式（MeshBrain 启发）
let encrypted = neural::encrypt(data, key);
let decrypted = neural::decrypt(&encrypted, key);

// AEAD 模式（认证加密）
let mut cipher = aead::YakshaAead::new(key);
let (ciphertext, tag) = cipher.encrypt(data, b"aad");
```

### 安全性

| 属性 | 值 |
|------|-----|
| 雪崩效应 | ~50% |
| 密钥敏感性 | ~50% |
| 确定性 | 是（原语层面）|
| AEAD 支持 | 是 |

---

<a name="日本語"></a>
## 🇯🇵 日本語

**夜叉暗号** はパラメータ化変換とステートフルAEADを持つRust対称暗号ライブラリです。

### インストール

```toml
[dependencies]
yaksha-cipher = "1.0"
```

### 使用方法

```rust
use yaksha_cipher::{core, parametric, neural, aead};

let key = b"your_secret_key!";
let data = b"機密データ";

// コアモード（基本）
let encrypted = core::encrypt(data, key);
let decrypted = core::decrypt(&encrypted, key);

// パラメータ化モード（鍵由来パラメータ）
let encrypted = parametric::encrypt(data, key);
let decrypted = parametric::decrypt(&encrypted, key);

// ニューラルモード（MeshBrain着想）
let encrypted = neural::encrypt(data, key);
let decrypted = neural::decrypt(&encrypted, key);

// AEADモード（認証暗号）
let mut cipher = aead::YakshaAead::new(key);
let (ciphertext, tag) = cipher.encrypt(data, b"aad");
```

### セキュリティ

| 特性 | 値 |
|------|-----|
| アバランシェ効果 | ~50% |
| 鍵感度 | ~50% |
| 決定性 | はい（プリミティブレベル）|
| AEADサポート | はい |

---

## License / 许可证 / ライセンス

MIT - [Rogue Intelligence Inc.](https://github.com/Rogue-Intelligence-INC)
