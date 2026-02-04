# 🌌 Fate Organizer - Valhalla 智能文件整理系统

> **"把 Hub 反过来用"** - 让 Valhalla 的 AI 能力向外输出，整理真实世界的文件

## 📖 概述

Fate Organizer 是 Valhalla 项目的一个**独立应用**，它将原本用于内部数据处理的 Hub 系统"反过来用"，让 Fate 引擎的模式识别能力作用于真实的文件系统。

## 📁 项目位置

```
/home/jouly/AI/Valhalla/fate-organize/   # 独立项目目录
├── Cargo.toml                           # 项目配置
├── src/
│   ├── lib.rs                           # 核心库 (FateOrganizer)
│   └── bin/
│       └── main.rs                      # CLI 入口
└── target/release/
    └── fate-organize                    # 编译后的可执行文件
```

```
┌─────────────────────────────────────────────────────────────────┐
│                    Fate Organizer (反向Hub)                      │
│   文件系统 ──→ Fate分析 ──→ 四元分类 ──→ 智能整理                │
└─────────────────────────────────────────────────────────────────┘
                             │
         ┌──────────────────┼──────────────────┐
         │                  │                  │
   ┌─────▼─────┐     ┌─────▼─────┐     ┌─────▼─────┐
   │ BlackHole │     │ MeshBrain │     │ MultiNova │
   │  压缩/去重  │     │  语义分析  │     │  批量处理  │
   └───────────┘     └───────────┘     └───────────┘
```

## ✨ 核心功能

### 🔮 智能分析

- **AI 来源识别**: 自动识别 DALL-E、Midjourney、ChatGPT 等 AI 生成的文件
- **日期模式提取**: 从文件名中提取日期信息 (YYYY-MM-DD, YYYYMMDD 等)
- **语言检测**: 识别中文、英文、混合语言文件名
- **文件分类**: 图片、视频、音频、文档、代码、压缩包等

### 🛡️ 保留区 (Preserve Zone)

保留区是一个特殊区域，其中的文件不会被自动整理：

```
/.preserve/
├── 重要文件.doc
├── 不要动.txt
└── ...
```

**功能特点:**
- 拖放文件到保留区，Fate 会跳过它们
- 从保留区拖出，文件会被 Fate 自动整理
- 支持模式匹配（如 `*.important`）

### 📁 fate-organize 目录

标准化的整理目录结构：

```
fate-organize/
├── AI生成/
│   ├── DALL-E/
│   ├── ChatGPT/
│   └── Midjourney/
├── 图片/
│   ├── 照片/
│   └── 截图/
├── 视频/
├── 音频/
├── 文档/
├── 压缩包/
├── 代码/
├── 其他/
└── .preserve/  (保留区)
```

### 🔧 高级操作

| 功能 | 说明 |
|------|------|
| **合并文件夹** | 将多个文件夹合并到一个目标文件夹 |
| **重命名文件夹** | 重命名单个文件夹 |
| **批量重命名** | 使用正则表达式批量重命名文件 |
| **清理空文件夹** | 删除整理后的空文件夹 |

### 🔐 权限系统

| 级别 | 权限 |
|------|------|
| `guest` | 只读 |
| `user` | 读写、重命名 |
| `power` | + 删除、整理、保留区 |
| `admin` | + 合并、递归操作 |
| `root` | 完全控制 |

## 💻 命令行使用

### 编译

```bash
cd /home/jouly/AI/Valhalla/fate-organize
cargo build --release
```

### 使用可执行文件

```bash
# 分析目录
./target/release/fate-organize "/path/to/dir"

# 执行整理
./target/release/fate-organize "/path/to/dir" --execute --force

# 递归整理整个文件树
./target/release/fate-organize "/path/to/dir" -r --execute --force

# 创建 fate-organize 目录结构
./target/release/fate-organize "/path/to/dir" --fate-dir

# 整理后清理空文件夹
./target/release/fate-organize "/path/to/dir" --execute --force --cleanup-empty

# 合并文件夹
./target/release/fate-organize "/path" --merge "folder1,folder2:merged"

# 重命名文件夹
./target/release/fate-organize "/path" --rename "old_name:new_name"

# 批量重命名
./target/release/fate-organize "/path" --batch-rename "IMG_(\\d+):photo_$1"

# 生成报告
./target/release/fate-organize "/path/to/dir" --output report.md -v
```

