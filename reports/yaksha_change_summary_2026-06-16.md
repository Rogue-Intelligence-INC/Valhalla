# Yaksha Cipher — 变更总结（2026-06-16）

> 行业标准测试报告：[`external_bench/results/industry_20260616_adaptive/YAKSHA_INDUSTRY_TEST_REPORT.md`](../external_bench/results/industry_20260616_adaptive/YAKSHA_INDUSTRY_TEST_REPORT.md)

## 架构

```text
AEAD:  YakshaKdf → enc_key + mac_key → Shell → YakshaMac
Shell: Ocean₁(2D 自适应) → core body → Ocean₂
GPU:   仅洋流 XOR 卸载；body / cascade 在 CPU
```

## 主要变更

1. **2D 洋流全局化** — `frac_container` + `ocean_pipeline`，shell 统一外壳
2. **GPU/CPU 对齐** — `shell_gpu`、body 委托 CPU、`cpu_gpu_compat` KAT
3. **自适应 mask v3** — `OceanMaskContext` 预计算 seed；`R(x,y)∈[10,28]`；rayon 并行
4. **Industry gate** — dieharder STS + PractRand + OpenSSL 基线（`external_bench/`）

## 测试结果（2026-06-16，严格 gate）

| 项 | 结果 |
|----|------|
| dieharder STS d100–102 (64MB, t/p=1000) | **PASS** (CPU + GPU) |
| PractRand @64MB | **182 tests, no anomalies** |
| CPU/GPU 64MB 流 MD5 | **一致** |
| AEAD 16MB 吞吐 | **~60 MB/s**（v2 固定 48 轮约 ~4 MB/s） |

## 目录

| 路径 | 说明 |
|------|------|
| `valhalla-crypto/yaksha-cipher/` | CPU 主 crate |
| `valhalla-crypto/yaksha-cipher-gpu/` | GPU 洋流 XOR |
| `external_bench/` | 行业测试 harness |

## 复现

```bash
./external_bench/run_all.sh
./external_bench/run_all_gpu.sh
cargo run --release -p yaksha-cipher-gpu --bin cpu_gpu_compat
```

**注意：** adaptive v3 密文与旧版固定 48 轮洋流不兼容。
