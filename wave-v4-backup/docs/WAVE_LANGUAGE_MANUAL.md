# 🌊 Wave 语言手册 v4.3

**直接生成机器码 · Fate 自主控制 · 混合界面**

---

## 目录

1. [概述](#概述)
2. [基础语法](#基础语法)
3. [Fate 系统](#fate-系统)
4. [内存管理](#内存管理)
5. [混合界面 UI](#混合界面-ui)
6. [后端开发](#后端开发)
7. [未来功能](#未来功能)

---

## 概述

Wave 是一种直接编译为 x86-64 机器码的语言，具有以下特点：

| 特性 | 说明 |
|------|------|
| 编译目标 | x86-64 ELF64 |
| 运行时依赖 | 零 |
| 内存管理 | Fate+四元 |
| 界面系统 | 混合 UI |

---

## 基础语法

### 输出

```wave
out "Hello, Wave!"
```

### 管道

```wave
data -> process -> store
```

### 条件

```wave
when condition {
    action -> execute
}
```

### 循环

```wave
for i in 0..10 {
    process(i) -> execute
}
```

### 函数

```wave
my_function -> function(param) {
    result -> process(param)
    return result
}
```

---

## Fate 系统

### 核心概念

Fate 是 Wave 的智能控制系统，负责：
- 资源分配
- 性能优化
- 自主决策

### Fate 分配

```wave
# Fate 自动选择最优分配策略
request -> fate_alloc

# Fate 分析后决定
decision -> fate_decide
```

### Collapse 机制

```wave
# 边际收益下降时自动触发
# 内存减少 25%，性能提升 50%
fate_monitor -> fate_detect_marginal -> fate_collapse
```

#### 阈值范围 (Fate 自主选择)

**Wave 语言 - 软稳定性** (响应快、灵活)

| 阈值类型 | 范围 | 说明 |
|----------|------|------|
| 边际收益阈值 | **1%-5%** | 改善率低于此值触发 Collapse |
| 性能阈值 | **65%-80%** | 性能低于此值触发 Collapse |
| 负载阈值 | **60%-75%** | 负载高于此值开始监控 |

**Valhalla 主系统 - 高稳定性** (趋近100%，慎重变化)

| 阈值类型 | 范围 | 说明 |
|----------|------|------|
| 效率阈值 | **95%-99.5%** | 只有极低效率才干预 |
| 停滞阈值 | **500-1000 ticks** | 高容忍度 |
| Spawn 阈值 | **95%+ 效率** | 需要高效率才能spawn |

Fate 根据系统状态动态选择:
- 高负载 → 低阈值 (更敏感)
- 低负载 → 高阈值 (更宽容)

### Triple-Fate 架构

```wave
# Fate-in-Fate: 嵌套自省
inner_fate -> fate_create -> outer_fate

# Fate Loop: 循环优化
fate_state -> fate_loop -> evolved_fate
```

---

## 内存管理

### 四元池系统

| 池 | 用途 | 特点 |
|----|------|------|
| BlackHole | 大块连续内存 | 代码、页表 |
| MeshBrain | 索引数据 | 字符串、符号表 |
| MultiNova | 并行数据 | 缓冲区、数组 |
| BaseForce | 小块临时 | 变量、临时对象 |

### Fate 自动选择

```wave
# 无需指定池，Fate 根据特征自动选择
data -> fate_alloc

# Fate 分析:
#   大小、访问模式、生命周期
#   → 选择最优池
```

---

## 混合界面 UI

### 核心理念

```
不是 CLI，不是 GUI
是 Wave 特有的统一界面
```

### 24-bit True Color

```wave
# RGB 直接控制 (16,777,216 种颜色)
rgb -> function(r, g, b) {
    foreground -> "\x1b[38;2;{r};{g};{b}m"
    background -> "\x1b[48;2;{r};{g};{b}m"
}

# 使用
text_color -> rgb(255, 100, 50)
```

### 渐变

```wave
# 线性渐变
gradient -> function(start, end, steps) {
    for i in 0..steps {
        t -> i / (steps - 1)
        yield -> interpolate(start, end, t)
    }
}

# 多点渐变
rainbow -> multi_gradient([
    rgb(255,0,0),    # 红
    rgb(255,255,0),  # 黄
    rgb(0,255,0),    # 绿
    rgb(0,255,255),  # 青
    rgb(0,0,255),    # 蓝
    rgb(255,0,255)   # 紫
])

# 渐变文字
title -> gradient_text("WAVE", gradients.fire)
```

### 透明度

```wave
# Alpha 混合
alpha_blend -> function(fg, bg, opacity) {
    r -> fg.r * opacity + bg.r * (1 - opacity)
    g -> fg.g * opacity + bg.g * (1 - opacity)
    b -> fg.b * opacity + bg.b * (1 - opacity)
    result -> rgb(r, g, b)
}

# 半透明面板
panel.opacity -> 0.7
```

### 预设渐变

| 名称 | 颜色 |
|------|------|
| fire | 红→橙→黄 |
| ocean | 深蓝→青 |
| sunset | 红→橙→紫 |
| forest | 深绿→浅绿 |
| neon | 粉→青→黄 |
| rainbow | 彩虹 |
| fate | 紫→金 |

### 三种显示模式

```wave
mode -> state {
    compact  -> { colors: none, panels: none }
    standard -> { colors: basic, panels: none }
    rich     -> { colors: full_gradient, panels: transparent }
}

# Fate 根据终端大小自动选择
layout -> fate_adaptive
```

### 位置控制

```wave
cursor -> control {
    move_to -> "\x1b[{row};{col}H"
    save    -> "\x1b[s"
    restore -> "\x1b[u"
}
```

### 面板

```wave
panel -> element {
    position -> fate_decide
    size -> fate_decide
    border_gradient -> gradients.ocean
    bg_opacity -> 0.8
}
```

---

## 后端开发

### HTTP 服务器 (基础)

```wave
# 创建服务器
server -> http_server {
    port -> fate_decide  # 或指定 8080
}

# 路由
route "/" -> handler {
    response -> "Hello from Wave!"
}

route "/api/data" -> handler {
    data -> fate_db.query("data")
    response -> json(data)
}

# 启动
server -> start
```

### HTTPS (TLS)

```wave
# HTTPS 服务器
server -> https_server {
    port -> 443
    cert -> fate_load("cert.pem")
    key  -> fate_load("key.pem")
}
```

### 请求处理

```wave
handler -> function(request) {
    method -> request.method
    path   -> request.path
    body   -> request.body
    
    when method == "GET" {
        response -> handle_get(path)
    }
    
    when method == "POST" {
        data -> parse_json(body)
        response -> handle_post(data)
    }
    
    return response
}
```

### WebSocket

```wave
ws_server -> websocket {
    port -> 8081
    
    on_connect -> function(client) {
        clients -> fate_add(client)
    }
    
    on_message -> function(client, message) {
        broadcast(clients, message)
    }
}
```

### 数据库连接

```wave
# Fate 内置数据库
db -> fate_db {
    store("key", value)
    value -> query("key")
}

# 外部数据库 (通过转译器)
external_db -> transpile_to("sqlite") {
    query -> "SELECT * FROM users"
}
```

---

## 转译器

### 概念

Wave 可以转译为其他语言/格式，扩展生态兼容性：

```wave
# 转译为 C (系统调用)
transpile_to("c") {
    syscall -> write(fd, buf, len)
}

# 转译为 WebAssembly (浏览器)
transpile_to("wasm") {
    export -> my_function
}

# 转译为 JavaScript (Node.js 兼容)
transpile_to("js") {
    module.exports -> { api_handler }
}
```

### 支持的目标

| 目标 | 用途 | 状态 |
|------|------|------|
| C | 系统调用、FFI | ✅ 基础 |
| WebAssembly | Web 前端 | 🔜 计划中 |
| JavaScript | Node.js | 🔜 计划中 |
| LLVM IR | 优化、多平台 | 🔜 计划中 |

---

## 未来功能

以下功能计划在未来版本中添加：

### 网络

| 功能 | 状态 | 预计版本 |
|------|------|----------|
| HTTP/1.1 服务器 | ✅ 基础 | v4.3 |
| HTTPS/TLS | 🔜 计划中 | v4.4 |
| HTTP/2 | 🔜 计划中 | v4.5 |
| WebSocket | 🔜 计划中 | v4.4 |
| gRPC | 🔜 计划中 | v5.0 |

### 转译器

| 目标 | 状态 | 预计版本 |
|------|------|----------|
| WebAssembly | 🔜 计划中 | v4.5 |
| JavaScript | 🔜 计划中 | v4.5 |
| LLVM IR | 🔜 计划中 | v5.0 |

### 跨平台

| 平台 | 状态 | 预计版本 |
|------|------|----------|
| Linux x86-64 | ✅ 完成 | v4.0 |
| Linux ARM64 | 🔜 计划中 | v5.0 |
| Windows | 🔜 计划中 | v5.0 |
| macOS | 🔜 计划中 | v5.0 |

### 标准库

| 模块 | 状态 | 预计版本 |
|------|------|----------|
| 文件 I/O | ✅ 基础 | v4.0 |
| 网络 | 🔜 开发中 | v4.4 |
| JSON | 🔜 计划中 | v4.4 |
| 正则表达式 | 🔜 计划中 | v4.5 |
| 加密 | 🔜 计划中 | v4.5 |

### 工具链

| 工具 | 状态 | 预计版本 |
|------|------|----------|
| 包管理器 | 🔜 计划中 | v5.0 |
| 调试器 | 🔜 计划中 | v5.0 |
| LSP | 🔜 计划中 | v5.0 |
| 格式化器 | 🔜 计划中 | v4.5 |

---

## 版本历史

| 版本 | 主要更新 |
|------|----------|
| v4.0 | 直接机器码生成 |
| v4.1 | TileQuattro 内存管理 |
| v4.2 | Fate+四元、无硬编码 |
| v4.3 | 混合界面、24-bit 色彩、后端基础 |

---

**Wave v4.3 - 直接机器码 · Fate 自主 · 完整色彩 · 后端开发** 🌊
