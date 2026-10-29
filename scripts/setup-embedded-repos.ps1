# Setup embedded repositories for the Valhalla workspace.
# One workspace member is a separately-managed repo (see .gitignore):
#   valhalla-crypto/yaksha-cipher-gpu/ <- Rogue-Intelligence-INC/yaksha-cipher-gpu
# (S-Wind was separated into its own product repo: Rogue-Intelligence-INC/s-wind)
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent

$embedded = @(
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

Write-Host "`nDone. Verify with: cargo check -p yaksha-cipher-gpu --release"
