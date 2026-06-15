# Yaksha-Cipher-GPU 加密评估报告

**生成日期**: 2026-01-30  
**项目状态**: 开发中  
**当前版本**: 波粒转换 + 量子涨落时序递进

---

## 1. 项目概述

### 1.1 目标
开发 GPU 加速的 Yaksha 加密算法，利用 wgpu/Vulkan 实现高性能加密，同时保持良好的密码学特性。

### 1.2 架构
```
┌─────────────────────────────────────────────────────────────┐
│                    Yaksha-Cipher-GPU                         │
├─────────────────────────────────────────────────────────────┤
│  Phase 1: 参数计算 (GPU)                                     │
│    └── 从密钥派生加密参数                                    │
├─────────────────────────────────────────────────────────────┤
│  Phase 2: 字节变换 (CPU 并行)                                │
│    └── Rayon 多线程逐字节变换                                │
├─────────────────────────────────────────────────────────────┤
│  Phase 3: 扩散层 (GPU)                                       │
│    └── 波粒转换 / 量子干涉 / 超级位置                        │
└─────────────────────────────────────────────────────────────┘
```

### 1.3 支持模式
| 模式 | 文件 | 说明 |
|------|------|------|
| Parametric | `dimension_gpu.rs` | 基于非整数维度的参数变换 |
| Neural | `quantum_gpu.rs` | 量子模拟 + MeshBrain 风格变换 |

---

## 2. 扩散机制演进

### 2.1 版本历史

| 版本 | 机制 | 雪崩效应 | Dieharder 结果 |
|------|------|----------|----------------|
| V1 原始 | CPU 顺序级联 | ~30-40% | 未测试 |
| V2 随机级联 | GPU random_cascade | ~40-50% | 问题：单缓冲并发 |
| V3 量子干涉 | InterferenceCascade | ~50% | 72P/16W/26F |
| V4 超级位置 | Superposition 单独 | ~50%(8轮) | 60P/22W/32F |
| **V5 波粒转换** | **WaveParticle (当前)** | **~50%(1轮)** | **80P/17W/17F** |
| V6 串联方案 | SP → RC | ~50% | 77P/21W/16F |

### 2.2 当前最优方案：波粒转换

```rust
// wave_particle.rs 核心设计
波粒转换 = 量子涨落 + 时序递进 + 波干涉 + 粒子非线性 + 量子隧穿

每轮执行:
  for step in 0..log2(n):
    1. 量子涨落: 位置相关随机扰动
    2. 波传播: 距离 step 的邻居干涉
    3. 粒子混合: 非线性乘法变换
    4. 量子隧穿: 随机远距离跳跃
```

**特点**:
- 1 轮即达 ~50% 雪崩效应
- 时序步数 = log2(n)，自适应数据大小
- 双缓冲设计，避免并发问题

---

## 3. Dieharder 测试详情

### 3.1 波粒转换版本 (V5) - 当前最优

| 统计 | 数量 |
|------|------|
| **PASSED** | 80 |
| WEAK | 17 |
| FAILED | 17 |

**通过的关键测试**:
- ✅ 全部 `sts_serial` (1-16)
- ✅ 全部 `rgb_bitdist` (1-12)
- ✅ `diehard_birthdays`, `diehard_runs`, `diehard_dna`
- ✅ `rgb_permutations` (大部分)
- ✅ `rgb_kstest_test`

**失败项目**:
| 测试 | 原因分析 |
|------|----------|
| `diehard_opso` | 经典难关，对重叠模式敏感 |
| `marsaglia_tsang_gcd` | 极端严格，许多算法都失败 |
| `rgb_lagged_sum` (部分) | 4k-1 模式周期性 |
| `dab_*` | 需要更大数据量 |

### 3.2 对比其他加密算法

| 算法 | Dieharder 通过率 | 备注 |
|------|-----------------|------|
| AES-CTR | ~95% | 行业标准 |
| ChaCha20 | ~95% | 现代流密码 |
| **Yaksha GPU V5** | **~70%** | 实验性，仍在优化 |
| RC4 | ~60% | 已弃用 |

---

## 4. 模块清单

### 4.1 核心模块

| 文件 | 状态 | 说明 |
|------|------|------|
| `gpu_context.rs` | ✅ 稳定 | wgpu 上下文管理 |
| `dimension_gpu.rs` | ✅ 稳定 | Parametric 模式 |
| `quantum_gpu.rs` | ✅ 稳定 | Neural 模式 + WaveParticle 集成 |
| `wave_particle.rs` | ✅ **当前最优** | 波粒转换扩散 |

### 4.2 实验模块

