#!/usr/bin/env bash
# Run external randomness tests (dieharder, PractRand) on Yaksha cipher output.
# Usage: ./scripts/run_external_tests.sh [mode] [size_mb]
#   mode: core | parametric | neural | evolve (default: neural)
#   size_mb: MB of data to generate (default: 20)
# Requires: cargo, generate_test_data binary. Optional: dieharder, RNG_test (PractRand)

set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
# Workspace target (Valhalla repo) or crate-local target
WORKSPACE_ROOT="$(cd "$CRATE_DIR/../.." 2>/dev/null && pwd)"
if [[ -f "$WORKSPACE_ROOT/Cargo.toml" ]] && grep -q '^\[workspace\]' "$WORKSPACE_ROOT/Cargo.toml" 2>/dev/null; then
  TARGET_DIR="$WORKSPACE_ROOT/target"
else
  TARGET_DIR="$CRATE_DIR/target"
fi
MODE="${1:-neural}"
SIZE_MB="${2:-20}"
DATA_FILE="${TARGET_DIR}/yaksha_external_test_${MODE}_${SIZE_MB}mb.bin"
REPORT_DIR="${TARGET_DIR}/strict_reports"
mkdir -p "$REPORT_DIR"

echo "=== Yaksha external strict tests ==="
echo "Mode: $MODE  Size: ${SIZE_MB} MB  Data: $DATA_FILE"
echo ""

# Build generate_test_data
cd "$CRATE_DIR"
cargo build --release --bin generate_test_data 2>/dev/null || true
GEN_BIN="${TARGET_DIR}/release/generate_test_data"
if [[ ! -x "$GEN_BIN" ]]; then
  echo "Error: $GEN_BIN not found. Run from repo root: cargo build --release -p yaksha-cipher --bin generate_test_data"
  exit 1
fi

# Generate data (random plaintext)
echo "Generating ${SIZE_MB} MB ($MODE)..."
if [[ "$MODE" == "evolve" ]]; then
  # Evolve mode: need to stream many small encrypts to get consistent byte stream
  # For simplicity we use neural here if evolve not yet in generate_test_data
  "$GEN_BIN" neural "$SIZE_MB" random > "$DATA_FILE" 2>/dev/null || \
  "$GEN_BIN" "$MODE" "$SIZE_MB" random > "$DATA_FILE"
else
  "$GEN_BIN" "$MODE" "$SIZE_MB" random > "$DATA_FILE"
fi
echo "Written $(wc -c < "$DATA_FILE") bytes."
echo ""

# dieharder
if command -v dieharder &>/dev/null; then
  echo "--- dieharder (subset: -d 0,100,101) ---"
  dieharder -g 201 -f "$DATA_FILE" -d 0   -p 20 2>&1 | tee "$REPORT_DIR/dieharder_d0.txt"  | tail -5
  dieharder -g 201 -f "$DATA_FILE" -d 100 -p 20 2>&1 | tee "$REPORT_DIR/dieharder_sts_monobit.txt" | tail -5
  dieharder -g 201 -f "$DATA_FILE" -d 101 -p 20 2>&1 | tee "$REPORT_DIR/dieharder_sts_runs.txt"   | tail -5
  echo "dieharder subset done. Full run: dieharder -g 201 -f $DATA_FILE -a"
else
  echo "dieharder not found; skip. Install e.g. sudo pacman -S dieharder"
fi
echo ""

# PractRand (pipe)
if command -v RNG_test &>/dev/null; then
  echo "--- PractRand (first 16 MB piped) ---"
  head -c $((16*1024*1024)) < "$DATA_FILE" | RNG_test stdin -tlmin 1KB 2>&1 | tee "$REPORT_DIR/practrand_16mb.txt" | tail -20
else
  echo "RNG_test (PractRand) not found; skip. Build from https://sourceforge.net/projects/pracrand/"
fi
echo ""
echo "Reports under $REPORT_DIR"
