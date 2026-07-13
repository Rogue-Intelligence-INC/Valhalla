#!/usr/bin/env bash
# Fundraise ablation matrix — CUDA · monitor · pause · error auto-stop · resume
#
# Usage:
#   ./tools/valhalla_inference/run_fundraise_ablation.sh
#   ./tools/valhalla_inference/run_fundraise_ablation.sh --fast
#   ./tools/valhalla_inference/run_fundraise_ablation.sh --industry-only
#   ./tools/valhalla_inference/run_fundraise_ablation.sh --resume
#   touch reports/valhalla_inference/ablation.pause   # graceful pause
#   rm   reports/valhalla_inference/ablation.pause && ./… --resume
#
set -uo pipefail
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$REPO"

export RUSTFLAGS="${RUSTFLAGS:--L /opt/cuda/lib64}"
export LIBRARY_PATH="/opt/cuda/lib64:${LIBRARY_PATH:-}"
export HF_ENDPOINT="${HF_ENDPOINT:-https://hf-mirror.com}"
export VALHALLA_BASE_NO_SERVE=1
export VALHALLA_CUDA_DEVICE="${VALHALLA_CUDA_DEVICE:-0}"
export VALHALLA_USE_CPU="${VALHALLA_USE_CPU:-0}"
export VALHALLA_LM_CACHE_MAX="${VALHALLA_LM_CACHE_MAX:-1}"
export VALHALLA_INGRESS_RESOURCE_PERCENT="${VALHALLA_INGRESS_RESOURCE_PERCENT:-50}"
export PYTHONUNBUFFERED=1

PY="${REPO}/.venv-llm/bin/python3"
[[ -x "$PY" ]] || PY=python3

REPORTS="$REPO/reports/valhalla_inference"
LOG="$REPORTS/fundraise_ablation_run.log"
PIDFILE="$REPORTS/fundraise_ablation.pid"
PAUSE_FILE="$REPORTS/ablation.pause"
CKPT_DIR="$REPORTS/ablation_ckpt"
STATUS="$REPORTS/fundraise_ablation.status"
mkdir -p "$REPORTS" "$CKPT_DIR"

export VALHALLA_PAUSE_FILE="${VALHALLA_PAUSE_FILE:-$PAUSE_FILE}"

MODE=(--moderate --with-paradigm --verbose --stop-on-error)
RESUME=0
for arg in "$@"; do
  case "$arg" in
    --fast) MODE=(--fast --verbose --stop-on-error) ;;
    --industry-only) MODE=(--moderate --verbose --stop-on-error) ;;
    --full) MODE=(--with-paradigm --verbose --stop-on-error) ;;
    --resume) RESUME=1 ;;
    --continue-on-error)
      # strip default stop-on-error if present
      NEW=()
      for m in "${MODE[@]}"; do
        [[ "$m" == "--stop-on-error" ]] && continue
        NEW+=("$m")
      done
      MODE=("${NEW[@]}" --continue-on-error)
      ;;
  esac
done

EXTRA=(--pause-file "$VALHALLA_PAUSE_FILE" --checkpoint-dir "$CKPT_DIR")
if [[ "$RESUME" -eq 1 ]]; then
  EXTRA+=(--resume)
fi

if [[ ! -x "$REPO/target/release/valhalla_base" ]]; then
  echo "Building valhalla_base …"
  cargo build -p hub-f64 --release --bin valhalla_base || exit 1
fi

echo $$ >"$PIDFILE"
{
  echo "=== fundraise ablation $(date -Iseconds) ${MODE[*]} resume=$RESUME ==="
  echo "pause_file=$VALHALLA_PAUSE_FILE  status=$STATUS  pid=$$"
  echo "cuda_device=$VALHALLA_CUDA_DEVICE  LIBRARY_PATH=$LIBRARY_PATH"
  "$PY" tools/valhalla_inference/run_fundraise_ablation_matrix.py "${MODE[@]}" "${EXTRA[@]}"
} 2>&1 | tee -a "$LOG"
rc=${PIPESTATUS[0]}
echo "=== end $(date -Iseconds) rc=$rc ===" | tee -a "$LOG"
exit "$rc"
