#!/usr/bin/env bash
# Full Valhalla test chain: build → pillars → independence paper panel → industry suite → audit
#
# Usage:
#   ./scripts/run_full_valhalla_test.sh
#   ./scripts/run_full_valhalla_test.sh --skip-build
#   ./scripts/run_full_valhalla_test.sh --cpu-only

set -euo pipefail
REPO="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO"

SKIP_BUILD=0
EXTRA=()
for arg in "$@"; do
  case "$arg" in
    --skip-build) SKIP_BUILD=1 ;;
    *) EXTRA+=("$arg") ;;
  esac
done

PY="${REPO}/.venv-llm/bin/python3"
[[ -x "$PY" ]] || PY=python3

export HF_ENDPOINT="${HF_ENDPOINT:-https://hf-mirror.com}"
export VALHALLA_INGRESS_RESOURCE_PERCENT="${VALHALLA_INGRESS_RESOURCE_PERCENT:-50}"
export VALHALLA_INGRESS_MAX_ITEMS="${VALHALLA_INGRESS_MAX_ITEMS:-48}"
export VALHALLA_LM_CACHE_MAX="${VALHALLA_LM_CACHE_MAX:-1}"
export VALHALLA_BASE_NO_SERVE="${VALHALLA_BASE_NO_SERVE:-1}"

release_between() {
  "$PY" -c "
import sys
sys.path.insert(0, 'tools/valhalla_inference')
from benchmark_env import release_resources
release_resources()
" 2>/dev/null || true
}

echo "════════════════════════════════════════════════════════"
echo "Valhalla full test chain (resource budget: ingress=${VALHALLA_INGRESS_RESOURCE_PERCENT}% cache_max=${VALHALLA_LM_CACHE_MAX})"
echo "════════════════════════════════════════════════════════"

release_between

HL_ARGS=(--full "${EXTRA[@]}")
[[ "$SKIP_BUILD" -eq 1 ]] && HL_ARGS=(--skip-build --full "${EXTRA[@]}")

./scripts/run_highlight_test.sh "${HL_ARGS[@]}"

release_between

echo ""
echo "════════════════════════════════════════════════════════"
echo "Industry benchmarks (fast, fresh)"
echo "════════════════════════════════════════════════════════"
"$PY" tools/valhalla_inference/run_industry_benchmarks.py --fast --fresh --verbose

release_between

echo ""
echo "════════════════════════════════════════════════════════"
echo "Global integration report"
echo "════════════════════════════════════════════════════════"
"$PY" tools/valhalla_inference/integrate_valhalla_reports.py

echo ""
echo "════════════════════════════════════════════════════════"
echo "Benchmark legitimacy audit"
echo "════════════════════════════════════════════════════════"
"$PY" tools/valhalla_inference/audit_benchmark_legitimacy.py

echo ""
echo "→ Reports: ${REPO}/reports/valhalla_inference/"
echo "→ Independence: valhalla_independence_test.json"
echo "→ Industry: INDUSTRY_BENCHMARKS_20260630.md"
