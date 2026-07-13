#!/usr/bin/env bash
# Unique-proof suite U1–U5 — capabilities memory APIs lack.
#
# Usage:
#   ./scripts/run_unique_proofs.sh --fast
#   ./scripts/run_unique_proofs.sh --full
#   ./scripts/run_unique_proofs.sh --fast --only U1,U5
#
set -euo pipefail
REPO="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO"

export RUSTFLAGS="${RUSTFLAGS:--L /opt/cuda/lib64}"
export LIBRARY_PATH="/opt/cuda/lib64:${LIBRARY_PATH:-}"
export HF_ENDPOINT="${HF_ENDPOINT:-https://hf-mirror.com}"
export VALHALLA_BASE_NO_SERVE=1
export PYTHONUNBUFFERED=1

PY="${REPO}/.venv-llm/bin/python3"
[[ -x "$PY" ]] || PY=python3

ARGS=()
for arg in "$@"; do
  ARGS+=("$arg")
done
# default --fast if neither flag given
has_scale=0
for a in "${ARGS[@]+"${ARGS[@]}"}"; do
  [[ "$a" == "--fast" || "$a" == "--full" ]] && has_scale=1
done
[[ "$has_scale" -eq 0 ]] && ARGS=(--fast "${ARGS[@]+"${ARGS[@]}"}")

echo "[unique-proofs] ${ARGS[*]}"
exec "$PY" tools/valhalla_inference/run_unique_proof_suite.py "${ARGS[@]}"
