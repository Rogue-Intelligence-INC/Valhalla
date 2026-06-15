# Yaksha Evolve 规范 (Evolution Specification)

**版本**: 1.0  
**状态**: 规范已定，实现按此测试。  
**约定**: 多次进化后形成的 Fate 控制协议与数据格式，写入本规范；实现须通过规范符合性测试。

---

## 1. 适用范围

- **Evolve 协议**：由 Fate（YakshaKdf）自动产生控制信号，选择基础变换与级联方式，实现“呼吸步数”与方向由 Fate 触发的加密/解密。
- **不覆盖**：原有稳定 API（`core` / `parametric` / `neural` / `aead`）的独立行为仍以各自文档与测试为准。

---

## 2. 术语

| 术语 | 含义 |
|------|------|
| **master_key** | 用户主密钥，任意非空字节序列。 |
| **nonce** | 会话/消息区分符，任意字节序列（建议唯一）。 |
| **Fate** | 本规范中指 YakshaKdf 的派生与状态演化。 |
| **Control** | FateYakshaControl，单次加密使用的模式与参数。 |
| **呼吸步数 (breaths)** | 级联应用的离散步数，取值 [1, 16]。 |

---

## 3. 控制派生（Control Derivation）

- **初始状态**：`KDF = YakshaKdf::new(master_key)`。
- **会话吸收**：`KDF.derive_key(nonce)`（必须执行，用于会话区分）。
- **控制派生**：`control_key = KDF.derive_key(b"yaksha_control")`；`domain = KDF.domain()`。
- **Control 各字段由 control_key 与 domain 确定**（实现必须与下表一致）：

| 字段 | 来源 | 取值/范围 |
|------|------|-----------|
| **mode** | control_key[0] % 4 | 0=Neural, 1=Parametric, 2=NeuralThenParametric, 3=ParametricThenNeural |
| **cascade_pattern** | control_key[1] % 3 | 0=ForwardFeistel, 1=Bidirectional, 2=MultiRound(2+(control_key[2]%5)) |
| **breaths** | 1 + (control_key[3] & 0x0F) | [1, 16] |
| **geom_level** | control_key[4] | [0, 255] |
| **neural_level** | control_key[5] | [0, 255] |
| **prg_level** | control_key[6] | [0, 255] |
| **label** | domain 枚举值转 u8 | 当前 KDF 域标识 |

- **确定性**：相同 (master_key, nonce) 必须得到相同的 FateYakshaControl。

---

## 4. 加密流程（Encrypt）

1. 使用 master_key 与 nonce 按 §3 得到 **control**。
2. **Base 变换**（按 control.mode，使用 master_key）：
   - Neural → `neural::encrypt(plaintext, master_key)`
   - Parametric → `parametric::encrypt(plaintext, master_key)`
   - NeuralThenParametric → `parametric::encrypt(neural::encrypt(plaintext, master_key), master_key)`
   - ParametricThenNeural → `neural::encrypt(parametric::encrypt(plaintext, master_key), master_key)`
3. **Fate 级联**（对上述结果 in-place，使用 master_key 与 control）：
   - **ForwardFeistel**：执行 `cascade_encrypt` 共 **breaths** 次。
   - **Bidirectional**：执行 **breaths** 次（每次：`cascade_encrypt` 一次，`cascade_decrypt` 一次）。
   - **MultiRound(extra)**：总轮数 = max(2, breaths + extra)，每次 `cascade_encrypt`。
4. 输出：**(ciphertext, control)**。解密方必须持有同一 **control**（或其规范编码）。

---

## 5. 解密流程（Decrypt）

1. 输入：ciphertext, master_key, nonce, **control**（与加密时一致）。
2. **Fate 级联之逆**（对 ciphertext 的副本 in-place）：
   - **ForwardFeistel**：执行 `cascade_decrypt` 共 **breaths** 次。
   - **Bidirectional**：执行 **breaths** 次（每次：`cascade_encrypt` 一次，`cascade_decrypt` 一次）。
   - **MultiRound(extra)**：总轮数 = max(2, breaths + extra)，每次 `cascade_decrypt`。
3. **Base 逆变换**（按 control.mode）：对结果做 neural/parametric 的对应 decrypt，顺序与 encrypt 相反。
4. 输出：plaintext。

---

## 6. 控制信号的规范编码（写入/持久化）

- **Control 的规范字节表示**用于与密文一起存储或传输，便于“写入后再测试”或离线解密。
- **格式**：16 字节，顺序如下（与 FateYakshaControl 一一对应）：

| 字节下标 | 内容 |
|----------|------|
| 0 | mode 枚举值 (0–3) |
| 1 | cascade_pattern 枚举值 (0=Forward, 1=Bidirectional, 2=MultiRound)；若为 MultiRound，与字节 2 一起解析 |
| 2 | MultiRound 时 extra；否则保留 0 |
| 3 | breaths (1–16) |
| 4 | geom_level |
| 5 | neural_level |
| 6 | prg_level |
| 7 | label |
| 8–15 | 保留（填 0），供将来扩展 |

- **解码**：从上述 16 字节还原 FateYakshaControl；若字节 1 == 2，则 cascade_pattern = MultiRound(字节2)，否则字节 2 忽略。

---

## 7. 符合性要求（Conformance）

实现必须满足：

1. **Roundtrip**：对任意 (master_key, nonce, plaintext)，encrypt 得到 (ciphertext, control)，decrypt(ciphertext, master_key, nonce, control) 必须等于 plaintext。
2. **控制确定性**：相同 (master_key, nonce) 必须产生相同的 control。
3. **会话区分**：相同 plaintext 与 master_key、不同 nonce 必须产生不同 ciphertext（在统计上应几乎全部不同）。
4. **规范编码**：FateYakshaControl 的 16 字节编码与解码满足 §6；解码后的 control 用于解密必须与加密时行为一致。
5. **覆盖所有 mode 与 cascade_pattern**：至少对 4 种 mode × 3 种 cascade_pattern 的组合各做一次 roundtrip 验证（MultiRound 可取一固定 extra）。

---

## 8. 测试入口

- **规范符合性测试**：`cargo test evolve_spec_conformance`
  - 覆盖：4×3 种 (mode, cascade_pattern) roundtrip、控制确定性、会话区分、控制 16 字节编解码、存储 control 后解密。
- 建议：修改控制派生或加密/解密流程后，必须通过上述测试并视情况更新本文档版本号。

---

**文档结束**
