#!/usr/bin/env bash
set -euo pipefail

OUT_DIR="${1:?out_dir missing}"
DIEHARDER_TSAMPLES="${2:-500}"
# Independent p-sample count (dieharder -p); defaults to tsamples if arg 3 omitted.
DIEHARDER_PSAMPLES="${3:-$DIEHARDER_TSAMPLES}"
# Optional: stream file basename under OUT_DIR (default: dieharder_stream.bin)
STREAM_BASENAME="${4:-dieharder_stream.bin}"
# Optional: prefix for stdout logs (e.g. gpu_ → gpu_dieharder_sts_monobit_d100.stdout)
OUT_PREFIX="${5:-}"

STREAM_BIN="$OUT_DIR/$STREAM_BASENAME"

if [[ ! -f "$STREAM_BIN" ]]; then
  echo "dieharder stream missing: $STREAM_BIN"
  exit 1
fi

# Require stream size ≥ 95% of nominal MB when DIEHARDER_MB is set (run_all.sh exports it).
if [[ -n "${DIEHARDER_MB:-}" ]]; then
  MIN_BYTES=$(( DIEHARDER_MB * 1024 * 1024 * 95 / 100 ))
  SZ=$(stat -c%s "$STREAM_BIN" 2>/dev/null || wc -c <"$STREAM_BIN")
  if [[ "$SZ" -lt "$MIN_BYTES" ]]; then
    echo "dieharder: stream too small (${SZ} B < ${MIN_BYTES} B expected from DIEHARDER_MB=${DIEHARDER_MB})" >&2
    exit 1
  fi
fi

# --- Strict gate: STS (NIST SP 800-22 family via dieharder) ---
GATE_TESTS=(
  "100:sts_monobit"
  "101:sts_runs"
  "102:sts_serial"
)

EXTENDED_TESTS=(
  "4:bitstream"
  "9:count1s_byte"
  "10:parking_lot"
  "15:runs"
  "16:craps"
  "205:byte_distribution"
)

fail_gate() {
  echo "$1" >&2
  exit 1
}

FAIL=0
for spec in "${GATE_TESTS[@]}"; do
  test_id="${spec%%:*}"
  test_name="${spec#*:}"
  out="$OUT_DIR/${OUT_PREFIX}dieharder_${test_name}_d${test_id}.stdout"
  echo "dieharder [GATE] $test_name (d=$test_id, -t $DIEHARDER_TSAMPLES, -p $DIEHARDER_PSAMPLES) -> $out"
  set +e
  dieharder -d "$test_id" -g 201 -f "$STREAM_BIN" -t "$DIEHARDER_TSAMPLES" -p "$DIEHARDER_PSAMPLES" >"$out" 2>&1
  rc=$?
  set -e
  if [[ $rc -ne 0 ]]; then
    echo "  ERROR: dieharder exit code $rc" >&2
    FAIL=$((FAIL + 1))
    continue
  fi
  if grep -E '\|[[:space:]]+FAILED[[:space:]]*$' "$out"; then
    echo "  ERROR: Assessment FAILED (see $out)" >&2
    FAIL=$((FAIL + 1))
  elif grep -E '\|[[:space:]]+WEAK[[:space:]]*$' "$out"; then
    echo "  ERROR: Assessment WEAK (strict gate)" >&2
    FAIL=$((FAIL + 1))
  else
    if ! grep -qE '\|[[:space:]]+PASSED[[:space:]]*$' "$out"; then
      echo "  ERROR: no PASSED line in gate output (empty/malformed?) — see $out" >&2
      FAIL=$((FAIL + 1))
    fi
  fi
done

if [[ "$FAIL" -ne 0 ]]; then
  fail_gate "dieharder GATE: $FAIL test(s) failed — see $OUT_DIR/dieharder_*.stdout"
fi
echo "dieharder GATE: STS (d=100–102) all PASSED (no FAILED/WEAK); each output contains PASSED."

for spec in "${EXTENDED_TESTS[@]}"; do
  test_id="${spec%%:*}"
  test_name="${spec#*:}"
  out="$OUT_DIR/${OUT_PREFIX}dieharder_info_${test_name}_d${test_id}.stdout"
  echo "dieharder [info only] $test_name (d=$test_id) -> $out"
  dieharder -d "$test_id" -g 201 -f "$STREAM_BIN" -t "$DIEHARDER_TSAMPLES" -p "$DIEHARDER_PSAMPLES" >"$out" 2>&1 || true
done
echo "dieharder: extended tests logged under dieharder_info_*.stdout (not gating)."
