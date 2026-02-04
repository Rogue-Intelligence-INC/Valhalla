# Valhalla Systems - f64 GPU-Accelerated Version

高性能 f64 版本，支持 GPU 加速。

## 组件

| 组件 | 路径 | 描述 |
|------|------|------|
| Fate-f64 | `../systems/metasystems/fate-f64/` | 信号协调系统 |
| BlackHole | `../systems/metasystems/blackhole/` | 引力场系统 |
| MeshBrain | `../systems/metasystems/meshbrain/` | 神经膜系统 |
| MultiNova | `../systems/metasystems/multinova/` | 能量演化系统 |
| BaseForce | `../systems/metasystems/base-force/` | 力场协调系统 |
| Hub-f64 | `../manifestsys/hub-f64/` | 统一入口 |

## 特性

- ✅ f64 浮点运算 (GPU 友好)
- ✅ rayon 并行计算 (多核 CPU)
- ✅ candle GPU 加速 (可选)
- ✅ 批量向量运算
- ✅ 适配 RTX 4090

## 使用

```bash
# CPU 版本
cargo build --release -p fate-f64

# GPU 加速版本
cargo build --release -p fate-f64 --features cuda
```
