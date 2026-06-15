# 夜叉密码 · 严格测试说明 (Strict Testing)

本文档说明如何对 Yaksha Cipher（含 Evolve 模式）进行更严格的统计与合规测试，以及如何对接业界测试套件与竞赛/标准。

---

## 1. 业界标准与竞赛（参考）

| 来源 | 说明 | 夜叉是否参与 |
|------|------|--------------|
| **NIST Lightweight Cryptography** | 轻量级密码标准化，已结束（Ascon 胜出，SP 800-232）。 | 未参与；可参考其评估维度（性能、安全、实现）。 |
| **NIST SP 800-22** | 随机/伪随机数生成器统计测试套件（15 项）。 | **可对接**：用本库生成密文流，喂给 NIST STS 做统计测试。 |
| **NIST CAVP** | 算法验证程序（AES/TDES 等 FIPS 算法）。 | 不适用（夜叉非 FIPS 标准算法）。 |
| **dieharder** | 随机性测试（Marsaglia 等）。 | **已对接**：见下文「外部工具」与 `scripts/run_external_tests.sh`。 |
| **PractRand** | 无上限数据量 RNG 测试。 | **可对接**：将密文流管道输入 `RNG_test stdin`。 |
| **eBACS / CAESAR** | 认证加密性能基准。 | 可作性能对比参考，不提供算法正确性测试。 |
| **Wycheproof** | 已知攻击测试向量（AES-GCM、ChaCha20 等）。 | 思路可借鉴：本库提供 KAT（Known Answer Test）与回归测试。 |

---

## 2. 主门禁：业界外置套件（推荐 / 唯一统计门）

**库内已移除统计向单元测试**；正式门禁请用 Valhalla 根目录：

```bash
./external_bench/run_all.sh        # CPU 版门禁（结果目录 results/cpu_<时间戳>/）
./external_bench/run_all_gpu.sh     # GPU 版门禁（结果目录 results/gpu_<时间戳>/），与 CPU 版独立
```

- **OpenSSL** `speed`：AEAD 吞吐基线。
- **dieharder**：**STS 门禁**（`-d 100,101,102`），使用 **`-t` 与 `-p`** 双样本量（默认各 **500**），任一 `FAILED` / `WEAK` 或非零退出 → 失败。生成流大小须 ≥ `DIEHARDER_MB` 的 **95%**（由脚本导出 `DIEHARDER_MB` 校验）。经典 Diehard 与 `-d 205` 仅 `dieharder_info_*.stdout` 记录。
- **CPU 版**：`yaksha_aead_bench`、`yaksha_test`（smoke）、`yaksha_aead_timing_keys`、`generate_test_data` 均须 **exit 0**。
- **GPU 版**：同上中 bench/timing 在 CPU crate；`yaksha_test_gpu`、`generate_test_data_gpu` 在 GPU crate；OpenSSL / dieharder 规则相同，密文流来自 GPU 生成器。

可调环境变量见 `external_bench/README.md`。两版共用 `bench_common.sh`、`run_dieharder.sh`、`run_openssl_aead.sh`、`collect_report.py`；`report_summary.md` 会标注 `bench_variant.txt`（`cpu` / `gpu`）。

### 2.1 可选：仅冒烟（非统计门）

```bash
cd valhalla-crypto/yaksha-cipher && cargo run --release --bin yaksha_test
```

---

## 3. 外部工具对接

### 3.1 dieharder（已用）

- **安装**：`sudo pacman -S dieharder`（Arch）或各发行版包管理。
- **生成数据**（示例：Neural 模式，50MB，随机明文）：
  ```bash
  cargo run --release --bin generate_test_data -- neural 50 random > yaksha_neural.bin
  ```
- **运行全部测试**：
  ```bash
  dieharder -g 201 -f yaksha_neural.bin -a
  ```
- **仅部分测试（加快）**：例如 `-d 0`（birthdays）、`-d 100`（sts_monobit）、`-d 101`（sts_runs）。

### 3.2 NIST Statistical Test Suite (SP 800-22)

- **下载**：  
  [NIST STS 2.1.2](https://csrc.nist.rip/Projects/Random-Bit-Generation/Documentation-and-Software) → `sts-2_1_2.zip`。
- **编译**：解压后 `make`（需 gcc）。
- **输入**：二进制比特流文件；建议单文件至少 1,000,000 比特（见 NIST 文档）。
- **用法示例**：
  ```bash
  # 生成至少 1M 比特（例如 1MB 密文）
  cargo run --release --bin generate_test_data -- neural 1 random > nist_input.bin
  # 将 nist_input.bin 复制到 STS 的 data/ 或按 STS 文档指定输入
  # 运行 STS（具体命令见 STS 包内 README）
  ./assess 1000000
  ```
- **说明**：STS 输入为单一大文件；本库 `generate_test_data` 输出为原始字节，按位展开后可作为比特流输入（或使用 STS 支持的格式）。

### 3.3 PractRand

- **获取**：<https://sourceforge.net/projects/pracrand/>；编译后得到 `RNG_test`。
- **用法**：将密文流管道传入，例如：
  ```bash
  cargo run --release --bin generate_test_data -- neural 100 random | ./RNG_test stdin
  ```
- **选项**：`-tlmin 1KB` 可加快初期输出；无数据量上限，适合长时间运行。

### 3.4 一键脚本（推荐）

项目内提供 `scripts/run_external_tests.sh`：

- 使用 `generate_test_data` 生成指定模式与大小的密文文件。
- 若已安装 dieharder，则运行 dieharder 全套（或子集）。
- 若检测到 PractRand 的 `RNG_test`，则用管道运行一段数据。
- 结果与建议写入报告目录或 stdout。

```bash
cd valhalla-crypto/yaksha-cipher
chmod +x scripts/run_external_tests.sh
./scripts/run_external_tests.sh neural 20
```

---

## 4. 已知限制与建议

- **统计测试不能替代密码分析**：通过 NIST/dieharder/PractRand 仅说明统计性质，不证明抗攻击强度。
- **Evolve 模式**：`generate_test_data` 支持 `evolve` 模式时，可用相同流程对 Fate 控制输出做统计测试。
- **竞赛**：NIST 轻量级密码竞赛已结束；若未来有新的公开竞赛或基准，可将夜叉作为非标准/研究算法参与对比测试。
- **回归**：修改核心算法或控制派生后，请至少运行 CPU 门禁 `./external_bench/run_all.sh`；若动到 GPU 路径，另跑 `./external_bench/run_all_gpu.sh`（并视情况加长 `DIEHARDER_MB` / PractRand 管道做对比）。

---

**文档版本**: 1.0  
**最后更新**: 2026-03
