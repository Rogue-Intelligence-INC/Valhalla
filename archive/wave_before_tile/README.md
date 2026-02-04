# 🌊 Wave v4

**直接生成 x86-64 机器码，零依赖**

---

## 版本

```
4.1.0
```

## 核心特性

| 特性 | 说明 |
|------|------|
| **目标** | x86-64 ELF64 机器码 |
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

## 语法

```wave
# 输出
out "Hello, Wave v4!"

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
├── bin/wave           # 编译器
├── src/wave_compiler.c # 源码 (760行)
├── examples/hello.wave
├── include/valhalla_core.h
├── Makefile
└── VERSION            # 4.1.0
```

---

## 架构

```
┌─────────────────────────────────────┐
│         Wave v4 编译器              │
├─────────────────────────────────────┤
│  FateMachine                        │
│  ├── code[]    - 机器码缓冲         │
│  ├── data[]    - 数据段缓冲         │
│  ├── Collapse  - 坍塌优化系统       │
│  └── FatePerf  - 按需资源分配       │
├─────────────────────────────────────┤
│  机器码生成                          │
│  ├── emit_mov_rax_imm64()           │
│  ├── emit_syscall()                 │
│  └── ELF64 Header                   │
├─────────────────────────────────────┤
│  输出: 可执行 ELF64                  │
└─────────────────────────────────────┘
```

---

## Collapse 系统

触发条件：
1. **边际收益递减** - `|marginal_gain| < 0.01`
2. **性能不足** - `current < required`

```c
if (diminishing_returns || performance_insufficient) {
    trigger_collapse();  // 内存-25%, 性能+50%
}
```

---

## 编译

```bash
make
# 或
gcc -O2 -o bin/wave src/wave_compiler.c -lm
```

---

**Wave v4 - 直接机器码 · Fate 控制 · 零依赖** 🌊
