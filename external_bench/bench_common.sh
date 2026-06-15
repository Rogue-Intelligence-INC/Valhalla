#!/usr/bin/env bash
# Shared defaults and helpers for external_bench/*.sh
# shellcheck source=bench_common.sh

bench_load_env() {
  : "${OPENSSL_SECONDS:=10}"
  : "${DIEHARDER_MB:=64}"
  : "${DIEHARDER_TSAMPLES:=500}"
  : "${DIEHARDER_PSAMPLES:=500}"
  : "${DIEHARDER_MODE:=neural}"
}

# Usage: require_zero_bench_exits <out_dir> <base1> <base2> ...
require_zero_bench_exits() {
  local d="$1"
  shift
  for base in "$@"; do
    local f="$d/${base}.exit"
    [[ -f "$f" ]] || { echo "FATAL: missing $f" >&2; exit 1; }
    read -r code <"$f" || { echo "FATAL: cannot read $f" >&2; exit 1; }
    if [[ "$code" != "0" ]]; then
      echo "FATAL: ${base} exited with code ${code} (see $d/${base}.stdout or .csv / .stderr)" >&2
      exit 1
    fi
  done
}
