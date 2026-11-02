<#
.SYNOPSIS
  Valhalla one-click reproduction (Windows, validated platform).
  Builds, runs library tests, check suite, industry battery,
  incubation causal proof (15 seeds) and the 50-round full flow,
  then verifies headline readings against expected_results.txt.
  Any failure exits non-zero; nothing is retried or re-judged.

.EXAMPLE
  .\reproduce\reproduce.ps1            # full suite (~8-12 min incl. build)
  .\reproduce\reproduce.ps1 -Quick     # smoke: lib tests + check + battery
  .\reproduce\reproduce.ps1 -SkipFlow  # skip the 50-round full flow
#>
param([switch]$Quick, [switch]$SkipFlow)

$ErrorActionPreference = 'Stop'
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

$Stamp  = Get-Date -Format 'yyyyMMdd_HHmmss'
$LogDir = Join-Path $PSScriptRoot "logs\$Stamp"
New-Item -ItemType Directory -Force $LogDir | Out-Null
$script:Failed = @()

function Note($msg) { Write-Host "== $msg" -ForegroundColor Cyan }

function Run-Step($Name, $LogName, $Action) {
    Note $Name
    $log = Join-Path $LogDir $LogName
    # Native stderr (cargo warnings) must not kill the step: judge by exit code only.
    $prev = $ErrorActionPreference; $ErrorActionPreference = 'Continue'
    try { & $Action *> $log; $code = $LASTEXITCODE } finally { $ErrorActionPreference = $prev }
    if ($code -ne 0) {
        Write-Host "FAIL $Name (exit $code) - log: $log" -ForegroundColor Red
        $script:Failed += $Name
    } else {
        Write-Host "PASS $Name - log: $log" -ForegroundColor Green
    }
}

# --- clean stale experiment env (a previous session must not leak in) ---
Get-ChildItem env:VH_*, env:VALHALLA_* -ErrorAction SilentlyContinue |
    ForEach-Object { Remove-Item "env:$($_.Name)" }

# --- prerequisites ---
foreach ($tool in 'cargo', 'rustc') {
    if (-not (Get-Command $tool -ErrorAction SilentlyContinue)) {
        Write-Host "MISSING prerequisite: $tool (install Rust stable)" -ForegroundColor Red
        exit 2
    }
}
Write-Host ("rustc: " + (& rustc --version))

# --- CUDA build env: inject only when the directories exist (mirrors valhalla.rs) ---
$CudaLib = $env:VALHALLA_CUDA_LIB
if (-not $CudaLib) { $CudaLib = 'C:\Ai\cuda-libx64' }
if (Test-Path $CudaLib) {
    $env:RUSTFLAGS = "-L $CudaLib"
    $cudaBin = 'C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v13.4\bin'
    $parts = @($CudaLib)
    if (Test-Path $cudaBin) { $parts += $cudaBin }
    $env:PATH = ($parts -join ';') + ';' + $env:PATH
    Write-Host "CUDA lib injected: $CudaLib"
} else {
    Write-Host "WARNING: CUDA import-lib dir not found ($CudaLib). The native_cuda build will likely fail; set VALHALLA_CUDA_LIB." -ForegroundColor Yellow
}

$Exe = Join-Path $Root 'target\release\valhalla.exe'

# --- build the unified runner, then let it build the suite ---
Run-Step 'build runner (valhalla)' 'build_runner.log' {
    cargo build --release -p hub-f64 --bin valhalla
}
if ($script:Failed.Count -gt 0) { Write-Host 'BUILD FAILED - stop.' -ForegroundColor Red; exit 1 }
Run-Step 'build suite binaries' 'build_suite.log' { & $Exe build }
if ($script:Failed.Count -gt 0) { Write-Host 'BUILD FAILED - stop.' -ForegroundColor Red; exit 1 }

# --- L0 library tests (incl. blackhole, which the runner's L0 omits) ---
Run-Step 'L0 library tests (fate 7 / stem 4 / tile 8 / blackhole 3)' 'lib_tests.log' {
    cargo test --release --lib -p fate-f64 -p stem-cell -p valhalla-tile-quattro -p valhalla_metasystems_blackhole
}

