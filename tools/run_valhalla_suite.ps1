<#
.SYNOPSIS
  Valhalla 全套运行器（兼容包装；主入口已整合为 Rust 程式
  target/release/valhalla.exe，子命令 run/check/flow/viz，
  规范见 docs/valhalla/RUN_MODE.md）。

.DESCRIPTION
  一条命令跑完四层：
    L0 库测      cargo test --lib（fate-f64 / stem-cell / valhalla-tile-quattro）
    L1 检查包    fate_storage / storage / data_flow / monitor / light / decision 六件
    L2 证明包    行业电池（5 seeds）；孵化证明（15 seeds，约 8 分钟，需 -IncludeIncubation）
    L3 全流程    valhalla_full_flow（VH_ROUNDS 轮，第 10/末轮测出口 A/B/C）
  每件输出落 logs\suite_<时间戳>\，逐件报 PASS/FAIL（退出码）。
  判读纪律见 docs/valhalla/RUN_MODE.md：孵化以逐种子行为准
  （汇总行有已知打印占位 bug）；exe 的 stderr 不要接管道。

.EXAMPLE
  .\tools\run_valhalla_suite.ps1                # L0+L1+L2(电池)+L3，约 3 分钟
  .\tools\run_valhalla_suite.ps1 -IncludeIncubation
  .\tools\run_valhalla_suite.ps1 -Rounds 10 -Seed 42
  .\tools\run_valhalla_suite.ps1 -Viz           # 展示模式：跑 valhalla_viz 并打开监控页
#>
param(
    [switch]$IncludeIncubation,
    [switch]$Viz,
    [int]$Rounds = 20,
    [int]$Seed = 7
)

$ErrorActionPreference = "Continue"
$repo = Split-Path -Parent $PSScriptRoot
Set-Location $repo

# 构建环境（与各 bin 既定命令一致）
$env:RUSTFLAGS = "-L C:\Ai\cuda-libx64"
$env:PATH = "C:\Ai\cuda-libx64;C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v13.4\bin;$env:PATH"

$stamp = Get-Date -Format "yyyyMMdd_HHmmss"
$logDir = Join-Path $repo "logs\suite_$stamp"
New-Item -ItemType Directory -Force $logDir | Out-Null

$results = New-Object System.Collections.Generic.List[object]
function Run-Item([string]$Layer, [string]$Name, [scriptblock]$Cmd) {
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    & $Cmd *> (Join-Path $logDir "$Name.log")
    $code = $LASTEXITCODE
    $sw.Stop()
    $ok = ($code -eq 0)
    $results.Add([pscustomobject]@{ Layer = $Layer; Item = $Name; Pass = $ok; Seconds = [math]::Round($sw.Elapsed.TotalSeconds, 1) })
    "{0} [{1}] {2}  ({3}s, exit {4})" -f $(if ($ok) { "PASS" } else { "FAIL" }), $Layer, $Name, [math]::Round($sw.Elapsed.TotalSeconds, 1), $code
}

Write-Host "== 构建 =="
cargo build --release -p hub-f64 --bin valhalla_fate_storage --bin valhalla_storage_test --bin valhalla_data_flow --bin valhalla_monitor_check --bin valhalla_light_train_check --bin valhalla_fate_decision_check --bin valhalla_industry_battery --bin valhalla_incubation_proof --bin valhalla_full_flow --bin valhalla_viz 2>$null | Select-Object -Last 1

Write-Host "== L0 库测 =="
Run-Item "L0" "lib_tests" { cargo test --release --lib -p fate-f64 -p stem-cell -p valhalla-tile-quattro }

Write-Host "== L1 检查包 =="
foreach ($b in @("valhalla_fate_storage", "valhalla_storage_test", "valhalla_data_flow", "valhalla_monitor_check", "valhalla_light_train_check", "valhalla_fate_decision_check")) {
    Run-Item "L1" $b { & ".\target\release\$b.exe" }
}

Write-Host "== L2 证明包 =="
Run-Item "L2" "valhalla_industry_battery" { .\target\release\valhalla_industry_battery.exe }
if ($IncludeIncubation) {
    Run-Item "L2" "valhalla_incubation_proof" { .\target\release\valhalla_incubation_proof.exe }
} else {
    Write-Host "SKIP [L2] valhalla_incubation_proof（加 -IncludeIncubation 才跑，约 8 分钟）"
}

Write-Host "== L3 全流程 =="
$env:VH_ROUNDS = "$Rounds"
$env:VH_SEED = "$Seed"
Run-Item "L3" "valhalla_full_flow" { .\target\release\valhalla_full_flow.exe }

if ($Viz) {
    Write-Host "== 展示（valhalla_viz + 监控页） =="
    Run-Item "VIZ" "valhalla_viz" { .\target\release\valhalla_viz.exe }
    $page = Join-Path $repo "viz_out\index.html"
    if (Test-Path $page) { Start-Process $page }
}
Remove-Item Env:VH_ROUNDS, Env:VH_SEED -ErrorAction SilentlyContinue

Write-Host ""
Write-Host "== 汇总（日志: $logDir）=="
$results | Format-Table -AutoSize
$failed = @($results | Where-Object { -not $_.Pass })
if ($failed.Count -eq 0) { Write-Host "ALL PASS" } else { Write-Host "FAILED: $($failed.Item -join ', ')"; exit 1 }
