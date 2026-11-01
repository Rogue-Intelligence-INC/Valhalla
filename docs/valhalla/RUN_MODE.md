# Valhalla 运行模式（程式化总纲，2026-11-01 固化）

一条命令：`.\tools\run_valhalla_suite.ps1`（仓库根目录）。
本文件是运行模式的规范；`FULL_FLOW_RUNBOOK.md` 是 L3 的细则；
证据口径见 `EVIDENCE_MAP_20261101.md`，逐跑记录见 `TEST_RUNS_20261101.md`。

## 四层结构

| 层 | 内容 | 命令 | 时长 | 通过判据（预先声明） |
|---|---|---|---|---|
| L0 库测 | fate-f64 / stem-cell / tile-quattro 单元测试 | `cargo test --release --lib -p fate-f64 -p stem-cell -p valhalla-tile-quattro` | ~10s | 7/7、4/4、8/8（基线数，变一即停查） |
| L1 检查包 | fate_storage、storage、data_flow、monitor、light、decision 六件 | `.\target\release\valhalla_<name>.exe`（构建名以 `Cargo.toml` 的 `[[bin]]` 为准） | 各 <10s | 六件 EXIT=0；各自末行自报核查项 |
| L2 证明包 | 行业电池（5 seeds）；孵化证明（15 seeds） | `valhalla_industry_battery.exe`；`valhalla_incubation_proof.exe` | 6s / ~8min | 电池与定稿逐字一致；孵化以**逐种子行**复算：L=+27.4±0.9、C−A=+0.2±1.4 |
| L3 全流程 | `valhalla_full_flow`：Fate 分割入口 + 四元记账 + 身体周期 + 出口 A/B/C | `.\target\release\valhalla_full_flow.exe`（`VH_ROUNDS/VH_CHUNK_KB/VH_CYCLES/VH_SEED`） | 20 轮 ~55s | 第 10/末轮：存储精确对账、react 100/100、网络读数在案；判据全文见 RUNBOOK |

## 环境（每层相同，运行器已内含）

```powershell
cd C:\Ai\rogue-products\Valhalla
$env:RUSTFLAGS="-L C:\Ai\cuda-libx64"
$env:PATH="C:\Ai\cuda-libx64;C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v13.4\bin;$env:PATH"
```

实操纪律（踩过的坑）：
- exe 的 stderr **不要接管道**（PowerShell NativeCommandError 会误杀）；
  仓库脚本一律 `2>$null` 或 `*> 日志文件`。
- 孵化证明**汇总行有已知打印占位 bug**（±53.7、"分位 1%"是假数）；
  只认逐种子行与据此复算的增益/L 两行。
- bin 目标名以 `Cargo.toml` 的 `[[bin]] name` 为准（源文件名 ≠ 目标名，
  如 `fate_storage_test.rs` → `valhalla_fate_storage`）。
- `cargo test`（非 `--lib`）会撞 fate-f64 既存的 `test_cuda` 目标编译
  错误（`native_cuda` 未解析，与主线无关，已记账未修）。
- 确定性：同命令同料同种子，身体指纹逐字复现；全流程跨种子
  （VH_SEED 只管事实序/题集）身体终指纹相同。读数变了先查仪器、再谈身体。

## 各层测什么（一句话版）

- **L0**：三个机制库的构造级正确性（方程、容器、周期原语）。
- **L1**：存储与接口的工程健康——Fate 存储完全控制、四元落盘、
  数据流收口、监控读数、轻训留存/干预敏感、Fate 决策 X/Y 对照。
- **L2**：证据主力——电池测喂入关系进没进自发连接拓扑（行业
  analogue 口径）；孵化测结构因果（L）与定向培养增益（C−A），
  是"规模必要、培养定向"命题的裁决仪器。
- **L3**：巩固后的唯一全流程——入口（原版 Fate 自动分割）、
  体内（身体周期）、出口 A（留出料回应）、出口 B（harness 读
  身体状态的结构问答）、出口 C（对外 Fate `react` 问答接口，
  喂入双写 `data_store` 接线）。vent 式抓取出口未纳入（D11 未立）。

## 什么时候跑哪层

- 改库（fate/stem/tile）：L0 → L1 → L3 冒烟（VH_ROUNDS=3）。
- 改喂法/出口：L3 双种子 20 轮 → L2 电池 → 视改动面加孵化。
- 对外汇报前：全套（含孵化），并确认 TEST_RUNS 已落新节。
