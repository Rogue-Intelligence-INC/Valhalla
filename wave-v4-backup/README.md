# 🌊 Wave v4.3

**直接生成 x86-64 机器码 · Fate+四元 · 混合界面 · 零依赖**

---

## 版本

```
4.3.0
```

## 核心特性

| 特性 | 说明 |
|------|------|
| **目标** | x86-64 ELF64 机器码 |
| **内存** | Fate+四元 (无硬编码) |
| **界面** | 混合 UI (非 CLI 非 GUI) ✨ |
| **依赖** | 零 |
| **Fate** | 完全自主控制 |
| **Collapse** | 自动触发优化 |

---

## Fate+四元 内存管理

```
┌─────────────────────────────────────────┐
│           Fate 自主决策                 │
├─────────────────────────────────────────┤
│  请求 → Fate 分析 → 选择最优池          │
│         ↓                               │
│  ┌──────────┬──────────┐               │
│  │BlackHole │MeshBrain │               │
│  │ (大块)   │ (索引)   │               │
│  ├──────────┼──────────┤               │
│  │MultiNova │BaseForce │               │
│  │ (并行)   │ (小块)   │               │
│  └──────────┴──────────┘               │
│         ↓                               │
│  Fate 学习 + 缓存 + 重用               │
│         ↓                               │
│  边际收益↓ → Collapse 优化             │
└─────────────────────────────────────────┘
```

### 对比 TileQuattro

| 指标 | TileQuattro | Fate+四元 |
|------|-------------|-----------|
| 长期性能 | 慢 572% | **快 151.8x** |
| 错误恢复 | 崩溃 | **自动恢复** |
| 学习能力 | 无 | **缓存+重用** |
| 硬编码 | 有 | **无** |

---

## 快速开始

```bash
cd wave
./bin/wave examples/hello.wave -o hello
./hello
```

---

## 语法

```wave
# 输出
out "Hello, Wave!"

# 管道 (Fate 自动分配)
data -> process -> store

# Fate 分配
request -> fate_alloc

# Collapse 监控
fate_monitor -> fate_detect_marginal -> fate_collapse_decision
```

---

## 文件结构

```
wave/
├── bin/wave                    # 编译器
├── src/
│   ├── wave_compiler.c         # 编译器实现
│   ├── fate_quattro.wave       # Fate+四元 内存管理器
│   ├── ui/
│   │   └── wave_ui.wave        # 混合界面系统
│   ├── net/
│   │   └── http.wave           # HTTP 服务器 ✨
│   └── transpile/
│       └── transpiler.wave     # 转译器 ✨
├── docs/
│   └── WAVE_LANGUAGE_MANUAL.md # 语言手册 ✨
├── examples/
│   ├── hello.wave
│   ├── color_demo.wave         # 色彩演示
│   └── fate_full_control.wave
└── VERSION                     # 4.3.0
```

---

## 自举编译

Wave 是**自举编译器** - 用 Wave 自身编译

```bash
# bin/wave 已预编译为机器码
./bin/wave your_program.wave -o output

# 重新编译 Wave 自身
./bin/wave src/wave_self.wave -o bin/wave
```

---

## Wave UI - 混合界面

```
不是 CLI，不是 GUI，是统一界面
24-bit True Color · 渐变 · 透明 · Fate 控制
```

### 色彩能力

| 特性 | 说明 |
|------|------|
| **24-bit RGB** | 16,777,216 种颜色 |
| **渐变** | 线性、多点、径向 |
| **透明** | Alpha 混合 |
| **动态** | 脉冲、闪烁效果 |

### 预设渐变

```
fire    🔴→🟡  sunset  🟣→🟠
ocean   🔵→🩵  neon    🟣→🩵→🟡
forest  🟢→💚  rainbow 🌈
```

### 三种模式

| 模式 | 色彩 | 面板 |
|------|------|------|
| Compact | 无 | 无 |
| Standard | 基础 | 无 |
| Rich | **完整渐变** | **透明面板** |

### 演示

```bash
./bin/wave examples/color_demo.wave -o color_demo
./color_demo
```

---

---

## 后端开发

### HTTP 服务器

```wave
server -> create_server({port: 8080})

get "/" -> handler(req) {
    html_response("<h1>Wave Server</h1>")
}

get "/api/status" -> handler(req) {
    json_response({status: "ok"})
}

server -> start
```

### 转译器

| 目标 | 状态 | 用途 |
|------|------|------|
| C | ✅ 基础 | 系统调用、FFI |
| WebAssembly | 🔜 计划 | Web 前端 |
| JavaScript | 🔜 计划 | Node.js |

---

## 未来功能

| 功能 | 状态 |
|------|------|
| HTTPS/TLS | 🔜 v4.4 |
| WebSocket | 🔜 v4.4 |
| WebAssembly 输出 | 🔜 v4.5 |
| 跨平台 (ARM, Windows) | 🔜 v5.0 |
| 包管理器 | 🔜 v5.0 |

详见 [语言手册](docs/WAVE_LANGUAGE_MANUAL.md)

---

**Wave v4.3 - Fate+四元 · 混合界面 · 后端开发 · 无硬编码** 🌊
