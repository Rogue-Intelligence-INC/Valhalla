#!/usr/bin/env bash
# Setup embedded repositories for the Valhalla workspace.
# Some workspace members are separately-managed repos (see .gitignore):
#   s-wind-sim/                        <- Rogue-Intelligence-INC/s-wind
#   valhalla-crypto/yaksha-cipher-gpu/ <- Rogue-Intelligence-INC/yaksha-cipher-gpu
# Clone them into place, then: cargo test -p s-wind-sim --release
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

clone_if_missing "s-wind-sim" "https://github.com/Rogue-Intelligence-INC/s-wind.git"
clone_if_missing "valhalla-crypto/yaksha-cipher-gpu" "https://github.com/Rogue-Intelligence-INC/yaksha-cipher-gpu.git"

# s-wind's standalone repo defaults to a vendored voyager; the monorepo layout
# expects the shared utils/voyager crate instead.
sed -i 's|voyager = { path = "vendor/voyager" }|voyager = { path = "../utils/voyager" }|' "$ROOT/s-wind-sim/Cargo.toml"
echo "s-wind-sim: voyager path set to ../utils/voyager"

echo
echo "Done. Verify with: cargo test -p s-wind-sim --release"
