# 🌊 Wave v4.2

**直接生成 x86-64 机器码 · TileQuattro 内存管理 · 零依赖**

---

## 版本

```
4.2.0
```

## 核心特性

| 特性 | 说明 |
|------|------|
| **目标** | x86-64 ELF64 机器码 |
| **内存** | TileQuattro 四元池 |
| **依赖** | 零 |
| **Fate** | 完全控制每个字节 |
| **Collapse** | 边际收益递减时触发 |

---

## 快速开始

```bash
cd wave
./bin/wave examples/hello.wave -o hello
./hello
```

---

## 命令行选项

```bash
./bin/wave <input.wave> [-o output] [--no-tile] [--benchmark]

选项:
  -o output    指定输出文件名
  --no-tile    禁用 TileQuattro (使用标准 malloc)
  --benchmark  运行内存管理对比测试
```

---

## TileQuattro 内存池

```
┌────────────────────────────────────────┐
│           TileQuattro 四元池           │
├──────────────┬─────────────────────────┤
│ BlackHole    │ 大块 (code, 1MB)        │
├──────────────┼─────────────────────────┤
│ MeshBrain    │ 智能 (symbols, 512KB)   │
├──────────────┼─────────────────────────┤
│ MultiNova    │ 并行 (data, 1MB)        │
├──────────────┼─────────────────────────┤
│ BaseForce    │ 小块 (temp, 256KB)      │
└──────────────┴─────────────────────────┘
```

---

## 语法

```wave
# 输出
out "Hello, Wave v4.2!"

# 变量
count = 42
name = "Wave"

# 资源限制
limit 80

# 数据管道
data -> analyze -> store

# 条件
when count > 10 {
    out "Large"
}
```

---

## 文件结构

```
wave/
├── bin/wave              # 编译器
├── src/wave_compiler.c   # 源码 (~900行)
├── examples/
│   ├── hello.wave
│   └── benchmark_test.wave
├── archive/
│   └── wave_before_tile/ # 备份
├── include/valhalla_core.h
├── Makefile
├── VERSION               # 4.2.0
└── TILEQUATTRO_COMPARISON_REPORT.md
```

---

## 对比测试结果

```
100 次编译:
  标准 malloc:  3.02 ms  (0.030 ms/次)
  TileQuattro: 10.35 ms  (0.104 ms/次)
```

> TileQuattro 为 OS 级系统设计，在短命令行工具中有池初始化开销。
> Val-OS 必需 TileQuattro (无系统 malloc)。

---

## 编译

```bash
make
# 或
gcc -O2 -o bin/wave src/wave_compiler.c -lm
```

---

**Wave v4.2 - 直接机器码 · TileQuattro · Fate 控制** 🌊
