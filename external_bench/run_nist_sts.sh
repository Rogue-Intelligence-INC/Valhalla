#!/usr/bin/env bash
# Run NIST SP 800-22 Statistical Test Suite on Yaksha ciphertext stream.
# Usage: ./run_nist_sts.sh [mode] [size_mb] [out_dir]
#   mode: core | neural | parametric | evolve (default: core)
#   size_mb: per-stream size in MB (default: 1 — min practical for STS)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
YAKSHA="$ROOT/valhalla-crypto/yaksha-cipher"
ASSESS="$ROOT/external_bench/tools/bin/nist-sts-assess"
[[ -x "$ASSESS" ]] || ASSESS="$ROOT/external_bench/tools/nist-sts/sts-2.1.2/sts-2.1.2/assess"

MODE="${1:-core}"
SIZE_MB="${2:-1}"
TS="$(date +%Y%m%d_%H%M%S)"
OUT="${3:-$ROOT/external_bench/results/nist_sts_${MODE}_${SIZE_MB}mb_$TS}"

if [[ ! -x "$ASSESS" ]]; then
  echo "NIST STS not built. Run: ./external_bench/install_benchmark_tools.sh" >&2
  exit 1
fi

mkdir -p "$OUT/data"
BITS=$((SIZE_MB * 1024 * 1024 * 8))

echo "NIST STS: mode=$MODE size=${SIZE_MB}MB bits_per_stream=$BITS"
echo "Output: $OUT"

cd "$ROOT"
cargo build --release -p yaksha-cipher --bin generate_test_data -q
"$ROOT/target/release/generate_test_data" "$MODE" "$SIZE_MB" random >"$OUT/data/data.e"

cd "$OUT"
# Non-interactive: 0=Input File, path, 1=all tests, 0=continue between tests
printf '0\ndata/data.e\n1\n0\n' | "$ASSESS" "$BITS" >"$OUT/nist_sts.stdout" 2>&1 || true

echo "Done. Summary tail:"
tail -40 "$OUT/nist_sts.stdout"
echo ""
echo "Full log: $OUT/nist_sts.stdout"
echo "Reports:  $OUT/experiments/AlgorithmTesting/finalAnalysisReport.txt (if generated)"
