#!/usr/bin/env bash
# Install / verify industry-standard randomness & crypto benchmarks for Yaksha.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TOOLS="$ROOT/external_bench/tools"
STS_DIR="$TOOLS/nist-sts"

echo "=== Yaksha benchmark toolchain ==="

missing=0
for cmd in dieharder openssl RNG_test cargo python3 gcc make curl unzip; do
  if command -v "$cmd" &>/dev/null; then
    echo "  OK  $cmd"
  else
    echo "  MISS $cmd"
    missing=$((missing + 1))
  fi
done

# Arch hints
if [[ $missing -gt 0 ]]; then
  echo ""
  echo "Arch Linux: sudo pacman -S dieharder openssl gcc make curl unzip python"
  echo "PractRand:  sudo pacman -S practrand   # provides RNG_test"
fi

# NIST STS 2.1.2 (not in most distros — build from source)
ASSESS=""
if [[ -x "$STS_DIR/sts-2.1.2/sts-2.1.2/assess" ]]; then
  ASSESS="$STS_DIR/sts-2.1.2/sts-2.1.2/assess"
elif [[ -x "$STS_DIR/sts-2.1.2/assess" ]]; then
  ASSESS="$STS_DIR/sts-2.1.2/assess"
fi

if [[ -z "$ASSESS" ]]; then
  echo ""
  echo "Building NIST SP 800-22 STS 2.1.2 ..."
  mkdir -p "$STS_DIR"
  cd "$STS_DIR"
  if [[ ! -f sts-2.1.2.zip ]]; then
    curl -fsSL -o sts-2.1.2.zip \
      "https://csrc.nist.gov/CSRC/media/Projects/Random-Bit-Generation/documents/sts-2_1_2.zip" \
      || curl -fsSL -o sts-2.1.2.zip \
      "https://csrc.nist.rip/CSRC/media/Projects/Random-Bit-Generation/documents/sts-2_1_2.zip"
  fi
  unzip -qo sts-2.1.2.zip -d .
  cd sts-2.1.2/sts-2.1.2 2>/dev/null || cd sts-2.1.2
  make -j"$(nproc)"
  ASSESS="$(pwd)/assess"
fi

echo "  OK  NIST STS assess → $ASSESS"

# Symlink for convenience
mkdir -p "$TOOLS/bin"
ln -sf "$ASSESS" "$TOOLS/bin/nist-sts-assess"

echo ""
echo "Installed. Run:"
echo "  ./external_bench/run_all.sh              # industry gate (OpenSSL + dieharder STS + Yaksha)"
echo "  ./external_bench/run_nist_sts.sh core 64 # full NIST SP 800-22 (15 tests)"
echo "  valhalla-crypto/yaksha-cipher/scripts/run_external_tests.sh core 64"