# --- L1 check suite ---
Run-Step 'L1 check suite (6 packages)' 'check.log' { & $Exe check --skip-build }

# --- L2 industry battery (seeds 1-5, the program default) ---
$env:VALHALLA_IND_SEEDS = '1,2,3,4,5'
Run-Step 'L2 industry battery (5 seeds)' 'battery.log' { & $Exe battery --skip-build }
Remove-Item env:VALHALLA_IND_SEEDS

if (-not $Quick) {
    # --- L2 incubation causal proof (15 seeds; program default is only 1-5) ---
    $env:VALHALLA_PROOF_SEEDS = '1,2,3,4,5,6,7,8,9,10,11,12,13,14,15'
    Run-Step 'L2 incubation causal proof (15 seeds)' 'incubation.log' { & $Exe incubation --skip-build }
    Remove-Item env:VALHALLA_PROOF_SEEDS

    if (-not $SkipFlow) {
        # --- L3 full flow: ext materials, 32KB x 50 rounds, seed 7 ---
        $env:VH_MATS = 'ext'; $env:VH_CHUNK_KB = '32'; $env:VH_ROUNDS = '50'
        $env:VH_CYCLES = '2'; $env:VH_SEED = '7'; $env:VH_HOLDOUT_KB = '32'
        Run-Step 'L3 full flow (ext, 32KB x 50 rounds)' 'flow.log' {
            & (Join-Path $Root 'target\release\valhalla_full_flow.exe')
        }
        Get-ChildItem env:VH_* | ForEach-Object { Remove-Item "env:$($_.Name)" }
    }
}

# --- verification against expected_results.txt (reference readings, baseline e1ff0b3) ---
Note 'verification (expected readings @ e1ff0b3)'
# Battery/incubation detail lives in the runner's per-suite logs, not in
# our console capture: search our log dir plus the newest matching suite log.
$scopeFile = @{
    battery    = 'valhalla_industry_battery.log'
    incubation = 'valhalla_incubation_proof.log'
}
$searchRoots = @($LogDir)
$expected = Get-Content (Join-Path $PSScriptRoot 'expected_results.txt') |
    Where-Object { $_ -and -not $_.StartsWith('#') }
$mismatch = 0
foreach ($line in $expected) {
    $key, $want = $line -split '\|', 2
    $key = $key.Trim(); $want = $want.Trim()
    $scope = ($key -split '-')[0]
    if ($Quick -and @('incubation', 'flow') -contains $scope) { continue }
    if ($SkipFlow -and $scope -eq 'flow') { continue }
    $hit = $false
    $roots = @($searchRoots)
    if ($scopeFile.ContainsKey($scope)) {
        $f = Get-ChildItem (Join-Path $Root 'logs') -Recurse -Filter $scopeFile[$scope] -ErrorAction SilentlyContinue |
            Sort-Object LastWriteTime -Descending | Select-Object -First 1
        if ($f) { $roots += $f.DirectoryName }
    }
    foreach ($r in $roots) {
        if (Select-String -Path (Join-Path $r '*') -Pattern ([regex]::Escape($want)) -SimpleMatch:$false -Quiet -ErrorAction SilentlyContinue) { $hit = $true; break }
    }
    if ($hit) { Write-Host "  REPRODUCED  $key = $want" -ForegroundColor Green }
    else { Write-Host "  MISMATCH    $key : expected substring '$want' not found in logs" -ForegroundColor Red; $mismatch++ }
}

Write-Host ""
Write-Host "logs: $LogDir"
if ($script:Failed.Count -gt 0) { Write-Host ("FAILED steps: " + ($script:Failed -join ', ')) -ForegroundColor Red; exit 1 }
if ($mismatch -gt 0) { Write-Host "DONE WITH $mismatch READING MISMATCH(ES) - compare raw logs before concluding (see reproduce/README.md)" -ForegroundColor Red; exit 1 }
Write-Host 'ALL STEPS PASS - headline readings reproduced.' -ForegroundColor Green
exit 0
