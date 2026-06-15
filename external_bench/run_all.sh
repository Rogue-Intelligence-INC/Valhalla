#!/usr/bin/env bash
# CPU-only external gate: yaksha-cipher + OpenSSL + dieharder (CPU-generated stream).
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
YAKSHA_DIR="$ROOT_DIR/valhalla-crypto/yaksha-cipher"
THIS_DIR="$ROOT_DIR/external_bench"

# shellcheck source=bench_common.sh
source "$THIS_DIR/bench_common.sh"
bench_load_env

ts="$(date +%Y%m%d_%H%M%S)"
OUT_DIR="$THIS_DIR/results/cpu_$ts"
mkdir -p "$OUT_DIR"
echo "cpu" >"$OUT_DIR/bench_variant.txt"

echo "External benchmark (CPU): $OUT_DIR"
echo "  OPENSSL_SECONDS=$OPENSSL_SECONDS DIEHARDER_MB=$DIEHARDER_MB DIEHARDER_TSAMPLES=$DIEHARDER_TSAMPLES DIEHARDER_PSAMPLES=$DIEHARDER_PSAMPLES"

set +e

cd "$YAKSHA_DIR"

cargo run --release --bin yaksha_aead_bench >"$OUT_DIR/yaksha_aead_bench.stdout" 2>&1
echo $? >"$OUT_DIR/yaksha_aead_bench.exit"

cargo run --release --bin yaksha_test >"$OUT_DIR/yaksha_test.stdout" 2>&1
echo $? >"$OUT_DIR/yaksha_test.exit"

cargo run --release --bin yaksha_aead_timing_keys -- 64 0.25 64 >"$OUT_DIR/yaksha_aead_timing_keys.csv" 2>&1
echo $? >"$OUT_DIR/yaksha_aead_timing_keys.exit"

rm -f "$OUT_DIR/dieharder_stream.bin"
cargo run --release --bin generate_test_data -- "$DIEHARDER_MODE" "$DIEHARDER_MB" random \
  >"$OUT_DIR/dieharder_stream.bin" 2>"$OUT_DIR/dieharder_stream.generate.stderr"
echo $? >"$OUT_DIR/dieharder_stream.generate.exit"

set -e

require_zero_bench_exits "$OUT_DIR" yaksha_aead_bench yaksha_test yaksha_aead_timing_keys dieharder_stream.generate

cd "$ROOT_DIR"

export DIEHARDER_MB
"$THIS_DIR/run_openssl_aead.sh" "$OUT_DIR" "$OPENSSL_SECONDS"
"$THIS_DIR/run_dieharder.sh" "$OUT_DIR" "$DIEHARDER_TSAMPLES" "$DIEHARDER_PSAMPLES"

python3 "$THIS_DIR/collect_report.py" "$OUT_DIR"

echo "Done (CPU)."
