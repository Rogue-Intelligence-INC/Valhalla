# 🌊 Val-OS v5.0

> **Wave Language · Fate Engine · Quattro Memory · Tile Manager**

Val-OS 是完全用 Wave 语言编写的操作系统。Wave 表达意图，Fate 执行决策。

## 🏗️ 项目结构

```
val-os/
├── bootloader/
│   └── fate_boot.wave      # Fate 控制的引导加载器
├── kernel/
│   ├── fate_unified.wave   # Fate 统一系统 (核心)
│   ├── wave_gui.wave       # Wave GUI 系统
│   ├── display.wave        # 显示驱动
│   ├── keyboard.wave       # 键盘驱动
│   ├── mouse.wave          # 鼠标驱动
│   ├── vga.wave            # VGA 驱动
│   ├── window.wave         # 窗口管理
│   └── network.wave        # 网络栈
├── drivers/
│   ├── fate_hw.wave        # Fate 硬件抽象
│   ├── fate_drivers.wave   # Fate 驱动系统
│   └── simple_drivers.wave # 简单驱动
├── storage/
│   └── fate_memory.wave    # Fate 持久化记忆
├── fs/
│   └── fate_fs.wave        # Fate 文件系统
├── ui/
│   ├── wave_ui.wave        # 24-bit UI 系统
│   └── mouse.wave          # 鼠标 UI
├── desktop/
│   └── desktop.wave        # 桌面环境
├── shell/
│   └── fate_shell.wave     # Wave Shell
├── apps/
│   ├── notepad.wave        # 记事本
│   ├── settings.wave       # 设置
│   ├── minesweeper.wave    # 扫雷
│   ├── music_player.wave   # 音乐播放器
│   ├── video_player.wave   # 视频播放器
│   ├── notebook.wave       # 笔记本
│   ├── pkg_manager.wave    # 软件商店
│   └── system_panel.wave   # 系统面板
├── security/
│   └── permission.wave     # 权限系统
├── users/
│   └── multiuser.wave      # 多用户支持
├── net/
│   └── tcpip.wave          # TCP/IP 栈
├── runtime/
│   └── wave_runtime.wave   # Wave 运行时
├── valos.wave              # 主系统入口
├── valos_v5.wave           # v5.0 入口
└── valos_full_system.wave  # 完整系统
```

## ✨ 核心特性

### 🔮 Fate Engine

Fate 是系统的大脑，控制一切决策：

```wave
# 传统方式
if (size > 1024) { pool = LARGE; } else { pool = SMALL; }

# Fate 方式
request -> fate_analyze -> fate_decide -> execute
```

### 🧠 Quattro / Tile Memory

四元内存池，由 Fate 智能调度：

| 池 | 用途 | 选择条件 |
|---|------|----------|
| **BlackHole** | 压缩数据 | 高 `i` (信息密度) |
| **MeshBrain** | 模式数据 | 高 `r` (关联强度) |
| **MultiNova** | 通用存储 | 默认 |
| **BaseForce** | 快速缓存 | 高 `e` (熵) |

```wave
# Tile 选择算法
unified { i: 800, e: 200, r: 500 }
request -> tile_select -> fate_decide -> allocate
```

### 📐 Fate Vector Layout

布局用比例而非像素：

```wave
layout -> fate_vector {
    element -> { x: 0.1, y: 0.2, w: 0.3, h: 0.15 }
    # Fate 根据分辨率计算实际像素
    # 320x200 → x=32, y=40, w=96, h=30
    # 1920x1080 → x=192, y=216, w=576, h=162
}
```

### 🧬 Fate Memory

持久化记忆系统：

```wave
# 首次启动: 完整检测
first_boot -> fate_full_discovery

# 后续启动: 从记忆恢复
subsequent_boot -> fate_fast_restore
# 跳过已知的硬件检测
```

## 🎨 Wave 风格

Val-OS 使用声明式 + 管道风格：

```wave
# 正确 ✓
"hardware" -> fate_init -> fate_ready
blackhole -> fate_init_pool -> ready
icons -> fate_render -> display

# 错误 ✗
fn init_hardware() { ... if ... else ... }
```

## 📱 应用程序

| 应用 | 文件 | 说明 |
|------|------|------|
| 文件管理 | `apps/pkg_manager.wave` | Fate-powered 文件系统 |
| 终端 | `shell/fate_shell.wave` | Wave Shell |
| 设置 | `apps/settings.wave` | 系统设置 |
| 记事本 | `apps/notepad.wave` | 文本编辑 |
| 扫雷 | `apps/minesweeper.wave` | 游戏 |
| 音乐 | `apps/music_player.wave` | 音乐播放 |
| 视频 | `apps/video_player.wave` | 视频播放 |

## 🚀 启动流程

```
[Fate] 唤醒
[Fate] 检查持久化记忆
  ├─ 有记忆 → 快速恢复
  └─ 无记忆 → 完整检测
[Fate] 检测 CPU, 内存, 存储, 外设
[Fate] 决定内存布局
[Fate] 初始化四元池
[Fate] 切换 CPU 模式 (16→32→64)
[Fate] 加载内核
[Fate] 跳转内核入口
```

## 🔧 构建

Val-OS 使用 wave-bin 编译为原始二进制：

```bash
# 使用 wave-bin 编译
wave-bin valos.wave → valos.bin

# 生成可启动镜像
# (Fate 决定具体格式)
```

## 📊 与传统 OS 对比

| 特性 | 传统 OS | Val-OS |
|------|---------|--------|
| 配置 | /etc/config.conf | Fate Memory |
| 内存 | 固定分配 | Tile 四元池 |
| 布局 | 像素坐标 | Fate Vector 比例 |
| 决策 | 硬编码 if/else | Fate 自主决定 |
| 学习 | 无 | Fate 持续学习优化 |

---

**Val-OS** - *Wave expresses intent, Fate executes*

*Copyright © 2026 Jouly Mars (ZHUOLI MA) - Rogue Intelligence LNC.*
