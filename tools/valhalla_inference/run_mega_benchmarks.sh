#!/usr/bin/env bash
# Mega benchmark — industry + AGI/training proxies (full, not --fast)
set -uo pipefail
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$REPO"
export LIBRARY_PATH="/opt/cuda/targets/x86_64-linux/lib:${LIBRARY_PATH:-}"
export RUSTFLAGS="-L /opt/cuda/lib64"
export HF_ENDPOINT="${HF_ENDPOINT:-https://hf-mirror.com}"
LOG="$REPO/reports/valhalla_inference/mega_benchmark_$(date +%Y%m%d_%H%M%S).log"
PY="${REPO}/.venv-llm/bin/python3"
[[ -x "$PY" ]] || PY=python3
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
  echo "=== MEGA BENCHMARK START $(date -Iseconds) ==="
  echo "LOG=$LOG"

  run_step "build-valhalla_base" \
    cargo build -p hub-f64 --release --bin valhalla_base

  run_step "download-bundles" \
    "$PY" tools/standard_benchmarks/download_standard_benchmarks.py --all --no-wiki-fetch

  # --- Industry / RAG / long-context (ACL, COLM, Salesforce, THUDM, …) ---
  run_step "industry-benchmarks-full" \
    python3 tools/valhalla_inference/run_industry_benchmarks.py --all

  # --- Recognized + scaling + backbone universality ---
  run_step "recognized-benchmarks" \
    python3 tools/valhalla_inference/run_recognized_benchmarks.py --with-backbone

  # --- Training / substrate potential (TPI, NPPI, Fate) ---
  run_step "TPI-v2" \
    python3 tools/valhalla_inference/test_valhalla_base_training_potential.py --protocol v2

  run_step "NPPI" \
    python3 tools/valhalla_inference/test_new_paradigm_potential.py

  run_step "proactive-fate-v2" \
    python3 tools/valhalla_inference/test_proactive_fate_v2.py

  run_step "long-memory-production" \
    python3 tools/valhalla_inference/test_long_memory_v1.py --arm production

  run_step "fate-output-gate-v2" \
    python3 tools/valhalla_inference/test_fate_output_gate.py

  run_step "crosslang-core" \
    python3 tools/valhalla_inference/test_crosslang_core_unified.py

  run_step "confidence-report" \
    python3 tools/valhalla_inference/run_confidence_report.py

  # --- AGI proxy: MMLU/GSM8K-style fair suite + multi-domain incubation ---
  run_step "fair-benchmark-test" \
    "$PY" tools/fair_benchmark/run_fair_benchmark.py --phase test

  run_step "incubation-benchmark-standard" \
    "$PY" tools/valhalla_model_bridge/run_incubation_benchmark.py --phase standard

  run_step "external-domain-GSM8K-MMLU" \
    "$PY" tools/valhalla_model_bridge/run_external_domain_experiment.py --limit 0 2>/dev/null || \
    echo "SKIP: external_domain (optional)"

  # --- Aggregate dashboards ---
  run_step "fundraise-aggregate" \
    python3 tools/valhalla_inference/aggregate_fundraise_benchmarks.py

  run_step "industry-aggregate" \
    python3 tools/valhalla_inference/run_industry_benchmarks.py --aggregate-only

  echo ""
  echo "=== MEGA BENCHMARK END $(date -Iseconds) failures=$failures ==="
} 2>&1 | tee "$LOG"

exit "$failures"
