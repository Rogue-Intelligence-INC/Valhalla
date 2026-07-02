#!/usr/bin/env bash
# Full Valhalla inference benchmark suite — industry + training potential + native.
set -uo pipefail
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$REPO"
export RUSTFLAGS="-L /opt/cuda/lib64"
export HF_ENDPOINT="${HF_ENDPOINT:-https://hf-mirror.com}"
LOG="$REPO/reports/valhalla_inference/full_benchmark_run_$(date +%Y%m%d_%H%M%S).log"
PY="${REPO}/.venv-llm/bin/python3"
[[ -x "$PY" ]] || PY=python3
RUST_PY=python3

failures=0
run_step() {
  local label="$1"
  shift
  echo ""
  echo "================================================================"
  echo "[$(date -Iseconds)] $label"
  echo "================================================================"
  if "$@"; then
    echo "OK: $label"
  else
    echo "FAIL: $label (exit $?)" >&2
    failures=$((failures + 1))
  fi
}

{
  echo "=== FULL BENCHMARK SUITE START $(date -Iseconds) ==="
  echo "LOG=$LOG"

  run_step "download-bundles" \
    "$PY" tools/standard_benchmarks/download_standard_benchmarks.py --all --no-wiki-fetch

  run_step "industry-benchmarks --all" \
    "$RUST_PY" tools/valhalla_inference/run_industry_benchmarks.py --all

  run_step "TPI-v2 training-potential" \
    "$RUST_PY" tools/valhalla_inference/test_valhalla_base_training_potential.py --protocol v2

  run_step "NPPI new-paradigm-potential" \
    "$RUST_PY" tools/valhalla_inference/test_new_paradigm_potential.py

  run_step "TPI-v2 traditional_lm RAG" \
    "$PY" tools/valhalla_inference/test_valhalla_base_training_potential.py --only-rag

  run_step "NPPI N5 traditional_lm" \
    "$PY" tools/valhalla_inference/test_new_paradigm_potential.py --only-n5

  run_step "proactive-fate-v2" \
    "$RUST_PY" tools/valhalla_inference/test_proactive_fate_v2.py

  run_step "long-memory-v1 production" \
    "$RUST_PY" tools/valhalla_inference/test_long_memory_v1.py --arm production

  run_step "fate-output-gate-v2" \
    "$RUST_PY" tools/valhalla_inference/test_fate_output_gate.py

  run_step "crosslang-core-unified" \
    "$RUST_PY" tools/valhalla_inference/test_crosslang_core_unified.py

        run_step "context-topic-drift" \
            "$PY" tools/valhalla_inference/test_context_topic_drift.py

        run_step "token-efficiency" \
            "$PY" tools/valhalla_inference/test_token_efficiency.py

  run_step "confidence-report" \
    "$RUST_PY" tools/valhalla_inference/run_confidence_report.py

  echo ""
  echo "=== FULL BENCHMARK SUITE END $(date -Iseconds) failures=$failures ==="
} 2>&1 | tee "$LOG"

exit "$failures"