### 使用 cargo run

```bash
cd /home/jouly/AI/Valhalla/fate-organize
cargo run --release -- "/path/to/dir" --execute --force
```

## 🖥️ Val-OS 集成

在 Val-OS v5.0 中，Fate Organizer 已集成为默认文件管理器：

### 文件管理器界面

```
┌─────────────────────────────────────────────────────────────┐
│ Files (Fate-powered 🔮)                              [─][□][×] │
├─────────────────────────────────────────────────────────────┤
│ ← → ↑ 🔄 │ 📁 📄 │ 🔲 📋 📊 🔮 │ 🛡️ ✨                     │
├────────────┬────────────────────────────────────────────────┤
│ 📌 快速访问  │                                                │
│   🏠 主目录  │  📁 Documents                                  │
│   📁 文档   │  📁 Pictures                                    │
│   🖼️ 图片   │  📁 Music                                       │
│   🎵 音乐   │  🖼️ photo.jpg                                   │
│   🎬 视频   │  🤖 DALL-E_image.png                            │
│   ⬇️ 下载   │  📄 notes.txt                                   │
├────────────┤                                                │
│ 🛡️ 保留区  │                                                │
│ ┌────────┐ │                                                │
│ │拖放文件 │ │                                                │
│ │到此保留 │ │                                                │
│ └────────┘ │                                                │
│ 📂 打开保留区│                                                │
├────────────┤                                                │
│ 🔮 Fate 智能 │                                                │
│ 🔍 分析目录  │                                                │
│ ✨ 智能整理  │                                                │
│ 👁️ Fate视图 │                                                │
│ 📁 fate-org │                                                │
├────────────┤                                                │
│ 🔧 高级操作  │                                                │
│ 📦 合并文件夹│                                                │
│ 📝 重命名   │                                                │
│ 🔄 批量重命名│                                                │
└────────────┴────────────────────────────────────────────────┘
```

### Wave 代码示例

```wave
# 导入 Fate Files
fate_files -> import("../kernel/fate_files.wave")

# 初始化
fate_files.fate_files_init()

# 检查权限
when fate_files.check_permission("organize") {
    # 分析目录
    analysis -> fate_files.analyze_directory("/home/user/Downloads")
    
    # 执行整理
    fate_files.execute_organization("/home/user/Downloads", false)
}

# 使用保留区
fate_files.add_to_preserve("/home/user/important.doc")
fate_files.move_to_preserve("/home/user/temp/keep_this.txt")

# 从保留区拖出到 Fate 管理
fate_files.drag_from_preserve("/home/user/.preserve/old_file.txt", null)

# 合并文件夹
fate_files.merge_folders(
    ["/home/user/folder1", "/home/user/folder2"],
    "/home/user/merged",
    "keep_both"
)
```

## 🎯 设计哲学

### "把 Hub 反过来用"

| 原来的 Hub | Fate Organizer |
|-----------|----------------|
| 外部请求 → 内部处理 | 文件系统 → 外部整理 |
| Fate 分流到四元系统 | Fate 分析模式并分类 |
| 输出处理结果 | 执行整理操作 |

### 四元系统在文件整理中的角色

- **BlackHole**: 压缩、去重、归档大文件
- **MeshBrain**: 语义分析、关联发现、智能分类
- **MultiNova**: 批量处理、并行操作
- **BaseForce**: 协调整理、维护一致性

## 📊 性能

在实际测试中：

| 操作 | 文件数 | 耗时 |
|------|--------|------|
| 目录分析 | 124 文件 | < 1秒 |
| 智能整理 | 124 文件 | < 2秒 |
| 清理空文件夹 | 525 个 | < 1秒 |

## 🔮 未来计划

- [ ] 文件内容分析（使用 AI 理解文件内容）
- [ ] 智能去重（基于内容哈希）
- [ ] 自动备份（整理前创建快照）
- [ ] 云存储集成
- [ ] 多设备同步

---

**Fate Organizer** - *Valhalla 项目的一部分*

*让 AI 的力量服务于日常生活*

*Version 1.0.0 | January 2026*
