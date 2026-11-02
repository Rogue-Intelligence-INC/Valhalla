# Valhalla 一键复现包 / One-Click Reproduction Package

本包把 Valhalla 当前对外数字（2026-11-03，代码基线
`e1ff0b3`）的全部生成程式固化为一条命令：构建 → 库测 →
检查包 → 行业电池 → 孵化因果证明 → 50 轮全流程，并逐项
核对预期读数。任何一步退出码非零或读数不符，脚本以非零
退出并指出哪一项对不上——不挑数、不自动重试、不改判据。

## 一键运行（Windows，已验证平台）

```powershell
cd <repo-root>
.\reproduce\reproduce.ps1            # 全套（约 8–12 分钟，含构建）
.\reproduce\reproduce.ps1 -Quick     # 冒烟：库测+检查+电池（约 3 分钟）
.\reproduce\reproduce.ps1 -SkipFlow  # 跳过 50 轮全流程
```

Linux/macOS（尽力而为，见下方平台说明）：

```bash
bash reproduce/reproduce.sh          # 全套
bash reproduce/reproduce.sh --quick  # 冒烟
```

## 前置条件

- Rust 工具链（cargo/rustc，stable）。
- NVIDIA CUDA 工具链（本仓 f64 线启用 `native_cuda` 特性，
  构建需链接 CUDA）。Windows 参考环境：CUDA v13.4，
  `C:\Ai\cuda-libx64` 存放导入库；脚本与统一入口
  `valhalla.exe` 会在该目录存在时自动注入
  `RUSTFLAGS=-L` 与 `PATH`。其他机器可用环境变量
  `VALHALLA_CUDA_LIB` 指定导入库目录。
- 磁盘：构建产物约数 GB；内存：全流程峰值随规模上升，
  50 轮配置在参考机（RTX 4090 / 64GB）已验证。

平台说明（诚实口径）：统一入口 `valhalla.exe` 与本包的
已验证路径是 Windows。`reproduce.sh` 直接调用各二进制
（不经统一入口），但 CUDA 构建在非 Windows 机器上未
经本项目验证；脚本会在缺少 CUDA 时明确报错退出，不会
假装跑通。

## 跑什么（与页面数字一一对应）

| 步 | 程式 | 配置 | 预期读数（基线 e1ff0b3） |
|---|---|---|---|
| L0 库测 | cargo test --lib（fate-f64 / stem-cell / tile-quattro / blackhole） | release | 7 / 4 / 8 / 3 全过 |
| L1 检查 | `valhalla check`（六件检查包） | — | ALL PASS |
| L2 电池 | `valhalla_industry_battery` | seeds 1–5 | T1 网络 67.6；T2 4–5 跳 91.7/91.1；T3 2 跳 87.0；BWT 网络 −0.7pp |
| L2 孵化 | `valhalla_incubation_proof` | seeds 1–15（逗号分隔） | C 57.1；结构因果 L = +11.6 ± 8.5pp（13/15 正）；距离 100/39.7/28.9；假题 C 77.8 |
| L3 全流程 | `valhalla_full_flow` | VH_MATS=ext、32KB×50 轮、2 周期/轮、种子 7、留出 32KB | 终局 9180 tiles / 9281 cells / 123,261 神经 / 31,030 存储；存储 100/100、react 100/100 |

注：孵化程式默认种子只有 1–5，本包显式设置
`VALHALLA_PROOF_SEEDS=1,2,...,15`（逗号分隔，写区间
语法会解析出 0 个种子）。运行前脚本会清掉残留的
`VH_*` / `VALHALLA_*` 环境变量，防止上一轮配置污染。

## 预期与核对

`expected_results.txt` 列出每步的预期关键行。脚本跑完
逐项在日志中查找并打印 `REPRODUCED` / `MISMATCH`。
参考机上同种子同料为确定性复现，应逐字一致；异机浮点
环境差异可能造成末位漂移——出现 MISMATCH 时先比对
`reproduce/logs/<时间戳>/` 与 `logs/suite_*/` 的原始
日志，区分"读数变了"与"仪器变了"，再下结论（纪律见
`docs/valhalla/RUN_MODE.md`）。

## 这些数字证明什么、不证明什么

- L2 孵化的 L（拆连接 vs 完整身体之差）是结构因果的
  消融读数；对照臂 A、C−A≈0 表示定向课程增益未检出，
  Hnull 分位 7% 表示拓扑特异性未成立——阴性结果同在
  日志里，不挑。
- 电池为自生成 analogue，非官方数据集分数；孵化为
  符号多跳任务，非通用智能证据。完整口径见
  `docs/valhalla/EVIDENCE_MAP_20261101.md` 与
  `docs/valhalla/WORK_SUMMARY_20261103.md`。

## 日志落点

- 本包日志：`reproduce/logs/<yyyyMMdd_HHmmss>/`
- 统一入口日志：`logs/suite_<epoch>/`（逐件全量输出）
