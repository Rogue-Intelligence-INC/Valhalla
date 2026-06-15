# Yaksha Cipher — 行业标准测试报告

> **日期：2026-06-16**  
> 完整版（含 artifact 路径与复现命令）：  
> [`external_bench/results/industry_20260616_013803/YAKSHA_INDUSTRY_TEST_REPORT.md`](../external_bench/results/industry_20260616_013803/YAKSHA_INDUSTRY_TEST_REPORT.md)

| 字段 | 内容 |
|------|------|
| 测试日期 | **2026-06-16** |
| Git | `294ac158` |
| 环境 | Linux x86_64 · RTX 4090 (Vulkan) |
| 配置 | `core` 模式 · 64MB 流 · dieharder t/p=1000 |

## 结果总览

| 测试 | CPU | GPU |
|------|-----|-----|
| dieharder STS gate (d100–102) | **PASS** | **PASS** |
| PractRand @64MB | **PASS** (182 tests) | **PASS** (182 tests) |
| Harness exit codes | **0** | **0** |
| 64MB 流 MD5 | `7ca0803f524c4942c9bfcf6edd89cb58` | **同左** |

**Industry Gate：PASS（CPU + GPU）**

## 架构（被测版本）

- **Shell：** Ocean₁ → core body → Ocean₂  
- **GPU：** 仅洋流 XOR 卸载；body / cascade 在 CPU  
- **AEAD：** KDF + Shell + YakshaMac  

## 性能参考

Yaksha AEAD（shell 含洋流）：约 **3–4 MB/s**（4KB–16MB 档位）。

## 说明

- dieharder 扩展项（bitstream、byte_distribution 等）对密文流 `FAILED` 为预期，非 gate。  
- 原语层 `crypto_analysis` 不在本 industry 报告范围内。

---

*报告版本 1.0 · 2026-06-16*
