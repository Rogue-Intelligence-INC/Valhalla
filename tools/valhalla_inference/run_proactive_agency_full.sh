#!/usr/bin/env bash
# Proactive Agency — 补齐版完整 benchmark（10 multimodal + 5 LoCoMo + full PPI-v2）
#
# Usage:
#   tools/valhalla_inference/run_proactive_agency_full.sh
#   tools/valhalla_inference/run_proactive_agency_full.sh --verbose

set -euo pipefail
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$REPO"

PY="${REPO}/.venv-llm/bin/python3"
[[ -x "$PY" ]] || PY=python3

export RUSTFLAGS="${RUSTFLAGS:--L /opt/cuda/lib64}"
export VALHALLA_USE_CPU="${VALHALLA_USE_CPU:-0}"

MODEL="${MODEL:-${REPO}/models/hf/Qwen2.5-3B-Instruct-Valhalla-TriModal-Fusion-v3-test-s0.10}"

exec "$PY" tools/valhalla_inference/run_proactive_agency_benchmark.py \
  --model "$MODEL" \
  --full \
  --verbose \
  "$@"
