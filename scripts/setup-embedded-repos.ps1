# Setup embedded repositories for the Valhalla workspace.
# Some workspace members are separately-managed repos (see .gitignore):
#   s-wind-sim/                        <- Rogue-Intelligence-INC/s-wind
#   valhalla-crypto/yaksha-cipher-gpu/ <- Rogue-Intelligence-INC/yaksha-cipher-gpu
# Clone them into place, then: cargo test -p s-wind-sim --release
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent

$embedded = @(
    @{ Dir = 's-wind-sim'; Url = 'https://github.com/Rogue-Intelligence-INC/s-wind.git' },
    @{ Dir = 'valhalla-crypto/yaksha-cipher-gpu'; Url = 'https://github.com/Rogue-Intelligence-INC/yaksha-cipher-gpu.git' }
)

foreach ($e in $embedded) {
    $dest = Join-Path $root $e.Dir
    if (Test-Path (Join-Path $dest 'Cargo.toml')) {
        Write-Host "OK (already present): $($e.Dir)"
    } else {
        Write-Host "Cloning $($e.Url) -> $($e.Dir)"
        git clone $e.Url $dest
    }
}

# s-wind's standalone repo defaults to a vendored voyager; the monorepo layout
# expects the shared utils/voyager crate instead.
$swindManifest = Join-Path $root 's-wind-sim/Cargo.toml'
if (Test-Path $swindManifest) {
    (Get-Content $swindManifest -Raw) -replace 'voyager = \{ path = "vendor/voyager" \}', 'voyager = { path = "../utils/voyager" }' | Set-Content $swindManifest
    Write-Host 's-wind-sim: voyager path set to ../utils/voyager'
}

Write-Host "`nDone. Verify with: cargo test -p s-wind-sim --release"
