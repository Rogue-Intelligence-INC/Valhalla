#!/usr/bin/env bash
# Valhalla one-click reproduction (Linux/macOS, best effort).
# Validated platform is Windows (reproduce.ps1); this script calls the
# suite binaries directly. The native_cuda build needs an NVIDIA CUDA
# toolchain; without it the script stops honestly instead of faking a run.
# Usage: bash reproduce/reproduce.sh [--quick] [--skip-flow]
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
STAMP="$(date +%Y%m%d_%H%M%S)"
LOGDIR="$ROOT/reproduce/logs/$STAMP"
mkdir -p "$LOGDIR"
QUICK=0; SKIPFLOW=0
for a in "$@"; do case "$a" in --quick) QUICK=1;; --skip-flow) SKIPFLOW=1;; *) echo "unknown arg: $a"; exit 2;; esac; done

# clean stale experiment env
for v in $(env | grep -oE '^(VH_|VALHALLA_)[A-Z_0-9]*' || true); do unset "$v"; done

command -v cargo >/dev/null || { echo "MISSING prerequisite: cargo (install Rust stable)"; exit 2; }
rustc --version
if ! command -v nvcc >/dev/null && [ -z "${CUDA_PATH:-}" ]; then
  echo "WARNING: no CUDA toolchain found (nvcc/CUDA_PATH). The native_cuda build will likely fail."
fi
if [ -n "${VALHALLA_CUDA_LIB:-}" ]; then export RUSTFLAGS="-L $VALHALLA_CUDA_LIB"; fi

FAILED=""
step() { # name logfile cmd...
  local name="$1" log="$2"; shift 2
  echo "== $name"
  "$@" > "$LOGDIR/$log" 2>&1
  local code=$?
  if [ $code -ne 0 ]; then echo "FAIL $name (exit $code) - log: $LOGDIR/$log"; FAILED="$FAILED $name";
  else echo "PASS $name - log: $LOGDIR/$log"; fi
}

step "build suite binaries" build.log cargo build --release -p hub-f64 \
  --bin valhalla_incubation_proof --bin valhalla_industry_battery --bin valhalla_full_flow \
  --bin valhalla_fate_storage --bin valhalla_storage_test --bin valhalla_data_flow \
  --bin valhalla_monitor_check --bin valhalla_light_train_check --bin valhalla_fate_decision_check
[ -n "$FAILED" ] && { echo "BUILD FAILED - stop."; exit 1; }

BIN="$ROOT/target/release"
step "L0 library tests (fate 7 / stem 4 / tile 8 / blackhole 3)" lib_tests.log \
  cargo test --release --lib -p fate-f64 -p stem-cell -p valhalla-tile-quattro -p valhalla_metasystems_blackhole

echo "== L1 check suite (6 packages, invoked directly)"
: > "$LOGDIR/check.log"
for b in valhalla_fate_storage valhalla_storage_test valhalla_data_flow valhalla_monitor_check valhalla_light_train_check valhalla_fate_decision_check; do
  "$BIN/$b" >> "$LOGDIR/check.log" 2>&1 || FAILED="$FAILED check:$b"
done
grep -q . "$LOGDIR/check.log" && echo "check outputs in $LOGDIR/check.log"

export VALHALLA_IND_SEEDS='1,2,3,4,5'
step "L2 industry battery (5 seeds)" battery.log "$BIN/valhalla_industry_battery"
unset VALHALLA_IND_SEEDS

if [ $QUICK -eq 0 ]; then
  export VALHALLA_PROOF_SEEDS='1,2,3,4,5,6,7,8,9,10,11,12,13,14,15'
  step "L2 incubation causal proof (15 seeds)" incubation.log "$BIN/valhalla_incubation_proof"
  unset VALHALLA_PROOF_SEEDS
  if [ $SKIPFLOW -eq 0 ]; then
    VH_MATS=ext VH_CHUNK_KB=32 VH_ROUNDS=50 VH_CYCLES=2 VH_SEED=7 VH_HOLDOUT_KB=32 \
      step "L3 full flow (ext, 32KB x 50 rounds)" flow.log "$BIN/valhalla_full_flow"
  fi
fi

echo "== verification (expected readings @ e1ff0b3)"
MISMATCH=0
while IFS='|' read -r key want; do
  key="$(echo "$key" | xargs)"; want="$(echo "$want" | xargs)"
  [ -z "$key" ] && continue; case "$key" in \#*) continue;; esac
  scope="${key%%-*}"
  [ $QUICK -eq 1 ] && { [ "$scope" = incubation ] || [ "$scope" = flow ]; } && continue
  [ $SKIPFLOW -eq 1 ] && [ "$scope" = flow ] && continue
  if grep -rF -- "$want" "$LOGDIR" >/dev/null 2>&1; then echo "  REPRODUCED  $key = $want";
  else echo "  MISMATCH    $key : expected substring '$want' not found"; MISMATCH=$((MISMATCH+1)); fi
done < "$ROOT/reproduce/expected_results.txt"

echo "logs: $LOGDIR"
[ -n "$FAILED" ] && { echo "FAILED steps:$FAILED"; exit 1; }
[ $MISMATCH -gt 0 ] && { echo "DONE WITH $MISMATCH READING MISMATCH(ES) - compare raw logs first (reproduce/README.md)"; exit 1; }
echo "ALL STEPS PASS - headline readings reproduced."
