#!/usr/bin/env bash
# MCQ scaling ladder — GPU F16, subprocess per model (memory-safe, no serve race).
set -euo pipefail
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$REPO"
LOG="${1:-reports/valhalla_inference/mcq_ladder_mem_gpu.log}"
PY="${REPO}/.venv-llm/bin/python3"
[[ -x "$PY" ]] || PY=python3

export HF_ENDPOINT="${HF_ENDPOINT:-https://hf-mirror.com}"
export LIBRARY_PATH="/opt/cuda/targets/x86_64-linux/lib:${LIBRARY_PATH:-}"
export RUSTFLAGS="-L /opt/cuda/lib64"
export VALHALLA_USE_CPU="${VALHALLA_USE_CPU:-0}"
export VALHALLA_LM_F16="${VALHALLA_LM_F16:-1}"

fuser -k 8787/tcp 8788/tcp 2>/dev/null || true
sleep 1

echo "=== MCQ LADDER (memory-safe + GPU F16) $(date -Iseconds) ===" | tee "$LOG"
echo "VALHALLA_USE_CPU=$VALHALLA_USE_CPU VALHALLA_LM_F16=$VALHALLA_LM_F16" | tee -a "$LOG"

"$PY" -u tools/valhalla_inference/test_mcq_ladder_v1.py \
  --full-ladder \
  --memory-safe \
  --output mcq_ladder_v1_test.json \
  2>&1 | tee -a "$LOG"
rc=${PIPESTATUS[0]}

cp -f reports/valhalla_inference/mcq_ladder_v1_test.json \
  reports/valhalla_inference/mcq_ladder_v1_full_test.json 2>/dev/null || true

echo "=== END $(date -Iseconds) exit=$rc ===" | tee -a "$LOG"
exit "$rc"