| 文件 | 状态 | 说明 |
|------|------|------|
| `superposition.rs` | 🔬 实验 | 超级位置（远距离链接）|
| `interference_cascade.rs` | 🔬 实验 | 量子干涉级联 |
| `superposition_cascade.rs` | 🔬 实验 | 量子叠加级联 |
| `random_cascade.rs` | ⚠️ 问题 | 单缓冲并发问题 |

### 4.3 测试工具

| 文件 | 说明 |
|------|------|
| `bin/gpu_test.rs` | 加密正确性 + 雪崩测试 |
| `bin/generate_gpu_data.rs` | 生成 Dieharder 测试数据 |
| `bin/gpu_bench.rs` | 性能基准测试 |

---

## 5. 性能指标

### 5.1 吞吐量 (RTX 4090)

| 模式 | GPU | CPU | 加速比 |
|------|-----|-----|--------|
| Parametric | 1.4 GB/s | 560 MB/s | 2.5x |
| Neural | 1.3 GB/s | 680 MB/s | 1.9x |

### 5.2 雪崩效应

| 数据大小 | 1轮 | 2轮 | 4轮 |
|----------|-----|-----|-----|
| 256 B | 50.2% | 49.2% | 49.2% |
| 1 KB | 51.1% | 49.6% | 50.6% |
| 4 KB | 50.3% | 50.0% | 50.0% |
| 16 KB | 49.7% | 50.1% | 49.8% |

**结论**: 1 轮即达理想 50% 扩散

---

## 6. 已知问题

### 6.1 密码学问题

| 问题 | 严重度 | 状态 |
|------|--------|------|
| `rgb_lagged_sum` 部分失败 | 中 | 研究中 |
| `marsaglia_tsang_gcd` 失败 | 低 | 可接受 |
| 可逆性未完全验证 | 高 | 需要测试 |

### 6.2 技术问题

| 问题 | 严重度 | 状态 |
|------|--------|------|
| `random_cascade.rs` 并发问题 | 高 | 已弃用，使用 wave_particle |
| 小数据 (<16字节) 直接返回 | 低 | 设计决策 |

---

## 7. 下一步计划

### 7.1 短期
- [ ] 完善可逆性测试（加密/解密往返）
- [ ] 优化 `rgb_lagged_sum` 相关问题
- [ ] 清理废弃的实验模块

### 7.2 中期
- [ ] 实现完整的 AEAD 模式
- [ ] 添加 IV/Nonce 支持
- [ ] 性能优化（减少 GPU-CPU 数据传输）

### 7.3 长期
- [ ] 安全审计
- [ ] 对比更多标准测试套件 (NIST SP 800-22)
- [ ] 文档和 API 稳定化

---

## 8. 结论

**当前状态**: 波粒转换版本 (V5) 是目前最优方案

**优势**:
- 1 轮达到 50% 雪崩效应
- 通过大部分 Dieharder 测试
- GPU 加速实现高吞吐量

**不足**:
- 部分统计测试失败
- 需要更多密码学分析
- 可逆性需要验证

**建议**: 继续使用波粒转换版本进行开发，同时研究 `rgb_lagged_sum` 问题的根本原因。

---

## 附录 A: 文件结构

```
yaksha-cipher-gpu/src/
├── lib.rs                    # 模块导出
├── gpu_context.rs            # GPU 上下文
├── dimension_gpu.rs          # Parametric 模式
├── quantum_gpu.rs            # Neural 模式
├── wave_particle.rs          # ★ 波粒转换（当前最优）
├── superposition.rs          # 超级位置
├── interference_cascade.rs   # 量子干涉
├── superposition_cascade.rs  # 量子叠加
├── random_cascade.rs         # 随机级联（已弃用）
├── vector_link.rs            # 向量链接
└── bin/
    ├── gpu_test.rs           # 测试工具
    ├── generate_gpu_data.rs  # 数据生成
    └── gpu_bench.rs          # 性能测试
```

## 附录 B: 波粒转换算法详解

```
输入: data[n], key, rounds

for round in 0..rounds:
  max_steps = log2(n)
  
  for step in 0..max_steps:
    parallel for idx in 0..n:
      
      // 1. 量子涨落
      fluct = hash(seed, idx, step)
      val = val XOR rotate(fluct, fluct % 31)
      
      // 2. 波传播 (距离 = step + 1)
      left = data[(idx - step - 1) % n]
      right = data[(idx + step + 1) % n]
      val = val XOR rotate(left, phase_l) XOR rotate(right, phase_r)
      
      // 3. 粒子非线性
      particle = val XOR (val >> 17)
      mixed = particle * 0x85EBCA6B
      val = val XOR (mixed >> 13)
      
      // 4. 量子隧穿
      jump_dist = fluct % (n/2)
      jump_val = data[(idx + jump_dist) % n]
      val = val XOR rotate(jump_val, fluct >> 8 % 31)
      
      output[idx] = val
    
    swap(data, output)

输出: data[n]
```
