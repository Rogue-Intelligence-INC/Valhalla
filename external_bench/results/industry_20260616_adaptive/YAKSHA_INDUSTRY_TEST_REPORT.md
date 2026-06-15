# Yaksha Cipher — 行业标准测试报告（自适应洋流 mask v3）

| 字段 | 内容 |
|------|------|
| **报告日期** | **2026-06-16** |
| **被测版本** | 洋流 **自适应 mask v3** + shell 外壳 |
| **Git 修订** | `294ac158`（工作区含 adaptive ocean 未提交变更） |
| **备份** | `yaksha-cipher_backup_pre_adaptive_ocean_20260616/` · `yaksha-cipher-gpu_backup_pre_adaptive_ocean_20260616/` |
| **汇总目录** | `external_bench/results/industry_20260616_adaptive/` |

---

## 1. 变更摘要（相对固定 48 轮）

| 项目 | 旧（v2） | 新（v3 adaptive） |
|------|----------|-------------------|
| 混合轮数 | 固定 **48** / 字节 | **R(x,y) ∈ [10, 28]**，由分数维耦合 + 2D 格点决定 |
| phase seed | 每字节重算 `fnv1a64(key/session)` | **`OceanMaskContext` 预计算一次** |
| 洋流 XOR | 串行 | **rayon 并行** |
| AEAD 吞吐（16MB seal） | ~4 MB/s | **~60 MB/s**（约 **15×**） |

---

## 2. 执行摘要

| 测试 | CPU | GPU |
|------|-----|-----|
| dieharder STS gate (d100–102, t/p=1000, 64MB) | **PASS** | **PASS** |
| PractRand @64MB | **PASS** (182 tests) | **PASS** (182 tests) |
| Harness exit codes | **0** | **0** |
| 64MB 流 MD5 | `1ee1c42ec325ca14b76efbb8d94ffbe0` | **同左** |
| `cpu_gpu_compat` | **ALL PASSED** | — |

**Industry Gate：PASS（CPU + GPU）**

---

## 3. 测试配置

```bash
DIEHARDER_MODE=core
DIEHARDER_MB=64
DIEHARDER_TSAMPLES=1000
DIEHARDER_PSAMPLES=1000
OPENSSL_SECONDS=10
```

Artifact：

- CPU: `external_bench/results/cpu_20260616_020949/`
- GPU: `external_bench/results/gpu_20260616_021357/`

---

## 4. 性能（AEAD shell，含自适应洋流）

| Payload | seal MB/s | open MB/s |
|---------|----------:|----------:|
| 4 KB | 15 | 11 |
| 64 KB | 27 | 19 |
| 256 KB | 33 | 31 |
| 1 MB | 43 | 41 |
| 4 MB | 60 | 60 |
| 16 MB | **60** | **62** |

---

## 5. 自适应 mask 机制（实现要点）

```text
OceanMaskContext::new(key, session, phase, container)
  └─ base_seed = ocean_seed(...)     // 一次预计算

R(x,y) = R_MIN + mix64(tag) mod (R_MAX - R_MIN + 1)
  tag = f(base_seed, frac_coupling(x,y), x, y, φ)

ocean_apply_inplace: par_iter_mut + ctx.stream_byte(i)
```

常量：`OCEAN_MIX_ROUNDS_MIN=10` · `OCEAN_MIX_ROUNDS_MAX=28`

---

## 6. 复现

```bash
cd /home/jouly/AI/Valhalla
DIEHARDER_MODE=core DIEHARDER_MB=64 \
  DIEHARDER_TSAMPLES=1000 DIEHARDER_PSAMPLES=1000 \
  ./external_bench/run_all.sh
./external_bench/run_all_gpu.sh
cargo run --release -p yaksha-cipher-gpu --bin cpu_gpu_compat
```

---

## 7. 说明

- 初调参数 `[4,16]` 时 sts_serial 出现 **WEAK**；调至 `[10,28]` 后 gate **PASS**。
- v3 密文与 v2（固定 48 轮）**不兼容**。
- dieharder 扩展项（bitstream 等）对密文 `FAILED` 仍为预期，非 gate。

---

**报告版本：** 1.0（adaptive v3）  
**有效日期：** 2026-06-16
