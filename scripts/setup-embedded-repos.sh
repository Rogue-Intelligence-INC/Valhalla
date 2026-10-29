#!/usr/bin/env bash
# Setup embedded repositories for the Valhalla workspace.
# One workspace member is a separately-managed repo (see .gitignore):
#   valhalla-crypto/yaksha-cipher-gpu/ <- Rogue-Intelligence-INC/yaksha-cipher-gpu
# (S-Wind was separated into its own product repo: Rogue-Intelligence-INC/s-wind)
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

clone_if_missing() {
    local dir="$1" url="$2"
    if [ -f "$ROOT/$dir/Cargo.toml" ]; then
        echo "OK (already present): $dir"
    else
        echo "Cloning $url -> $dir"
        git clone "$url" "$ROOT/$dir"
    fi
}

clone_if_missing "valhalla-crypto/yaksha-cipher-gpu" "https://github.com/Rogue-Intelligence-INC/yaksha-cipher-gpu.git"

echo
echo "Done. Verify with: cargo check -p yaksha-cipher-gpu --release"
