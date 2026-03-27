## Wave 当前严格状态总结（可复现 / 可证据追踪）

**日期**：2026-03-20  
**范围**：仅总结仓库内 Wave 相关实现与可运行能力（不把“设计/伪代码/愿景”当作已实现能力）。

---

## 1. 版本矩阵（以路径为准）

> 目标：避免“二进制版 / C版 / ELF版”在文档与讨论中混用。

| 版本 / 称呼（严格） | 路径 | 形态 | 主要用途 | 备注 |
|---|---|---|---|---|
| **Wave 主目录（聚合）** | `wave/` | 多产物 | 放置多条实现线：bin、bootstrap、gpu、src 等 | 见 `wave/README.md` |
| **Wave 主实现二进制（运行时/解释器线）** | `wave/bin/` | ELF 可执行文件 | 运行 Wave 主实现（如 `wave5`、`wave5_new`） | 具体 I/O 协议以该二进制为准 |
| **Wave-C（规则驱动编译器）** | `wave/bin/wavec` | ELF 可执行文件 | 将 `.wave` 编译为 ELF（支持 `-o`） | **本仓库中可用的 wavec CLI** |
| **Wave-C（单文件 direct machine code 编译器）** | `wave/c/wavec` | ELF 可执行文件（由 C 编译） | 直接生成 x86-64 机器码/ELF（支持 `-o`） | 语法子集明显，且大量块被跳过（见 `wave/c/wave_compiler.c`） |
| **archive: wave_tile（Tile/ELF 版）** | `archive/tile_version/wave_tile/` | ELF 可执行文件 | TileQuattro + Fate 的 C 编译器线 | 常用于 benchmark/对比文档 |
| **archive: wave_c_compiler（C 版）** | `archive/wave_c_compiler/` | ELF / raw / bare-metal | C 编译器线（含 `--bare-metal` 等） | 与 wave_tile 同属 archive 线 |

对应的更细命名澄清见：`docs/WAVE_VERSION_LAYOUT.md`。

---

## 2. Wave-C 编译器（`wave/bin/wavec`）——已验证能力

### 2.1 CLI（证据：二进制自带 usage）

`wave/bin/wavec` 支持：

- 输入：`<input.wave>`
- 输出：`-o <output>`
- raw 输出：`--raw`

该编译器运行时会打印编译统计（Code 字节数、Functions 数、Unified、Fate 模式等）。

### 2.2 自举链路（“闭环”的严格定义）

严格自举闭环的最小判据：

- **Gen1**：用已有编译器编译“下一代编译器源码”，产出可执行物 \(C_1 = compile(C_{src})\)  
- **Gen2**：用 \(C_1\) 再编译同一份 \(C_{src}\)，产出 \(C_2\)  
- 若 \(C_2\) 可运行且能继续编译目标输入（至少能编译一个最小 hello），则判定闭环成立

本仓库中，`wave/src_bootstrap_new/wavec_bootstrap.wave` 是一个候选的“下一代编译器源码”入口（自举源码集合在 `wave/src_bootstrap_new/`）。

复现命令（建议在仓库根目录执行）：

```bash
# Gen1：用 wave/bin/wavec 编译 wavec_bootstrap.wave
./wave/bin/wavec wave/src_bootstrap_new/wavec_bootstrap.wave -o /tmp/wavec_gen1.elf

# Gen2：用 Gen1 再编译同一份源码（注意：Gen1 的 I/O 协议可能与 wave/bin/wavec 不同）
# 若 Gen1 也是 wavec CLI：应支持 -o；若是 bootstrap 协议：可能固定写 /tmp/output
/tmp/wavec_gen1.elf wave/src_bootstrap_new/wavec_bootstrap.wave -o /tmp/wavec_gen2.elf
```

**注意**：不同阶段产物可能采用不同的 I/O 协议（例如固定路径 `/tmp/input.wave`、`/tmp/output`，或 stdin→stdout），复现时应先通过二进制自带输出/strings 确认协议，再进行 Gen2。

---

## 3. 自举源码线（`wave/src_bootstrap_new/`）——重要约束

`wave/src_bootstrap_new/wave5_silent.wave` 与 `wave5_full.wave` 顶层包含：

- `source = file.read(0, 0x10000)`（从 **stdin** 读取源码）
- `compiler.compile(source)`

因此若将其产物当编译器使用，输入应优先按 **stdin** 协议喂入；而不是默认假设它读取 argv 里的文件路径。

---

## 4. 网络能力（严格边界）

结论来自 `docs/WAVE_NETWORK_CAPABILITY_ASSESSMENT.md`：

- **规范/设计层**：有 HTTP/TCP/IP 的 `.wave` 规范与伪代码（✅）
- **当前编译器能力**：`wavec` 仅覆盖 `syscall.exit` / `syscall.write` / `syscall.read`（⚠️）
- **可运行网络服务**：当前不可直接用 wavec 编译生成（❌），需外部后端或扩展 syscall 生成（socket/bind/listen/accept）

---

## 5. GPU（严格边界）

`wave/README.md` 声明了“规则组合 → GPU 并行模式”的映射（设计方向），但“是否已经在某条编译器线产生真实 OpenCL/CUDA 调用或 kernel 代码”需要以具体实现路径为准逐条验证。

在做 GPU 结论前，必须明确：

- 哪个版本（`wave/bin/*` / `wave/bin/wavec` / `wave/c/wavec` / `archive/*`）在被测试
- 输入程序里是否有语法开关（如 `opencl on` / `gpu on`）
- 输出物里是否出现真实 OpenCL/CUDA 运行时绑定（而不只是打印/flag）

---

## 6. 推荐的“严格验证清单”（最小集）

- **版本确认**：明确本次验证用的是哪个可执行文件（绝对路径）
- **I/O 协议确认**：help 输出或 `strings` 提取 input/output 约定
- **产物确认**：用 `file` 确认输出是否为 ELF / raw bin / 文本日志
- **闭环确认**：Gen1/Gen2 能否连续编译同一份编译器源码
- **最小程序确认**：能否编译并运行 `wave/examples/hello.wave`

